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
//! This mirrors the `demo` crate in this workspace (delays elided):
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//!
//! use api::{common::Write, gpio::Gpio};
//! use pico2::common::board::Rp2350;
//! use pico2::gpio::gpio::Rp2350Gpio;
//!
//! pico2::entry!(main);
//!
//! fn main() -> ! {
//!     let board = Rp2350::take().unwrap();        // once per boot; constructs every handle
//!     let mut gpio = Rp2350Gpio::new(board.gpio); // consumes the device handle; releases the GPIO blocks from reset
//!     let mut led = gpio.output_from_handle(board.pins.led).unwrap();
//!     loop {
//!         led.write(true);
//!         led.write(false);
//!     }
//! }
//! ```
//!
//! The two crate attributes are required, not style. `#![no_std]`
//! links only Rust's `core` library — the language items, with no operating
//! system services — which is why there is no `println!`, no heap, and why
//! this crate must supply the `#[panic_handler]`. `#![no_main]` is explained
//! at [`entry!`].
//!
//! Building an application needs configuration as well as source: the
//! `thumbv8m.main-none-eabihf` target and the `-Tlink.ld` linker flag. In
//! this workspace `.cargo/config.toml` supplies both, and `pico2`'s
//! `build.rs` puts `link.ld` on the linker search path — a crate outside the
//! workspace must replicate that configuration (and first run
//! `rustup target add thumbv8m.main-none-eabihf`), or the build links for
//! the host, or fails to link at all.
//!
//! `main` is an ordinary unattributed function taking no arguments: it claims
//! the board singleton from [`common::board`] (whose `take` succeeds at most
//! once per boot and returns the full set of pin- and device-ownership
//! handles), exchanges device handles for drivers, converts pin handles into
//! configured pins, and never returns. See [`entry`] for why the macro is needed and
//! what it protects you from, and [`common::board`] for what a pin handle
//! proves.
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
//!    word 0 into `SP` (the stack pointer register) and word 1 into `PC` (the
//!    program counter) — see `VECTOR_TABLE`, which the linker script pins to
//!    the flash base.
//! 3. [`OnReset`] runs with flash mapped read-only over XIP and **RAM
//!    uninitialised**: enable the FPU (the floating-point unit; see
//!    `enable_fpu`), point `VTOR` at the table, copy `.data` from flash to
//!    RAM (`reset_data`), zero `.bss` (`reset_bss` — both explained on those
//!    functions below). (XIP is *execute-in-place*: the
//!    QMI flash controller presents the flash contents as ordinary readable
//!    memory starting at `0x1000_0000`, so the CPU fetches instructions
//!    directly from flash with nothing copied to RAM first.)
//! 4. `OnReset` tail-calls the application entry point, which never returns.
//!
//! ## Layering note
//!
//! This crate currently holds three logically distinct layers, which is fine
//! at this size but worth naming, since only the first is genuinely tied to
//! Arm:
//!
//! * **Cortex-M runtime** — `VECTOR_TABLE`, [`OnReset`], `enable_fpu`,
//!   `VTOR`. Portable to any Armv8-M chip.
//! * **RP2350 chip support** — `BOOT_INFO`, [`common::reg`],
//!   [`common::reset`], [`gpio`]. Portable to any RP2350 board, and notably
//!   *not* Arm-specific: RP2350 can boot RISC-V Hazard3 cores instead, driving
//!   these same registers (p14).
//! * **Pico 2 board support** — [`common::board`], which declares exactly the
//!   pins this board has and names the ones its circuitry has already
//!   claimed, and [`common::MAX_GPIO_PIN`], which records the package pin
//!   count — the constant a pin-number validation would check against,
//!   though today the only enforcement is that the board definition declares
//!   handles solely for pins 0–29 (see the constant's doc). See each item's
//!   doc for why these live here.
//!
//! The `Board` type itself is generated in [`common::board`] by the `api`
//! crate's `define_board!` macro; the application does not state board facts
//! any more — the `demo` crate reaches the on-board LED as `board.pins.led`
//! rather than by hard-coding pin 25.
//!
//! ## Host builds
//!
//! The Cortex-M runtime above (`BOOT_INFO`, `VECTOR_TABLE`, [`OnReset`] and the
//! helpers it calls) exists only when the crate is compiled for a bare-metal
//! target (`target_os = "none"`). It references linker-script symbols and
//! Armv8-M barrier instructions that a host toolchain cannot resolve, so on the
//! host those items are compiled out and the rest of the crate (register
//! layouts, the drivers, and the pure decision functions the drivers are
//! built on) compiles and runs under `cargo test -p pico2 --lib` and Miri.
//! Nothing a host test can reach touches a register: the drivers only
//! dereference peripheral addresses on the target.

