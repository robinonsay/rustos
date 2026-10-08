//! The Cortex-M33 SysTick counter, run free on the processor clock: a
//! second witness of elapsed time, apart from the microsecond tick.
//!
//! # Why a second clock
//!
//! Every other time source in this crate counts the 1 µs tick that the
//! `TICKS` block divides from `clk_ref`: `TIMER0` ([`crate::timer`]) and the
//! watchdog ([`crate::watchdog`]) alike. Anything that stops that tick —
//! its generator switched off, `TIMER0` paused or held in reset, `clk_ref`
//! stopping — stops every deadline measured on it at once, while the code
//! that should notice runs on. Each Cortex-M33 core also has "a standard
//! 24-bit SysTick timer, counting either the microsecond tick (Section 8.5)
//! or the system clock" (§12.8.1.2, p1181). Counting the system clock
//! (`SYST_CSR.CLKSOURCE` = 1, "Processor clock", Table 188, p178), it
//! shares none of that path: it counts `clk_sys`, 150 MHz from `PLL_SYS`
//! ([`crate::clocks`]). Software that reads both can tell when the
//! microsecond tick has stopped while the processor has not.
//!
//! What the two still share: `clk_sys` and `clk_ref` both descend from
//! the crystal, `clk_ref` directly and `clk_sys` through `PLL_SYS`. What
//! `clk_sys` does when the crystal stops — whether the PLL's oscillator
//! runs on without its reference, slows, or stops — the datasheet does not
//! say (inferred: unspecified). If the processor clock stops, no software
//! runs to notice anything; that case needs hardware outside the chip.
//!
//! # Free-running, no interrupt
//!
//! [`Rp2350SysTick::new`] disables the counter, loads the largest reload
//! value, `0x00ff_ffff` (Table 189, p179: "any value between 0 and
//! 0x00FFFFFF"), clears the current value (Table 190, p179: "Writing to it
//! with any value clears the register to 0"), then enables it on the
//! processor clock with `TICKINT` = 0, so it never raises the SysTick
//! exception (Table 188, p178). It then counts down from `0x00ff_ffff` to 0
//! and reloads, for ever: one count per `clk_sys` cycle, wrapping every
//! 2^24 cycles, about 112 ms at 150 MHz. [`Rp2350SysTick::cycles`] turns
//! it into a count that goes up, modulo 2^24 ([`CYCLES_MASK`]); a reader
//! that wants elapsed time takes the difference of two readings modulo
//! 2^24, which is right as long as the readings are less than one wrap
//! apart.
//!
//! The registers are the core's own, at `PPB_BASE` `0xe000_0000` plus
//! `0x0e010`–`0x0e018` (§3.7.5, p149; Table 120, p153). Each core has its
//! own: this driver drives the SysTick of the core that calls it.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

use api::device::DeviceHandle;

use crate::clocks::{Rp2350Clocks, CLK_SYS_HZ};

/// `SYST_CSR`, `SYST_RVR`, `SYST_CVR`: `PPB_BASE` + `0x0e010` (Table 120,
/// p153).
const SYSTICK_BASE: usize = 0xe000_e010;

/// The three SysTick registers this driver uses (Tables 188–190,
/// p178–179).
#[repr(C)]
struct SysTick {
    /// `SYST_CSR`: bit 0 `ENABLE`, bit 1 `TICKINT`, bit 2 `CLKSOURCE`,
    /// bit 16 `COUNTFLAG`.
    csr: u32,
    /// `SYST_RVR`: bits 23:0 `RELOAD`; reset value UNKNOWN.
    rvr: u32,
    /// `SYST_CVR`: bits 23:0 `CURRENT`; any write clears it to 0.
    cvr: u32,
}

const _: () = assert!(core::mem::offset_of!(SysTick, rvr) == 0x04);
const _: () = assert!(core::mem::offset_of!(SysTick, cvr) == 0x08);

/// `SYST_CSR.ENABLE`.
const CSR_ENABLE: u32 = 1 << 0;
/// `SYST_CSR.CLKSOURCE`: 1 = processor clock.
const CSR_CLKSOURCE_PROCESSOR: u32 = 1 << 2;

/// The counter's 24 bits: [`Rp2350SysTick::cycles`] counts modulo
/// `CYCLES_MASK + 1`.
pub const CYCLES_MASK: u32 = 0x00ff_ffff;

/// `clk_sys` cycles per millisecond: what [`Rp2350SysTick::cycles`] advances
/// by in one millisecond.
pub const CYCLES_PER_MS: u32 = CLK_SYS_HZ / 1000;

// One wrap must be longer than a millisecond, or a reader could not tell
// elapsed milliseconds at all.
const _: () = assert!(CYCLES_PER_MS < CYCLES_MASK);

/// The SysTick counter of the calling core, counting `clk_sys` cycles. See
/// the [module documentation](self).
pub struct Rp2350SysTick {
    _private: (),
}

impl Rp2350SysTick {
    /// Start the counter free-running on the processor clock, with no
    /// interrupt. Takes `&Rp2350Clocks` because the count is only
    /// [`CYCLES_PER_MS`] per millisecond once `clk_sys` runs from
    /// `PLL_SYS`.
    pub fn new(_handle: DeviceHandle<Rp2350SysTick>, _clocks: &Rp2350Clocks) -> Self {
        let st = SYSTICK_BASE as *mut SysTick;
        unsafe {
            (&raw mut (*st).csr).write_volatile(0);
            (&raw mut (*st).rvr).write_volatile(CYCLES_MASK);
            (&raw mut (*st).cvr).write_volatile(0);
            (&raw mut (*st).csr).write_volatile(CSR_ENABLE | CSR_CLKSOURCE_PROCESSOR);
        }
        Self { _private: () }
    }

    /// `clk_sys` cycles since an arbitrary start, modulo 2^24: the counter
    /// counts down, so this is `CYCLES_MASK - CURRENT`.
    pub fn cycles(&self) -> u32 {
        let st = SYSTICK_BASE as *const SysTick;
        let current = unsafe { (&raw const (*st).cvr).read_volatile() } & CYCLES_MASK;
        CYCLES_MASK - current
    }
}
