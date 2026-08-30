//! RP2350 GPIO driver.
//!
//! Implements the portable [`api::gpio`] traits — a trait is Rust's named
//! interface: a set of method signatures that a type implements, letting
//! callers be written against the signatures rather than against the concrete
//! type — on top of the register layouts in [`crate::gpio`]. Four hardware
//! blocks cooperate to make one pin work:
//!
//! | Peripheral | Question it answers |
//! |------------|---------------------|
//! | `RESETS`   | Are the GPIO blocks powered up at all? |
//! | `IO_BANK0` | Which peripheral is connected to this pin? (`FUNCSEL`) |
//! | `PADS_BANK0` | How does the physical pad behave electrically? |
//! | `SIO`      | Holds the live pin level (`GPIO_IN`), the output level (`GPIO_OUT`), and the output enable (`GPIO_OE`) |
//!
//! All four participate in configuration: the first three must be released
//! from reset and set up, and an output pin additionally needs its `GPIO_OE`
//! bit set in SIO. A pin that "does nothing" almost always means one of those
//! steps was skipped, because none of the blocks report an error when
//! neglected.
//!
//! The intended call sequence, once per boot: claim the board singleton
//! (`Rp2350::take()` in [`crate::common::board`]) to obtain the
//! [`PinHandle`]s, construct [`Rp2350Gpio`] to release the GPIO blocks from
//! reset, then exchange each handle for a configured pin through
//! [`Gpio::input_from_handle`] / [`Gpio::output_from_handle`].
//!
//! A complete runnable program — including the `#![no_std]`/`#![no_main]`
//! attributes, the `entry!` declaration, and a delay loop — is the demo crate
//! (`demo/src/main.rs`). One rule it demonstrates that nothing else here
//! states: a trait method such as [`Write::write`] is only callable when the
//! trait itself is imported (`use api::common::Write;`), because Rust resolves
//! method calls only through traits in scope.

use core::fmt::Debug;

use crate::common::MAX_GPIO_PIN;
use crate::common::reset::{clr_reset_reg, wait_for_reset_done};
use crate::gpio::{IoBank, PadsBank, Sio};
use crate::common::reg::RegAddr;
use api::common::{ErrorType, Read, Write};
use api::device::{DeviceHandle, PinHandle};
use api::gpio::{Gpio, GpioPinIn, GpioPinOut, Pull};

/// Something went wrong configuring a GPIO pin.
pub enum GpioError
{
    /// The requested pin number is not bonded out on this package.
    ///
    /// No safe code path currently returns this. The [`Gpio`] methods take a
    /// [`PinHandle<N>`](PinHandle), and safe code can only obtain handles
    /// from the board definition in [`crate::common::board`], which names
    /// pins 0–29 — exactly the pins that exist. The variant is kept because
    /// out-of-range pins are the failure the hardware itself will never
    /// report, so any future path that accepts a runtime pin number needs an
    /// error to return.
    ///
    /// Why the hardware stays silent: the
    /// `IO_BANK0` and `PADS_BANK0` register arrays are 48 entries wide in
    /// every RP2350 package, so configuring pin 40 on an RP2350A — whose
    /// package bonds out only 30 user GPIOs — succeeds at the bus level and
    /// drives a pad connected to nothing.
    /// Worse, `1 << pin` in the SIO path misbehaves quietly: in debug builds
    /// an oversized shift panics, but in release builds the hardware shift
    /// instruction uses only the low 5 bits of the amount (shifts of a 32-bit
    /// value are taken mod 32), so `1 << 33` computes `1 << 1` and pin 33
    /// would end up toggling pin 1.
    PinOOB {
        /// The pin number that was requested.
        pin: usize,
        /// How many pins this package actually has; see
        /// [`MAX_GPIO_PIN`].
        count: usize,
    },
}

/// Hand-written rather than derived so the error type does not require
/// `#[derive(Debug)]` on anything else, and so the formatting stays explicit
/// about what each field means.
impl Debug for GpioError
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PinOOB { pin, count } => f.debug_struct("PinOOB").field("pin", pin).field("count", count).finish(),
        }
    }
}


