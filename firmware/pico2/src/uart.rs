//! The two PL011 UARTs: asynchronous serial with 32-byte FIFOs, polled,
//! never blocking.
//!
//! # Using it
//!
//! ```ignore
//! use api::common::{Read, Write};
//! use pico2::uart::{Rp2350Uart, UartConfig};
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! // GPS on UART1: TX on GPIO4, RX on GPIO5, 9600 baud, 8 data bits, no parity, 1 stop bit.
//! let mut gps = Rp2350Uart::new(board.uart1, &clocks, board.pins.gpio4, board.pins.gpio5,
//!                               UartConfig::new_8n1(9600)).unwrap();
//! let mut buf = [0u8; 64];
//! let n = gps.read(&mut buf);          // whatever has arrived, possibly 0
//! let sent = gps.write(b"$PMTK...\r\n"); // as much as fits in the TX FIFO
//! ```
//!
//! # The block
//!
//! RP2350 has two identical UARTs, each an Arm PrimeCell PL011 r1p5 with a
//! 32 × 8-bit transmit FIFO and a 32 × 12-bit receive FIFO; modem and IrDA
//! modes are not supported
//! ([§12.1, p959](../../../docs/rp2350-datasheet.pdf#page=959)). The baud
//! rate generator runs from `clk_peri` (`UARTCLK`); the register interface
//! runs from `clk_sys` (`PCLK`) (same page). [`crate::clocks`] runs both
//! at 150 MHz.
//!
//! # Bring-up order
//!
//! [`Rp2350Uart::new`] follows the order of the SDK's `uart_init`, which
//! the datasheet reproduces
//! ([§12.1.7, p968–969](../../../docs/rp2350-datasheet.pdf#page=968)):
//!
//! 1. Reset the UART (`RESETS` bit 26 or 27,
//!    [Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)), so
//!    every register holds its reset value even after a debugger warm
//!    reset. `UARTCR.UARTEN` resets to 0: the UART is disabled.
//! 2. Write the baud-rate divisors `UARTIBRD` and `UARTFBRD`.
//! 3. Write `UARTLCR_H`: word length, parity, stop bits, FIFOs on. This
//!    write also latches the divisors: "PL011 needs a (dummy) LCR_H write
//!    to latch in the divisors" (SDK source in §12.1.7.1,
//!    [p970](../../../docs/rp2350-datasheet.pdf#page=970)), and "Any LCR
//!    writes need to take place before enabling the UART" (p969).
//! 4. Write `UARTCR`: `UARTEN`, `TXE`, `RXE`; flow control and loopback
//!    off.
//! 5. Connect the TX and RX pins (`common::pinmux`).
//!
//! No interrupt and no DMA request is enabled: `UARTIMSC` and `UARTDMACR`
//! keep their reset value 0
//! ([Table 1037, p976](../../../docs/rp2350-datasheet.pdf#page=976);
//! [Table 1041, p979](../../../docs/rp2350-datasheet.pdf#page=979)).
//!
//! # Receive errors
//!
//! Each received character carries four status bits in the FIFO
//! ([Table 1024, p964](../../../docs/rp2350-datasheet.pdf#page=964)):
//! framing, parity and break describe *that* character, which is
//! therefore not data; overrun says characters were lost *before* it, but
//! the character itself is good ("The overrun bit is not associated with
//! the character in the receive FIFO",
//! [§12.1.3.2.4, p963](../../../docs/rp2350-datasheet.pdf#page=963)). So
//! [`Rp2350Uart::read`] drops framing, parity and break characters, keeps
//! overrun ones, and counts all four in [`LineErrors`].
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer), so the `#page=`
//! links open the cited page.

use api::common::{ErrorType, Read, Write};
use api::device::{DeviceHandle, PinHandle};

use crate::clocks::{CLK_PERI_HZ, CLK_SYS_HZ, Rp2350Clocks};
use crate::common::pinmux::{self, PadConfig, Role};
use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

// "The frequency of UARTCLK must be no more than 5/3 times faster than the
// frequency of PCLK" (§12.1.3.1, p962). UARTCLK = clk_peri, PCLK = clk_sys.
const _: () = assert!(CLK_PERI_HZ as u64 * 3 <= CLK_SYS_HZ as u64 * 5);

