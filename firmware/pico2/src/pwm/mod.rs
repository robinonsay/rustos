//! PWM: one output per slice, frequency and duty from `clk_sys`.
//!
//! [`pwm::Rp2350Pwm`] releases the PWM block from reset and turns a pin
//! handle into a [`pwm::Rp2350PwmOut`], which implements
//! [`api::pwm::PwmOutput`]. On the RP2350A, GPIO `n` is channel A (even `n`)
//! or B (odd `n`) of slice `(n / 2) mod 8` (Table 1129), so only slices 0 to 7
//! are reachable (cwht coding standard CS-36). The two channels of a slice
//! share its period; this driver lets one pin per slice be a PWM output, so a
//! frequency change can never move another output.
//!
//! The decisions are pure functions tested on the host:
//!
//! * [`timing`]: divider and period for a frequency, or a refusal;
//! * [`cc_value`]: the compare value for a duty, on or off.
//!
//! ## Datasheet sources
//!
//! RP2350 datasheet, build 2025-02-20 (register ICD extract:
//! `docs/icd/rp2350/pwm/`):
//!
//! - §12.5.2, Table 1129 (p1074): GPIO to slice and channel map
//! - §12.5.2.2 (p1076): `CC = 0` is 0 %, `CC = TOP + 1` is 100 %, glitch-free
//! - §12.5.2.3 (p1077): `CC` and `TOP` are double-buffered, latched at the wrap
//! - §12.5.2.6 (p1079, p1080): period `(TOP + 1) × (DIV_INT + DIV_FRAC / 16)` in `clk_sys` cycles
//! - §12.5.3, Tables 1130 to 1136 (p1083 to p1087): registers
//! - §7.5, Table 534 (p504): `RESETS` bit 16, `PWM`
//! - §9.4, Table 650 (p608): `FUNCSEL` 4 selects PWM; §9.11.3, Table 852 (p785): pad `ISO`, `OD`

pub mod pwm;

use core::mem::offset_of;

/// One slice's registers; slice `n` starts at `0x14 × n` (Table 1130).
#[repr(C)]
struct SliceRegs {
    /// `CHn_CSR`: `EN` bit 0, `PH_CORRECT` bit 1, `A_INV` 2, `B_INV` 3, `DIVMODE` 5:4 (Table 1131).
    csr: u32,
    /// `CHn_DIV`: `INT` 11:4, `FRAC` 3:0 (Table 1132).
    div: u32,
    /// `CHn_CTR`: the counter, 15:0 (Table 1133).
    ctr: u32,
    /// `CHn_CC`: `B` 31:16, `A` 15:0 (Table 1134).
    cc: u32,
    /// `CHn_TOP`: wrap value, 15:0, reset `0xffff` (Table 1135).
    top: u32,
}

pub(crate) const SLICE_STRIDE: usize = core::mem::size_of::<SliceRegs>();
pub(crate) const CSR: usize = offset_of!(SliceRegs, csr);
pub(crate) const DIV: usize = offset_of!(SliceRegs, div);
pub(crate) const CTR: usize = offset_of!(SliceRegs, ctr);
pub(crate) const CC: usize = offset_of!(SliceRegs, cc);
pub(crate) const TOP: usize = offset_of!(SliceRegs, top);
/// `EN` `0x0f0`: one enable bit per slice (Table 1136).
pub(crate) const EN: usize = 0x0f0;
const _: () =
    assert!(SLICE_STRIDE == 0x14 && CSR == 0 && DIV == 4 && CTR == 8 && CC == 0xc && TOP == 0x10);
const _: () = assert!(EN == 12 * SLICE_STRIDE);

/// `CHn_CSR.EN`, free-running, trailing-edge, not inverted (Table 1131).
pub(crate) const CSR_EN: u32 = 1;
/// `RESETS.RESET` bit of PWM (Table 534).
pub(crate) const RESET_BIT_PWM: u32 = 1 << 16;
/// Reads allowed for `RESET_DONE` after the release (as for TIMER0).
pub(crate) const RESET_POLL_BUDGET: u32 = 10_000;
/// `IO_BANK0.GPIOn_CTRL` offset: `8n + 4` (§9.11.1).
pub(crate) const fn gpio_ctrl(pin: usize) -> usize {
    8 * pin + 4
}
/// `PADS_BANK0.GPIOn` offset: `4n + 4` (Table 852).
pub(crate) const fn pad_ctrl(pin: usize) -> usize {
    4 * pin + 4
}
/// `GPIOn_CTRL` value: `FUNCSEL = 4` (PWM), every override field 0 (Table 650).
pub(crate) const FUNCSEL_PWM: u32 = 4;
/// Pad `ISO` (bit 8) and `OD` (bit 7): both cleared so the PWM drives the pad (Table 852).
pub(crate) const PAD_ISO_OD: u32 = (1 << 8) | (1 << 7);

