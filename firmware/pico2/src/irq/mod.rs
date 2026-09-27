//! Interrupt lines and the Cortex-M33 NVIC: which peripheral request is
//! which line, how an application installs a handler for a line, and how it
//! enables the line.
//!
//! ## Installing a handler
//!
//! The vector table in this crate points every one of the 52 device-interrupt
//! slots (exception numbers 16 to 67) at a symbol named after the line, such as
//! `TIMER0_IRQ_1`. The linker script gives each of those symbols the default
//! value `DefaultHandler` with `PROVIDE`, so a line nobody handles spins in
//! `DefaultHandler` exactly as before. An application that defines the symbol
//! overrides the default at link time; [`interrupt!`](crate::interrupt) is the
//! checked way to define it:
//!
//! ```ignore
//! fn on_tick() { /* acknowledge, sample, re-arm; run to completion */ }
//! pico2::interrupt!(TIMER0_IRQ_1, on_tick);
//! ```
//!
//! The macro rejects a name that is not a line of [`Irq`] at compile time, so
//! a misspelt handler cannot silently never run.
//!
//! ## One priority
//!
//! [`Rp2350Nvic`] implements [`api::irq::InterruptController`], which has no
//! priority operation, and this crate writes no `NVIC_IPR` register anywhere.
//! Every enabled line therefore keeps the reset priority 0 and no handler
//! preempts another: handlers run to completion, one at a time (cwht coding
//! standard CS-34). When two lines are pending at once the lower line number
//! is taken first (datasheet §3.2, p83).
//!
//! ## Datasheet sources
//!
//! - §3.2, Table 94 (p82, p83): system-level interrupt numbering
//! - §3.7.5, Tables 192 to 195 (p179): `NVIC_ISER0/1`, `NVIC_ICER0/1`,
//!   `NVIC_ISPR0/1`, `NVIC_ICPR0/1`, at PPB offsets `0x0e100`, `0x0e180`,
//!   `0x0e200`, `0x0e280`

use core::mem::offset_of;

use api::common::ErrorType;
#[cfg(target_os = "none")]
use api::device::DeviceHandle;
#[cfg(target_os = "none")]
use api::irq::InterruptController;

use crate::common::reg::{RegAddr, Regs};