/// The GPIO port: bring-up for the whole bank, and the factory for individual
/// pins.
///
/// Zero-sized. There is no state to hold — the registers live at fixed
/// addresses — so this exists purely to give the [`Gpio`] and [`ErrorType`]
/// trait impls (an `impl Trait for Type` block: the code that provides a
/// trait's methods for one concrete type) somewhere to live.
///
/// The `_private: ()` field makes the struct literal `Rp2350Gpio{}`
/// unwritable outside this module, so the only route to an instance is
/// [`new`](Self::new) — which demands the board's
/// [`DeviceHandle<Rp2350Gpio>`](api::device::DeviceHandle). At most one such
/// handle exists per boot, and `new` consumes it, so at most one
/// `Rp2350Gpio` value ever exists, with no runtime check in this module at
/// all.
pub struct Rp2350Gpio
{
    _private: ()
}

impl Rp2350Gpio
{
    /// Release the GPIO blocks from reset and construct the port driver.
    ///
    /// Clears the `IO_BANK0` and `PADS_BANK0` bits in `RESETS.RESET` and
    /// waits for both to report done. Every pin constructor in this module
    /// assumes those blocks are running, and this is the only place that
    /// starts them — and because the [`DeviceHandle`] argument can only come
    /// from the board, the bring-up provably runs before any pin is
    /// configured. The mask is passed complemented because
    /// [`clr_reset_reg`] ANDs the register with its argument; see
    /// [`crate::common::reset`].
    ///
    /// Consuming the handle is the whole once-per-boot story. The handle is
    /// created once, inside the board's `take()` (see
    /// [`define_board!`](api::define_board)); taking it here by value moves
    /// it, so a second `new(board.gpio)` is a compile error — use of a moved
    /// value. That is why this returns `Self`, not `Option<Self>`: with the
    /// only failure mode (a second construction) rejected by the compiler,
    /// there is nothing left to report. A second instance would have been an
    /// independent `&mut`-taking view of the same physical registers,
    /// defeating the exclusive access the [`Gpio`] methods guarantee.
    pub fn new(_handle: DeviceHandle<Rp2350Gpio>) -> Self
    {
        unsafe {
            clr_reset_reg(!IO_PAD_BITMASK);
            wait_for_reset_done(IO_PAD_BITMASK);
        }
        return Self{_private: ()};
    }
}

/// The factory half of the driver: each method consumes a [`PinHandle`] and
/// returns the configured pin type for it.
///
/// Consuming the handle — taking it by value, not by reference — is what
/// makes pin ownership linear. Safe code receives each handle exactly once,
/// from the board singleton in [`crate::common::board`], and once a handle is
/// passed by value to one of these methods it has been moved: the compiler
/// rejects any later use of the binding it came from, so the same pin cannot
/// be configured twice, or as both input and output.
impl Gpio for Rp2350Gpio
{
    type Input<const N: usize> = Rp2350GpioIn<N>;

    type Output<const N: usize> = Rp2350GpioOut<N>;

    fn input_from_handle<const N: usize>(&mut self, handle: PinHandle<N>, pull: Pull) -> Result<Self::Input<N>, Self::Error> {
        return Rp2350GpioIn::new_input(handle, pull);
    }

    fn output_from_handle<const N: usize>(&mut self, handle: PinHandle<N>) -> Result<Self::Output<N>, Self::Error> {
        return Rp2350GpioOut::new_output(handle);
    }
}

/// `IO_BANK0` — bit 6 of `RESETS.RESET` (Table 534, p504).
const IOBANK_RESET_BIT:u8 = 6;
/// `PADS_BANK0` — bit 9 of `RESETS.RESET`.
const PADBANK_RESET_BIT:u8 = 9;
/// Both GPIO blocks. Neither alone is sufficient: `IO_BANK0` routes the signal
/// and `PADS_BANK0` connects it to a physical leg of the package.
const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;

