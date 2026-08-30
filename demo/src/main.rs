//! Blinky — the smallest complete application on this runtime.
//!
//! This crate contains no `unsafe`, no register addresses, no pin numbers,
//! and no startup code. It is not hardware-agnostic — it names two concrete
//! Pico 2 types — but everything past the first three lines of `main` goes
//! through the portable `api` traits (a trait is a named set of method
//! signatures a type can implement, so code can be written against the
//! signatures rather than against the concrete type). Those three lines each
//! establish one guarantee:
//!
//! 1. `Rp2350::take()` claims the board singleton that `define_board!`
//!    generated in `pico2::common::board`. It succeeds at most once per boot
//!    (an `AtomicBool` compare-exchange — one indivisible read-modify-write,
//!    so exactly one caller can observe `false` and store `true`; every
//!    later call returns `None` — see the `define_board!` docs in
//!    `api::device`), and the `board.pins` it returns holds one zero-sized
//!    `PinHandle` per physical pin. From here on, possession of a handle is
//!    the proof that a pin exists and is unowned — handles can be moved but
//!    never duplicated.
//! 2. `Rp2350Gpio::new()` performs the hardware bring-up: it releases the
//!    `IO_BANK0` and `PADS_BANK0` blocks from reset and waits until
//!    `RESET_DONE` reports them ready (releasing an already-released block
//!    changes nothing, so this is safe to repeat). It then claims the driver
//!    singleton with a compare-exchange, so at most one `Rp2350Gpio` value
//!    ever exists and the exclusive access its `&mut` methods guarantee
//!    cannot be defeated by a second instance; nothing in the safe API can
//!    assert reset on the blocks again.
//! 3. `gpio.output_from_handle(board.pins.led)` consumes the handle by value
//!    and configures the pin it names — GPIO25, which the board wires to the
//!    on-board LED — as a push-pull output: one that actively drives the
//!    line to both levels, a transistor to the supply for high and a
//!    transistor to ground for low, as opposed to an open-drain output that
//!    only drives low and otherwise leaves the line floating. `PinHandle`
//!    implements neither `Copy` nor `Clone`, so passing it by value moves it
//!    out of `board.pins`, and the compiler rejects any later use of the
//!    moved-from field — configuring that pin a second time is a compile
//!    error, not a runtime conflict. The returned pin's `Write::write` is
//!    infallible (`Error = Infallible`): every failure mode was resolved
//!    during these three steps.
//!
//! On different hardware, only the two type names and the choice of pin
//! field would change; the loop is written entirely against
//! `api::common::Write`.
//!
//! Build and flash:
//!
//! ```text
//! cargo build --release -p demo
//! picotool uf2 convert target/thumbv8m.main-none-eabihf/release/demo demo.uf2
//! ```
//!
//! (`elf2uf2-rs` also does the conversion.) To flash: hold the BOOTSEL
//! button while plugging the Pico 2 into USB and it enumerates as a
//! mass-storage drive; copy `demo.uf2` onto that drive and the board reboots
//! into the program.

// `#![no_std]` links only Rust's `core` library — language items, no OS
// services — and `#![no_main]` hands the entry point to the runtime; both
// are explained at the top of the `pico2` crate docs.
#![no_std]
#![no_main]

use core::hint::spin_loop;

use api::gpio::Gpio;
use api::{common::Write};
use pico2::common::board::Rp2350;
use pico2::gpio::gpio::Rp2350Gpio;


// Declares this crate's entry point to the runtime. Expands to a
// `__rustos_main` shim that calls `main`, and — critically — type-checks
// `main`'s signature against `fn() -> !` at compile time. See the
// `pico2::entry` docs for why a plain `extern` declaration would not.
pico2::entry!(main);

/// Application entry point.
///
/// Runs once per boot: the runtime's `OnReset` — after enabling the FPU,
/// setting `VTOR`, copying `.data` and zeroing `.bss` — tail-calls the
/// `__rustos_main` shim that `entry!` above expanded to, and that shim calls
/// `main` through a checked `fn() -> !` pointer. Diverges: on bare metal
/// there is nothing to return to, and the `-> !` makes that a type error
/// rather than a convention.
fn main() -> ! {
    let board = Rp2350::take().unwrap();
    let mut gpio = Rp2350Gpio::new(board.gpio);
    let mut led = gpio.output_from_handle(board.pins.led).unwrap();
    loop {
        led.write(true);
        delay();
        led.write(false);
        delay();
    }
}

/// Busy-wait long enough for a blink to be visible.
///
/// `spin_loop` lowers to the CPU's spin-wait hint instruction, which also
/// keeps the optimiser from deleting the otherwise empty loop. The count is
/// calibrated by eye, not by clock — there is no configured timer yet.
fn delay()
{
    for _ in 0 .. 5_000_000
    {
        spin_loop();
    }
}