// --- Register layout -----------------------------------------------------

/// The PL011 registers this driver uses, at their offsets from
/// [Table 1027, p970–971](../../../docs/rp2350-datasheet.pdf#page=970).
///
/// `pub(crate)` so the DMA driver, [`crate::uart_dma`], can name `UARTDR`
/// as a DMA address and set `UARTDMACR`.
#[repr(C)]
pub(crate) struct Uart {
    /// `0x000` `UARTDR` (Table 1028, p971–972): bits 7:0 data, 8 `FE`,
    /// 9 `PE`, 10 `BE`, 11 `OE`. A read pops the receive FIFO; a write
    /// pushes the transmit FIFO.
    pub(crate) dr: u32,
    /// `0x004` `UARTRSR`/`UARTECR` (Table 1029, p972). Not used: the same
    /// error bits arrive with each character in `UARTDR`.
    _rsr: u32,
    /// `0x008`–`0x014`: not in the register list.
    _reserved0: [u32; 4],
    /// `0x018` `UARTFR` (Table 1030, p972–973): FIFO and busy flags.
    pub(crate) fr: u32,
    /// `0x01c`: not in the register list.
    _reserved1: u32,
    /// `0x020` `UARTILPR` (Table 1031, p973): IrDA only, not used.
    _ilpr: u32,
    /// `0x024` `UARTIBRD` (Table 1032, p973–974): bits 15:0, the integer
    /// part of the baud-rate divisor.
    ibrd: u32,
    /// `0x028` `UARTFBRD` (Table 1033, p974): bits 5:0, the fractional
    /// part in 64ths.
    fbrd: u32,
    /// `0x02c` `UARTLCR_H` (Table 1034, p974): frame format and FIFO
    /// enable.
    lcr_h: u32,
    /// `0x030` `UARTCR` (Table 1035, p975–976): enables.
    cr: u32,
    /// `0x034`–`0x044`: `UARTIFLS`, `UARTIMSC`, `UARTRIS`, `UARTMIS`,
    /// `UARTICR`; not used (Tables 1036–1040, p976–979).
    _reserved34: [u32; 5],
    /// `0x048` `UARTDMACR` (Table 1041, p979): `RXDMAE` bit 0, `TXDMAE`
    /// bit 1, `DMAONERR` bit 2. Reset 0. Used by [`crate::uart_dma`].
    pub(crate) dmacr: u32,
}

// The offsets above, checked against Table 1027 (p970–971).
const _: () = assert!(core::mem::offset_of!(Uart, fr) == 0x018);
const _: () = assert!(core::mem::offset_of!(Uart, ibrd) == 0x024);
const _: () = assert!(core::mem::offset_of!(Uart, fbrd) == 0x028);
const _: () = assert!(core::mem::offset_of!(Uart, lcr_h) == 0x02c);
const _: () = assert!(core::mem::offset_of!(Uart, cr) == 0x030);
const _: () = assert!(core::mem::offset_of!(Uart, dmacr) == 0x048);

/// `UARTDR.DATA`, bits 7:0 (Table 1028, p972).
const DR_DATA: u32 = 0xff;
/// `UARTDR.FE`, bit 8: framing error (Table 1028, p972).
const DR_FE: u32 = 1 << 8;
/// `UARTDR.PE`, bit 9: parity error (Table 1028, p971).
const DR_PE: u32 = 1 << 9;
/// `UARTDR.BE`, bit 10: break error (Table 1028, p971).
const DR_BE: u32 = 1 << 10;
/// `UARTDR.OE`, bit 11: overrun before this character (Table 1028, p971).
const DR_OE: u32 = 1 << 11;

/// `UARTFR.BUSY`, bit 3: transmitting, or the TX FIFO is not empty
/// (Table 1030, p973).
pub(crate) const FR_BUSY: u32 = 1 << 3;
/// `UARTFR.RXFE`, bit 4, reset 1: receive FIFO empty (Table 1030, p973).
pub(crate) const FR_RXFE: u32 = 1 << 4;
/// `UARTFR.TXFF`, bit 5: transmit FIFO full (Table 1030, p973).
const FR_TXFF: u32 = 1 << 5;

