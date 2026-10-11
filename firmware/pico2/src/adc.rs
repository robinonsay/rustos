//! The `ADC`, sampling one input pin at a steady rate into a ring buffer in
//! RAM through a DMA channel, with no interrupts and nothing to service
//! faster than every quarter of a second.
//!
//! # Using it
//!
//! ```ignore
//! use pico2::adc::{AdcError, AdcRing, Rp2350Adc};
//! use pico2::clocks::Rp2350Clocks;
//! use pico2::common::board::Rp2350;
//! use pico2::dma::Rp2350Dma;
//!
//! static mut RING: AdcRing = AdcRing::new();             // 16 kB in .bss
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let dma = Rp2350Dma::new(board.dma);
//! let adc = Rp2350Adc::new(board.adc, &clocks).unwrap(); // clk_adc on, ADC reset and powered
//! let ring = unsafe { &mut *(&raw mut RING) };            // the only reference ever made
//! let mut audio = adc.stream_from_handle(board.pins.gpio26, dma.ch0, ring, 32_000).unwrap();
//! let mut block = [0u16; 64];
//! loop {
//!     match audio.read(&mut block) {
//!         Ok(n) => { /* block[..n]: the next n samples, oldest first */ }
//!         Err(AdcError::Overrun) => { /* fell behind by a quarter second: samples lost */ }
//!         Err(e) => { /* see AdcError */ }
//!     }
//! }
//! ```
//!
//! # The converter
//!
//! A 12-bit successive-approximation ADC, 9.2 effective bits (§12.4,
//! p1064), on its own 48 MHz clock, `clk_adc`. One conversion takes 96
//! `clk_adc` cycles, 2 µs (§12.4.3, p1068). On the Pico 2's RP2350A the
//! four inputs are GPIO26–29, `AINSEL` 0–3 (Table 1117, p1067); GPIO29 is
//! the board's VSYS/3 divider (see [`crate::common::board`]). In
//! free-running mode (`CS.START_MANY`) a pacing divider starts a
//! conversion every 1 + `DIV.INT` + `DIV.FRAC` / 256 cycles of `clk_adc`
//! (Table 1119, p1071; §12.4.3.2, p1069), and each result goes into an
//! eight-entry FIFO (§12.4.3.4, p1069). "If the FIFO is full when a
//! conversion completes, the sticky error flag FCS.OVER is set ... any
//! conversions that complete during this time are lost" (p1069).
//!
//! At 32 kHz eight entries are 250 µs, far too little to poll beside USB.
//! So a DMA channel empties the FIFO as each result arrives (§12.4.3.5,
//! p1070) into a 16 kB ring of 8192 samples, 256 ms at 32 kHz, and
//! software reads the ring at its own pace.
//!
//! # The ring
//!
//! The channel runs **endlessly**: `TRANS_COUNT.MODE` = `ENDLESS`, so the
//! count never decrements and the channel never stops (Table 1149, p1124),
//! with the write address **wrapping** on the ring's size, which the
//! hardware does by changing only the low 14 bits of the address (`CTRL`
//! `RING_SEL` = write, `RING_SIZE` = 14, Table 1150, p1126). That needs the
//! ring aligned to its own size, which [`AdcRing`]'s type guarantees. It
//! reads the FIFO register without incrementing, paced by `DREQ_ADC`
//! (Table 1145, p1099), and writes halfwords.
//!
//! Software finds how far the channel has got from its `WRITE_ADDR`, which
//! "updates automatically each time a write completes. The current value is
//! the next address to be written" (Table 1148, p1124): every sample before
//! it is in RAM. RP2350 no longer adjusts that value for transfers in
//! flight when the address wraps (§12.6.1, p1094), so the value read is the
//! completed position.
//!
//! An endless channel cannot count, so nothing in the hardware says how
//! many times it has gone round. [`Rp2350AdcStream::read`] tells "fell a
//! whole ring behind" from "is ahead" with a sentinel: every slot is set to
//! `0xffff` before the channel starts, and again as soon as its sample has
//! been read. The converter never writes `0xffff`: a FIFO entry has the
//! result in bits 11:0, the error flag in bit 15 and bits 14:12 reserved
//! (Table 1123, p1072). So a slot ahead of the channel still holds the
//! sentinel unless the channel has written it again since it was read —
//! that is, unless reading has fallen a whole ring behind. `read` looks at
//! the slot [`GUARD_SAMPLES`] ahead of `WRITE_ADDR`, so the channel moving
//! on a few samples between the two reads cannot look like an overrun, and
//! reports [`AdcError::Overrun`] once reading lags by
//! [`RING_SAMPLES`] − [`GUARD_SAMPLES`] samples (240 ms at 32 kHz).
//!
//! After an overrun the ring is set back to the sentinel while the channel
//! goes on writing. The slots it may write during that are the
//! [`GUARD_SAMPLES`] from where it was when the clearing began: those are
//! left alone, and reading starts from that point, so no slot ahead of the
//! channel keeps a sample and no slot to be read holds the sentinel. If the
//! channel has moved on that far by the time the clearing ends, it starts
//! again.
//!
//! # The pin
//!
//! "When using an ADC input shared with a GPIO pin, always disable the
//! pin's digital functions by setting IE low and OD high in the pin's pad
//! control register" (§12.4, p1066). On the RP2350 that is not only about
//! current: with the input buffer enabled, a pad between the logic levels
//! sources about 120 µA and settles near 2.2 V (erratum RP2350-E9,
//! Appendix E, p1358–1359), and an audio input biased at mid-supply sits
//! there all the time. [`Rp2350Adc::stream_from_handle`] therefore, in
//! order:
//!
//! 1. sets `GPIOn_CTRL` to the null function, high-impedance (Table 650,
//!    p608–609; §9.3, p586), in case a run before a processor-only reset
//!    left the pin with another;
//! 2. clears `IE`, sets `OD`, and clears both pulls (Table 852, p785). The
//!    pad's pull-down, on out of reset, is 36–113 kΩ (Table 1435, p1337) and
//!    would pull a biased input off its mid-point;
//! 3. clears `ISO` **last**. The isolation latches hold the pad's output
//!    enable, output level and pulls, and its input enable, at their reset
//!    values until `ISO` is cleared (§9.7, p594–595), so only then do the
//!    pull-down and the input buffer actually turn off.
//!
//! The ADC's input is over 100 kΩ (Table 1437, p1338), so it does not load
//! a source of a few kΩ.
//!
//! # Bring-up requirements
//!
//! * **`clk_adc`.** The ADC "requires a 48 MHz clock ... The clock must be
//!   set up correctly before enabling the ADC" (§12.4.3, p1068), and its
//!   registers time out on the bus while `clk_adc` is stopped (§12.6.7,
//!   p1103). [`Rp2350Adc::new`] starts it from `PLL_USB`, which
//!   `&Rp2350Clocks` proves is running, before touching the ADC.
//! * **Reset.** `ADC` is bit 0 of `RESETS.RESET` (Table 534, p504); `new`
//!   puts it through a full reset cycle, so every register holds its reset
//!   value (Tables 1120–1128, p1072–1074) even after a debugger warm reset.
//!   The pin needs `IO_BANK0` and `PADS_BANK0` (bits 6 and 9), which `new`
//!   releases if nothing has yet, without ever asserting their reset.
//! * **Power-up.** "Writing a 1 to CS.EN will start a short internal
//!   power-up sequence for the ADC's analogue hardware. After a few clock
//!   cycles, CS.READY will go high" (§12.4.3, p1068).
//! * **DMA.** A [`Rp2350DmaChannel`] from [`crate::dma`], which has reset
//!   the DMA block. The bus lets the DMA read the ADC and write SRAM out of
//!   reset (Table 942, p847; Table 918, p831).
//! * **Supply noise.** Not this driver's business, but driving GPIO23
//!   (`smps_ps`) high puts the Pico 2's regulator into PWM mode, which
//!   "can greatly reduce the inherent ripple of the SMPS at light load, and
//!   therefore the ripple on the ADC supply" (Pico 2 datasheet §4.4).
//!
//! No interrupt is enabled (`INTE` stays 0, Table 1126, p1073; the DMA's
//! `INTE0`–`INTE3` stay 0).
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use core::sync::atomic::{Ordering, fence};

