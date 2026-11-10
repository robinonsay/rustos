//! The two PL022 SPI controllers, as polled SPI bus controllers moving
//! 8-bit words in Motorola SPI format.
//!
//! # Using it
//!
//! ```ignore
//! use api::common::Write;
//! use api::gpio::Gpio;
//! use api::spi::{Mode, SpiBus};
//! use pico2::spi::{Rp2350Spi, SpiConfig};
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let mut gpio = Rp2350Gpio::new(board.gpio);
//! // SD card on SPI0: RX (MISO) GPIO16, SCK GPIO18, TX (MOSI) GPIO19;
//! // chip-select is an ordinary GPIO output on GPIO17.
//! let mut spi = Rp2350Spi::new(board.spi0, &clocks, board.pins.gpio16, board.pins.gpio18,
//!                              board.pins.gpio19, SpiConfig { frequency_hz: 400_000, mode: Mode::Mode0 }).unwrap();
//! let mut cs = gpio.output_from_handle(board.pins.gpio17).unwrap();
//! cs.write(true).unwrap();                 // deselected
//! cs.write(false).unwrap();                // select
//! spi.write(&[0x40, 0, 0, 0, 0, 0x95]).unwrap(); // CMD0
//! cs.write(true).unwrap();                 // deselect
//! spi.set_config(SpiConfig { frequency_hz: 25_000_000, mode: Mode::Mode0 }).unwrap();
//! ```
//!
//! # The block
//!
//! Each controller is an Arm PrimeCell PL022 synchronous serial port
//! ([§12.3, p1044](../../../docs/rp2350-datasheet.pdf#page=1044)) with
//! 16-bit-wide, 8-entry transmit and receive FIFOs
//! ([§12.3.3.4–12.3.3.5, p1046–1047](../../../docs/rp2350-datasheet.pdf#page=1046)).
//! Its bit clock is divided from `SSPCLK` = `clk_peri`; the register
//! interface runs from `PCLK` = `clk_sys` (note in §12.3.3.3, p1046), both
//! 150 MHz in [`crate::clocks`]. `SSPCLK` must not be faster than `PCLK`
//! ([§12.3.4.4, p1048](../../../docs/rp2350-datasheet.pdf#page=1048)).
//!
//! Chip-select is **not** driven by this driver. In Motorola mode with
//! `SPH` = 0 the PL022's own `CSn` output pulses "HIGH between each data
//! word transfer"
//! ([§12.3.4.10, p1052](../../../docs/rp2350-datasheet.pdf#page=1052)),
//! which devices like SD cards, which need chip-select held low across a
//! whole command, do not accept; drive chip-select with a GPIO output from
//! [`crate::gpio`] instead.
//!
//! # Bring-up
//!
//! [`Rp2350Spi::new`] follows "Configuring the SSP" and "Enable PrimeCell
//! SSP operation"
//! ([§12.3.4.2–12.3.4.3, p1048](../../../docs/rp2350-datasheet.pdf#page=1048)):
//!
//! 1. Reset the block (`RESETS` bit 18 or 19,
//!    [Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)): it
//!    comes out disabled with empty FIFOs.
//! 2. `SSPCR0`: Motorola frame format, 8-bit words, the clock mode and the
//!    serial clock rate `SCR`; `SSPCPSR`: the prescale divisor.
//! 3. `SSPCR1`: controller ("master") mode, loopback off, `SSE` = 1.
//! 4. Connect SCK, TX and RX
//!    (`common::pinmux`). From `SSE` = 1 on, the idle clock level
//!    is set by `SPO`, so the clock pin starts at its idle level.
//!
//! Interrupts and DMA stay off: `SSPIMSC` and `SSPDMACR` keep their reset
//! value 0
//! ([Table 1104, p1061](../../../docs/rp2350-datasheet.pdf#page=1061);
//! [Table 1108, p1062](../../../docs/rp2350-datasheet.pdf#page=1062)).
//!
//! # Bit rate
//!
//! "The clock is first divided by an even prescale value CPSDVSR in the
//! range 2-254 ... The clock is divided again by a value in the range
//! 1-256, that is 1 + SCR", so the bit rate is
//! `SSPCLK / (CPSDVSR × (1 + SCR))`
//! ([§12.3.4.6.1, p1049](../../../docs/rp2350-datasheet.pdf#page=1049);
//! [Table 1099, p1059](../../../docs/rp2350-datasheet.pdf#page=1059)).
//! [`SpiClock::for_frequency`] takes the smallest prescale for which some
//! `SCR` reaches the requested rate, and the smallest such `SCR`: the
//! fastest rate not above the request. At 150 MHz that spans
//! 2.3 kHz–75 MHz.
//!
//! # Transfers
//!
//! Each byte written to `SSPDR` clocks one byte out and one byte in
//! ([Table 1101, p1060](../../../docs/rp2350-datasheet.pdf#page=1060)).
//! The driver keeps at most 8 bytes in flight, the receive FIFO's depth,
//! so a received byte always has room and the receive overrun can never
//! occur. Nothing can make a controller transfer stall — the clock is the
//! RP2350's own — so transfers have no timeout and cannot fail.
//!
//! Register accesses rely on Device-nGnRE ordering, as explained in
//! [`crate::i2c`](crate::i2c#ordering-of-register-accesses).
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer), so the `#page=`
//! links open the cited page.

