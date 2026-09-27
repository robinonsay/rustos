//! Contract checks for [`api::time::Clock`] and [`api::time::Alarm`]
//! (clauses CLK-1, CLK-2 and ALM-1 to ALM-6 of the module documentation).
//!
//! Each `check_*` function states one clause against any implementation that
//! a test can drive: a clock, an alarm on the same time base, and a way to let
//! time pass. The tests run every check against a reference model, and chosen
//! checks against faulty models to show that each check detects its clause.

use std::cell::Cell;
use std::rc::Rc;

use api::common::ErrorType;
use api::time::{Alarm, Clock, Duration, Instant, Scheduled};

type Check = Result<(), &'static str>;

fn ok_or(cond: bool, clause: &'static str) -> Check {
    if cond { Ok(()) } else { Err(clause) }
}

/// A clock, an alarm and a time source a test can advance.
trait Rig {
    type C: Clock;
    type A: Alarm;
    fn parts(&mut self) -> (&mut Self::C, &mut Self::A);
    fn advance(&mut self, micros: u64);
    /// Largest `at - now` the alarm can arm.
    fn span(&self) -> u64;
}

fn clock_now<R: Rig>(r: &mut R) -> Result<Instant, &'static str> {
    r.parts().0.now().map_err(|_| "CLK-1")
}

fn later(t: Instant, us: u64) -> Instant {
    t.checked_add(Duration::from_micros(us)).unwrap_or(t)
}

/// CLK-1 and CLK-2.
fn check_clock<R: Rig>(r: &mut R) -> Check {
    let t0 = clock_now(r)?;
    ok_or(clock_now(r)? >= t0, "CLK-1")?;
    r.advance(1234);
    let t1 = clock_now(r)?;
    ok_or(
        t1.checked_duration_since(t0) == Some(Duration::from_micros(1234)),
        "CLK-2",
    )
}

/// ALM-1: now and the past are Due, nothing armed, nothing latched.
fn check_due<R: Rig>(r: &mut R) -> Check {
    r.advance(10_000);
    let t = clock_now(r)?;
    for at in [
        t,
        Instant::from_micros(t.as_micros() - 1),
        Instant::from_micros(0),
    ] {
        let (_, a) = r.parts();
        ok_or(
            a.schedule_at(at).map_err(|_| "ALM-1")? == Scheduled::Due,
            "ALM-1",
        )?;
        ok_or(!a.is_armed().map_err(|_| "ALM-1")?, "ALM-1")?;
        ok_or(!a.take_fired().map_err(|_| "ALM-1")?, "ALM-1")?;
    }
    Ok(())
}

/// ALM-2: armed from the call until the target, and nothing latched before it.
fn check_armed_until_target<R: Rig>(r: &mut R) -> Check {
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    ok_or(
        a.schedule_at(later(t, 500)).map_err(|_| "ALM-2")? == Scheduled::Armed,
        "ALM-2",
    )?;
    ok_or(a.is_armed().map_err(|_| "ALM-2")?, "ALM-2")?;
    r.advance(499);
    let (_, a) = r.parts();
    ok_or(a.is_armed().map_err(|_| "ALM-2")?, "ALM-2")?;
    ok_or(!a.take_fired().map_err(|_| "ALM-2")?, "ALM-2")
}

/// ALM-3: one fire at the target, taken exactly once, nothing after.
fn check_fire<R: Rig>(r: &mut R) -> Check {
    check_armed_until_target(r)?;
    r.advance(1);
    let (_, a) = r.parts();
    ok_or(!a.is_armed().map_err(|_| "ALM-3")?, "ALM-3")?;
    ok_or(a.take_fired().map_err(|_| "ALM-3")?, "ALM-3")?;
    ok_or(!a.take_fired().map_err(|_| "ALM-3")?, "ALM-3")?;
    r.advance(10_000);
    let (_, a) = r.parts();
    ok_or(!a.take_fired().map_err(|_| "ALM-3")?, "ALM-3")
}