#![no_std]
// On the host the target-only entry points (`Mmio` and the driver
// constructors that use it) are compiled out, so the sequences they call look
// unused outside `cargo test`.
#![cfg_attr(not(target_os = "none"), allow(dead_code))]

// Host unit tests use `std` collections for the scripted register file.
#[cfg(test)]
extern crate std;

#[cfg(target_os = "none")]
use core::ptr::copy_nonoverlapping;

pub mod clocks;
pub mod common;
pub mod critical_section;
pub mod gpio;
pub mod irq;
pub mod timer;

#[cfg(target_os = "none")]
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
// SAFETY: `link_section` is unsafe because a wrong section can place data where code or
// other data is expected. `.boot_info` is a section of its own in `link.ld`, `KEEP`ed,
// word-aligned and asserted to start within the first 4 kB of flash, and this 20-byte
// immutable array is the only object placed in it; its words are the minimum Arm
// IMAGE_DEF of RP2350 datasheet section 5.9.5.1, read only by the bootrom.
#[used]
#[unsafe(link_section = ".boot_info")]
static BOOT_INFO: [u32; 5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579,
];

#[cfg(target_os = "none")]
// Symbols defined by `link.ld`. These have an ADDRESS but no VALUE — the
// linker places them, it does not store anything at them. Reading one as a
// `u32` yields whatever bytes happen to live there; always take `&raw const`
// and use the resulting pointer.
// SAFETY: an extern block is unsafe because its declarations are not checked against a
// definition. `_stack_top` is defined by `link.ld` as `ORIGIN(RAM) + LENGTH(RAM)`, a
// symbol with an address and no storage. This crate only ever takes `&raw const
// _stack_top` (for vector table slot 0) and never reads through it, so the declared type
// `u32` is never used to load a value.
unsafe extern "C" {
    /// One past the last valid RAM byte; the initial stack pointer. The stack
    /// is full-descending, so the first push lands at `_stack_top - 4` and
    /// this address is never itself dereferenced — which matters, because it
    /// is outside the decoded SRAM range and would bus-fault.
    static _stack_top: u32;
}

#[cfg(target_os = "none")]
// SAFETY: an extern block is unsafe because its declarations are not checked against a
// definition. The five symbols are defined by `link.ld` at the word-aligned bounds of
// `.data` (RAM, with its load address in flash) and `.bss`, each section wrapped in
// `ALIGN(4)`. This crate only takes their addresses with `&raw const` and never loads a
// value through them, so the declared type `u32` only fixes pointer alignment, which the
// `ALIGN(4)` of each bound satisfies.
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

#[cfg(target_os = "none")]
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

#[cfg(target_os = "none")]
// SAFETY: `Vector` contains a raw pointer, which is not `Sync`, so the
// compiler will not let a `static` hold one without this. It is sound here
// because the table is immutable, lives in read-only flash, and is only ever
// read by hardware performing vector fetches.
unsafe impl Sync for Vector {}

#[cfg(target_os = "none")]
/// Cortex-M33 private peripheral block base. Datasheet §3.7.5: "The Arm
/// Cortex-M33 registers start at a base address of 0xe0000000, defined as
/// PPB_BASE".
///
/// Everything in this region is defined by Arm, not by Raspberry Pi — it is
/// the one peripheral area that is genuinely portable across Cortex-M parts.
const PPB_BASE: usize = 0xE000_0000;