/// Port-level operations name [`GpioError`] as their error type. Nothing in
/// the current driver actually returns it — see [`GpioError::PinOOB`] for why
/// the variant is kept anyway — but the [`Gpio`] trait's methods are fallible
/// and [`ErrorType`] is `Gpio`'s supertrait (a trait that every implementor
/// of `Gpio` is required to also implement), so this impl is where their
/// error type must be declared.
impl ErrorType for Rp2350Gpio
{
    type Error = GpioError;
}

/// A pin configured as a SIO push-pull output — the pad actively drives both
/// levels, sourcing current to pull the pin high and sinking current to pull
/// it low (as opposed to open-drain, which only pulls low). Drive it with
/// [`Write::write`].
///
/// Zero-sized: the pin number is the const parameter `N` (see [`PinHandle`]'s
/// doc for what a zero-sized type is and how a const parameter bakes `N` into
/// the type), and all state lives in the hardware. Holding one of these is
/// proof the pin was configured — construction is the only place
/// configuration happens, the field is private, and the constructor is
/// private to this module, so the only way to obtain one is through
/// [`Rp2350Gpio`]'s [`Gpio`] impl.
///
/// One-owner-per-pin, and the limits of that guarantee: construction consumes
/// a [`PinHandle<N>`](PinHandle), and safe code obtains each handle exactly
/// once per boot, from the `take()` singleton that `define_board!` generates
/// in [`crate::common::board`]. So in safe code there is at most one live pin
/// object per physical pin. Rust permits at most one live `&mut` reference to
/// a value at a time, and the compiler (specifically its borrow checker)
/// rejects code that would create a second; because [`Write::write`] takes
/// `&mut self`, two call sites driving one pin would need two simultaneous
/// `&mut` borrows of the single pin object — which is exactly that rejected
/// case, so the conflict is a compile error. The guarantee
/// ends at two boundaries. First, `unsafe`: [`PinHandle::new`] is a
/// `const unsafe fn` that constructs a handle for any `N` with no record kept
/// anywhere, so unsafe code can construct a second `PinHandle<N>` for a pin
/// whose handle is already live, and nothing detects the duplicate —
/// correctness is then that caller's obligation. Second, the handle
/// is consumed, not stored, and nothing gives it back: there is no method to
/// release a pin, so reconfiguring one (say, output back to input) is
/// impossible in safe code, and dropping this object leaves the hardware
/// configured — an output keeps driving its last written level.
pub struct Rp2350GpioOut<const N: usize>
{
    _private: ()
}

/// A pin configured as a SIO input with a chosen [`Pull`] resistor. Sample it
/// with [`Read::read`].
///
/// Zero-sized, and governed by the same ownership rules as
/// [`Rp2350GpioOut`] — see that type for the one-owner guarantee and its
/// limits.
pub struct Rp2350GpioIn<const N: usize>
{
    _private: ()
}

impl<const N: usize> Rp2350GpioOut<N>{
    // Intended as a compile-time bound on N. Know its limit: an associated
    // const is only evaluated where it is referenced, and nothing references
    // this one, so today it rejects nothing (a build with N = 50 compiles).
    // The effective bound is that define_board! only creates handles for
    // pins that exist, 0-29.
    const _VALID: () = assert!(N < MAX_GPIO_PIN);
    /// Configure pin `N` as a push-pull output, driven low.
    ///
    /// The handle is consumed and discarded: its entire job was done at the
    /// type level, proving the caller owns pin `N`, so `N` needs no runtime
    /// check. Private — callers reach this through
    /// [`Gpio::output_from_handle`].
    fn new_output(_handle: PinHandle<N>) -> Result<Self, GpioError>
    {
        unsafe{
            configure_gpio_pin_out(N);
        }
        return Ok(Self{_private: {}})
    }
}