/// `UARTLCR_H.PEN`, bit 1: parity enable (Table 1034, p974).
const LCR_H_PEN: u32 = 1 << 1;
/// `UARTLCR_H.EPS`, bit 2: 1 = even parity, 0 = odd (Table 1034, p974).
const LCR_H_EPS: u32 = 1 << 2;
/// `UARTLCR_H.STP2`, bit 3: two stop bits (Table 1034, p974).
const LCR_H_STP2: u32 = 1 << 3;
/// `UARTLCR_H.FEN`, bit 4: FIFOs enabled (Table 1034, p974).
const LCR_H_FEN: u32 = 1 << 4;
/// `UARTLCR_H.WLEN` starts at bit 5; `0b11` = 8 bits, `0b00` = 5 bits
/// (Table 1034, p974).
const LCR_H_WLEN_SHIFT: u32 = 5;

/// `UARTCR.UARTEN`, bit 0: UART enable (Table 1035, p976).
const CR_UARTEN: u32 = 1 << 0;
/// `UARTCR.TXE`, bit 8: transmit enable (Table 1035, p975).
const CR_TXE: u32 = 1 << 8;
/// `UARTCR.RXE`, bit 9: receive enable (Table 1035, p975).
const CR_RXE: u32 = 1 << 9;

/// `UART0` is bit 26 of `RESETS.RESET`, `UART1` bit 27
/// ([Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)).
const RESET_UART0_BIT: usize = 26;

// --- Configuration -------------------------------------------------------

/// Data bits per character (`UARTLCR_H.WLEN`, Table 1034, p974).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataBits {
    /// 5 data bits, `WLEN` = `0b00`.
    Five = 0,
    /// 6 data bits, `WLEN` = `0b01`.
    Six = 1,
    /// 7 data bits, `WLEN` = `0b10`.
    Seven = 2,
    /// 8 data bits, `WLEN` = `0b11`.
    Eight = 3,
}

/// Parity bit (`UARTLCR_H.PEN` and `EPS`, Table 1034, p974). Stick parity
/// (`SPS`) is not offered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    /// No parity bit: `PEN` = 0.
    None,
    /// Even number of 1s in data plus parity: `PEN` = 1, `EPS` = 1.
    Even,
    /// Odd number of 1s in data plus parity: `PEN` = 1, `EPS` = 0.
    Odd,
}

/// Stop bits sent after each character (`UARTLCR_H.STP2`, Table 1034,
/// p974). "The receive logic does not check for two stop bits being
/// received."
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopBits {
    /// One stop bit: `STP2` = 0.
    One,
    /// Two stop bits: `STP2` = 1.
    Two,
}

/// Line settings for [`Rp2350Uart::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UartConfig {
    /// Bits per second, [`MIN_BAUD`]–[`MAX_BAUD`].
    pub baud: u32,
    /// Data bits per character.
    pub data_bits: DataBits,
    /// Parity bit.
    pub parity: Parity,
    /// Stop bits.
    pub stop_bits: StopBits,
}

impl UartConfig {
    /// `baud` with 8 data bits, no parity and 1 stop bit ("8N1"), the
    /// most common format.
    pub const fn new_8n1(baud: u32) -> Self {
        // The common default; every field spelled out for review.
        Self { baud, data_bits: DataBits::Eight, parity: Parity::None, stop_bits: StopBits::One }
    }

    /// The `UARTLCR_H` value for this format, with the FIFOs enabled and
    /// `BRK` and `SPS` clear (Table 1034, p974).
    const fn lcr_h(&self) -> u32 {
        // WLEN: the enum's discriminant is the 2-bit field value.
        let wlen = (self.data_bits as u32) << LCR_H_WLEN_SHIFT;
        // PEN and EPS from the parity choice.
        let parity = match self.parity {
            Parity::None => 0,
            Parity::Even => LCR_H_PEN | LCR_H_EPS,
            Parity::Odd => LCR_H_PEN,
        };
        // STP2 from the stop-bit choice.
        let stop = match self.stop_bits {
            StopBits::One => 0,
            StopBits::Two => LCR_H_STP2,
        };
        // FEN on: 32-deep FIFOs instead of 1-byte holding registers.
        wlen | parity | stop | LCR_H_FEN
    }
}

