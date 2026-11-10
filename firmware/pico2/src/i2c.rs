//! The two DW_apb_i2c controllers, as polled I2C controllers ("masters")
//! with 7-bit addressing.
//!
//! # Using it
//!
//! ```ignore
//! use api::i2c::I2c;
//! use pico2::i2c::Rp2350I2c;
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! // Baro + IMU on I2C0: SDA on GPIO8, SCL on GPIO9, 400 kHz.
//! let mut bus = Rp2350I2c::new(board.i2c0, &clocks, board.pins.gpio8, board.pins.gpio9, 400_000).unwrap();
//! let mut who_am_i = [0u8; 1];
//! bus.write_read(0x68, &[0x75], &mut who_am_i)?; // MPU-6050 WHO_AM_I
//! ```
//!
//! # The block
//!
//! Each controller is a Synopsys DW_apb_i2c v2.03a with 16-entry transmit
//! and receive FIFOs, supporting standard (100 kb/s), fast (400 kb/s) and
//! fast-plus (1 Mb/s) modes
//! ([§12.2.1, p982](../../../docs/rp2350-datasheet.pdf#page=982)). "All
//! clocks in the I2C controller are connected to `clk_sys`, including
//! `ic_clk`" (§12.2.1.2, same page), so every count below is in
//! `clk_sys` cycles: 150 MHz from [`crate::clocks`].
//!
//! # Bring-up
//!
//! [`Rp2350I2c::new`] follows "Master Mode Operation — Initial
//! Configuration"
//! ([§12.2.10.2.1, p997–998](../../../docs/rp2350-datasheet.pdf#page=997)),
//! after a full reset of the block (`RESETS` bit 4 or 5,
//! [Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)):
//!
//! 1. `IC_ENABLE.ENABLE` = 0: `IC_CON`, `IC_TAR`, the SCL counts and
//!    `IC_FS_SPKLEN` can only be written while it is 0.
//! 2. `IC_CON`: controller on, target mode off, 7-bit addressing,
//!    RESTART allowed, the speed mode, and "hold the bus when the RX FIFO
//!    is full".
//! 3. The SCL high and low counts and the spike filter length; see
//!    [SCL timing](#scl-timing).
//! 4. Connect SDA and SCL with the pads the datasheet asks for: "pull-up
//!    enabled, slew rate limited, schmitt trigger enabled"
//!    ([§12.2.1.3, p982](../../../docs/rp2350-datasheet.pdf#page=982)).
//!    The internal pull-ups are weak (about 50 kΩ); the bus still needs
//!    external pull-up resistors, as the datasheet's note there says.
//!
//! `IC_TAR` (step 3 of §12.2.10.2.1) and `IC_ENABLE` = 1 (step 4) are done
//! by the first transfer, and again whenever the device address changes.
//!
//! # Transfers
//!
//! Each byte of a transfer is one write to `IC_DATA_CMD`: a data byte to
//! send (`CMD` = 0) or a request to receive one (`CMD` = 1), with `STOP`
//! set on the last command and `RESTART` on the first read after a write
//! ([Table 1058, p1012–1014](../../../docs/rp2350-datasheet.pdf#page=1012)).
//! The controller "does not generate a STOP if the TX FIFO becomes empty;
//! in this situation the component holds the SCL line low"
//! ([§12.2.7.1, p991](../../../docs/rp2350-datasheet.pdf#page=991)), so a
//! slow refill only stretches the clock and never splits the transfer.
//! At most 16 read requests are outstanding at once, so the 16-entry
//! receive FIFO cannot overflow.
//!
//! A transfer ends when `IC_RAW_INTR_STAT.STOP_DET` shows the STOP was
//! sent, or fails when `TX_ABRT` shows the controller gave up; the reason
//! is then in `IC_TX_ABRT_SOURCE`
//! ([Table 1065, p1021–1024](../../../docs/rp2350-datasheet.pdf#page=1023);
//! [Table 1084, p1033–1038](../../../docs/rp2350-datasheet.pdf#page=1033)).
//! Every wait is bounded; on a timeout the block is reset and reprogrammed,
//! the only recovery the datasheet offers for a clock held low
//! ([§12.2.13.2, p1001](../../../docs/rp2350-datasheet.pdf#page=1001)).
//!
//! The interrupt line is never enabled in the NVIC and DMA is never
//! enabled, so the controller cannot interrupt the application.
//!
//! # SCL timing
//!
//! The controller makes SCL's low phase `LCNT + 1` cycles and its high
//! phase `HCNT + SPKLEN + 7` cycles, plus the bus's rise time, which
//! only slows it
//! ([§12.2.14.1, p1002; Figure 89, p1003](../../../docs/rp2350-datasheet.pdf#page=1002)).
//! [`SclTiming::for_frequency`] picks the counts so that:
//!
//! * the low and high phases are at least the I2C minimums the datasheet
//!   lists: 4.7 µs / 4.0 µs up to 100 kHz, 1.3 µs / 0.6 µs up to
//!   400 kHz, 0.5 µs / 0.26 µs up to 1 MHz
//!   ([§12.2.14.3, p1005](../../../docs/rp2350-datasheet.pdf#page=1005));
//! * the period is at least the requested one, so the bus never runs
//!   faster than asked; and
//! * the counts respect the controller's own floors: `LCNT > SPKLEN + 7`,
//!   `HCNT > SPKLEN + 5` (§12.2.14.1, p1002), `HCNT` ≤ 65525
//!   ([Table 1059, p1014](../../../docs/rp2350-datasheet.pdf#page=1014)).
//!
//! `SPKLEN` is the spike filter, set to suppress spikes up to 50 ns, the
//! figure the datasheet gives for standard, fast and fast-plus modes
//! ([§12.2.11–12.2.12, p1000](../../../docs/rp2350-datasheet.pdf#page=1000)):
//! 8 cycles at 150 MHz.
//!
//! # Ordering of register accesses
//!
//! The polling loops rely on accesses to one peripheral happening in
//! program order. The I2C blocks sit in the Armv8-M "Peripheral" region,
//! `0x4000_0000`–`0x5fff_ffff`, whose default memory type is
//! Device-nGnRE ([Armv8-M ARM §B8.1, p265](../../../docs/DDI0553B_z_armv8m_arm.pdf#page=265)),
//! and for that type accesses to a peripheral "arrive at the peripheral in
//! program order"
//! ([§B7.12.2, p239](../../../docs/DDI0553B_z_armv8m_arm.pdf#page=239)).
//! So no barrier instruction is needed between a write to `IC_DATA_CMD`
//! and the status reads that follow it.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer), so the `#page=`
//! links open the cited page.