use core::convert::Infallible;

use api::common::ErrorType;
use api::device::{DeviceHandle, PinHandle};
use api::spi::{Mode, SpiBus};

use crate::clocks::{CLK_PERI_HZ, CLK_SYS_HZ, Rp2350Clocks};
use crate::common::pinmux::{self, PadConfig, Role};
use crate::common::reg::RegAddr;
use crate::common::reset::cycle_reset;

// "The frequency of SSPCLK must be less than or equal to that of PCLK"
// (§12.3.4.4, p1048). SSPCLK = clk_peri, PCLK = clk_sys.
const _: () = assert!(CLK_PERI_HZ <= CLK_SYS_HZ);

// --- Register layout -----------------------------------------------------

/// The PL022 registers this driver uses, at the offsets of
/// [Table 1098, p1058–1059](../../../docs/rp2350-datasheet.pdf#page=1058).
#[repr(C)]
struct Spi {
    /// `0x000` `SSPCR0` (Table 1099, p1059): `SCR` 15:8, `SPH` 7, `SPO` 6,
    /// `FRF` 5:4, `DSS` 3:0.
    cr0: u32,
    /// `0x004` `SSPCR1` (Table 1100, p1060): `SOD` 3, `MS` 2, `SSE` 1,
    /// `LBM` 0.
    cr1: u32,
    /// `0x008` `SSPDR` (Table 1101, p1060): write pushes the TX FIFO, read
    /// pops the RX FIFO.
    dr: u32,
    /// `0x00c` `SSPSR` (Table 1102, p1060): FIFO and busy flags.
    sr: u32,
    /// `0x010` `SSPCPSR` (Table 1103, p1061): `CPSDVSR` 7:0, even, 2–254.
    cpsr: u32,
}

// The offsets above, checked against Table 1098 (p1058).
const _: () = assert!(core::mem::offset_of!(Spi, dr) == 0x008);
const _: () = assert!(core::mem::offset_of!(Spi, sr) == 0x00c);
const _: () = assert!(core::mem::offset_of!(Spi, cpsr) == 0x010);

/// `SSPCR0.DSS` = `0b0111`: 8-bit data (Table 1099, p1059).
const CR0_DSS_8BIT: u32 = 0b0111;
/// `SSPCR0.FRF` = `0b00`, bits 5:4: Motorola SPI frame format (Table
/// 1099, p1059).
const CR0_FRF_MOTOROLA: u32 = 0b00 << 4;
/// `SSPCR0.SPO`, bit 6: clock idles high (Table 1099, p1059; §12.3.4.9.1,
/// p1051).
const CR0_SPO: u32 = 1 << 6;
/// `SSPCR0.SPH`, bit 7: capture on the second edge (Table 1099, p1059;
/// §12.3.4.9.2, p1051).
const CR0_SPH: u32 = 1 << 7;
/// `SSPCR0.SCR` starts at bit 8 (Table 1099, p1059).
const CR0_SCR_SHIFT: u32 = 8;