impl<const N: usize> Rp2350GpioIn<N>{
    // See the note on Rp2350GpioOut::_VALID: unreferenced, so inert.
    const _VALID: () = assert!(N < MAX_GPIO_PIN);
    /// Configure pin `N` as an input with the requested pull resistor.
    ///
    /// As with [`Rp2350GpioOut::new_output`], the consumed handle is the
    /// proof that pin `N` exists and is owned by the caller. Private —
    /// callers reach this through [`Gpio::input_from_handle`].
    fn new_input(_handle: PinHandle<N>, pull: Pull) -> Result<Self, GpioError>
    {
        unsafe{
            configure_gpio_pin_in(N, pull);
        }
        return Ok(Self{_private: {}})
    }
}

/// Driving and sampling a configured pin cannot fail.
///
/// [`Infallible`](core::convert::Infallible) is an empty enum, so it has no
/// values: `Result<(), Infallible>` is the same size as `()` and the error
/// branch is eliminated at compile time. This is honest rather than
/// optimistic — every failure mode was excluded before construction, when
/// possession of the [`PinHandle`] established that pin `N` exists on this
/// package. A pin that exists cannot fail to be written. (One caveat,
/// inherited from the [`Rp2350Gpio`] struct-literal hole documented there: a
/// pin configured through a literal-constructed port may act on GPIO blocks
/// still held in reset, and such a pin silently does nothing.)
impl<const N: usize> ErrorType for Rp2350GpioOut<N>
{
    type Error = core::convert::Infallible;
}

impl<const N: usize> ErrorType for Rp2350GpioIn<N>
{
    type Error = core::convert::Infallible;
}

impl<const N: usize> Write<bool> for Rp2350GpioOut<N>
{
    /// Drive the pin high or low.
    ///
    /// Selects the *register* rather than computing the *data*, which is the
    /// only correct way to do this. `GPIO_OUT_SET` and `GPIO_OUT_CLR` both act
    /// on the bits you write as `1` and ignore zeros, so the tempting
    /// `gpio_out_set.write_volatile(value << pin)` writes `0` for `false` —
    /// which is a no-op, leaving the pin stuck high forever.
    ///
    /// One store, no read-modify-write, so no window in which an interrupt or
    /// the other core could lose an update to a neighbouring pin. `SIO` is
    /// excluded from the `+0x2000`/`+0x3000` atomic aliases precisely because
    /// it provides these dedicated registers instead (§2.1.3, p27).
    fn write(&mut self, value: bool) -> Result<(), Self::Error> {
        let sio_addr = RegAddr::SIO as usize as *mut Sio;
        unsafe
        {
            let set_reg = match value{
                true => &raw mut (*sio_addr).gpio_out_set,
                false => &raw mut (*sio_addr).gpio_out_clr
            };
            set_reg.write_volatile(1 << N);
        }
        return  Ok(());
    }
}

impl<const N: usize> Read<bool> for Rp2350GpioIn<N>
{
    /// Sample the level actually present on the pad.
    ///
    /// Reads `GPIO_IN`, not `GPIO_OUT`. The distinction matters: `GPIO_OUT`
    /// reads back the last value *written*, whereas `GPIO_IN` reports what the
    /// pin is really at. On an output pin those differ whenever external
    /// circuitry sets the pad's level despite the driver — a short to ground,
    /// or a load drawing more current than the configured `DRIVE` strength
    /// can supply — which makes this the cheapest fault detection available.
    ///
    /// Requires `IE` to be set in the pad register. With the input buffer
    /// disabled this returns `false` regardless of the voltage on the leg,
    /// which is why both configuration paths set `IE`.
    fn read(&mut self) -> Result<bool, Self::Error> {
        let sio_addr = RegAddr::SIO as usize as *mut Sio;
        unsafe{
            let in_reg = &raw const (*sio_addr).gpio_in;
            return Ok(((in_reg.read_volatile() & (1 << N))) == 1 << N)
        }
    }
}

