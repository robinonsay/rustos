//! Clock tree bring-up: crystal oscillator, both PLLs, and the four clock
//! generators this crate uses.
//!
//! # What the chip runs on before this code
//!
//! Out of reset the RP2350 has no crystal and no PLL running. The power-on
//! state machine starts the ring oscillator (ROSC, roughly 11 MHz and
//! drifting with voltage and temperature), runs `clk_ref` from it, and
//! `clk_sys` from `clk_ref` (§7.4.1 step 6, p496). The bootrom then moves
//! `clk_sys` onto the ROSC through its aux mux and may change dividers. None
//! of that is accurate enough for USB, which needs exactly 48 MHz, or for a
//! microsecond timer. [`Rp2350Clocks::new`] replaces it with:
//!
//! ```text
//!  12 MHz crystal --XOSC--+------------------------------> clk_ref  12 MHz
//!                         |                                  (TICKS divide by 12 -> 1 us)
//!                         +--PLL_SYS 12x125=1500 /5/2------> clk_sys 150 MHz --> clk_peri 150 MHz
//!                         |
//!                         +--PLL_USB 12x100=1200 /5/5------> clk_usb  48 MHz
//! ```
//!
//! The resulting frequencies are the constants below ([`CLK_SYS_HZ`] and
//! friends). They are compile-time facts, not measurements: the code that
//! sets them up is the only code that touches these generators, and every
//! driver that depends on a frequency takes `&Rp2350Clocks` as proof that
//! the setup has run.
//!
//! # Two kinds of multiplexer, two switching rules
//!
//! Every clock generator has an **aux mux**, which glitches if switched
//! while running, and a divider. `clk_ref` and `clk_sys` — the two clocks
//! that may never stop — additionally have a **glitchless mux** in front of
//! the aux mux, whose `SELECTED` register reports which input is live
//! (§8.1.2.2, p517). The rules from that section, each implemented by one
//! function here:
//!
//! * Glitchless switch ([`switch_glitchless`]): write `SRC`, then poll
//!   `SELECTED` until the new source's bit is set.
//! * Aux switch on a generator with a glitchless mux (same function): first
//!   move the glitchless mux *off* the aux input and wait for `SELECTED`,
//!   then change `AUXSRC`, then switch back to aux and wait again.
//! * Aux switch on a generator without one ([`switch_aux_only`]): clear
//!   `ENABLE`, wait until `ENABLED` reads back 0 (two cycles of the old
//!   source), change `AUXSRC`, set `ENABLE`, wait for `ENABLED`.
//!
//! And from the SDK's `clock_configure` listing reproduced in §8.1.5.1
//! (p522): if the new divisor is *larger* than the current one, write it
//! *before* switching source, otherwise after, so the output never runs
//! faster than either the old or the new setting.
//!
//! # Order of operations
//!
//! 1. Disable resus (`CLK_SYS_RESUS_CTRL = 0`, Table 576, p543), so the
//!    momentary absence of `clk_sys` edges while switching cannot trigger a
//!    "resuscitation" back to the ROSC.
//! 2. Start the XOSC and wait for `STATUS.STABLE` ([`start_xosc`]).
//! 3. Move `clk_sys` onto `clk_ref` and `clk_ref` onto the ROSC, both via
//!    their glitchless muxes. After this neither clock depends on a PLL, so
//!    the PLLs can be reset safely. (On a cold boot this is already the
//!    case for `clk_ref`; after a debugger warm reset it may not be.)
//! 4. Reset and program `PLL_SYS` and `PLL_USB`, waiting for lock
//!    ([`start_pll`]).
//! 5. `clk_ref` ← XOSC ÷ 1 (glitchless source 2).
//! 6. `clk_sys` ← aux `PLL_SYS` ÷ 1.
//! 7. `clk_usb` ← `PLL_USB` ÷ 1, `clk_peri` ← `clk_sys` ÷ 1 (aux-only
//!    generators).
//!
//! `clk_adc`, `clk_hstx` and the four `clk_gpout` generators are not
//! touched.
//!
//! # Flash speed
//!
//! Code executes from QSPI flash, whose SCK is `clk_sys` divided by the QMI
//! divisor the bootrom chose while probing the flash. The bootrom only ever
//! picks a divisor of 3 or more (§5.2.7, Table 451, p373), so at 150 MHz
//! the flash clock is at most 50 MHz. This module does not reprogram the
//! QMI.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::device::DeviceHandle;

