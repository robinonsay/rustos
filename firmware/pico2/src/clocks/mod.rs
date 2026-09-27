//! Clock tree bring-up: crystal oscillator, system PLL, the `clk_ref`,
//! `clk_sys` and `clk_peri` generators, and the 1 µs tick generators.
//!
//! After the bootrom hands over, the RP2350 runs from its ring oscillator
//! (ROSC), whose frequency "varies with PVT" (datasheet §8.1.1.2): nothing
//! timed from it is accurate. [`clocks::Rp2350Clocks::init`] moves the chip
//! onto the 12 MHz crystal, runs `clk_sys` at 150 MHz from `PLL_SYS`, starts
//! the TIMER0 and watchdog tick generators at 1 µs, and then measures
//! `clk_sys` and `clk_peri` against the crystal with the frequency counter
//! before it reports success. Every wait on the hardware is bounded by a loop
//! count, so a crystal that never starts or a PLL that never locks ends in a
//! [`ClockFault`], never in a hang.
//!
//! The register layouts below give the offsets the driver uses; the offsets
//! are taken with `offset_of!` and checked against the datasheet tables at
//! compile time. The sequence itself lives in [`clocks`] and runs against the
//! [`Regs`](crate::common::reg::Regs) capability, so the whole of it, every
//! decision included, is exercised by host tests against a scripted register
//! file.
//!
//! ## Datasheet sources
//!
//! RP2350 datasheet (`docs/rp2350-datasheet.pdf`, build 2025-02-20), by
//! section and PDF page:
//!
//! - §8.1.2 and §8.1.5.1 (p516, p522): clock generators and the configuration order
//! - §8.1.3 and §8.1.5.2 (p517, p526): frequency counter
//! - §8.1.6, Tables 542, 555 to 563, 578 to 585 (p521 to p544): `CLOCKS` registers
//! - §8.2.3, §8.2.4, §8.2.8, Tables 597 to 601 (p554 to p557): `XOSC`
//! - §8.5, Tables 616 to 618 (p567, p568): `TICKS`
//! - §8.6.3, §8.6.4, §8.6.5, Tables 635 to 639 (p572 to p582): `PLL_SYS`
//! - §7.5, Table 534 (p504): `RESETS` bit 14, `PLL_SYS`
//!
//! The July 2025 datasheet revision changed the documented reset values of
//! `CLK_SYS_CTRL.SRC` and `CLK_SYS_CTRL.AUXSRC` for stepping A3, so this driver
//! writes every field of every control register it uses and relies on no
//! reset value.

pub mod clocks;
mod regs;

pub(crate) use regs::*;

/// Frequency limits the datasheet places on the PLL and the clock tree, in Hz.
pub mod limits {
    /// Lowest PLL reference `FREF / REFDIV` (§8.6.3).
    pub const PLL_REF_MIN_HZ: u32 = 5_000_000;
    /// VCO range (§8.6.3: "750 MHz-1600 MHz").
    pub const PLL_VCO_MIN_HZ: u32 = 750_000_000;
    /// Upper end of the VCO range (§8.6.3).
    pub const PLL_VCO_MAX_HZ: u32 = 1_600_000_000;
    /// Highest `clk_sys` the system PLL may produce (§8.6.3: "For the system PLL this is 150 MHz").
    pub const CLK_SYS_MAX_HZ: u32 = 150_000_000;
    /// Crystal range of `XOSC.CTRL.FREQ_RANGE = 1_15MHZ` (Table 598).
    pub const XOSC_MIN_HZ: u32 = 1_000_000;
    /// Upper end of the `1_15MHZ` range (Table 598).
    pub const XOSC_MAX_HZ: u32 = 15_000_000;
}

/// `PLL_SYS` settings: `FOUTPOSTDIV = (FREF / REFDIV) × FBDIV / (POSTDIV1 × POSTDIV2)` (§8.6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PllConfig {
    /// Reference divider, 1 to 63 (`CS.REFDIV`, Table 636).
    pub refdiv: u32,
    /// Feedback divider, 16 to 320 (`FBDIV_INT`, Table 638).
    pub fbdiv: u32,
    /// First post divider, 1 to 7 (`PRIM.POSTDIV1`, Table 639).
    pub postdiv1: u32,
    /// Second post divider, 1 to 7 (`PRIM.POSTDIV2`, Table 639).
    pub postdiv2: u32,
}