#[cfg(target_os = "none")]
/// Coprocessor Access Control Register, PPB offset `0x0ED88` (§3.7).
const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;

#[cfg(target_os = "none")]
/// Vector Table Offset Register, PPB offset `0x0ED08`.
const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

#[cfg(target_os = "none")]
/// Full access (`0b11`) for CP10 and CP11 — together these are the FP
/// extension. Both fields must hold the same value or the result is UNKNOWN
/// (Table 229).
const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22); // == 0x00F0_0000

#[cfg(target_os = "none")]
/// Enable the floating-point unit.
///
/// The FPU is disabled out of reset. Executing any FP instruction before this
/// runs raises a UsageFault with `NOCP` (no coprocessor) set — and the
/// compiler emits FP instructions freely for a `-none-eabihf` target, so this
/// must happen before essentially any other code.
///
/// Read-modify-write rather than a plain store, to preserve the other
/// coprocessor fields (CP0/CP4/CP5/CP7) that share this register. The
/// accesses are *volatile*: `read_volatile`/`write_volatile` tell the
/// compiler the access has an effect it cannot see, so it must perform each
/// one exactly as written — neither merged, reordered, nor deleted. (See the
/// `RESET_DONE` doc in [`common::reset`] for the failure mode when volatile
/// is omitted.) The
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
// SAFETY: `unsafe` passes the obligation of the "# Safety" section to the caller: called
// once, before any floating-point instruction. The sole caller, `OnReset`, calls it first,
// before any other Rust code of the image runs.
unsafe fn enable_fpu() {
    // SAFETY: CPACR is PPB_BASE `0xe000_0000` plus `0x0ed88` (RP2350 datasheet section
    // 3.7.5, List of Registers), an aligned 32-bit system register; the read-modify-write
    // sets only CP10 and CP11 to full access (the FP extension) and keeps the other fields.
    // Volatile accesses through a raw pointer form no reference. `dsb` and `isb` touch no
    // memory or stack and preserve flags, as the `asm!` options declare, and make the new
    // setting take effect before the next FP instruction.
    unsafe {
        let current = CPACR.read_volatile(); // READ
        let updated = current | CPACR_FPU_FULL; // MODIFY — preserves CP0/CP4/CP5/CP7
        CPACR.write_volatile(updated); // WRITE
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

#[cfg(target_os = "none")]
/// Point `VTOR` at our vector table.
///
/// The bootrom entered us using the table at the flash base, but it does not
/// necessarily leave `VTOR` pointing there — and interrupts taken later are
/// dispatched through `VTOR`, not through wherever the reset vector came from.
/// Setting it explicitly makes the two agree.
///
/// Alignment matters: Armv8-M requires the table to be aligned to the next
/// power of two at least as large as its byte size. 68 entries × 4 bytes =
/// 272 bytes; the next power of two at or above 272 is 512, so the table
/// must be 512-byte aligned. `link.ld` enforces this and asserts it. (RP2350's `VTOR`
/// only implements bits 31:7, a 128-byte granularity — the stricter 512 comes
/// from the architecture, so keep it.)
///
/// # Safety
///
/// Writes a CPU control register; call once, from [`OnReset`].
#[inline]
// SAFETY: `unsafe` passes the obligation of the "# Safety" section to the caller: called
// once, from `OnReset`, before any interrupt is enabled. `OnReset` is its sole caller.
unsafe fn reset_vtor() {
    // SAFETY: VTOR is PPB_BASE `0xe000_0000` plus `0x0ed08` (RP2350 datasheet section
    // 3.7.5), an aligned 32-bit system register, written once with a volatile store through
    // a raw pointer. The value is the address of `VECTOR_TABLE`, which `link.ld` places at
    // the flash origin with 512-byte alignment (asserted), meeting the Armv8-M alignment for
    // 68 entries and the bits 31:7 that RP2350 implements. `dsb` and `isb` touch no memory
    // or stack and preserve flags, as the `asm!` options declare.
    unsafe {
        VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

#[cfg(target_os = "none")]
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
// SAFETY: `unsafe` passes the obligation of the "# Safety" section to the caller: called
// once, from `OnReset`, before any code reads a static. `OnReset` is its sole caller.
unsafe fn reset_data() {
    let src = &raw const __sidata; // flash (LMA)
    let dst = &raw const __sdata as *mut u32; // RAM (VMA)
    let end = &raw const __edata as *const u32;
    let count = (end as usize - dst as usize) / 4;
    // SAFETY: `copy_nonoverlapping` requires `src` valid for reading and `dst` valid for
    // writing `count` words, both aligned, and the ranges disjoint. `link.ld` sets `__sdata`
    // and `__edata` as the word-aligned bounds of `.data` in RAM and `__sidata` as its load
    // address in flash (`> RAM AT > FLASH`, `ALIGN(4)`), so `count` is the exact word count,
    // the source lies in flash and the destination in RAM, which do not overlap. Nothing
    // else accesses `.data` yet: this runs before any code that reads a static.
    unsafe { copy_nonoverlapping(src, dst, count) }
}

#[cfg(target_os = "none")]
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
// SAFETY: `unsafe` passes the obligation of the "# Safety" section to the caller: called
// once, from `OnReset`, before any code reads a static. `OnReset` is its sole caller.
unsafe fn reset_bss() {
    let p = &raw const __sbss as *mut u32;
    let end = &raw const __ebss as *const u32;
    let count = (end as usize - p as usize) / 4;
    // SAFETY: `write_bytes` requires `p` valid for writing `count` aligned words. `link.ld`
    // sets `__sbss` and `__ebss` as the word-aligned bounds of `.bss` in RAM (`NOLOAD`,
    // `ALIGN(4)`), so `count` is its exact word count and every byte written belongs to
    // `.bss`. Nothing else accesses `.bss` yet: this runs before any code that reads a static.
    unsafe { p.write_bytes(0, count) }
}

#[cfg(target_os = "none")]
// The application entry point. This symbol is not defined anywhere in this
// crate — it is defined by the binary that links against it, via `entry!`.
// This is the Rust equivalent of a C forward declaration, with the same
// property that the linker matches on name alone; see `entry!` for how the
// signature is nonetheless checked.
// SAFETY: an extern block is unsafe because the declaration is not checked against the
// definition. The only definition is the one `entry!` emits, whose body coerces the
// application function to `fn() -> !` before the symbol exists, so the definition has
// exactly the declared signature; a mismatch is a compile error in the application.
unsafe extern "Rust" {
    fn __rustos_main() -> !;
}

/// Declare the application entry point: a function of type `fn() -> !`.
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
/// -> u32`, and the linker matches them on name alone; at runtime the callee
/// reads an argument nobody passed.
///
/// This macro removes that unchecked case with one line:
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
        // SAFETY: `no_mangle` is unsafe because an unmangled symbol can collide with another of
        // the same name. `__rustos_main` is defined only here, the macro is invoked once per
        // application binary, and a second invocation is a duplicate-symbol error at link time;
        // the signature matches the extern declaration in this crate (see its SAFETY comment).
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_main() -> ! {
            // Type check: rejects any signature other than fn() -> !.
            let f: fn() -> ! = $f;
            f()
        }
    };
}

#[cfg(target_os = "none")]
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
// SAFETY: `no_mangle` is unsafe because an unmangled symbol can collide with another of
// the same name. `OnReset` is defined once, here, and is the name `link.ld` gives
// `ENTRY`; no other object of the image defines it.
#[unsafe(no_mangle)]
pub extern "C" fn OnReset() -> ! {
    // SAFETY: each callee is called exactly once, here, in the order its contract requires:
    // the FPU is enabled before any FP instruction, VTOR is set before any interrupt is
    // enabled, and `.data` and `.bss` are initialised before any code reads a static, because
    // this is the reset handler and no other Rust code of the image has run. `__rustos_main`
    // has the declared signature `fn() -> !` (see the extern block's SAFETY comment).
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
// SAFETY: `no_mangle` is unsafe because an unmangled symbol can collide with another of
// the same name. `DefaultHandler` is defined once, here, and no other object of the
// image defines it; it takes no argument and never returns, as a vector-table handler.
#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() {
    loop {}
}

/// HardFault handler, exception 3.
///
/// Reached by a bus fault, a misaligned or illegal access, an escalated
/// lower-priority fault, or — most often during bring-up — a call through a
/// null or garbage function pointer.
// SAFETY: `no_mangle` is unsafe because an unmangled symbol can collide with another of
// the same name. `OnHardFault` is defined once, here, and no other object of the image
// defines it; it takes no argument and never returns, as a vector-table handler.
#[unsafe(no_mangle)]
pub extern "C" fn OnHardFault() {
    loop {}
}

#[cfg(target_os = "none")]
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
/// `ORIGIN(FLASH)` and wraps in `KEEP()`. Both halves are required: without
/// the `KEEP()`, `--gc-sections` deletes the table, since nothing in Rust
/// *calls* a vector table; without the placement, the bootrom cannot find it
/// at offset 0.
// SAFETY: `link_section` is unsafe because a wrong section can place data where code or
// other data is expected. `.vector_table` is a section of its own in `link.ld`, `KEEP`ed
// at the flash origin with 512-byte alignment (both asserted), and this immutable table
// of 68 word-sized `Vector` entries is the only object placed in it, read only by the
// hardware on reset and exception entry (RP2350 datasheet section 5.9.3.3: with no
// VECTOR_TABLE or ENTRY_POINT item the bootrom assumes the table at the image start).
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
    t[16] = Vector { handler: TIMER0_IRQ_0 };
    t[17] = Vector { handler: TIMER0_IRQ_1 };
    t[18] = Vector { handler: TIMER0_IRQ_2 };
    t[19] = Vector { handler: TIMER0_IRQ_3 };
    t[20] = Vector { handler: TIMER1_IRQ_0 };
    t[21] = Vector { handler: TIMER1_IRQ_1 };
    t[22] = Vector { handler: TIMER1_IRQ_2 };
    t[23] = Vector { handler: TIMER1_IRQ_3 };
    t[24] = Vector { handler: PWM_IRQ_WRAP_0 };
    t[25] = Vector { handler: PWM_IRQ_WRAP_1 };
    t[26] = Vector { handler: DMA_IRQ_0 };
    t[27] = Vector { handler: DMA_IRQ_1 };
    t[28] = Vector { handler: DMA_IRQ_2 };
    t[29] = Vector { handler: DMA_IRQ_3 };
    t[30] = Vector { handler: USBCTRL_IRQ };
    t[31] = Vector { handler: PIO0_IRQ_0 };
    t[32] = Vector { handler: PIO0_IRQ_1 };
    t[33] = Vector { handler: PIO1_IRQ_0 };
    t[34] = Vector { handler: PIO1_IRQ_1 };
    t[35] = Vector { handler: PIO2_IRQ_0 };
    t[36] = Vector { handler: PIO2_IRQ_1 };
    t[37] = Vector { handler: IO_IRQ_BANK0 };
    t[38] = Vector { handler: IO_IRQ_BANK0_NS };
    t[39] = Vector { handler: IO_IRQ_QSPI };
    t[40] = Vector { handler: IO_IRQ_QSPI_NS };
    t[41] = Vector { handler: SIO_IRQ_FIFO };
    t[42] = Vector { handler: SIO_IRQ_BELL };
    t[43] = Vector { handler: SIO_IRQ_FIFO_NS };
    t[44] = Vector { handler: SIO_IRQ_BELL_NS };
    t[45] = Vector { handler: SIO_IRQ_MTIMECMP };
    t[46] = Vector { handler: CLOCKS_IRQ };
    t[47] = Vector { handler: SPI0_IRQ };
    t[48] = Vector { handler: SPI1_IRQ };
    t[49] = Vector { handler: UART0_IRQ };
    t[50] = Vector { handler: UART1_IRQ };
    t[51] = Vector { handler: ADC_IRQ_FIFO };
    t[52] = Vector { handler: I2C0_IRQ };
    t[53] = Vector { handler: I2C1_IRQ };
    t[54] = Vector { handler: OTP_IRQ };
    t[55] = Vector { handler: TRNG_IRQ };
    t[56] = Vector { handler: PROC0_IRQ_CTI };
    t[57] = Vector { handler: PROC1_IRQ_CTI };
    t[58] = Vector { handler: PLL_SYS_IRQ };
    t[59] = Vector { handler: PLL_USB_IRQ };
    t[60] = Vector { handler: POWMAN_IRQ_POW };
    t[61] = Vector { handler: POWMAN_IRQ_TIMER };
    t[62] = Vector { handler: SPAREIRQ_IRQ_0 };
    t[63] = Vector { handler: SPAREIRQ_IRQ_1 };
    t[64] = Vector { handler: SPAREIRQ_IRQ_2 };
    t[65] = Vector { handler: SPAREIRQ_IRQ_3 };
    t[66] = Vector { handler: SPAREIRQ_IRQ_4 };
    t[67] = Vector { handler: SPAREIRQ_IRQ_5 };
    t
};

#[cfg(target_os = "none")]
// SAFETY: an extern block is unsafe because its declarations are not checked against a
// definition. Each name is defined by `link.ld` as `PROVIDE(<name> = DefaultHandler)`, a
// function of this signature, unless the application defines it with `interrupt!`, whose
// expansion has exactly this signature (`extern "C" fn()`). This crate never calls them; it
// only places their addresses in `VECTOR_TABLE` for the hardware (RP2350 datasheet section
// 3.2, Table 94: device interrupt n is exception 16 + n).
unsafe extern "C" {
    fn TIMER0_IRQ_0();
    fn TIMER0_IRQ_1();
    fn TIMER0_IRQ_2();
    fn TIMER0_IRQ_3();
    fn TIMER1_IRQ_0();
    fn TIMER1_IRQ_1();
    fn TIMER1_IRQ_2();
    fn TIMER1_IRQ_3();
    fn PWM_IRQ_WRAP_0();
    fn PWM_IRQ_WRAP_1();
    fn DMA_IRQ_0();
    fn DMA_IRQ_1();
    fn DMA_IRQ_2();
    fn DMA_IRQ_3();
    fn USBCTRL_IRQ();
    fn PIO0_IRQ_0();
    fn PIO0_IRQ_1();
    fn PIO1_IRQ_0();
    fn PIO1_IRQ_1();
    fn PIO2_IRQ_0();
    fn PIO2_IRQ_1();
    fn IO_IRQ_BANK0();
    fn IO_IRQ_BANK0_NS();
    fn IO_IRQ_QSPI();
    fn IO_IRQ_QSPI_NS();
    fn SIO_IRQ_FIFO();
    fn SIO_IRQ_BELL();
    fn SIO_IRQ_FIFO_NS();
    fn SIO_IRQ_BELL_NS();
    fn SIO_IRQ_MTIMECMP();
    fn CLOCKS_IRQ();
    fn SPI0_IRQ();
    fn SPI1_IRQ();
    fn UART0_IRQ();
    fn UART1_IRQ();
    fn ADC_IRQ_FIFO();
    fn I2C0_IRQ();
    fn I2C1_IRQ();
    fn OTP_IRQ();
    fn TRNG_IRQ();
    fn PROC0_IRQ_CTI();
    fn PROC1_IRQ_CTI();
    fn PLL_SYS_IRQ();
    fn PLL_USB_IRQ();
    fn POWMAN_IRQ_POW();
    fn POWMAN_IRQ_TIMER();
    fn SPAREIRQ_IRQ_0();
    fn SPAREIRQ_IRQ_1();
    fn SPAREIRQ_IRQ_2();
    fn SPAREIRQ_IRQ_3();
    fn SPAREIRQ_IRQ_4();
    fn SPAREIRQ_IRQ_5();
}
