//! The application lifecycle: the unit a scheduler runs.
//!
//! An application ("app") is one capability of a system — read a sensor,
//! run a control loop, send telemetry — packaged so that a
//! scheduler can drive it without knowing what it does. This is the Rust
//! form of LibJuno's `juno::app::APP_API_T` (`juno/app/app_api.hpp`), with
//! the same three hooks:
//!
//! | Hook | LibJuno | When |
//! |------|---------|------|
//! | [`App::on_start`] | `OnStart` | once, before the first `on_process` |
//! | [`App::on_process`] | `OnProcess` | every time the app's schedule slot comes up |
//! | [`App::on_exit`] | `OnExit` | once, at shutdown (never, on a system that runs until power-off) |
//!
//! Where LibJuno passes an `APP_ROOT_T&` and the implementation downcasts
//! it to its own aggregate, a Rust app is simply a type that implements
//! [`App`]: `self` *is* the app's state, with no cast. Dependencies (a
//! driver, a clock, a bus) are fields of that type, set when it is
//! constructed — the same dependency injection LibJuno's composition root
//! does, checked by the compiler.
//!
//! Each hook returns `Result<(), Self::Error>`. The error type is the app's
//! own, but must implement [`api::error::Error`](crate::error::Error), so a
//! scheduler can log it and classify it without knowing the type.
//!
//! What an app reports about itself, run by run, is its [`Health`]; see
//! [`health`].
//!
//! ```ignore
//! use api::app::App;
//! use api::error::{Error, ErrorKind};
//!
//! struct Blink<P> { led: P, on: bool }
//!
//! impl<P: api::common::Write<bool>> App for Blink<P> {
//!     type Error = ErrorKind;
//!     fn on_start(&mut self) -> Result<(), ErrorKind> { Ok(()) }
//!     fn on_process(&mut self) -> Result<(), ErrorKind> {
//!         self.on = !self.on;
//!         self.led.write(self.on).map_err(|_| ErrorKind::Write)
//!     }
//!     fn on_exit(&mut self) -> Result<(), ErrorKind> { Ok(()) }
//! }
//! ```

pub mod health;

pub use health::{AppState, Health, HealthTlm};

use crate::error::Error;

/// An application with a start, a periodic step and an exit.
///
/// A scheduler guarantees the order: `on_start` once, then `on_process`
/// any number of times, then at most one `on_exit`. A failed hook is
/// reported, not fatal: the scheduler keeps calling the app's later hooks
/// and keeps running the other apps.
pub trait App {
    /// What this app's hooks report when they fail.
    type Error: Error;

    /// Prepare to run: claim resources, probe hardware, set the initial
    /// state. Called once, before the first [`on_process`](Self::on_process).
    fn on_start(&mut self) -> Result<(), Self::Error>;

    /// Do one period's work. Called once per scheduled slot; must finish
    /// well within the scheduler's minor frame, because the scheduler
    /// cannot interrupt it.
    fn on_process(&mut self) -> Result<(), Self::Error>;

    /// Release resources and leave outputs safe. Called at most once,
    /// after the last `on_process`.
    fn on_exit(&mut self) -> Result<(), Self::Error>;
}

/// Which lifecycle hook a report is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    /// [`App::on_start`].
    Start,
    /// [`App::on_process`].
    Process,
    /// [`App::on_exit`].
    Exit,
}

/// An [`App`] with its error type erased, so apps with different error
/// types can sit in one array: `[&mut dyn DynApp; N]`.
///
/// Implemented automatically for every [`App`]; never implement it by hand.
/// A hook's error, if any, is handed to `report` as a `&dyn Error` while it
/// is still alive — no allocation, and the full error stays available for
/// logging. Each method returns `true` if the hook succeeded.
pub trait DynApp {
    /// Run `hook`, passing its error, if it fails, to `report`.
    fn run_hook(&mut self, hook: Hook, report: &mut dyn FnMut(&dyn Error)) -> bool;
}

impl<A: App> DynApp for A {
    fn run_hook(&mut self, hook: Hook, report: &mut dyn FnMut(&dyn Error)) -> bool {
        // Call the typed hook.
        let result = match hook {
            Hook::Start => self.on_start(),
            Hook::Process => self.on_process(),
            Hook::Exit => self.on_exit(),
        };
        match result {
            // Success: nothing to report.
            Ok(()) => true,
            // Failure: let the caller see the error, then drop it.
            Err(error) => {
                report(&error);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    /// Counts its hooks; fails `on_process` when told to.
    struct Counter {
        starts: u32,
        processes: u32,
        exits: u32,
        fail: bool,
    }

    impl App for Counter {
        type Error = ErrorKind;
        fn on_start(&mut self) -> Result<(), ErrorKind> {
            self.starts += 1;
            Ok(())
        }
        fn on_process(&mut self) -> Result<(), ErrorKind> {
            self.processes += 1;
            if self.fail { Err(ErrorKind::Timeout) } else { Ok(()) }
        }
        fn on_exit(&mut self) -> Result<(), ErrorKind> {
            self.exits += 1;
            Ok(())
        }
    }

    #[test]
    fn dyn_app_runs_the_named_hook() {
        let mut c = Counter { starts: 0, processes: 0, exits: 0, fail: false };
        let app: &mut dyn DynApp = &mut c;
        let mut never = |_: &dyn Error| panic!("no error expected");
        assert!(app.run_hook(Hook::Start, &mut never));
        assert!(app.run_hook(Hook::Process, &mut never));
        assert!(app.run_hook(Hook::Exit, &mut never));
        assert_eq!((c.starts, c.processes, c.exits), (1, 1, 1));
    }

    #[test]
    fn dyn_app_reports_the_error_it_got() {
        let mut c = Counter { starts: 0, processes: 0, exits: 0, fail: true };
        let mut seen = None;
        let ok = DynApp::run_hook(&mut c, Hook::Process, &mut |e| seen = Some(e.kind()));
        assert!(!ok);
        assert_eq!(seen, Some(ErrorKind::Timeout));
    }
}
