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

use core::hint::spin_loop;

use api::{common::{Write, board::Board}, gpio::Gpio};
use pico2::gpio::gpio::Rp2350Gpio;


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
    let mut gpio: Rp2350Gpio = Rp2350Gpio{};
    let _board = Board::take([&mut gpio]).unwrap();
    let mut pin25_o = gpio.init_output(25).unwrap();
    loop {
        pin25_o.write(true);
        delay();
        pin25_o.write(false);
        delay();
    }
}

fn delay()
{
    for _ in 0 .. 5_000_000
    {
        spin_loop();
    }
}