use api::device::{DeviceHandle, PinHandle};

use crate::clocks::{CLK_ADC_HZ, Rp2350Clocks, start_clk_adc};
use crate::common::reg::RegAddr;
use crate::common::reset::{clr_reset_reg, cycle_reset, wait_for_reset_done};
use crate::dma::{
    self, CTRL_AHB_ERROR, CTRL_CHAIN_TO_SHIFT, CTRL_DATA_SIZE_SHIFT, CTRL_EN, CTRL_INCR_WRITE,
    CTRL_RING_SEL_WRITE, CTRL_RING_SIZE_SHIFT, CTRL_TREQ_SEL_SHIFT, DATA_SIZE_HALFWORD, DREQ_ADC,
    MODE_ENDLESS, Rp2350DmaChannel, TRANS_COUNT_MODE_SHIFT,
};
use crate::gpio::{IoBank, PadsBank};

/// Lowest sample rate [`Rp2350Adc::stream_from_handle`] accepts, in hertz:
/// a pacing period of 48 000 `clk_adc` cycles, well inside the 16-bit
/// `DIV.INT` (Table 1124, p1072).
pub const MIN_RATE_HZ: u32 = 1_000;

/// Highest sample rate [`Rp2350Adc::stream_from_handle`] accepts, in hertz:
/// a pacing period of 100 `clk_adc` cycles, so every conversion (96 cycles,
/// §12.4.3, p1068) has finished before the next is due. "The ADC ignores
/// this if a conversion is currently in progress, so generally n will be ≥
/// 96" (§12.4.3.2, p1069).
pub const MAX_RATE_HZ: u32 = 480_000;

