//! Contract checks for [`api::pwm::PwmOutput`] (clauses PWM-1 to PWM-6).
//!
//! An implementation under test is wrapped in a [`Probe`] that reports what
//! the output is doing: its frequency in millihertz and its effective duty
//! (0 when disabled). The tests run every check against a reference model,
//! and chosen checks against faulty models.

use api::common::ErrorType;
use api::pwm::{Duty, PwmOutput};

type Check = Result<(), &'static str>;

fn ok_or(cond: bool, clause: &'static str) -> Check {
    if cond { Ok(()) } else { Err(clause) }
}

/// What a test can observe of an output.
trait Probe: PwmOutput {
    /// Frequency in millihertz and effective duty in permille.
    fn observe(&self) -> (u64, u16);
}

fn d(p: u16) -> Duty {
    Duty::from_permille(p).unwrap_or(Duty::OFF)
}

/// PWM-6, then PWM-1 over the sidetone and backlight range.
fn check_start_and_frequency<P: Probe>(p: &mut P) -> Check {
    ok_or(p.observe().1 == 0, "PWM-6")?;
    for f in [100u32, 400, 600, 700, 800, 1000, 20_000] {
        let m = p.set_frequency_hz(f).map_err(|_| "PWM-1")?;
        ok_or(m.abs_diff(u64::from(f) * 1000) <= u64::from(f), "PWM-1")?;
        ok_or(p.observe().0 == m, "PWM-1")?;
    }
    Ok(())
}

/// PWM-2: 0 and an absurd frequency are rejected and change nothing.
fn check_rejects<P: Probe>(p: &mut P) -> Check {
    p.set_frequency_hz(700).map_err(|_| "PWM-2")?;
    p.set_duty(d(300)).map_err(|_| "PWM-2")?;
    p.set_enabled(true).map_err(|_| "PWM-2")?;
    let before = p.observe();
    ok_or(p.set_frequency_hz(0).is_err(), "PWM-2")?;
    ok_or(p.set_frequency_hz(u32::MAX).is_err(), "PWM-2")?;
    ok_or(p.observe() == before, "PWM-2")
}

/// PWM-3: duty survives a frequency change.
fn check_duty_kept<P: Probe>(p: &mut P) -> Check {
    p.set_enabled(true).map_err(|_| "PWM-3")?;
    p.set_duty(d(250)).map_err(|_| "PWM-3")?;
    p.set_frequency_hz(600).map_err(|_| "PWM-3")?;
    ok_or(p.observe().1 == 250, "PWM-3")?;
    p.set_frequency_hz(800).map_err(|_| "PWM-3")?;
    ok_or(p.observe().1 == 250, "PWM-3")
}

/// PWM-4 and PWM-5.
fn check_enable_and_edges<P: Probe>(p: &mut P) -> Check {
    p.set_frequency_hz(700).map_err(|_| "PWM-4")?;
    p.set_duty(d(500)).map_err(|_| "PWM-4")?;
    p.set_enabled(true).map_err(|_| "PWM-4")?;
    ok_or(p.observe().1 == 500, "PWM-4")?;
    p.set_enabled(false).map_err(|_| "PWM-4")?;
    ok_or(p.observe().1 == 0, "PWM-4")?;
    p.set_enabled(true).map_err(|_| "PWM-4")?;
    ok_or(p.observe().1 == 500, "PWM-4")?;
    p.set_duty(Duty::OFF).map_err(|_| "PWM-5")?;
    ok_or(p.observe().1 == 0, "PWM-5")?;
    p.set_duty(Duty::FULL).map_err(|_| "PWM-5")?;
    ok_or(p.observe().1 == 1000, "PWM-5")
}

fn check_all<P: Probe>(make: impl Fn() -> P) -> Check {
    check_start_and_frequency(&mut make())?;
    check_rejects(&mut make())?;
    check_duty_kept(&mut make())?;
    check_enable_and_edges(&mut make())
}

// ---- reference model -------------------------------------------------------

#[derive(Debug)]
struct OutOfRange;

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    None,
    /// Frequency change resets the duty to 0.
    ForgetsDuty,
    /// Disabling does nothing.
    DisableIgnored,
    /// Accepts 0 Hz by keeping the old setting and returning Ok.
    AcceptsZero,
}

struct Model {
    mhz: u64,
    duty: u16,
    on: bool,
    fault: Fault,
}

impl Model {
    fn new(fault: Fault) -> Self {
        Self {
            mhz: 1_000_000,
            duty: 0,
            on: false,
            fault,
        }
    }
}

impl ErrorType for Model {
    type Error = OutOfRange;
}

impl PwmOutput for Model {
    fn set_frequency_hz(&mut self, hz: u32) -> Result<u64, OutOfRange> {
        if hz == 0 && self.fault == Fault::AcceptsZero {
            return Ok(self.mhz);
        }
        if hz == 0 || hz > 1_000_000 {
            return Err(OutOfRange);
        }
        self.mhz = u64::from(hz) * 1000;
        if self.fault == Fault::ForgetsDuty {
            self.duty = 0;
        }
        Ok(self.mhz)
    }
    fn set_duty(&mut self, duty: Duty) -> Result<(), OutOfRange> {
        self.duty = duty.permille();
        Ok(())
    }
    fn set_enabled(&mut self, on: bool) -> Result<(), OutOfRange> {
        if self.fault != Fault::DisableIgnored || on {
            self.on = on;
        }
        Ok(())
    }
}

impl Probe for Model {
    fn observe(&self) -> (u64, u16) {
        (self.mhz, if self.on { self.duty } else { 0 })
    }
}

#[test]
fn reference_model_satisfies_the_contract() {
    assert_eq!(check_all(|| Model::new(Fault::None)), Ok(()));
}

#[test]
fn forgotten_duty_is_detected_as_pwm_3() {
    assert_eq!(
        check_duty_kept(&mut Model::new(Fault::ForgetsDuty)),
        Err("PWM-3")
    );
}

#[test]
fn ignored_disable_is_detected_as_pwm_4() {
    assert_eq!(
        check_enable_and_edges(&mut Model::new(Fault::DisableIgnored)),
        Err("PWM-4")
    );
}

#[test]
fn accepted_zero_is_detected_as_pwm_2() {
    assert_eq!(
        check_rejects(&mut Model::new(Fault::AcceptsZero)),
        Err("PWM-2")
    );
}

#[test]
fn duty_limits() {
    assert_eq!(Duty::from_permille(1000), Some(Duty::FULL));
    assert_eq!(Duty::from_permille(1001), None);
    assert_eq!(Duty::HALF.permille(), 500);
    assert_eq!(Duty::default(), Duty::OFF);
}