use api::common::ErrorType;
use api::device::{DeviceHandle, PinHandle};
use api::i2c::I2c;

use crate::clocks::{CLK_SYS_HZ, Rp2350Clocks};
use crate::common::pinmux::{self, PadConfig, Role};
use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

// --- Register layout -----------------------------------------------------

/// The DW_apb_i2c registers, at the offsets of
/// [Table 1054, p1007–1008](../../../docs/rp2350-datasheet.pdf#page=1007).
/// Registers this driver does not use are named with a leading `_`.
#[repr(C)]
struct I2cRegs {
    /// `0x00` `IC_CON` (Table 1055, p1008–1010). Writable only while
    /// `IC_ENABLE[0]` = 0.
    con: u32,
    /// `0x04` `IC_TAR` (Table 1056, p1011): bits 9:0 target address.
    /// Writable only while `IC_ENABLE[0]` = 0 (p1010).
    tar: u32,
    /// `0x08` `IC_SAR`: target-mode address, not used.
    _sar: u32,
    /// `0x0c`: not in the register list.
    _reserved0c: u32,
    /// `0x10` `IC_DATA_CMD` (Table 1058, p1012–1014): write a command,
    /// read a received byte.
    data_cmd: u32,
    /// `0x14` `IC_SS_SCL_HCNT` (Table 1059, p1014).
    ss_scl_hcnt: u32,
    /// `0x18` `IC_SS_SCL_LCNT` (Table 1060, p1014–1015).
    ss_scl_lcnt: u32,
    /// `0x1c` `IC_FS_SCL_HCNT` (Table 1061, p1015).
    fs_scl_hcnt: u32,
    /// `0x20` `IC_FS_SCL_LCNT` (Table 1062, p1015–1016).
    fs_scl_lcnt: u32,
    /// `0x24`–`0x28`: not in the register list.
    _reserved24: [u32; 2],
    /// `0x2c` `IC_INTR_STAT`: masked status, not used.
    _intr_stat: u32,
    /// `0x30` `IC_INTR_MASK`: not used; the interrupt line is never
    /// enabled in the NVIC.
    _intr_mask: u32,
    /// `0x34` `IC_RAW_INTR_STAT` (Table 1065, p1021–1024): unmasked status.
    raw_intr_stat: u32,
    /// `0x38`–`0x50`: `IC_RX_TL`, `IC_TX_TL`, `IC_CLR_INTR`,
    /// `IC_CLR_RX_UNDER`, `IC_CLR_RX_OVER`, `IC_CLR_TX_OVER`,
    /// `IC_CLR_RD_REQ`; not used.
    _reserved38: [u32; 7],
    /// `0x54` `IC_CLR_TX_ABRT` (Table 1073, p1027): reading clears
    /// `TX_ABRT` and releases the flushed FIFOs.
    clr_tx_abrt: u32,
    /// `0x58`–`0x5c`: `IC_CLR_RX_DONE`, `IC_CLR_ACTIVITY`; not used.
    _reserved58: [u32; 2],
    /// `0x60` `IC_CLR_STOP_DET` (Table 1076, p1028): reading clears
    /// `STOP_DET`.
    clr_stop_det: u32,
    /// `0x64`–`0x68`: `IC_CLR_START_DET`, `IC_CLR_GEN_CALL`; not used.
    _reserved64: [u32; 2],
    /// `0x6c` `IC_ENABLE` (Table 1079, p1029–1030).
    enable: u32,
    /// `0x70` `IC_STATUS` (Table 1080, p1030–1032): FIFO flags.
    status: u32,
    /// `0x74`–`0x7c`: `IC_TXFLR`, `IC_RXFLR`, `IC_SDA_HOLD`; not used.
    _reserved74: [u32; 3],
    /// `0x80` `IC_TX_ABRT_SOURCE` (Table 1084, p1033–1038): why the last
    /// transfer was aborted.
    tx_abrt_source: u32,
    /// `0x84`–`0x98`: `IC_SLV_DATA_NACK_ONLY`, `IC_DMA_CR`, `IC_DMA_TDLR`,
    /// `IC_DMA_RDLR`, `IC_SDA_SETUP`, `IC_ACK_GENERAL_CALL`; not used.
    _reserved84: [u32; 6],
    /// `0x9c` `IC_ENABLE_STATUS` (Table 1091, p1040–1042).
    enable_status: u32,
    /// `0xa0` `IC_FS_SPKLEN` (Table 1092, p1042). Writable only while
    /// `IC_ENABLE[0]` = 0.
    fs_spklen: u32,
}

