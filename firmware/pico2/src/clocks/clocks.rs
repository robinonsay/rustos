//! The clock bring-up driver: [`Rp2350Clocks`] and its proof of success,
//! [`ClocksReady`].
//!
//! Bring-up is a boot-time sequence with no ongoing state, so there is no
//! `api` trait for it: an application calls [`Rp2350Clocks::init`] once, as
//! early as possible, and passes the returned [`ClocksReady`] by reference to
//! every driver whose timing depends on the clock tree. Those drivers take
//! `&ClocksReady` in their constructors, so "TIMER0 used before its tick was
//! started" is a compile error, not a wrong timestamp.
//!
//! ## The sequence
//!
//! [`bring_up`] runs these steps in order; each wait is a bounded
//! [`poll`](crate::common::reg::poll) and a failed wait ends the sequence with
//! the [`ClockFault`] that names it.
//!
//! | # | Step | Registers | Source |
//! |---|------|-----------|--------|
//! | 1 | Check the configuration and derive every value | none | [`plan`] |
//! | 2 | Start the crystal: range, start-up delay, enable; wait `STABLE` | `XOSC.CTRL`, `STARTUP`, `STATUS` | §8.2.3, §8.2.4, Tables 598 to 601 |
//! | 3 | Move `clk_sys` to `clk_ref` (off any PLL) and wait for the glitchless mux | `CLK_SYS_CTRL`, `CLK_SYS_SELECTED` | §8.1.5.1, Tables 558, 560 |
//! | 4 | Divide `clk_ref` by 1 and move it to the crystal; wait for the mux | `CLK_REF_DIV`, `CLK_REF_CTRL`, `CLK_REF_SELECTED` | Tables 555 to 557 |
//! | 5 | Reset `PLL_SYS` through the atomic aliases and wait for `RESET_DONE` | `RESETS.RESET`, `RESET_DONE` | §7.5, Table 534 |
//! | 6 | Program `REFDIV` and `FBDIV`, power the PLL and VCO, wait `LOCK`, then set and power the post dividers | `PLL.CS`, `FBDIV_INT`, `PWR`, `PRIM` | §8.6.4 steps 1 to 5 |
//! | 7 | `clk_sys` divider 1, aux source `PLL_SYS`, then the glitchless mux to aux; wait | `CLK_SYS_DIV`, `CLK_SYS_CTRL`, `CLK_SYS_SELECTED` | §8.1.5.1 |
//! | 8 | Stop `clk_peri`, wait until stopped, source `clk_sys` divided by 1, start; wait until running | `CLK_PERI_CTRL`, `CLK_PERI_DIV` | §8.1.5.1, Tables 561, 562 |
//! | 9 | TIMER0 and watchdog ticks: stop, `CYCLES = xosc MHz`, start; wait `RUNNING` | `TICKS.*_CTRL`, `*_CYCLES` | §8.5.1, Tables 617, 618 |
//! | 10 | Measure `clk_sys`, then `clk_peri`, against `clk_ref` over 1 ms and judge each | `FC0_*` | §8.1.3, §8.1.5.2, Tables 578 to 585 |
//!
//! Step 3 matters when the bootrom or an earlier image left `clk_sys` on the
//! PLL: the PLL is reset in step 5, and `clk_sys` must not be running from it
//! then. Step 10 is an independent check, not a datasheet requirement: the
//! frequency counter counts `clk_sys` and `clk_peri` edges over a window timed
//! by the crystal, so a wrong divider or a PLL that locked to the wrong
//! frequency is caught before any timing depends on it.
//!
//! On a fault, the outputs the application drives are untouched by this
//! driver; the application stays in its safe state (cwht coding standard
//! CS-37).

#[cfg(target_os = "none")]
use api::device::DeviceHandle;