use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

/// Crystal frequency on the Pico 2 (and the RP2350 reference design, which
/// the bootrom's USB code also assumes): 12 MHz.
pub const XOSC_HZ: u32 = 12_000_000;

/// `clk_ref`: the crystal, undivided. The tick generators divide this down
/// to the 1 µs timebase.
pub const CLK_REF_HZ: u32 = XOSC_HZ;

/// `clk_sys`: the processors, bus fabric and SRAM. 150 MHz is the RP2350's
/// rated maximum at the default core voltage (Table 540, p516).
pub const CLK_SYS_HZ: u32 = PLL_SYS_CONFIG.output_hz();

/// `clk_peri`: UART, SPI and other serial peripherals. Fed from `clk_sys`,
/// undivided.
pub const CLK_PERI_HZ: u32 = CLK_SYS_HZ;

/// `clk_usb`: the USB controller. Must be exactly 48 MHz (Table 540,
/// p516; §12.7.3.1, p1142).
pub const CLK_USB_HZ: u32 = PLL_USB_CONFIG.output_hz();

/// `PLL_SYS` voltage-controlled-oscillator frequency, 1500 MHz.
pub const PLL_SYS_VCO_HZ: u32 = PLL_SYS_CONFIG.vco_hz();

/// `PLL_USB` VCO frequency, 1200 MHz.
pub const PLL_USB_VCO_HZ: u32 = PLL_USB_CONFIG.vco_hz();

/// One PLL's dividers.
///
/// Output = (XOSC ÷ `refdiv`) × `fbdiv` ÷ (`postdiv1` × `postdiv2`)
/// (§8.6.3, p574). The values used are the SDK defaults that the datasheet
/// reproduces for the RP2350 (§8.6.4, p579).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PllConfig {
    /// Reference divider, `CS.REFDIV`.
    pub refdiv: u32,
    /// Feedback divider, `FBDIV_INT`: the VCO runs at this multiple of the
    /// divided reference.
    pub fbdiv: u32,
    /// First post divider, `PRIM.POSTDIV1`.
    pub postdiv1: u32,
    /// Second post divider, `PRIM.POSTDIV2`.
    pub postdiv2: u32,
}

impl PllConfig {
    /// VCO frequency in Hz.
    pub const fn vco_hz(&self) -> u32 {
        XOSC_HZ / self.refdiv * self.fbdiv
    }

    /// Output frequency in Hz.
    pub const fn output_hz(&self) -> u32 {
        self.vco_hz() / (self.postdiv1 * self.postdiv2)
    }

    /// The constraints of §8.6.3 (p574): reference after `refdiv` at least
    /// 5 MHz, VCO 750–1600 MHz, `fbdiv` 16–320, each post divider 1–7.
    ///
    /// The `CS` register description (Table 636, p582) gives a lower VCO
    /// minimum of 400 MHz; the stricter 750 MHz from the overview is used.
    pub const fn is_valid(&self) -> bool {
        let fref = XOSC_HZ / self.refdiv;
        let vco = self.vco_hz() as u64;
        self.refdiv >= 1
            && self.refdiv <= 63
            && fref >= 5_000_000
            && self.fbdiv >= 16
            && self.fbdiv <= 320
            && vco >= 750_000_000
            && vco <= 1_600_000_000
            && self.postdiv1 >= 1
            && self.postdiv1 <= 7
            && self.postdiv2 >= 1
            && self.postdiv2 <= 7
    }
}

