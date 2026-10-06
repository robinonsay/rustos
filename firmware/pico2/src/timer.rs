//! The 1 µs timebase: the `TICKS` tick generators and system timer `TIMER0`.
//!
//! # Ticks are not clocks
//!
//! On RP2350 the timers do not count a clock directly. A separate block,
//! `TICKS`, divides `clk_ref` by a programmable cycle count and emits one
//! *tick* pulse per period to each of six destinations: the two
//! Cortex-M33 SysTicks, `TIMER0`, `TIMER1`, the watchdog and the RISC-V
//! platform timer (§8.5.1, p568). Each destination has its own generator,
//! and each generator **resets disabled** (`ENABLE` reset value 0, Table
//! 623, p571; Table 629, p572). A timer whose generator is off simply never
//! counts: "The timer's tick must be running for the timer to start
//! counting" (§12.8.4, p1182).
//!
//! With `clk_ref` = 12 MHz from the crystal ([`crate::clocks`]), a cycle
//! count of 12 gives exactly one tick per microsecond (§8.5.1, p569). The
//! generator must be stopped before its cycle count changes (same page).
//!
//! # Two questions about the watchdog's tick, answered here
//!
//! **Does the RP2350 watchdog count only once the `TICKS` WATCHDOG
//! generator is enabled?** Yes. On RP2040 the watchdog contained its own
//! tick generator; on RP2350 it "instead takes a tick input from the
//! system-level ticks block" (§12.9.2, p1191), and that generator resets to
//! disabled like the others (Table 629, p572). An enabled watchdog with a
//! stopped tick never reaches zero, so [`crate::watchdog`] starts the
//! WATCHDOG generator itself rather than relying on anyone else to. (The
//! bootrom's own reboot routine does the same: it starts the WATCHDOG tick
//! with 12 cycles if it finds it off.)
//!
//! **Does the RP2350 watchdog still decrement twice per tick, like
//! RP2040?** No. That was erratum RP2040-E1, and nothing like it appears in
//! the RP2350 errata (Appendix E, p1351–1367, which has no watchdog entry
//! at all: its sections are ACCESSCTRL, Bootrom, DMA, GPIO, Hazard3, OTP,
//! RCP, SIO, XIP and USB). The register descriptions agree: `CTRL.TIME` "indicates
//! the time in usec before a watchdog reset", and `LOAD`'s maximum
//! `0xffffff` "corresponds to approximately 16 seconds" (Tables 1247–1248,
//! p1194–1195) — 16.7 million µs, i.e. one count per microsecond tick. The
//! SDK accordingly uses a scale factor of 1 on RP2350 and 2 only on RP2040.
//! Not verified on hardware.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::device::DeviceHandle;

use crate::clocks::{Rp2350Clocks, CLK_REF_HZ};
use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

/// `clk_ref` cycles per tick: 12, for a 1 µs tick from the 12 MHz crystal.
pub const TICK_CYCLES: u32 = CLK_REF_HZ / 1_000_000;
// CYCLES is a 9-bit field (Table 630, p572).
const _: () = assert!(TICK_CYCLES >= 1 && TICK_CYCLES < (1 << 9));

/// One tick generator: `CTRL`, `CYCLES`, `COUNT` (Table 616, p569).
#[repr(C)]
struct TickGenerator {
    /// Bit 0 `ENABLE` (RW, reset 0), bit 1 `RUNNING` (RO).
    ctrl: u32,
    /// Bits 8:0: `clk_ref` cycles per tick.
    cycles: u32,
    /// Bits 8:0: cycles remaining until the next tick (read-only).
    _count: u32,
}

/// The `TICKS` block, base `0x4010_8000`: six generators in the order
/// PROC0, PROC1, TIMER0, TIMER1, WATCHDOG, RISCV (Table 616, p569).
#[repr(C)]
struct Ticks {
    generator: [TickGenerator; 6],
}

const _: () = assert!(core::mem::size_of::<TickGenerator>() == 0x0c);
const _: () = assert!(core::mem::size_of::<Ticks>() == 0x48);

/// `TICKS_x_CTRL.ENABLE`.
const TICK_CTRL_ENABLE: u32 = 1 << 0;

/// Which tick generator to start.
#[derive(Clone, Copy)]
pub(crate) enum TickDestination {
    /// `TIMER0_CTRL` at `0x18`.
    Timer0 = 2,
    /// `WATCHDOG_CTRL` at `0x30`.
    Watchdog = 4,
}

/// Program one tick generator for a 1 µs tick and start it.
///
/// Stop, set `CYCLES`, start: §8.5.1 (p569) requires the generator to be
/// stopped while its cycle count changes.
///
/// # Safety
///
/// Changes the timebase of whatever the generator feeds; the caller must
/// own that destination.
pub(crate) unsafe fn start_tick(dest: TickDestination) {
    let ticks = RegAddr::TICKS as usize as *mut Ticks;
    unsafe {
        let generator = &raw mut (*ticks).generator[dest as usize];
        (&raw mut (*generator).ctrl).write_volatile(0);
        (&raw mut (*generator).cycles).write_volatile(TICK_CYCLES);
        (&raw mut (*generator).ctrl).write_volatile(TICK_CTRL_ENABLE);
    }
}

