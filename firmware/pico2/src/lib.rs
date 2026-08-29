//! # `pico2` — bare-metal runtime and HAL for the Raspberry Pi Pico 2 (RP2350)
//!
//! This crate is a **library**, not a binary. It owns everything that has to
//! happen before and around user code — the boot metadata block, the vector
//! table, the reset handler, and the drivers — and calls out to an application
//! that lives in a separate binary crate. The application supplies one
//! function; this crate supplies everything else.
//!
//! ## Writing an application
//!
//! This mirrors the `demo` crate in this workspace:
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use api::{common::{Write, board::Board}, gpio::Gpio};
//! use pico2::gpio::gpio::Rp2350Gpio;
//!
//! pico2::entry!(main);
//!
//! fn main() -> ! {
//!     let mut gpio = Rp2350Gpio {};
//!     let _board = Board::take([&mut gpio]).unwrap(); // starts the GPIO blocks
//!     let mut led = gpio.init_output(25).unwrap();
//!     loop {
//!         led.write(true).ok();
//!         led.write(false).ok();
//!     }
//! }
//! ```
//!
//! `main` is an ordinary unattributed function taking no arguments: it
//! constructs its hardware drivers itself, registers them with the `Board`
//! singleton from the `api` crate (whose `take` starts each one and succeeds
//! at most once per boot), and never returns. See [`entry`] for why the macro
//! is needed and what it protects you from.
//!
//! ## Boot sequence
//!
//! Power-on to `main`, in order:
//!
//! 1. **Bootrom** scans the first 4 kB of flash for a valid `IMAGE_DEF`
//!    metadata block (`BOOT_INFO`). Without one it refuses to boot and falls
//!    through to USB mass-storage mode.
//! 2. Since that block declares no explicit entry point, the bootrom assumes
//!    the image begins with a Cortex-M vector table (§5.9.5.1, p427). It loads
//!    word 0 into `SP` and word 1 into `PC` — see `VECTOR_TABLE`, which the
//!    linker script pins to the flash base.
//! 3. [`OnReset`] runs with flash mapped read-only over XIP and **RAM
//!    uninitialised**: enable the FPU, point `VTOR` at the table, copy
//!    `.data` from flash to RAM, zero `.bss`. (XIP is *execute-in-place*: the
//!    QMI flash controller presents the flash contents as ordinary readable
//!    memory starting at `0x1000_0000`, so the CPU fetches instructions
//!    directly from flash with nothing copied to RAM first.)
//! 4. `OnReset` tail-calls the application entry point, which never returns.
//!
//! ## Layering note
//!
//! This crate currently holds two logically distinct layers, which is fine
//! at this size but worth naming, since only the first is genuinely tied to
//! Arm:
//!
//! * **Cortex-M runtime** — `VECTOR_TABLE`, [`OnReset`], `enable_fpu`,
//!   `VTOR`. Portable to any Armv8-M chip.
//! * **RP2350 chip support** — `BOOT_INFO`, [`common::reg`],
//!   [`common::reset`], [`gpio`]. Portable to any RP2350 board, and notably
//!   *not* Arm-specific: RP2350 can boot RISC-V Hazard3 cores instead, driving
//!   these same registers (p14).
//!
//! Board-level facts — which pin the LED is on, what is wired where — mostly
//! do not live in this crate; the application states them (the `demo` crate
//! hard-codes pin 25), and the `Board` singleton type lives in the `api`
//! crate. The one exception is [`common::MAX_GPIO_PIN`], a package fact this
//! crate keeps so pin numbers can be validated; see its doc for why.

#![no_std]

use core::{panic::PanicInfo, ptr::copy_nonoverlapping};

pub mod common;
pub mod gpio;

/// RP2350 `IMAGE_DEF` metadata block — **mandatory**; the chip will not boot
/// without it.
///
/// This replaces RP2040's 256-byte checksummed second-stage bootloader. The
/// bootrom searches the first 4 kB of the image for this structure, and if it
/// does not find a valid one the image is rejected outright.
///
/// These five words are the *minimum valid Arm `IMAGE_DEF`* given verbatim in
/// the datasheet (§5.9.5.1, p427):
///
/// | Word | Value | Meaning |
/// |------|-------------|---------|
/// | 0 | `0xffffded3` | `PICOBIN_BLOCK_MARKER_START` |
/// | 1 | `0x10210142` | item `0x42` = `IMAGE_TYPE`, size `0x01` word, flags `0x1021` = EXE, secure, Arm, RP2350 |
/// | 2 | `0x000001ff` | item `0xff` = `BLOCK_ITEM_2BS_LAST`, block size `0x0001` |
/// | 3 | `0x00000000` | relative pointer to next block; `0` means link to self, i.e. a loop of one |
/// | 4 | `0xab123579` | `PICOBIN_BLOCK_MARKER_END` |
///
/// The marker values were chosen to be unlikely to occur in compiled Arm or
/// RISC-V code, so the scan does not false-positive on ordinary instructions
/// (p357).
///
/// Because this block specifies no entry point, the bootrom falls back to
/// assuming a vector table at the image start. That fallback is why
/// `VECTOR_TABLE` must be at offset 0 and why the linker script asserts it.
///
/// # Attributes
///
/// * `#[used]` — nothing in the program reads this static, so without it
///   rustc is entitled to discard the symbol before the linker ever sees it.
/// * `#[link_section = ".boot_info"]` — the name must match the section in
///   `link.ld`, which `KEEP`s it (defeating `--gc-sections`) and places it in
///   the first 4 kB.
#[used]
#[unsafe(link_section = ".boot_info")]
static BOOT_INFO: [u32; 5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579,
];

