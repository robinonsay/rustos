//! Portable watchdog trait and reset-cause vocabulary.
//!
//! A watchdog is a countdown timer that resets the system when it reaches
//! zero. Software that is working reloads ("feeds") it regularly; software
//! that has hung stops feeding it, and the reset puts the system back into
//! a known state. Almost every processor family has one, from
//! microcontrollers to the TCO watchdog of an Intel chipset, and they all
//! reduce to the same operations: start with a timeout, feed, and
//! optionally force a reset now.
//!
//! After boot, the most useful question is *why* the system last reset.
//! [`ResetReason`] is a chip-independent answer: each implementation maps
//! whatever its hardware records onto it, so application code can log or
//! react to "the watchdog fired" the same way on any board.
//!
//! ```ignore
//! use api::time::Duration;
//! use api::watchdog::{ResetReason, Watchdog};
//!
//! fn run<W: Watchdog>(wd: &mut W) -> ! {
//!     if wd.reset_reason() == ResetReason::WatchdogTimeout {
//!         // last run hung: enter a degraded mode, report it, ...
//!     }
//!     wd.start(Duration::from_millis(100)).ok();
//!     loop {
//!         // ... work ...
//!         wd.feed();
//!     }
//! }
//! ```

use crate::common::ErrorType;
use crate::time::Duration;

/// Why the system last reset, in terms that apply to any hardware.
///
/// Hardware records different amounts of detail; an implementation maps
/// what it can tell apart onto these, and uses [`Unknown`](Self::Unknown)
/// for anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    /// Power was applied.
    PowerOn,
    /// The supply dropped below the level needed to run reliably.
    Brownout,
    /// An external reset input was asserted (a reset button or pin).
    ExternalPin,
    /// A debugger asked for the reset.
    Debugger,
    /// The watchdog ran out: the software stopped feeding it.
    WatchdogTimeout,
    /// Software asked for the reset on purpose: [`Watchdog::force_reset`],
    /// a bootloader's reboot, or the platform's equivalent.
    Software,
    /// A power or clock fault detector fired (a supply glitch detector,
    /// a clock-loss detector, ...).
    Fault,
    /// The system woke from a low-power state that powered the processor
    /// off.
    Wake,
    /// The hardware gave no reason this implementation recognises.
    Unknown,
}

/// A watchdog that resets the whole system when it is not fed in time.
///
/// Timeouts are [`Duration`]s. Hardware counts in its own units, so the
/// timeout actually used may differ from the one asked for; [`start`]
/// returns it. Implementations should round a timeout *up* to their next
/// unit ([`Duration::as_micros_ceil`] and friends), so the watchdog never
/// fires earlier than asked.
///
/// [`start`]: Self::start
pub trait Watchdog: ErrorType {
    /// The longest timeout this watchdog supports.
    fn max_timeout(&self) -> Duration;

    /// Start the watchdog, or restart it with a new timeout. The system
    /// resets unless [`feed`](Self::feed) is called at least every
    /// `timeout` from now on. Returns the timeout actually used. A timeout
    /// the hardware cannot represent is clamped or reported as an error,
    /// as the implementation documents.
    fn start(&mut self, timeout: Duration) -> Result<Duration, Self::Error>;

    /// Reload the countdown. Does nothing before [`start`](Self::start).
    ///
    /// Feed from the place that proves the software is healthy — the main
    /// loop after its work is done — not from a timer interrupt that keeps
    /// running when the main loop hangs.
    fn feed(&mut self);

    /// `true` while the watchdog is counting down.
    fn is_running(&self) -> bool;

    /// Time left before the watchdog fires, if the hardware can tell;
    /// `None` if it cannot or the watchdog is not running.
    fn time_remaining(&self) -> Option<Duration>;

    /// Why the system last reset. Implementations capture this at
    /// construction, before anything can disturb the evidence.
    fn reset_reason(&self) -> ResetReason;

    /// Reset the system now, through the watchdog. Never returns. The
    /// next boot reports [`ResetReason::Software`].
    fn force_reset(&mut self) -> !;
}