/// `SSPCR1.SSE`, bit 1: port enabled (Table 1100, p1060). `MS` (bit 2) = 0
/// is controller mode, "default value on reset" (§12.3.4.6, p1049).
const CR1_SSE: u32 = 1 << 1;

/// `SSPSR.TNF`, bit 1, reset 1: transmit FIFO not full (Table 1102, p1060).
const SR_TNF: u32 = 1 << 1;
/// `SSPSR.RNE`, bit 2: receive FIFO not empty (Table 1102, p1060).
const SR_RNE: u32 = 1 << 2;
/// `SSPSR.BSY`, bit 4: a frame is in progress or the TX FIFO is not
/// empty (Table 1102, p1060).
const SR_BSY: u32 = 1 << 4;

/// Depth of each FIFO: "8 locations deep" (§12.3.3.4–12.3.3.5, p1046–1047).
const FIFO_DEPTH: usize = 8;

/// Byte sent when there is nothing to send: all ones, the idle level of
/// the data line on most devices (an SD card in SPI mode expects it).
const FILL_BYTE: u8 = 0xff;

/// `SPI0` is bit 18 of `RESETS.RESET`, `SPI1` bit 19
/// ([Table 534, p504](../../../docs/rp2350-datasheet.pdf#page=504)).
const RESET_SPI0_BIT: usize = 18;

// --- Bit rate ------------------------------------------------------------

/// Smallest prescale divisor, `CPSDVSR` (Table 1103, p1061).
const CPSDVSR_MIN: u32 = 2;
/// Largest prescale divisor, `CPSDVSR` (Table 1103, p1061).
const CPSDVSR_MAX: u32 = 254;
/// Largest second divisor, `1 + SCR` with `SCR` = 255 (Table 1099, p1059).
const SCR_PLUS_ONE_MAX: u32 = 256;

/// Fastest bit rate: `clk_peri / 2`, "the ratio ... for the master mode,
/// it is two" (§12.3.4.4, p1048). 75 MHz.
pub const MAX_FREQUENCY_HZ: u32 = CLK_PERI_HZ / CPSDVSR_MIN;
/// Slowest bit rate, rounded up: `clk_peri / (254 × 256)`, about 2.3 kHz
/// (§12.3.4.4, p1048).
pub const MIN_FREQUENCY_HZ: u32 = CLK_PERI_HZ.div_ceil(CPSDVSR_MAX * SCR_PLUS_ONE_MAX);

/// The two clock divisors for one bit rate. See [Bit rate](self#bit-rate).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpiClock {
    /// `SSPCPSR.CPSDVSR`: even, 2–254.
    cpsdvsr: u32,
    /// `SSPCR0.SCR`: 0–255.
    scr: u32,
}

impl SpiClock {
    /// The divisors for the fastest rate not above `hz` with `SSPCLK` =
    /// `clk_hz`, or `None` if `hz` is outside
    /// [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`] (for this crate's
    /// `clk_peri`).
    pub const fn for_frequency(clk_hz: u32, hz: u32) -> Option<Self> {
        // Faster than clk / 2 cannot be produced (§12.3.4.4, p1048).
        if hz == 0 || hz > clk_hz / CPSDVSR_MIN {
            return None;
        }
        // Try each even prescale from the smallest (Table 1103, p1061).
        let mut cpsdvsr = CPSDVSR_MIN;
        while cpsdvsr <= CPSDVSR_MAX {
            // Smallest 1 + SCR giving a rate ≤ hz with this prescale.
            let scr_plus_one = clk_hz.div_ceil(cpsdvsr * hz);
            // It fits the 8-bit SCR field: done.
            if scr_plus_one <= SCR_PLUS_ONE_MAX {
                return Some(Self { cpsdvsr, scr: scr_plus_one - 1 });
            }
            // Next even prescale.
            cpsdvsr += 2;
        }
        // Too slow even with both divisors at their maximum.
        None
    }

