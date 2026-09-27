//! TIMER0: the 64-bit microsecond time base and its four alarms.
//!
//! TIMER0 counts the 1 µs tick that [`crate::clocks`] starts from the crystal
//! (datasheet §8.5, §12.8.1). [`timer::Rp2350Timer0`] releases the block from
//! reset and splits into one [`timer::Rp2350Clock`], which implements
//! [`api::time::Clock`], and four [`timer::Rp2350Alarm`]s, which implement
//! [`api::time::Alarm`] and raise `TIMER0_IRQ_0` to `TIMER0_IRQ_3`.
//!
//! This module holds the register layout and the three decisions the driver
//! makes, each a pure function tested on the host:
//!
//! * [`select_time`]: which of two raw samples is a consistent 64-bit time;
//! * [`plan_arm`]: whether a target is due, too far ahead, or armable, and
//!   the 32-bit compare value;
//! * [`after_arm`]: whether an alarm just written is pending, was missed,
//!   or failed to arm.
//!
//! ## Datasheet sources
//!
//! RP2350 datasheet, build 2025-02-20 (register ICD extract:
//! `docs/icd/rp2350/timer/`):
//!
//! - §12.8.2 (p1180): reading `TIMELR` latches `TIMEHR`; the `TIMERAW*`
//!   registers do not latch
//! - §12.8.3 (p1180): alarms match the low 32 bits; arming by writing `ALARMn`
//! - §12.8.4.1 (p1181): the SDK's race-free 64-bit read from `TIMERAWH`,
//!   `TIMERAWL`, `TIMERAWH`
//! - §12.8.5, Tables 1226 to 1245 (p1184 to p1189): register list and fields
//! - §7.5, Table 534 (p504): `RESETS` bit 23, `TIMER0`
//! - §3.2, Table 94 (p82): `TIMER0_IRQ_0..3` are lines 0 to 3

pub mod timer;

use core::mem::offset_of;

/// TIMER0 register block, base `0x400b_0000` (Table 1226).
#[repr(C)]
struct TimerRegs {
    /// `TIMEHW` `0x00`, `TIMELW` `0x04`: write the time. Never written here.
    timew: [u32; 2],
    /// `TIMEHR` `0x08`, `TIMELR` `0x0c`: latching read. Not used (§12.8.4.1).
    timer: [u32; 2],
    /// `ALARM0..3` `0x10` to `0x1c`: writing arms; match on `TIMELR` (Tables 1231 to 1234).
    alarm: [u32; 4],
    /// `ARMED` `0x20`: bits 3:0, write 1 to disarm (WC) (Table 1235).
    armed: u32,
    /// `TIMERAWH` `0x24`: bits 63:32, no side effect (Table 1236).
    timerawh: u32,
    /// `TIMERAWL` `0x28`: bits 31:0, no side effect (Table 1237).
    timerawl: u32,
    /// `DBGPAUSE` `0x2c`: `DBG1` bit 2, `DBG0` bit 1, reset `0x6` (Table 1238).
    dbgpause: u32,
    /// `PAUSE` `0x30`: bit 0 (Table 1239).
    pause: u32,
    /// `LOCKED` `0x34`: bit 0, write-once. Never written here (Table 1240).
    locked: u32,
    /// `SOURCE` `0x38`: bit 0, `0` = tick, `1` = `clk_sys` (Table 1241).
    source: u32,
    /// `INTR` `0x3c`: raw alarm interrupts 3:0, write 1 to clear (Table 1242).
    intr: u32,
    /// `INTE` `0x40`: alarm interrupt enables 3:0 (Table 1243).
    inte: u32,
}

pub(crate) const ALARM0: usize = offset_of!(TimerRegs, alarm);
pub(crate) const ARMED: usize = offset_of!(TimerRegs, armed);
pub(crate) const TIMERAWH: usize = offset_of!(TimerRegs, timerawh);
pub(crate) const TIMERAWL: usize = offset_of!(TimerRegs, timerawl);
pub(crate) const DBGPAUSE: usize = offset_of!(TimerRegs, dbgpause);
pub(crate) const PAUSE: usize = offset_of!(TimerRegs, pause);
pub(crate) const SOURCE: usize = offset_of!(TimerRegs, source);
pub(crate) const INTR: usize = offset_of!(TimerRegs, intr);
pub(crate) const INTE: usize = offset_of!(TimerRegs, inte);
const _: () = assert!(
    ALARM0 == 0x10
        && ARMED == 0x20
        && TIMERAWH == 0x24
        && TIMERAWL == 0x28
        && DBGPAUSE == 0x2c
        && PAUSE == 0x30
        && SOURCE == 0x38
        && INTR == 0x3c
        && INTE == 0x40
);

