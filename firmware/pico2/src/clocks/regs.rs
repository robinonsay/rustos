//! Register layouts and field values of the blocks the clock bring-up
//! drives: `XOSC`, `PLL_SYS`, `CLOCKS` (generators and frequency counter),
//! `TICKS`, and the `RESETS` bit of `PLL_SYS`. Offsets are taken with
//! `offset_of!` from the layouts and checked against the datasheet tables at
//! compile time.

use core::mem::offset_of;

/// `XOSC` register block, base `0x4004_8000` (Table 597).
#[repr(C)]
struct XoscRegs {
    /// `CTRL` `0x00`: `ENABLE` 23:12 (`0xfab` enable, `0xd1e` disable),
    /// `FREQ_RANGE` 11:0 (`0xaa0` for 1 to 15 MHz). Table 598.
    ctrl: u32,
    /// `STATUS` `0x04`: `STABLE` bit 31, `BADWRITE` bit 24, `ENABLED` bit 12. Table 599.
    status: u32,
    /// `DORMANT` `0x08`. Not used. Table 600.
    dormant: u32,
    /// `STARTUP` `0x0c`: `X4` bit 20, `DELAY` 13:0 in units of 256 crystal periods. Table 601.
    startup: u32,
}

pub(crate) const XOSC_CTRL: usize = offset_of!(XoscRegs, ctrl);
pub(crate) const XOSC_STATUS: usize = offset_of!(XoscRegs, status);
pub(crate) const XOSC_STARTUP: usize = offset_of!(XoscRegs, startup);
const _: () = assert!(XOSC_CTRL == 0x00 && XOSC_STATUS == 0x04 && XOSC_STARTUP == 0x0c);

/// `XOSC.CTRL.ENABLE` value that starts the oscillator (Table 598).
pub(crate) const XOSC_ENABLE: u32 = 0xfab;
/// `XOSC.CTRL.FREQ_RANGE` value for a 1 to 15 MHz crystal (Table 598).
pub(crate) const XOSC_RANGE_1_15MHZ: u32 = 0xaa0;
/// `XOSC.STATUS.STABLE` (Table 599).
pub(crate) const XOSC_STATUS_STABLE: u32 = 1 << 31;

/// `PLL_SYS` register block, base `0x4005_0000` (Table 635).
#[repr(C)]
struct PllRegs {
    /// `CS` `0x00`: `LOCK` bit 31, `LOCK_N` bit 30, `BYPASS` bit 8, `REFDIV` 5:0. Table 636.
    cs: u32,
    /// `PWR` `0x04`: `VCOPD` bit 5, `POSTDIVPD` bit 3, `DSMPD` bit 2, `PD` bit 0,
    /// all resetting to 1 (powered down). Table 637.
    pwr: u32,
    /// `FBDIV_INT` `0x08`: feedback divisor 11:0. Table 638.
    fbdiv_int: u32,
    /// `PRIM` `0x0c`: `POSTDIV1` 18:16, `POSTDIV2` 14:12. Table 639.
    prim: u32,
}

pub(crate) const PLL_CS: usize = offset_of!(PllRegs, cs);
pub(crate) const PLL_PWR: usize = offset_of!(PllRegs, pwr);
pub(crate) const PLL_FBDIV_INT: usize = offset_of!(PllRegs, fbdiv_int);
pub(crate) const PLL_PRIM: usize = offset_of!(PllRegs, prim);
const _: () =
    assert!(PLL_CS == 0x00 && PLL_PWR == 0x04 && PLL_FBDIV_INT == 0x08 && PLL_PRIM == 0x0c);

/// `PLL.CS.LOCK` (Table 636).
pub(crate) const PLL_CS_LOCK: u32 = 1 << 31;
/// `PLL.PWR.POSTDIVPD` (Table 637).
pub(crate) const PLL_PWR_POSTDIVPD: u32 = 1 << 3;
/// `PLL.PWR.DSMPD` (Table 637). Left set: "Nothing is achieved by setting this low."
pub(crate) const PLL_PWR_DSMPD: u32 = 1 << 2;