/// Samples in an [`AdcRing`].
pub const RING_SAMPLES: usize = 8192;

/// `log2` of an [`AdcRing`]'s size in bytes, the DMA's `RING_SIZE`.
const RING_SIZE_LOG2: u32 = 14;

/// How far ahead of the channel [`Rp2350AdcStream::read`] looks for the
/// sentinel, and how many slots an overrun's clearing leaves alone, in
/// samples: 16 ms at 32 kHz, 1.07 ms at [`MAX_RATE_HZ`]. The first is
/// against the few instructions between two reads, the second against
/// clearing the rest of the ring, about 8000 stores (tens of µs at
/// 150 MHz).
pub const GUARD_SAMPLES: usize = 512;

/// Bit 15 of a sample: the conversion did not converge, and "conversion
/// errors produce undefined results, you should always discard samples
/// that contain conversion errors" (§12.4.3.4, p1070; `FIFO.ERR`, Table
/// 1123, p1072).
pub const SAMPLE_ERROR: u16 = 1 << 15;

/// Bits 11:0 of a sample: the conversion result, 0 for 0 V to 4095 for
/// `ADC_AVDD` (`FIFO.VAL`, Table 1123, p1072; Table 1437, p1338).
pub const SAMPLE_VALUE: u16 = 0x0fff;

/// What a ring slot holds when no sample is waiting in it. The converter
/// never writes it: bits 14:12 of a FIFO entry are reserved (Table 1123,
/// p1072).
const SENTINEL: u16 = 0xffff;

const _: () = assert!(RING_SAMPLES * 2 == 1 << RING_SIZE_LOG2);
// "Ring sizes between 2 and 32768 bytes are possible" (Table 1150, p1126).
const _: () = assert!(RING_SIZE_LOG2 >= 1 && RING_SIZE_LOG2 <= 15);
const _: () = assert!(GUARD_SAMPLES < RING_SAMPLES / 2);
const _: () = assert!(SENTINEL & !(SAMPLE_ERROR | SAMPLE_VALUE) != 0);

// --- Register layouts ----------------------------------------------------

/// The `ADC` block, base `0x400a_0000` (Table 1119, p1071).
#[repr(C)]
struct Adc {
    /// `CS` (Table 1120, p1072): `EN` 0, `START_MANY` 3, `READY` 8 (read
    /// only), `AINSEL` 15:12, `RROBIN` 24:16. Reset 0.
    cs: u32,
    /// `RESULT` (Table 1121, p1072): the most recent conversion. Not used.
    _result: u32,
    /// `FCS` (Table 1122, p1072): `EN` 0, `SHIFT` 1, `ERR` 2, `DREQ_EN` 3,
    /// `EMPTY` 8, `UNDER` 10 and `OVER` 11 (write 1 to clear), `THRESH`
    /// 27:24. Reset 0.
    fcs: u32,
    /// `FIFO` (Table 1123, p1072): reading pops the oldest result. A 16-bit
    /// read, as the DMA makes here, is a 32-bit read to the register and
    /// returns bits 15:0 (§2.1.5, p28).
    fifo: u32,
    /// `DIV` (Table 1124, p1072): `INT` 23:8, `FRAC` 7:0. "The divider is
    /// reset when either of these fields are written."
    div: u32,
}

const _: () = assert!(core::mem::offset_of!(Adc, fcs) == 0x08);
const _: () = assert!(core::mem::offset_of!(Adc, fifo) == 0x0c);
const _: () = assert!(core::mem::offset_of!(Adc, div) == 0x10);

/// The registers.
const fn adc() -> *mut Adc {
    RegAddr::ADC as usize as *mut Adc
}

/// `CS.EN`, bit 0: "Power on ADC and enable its clock."
const CS_EN: u32 = 1 << 0;
/// `CS.START_MANY`, bit 3: "Continuously perform conversions whilst this bit
/// is 1."
const CS_START_MANY: u32 = 1 << 3;
/// `CS.READY`, bit 8: "1 if the ADC is ready to start a new conversion.
/// Implies any previous conversion has completed."
const CS_READY: u32 = 1 << 8;
/// `CS.AINSEL` starts at bit 12.
const CS_AINSEL_SHIFT: u32 = 12;

