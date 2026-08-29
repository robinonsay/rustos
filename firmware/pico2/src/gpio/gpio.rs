//! RP2350 GPIO driver.
//!
//! Implements the portable [`api::gpio`] traits on top of the register layouts
//! in [`crate::gpio`]. Three peripherals cooperate to make one pin work, and
//! all three must be configured or the pin does nothing:
//!
//! | Peripheral | Question it answers |
//! |------------|---------------------|
//! | `RESETS`   | Are the GPIO blocks powered up at all? |
//! | `IO_BANK0` | Which peripheral is connected to this pin? (`FUNCSEL`) |
//! | `PADS_BANK0` | How does the physical pad behave electrically? |
//! | `SIO`      | What level is the pin at right now? |
//!
//! A pin that "does nothing" almost always means one of the first three was
//! skipped, because none of them report an error when neglected.

use core::fmt::Debug;

use crate::common::MAX_GPIO_PIN;
use crate::common::reset::{Block, clr_reset_reg, set_reset_reg, wait_for_reset_done};
use crate::gpio::{IoBank, PadsBank, Sio};
use crate::common::reg::RegAddr;
use api::common::{ErrorType, Write, Read};
use api::gpio::{Gpio, Pull};

/// Something went wrong configuring a GPIO pin.
pub enum GpioError
{
    /// The requested pin number is not bonded out on this package.
    ///
    /// Returned by [`Rp2350Gpio::init_input`] and
    /// [`Rp2350Gpio::init_output`] before any register is touched.
    ///
    /// This error exists because the hardware will not produce one. The
    /// `IO_BANK0` and `PADS_BANK0` register arrays are 48 entries wide in
    /// every RP2350 package, so configuring pin 40 on a 30-pin RP2350A
    /// succeeds at the bus level and drives a pad connected to nothing.
    /// Worse, `1 << pin` in the SIO path silently masks the shift to 5 bits in
    /// release builds, so pin 33 would end up toggling pin 1.
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
/// addresses — so this exists purely to give the [`Block`] and [`Gpio`] trait
/// impls somewhere to live.
pub struct Rp2350Gpio
{

}

/// `IO_BANK0` — bit 6 of `RESETS.RESET` (Table 534, p504).
// Bit for IOBANK
const IOBANK_RESET_BIT:u8 = 6;
/// `PADS_BANK0` — bit 9 of `RESETS.RESET`.
// Bit for Pad
const PADBANK_RESET_BIT:u8 = 9;
/// Both GPIO blocks. Neither alone is sufficient: `IO_BANK0` routes the signal
/// and `PADS_BANK0` connects it to a physical leg of the package.
// IO Pad bitmask
const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;

impl Block for Rp2350Gpio
{
    /// Release `IO_BANK0` and `PADS_BANK0`, then wait for both to report
    /// ready.
    ///
    /// The complement is passed because [`clr_reset_reg`] performs
    /// `RESET &= mask` and `0` means released — so zeros in the mask are the
    /// bits being freed. The [`wait_for_reset_done`] call is not optional:
    /// releasing a reset takes time, and registers written before the block is
    /// ready are accepted by the bus and discarded.
    unsafe fn start(&self) {
        unsafe{
            clr_reset_reg(!IO_PAD_BITMASK);
            wait_for_reset_done(IO_PAD_BITMASK);
        }
    }

    /// Return `IO_BANK0` and `PADS_BANK0` to reset.
    ///
    /// [`set_reset_reg`] performs `RESET |= mask`, so the mask is passed
    /// uncomplemented here — the inverse of [`start`](Self::start), which
    /// clears bits and therefore passes `!IO_PAD_BITMASK`. Only these two bits
    /// are touched; the other 27 blocks keep whatever state they were in.
    ///
    /// There is no `wait_for_reset_done` counterpart: `RESET_DONE` reports
    /// readiness, and a block being held in reset simply never reports ready.
    ///
    /// # Caveat
    ///
    /// Any [`Rp2350GpioPin`] handed out earlier stays alive and keeps
    /// compiling, but the block behind it is now in reset, so writes through
    /// it are accepted by the bus and discarded. Nothing in the type system
    /// catches that — it is the reason this method is `unsafe` despite writing
    /// only one register.
    unsafe fn reset(&self) {
        unsafe{
            set_reset_reg(IO_PAD_BITMASK);
        }
    }
}

/// Port-level operations report [`GpioError`], because validating a pin number
/// is the one thing here that can genuinely fail.
impl ErrorType for Rp2350Gpio
{
    type Error = GpioError;
}

impl Gpio<Rp2350GpioPin> for Rp2350Gpio
{
    fn init_input(pin_no: usize, pull: Pull) -> Result<Rp2350GpioPin, Self::Error> {
        return Rp2350GpioPin::new_input(pin_no, pull);
    }

