//! Host tests of the parent module (split out to keep each file under the
//! 500-line limit of cwht coding standard CS-18).

extern crate std;
use std::vec::Vec;

use super::*;
use crate::clocks::{ClockConfigError, FC0_STATUS_PASS};
use crate::common::reg::fake::{Access, FakeRegs};
use crate::common::reg::{ALIAS_CLR, ALIAS_SET};

/// Datasheet reset values: `CLK_SYS_CTRL` `AUXSRC = 0x2` (ROSC) and `SRC = 1`
/// (aux), so `clk_sys` runs from the ROSC through the aux mux (Table 558);
/// `CLK_REF_CTRL` on the ROSC (Table 555); `CLK_PERI_CTRL` zero (Table 561).
const CLK_SYS_CTRL_RESET: u32 = 0x41;
const CLK_REF_CTRL_RESET: u32 = 0x0;
const CLK_PERI_CTRL_RESET: u32 = 0x0;
/// `AUXSRC`: bits 7:5 of `CLK_SYS_CTRL`, `CLK_PERI_CTRL`; 6:5 of `CLK_REF_CTRL`.
const SYS_AUXSRC: u32 = 0b111 << 5;
const REF_AUXSRC: u32 = 0b11 << 5;

const CFG: ClockConfig = ClockConfig {
    poll_budget: 8,
    ..ClockConfig::PICO2_150_MHZ
};

/// A register file in which every wait succeeds on its first read and both
/// measurements read 150 MHz with the hardware PASS flag.
fn healthy() -> FakeRegs {
    let mut r = FakeRegs::new();
    // The datasheet reset values, which the golden sequence starts from.
    r.set(RegAddr::CLOCKS, CLK_SYS_CTRL, CLK_SYS_CTRL_RESET);
    r.set(RegAddr::CLOCKS, CLK_REF_CTRL, CLK_REF_CTRL_RESET);
    r.set(RegAddr::CLOCKS, CLK_PERI_CTRL, CLK_PERI_CTRL_RESET);
    r.set(RegAddr::XOSC, XOSC_STATUS, XOSC_STATUS_STABLE);
    r.script_reads(RegAddr::CLOCKS, CLK_SYS_SELECTED, &[0b01, 0b10]);
    r.set(RegAddr::CLOCKS, CLK_REF_SELECTED, 0b0100);
    r.set(RegAddr::RESET, RESETS_RESET_DONE, RESET_BIT_PLL_SYS);
    // Registers the driver also writes are scripted: their read-only status
    // bits must survive the write, as on the hardware.
    r.script_reads(RegAddr::PLL_SYS, PLL_CS, &[PLL_CS_LOCK]);
    r.script_reads(RegAddr::CLOCKS, CLK_PERI_CTRL, &[0, CLK_PERI_ENABLED]);
    r.script_reads(
        RegAddr::TICKS,
        TICKS_TIMER0_CTRL,
        &[TICK_RUNNING | TICK_ENABLE],
    );
    r.script_reads(
        RegAddr::TICKS,
        TICKS_WATCHDOG_CTRL,
        &[TICK_RUNNING | TICK_ENABLE],
    );
    r.set(
        RegAddr::CLOCKS,
        FC0_STATUS,
        FC0_STATUS_DONE | FC0_STATUS_PASS,
    );
    r.set(RegAddr::CLOCKS, FC0_RESULT, 150_000 << 5);
    r
}

