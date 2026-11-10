//! Core 1: starting it, the FIFOs between the two cores, and telling the
//! cores apart.
//!
//! # Using it
//!
//! ```ignore
//! use pico2::multicore::{Core1Stack, Fifo, Rp2350Core1};
//!
//! // 16 kB for core 1's stack: 2048 eight-byte words.
//! static CORE1_STACK: Core1Stack<2048> = Core1Stack::new();
//!
//! fn main() -> ! {
//!     let board = Rp2350::take().unwrap();
//!     let clocks = Rp2350Clocks::new(board.clocks);
//!     let timer = Rp2350Timer::new(board.timer, &clocks);
//!     let core1 = Rp2350Core1::new(board.core1);
//!     let fifo = core1.launch(&CORE1_STACK, core1_main, &timer, 10_000).unwrap();
//!     loop { /* core 0's work */ }
//! }
//!
//! fn core1_main(fifo: Fifo) -> ! {
//!     loop { /* core 1's work */ }
//! }
//! ```
//!
//! # What core 1 is doing before this
//!
//! Both cores leave reset together and enter the bootrom; core 1 "redirects
//! very early in the boot sequence to a low-power state where it waits to be
//! launched, after boot, by user software on core 0", and "if core 1 is
//! unused, it remains in this low-power state" (§5.2, p365). Every reset
//! that restarts core 1 puts it back there: power-on, the RUN pin, and a
//! watchdog reset as [`crate::watchdog`] sets it up, whose scope
//! (`PSM.WDSEL`) includes `PROC1`. [`Rp2350Core1::launch`] wakes it with the
//! bootrom's handshake (§5.3, p376), which [`api::multicore`] implements and
//! tests on the host: six words over the FIFO, each echoed back, the last
//! three being core 1's vector table, initial stack pointer and entry point.
//!
//! The handshake is for a core 1 that has been reset since it last ran
//! anything. A debugger that restarts core 0 alone leaves core 1 running
//! whatever it ran; such a core 1 does not answer, and `launch` reports
//! [`LaunchError::NoAnswer`] when its time is up. Reset the chip.
//!
//! # What core 1 runs first
//!
//! Its entry point is not the application's function but `core1_start`,
//! below. Before anything uses the stack, it sets core 1's stack limit,
//! `MSPLIM`, to the bottom of the stack it was given. "The stack limit
//! registers limit the extent to which the MSP and PSP registers can
//! descend" (§3.7, p135): a push below it faults on core 1 (a UsageFault
//! with `STKOF` set, escalated to HardFault) instead of writing over
//! whatever lies below the stack. Then it does on core 1 what
//! [`OnReset`](crate::OnReset) does on core 0 before any other code: it
//! enables core 1's own floating-point unit (`CPACR` is per core, like every
//! register in the Private Peripheral Bus). Then it calls the application's
//! function with core 1's [`Fifo`].
//!
//! `.data` and `.bss` are not touched again: core 0 set them up before
//! `main`, and both cores share them.
//!
//! # Both cores share the vector table
//!
//! Core 1's `VTOR` is set to the same table as core 0's, so a fault on either
//! core runs the same handlers, [`OnHardFault`](crate::OnHardFault) and
//! [`DefaultHandler`](crate::DefaultHandler), and with them the
//! application's safe state (see [`entry!`](crate::entry)), on the core that
//! faulted. [`core()`] tells a handler which one that is. The interrupt
//! controllers are per core, and core 1 starts with no interrupt enabled.
//!
//! # The FIFOs
//!
//! "The SIO contains two FIFOs for passing data, messages or ordered events
//! between the two cores. Each FIFO is 32 bits wide and four entries deep.
//! One of the FIFOs can only be written by core 0 and read by core 1. The
//! other can only be written by core 1 and read by core 0" (§3.1.5, p43; the
//! `FIFO_ST` register description says "8 words deep", p67). Each core
//! reaches its own ends at the same addresses: `FIFO_WR` writes its outgoing
//! FIFO, `FIFO_RD` reads its incoming one, and `FIFO_ST` reports whether
//! there is data (`VLD`), room (`RDY`), and two sticky flags for a read when
//! empty (`ROE`) or a write when full (`WOF`). Either access "does not affect
//! the FIFO state", but "this does represent some loss of data or reception
//! of invalid data by the software accessing the FIFO" (§3.1.5, p43).
//!
//! [`Fifo`] only ever reads when `VLD` is set and writes when `RDY` is set,
//! and never waits: it implements [`api::multicore::Mailbox`]. For anything
//! more than a few words between the cores, use memory both cores share,
//! with atomics: "the processors on RP2350 support standard atomic/exclusive
//! access instructions which, in concert with the global exclusive monitor
//! (Section 2.1.6), allow both cores to safely share variables in SRAM"
//! (§3.1, p42). This module never uses the SIO spinlocks, which erratum
//! RP2350-E2 makes unreliable (p1364); the FIFO registers, at `0x050`–`0x058`,
//! are below the `0x180` boundary that erratum concerns.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::Range;
use core::sync::atomic::{AtomicUsize, Ordering};