    /// The bit rate produced, rounded down: `SSPCLK / (CPSDVSR × (1 +
    /// SCR))` (§12.3.4.6.1, p1049).
    pub const fn frequency_hz(&self, clk_hz: u32) -> u32 {
        // The datasheet's formula.
        clk_hz / (self.cpsdvsr * (1 + self.scr))
    }
}

// The datasheet's example (§12.3.4.6.1, p1049): SSPCLK 125 MHz with
// CPSDVSR = 2 spans 244 kHz (SCR 255) to 62.5 MHz (SCR 0).
const _: () = {
    // Fastest: CPSDVSR 2, SCR 0.
    let Some(c) = SpiClock::for_frequency(125_000_000, 62_500_000) else { panic!() };
    assert!(c.cpsdvsr == 2 && c.scr == 0 && c.frequency_hz(125_000_000) == 62_500_000);
    // Slowest with CPSDVSR 2: SCR 255, 244 kHz.
    let Some(c) = SpiClock::for_frequency(125_000_000, 244_141) else { panic!() };
    assert!(c.cpsdvsr == 2 && c.scr == 255 && c.frequency_hz(125_000_000) == 244_140);
};
// SD card rates at this crate's clk_peri: never faster than asked.
const _: () = {
    // 400 kHz initialisation clock: 150 MHz / (2 × 188) = 398.9 kHz.
    let Some(c) = SpiClock::for_frequency(CLK_PERI_HZ, 400_000) else { panic!() };
    assert!(c.frequency_hz(CLK_PERI_HZ) <= 400_000 && c.frequency_hz(CLK_PERI_HZ) > 395_000);
    // 25 MHz data clock: 150 MHz / (2 × 3) = 25 MHz exactly.
    let Some(c) = SpiClock::for_frequency(CLK_PERI_HZ, 25_000_000) else { panic!() };
    assert!(c.frequency_hz(CLK_PERI_HZ) == 25_000_000);
};
// Both ends of the accepted range work; one step past either does not.
const _: () = assert!(SpiClock::for_frequency(CLK_PERI_HZ, MAX_FREQUENCY_HZ).is_some());
const _: () = assert!(SpiClock::for_frequency(CLK_PERI_HZ, MIN_FREQUENCY_HZ).is_some());
const _: () = assert!(SpiClock::for_frequency(CLK_PERI_HZ, MAX_FREQUENCY_HZ + 1).is_none());
const _: () = assert!(SpiClock::for_frequency(CLK_PERI_HZ, MIN_FREQUENCY_HZ - 1).is_none());

// --- Configuration and errors --------------------------------------------

/// Bus settings for [`Rp2350Spi::new`] and [`Rp2350Spi::set_config`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpiConfig {
    /// Requested bit rate; the fastest rate not above it is used.
    pub frequency_hz: u32,
    /// Clock polarity and phase.
    pub mode: Mode,
}

impl SpiConfig {
    /// The `SSPCR0` value: Motorola format, 8-bit words, this mode, and
    /// `SCR` from `clock` (Table 1099, p1059).
    const fn cr0(&self, clock: &SpiClock) -> u32 {
        // SPO: clock idles high in modes 2 and 3.
        let spo = if self.mode.idle_high() { CR0_SPO } else { 0 };
        // SPH: capture on the second edge in modes 1 and 3.
        let sph = if self.mode.capture_on_second_edge() { CR0_SPH } else { 0 };
        // All fields of SSPCR0 in one value.
        (clock.scr << CR0_SCR_SHIFT) | sph | spo | CR0_FRF_MOTOROLA | CR0_DSS_8BIT
    }
}

/// Why [`Rp2350Spi::new`] or [`Rp2350Spi::set_config`] refused a
/// configuration. Nothing is changed when either returns this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiError {
    /// The bit rate is outside [`MIN_FREQUENCY_HZ`]–[`MAX_FREQUENCY_HZ`].
    FrequencyOutOfRange {
        /// The rate requested.
        hz: u32,
    },
}

