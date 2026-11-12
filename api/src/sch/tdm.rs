//! [`TdmSch`]: a time-division-multiplexed cyclic executive.
//!
//! Time is cut into **minor frames** of a fixed length, say 5 ms, and
//! `FRAMES` of them make a **major frame**, say 200 × 5 ms = 1 s, after which
//! the schedule repeats. A table with one row per minor frame and `SLOTS`
//! columns says which apps run in that frame, in column order. Each frame
//! runs its row and then sleeps until the frame's end; the next frame
//! starts there. This is LibJuno's `SCH_ROOT_T<NAppsPerFrame, NFrames>`,
//! with three differences:
//!
//! * **Apps are stored once, and the table holds indices.** The table is
//!   `[[u8; SLOTS]; FRAMES]` of indices into an app array of `&mut dyn
//!   DynApp`, where LibJuno's holds `APP_ROOT_T*` pointers. The same app
//!   appears in many rows, and Rust allows only one `&mut` to it; an index
//!   per slot also takes a quarter of a 32-bit pointer's memory (1.6 kB for
//!   8 × 200).
//! * **Registration is by period, and only before the scheduler exists.**
//!   [`TdmSchBuilder::register`] places an app in every row its period and
//!   offset select: row *i* when `i × minor ≡ offset (mod period)`.
//!   [`TdmSchBuilder::build`] consumes the builder, and [`TdmSch`] has no
//!   registration method, so a schedule cannot change once it runs — a
//!   compile error, not a review rule.
//! * **Overruns are logged.** After each frame's apps, the clock is read;
//!   if it is already past the frame's end, [`SchEvent::Overrun`] says by
//!   how much, and which app ran longest. LibJuno only tolerates overruns
//!   silently.
//! * **Each app is timed.** Every `on_process` is timed, and
//!   [`TdmSch::take_timings`] hands back each app's [`AppTiming`]: its
//!   runs, its total and longest run, and the overruns it was the slowest
//!   app in, so the load and the overruns can be put down to apps.
//!
//! What is the same: frames are paced to **absolute** deadlines (start +
//! (i + 1) × minor), so one late frame does not delay the rest; a failing
//! app is reported and the schedule continues; capacity is fixed by the
//! type's const parameters; nothing is allocated.
//!
//! Deadlines also run on **across** major frames: each one starts at the
//! previous one's last deadline, not at "now", so the schedule stays phase
//! locked to its first start for as long as it runs, instead of letting
//! the time between calls accumulate.

use crate::app::{DynApp, Hook};
use crate::error::{Error, ErrorKind};
use crate::sch::{Report, SchEvent, SchLog, Scheduler};
use crate::time::{Duration, Sleep, Time};

/// A table slot with no app.
const EMPTY: u8 = u8::MAX;

/// Why a [`TdmSchBuilder`] refused a configuration. Nothing is changed when
/// one is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchConfigError {
    /// The minor frame is zero long.
    ZeroMinorFrame,
    /// `minor frame × FRAMES` does not fit a [`Duration`].
    MajorFrameTooLong,
    /// The app index is not in the app array.
    NoSuchApp {
        /// The index given.
        app: usize,
        /// How many apps there are.
        apps: usize,
    },
    /// The period is zero, or not a whole number of minor frames.
    PeriodNotMultipleOfMinorFrame,
    /// The period does not divide the major frame evenly, so the app
    /// would not run at a steady rate across the major-frame boundary.
    PeriodDoesNotDivideMajorFrame,
    /// The offset is not a whole number of minor frames, or is not shorter
    /// than the period.
    InvalidOffset,
    /// A minor frame this registration needs has no free slot.
    FrameFull {
        /// The minor frame (row) that is full.
        minor_frame: usize,
    },
    /// The app already runs in a minor frame this registration selects.
    AlreadyScheduled {
        /// The minor frame (row) it is already in.
        minor_frame: usize,
    },
    /// [`TdmSchBuilder::keep_free`]: there is no such minor frame.
    NoSuchFrame {
        /// The minor frame given.
        minor_frame: usize,
    },
    /// [`TdmSchBuilder::keep_free`]: an app already runs in the frame.
    FrameInUse {
        /// The minor frame given.
        minor_frame: usize,
    },
    /// Every frame this registration selects is a catch-up frame
    /// ([`TdmSchBuilder::keep_free`]), so the app would never run.
    OnlyFreeFrames,
}

