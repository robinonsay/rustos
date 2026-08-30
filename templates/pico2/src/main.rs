//! Blinky — the smallest complete application on this runtime.
//!
//! `main` claims the board singleton, brings up GPIO, and configures the
//! on-board LED pin as a push-pull output. Everything past those three
//! lines is written against the portable `api` traits, so on different
//! hardware only the two concrete type names and the pin field change.
//!
//! Build and flash (put the board in BOOTSEL mode, or have picotool reboot
//! a running image):
//!
//! ```text
//! cargo run --release
//! ```

#![no_std]
#![no_main]

use core::hint::spin_loop;
use core::panic::PanicInfo;

use api::gpio::Gpio;
use api::common::Write;
use pico2::common::board::Rp2350;
use pico2::gpio::gpio::Rp2350Gpio;

/// Where the program ends up after any `panic!`, failed `unwrap`, array
/// bounds violation, or arithmetic overflow in a debug build.
///
/// The runtime leaves panic policy to the application, so every binary must
/// define exactly one `#[panic_handler]`. Spinning forever is the honest
/// minimum: there is no console to print to and no operating system to
/// return to, and it holds the machine at the fault for a debugger.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Declares this crate's entry point to the runtime and type-checks its
// signature against `fn() -> !` at compile time.
pico2::entry!(main);

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

/// Busy-wait long enough for a blink to be visible. There is no configured
/// timer yet; the count is calibrated by eye.
fn delay() {
    for _ in 0..5_000_000 {
        spin_loop();
    }
}