    fn init_output(pin_no: usize) -> Result<Rp2350GpioPin, Self::Error>
    {
        return Rp2350GpioPin::new_output(pin_no);
    }
}

/// An owned, already-configured GPIO pin.
///
/// Holding one of these is proof that the pin number was validated and the
/// hardware was configured — construction is the only place either happens.
/// The field is private and both constructors are private to this module, so
/// the only way to obtain one is through [`Rp2350Gpio`]'s [`Gpio`] impl.
///
/// Because it is an owning handle, moving it moves control of the pin. Two
/// parts of an application cannot both drive it, since [`Write::write`] needs
/// `&mut self` and there is only ever one value.
pub struct Rp2350GpioPin
{
    /// Validated to be `< MAX_GPIO_PIN`. Every `1 << pin_no` in this module
    /// depends on that invariant holding.
    pin_no: usize,
}

impl Rp2350GpioPin{
    /// Validate and configure `pin_no` as an input.
    ///
    /// The bounds check happens *before* any register write, so a bad pin
    /// number leaves the hardware untouched.
    fn new_input(pin_no: usize, pull: Pull) -> Result<Self, GpioError>
    {
        if pin_no >= MAX_GPIO_PIN
        {
            return Err(GpioError::PinOOB { pin: pin_no, count: MAX_GPIO_PIN })
        }
        unsafe{
            configure_gpio_pin_in(pin_no, pull);
        }
        return Ok(Self{pin_no: pin_no})
    }

    /// Validate and configure `pin_no` as a push-pull output, driven low.
    fn new_output(pin_no: usize) -> Result<Self, GpioError>
    {
        if pin_no >= MAX_GPIO_PIN
        {
            return Err(GpioError::PinOOB { pin: pin_no, count: MAX_GPIO_PIN })
        }
        unsafe{
            configure_gpio_pin_out(pin_no);
        }
        return Ok(Self{pin_no: pin_no})
    }
}

/// Driving and sampling a configured pin cannot fail.
///
/// [`Infallible`](core::convert::Infallible) is an empty enum, so it has no
/// values: `Result<(), Infallible>` is the same size as `()` and the error
/// branch is eliminated at compile time. This is honest rather than
/// optimistic — every failure mode was already handled at construction, when
/// the pin number was checked. A pin that exists cannot fail to be written.
impl ErrorType for Rp2350GpioPin
{
    type Error = core::convert::Infallible;
}

impl Write<bool> for Rp2350GpioPin
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
            set_reg.write_volatile(1 << self.pin_no);
        }
        return  Ok(());
    }
}

impl Read<bool> for Rp2350GpioPin
{
    /// Sample the level actually present on the pad.
    ///
    /// Reads `GPIO_IN`, not `GPIO_OUT`. The distinction matters: `GPIO_OUT`
    /// reads back the last value *written*, whereas `GPIO_IN` reports what the
    /// pin is really at. On an output pin those differ whenever the outside
    /// world wins — a short to ground, or a load heavier than the pad's drive
    /// strength — which makes this the cheapest fault detection available.
    ///
    /// Requires `IE` to be set in the pad register. With the input buffer
    /// disabled this returns `false` regardless of the voltage on the leg,
    /// which is why both configuration paths set `IE`.
    fn read(&mut self) -> Result<bool, Self::Error> {
        let sio_addr = RegAddr::SIO as usize as *mut Sio;
        unsafe{
            let in_reg = &raw const (*sio_addr).gpio_in;
            return Ok(((in_reg.read_volatile() & (1 << self.pin_no))) == 1 << self.pin_no)
        }
    }
}

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
/// Callers reach this only through the checked constructors.
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
        // OD disables the output driver and "has priority over output enable
        // from peripherals" (Table 852) — belt to the gpio_oe_clr suspenders.
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
