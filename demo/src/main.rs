//! Blinky — the smallest complete application on this runtime.
//!
//! This crate contains no `unsafe`, no register addresses, and no startup
//! code, but it is not hardware-agnostic: it names the RP2350 driver type
//! (`pico2::gpio::gpio::Rp2350Gpio`) and hard-codes pin 25, the Pico 2's
//! on-board LED. `main` constructs that driver itself, hands it to
//! `Board::take` (which starts the GPIO blocks), and from then on touches the
//! pin only through the portable `api` traits (`Gpio::init_output`,
//! `Write::write`) — so only the lines that name the driver and the pin would
//! change on different hardware.
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
    let mut gpio = Rp2350Gpio::new().unwrap();
    let mut led = gpio.output_from_handle(board.pins.led).unwrap();
    loop {
        led.write(true);
        delay();
        led.write(false);
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