/// Configuration mistakes classified for [`crate::error`].
impl Error for SchConfigError {
    fn kind(&self) -> ErrorKind {
        match self {
            // No room in a fixed-size table.
            SchConfigError::FrameFull { .. } => ErrorKind::Full,
            // An index or size beyond what fits.
            SchConfigError::NoSuchApp { .. }
            | SchConfigError::NoSuchFrame { .. }
            | SchConfigError::MajorFrameTooLong => ErrorKind::OutOfRange,
            // Everything else is an unacceptable argument.
            _ => ErrorKind::InvalidInput,
        }
    }
}

/// The schedule being built, before it runs. Make one with
/// [`new`](Self::new), place apps with [`register`](Self::register), and
/// turn it into a [`TdmSch`] with [`build`](Self::build).
///
/// Type parameters: `C` the clock that paces frames; `L` the [`SchLog`];
/// `SLOTS` apps per minor frame; `FRAMES` minor frames per major frame;
/// `APPS` the number of apps.
pub struct TdmSchBuilder<'a, C: Sleep, L: SchLog, const SLOTS: usize, const FRAMES: usize, const APPS: usize> {
    /// Paces the frames.
    clock: &'a C,
    /// Receives events once running.
    log: L,
    /// Every app, once each. The table refers to them by index.
    apps: [&'a mut dyn DynApp; APPS],
    /// `table[frame][slot]`: an index into `apps`, or [`EMPTY`].
    table: [[u8; SLOTS]; FRAMES],
    /// One minor frame.
    minor: Duration,
    /// Catch-up frames: see [`keep_free`](Self::keep_free).
    free: [bool; FRAMES],
}

