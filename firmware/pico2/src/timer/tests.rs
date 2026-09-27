//! Host tests of the timer decisions, with MC/DC independence pairs for each
//! boolean decision (cwht 07 section 9.6).

use super::*;

#[test]
fn select_time_keeps_the_first_sample_when_the_high_word_held() {
    assert_eq!(select_time(7, 0xffff_fff0, 7, 0x0000_0005), 0x7_ffff_fff0);
}

#[test]
fn select_time_takes_the_second_sample_across_a_wrap() {
    // lo wrapped between the two reads of hi: (6, 0xffff_ffff) then (7, 3).
    assert_eq!(select_time(6, 0xffff_ffff, 7, 3), 0x7_0000_0003);
}

#[test]
fn select_time_uses_all_64_bits() {
    assert_eq!(select_time(u32::MAX, u32::MAX, u32::MAX, 0), u64::MAX);
}

// plan_arm: decision 1 `at <= now`, decision 2 `at - now > MAX_SPAN_US`.

#[test]
fn plan_arm_due_at_and_before_now() {
    assert_eq!(plan_arm(1000, 1000), ArmPlan::Due);
    assert_eq!(plan_arm(1000, 999), ArmPlan::Due);
    assert_eq!(plan_arm(1000, 0), ArmPlan::Due);
}

#[test]
fn plan_arm_arms_one_microsecond_ahead() {
    assert_eq!(plan_arm(1000, 1001), ArmPlan::Arm(1001));
}

#[test]
fn plan_arm_span_edge() {
    let now = 0x5_ffff_ff00;
    assert_eq!(
        plan_arm(now, now + MAX_SPAN_US),
        ArmPlan::Arm(((now + MAX_SPAN_US) & 0xffff_ffff) as u32)
    );
    assert_eq!(plan_arm(now, now + MAX_SPAN_US + 1), ArmPlan::TooFar);
}

#[test]
fn plan_arm_compare_value_is_the_low_word_across_a_high_word_boundary() {
    // now just below 2^32, target just above: the comparator value wraps to a
    // small number, which TIMELR reaches after the high word increments.
    assert_eq!(plan_arm(0xffff_fff0, 0x1_0000_0010), ArmPlan::Arm(0x10));
}

// after_arm: conditions armed (A), intr (I), now >= at (L).
// A: (A=1,L=0) Pending vs (A=0,I=0) Fault.  I: (A=0,I=1) Pending vs (A=0,I=0) Fault.
// L: (A=1,L=0) Pending vs (A=1,L=1) Missed.

#[test]
fn after_arm_armed_and_not_late_is_pending() {
    assert_eq!(after_arm(true, false, 100, 200), AfterArm::Pending);
    assert_eq!(after_arm(true, false, 199, 200), AfterArm::Pending);
}

#[test]
fn after_arm_armed_and_late_is_missed() {
    // now == at is the INSP-097 finding-1 edge: still armed in the matching
    // microsecond is treated as a missed match.
    assert_eq!(after_arm(true, false, 200, 200), AfterArm::Missed);
    assert_eq!(after_arm(true, false, 201, 200), AfterArm::Missed);
    assert_eq!(after_arm(true, true, 201, 200), AfterArm::Missed);
}

#[test]
fn after_arm_disarmed_with_a_latched_fire_is_pending() {
    assert_eq!(after_arm(false, true, 250, 200), AfterArm::Pending);
}

#[test]
fn after_arm_disarmed_without_a_fire_is_a_fault() {
    assert_eq!(after_arm(false, false, 100, 200), AfterArm::Fault);
    assert_eq!(after_arm(false, false, 300, 200), AfterArm::Fault);
}
