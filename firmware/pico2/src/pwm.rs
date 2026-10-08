//! Square-wave outputs on the `PWM` block: one GPIO, one fixed frequency,
//! 50 % duty, switched on and off from a polled loop.
//!
//! # Using it
//!
//! ```ignore
//! use api::common::Write;
//! use pico2::pwm::Rp2350Pwm;
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let mut pwm = Rp2350Pwm::new(board.pwm, &clocks);     // PWM reset, every slice off
//! let mut tone = pwm.square_from_handle(board.pins.gpio18, 700).unwrap(); // slice 1, channel A; pin low
//! tone.write(true).unwrap();    // 50 % square wave from the next counter wrap on
//! tone.write(false).unwrap();   // held low from the next counter wrap on; the slice keeps running
//! let hz = tone.frequency_hz(); // 700: the frequency produced, to the nearest hertz
//! ```
//!
//! # The block
//!
//! Twelve identical *slices*. Each has a 16-bit counter, an 8.4 fractional
//! clock divider, a wrap value `TOP` and two output channels, A and B, that
//! share the slice's period but have their own compare value (§12.5.1,
//! p1075). In the default free-running mode the counter counts from 0 up to
//! `TOP` and wraps to 0 (§12.5.2.1, p1077). The counting rate is `clk_sys`
//! divided by the slice's `DIV` (Table 1132, p1087); `clk_sys` reaches the
//! block through the `CLK_SYS_PWM` gate, which is enabled out of reset
//! (`WAKE_EN0` bit 25, reset 1, Table 586, p546) and which [`crate::clocks`]
//! does not touch. An output is high while the counter is below its compare
//! value `CC` and low otherwise (Figure 111, p1077), so:
//!
//! * `CC` = 0 gives a 0 % output, "the output signal is always low"
//!   (§12.5.2.2, p1078). This is the off state of every output here.
//! * With `TOP` + 1 = 2*h* counts per period and `CC` = *h*, the output is
//!   high for counts 0 to *h* − 1 and low for counts *h* to `TOP`: *h* counts
//!   each, an exact 50 % square wave. This is the on state.
//!
//! # Which slice drives which pin
//!
//! Table 1129 (p1076), which the GPIO function table repeats in its F4
//! column (Table 644, p588–589):
//!
//! | GPIO | 0 | 1 | 2 | 3 | … | 14 | 15 | 16 | 17 | 18 | 19 | … | 28 | 29 |
//! |------|---|---|---|---|---|----|----|----|----|----|----|---|----|----|
//! | PWM  | 0A | 0B | 1A | 1B | … | 7A | 7B | 0A | 0B | 1A | 1B | … | 6A | 6B |
//!
//! So for GPIO0–31 the slice is (*gpio* / 2) mod 8, and even pins are
//! channel A, odd pins channel B; GPIO32–47 use slices 8–11 in the same
//! pattern ([`gpio_to_slice`], [`gpio_to_channel`]). GPIO30–47 exist on the
//! QFN-80 package only. Every pin on
//! the Pico 2 therefore has a PWM output, but GPIO*n* and GPIO*n*+16 are the
//! *same* output: "If you select the same PWM output on two GPIO pins, the
//! same signal appears on both" (§12.5.2, p1076). This driver hands each
//! output out once and refuses the second pin
//! ([`PwmError::ChannelInUse`]). (§12.5.2 cites the function table as
//! "Table 645"; in this revision of the datasheet that number belongs to the
//! function *descriptions*, and the table itself is Table 644.)
//!
//! # Period and divider
//!
//! With phase-correct mode off, one period lasts
//! (`TOP` + 1) × (`DIV_INT` + `DIV_FRAC` / 16) `clk_sys` cycles, and the
//! output frequency is `clk_sys` divided by that (§12.5.2.6, p1081–1082; the
//! two equations are typeset images on p1082). The divider runs "with a
//! maximum speed of one count per cycle" (p1081), i.e. `DIV` ≥ 1, and
//! `DIV_INT` = 0 means 256 (p1082), which this driver never needs.
//! [`SquareConfig::for_frequency`] works in sixteenths of a cycle, where the
//! period is 2*h* × *d* with *d* = 16 × `DIV_INT` + `DIV_FRAC`:
//!
//! 1. *d* = the smallest divider for which the nearest whole number of
//!    counts still fits `TOP` = 2*h* − 1 ≤ `0xffff` — that is, *h* ≤
//!    32 768 — but at least 16 (`DIV` = 1.0).
//! 2. *h* = that nearest whole number of counts.
//!
//! The smallest divider gives the largest *h*, so the period is rounded to
//! the finest step the counter allows: the frequency error is at most
//! 1 / (2*h*) of the request. At 150 MHz that is at most 134 ppm over the
//! accepted range of [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`] (2.7 Hz near
//! 20 kHz, where *d* = 16 and *h* is only 3750) and at most 20 ppm from 300
//! to 3000 Hz (0.06 Hz). Compile-time assertions below check both bounds for
//! every integer frequency in the range.
//!
//! Below about 2.3 kHz the divider usually has a fractional part (700 Hz
//! is `DIV` = 3 + 5/16, `TOP` = 64 689). The fractional divider is a
//! "first-order delta-sigma type" (§12.5.2.4, p1080) that reaches the
//! average rate "by spacing some enable pulses further apart than others"
//! (Figure 118, p1080). The datasheet does not say how far apart; for a
//! first-order modulator it is `DIV_INT` or `DIV_INT` + 1 cycles, so an
//! edge would sit about one `clk_sys` cycle (6.7 ns) from its ideal
//! position and the two halves of a period could differ by about that
//! much. On an audio tone that is far below anything audible.
//!
//! # Double buffering: whole cycles only
//!
//! "Each slice has two copies of the CC and TOP registers": software writes
//! one, and the other, which the output actually uses, "is updated from the
//! first register at the instant the counter wraps" (§12.5.2.3, p1079); in
//! free-running mode that is "once every TOP + 1 cycles" (p1080). So
//! [`Write::write`] on a [`Rp2350PwmSquare`] does not change the output
//! immediately: the new compare value takes effect at the next wrap, up to
//! one period later. That is the point. A tone always starts with a full
//! high half-period at a wrap and always stops after a full low half-period
//! at a wrap — no runt pulses at either end — and both edges of a keyed
//! element are delayed by the same rule, so its length is quantised to whole
//! periods (1.43 ms at 700 Hz) rather than distorted. Writing the same value
//! again, or writing `true` then `false` within one period, is harmless: the
//! value present at the wrap wins.
//!
//! # Safe start
//!
//! The pin must be driven low from the moment it is connected until the
//! first `write(true)`. [`Rp2350Pwm::square_from_handle`] does, in order:
//!
//! 1. For the first pin on a slice: stop the slice and set `CSR` to
//!    free-running with nothing inverted, program `DIV` and `TOP`, write
//!    `CC` = 0 for both channels and `CTR` = 0, then start the slice and wait
//!    for its first wrap (the `INTR` flag, which a wrap sets whether or not
//!    the interrupt is enabled, §12.5.2.7, p1082). After that wrap the
//!    copies the output uses hold `TOP` and `CC` = 0, so the slice's outputs
//!    are low by the datasheet's own rules, without relying on the reset
//!    value of the internal copies, which the datasheet does not give. The
//!    wait is one period, at most 65 536 × `DIV` `clk_sys` cycles: 10 ms at
//!    100 Hz, 1.5 ms at 700 Hz, 0.44 ms above 2.3 kHz. It gives up after
//!    at least two periods with [`PwmError::NoWrap`], stopping the slice
//!    again and leaving the pin unconnected.
//! 2. Pad: clear `OD`, set `IE` (Table 852, p785).
//! 3. `GPIOn_CTRL.FUNCSEL` = 4, the PWM function (Table 644, p588–589;
//!    e.g. `0x04 → PWM_A_1` for GPIO18, Table 686, p639).
//! 4. Clear the pad's `ISO` bit **last**.
//!
//! Steps 2–4 are the GPIO driver's output sequence with `FUNCSEL` = PWM in
//! place of SIO, and follow §9.7 (p594): "Once software has finished setting
//! up the IO muxing for a given pad, and the peripheral which is to be muxed
//! in, the ISO bit should be cleared." Until step 4 the pad holds its
//! latched state — on a pin untouched since power-on, the reset state:
//! output disabled, pulled low (§9.3, p586). After it, the PWM drives the
//! pin, already low.
//!
//! A reset of the processors alone (a debugger's warm reset) leaves
//! `IO_BANK0` and `PADS_BANK0` as they were: pads return to their reset
//! state at first power-up, and then only on a brown-out, `RUN` held low,
//! `CDBGRSTREQ` or a rescue reset (§9.3, p586–587). A pin the last run gave
//! to the PWM then still has `FUNCSEL` = 4 and its isolation released, and
//! its tone keeps playing until [`Rp2350Pwm::new`]. That first disconnects
//! every such pin, setting `FUNCSEL` to the null function (0x1f, Table 650,
//! p609), which "ensures that the output buffer is high-impedance" (§9.3,
//! p586), and only then resets the PWM block. The watchdog reset this
//! crate uses goes through the power-on state machine and resets all
//! three blocks. Free-running mode makes both A and B pins outputs
//! (§12.5.2.5, p1081), and `OEOVER` = 0, which the whole-register `CTRL`
//! write sets, takes the output enable from the selected peripheral (Table
//! 650, p608).
//!
//! # Turning a tone off
//!
//! Always by `CC` = 0 with the slice left running, never by stopping the
//! slice: an output is high while the counter is below `CC` (Figure 111,
//! p1077) and the counter only moves while the slice is enabled
//! (§12.5.2.5, p1080), so a slice stopped in the high half of a period
//! holds its pin high. [`Write::write`]`(false)` and dropping a
//! [`Rp2350PwmSquare`] both store `CC` = 0 for that channel;
//! [`Rp2350Pwm::silence_all`] does it for every channel at once, for a
//! panic or fault handler that holds no handles.
//!
//! # Two pins on one slice
//!
//! Channels A and B of a slice share `DIV` and `TOP`, so a second pin on a
//! slice that is already running must ask for a frequency that produces the
//! same divider and `TOP` — in practice the same frequency — or it gets
//! [`PwmError::SliceFrequencyConflict`] and the running pin is left alone.
//! Their compare values share the 32-bit `CC` register (A in bits 15:0, B in
//! 31:16, Table 1134, p1088). A 16-bit store cannot update one half: the
//! RP2350 treats narrow writes to IO registers as 32-bit and replicates the
//! value across the bus (§2.1.5, p28). Each pin therefore writes only its
//! own half through the atomic set and clear aliases (§2.1.3, p27; `PWM` is
//! not on that section's list of exceptions): a single store, with no
//! read-modify-write that could lose the other channel's update, even from
//! an interrupt handler or the other core.
//!
//! # Bring-up requirements
//!
//! * **Reset.** `PWM` is bit 16 of `RESETS.RESET` (Table 534, p504);
//!   [`Rp2350Pwm::new`] puts it through a full reset cycle, so every slice
//!   starts disabled (`CSR.EN` reset 0, Table 1131, p1087) even after a
//!   debugger warm reset. Routing an output to a pin also needs `IO_BANK0`
//!   and `PADS_BANK0` (bits 6 and 9), which `new` releases from reset if
//!   nothing has yet — without ever asserting their reset, which would
//!   disturb pins the GPIO driver owns.
//! * **`clk_sys` at [`CLK_SYS_HZ`]**, which every frequency here is computed
//!   from; `new` takes `&Rp2350Clocks` as proof.
//! * `ACCESSCTRL` lets Secure code on either core reach `PWM` out of reset:
//!   "Defaults to Secure access from any master" (Table 946, p849–850).
//!
//! No interrupt is enabled (`IRQ0_INTE` and `IRQ1_INTE` stay at their reset
//! value 0, Table 1138, p1089; Table 1141, p1091) and nothing here waits
//! after configuration: [`Write::write`] is one store.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::common::{ErrorType, Write};
use api::device::{DeviceHandle, PinHandle};