// --- Pins ----------------------------------------------------------------

/// The `FUNCSEL` values of an RX/SCK/TX pin set for SPI `I`, worked out at
/// compile time from Table 644 ([`pinmux`]).
struct SpiPins<const I: usize, const RX: usize, const SCK: usize, const TX: usize>;

impl<const I: usize, const RX: usize, const SCK: usize, const TX: usize> SpiPins<I, RX, SCK, TX> {
    /// `FUNCSEL` for GPIO `RX` as SPI `I` RX: column F1.
    const RX_FUNCSEL: u32 = match pinmux::funcsel(RX, I, Role::SpiRx) {
        // The pin carries this controller's RX.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this SPI controller's RX (RP2350 datasheet Table 644)"),
    };
    /// `FUNCSEL` for GPIO `SCK` as SPI `I` SCK: column F1.
    const SCK_FUNCSEL: u32 = match pinmux::funcsel(SCK, I, Role::SpiSck) {
        // The pin carries this controller's SCK.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this SPI controller's SCK (RP2350 datasheet Table 644)"),
    };
    /// `FUNCSEL` for GPIO `TX` as SPI `I` TX: column F1.
    const TX_FUNCSEL: u32 = match pinmux::funcsel(TX, I, Role::SpiTx) {
        // The pin carries this controller's TX.
        Some(f) => f,
        // Table 644 (p588–589) has no such cell: refuse to build.
        None => panic!("this GPIO cannot carry this SPI controller's TX (RP2350 datasheet Table 644)"),
    };
}

/// Pad settings for SCK and TX, the two outputs: fast edges. The datasheet
/// gives no pad recommendation for SPI; fast slew is a choice made here so
/// that edges stay sharp at the 25 MHz an SD card runs at. `DRIVE` stays
/// at its reset value, 4 mA (Table 852, p785).
const SPI_OUTPUT_PAD: PadConfig = PadConfig { pull_up: false, slew_fast: true };
/// Pad settings for RX: pulled up, so a data line that no device drives
/// (every device deselected) reads as `0xFF` instead of floating. Also a
/// choice made here, not a datasheet requirement.
const SPI_INPUT_PAD: PadConfig = PadConfig { pull_up: true, slew_fast: false };

// --- Driver --------------------------------------------------------------

/// SPI controller `I` (0 or 1), configured as bus controller and
/// connected to its RX, SCK and TX pins.
///
/// `new` consumes the board's `DeviceHandle<Rp2350Spi<I>>` and the three
/// pin handles, so in safe code at most one driver exists per controller.
pub struct Rp2350Spi<const I: usize> {
    /// The divisors in use, for [`frequency_hz`](Self::frequency_hz).
    clock: SpiClock,
}

impl<const I: usize> Rp2350Spi<I> {
    /// There are two controllers ([§12.3, p1044](../../../docs/rp2350-datasheet.pdf#page=1044)).
    const INSTANCE_EXISTS: () = assert!(I < 2, "RP2350 has SPI0 and SPI1 only");

    /// This instance's registers: `SPI0_BASE` or `SPI1_BASE` (§2.2.4,
    /// p32–33).
    const fn regs() -> *mut Spi {
        // I is 0 or 1, checked by INSTANCE_EXISTS in `new`.
        (if I == 0 { RegAddr::SPI0 } else { RegAddr::SPI1 }) as usize as *mut Spi
    }