// The offsets above, checked against Table 1054 (p1007–1008).
const _: () = assert!(core::mem::offset_of!(I2cRegs, data_cmd) == 0x10);
const _: () = assert!(core::mem::offset_of!(I2cRegs, fs_scl_lcnt) == 0x20);
const _: () = assert!(core::mem::offset_of!(I2cRegs, raw_intr_stat) == 0x34);
const _: () = assert!(core::mem::offset_of!(I2cRegs, clr_tx_abrt) == 0x54);
const _: () = assert!(core::mem::offset_of!(I2cRegs, clr_stop_det) == 0x60);
const _: () = assert!(core::mem::offset_of!(I2cRegs, enable) == 0x6c);
const _: () = assert!(core::mem::offset_of!(I2cRegs, status) == 0x70);
const _: () = assert!(core::mem::offset_of!(I2cRegs, tx_abrt_source) == 0x80);
const _: () = assert!(core::mem::offset_of!(I2cRegs, enable_status) == 0x9c);
const _: () = assert!(core::mem::offset_of!(I2cRegs, fs_spklen) == 0xa0);

/// `IC_CON.MASTER_MODE`, bit 0, reset 1 (Table 1055, p1010).
const CON_MASTER_MODE: u32 = 1 << 0;
/// `IC_CON.SPEED` starts at bit 1: 1 = standard, 2 = fast or fast-plus
/// (Table 1055, p1010).
const CON_SPEED_SHIFT: u32 = 1;
/// `IC_CON.IC_RESTART_EN`, bit 5, reset 1 (Table 1055, p1009).
const CON_RESTART_EN: u32 = 1 << 5;
/// `IC_CON.IC_SLAVE_DISABLE`, bit 6, reset 1 (Table 1055, p1009). Must be
/// 1 whenever `MASTER_MODE` is 1 (same table, p1010).
const CON_SLAVE_DISABLE: u32 = 1 << 6;
/// `IC_CON.RX_FIFO_FULL_HLD_CTRL`, bit 9: "Hold bus when RX_FIFO is full"
/// (Table 1055, p1008).
const CON_RX_FIFO_FULL_HLD: u32 = 1 << 9;

/// `IC_DATA_CMD.CMD`, bit 8: 1 = read (Table 1058, p1013).
const CMD_READ: u32 = 1 << 8;
/// `IC_DATA_CMD.STOP`, bit 9: STOP after this byte (Table 1058, p1013).
const CMD_STOP: u32 = 1 << 9;
/// `IC_DATA_CMD.RESTART`, bit 10: RESTART before this byte (Table 1058,
/// p1013).
const CMD_RESTART: u32 = 1 << 10;
/// `IC_DATA_CMD.DAT`, bits 7:0 (Table 1058, p1014).
const CMD_DATA: u32 = 0xff;

/// `IC_RAW_INTR_STAT.TX_ABRT`, bit 6 (Table 1065, p1023).
const INTR_TX_ABRT: u32 = 1 << 6;
/// `IC_RAW_INTR_STAT.STOP_DET`, bit 9 (Table 1065, p1022).
const INTR_STOP_DET: u32 = 1 << 9;

/// `IC_ENABLE.ENABLE`, bit 0 (Table 1079, p1030).
const ENABLE_ENABLE: u32 = 1 << 0;
/// `IC_ENABLE.ABORT`, bit 1 (Table 1079, p1030).
const ENABLE_ABORT: u32 = 1 << 1;

/// `IC_STATUS.TFNF`, bit 1, reset 1: transmit FIFO not full (Table 1080,
/// p1031).
const STATUS_TFNF: u32 = 1 << 1;
/// `IC_STATUS.RFNE`, bit 3: receive FIFO not empty (Table 1080, p1031).
const STATUS_RFNE: u32 = 1 << 3;

/// `IC_ENABLE_STATUS.IC_EN`, bit 0 (Table 1091, p1042).
const ENABLE_STATUS_IC_EN: u32 = 1 << 0;

/// `IC_TX_ABRT_SOURCE.ABRT_7B_ADDR_NOACK`, bit 0: no device acknowledged
/// the address (Table 1084, p1038).
const ABRT_7B_ADDR_NOACK: u32 = 1 << 0;
/// `IC_TX_ABRT_SOURCE.ABRT_TXDATA_NOACK`, bit 3: the device acknowledged
/// the address but not a data byte (Table 1084, p1037).
const ABRT_TXDATA_NOACK: u32 = 1 << 3;
/// `IC_TX_ABRT_SOURCE.ARB_LOST`, bit 12: another controller won the bus
/// (Table 1084, p1035).
const ABRT_ARB_LOST: u32 = 1 << 12;

/// Receive FIFO depth: "16-element receive buffer"
/// ([§12.2.1, p982](../../../docs/rp2350-datasheet.pdf#page=982)).
const RX_FIFO_DEPTH: usize = 16;

/// `I2C0` is bit 4 of `RESETS.RESET`, `I2C1` bit 5
/// ([Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)).
const RESET_I2C0_BIT: usize = 4;