/// Longest period in counts: `TOP = 65534`, so that `CC = TOP + 1` (100 %)
/// still fits the 16-bit `CC` field (§12.5.2.2, Table 1134).
pub const MAX_PERIOD: u32 = 65_535;
/// Shortest period in counts (`TOP = 1`): a square wave needs two counts.
pub const MIN_PERIOD: u32 = 2;
/// Largest divider in sixteenths: `INT = 255`, `FRAC = 15` (Table 1132).
pub const MAX_DIV16: u32 = 4_095;
/// Smallest divider in sixteenths: 1.0, one count per `clk_sys` cycle.
pub const MIN_DIV16: u32 = 16;
/// Frequency a new output starts with, disabled (PWM-6).
pub const DEFAULT_HZ: u32 = 1_000;

/// A slice setting for one frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Timing {
    /// `CHn_DIV` value: the divider in sixteenths, `INT << 4 | FRAC`.
    pub div16: u32,
    /// Counts per period, `TOP + 1`.
    pub period: u32,
    /// Achieved frequency in millihertz.
    pub achieved_mhz: u64,
}

/// Why a PWM operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwmError {
    /// `RESETS.RESET_DONE` did not report PWM within the poll budget.
    ResetTimeout,
    /// The other channel of this pin's slice is already a PWM output.
    SliceInUse,
    /// The frequency is 0, outside what the divider and period can make, or
    /// cannot be made within 0.1 % (PWM-1, PWM-2).
    FrequencyOutOfRange,
}

/// Divider and period for `hz` from a `clk_hz` count clock. Pure.
///
/// Chooses the smallest divider that fits the period in 16 bits (the finest
/// period resolution), rounds the period to the nearest count, and refuses a
/// result more than 0.1 % from the request.
///
/// # Errors
///
/// [`PwmError::FrequencyOutOfRange`] as described.
pub(crate) fn timing(clk_hz: u32, hz: u32) -> Result<Timing, PwmError> {
    if hz == 0 {
        return Err(PwmError::FrequencyOutOfRange);
    }
    let clk16 = u64::from(clk_hz) * 16;
    let f = u64::from(hz);
    let div16 = clk16
        .div_ceil(f * u64::from(MAX_PERIOD))
        .max(u64::from(MIN_DIV16));
    if div16 > u64::from(MAX_DIV16) {
        return Err(PwmError::FrequencyOutOfRange);
    }
    let period = (clk16 + div16 * f / 2) / (div16 * f);
    if period < u64::from(MIN_PERIOD) || period > u64::from(MAX_PERIOD) {
        return Err(PwmError::FrequencyOutOfRange);
    }
    let achieved_mhz = (clk16 * 1000 + div16 * period / 2) / (div16 * period);
    if achieved_mhz.abs_diff(f * 1000) > f {
        return Err(PwmError::FrequencyOutOfRange);
    }
    // div16 <= 4095 and period <= 65535 were checked, so both fit in u32.
    match (u32::try_from(div16), u32::try_from(period)) {
        (Ok(div16), Ok(period)) => Ok(Timing {
            div16,
            period,
            achieved_mhz,
        }),
        _ => Err(PwmError::FrequencyOutOfRange),
    }
}

/// `CC` field value for `permille` of `period` counts, or 0 when off. Pure.
///
/// Rounds to the nearest count; 0 ‰ gives 0 (constant low) and 1000 ‰ gives
/// `period` (= `TOP + 1`, constant high), the glitch-free ends of §12.5.2.2.
#[must_use]
pub(crate) fn cc_value(permille: u16, period: u32, on: bool) -> u32 {
    if !on {
        return 0;
    }
    (u32::from(permille) * period + 500) / 1000
}

#[cfg(test)]
mod tests;