/// The RP2350's 52 system-level interrupt lines, numbered as in Table 94.
///
/// Only lines 0 to 45 are wired to peripherals; 46 to 51 are
/// `SPAREIRQ_IRQ_0..5`, "hardwired to zero (never firing)", which software can
/// pend on itself.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Names match the datasheet's line names exactly, and the handler symbols.
#[allow(non_camel_case_types)]
pub enum Irq {
    /// IRQ 0.
    TIMER0_IRQ_0 = 0,
    /// IRQ 1.
    TIMER0_IRQ_1 = 1,
    /// IRQ 2.
    TIMER0_IRQ_2 = 2,
    /// IRQ 3.
    TIMER0_IRQ_3 = 3,
    /// IRQ 4.
    TIMER1_IRQ_0 = 4,
    /// IRQ 5.
    TIMER1_IRQ_1 = 5,
    /// IRQ 6.
    TIMER1_IRQ_2 = 6,
    /// IRQ 7.
    TIMER1_IRQ_3 = 7,
    /// IRQ 8.
    PWM_IRQ_WRAP_0 = 8,
    /// IRQ 9.
    PWM_IRQ_WRAP_1 = 9,
    /// IRQ 10.
    DMA_IRQ_0 = 10,
    /// IRQ 11.
    DMA_IRQ_1 = 11,
    /// IRQ 12.
    DMA_IRQ_2 = 12,
    /// IRQ 13.
    DMA_IRQ_3 = 13,
    /// IRQ 14.
    USBCTRL_IRQ = 14,
    /// IRQ 15.
    PIO0_IRQ_0 = 15,
    /// IRQ 16.
    PIO0_IRQ_1 = 16,
    /// IRQ 17.
    PIO1_IRQ_0 = 17,
    /// IRQ 18.
    PIO1_IRQ_1 = 18,
    /// IRQ 19.
    PIO2_IRQ_0 = 19,
    /// IRQ 20.
    PIO2_IRQ_1 = 20,
    /// IRQ 21.
    IO_IRQ_BANK0 = 21,
    /// IRQ 22.
    IO_IRQ_BANK0_NS = 22,
    /// IRQ 23.
    IO_IRQ_QSPI = 23,
    /// IRQ 24.
    IO_IRQ_QSPI_NS = 24,
    /// IRQ 25.
    SIO_IRQ_FIFO = 25,
    /// IRQ 26.
    SIO_IRQ_BELL = 26,
    /// IRQ 27.
    SIO_IRQ_FIFO_NS = 27,
    /// IRQ 28.
    SIO_IRQ_BELL_NS = 28,
    /// IRQ 29.
    SIO_IRQ_MTIMECMP = 29,
    /// IRQ 30.
    CLOCKS_IRQ = 30,
    /// IRQ 31.
    SPI0_IRQ = 31,
    /// IRQ 32.
    SPI1_IRQ = 32,
    /// IRQ 33.
    UART0_IRQ = 33,
    /// IRQ 34.
    UART1_IRQ = 34,
    /// IRQ 35.
    ADC_IRQ_FIFO = 35,
    /// IRQ 36.
    I2C0_IRQ = 36,
    /// IRQ 37.
    I2C1_IRQ = 37,
    /// IRQ 38.
    OTP_IRQ = 38,
    /// IRQ 39.
    TRNG_IRQ = 39,
    /// IRQ 40.
    PROC0_IRQ_CTI = 40,
    /// IRQ 41.
    PROC1_IRQ_CTI = 41,
    /// IRQ 42.
    PLL_SYS_IRQ = 42,
    /// IRQ 43.
    PLL_USB_IRQ = 43,
    /// IRQ 44.
    POWMAN_IRQ_POW = 44,
    /// IRQ 45.
    POWMAN_IRQ_TIMER = 45,
    /// IRQ 46.
    SPAREIRQ_IRQ_0 = 46,
    /// IRQ 47.
    SPAREIRQ_IRQ_1 = 47,
    /// IRQ 48.
    SPAREIRQ_IRQ_2 = 48,
    /// IRQ 49.
    SPAREIRQ_IRQ_3 = 49,
    /// IRQ 50.
    SPAREIRQ_IRQ_4 = 50,
    /// IRQ 51.
    SPAREIRQ_IRQ_5 = 51,
}

impl Irq {
    /// The line number, 0 to 51.
    #[must_use]
    pub const fn line(self) -> u16 {
        self as u16
    }
}

/// Number of interrupt lines (Table 94).
pub const LINES: u16 = 52;

/// The NVIC enable and pending registers, from `NVIC_ISER0` at PPB offset
/// `0x0e100` (§3.7.5, Tables 192 to 195). Each group is two words: lines
/// 0 to 31, then 32 to 63.
#[repr(C)]
struct NvicRegs {
    /// `NVIC_ISER0/1` `0x0e100`: write 1 to enable; read enabled state.
    iser: [u32; 2],
    reserved0: [u32; 30],
    /// `NVIC_ICER0/1` `0x0e180`: write 1 to disable; read enabled state.
    icer: [u32; 2],
    reserved1: [u32; 30],
    /// `NVIC_ISPR0/1` `0x0e200`: write 1 to pend; read pending state.
    ispr: [u32; 2],
    reserved2: [u32; 30],
    /// `NVIC_ICPR0/1` `0x0e280`: write 1 to clear pending; read pending state.
    icpr: [u32; 2],
}

const ISER: usize = offset_of!(NvicRegs, iser);
const ICER: usize = offset_of!(NvicRegs, icer);
const ISPR: usize = offset_of!(NvicRegs, ispr);
const ICPR: usize = offset_of!(NvicRegs, icpr);
// Offsets from RegAddr::NVIC = PPB_BASE + 0x0e100.
const _: () = assert!(ISER == 0x000 && ICER == 0x080 && ISPR == 0x100 && ICPR == 0x180);

