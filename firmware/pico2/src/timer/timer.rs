//! The TIMER0 driver: [`Rp2350Timer0`], [`Rp2350Clock`] and [`Rp2350Alarm`].
//!
//! Every register sequence here is a generic function over
//! [`Regs`](crate::common::reg::Regs), run on the target through `Mmio` and
//! in host tests through the scripted register file; the trait methods on the
//! target are one call each.
//!
//! ## Using an alarm from an interrupt handler
//!
//! An alarm latches its fire in `INTR`; with its `INTE` bit set (done by
//! `schedule_at`) the latch raises `TIMER0_IRQ_N`, and once the application
//! enables that line at the NVIC the handler runs. The handler calls
//! [`Alarm::take_fired`], which clears the latch and returns whether there was
//! one. A handler can be entered with nothing latched (the alarm was cancelled
//! after its line became pending), so it acts only when `take_fired` returns
//! `true`. The alarm value itself lives in a
//! [`CsCell`](crate::critical_section::CsCell) shared by thread code and the
//! handler.

use core::convert::Infallible;

use api::common::ErrorType;
#[cfg(target_os = "none")]
use api::device::DeviceHandle;
use api::time::Scheduled;
#[cfg(target_os = "none")]
use api::time::{Alarm, Clock, Instant};

#[cfg(target_os = "none")]
use crate::clocks::clocks::ClocksReady;
use crate::common::reg::{ALIAS_CLR, ALIAS_SET, RegAddr, Regs, poll};
use crate::common::reset::{RESET_DONE_OFFSET, RESET_OFFSET};
use crate::irq::Irq;

use super::{
    ALARM0, ALL_ALARMS, ARMED, AfterArm, ArmPlan, DBGPAUSE, DBGPAUSE_BOTH, INTE, INTR, PAUSE,
    RESET_BIT_TIMER0, RESET_POLL_BUDGET, SOURCE, SOURCE_TICK, TIMERAWH, TIMERAWL, after_arm,
    plan_arm, select_time,
};

/// TIMER0 as the board's `timer0` device. Split it into its clock and alarms.
pub struct Rp2350Timer0 {
    _private: (),
}

/// Why [`Rp2350Timer0::new`] failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    /// `RESETS.RESET_DONE` did not report TIMER0 within the poll budget.
    ResetTimeout,
}

/// Why an alarm operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmError {
    /// The target is more than [`MAX_SPAN_US`](super::MAX_SPAN_US) ahead (ALM-6).
    TooFar,
    /// After `ALARMn` was written the alarm was neither armed nor fired: the
    /// hardware did not arm. The alarm is left disarmed.
    NotArmed,
}

/// The TIMER0 time base, [`api::time::Clock`]. Zero-sized and `Clone`: reading
/// the raw time registers has no side effect (Tables 1236, 1237), so any
/// number of clocks may exist.
#[derive(Clone)]
pub struct Rp2350Clock {
    _private: (),
}

/// TIMER0 alarm `N` (0 to 3), [`api::time::Alarm`], raising `TIMER0_IRQ_N`.
pub struct Rp2350Alarm<const N: usize> {
    _private: (),
}

impl<const N: usize> Rp2350Alarm<N> {
    /// Rejects `N > 3` at compile time wherever an alarm is constructed.
    const VALID: () = assert!(N < 4, "TIMER0 has alarms 0 to 3");

    const fn new() -> Self {
        let () = Self::VALID;
        Self { _private: () }
    }

    /// The interrupt line this alarm raises (Table 94: `TIMER0_IRQ_N` is line `N`).
    #[must_use]
    pub const fn irq() -> Irq {
        match N {
            0 => Irq::TIMER0_IRQ_0,
            1 => Irq::TIMER0_IRQ_1,
            2 => Irq::TIMER0_IRQ_2,
            _ => Irq::TIMER0_IRQ_3,
        }
    }
}

impl Rp2350Timer0 {
    /// Release TIMER0 from reset and bring it to a known state: counting the
    /// tick (`SOURCE = 0`), not paused, paused only while a debugger halts a
    /// core (`DBGPAUSE` reset value), all alarm interrupts off, all alarms
    /// disarmed, all latches cleared.
    ///
    /// Takes `&ClocksReady` so that TIMER0 cannot be used before its 1 µs tick
    /// runs from the crystal (cwht coding standard CS-37).
    ///
    /// # Errors
    ///
    /// [`TimerError::ResetTimeout`] if the block never reports out of reset.
    #[cfg(target_os = "none")]
    pub fn new(_handle: DeviceHandle<Self>, _clocks: &ClocksReady) -> Result<Self, TimerError> {
        release(&mut crate::common::reg::Mmio).map(|()| Self { _private: () })
    }

    /// The clock and the four alarms. Each alarm exists once.
    #[must_use]
    pub const fn split(
        self,
    ) -> (
        Rp2350Clock,
        Rp2350Alarm<0>,
        Rp2350Alarm<1>,
        Rp2350Alarm<2>,
        Rp2350Alarm<3>,
    ) {
        (
            Rp2350Clock { _private: () },
            Rp2350Alarm::new(),
            Rp2350Alarm::new(),
            Rp2350Alarm::new(),
            Rp2350Alarm::new(),
        )
    }
}