/// What [`clocks::Rp2350Clocks::init`] is asked to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockConfig {
    /// Crystal frequency in Hz. Must be a whole number of MHz in 1 to 15 MHz,
    /// so that the tick generators can divide it to exactly 1 µs.
    pub xosc_hz: u32,
    /// `PLL_SYS` dividers.
    pub pll_sys: PllConfig,
    /// Accepted deviation of the measured `clk_sys` and `clk_peri` from the
    /// planned value, in parts per thousand (1 to 100).
    pub tolerance_permille: u32,
    /// Maximum number of register reads spent on any one hardware wait before
    /// the wait is declared failed. Not a time: the bound holds even when the
    /// clock being waited for never starts.
    pub poll_budget: u32,
}

impl ClockConfig {
    /// The Pico 2 configuration: 12 MHz crystal (Pico 2 datasheet §2), `PLL_SYS`
    /// `12 MHz / 1 × 125 = 1500 MHz / 5 / 2 = 150 MHz` (the SDK default of
    /// §8.6.4), measured frequencies accepted within ±1 %, and a poll budget of
    /// 100 000 reads per wait.
    ///
    /// The budget: the slowest wait is the crystal start-up of about 1 ms
    /// (§8.2.4) while the core still runs from the ROSC. A poll iteration is at
    /// least one APB read of three cycles plus loop overhead, so at any ROSC
    /// frequency up to 150 MHz, 100 000 iterations last at least 2 ms, and at
    /// the nominal 11 MHz about 90 ms or more. The margin actually used is
    /// reported in [`clocks::ClockReport::polls`] and measured on the
    /// development board.
    pub const PICO2_150_MHZ: Self = Self {
        xosc_hz: 12_000_000,
        pll_sys: PllConfig {
            refdiv: 1,
            fbdiv: 125,
            postdiv1: 5,
            postdiv2: 2,
        },
        tolerance_permille: 10,
        poll_budget: 100_000,
    };
}

/// Why a [`ClockConfig`] cannot be built. Returned before any register is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockConfigError {
    /// `xosc_hz` is outside 1 to 15 MHz.
    XoscOutOfRange,
    /// `xosc_hz` is not a whole number of MHz, so no tick cycle count gives 1 µs.
    XoscNotWholeMhz,
    /// `refdiv` is outside 1 to 63.
    RefdivOutOfRange,
    /// `xosc_hz / refdiv` is below 5 MHz.
    PllRefTooLow,
    /// `fbdiv` is outside 16 to 320.
    FbdivOutOfRange,
    /// The VCO frequency is outside 750 to 1600 MHz.
    VcoOutOfRange,
    /// A post divider is outside 1 to 7.
    PostdivOutOfRange,
    /// The VCO does not divide evenly to the output, or the output is above 150 MHz.
    ClkSysOutOfRange,
    /// `tolerance_permille` is outside 1 to 100.
    ToleranceOutOfRange,
    /// `poll_budget` is zero.
    PollBudgetZero,
}