// --- Timeouts ------------------------------------------------------------

/// Longest a transfer may go without a byte moving before it is given up:
/// 10 ms. A byte at the slowest accepted rate, [`MIN_FREQUENCY_HZ`], takes
/// 0.9 ms (9 clocks), so this only trips when a device holds SCL low
/// (clock stretching) for longer than 10 ms, or the bus is stuck.
pub const STALL_TIMEOUT_MS: u32 = 10;

/// How many status reads span at least [`STALL_TIMEOUT_MS`]. "APB reads
/// [cost] a minimum of three" `clk_sys` cycles
/// ([§2.2.4, p32](../../../docs/rp2350-datasheet.pdf#page=32)), and every
/// polling loop here reads an I2C register on each pass, so this many
/// passes take at least that long. They may take longer; the bound only
/// guarantees the timeout is never early.
const STALL_LOOKS: u32 = CLK_SYS_HZ / 1000 * STALL_TIMEOUT_MS / 3;

// --- SCL timing ----------------------------------------------------------

/// Lowest SCL frequency [`Rp2350I2c::new`] accepts: 10 kHz.
pub const MIN_FREQUENCY_HZ: u32 = 10_000;
/// Highest SCL frequency [`Rp2350I2c::new`] accepts: 1 MHz, fast-plus
/// mode, the fastest the controller supports (§12.2.1, p982).
pub const MAX_FREQUENCY_HZ: u32 = 1_000_000;

/// `IC_CON.SPEED` value: standard mode, up to 100 kb/s (Table 1055, p1010).
const SPEED_STANDARD: u32 = 1;
/// `IC_CON.SPEED` value: fast mode or fast-plus, up to 1000 kb/s (Table
/// 1055, p1010).
const SPEED_FAST: u32 = 2;

/// Register values for one SCL frequency. See [SCL timing](self#scl-timing).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SclTiming {
    /// `IC_CON.SPEED`: [`SPEED_STANDARD`] or [`SPEED_FAST`].
    speed: u32,
    /// `IC_*_SCL_HCNT`.
    hcnt: u32,
    /// `IC_*_SCL_LCNT`.
    lcnt: u32,
    /// `IC_FS_SPKLEN`.
    spklen: u32,
}

/// Nanoseconds to `ic_clk` cycles at `clk_hz`, rounded up, as
/// `ROUNDUP(MIN_SCL_xxxtime × OSCFREQ)` in §12.2.14.3 (p1005).
const fn ns_to_cycles_ceil(ns: u32, clk_hz: u32) -> u32 {
    // u64 so 4700 ns × 150 MHz cannot overflow.
    (ns as u64 * clk_hz as u64).div_ceil(1_000_000_000) as u32
}

impl SclTiming {
    /// The register values for SCL at no more than `hz` with `ic_clk` =
    /// `clk_hz`, or `None` if `hz` is outside
    /// [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`] or the counts do not fit.
    pub const fn for_frequency(clk_hz: u32, hz: u32) -> Option<Self> {
        // Outside the supported modes (§12.2.3, p983).
        if hz < MIN_FREQUENCY_HZ || hz > MAX_FREQUENCY_HZ {
            return None;
        }
        // Mode and its minimum SCL low/high times in ns (§12.2.14.3, p1005).
        let (speed, low_ns, high_ns) = if hz <= 100_000 {
            // Standard mode: 4700 ns low, 4000 ns high.
            (SPEED_STANDARD, 4_700, 4_000)
        } else if hz <= 400_000 {
            // Fast mode: 1300 ns low, 600 ns high.
            (SPEED_FAST, 1_300, 600)
        } else {
            // Fast-plus: 500 ns low, 260 ns high.
            (SPEED_FAST, 500, 260)
        };
        // Spike filter: the longest spike to reject is 50 ns (§12.2.12,
        // p1000); the register's minimum is 1 (Table 1092, p1042).
        let spklen = {
            let cycles = ns_to_cycles_ceil(50, clk_hz);
            if cycles < 1 { 1 } else { cycles }
        };
        // Fewest cycles each phase may last.
        let low_min = ns_to_cycles_ceil(low_ns, clk_hz);
        let high_min = ns_to_cycles_ceil(high_ns, clk_hz);
        // Whole period, rounded up so the bus is never faster than `hz`.
        let period = clk_hz.div_ceil(hz);
        // Below the two minimums: this clock cannot meet the mode.
        if period < low_min + high_min {
            return None;
        }
        // Spread the spare cycles in proportion to the two minimum times,
        // so both phases keep their minimum.
        let spare = period - (low_min + high_min);
        let low = low_min + (spare as u64 * low_ns as u64 / (low_ns + high_ns) as u64) as u32;
        // The high phase takes the rest of the period.
        let high = period - low;
        // Low phase is LCNT + 1 cycles (§12.2.14.1, p1002).
        let lcnt = low - 1;
        // High phase is HCNT + SPKLEN + 7 cycles (§12.2.14.1, p1002).
        if high < spklen + 7 {
            return None;
        }
        let hcnt = high - spklen - 7;
        // Controller floors: LCNT > SPKLEN + 7, HCNT > SPKLEN + 5 (p1002).
        if lcnt <= spklen + 7 || hcnt <= spklen + 5 {
            return None;
        }
        // Register ceilings: HCNT ≤ 65525 (Table 1059, p1014); LCNT is a
        // 16-bit field (Table 1060, p1015).
        if hcnt > 65_525 || lcnt > 0xffff {
            return None;
        }
        // All constraints met.
        Some(Self { speed, hcnt, lcnt, spklen })
    }

