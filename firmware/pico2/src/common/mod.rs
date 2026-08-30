//! Fundamentals shared by every driver, depending on none of them.
//!
//! Two kinds of fact live here, at different levels. [`reg`] and [`reset`]
//! are *chip* facts: peripheral base addresses and the subsystem reset
//! controller, true of any RP2350. [`board`] and [`MAX_GPIO_PIN`] are
//! *board/package* facts: which pins the Pico 2 actually has and what its
//! circuitry wired them to.

pub mod reg;
pub mod reset;
pub mod board;

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
/// That silence is why pin validity must be established in software; the
/// hardware will never report it. Today one mechanism does so: [`board`]
/// constructs a `PinHandle` only for pins 0–29, so safe code cannot even
/// name a pin that is not bonded out. (The GPIO driver's pin types also
/// carry a `const` assertion against this constant, but it is an associated
/// const that nothing references, so it is never evaluated and currently
/// rejects nothing — see the note on `_VALID` in the driver.) Strictly it is a fact about the
/// package the board carries rather than about the RP2350 die itself; it
/// lives in this chip-support crate, rather than in the application, so
/// validation does not depend on trusting the caller. A build for the
/// RP2350B would change this one constant.
pub const MAX_GPIO_PIN: usize = 30;