use super::{
    CLK_PERI_AUXSRC_CLK_SYS, CLK_PERI_CTRL, CLK_PERI_DIV, CLK_PERI_ENABLE, CLK_PERI_ENABLED,
    CLK_REF_CTRL, CLK_REF_DIV, CLK_REF_DIV_BY_ONE, CLK_REF_SELECTED, CLK_REF_SRC_XOSC,
    CLK_SYS_AUXSRC_PLL_SYS, CLK_SYS_CTRL, CLK_SYS_DIV, CLK_SYS_SELECTED, CLK_SYS_SRC_AUX,
    CLK_SYS_SRC_REF, ClockConfig, ClockFault, ClockPlan, DIV_BY_ONE_INT16, FC0_DELAY,
    FC0_DELAY_ONE, FC0_INTERVAL, FC0_INTERVAL_1MS, FC0_MAX_KHZ, FC0_MIN_KHZ, FC0_REF_KHZ,
    FC0_RESULT, FC0_SRC, FC0_SRC_CLK_PERI, FC0_SRC_CLK_SYS, FC0_SRC_NULL, FC0_STATUS,
    FC0_STATUS_DONE, FC0_STATUS_RUNNING, Measurement, PLL_CS, PLL_CS_LOCK, PLL_FBDIV_INT, PLL_PRIM,
    PLL_PWR, PLL_PWR_DSMPD, PLL_PWR_POSTDIVPD, RESET_BIT_PLL_SYS, RESETS_RESET, RESETS_RESET_DONE,
    TICK_ENABLE, TICK_RUNNING, TICKS_TIMER0_CTRL, TICKS_TIMER0_CYCLES, TICKS_WATCHDOG_CTRL,
    TICKS_WATCHDOG_CYCLES, XOSC_CTRL, XOSC_ENABLE, XOSC_RANGE_1_15MHZ, XOSC_STARTUP, XOSC_STATUS,
    XOSC_STATUS_STABLE, judge, plan,
};
use crate::common::reg::{ALIAS_CLR, ALIAS_SET, RegAddr, Regs, poll};

/// The clock tree as a peripheral: the board's `clocks` device.
///
/// Zero-sized. The only way to use it is [`Rp2350Clocks::init`], which
/// consumes the board's `DeviceHandle<Rp2350Clocks>`, so the bring-up runs at
/// most once per boot.
pub struct Rp2350Clocks {
    _private: (),
}

/// Register reads spent on each bounded wait of the bring-up, in sequence
/// order. Each is at most the configured poll budget; the ratio is the
/// measured margin of that wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PollCounts {
    /// Step 2: `XOSC.STATUS.STABLE`.
    pub xosc_stable: u32,
    /// Step 3: `CLK_SYS_SELECTED` = `clk_ref`.
    pub clk_sys_to_ref: u32,
    /// Step 4: `CLK_REF_SELECTED` = XOSC.
    pub clk_ref_to_xosc: u32,
    /// Step 5: `RESET_DONE` for `PLL_SYS`.
    pub pll_reset: u32,
    /// Step 6: `PLL_SYS.CS.LOCK`.
    pub pll_lock: u32,
    /// Step 7: `CLK_SYS_SELECTED` = aux.
    pub clk_sys_to_pll: u32,
    /// Step 8: `CLK_PERI_CTRL.ENABLED` clear.
    pub clk_peri_stop: u32,
    /// Step 8: `CLK_PERI_CTRL.ENABLED` set.
    pub clk_peri_start: u32,
    /// Step 9: TIMER0 tick running.
    pub timer0_tick: u32,
    /// Step 9: watchdog tick running.
    pub watchdog_tick: u32,
    /// Step 10: `FC0_STATUS.DONE` for `clk_sys`.
    pub measure_clk_sys: u32,
    /// Step 10: `FC0_STATUS.DONE` for `clk_peri`.
    pub measure_clk_peri: u32,
}

/// What the bring-up built and measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockReport {
    /// `clk_ref` in Hz: the crystal, undivided.
    pub clk_ref_hz: u32,
    /// `clk_sys` in Hz as planned.
    pub clk_sys_hz: u32,
    /// `clk_peri` in Hz as planned (equal to `clk_sys`).
    pub clk_peri_hz: u32,
    /// Tick rate of TIMER0 and the watchdog, in Hz (1 MHz).
    pub tick_hz: u32,
    /// `clk_sys` as measured by the frequency counter, in kHz (±2 kHz, Table 541).
    pub measured_clk_sys_khz: u32,
    /// `clk_peri` as measured by the frequency counter, in kHz.
    pub measured_clk_peri_khz: u32,
    /// Reads spent on each wait.
    pub polls: PollCounts,
}

/// Proof that [`Rp2350Clocks::init`] succeeded: the crystal, `PLL_SYS`,
/// `clk_sys`, `clk_peri` and the TIMER0 and watchdog ticks run as reported.
///
/// Neither `Clone` nor `Copy`, and constructed only by the bring-up, so at
/// most one exists per boot. Drivers that need the clock tree borrow it.
pub struct ClocksReady {
    report: ClockReport,
}