use crate::clocks::{CLK_SYS_HZ, Rp2350Clocks};
use crate::common::MAX_GPIO_PIN;
use crate::common::reg::RegAddr;
use crate::common::reset::{clr_reset_reg, cycle_reset, is_reset_done, wait_for_reset_done};
use crate::gpio::{IoBank, PadsBank};

/// Lowest frequency [`Rp2350Pwm::square_from_handle`] accepts, in hertz.
pub const MIN_FREQUENCY_HZ: u32 = 100;

/// Highest frequency [`Rp2350Pwm::square_from_handle`] accepts, in hertz.
pub const MAX_FREQUENCY_HZ: u32 = 20_000;

// --- Register layouts ----------------------------------------------------

/// One slice's five registers (Table 1130, p1084–1086). The `PWM` block is
/// twelve of these, `CH0_*` to `CH11_*`, at a stride of `0x14`.
#[repr(C)]
struct Slice {
    /// `CHn_CSR` (Table 1131, p1087), reset 0: bit 0 `EN`, bit 1
    /// `PH_CORRECT`, bit 2 `A_INV`, bit 3 `B_INV`, bits 5:4 `DIVMODE`
    /// (0 = free-running), bits 6/7 `PH_RET`/`PH_ADV`. Every field is
    /// wanted at 0 except `EN`.
    csr: u32,
    /// `CHn_DIV` (Table 1132, p1087): `INT` bits 11:4 (reset 1), `FRAC`
    /// bits 3:0 (reset 0). "Counting rate is system clock frequency
    /// divided by this number."
    div: u32,
    /// `CHn_CTR` (Table 1133, p1088): the counter itself, bits 15:0,
    /// read/write, reset 0.
    ctr: u32,
    /// `CHn_CC` (Table 1134, p1088): compare value of channel B in bits
    /// 31:16 and of channel A in bits 15:0, reset 0. Double-buffered; see
    /// the [module documentation](self#double-buffering-whole-cycles-only).
    cc: u32,
    /// `CHn_TOP` (Table 1135, p1088): wrap value, bits 15:0, reset
    /// `0xffff`. Double-buffered like `CC`.
    top: u32,
}