/// `PLL_SYS`: 12 MHz × 125 = 1500 MHz VCO, ÷ 5 ÷ 2 = 150 MHz.
pub const PLL_SYS_CONFIG: PllConfig = PllConfig { refdiv: 1, fbdiv: 125, postdiv1: 5, postdiv2: 2 };

/// `PLL_USB`: 12 MHz × 100 = 1200 MHz VCO, ÷ 5 ÷ 5 = 48 MHz.
pub const PLL_USB_CONFIG: PllConfig = PllConfig { refdiv: 1, fbdiv: 100, postdiv1: 5, postdiv2: 5 };

const _: () = assert!(PLL_SYS_CONFIG.is_valid());
const _: () = assert!(PLL_USB_CONFIG.is_valid());
const _: () = assert!(CLK_SYS_HZ == 150_000_000);
const _: () = assert!(CLK_USB_HZ == 48_000_000);
// The USB controller needs clk_sys > 48 MHz (§12.7.3.1, p1142), and
// erratum RP2350-E12 asks for at least 10% faster than clk_usb (Appendix E,
// p1367). 150 MHz is more than three times 48 MHz.
const _: () = assert!(CLK_SYS_HZ as u64 * 10 >= CLK_USB_HZ as u64 * 11);
// The tick generators count whole clk_ref cycles per microsecond.
const _: () = assert!(CLK_REF_HZ.is_multiple_of(1_000_000));

// --- Register layouts ----------------------------------------------------

/// One clock generator's three registers. The `CLOCKS` block is ten of
/// these back to back (Table 542, p527), followed by the frequency counter
/// and resus registers.
#[repr(C)]
struct ClockGenerator {
    /// `CLK_x_CTRL`. Field positions differ per generator; see the
    /// `*_CTRL_*` constants.
    ctrl: u32,
    /// `CLK_x_DIV`. The integer part always starts at bit 16, but its
    /// width differs: 8 bits for `clk_ref` (Table 556, p536), 16 for
    /// `clk_sys` (Table 559, p537), 2 for `clk_peri` (Table 562, p538), 4
    /// for `clk_usb` (Table 568, p540). `1 << 16` is divide-by-1 in all of
    /// them.
    div: u32,
    /// `CLK_x_SELECTED`: one-hot, which glitchless-mux input is live. Only
    /// meaningful on `clk_ref` and `clk_sys`; reads 1 elsewhere. "Whilst
    /// switching is in progress, this register may briefly show all-0s"
    /// (Table 557, p536).
    selected: u32,
}

/// The `CLOCKS` block, base `0x4001_0000` (Table 542, p527).
#[repr(C)]
struct Clocks {
    /// `0x00`–`0x77`: generators in the order `clk_gpout0`–`3`, `clk_ref`,
    /// `clk_sys`, `clk_peri`, `clk_hstx`, `clk_usb`, `clk_adc`. Index with
    /// [`GEN_REF`] etc.
    clk: [ClockGenerator; 10],
    /// `0x78`–`0x83`: `DFTCLK_*_CTRL`, test only.
    _dftclk: [u32; 3],
    /// `0x84` `CLK_SYS_RESUS_CTRL` (Table 576, p543). Bit 8 `ENABLE`; reset
    /// value has it clear, but nothing guarantees the bootrom left it so.
    sys_resus_ctrl: u32,
}

const _: () = assert!(core::mem::offset_of!(Clocks, clk) == 0x00);
const _: () = assert!(core::mem::size_of::<ClockGenerator>() == 0x0c);
const _: () = assert!(core::mem::offset_of!(Clocks, sys_resus_ctrl) == 0x84);

/// Index of `clk_ref` in [`Clocks::clk`] (offset `0x30`).
const GEN_REF: usize = 4;
/// Index of `clk_sys` (offset `0x3c`).
const GEN_SYS: usize = 5;
/// Index of `clk_peri` (offset `0x48`).
const GEN_PERI: usize = 6;
/// Index of `clk_usb` (offset `0x60`).
const GEN_USB: usize = 8;
const _: () = assert!(GEN_REF * 0x0c == 0x30 && GEN_SYS * 0x0c == 0x3c);
const _: () = assert!(GEN_PERI * 0x0c == 0x48 && GEN_USB * 0x0c == 0x60);

