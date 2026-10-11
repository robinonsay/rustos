//! The `DMA` block: a reset bring-up and one ownership token per channel.
//!
//! # Using it
//!
//! ```ignore
//! let board = Rp2350::take().unwrap();
//! let dma = Rp2350Dma::new(board.dma);   // DMA reset: every channel stopped
//! let ch0 = dma.ch0;                      // hand a channel to a driver, e.g. crate::adc
//! ```
//!
//! This module only resets the block and hands out the channels. A channel
//! is used by the driver it is given to, which programs it for one job;
//! [`crate::adc`] runs one endlessly from the ADC's sample FIFO into a ring
//! buffer in RAM.
//!
//! # The block
//!
//! Sixteen independent channels, each with four live registers —
//! `READ_ADDR`, `WRITE_ADDR`, `TRANS_COUNT` and `CTRL` — that software
//! programs and then triggers, and that update as the channel runs
//! (§12.6.2, p1094). One read and one write of up to 32 bits can complete
//! every `clk_sys` cycle, shared round-robin between the channels that want
//! to transfer (§12.6, p1093). A channel paced by a peripheral's data
//! request (DREQ) transfers only when the peripheral has data or room
//! (§12.6.4, p1098), and two channels must never be given the same DREQ
//! (§12.6.4.2, p1100): handing out each channel once, as a token, is half
//! of that rule, and the driver that owns the peripheral is the other half.
//!
//! # Bring-up requirements
//!
//! * **Reset.** `DMA` is bit 2 of `RESETS.RESET` (Table 534, p504).
//!   [`Rp2350Dma::new`] puts it through a full reset cycle, so whatever ran
//!   before — a debugger warm reset restarts the processors, not the
//!   peripherals — every channel starts disabled and idle (`CTRL.EN` and
//!   `CTRL.BUSY` reset 0, Table 1150, p1125–1127), and every channel's
//!   `SECCFG` lock is clear, which only a reset of the whole block can do
//!   (§12.6.6.1, p1102).
//! * **Clock.** The DMA runs on `clk_sys` through the `CLK_SYS_DMA` gate,
//!   enabled out of reset (`WAKE_EN0` bit 7, reset 1, Table 586, p546) and
//!   not touched by [`crate::clocks`]. No transfer here depends on the
//!   frequency, so `new` does not ask for [`Rp2350Clocks`](crate::clocks::Rp2350Clocks).
//! * **Security.** At reset every channel is Secure and Privileged
//!   (`SECCFG_CHn` S = 1, P = 1, Table 1183, p1136), the level this crate's
//!   code runs at, so its registers are reachable and its transfers are
//!   made at the highest level. The DMA's own memory-protection unit lets
//!   such a channel reach every address: with no region enabled, an address
//!   counts as Non-secure and Unprivileged (`MPU_CTRL` S = 0, P = 0, Table
//!   1186, p1138), the lowest level. The bus filters let the DMA reach
//!   SRAM ("Defaults to fully open access", Table 918, p831) and the ADC
//!   ("Defaults to Secure access from any master", Table 942, p847), and
//!   let the processors reach the DMA's registers (Table 928, p837–838).
//!
//! No interrupt is enabled: `INTE0`–`INTE3` stay at their reset value 0
//! (§12.6.5, p1100), so a channel's completion or bus error only sets its
//! flag.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::device::DeviceHandle;

use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

// --- Register layouts ----------------------------------------------------

/// One channel's registers (Table 1146, p1110–1111): the four live
/// registers, then the same four in three more orders (Table 1144, p1097),
/// `0x40` bytes in all.
#[repr(C)]
pub(crate) struct Channel {
    /// `CHn_READ_ADDR` (Table 1147, p1123): the next address the channel
    /// reads.
    pub read_addr: u32,
    /// `CHn_WRITE_ADDR` (Table 1148, p1124): the next address the channel
    /// writes, updated "each time a write completes".
    pub write_addr: u32,
    /// `CHn_TRANS_COUNT` (Table 1149, p1124): `MODE` bits 31:28 and the
    /// transfer count, bits 27:0.
    pub trans_count: u32,
    /// `CHn_CTRL_TRIG` (Table 1150, p1125–1127). Writing it **triggers** the
    /// channel when the value written has `EN` set and is not 0 (§12.6.3.1,
    /// p1097).
    pub ctrl_trig: u32,
    /// `CHn_AL1_CTRL` (Table 1151, p1127): `CTRL` again, at an address that
    /// is not a trigger. Write here to change `CTRL` without starting the
    /// channel.
    pub al1_ctrl: u32,
    /// The rest of the three alias groups, `+0x14`–`+0x3f`. Not used.
    _aliases: [u32; 11],
}