/// The `PWM` block, base `0x400a_8000` (§12.5.3, p1084). Only the
/// registers this driver uses are named.
#[repr(C)]
struct Pwm {
    /// `CH0_*`–`CH11_*`, `0x000`–`0x0ef`. Index by slice number.
    slice: [Slice; SLICES],
    /// `0x0f0` `EN` (Table 1136, p1088): an alias of every slice's
    /// `CSR.EN`. Not used; `CSR` is written instead.
    _en: u32,
    /// `0x0f4` `INTR` (Table 1137, p1089): raw interrupt flags, one per
    /// slice, set at every wrap of that slice's counter and cleared by
    /// writing 1 (type `WC`, Appendix A, p1346–1347). "To clear flags, write a
    /// mask back to INTR" (§12.5.2.7, p1082).
    intr: u32,
}

const _: () = assert!(core::mem::size_of::<Slice>() == 0x14);
const _: () = assert!(core::mem::offset_of!(Slice, cc) == 0x0c);
const _: () = assert!(core::mem::offset_of!(Slice, top) == 0x10);
const _: () = assert!(core::mem::offset_of!(Pwm, _en) == 0x0f0);
const _: () = assert!(core::mem::offset_of!(Pwm, intr) == 0x0f4);

/// Number of slices in the register map (§12.5.1, p1075). The Pico 2's
/// GPIO0–29 reach slices 0–7 only; 8–11 are wired to GPIO32–47, which the
/// QFN-60 package does not have.
const SLICES: usize = 12;

/// The normal view of the registers.
const fn pwm() -> *mut Pwm {
    RegAddr::PWM as usize as *mut Pwm
}

/// The atomic **set** alias (`+0x2000`, §2.1.3, p27): a write sets exactly
/// the bits written and leaves the others alone.
const fn pwm_set() -> *mut Pwm {
    (RegAddr::PWM as usize + 0x2000) as *mut Pwm
}

/// The atomic **clear** alias (`+0x3000`, §2.1.3, p27): a write clears
/// exactly the bits written.
const fn pwm_clr() -> *mut Pwm {
    (RegAddr::PWM as usize + 0x3000) as *mut Pwm
}

