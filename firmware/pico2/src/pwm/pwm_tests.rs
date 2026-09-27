//! Host tests of the PWM register sequences against the scripted register
//! file.

extern crate std;
use std::vec;

use super::*;
use crate::common::reg::fake::FakeRegs;

const P: RegAddr = RegAddr::PWM;
const CLK: u32 = 150_000_000;

fn out<const N: usize>() -> Rp2350PwmOut<N> {
    let t = timing(CLK, DEFAULT_HZ).unwrap();
    Rp2350PwmOut {
        clk_sys_hz: CLK,
        period: t.period,
        duty_permille: 0,
        on: false,
    }
}

#[test]
fn release_waits_then_disables_all_slices() {
    let mut regs = FakeRegs::new();
    regs.set(RegAddr::RESET, RESET_DONE_OFFSET, RESET_BIT_PWM);
    assert_eq!(release(&mut regs), Ok(()));
    assert_eq!(
        regs.writes(),
        vec![(RegAddr::RESET, 0x3000, 1 << 16), (P, EN, 0)]
    );
}

#[test]
fn release_times_out_without_touching_pwm() {
    let mut regs = FakeRegs::new();
    assert_eq!(release(&mut regs), Err(PwmError::ResetTimeout));
    assert!(!regs.wrote_block(P));
}

#[test]
fn attach_runs_the_slice_at_zero_duty_before_switching_the_pin() {
    let mut regs = FakeRegs::new();
    // GPIO 14 (a sidetone candidate) is slice 7 channel A.
    let (claimed, t) = attach(&mut regs, 0, CLK, 14, 7).unwrap();
    assert_eq!(claimed, 1 << 7);
    assert_eq!((t.div16, t.period), (37, 64_865)); // 1 kHz: DIV 2.3125, TOP 64864
    let b = 7 * SLICE_STRIDE;
    assert_eq!(
        regs.writes(),
        vec![
            (P, b + CSR, 0),
            (P, b + DIV, t.div16),
            (P, b + TOP, t.period - 1),
            (P, b + CC, 0),
            (P, b + CTR, 0),
            (P, b + CSR, 1),
            (RegAddr::IO_BANK0, 14 * 8 + 4, 4),
            (
                RegAddr::PADS_BANK0,
                14 * 4 + 4 + 0x3000,
                (1 << 8) | (1 << 7)
            ),
        ]
    );
}

#[test]
fn attach_refuses_a_claimed_slice_before_any_write() {
    let mut regs = FakeRegs::new();
    assert_eq!(
        attach(&mut regs, 1 << 7, CLK, 15, 7),
        Err(PwmError::SliceInUse)
    );
    assert!(regs.log().is_empty());
}

#[test]
fn pin_to_slice_and_channel_follows_table_1129() {
    assert_eq!(
        (Rp2350PwmOut::<0>::SLICE, Rp2350PwmOut::<0>::CHANNEL),
        (0, 0)
    );
    assert_eq!(
        (Rp2350PwmOut::<15>::SLICE, Rp2350PwmOut::<15>::CHANNEL),
        (7, 1)
    );
    assert_eq!(
        (Rp2350PwmOut::<16>::SLICE, Rp2350PwmOut::<16>::CHANNEL),
        (0, 0)
    );
    assert_eq!(
        (Rp2350PwmOut::<25>::SLICE, Rp2350PwmOut::<25>::CHANNEL),
        (4, 1)
    );
    assert_eq!(
        (Rp2350PwmOut::<29>::SLICE, Rp2350PwmOut::<29>::CHANNEL),
        (6, 1)
    );
}

#[test]
fn duty_and_enable_write_only_cc_in_the_right_half() {
    let mut regs = FakeRegs::new();
    let mut a = out::<14>(); // slice 7, channel A
    a.update_on(&mut regs, 500, true);
    let mut b = out::<25>(); // slice 4, channel B
    b.update_on(&mut regs, 1000, true);
    b.update_on(&mut regs, 1000, false);
    assert_eq!(
        regs.writes(),
        vec![
            (P, 7 * SLICE_STRIDE + CC, 32_433),
            (P, 4 * SLICE_STRIDE + CC, 64_865 << 16),
            (P, 4 * SLICE_STRIDE + CC, 0)
        ]
    );
}

#[test]
fn frequency_change_keeps_the_duty_and_rescales_cc() {
    let mut regs = FakeRegs::new();
    let mut o = out::<14>();
    o.update_on(&mut regs, 250, true);
    let mhz = o.frequency_on(&mut regs, 700).unwrap();
    assert!(mhz.abs_diff(700_000) <= 700);
    let w = regs.writes();
    let b = 7 * SLICE_STRIDE;
    assert_eq!(
        &w[1..],
        &[(P, b + DIV, 53), (P, b + TOP, 64_689), (P, b + CC, 16_173)]
    );
    assert_eq!((o.period, o.duty_permille, o.on), (64_690, 250, true));
}

#[test]
fn refused_frequency_changes_nothing() {
    let mut regs = FakeRegs::new();
    let mut o = out::<14>();
    o.update_on(&mut regs, 300, true);
    let before = (o.period, o.duty_permille, o.on);
    assert_eq!(
        o.frequency_on(&mut regs, 0),
        Err(PwmError::FrequencyOutOfRange)
    );
    assert_eq!(regs.writes().len(), 1);
    assert_eq!((o.period, o.duty_permille, o.on), before);
}