/// The register writes of the nominal sequence, in order: the golden
/// sequence checked against the datasheet steps in the module table.
fn golden_writes() -> Vec<(RegAddr, usize, u32)> {
    use RegAddr::{CLOCKS, PLL_SYS, RESET, TICKS, XOSC};
    let mut w = std::vec![
        (XOSC, XOSC_CTRL, 0x0000_0aa0),
        (XOSC, XOSC_STARTUP, 47),
        (XOSC, XOSC_CTRL, 0x00fa_baa0),
        (CLOCKS, CLK_SYS_CTRL + ALIAS_CLR, 0x1),
        (CLOCKS, CLK_REF_DIV, 0x0001_0000),
        (CLOCKS, CLK_REF_CTRL, 0x2),
        (RESET, 0x2000, 1 << 14),
        (RESET, 0x3000, 1 << 14),
        (PLL_SYS, PLL_CS, 1),
        (PLL_SYS, PLL_FBDIV_INT, 125),
        (PLL_SYS, PLL_PWR, 0x0c),
        (PLL_SYS, PLL_PRIM, 0x0005_2000),
        (PLL_SYS, PLL_PWR, 0x04),
        (CLOCKS, CLK_SYS_DIV, 0x0001_0000),
        (CLOCKS, CLK_SYS_CTRL, 0x0),
        (CLOCKS, CLK_SYS_CTRL, 0x1),
        (CLOCKS, CLK_PERI_CTRL + ALIAS_CLR, 0x0800),
        (CLOCKS, CLK_PERI_CTRL, 0x0),
        (CLOCKS, CLK_PERI_DIV, 0x0001_0000),
        (CLOCKS, CLK_PERI_CTRL + ALIAS_SET, 0x0800),
        (TICKS, TICKS_TIMER0_CTRL, 0),
        (TICKS, TICKS_TIMER0_CYCLES, 12),
        (TICKS, TICKS_TIMER0_CTRL, 1),
        (TICKS, TICKS_WATCHDOG_CTRL, 0),
        (TICKS, TICKS_WATCHDOG_CYCLES, 12),
        (TICKS, TICKS_WATCHDOG_CTRL, 1),
    ];
    for src in [FC0_SRC_CLK_SYS, FC0_SRC_CLK_PERI] {
        w.extend([
            (CLOCKS, FC0_REF_KHZ, 12_000),
            (CLOCKS, FC0_MIN_KHZ, 148_500),
            (CLOCKS, FC0_MAX_KHZ, 151_500),
            (CLOCKS, FC0_DELAY, 1),
            (CLOCKS, FC0_INTERVAL, 10),
            (CLOCKS, FC0_SRC, src),
            (CLOCKS, FC0_SRC, 0),
        ]);
    }
    w
}

#[test]
fn nominal_bring_up_writes_the_golden_sequence() {
    let mut regs = healthy();
    let ready = bring_up(&mut regs, &CFG).unwrap();
    assert_eq!(regs.writes(), golden_writes());
    let r = ready.report();
    assert_eq!(
        (r.clk_ref_hz, r.clk_sys_hz, r.clk_peri_hz, r.tick_hz),
        (12_000_000, 150_000_000, 150_000_000, 1_000_000)
    );
    assert_eq!(
        (r.measured_clk_sys_khz, r.measured_clk_peri_khz),
        (150_000, 150_000)
    );
    assert_eq!(
        (ready.clk_sys_hz(), ready.clk_peri_hz()),
        (150_000_000, 150_000_000)
    );
}

#[test]
fn poll_counts_report_the_reads_each_wait_took() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::XOSC, XOSC_STATUS, &[0, 0, 0, XOSC_STATUS_STABLE]);
    regs.script_reads(RegAddr::PLL_SYS, PLL_CS, &[1, PLL_CS_LOCK]);
    let ready = bring_up(&mut regs, &CFG).unwrap();
    let p = ready.report().polls;
    assert_eq!(
        (
            p.xosc_stable,
            p.pll_lock,
            p.clk_sys_to_ref,
            p.clk_peri_start
        ),
        (4, 2, 1, 1)
    );
}

#[test]
fn invalid_config_writes_nothing() {
    let mut regs = healthy();
    let bad = ClockConfig {
        xosc_hz: 20_000_000,
        ..CFG
    };
    assert_eq!(
        bring_up(&mut regs, &bad).err(),
        Some(ClockFault::Config(ClockConfigError::XoscOutOfRange))
    );
    assert!(regs.log().is_empty());
}