/// `RESETS.RESET` bit of `PLL_SYS` (Table 534).
pub(crate) const RESET_BIT_PLL_SYS: u32 = 1 << 14;
/// Offset of `RESETS.RESET` (§7.5.3).
pub(crate) const RESETS_RESET: usize = 0x0;
/// Offset of `RESETS.RESET_DONE` (§7.5.3).
pub(crate) const RESETS_RESET_DONE: usize = 0x8;

/// The part of the `CLOCKS` block this driver uses, base `0x4001_0000`
/// (Table 542). The four `CLK_GPOUTn` generators occupy `0x00` to `0x2f`.
#[repr(C)]
struct ClocksRegs {
    /// `CLK_GPOUT0..3_{CTRL,DIV,SELECTED}` `0x00` to `0x2c`. Not used.
    gpout: [u32; 12],
    /// `CLK_REF_CTRL` `0x30`: `AUXSRC` 6:5, `SRC` 1:0 (`2` = `XOSC_CLKSRC`). Table 555.
    clk_ref_ctrl: u32,
    /// `CLK_REF_DIV` `0x34`: `INT` 23:16. Table 556.
    clk_ref_div: u32,
    /// `CLK_REF_SELECTED` `0x38`: one-hot, bit `n` for `SRC = n`. Table 557.
    clk_ref_selected: u32,
    /// `CLK_SYS_CTRL` `0x3c`: `AUXSRC` 7:5 (`0` = `CLKSRC_PLL_SYS`), `SRC` bit 0
    /// (`0` = `CLK_REF`, `1` = `CLKSRC_CLK_SYS_AUX`). Table 558.
    clk_sys_ctrl: u32,
    /// `CLK_SYS_DIV` `0x40`: `INT` 31:16, `FRAC` 15:0. Table 559.
    clk_sys_div: u32,
    /// `CLK_SYS_SELECTED` `0x44`: one-hot over `SRC`. Table 560.
    clk_sys_selected: u32,
    /// `CLK_PERI_CTRL` `0x48`: `ENABLED` bit 28 (RO), `ENABLE` bit 11, `KILL` bit 10,
    /// `AUXSRC` 7:5 (`0` = `CLK_SYS`). Table 561.
    clk_peri_ctrl: u32,
    /// `CLK_PERI_DIV` `0x4c`: `INT` 17:16. Table 562.
    clk_peri_div: u32,
    /// `0x50` to `0x88`: `CLK_PERI_SELECTED` to `CLK_SYS_RESUS_STATUS`. Not used.
    unused: [u32; 15],
    /// `FC0_REF_KHZ` `0x8c`: reference frequency in kHz, 19:0. Table 578.
    fc0_ref_khz: u32,
    /// `FC0_MIN_KHZ` `0x90`: pass floor in kHz, 24:0. Table 579.
    fc0_min_khz: u32,
    /// `FC0_MAX_KHZ` `0x94`: pass ceiling in kHz, 24:0. Table 580.
    fc0_max_khz: u32,
    /// `FC0_DELAY` `0x98`: mux settle delay in `clk_ref` periods, 2:0. Table 581.
    fc0_delay: u32,
    /// `FC0_INTERVAL` `0x9c`: test interval `1 µs × 2^n`, 3:0. Table 582.
    fc0_interval: u32,
    /// `FC0_SRC` `0xa0`: clock to count; writing starts the count. Table 583.
    fc0_src: u32,
    /// `FC0_STATUS` `0xa4`: `DIED` 28, `FAST` 24, `SLOW` 20, `FAIL` 16, `WAITING` 12,
    /// `RUNNING` 8, `DONE` 4, `PASS` 0. Table 584.
    fc0_status: u32,
    /// `FC0_RESULT` `0xa8`: `KHZ` 29:5, `FRAC` 4:0. Table 585.
    fc0_result: u32,
}