/// `CHn_CSR.EN`, bit 0 (Table 1131, p1087).
const CSR_EN: u32 = 1 << 0;
/// `CHn_DIV.INT` starts at bit 4 (Table 1132, p1087).
const DIV_INT_SHIFT: u32 = 4;
/// One channel's half of `CHn_CC`, before shifting (Table 1134, p1088).
const CC_HALF_MASK: u32 = 0xffff;
/// `CHn_CC.B` starts at bit 16 (Table 1134, p1088).
const CC_B_SHIFT: u32 = 16;

/// `GPIOn_CTRL.FUNCSEL` value for the PWM function: column F4 of Table
/// 644 (p588–589), the same for every GPIO, e.g. `0x04 → PWM_A_0` in
/// `GPIO0_CTRL` (Table 650, p609).
const FUNCSEL_PWM: u32 = 4;
/// `GPIOn_CTRL.FUNCSEL` mask, bits 4:0 (Table 650, p609).
const FUNCSEL_MASK: u32 = 0x1f;
/// `GPIOn_CTRL` at its reset value (Table 650, p608–609): `FUNCSEL` = 0x1f,
/// the null function, whose output buffer is high-impedance (§9.3, p586),
/// and every override field "normal" (0).
const GPIO_CTRL_NULL: u32 = 0x1f;
/// Pad `ISO`, bit 8, reset 1: "Pad isolation control. Remove this once the
/// pad is configured by software." (Table 852, p785.)
const PAD_ISO: u32 = 1 << 8;
/// Pad `OD`, bit 7: "Output disable. Has priority over output enable from
/// peripherals" (Table 852, p785).
const PAD_OD: u32 = 1 << 7;
/// Pad `IE`, bit 6, reset 0: input enable (Table 852, p785).
const PAD_IE: u32 = 1 << 6;

/// `PWM` — bit 16 of `RESETS.RESET` (Table 534, p504).
const RESET_PWM: u32 = 1 << 16;
/// `IO_BANK0` (bit 6) and `PADS_BANK0` (bit 9) of `RESETS.RESET` (Table
/// 534, p504): the two blocks every pin is routed through.
const RESET_IO_PADS: u32 = (1 << 6) | (1 << 9);

// --- Pure arithmetic -----------------------------------------------------

/// One of a slice's two outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Output A: even GPIO numbers, compare value in `CC` bits 15:0.
    A = 0,
    /// Output B: odd GPIO numbers, compare value in `CC` bits 31:16.
    B = 1,
}

/// The PWM slice whose output appears on `gpio`, per Table 1129 (p1076):
/// GPIO0–15 carry slices 0–7, two pins each; GPIO16–31 repeat them;
/// GPIO32–39 carry slices 8–11 and GPIO40–47 repeat those.
///
/// Meaningful for `gpio` < 48, the width of `IO_BANK0`.
pub const fn gpio_to_slice(gpio: usize) -> usize {
    if gpio < 32 {
        (gpio / 2) % 8
    } else {
        8 + ((gpio - 32) / 2) % 4
    }
}

/// The channel of [`gpio_to_slice`]`(gpio)` that drives `gpio`: A for even
/// pins, B for odd (Table 1129, p1076).
pub const fn gpio_to_channel(gpio: usize) -> Channel {
    if gpio.is_multiple_of(2) {
        Channel::A
    } else {
        Channel::B
    }
}

/// Table 1129 (p1076) as printed, GPIO0–47: slice number and channel
/// letter. Only here to check [`gpio_to_slice`] and [`gpio_to_channel`]
/// against at compile time. Laid out eight pins to a row, so each row is
/// one half of a row of the printed table.
#[rustfmt::skip]
const TABLE_1129: [(usize, char); 48] = [
    (0, 'A'), (0, 'B'), (1, 'A'), (1, 'B'), (2, 'A'), (2, 'B'), (3, 'A'), (3, 'B'),
    (4, 'A'), (4, 'B'), (5, 'A'), (5, 'B'), (6, 'A'), (6, 'B'), (7, 'A'), (7, 'B'),
    (0, 'A'), (0, 'B'), (1, 'A'), (1, 'B'), (2, 'A'), (2, 'B'), (3, 'A'), (3, 'B'),
    (4, 'A'), (4, 'B'), (5, 'A'), (5, 'B'), (6, 'A'), (6, 'B'), (7, 'A'), (7, 'B'),
    (8, 'A'), (8, 'B'), (9, 'A'), (9, 'B'), (10, 'A'), (10, 'B'), (11, 'A'), (11, 'B'),
    (8, 'A'), (8, 'B'), (9, 'A'), (9, 'B'), (10, 'A'), (10, 'B'), (11, 'A'), (11, 'B'),
];

const _: () = {
    let mut gpio = 0;
    while gpio < TABLE_1129.len() {
        let (slice, letter) = TABLE_1129[gpio];
        assert!(gpio_to_slice(gpio) == slice);
        let channel = match gpio_to_channel(gpio) {
            Channel::A => 'A',
            Channel::B => 'B',
        };
        assert!(channel == letter);
        gpio += 1;
    }
};

/// Smallest divider, in sixteenths: `DIV` = 1.0, one count per `clk_sys`
/// cycle, the fastest the counter goes (§12.5.2.6, p1081).
const DIV_MIN: u64 = 16;
/// Largest divider with a non-zero `DIV_INT`, in sixteenths: 255 + 15/16.
/// `DIV_INT` = 0 would mean 256, with `DIV_FRAC` required to be 0
/// (§12.5.2.6, p1082); this driver never needs it.
const DIV_MAX: u64 = 255 * 16 + 15;
/// Largest half-period in counts: `TOP` = 2 × 32 768 − 1 = `0xffff`, the
/// widest the 16-bit `TOP` allows (§12.5.2.1, p1076).
const MAX_HALF_PERIOD: u64 = 32_768;