/// The baud-rate divisor, `UARTCLK / (16 × baud)`, as the 16-bit integer
/// and 6-bit fraction the PL011 takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaudDivisor {
    /// The whole divisor in 64ths: `64 × IBRD + FBRD`.
    sixty_fourths: u32,
}

/// Smallest divisor: `IBRD` = 1, `FBRD` = 0, i.e. `UARTCLK / 16` baud,
/// the PL011's maximum rate "UARTCLK / 16" ([§12.1.1, p959](../../../docs/rp2350-datasheet.pdf#page=959)).
const MIN_DIVISOR_64THS: u32 = 64;
/// Largest divisor: `IBRD` = 65535 (16 bits, Table 1032, p974), `FBRD` = 0;
/// §12.1.3.1 (p962) bounds the slowest rate by `16 × 65535 × baud`.
const MAX_DIVISOR_64THS: u32 = 65535 * 64;

/// Fastest rate [`Rp2350Uart::new`] accepts: `clk_peri / 16`, 9.375 Mbaud.
pub const MAX_BAUD: u32 = CLK_PERI_HZ / 16;
/// Slowest rate [`Rp2350Uart::new`] accepts, rounded up: about 143 baud.
pub const MIN_BAUD: u32 = (4 * CLK_PERI_HZ as u64).div_ceil(MAX_DIVISOR_64THS as u64) as u32;

impl BaudDivisor {
    /// The divisor for `baud` at `UARTCLK` = `clk_hz`, or `None` if it does
    /// not fit the registers.
    ///
    /// §12.1.3.2.1 (p962–963) splits `UARTCLK / (16 × baud)` into an
    /// integer part and a fraction `F`, and rounds the fraction to 64ths as
    /// `m = integer(F × 64 + 0.5)`; the worked example is in §12.1.7.1
    /// ([p970](../../../docs/rp2350-datasheet.pdf#page=970)). In 64ths the
    /// whole divisor is `4 × clk_hz / baud`, so this rounds that to the
    /// nearest integer, as `(8 × clk_hz / baud + 1) / 2` — the expression
    /// the SDK's `uart_set_baudrate` uses (p970, line 157). Rounding the
    /// whole value at once also carries a fraction that rounds up to 64/64
    /// into the integer part, where the per-part formula would produce an
    /// `FBRD` of 64 that does not fit 6 bits.
    pub const fn for_baud(clk_hz: u32, baud: u32) -> Option<Self> {
        // A zero rate has no divisor.
        if baud == 0 {
            return None;
        }
        // 8 × clk / baud, in u64 so 8 × 150 MHz cannot overflow.
        let eighth_units = 8 * clk_hz as u64 / baud as u64;
        // Round half up to whole 64ths.
        let sixty_fourths = eighth_units.div_ceil(2);
        // IBRD must be 1..=65535 with FBRD 0 at the top end.
        if sixty_fourths < MIN_DIVISOR_64THS as u64 || sixty_fourths > MAX_DIVISOR_64THS as u64 {
            return None;
        }
        // In range, so it fits a u32.
        Some(Self { sixty_fourths: sixty_fourths as u32 })
    }

    /// `UARTIBRD`: the integer part.
    pub const fn ibrd(&self) -> u32 {
        // 64ths → whole units.
        self.sixty_fourths >> 6
    }

    /// `UARTFBRD`: the fraction, in 64ths.
    pub const fn fbrd(&self) -> u32 {
        // The low 6 bits.
        self.sixty_fourths & 0x3f
    }

    /// The rate actually produced at `UARTCLK` = `clk_hz`, to the nearest
    /// baud: `clk_hz / (16 × divisor)` = `4 × clk_hz / sixty_fourths`
    /// (p970, line 179 of the SDK source).
    pub const fn baud(&self, clk_hz: u32) -> u32 {
        // Numerator in u64; add half the divisor to round to nearest.
        ((4 * clk_hz as u64 + self.sixty_fourths as u64 / 2) / self.sixty_fourths as u64) as u32
    }
}