impl<'a, C: Sleep, L: SchLog, const SLOTS: usize, const FRAMES: usize, const APPS: usize>
    TdmSchBuilder<'a, C, L, SLOTS, FRAMES, APPS>
{
    /// The table dimensions and app count fit the representation: at least
    /// one slot and one frame (LibJuno's two `static_assert`s), and every
    /// app index below [`EMPTY`].
    const SHAPE_OK: () = {
        assert!(SLOTS > 0, "a minor frame needs at least one slot");
        assert!(FRAMES > 0, "a major frame needs at least one minor frame");
        assert!(APPS < EMPTY as usize, "at most 254 apps: indices are u8 and 255 marks an empty slot");
        assert!(FRAMES <= u32::MAX as usize, "the frame count must fit a u32");
    };

    /// An empty schedule of `FRAMES` minor frames of `minor_frame` each,
    /// for `apps`, paced by `clock` and logging to `log`.
    pub fn new(
        clock: &'a C,
        minor_frame: Duration,
        apps: [&'a mut dyn DynApp; APPS],
        log: L,
    ) -> Result<Self, SchConfigError> {
        // Evaluate the shape check: a bad SLOTS/FRAMES/APPS fails the build.
        let () = Self::SHAPE_OK;
        // A zero-length frame cannot pace anything.
        if minor_frame == Duration::ZERO {
            return Err(SchConfigError::ZeroMinorFrame);
        }
        // The major frame must be representable.
        if minor_frame.checked_mul(FRAMES as u32).is_none() {
            return Err(SchConfigError::MajorFrameTooLong);
        }
        // Every slot starts empty.
        Ok(Self { clock, log, apps, table: [[EMPTY; SLOTS]; FRAMES], minor: minor_frame, free: [false; FRAMES] })
    }

    /// Make `minor_frame` a *catch-up* frame: apps registered afterwards
    /// skip it, unless they run every frame. The apps that run every frame
    /// are usually the ones that must not miss a beat (a bus that must be
    /// polled each frame, an input timed by when it is looked at) and the
    /// cheapest; everything else stays out, so if earlier frames overran,
    /// the schedule has most of a frame of slack to get back onto its
    /// deadlines before the next major frame starts. Every app still runs
    /// as often as its period says.
    ///
    /// Call it before registering: [`SchConfigError::FrameInUse`] if an app
    /// is already in the frame.
    pub fn keep_free(&mut self, minor_frame: usize) -> Result<(), SchConfigError> {
        // The frame must exist and be empty.
        let row = self.table.get(minor_frame).ok_or(SchConfigError::NoSuchFrame { minor_frame })?;
        if row.iter().any(|&s| s != EMPTY) {
            return Err(SchConfigError::FrameInUse { minor_frame });
        }
        self.free[minor_frame] = true;
        Ok(())
    }

    /// Run app number `app` every `period`, starting `offset` into the
    /// major frame: in minor frame *i* exactly when `i × minor = offset +
    /// k × period` for some whole *k*. Within a frame, apps run in the order
    /// they were registered, each taking the first free slot, so the
    /// order is the same on every pass.
    ///
    /// Both `period` and `offset` must be whole numbers of minor frames,
    /// `period` must divide the major frame evenly, and `offset` must be
    /// shorter than `period`. Frames kept free
    /// ([`keep_free`](Self::keep_free)) are skipped. All of the frames it
    /// selects are checked before any is written, so an error leaves the
    /// table as it was.
    pub fn register(&mut self, app: usize, period: Duration, offset: Duration) -> Result<(), SchConfigError> {
        // The app must exist.
        if app >= APPS {
            return Err(SchConfigError::NoSuchApp { app, apps: APPS });
        }
        // Frames between runs: a whole, non-zero number of minor frames.
        let stride = match whole_frames(period, self.minor) {
            Some(n) if n > 0 => n,
            _ => return Err(SchConfigError::PeriodNotMultipleOfMinorFrame),
        };
        // The same frames in every major frame: stride divides FRAMES.
        if stride > FRAMES as u64 || !(FRAMES as u64).is_multiple_of(stride) {
            return Err(SchConfigError::PeriodDoesNotDivideMajorFrame);
        }
        // The first frame: a whole number of frames, inside one period.
        let first = match whole_frames(offset, self.minor) {
            Some(n) if n < stride => n,
            _ => return Err(SchConfigError::InvalidOffset),
        };
        // Both are below FRAMES, so they fit a usize.
        let (first, stride) = (first as usize, stride as usize);
        // The selected rows, less catch-up frames for an app that does not
        // run every frame; at least one.
        let free = self.free;
        let rows = (first..FRAMES).step_by(stride).filter(move |&f| stride == 1 || !free[f]);
        if rows.clone().next().is_none() {
            return Err(SchConfigError::OnlyFreeFrames);
        }
        // Check every selected row before writing any.
        for frame in rows.clone() {
            // Running twice in one frame is almost certainly a mistake.
            if self.table[frame].contains(&(app as u8)) {
                return Err(SchConfigError::AlreadyScheduled { minor_frame: frame });
            }
            // There must be a free slot.
            if !self.table[frame].contains(&EMPTY) {
                return Err(SchConfigError::FrameFull { minor_frame: frame });
            }
        }
        // All rows have room: take the first free slot of each.
        for frame in rows {
            // Checked above, so a free slot exists.
            if let Some(slot) = self.table[frame].iter_mut().find(|s| **s == EMPTY) {
                *slot = app as u8;
            }
        }
        Ok(())
    }

    /// The app in `slot` of `minor_frame`, if any. For inspecting a
    /// schedule before it runs.
    pub fn app_at(&self, minor_frame: usize, slot: usize) -> Option<usize> {
        // Out-of-range coordinates simply have no app.
        app_at(&self.table, minor_frame, slot)
    }

    /// Freeze the schedule. The result cannot be registered into.
    pub fn build(self) -> TdmSch<'a, C, L, SLOTS, FRAMES, APPS> {
        // Same parts, plus the run state.
        TdmSch {
            clock: self.clock,
            log: self.log,
            apps: self.apps,
            table: self.table,
            minor: self.minor,
            next: None,
            state: State::Running,
            timings: [AppTiming::default(); APPS],
        }
    }
}

/// `span` as a whole number of `minor` frames, or `None` if it is not one.
///
/// Not an exact division: a [`Duration`] counts units of 2^-32 s, and a
/// decimal span is generally not a whole number of those — 5 ms is
/// 21 474 836.48 units, stored as 21 474 836 — so `25 ms` is not exactly
/// `5 × 5 ms` (107 374 182 units against 107 374 180). Each conversion
/// truncates by less than one unit, so for `n` frames the true multiple
/// and `span` differ by less than `n + 1` units; anything closer than that
/// counts as `n` frames. A genuine mismatch is far larger: 7 ms against
/// 5 ms frames is off by millions of units.
fn whole_frames(span: Duration, minor: Duration) -> Option<u64> {
    // u128 so n × minor cannot overflow.
    let (span, minor) = (span.to_fixed() as u128, minor.to_fixed() as u128);
    // Nearest whole number of frames.
    let n = (span + minor / 2) / minor;
    // Accept only a difference the conversions themselves could explain.
    if span.abs_diff(n * minor) <= n + 1 { Some(n as u64) } else { None }
}