/// One slice's period settings for a 50 % square wave, as computed by
/// [`for_frequency`](Self::for_frequency). See "Period and divider" in the
/// [module documentation](self#period-and-divider).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SquareConfig {
    /// `DIV_INT`, 1–255.
    pub div_int: u8,
    /// `DIV_FRAC`, 0–15, in sixteenths.
    pub div_frac: u8,
    /// `TOP`. Always odd, so the period `TOP` + 1 is even and splits into
    /// two equal halves.
    pub top: u16,
}

impl SquareConfig {
    /// Divider and `TOP` for a square wave of `hz` hertz from a `clk_hz`
    /// system clock: the smallest divider that fits, then the nearest
    /// whole number of counts. `None` if `hz` is outside
    /// [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`] or, for some other clock,
    /// no divider fits.
    pub const fn for_frequency(clk_hz: u32, hz: u32) -> Option<Self> {
        if hz < MIN_FREQUENCY_HZ || hz > MAX_FREQUENCY_HZ {
            return None;
        }
        let hz = hz as u64;
        // A period of 2h counts at d sixteenths of a cycle per count lasts
        // 2h * d / 16 cycles, so h * d * hz should equal 8 * clk_hz.
        let target = 8 * clk_hz as u64;
        // Smallest d whose nearest h fits: round(target / (hz * d)) <= 32768
        // exactly when target / (hz * d) < 32768.5, i.e. when
        // d > 2 * target / (65537 * hz).
        let mut d = 2 * target / ((2 * MAX_HALF_PERIOD + 1) * hz) + 1;
        if d < DIV_MIN {
            d = DIV_MIN;
        }
        if d > DIV_MAX {
            return None;
        }
        // Nearest h: round(target / (hz * d)), halves rounded up.
        let h = (2 * target + hz * d) / (2 * hz * d);
        if h == 0 || h > MAX_HALF_PERIOD {
            return None;
        }
        Some(Self {
            div_int: (d >> DIV_INT_SHIFT) as u8,
            div_frac: (d & 0xf) as u8,
            top: (2 * h - 1) as u16,
        })
    }

    /// The divider in sixteenths: 16 × `DIV_INT` + `DIV_FRAC`.
    pub const fn divider_sixteenths(&self) -> u32 {
        ((self.div_int as u32) << DIV_INT_SHIFT) | self.div_frac as u32
    }

    /// Counts per period, `TOP` + 1.
    pub const fn period_counts(&self) -> u32 {
        self.top as u32 + 1
    }

    /// The compare value for 50 % duty: half the period. The output is high
    /// for counts below it (Figure 111, p1077).
    pub const fn compare_level(&self) -> u16 {
        (self.period_counts() / 2) as u16
    }

    /// One period in sixteenths of a `clk_sys` cycle: (`TOP` + 1) × `DIV`
    /// × 16 (§12.5.2.6, p1081–1082). At most 65 536 × 4095, so it fits.
    pub const fn period_sixteenths(&self) -> u32 {
        self.period_counts() * self.divider_sixteenths()
    }

    /// The frequency produced from a `clk_hz` system clock, rounded to the
    /// nearest hertz. The exact value is `16 * clk_hz` /
    /// [`period_sixteenths`](Self::period_sixteenths).
    pub const fn frequency_hz(&self, clk_hz: u32) -> u32 {
        let p = self.period_sixteenths() as u64;
        ((32 * clk_hz as u64 + p) / (2 * p)) as u32
    }

    /// `CHn_DIV` register value.
    const fn div_register(&self) -> u32 {
        self.divider_sixteenths()
    }

    /// Whether the produced frequency is within `ppm` parts per million of
    /// `hz`: |16 × clk − hz × period| × 10⁶ ≤ ppm × hz × period, all in
    /// sixteenths of a cycle.
    const fn within_ppm(&self, clk_hz: u32, hz: u32, ppm: u64) -> bool {
        let produced = 16 * clk_hz as u64;
        let wanted = hz as u64 * self.period_sixteenths() as u64;
        produced.abs_diff(wanted) * 1_000_000 <= ppm * wanted
    }
}

/// Frequency error bound over the whole accepted range, in ppm. The worst
/// case is 133.1 ppm, at 19 992 Hz.
const MAX_ERROR_PPM: u64 = 134;
/// Frequency error bound from 300 to 3000 Hz, the audio-tone range, in
/// ppm. The worst case is 19.5 ppm, at 2952 Hz.
const TONE_ERROR_PPM: u64 = 20;

// Every frequency the driver accepts has a configuration at the real
// clk_sys, within the error bounds above. Checked for each of the 19 901
// integer frequencies, so a change to the arithmetic, the range or the
// clock that breaks this fails the build.
const _: () = {
    let mut hz = MIN_FREQUENCY_HZ;
    while hz <= MAX_FREQUENCY_HZ {
        let bound = if hz >= 300 && hz <= 3000 {
            TONE_ERROR_PPM
        } else {
            MAX_ERROR_PPM
        };
        match SquareConfig::for_frequency(CLK_SYS_HZ, hz) {
            Some(config) => {
                assert!(config.div_int >= 1);
                assert!(config.top % 2 == 1);
                assert!(config.within_ppm(CLK_SYS_HZ, hz, bound));
            }
            None => panic!("an accepted frequency has no PWM configuration"),
        }
        hz += 1;
    }
};
// Outside the range there is none.
const _: () = assert!(SquareConfig::for_frequency(CLK_SYS_HZ, MIN_FREQUENCY_HZ - 1).is_none());
const _: () = assert!(SquareConfig::for_frequency(CLK_SYS_HZ, MAX_FREQUENCY_HZ + 1).is_none());
// The worked example in the module documentation: 700 Hz is DIV 3 + 5/16,
// TOP 64 689, and reports 700 Hz.
const _: () = match SquareConfig::for_frequency(CLK_SYS_HZ, 700) {
    Some(c) => assert!(
        c.div_int == 3 && c.div_frac == 5 && c.top == 64_689 && c.frequency_hz(CLK_SYS_HZ) == 700
    ),
    None => panic!(),
};