/// ALM-4: cancel before the fire, and cancel after an untaken fire.
fn check_cancel<R: Rig>(r: &mut R) -> Check {
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    a.schedule_at(later(t, 100)).map_err(|_| "ALM-4")?;
    a.cancel().map_err(|_| "ALM-4")?;
    ok_or(!a.is_armed().map_err(|_| "ALM-4")?, "ALM-4")?;
    r.advance(200);
    let (_, a) = r.parts();
    ok_or(!a.take_fired().map_err(|_| "ALM-4")?, "ALM-4")?;
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    a.schedule_at(later(t, 100)).map_err(|_| "ALM-4")?;
    r.advance(100);
    let (_, a) = r.parts();
    a.cancel().map_err(|_| "ALM-4")?;
    ok_or(!a.take_fired().map_err(|_| "ALM-4")?, "ALM-4")
}

/// ALM-5, to a later target: no fire at the old one, one at the new one.
fn check_replace_later<R: Rig>(r: &mut R) -> Check {
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    a.schedule_at(later(t, 100)).map_err(|_| "ALM-5")?;
    a.schedule_at(later(t, 300)).map_err(|_| "ALM-5")?;
    r.advance(150);
    let (_, a) = r.parts();
    ok_or(!a.take_fired().map_err(|_| "ALM-5")?, "ALM-5")?;
    ok_or(a.is_armed().map_err(|_| "ALM-5")?, "ALM-5")?;
    r.advance(150);
    let (_, a) = r.parts();
    ok_or(a.take_fired().map_err(|_| "ALM-5")?, "ALM-5")
}

/// ALM-5, to an earlier target: one fire at the new one, none at the old one.
fn check_replace_earlier<R: Rig>(r: &mut R) -> Check {
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    a.schedule_at(later(t, 300)).map_err(|_| "ALM-5")?;
    a.schedule_at(later(t, 100)).map_err(|_| "ALM-5")?;
    r.advance(100);
    let (_, a) = r.parts();
    ok_or(a.take_fired().map_err(|_| "ALM-5")?, "ALM-5")?;
    r.advance(300);
    let (_, a) = r.parts();
    ok_or(!a.take_fired().map_err(|_| "ALM-5")?, "ALM-5")
}

/// ALM-6: the largest span arms; one more microsecond is rejected and leaves
/// nothing armed.
fn check_span<R: Rig>(r: &mut R) -> Check {
    let span = r.span();
    let t = clock_now(r)?;
    let (_, a) = r.parts();
    ok_or(
        a.schedule_at(later(t, span)).map_err(|_| "ALM-6")? == Scheduled::Armed,
        "ALM-6",
    )?;
    ok_or(a.schedule_at(later(t, span + 1)).is_err(), "ALM-6")?;
    ok_or(!a.is_armed().map_err(|_| "ALM-6")?, "ALM-6")?;
    ok_or(!a.take_fired().map_err(|_| "ALM-6")?, "ALM-6")
}

fn check_all<R: Rig>(make: impl Fn() -> R) -> Check {
    check_clock(&mut make())?;
    check_due(&mut make())?;
    check_fire(&mut make())?;
    check_cancel(&mut make())?;
    check_replace_later(&mut make())?;
    check_replace_earlier(&mut make())?;
    check_span(&mut make())
}

// ---- reference model -------------------------------------------------------

#[derive(Debug)]
struct TooFar;

struct ModelClock(Rc<Cell<u64>>);

impl ErrorType for ModelClock {
    type Error = core::convert::Infallible;
}