// Symbols defined by `link.ld`. These have an ADDRESS but no VALUE — the
// linker places them, it does not store anything at them. Reading one as a
// `u32` yields whatever bytes happen to live there; always take `&raw const`
// and use the resulting pointer.
unsafe extern "C" {
    /// One past the last valid RAM byte; the initial stack pointer. The stack
    /// is full-descending, so the first push lands at `_stack_top - 4` and
    /// this address is never itself dereferenced — which matters, because it
    /// is outside the decoded SRAM range and would bus-fault.
    static _stack_top: u32;
}

unsafe extern "C" {
    /// Load address of `.data` in flash: where the initial values ship.
    static __sidata: u32;
    /// Start of `.data` in RAM: where they must be copied to.
    static __sdata: u32;
    /// End of `.data` in RAM.
    static __edata: u32;
    /// Start of `.bss` in RAM.
    static __sbss: u32;
    /// End of `.bss` in RAM.
    static __ebss: u32;
}

/// Where the program ends up after any `panic!`, failed `unwrap`, array
/// bounds violation, or arithmetic overflow in a debug build.
///
/// Spinning forever is the honest minimum: there is no console to print to and
/// no operating system to return to. It also stops execution at the fault
/// rather than letting it propagate, so a debugger attached afterwards finds
/// the machine still holding the state that caused it.
///
/// # Policy note
///
/// A program may contain exactly one `#[panic_handler]`. Because this library
/// defines it, applications built on this crate **cannot** define their own.
/// That is a deliberate choice appropriate to a runtime that owns startup; if
/// applications should be able to choose their own panic behaviour, this must
/// move out to the binary crate (which is what `cortex-m-rt` does).
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// One entry in the vector table.
///
/// A union rather than a plain `u32` so each slot can be written with the
/// value that is actually correct for it while keeping the array homogeneous:
/// slot 0 holds a stack pointer, slot 1 a diverging reset handler, most slots
/// an ordinary handler, and the architecturally reserved slots a literal zero.
/// All variants are word-sized, so the union is a word and the array is a
/// plain table of addresses — which is exactly what the hardware fetches.
#[repr(C)]
#[derive(Clone, Copy)]
union Vector {
    /// An ordinary exception or interrupt handler.
    handler: unsafe extern "C" fn(),
    /// The reset handler. Diverges: there is nothing to return to.
    reset: unsafe extern "C" fn() -> !,
    /// Slot 0 only: the initial stack pointer value.
    stack_top: *const u32,
    /// An architecturally reserved slot, which must read as zero.
    reserved: u32,
}

// SAFETY: `Vector` contains a raw pointer, which is not `Sync`, so the
// compiler will not let a `static` hold one without this. It is sound here
// because the table is immutable, lives in read-only flash, and is only ever
// read by hardware performing vector fetches.
unsafe impl Sync for Vector {}

/// Cortex-M33 private peripheral block base. Datasheet §3.7.5: "The Arm
/// Cortex-M33 registers start at a base address of 0xe0000000, defined as
/// PPB_BASE".
///
/// Everything in this region is defined by Arm, not by Raspberry Pi — it is
/// the one peripheral area that is genuinely portable across Cortex-M parts.
const PPB_BASE: usize = 0xE000_0000;

/// Coprocessor Access Control Register, PPB offset `0x0ED88` (§3.7).
const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;

/// Vector Table Offset Register, PPB offset `0x0ED08`.
const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

/// Full access (`0b11`) for CP10 and CP11 — together these are the FP
/// extension. Both fields must hold the same value or the result is UNKNOWN
/// (Table 229).
const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22); // == 0x00F0_0000