// The datasheet's worked example (§12.1.7.1, p970): 115 200 baud at
// 125 MHz gives IBRD 67, FBRD 52, about 115 207 baud.
const _: () = {
    // Unwrap in a const context: a None here fails the build.
    let Some(d) = BaudDivisor::for_baud(125_000_000, 115_200) else { panic!() };
    assert!(d.ibrd() == 67 && d.fbrd() == 52);
    assert!(d.baud(125_000_000) == 115_207);
};
// 9600 baud at this crate's clk_peri divides exactly: 976 + 36/64.
const _: () = {
    // 4 × 150 MHz / 9600 = 62 500 = 976 × 64 + 36.
    let Some(d) = BaudDivisor::for_baud(CLK_PERI_HZ, 9_600) else { panic!() };
    assert!(d.ibrd() == 976 && d.fbrd() == 36 && d.baud(CLK_PERI_HZ) == 9_600);
};
// Both ends of the accepted range produce a divisor, and one step past
// either end does not.
const _: () = assert!(BaudDivisor::for_baud(CLK_PERI_HZ, MAX_BAUD).is_some());
const _: () = assert!(BaudDivisor::for_baud(CLK_PERI_HZ, MIN_BAUD).is_some());
const _: () = assert!(BaudDivisor::for_baud(CLK_PERI_HZ, MAX_BAUD + MAX_BAUD / 64).is_none());
const _: () = assert!(BaudDivisor::for_baud(CLK_PERI_HZ, MIN_BAUD - 1).is_none());

// --- Errors --------------------------------------------------------------

/// What can go wrong with a UART.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UartError {
    /// [`UartConfig::baud`] is outside [`MIN_BAUD`]–[`MAX_BAUD`]. Returned
    /// by [`Rp2350Uart::new`] before any register is written; the pins are
    /// left unconnected.
    BaudOutOfRange {
        /// The rate requested.
        baud: u32,
    },
    /// Nothing to read, or no room to write. Try again later.
    WouldBlock,
    /// The character had no valid stop bit (`UARTDR.FE`, Table 1028,
    /// p972): wrong baud rate, or noise. It was discarded.
    Framing,
    /// The character's parity was wrong (`UARTDR.PE`, Table 1028, p971).
    /// It was discarded.
    Parity,
    /// The line was held low for longer than a whole character (`UARTDR.BE`,
    /// Table 1028, p971): a break, or a disconnected cable. The 0
    /// character the PL011 loads for a break was discarded.
    Break,
    /// DMA driver only ([`crate::uart_dma`]): reading fell so far behind
    /// that the receive ring wrapped and characters were overwritten
    /// before they were read. Everything in the ring was discarded; the
    /// next read returns characters from now on.
    RingOverrun,
    /// DMA driver only: a DMA channel received a bus error and halted
    /// (`CTRL.AHB_ERROR`, Table 1150, p1125). No more characters move in
    /// that direction; every later call reports this.
    DmaError,
}

/// Receive errors counted since the last [`Rp2350Uart::take_line_errors`].
/// Counts saturate at `u32::MAX`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineErrors {
    /// Characters dropped for a framing error.
    pub framing: u32,
    /// Characters dropped for a parity error.
    pub parity: u32,
    /// Break conditions seen.
    pub breaks: u32,
    /// Times the receive FIFO overflowed and characters were lost before
    /// the one that reported it. One overrun can lose any number of
    /// characters; the datasheet gives no count.
    pub overruns: u32,
}

// --- Pins ----------------------------------------------------------------

/// The `FUNCSEL` values of a TX/RX pin pair for UART `I`, worked out at
/// compile time from Table 644 ([`pinmux`]). Naming either constant with a
/// pin that cannot carry that signal is a build error.
struct UartPins<const I: usize, const TX: usize, const RX: usize>;