// --- Driver --------------------------------------------------------------

/// Why [`Rp2350Pwm::square_from_handle`] could not configure a pin. In
/// every case the pin handle has been consumed and the pin left as it was:
/// unconnected and, on a pin untouched since power-on, isolated in its
/// reset state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PwmError {
    /// The requested frequency is outside the accepted range.
    FrequencyOutOfRange {
        /// The frequency that was requested, in hertz.
        hz: u32,
        /// [`MIN_FREQUENCY_HZ`].
        min: u32,
        /// [`MAX_FREQUENCY_HZ`].
        max: u32,
    },
    /// Another pin already has this PWM output. GPIO*n* and GPIO*n*+16
    /// share one (Table 1129, p1076), and two pin objects writing the same
    /// compare value would undo each other's writes.
    ChannelInUse {
        /// The slice.
        slice: usize,
        /// The channel of that slice.
        channel: Channel,
    },
    /// The pin's slice already runs for the other channel at a period
    /// this frequency does not produce. Both channels of a slice share its
    /// `DIV` and `TOP`.
    SliceFrequencyConflict {
        /// The slice.
        slice: usize,
        /// The frequency the slice is producing, to the nearest hertz.
        running_hz: u32,
        /// The frequency that was requested, in hertz.
        requested_hz: u32,
    },
    /// The slice, started for this pin, did not wrap within two periods,
    /// so its outputs cannot be known to be low: for example, `clk_sys`
    /// does not reach the PWM block (`CLK_SYS_PWM`, `WAKE_EN0` bit 25,
    /// Table 586, p546). The slice is stopped again and the pin was never
    /// connected.
    NoWrap {
        /// The slice.
        slice: usize,
    },
}

/// The PWM block: reset bring-up, and the factory for square-wave pins.
///
/// Not zero-sized, unlike [`Rp2350Gpio`](crate::gpio::gpio::Rp2350Gpio): it
/// records which outputs it has handed out and the period of each slice it
/// has started, which the hardware cannot tell it (a `CC` half of 0 looks
/// the same whether or not a pin owns it). `new` consumes the board's
/// [`DeviceHandle<Rp2350Pwm>`], so at most one exists per boot and this
/// record is complete.
pub struct Rp2350Pwm {
    /// Bit 2 × slice + channel is set once that output has a pin.
    claimed: u32,
    /// The period each slice runs at, `None` while it is stopped. A slice
    /// is started by its first pin and never stopped.
    running: [Option<SquareConfig>; SLICES],
}

const _: () = assert!(2 * SLICES <= u32::BITS as usize);

impl Rp2350Pwm {
    /// Reset the PWM block, leaving every slice stopped, and make sure the
    /// GPIO blocks a PWM output is routed through are out of reset.
    ///
    /// `IO_BANK0` and `PADS_BANK0` are only released, never reset: if
    /// [`Rp2350Gpio::new`](crate::gpio::gpio::Rp2350Gpio::new) has already
    /// run, this changes nothing, and if it runs later it does the same
    /// release again, harmlessly. (The watchdog does the same with
    /// `SYSCFG`.) Then every bonded-out pin whose `FUNCSEL` is the PWM —
    /// which in this boot only a run before a processor-only reset can have
    /// left (see [Safe start](self#safe-start)) — is set back to the null
    /// function, high-impedance. No other pin is touched. Last, `PWM` goes
    /// through a full reset cycle (Table 534, p504), so all its registers
    /// hold their reset values whatever ran before.
    ///
    /// `&Rp2350Clocks` is required because every divider is computed from
    /// [`CLK_SYS_HZ`].
    pub fn new(_handle: DeviceHandle<Rp2350Pwm>, _clocks: &Rp2350Clocks) -> Self {
        unsafe {
            clr_reset_reg(!RESET_IO_PADS);
            wait_for_reset_done(RESET_IO_PADS);
            disconnect_pwm_pins();
            cycle_reset(RESET_PWM);
        }
        Self {
            claimed: 0,
            running: [None; SLICES],
        }
    }