impl ClocksReady {
    /// The frequencies built, the measurements, and the poll counts.
    #[must_use]
    pub const fn report(&self) -> &ClockReport {
        &self.report
    }

    /// Planned `clk_sys` in Hz.
    #[must_use]
    pub const fn clk_sys_hz(&self) -> u32 {
        self.report.clk_sys_hz
    }

    /// Planned `clk_peri` in Hz.
    #[must_use]
    pub const fn clk_peri_hz(&self) -> u32 {
        self.report.clk_peri_hz
    }
}

impl Rp2350Clocks {
    /// Bring the clock tree up as `config` describes, consuming the board's
    /// clocks handle.
    ///
    /// Call first thing in `main`, after the application has driven its
    /// outputs to their safe levels: until this returns `Ok`, nothing on the
    /// chip is timed from the crystal. On `Err` the handle is gone and the
    /// bring-up cannot be retried in this boot; the application stays in its
    /// safe state.
    ///
    /// Straight-line: every decision is in [`bring_up`], which the host tests
    /// run against a scripted register file.
    ///
    /// # Errors
    ///
    /// The [`ClockFault`] of the first step that failed; the steps after it
    /// were not run.
    #[cfg(target_os = "none")]
    pub fn init(
        _handle: DeviceHandle<Self>,
        config: &ClockConfig,
    ) -> Result<ClocksReady, ClockFault> {
        bring_up(&mut crate::common::reg::Mmio, config)
    }
}

/// Wait on one register, mapping a timeout to `fault`.
fn wait<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    block: RegAddr,
    offset: usize,
    (mask, want): (u32, u32),
    fault: ClockFault,
) -> Result<u32, ClockFault> {
    poll(regs, block, offset, mask, want, plan.poll_budget).map_err(|_| fault)
}

/// The whole bring-up against any [`Regs`]: the hardware on the target, the
/// scripted register file in tests.
pub(crate) fn bring_up<R: Regs>(
    regs: &mut R,
    config: &ClockConfig,
) -> Result<ClocksReady, ClockFault> {
    let plan = plan(config).map_err(ClockFault::Config)?;
    let mut polls = PollCounts::default();
    start_xosc(regs, &plan, &mut polls)?;
    move_clk_ref_to_xosc(regs, &plan, &mut polls)?;
    start_pll_sys(regs, &plan, &mut polls)?;
    move_clk_sys_to_pll(regs, &plan, &mut polls)?;
    start_clk_peri(regs, &plan, &mut polls)?;
    start_ticks(regs, &plan, &mut polls)?;
    let (sys_khz, peri_khz) = measure(regs, &plan, &mut polls)?;
    Ok(ClocksReady {
        report: ClockReport {
            clk_ref_hz: plan.clk_ref_hz,
            clk_sys_hz: plan.clk_sys_hz,
            clk_peri_hz: plan.clk_sys_hz,
            tick_hz: 1_000_000,
            measured_clk_sys_khz: sys_khz,
            measured_clk_peri_khz: peri_khz,
            polls,
        },
    })
}

/// Step 2 (§8.2.3, §8.2.4): range and start-up delay first, then the enable
/// code, written as whole words; then wait for `STABLE`.
fn start_xosc<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    regs.write(RegAddr::XOSC, XOSC_CTRL, XOSC_RANGE_1_15MHZ);
    regs.write(RegAddr::XOSC, XOSC_STARTUP, plan.xosc_startup_delay);
    regs.write(
        RegAddr::XOSC,
        XOSC_CTRL,
        (XOSC_ENABLE << 12) | XOSC_RANGE_1_15MHZ,
    );
    let stable = (XOSC_STATUS_STABLE, XOSC_STATUS_STABLE);
    polls.xosc_stable = wait(
        regs,
        plan,
        RegAddr::XOSC,
        XOSC_STATUS,
        stable,
        ClockFault::XoscNotStable,
    )?;
    Ok(())
}