/// `CLK_REF_CTRL.SRC`, bits 1:0 (Table 555, p535): 0 ROSC, 1 aux, 2 XOSC,
/// 3 LPOSC.
const REF_CTRL_SRC_MASK: u32 = 0b11;
/// `CLK_REF_CTRL.SRC` value for the crystal.
const REF_SRC_XOSC: u32 = 2;
/// `CLK_REF_CTRL.SRC` value for the ring oscillator.
const REF_SRC_ROSC: u32 = 0;

/// `CLK_SYS_CTRL.SRC`, bit 0 (Table 558, p536): 0 `clk_ref`, 1 aux.
const SYS_CTRL_SRC_MASK: u32 = 0b1;
/// `CLK_SYS_CTRL.SRC` value for `clk_ref`.
const SYS_SRC_CLK_REF: u32 = 0;
/// `CLK_SYS_CTRL.SRC` value for the aux mux.
const SYS_SRC_AUX: u32 = 1;
/// `CLK_SYS_CTRL.AUXSRC`, bits 7:5 (Table 558, p536). Reset value 2, the
/// ROSC.
const SYS_CTRL_AUXSRC_SHIFT: u32 = 5;
/// Width mask of `CLK_SYS_CTRL.AUXSRC` before shifting.
const SYS_CTRL_AUXSRC_MASK: u32 = 0b111;
/// `CLK_SYS_CTRL.AUXSRC` value for `PLL_SYS`.
const SYS_AUXSRC_PLL_SYS: u32 = 0;

/// `CLK_PERI_CTRL` / `CLK_USB_CTRL`: `ENABLE` bit 11, `ENABLED` bit 28
/// (read-only), `AUXSRC` bits 7:5 (Table 561, p537; Table 567, p539).
const AUX_CTRL_ENABLE: u32 = 1 << 11;
/// Read-only "generator is running" status, bit 28.
const AUX_CTRL_ENABLED: u32 = 1 << 28;
/// `AUXSRC` position in the aux-only generators.
const AUX_CTRL_AUXSRC_SHIFT: u32 = 5;
/// `AUXSRC` width mask in the aux-only generators.
const AUX_CTRL_AUXSRC_MASK: u32 = 0b111;
/// `CLK_PERI_CTRL.AUXSRC` value for `clk_sys`.
const PERI_AUXSRC_CLK_SYS: u32 = 0;
/// `CLK_USB_CTRL.AUXSRC` value for `PLL_USB`.
const USB_AUXSRC_PLL_USB: u32 = 0;

/// Divide-by-1 in every `CLK_x_DIV` register used here: integer part 1 at
/// bit 16, fractional part (where present) 0.
const DIV_BY_1: u32 = 1 << 16;

/// The `XOSC` block, base `0x4004_8000` (Table 597, p557).
#[repr(C)]
struct Xosc {
    /// `CTRL` (Table 598, p557). `ENABLE` bits 23:12 takes the 12-bit code
    /// `0xfab` (enable) or `0xd1e` (disable); `FREQ_RANGE` bits 11:0 takes
    /// `0xaa0` for a 1–15 MHz crystal. Any other code is ignored and the
    /// old value kept — protection against stray writes.
    ctrl: u32,
    /// `STATUS` (Table 599, p558). Bit 31 `STABLE`: running and past the
    /// startup delay.
    status: u32,
    /// `DORMANT`: never written here.
    _dormant: u32,
    /// `STARTUP` (Table 601, p559). `DELAY` bits 13:0, in units of 256
    /// crystal periods; `X4` bit 20 multiplies it by 4.
    startup: u32,
}

const _: () = assert!(core::mem::offset_of!(Xosc, ctrl) == 0x00);
const _: () = assert!(core::mem::offset_of!(Xosc, status) == 0x04);
const _: () = assert!(core::mem::offset_of!(Xosc, startup) == 0x0c);