    /// SCL cycles of `ic_clk` per bit: `(HCNT + SPKLEN + 7) + (LCNT + 1)`,
    /// ignoring rise and fall times (Figure 89, p1003).
    pub const fn period_cycles(&self) -> u32 {
        // High phase plus low phase.
        (self.hcnt + self.spklen + 7) + (self.lcnt + 1)
    }

    /// The SCL frequency produced, rounded down, before rise time slows it
    /// further.
    pub const fn frequency_hz(&self, clk_hz: u32) -> u32 {
        // One bit per period.
        clk_hz / self.period_cycles()
    }

    /// Low-phase length in `ic_clk` cycles.
    const fn low_cycles(&self) -> u32 {
        // LCNT + 1 (§12.2.14.1, p1002).
        self.lcnt + 1
    }

    /// High-phase length in `ic_clk` cycles.
    const fn high_cycles(&self) -> u32 {
        // HCNT + SPKLEN + 7 (§12.2.14.1, p1002).
        self.hcnt + self.spklen + 7
    }
}

// The three standard rates at this crate's clk_sys: each must exist, never
// exceed the requested rate, and meet the minimum phase times (p1005).
const _: () = {
    // Standard mode, 100 kHz: low ≥ 4.7 µs, high ≥ 4.0 µs.
    let Some(t) = SclTiming::for_frequency(CLK_SYS_HZ, 100_000) else { panic!() };
    assert!(t.frequency_hz(CLK_SYS_HZ) <= 100_000);
    assert!(t.low_cycles() >= ns_to_cycles_ceil(4_700, CLK_SYS_HZ));
    assert!(t.high_cycles() >= ns_to_cycles_ceil(4_000, CLK_SYS_HZ));
};
const _: () = {
    // Fast mode, 400 kHz: low ≥ 1.3 µs, high ≥ 0.6 µs.
    let Some(t) = SclTiming::for_frequency(CLK_SYS_HZ, 400_000) else { panic!() };
    assert!(t.frequency_hz(CLK_SYS_HZ) <= 400_000);
    assert!(t.low_cycles() >= ns_to_cycles_ceil(1_300, CLK_SYS_HZ));
    assert!(t.high_cycles() >= ns_to_cycles_ceil(600, CLK_SYS_HZ));
};
const _: () = {
    // Fast-plus, 1 MHz: low ≥ 0.5 µs, high ≥ 0.26 µs.
    let Some(t) = SclTiming::for_frequency(CLK_SYS_HZ, 1_000_000) else { panic!() };
    assert!(t.frequency_hz(CLK_SYS_HZ) <= 1_000_000);
    assert!(t.low_cycles() >= ns_to_cycles_ceil(500, CLK_SYS_HZ));
    assert!(t.high_cycles() >= ns_to_cycles_ceil(260, CLK_SYS_HZ));
};
// The slowest accepted rate still fits the registers.
const _: () = assert!(SclTiming::for_frequency(CLK_SYS_HZ, MIN_FREQUENCY_HZ).is_some());
// The 50 ns spike filter is 8 cycles at 150 MHz.
const _: () = assert!(ns_to_cycles_ceil(50, 150_000_000) == 8);

// --- Errors --------------------------------------------------------------

/// What can go wrong on an I2C bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I2cError {
    /// The requested SCL frequency is outside
    /// [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`]. Returned by
    /// [`Rp2350I2c::new`] before any register is written.
    FrequencyOutOfRange {
        /// The frequency requested.
        hz: u32,
    },
    /// The address is not a usable 7-bit address: above `0x7f`, or one
    /// of the reserved `0x00`–`0x07` and `0x78`–`0x7f`, which the
    /// datasheet says not to program into `IC_TAR`
    /// ([Table 1050, p988](../../../docs/rp2350-datasheet.pdf#page=988);
    /// note in Table 1057, p1012).
    InvalidAddress(u8),
    /// Nothing to write and nothing to read. The controller has no way to
    /// send an address with no data byte.
    EmptyTransfer,
    /// No device acknowledged the address (`ABRT_7B_ADDR_NOACK`).
    AddressNack,
    /// The device acknowledged its address but refused a data byte
    /// (`ABRT_TXDATA_NOACK`).
    DataNack,
    /// Another controller won the bus (`ARB_LOST`).
    ArbitrationLost,
    /// Aborted for another reason; the raw `IC_TX_ABRT_SOURCE` value
    /// (Table 1084, p1033–1038).
    Aborted(u32),
    /// No byte moved for [`STALL_TIMEOUT_MS`]. The controller has been
    /// reset and reprogrammed; the bus may still be held by a device.
    Timeout,
}

// --- Pins ----------------------------------------------------------------

/// The `FUNCSEL` values of an SDA/SCL pin pair for I2C `I`, worked out at
/// compile time from Table 644 ([`pinmux`]).
struct I2cPins<const I: usize, const SDA: usize, const SCL: usize>;

