//! Portable pulse-width-modulated output.
//!
//! A PWM output is a square wave whose frequency and duty cycle (the fraction
//! of each period spent high) software sets. cwht uses one for the keyer
//! sidetone and one for the display backlight.
//!
//! # Contract
//!
//! Every implementation satisfies these clauses; `api/tests/pwm_contract.rs`
//! states each as a check that runs against any implementation a test can
//! observe.
//!
//! * **PWM-1** `set_frequency_hz(f)` returning `Ok(m)` reports the achieved
//!   frequency `m` in millihertz, and `|m - 1000 f| <= f` (within 0.1 %).
//! * **PWM-2** A frequency the implementation cannot produce, including 0, is
//!   rejected with `Err`, and the previous frequency and duty stay in force.
//! * **PWM-3** The duty set by `set_duty` survives later frequency changes.
//! * **PWM-4** `set_enabled(false)` holds the output low from the end of the
//!   current period; `set_enabled(true)` restores the set duty.
//! * **PWM-5** Duty 0 is a constant low and duty 1000 ‰ a constant high,
//!   with no pulses.
//! * **PWM-6** A newly constructed output is disabled (low).

use crate::common::ErrorType;

/// Duty cycle in parts per thousand of the period, 0 to 1000.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Duty {
    permille: u16,
}

impl Duty {
    /// 0 %: constant low.
    pub const OFF: Self = Self { permille: 0 };
    /// 50 %: a square wave.
    pub const HALF: Self = Self { permille: 500 };
    /// 100 %: constant high.
    pub const FULL: Self = Self { permille: 1000 };

    /// The duty `permille` / 1000, or `None` above 1000.
    #[must_use]
    pub const fn from_permille(permille: u16) -> Option<Self> {
        if permille > 1000 {
            return None;
        }
        Some(Self { permille })
    }

    /// Parts per thousand, 0 to 1000.
    #[must_use]
    pub const fn permille(self) -> u16 {
        self.permille
    }
}

/// One PWM output (PWM-1 to PWM-6).
pub trait PwmOutput: ErrorType {
    /// Set the frequency to `hz`, keeping the duty; returns the achieved
    /// frequency in millihertz (PWM-1, PWM-3).
    ///
    /// # Errors
    ///
    /// `hz` is outside what the implementation can produce (PWM-2).
    fn set_frequency_hz(&mut self, hz: u32) -> Result<u64, Self::Error>;

    /// Set the duty, effective from the next period.
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot reach its hardware.
    fn set_duty(&mut self, duty: Duty) -> Result<(), Self::Error>;

    /// Turn the output on (the set duty) or off (held low) from the next
    /// period (PWM-4).
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot reach its hardware.
    fn set_enabled(&mut self, on: bool) -> Result<(), Self::Error>;
}
