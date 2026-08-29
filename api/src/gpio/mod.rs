//! Portable general-purpose I/O traits.
//!
//! A GPIO pin is modelled as *a thing you can write a `bool` to and read a
//! `bool` from*, plus a factory that hands out configured pins. Nothing here
//! knows what a register is; see the `pico2` crate for the RP2350
//! implementation.

use crate::common::{ErrorType, Read, Write};

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
pub trait GpioPin: Write<bool> + Read<bool> {}

/// Blanket implementation: **every** type that is `Write<bool> + Read<bool>`
/// is automatically a `GpioPin`.
///
/// Implementors therefore never write `impl GpioPin for MyPin {}`; they
/// implement the two supertraits and membership follows. Two consequences are
/// worth knowing:
///
/// * There is no way to *opt out*. Any type meeting the bounds is a `GpioPin`
///   whether or not it is conceptually a pin. That is an acceptable trade for
///   a marker this thin, but it means the trait cannot later grow a required
///   method without breaking every implementor at once.
/// * The orphan rule means only this crate can write this blanket impl. A
///   downstream crate defining its own marker over foreign supertraits would
///   be rejected.
impl<T: Write<bool> + Read<bool>> GpioPin for T {}

/// The GPIO port: a factory that validates a pin number and hands back a
/// configured, ready-to-use pin.
///
/// Configuration happens **once, at construction**, and the returned `T` is
/// the proof that it succeeded. There is no `set_direction` on the pin itself,
/// so a pin cannot be reconfigured out from under code holding it, and an
/// out-of-range pin number is rejected before any register is touched rather
/// than silently aliasing onto a different pin.
///
/// Both methods take `&mut self`: configuring a pin changes hardware state,
/// and routing every configuration through a borrow of the port value gives
/// the application one place where that authority lives. The port type
/// itself may well be zero-sized — on most microcontrollers the port is a
/// fixed set of registers at a fixed address, so there is no data for the
/// value to carry — and the RP2350 implementation's `Rp2350Gpio` is exactly
/// that.
pub trait Gpio<T: GpioPin>: ErrorType {
    /// Configure `pin_no` as an input with the given [`Pull`].
    ///
    /// Returns `Self::Error` if `pin_no` is not a usable pin on this device.
    /// Note that "usable" is a property of the *package*, not the chip: the
    /// same silicon in a larger package bonds out more pins.
    fn init_input(&mut self, pin_no: usize, pull: Pull) -> Result<T, Self::Error>;

    /// Configure `pin_no` as a push-pull output, initially driven low.
    ///
    /// Starting low rather than high is deliberate: it is the state least
    /// likely to do something visible or damaging on a pin whose external
    /// wiring this layer knows nothing about.
    ///
    /// There is no [`Pull`] argument because an output drives both levels
    /// itself, making a pull resistor redundant at best and a small constant
    /// current draw at worst.
    fn init_output(&mut self, pin_no: usize) -> Result<T, Self::Error>;
}