/// `XOSC_CTRL` value: `ENABLE` = `0xfab`, `FREQ_RANGE` = `1_15MHZ` (`0xaa0`).
const XOSC_CTRL_ENABLE_1_15MHZ: u32 = (0xfab << 12) | 0xaa0;
/// `XOSC_STATUS.STABLE`, bit 31.
const XOSC_STATUS_STABLE: u32 = 1 << 31;
/// How long the XOSC startup timer waits, in milliseconds.
///
/// §8.2.4 (p555) says 1 ms suffices for the reference design's 12 MHz
/// crystal. The SDK multiplies that by `PICO_XOSC_STARTUP_DELAY_MULTIPLIER`
/// (default 6) to accommodate slow-starting crystals; the same margin is
/// used here. It costs 6 ms once per boot.
const XOSC_STARTUP_MS: u32 = 6;
/// `STARTUP.DELAY`: crystal cycles in the startup wait ÷ 256, rounded up
/// (§8.2.4, p555). 12 000 cycles/ms × 6 ms ÷ 256 = 281.25 → 282.
const XOSC_STARTUP_DELAY: u32 = (XOSC_HZ / 1000 * XOSC_STARTUP_MS).div_ceil(256);
const _: () = assert!(XOSC_STARTUP_DELAY < (1 << 14));

/// One PLL's registers; `PLL_SYS` and `PLL_USB` share the layout (Table
/// 635, p582).
#[repr(C)]
struct Pll {
    /// `CS` (Table 636, p582). Bit 31 `LOCK` (read-only), bits 5:0
    /// `REFDIV`.
    cs: u32,
    /// `PWR` (Table 637, p582). Power-down bits, all reset to 1 (powered
    /// down): `VCOPD` 5, `POSTDIVPD` 3, `DSMPD` 2, `PD` 0.
    pwr: u32,
    /// `FBDIV_INT` (Table 638, p583), bits 11:0.
    fbdiv_int: u32,
    /// `PRIM` (Table 639, p583). `POSTDIV1` bits 18:16, `POSTDIV2` 14:12.
    prim: u32,
}

const _: () = assert!(core::mem::offset_of!(Pll, cs) == 0x00);
const _: () = assert!(core::mem::offset_of!(Pll, pwr) == 0x04);
const _: () = assert!(core::mem::offset_of!(Pll, fbdiv_int) == 0x08);
const _: () = assert!(core::mem::offset_of!(Pll, prim) == 0x0c);

/// `CS.LOCK`.
const PLL_CS_LOCK: u32 = 1 << 31;
/// `PWR.PD`: main power-down.
const PLL_PWR_PD: u32 = 1 << 0;
/// `PWR.POSTDIVPD`: post-divider power-down.
const PLL_PWR_POSTDIVPD: u32 = 1 << 3;
/// `PWR.VCOPD`: VCO power-down.
const PLL_PWR_VCOPD: u32 = 1 << 5;

/// `PLL_SYS` — bit 14 of `RESETS.RESET` (Table 534, p504).
const RESET_PLL_SYS: u32 = 1 << 14;
/// `PLL_USB` — bit 15 of `RESETS.RESET` (Table 534, p504).
const RESET_PLL_USB: u32 = 1 << 15;

// --- Driver --------------------------------------------------------------

/// Proof that the clock tree has been configured.
///
/// Zero-sized. [`new`](Self::new) consumes the board's
/// `DeviceHandle<Rp2350Clocks>`, so the setup runs exactly once per boot;
/// drivers that need a particular frequency ([`crate::timer`],
/// [`crate::watchdog`], [`crate::usb`]) take `&Rp2350Clocks`, which makes
/// "configure the clocks first" a type error to forget.
pub struct Rp2350Clocks {
    _private: (),
}

