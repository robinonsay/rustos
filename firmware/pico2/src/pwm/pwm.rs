//! The PWM driver: [`Rp2350Pwm`] and [`Rp2350PwmOut`].
//!
//! Register sequences are generic functions over
//! [`Regs`](crate::common::reg::Regs), run on the target through `Mmio` and in
//! host tests through the scripted register file.
//!
//! ## Glitch-free start and changes
//!
//! A new output's slice is configured and running with `CC = 0` before the
//! pin is switched to the PWM function, and the pad isolation is released
//! last, so the pin comes up low (PWM-6). Duty and on/off change only `CC`,
//! and `CC` and `TOP` are double-buffered (§12.5.2.3), so they take effect at
//! the next wrap with no partial pulse. `DIV` is not buffered: a frequency
//! change can stretch or shorten the one period in progress.

use api::common::ErrorType;
#[cfg(target_os = "none")]
use api::device::{DeviceHandle, PinHandle};
#[cfg(target_os = "none")]
use api::pwm::{Duty, PwmOutput};

#[cfg(target_os = "none")]
use crate::clocks::clocks::ClocksReady;
use crate::common::reg::{ALIAS_CLR, RegAddr, Regs, poll};
use crate::common::reset::{RESET_DONE_OFFSET, RESET_OFFSET};
#[cfg(target_os = "none")]
use crate::gpio::gpio::Rp2350Gpio;

use super::{
    CC, CSR, CSR_EN, CTR, DEFAULT_HZ, DIV, EN, FUNCSEL_PWM, PAD_ISO_OD, PwmError, RESET_BIT_PWM,
    RESET_POLL_BUDGET, SLICE_STRIDE, TOP, Timing, cc_value, gpio_ctrl, pad_ctrl, timing,
};

/// The PWM block as the board's `pwm` device: the factory for outputs.
pub struct Rp2350Pwm {
    clk_sys_hz: u32,
    /// Slices already given to an output, one bit per slice 0 to 7.
    claimed: u8,
}

/// Pin `N` as a PWM output, [`api::pwm::PwmOutput`].
pub struct Rp2350PwmOut<const N: usize> {
    clk_sys_hz: u32,
    period: u32,
    duty_permille: u16,
    on: bool,
}

impl<const N: usize> Rp2350PwmOut<N> {
    /// The slice of pin `N`: `(N / 2) mod 8` (Table 1129). Referenced by the
    /// constructor, so `N >= 30` is a compile error.
    pub const SLICE: usize = {
        assert!(N < crate::common::MAX_GPIO_PIN);
        (N >> 1) & 7
    };
    /// The channel of pin `N`: 0 = A (even pins), 1 = B (odd pins).
    pub const CHANNEL: usize = N & 1;
}

impl Rp2350Pwm {
    /// Release the PWM block from reset and disable every slice.
    ///
    /// Takes `&ClocksReady` because frequencies are computed from `clk_sys`,
    /// and `&Rp2350Gpio` because an output switches its pin's `IO_BANK0` and
    /// `PADS_BANK0` registers, which only work once `Rp2350Gpio::new` has
    /// released those blocks.
    ///
    /// # Errors
    ///
    /// [`PwmError::ResetTimeout`] if the block never reports out of reset.
    #[cfg(target_os = "none")]
    pub fn new(
        _handle: DeviceHandle<Self>,
        clocks: &ClocksReady,
        _gpio: &Rp2350Gpio,
    ) -> Result<Self, PwmError> {
        release(&mut crate::common::reg::Mmio).map(|()| Self {
            clk_sys_hz: clocks.clk_sys_hz(),
            claimed: 0,
        })
    }

    /// Make pin `N` a PWM output at [`DEFAULT_HZ`](super::DEFAULT_HZ),
    /// disabled (low), consuming its handle.
    ///
    /// # Errors
    ///
    /// [`PwmError::SliceInUse`] if the other channel of the slice is already
    /// an output.
    #[cfg(target_os = "none")]
    pub fn output_from_handle<const N: usize>(
        &mut self,
        _pin: PinHandle<N>,
    ) -> Result<Rp2350PwmOut<N>, PwmError> {
        let mmio = &mut crate::common::reg::Mmio;
        let slice = Rp2350PwmOut::<N>::SLICE;
        attach(mmio, self.claimed, self.clk_sys_hz, N, slice).map(|(claimed, t)| {
            self.claimed = claimed;
            Rp2350PwmOut {
                clk_sys_hz: self.clk_sys_hz,
                period: t.period,
                duty_permille: 0,
                on: false,
            }
        })
    }
}