    /// Reset, configure and enable controller `I`, then connect GPIO `RX`,
    /// `SCK` and `TX` to it. The steps are in the
    /// [module documentation](self#bring-up).
    ///
    /// Each pin must carry that signal of this controller in column F1 of
    /// Table 644; any other pin is a **build error**. On the Pico 2, SPI0:
    /// RX GPIO0, 4, 16, 20; SCK GPIO2, 6, 18, 22; TX GPIO3, 7, 19, 23.
    /// SPI1: RX GPIO8, 12, 24, 28; SCK GPIO10, 14, 26; TX GPIO11, 15, 27.
    pub fn new<const RX: usize, const SCK: usize, const TX: usize>(
        _handle: DeviceHandle<Self>,
        _clocks: &Rp2350Clocks,
        _rx: PinHandle<RX>,
        _sck: PinHandle<SCK>,
        _tx: PinHandle<TX>,
        config: SpiConfig,
    ) -> Result<Self, SpiError> {
        // Evaluating these turns a wrong instance or pin into a build error.
        let () = Self::INSTANCE_EXISTS;
        // FUNCSEL for each pin, or a build error naming the bad pin.
        let rx_funcsel = SpiPins::<I, RX, SCK, TX>::RX_FUNCSEL;
        let sck_funcsel = SpiPins::<I, RX, SCK, TX>::SCK_FUNCSEL;
        let tx_funcsel = SpiPins::<I, RX, SCK, TX>::TX_FUNCSEL;
        // Check the rate before touching any register.
        let Some(clock) = SpiClock::for_frequency(CLK_PERI_HZ, config.frequency_hz) else {
            return Err(SpiError::FrequencyOutOfRange { hz: config.frequency_hz });
        };
        // Register block of this controller.
        let regs = Self::regs();
        unsafe {
            // Pins are routed through IO_BANK0/PADS_BANK0: make sure they run.
            pinmux::release_io_banks();
            // Step 1: full reset; comes out disabled (§12.3.4.2, p1048).
            cycle_reset(1 << (RESET_SPI0_BIT + I));
            // Step 2: format, word size, mode and SCR (Table 1099, p1059).
            (&raw mut (*regs).cr0).write_volatile(config.cr0(&clock));
            // Step 2: prescale divisor (Table 1103, p1061).
            (&raw mut (*regs).cpsr).write_volatile(clock.cpsdvsr);
            // Step 3: controller mode (MS = 0), no loopback, enabled (Table 1100, p1060).
            (&raw mut (*regs).cr1).write_volatile(CR1_SSE);
            // Step 4: pins, outputs first so SCK settles at its idle level.
            pinmux::connect(SCK, sck_funcsel, SPI_OUTPUT_PAD);
            pinmux::connect(TX, tx_funcsel, SPI_OUTPUT_PAD);
            pinmux::connect(RX, rx_funcsel, SPI_INPUT_PAD);
        }
        // Configured and enabled.
        Ok(Self { clock })
    }

    /// The bit rate in use, rounded down.
    pub fn frequency_hz(&self) -> u32 {
        // From the programmed divisors.
        self.clock.frequency_hz(CLK_PERI_HZ)
    }

    /// Change the bit rate and mode, for example from an SD card's
    /// 400 kHz initialisation clock to its 25 MHz data clock. Waits for
    /// any frame in progress (none, after a transfer has returned), then
    /// disables the port, writes the new settings and enables it again, as
    /// §12.3.4.2 (p1048) requires configuration to be done while disabled.
    /// On error nothing is changed.
    pub fn set_config(&mut self, config: SpiConfig) -> Result<(), SpiError> {
        // Check the rate before touching any register.
        let Some(clock) = SpiClock::for_frequency(CLK_PERI_HZ, config.frequency_hz) else {
            return Err(SpiError::FrequencyOutOfRange { hz: config.frequency_hz });
        };
        // Register block of this controller.
        let regs = Self::regs();
        unsafe {
            // Wait until idle: SSPSR.BSY clear (Table 1102, p1060).
            while (&raw const (*regs).sr).read_volatile() & SR_BSY != 0 {}
            // Disable: SSE = 0 (Table 1100, p1060).
            (&raw mut (*regs).cr1).write_volatile(0);
            // New SSPCR0 and SSPCPSR (Tables 1099 and 1103).
            (&raw mut (*regs).cr0).write_volatile(config.cr0(&clock));
            (&raw mut (*regs).cpsr).write_volatile(clock.cpsdvsr);
            // Enable again.
            (&raw mut (*regs).cr1).write_volatile(CR1_SSE);
        }
        // Remember the new divisors.
        self.clock = clock;
        Ok(())
    }