impl Clock for ModelClock {
    fn now(&mut self) -> Result<Instant, Self::Error> {
        Ok(Instant::from_micros(self.0.get()))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    None,
    /// Fires one microsecond late.
    Late,
    /// `cancel` leaves a latched fire in place.
    CancelKeepsFire,
    /// Arms any span.
    NoSpanLimit,
}

struct ModelAlarm {
    time: Rc<Cell<u64>>,
    target: Option<u64>,
    fired: bool,
    fault: Fault,
}

impl ModelAlarm {
    /// Latch the fire if the clock has reached the target.
    fn update(&mut self) {
        let lag = u64::from(self.fault == Fault::Late);
        if let Some(t) = self.target
            && self.time.get() >= t + lag
        {
            self.target = None;
            self.fired = true;
        }
    }
}

impl ErrorType for ModelAlarm {
    type Error = TooFar;
}

const SPAN: u64 = u32::MAX as u64;

impl Alarm for ModelAlarm {
    fn schedule_at(&mut self, at: Instant) -> Result<Scheduled, TooFar> {
        let now = self.time.get();
        self.target = None;
        self.fired = false;
        let at = at.as_micros();
        if at <= now {
            return Ok(Scheduled::Due);
        }
        if at - now > SPAN && self.fault != Fault::NoSpanLimit {
            return Err(TooFar);
        }
        self.target = Some(at);
        Ok(Scheduled::Armed)
    }
    fn cancel(&mut self) -> Result<(), TooFar> {
        self.update();
        self.target = None;
        if self.fault != Fault::CancelKeepsFire {
            self.fired = false;
        }
        Ok(())
    }
    fn is_armed(&mut self) -> Result<bool, TooFar> {
        self.update();
        Ok(self.target.is_some())
    }
    fn take_fired(&mut self) -> Result<bool, TooFar> {
        self.update();
        Ok(core::mem::replace(&mut self.fired, false))
    }
}

struct Model {
    time: Rc<Cell<u64>>,
    clock: ModelClock,
    alarm: ModelAlarm,
}

impl Model {
    fn new(fault: Fault) -> Self {
        let time = Rc::new(Cell::new(1_000_000));
        Self {
            clock: ModelClock(time.clone()),
            alarm: ModelAlarm {
                time: time.clone(),
                target: None,
                fired: false,
                fault,
            },
            time,
        }
    }
}

impl Rig for Model {
    type C = ModelClock;
    type A = ModelAlarm;
    fn parts(&mut self) -> (&mut ModelClock, &mut ModelAlarm) {
        (&mut self.clock, &mut self.alarm)
    }
    fn advance(&mut self, micros: u64) {
        self.time.set(self.time.get() + micros);
    }
    fn span(&self) -> u64 {
        SPAN
    }
}

#[test]
fn reference_model_satisfies_the_contract() {
    assert_eq!(check_all(|| Model::new(Fault::None)), Ok(()));
}

#[test]
fn late_fire_is_detected_as_alm_3() {
    assert_eq!(check_fire(&mut Model::new(Fault::Late)), Err("ALM-3"));
}

#[test]
fn cancel_that_keeps_the_fire_is_detected_as_alm_4() {
    assert_eq!(
        check_cancel(&mut Model::new(Fault::CancelKeepsFire)),
        Err("ALM-4")
    );
}

#[test]
fn missing_span_limit_is_detected_as_alm_6() {
    assert_eq!(
        check_span(&mut Model::new(Fault::NoSpanLimit)),
        Err("ALM-6")
    );
}

#[test]
fn duration_and_instant_arithmetic() {
    assert_eq!(Duration::from_millis(60).as_micros(), 60_000);
    assert_eq!(
        Duration::from_millis(u32::MAX).as_micros(),
        4_294_967_295_000
    );
    let t = Instant::from_micros(10);
    assert_eq!(
        t.checked_add(Duration::from_micros(5)),
        Some(Instant::from_micros(15))
    );
    assert_eq!(
        Instant::from_micros(u64::MAX).checked_add(Duration::from_micros(1)),
        None
    );
    assert_eq!(
        t.checked_duration_since(Instant::from_micros(4)),
        Some(Duration::from_micros(6))
    );
    assert_eq!(t.checked_duration_since(Instant::from_micros(11)), None);
}