/// Steps 3 and 4: `clk_sys` onto `clk_ref` (glitchless `SRC = 0`, aux source
/// written explicitly), then `clk_ref` divided by one from the crystal.
fn move_clk_ref_to_xosc<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    regs.write(
        RegAddr::CLOCKS,
        CLK_SYS_CTRL,
        CLK_SYS_AUXSRC_PLL_SYS | CLK_SYS_SRC_REF,
    );
    let sys_on_ref = (0b11, 1 << CLK_SYS_SRC_REF);
    polls.clk_sys_to_ref = wait(
        regs,
        plan,
        RegAddr::CLOCKS,
        CLK_SYS_SELECTED,
        sys_on_ref,
        ClockFault::ClkSysToRefTimeout,
    )?;
    regs.write(RegAddr::CLOCKS, CLK_REF_DIV, CLK_REF_DIV_BY_ONE);
    regs.write(RegAddr::CLOCKS, CLK_REF_CTRL, CLK_REF_SRC_XOSC);
    let ref_on_xosc = (0b1111, 1 << CLK_REF_SRC_XOSC);
    polls.clk_ref_to_xosc = wait(
        regs,
        plan,
        RegAddr::CLOCKS,
        CLK_REF_SELECTED,
        ref_on_xosc,
        ClockFault::ClkRefToXoscTimeout,
    )?;
    Ok(())
}

/// Steps 5 and 6 (§8.6.4): reset the PLL through the `RESETS` set and clear
/// aliases, then the SDK order: dividers, power and VCO on, `LOCK`, post
/// dividers, post-divider power on.
fn start_pll_sys<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    regs.write(RegAddr::RESET, RESETS_RESET + ALIAS_SET, RESET_BIT_PLL_SYS);
    regs.write(RegAddr::RESET, RESETS_RESET + ALIAS_CLR, RESET_BIT_PLL_SYS);
    let done = (RESET_BIT_PLL_SYS, RESET_BIT_PLL_SYS);
    polls.pll_reset = wait(
        regs,
        plan,
        RegAddr::RESET,
        RESETS_RESET_DONE,
        done,
        ClockFault::PllResetTimeout,
    )?;
    regs.write(RegAddr::PLL_SYS, PLL_CS, plan.pll_cs);
    regs.write(RegAddr::PLL_SYS, PLL_FBDIV_INT, plan.pll_fbdiv);
    // PD and VCOPD cleared; POSTDIVPD and DSMPD stay set until the VCO locks.
    regs.write(RegAddr::PLL_SYS, PLL_PWR, PLL_PWR_POSTDIVPD | PLL_PWR_DSMPD);
    let lock = (PLL_CS_LOCK, PLL_CS_LOCK);
    polls.pll_lock = wait(
        regs,
        plan,
        RegAddr::PLL_SYS,
        PLL_CS,
        lock,
        ClockFault::PllLockTimeout,
    )?;
    regs.write(RegAddr::PLL_SYS, PLL_PRIM, plan.pll_prim);
    regs.write(RegAddr::PLL_SYS, PLL_PWR, PLL_PWR_DSMPD);
    Ok(())
}

/// Step 7 (§8.1.5.1): divider before source, aux mux while the glitchless
/// mux is on `clk_ref`, then the glitchless mux to aux.
fn move_clk_sys_to_pll<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    regs.write(RegAddr::CLOCKS, CLK_SYS_DIV, DIV_BY_ONE_INT16);
    regs.write(
        RegAddr::CLOCKS,
        CLK_SYS_CTRL,
        CLK_SYS_AUXSRC_PLL_SYS | CLK_SYS_SRC_REF,
    );
    regs.write(
        RegAddr::CLOCKS,
        CLK_SYS_CTRL,
        CLK_SYS_AUXSRC_PLL_SYS | CLK_SYS_SRC_AUX,
    );
    let sys_on_aux = (0b11, 1 << CLK_SYS_SRC_AUX);
    polls.clk_sys_to_pll = wait(
        regs,
        plan,
        RegAddr::CLOCKS,
        CLK_SYS_SELECTED,
        sys_on_aux,
        ClockFault::ClkSysToPllTimeout,
    )?;
    Ok(())
}

/// Step 8 (§8.1.5.1 step 3, Table 561): stop the generator cleanly, wait for
/// `ENABLED` to clear, select `clk_sys` divided by one, start, wait for
/// `ENABLED`.
fn start_clk_peri<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    regs.write(RegAddr::CLOCKS, CLK_PERI_CTRL, CLK_PERI_AUXSRC_CLK_SYS);
    let stopped = (CLK_PERI_ENABLED, 0);
    polls.clk_peri_stop = wait(
        regs,
        plan,
        RegAddr::CLOCKS,
        CLK_PERI_CTRL,
        stopped,
        ClockFault::ClkPeriStopTimeout,
    )?;
    regs.write(RegAddr::CLOCKS, CLK_PERI_DIV, DIV_BY_ONE_INT16);
    regs.write(
        RegAddr::CLOCKS,
        CLK_PERI_CTRL,
        CLK_PERI_ENABLE | CLK_PERI_AUXSRC_CLK_SYS,
    );
    let running = (CLK_PERI_ENABLED, CLK_PERI_ENABLED);
    polls.clk_peri_start = wait(
        regs,
        plan,
        RegAddr::CLOCKS,
        CLK_PERI_CTRL,
        running,
        ClockFault::ClkPeriStartTimeout,
    )?;
    Ok(())
}

