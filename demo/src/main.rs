//! Blinky — the smallest complete application on this runtime.
//!
//! Demonstrates the dependency-injection shape the whole system is built
//! around: this crate contains no `unsafe`, no register addresses, no
//! knowledge of the RP2350, and no startup code. It receives its hardware as
//! an argument and drives it through the portable `api` traits.
//!
//! Build and flash:
//!
//! ```text
//! cargo build --release -p demo
//! # then convert target/thumbv8m.main-none-eabihf/release/demo to UF2
//! ```

#![no_std]
#![no_main]

mod board;

use api::common::Write;
use board::Board;

// Declares this crate's entry point to the runtime. Expands to a shim that
// calls `main`, and — critically — type-checks `main`'s signature against `fn(Board) -> !` at compile time. See the
// `pico2::entry` docs for why a plain `extern` declaration would not.
pico2::entry!(main);

/// Application entry point.
///
/// Called once from `OnReset` after the FPU is enabled, `VTOR` is set, `.data`
/// is copied and `.bss` is zeroed. Diverges: on bare metal there is nothing to
/// return to, and the `-> !` makes that a type error rather than a convention.
fn main() -> ! {
    // SAFETY: called once, at the top of the only entry point.
    let mut board = unsafe { Board::take() };

    loop {
        // `Rp2350GpioPin`'s error type is `Infallible`, so this `Result` can
        // only ever be `Ok`. `.ok()` discards it without dragging panic
        // formatting into a binary that has no way to print it.
        board.led.write(true).ok();
        delay();

        board.led.write(false).ok();
        delay();
    }
}

/// Crude busy-wait, roughly a quarter second at the 150 MHz default clock.
///
/// Deliberately not a calibrated delay: there is no timer driver yet, and
/// counting instructions is honest about that. `spin_loop` emits a `yield`
/// hint and, more usefully here, is a side effect the optimiser is not allowed
/// to delete — a plain empty `for` loop would be compiled away entirely.
fn delay() {
    for _ in 0..5_000_000 {
        core::hint::spin_loop();
    }
}