    /// Discard anything left in the receive FIFO, so that the first byte
    /// read by a transfer is the first byte it clocked in. After a
    /// completed transfer there is nothing to discard.
    fn drain_rx(&mut self) {
        // Register block of this controller.
        let regs = Self::regs();
        unsafe {
            // Pop SSPDR while SSPSR.RNE is set (Tables 1101–1102, p1060).
            while (&raw const (*regs).sr).read_volatile() & SR_RNE != 0 {
                let _ = (&raw const (*regs).dr).read_volatile();
            }
        }
    }
}

impl<const I: usize> ErrorType for Rp2350Spi<I> {
    /// A transfer cannot fail: see [Transfers](self#transfers).
    type Error = Infallible;
}

impl<const I: usize> SpiBus for Rp2350Spi<I> {
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Infallible> {
        // Bytes to clock: the longer of the two buffers.
        let n = read.len().max(write.len());
        // Register block of this controller.
        let regs = Self::regs();
        // Start with an empty receive FIFO.
        self.drain_rx();
        // Bytes pushed into the TX FIFO, and bytes popped from the RX FIFO.
        let (mut sent, mut received) = (0, 0);
        // Done when every clocked-in byte has been collected.
        while received < n {
            // One read of SSPSR per pass (Table 1102, p1060).
            let status = unsafe { (&raw const (*regs).sr).read_volatile() };
            // Push the next byte if the TX FIFO has room and the RX FIFO
            // will have room for the byte it brings back.
            if sent < n && status & SR_TNF != 0 && sent - received < FIFO_DEPTH {
                // The caller's byte, or the fill byte past its end.
                let byte = if sent < write.len() { write[sent] } else { FILL_BYTE };
                // Writing SSPDR starts the frame (Table 1101, p1060).
                unsafe { (&raw mut (*regs).dr).write_volatile(byte as u32) };
                sent += 1;
            }
            // Pop a received byte if one is waiting.
            if status & SR_RNE != 0 {
                // 8-bit frames are right-justified in SSPDR (Table 1101).
                let byte = unsafe { (&raw const (*regs).dr).read_volatile() } as u8;
                // Keep it if the caller has room; past the end, discard.
                if received < read.len() {
                    read[received] = byte;
                }
                received += 1;
            }
        }
        // Every frame's last bit has been received, so the bus is idle.
        Ok(())
    }

    fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Infallible> {
        // Same loop as `transfer`, with one buffer as source and sink.
        // `sent` never trails `received`, so each byte is sent before the
        // byte received in its place overwrites it.
        let n = buffer.len();
        // Register block of this controller.
        let regs = Self::regs();
        // Start with an empty receive FIFO.
        self.drain_rx();
        // Bytes pushed into the TX FIFO, and bytes popped from the RX FIFO.
        let (mut sent, mut received) = (0, 0);
        // Done when every clocked-in byte has been stored.
        while received < n {
            // One read of SSPSR per pass (Table 1102, p1060).
            let status = unsafe { (&raw const (*regs).sr).read_volatile() };
            // Push the next byte if both FIFOs have room for it.
            if sent < n && status & SR_TNF != 0 && sent - received < FIFO_DEPTH {
                // Send the caller's byte (Table 1101, p1060).
                unsafe { (&raw mut (*regs).dr).write_volatile(buffer[sent] as u32) };
                sent += 1;
            }
            // Pop a received byte into the slot of the byte that produced it.
            if status & SR_RNE != 0 {
                buffer[received] = unsafe { (&raw const (*regs).dr).read_volatile() } as u8;
                received += 1;
            }
        }
        // All frames complete.
        Ok(())
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        // Send `bytes`, keep nothing.
        self.transfer(&mut [], bytes)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Infallible> {
        // Send fill bytes, keep what comes back.
        self.transfer(buffer, &[])
    }
}