impl<const N: usize>  GpioPinIn<N> for Rp2350GpioIn<N>{}
impl<const N: usize>  GpioPinOut<N> for Rp2350GpioOut<N>{}

/// Configure one pin as a SIO input with the requested pull resistor.
///
/// Order is deliberate and matters:
///
/// 1. Clear the output enable in SIO, so the pin is not driven while it is
///    being reconfigured.
/// 2. Set up the pad — `OD`, `IE`, and the pull resistors.
/// 3. Point `FUNCSEL` at SIO.
/// 4. Clear `ISO` **last**.
///
/// Step 4 is the one that is easy to miss. Pads come out of reset isolated
/// (`ISO` resets to `1`, Table 852) so that a half-configured pad never drives
/// anything; releasing the latch before the rest is set up defeats the point,
/// and never releasing it at all leaves a pin that is configured correctly and
/// still does nothing.
///
/// # Safety
///
/// `pin` must be `< MAX_GPIO_PIN`. Larger values index past the pad array
/// (a bounds panic) or overflow the shift (silently masked in release builds).
/// Callers reach this only through the pin constructors, whose `N` came from
/// a [`PinHandle`] and is therefore a pin the board actually has.
unsafe fn configure_gpio_pin_in(pin: usize, pull: Pull)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        // Stop SIO driving the pin before reconfiguring it.
        let gpio_oe_clr = &raw mut (*sio_addr).gpio_oe_clr;
        gpio_oe_clr.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        // Read-modify-write: DRIVE and SCHMITT keep their reset values.
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        // OD disables the pad's output driver and "has priority over output
        // enable from peripherals" (Table 852). The gpio_oe_clr write above
        // already stopped SIO asserting output enable; OD additionally cuts
        // the driver at the pad itself, so the pin stays undriven even if
        // some later code sets the SIO output enable again.
        current_pad |= 1 << OD;
        // IE enables the input buffer. Without it GPIO_IN reads 0 forever, no
        // matter what voltage is on the pin. IE resets to 0, so this is the
        // one bit an input cannot do without.
        current_pad |= 1 << IE;
        // Pull resistors, Table 852 p785. PDE resets to 1 and PUE to 0, so a
        // fresh pad already has a pull-down. Every arm therefore writes BOTH
        // bits: setting one without clearing the other leaves both enabled,
        // which is a legal bus-keeper configuration and almost never intended.
        const PUE: u8 = 3;
        const PDE: u8 = 2;
        match pull {
            Pull::Up   => { current_pad |=  1 << PUE; current_pad &= !(1 << PDE); }
            Pull::Down => { current_pad &= !(1 << PUE); current_pad |=  1 << PDE; }
            Pull::None => { current_pad &= !(1 << PUE); current_pad &= !(1 << PDE); }
        }
        pad.write_volatile( current_pad);
        // FUNCSEL = 5 connects the pin to SIO. Writing the whole register also
        // clears IRQOVER/INOVER/OEOVER/OUTOVER to "normal", which is wanted.
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
        // Release the isolation latch now that mux and pad are both set.
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
    }
}

/// Configure one pin as a SIO push-pull output, initially driven low.
///
/// Same four-step ordering as [`configure_gpio_pin_in`], with the pad set up
/// to drive instead of to sample. The output value is cleared *before* the
/// output enable is set, so the pin never briefly drives high on its way to
/// being configured.
///
/// `IE` is set here too, even though this is an output. That is deliberate and
/// matches what the SDK's `gpio_set_function` does: it enables the input
/// buffer so [`Read::read`] can report the level actually on the pad.
///
/// # Safety
///
/// `pin` must be `< MAX_GPIO_PIN`; see [`configure_gpio_pin_in`].
unsafe fn configure_gpio_pin_out(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        // Drive low first, then enable the driver: no glitch high.
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        // OD must be clear or the pad refuses to drive regardless of SIO.
        current_pad &= !(1 << OD);
        // IE on so the pin can also be read back; see the doc comment.
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
        // Release the isolation latch last.
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
    }
}