use api::device::DeviceHandle;
use api::multicore::{Launch, Mailbox, Step};

use crate::common::reg::RegAddr;
use crate::timer::Rp2350Timer;

// --- Registers -----------------------------------------------------------

/// SIO `CPUID`, `0x000` (Table 16, p55; Table 17, p62): reads 0 on core 0
/// and 1 on core 1.
const SIO_CPUID: *const u32 = RegAddr::SIO as usize as *const u32;

/// The SIO's FIFO registers, `0x050`–`0x058` (Table 16, p56). The same
/// addresses on both cores, each reaching its own ends of the FIFOs.
#[repr(C)]
struct SioFifo {
    /// `FIFO_ST` (Table 36, p68): `VLD`, `RDY`, `WOF`, `ROE`. `WOF` and
    /// `ROE` are write-clear: "Writing a 1 to a bit in a WC field will write
    /// that bit to 0" (p1346).
    fifo_st: u32,
    /// `FIFO_WR` (Table 37, p68): "Write access to this core's TX FIFO".
    fifo_wr: u32,
    /// `FIFO_RD` (Table 38, p68): "Read access to this core's RX FIFO".
    fifo_rd: u32,
}

const _: () = assert!(core::mem::offset_of!(SioFifo, fifo_rd) == 0x08);

const SIO_FIFO: *mut SioFifo = (RegAddr::SIO as usize + 0x050) as *mut SioFifo;

/// `FIFO_ST.VLD`, bit 0: "1 if this core's RX FIFO is not empty".
const FIFO_ST_VLD: u32 = 1 << 0;
/// `FIFO_ST.RDY`, bit 1: "1 if this core's TX FIFO is not full".
const FIFO_ST_RDY: u32 = 1 << 1;
/// `FIFO_ST.WOF`, bit 2: "the TX FIFO was written when full".
const FIFO_ST_WOF: u32 = 1 << 2;
/// `FIFO_ST.ROE`, bit 3: "the RX FIFO was read when empty".
const FIFO_ST_ROE: u32 = 1 << 3;

/// Wake the other core from a wait for an event (Arm `SEV`). Core 1 waits
/// for one in the bootrom while its FIFO is empty or full.
fn sev() {
    unsafe { core::arch::asm!("sev", options(nomem, nostack, preserves_flags)) };
}

// --- Which core ----------------------------------------------------------

/// One of the RP2350's two processor cores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Core {
    /// Core 0, which runs `main`.
    Core0,
    /// Core 1, which runs only what [`Rp2350Core1::launch`] started.
    Core1,
}

/// The core this runs on: one load of SIO `CPUID`, which reads 0 on core 0
/// and 1 on core 1 at the same address. Safe anywhere, a fault handler
/// included.
pub fn core() -> Core {
    match unsafe { SIO_CPUID.read_volatile() } {
        0 => Core::Core0,
        _ => Core::Core1,
    }
}

// --- The FIFOs -----------------------------------------------------------

/// This core's ends of the two FIFOs between the cores: the one to the
/// other core and the one from it. See the module documentation.
///
/// Each core has its own: [`Rp2350Core1::launch`] returns core 0's, and core
/// 1's function receives core 1's. The registers are the same addresses on
/// both cores, each reaching its own ends, so a `Fifo` must stay on the core
/// it was given to: it is not `Send`.
pub struct Fifo {
    _not_send: PhantomData<*mut ()>,
}

/// The sticky error flags [`Fifo::take_errors`] reads and clears.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FifoErrors {
    /// `WOF`: this core wrote to its outgoing FIFO while it was full. "This
    /// write was ignored by the FIFO" (Table 36, p68): the word was lost.
    pub written_when_full: bool,
    /// `ROE`: this core read its incoming FIFO while it was empty. "This
    /// read was ignored by the FIFO" (Table 36, p68): what it returned was
    /// no word the other core sent.
    pub read_when_empty: bool,
}

impl Fifo {
    const fn new() -> Self {
        Self {
            _not_send: PhantomData,
        }
    }

