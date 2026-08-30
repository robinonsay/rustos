//! Portable general-purpose I/O traits.
//!
//! A GPIO pin is modelled as *a thing you can write a `bool` to and read a
//! `bool` from*, plus a factory that hands out configured pins. Nothing here
//! knows what a register is; see the `pico2` crate for the RP2350
//! implementation.

use crate::{common::{ErrorType, Read, Write}, device::PinHandle};

/// Which internal resistor, if any, holds an input pin at a known level when
/// nothing external is driving it.
///
/// An input with no pull is **floating**: its voltage is set by leakage and
/// nearby capacitive coupling, and it will read as noise — often oscillating,
/// which on an interrupt-enabled pin means a storm of spurious edges. A pull
/// resistor (typically 50–100 kΩ) is weak enough that any real driver
/// overrides it, but strong enough to define the level when none does.
///
/// Choosing between [`Up`](Pull::Up) and [`Down`](Pull::Down) is determined by
/// how the external circuit is wired, not by preference:
///
/// * **Button to ground** — the switch pulls the pin low when pressed, so the
///   pin must be held high otherwise: [`Pull::Up`], and the pressed state
///   reads `false`. This is the most common arrangement, because it needs no
///   extra components and is what most breakout boards assume.
/// * **Button to VCC** — the switch drives high when pressed, so the pin must
///   be held low otherwise: [`Pull::Down`], and pressed reads `true`.
/// * **Actively driven signal** — another chip drives both levels at all
///   times, so no pull is needed: [`Pull::None`]. Use this also when an
///   external resistor is already fitted, to avoid fighting it.
///
/// Getting this backwards does not fail loudly. The pin reads a constant
/// value, or works intermittently depending on lead capacitance, which is why
/// this is a required argument rather than something with a default.
pub enum Pull {
    /// Hold the pin high when undriven. Pair with a switch to ground.
    Up,
    /// Hold the pin low when undriven. Pair with a switch to VCC.
    Down,
    /// Leave the pin floating. Only correct when something else always drives
    /// it, or an external pull resistor is fitted.
    None,
}

/// A configured GPIO pin.
///
/// This is a **marker trait**: it adds no methods of its own and exists purely
/// to give a name to the capability "readable and writable as a `bool`". All
/// the actual behaviour arrives through the supertraits, so a `GpioPin` is
/// used via [`Write::write`] and [`Read::read`].
///
/// Reading a pin configured as an output is legitimate and useful — it reports
/// the level actually present on the pad, which is not always the level you
/// wrote. A pin shorted to ground, or loaded beyond its drive strength, reads
/// back the value the outside world won, which is the cheapest fault
/// detection available without extra hardware.
pub trait GpioPinIn<const N: usize>: Read<bool> {}

pub trait GpioPinOut<const N: usize>: Write<bool> {}

pub trait Gpio: ErrorType
{
    type Input<const N: usize>: GpioPinIn<N>;
    type Output<const N: usize>: GpioPinOut<N>;

    fn input_from_handle<const N: usize>(&mut self, handle: PinHandle<N>, pull: Pull) -> Result<Self::Input<N>, Self::Error>;
    fn output_from_handle<const N: usize>(&mut self, handle: PinHandle<N>) -> Result<Self::Output<N>, Self::Error>;
}