impl<const I: usize, const TX: usize, const RX: usize> UartPins<I, TX, RX> {
    /// `FUNCSEL` for GPIO `TX` as UART `I` TX: column F2 or F11.
    const TX_FUNCSEL: u32 = match pinmux::funcsel(TX, I, Role::UartTx) {
        // The pin carries this UART's TX in one of the columns.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this UART's TX (RP2350 datasheet Table 644)"),
    };
    /// `FUNCSEL` for GPIO `RX` as UART `I` RX: column F2 or F11.
    const RX_FUNCSEL: u32 = match pinmux::funcsel(RX, I, Role::UartRx) {
        // The pin carries this UART's RX in one of the columns.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this UART's RX (RP2350 datasheet Table 644)"),
    };
}

// --- Driver --------------------------------------------------------------

/// UART `I` (0 or 1), configured and connected to its pins.
///
/// `new` consumes the board's `DeviceHandle<Rp2350Uart<I>>` and both pin
/// handles, so in safe code at most one driver exists per UART and nothing
/// else drives its pins. Nothing here waits except [`flush`](Self::flush).
pub struct Rp2350Uart<const I: usize> {
    /// The divisor in use, for [`baud`](Self::baud). `pub(crate)` so
    /// [`crate::uart_dma`] can carry it over.
    pub(crate) divisor: BaudDivisor,
    /// Receive errors since the last [`take_line_errors`](Self::take_line_errors).
    /// `pub(crate)` so [`crate::uart_dma`] can carry the counts over.
    pub(crate) errors: LineErrors,
}

impl<const I: usize> Rp2350Uart<I> {
    /// There are two UARTs ([§12.1, p959](../../../docs/rp2350-datasheet.pdf#page=959)).
    const INSTANCE_EXISTS: () = assert!(I < 2, "RP2350 has UART0 and UART1 only");

    /// This instance's registers: `UART0_BASE` or `UART1_BASE`
    /// (§2.2.4, p32–33). `pub(crate)` for [`crate::uart_dma`].
    pub(crate) const fn regs() -> *mut Uart {
        // I is 0 or 1, checked by INSTANCE_EXISTS in `new`.
        (if I == 0 { RegAddr::UART0 } else { RegAddr::UART1 }) as usize as *mut Uart
    }

    /// Reset, configure and enable UART `I`, then connect GPIO `TX` and
    /// GPIO `RX` to it. The steps are listed in the
    /// [module documentation](self#bring-up-order).
    ///
    /// `TX` must be a pin whose F2 or F11 column in Table 644 is this
    /// UART's TX, and `RX` likewise for RX; any other pin is a **build
    /// error**. On the Pico 2: UART0 TX on GPIO0, 2, 12, 14, 16, 18, 28 and
    /// RX on GPIO1, 3, 13, 15, 17, 19, 29; UART1 TX on GPIO4, 6, 8, 10, 20,
    /// 22, 24, 26 and RX on GPIO5, 7, 9, 11, 21, 23, 25, 27.
    ///
    /// The TX pad drives with no pull; the RX pad has its pull-up on, so
    /// an unconnected RX idles high — the line's "marking state"
    /// ([§12.1.3.2.2, p963](../../../docs/rp2350-datasheet.pdf#page=963))
    /// — instead of floating and reading as breaks.
    ///
    /// `&Rp2350Clocks` is required because the divisor is computed from
    /// [`CLK_PERI_HZ`].
    pub fn new<const TX: usize, const RX: usize>(
        _handle: DeviceHandle<Self>,
        _clocks: &Rp2350Clocks,
        _tx: PinHandle<TX>,
        _rx: PinHandle<RX>,
        config: UartConfig,
    ) -> Result<Self, UartError> {
        // Evaluating these turns a wrong instance or pin into a build error.
        let () = Self::INSTANCE_EXISTS;
        // FUNCSEL for each pin, or a build error naming the bad pin.
        let tx_funcsel = UartPins::<I, TX, RX>::TX_FUNCSEL;
        let rx_funcsel = UartPins::<I, TX, RX>::RX_FUNCSEL;
        // Check the rate before touching any register.
        let Some(divisor) = BaudDivisor::for_baud(CLK_PERI_HZ, config.baud) else {
            return Err(UartError::BaudOutOfRange { baud: config.baud });
        };
        // Register block of this UART.
        let regs = Self::regs();
        unsafe {
            // Pins are routed through IO_BANK0/PADS_BANK0: make sure they run.
            pinmux::release_io_banks();
            // Step 1: full reset of this UART (Table 534, p504).
            cycle_reset(1 << (RESET_UART0_BIT + I));
            // Step 2: divisors (Tables 1032–1033, p973–974).
            (&raw mut (*regs).ibrd).write_volatile(divisor.ibrd());
            (&raw mut (*regs).fbrd).write_volatile(divisor.fbrd());
            // Step 3: format + FIFOs; also latches the divisors (p970).
            (&raw mut (*regs).lcr_h).write_volatile(config.lcr_h());
            // Step 4: enable UART, TX and RX; CTSEN/RTSEN/LBE stay 0
            // (Table 1035, p975–976).
            (&raw mut (*regs).cr).write_volatile(CR_UARTEN | CR_TXE | CR_RXE);
            // Step 5a: TX pin, no pull: the UART drives it.
            pinmux::connect(TX, tx_funcsel, PadConfig { pull_up: false, slew_fast: false });
            // Step 5b: RX pin, pulled up to the idle level.
            pinmux::connect(RX, rx_funcsel, PadConfig { pull_up: true, slew_fast: false });
        }
        // Start with no errors counted.
        Ok(Self { divisor, errors: LineErrors::default() })
    }

