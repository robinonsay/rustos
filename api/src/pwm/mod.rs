//! Portable pulse-width modulation (PWM) traits.
//!
//! A PWM output is a pin that repeats one fixed-length *period* forever,
//! high for the first part of each period and low for the rest. The
//! fraction of the period spent high is the *duty cycle*. Averaged over
//! time, the pin delivers that fraction of its supply, which is how PWM
//! dims an LED, sets a motor's speed or a servo's angle, and — at 50 % —
//! makes a square-wave tone.
//!
//! Hardware counts the period in steps of some internal clock, so a duty
//! cycle can only be a whole number of those steps. This module therefore
//! works in **steps**: [`PwmPin::max_duty`] is the number of steps in one
//! period, and a duty of `d` means "high for `d` of them". `0` is always
//! low and `max_duty()` is always high. [`PwmPin::set_duty_fraction`] and
//! [`PwmPin::set_duty_percent`] convert from fractions for callers that do
//! not care about the step size.
//!
//! The path to a working output mirrors [`crate::gpio`]: the PWM driver's
//! constructor consumes its [`DeviceHandle`](crate::device::DeviceHandle),
//! then [`Pwm::output_from_handle`] consumes a pin's [`PinHandle`] and
//! returns a [`PwmPin`] typed by the pin number.
//!
//! ```ignore
//! use api::pwm::{Pwm, PwmPin};
//!
//! let mut pwm = Rp2350Pwm::new(board.pwm, &clocks);
//! let mut led = pwm.output_from_handle(board.pins.gpio18, 1_000)?; // 1 kHz, starts off
//! led.set_duty_percent(25)?;                                       // quarter brightness
//! led.set_off()?;
//! ```
//!
//! Nothing here assumes a particular chip: an implementation might be a
//! microcontroller's timer block, a dedicated PWM controller on an I2C bus,
//! or a sysfs file on a Linux board.

use crate::{common::ErrorType, device::PinHandle};

/// A PWM output on pin `N`, running at a fixed frequency.
///
/// Changes of duty take effect at the implementation's next safe point.
/// Implementations should change the output only between whole periods,
/// so that a change never produces a runt pulse; the RP2350 driver does.
pub trait PwmPin<const N: usize>: ErrorType {
    /// The frequency actually produced, in hertz, which may differ from
    /// the one requested by the rounding the hardware needs.
    fn frequency_hz(&self) -> u32;

    /// Steps in one period: the largest valid duty, meaning "always
    /// high". At least 1.
    fn max_duty(&self) -> u32;

    /// The duty last set, in steps. A new output starts at 0.
    fn duty(&self) -> u32;

    /// Set the duty to `duty` steps out of [`max_duty`](Self::max_duty).
    /// A value above `max_duty()` is an error and changes nothing.
    fn set_duty(&mut self, duty: u32) -> Result<(), Self::Error>;

    /// Hold the pin low: duty 0.
    fn set_off(&mut self) -> Result<(), Self::Error> {
        // 0 steps high is always low.
        self.set_duty(0)
    }

    /// Set the duty to `numerator / denominator` of the period, rounded
    /// down to a whole step. A fraction above 1 is treated as 1 (always
    /// high), and a zero `denominator` as 0 (always low), so this never
    /// fails for being out of range.
    fn set_duty_fraction(&mut self, numerator: u32, denominator: u32) -> Result<(), Self::Error> {
        // A zero denominator has no meaningful fraction: off is the safe reading.
        if denominator == 0 {
            return self.set_off();
        }
        // Clamp to a whole period.
        let numerator = numerator.min(denominator);
        // In u64 so max_duty × numerator cannot overflow.
        let steps = self.max_duty() as u64 * numerator as u64 / denominator as u64;
        // At most max_duty(), so it fits a u32.
        self.set_duty(steps as u32)
    }

    /// Set the duty to `percent` % of the period, rounded down to a whole
    /// step. Values above 100 are treated as 100.
    fn set_duty_percent(&mut self, percent: u8) -> Result<(), Self::Error> {
        // A percentage is a fraction over 100.
        self.set_duty_fraction(percent as u32, 100)
    }
}

/// The factory half of a PWM driver: turns a pin into a [`PwmPin`].
pub trait Pwm: ErrorType {
    /// The output type this driver returns for pin `N`.
    type Output<const N: usize>: PwmPin<N>;

    /// Connect pin `N` to a PWM output running at `frequency_hz`,
    /// consuming its handle. The output starts at duty 0, held low, so
    /// connecting a pin never produces a pulse. Errors are the
    /// implementation's: a frequency it cannot make, a pin with no PWM
    /// output, or an output already in use.
    fn output_from_handle<const N: usize>(
        &mut self,
        handle: PinHandle<N>,
        frequency_hz: u32,
    ) -> Result<Self::Output<N>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;

    /// A PWM pin that records the duty it was given.
    struct Mock {
        /// Steps per period.
        max: u32,
        /// Duty last set.
        duty: u32,
    }

    impl ErrorType for Mock {
        type Error = Infallible;
    }

    impl PwmPin<0> for Mock {
        fn frequency_hz(&self) -> u32 {
            1_000
        }
        fn max_duty(&self) -> u32 {
            self.max
        }
        fn duty(&self) -> u32 {
            self.duty
        }
        fn set_duty(&mut self, duty: u32) -> Result<(), Infallible> {
            // The mock never sees an out-of-range value from the defaults.
            assert!(duty <= self.max);
            self.duty = duty;
            Ok(())
        }
    }

    fn mock(max: u32) -> Mock {
        Mock { max, duty: 0 }
    }

    #[test]
    fn percent_rounds_down_to_whole_steps() {
        let mut p = mock(1000);
        p.set_duty_percent(25).unwrap();
        assert_eq!(p.duty(), 250);
        // 1/3 of 10 steps is 3.33: rounds down to 3.
        let mut p = mock(10);
        p.set_duty_fraction(1, 3).unwrap();
        assert_eq!(p.duty(), 3);
    }

    #[test]
    fn full_and_off_are_exact() {
        let mut p = mock(65_535);
        p.set_duty_percent(100).unwrap();
        assert_eq!(p.duty(), 65_535);
        p.set_off().unwrap();
        assert_eq!(p.duty(), 0);
    }

    #[test]
    fn out_of_range_fractions_are_clamped() {
        let mut p = mock(100);
        p.set_duty_percent(250).unwrap();
        assert_eq!(p.duty(), 100);
        p.set_duty_fraction(7, 0).unwrap();
        assert_eq!(p.duty(), 0);
    }

    #[test]
    fn large_step_counts_do_not_overflow() {
        let mut p = mock(u32::MAX);
        p.set_duty_fraction(u32::MAX - 1, u32::MAX).unwrap();
        assert_eq!(p.duty(), u32::MAX - 1);
    }
}