/// Run the bring-up with one register stuck at `value`, and check that
/// the named fault is returned after exactly the poll budget of reads and
/// that no block after the failing step was written.
fn stuck(block: RegAddr, offset: usize, value: u32, fault: ClockFault, untouched: &[RegAddr]) {
    let mut regs = healthy();
    regs.script_reads(block, offset, &[value]);
    assert_eq!(bring_up(&mut regs, &CFG).err(), Some(fault));
    for b in untouched {
        assert!(!regs.wrote_block(*b), "{b:?} written after {fault:?}");
    }
}

#[test]
fn xosc_that_never_stabilises_stops_before_any_clock_switch() {
    stuck(
        RegAddr::XOSC,
        XOSC_STATUS,
        0,
        ClockFault::XoscNotStable,
        &[
            RegAddr::CLOCKS,
            RegAddr::PLL_SYS,
            RegAddr::RESET,
            RegAddr::TICKS,
        ],
    );
}

#[test]
fn xosc_wait_is_bounded_by_the_budget() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::XOSC, XOSC_STATUS, &[0]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::XoscNotStable)
    );
    assert_eq!(regs.reads_of(RegAddr::XOSC, XOSC_STATUS), 8);
}

#[test]
fn clk_sys_that_never_reaches_clk_ref_stops_before_the_pll() {
    stuck(
        RegAddr::CLOCKS,
        CLK_SYS_SELECTED,
        0b10,
        ClockFault::ClkSysToRefTimeout,
        &[RegAddr::PLL_SYS, RegAddr::RESET, RegAddr::TICKS],
    );
}

#[test]
fn clk_ref_that_never_reaches_xosc_stops_before_the_pll() {
    stuck(
        RegAddr::CLOCKS,
        CLK_REF_SELECTED,
        0b0001,
        ClockFault::ClkRefToXoscTimeout,
        &[RegAddr::PLL_SYS, RegAddr::RESET, RegAddr::TICKS],
    );
}

#[test]
fn pll_reset_timeout_stops_before_pll_programming() {
    stuck(
        RegAddr::RESET,
        RESETS_RESET_DONE,
        0,
        ClockFault::PllResetTimeout,
        &[RegAddr::PLL_SYS, RegAddr::TICKS],
    );
}

#[test]
fn pll_that_never_locks_leaves_post_dividers_off_and_clk_sys_on_clk_ref() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::PLL_SYS, PLL_CS, &[1]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::PllLockTimeout)
    );
    assert_eq!(
        regs.get(RegAddr::PLL_SYS, PLL_PWR),
        PLL_PWR_POSTDIVPD | PLL_PWR_DSMPD
    );
    assert!(
        !regs
            .writes()
            .iter()
            .any(|w| w.0 == RegAddr::PLL_SYS && w.1 == PLL_PRIM)
    );
    // clk_sys on clk_ref, and the aux select still at its reset source.
    assert_eq!(
        regs.get(RegAddr::CLOCKS, CLK_SYS_CTRL),
        CLK_SYS_CTRL_RESET & !CLK_SYS_CTRL_SRC
    );
    assert!(!regs.wrote_block(RegAddr::TICKS));
}

#[test]
fn clk_sys_that_never_reaches_the_pll_is_a_fault() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::CLOCKS, CLK_SYS_SELECTED, &[0b01, 0b01]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkSysToPllTimeout)
    );
    assert!(!regs.wrote_block(RegAddr::TICKS));
}

#[test]
fn clk_peri_stop_and_start_timeouts() {
    stuck(
        RegAddr::CLOCKS,
        CLK_PERI_CTRL,
        CLK_PERI_ENABLED,
        ClockFault::ClkPeriStopTimeout,
        &[RegAddr::TICKS],
    );
    let mut regs = healthy();
    regs.script_reads(RegAddr::CLOCKS, CLK_PERI_CTRL, &[0, 0]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkPeriStartTimeout)
    );
    assert!(!regs.wrote_block(RegAddr::TICKS));
}

