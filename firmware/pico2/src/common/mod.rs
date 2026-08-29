//! Chip-wide fundamentals: peripheral base addresses and the reset controller.
//!
//! Everything here is shared by all drivers and depends on none of them.

pub mod reg;
pub mod reset;

/// Number of GPIO pins usable on this board: **30**.
///
/// This is a *package* property, not a chip property, and the distinction
/// matters. RP2350 comes in two packages (datasheet p585):
///
/// | Part | Package | User GPIOs |
/// |------|---------|------------|
/// | RP2350A | QFN-60 | 30 |
/// | RP2350B | QFN-80 | 48 |
///
/// The Pico 2 carries the **RP2350A**, so GPIO0–29 exist and GPIO30–47 do not.
/// The register maps are identical in both packages — `IO_BANK0` still has 48
/// `GPIO*_CTRL` registers and `PADS_BANK0` still has 48 pad registers — so
/// writing to pin 40 on this board succeeds at the bus level, reports no
/// error, and drives a pad that is not bonded to any leg of the chip.
///
/// That silence is why this constant is checked at pin construction rather
/// than trusted. Strictly it is the *board* layer's fact rather than the
/// chip's; it lives here because this crate is currently both.
pub const MAX_GPIO_PIN: usize = 30;