/// Release TIMER0 from reset and set its control registers (see
/// [`Rp2350Timer0::new`]).
pub(crate) fn release<R: Regs>(regs: &mut R) -> Result<(), TimerError> {
    regs.write(RegAddr::RESET, RESET_OFFSET + ALIAS_CLR, RESET_BIT_TIMER0);
    let done = (RESET_BIT_TIMER0, RESET_BIT_TIMER0);
    poll(
        regs,
        RegAddr::RESET,
        RESET_DONE_OFFSET,
        done.0,
        done.1,
        RESET_POLL_BUDGET,
    )
    .map_err(|_| TimerError::ResetTimeout)?;
    regs.write(RegAddr::TIMER0, SOURCE, SOURCE_TICK);
    regs.write(RegAddr::TIMER0, PAUSE, 0);
    regs.write(RegAddr::TIMER0, DBGPAUSE, DBGPAUSE_BOTH);
    regs.write(RegAddr::TIMER0, INTE, 0);
    regs.write(RegAddr::TIMER0, ARMED, ALL_ALARMS);
    regs.write(RegAddr::TIMER0, INTR, ALL_ALARMS);
    Ok(())
}

/// The 64-bit time: `TIMERAWH`, `TIMERAWL`, `TIMERAWH`, `TIMERAWL`, then
/// [`select_time`].
pub(crate) fn read_time<R: Regs>(regs: &mut R) -> u64 {
    let hi1 = regs.read(RegAddr::TIMER0, TIMERAWH);
    let lo1 = regs.read(RegAddr::TIMER0, TIMERAWL);
    let hi2 = regs.read(RegAddr::TIMER0, TIMERAWH);
    let lo2 = regs.read(RegAddr::TIMER0, TIMERAWL);
    select_time(hi1, lo1, hi2, lo2)
}

/// Disarm alarm `n` (write 1 to its `ARMED` bit) and clear its latch.
pub(crate) fn disarm<R: Regs>(regs: &mut R, n: usize) {
    let bit = 1u32 << n;
    regs.write(RegAddr::TIMER0, ARMED, bit);
    regs.write(RegAddr::TIMER0, INTR, bit);
}

/// Arm alarm `n` for `at` (ALM-1, ALM-2, ALM-5, ALM-6).
pub(crate) fn schedule<R: Regs>(regs: &mut R, n: usize, at: u64) -> Result<Scheduled, AlarmError> {
    disarm(regs, n);
    let value = match plan_arm(read_time(regs), at) {
        ArmPlan::Due => return Ok(Scheduled::Due),
        ArmPlan::TooFar => return Err(AlarmError::TooFar),
        ArmPlan::Arm(value) => value,
    };
    let bit = 1u32 << n;
    regs.write(RegAddr::TIMER0, INTE + ALIAS_SET, bit);
    regs.write(RegAddr::TIMER0, ALARM0 + 4 * n, value);
    let now = read_time(regs);
    let armed = regs.read(RegAddr::TIMER0, ARMED) & bit != 0;
    let fired = regs.read(RegAddr::TIMER0, INTR) & bit != 0;
    match after_arm(armed, fired, now, at) {
        AfterArm::Pending => Ok(Scheduled::Armed),
        AfterArm::Missed => {
            disarm(regs, n);
            Ok(Scheduled::Due)
        }
        AfterArm::Fault => {
            disarm(regs, n);
            Err(AlarmError::NotArmed)
        }
    }
}

/// Whether alarm `n` is armed (its `ARMED` bit).
pub(crate) fn armed<R: Regs>(regs: &mut R, n: usize) -> bool {
    regs.read(RegAddr::TIMER0, ARMED) & (1u32 << n) != 0
}

/// Clear alarm `n`'s latch if set, returning whether it was (ALM-3). The
/// write happens only after a read saw the bit, so a fire that latches between
/// the read and the write is not lost.
pub(crate) fn clear_fired<R: Regs>(regs: &mut R, n: usize) -> bool {
    let bit = 1u32 << n;
    let fired = regs.read(RegAddr::TIMER0, INTR) & bit != 0;
    if fired {
        regs.write(RegAddr::TIMER0, INTR, bit);
    }
    fired
}

impl ErrorType for Rp2350Clock {
    type Error = Infallible;
}

#[cfg(target_os = "none")]
impl Clock for Rp2350Clock {
    fn now(&mut self) -> Result<Instant, Infallible> {
        Ok(Instant::from_micros(read_time(
            &mut crate::common::reg::Mmio,
        )))
    }
}

impl<const N: usize> ErrorType for Rp2350Alarm<N> {
    type Error = AlarmError;
}

#[cfg(target_os = "none")]
impl<const N: usize> Alarm for Rp2350Alarm<N> {
    fn schedule_at(&mut self, at: Instant) -> Result<Scheduled, AlarmError> {
        schedule(&mut crate::common::reg::Mmio, N, at.as_micros())
    }
    fn cancel(&mut self) -> Result<(), AlarmError> {
        disarm(&mut crate::common::reg::Mmio, N);
        Ok(())
    }
    fn is_armed(&mut self) -> Result<bool, AlarmError> {
        Ok(armed(&mut crate::common::reg::Mmio, N))
    }
    fn take_fired(&mut self) -> Result<bool, AlarmError> {
        Ok(clear_fired(&mut crate::common::reg::Mmio, N))
    }
}

#[cfg(test)]
#[path = "timer_tests.rs"]
mod tests;