    /// The baud rate actually produced, to the nearest baud. For 9600 it
    /// is exact; for 115 200 it is 115 207 (0.006 % fast).
    pub fn baud(&self) -> u32 {
        // Recomputed from the divisor in the registers.
        self.divisor.baud(CLK_PERI_HZ)
    }

    /// Move received characters into `buf`, up to `buf.len()`; returns how
    /// many. 0 means nothing is waiting. Never blocks.
    ///
    /// Characters with a framing, parity or break error are dropped and
    /// counted; see [Receive errors](self#receive-errors).
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        // How many good bytes are in `buf` so far.
        let mut n = 0;
        // Stop when `buf` is full.
        while n < buf.len() {
            // One character, or stop if the FIFO is empty.
            let Some(entry) = self.pop() else { break };
            // Keep it only if it is data; errors were counted by `classify`.
            if let Ok(byte) = self.classify(entry) {
                buf[n] = byte;
                n += 1;
            }
        }
        // Good bytes stored.
        n
    }

    /// Queue bytes for transmission; returns how many fit in the transmit
    /// FIFO, which may be fewer than `data.len()`, or 0. Never blocks.
    pub fn write(&mut self, data: &[u8]) -> usize {
        // Register block of this UART.
        let regs = Self::regs();
        // How many bytes have been pushed.
        let mut n = 0;
        for &byte in data {
            // UARTFR.TXFF set: the 32-byte FIFO is full (Table 1030, p973).
            if unsafe { (&raw const (*regs).fr).read_volatile() } & FR_TXFF != 0 {
                break;
            }
            // A write to UARTDR pushes the TX FIFO (Table 1028, p972).
            unsafe { (&raw mut (*regs).dr).write_volatile(byte as u32) };
            n += 1;
        }
        // Bytes accepted.
        n
    }

    /// `true` once every queued byte, stop bits included, has left the
    /// pin: `UARTFR.BUSY` is clear ([Table 1030, p973](../../../docs/rp2350-datasheet.pdf#page=973)).
    pub fn is_idle(&self) -> bool {
        // BUSY "remains set until the complete byte, including all the
        // stop bits, has been sent from the shift register".
        unsafe { (&raw const (*Self::regs()).fr).read_volatile() & FR_BUSY == 0 }
    }

    /// Wait until [`is_idle`](Self::is_idle). Flow control is off, so the
    /// wait ends after at most 33 characters (32 in the FIFO plus one in
    /// the shift register): 34 ms at 9600 baud 8N1.
    pub fn flush(&mut self) {
        // Spin on UARTFR.BUSY; read_volatile so the load is repeated.
        while !self.is_idle() {}
    }

    /// The receive errors counted since the last call, and reset the
    /// counts to zero.
    pub fn take_line_errors(&mut self) -> LineErrors {
        // Swap the counts out for zeros.
        core::mem::take(&mut self.errors)
    }

    /// Pop one entry, 12 bits wide, from the receive FIFO, or `None` if it
    /// is empty (`UARTFR.RXFE`, Table 1030, p973).
    fn pop(&mut self) -> Option<u32> {
        // Register block of this UART.
        let regs = Self::regs();
        // RXFE set: nothing to read.
        if unsafe { (&raw const (*regs).fr).read_volatile() } & FR_RXFE != 0 {
            return None;
        }
        // Reading UARTDR pops data and its four status bits (Table 1024, p964).
        Some(unsafe { (&raw const (*regs).dr).read_volatile() })
    }

    /// Sort one FIFO entry into data or an error, counting errors.
    fn classify(&mut self, entry: u32) -> Result<u8, UartError> {
        // Shared with the DMA driver, which sees the same 12-bit entries.
        classify(entry, &mut self.errors)
    }
}

