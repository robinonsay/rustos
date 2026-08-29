//! Raspberry Pi Pico 2 board definition.
//!
//! This module is the only place in the workspace that encodes facts about the
//! *board* rather than the *chip*. Everything here would be wrong on a
//! different RP2350 carrier, and nothing here would be wrong on a different
//! RP2350 revision — that is the line it draws.
//!
//! It lives in the application rather than in `pico2` on purpose. The `pico2`
//! crate knows how to bring up an RP2350; it has no business asserting that an
//! LED is on GPIO25, because that is true of this board and no other.

use api::gpio::Gpio;
use pico2::common::MAX_GPIO_PIN;
use pico2::common::reset::Block;
use pico2::gpio::gpio::{Rp2350Gpio, Rp2350GpioPin};

/// Every peripheral this application is allowed to touch, already brought out
/// of reset and configured.
///
/// Constructed once at the top of `main`, after which hardware is reached only
/// through its fields. Peripherals are moved out by value, so the borrow
/// checker enforces that two parts of the program cannot end up driving the
/// same pin.
///
/// Growing this struct is how the board gains capabilities: add a field and
/// add its bring-up to [`take`](Board::take).
pub struct Board {
    /// The green user LED, wired to GPIO25 and already configured as an output
    /// driven low.
    ///
    /// "GPIO25 OP Connected to user LED" — Pico 2 datasheet, pinout table.
    /// Active high: writing `true` lights it. One of the four RP2350 IOs the
    /// Pico 2 reserves for board functions; the other three handle SMPS power
    /// control and system voltage sensing, and are not exposed here.
    pub led: Rp2350GpioPin,
}

impl Board {
    /// GPIO number of the on-board user LED.
    pub const LED_PIN: usize = 25;

    /// Bring up the board: release the GPIO blocks from reset, wait for them
    /// to report ready, and configure the LED pin as an output.
    ///
    /// # Safety
    ///
    /// Creates owning handles to fixed hardware out of thin air, so calling it
    /// twice would produce two owners of the same pin. Call once, at the top
    /// of `main`.
    pub unsafe fn take() -> Self {
        // Release IO_BANK0 and PADS_BANK0 and block until RESET_DONE agrees.
        // Skipping the wait is the classic bring-up bug: writes to a block
        // still in reset are accepted by the bus and quietly discarded.
        unsafe { Rp2350Gpio {}.start() };

        let led = match Rp2350Gpio::init_output(Self::LED_PIN) {
            Ok(pin) => pin,
            // Unreachable: LED_PIN is checked against MAX_GPIO_PIN at compile
            // time by the assertion below, so this arm cannot be taken.
            Err(_) => unreachable!(),
        };

        Board { led }
    }
}

/// Compile-time proof that the LED is a pin this package actually bonds out.
///
/// A `const` block is evaluated during compilation, so a bad `LED_PIN` is a
/// build failure rather than a runtime `Err` the board has no way to report.
/// This is what lets `take` treat its error arm as unreachable.
const _: () = assert!(
    Board::LED_PIN < MAX_GPIO_PIN,
    "LED_PIN is not bonded out on this package"
);
