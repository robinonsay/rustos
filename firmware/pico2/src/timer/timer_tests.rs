//! Host tests of the TIMER0 register sequences against the scripted register
//! file.

extern crate std;
use std::vec;

use super::*;
use crate::common::reg::fake::FakeRegs;

const T: RegAddr = RegAddr::TIMER0;

/// A register file whose raw time reads `hi:lo` and stays there.
fn at_time(t: u64) -> FakeRegs {
    let mut r = FakeRegs::new();
    r.set(T, TIMERAWH, (t >> 32) as u32);
    r.set(T, TIMERAWL, t as u32);
    r
}

#[test]
fn release_waits_for_reset_done_then_sets_every_control_register() {
    let mut regs = FakeRegs::new();
    regs.set(RegAddr::RESET, RESET_DONE_OFFSET, RESET_BIT_TIMER0);
    assert_eq!(release(&mut regs), Ok(()));
    assert_eq!(
        regs.writes(),
        vec![
            (RegAddr::RESET, 0x3000, 1 << 23),
            (T, SOURCE, 0),
            (T, PAUSE, 0),
            (T, DBGPAUSE, 0b110),
            (T, INTE, 0),
            (T, ARMED, 0b1111),
            (T, INTR, 0b1111),
        ]
    );
}

#[test]
fn release_times_out_and_leaves_timer_registers_alone() {
    let mut regs = FakeRegs::new();
    assert_eq!(release(&mut regs), Err(TimerError::ResetTimeout));
    assert_eq!(
        regs.reads_of(RegAddr::RESET, RESET_DONE_OFFSET),
        RESET_POLL_BUDGET as usize
    );
    assert!(!regs.wrote_block(T));
}

#[test]
fn read_time_reads_high_low_high_low() {
    let mut regs = FakeRegs::new();
    regs.script_reads(T, TIMERAWH, &[6, 7]);
    regs.script_reads(T, TIMERAWL, &[0xffff_ffff, 3]);
    assert_eq!(read_time(&mut regs), 0x7_0000_0003);
    assert_eq!(
        (regs.reads_of(T, TIMERAWH), regs.reads_of(T, TIMERAWL)),
        (2, 2)
    );
}

#[test]
fn schedule_in_the_future_disarms_then_enables_then_arms() {
    let mut regs = at_time(1_000);
    regs.set(T, ARMED, 1 << 1);
    assert_eq!(schedule(&mut regs, 1, 1_500), Ok(Scheduled::Armed));
    assert_eq!(
        regs.writes(),
        vec![
            (T, ARMED, 1 << 1),
            (T, INTR, 1 << 1),
            (T, INTE + 0x2000, 1 << 1),
            (T, ALARM0 + 4, 1_500)
        ]
    );
}

#[test]
fn schedule_now_or_past_is_due_and_leaves_the_alarm_disarmed() {
    for at in [1_000, 999, 0] {
        let mut regs = at_time(1_000);
        assert_eq!(schedule(&mut regs, 0, at), Ok(Scheduled::Due));
        assert_eq!(regs.writes(), vec![(T, ARMED, 1), (T, INTR, 1)]);
    }
}

#[test]
fn schedule_too_far_is_an_error_and_leaves_the_alarm_disarmed() {
    let mut regs = at_time(10);
    assert_eq!(
        schedule(&mut regs, 2, 10 + (1u64 << 32)),
        Err(AlarmError::TooFar)
    );
    assert_eq!(regs.writes(), vec![(T, ARMED, 1 << 2), (T, INTR, 1 << 2)]);
}

#[test]
fn schedule_that_fires_during_arming_is_armed_with_the_fire_latched() {
    let mut regs = FakeRegs::new();
    regs.set(T, TIMERAWH, 0);
    regs.script_reads(T, TIMERAWL, &[99, 99, 101, 101]);
    regs.script_reads(T, ARMED, &[0]);
    regs.script_reads(T, INTR, &[1 << 3]);
    assert_eq!(schedule(&mut regs, 3, 100), Ok(Scheduled::Armed));
}

#[test]
fn schedule_that_missed_the_match_is_due_and_disarmed() {
    let mut regs = FakeRegs::new();
    regs.set(T, TIMERAWH, 0);
    regs.script_reads(T, TIMERAWL, &[99, 99, 101, 101]);
    regs.script_reads(T, ARMED, &[1]);
    assert_eq!(schedule(&mut regs, 0, 100), Ok(Scheduled::Due));
    let w = regs.writes();
    assert_eq!(&w[w.len() - 2..], &[(T, ARMED, 1), (T, INTR, 1)]);
}

#[test]
fn schedule_still_armed_when_the_time_equals_the_target_is_due_and_disarmed() {
    let mut regs = FakeRegs::new();
    regs.set(T, TIMERAWH, 0);
    regs.script_reads(T, TIMERAWL, &[99, 99, 100, 100]);
    regs.script_reads(T, ARMED, &[1]);
    assert_eq!(schedule(&mut regs, 0, 100), Ok(Scheduled::Due));
    let w = regs.writes();
    assert_eq!(&w[w.len() - 2..], &[(T, ARMED, 1), (T, INTR, 1)]);
}

#[test]
fn schedule_that_did_not_arm_is_an_error_and_disarmed() {
    let mut regs = at_time(1_000);
    regs.script_reads(T, ARMED, &[0]);
    regs.script_reads(T, INTR, &[0]);
    assert_eq!(schedule(&mut regs, 1, 5_000), Err(AlarmError::NotArmed));
    let w = regs.writes();
    assert_eq!(&w[w.len() - 2..], &[(T, ARMED, 1 << 1), (T, INTR, 1 << 1)]);
}

#[test]
fn clear_fired_clears_only_a_latch_it_saw() {
    let mut regs = FakeRegs::new();
    regs.script_reads(T, INTR, &[0b0100, 0]);
    assert!(clear_fired(&mut regs, 2));
    assert!(!clear_fired(&mut regs, 2));
    assert_eq!(regs.writes(), vec![(T, INTR, 0b0100)]);
}

#[test]
fn clear_fired_ignores_other_alarms_latches() {
    let mut regs = FakeRegs::new();
    regs.set(T, INTR, 0b1011);
    assert!(!clear_fired(&mut regs, 2));
    assert!(regs.writes().is_empty());
}

#[test]
fn armed_reads_its_own_bit() {
    let mut regs = FakeRegs::new();
    regs.set(T, ARMED, 0b0010);
    assert!(armed(&mut regs, 1));
    assert!(!armed(&mut regs, 0));
}

#[test]
fn alarms_map_to_timer0_irq_lines_0_to_3() {
    assert_eq!(Rp2350Alarm::<0>::irq(), Irq::TIMER0_IRQ_0);
    assert_eq!(Rp2350Alarm::<1>::irq(), Irq::TIMER0_IRQ_1);
    assert_eq!(Rp2350Alarm::<2>::irq(), Irq::TIMER0_IRQ_2);
    assert_eq!(Rp2350Alarm::<3>::irq(), Irq::TIMER0_IRQ_3);
}

#[test]
fn split_yields_one_clock_and_four_alarms() {
    let timer = Rp2350Timer0 { _private: () };
    let (_clock, _a0, _a1, _a2, _a3) = timer.split();
}