/// Enable the floating-point unit.
///
/// The FPU is disabled out of reset. Executing any FP instruction before this
/// runs raises a UsageFault with `NOCP` (no coprocessor) set — and the
/// compiler emits FP instructions freely for a `-none-eabihf` target, so this
/// must happen before essentially any other code.
///
/// Read-modify-write rather than a plain store, to preserve the other
/// coprocessor fields (CP0/CP4/CP5/CP7) that share this register. The
/// `dsb`/`isb` pair afterwards is architecturally required: `dsb` ensures the
/// write has reached the register, `isb` flushes the pipeline so instructions
/// already fetched are re-fetched under the new configuration. Without it the
/// very next FP instruction may still fault.
///
/// # Safety
///
/// Writes a CPU control register. Must be called exactly once, early in
/// [`OnReset`], before any floating-point code runs.
#[inline]
unsafe fn enable_fpu() {
    unsafe {
        let current = CPACR.read_volatile(); // READ
        let updated = current | CPACR_FPU_FULL; // MODIFY — preserves CP0/CP4/CP5/CP7
        CPACR.write_volatile(updated); // WRITE
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

/// Point `VTOR` at our vector table.
///
/// The bootrom entered us using the table at the flash base, but it does not
/// necessarily leave `VTOR` pointing there — and interrupts taken later are
/// dispatched through `VTOR`, not through wherever the reset vector came from.
/// Setting it explicitly makes the two agree.
///
/// Alignment matters: Armv8-M requires the table to be aligned to the next
/// power of two at least as large as its byte size. 68 entries round up to
/// 128, so 512 bytes. `link.ld` enforces this and asserts it. (RP2350's `VTOR`
/// only implements bits 31:7, a 128-byte granularity — the stricter 512 comes
/// from the architecture, so keep it.)
///
/// # Safety
///
/// Writes a CPU control register; call once, from [`OnReset`].
#[inline]
unsafe fn reset_vtor() {
    unsafe {
        VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

/// Copy initialised statics from flash to RAM.
///
/// `.data` is the one section with two addresses. Its initial values must
/// survive power-off, so they ship in flash at the section's *load address*
/// (LMA, "load memory address" — where the bytes are stored in the image;
/// here `__sidata`). But the variables must be writable, so compiled code
/// refers to them at the section's *runtime address* in RAM (VMA, "virtual
/// memory address"; here `__sdata`). `link.ld` sets the two apart with
/// `> RAM AT > FLASH`. Nothing moves the bytes for you — this function is
/// that step, and until it runs every non-zero `static mut` holds garbage.
///
/// # Safety
///
/// Writes across the whole `.data` region using linker-provided bounds. Call
/// once, from [`OnReset`], before any Rust code that reads a static.
#[inline]
unsafe fn reset_data() {
    let src = &raw const __sidata; // flash (LMA)
    let dst = &raw const __sdata as *mut u32; // RAM (VMA)
    let end = &raw const __edata as *const u32;
    let count = (end as usize - dst as usize) / 4;
    unsafe { copy_nonoverlapping(src, dst, count) }
}

/// Zero the uninitialised statics.
///
/// `.bss` holds statics whose initial value is all-zero. Storing those zeros
/// in flash would waste flash proportional to the size of every zeroed buffer,
/// so the section is `NOLOAD` — it occupies address space but ships no bytes,
/// and this function writes the zeros at runtime instead.
///
/// # Safety
///
/// Writes across the whole `.bss` region using linker-provided bounds. Call
/// once, from [`OnReset`].
#[inline]
unsafe fn reset_bss() {
    let p = &raw const __sbss as *mut u32;
    let end = &raw const __ebss as *const u32;
    let count = (end as usize - p as usize) / 4;
    unsafe { p.write_bytes(0, count) }
}

// The application entry point. This symbol is not defined anywhere in this
// crate — it is defined by the binary that links against it, via `entry!`.
// This is the Rust equivalent of a C forward declaration, with the same
// property that the linker matches on name alone; see `entry!` for how the
// signature is nonetheless checked.
unsafe extern "Rust" {
    fn __rustos_main() -> !;
}

/// Declare the application entry point.
///
/// ```ignore
/// pico2::entry!(main);
///
/// fn main() -> ! { loop {} }
/// ```
///
/// # Why a macro is necessary
///
/// On a `no_std` target you cannot have an ordinary `fn main`. rustc's normal
/// `main` is a shim that calls `lang_start`, which only `std` provides — the
/// error is a flat `using 'fn main' requires the standard library`, and
/// supplying `lang_start` yourself needs nightly. So the binary declares
/// `#![no_main]` and the runtime reaches user code through a named symbol.
///
/// The naive way to do that is a bare `extern` block, which is what C does and
/// carries C's defect: **extern declarations are not type-checked across the
/// link.** Declare `fn __rustos_main()`, define it as `fn __rustos_main(x: u32)
/// -> u32`, and the linker happily matches them on name; at runtime the callee
/// reads an argument nobody passed.
///
/// This macro closes that hole with one line:
///
/// ```ignore
/// let f: fn() -> ! = $f;
/// ```
///
/// Coercing the function item to a typed function pointer forces the compiler
/// to prove the signature matches *before* the symbol is emitted. A mismatch
/// is a `mismatched types` error in the application crate, pointing at the
/// `entry!` invocation.
///
/// `$crate` is not needed in the current expansion, but the macro is exported
/// at the crate root, so `pico2::entry!(..)` works with no accompanying `use`.
#[macro_export]
macro_rules! entry {
    ($f:path) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_main() -> ! {
            // Type check: rejects any signature other than fn() -> !.
            let f: fn() -> ! = $f;
            f()
        }
    };
}

/// Reset handler — the first Rust code to execute, entered directly from the
/// bootrom via vector table slot 1.
///
/// On entry, `SP` is set from slot 0 but **RAM holds whatever survived the
/// last power cycle**: `.data` is uninitialised and `.bss` is not zeroed. Any
/// code that touches a static before those steps run reads garbage, which is
/// why the four setup calls come first and in this order.
///
/// Diverges, and tail-calls the application entry point declared by
/// [`entry!`]. Because that function is itself `-> !`, there is no trailing
/// `loop {}`: "the application never returns" is enforced by the type system
/// rather than by a fallback.
///
/// `extern "C"` and `#[no_mangle]` because `link.ld` names this symbol in its
/// `ENTRY(OnReset)` directive, which both records the ELF entry point and
/// gives `--gc-sections` a root to trace reachability from.
#[unsafe(no_mangle)]
pub extern "C" fn OnReset() -> ! {
    unsafe {
        enable_fpu();
        reset_vtor();
        reset_data();
        reset_bss();
        __rustos_main()
    }
}

/// Catch-all for every exception and interrupt without a dedicated handler.
///
/// Spins, so an unexpected interrupt stops the program at the point of the
/// fault instead of returning into a corrupted state. Every slot in
/// `VECTOR_TABLE` starts out pointing here.
#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() {
    loop {}
}

/// HardFault handler, exception 3.
///
/// Reached by a bus fault, a misaligned or illegal access, an escalated
/// lower-priority fault, or — most often during bring-up — a call through a
/// null or garbage function pointer.
#[unsafe(no_mangle)]
pub extern "C" fn OnHardFault() {
    loop {}
}

/// The Armv8-M vector table: 68 words at the very start of flash.
///
/// Not code — an array of addresses. The hardware fetches from it directly on
/// reset and on every exception, indexed by exception number.
///
/// | Index | Contents |
/// |-------|----------|
/// | 0 | initial `SP` (`_stack_top`) |
/// | 1 | Reset ([`OnReset`]) |
/// | 2 | NMI |
/// | 3 | HardFault ([`OnHardFault`]) |
/// | 4–6 | MemManage, BusFault, UsageFault |
/// | 7 | SecureFault — Armv8-M Security Extension only |
/// | 8–10 | reserved, must read zero |
/// | 11 | SVCall |
/// | 12 | DebugMonitor |
/// | 13 | reserved, must read zero |
/// | 14–15 | PendSV, SysTick |
/// | 16–67 | 52 external interrupts (§3.2) |
///
/// The 52 device interrupts are an RP2350 number, not an Arm one, which is why
/// this array is 68 rather than some architectural constant. Only the lower 46
/// are wired to peripherals; IRQ46–51 are `SPAREIRQ_IRQ_0..5`, "hardwired to
/// zero (never firing)", reserved for a core to interrupt itself. The table is
/// still 68 entries.
///
/// Built in a `const` block so the whole thing is computed at compile time and
/// emitted as initialised flash contents: fill every slot with
/// [`DefaultHandler`], then overwrite the ones that are known.
///
/// `#[link_section = ".vector_table"]` puts it in the section `link.ld` pins to
/// `ORIGIN(FLASH)` and wraps in `KEEP()`. Both halves are load-bearing —
/// nothing in Rust *calls* a vector table, so `--gc-sections` would otherwise
/// delete it, and the bootrom requires it at offset 0.
#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0] = Vector { stack_top: &raw const _stack_top };
    t[1] = Vector { reset: OnReset };
    t[3] = Vector { handler: OnHardFault };
    t[8] = Vector { reserved: 0 };
    t[9] = Vector { reserved: 0 };
    t[10] = Vector { reserved: 0 };
    t[13] = Vector { reserved: 0 };
    t
};