pub(crate) const CLK_REF_CTRL: usize = offset_of!(ClocksRegs, clk_ref_ctrl);
pub(crate) const CLK_REF_DIV: usize = offset_of!(ClocksRegs, clk_ref_div);
pub(crate) const CLK_REF_SELECTED: usize = offset_of!(ClocksRegs, clk_ref_selected);
pub(crate) const CLK_SYS_CTRL: usize = offset_of!(ClocksRegs, clk_sys_ctrl);
pub(crate) const CLK_SYS_DIV: usize = offset_of!(ClocksRegs, clk_sys_div);
pub(crate) const CLK_SYS_SELECTED: usize = offset_of!(ClocksRegs, clk_sys_selected);
pub(crate) const CLK_PERI_CTRL: usize = offset_of!(ClocksRegs, clk_peri_ctrl);
pub(crate) const CLK_PERI_DIV: usize = offset_of!(ClocksRegs, clk_peri_div);
pub(crate) const FC0_REF_KHZ: usize = offset_of!(ClocksRegs, fc0_ref_khz);
pub(crate) const FC0_MIN_KHZ: usize = offset_of!(ClocksRegs, fc0_min_khz);
pub(crate) const FC0_MAX_KHZ: usize = offset_of!(ClocksRegs, fc0_max_khz);
pub(crate) const FC0_DELAY: usize = offset_of!(ClocksRegs, fc0_delay);
pub(crate) const FC0_INTERVAL: usize = offset_of!(ClocksRegs, fc0_interval);
pub(crate) const FC0_SRC: usize = offset_of!(ClocksRegs, fc0_src);
pub(crate) const FC0_STATUS: usize = offset_of!(ClocksRegs, fc0_status);
pub(crate) const FC0_RESULT: usize = offset_of!(ClocksRegs, fc0_result);
const _: () = assert!(
    CLK_REF_CTRL == 0x30
        && CLK_REF_DIV == 0x34
        && CLK_REF_SELECTED == 0x38
        && CLK_SYS_CTRL == 0x3c
        && CLK_SYS_DIV == 0x40
        && CLK_SYS_SELECTED == 0x44
        && CLK_PERI_CTRL == 0x48
        && CLK_PERI_DIV == 0x4c
        && FC0_REF_KHZ == 0x8c
        && FC0_MIN_KHZ == 0x90
        && FC0_MAX_KHZ == 0x94
        && FC0_DELAY == 0x98
        && FC0_INTERVAL == 0x9c
        && FC0_SRC == 0xa0
        && FC0_STATUS == 0xa4
        && FC0_RESULT == 0xa8
);