/// `FCS.EN`, bit 0: "write result to the FIFO after each conversion."
const FCS_EN: u32 = 1 << 0;
/// `FCS.ERR`, bit 2: the conversion error flag goes into bit 15 of each
/// FIFO entry.
const FCS_ERR: u32 = 1 << 2;
/// `FCS.DREQ_EN`, bit 3: "assert DMA requests when FIFO contains data."
const FCS_DREQ_EN: u32 = 1 << 3;
/// `FCS.EMPTY`, bit 8 (read only).
const FCS_EMPTY: u32 = 1 << 8;
/// `FCS.UNDER`, bit 10, write 1 to clear.
const FCS_UNDER: u32 = 1 << 10;
/// `FCS.OVER`, bit 11, write 1 to clear: "1 if the FIFO has been
/// overflowed."
const FCS_OVER: u32 = 1 << 11;
/// `FCS.THRESH` starts at bit 24: DREQ while the FIFO holds at least this
/// many results. §12.4.3.5 (p1070) asks for 1 with the DMA.
const FCS_THRESH_SHIFT: u32 = 24;

/// `FCS` while streaming: FIFO on with error flags, DREQ on at one result,
/// 12-bit results (no `SHIFT`), as §12.4.3.5 (p1070) lists for a 16-bit
/// DMA.
const FCS_STREAM: u32 = FCS_EN | FCS_ERR | FCS_DREQ_EN | (1 << FCS_THRESH_SHIFT);

/// `GPIOn_CTRL` at its reset value (Table 650, p608–609): `FUNCSEL` 0x1f,
/// the null function, whose output buffer is high-impedance (§9.3, p586).
const GPIO_CTRL_NULL: u32 = 0x1f;
/// Pad `ISO`, bit 8, reset 1 (Table 852, p785).
const PAD_ISO: u32 = 1 << 8;
/// Pad `OD`, bit 7: output disable (Table 852, p785).
const PAD_OD: u32 = 1 << 7;
/// Pad `IE`, bit 6: input enable (Table 852, p785).
const PAD_IE: u32 = 1 << 6;
/// Pad `PUE`, bit 3: pull-up enable (Table 852, p785).
const PAD_PUE: u32 = 1 << 3;
/// Pad `PDE`, bit 2, reset 1: pull-down enable (Table 852, p785).
const PAD_PDE: u32 = 1 << 2;

/// `ADC` — bit 0 of `RESETS.RESET` (Table 534, p504).
const RESET_ADC: u32 = 1 << 0;
/// `IO_BANK0` (bit 6) and `PADS_BANK0` (bit 9) of `RESETS.RESET` (Table
/// 534, p504).
const RESET_IO_PADS: u32 = (1 << 6) | (1 << 9);

/// How many times [`Rp2350Adc::new`] reads `CS` for `READY` before giving
/// up. Each read of an APB register takes at least three `clk_sys` cycles
/// (§2.1, p26 as summarised in [`RegAddr`]), so this is over 1 ms, against
/// "a few clock cycles" of `clk_adc` (§12.4.3, p1068).
const READY_POLLS: u32 = 100_000;

// --- Pure arithmetic -----------------------------------------------------

/// The pacing divider for one sample rate, as computed by
/// [`for_rate`](Self::for_rate).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateConfig {
    /// `DIV.INT`.
    pub div_int: u16,
    /// `DIV.FRAC`, in 256ths.
    pub div_frac: u8,
}

impl RateConfig {
    /// The divider for `hz` samples per second from a `clk_hz` `clk_adc`:
    /// the period 1 + `INT` + `FRAC` / 256 cycles (Table 1124, p1072) set to
    /// the nearest 256th of a cycle to `clk_hz` / `hz`. `None` outside
    /// [`MIN_RATE_HZ`]–[`MAX_RATE_HZ`].
    ///
    /// The fractional divider is "first order delta sigma" (§12.4.3.2,
    /// p1069): the gap between conversions is a whole number of cycles,
    /// `INT` + 1 or `INT` + 2, and averages the fractional period. At
    /// 32 kHz from 48 MHz there is no fraction: exactly 1500 cycles.
    pub const fn for_rate(clk_hz: u32, hz: u32) -> Option<Self> {
        if hz < MIN_RATE_HZ || hz > MAX_RATE_HZ {
            return None;
        }
        let period_256ths = (256 * clk_hz as u64 + hz as u64 / 2) / hz as u64;
        let div_int = period_256ths / 256 - 1;
        if div_int < 96 || div_int > u16::MAX as u64 {
            return None;
        }
        Some(Self {
            div_int: div_int as u16,
            div_frac: (period_256ths % 256) as u8,
        })
    }

    /// One sampling period in 256ths of a `clk_adc` cycle.
    pub const fn period_256ths(&self) -> u32 {
        ((self.div_int as u32 + 1) << 8) | self.div_frac as u32
    }