#[test]
fn tick_generators_that_do_not_run_are_faults() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::TICKS, TICKS_TIMER0_CTRL, &[TICK_ENABLE]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::Timer0TickNotRunning)
    );
    let mut regs = healthy();
    regs.script_reads(RegAddr::TICKS, TICKS_WATCHDOG_CTRL, &[TICK_ENABLE]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::WatchdogTickNotRunning)
    );
}

#[test]
fn measurement_that_never_finishes_is_a_fault() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::CLOCKS, FC0_STATUS, &[FC0_STATUS_RUNNING]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkSysMeasureTimeout)
    );
    let mut regs = healthy();
    let done_ok = FC0_STATUS_DONE | FC0_STATUS_PASS;
    // clk_sys: wait read, status read; clk_peri: never done.
    regs.script_reads(
        RegAddr::CLOCKS,
        FC0_STATUS,
        &[done_ok, done_ok, FC0_STATUS_RUNNING],
    );
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkPeriMeasureTimeout)
    );
}

#[test]
fn wrong_clk_sys_frequency_is_a_fault_before_clk_peri_is_measured() {
    let mut regs = healthy();
    regs.set(RegAddr::CLOCKS, FC0_RESULT, 125_000 << 5);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkSysOutOfTolerance(125_000))
    );
    let fc_src_writes = regs
        .writes()
        .iter()
        .filter(|w| w.0 == RegAddr::CLOCKS && w.1 == FC0_SRC)
        .count();
    assert_eq!(fc_src_writes, 2); // start and release of the clk_sys count only
}

#[test]
fn wrong_clk_peri_frequency_is_a_fault() {
    let mut regs = healthy();
    regs.script_reads(RegAddr::CLOCKS, FC0_RESULT, &[150_000 << 5, 75_000 << 5]);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkPeriOutOfTolerance(75_000))
    );
}

#[test]
fn hardware_fail_flag_alone_fails_the_check() {
    let mut regs = healthy();
    regs.set(RegAddr::CLOCKS, FC0_STATUS, FC0_STATUS_DONE);
    assert_eq!(
        bring_up(&mut regs, &CFG).err(),
        Some(ClockFault::ClkSysOutOfTolerance(150_000))
    );
}

/// A restart that did not reset `CLOCKS`: `clk_sys` on aux from `PLL_SYS`,
/// `clk_ref` on the crystal, `clk_peri` running from `PLL_SYS` (`AUXSRC = 1`).
const RESTART: [u32; 3] = [0x01, 0x02, (1 << 11) | (1 << 5)];
const COLD: [u32; 3] = [CLK_SYS_CTRL_RESET, CLK_REF_CTRL_RESET, CLK_PERI_CTRL_RESET];

/// `healthy()` with the three control registers preset to `ctrl`
/// (`CLK_SYS_CTRL`, `CLK_REF_CTRL`, `CLK_PERI_CTRL`).
fn healthy_from(ctrl: [u32; 3]) -> FakeRegs {
    let mut r = healthy();
    r.set(RegAddr::CLOCKS, CLK_SYS_CTRL, ctrl[0]);
    r.set(RegAddr::CLOCKS, CLK_REF_CTRL, ctrl[1]);
    r.set(RegAddr::CLOCKS, CLK_PERI_CTRL, ctrl[2]);
    r
}

/// One generator as the aux-mux rule of §8.1.2.2 sees it: `status` is the
/// register whose read value passes `off_aux` when the generator is off its
/// aux path; `held` says the control value keeps it there.
struct Generator {
    ctrl: usize,
    value: u32,
    auxsrc: u32,
    status: usize,
    off_aux: fn(u32) -> bool,
    held: fn(u32) -> bool,
    confirmed: bool,
}