/// A line number the NVIC does not have (IRQ-4 of the `api::irq` contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoSuchLine(pub u16);

/// Word offset and bit of `line` in a two-word NVIC register group. Pure.
///
/// # Errors
///
/// [`NoSuchLine`] for `line >= LINES`.
pub(crate) fn locate(line: u16) -> Result<(usize, u32), NoSuchLine> {
    if line >= LINES {
        return Err(NoSuchLine(line));
    }
    Ok((usize::from(line / 32) * 4, 1 << (line % 32)))
}

/// Write the line's bit into the register group at `group` (set-by-one
/// semantics: the other bits written are 0 and change nothing).
pub(crate) fn write_line<R: Regs>(regs: &mut R, group: usize, line: u16) -> Result<(), NoSuchLine> {
    let (word, bit) = locate(line)?;
    regs.write(RegAddr::NVIC, group + word, bit);
    Ok(())
}

/// Read the line's bit from the register group at `group`.
pub(crate) fn read_line<R: Regs>(
    regs: &mut R,
    group: usize,
    line: u16,
) -> Result<bool, NoSuchLine> {
    let (word, bit) = locate(line)?;
    Ok(regs.read(RegAddr::NVIC, group + word) & bit != 0)
}

/// The NVIC of the running core, as the board's `nvic` device.
///
/// Zero-sized. Constructing it takes the board's `DeviceHandle<Rp2350Nvic>`,
/// so one value exists per boot. Implements
/// [`api::irq::InterruptController`] on the target.
pub struct Rp2350Nvic {
    _private: (),
}

impl Rp2350Nvic {
    /// Claim the NVIC. No register is written: every line is disabled and
    /// not pending out of reset (IRQ-6).
    #[cfg(target_os = "none")]
    #[must_use]
    pub const fn new(_handle: DeviceHandle<Self>) -> Self {
        Self { _private: () }
    }
}

impl ErrorType for Rp2350Nvic {
    type Error = NoSuchLine;
}

/// Each method is one call of [`write_line`] or [`read_line`] on the
/// hardware; the decisions are in [`locate`], host-tested.
#[cfg(target_os = "none")]
impl InterruptController for Rp2350Nvic {
    fn enable(&mut self, line: u16) -> Result<(), NoSuchLine> {
        write_line(&mut crate::common::reg::Mmio, ISER, line)
    }
    fn disable(&mut self, line: u16) -> Result<(), NoSuchLine> {
        write_line(&mut crate::common::reg::Mmio, ICER, line)
    }
    fn is_enabled(&mut self, line: u16) -> Result<bool, NoSuchLine> {
        read_line(&mut crate::common::reg::Mmio, ISER, line)
    }
    fn pend(&mut self, line: u16) -> Result<(), NoSuchLine> {
        write_line(&mut crate::common::reg::Mmio, ISPR, line)
    }
    fn unpend(&mut self, line: u16) -> Result<(), NoSuchLine> {
        write_line(&mut crate::common::reg::Mmio, ICPR, line)
    }
    fn is_pending(&mut self, line: u16) -> Result<bool, NoSuchLine> {
        read_line(&mut crate::common::reg::Mmio, ISPR, line)
    }
}