    /// Read the sticky flags `WOF` and `ROE`, then clear them. [`Fifo`]'s
    /// own methods never set them; anything else that touches this core's
    /// FIFO registers can.
    ///
    /// Clears only the flags it read as set, by writing 1 to those bits, so
    /// a flag raised between the read and the write is kept for the next
    /// call rather than lost.
    pub fn take_errors(&mut self) -> FifoErrors {
        unsafe {
            let st = (&raw const (*SIO_FIFO).fifo_st).read_volatile();
            let seen = st & (FIFO_ST_WOF | FIFO_ST_ROE);
            (&raw mut (*SIO_FIFO).fifo_st).write_volatile(seen);
            FifoErrors {
                written_when_full: st & FIFO_ST_WOF != 0,
                read_when_empty: st & FIFO_ST_ROE != 0,
            }
        }
    }
}

impl Mailbox for Fifo {
    fn try_send(&mut self, word: u32) -> bool {
        unsafe {
            if (&raw const (*SIO_FIFO).fifo_st).read_volatile() & FIFO_ST_RDY == 0 {
                return false;
            }
            (&raw mut (*SIO_FIFO).fifo_wr).write_volatile(word);
        }
        sev();
        true
    }

    fn try_receive(&mut self) -> Option<u32> {
        unsafe {
            if (&raw const (*SIO_FIFO).fifo_st).read_volatile() & FIFO_ST_VLD == 0 {
                return None;
            }
            Some((&raw const (*SIO_FIFO).fifo_rd).read_volatile())
        }
    }

    fn wake(&mut self) {
        sev();
    }
}

// --- Core 1's stack ------------------------------------------------------

/// Memory for core 1's stack: `WORDS` eight-byte words.
///
/// Aligned to 32 bytes, and [`Rp2350Core1::launch`] takes only a whole
/// number of 32-byte blocks (`WORDS` a multiple of 4), so that one
/// Cortex-M33 MPU region can cover exactly this stack: a region's base is
/// "bits \[31:5\] of the lower inclusive limit" and its limit is
/// "postfixed with 0x1F" (Tables 234 and 235, p196). That is more than the
/// eight bytes the procedure call standard requires of a stack pointer and
/// `MSPLIM` requires of a limit. [`range`](Self::range) gives the
/// addresses.
///
/// Declare one as a `static` and hand it to [`Rp2350Core1::launch`]:
///
/// ```ignore
/// static CORE1_STACK: Core1Stack<2048> = Core1Stack::new(); // 16 kB
/// ```
///
/// It goes in `.bss` like any zero-initialised static, so `link.ld`'s check
/// that `.bss` leaves room for core 0's stack counts it. Core 1's stack
/// limit is set to its first byte, so core 1 faults rather than grow out of
/// it.
#[repr(C, align(32))]
pub struct Core1Stack<const WORDS: usize> {
    words: UnsafeCell<[u64; WORDS]>,
}

// SAFETY: nothing but core 1's stack pointer ever reaches the memory: the
// field is private, and `launch`, which takes the only route to it, runs at
// most once per boot (it consumes the board's single `core1` handle).
unsafe impl<const WORDS: usize> Sync for Core1Stack<WORDS> {}

impl<const WORDS: usize> Core1Stack<WORDS> {
    /// The smallest stack [`Rp2350Core1::launch`] takes: 1 kB.
    pub const MIN_WORDS: usize = 128;

    /// A stack of `WORDS` words, zeroed.
    pub const fn new() -> Self {
        Self {
            words: UnsafeCell::new([0; WORDS]),
        }
    }

    /// The stack's addresses, from its lowest byte, where core 1's stack
    /// limit is set, to one past its highest, where core 1's stack pointer
    /// starts. For an MPU region; reading them touches nothing.
    pub fn range(&self) -> Range<usize> {
        let bottom = self.words.get() as usize;
        bottom..bottom + WORDS * 8
    }
}

impl<const WORDS: usize> Default for Core1Stack<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

// --- Launching -----------------------------------------------------------

/// Why [`Rp2350Core1::launch`] did not start core 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchError {
    /// Core 1 had not echoed the whole handshake when the time given ran
    /// out: it is not waiting in the bootrom (see the module documentation)
    /// or not answering. It was not sent its entry point, or was sent it and
    /// had not echoed it; in the second case it can still start.
    NoAnswer,
}

/// Core 1, not yet started: the right to start it, once.
///
/// Built from the board's `core1` handle, which exists once per boot, so
/// core 1 is started at most once.
pub struct Rp2350Core1 {
    _private: (),
}

