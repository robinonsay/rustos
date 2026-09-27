//! Portable time: a monotonic microsecond clock and one-shot alarms.
//!
//! [`Instant`] and [`Duration`] count whole microseconds in a `u64`, which at
//! one count per microsecond wraps after more than 584 000 years: code written
//! against these types never handles wrap-around. [`Clock`] reads the current
//! instant; [`Alarm`] requests one notification when the clock reaches a
//! chosen instant. Neither trait knows about registers or interrupt lines.
//!
//! # Contract
//!
//! Every implementation satisfies these clauses; `api/tests/time_contract.rs`
//! states each as a check that runs against any implementation.
//!
//! * **CLK-1** Successive `now()` results never decrease.
//! * **CLK-2** `now()` advances by the elapsed time in microseconds, to the
//!   implementation's stated accuracy.
//! * **ALM-1** `schedule_at(at)` with `at` not later than the clock at the
//!   time of the call returns `Ok(Scheduled::Due)`; the alarm is left not
//!   armed and no fire is latched by the call.
//! * **ALM-2** `schedule_at(at)` with `at` later than the clock returns
//!   `Ok(Scheduled::Armed)`, and exactly one fire is latched when the clock
//!   reaches `at`, unless the alarm is cancelled or rescheduled first. While
//!   armed and not yet fired, `is_armed()` is `true`.
//! * **ALM-3** After a fire, `is_armed()` is `false`, and `take_fired()`
//!   returns `true` exactly once, then `false` until the next fire.
//! * **ALM-4** `cancel()` leaves the alarm not armed and discards a latched
//!   fire that was not yet taken.
//! * **ALM-5** `schedule_at` on an armed alarm replaces its target: no fire is
//!   latched for the old target.
//! * **ALM-6** An `at` further ahead than the implementation can arm is
//!   rejected with `Err`, and the alarm is left not armed with no latched fire.
//!
//! A fire is *latched*, not delivered: how the application learns of it (an
//! interrupt handler that calls `take_fired`, or polling) is outside this
//! trait.

use crate::common::ErrorType;

/// A point in time: microseconds since an implementation-defined epoch
/// (on the RP2350, the release of TIMER0 from reset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    micros: u64,
}

/// A span of time in microseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration {
    micros: u64,
}

impl Instant {
    /// The instant `micros` microseconds after the epoch.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self { micros }
    }

    /// Microseconds since the epoch.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.micros
    }

    /// `self + d`, or `None` if that is not representable.
    #[must_use]
    pub const fn checked_add(self, d: Duration) -> Option<Self> {
        match self.micros.checked_add(d.micros) {
            Some(micros) => Some(Self { micros }),
            None => None,
        }
    }

    /// `self - earlier`, or `None` if `earlier` is later than `self`.
    #[must_use]
    pub const fn checked_duration_since(self, earlier: Self) -> Option<Duration> {
        match self.micros.checked_sub(earlier.micros) {
            Some(micros) => Some(Duration { micros }),
            None => None,
        }
    }
}

impl Duration {
    /// A span of `micros` microseconds.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self { micros }
    }

    /// A span of `millis` milliseconds.
    #[must_use]
    pub fn from_millis(millis: u32) -> Self {
        // u32::MAX × 1000 < u64::MAX, so this cannot overflow.
        Self {
            micros: u64::from(millis) * 1000,
        }
    }

    /// The span in microseconds.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.micros
    }
}

/// A monotonic clock (CLK-1, CLK-2).
pub trait Clock: ErrorType {
    /// The current instant.
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot read its time source; an
    /// implementation that always can uses `Infallible`.
    fn now(&mut self) -> Result<Instant, Self::Error>;
}

/// How [`Alarm::schedule_at`] left the alarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheduled {
    /// Armed: a fire will be latched when the clock reaches the target (ALM-2).
    Armed,
    /// The target was not in the future; nothing is armed and nothing
    /// latched. Act on the expiry now (ALM-1).
    Due,
}

/// A one-shot alarm (ALM-1 to ALM-6).
pub trait Alarm: ErrorType {
    /// Arm the alarm for `at`, replacing any earlier target.
    ///
    /// # Errors
    ///
    /// `at` is further ahead than the implementation can arm (ALM-6), or the
    /// implementation detected that the hardware did not arm.
    fn schedule_at(&mut self, at: Instant) -> Result<Scheduled, Self::Error>;

    /// Disarm and discard any latched fire not yet taken (ALM-4).
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot reach its hardware.
    fn cancel(&mut self) -> Result<(), Self::Error>;

    /// Whether the alarm is armed and has not yet fired.
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot reach its hardware.
    fn is_armed(&mut self) -> Result<bool, Self::Error>;

    /// Clear the latched fire, returning whether there was one (ALM-3).
    ///
    /// # Errors
    ///
    /// Only if the implementation cannot reach its hardware.
    fn take_fired(&mut self) -> Result<bool, Self::Error>;
}
