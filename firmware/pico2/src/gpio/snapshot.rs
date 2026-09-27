//! The input snapshot of cwht WP-SW-02: every input of a group sampled in
//! one `SIO.GPIO_IN` read ([`InputSnapshot`], clauses SNP-1 to SNP-4).
//!
//! A timer handler that samples the key, the paddle contacts and the encoder
//! at 1 kHz needs all levels from the same instant; separate
//! [`api::gpio::GpioPinIn`] reads cannot give that. Sampling instead of edge
//! interrupts is the cwht input design (coding standard CS-35): no `IO_BANK0`
//! interrupt register is touched here.

use api::common::ErrorType;
use api::gpio::InputLevels;
#[cfg(target_os = "none")]
use api::gpio::InputSnapshot;

use crate::common::reg::{RegAddr, Regs};
use crate::gpio::Sio;
use crate::gpio::gpio::GpioError;

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
pub(crate) const fn check_snapshot_mask(requested: u32, inputs: u32) -> Result<u32, GpioError> {
    let stray = requested & !inputs;
    if requested == 0 || stray != 0 {
        return Err(GpioError::NotInput { mask: stray });
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
    use crate::gpio::gpio::Rp2350GpioIn;

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
        assert!(matches!(r, Err(GpioError::NotInput { mask }) if mask == 1 << 25));
    }

    #[test]
    fn empty_mask_is_rejected() {
        assert!(matches!(
            check_snapshot_mask(0, u32::MAX),
            Err(GpioError::NotInput { mask: 0 })
        ));
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