/// Replay the access log from the control values `start` and fail if any
/// write changes an `AUXSRC` field while its generator may be on the aux
/// path. `clk_sys` and `clk_ref` are off it only when `SRC` selects a non-aux
/// source and a `SELECTED` read since that choice shows it; `clk_peri` only
/// when `ENABLE` is clear and a `CTRL` read since then shows `ENABLED` clear
/// (§8.1.2.2: "Before switching the clock source of an auxiliary mux you
/// must either" leave the aux path or disable the generator).
fn assert_aux_changes_only_off_the_aux_path(start: [u32; 3], log: &[Access]) {
    let mut gens = [
        Generator {
            ctrl: CLK_SYS_CTRL,
            value: start[0],
            auxsrc: SYS_AUXSRC,
            status: CLK_SYS_SELECTED,
            off_aux: |v| v == 1 << CLK_SYS_SRC_REF,
            held: |v| v & CLK_SYS_CTRL_SRC == CLK_SYS_SRC_REF,
            confirmed: false,
        },
        Generator {
            ctrl: CLK_REF_CTRL,
            value: start[1],
            auxsrc: REF_AUXSRC,
            status: CLK_REF_SELECTED,
            off_aux: |v| v != 0 && v & 0b0010 == 0,
            held: |v| v & 0b11 != 1,
            confirmed: false,
        },
        Generator {
            ctrl: CLK_PERI_CTRL,
            value: start[2],
            auxsrc: SYS_AUXSRC,
            status: CLK_PERI_CTRL,
            off_aux: |v| v & CLK_PERI_ENABLED == 0,
            held: |v| v & CLK_PERI_ENABLE == 0,
            confirmed: false,
        },
    ];
    for access in log {
        match *access {
            Access::Read {
                block: RegAddr::CLOCKS,
                offset,
                value,
            } => {
                for g in gens.iter_mut().filter(|g| g.status == offset) {
                    g.confirmed = (g.held)(g.value) && (g.off_aux)(value);
                }
            }
            Access::Write {
                block: RegAddr::CLOCKS,
                offset,
                value,
            } => {
                let alias = offset & 0x3000;
                for g in gens.iter_mut().filter(|g| g.ctrl == offset - alias) {
                    let next = match alias {
                        ALIAS_SET => g.value | value,
                        ALIAS_CLR => g.value & !value,
                        _ => value,
                    };
                    assert!(
                        (next ^ g.value) & g.auxsrc == 0 || g.confirmed,
                        "AUXSRC of {:#x} changed on the aux path: {access:?}",
                        g.ctrl
                    );
                    if (g.held)(next) != (g.held)(g.value) {
                        g.confirmed = false;
                    }
                    g.value = next;
                }
            }
            Access::Read { .. } | Access::Write { .. } => {}
        }
    }
}

#[test]
fn no_aux_select_changes_while_its_generator_is_on_the_aux_path() {
    for start in [COLD, RESTART] {
        let mut regs = healthy_from(start);
        bring_up(&mut regs, &CFG).unwrap();
        assert_aux_changes_only_off_the_aux_path(start, regs.log());
        assert_eq!(
            regs.get(RegAddr::CLOCKS, CLK_SYS_CTRL),
            CLK_SYS_AUXSRC_PLL_SYS | CLK_SYS_SRC_AUX
        );
        assert_eq!(
            regs.get(RegAddr::CLOCKS, CLK_PERI_CTRL),
            CLK_PERI_ENABLE | CLK_PERI_AUXSRC_CLK_SYS
        );
    }
}

#[test]
#[should_panic(expected = "AUXSRC of 0x3c changed on the aux path")]
fn the_aux_rule_check_rejects_the_iteration_1_step_3_write() {
    // One word that moves the glitchless mux and the aux select together,
    // from the reset state (INSP-095 finding-1).
    let mut regs = healthy_from(COLD);
    regs.write(RegAddr::CLOCKS, CLK_SYS_CTRL, 0x0);
    assert_aux_changes_only_off_the_aux_path(COLD, regs.log());
}

#[test]
#[should_panic(expected = "AUXSRC of 0x48 changed on the aux path")]
fn the_aux_rule_check_rejects_the_iteration_1_step_8_write() {
    // AUXSRC in the same word as the stop, before ENABLED clears.
    let mut regs = healthy_from(RESTART);
    regs.write(RegAddr::CLOCKS, CLK_PERI_CTRL, 0x0);
    assert_aux_changes_only_off_the_aux_path(RESTART, regs.log());
}