    /// The average sample rate produced from a `clk_hz` `clk_adc`, rounded
    /// to the nearest hertz.
    pub const fn rate_hz(&self, clk_hz: u32) -> u32 {
        let p = self.period_256ths() as u64;
        ((256 * clk_hz as u64 + p / 2) / p) as u32
    }

    /// `DIV` register value.
    const fn div_register(&self) -> u32 {
        ((self.div_int as u32) << 8) | self.div_frac as u32
    }
}

// The period falls as the rate rises, so the two ends of the accepted range
// bound every rate in it: both have a divider, with at least 96 + 1 cycles
// per conversion and INT within 16 bits, and nothing outside does.
const _: () = match RateConfig::for_rate(CLK_ADC_HZ, MAX_RATE_HZ) {
    Some(c) => assert!(c.div_int == 99 && c.div_frac == 0),
    None => panic!(),
};
const _: () = match RateConfig::for_rate(CLK_ADC_HZ, MIN_RATE_HZ) {
    Some(c) => assert!(c.div_int == 47_999 && c.div_frac == 0),
    None => panic!(),
};
const _: () = assert!(RateConfig::for_rate(CLK_ADC_HZ, MIN_RATE_HZ - 1).is_none());
const _: () = assert!(RateConfig::for_rate(CLK_ADC_HZ, MAX_RATE_HZ + 1).is_none());
// The worked example in the module documentation: 32 kHz is exactly 1500
// cycles, INT 1499, and reports 32 000 Hz.
const _: () = match RateConfig::for_rate(CLK_ADC_HZ, 32_000) {
    Some(c) => assert!(c.div_int == 1499 && c.div_frac == 0 && c.rate_hz(CLK_ADC_HZ) == 32_000),
    None => panic!(),
};
// A rate with a fraction: 44.1 kHz is 1088.435 cycles, INT 1087, FRAC 111
// (1088 + 111/256 = 1088.434), and reports 44 100 Hz.
const _: () = match RateConfig::for_rate(CLK_ADC_HZ, 44_100) {
    Some(c) => assert!(c.div_int == 1087 && c.div_frac == 111 && c.rate_hz(CLK_ADC_HZ) == 44_100),
    None => panic!(),
};

/// The number of samples from ring index `from` up to, not including, ring
/// index `to`, going forward round the ring.
const fn ring_distance(from: usize, to: usize) -> usize {
    (to + RING_SAMPLES - from) % RING_SAMPLES
}

const _: () = assert!(ring_distance(10, 10) == 0);
const _: () = assert!(ring_distance(10, 74) == 64);
const _: () = assert!(ring_distance(RING_SAMPLES - 1, 0) == 1);
const _: () = assert!(ring_distance(1, 0) == RING_SAMPLES - 1);

// --- Driver --------------------------------------------------------------

/// Why the ADC could not start, or what [`Rp2350AdcStream::read`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdcError {
    /// [`Rp2350Adc::new`]: `CS.READY` did not rise after `CS.EN` was set.
    /// The ADC is left powered.
    NotReady,
    /// [`Rp2350Adc::stream_from_handle`]: the sample rate is outside the
    /// accepted range. Nothing was configured.
    RateOutOfRange {
        /// The rate that was requested, in hertz.
        hz: u32,
        /// [`MIN_RATE_HZ`].
        min: u32,
        /// [`MAX_RATE_HZ`].
        max: u32,
    },
    /// [`Rp2350AdcStream::read`]: reading fell [`RING_SAMPLES`] −
    /// [`GUARD_SAMPLES`] samples or more behind, so samples were
    /// overwritten before they were read. Everything in the ring is
    /// discarded, and the next `read` returns samples from now on.
    Overrun,
    /// [`Rp2350AdcStream::read`]: the ADC's FIFO filled up, so conversions
    /// were lost before the DMA took them (`FCS.OVER`, Table 1122, p1072).
    /// The flag is cleared; the samples already in the ring are still
    /// good, and the next `read` returns them.
    FifoOverflow,
    /// [`Rp2350AdcStream::read`]: the DMA channel received a bus error and
    /// halted (`CTRL.AHB_ERROR`, Table 1150, p1125; §12.6.7.1, p1103). No
    /// more samples arrive; every later `read` returns this.
    DmaError,
}

/// The ADC, out of reset and powered, with `clk_adc` running: the factory
/// for the one [`Rp2350AdcStream`].
///
/// `new` consumes the board's [`DeviceHandle<Rp2350Adc>`] and
/// [`stream_from_handle`](Self::stream_from_handle) consumes this, so there
/// is at most one stream per boot, and so only one DMA channel is ever
/// paced by `DREQ_ADC` ("Multiple channels must not be connected to the
/// same DREQ", §12.6.4.2, p1100).
pub struct Rp2350Adc {
    _private: (),
}