impl<const I: usize, const SDA: usize, const SCL: usize> I2cPins<I, SDA, SCL> {
    /// `FUNCSEL` for GPIO `SDA` as I2C `I` SDA: column F3.
    const SDA_FUNCSEL: u32 = match pinmux::funcsel(SDA, I, Role::I2cSda) {
        // The pin carries this controller's SDA.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this I2C controller's SDA (RP2350 datasheet Table 644)"),
    };
    /// `FUNCSEL` for GPIO `SCL` as I2C `I` SCL: column F3.
    const SCL_FUNCSEL: u32 = match pinmux::funcsel(SCL, I, Role::I2cScl) {
        // The pin carries this controller's SCL.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this I2C controller's SCL (RP2350 datasheet Table 644)"),
    };
}

/// Pad settings for both I2C pins (§12.2.1.3, p982): pull-up on, slow
/// slew. [`pinmux::connect`] leaves the Schmitt trigger on.
const I2C_PAD: PadConfig = PadConfig { pull_up: true, slew_fast: false };

// --- Driver --------------------------------------------------------------

/// I2C controller `I` (0 or 1), configured as the bus controller and
/// connected to its pins.
///
/// `new` consumes the board's `DeviceHandle<Rp2350I2c<I>>` and both pin
/// handles, so in safe code at most one driver exists per controller.
pub struct Rp2350I2c<const I: usize> {
    /// The SCL counts, kept to reprogram the block after a timeout.
    timing: SclTiming,
    /// The address in `IC_TAR` with the controller enabled, or `None`
    /// while it is disabled.
    target: Option<u8>,
}

impl<const I: usize> Rp2350I2c<I> {
    /// There are two controllers ([§12.2, p981](../../../docs/rp2350-datasheet.pdf#page=981)).
    const INSTANCE_EXISTS: () = assert!(I < 2, "RP2350 has I2C0 and I2C1 only");

    /// This instance's registers: `I2C0_BASE` or `I2C1_BASE` (§2.2.4,
    /// p32–33).
    const fn regs() -> *mut I2cRegs {
        // I is 0 or 1, checked by INSTANCE_EXISTS in `new`.
        (if I == 0 { RegAddr::I2C0 } else { RegAddr::I2C1 }) as usize as *mut I2cRegs
    }

    /// Reset and configure controller `I` for SCL at no more than `hz`,
    /// then connect GPIO `SDA` and GPIO `SCL` to it. The steps are in the
    /// [module documentation](self#bring-up).
    ///
    /// `SDA` must be an even pin and `SCL` an odd pin whose F3 column in
    /// Table 644 is this controller; any other pin is a **build error**.
    /// On the Pico 2: I2C0 on GPIO0/1, 4/5, 8/9, 12/13, 16/17, 20/21,
    /// 24/25, 28/29; I2C1 on GPIO2/3, 6/7, 10/11, 14/15, 18/19, 22/23,
    /// 26/27.
    pub fn new<const SDA: usize, const SCL: usize>(
        _handle: DeviceHandle<Self>,
        _clocks: &Rp2350Clocks,
        _sda: PinHandle<SDA>,
        _scl: PinHandle<SCL>,
        hz: u32,
    ) -> Result<Self, I2cError> {
        // Evaluating these turns a wrong instance or pin into a build error.
        let () = Self::INSTANCE_EXISTS;
        // FUNCSEL for each pin, or a build error naming the bad pin.
        let sda_funcsel = I2cPins::<I, SDA, SCL>::SDA_FUNCSEL;
        let scl_funcsel = I2cPins::<I, SDA, SCL>::SCL_FUNCSEL;
        // Check the frequency before touching any register.
        let Some(timing) = SclTiming::for_frequency(CLK_SYS_HZ, hz) else {
            return Err(I2cError::FrequencyOutOfRange { hz });
        };
        // The driver value; `configure` fills the registers.
        let mut bus = Self { timing, target: None };
        unsafe {
            // Pins are routed through IO_BANK0/PADS_BANK0: make sure they run.
            pinmux::release_io_banks();
        }
        // Reset and program the controller, left disabled.
        bus.configure();
        unsafe {
            // Step 4: SDA and SCL, pulled up, slow slew (§12.2.1.3, p982).
            pinmux::connect(SDA, sda_funcsel, I2C_PAD);
            pinmux::connect(SCL, scl_funcsel, I2C_PAD);
        }
        // Ready; the first transfer sets IC_TAR and enables.
        Ok(bus)
    }

    /// The SCL frequency produced, rounded down, before the bus's rise
    /// time slows it further.
    pub fn frequency_hz(&self) -> u32 {
        // From the programmed counts.
        self.timing.frequency_hz(CLK_SYS_HZ)
    }

    /// Reset the block and program steps 1–3 of the bring-up, leaving it
    /// disabled. Used by `new` and to recover from a timeout; the pins
    /// stay connected because `IO_BANK0` and `PADS_BANK0` are not reset.
    fn configure(&mut self) {
        // Register block of this controller.
        let regs = Self::regs();
        // Short name for the timing values.
        let t = self.timing;
        unsafe {
            // Full reset: every register back to its reset value (Table 534, p504).
            cycle_reset(1 << (RESET_I2C0_BIT + I));
            // Step 1: disabled, so the writes below take effect (Table 1079, p1030).
            (&raw mut (*regs).enable).write_volatile(0);
            // Step 2: controller mode, target off, 7-bit (bit 4 = 0),
            // RESTART on, speed, hold bus when RX full (Table 1055).
            (&raw mut (*regs).con).write_volatile(
                CON_MASTER_MODE
                    | (t.speed << CON_SPEED_SHIFT)
                    | CON_RESTART_EN
                    | CON_SLAVE_DISABLE
                    | CON_RX_FIFO_FULL_HLD,
            );
            // Step 3: SCL counts for the active speed mode (Table 1051, p1002).
            if t.speed == SPEED_STANDARD {
                // Standard mode uses the SS pair (Tables 1059–1060).
                (&raw mut (*regs).ss_scl_hcnt).write_volatile(t.hcnt);
                (&raw mut (*regs).ss_scl_lcnt).write_volatile(t.lcnt);
            } else {
                // Fast and fast-plus use the FS pair (Tables 1061–1062).
                (&raw mut (*regs).fs_scl_hcnt).write_volatile(t.hcnt);
                (&raw mut (*regs).fs_scl_lcnt).write_volatile(t.lcnt);
            }
            // Spike filter, used in every mode (Table 1092, p1042).
            (&raw mut (*regs).fs_spklen).write_volatile(t.spklen);
        }
        // Disabled: no target programmed.
        self.target = None;
    }