/// The `DMA` block, base `0x5000_0000` (§12.6.10, p1110). Only the
/// registers this crate uses are named.
#[repr(C)]
pub(crate) struct Dma {
    /// `CH0_*`–`CH15_*`, `0x000`–`0x3ff`. Index by channel number.
    pub ch: [Channel; CHANNELS],
    /// `0x400`–`0x463`: interrupt, pacing timer, trigger, sniffer and debug
    /// FIFO registers (Table 1146, p1119). Not used.
    _between: [u32; 25],
    /// `0x464` `CHAN_ABORT` (Table 1181, p1135): writing 1 to a channel's
    /// bit aborts its transfer sequence; the bit "will remain high until any
    /// in-flight transfers have been flushed", and the channel must not be
    /// restarted before it reads 0.
    pub chan_abort: u32,
}

const _: () = assert!(core::mem::size_of::<Channel>() == 0x40);
const _: () = assert!(core::mem::offset_of!(Channel, ctrl_trig) == 0x0c);
const _: () = assert!(core::mem::offset_of!(Channel, al1_ctrl) == 0x10);
const _: () = assert!(core::mem::offset_of!(Dma, chan_abort) == 0x464);

/// Number of channels (§12.6, p1093).
pub(crate) const CHANNELS: usize = 16;

/// The registers.
pub(crate) const fn dma() -> *mut Dma {
    RegAddr::DMA as usize as *mut Dma
}

/// `CTRL.EN`, bit 0: "When 1, the channel will respond to triggering
/// events ... When 0, the channel will ignore triggers, stop issuing
/// transfers, and pause the current transfer sequence" (Table 1150, p1127).
pub(crate) const CTRL_EN: u32 = 1 << 0;
/// `CTRL.DATA_SIZE`, bits 3:2: 0 byte, 1 halfword, 2 word (Table 1150,
/// p1126).
pub(crate) const CTRL_DATA_SIZE_SHIFT: u32 = 2;
/// `CTRL.DATA_SIZE` value for halfwords.
pub(crate) const DATA_SIZE_HALFWORD: u32 = 1;
/// `CTRL.INCR_WRITE`, bit 6: the write address advances by the transfer
/// size after each write (Table 1150, p1126).
pub(crate) const CTRL_INCR_WRITE: u32 = 1 << 6;
/// `CTRL.RING_SIZE`, bits 11:8: with value *n* > 0, "only the lower n bits
/// of the address will change", wrapping on a 2*ⁿ*-byte boundary (Table
/// 1150, p1126).
pub(crate) const CTRL_RING_SIZE_SHIFT: u32 = 8;
/// `CTRL.RING_SEL`, bit 12: 1 applies `RING_SIZE` to the write address
/// (Table 1150, p1126).
pub(crate) const CTRL_RING_SEL_WRITE: u32 = 1 << 12;
/// `CTRL.CHAIN_TO`, bits 16:13. It "resets to 0, so channels 1 and above
/// will chain to channel 0 by default"; a channel's own number disables
/// chaining (Table 1150, p1126).
pub(crate) const CTRL_CHAIN_TO_SHIFT: u32 = 13;
/// `CTRL.TREQ_SEL`, bits 22:17: 0x00–0x3a select DREQ *n* (Table 1150,
/// p1125).
pub(crate) const CTRL_TREQ_SEL_SHIFT: u32 = 17;
/// `CTRL.AHB_ERROR`, bit 31 (read-only): `READ_ERROR` (bit 30) or
/// `WRITE_ERROR` (bit 29). "The channel halts when it encounters any bus
/// error" (Table 1150, p1125).
pub(crate) const CTRL_AHB_ERROR: u32 = 1 << 31;

/// `TRANS_COUNT.MODE`, bits 31:28 (Table 1149, p1124).
pub(crate) const TRANS_COUNT_MODE_SHIFT: u32 = 28;
/// `TRANS_COUNT.MODE` value `ENDLESS`: "the transfer count does not
/// decrement. The DMA channel performs an endless sequence of transfers,
/// never triggering other channels or raising interrupts, until an ABORT is
/// raised" (Table 1149, p1124).
pub(crate) const MODE_ENDLESS: u32 = 0xf;

/// `DREQ_ADC`, the ADC's data request: number 48 in the system DREQ table
/// (Table 1145, p1099).
pub(crate) const DREQ_ADC: u32 = 48;

/// `DMA` — bit 2 of `RESETS.RESET` (Table 534, p504).
const RESET_DMA: u32 = 1 << 2;

// --- Driver --------------------------------------------------------------

/// Ownership of DMA channel `C`.
///
/// Zero-sized, like a [`PinHandle`](api::device::PinHandle): holding one is
/// what proves that no other code programs channel `C`. The only ones that
/// exist are the sixteen fields of the [`Rp2350Dma`] that
/// [`Rp2350Dma::new`] returns, so each channel can be given to at most one
/// driver per boot. A driver takes it by value and keeps the channel for
/// as long as it runs.
pub struct Rp2350DmaChannel<const C: usize> {
    _private: (),
}