/// The table lookup shared by the builder and the scheduler.
fn app_at<const SLOTS: usize, const FRAMES: usize>(
    table: &[[u8; SLOTS]; FRAMES],
    minor_frame: usize,
    slot: usize,
) -> Option<usize> {
    // `get` turns out-of-range coordinates into None.
    match table.get(minor_frame).and_then(|row| row.get(slot)) {
        Some(&index) if index != EMPTY => Some(index as usize),
        _ => None,
    }
}

/// Where a [`TdmSch`] is in the app lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Built: its apps were ready to run once constructed.
    Running,
    /// Every app's `on_exit` has run.
    Exited,
}

/// A running time-division-multiplexed schedule. Made by
/// [`TdmSchBuilder::build`]; driven through [`Scheduler`]. See the
/// [module documentation](self).
pub struct TdmSch<'a, C: Sleep, L: SchLog, const SLOTS: usize, const FRAMES: usize, const APPS: usize> {
    /// Paces the frames.
    clock: &'a C,
    /// Receives events.
    log: L,
    /// Every app, once each.
    apps: [&'a mut dyn DynApp; APPS],
    /// `table[frame][slot]`: an index into `apps`, or [`EMPTY`].
    table: [[u8; SLOTS]; FRAMES],
    /// One minor frame.
    minor: Duration,
    /// The boundary the next major frame starts at; `None` before the
    /// first, which starts at the clock's current time.
    next: Option<Time<C::ThisTime>>,
    /// Lifecycle position.
    state: State,
    /// Each app's `on_process` timing since the last
    /// [`take_timings`](Self::take_timings).
    timings: [AppTiming; APPS],
}

/// How one app's `on_process` ran over some span of the schedule: see
/// [`TdmSch::take_timings`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppTiming {
    /// Times it ran.
    pub runs: u32,
    /// Its runs' durations, summed.
    pub busy: Duration,
    /// Its longest run.
    pub longest: Duration,
    /// Minor frames that overran with it as their slowest app.
    pub overruns: u32,
}

impl<'a, C: Sleep, L: SchLog, const SLOTS: usize, const FRAMES: usize, const APPS: usize>
    TdmSch<'a, C, L, SLOTS, FRAMES, APPS>
{
    /// Each app's timing, by its index in the app array, since the last
    /// call (or since the schedule started), and start counting afresh.
    /// Call it between major frames, for example after each
    /// [`run_major_frame`](Scheduler::run_major_frame), for per-second
    /// figures.
    pub fn take_timings(&mut self) -> [AppTiming; APPS] {
        // Hand back the totals and zero them.
        core::mem::replace(&mut self.timings, [AppTiming::default(); APPS])
    }

    /// The app in `slot` of `minor_frame`, if any.
    pub fn app_at(&self, minor_frame: usize, slot: usize) -> Option<usize> {
        // Same lookup as the builder's.
        app_at(&self.table, minor_frame, slot)
    }

    /// The log, for reading back what it collected.
    pub fn log(&self) -> &L {
        // Shared borrow of the field.
        &self.log
    }

    /// Run `hook` on every app, in app order, logging and counting
    /// failures.
    fn every_app(&mut self, hook: Hook) -> Report {
        // Counts for this pass.
        let mut report = Report::default();
        // Disjoint borrows of the two fields the loop needs.
        let (apps, log) = (&mut self.apps, &mut self.log);
        for (index, app) in apps.iter_mut().enumerate() {
            // Any error goes to the log, tagged with the app and hook.
            let ok = app.run_hook(hook, &mut |error| log.log(SchEvent::AppFailed { app: index, hook, error }));
            if !ok {
                report.app_failures = report.app_failures.saturating_add(1);
            }
        }
        report
    }
}