    /// Make `address` the target with the controller enabled. `IC_TAR` can
    /// only be written while disabled (`IC_TAR` description, p1010), so a new
    /// address costs a disable and an enable; the same address costs
    /// nothing.
    fn select(&mut self, address: u8) -> Result<(), I2cError> {
        // Already enabled for this device.
        if self.target == Some(address) {
            return Ok(());
        }
        // Register block of this controller.
        let regs = Self::regs();
        // Disable, and wait until the hardware says it is off (§12.2.10.3).
        self.disable()?;
        unsafe {
            // Target address in bits 9:0; SPECIAL (bit 11) = 0 (Table 1056, p1011).
            (&raw mut (*regs).tar).write_volatile(address as u32);
            // Enable: transfers may start (Table 1079, p1030).
            (&raw mut (*regs).enable).write_volatile(ENABLE_ENABLE);
        }
        // Remember what is programmed.
        self.target = Some(address);
        Ok(())
    }

    /// Clear `IC_ENABLE.ENABLE` and wait for `IC_ENABLE_STATUS.IC_EN` to
    /// read 0: the procedure of
    /// [§12.2.10.3.1, p999](../../../docs/rp2350-datasheet.pdf#page=999),
    /// with [`STALL_LOOKS`] as its `MAX_T_POLL_COUNT`. Only called between
    /// transfers, after a STOP, so the bus is idle and the controller
    /// stops at once.
    fn disable(&mut self) -> Result<(), I2cError> {
        // Register block of this controller.
        let regs = Self::regs();
        // From here on IC_TAR is not known to be valid.
        self.target = None;
        unsafe {
            // ENABLE = 0 (and ABORT = 0, which software cannot set anyway).
            (&raw mut (*regs).enable).write_volatile(0);
            // Bounded wait for IC_EN to read 0 (Table 1091, p1042).
            for _ in 0..STALL_LOOKS {
                if (&raw const (*regs).enable_status).read_volatile() & ENABLE_STATUS_IC_EN == 0 {
                    return Ok(());
                }
            }
        }
        // Never shut down: recover as for any other stall.
        self.recover();
        Err(I2cError::Timeout)
    }

    /// Recover from a stalled bus or controller: reset and reprogram the
    /// block. §12.2.13.2 (p1001): for SCL stuck low "reset the bus using
    /// the hardware reset signal".
    fn recover(&mut self) {
        // Reset + steps 1–3; leaves the block disabled, target None.
        self.configure();
    }

