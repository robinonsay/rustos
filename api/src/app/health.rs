//! What every app reports about itself: [`Health`], counted every run and
//! published as a [`HealthTlm`].
//!
//! The record names its app by an id of the system's choosing, `Id`:
//! usually a `#[repr(u8)]` enum of every app in the system, so the record
//! has a fixed number on the wire.

use crate::error::ErrorKind;

/// How an app's last run went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AppState {
    /// The app ran and produced data.
    Ok = 0,
    /// The app ran but had nothing new. Not a fault.
    NoData = 1,
    /// The app failed this run; `last_error` says how.
    Failing = 2,
    /// The app's device did not come up at boot; `last_error` says why.
    Absent = 3,
}

/// One app's health, as published.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealthTlm<Id> {
    /// Whose.
    pub app: Id,
    /// How the last run went.
    pub state: AppState,
    /// Runs so far.
    pub runs: u32,
    /// Runs that failed so far.
    pub failures: u32,
    /// The most recent failure, if any.
    pub last_error: Option<ErrorKind>,
}

/// An app's running health counts.
#[derive(Clone, Copy, Debug)]
pub struct Health<Id> {
    /// The record as of the last run.
    tlm: HealthTlm<Id>,
}

impl<Id: Copy> Health<Id> {
    /// A fresh record for `app`. `absent` is the bring-up error, if its
    /// device did not come up.
    pub const fn new(app: Id, absent: Option<ErrorKind>) -> Self {
        // Absent apps say so from the first record.
        let state = if absent.is_some() { AppState::Absent } else { AppState::NoData };
        Self { tlm: HealthTlm { app, state, runs: 0, failures: 0, last_error: absent } }
    }

    /// Count one run that ended in `state`, with `error` if it failed.
    /// Returns the record to publish.
    pub fn record(&mut self, state: AppState, error: Option<ErrorKind>) -> HealthTlm<Id> {
        // Every run counts.
        self.tlm.runs = self.tlm.runs.wrapping_add(1);
        self.tlm.state = state;
        // Failures count, and keep their cause until the next failure.
        if let Some(kind) = error {
            self.tlm.failures = self.tlm.failures.wrapping_add(1);
            self.tlm.last_error = Some(kind);
        }
        self.tlm
    }

    /// The record as of the last run.
    pub const fn get(&self) -> HealthTlm<Id> {
        // As recorded.
        self.tlm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_from_the_start_and_failures_are_counted() {
        // An absent device says so before its first run.
        let mut h = Health::new(7u8, Some(ErrorKind::NotFound));
        assert_eq!(h.get().state, AppState::Absent);
        assert_eq!(h.get().last_error, Some(ErrorKind::NotFound));
        // A failure counts and replaces the cause; success keeps it.
        h.record(AppState::Failing, Some(ErrorKind::Timeout));
        let t = h.record(AppState::Ok, None);
        assert_eq!((t.app, t.runs, t.failures, t.last_error), (7, 2, 1, Some(ErrorKind::Timeout)));
        assert_eq!(t.state, AppState::Ok);
    }

    #[test]
    fn a_present_app_starts_with_no_data() {
        let h = Health::new((), None);
        assert_eq!((h.get().state, h.get().runs, h.get().last_error), (AppState::NoData, 0, None));
    }
}