/// Register values and checks derived from a valid [`ClockConfig`], computed
/// before any register is touched. Every number [`clocks`] writes comes from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockPlan {
    /// `XOSC.STARTUP.DELAY`: 1 ms of crystal cycles in units of 256, rounded up (§8.2.4).
    pub xosc_startup_delay: u32,
    /// `TICKS.*_CYCLES`: `clk_ref` cycles per 1 µs tick (§8.5.1).
    pub tick_cycles: u32,
    /// Planned `clk_ref` in Hz (the crystal, undivided).
    pub clk_ref_hz: u32,
    /// Planned VCO frequency in Hz.
    pub vco_hz: u32,
    /// Planned `clk_sys` (and `clk_peri`) in Hz.
    pub clk_sys_hz: u32,
    /// `PLL.CS` word: `REFDIV`, `BYPASS = 0`.
    pub pll_cs: u32,
    /// `PLL.FBDIV_INT` word.
    pub pll_fbdiv: u32,
    /// `PLL.PRIM` word: `POSTDIV1 << 16 | POSTDIV2 << 12`.
    pub pll_prim: u32,
    /// `FC0_REF_KHZ`: `clk_ref` in kHz.
    pub fc_ref_khz: u32,
    /// Lowest accepted measurement of `clk_sys`, in kHz.
    pub fc_min_khz: u32,
    /// Highest accepted measurement of `clk_sys`, in kHz.
    pub fc_max_khz: u32,
    /// Poll budget per wait.
    pub poll_budget: u32,
}

/// `true` when `lo <= v <= hi`.
const fn within(v: u32, lo: u32, hi: u32) -> bool {
    v >= lo && v <= hi
}

/// Check a [`ClockConfig`] against the datasheet limits and derive every
/// register value from it. Pure: no register access, host-tested.
///
/// The checks run in a fixed order and the first failure is returned.
///
/// # Errors
///
/// The first [`ClockConfigError`] the configuration violates, in the order
/// the variants are declared.
pub fn plan(config: &ClockConfig) -> Result<ClockPlan, ClockConfigError> {
    let xosc = check_xosc(config.xosc_hz)?;
    let (vco, clk_sys) = check_pll(xosc, &config.pll_sys)?;
    if !within(config.tolerance_permille, 1, 100) {
        return Err(ClockConfigError::ToleranceOutOfRange);
    }
    if config.poll_budget == 0 {
        return Err(ClockConfigError::PollBudgetZero);
    }
    let pll = config.pll_sys;
    let clk_sys_khz = clk_sys / 1000;
    let margin_khz = clk_sys_khz * config.tolerance_permille / 1000;
    Ok(ClockPlan {
        // 1 ms of crystal cycles, in units of 256, rounded up (§8.2.4); at most
        // 15 000 / 256 + 1 = 59, well inside the 14-bit field.
        xosc_startup_delay: (xosc / 1000).div_ceil(256),
        tick_cycles: xosc / 1_000_000,
        clk_ref_hz: xosc,
        vco_hz: vco,
        clk_sys_hz: clk_sys,
        pll_cs: pll.refdiv,
        pll_fbdiv: pll.fbdiv,
        pll_prim: (pll.postdiv1 << 16) | (pll.postdiv2 << 12),
        fc_ref_khz: xosc / 1000,
        fc_min_khz: clk_sys_khz - margin_khz,
        fc_max_khz: clk_sys_khz + margin_khz,
        poll_budget: config.poll_budget,
    })
}

/// The crystal checks of [`plan`]: range of `FREQ_RANGE = 1_15MHZ`, and a
/// whole number of MHz for the 1 µs tick.
fn check_xosc(xosc: u32) -> Result<u32, ClockConfigError> {
    if !within(xosc, limits::XOSC_MIN_HZ, limits::XOSC_MAX_HZ) {
        return Err(ClockConfigError::XoscOutOfRange);
    }
    if !xosc.is_multiple_of(1_000_000) {
        return Err(ClockConfigError::XoscNotWholeMhz);
    }
    Ok(xosc)
}