/// The `TIMER0` register block, base `0x400b_0000` (Table 1226, p1186).
/// Only the registers this driver uses are named; alarms and interrupts
/// are left for a later driver.
#[repr(C)]
struct Timer {
    /// `0x00`–`0x0c`: `TIMEHW`, `TIMELW`, `TIMEHR`, `TIMELR`. The latching
    /// pair `TIMELR`/`TIMEHR` is not used — see [`Rp2350Timer::now`].
    _time: [u32; 4],
    /// `0x10`–`0x1c`: `ALARM0`–`ALARM3`.
    _alarm: [u32; 4],
    /// `0x20`: `ARMED`.
    _armed: u32,
    /// `0x24`: `TIMERAWH`, bits 63:32 of the count, no side effects
    /// (Table 1236, p1189).
    timerawh: u32,
    /// `0x28`: `TIMERAWL`, bits 31:0 of the count, no side effects.
    timerawl: u32,
    /// `0x2c`: `DBGPAUSE` (Table 1238, p1189). Bits 1 and 2 pause the
    /// timer while core 0 / core 1 is halted by a debugger; both reset to
    /// 1 and are left that way.
    _dbgpause: u32,
    /// `0x30`: `PAUSE`, reset 0.
    _pause: u32,
    /// `0x34`: `LOCKED`, reset 0.
    _locked: u32,
    /// `0x38`: `SOURCE` (Table 1241, p1189). 0 (reset) counts ticks; 1
    /// would count `clk_sys` cycles instead.
    _source: u32,
}

const _: () = assert!(core::mem::offset_of!(Timer, timerawh) == 0x24);
const _: () = assert!(core::mem::offset_of!(Timer, timerawl) == 0x28);
const _: () = assert!(core::mem::offset_of!(Timer, _source) == 0x38);

/// `TIMER0` — bit 23 of `RESETS.RESET` (Table 534, p504).
const RESET_TIMER0: u32 = 1 << 23;

/// The microsecond system timer, `TIMER0`.
///
/// Counts microseconds since [`new`](Self::new) ran, in 64 bits. 2^64 µs
/// is about 584 000 years — the datasheet's "effectively cannot overflow"
/// (§12.8.1, p1180) — so this driver treats it as never wrapping.
///
/// All methods take `&self` and only read the counter, so one
/// `Rp2350Timer` can be shared by reference with every part of the
/// application that needs time.
pub struct Rp2350Timer {
    _private: (),
}

impl Rp2350Timer {
    /// Start the TIMER0 tick and bring `TIMER0` out of reset.
    ///
    /// Takes `&Rp2350Clocks` because the tick count of
    /// [`TICK_CYCLES`] is only right once `clk_ref` runs from the crystal.
    /// `TIMER0` is put through a full reset (Table 534, p504) so the count
    /// starts at 0 and every register is at its reset value even after a
    /// debugger warm reset, which restarts the processors but not the
    /// peripherals. The reset values are what this driver wants: count
    /// ticks (`SOURCE` = 0), not paused, pause while a debugger halts a
    /// core (`DBGPAUSE` = `0b110`), no alarms armed.
    pub fn new(_handle: DeviceHandle<Rp2350Timer>, _clocks: &Rp2350Clocks) -> Self {
        unsafe {
            start_tick(TickDestination::Timer0);
            cycle_reset(RESET_TIMER0);
        }
        Self { _private: () }
    }

    /// Microseconds since [`new`](Self::new).
    ///
    /// Reads the two raw halves without latching, retrying if the high
    /// word changed in between — the method the datasheet gives for reads
    /// that may race with the other core or an interrupt (§12.8.4.1,
    /// p1182–1183). The latching `TIMELR`/`TIMEHR` pair is avoided on
    /// purpose: reading `TIMELR` latches `TIMEHR` for whoever reads it next,
    /// so two readers interleaving their reads would corrupt each other.
    pub fn now(&self) -> u64 {
        let timer = RegAddr::TIMER0 as usize as *const Timer;
        unsafe {
            let raw_hi = &raw const (*timer).timerawh;
            let raw_lo = &raw const (*timer).timerawl;
            let mut hi = raw_hi.read_volatile();
            loop {
                let lo = raw_lo.read_volatile();
                let next_hi = raw_hi.read_volatile();
                if next_hi == hi {
                    return ((hi as u64) << 32) | lo as u64;
                }
                // The low word wrapped between the reads: try again with
                // the new high word.
                hi = next_hi;
            }
        }
    }

    /// Microseconds elapsed since `start`, a value earlier returned by
    /// [`now`](Self::now).
    pub fn elapsed_since(&self, start: u64) -> u64 {
        self.now().wrapping_sub(start)
    }

    /// Busy-wait for at least `us` microseconds.
    ///
    /// The wait is measured on the timer, so it is independent of
    /// `clk_sys`, the compiler and the cache, and it is at least `us`
    /// (it can overshoot by up to one microsecond plus however long
    /// interrupts, if any, delay the loop). It does not feed the watchdog:
    /// keep delays well below the watchdog timeout.
    pub fn delay_us(&self, us: u64) {
        let start = self.now();
        // `> us` rather than `>= us`: the first tick may come almost
        // immediately after `start` was read.
        while self.elapsed_since(start) <= us {
            core::hint::spin_loop();
        }
    }

    /// Busy-wait for at least `ms` milliseconds; see
    /// [`delay_us`](Self::delay_us).
    pub fn delay_ms(&self, ms: u32) {
        self.delay_us(ms as u64 * 1000);
    }
}