impl Rp2350Clocks {
    /// Bring up the XOSC, both PLLs, `clk_ref`, `clk_sys`, `clk_peri` and
    /// `clk_usb` as described in the [module documentation](self).
    ///
    /// Call this first, before any other driver: it changes the frequency
    /// of everything. It busy-waits for the crystal (about 6 ms) and the
    /// two PLL locks. If the crystal never starts it waits forever; at this
    /// point in boot no GPIO has been configured, so every pin is still a
    /// high-impedance input.
    pub fn new(_handle: DeviceHandle<Rp2350Clocks>) -> Self {
        unsafe { init() };
        Self { _private: () }
    }
}

/// The whole sequence. See the module docs for the order and its reasons.
unsafe fn init() {
    let clocks = RegAddr::CLOCKS as usize as *mut Clocks;
    unsafe {
        // 1. No resus while clk_sys is being switched.
        (&raw mut (*clocks).sys_resus_ctrl).write_volatile(0);

        // 2. Crystal.
        start_xosc();

        // 3. Take clk_sys and clk_ref off anything PLL-derived.
        let sys = &raw mut (*clocks).clk[GEN_SYS];
        let reference = &raw mut (*clocks).clk[GEN_REF];
        select_glitchless(sys, SYS_CTRL_SRC_MASK, SYS_SRC_CLK_REF);
        select_glitchless(reference, REF_CTRL_SRC_MASK, REF_SRC_ROSC);

        // 4. PLLs.
        start_pll(RegAddr::PLL_SYS as usize as *mut Pll, RESET_PLL_SYS, &PLL_SYS_CONFIG);
        start_pll(RegAddr::PLL_USB as usize as *mut Pll, RESET_PLL_USB, &PLL_USB_CONFIG);

        // 5. clk_ref <- XOSC / 1. Not an aux source, so no aux change.
        switch_glitchless(reference, REF_CTRL_SRC_MASK, REF_SRC_XOSC, None, DIV_BY_1);

        // 6. clk_sys <- aux PLL_SYS / 1.
        switch_glitchless(
            sys,
            SYS_CTRL_SRC_MASK,
            SYS_SRC_AUX,
            Some((SYS_CTRL_AUXSRC_SHIFT, SYS_CTRL_AUXSRC_MASK, SYS_AUXSRC_PLL_SYS)),
            DIV_BY_1,
        );

        // 7. The aux-only generators.
        switch_aux_only(&raw mut (*clocks).clk[GEN_USB], USB_AUXSRC_PLL_USB, DIV_BY_1);
        switch_aux_only(&raw mut (*clocks).clk[GEN_PERI], PERI_AUXSRC_CLK_SYS, DIV_BY_1);
    }
}

/// Start the crystal oscillator and wait until it is stable.
///
/// §8.2.3–8.2.4 (p555): the startup delay must be programmed *before*
/// enabling, because the XOSC's own timer — not software — decides when
/// `STATUS.STABLE` rises. Writing `CTRL` with the enable code while the
/// XOSC already runs (a warm restart) rewrites the same values and is
/// harmless.
unsafe fn start_xosc() {
    let xosc = RegAddr::XOSC as usize as *mut Xosc;
    unsafe {
        (&raw mut (*xosc).startup).write_volatile(XOSC_STARTUP_DELAY);
        (&raw mut (*xosc).ctrl).write_volatile(XOSC_CTRL_ENABLE_1_15MHZ);
        let status = &raw const (*xosc).status;
        while status.read_volatile() & XOSC_STATUS_STABLE == 0 {}
    }
}