/// The PLL checks of [`plan`] (§8.6.3): returns the VCO and output frequencies.
fn check_pll(xosc: u32, pll: &PllConfig) -> Result<(u32, u32), ClockConfigError> {
    if !within(pll.refdiv, 1, 63) {
        return Err(ClockConfigError::RefdivOutOfRange);
    }
    let pll_ref = xosc / pll.refdiv;
    if pll_ref < limits::PLL_REF_MIN_HZ {
        return Err(ClockConfigError::PllRefTooLow);
    }
    if !within(pll.fbdiv, 16, 320) {
        return Err(ClockConfigError::FbdivOutOfRange);
    }
    // pll_ref <= 15 MHz and fbdiv <= 320: the product is at most 4.8 GHz, which
    // fits in u64; anything that does not fit in u32 is above the VCO range.
    let Ok(vco) = u32::try_from(u64::from(pll_ref) * u64::from(pll.fbdiv)) else {
        return Err(ClockConfigError::VcoOutOfRange);
    };
    if !within(vco, limits::PLL_VCO_MIN_HZ, limits::PLL_VCO_MAX_HZ) {
        return Err(ClockConfigError::VcoOutOfRange);
    }
    if !within(pll.postdiv1, 1, 7) || !within(pll.postdiv2, 1, 7) {
        return Err(ClockConfigError::PostdivOutOfRange);
    }
    let post = pll.postdiv1 * pll.postdiv2;
    if !vco.is_multiple_of(post) || vco / post > limits::CLK_SYS_MAX_HZ {
        return Err(ClockConfigError::ClkSysOutOfRange);
    }
    Ok((vco, vco / post))
}

/// Which step of the bring-up failed. The steps are in the order they run;
/// a fault stops the sequence, and every later step is left undone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockFault {
    /// The configuration failed [`plan`]; no register was written.
    Config(ClockConfigError),
    /// `XOSC.STATUS.STABLE` did not set within the poll budget.
    XoscNotStable,
    /// `CLK_SYS_SELECTED` did not report `clk_ref` after the switch away from the PLL.
    ClkSysToRefTimeout,
    /// `CLK_REF_SELECTED` did not report the crystal after the switch.
    ClkRefToXoscTimeout,
    /// `RESETS.RESET_DONE` did not report `PLL_SYS` out of reset.
    PllResetTimeout,
    /// `PLL_SYS.CS.LOCK` did not set.
    PllLockTimeout,
    /// `CLK_SYS_SELECTED` did not report the auxiliary (PLL) source.
    ClkSysToPllTimeout,
    /// `CLK_PERI_CTRL.ENABLED` did not clear after the generator was stopped.
    ClkPeriStopTimeout,
    /// `CLK_PERI_CTRL.ENABLED` did not set after the generator was started.
    ClkPeriStartTimeout,
    /// `TICKS.TIMER0_CTRL.RUNNING` did not set.
    Timer0TickNotRunning,
    /// `TICKS.WATCHDOG_CTRL.RUNNING` did not set.
    WatchdogTickNotRunning,
    /// The frequency counter did not finish measuring `clk_sys`.
    ClkSysMeasureTimeout,
    /// The measured `clk_sys`, in kHz, is outside the planned tolerance.
    ClkSysOutOfTolerance(u32),
    /// The frequency counter did not finish measuring `clk_peri`.
    ClkPeriMeasureTimeout,
    /// The measured `clk_peri`, in kHz, is outside the planned tolerance.
    ClkPeriOutOfTolerance(u32),
}

/// Outcome of one frequency-counter measurement, decided from the two
/// registers read after `FC0_STATUS.DONE` (Tables 584, 585).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Measurement {
    /// Inside `[min_khz, max_khz]` by both the hardware verdict and the result.
    Pass(u32),
    /// Outside the window by either the hardware verdict or the result.
    Fail(u32),
}

/// Judge a frequency-counter measurement. Pure, host-tested.
///
/// Passes only when the hardware's own `PASS` flag is set **and** the
/// `KHZ` field of `FC0_RESULT` lies in `[min_khz, max_khz]`: the two are
/// computed independently (the flag by the counter's comparators against
/// `FC0_MIN_KHZ` and `FC0_MAX_KHZ`, the window by this function), so either
/// one alone disagreeing fails the check.
pub(crate) const fn judge(status: u32, result: u32, min_khz: u32, max_khz: u32) -> Measurement {
    let khz = (result >> 5) & 0x01ff_ffff;
    let hw_pass = status & FC0_STATUS_PASS != 0;
    if hw_pass && within(khz, min_khz, max_khz) {
        Measurement::Pass(khz)
    } else {
        Measurement::Fail(khz)
    }
}

#[cfg(test)]
mod tests;