impl<const C: usize> Rp2350DmaChannel<C> {
    /// Channel `C` exists. Referenced by every function that programs the
    /// channel, so it is evaluated, unlike an unreferenced associated const.
    pub(crate) const EXISTS: () = assert!(C < CHANNELS);

    /// The channel number.
    pub const fn number(&self) -> usize {
        let () = Self::EXISTS;
        C
    }

    /// Only [`Rp2350Dma::new`] makes these.
    const fn new() -> Self {
        Self { _private: () }
    }
}

/// The DMA block, reset, with every channel stopped: one
/// [`Rp2350DmaChannel`] per channel, to hand to the drivers that use them.
pub struct Rp2350Dma {
    /// Channel 0.
    pub ch0: Rp2350DmaChannel<0>,
    /// Channel 1.
    pub ch1: Rp2350DmaChannel<1>,
    /// Channel 2.
    pub ch2: Rp2350DmaChannel<2>,
    /// Channel 3.
    pub ch3: Rp2350DmaChannel<3>,
    /// Channel 4.
    pub ch4: Rp2350DmaChannel<4>,
    /// Channel 5.
    pub ch5: Rp2350DmaChannel<5>,
    /// Channel 6.
    pub ch6: Rp2350DmaChannel<6>,
    /// Channel 7.
    pub ch7: Rp2350DmaChannel<7>,
    /// Channel 8.
    pub ch8: Rp2350DmaChannel<8>,
    /// Channel 9.
    pub ch9: Rp2350DmaChannel<9>,
    /// Channel 10.
    pub ch10: Rp2350DmaChannel<10>,
    /// Channel 11.
    pub ch11: Rp2350DmaChannel<11>,
    /// Channel 12.
    pub ch12: Rp2350DmaChannel<12>,
    /// Channel 13.
    pub ch13: Rp2350DmaChannel<13>,
    /// Channel 14.
    pub ch14: Rp2350DmaChannel<14>,
    /// Channel 15.
    pub ch15: Rp2350DmaChannel<15>,
}

impl Rp2350Dma {
    /// Reset the DMA block (Table 534, p504) and wait until it reports
    /// ready, so every register holds its reset value and every channel is
    /// stopped, then hand out the sixteen channels.
    ///
    /// The board's [`DeviceHandle<Rp2350Dma>`] is consumed, so this runs at
    /// most once per boot and each channel token exists once.
    pub fn new(_handle: DeviceHandle<Rp2350Dma>) -> Self {
        unsafe { cycle_reset(RESET_DMA) };
        Self {
            ch0: Rp2350DmaChannel::new(),
            ch1: Rp2350DmaChannel::new(),
            ch2: Rp2350DmaChannel::new(),
            ch3: Rp2350DmaChannel::new(),
            ch4: Rp2350DmaChannel::new(),
            ch5: Rp2350DmaChannel::new(),
            ch6: Rp2350DmaChannel::new(),
            ch7: Rp2350DmaChannel::new(),
            ch8: Rp2350DmaChannel::new(),
            ch9: Rp2350DmaChannel::new(),
            ch10: Rp2350DmaChannel::new(),
            ch11: Rp2350DmaChannel::new(),
            ch12: Rp2350DmaChannel::new(),
            ch13: Rp2350DmaChannel::new(),
            ch14: Rp2350DmaChannel::new(),
            ch15: Rp2350DmaChannel::new(),
        }
    }
}

/// Stop channel `ch` for good, whatever it is doing, and wait until it has
/// nothing left in flight: the abort sequence of §12.6.8.3 (p1105–1106),
/// which includes the workaround for erratum RP2350-E5 (Appendix E, p1357):
///
/// 1. Clear `EN`, through the non-trigger `AL1_CTRL` alias so the write
///    cannot start anything. A channel that is not enabled ignores the
///    re-trigger E5 describes, on the last cycle of an abort. (Channels
///    here never chain, so "and any channel it chains to" is the channel
///    itself.)
/// 2. Write its bit to `CHAN_ABORT`.
/// 3. Poll `CHAN_ABORT` until the bit reads 0 (Table 1181, p1135), after
///    which `BUSY` is low (§12.6.8.3, p1106).
///
/// The wait is for transfers already in flight between the DMA's read and
/// write ports, "generally ... only a few cycles" (p1106), so it is not
/// bounded.
///
/// # Safety
///
/// `ch` < [`CHANNELS`], and the caller owns the channel.
pub(crate) unsafe fn abort(ch: usize) {
    let dma = dma();
    unsafe {
        let ctrl = &raw mut (*dma).ch[ch].al1_ctrl;
        ctrl.write_volatile(ctrl.read_volatile() & !CTRL_EN);
        let abort = &raw mut (*dma).chan_abort;
        abort.write_volatile(1 << ch);
        while abort.read_volatile() & (1 << ch) != 0 {}
    }
}