impl Rp2350Adc {
    /// Start `clk_adc` at 48 MHz from `PLL_USB`, put the ADC through a full
    /// reset, power it up and wait for `CS.READY`, and make sure the GPIO
    /// blocks are out of reset. See
    /// [Bring-up requirements](self#bring-up-requirements).
    ///
    /// `&Rp2350Clocks` proves `PLL_USB` runs at 48 MHz. Waits a few
    /// `clk_adc` cycles for the analogue power-up; [`AdcError::NotReady`]
    /// if `READY` has not risen after [`READY_POLLS`] reads of `CS`.
    pub fn new(_handle: DeviceHandle<Rp2350Adc>, _clocks: &Rp2350Clocks) -> Result<Self, AdcError> {
        let adc = adc();
        unsafe {
            clr_reset_reg(!RESET_IO_PADS);
            wait_for_reset_done(RESET_IO_PADS);
            start_clk_adc();
            cycle_reset(RESET_ADC);
            let cs = &raw mut (*adc).cs;
            cs.write_volatile(CS_EN);
            for _ in 0..READY_POLLS {
                if cs.read_volatile() & CS_READY != 0 {
                    return Ok(Self { _private: () });
                }
            }
        }
        Err(AdcError::NotReady)
    }

    /// Sample pin `N` at `hz` samples per second, endlessly, into `ring`
    /// through DMA channel `C`. Read the samples with
    /// [`Rp2350AdcStream::read`] at least every 240 ms at 32 kHz (see
    /// [The ring](self#the-ring)).
    ///
    /// In order: the pin's pad (see [The pin](self#the-pin)); the ADC's
    /// input, pacing divider and FIFO, with any stale results drained; every
    /// ring slot set to the sentinel; the channel started; and last the
    /// converter, "start the DMA channel first, then the ADC conversion via
    /// CS.START_MANY" (§12.4.3.5, p1070).
    ///
    /// `N` must be 26–29, an ADC input on this package (Table 1117, p1067):
    /// any other pin is a build error. The pin handle, the channel, the ring
    /// and this ADC are all consumed, even by an error; the only error,
    /// [`AdcError::RateOutOfRange`], is found before anything is written.
    pub fn stream_from_handle<const N: usize, const C: usize>(
        self,
        _pin: PinHandle<N>,
        _channel: Rp2350DmaChannel<C>,
        ring: &'static mut AdcRing,
        hz: u32,
    ) -> Result<Rp2350AdcStream, AdcError> {
        let () = AdcPin::<N>::VALID;
        let () = Rp2350DmaChannel::<C>::EXISTS;
        let Some(rate) = RateConfig::for_rate(CLK_ADC_HZ, hz) else {
            return Err(AdcError::RateOutOfRange {
                hz,
                min: MIN_RATE_HZ,
                max: MAX_RATE_HZ,
            });
        };
        let ring = ring.samples.as_mut_ptr();
        unsafe {
            connect_pin(N);
            let adc = adc();
            // Free-running is off after the reset in `new`; select the input,
            // no round robin.
            (&raw mut (*adc).cs).write_volatile(CS_EN | (((N - 26) as u32) << CS_AINSEL_SHIFT));
            (&raw mut (*adc).div).write_volatile(rate.div_register());
            let fcs = &raw mut (*adc).fcs;
            fcs.write_volatile(FCS_STREAM | FCS_OVER | FCS_UNDER);
            drain_fifo();
            for i in 0..RING_SAMPLES {
                ring.add(i).write_volatile(SENTINEL);
            }
            start_channel(C, ring);
            let cs = &raw mut (*adc).cs;
            cs.write_volatile(cs.read_volatile() | CS_START_MANY);
        }
        Ok(Rp2350AdcStream {
            ring,
            read: 0,
            channel: C,
            rate,
            halted: false,
        })
    }
}

/// The ring an [`Rp2350AdcStream`] fills: [`RING_SAMPLES`] samples, 16 kB,
/// aligned to its own size as the DMA's address wrap requires (Table 1150,
/// p1126).
///
/// Make one `static mut` with [`AdcRing::new`] (all zeros, so it lives in
/// `.bss` and costs no flash) and pass the stream the one `&'static mut`
/// ever made from it: the DMA writes it for the rest of the boot.
#[repr(C, align(16384))]
pub struct AdcRing {
    samples: [u16; RING_SAMPLES],
}

const _: () = assert!(core::mem::size_of::<AdcRing>() == 1 << RING_SIZE_LOG2);
const _: () = assert!(core::mem::align_of::<AdcRing>() == 1 << RING_SIZE_LOG2);

impl AdcRing {
    /// A zeroed ring. The stream fills it with the sentinel before it
    /// starts.
    pub const fn new() -> Self {
        Self {
            samples: [0; RING_SAMPLES],
        }
    }
}