impl<'a, C: Sleep, L: SchLog, const SLOTS: usize, const FRAMES: usize, const APPS: usize> Scheduler
    for TdmSch<'a, C, L, SLOTS, FRAMES, APPS>
{
    fn minor_frame_period(&self) -> Duration {
        // As configured.
        self.minor
    }

    fn major_frame_period(&self) -> Duration {
        // Checked in `new` to fit.
        self.minor * FRAMES as u32
    }

    fn run_major_frame(&mut self) -> Report {
        // Nothing runs after exit.
        if self.state == State::Exited {
            return Report::default();
        }
        // This frame's counts.
        let mut report = Report::default();
        // This major frame begins where the last one ended, or now.
        let mut deadline = match self.next {
            Some(boundary) => boundary,
            None => self.clock.now(),
        };
        for frame in 0..FRAMES {
            // Busy from here, or from the frame's start if it began late.
            let began = self.clock.now();
            // Disjoint borrows for the dispatch loop.
            let (apps, log, row) = (&mut self.apps, &mut self.log, &self.table[frame]);
            // The frame's slowest app so far, and how long it took.
            let mut slowest: Option<(usize, Duration)> = None;
            let mut mark = began;
            // Every occupied slot, in column order.
            for &index in row.iter().filter(|&&i| i != EMPTY) {
                let app = index as usize;
                // Failures are logged and counted; the row carries on.
                let ok = apps[app].run_hook(Hook::Process, &mut |error| {
                    log.log(SchEvent::AppFailed { app, hook: Hook::Process, error })
                });
                if !ok {
                    report.app_failures = report.app_failures.saturating_add(1);
                }
                // Its time, from the end of the one before.
                let now = self.clock.now();
                let took = now.checked_sub(mark).unwrap_or(Duration::ZERO);
                mark = now;
                let t = &mut self.timings[app];
                t.runs = t.runs.saturating_add(1);
                t.busy = t.busy.checked_add(took).unwrap_or(Duration::MAX);
                t.longest = t.longest.max(took);
                if slowest.is_none_or(|(_, longest)| took > longest) {
                    slowest = Some((app, took));
                }
            }
            // How long its apps ran.
            let finished = mark;
            let ran = finished.checked_sub(began).unwrap_or(Duration::ZERO);
            report.busy = report.busy.checked_add(ran).unwrap_or(Duration::MAX);
            report.busiest_frame = report.busiest_frame.max(ran);
            // This frame ends one minor frame after it began.
            deadline = deadline + self.minor;
            // Finished after the end: an overrun, by now − deadline.
            if let Some(late_by) = finished.checked_sub(deadline)
                && late_by > Duration::ZERO
            {
                let slowest_app = slowest.map(|(app, _)| app);
                self.log.log(SchEvent::Overrun { minor_frame: frame, late_by, slowest_app });
                report.overruns = report.overruns.saturating_add(1);
                if let Some(app) = slowest_app {
                    self.timings[app].overruns = self.timings[app].overruns.saturating_add(1);
                }
            }
            // Wait for the boundary (at once if it has passed).
            if let Err(error) = self.clock.sleep_until(deadline) {
                self.log.log(SchEvent::SleepFailed { minor_frame: frame, error: &error });
                report.sleep_failures = report.sleep_failures.saturating_add(1);
            }
        }
        // The next major frame starts at this one's last boundary.
        self.next = Some(deadline);
        report
    }

    fn exit(&mut self) -> Report {
        // Exit is once per app; nothing runs after.
        if self.state == State::Exited {
            return Report::default();
        }
        let report = self.every_app(Hook::Exit);
        self.state = State::Exited;
        report
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::app::App;
    use crate::sch::NoLog;
    use crate::time::{Clock, ClockId};
    use core::cell::{Cell, RefCell};
    use core::convert::Infallible;
    use std::vec::Vec;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Fake;
    impl ClockId for Fake {
        const NAME: &'static str = "FAKE";
    }

    /// A clock that only moves when told to, recording every sleep target.
    struct FakeClock {
        now: Cell<Time<Fake>>,
        sleeps: RefCell<Vec<Time<Fake>>>,
    }

    impl FakeClock {
        fn at(seconds: u32) -> Self {
            Self { now: Cell::new(Time::new(seconds, 0)), sleeps: RefCell::new(Vec::new()) }
        }
        fn advance(&self, by: Duration) {
            self.now.set(self.now.get() + by);
        }
    }

    impl Clock for FakeClock {
        type ThisTime = Fake;
        fn now(&self) -> Time<Fake> {
            self.now.get()
        }
    }

    impl Sleep for FakeClock {
        type Error = Infallible;
        fn sleep_until(&self, deadline: Time<Fake>) -> Result<(), Infallible> {
            self.sleeps.borrow_mut().push(deadline);
            // Sleeping takes the clock to the deadline, unless already past it.
            if deadline > self.now.get() {
                self.now.set(deadline);
            }
            Ok(())
        }
    }

    /// An app that records its hooks and can take time or fail.
    struct Probe<'c> {
        id: u8,
        clock: &'c FakeClock,
        trace: &'c RefCell<Vec<(u8, Hook)>>,
        work: Duration,
        fail: bool,
    }

    impl<'c> Probe<'c> {
        fn new(id: u8, clock: &'c FakeClock, trace: &'c RefCell<Vec<(u8, Hook)>>) -> Self {
            Self { id, clock, trace, work: Duration::ZERO, fail: false }
        }
    }

    impl App for Probe<'_> {
        type Error = ErrorKind;
        fn on_process(&mut self) -> Result<(), ErrorKind> {
            self.trace.borrow_mut().push((self.id, Hook::Process));
            self.clock.advance(self.work);
            if self.fail { Err(ErrorKind::Timeout) } else { Ok(()) }
        }
        fn on_exit(&mut self) -> Result<(), ErrorKind> {
            self.trace.borrow_mut().push((self.id, Hook::Exit));
            Ok(())
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// What a test log keeps of an event.
    #[derive(Debug, PartialEq)]
    enum Seen {
        App(usize, Hook, ErrorKind),
        Overrun(usize, Duration, Option<usize>),
        Sleep(usize),
    }

    fn seen(event: SchEvent<'_>) -> Seen {
        match event {
            SchEvent::AppFailed { app, hook, error } => Seen::App(app, hook, error.kind()),
            SchEvent::Overrun { minor_frame, late_by, slowest_app } => Seen::Overrun(minor_frame, late_by, slowest_app),
            SchEvent::SleepFailed { minor_frame, .. } => Seen::Sleep(minor_frame),
        }
    }

    #[test]
    fn register_places_apps_by_period_and_offset() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b, mut c) =
            (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace), Probe::new(2, &clock, &trace));
        let mut sch = TdmSchBuilder::<_, _, 3, 10, 3>::new(&clock, ms(5), [&mut a, &mut b, &mut c], NoLog).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap(); // every frame
        sch.register(1, ms(10), ms(0)).unwrap(); // even frames
        sch.register(2, ms(25), ms(5)).unwrap(); // frames 1 and 6
        // Column 0 is always app 0.
        assert!((0..10).all(|f| sch.app_at(f, 0) == Some(0)));
        // Column 1: app 1 on even frames, app 2 on frames 1 and 6.
        // App 1 (10 ms) runs in even frames, app 2 (25 ms, offset 5 ms) in
        // frames 1 and 6. Frame 6 has both: app 1 registered first, so it
        // took column 1 and app 2 the next free column.
        let col = |slot| (0..10).map(|f| sch.app_at(f, slot)).collect::<Vec<_>>();
        assert_eq!(col(1), [Some(1), Some(2), Some(1), None, Some(1), None, Some(1), None, Some(1), None]);
        assert_eq!(col(2), [None, None, None, None, None, None, Some(2), None, None, None]);
    }

    #[test]
    fn register_rejects_bad_configurations_without_changing_the_table() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        let mut sch = TdmSchBuilder::<_, _, 1, 10, 2>::new(&clock, ms(5), [&mut a, &mut b], NoLog).unwrap();
        assert_eq!(sch.register(2, ms(5), ms(0)), Err(SchConfigError::NoSuchApp { app: 2, apps: 2 }));
        assert_eq!(sch.register(0, ms(7), ms(0)), Err(SchConfigError::PeriodNotMultipleOfMinorFrame));
        assert_eq!(sch.register(0, ms(0), ms(0)), Err(SchConfigError::PeriodNotMultipleOfMinorFrame));
        assert_eq!(sch.register(0, ms(15), ms(0)), Err(SchConfigError::PeriodDoesNotDivideMajorFrame));
        assert_eq!(sch.register(0, ms(10), ms(10)), Err(SchConfigError::InvalidOffset));
        assert_eq!(sch.register(0, ms(10), ms(3)), Err(SchConfigError::InvalidOffset));
        sch.register(0, ms(10), ms(0)).unwrap();
        assert_eq!(sch.register(0, ms(10), ms(0)), Err(SchConfigError::AlreadyScheduled { minor_frame: 0 }));
        // One slot per frame: app 1 every frame collides at frame 0, and
        // nothing is written, so frame 1 stays empty.
        assert_eq!(sch.register(1, ms(5), ms(0)), Err(SchConfigError::FrameFull { minor_frame: 0 }));
        assert_eq!(sch.app_at(1, 0), None);
        assert_eq!(SchConfigError::FrameFull { minor_frame: 0 }.kind(), ErrorKind::Full);
    }

    #[test]
    fn new_rejects_a_zero_minor_frame() {
        let clock = FakeClock::at(0);
        let r = TdmSchBuilder::<_, _, 1, 1, 0>::new(&clock, Duration::ZERO, [], NoLog);
        assert!(matches!(r, Err(SchConfigError::ZeroMinorFrame)));
    }

    #[test]
    fn major_frame_runs_rows_in_order() {
        let clock = FakeClock::at(100);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        let mut sch = TdmSchBuilder::<_, _, 2, 4, 2>::new(&clock, ms(5), [&mut a, &mut b], NoLog).unwrap();
        sch.register(1, ms(5), ms(0)).unwrap(); // registered first: column 0
        sch.register(0, ms(10), ms(5)).unwrap(); // odd frames, column 1
        let mut sch = sch.build();
        let report = sch.run_major_frame();
        assert!(report.is_clean());
        use Hook::Process as P;
        assert_eq!(*trace.borrow(), [(1, P), (1, P), (0, P), (1, P), (1, P), (0, P)]);
    }

    #[test]
    fn frames_are_paced_to_absolute_deadlines_across_major_frames() {
        let clock = FakeClock::at(100);
        let trace = RefCell::new(Vec::new());
        let mut a = Probe::new(0, &clock, &trace);
        a.work = ms(1);
        let mut sch = TdmSchBuilder::<_, _, 1, 4, 1>::new(&clock, ms(5), [&mut a], NoLog).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap();
        let mut sch = sch.build();
        // 4 × 5 ms, in 2^-32 s units; 20 ms converted directly is one unit
        // longer, because 5 ms is not a whole number of units.
        assert_eq!(sch.major_frame_period(), ms(5) * 4);
        sch.run_major_frame();
        // Something between major frames takes 2 ms; the schedule must not drift.
        clock.advance(ms(2));
        sch.run_major_frame();
        let start = Time::<Fake>::new(100, 0);
        let expected: Vec<_> = (1..=8).map(|k| start + ms(5) * k).collect();
        assert_eq!(*clock.sleeps.borrow(), expected);
    }

    #[test]
    fn a_failing_app_is_logged_and_the_rest_still_run() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        a.fail = true;
        let events = RefCell::new(Vec::new());
        let log = |e: SchEvent<'_>| events.borrow_mut().push(seen(e));
        let mut sch = TdmSchBuilder::<_, _, 2, 2, 2>::new(&clock, ms(5), [&mut a, &mut b], log).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap();
        sch.register(1, ms(5), ms(0)).unwrap();
        let report = sch.build().run_major_frame();
        assert_eq!(report.app_failures, 2);
        assert_eq!(
            *events.borrow(),
            [Seen::App(0, Hook::Process, ErrorKind::Timeout), Seen::App(0, Hook::Process, ErrorKind::Timeout)]
        );
        // App 1 still ran in both frames.
        assert_eq!(trace.borrow().iter().filter(|(id, h)| *id == 1 && *h == Hook::Process).count(), 2);
    }

    #[test]
    fn a_catch_up_frame_holds_only_every_frame_apps_and_gives_slack() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        b.work = ms(3); // overruns its 2 ms frame by 1 ms
        let events = RefCell::new(Vec::new());
        let log = |e: SchEvent<'_>| events.borrow_mut().push(seen(e));
        let mut sch = TdmSchBuilder::<_, _, 2, 4, 2>::new(&clock, ms(2), [&mut a, &mut b], log).unwrap();
        assert_eq!(sch.keep_free(4), Err(SchConfigError::NoSuchFrame { minor_frame: 4 }));
        sch.keep_free(3).unwrap();
        // An app whose only frame is the catch-up frame would never run.
        assert_eq!(sch.register(1, ms(8), ms(6)), Err(SchConfigError::OnlyFreeFrames));
        // Every frame: in the catch-up frame too. Every other frame: not.
        sch.register(0, ms(2), ms(0)).unwrap();
        sch.register(1, ms(4), ms(2)).unwrap();
        assert_eq!((sch.app_at(3, 0), sch.app_at(1, 1), sch.app_at(3, 1)), (Some(0), Some(1), None));
        let mut sch = sch.build();
        // b overruns frame 1; frame 2 starts late but has slack, and the
        // major frame ends on its deadline.
        sch.run_major_frame();
        assert_eq!(*events.borrow(), [Seen::Overrun(1, ms(3) - ms(2), Some(1))]);
        assert_eq!(clock.now(), Time::new(0, 0) + ms(2) * 4);
    }

    #[test]
    fn the_report_says_how_long_the_apps_ran() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        a.work = ms(1);
        b.work = ms(2);
        let mut sch = TdmSchBuilder::<_, _, 2, 4, 2>::new(&clock, ms(5), [&mut a, &mut b], NoLog).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap(); // every frame
        sch.register(1, ms(20), ms(5)).unwrap(); // frame 1 only
        let mut sch = sch.build();
        let report = sch.run_major_frame();
        // 4 × 1 ms, plus 2 ms in frame 1, which is the busiest at 3 ms
        // (as sums: 1 ms is not a whole number of 2^-32 s units).
        assert_eq!((report.busy, report.busiest_frame), (ms(1) * 4 + ms(2), ms(1) + ms(2)));
        // Per app: a ran 4 times, b once; nothing overran.
        let [ta, tb] = sch.take_timings();
        assert_eq!((ta.runs, ta.busy, ta.longest, ta.overruns), (4, ms(1) * 4, ms(1), 0));
        assert_eq!((tb.runs, tb.busy, tb.longest, tb.overruns), (1, ms(2), ms(2), 0));
        // Taken: counting starts afresh.
        assert_eq!(sch.take_timings(), [AppTiming::default(); 2]);
        let mut twice = report;
        twice.merge(report);
        assert_eq!((twice.busy, twice.busiest_frame), (report.busy + report.busy, ms(1) + ms(2)));
    }

    #[test]
    fn an_overrun_is_logged_with_how_late_it_was() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let mut a = Probe::new(0, &clock, &trace);
        a.work = ms(7); // a 5 ms frame's app takes 7 ms
        let events = RefCell::new(Vec::new());
        let log = |e: SchEvent<'_>| events.borrow_mut().push(seen(e));
        let mut sch = TdmSchBuilder::<_, _, 1, 2, 1>::new(&clock, ms(5), [&mut a], log).unwrap();
        sch.register(0, ms(10), ms(0)).unwrap(); // frame 0 only
        let report = sch.build().run_major_frame();
        // Frame 0 ends 2 ms late; frame 1 has slack and is back on time.
        assert_eq!(report.overruns, 1);
        assert_eq!(*events.borrow(), [Seen::Overrun(0, ms(7) - ms(5), Some(0))]);
        assert_eq!(clock.now(), Time::new(0, 0) + ms(10));
    }

    #[test]
    fn an_overrun_is_put_down_to_the_slowest_app_in_its_frame() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let (mut a, mut b) = (Probe::new(0, &clock, &trace), Probe::new(1, &clock, &trace));
        a.work = ms(1);
        b.work = ms(6); // with a, 7 ms in a 5 ms frame
        let events = RefCell::new(Vec::new());
        let log = |e: SchEvent<'_>| events.borrow_mut().push(seen(e));
        let mut sch = TdmSchBuilder::<_, _, 2, 2, 2>::new(&clock, ms(5), [&mut a, &mut b], log).unwrap();
        sch.register(0, ms(10), ms(0)).unwrap();
        sch.register(1, ms(10), ms(0)).unwrap();
        let mut sch = sch.build();
        assert_eq!(sch.run_major_frame().overruns, 1);
        assert_eq!(*events.borrow(), [Seen::Overrun(0, ms(1) + ms(6) - ms(5), Some(1))]);
        let [ta, tb] = sch.take_timings();
        assert_eq!((ta.overruns, tb.overruns, tb.longest), (0, 1, ms(6)));
    }

    #[test]
    fn exactly_on_the_deadline_is_not_an_overrun() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let mut a = Probe::new(0, &clock, &trace);
        a.work = ms(5);
        let mut sch = TdmSchBuilder::<_, _, 1, 1, 1>::new(&clock, ms(5), [&mut a], NoLog).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap();
        assert_eq!(sch.build().run_major_frame().overruns, 0);
    }

    #[test]
    fn lifecycle_order_is_enforced() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let mut a = Probe::new(0, &clock, &trace);
        let mut sch = TdmSchBuilder::<_, _, 1, 1, 1>::new(&clock, ms(5), [&mut a], NoLog).unwrap();
        sch.register(0, ms(5), ms(0)).unwrap();
        let mut sch = sch.build();
        sch.run_major_frame();
        sch.exit();
        sch.exit(); // a second exit does nothing
        assert_eq!(sch.run_major_frame(), Report::default()); // nothing after exit
        use Hook::{Exit as E, Process as P};
        assert_eq!(*trace.borrow(), [(0, P), (0, E)]);
    }

    #[test]
    fn exit_before_any_frame_exits_each_app_and_runs_nothing_after() {
        let clock = FakeClock::at(0);
        let trace = RefCell::new(Vec::new());
        let mut a = Probe::new(0, &clock, &trace);
        let mut sch = TdmSchBuilder::<_, _, 1, 1, 1>::new(&clock, ms(5), [&mut a], NoLog).unwrap().build();
        sch.exit();
        sch.run_major_frame();
        assert_eq!(*trace.borrow(), [(0, Hook::Exit)]);
    }
}