/// Sort one receive-FIFO entry (Table 1024, p964) into a character or an
/// error, counting errors in `errors`. An overrun is counted but its
/// character is data (§12.1.3.2.4, p963). Used by both drivers.
pub(crate) fn classify(entry: u32, errors: &mut LineErrors) -> Result<u8, UartError> {
    // OE: characters were lost before this one; this one is good.
    if entry & DR_OE != 0 {
        errors.overruns = errors.overruns.saturating_add(1);
    }
    // BE first: a break also fails the stop-bit check, so FE is
    // likely set with it, and the break is the more useful report.
    if entry & DR_BE != 0 {
        errors.breaks = errors.breaks.saturating_add(1);
        Err(UartError::Break)
    } else if entry & DR_FE != 0 {
        // FE: no valid stop bit.
        errors.framing = errors.framing.saturating_add(1);
        Err(UartError::Framing)
    } else if entry & DR_PE != 0 {
        // PE: parity mismatch.
        errors.parity = errors.parity.saturating_add(1);
        Err(UartError::Parity)
    } else {
        // Bits 7:0 are the character.
        Ok((entry & DR_DATA) as u8)
    }
}

/// Byte-at-a-time access through the portable traits. As with the USB
/// driver, `uart.read(&mut buf)` and `uart.write(&data)` name the inherent
/// slice methods; the trait methods are reached as `Read::read(&mut uart)`
/// or through a generic bound.
impl<const I: usize> ErrorType for Rp2350Uart<I> {
    type Error = UartError;
}

/// Classifies each [`UartError`] in the chip-independent vocabulary, so
/// code generic over a byte stream can tell "nothing yet" from a fault.
impl api::error::Error for UartError {
    fn kind(&self) -> api::error::ErrorKind {
        // Shorter name for the variants.
        use api::error::ErrorKind;
        match self {
            // The configuration asked for was not usable.
            UartError::BaudOutOfRange { .. } => ErrorKind::InvalidInput,
            // Empty receive FIFO or full transmit FIFO: try again later.
            UartError::WouldBlock => ErrorKind::WouldBlock,
            // A character arrived damaged and was discarded.
            UartError::Framing | UartError::Parity => ErrorKind::InvalidData,
            // The line itself misbehaved: held low, or a DMA bus fault.
            UartError::Break | UartError::DmaError => ErrorKind::Hardware,
            // Characters arrived faster than they were read and were lost.
            UartError::RingOverrun => ErrorKind::Overrun,
        }
    }
}

impl<const I: usize> Read<u8> for Rp2350Uart<I> {
    /// One received character; [`UartError::WouldBlock`] if none is
    /// waiting; or the error that character carried.
    fn read(&mut self) -> Result<u8, UartError> {
        // Empty FIFO: try again later.
        let entry = self.pop().ok_or(UartError::WouldBlock)?;
        // Data, or the character's error.
        self.classify(entry)
    }
}

impl<const I: usize> Write<u8> for Rp2350Uart<I> {
    /// Queue one byte; [`UartError::WouldBlock`] if the transmit FIFO is
    /// full.
    fn write(&mut self, byte: u8) -> Result<(), UartError> {
        // The slice method returns how many bytes it queued: 0 or 1 here.
        if Rp2350Uart::write(self, &[byte]) == 1 {
            Ok(())
        } else {
            Err(UartError::WouldBlock)
        }
    }
}