/// `CLK_REF_CTRL.SRC` value `XOSC_CLKSRC` (Table 555).
pub(crate) const CLK_REF_SRC_XOSC: u32 = 0x2;
/// `CLK_SYS_CTRL.SRC` value `CLK_REF` (Table 558).
pub(crate) const CLK_SYS_SRC_REF: u32 = 0x0;
/// `CLK_SYS_CTRL.SRC` value `CLKSRC_CLK_SYS_AUX` (Table 558).
pub(crate) const CLK_SYS_SRC_AUX: u32 = 0x1;
/// `CLK_SYS_CTRL.AUXSRC` value `CLKSRC_PLL_SYS`, at bits 7:5 (Table 558).
pub(crate) const CLK_SYS_AUXSRC_PLL_SYS: u32 = 0x0 << 5;
/// `CLK_PERI_CTRL.ENABLE` (Table 561).
pub(crate) const CLK_PERI_ENABLE: u32 = 1 << 11;
/// `CLK_PERI_CTRL.ENABLED`, read-only (Table 561).
pub(crate) const CLK_PERI_ENABLED: u32 = 1 << 28;
/// `CLK_PERI_CTRL.AUXSRC` value `CLK_SYS`, at bits 7:5 (Table 561).
pub(crate) const CLK_PERI_AUXSRC_CLK_SYS: u32 = 0x0 << 5;
/// A divider word with `INT = 1` and `FRAC = 0` for `CLK_SYS_DIV` and
/// `CLK_PERI_DIV` (`INT` at bit 16, Tables 559, 562): divide by one.
pub(crate) const DIV_BY_ONE_INT16: u32 = 1 << 16;
/// `CLK_REF_DIV` with `INT = 1` (`INT` at bits 23:16, Table 556).
pub(crate) const CLK_REF_DIV_BY_ONE: u32 = 1 << 16;
/// `FC0_SRC` value `CLK_SYS` (Table 583).
pub(crate) const FC0_SRC_CLK_SYS: u32 = 0x09;
/// `FC0_SRC` value `CLK_PERI` (Table 583).
pub(crate) const FC0_SRC_CLK_PERI: u32 = 0x0a;
/// `FC0_SRC` value `NULL`: stops the counter (Table 583).
pub(crate) const FC0_SRC_NULL: u32 = 0x00;
/// `FC0_INTERVAL` value 10: a 1 ms test with 2 kHz accuracy (Table 541, Table 582).
pub(crate) const FC0_INTERVAL_1MS: u32 = 10;
/// `FC0_DELAY` value 1 (the reset value): one `clk_ref` period of mux settle (Table 581).
pub(crate) const FC0_DELAY_ONE: u32 = 1;
/// `FC0_STATUS.DONE` (Table 584).
pub(crate) const FC0_STATUS_DONE: u32 = 1 << 4;
/// `FC0_STATUS.PASS` (Table 584).
pub(crate) const FC0_STATUS_PASS: u32 = 1 << 0;
/// `FC0_STATUS.RUNNING` (Table 584).
pub(crate) const FC0_STATUS_RUNNING: u32 = 1 << 8;

/// The part of the `TICKS` block this driver uses, base `0x4010_8000` (Table 616).
#[repr(C)]
struct TicksRegs {
    /// `PROC0_*` and `PROC1_*` `0x00` to `0x14`. Not used.
    proc: [u32; 6],
    /// `TIMER0_CTRL` `0x18`: `RUNNING` bit 1 (RO), `ENABLE` bit 0 (Table 617 layout).
    timer0_ctrl: u32,
    /// `TIMER0_CYCLES` `0x1c`: `clk_ref` cycles per tick, 8:0 (Table 618 layout).
    timer0_cycles: u32,
    /// `TIMER0_COUNT` `0x20`. Not used.
    timer0_count: u32,
    /// `TIMER1_*` `0x24` to `0x2c`. Not used.
    timer1: [u32; 3],
    /// `WATCHDOG_CTRL` `0x30`: as `TIMER0_CTRL`.
    watchdog_ctrl: u32,
    /// `WATCHDOG_CYCLES` `0x34`: as `TIMER0_CYCLES`.
    watchdog_cycles: u32,
}

pub(crate) const TICKS_TIMER0_CTRL: usize = offset_of!(TicksRegs, timer0_ctrl);
pub(crate) const TICKS_TIMER0_CYCLES: usize = offset_of!(TicksRegs, timer0_cycles);
pub(crate) const TICKS_WATCHDOG_CTRL: usize = offset_of!(TicksRegs, watchdog_ctrl);
pub(crate) const TICKS_WATCHDOG_CYCLES: usize = offset_of!(TicksRegs, watchdog_cycles);
const _: () = assert!(
    TICKS_TIMER0_CTRL == 0x18
        && TICKS_TIMER0_CYCLES == 0x1c
        && TICKS_WATCHDOG_CTRL == 0x30
        && TICKS_WATCHDOG_CYCLES == 0x34
);

/// `TICKS.*_CTRL.ENABLE` (Table 617).
pub(crate) const TICK_ENABLE: u32 = 1 << 0;
/// `TICKS.*_CTRL.RUNNING`, read-only (Table 617).
pub(crate) const TICK_RUNNING: u32 = 1 << 1;