/// `DBGPAUSE` reset value: pause while either core is halted by a debugger (Table 1238).
pub(crate) const DBGPAUSE_BOTH: u32 = 0b110;
/// `SOURCE` value `TICK` (Table 1241).
pub(crate) const SOURCE_TICK: u32 = 0;
/// The four alarm bits of `ARMED`, `INTR` and `INTE`.
pub(crate) const ALL_ALARMS: u32 = 0b1111;
/// `RESETS.RESET` bit of TIMER0 (Table 534).
pub(crate) const RESET_BIT_TIMER0: u32 = 1 << 23;
/// Reads allowed for `RESET_DONE` after the release. `clk_sys` already runs
/// at 150 MHz, and the release takes a few cycles; the bound only turns a
/// block that never reports ready into [`timer::TimerError::ResetTimeout`].
pub(crate) const RESET_POLL_BUDGET: u32 = 10_000;

/// The consistent 64-bit time from the raw sequence `hi1`, `lo1`, `hi2`, `lo2`
/// read in that order. Pure.
///
/// If the high word did not change between its two reads, `lo1` was read
/// while it held and `(hi1, lo1)` is consistent. If it changed, the low word
/// wrapped between the reads, and `(hi2, lo2)` is consistent: the next change
/// of the high word is 2^32 µs (71 minutes) later. This is the SDK's
/// `timer_time_us_64` of §12.8.4.1 with its loop unrolled to the one retry it
/// can ever take.
#[must_use]
pub(crate) fn select_time(hi1: u32, lo1: u32, hi2: u32, lo2: u32) -> u64 {
    let (hi, lo) = if hi1 == hi2 { (hi1, lo1) } else { (hi2, lo2) };
    (u64::from(hi) << 32) | u64::from(lo)
}

/// What to do with an alarm target, decided from the time read just before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArmPlan {
    /// The target is not in the future: report `Scheduled::Due`.
    Due,
    /// The target is more than `2^32 - 1` µs ahead; the 32-bit comparator
    /// would fire a wrap early (§12.8.3).
    TooFar,
    /// Write this value to `ALARMn`: the low 32 bits of the target.
    Arm(u32),
}

/// Largest `at - now` an alarm can arm: the comparator sees 32 bits (§12.8.3).
pub const MAX_SPAN_US: u64 = 0xffff_ffff;

/// Decide [`ArmPlan`] for target `at` given the time `now`. Pure.
#[must_use]
pub(crate) const fn plan_arm(now: u64, at: u64) -> ArmPlan {
    if at <= now {
        return ArmPlan::Due;
    }
    if at - now > MAX_SPAN_US {
        return ArmPlan::TooFar;
    }
    // The low 32 bits of the target, without a numeric cast (cwht CS-15).
    let [b0, b1, b2, b3, ..] = at.to_le_bytes();
    ArmPlan::Arm(u32::from_le_bytes([b0, b1, b2, b3]))
}

/// How an alarm stands right after `ALARMn` was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AfterArm {
    /// Armed and not yet due, or already fired and latched in `INTR`.
    Pending,
    /// Still armed although the time is past the target: the comparator
    /// missed the match while the value was being written, and would next
    /// match 2^32 µs later. The driver disarms and reports `Due`.
    Missed,
    /// Not armed and nothing latched: the hardware refused the arm.
    Fault,
}

/// Decide [`AfterArm`] from `ARMED` and `INTR` bits of the alarm and the time
/// `now` read after the write. Pure.
///
/// | armed | intr | `now > at` | result |
/// |-------|------|------------|--------|
/// | 1 | any | 0 | `Pending` |
/// | 1 | any | 1 | `Missed` |
/// | 0 | 1 | any | `Pending` (fired) |
/// | 0 | 0 | any | `Fault` |
#[must_use]
pub(crate) const fn after_arm(armed: bool, intr: bool, now: u64, at: u64) -> AfterArm {
    if armed {
        if now > at {
            AfterArm::Missed
        } else {
            AfterArm::Pending
        }
    } else if intr {
        AfterArm::Pending
    } else {
        AfterArm::Fault
    }
}

#[cfg(test)]
mod tests;