/// Release PWM from reset and clear every slice enable.
pub(crate) fn release<R: Regs>(regs: &mut R) -> Result<(), PwmError> {
    regs.write(RegAddr::RESET, RESET_OFFSET + ALIAS_CLR, RESET_BIT_PWM);
    poll(
        regs,
        RegAddr::RESET,
        RESET_DONE_OFFSET,
        RESET_BIT_PWM,
        RESET_BIT_PWM,
        RESET_POLL_BUDGET,
    )
    .map_err(|_| PwmError::ResetTimeout)?;
    regs.write(RegAddr::PWM, EN, 0);
    Ok(())
}

/// Configure `slice` for `pin` and switch the pin to it; returns the new
/// claimed mask and the timing. Refuses a claimed slice before any write.
pub(crate) fn attach<R: Regs>(
    regs: &mut R,
    claimed: u8,
    clk_hz: u32,
    pin: usize,
    slice: usize,
) -> Result<(u8, Timing), PwmError> {
    let bit = 1u8 << slice;
    if claimed & bit != 0 {
        return Err(PwmError::SliceInUse);
    }
    let t = timing(clk_hz, DEFAULT_HZ)?;
    let base = SLICE_STRIDE * slice;
    regs.write(RegAddr::PWM, base + CSR, 0);
    regs.write(RegAddr::PWM, base + DIV, t.div16);
    regs.write(RegAddr::PWM, base + TOP, t.period - 1);
    regs.write(RegAddr::PWM, base + CC, 0);
    regs.write(RegAddr::PWM, base + CTR, 0);
    regs.write(RegAddr::PWM, base + CSR, CSR_EN);
    regs.write(RegAddr::IO_BANK0, gpio_ctrl(pin), FUNCSEL_PWM);
    regs.write(RegAddr::PADS_BANK0, pad_ctrl(pin) + ALIAS_CLR, PAD_ISO_OD);
    Ok((claimed | bit, t))
}

/// Write the slice's compare value for the channel's duty.
pub(crate) fn write_cc<R: Regs>(regs: &mut R, slice: usize, channel: usize, cc: u32) {
    regs.write(
        RegAddr::PWM,
        SLICE_STRIDE * slice + CC,
        cc << (16 * channel),
    );
}

/// Apply a new timing: divider, wrap, and the compare value rescaled to it.
pub(crate) fn write_timing<R: Regs>(
    regs: &mut R,
    slice: usize,
    channel: usize,
    t: &Timing,
    cc: u32,
) {
    let base = SLICE_STRIDE * slice;
    regs.write(RegAddr::PWM, base + DIV, t.div16);
    regs.write(RegAddr::PWM, base + TOP, t.period - 1);
    write_cc(regs, slice, channel, cc);
}

impl<const N: usize> Rp2350PwmOut<N> {
    /// Frequency change against any [`Regs`]; state updated only on success (PWM-2, PWM-3).
    pub(crate) fn frequency_on<R: Regs>(&mut self, regs: &mut R, hz: u32) -> Result<u64, PwmError> {
        let t = timing(self.clk_sys_hz, hz)?;
        write_timing(
            regs,
            Self::SLICE,
            Self::CHANNEL,
            &t,
            cc_value(self.duty_permille, t.period, self.on),
        );
        self.period = t.period;
        Ok(t.achieved_mhz)
    }

    /// Duty or on/off change against any [`Regs`] (PWM-4, PWM-5).
    pub(crate) fn update_on<R: Regs>(&mut self, regs: &mut R, permille: u16, on: bool) {
        self.duty_permille = permille;
        self.on = on;
        write_cc(
            regs,
            Self::SLICE,
            Self::CHANNEL,
            cc_value(permille, self.period, on),
        );
    }
}

impl ErrorType for Rp2350Pwm {
    type Error = PwmError;
}

impl<const N: usize> ErrorType for Rp2350PwmOut<N> {
    type Error = PwmError;
}

#[cfg(target_os = "none")]
impl<const N: usize> PwmOutput for Rp2350PwmOut<N> {
    fn set_frequency_hz(&mut self, hz: u32) -> Result<u64, PwmError> {
        self.frequency_on(&mut crate::common::reg::Mmio, hz)
    }
    fn set_duty(&mut self, duty: Duty) -> Result<(), PwmError> {
        let on = self.on;
        self.update_on(&mut crate::common::reg::Mmio, duty.permille(), on);
        Ok(())
    }
    fn set_enabled(&mut self, on: bool) -> Result<(), PwmError> {
        let permille = self.duty_permille;
        self.update_on(&mut crate::common::reg::Mmio, permille, on);
        Ok(())
    }
}

#[cfg(test)]
#[path = "pwm_tests.rs"]
mod tests;
