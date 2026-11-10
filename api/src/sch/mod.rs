//! Schedulers: what decides when each [`App`](crate::app::App) runs.
//!
//! [`Scheduler`] is the interface — start every app, run the schedule,
//! exit — and [`TdmSch`] is the time-division-multiplexed cyclic executive
//! that implements it, the Rust form of LibJuno's `SCH_API_T` /
//! `SCH_ROOT_T` (`juno/sch/juno_sch_api.hpp`).
//!
//! A scheduler never stops for a failure. A failing app hook, a minor frame
//! that overran its end, or a sleep that could not wait is reported to the
//! scheduler's [`SchLog`] and counted in the [`Report`] the call returns,
//! and the schedule carries on: one app's failure never stops the others.

pub mod tdm;

pub use tdm::{SchConfigError, TdmSch, TdmSchBuilder};

use crate::app::Hook;
use crate::error::Error;
use crate::time::Duration;

/// Something a scheduler logs. Borrowed for the duration of one
/// [`SchLog::log`] call: copy out what you need to keep.
#[derive(Clone, Copy, Debug)]
pub enum SchEvent<'e> {
    /// An app's hook returned an error. The schedule continued.
    AppFailed {
        /// The app's index in the scheduler's app array.
        app: usize,
        /// Which hook failed.
        hook: Hook,
        /// The app's own error, with all its detail.
        error: &'e dyn Error,
    },
    /// A minor frame's apps finished after the frame should have ended,
    /// so the next frame starts late. The schedule keeps its absolute
    /// deadlines, so later frames catch up if they have slack.
    Overrun {
        /// The minor frame (row of the schedule table) that ran long.
        minor_frame: usize,
        /// How far past the frame's end its apps finished.
        late_by: Duration,
    },
    /// The clock could not wait for a frame boundary. The next frame
    /// starts at once.
    SleepFailed {
        /// The minor frame after which the sleep was due.
        minor_frame: usize,
        /// The clock's error.
        error: &'e dyn Error,
    },
}

/// Where a scheduler sends its [`SchEvent`]s.
///
/// Implement it to forward events to a UART console, a telemetry queue or
/// a fault counter. A closure `FnMut(SchEvent<'_>)` is one, and [`NoLog`]
/// discards everything. Logging runs inside the schedule, so it should be
/// quick: queue the event, do not print it synchronously over a slow link.
pub trait SchLog {
    /// Record one event.
    fn log(&mut self, event: SchEvent<'_>);
}

/// Any `FnMut(SchEvent<'_>)` closure is a log.
impl<F: FnMut(SchEvent<'_>)> SchLog for F {
    fn log(&mut self, event: SchEvent<'_>) {
        // Call the closure.
        self(event)
    }
}

/// A log that discards every event. [`Report`]s still count them.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoLog;

impl SchLog for NoLog {
    fn log(&mut self, _event: SchEvent<'_>) {
        // Deliberately nothing.
    }
}

/// What happened during one scheduler call: the counts of the events
/// logged, so a caller without a log still sees them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// App hooks that returned an error.
    pub app_failures: u32,
    /// Minor frames that overran.
    pub overruns: u32,
    /// Frame-boundary sleeps that failed.
    pub sleep_failures: u32,
}

impl Report {
    /// `true` if nothing went wrong.
    pub const fn is_clean(&self) -> bool {
        // All three counts zero.
        self.app_failures == 0 && self.overruns == 0 && self.sleep_failures == 0
    }

    /// Add `other`'s counts to this one's, saturating.
    pub fn merge(&mut self, other: Report) {
        // Field by field.
        self.app_failures = self.app_failures.saturating_add(other.app_failures);
        self.overruns = self.overruns.saturating_add(other.overruns);
        self.sleep_failures = self.sleep_failures.saturating_add(other.sleep_failures);
    }
}

/// A scheduler of [`App`](crate::app::App)s.
///
/// The lifecycle it guarantees each app: [`start`](Self::start) runs every
/// app's `on_start` once; then [`run_major_frame`](Self::run_major_frame)
/// runs `on_process` on the schedule, as often as it is called; then
/// [`exit`](Self::exit) runs every app's `on_exit` once. Calls out of that
/// order do the least surprising thing and say so in their documentation.
pub trait Scheduler {
    /// The length of one minor frame, the scheduler's base tick.
    fn minor_frame_period(&self) -> Duration;

    /// The length of one major frame, after which the schedule repeats.
    fn major_frame_period(&self) -> Duration;

    /// Run every app's `on_start` once, in app order. Calling it again, or
    /// after [`exit`](Self::exit), does nothing.
    fn start(&mut self) -> Report;

    /// Run one major frame: every minor frame in turn, its apps in
    /// schedule order, each frame paced to its absolute deadline. Calls
    /// [`start`](Self::start) first if it has not run, so every app's
    /// `on_start` comes before its first `on_process`. After
    /// [`exit`](Self::exit), does nothing.
    fn run_major_frame(&mut self) -> Report;

    /// Run every started app's `on_exit` once, in app order. After this the
    /// scheduler runs nothing more. Calling it again does nothing.
    fn exit(&mut self) -> Report;
}