/// Define the handler of one interrupt line.
///
/// `pico2::interrupt!(LINE, handler)` defines the linker symbol `LINE`,
/// overriding the `PROVIDE(LINE = DefaultHandler)` default of `link.ld`, as an
/// `extern "C"` function that calls `handler: fn()`. `LINE` must be a variant
/// of [`Irq`](crate::irq::Irq) (checked at compile time) and `handler` must
/// have type `fn()` (checked by coercion, as [`entry!`](crate::entry) does).
///
/// Enabling the line at the NVIC is a separate step
/// ([`api::irq::InterruptController::enable`]); until then the handler never
/// runs.
#[macro_export]
macro_rules! interrupt {
    ($line:ident, $handler:path) => {
        const _: $crate::irq::Irq = $crate::irq::Irq::$line;
        // SAFETY: `no_mangle` is unsafe because an unmangled symbol can collide with another of
        // the same name. The name is a line of `Irq` (checked by the const above), whose only
        // other definition is the weak `PROVIDE(<line> = DefaultHandler)` of `link.ld`, which a
        // strong definition overrides by design; a second `interrupt!` for the same line is a
        // duplicate-symbol link error. The vector table calls it as `unsafe extern "C" fn()`,
        // the signature defined here.
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $line() {
            let f: fn() = $handler;
            f()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::reg::fake::FakeRegs;

    #[test]
    fn line_numbers_match_table_94() {
        assert_eq!(Irq::TIMER0_IRQ_0.line(), 0);
        assert_eq!(Irq::TIMER0_IRQ_1.line(), 1);
        assert_eq!(Irq::PWM_IRQ_WRAP_0.line(), 8);
        assert_eq!(Irq::IO_IRQ_BANK0.line(), 21);
        assert_eq!(Irq::CLOCKS_IRQ.line(), 30);
        assert_eq!(Irq::UART0_IRQ.line(), 33);
        assert_eq!(Irq::PLL_SYS_IRQ.line(), 42);
        assert_eq!(Irq::SPAREIRQ_IRQ_5.line(), 51);
    }

    #[test]
    fn locate_splits_lines_across_the_two_words() {
        assert_eq!(locate(0), Ok((0, 1)));
        assert_eq!(locate(31), Ok((0, 1 << 31)));
        assert_eq!(locate(32), Ok((4, 1)));
        assert_eq!(locate(51), Ok((4, 1 << 19)));
    }

    #[test]
    fn locate_rejects_lines_past_51() {
        assert_eq!(locate(52), Err(NoSuchLine(52)));
        assert_eq!(locate(u16::MAX), Err(NoSuchLine(u16::MAX)));
    }

    #[test]
    fn enable_disable_pend_unpend_write_one_bit_in_the_right_register() {
        let mut regs = FakeRegs::new();
        write_line(&mut regs, ISER, 1).unwrap();
        write_line(&mut regs, ICER, 33).unwrap();
        write_line(&mut regs, ISPR, 46).unwrap();
        write_line(&mut regs, ICPR, 0).unwrap();
        assert_eq!(
            regs.writes(),
            std::vec![
                (RegAddr::NVIC, 0x000, 1 << 1),
                (RegAddr::NVIC, 0x084, 1 << 1),
                (RegAddr::NVIC, 0x104, 1 << 14),
                (RegAddr::NVIC, 0x180, 1 << 0),
            ]
        );
    }

    #[test]
    fn out_of_range_lines_touch_no_register() {
        let mut regs = FakeRegs::new();
        assert_eq!(write_line(&mut regs, ISER, 52), Err(NoSuchLine(52)));
        assert_eq!(read_line(&mut regs, ISPR, 60), Err(NoSuchLine(60)));
        assert!(regs.log().is_empty());
    }

    #[test]
    fn read_line_reports_only_its_own_bit() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::NVIC, 0x004, !(1 << 3));
        assert_eq!(read_line(&mut regs, ISER, 35), Ok(false));
        assert_eq!(read_line(&mut regs, ISER, 36), Ok(true));
    }

    #[test]
    fn no_priority_register_is_ever_addressed() {
        // NVIC_IPR0 is at PPB 0x0e400, offset 0x300 from RegAddr::NVIC.
        let mut regs = FakeRegs::new();
        for line in 0..LINES {
            for group in [ISER, ICER, ISPR, ICPR] {
                write_line(&mut regs, group, line).unwrap();
            }
        }
        assert!(regs.writes().iter().all(|(_, off, _)| *off < 0x188));
    }
}