    /// Connect pin `N` to its PWM output as a square wave of `hz` hertz at
    /// 50 % duty, initially off (driven low). Turn it on and off with
    /// [`Write::write`]; [`Rp2350PwmSquare::frequency_hz`] reports the
    /// frequency actually produced.
    ///
    /// Runs the safe-start sequence in the
    /// [module documentation](self#safe-start). If pin `N` is the first on
    /// its slice this starts the slice and busy-waits for its first wrap,
    /// at most 65 536 × `DIV` `clk_sys` cycles (10 ms at 100 Hz, 1.5 ms at
    /// 700 Hz), so start a watchdog with a longer timeout than that, or
    /// configure the PWM pins before starting it. A second pin on a running
    /// slice does not wait.
    ///
    /// The handle is consumed, as by
    /// [`Gpio::output_from_handle`](api::gpio::Gpio::output_from_handle):
    /// it proves pin `N` exists and that nobody else holds it. Errors, all
    /// but the last detected before any register is written:
    ///
    /// * [`PwmError::FrequencyOutOfRange`] — `hz` outside
    ///   [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`].
    /// * [`PwmError::ChannelInUse`] — the pin 16 above or below already has
    ///   this output.
    /// * [`PwmError::SliceFrequencyConflict`] — the other channel of this
    ///   slice already runs at a different period.
    /// * [`PwmError::NoWrap`] — the slice started for this pin did not wrap
    ///   within two periods; it is stopped again.
    pub fn square_from_handle<const N: usize>(
        &mut self,
        _handle: PinHandle<N>,
        hz: u32,
    ) -> Result<Rp2350PwmSquare<N>, PwmError> {
        // Evaluated for every N this is called with, so a pin number the
        // package does not have is a build error here.
        let () = Rp2350PwmSquare::<N>::PIN_EXISTS;
        let Some(config) = SquareConfig::for_frequency(CLK_SYS_HZ, hz) else {
            return Err(PwmError::FrequencyOutOfRange {
                hz,
                min: MIN_FREQUENCY_HZ,
                max: MAX_FREQUENCY_HZ,
            });
        };
        let slice = gpio_to_slice(N);
        let channel = gpio_to_channel(N);
        let claim = 1 << (2 * slice + channel as usize);
        if self.claimed & claim != 0 {
            return Err(PwmError::ChannelInUse { slice, channel });
        }
        match self.running[slice] {
            None => {
                if !unsafe { start_slice(slice, &config) } {
                    return Err(PwmError::NoWrap { slice });
                }
            }
            Some(running) if running == config => {}
            Some(running) => {
                return Err(PwmError::SliceFrequencyConflict {
                    slice,
                    running_hz: running.frequency_hz(CLK_SYS_HZ),
                    requested_hz: hz,
                });
            }
        }
        self.running[slice] = Some(config);
        self.claimed |= claim;
        unsafe { connect_pin(N) };
        Ok(Rp2350PwmSquare { config })
    }
}

impl Rp2350Pwm {
    /// Turn off every PWM output: `CC` = 0 for both channels of every
    /// slice, one store per slice through the atomic clear alias, every
    /// slice left running. Each output goes low at its slice's next wrap,
    /// within one period (10 ms at [`MIN_FREQUENCY_HZ`]). It never stops a
    /// slice, which could hold a pin high (see
    /// [Turning a tone off](self#turning-a-tone-off)).
    ///
    /// For a panic or fault handler, which holds none of the
    /// [`Rp2350PwmSquare`]s, and in which `Drop` does not run. In such a
    /// handler, open any key or PTT line first and call this second, so the
    /// radio is unkeyed before anything else runs.
    ///
    /// It can be called at any point of a boot. If the PWM block is not out
    /// of reset (`RESET_DONE` bit 16 clear, Table 536, p506), as before
    /// [`new`](Self::new), it writes nothing: the block then holds its reset
    /// values, `CC` = 0 (Table 1134, p1088), and the datasheet does not say
    /// what a write to a block held in reset does.
    ///
    /// # Safety
    ///
    /// It overrides the owners of the outputs: a tone its owner turned on
    /// is off, and the owner is not told. Call it only where no owner runs
    /// again, such as a panic handler, a fault handler or just before a
    /// reset.
    pub unsafe fn silence_all() {
        if !unsafe { is_reset_done(RESET_PWM) } {
            return;
        }
        for slice in 0..SLICES {
            unsafe { (&raw mut (*pwm_clr()).slice[slice].cc).write_volatile(u32::MAX) };
        }
    }
}

/// [`Rp2350Pwm::square_from_handle`]'s error type.
impl ErrorType for Rp2350Pwm {
    type Error = PwmError;
}

/// A pin driven by its PWM output as a fixed-frequency, 50 % square wave
/// that can be switched on and off. Drive it with [`Write::write`].
///
/// Ownership works as for [`Rp2350GpioOut`](crate::gpio::gpio::Rp2350GpioOut):
/// construction consumed the pin's [`PinHandle`], so in safe code at most
/// one of these exists per pin, and [`Write::write`] takes `&mut self`, and
/// nothing gives the pin back. Unlike a GPIO output, dropping this value
/// does change the hardware: it turns the tone off, as `write(false)`, so
/// an early return cannot leave a tone playing. The pin stays connected to
/// its PWM output, held low, with the slice running.
///
/// Holds only the slice's [`SquareConfig`]; the slice and channel follow
/// from `N`.
pub struct Rp2350PwmSquare<const N: usize> {
    config: SquareConfig,
}

impl<const N: usize> Rp2350PwmSquare<N> {
    /// Pin `N` is bonded out on this package. Referenced by
    /// [`Rp2350Pwm::square_from_handle`], so unlike an unreferenced
    /// associated const it is actually evaluated.
    const PIN_EXISTS: () = assert!(N < MAX_GPIO_PIN);

    /// The frequency this pin produces when on, rounded to the nearest
    /// hertz. Within 134 ppm of the frequency requested, and within 20 ppm
    /// from 300 to 3000 Hz; see the
    /// [module documentation](self#period-and-divider).
    pub fn frequency_hz(&self) -> u32 {
        self.config.frequency_hz(CLK_SYS_HZ)
    }
}

/// Switching the tone cannot fail: every failure mode was ruled out when
/// the pin was configured.
impl<const N: usize> ErrorType for Rp2350PwmSquare<N> {
    type Error = core::convert::Infallible;
}

