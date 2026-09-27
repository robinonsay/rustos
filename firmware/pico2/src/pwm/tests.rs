//! Host tests of the PWM decisions, with MC/DC pairs for each condition.

use super::*;

const CLK: u32 = 150_000_000;

#[test]
fn sidetone_700_hz() {
    let t = timing(CLK, 700).unwrap();
    assert_eq!((t.div16, t.period), (53, 64_690)); // DIV 3.3125, TOP 64689
    assert!(t.achieved_mhz.abs_diff(700_000) <= 1);
}

#[test]
fn backlight_20_khz_uses_divider_one() {
    assert_eq!(
        timing(CLK, 20_000),
        Ok(Timing {
            div16: 16,
            period: 7_500,
            achieved_mhz: 20_000_000
        })
    );
}

#[test]
fn every_audio_frequency_is_within_the_contract() {
    for hz in (100..=3_000).step_by(7) {
        let t = timing(CLK, hz).unwrap();
        assert!(
            t.achieved_mhz.abs_diff(u64::from(hz) * 1000) <= u64::from(hz),
            "{hz} Hz"
        );
        assert!((MIN_DIV16..=MAX_DIV16).contains(&t.div16));
        assert!((MIN_PERIOD..=MAX_PERIOD).contains(&t.period));
    }
}

#[test]
fn zero_is_refused() {
    assert_eq!(timing(CLK, 0), Err(PwmError::FrequencyOutOfRange));
}

#[test]
fn lowest_frequency_edge_is_the_divider_limit() {
    assert!(timing(CLK, 9).is_ok()); // divider 4070/16
    assert_eq!(timing(CLK, 8), Err(PwmError::FrequencyOutOfRange)); // would need 4578/16
}

#[test]
fn highest_frequency_edge_is_a_two_count_period() {
    assert_eq!(timing(CLK, 75_000_000).map(|t| t.period), Ok(2));
    assert_eq!(timing(CLK, u32::MAX), Err(PwmError::FrequencyOutOfRange)); // period rounds to 0
}

#[test]
fn inexact_high_frequency_is_refused() {
    // 7 MHz needs 21.43 counts: 21 gives 7.143 MHz, 2 % off.
    assert_eq!(timing(CLK, 7_000_000), Err(PwmError::FrequencyOutOfRange));
    assert!(timing(CLK, 7_500_000).is_ok()); // exactly 20 counts
}

#[test]
fn cc_value_ends_and_rounding() {
    assert_eq!(cc_value(0, 64_690, true), 0);
    assert_eq!(cc_value(1000, 64_690, true), 64_690); // TOP + 1: 100 %
    assert_eq!(cc_value(500, 64_690, true), 32_345);
    assert_eq!(cc_value(1, 7_500, true), 8); // 7.5 rounds up
}

#[test]
fn cc_value_off_is_zero_whatever_the_duty() {
    assert_eq!(cc_value(1000, 64_690, false), 0);
    assert_eq!(cc_value(500, 7_500, false), 0);
}

#[test]
fn register_offsets() {
    assert_eq!(gpio_ctrl(25), 0x0cc);
    assert_eq!(pad_ctrl(25), 0x68);
    assert_eq!(7 * SLICE_STRIDE + TOP, 0x09c); // CH7_TOP
}