impl Default for AdcRing {
    fn default() -> Self {
        Self::new()
    }
}

/// One pin, sampled endlessly at a fixed rate into an [`AdcRing`]. Take the
/// samples with [`read`](Self::read).
///
/// Dropping it stops the converter and the channel; see the [`Drop`]
/// implementation.
pub struct Rp2350AdcStream {
    /// The ring's first slot. The DMA writes the ring behind the compiler's
    /// back, so every access is volatile through this pointer and no
    /// reference to the ring is kept.
    ring: *mut u16,
    /// The next slot to read.
    read: usize,
    /// The DMA channel.
    channel: usize,
    /// The pacing divider.
    rate: RateConfig,
    /// The channel halted on a bus error.
    halted: bool,
}

/// Pin `N`, as an ADC input.
struct AdcPin<const N: usize>;

impl<const N: usize> AdcPin<N> {
    /// Pin `N` is an ADC input on this package, GPIO26–29 (Table 1117,
    /// p1067). Referenced by [`Rp2350Adc::stream_from_handle`], so it is
    /// evaluated, unlike an unreferenced associated const.
    const VALID: () = assert!(N >= 26 && N <= 29);
}

impl Rp2350AdcStream {
    /// The average sample rate, rounded to the nearest hertz: the rate
    /// asked for, to within one part in 256 × the pacing period in cycles
    /// (exactly 32 000 Hz for 32 kHz).
    pub fn rate_hz(&self) -> u32 {
        self.rate.rate_hz(CLK_ADC_HZ)
    }

    /// Copy the samples that have arrived since the last call, oldest
    /// first, into the front of `out`, as many as fit, and return how many.
    /// Samples that do not fit stay in the ring for the next call. Each
    /// sample is a FIFO entry: the result in [`SAMPLE_VALUE`], and
    /// [`SAMPLE_ERROR`] set if the conversion failed — keep such a sample's
    /// place in time but not its value.
    ///
    /// It never waits. Errors, checked before anything is copied:
    ///
    /// * [`AdcError::DmaError`] — the channel has halted; nothing more will
    ///   arrive.
    /// * [`AdcError::FifoOverflow`] — conversions were lost before the DMA
    ///   took them. Reported once; the ring is untouched.
    /// * [`AdcError::Overrun`] — reading fell [`RING_SAMPLES`] −
    ///   [`GUARD_SAMPLES`] samples behind (240 ms at 32 kHz); the ring is
    ///   cleared and reading starts again from the newest sample.
    pub fn read(&mut self, out: &mut [u16]) -> Result<usize, AdcError> {
        let dma = dma::dma();
        let adc = adc();
        unsafe {
            if self.halted
                || (&raw const (*dma).ch[self.channel].ctrl_trig).read_volatile() & CTRL_AHB_ERROR
                    != 0
            {
                self.halted = true;
                return Err(AdcError::DmaError);
            }
            let fcs = &raw mut (*adc).fcs;
            if fcs.read_volatile() & FCS_OVER != 0 {
                fcs.write_volatile(FCS_STREAM | FCS_OVER);
                return Err(AdcError::FifoOverflow);
            }
            let written = self.write_index();
            let ahead = self.ring.add((written + GUARD_SAMPLES) % RING_SAMPLES);
            if ahead.read_volatile() != SENTINEL {
                self.resync();
                return Err(AdcError::Overrun);
            }
            let n = ring_distance(self.read, written).min(out.len());
            for sample in out.iter_mut().take(n) {
                let slot = self.ring.add(self.read);
                *sample = slot.read_volatile();
                slot.write_volatile(SENTINEL);
                self.read = (self.read + 1) % RING_SAMPLES;
            }
            Ok(n)
        }
    }

    /// The ring index of the next slot the channel writes. Every slot
    /// before it holds a sample (see [The ring](self#the-ring)); the fence
    /// keeps the processor from reading any of them before this read of
    /// `WRITE_ADDR`.
    fn write_index(&self) -> usize {
        let dma = dma::dma();
        let addr = unsafe { (&raw const (*dma).ch[self.channel].write_addr).read_volatile() };
        fence(Ordering::Acquire);
        (addr as usize - self.ring as usize) / 2 % RING_SAMPLES
    }

    /// After an overrun: every slot except the [`GUARD_SAMPLES`] from the
    /// channel's position back to the sentinel, and reading restarted from
    /// that position (see [The ring](self#the-ring)). The channel writes on
    /// throughout; if it has left that window by the end, it may have
    /// written a slot after it was cleared, so this starts again.
    fn resync(&mut self) {
        loop {
            let from = self.write_index();
            for i in GUARD_SAMPLES..RING_SAMPLES {
                let slot = (from + i) % RING_SAMPLES;
                unsafe { self.ring.add(slot).write_volatile(SENTINEL) };
            }
            if ring_distance(from, self.write_index()) < GUARD_SAMPLES {
                self.read = from;
                return;
            }
        }
    }
}