/// Reset, program and start one PLL, following the five-step sequence of
/// §8.6.4 (p581):
///
/// 1. reference divider;
/// 2. feedback divider;
/// 3. power up the PLL and its VCO;
/// 4. wait for `CS.LOCK`;
/// 5. set the post dividers and power them up.
///
/// The post dividers are powered last so that nothing downstream ever sees
/// the VCO while it is still settling. The PLL is reset first (via
/// `RESETS`, Table 534, p504) so that the sequence always starts from the
/// power-down reset state, whatever ran before.
unsafe fn start_pll(pll: *mut Pll, reset_bit: u32, cfg: &PllConfig) {
    unsafe {
        cycle_reset(reset_bit);
        (&raw mut (*pll).cs).write_volatile(cfg.refdiv);
        (&raw mut (*pll).fbdiv_int).write_volatile(cfg.fbdiv);
        let pwr = &raw mut (*pll).pwr;
        pwr.write_volatile(pwr.read_volatile() & !(PLL_PWR_PD | PLL_PWR_VCOPD));
        let cs = &raw const (*pll).cs;
        while cs.read_volatile() & PLL_CS_LOCK == 0 {}
        (&raw mut (*pll).prim).write_volatile((cfg.postdiv1 << 16) | (cfg.postdiv2 << 12));
        pwr.write_volatile(pwr.read_volatile() & !PLL_PWR_POSTDIVPD);
    }
}

/// Point a glitchless mux at `src` and wait for `SELECTED` to confirm it
/// (§8.1.2.2, p517: "Switch the glitchless mux to an alternate source.
/// Poll the SELECTED register until the switch completes.").
unsafe fn select_glitchless(clk: *mut ClockGenerator, src_mask: u32, src: u32) {
    unsafe {
        let ctrl = &raw mut (*clk).ctrl;
        ctrl.write_volatile((ctrl.read_volatile() & !src_mask) | src);
        let selected = &raw const (*clk).selected;
        while selected.read_volatile() & (1 << src) == 0 {}
    }
}

/// Configure `clk_ref` or `clk_sys`: glitchless source `src`, optionally a
/// new aux source `(shift, mask, value)`, and divider register value `div`.
///
/// Implements §8.1.2.2 (p517) "to switch between clock sources for the aux
/// mux when the generator has a glitchless mux", plus the divisor ordering
/// rule from the `clock_configure` listing (p522).
unsafe fn switch_glitchless(
    clk: *mut ClockGenerator,
    src_mask: u32,
    src: u32,
    aux: Option<(u32, u32, u32)>,
    div: u32,
) {
    unsafe {
        let div_reg = &raw mut (*clk).div;
        // A larger divisor goes in before the switch, so the output never
        // exceeds the slower of the two settings.
        if div > div_reg.read_volatile() {
            div_reg.write_volatile(div);
        }
        if let Some((shift, mask, value)) = aux {
            // Off the aux input (source 0 is never aux) before touching the
            // glitchy aux mux, then change it.
            select_glitchless(clk, src_mask, 0);
            let ctrl = &raw mut (*clk).ctrl;
            ctrl.write_volatile((ctrl.read_volatile() & !(mask << shift)) | (value << shift));
        }
        select_glitchless(clk, src_mask, src);
        div_reg.write_volatile(div);
    }
}

/// Configure a generator that has only an aux mux (`clk_peri`, `clk_usb`):
/// §8.1.2.2 (p517), "to switch between clock sources for the aux mux when
/// the generator does not have a glitchless mux".
///
/// "Wait for the generated clock to stop" is done by polling `ENABLED`
/// until it matches `ENABLE`, as the same section recommends when the
/// destination may be much slower than `clk_sys`.
unsafe fn switch_aux_only(clk: *mut ClockGenerator, auxsrc: u32, div: u32) {
    unsafe {
        let ctrl = &raw mut (*clk).ctrl;
        ctrl.write_volatile(ctrl.read_volatile() & !AUX_CTRL_ENABLE);
        while ctrl.read_volatile() & AUX_CTRL_ENABLED != 0 {}
        let v = ctrl.read_volatile() & !(AUX_CTRL_AUXSRC_MASK << AUX_CTRL_AUXSRC_SHIFT);
        ctrl.write_volatile(v | (auxsrc << AUX_CTRL_AUXSRC_SHIFT));
        // Stopped, so the divisor can change without an intermediate speed.
        (&raw mut (*clk).div).write_volatile(div);
        ctrl.write_volatile(ctrl.read_volatile() | AUX_CTRL_ENABLE);
        while ctrl.read_volatile() & AUX_CTRL_ENABLED == 0 {}
    }
}
