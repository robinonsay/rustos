//! iLoads: the configuration tables apps load at start.
//!
//! Every number an app is configured with — its schedule slot, its chip's
//! ranges, its sensor's biases, its radio's channel — lives in a table,
//! never in the app's code. Each app has its own `table.rs` beside its
//! `app.rs`, holding the table's type and its values. `main` hands each
//! app its table when it builds it, and the app keeps a reference and
//! reads it from then on, so changing a value is an edit to one
//! `table.rs`, and the next build runs it.
//!
//! Tables are compiled in: there is no file system to load them from at
//! boot. Each table type has a `const fn check`, and each `table.rs` runs
//! it on its values at compile time through [`assert_ok`], so a table that
//! is out of range fails the build instead of the boot.
//!
//! ```ignore
//! use api::iload::{Slot, assert_ok};
//!
//! pub struct BlinkTable { pub slot: Slot }
//!
//! impl BlinkTable {
//!     pub const fn check(&self) -> Result<(), &'static str> {
//!         // 1 ms minor frames, 1 s major frame.
//!         self.slot.check(1, 1000)
//!     }
//! }
//!
//! pub const DEFAULT: BlinkTable = BlinkTable { slot: Slot { period_ms: 500, offset_ms: 0 } };
//!
//! const _: () = assert_ok(DEFAULT.check());
//! ```

use crate::time::Duration;

/// Where an app runs in its core's schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// Period, in milliseconds: how often the scheduler runs the app. Must
    /// be a whole number of the core's minor frames and divide its major
    /// frame.
    pub period_ms: u64,
    /// Offset, in milliseconds, into each period: which minor frame the
    /// app runs in. Less than the period.
    pub offset_ms: u64,
}

impl Slot {
    /// Every minor frame of a core whose minor frame is `minor_ms`.
    pub const fn every(minor_ms: u64) -> Self {
        Self { period_ms: minor_ms, offset_ms: 0 }
    }

    /// Runs per second, rounded down; 0 for a period of 0.
    pub const fn hz(&self) -> u32 {
        match 1000u64.checked_div(self.period_ms) {
            Some(hz) => hz as u32,
            None => 0,
        }
    }

    /// The period and offset, as
    /// [`TdmSchBuilder::register`](crate::sch::TdmSchBuilder::register)
    /// takes them.
    pub const fn durations(&self) -> (Duration, Duration) {
        // Tables hold milliseconds; the scheduler takes durations.
        (Duration::from_millis(self.period_ms), Duration::from_millis(self.offset_ms))
    }

    /// `Err` unless this slot fits a schedule of `minor_ms` minor frames
    /// and a `major_ms` major frame.
    pub const fn check(&self, minor_ms: u64, major_ms: u64) -> Result<(), &'static str> {
        if minor_ms == 0 || self.period_ms == 0 || !self.period_ms.is_multiple_of(minor_ms) {
            return Err("slot period is not a whole number of minor frames");
        }
        if !major_ms.is_multiple_of(self.period_ms) {
            return Err("slot period does not divide the major frame");
        }
        if self.offset_ms >= self.period_ms || !self.offset_ms.is_multiple_of(minor_ms) {
            return Err("slot offset is not a minor frame inside the period");
        }
        Ok(())
    }
}

/// Fail the build with `check`'s message, when called in a `const`:
///
/// ```ignore
/// const _: () = assert_ok(DEFAULT.check());
/// ```
pub const fn assert_ok(check: Result<(), &'static str>) {
    if let Err(why) = check {
        panic!("{}", why);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_must_fit_the_schedule() {
        assert_eq!(Slot { period_ms: 10, offset_ms: 3 }.check(1, 1000), Ok(()));
        assert!(Slot { period_ms: 3, offset_ms: 0 }.check(1, 1000).is_err());
        assert!(Slot { period_ms: 10, offset_ms: 10 }.check(1, 1000).is_err());
        assert!(Slot { period_ms: 15, offset_ms: 0 }.check(5, 1000).is_err());
        assert!(Slot { period_ms: 15, offset_ms: 0 }.check(5, 30).is_ok());
        assert!(Slot { period_ms: 100, offset_ms: 5 }.check(10, 1000).is_err());
        assert!(Slot::every(1).check(0, 1000).is_err());
    }

    #[test]
    fn rates_and_durations() {
        assert_eq!(Slot::every(1).hz(), 1000);
        assert_eq!(Slot { period_ms: 40, offset_ms: 0 }.hz(), 25);
        assert_eq!(Slot { period_ms: 0, offset_ms: 0 }.hz(), 0);
        let s = Slot { period_ms: 250, offset_ms: 10 };
        assert_eq!(s.durations(), (Duration::from_millis(250), Duration::from_millis(10)));
    }

    #[test]
    fn a_good_check_passes_at_compile_time() {
        const _: () = assert_ok(Slot::every(1).check(1, 1000));
    }
}
