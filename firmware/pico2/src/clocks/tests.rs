//! Host tests of the parent module (split out to keep each file under the
//! 500-line limit of cwht coding standard CS-18).

use super::*;

const GOOD: ClockConfig = ClockConfig::PICO2_150_MHZ;

fn with_pll(refdiv: u32, fbdiv: u32, postdiv1: u32, postdiv2: u32) -> ClockConfig {
    ClockConfig {
        pll_sys: PllConfig {
            refdiv,
            fbdiv,
            postdiv1,
            postdiv2,
        },
        ..GOOD
    }
}

#[test]
fn pico2_plan_matches_the_datasheet_numbers() {
    let p = plan(&GOOD).unwrap();
    assert_eq!(p.xosc_startup_delay, 47); // 12 000 / 256 = 46.875, rounded up (§8.2.4)
    assert_eq!(p.tick_cycles, 12); // "For a 12 MHz reference clock, set the cycle count to 12" (§8.5.1)
    assert_eq!(p.vco_hz, 1_500_000_000);
    assert_eq!(p.clk_sys_hz, 150_000_000);
    assert_eq!(p.pll_cs, 1);
    assert_eq!(p.pll_fbdiv, 125);
    assert_eq!(p.pll_prim, 0x0005_2000);
    assert_eq!(p.fc_ref_khz, 12_000);
    assert_eq!((p.fc_min_khz, p.fc_max_khz), (148_500, 151_500));
}

#[test]
fn xosc_range_edges() {
    assert_eq!(
        plan(&ClockConfig {
            xosc_hz: 999_999,
            ..GOOD
        }),
        Err(ClockConfigError::XoscOutOfRange)
    );
    assert_eq!(
        plan(&ClockConfig {
            xosc_hz: 16_000_000,
            ..GOOD
        }),
        Err(ClockConfigError::XoscOutOfRange)
    );
    assert!(
        plan(&ClockConfig {
            xosc_hz: 15_000_000,
            pll_sys: PllConfig {
                refdiv: 1,
                fbdiv: 100,
                postdiv1: 5,
                postdiv2: 2
            },
            ..GOOD
        })
        .is_ok()
    );
    assert_eq!(
        plan(&ClockConfig {
            xosc_hz: 12_500_000,
            ..GOOD
        }),
        Err(ClockConfigError::XoscNotWholeMhz)
    );
}

#[test]
fn refdiv_limits_and_reference_floor() {
    assert_eq!(
        plan(&with_pll(0, 125, 5, 2)),
        Err(ClockConfigError::RefdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(64, 125, 5, 2)),
        Err(ClockConfigError::RefdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(3, 125, 5, 2)),
        Err(ClockConfigError::PllRefTooLow)
    ); // 4 MHz
    assert!(plan(&with_pll(2, 250, 5, 2)).is_ok()); // 6 MHz × 250 = 1500 MHz
}

#[test]
fn fbdiv_and_vco_limits() {
    assert_eq!(
        plan(&with_pll(1, 15, 1, 1)),
        Err(ClockConfigError::FbdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(1, 321, 5, 2)),
        Err(ClockConfigError::FbdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(1, 62, 5, 2)),
        Err(ClockConfigError::VcoOutOfRange)
    ); // 744 MHz
    assert_eq!(
        plan(&with_pll(1, 134, 5, 2)),
        Err(ClockConfigError::VcoOutOfRange)
    ); // 1608 MHz
    assert!(plan(&with_pll(1, 63, 7, 1)).is_ok()); // 756 MHz / 7 = 108 MHz
    assert_eq!(
        plan(&with_pll(1, 131, 7, 2)),
        Err(ClockConfigError::ClkSysOutOfRange)
    ); // 1572 / 14 not whole
    assert!(plan(&with_pll(1, 133, 7, 2)).is_ok()); // 1596 MHz, the top of the range for 12 MHz
}

#[test]
fn postdiv_limits_and_output_ceiling() {
    assert_eq!(
        plan(&with_pll(1, 125, 0, 2)),
        Err(ClockConfigError::PostdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(1, 125, 5, 8)),
        Err(ClockConfigError::PostdivOutOfRange)
    );
    assert_eq!(
        plan(&with_pll(1, 125, 5, 1)),
        Err(ClockConfigError::ClkSysOutOfRange)
    ); // 300 MHz
    assert_eq!(
        plan(&with_pll(1, 125, 6, 2)).map(|p| p.clk_sys_hz),
        Ok(125_000_000)
    );
}

#[test]
fn tolerance_and_budget_limits() {
    assert_eq!(
        plan(&ClockConfig {
            tolerance_permille: 0,
            ..GOOD
        }),
        Err(ClockConfigError::ToleranceOutOfRange)
    );
    assert_eq!(
        plan(&ClockConfig {
            tolerance_permille: 101,
            ..GOOD
        }),
        Err(ClockConfigError::ToleranceOutOfRange)
    );
    assert!(
        plan(&ClockConfig {
            tolerance_permille: 100,
            ..GOOD
        })
        .is_ok()
    );
    assert_eq!(
        plan(&ClockConfig {
            poll_budget: 0,
            ..GOOD
        }),
        Err(ClockConfigError::PollBudgetZero)
    );
}

// MC/DC for `judge`: the decision is `hw_pass && lo <= khz && khz <= hi`.
// Each condition is shown to change the outcome with the others held true.
const R150: u32 = 150_000 << 5;

#[test]
fn judge_passes_only_with_flag_and_in_window() {
    assert_eq!(
        judge(FC0_STATUS_PASS, R150, 148_500, 151_500),
        Measurement::Pass(150_000)
    );
}

#[test]
fn judge_fails_without_the_hardware_flag() {
    assert_eq!(
        judge(FC0_STATUS_DONE, R150, 148_500, 151_500),
        Measurement::Fail(150_000)
    );
}

#[test]
fn judge_fails_below_the_window() {
    assert_eq!(
        judge(FC0_STATUS_PASS, 148_499 << 5, 148_500, 151_500),
        Measurement::Fail(148_499)
    );
    assert_eq!(
        judge(FC0_STATUS_PASS, 148_500 << 5, 148_500, 151_500),
        Measurement::Pass(148_500)
    );
}

#[test]
fn judge_fails_above_the_window() {
    assert_eq!(
        judge(FC0_STATUS_PASS, 151_501 << 5, 148_500, 151_500),
        Measurement::Fail(151_501)
    );
    assert_eq!(
        judge(FC0_STATUS_PASS, (151_500 << 5) | 0x1f, 148_500, 151_500),
        Measurement::Pass(151_500)
    );
}