/// The application's core 1 function, for `core1_run`: written by
/// `launch` before the handshake, read by core 1 after it.
static CORE1_ENTRY: AtomicUsize = AtomicUsize::new(0);
/// The lowest address of core 1's stack, for `core1_start`'s `MSPLIM`.
static CORE1_STACK_LIMIT: AtomicUsize = AtomicUsize::new(0);

impl Rp2350Core1 {
    /// Take the right to start core 1. Touches no hardware: core 1 is
    /// waiting in the bootrom until [`launch`](Self::launch).
    pub fn new(_handle: DeviceHandle<Rp2350Core1>) -> Self {
        Self { _private: () }
    }

    /// Start core 1 running `entry` on `stack`, waiting at most `timeout_us`
    /// microseconds on `timer` for the handshake. Returns core 0's [`Fifo`].
    ///
    /// Never waits longer than that: each [`Launch::step`] returns at once,
    /// and the time is checked between steps. On a core 1 waiting in the
    /// bootrom the handshake takes about `2 * QUIET_US`
    /// ([`api::multicore::QUIET_US`]): 200 µs. Give a margin, and keep the
    /// time short beside any watchdog already running.
    ///
    /// Core 1 runs `entry` with its own floating-point unit on, its stack
    /// limit at the bottom of `stack`, the same vector table as core 0 and
    /// core 1's [`Fifo`]. See the module documentation.
    pub fn launch<const WORDS: usize>(
        self,
        stack: &'static Core1Stack<WORDS>,
        entry: fn(Fifo) -> !,
        timer: &Rp2350Timer,
        timeout_us: u64,
    ) -> Result<Fifo, LaunchError> {
        const {
            assert!(
                WORDS >= Core1Stack::<WORDS>::MIN_WORDS,
                "core 1's stack is too small"
            );
            assert!(
                WORDS.is_multiple_of(4),
                "core 1's stack must be whole 32-byte blocks"
            );
        };
        let Range {
            start: bottom,
            end: top,
        } = stack.range();
        CORE1_ENTRY.store(entry as usize, Ordering::Release);
        CORE1_STACK_LIMIT.store(bottom, Ordering::Release);
        // The two stores reach memory before any word of the handshake
        // leaves for core 1.
        unsafe { core::arch::asm!("dsb", options(nostack, preserves_flags)) };

        // `core1_start`'s address already has the Thumb bit (bit 0) set, as
        // every function pointer here does; the bootrom branches to it as
        // given.
        let mut launch = Launch::new(
            &raw const crate::VECTOR_TABLE as u32,
            top as u32,
            core1_start as *const () as u32,
        );
        let mut fifo = Fifo::new();
        let start = timer.now();
        loop {
            if launch.step(&mut fifo, timer.now()) == Step::Launched {
                return Ok(fifo);
            }
            if timer.elapsed_since(start) > timeout_us {
                return Err(LaunchError::NoAnswer);
            }
        }
    }
}

/// Core 1's first code: the entry point `launch` hands the bootrom, entered
/// with `SP` at the top of core 1's stack and `VTOR` at the vector table.
///
/// Sets core 1's stack limit before anything touches the stack, so that
/// whatever limit core 1 was left with cannot fault the first push: an
/// ordinary function would push its return address before its first line
/// ran. It takes no stack itself: `#[unsafe(naked)]` gives it no prologue,
/// only the instructions written here. Then it branches to `core1_run`.
#[unsafe(naked)]
unsafe extern "C" fn core1_start() -> ! {
    core::arch::naked_asm!(
        "movw r0, :lower16:{limit}",
        "movt r0, :upper16:{limit}",
        // Load-acquire, as `CORE1_STACK_LIMIT.load(Ordering::Acquire)`.
        "lda r0, [r0]",
        "msr MSPLIM, r0",
        "b {run}",
        limit = sym CORE1_STACK_LIMIT,
        run = sym core1_run,
    )
}

/// Core 1's code after `core1_start`: enables core 1's floating-point unit,
/// as `OnReset` does on core 0 and before any floating-point instruction,
/// then runs the application's function with core 1's [`Fifo`].
extern "C" fn core1_run() -> ! {
    unsafe { crate::enable_fpu() };
    let entry = CORE1_ENTRY.load(Ordering::Acquire);
    // SAFETY: `launch` stored a `fn(Fifo) -> !` here before the handshake,
    // and nothing else writes it.
    let entry = unsafe { core::mem::transmute::<usize, fn(Fifo) -> !>(entry) };
    entry(Fifo::new())
}