/// Stops sampling, as §12.4.3.5 (p1070) describes, and leaves the ADC
/// powered down: `CS.START_MANY` cleared, then `CS.READY` awaited "to make
/// sure the last conversion has finished" (at most 96 `clk_adc` cycles, and
/// not bounded here), then the channel aborted (see `dma::abort`), then
/// "drain any stray samples from the FIFO", then the FIFO, its DREQ and the
/// ADC turned off. The ring is left as it is; the DMA no longer writes it.
impl Drop for Rp2350AdcStream {
    fn drop(&mut self) {
        let adc = adc();
        unsafe {
            let cs = &raw mut (*adc).cs;
            cs.write_volatile(cs.read_volatile() & !CS_START_MANY);
            while cs.read_volatile() & CS_READY == 0 {}
            dma::abort(self.channel);
            drain_fifo();
            (&raw mut (*adc).fcs).write_volatile(FCS_OVER | FCS_UNDER);
            cs.write_volatile(0);
        }
    }
}

/// Program and start channel `ch`: an endless, `DREQ_ADC`-paced copy of
/// halfwords from the ADC's `FIFO` into the ring at `ring`, wrapping on the
/// ring's size. `READ_ADDR`, `WRITE_ADDR` and `TRANS_COUNT` first, then
/// `CTRL_TRIG`, whose write starts the channel (§12.6.3.1, p1097).
///
/// `TRANS_COUNT` is `ENDLESS` with a count of 1: "triggering a channel with
/// a mode of 0xf and a count of 0x0 will result in the channel halting
/// immediately" (§12.6.2.2.1, p1096), and in that mode the count never
/// decrements. `CHAIN_TO` is the channel itself, which disables chaining
/// (Table 1150, p1126).
///
/// # Safety
///
/// `ch` < 16, the caller owns the channel and it is idle (as after
/// [`Rp2350Dma::new`](crate::dma::Rp2350Dma::new)), and `ring` is the first
/// slot of an [`AdcRing`] that nothing else accesses except through
/// volatile reads and writes.
unsafe fn start_channel(ch: usize, ring: *mut u16) {
    let dma = dma::dma();
    unsafe {
        let regs = &raw mut (*dma).ch[ch];
        (&raw mut (*regs).read_addr).write_volatile(&raw const (*adc()).fifo as usize as u32);
        (&raw mut (*regs).write_addr).write_volatile(ring as usize as u32);
        (&raw mut (*regs).trans_count).write_volatile((MODE_ENDLESS << TRANS_COUNT_MODE_SHIFT) | 1);
        (&raw mut (*regs).ctrl_trig).write_volatile(
            CTRL_EN
                | (DATA_SIZE_HALFWORD << CTRL_DATA_SIZE_SHIFT)
                | CTRL_INCR_WRITE
                | (RING_SIZE_LOG2 << CTRL_RING_SIZE_SHIFT)
                | CTRL_RING_SEL_WRITE
                | ((ch as u32) << CTRL_CHAIN_TO_SHIFT)
                | (DREQ_ADC << CTRL_TREQ_SEL_SHIFT),
        );
    }
}

/// Read and discard FIFO entries until `FCS.EMPTY`. The FIFO holds at
/// most eight (§12.4.3.4, p1069), and nothing adds to it while this runs:
/// the converter is stopped.
///
/// # Safety
///
/// The ADC is out of reset with `clk_adc` running, the converter is not
/// free-running, and no DMA channel is reading the FIFO ("The user must not
/// access a FIFO currently being serviced by the DMA", §12.6.4.2, p1100).
unsafe fn drain_fifo() {
    let adc = adc();
    unsafe {
        let fcs = &raw const (*adc).fcs;
        let fifo = &raw const (*adc).fifo;
        while fcs.read_volatile() & FCS_EMPTY == 0 {
            let _ = fifo.read_volatile();
        }
    }
}

/// Give `pin` to the ADC: steps 1–3 of [The pin](self#the-pin), null
/// function, digital input and pulls off, isolation released last.
///
/// # Safety
///
/// `pin` is 26–29 and owned by the caller, and `IO_BANK0` and `PADS_BANK0`
/// are out of reset.
unsafe fn connect_pin(pin: usize) {
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe {
        (&raw mut (*io_addr).gpio[pin].ctrl).write_volatile(GPIO_CTRL_NULL);
        let pad = &raw mut (*pads_addr).pads[pin];
        let current = pad.read_volatile();
        pad.write_volatile((current & !(PAD_IE | PAD_PUE | PAD_PDE)) | PAD_OD);
        let current = pad.read_volatile();
        pad.write_volatile(current & !PAD_ISO);
    }
}