impl<const N: usize> Write<bool> for Rp2350PwmSquare<N> {
    /// `true`: 50 % square wave; `false`: held low (`CC` = 0) with the
    /// slice still running. Takes effect at the next counter wrap, up to
    /// one period later, so the output only ever changes between whole
    /// cycles; see the
    /// [module documentation](self#double-buffering-whole-cycles-only).
    ///
    /// One store to this channel's half of `CC`, through an atomic alias,
    /// so the other channel's half is never read or written. `false`
    /// clears all 16 bits. `true` sets the bits of the 50 % level; that is
    /// enough because this half only ever holds 0 or that level (nothing
    /// else writes it once the pin is configured), and it is a single store
    /// so a tone that is already on never passes through 0 — which a clear
    /// followed by a set could, if a wrap fell between the two.
    fn write(&mut self, value: bool) -> Result<(), Self::Error> {
        let slice = gpio_to_slice(N);
        let shift = match gpio_to_channel(N) {
            Channel::A => 0,
            Channel::B => CC_B_SHIFT,
        };
        unsafe {
            if value {
                let level = self.config.compare_level() as u32;
                (&raw mut (*pwm_set()).slice[slice].cc).write_volatile(level << shift);
            } else {
                (&raw mut (*pwm_clr()).slice[slice].cc).write_volatile(CC_HALF_MASK << shift);
            }
        }
        Ok(())
    }
}

/// Turns the tone off: see the [`Rp2350PwmSquare`] documentation.
impl<const N: usize> Drop for Rp2350PwmSquare<N> {
    fn drop(&mut self) {
        let Ok(()) = self.write(false);
    }
}

/// Program a stopped slice for `config` with both outputs at 0 %, start
/// it, and wait for its first wrap: step 1 of the safe-start sequence in
/// the module documentation. `false` if it did not wrap within two
/// periods; the slice is then stopped again.
///
/// # Safety
///
/// `slice` < [`SLICES`], and no pin may own an output of it yet: this
/// rewrites the whole slice, both channels included.
unsafe fn start_slice(slice: usize, config: &SquareConfig) -> bool {
    let pwm = pwm();
    let wrapped = 1 << slice;
    unsafe {
        let regs = &raw mut (*pwm).slice[slice];
        let csr = &raw mut (*regs).csr;
        // Stopped; free-running, not inverted, not phase-correct.
        csr.write_volatile(0);
        (&raw mut (*regs).div).write_volatile(config.div_register());
        (&raw mut (*regs).top).write_volatile(config.top as u32);
        // Both channels at 0 %: always low.
        (&raw mut (*regs).cc).write_volatile(0);
        (&raw mut (*regs).ctr).write_volatile(0);
        // Clear this slice's wrap flag, start counting, and wait for the
        // wrap that loads TOP and CC into the copies the outputs use.
        let intr = &raw mut (*pwm).intr;
        intr.write_volatile(wrapped);
        csr.write_volatile(CSR_EN);
        // Each look at INTR takes at least one clk_sys cycle, so this many
        // looks span at least two periods.
        let looks = 2 * (config.period_sixteenths() / 16 + 1);
        for _ in 0..looks {
            if intr.read_volatile() & wrapped != 0 {
                return true;
            }
        }
        // No pin owns either output of this slice yet, so stopping it
        // cannot hold one high.
        csr.write_volatile(0);
        false
    }
}

/// Set every bonded-out pin whose `FUNCSEL` is the PWM back to the null
/// function, high-impedance, with every override "normal": the register's
/// reset value. Pins with any other function are not touched.
///
/// # Safety
///
/// `IO_BANK0` is out of reset, and no pin has been given to the PWM in this
/// boot: only [`Rp2350Pwm::new`] calls this, once.
unsafe fn disconnect_pwm_pins() {
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    for pin in 0..MAX_GPIO_PIN {
        unsafe {
            let ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
            if ctrl.read_volatile() & FUNCSEL_MASK == FUNCSEL_PWM {
                ctrl.write_volatile(GPIO_CTRL_NULL);
            }
        }
    }
}

/// Route `pin` to its PWM output: steps 2–4 of the safe-start sequence in
/// the module documentation, the same order as the GPIO driver's output
/// path.
///
/// `IE` is set, as the GPIO driver does for an output: §9.3 (p587) asks
/// for `IE` = 1 and `ISO` = 0 "before using the pads for digital I/O".
/// `DRIVE`, `SLEWFAST`, `SCHMITT` and the pulls are left as they are —
/// on a pad nothing else has touched, their reset values: 4 mA, slow slew,
/// Schmitt trigger on, pull-down on (Table 852, p785) — as in the GPIO
/// driver's output path.
///
/// # Safety
///
/// `pin` < [`MAX_GPIO_PIN`], and the caller owns it. Its slice must already
/// be running with this channel at `CC` = 0, so that the pin is low the
/// moment `ISO` is cleared.
unsafe fn connect_pin(pin: usize) {
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe {
        let pad = &raw mut (*pads_addr).pads[pin];
        // Read-modify-write: DRIVE, SCHMITT and the pulls keep their values.
        let mut current_pad = pad.read_volatile();
        // OD must be clear or the pad refuses to drive regardless of the PWM.
        current_pad &= !PAD_OD;
        current_pad |= PAD_IE;
        pad.write_volatile(current_pad);
        // FUNCSEL = PWM. Writing the whole register also sets
        // IRQOVER/INOVER/OEOVER/OUTOVER to "normal": output and output
        // enable come from the PWM.
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        io_ctrl.write_volatile(FUNCSEL_PWM);
        // Release the isolation latch last.
        let current_pad = pad.read_volatile();
        pad.write_volatile(current_pad & !PAD_ISO);
    }
}