    /// One whole transfer: send `write`, then read into `read`, with a
    /// RESTART between them if both are non-empty and a STOP at the end.
    /// See [Transfers](self#transfers).
    fn transfer(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<(), I2cError> {
        // 7-bit only, and not a reserved address (Table 1050, p988).
        if address > 0x7f || address <= 0x07 || address >= 0x78 {
            return Err(I2cError::InvalidAddress(address));
        }
        // The controller cannot send an address alone.
        if write.is_empty() && read.is_empty() {
            return Err(I2cError::EmptyTransfer);
        }
        // IC_TAR = address, controller enabled.
        self.select(address)?;
        // Register block of this controller.
        let regs = Self::regs();
        unsafe {
            // Reading the clear registers drops stale flags from earlier
            // transfers (Tables 1073 and 1076, p1027–1028).
            let _ = (&raw const (*regs).clr_tx_abrt).read_volatile();
            let _ = (&raw const (*regs).clr_stop_det).read_volatile();
        }
        // Commands to queue: one per byte written, one per byte read.
        let total = write.len() + read.len();
        // Commands queued so far.
        let mut queued = 0;
        // Bytes received so far.
        let mut received = 0;
        // Status reads since a byte last moved.
        let mut idle_looks = 0;
        // Run until every command is queued and every byte received.
        while queued < total || received < read.len() {
            // One read of IC_STATUS for this pass (Table 1080, p1031).
            let status = unsafe { (&raw const (*regs).status).read_volatile() };
            // Abort: the controller flushed both FIFOs (Table 1065, p1023).
            if unsafe { (&raw const (*regs).raw_intr_stat).read_volatile() } & INTR_TX_ABRT != 0 {
                return Err(self.finish_abort());
            }
            // Did this pass move a byte?
            let mut progress = false;
            // Read requests queued but not yet answered.
            let outstanding = queued.saturating_sub(write.len()) - received;
            // Queue the next command if the TX FIFO has room, and, for a
            // read, if the RX FIFO is sure to have room for its byte.
            let room_for_read = queued < write.len() || outstanding < RX_FIFO_DEPTH;
            if queued < total && status & STATUS_TFNF != 0 && room_for_read {
                // STOP after the last command (Table 1058, p1013).
                let stop = if queued == total - 1 { CMD_STOP } else { 0 };
                let command = if queued < write.len() {
                    // A data byte to send: CMD = 0, DAT = the byte.
                    write[queued] as u32 | stop
                } else {
                    // A byte to receive: CMD = 1; RESTART on the first read
                    // after a write, to turn the bus around (p1013).
                    let restart = if queued == write.len() && !write.is_empty() { CMD_RESTART } else { 0 };
                    CMD_READ | restart | stop
                };
                // Push the command into the TX FIFO.
                unsafe { (&raw mut (*regs).data_cmd).write_volatile(command) };
                queued += 1;
                progress = true;
            }
            // Collect a received byte if one is waiting (IC_STATUS.RFNE).
            if received < read.len() && status & STATUS_RFNE != 0 {
                // Reading IC_DATA_CMD pops the RX FIFO; DAT is bits 7:0.
                let value = unsafe { (&raw const (*regs).data_cmd).read_volatile() };
                read[received] = (value & CMD_DATA) as u8;
                received += 1;
                progress = true;
            }
            // Count passes without progress; give up after STALL_LOOKS.
            if progress {
                idle_looks = 0;
            } else {
                idle_looks += 1;
                if idle_looks > STALL_LOOKS {
                    return Err(self.timeout());
                }
            }
        }
        // Every byte is through; wait for the STOP to go out.
        for _ in 0..STALL_LOOKS {
            // One read of the raw status per pass (Table 1065).
            let raw = unsafe { (&raw const (*regs).raw_intr_stat).read_volatile() };
            // A NACK on the last written byte aborts after it was queued.
            if raw & INTR_TX_ABRT != 0 {
                return Err(self.finish_abort());
            }
            // STOP sent: clear the flag and report success.
            if raw & INTR_STOP_DET != 0 {
                unsafe { let _ = (&raw const (*regs).clr_stop_det).read_volatile(); }
                return Ok(());
            }
        }
        // The STOP never appeared.
        Err(self.timeout())
    }

    /// After `TX_ABRT`: read why, release the FIFOs, and wait for the STOP
    /// the controller sends. Returns the error to report.
    fn finish_abort(&mut self) -> I2cError {
        // Register block of this controller.
        let regs = Self::regs();
        // Read the cause before clearing it (Table 1084, p1033–1038).
        let source = unsafe { (&raw const (*regs).tx_abrt_source).read_volatile() };
        // Reading IC_CLR_TX_ABRT clears TX_ABRT; the FIFOs stay flushed
        // until this read (Table 1065, p1023).
        unsafe { let _ = (&raw const (*regs).clr_tx_abrt).read_volatile(); }
        // Wait, bounded, for the STOP; an abort is followed by one (§12.2.10.4, p999).
        let mut stopped = false;
        for _ in 0..STALL_LOOKS {
            if unsafe { (&raw const (*regs).raw_intr_stat).read_volatile() } & INTR_STOP_DET != 0 {
                stopped = true;
                break;
            }
        }
        if stopped {
            // Clear STOP_DET for the next transfer (Table 1076, p1028).
            unsafe { let _ = (&raw const (*regs).clr_stop_det).read_volatile(); }
        } else {
            // No STOP: the bus is stuck; reset the block.
            self.recover();
        }
        // Translate the most specific cause.
        if source & ABRT_7B_ADDR_NOACK != 0 {
            I2cError::AddressNack
        } else if source & ABRT_TXDATA_NOACK != 0 {
            I2cError::DataNack
        } else if source & ABRT_ARB_LOST != 0 {
            I2cError::ArbitrationLost
        } else {
            I2cError::Aborted(source)
        }
    }

    /// A transfer stalled: ask the controller to abort (`IC_ENABLE.ABORT`,
    /// [§12.2.10.4.1, p999](../../../docs/rp2350-datasheet.pdf#page=999)),
    /// then reset it regardless, since a stall usually means a device is
    /// holding SCL low and the abort cannot complete.
    fn timeout(&mut self) -> I2cError {
        // Register block of this controller.
        let regs = Self::regs();
        // ABORT is only accepted while ENABLE is set (Table 1079, p1030).
        unsafe { (&raw mut (*regs).enable).write_volatile(ENABLE_ENABLE | ENABLE_ABORT) };
        // Give the abort one stall period to send its STOP.
        for _ in 0..STALL_LOOKS {
            if unsafe { (&raw const (*regs).raw_intr_stat).read_volatile() } & INTR_TX_ABRT != 0 {
                break;
            }
        }
        // Whatever happened, start again from reset (§12.2.13.2, p1001).
        self.recover();
        I2cError::Timeout
    }
}

impl<const I: usize> ErrorType for Rp2350I2c<I> {
    type Error = I2cError;
}

/// The portable trait: each method is one transfer through
/// `Rp2350I2c::transfer`.
impl<const I: usize> I2c for Rp2350I2c<I> {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), I2cError> {
        // Write only: nothing to read.
        self.transfer(address, bytes, &mut [])
    }

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), I2cError> {
        // Read only: nothing to write.
        self.transfer(address, &[], buffer)
    }

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), I2cError> {
        // Write, RESTART, read, STOP.
        self.transfer(address, bytes, buffer)
    }
}
