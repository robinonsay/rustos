//! The input snapshot of cwht WP-SW-02: every input of a group sampled in
//! one `SIO.GPIO_IN` read ([`InputSnapshot`], clauses SNP-1 to SNP-4).
//!
//! A timer handler that samples the key, the paddle contacts and the encoder
//! at 1 kHz needs all levels from the same instant; separate
//! [`api::gpio::GpioPinIn`] reads cannot give that. Sampling instead of edge
//! interrupts is the cwht input design (coding standard CS-35): no `IO_BANK0`
//! interrupt register is touched here.
//!
//! Everything the snapshot adds to the port driver lives in this file: the
//! input record kept by [`Rp2350Gpio`], the [`Rp2350Gpio::input_snapshot`]
//! constructor, its [`NotInput`] error and each input pin's `GPIO_IN` bit, so
//! that `gpio.rs` does not grow (cwht coding standard CS-18, INSP-098
//! finding-1).

use api::common::ErrorType;
use api::gpio::InputLevels;
#[cfg(target_os = "none")]
use api::gpio::InputSnapshot;

use crate::common::MAX_GPIO_PIN;
use crate::common::reg::{RegAddr, Regs};
use crate::gpio::Sio;
use crate::gpio::gpio::{GpioError, Rp2350Gpio, Rp2350GpioIn};

/// [`Rp2350Gpio::input_snapshot`] was asked for pins this port has not
/// configured as inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotInput {
    /// The requested pins that are not inputs of this port; an empty request
    /// reports `0`.
    pub mask: u32,
}

impl Rp2350Gpio {
    /// A sampler for the input pins in `mask` that reads them all in one
    /// `SIO.GPIO_IN` load (cwht WP-SW-02; [`InputSnapshot`](api::gpio::InputSnapshot)).
    ///
    /// Every pin in `mask` must already be an input of this port, configured
    /// through [`Gpio::input_from_handle`](api::gpio::Gpio::input_from_handle):
    /// the snapshot then only observes pins whose owner chose to make them
    /// inputs, and reading `GPIO_IN` has no side effect (§3.1.11, p55), so any
    /// number of samplers may exist.
    ///
    /// # Errors
    ///
    /// [`NotInput`] if `mask` is empty or names a pin that is not an input of
    /// this port.
    pub fn input_snapshot(&self, mask: u32) -> Result<Rp2350InputSnapshot, NotInput> {
        check_snapshot_mask(mask, self.inputs).map(Rp2350InputSnapshot::new)
    }

    /// Record pin `N` in the input record when its configuration as an input
    /// succeeded, and pass the result through (`input_from_handle`).
    pub(super) fn track_input<const N: usize>(
        &mut self,
        pin: Result<Rp2350GpioIn<N>, GpioError>,
    ) -> Result<Rp2350GpioIn<N>, GpioError> {
        if pin.is_ok() {
            self.inputs |= Rp2350GpioIn::<N>::BIT;
        }
        pin
    }
}

impl<const N: usize> Rp2350GpioIn<N> {
    /// This pin's bit in `GPIO_IN`. Referenced by `track_input`, so the bound
    /// is checked at compile time for every input pin actually built.
    pub(crate) const BIT: u32 = {
        assert!(N < MAX_GPIO_PIN);
        1 << N
    };
}

/// Samples a fixed group of input pins in one `SIO.GPIO_IN` read
/// ([`InputSnapshot`], clauses SNP-1 to SNP-4). Made by
/// [`Rp2350Gpio::input_snapshot`]; `Copy`, so a timer handler can own one.
#[derive(Clone, Copy)]
pub struct Rp2350InputSnapshot {
    mask: u32,
}

impl Rp2350InputSnapshot {
    /// A sampler for `mask`; the mask was accepted by [`check_snapshot_mask`].
    pub(crate) const fn new(mask: u32) -> Self {
        Self { mask }
    }

    /// The pins this sampler covers.
    #[must_use]
    pub const fn mask(&self) -> u32 {
        self.mask
    }
}

/// Accept a snapshot mask only if it is non-empty and every pin in it is a
/// configured input. Pure, host-tested.
pub(crate) const fn check_snapshot_mask(requested: u32, inputs: u32) -> Result<u32, NotInput> {
    let stray = requested & !inputs;
    if requested == 0 || stray != 0 {
        return Err(NotInput { mask: stray });
    }
    Ok(requested)
}

/// Offset of `GPIO_IN` in the SIO block (§3.1.11, Table 16, p55).
pub(crate) const SIO_GPIO_IN: usize = core::mem::offset_of!(Sio, gpio_in);
const _: () = assert!(SIO_GPIO_IN == 0x004);

/// One read of `GPIO_IN`, masked to the group (SNP-2, SNP-4).
pub(crate) fn read_inputs<R: Regs>(regs: &mut R, mask: u32) -> InputLevels {
    InputLevels::new(regs.read(RegAddr::SIO, SIO_GPIO_IN), mask)
}

impl ErrorType for Rp2350InputSnapshot {
    type Error = core::convert::Infallible;
}

#[cfg(target_os = "none")]
impl InputSnapshot for Rp2350InputSnapshot {
    fn snapshot(&mut self) -> Result<InputLevels, Self::Error> {
        Ok(read_inputs(&mut crate::common::reg::Mmio, self.mask))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::common::reg::fake::FakeRegs;

    const KEY_PADDLES: u32 = 0b111 << 2;

    #[test]
    fn mask_of_configured_inputs_is_accepted() {
        assert_eq!(
            check_snapshot_mask(KEY_PADDLES, KEY_PADDLES | 1 << 9).ok(),
            Some(KEY_PADDLES)
        );
    }

    #[test]
    fn mask_naming_a_non_input_is_rejected_with_the_stray_pins() {
        let r = check_snapshot_mask(KEY_PADDLES | 1 << 25, KEY_PADDLES);
        assert_eq!(r, Err(NotInput { mask: 1 << 25 }));
    }

    #[test]
    fn empty_mask_is_rejected() {
        assert_eq!(check_snapshot_mask(0, u32::MAX), Err(NotInput { mask: 0 }));
    }

    #[test]
    fn one_gpio_in_read_masked_to_the_group() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::SIO, 0x004, 0xffff_ffff);
        let levels = read_inputs(&mut regs, KEY_PADDLES);
        assert_eq!((levels.bits(), levels.mask()), (KEY_PADDLES, KEY_PADDLES));
        assert_eq!(regs.reads_of(RegAddr::SIO, 0x004), 1);
        assert_eq!(regs.log().len(), 1);
    }

    #[test]
    fn levels_follow_the_register() {
        let mut regs = FakeRegs::new();
        regs.script_reads(RegAddr::SIO, 0x004, &[0b01000, 0b10100]);
        assert_eq!(read_inputs(&mut regs, KEY_PADDLES).bits(), 0b01000);
        assert_eq!(read_inputs(&mut regs, KEY_PADDLES).bits(), 0b10100);
    }

    #[test]
    fn input_bit_constants() {
        assert_eq!(Rp2350GpioIn::<0>::BIT, 1);
        assert_eq!(Rp2350GpioIn::<29>::BIT, 1 << 29);
    }
}