/// Step 9 (§8.5.1): "Before changing the cycle count, always stop the tick
/// generator". TIMER0 first, then the watchdog.
fn start_ticks<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(), ClockFault> {
    let running = (TICK_RUNNING, TICK_RUNNING);
    regs.write(RegAddr::TICKS, TICKS_TIMER0_CTRL, 0);
    regs.write(RegAddr::TICKS, TICKS_TIMER0_CYCLES, plan.tick_cycles);
    regs.write(RegAddr::TICKS, TICKS_TIMER0_CTRL, TICK_ENABLE);
    polls.timer0_tick = wait(
        regs,
        plan,
        RegAddr::TICKS,
        TICKS_TIMER0_CTRL,
        running,
        ClockFault::Timer0TickNotRunning,
    )?;
    regs.write(RegAddr::TICKS, TICKS_WATCHDOG_CTRL, 0);
    regs.write(RegAddr::TICKS, TICKS_WATCHDOG_CYCLES, plan.tick_cycles);
    regs.write(RegAddr::TICKS, TICKS_WATCHDOG_CTRL, TICK_ENABLE);
    polls.watchdog_tick = wait(
        regs,
        plan,
        RegAddr::TICKS,
        TICKS_WATCHDOG_CTRL,
        running,
        ClockFault::WatchdogTickNotRunning,
    )?;
    Ok(())
}

/// One frequency-counter run (§8.1.5.2): reference, window, interval, source
/// (which starts the count), wait `DONE`, judge, and release the counter.
fn measure_one<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    src: u32,
    timeout: ClockFault,
) -> Result<(u32, Measurement), ClockFault> {
    regs.write(RegAddr::CLOCKS, FC0_REF_KHZ, plan.fc_ref_khz);
    regs.write(RegAddr::CLOCKS, FC0_MIN_KHZ, plan.fc_min_khz);
    regs.write(RegAddr::CLOCKS, FC0_MAX_KHZ, plan.fc_max_khz);
    regs.write(RegAddr::CLOCKS, FC0_DELAY, FC0_DELAY_ONE);
    regs.write(RegAddr::CLOCKS, FC0_INTERVAL, FC0_INTERVAL_1MS);
    regs.write(RegAddr::CLOCKS, FC0_SRC, src);
    let done = (FC0_STATUS_DONE | FC0_STATUS_RUNNING, FC0_STATUS_DONE);
    let reads = wait(regs, plan, RegAddr::CLOCKS, FC0_STATUS, done, timeout)?;
    let status = regs.read(RegAddr::CLOCKS, FC0_STATUS);
    let result = regs.read(RegAddr::CLOCKS, FC0_RESULT);
    regs.write(RegAddr::CLOCKS, FC0_SRC, FC0_SRC_NULL);
    Ok((
        reads,
        judge(status, result, plan.fc_min_khz, plan.fc_max_khz),
    ))
}

/// Step 10: measure `clk_sys`, then `clk_peri`; both must pass.
fn measure<R: Regs>(
    regs: &mut R,
    plan: &ClockPlan,
    polls: &mut PollCounts,
) -> Result<(u32, u32), ClockFault> {
    let (reads, sys) = measure_one(
        regs,
        plan,
        FC0_SRC_CLK_SYS,
        ClockFault::ClkSysMeasureTimeout,
    )?;
    polls.measure_clk_sys = reads;
    let sys_khz = match sys {
        Measurement::Pass(khz) => khz,
        Measurement::Fail(khz) => return Err(ClockFault::ClkSysOutOfTolerance(khz)),
    };
    let (reads, peri) = measure_one(
        regs,
        plan,
        FC0_SRC_CLK_PERI,
        ClockFault::ClkPeriMeasureTimeout,
    )?;
    polls.measure_clk_peri = reads;
    match peri {
        Measurement::Pass(khz) => Ok((sys_khz, khz)),
        Measurement::Fail(khz) => Err(ClockFault::ClkPeriOutOfTolerance(khz)),
    }
}

#[cfg(test)]
#[path = "clocks_tests.rs"]
mod tests;
