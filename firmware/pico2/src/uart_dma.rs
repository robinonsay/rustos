//! A UART whose FIFOs are serviced by two DMA channels: received
//! characters land in a RAM ring without the processor, and queued bytes
//! are fed to the transmitter without it.
//!
//! # Using it
//!
//! ```ignore
//! use api::common::{Read, Write};
//! use pico2::uart::{Rp2350Uart, UartConfig};
//! use pico2::uart_dma::{UartRxRing, UartTxRing};
//!
//! static mut RX: UartRxRing = UartRxRing::new(); // 4 kB in .bss
//! static mut TX: UartTxRing = UartTxRing::new(); // 1 kB in .bss
//!
//! let board = Rp2350::take().unwrap();
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let dma = Rp2350Dma::new(board.dma);
//! let uart = Rp2350Uart::new(board.uart1, &clocks, board.pins.gpio4, board.pins.gpio5,
//!                            UartConfig::new_8n1(9600)).unwrap();
//! let (rx, tx) = unsafe { (&mut *(&raw mut RX), &mut *(&raw mut TX)) }; // the only references ever made
//! let mut gps = uart.into_dma(dma.ch1, dma.ch2, rx, tx);
//! let mut buf = [0u8; 128];
//! loop {
//!     let n = gps.read(&mut buf)?;   // everything received since last time, up to 128
//!     gps.write(b"$PMTK...\r\n")?;   // queued; the DMA sends it
//!     gps.poll()?;                   // starts any bytes queued while the DMA was busy
//! }
//! ```
//!
//! The polled [`Rp2350Uart`] is converted, not wrapped: `into_dma`
//! consumes it, because once a DMA channel services a FIFO "the user must
//! not access a FIFO currently being serviced by the DMA. This causes the
//! channel and peripheral to become desynchronised"
//! ([§12.6.4.2, p1100](../../../docs/rp2350-datasheet.pdf#page=1100)).
//!
//! # Receive: an endless channel into a ring
//!
//! The RX channel reads `UARTDR` as a **halfword** — a 16-bit read of an
//! IO register is a 32-bit read that returns bits 15:0
//! ([§2.1.5, p28](../../../docs/rp2350-datasheet.pdf#page=28)) — so each
//! ring entry keeps the character *and* its four error bits
//! ([Table 1024, p964](../../../docs/rp2350-datasheet.pdf#page=964)), and
//! [`read`](Rp2350UartDma::read) sorts them exactly as the polled driver
//! does. The channel is paced by `DREQ_UARTn_RX`
//! ([Table 1145, p1099](../../../docs/rp2350-datasheet.pdf#page=1099)),
//! runs `ENDLESS` (the count never decrements and the channel never stops,
//! [Table 1149, p1124](../../../docs/rp2350-datasheet.pdf#page=1124)), and
//! wraps its write address on the ring's size (`RING_SEL` = write,
//! `RING_SIZE` = 12, [Table 1150, p1126](../../../docs/rp2350-datasheet.pdf#page=1126)),
//! which needs the ring aligned to its size: [`UartRxRing`]'s type does
//! that.
//!
//! This is the scheme [`crate::adc`] uses, and so is overrun detection: an
//! endless channel cannot count its laps, so every slot holds the sentinel
//! `0xffff` until a character is written into it, and is reset to it once
//! read. `UARTDR` never reads `0xffff`: bits 31:12 are reserved (Table 1028,
//! [p971](../../../docs/rp2350-datasheet.pdf#page=971)). (That reserved
//! bits read as 0 is assumed here, as in [`crate::adc`]; the table gives
//! no reset value for them.) If the slot [`RX_GUARD_ENTRIES`] ahead of the
//! channel does not hold the sentinel, reading has fallen a whole ring
//! behind: [`UartError::RingOverrun`]. At 115 200 baud the ring holds
//! 178 ms; at 9600 baud, 2.1 s.
//!
//! # Transmit: a ring the channel drains
//!
//! [`write`](Rp2350UartDma::write) copies bytes into [`UartTxRing`] and,
//! if the TX channel is idle, starts it on everything queued: `NORMAL`
//! mode with the count of queued bytes, byte transfers, read address
//! incrementing and wrapping on the ring (`RING_SEL` = read, `RING_SIZE` =
//! 10), write address fixed at `UARTDR`, paced by `DREQ_UARTn_TX`. A byte
//! write to an IO register is replicated across all four byte lanes
//! (§2.1.5, p28), so `UARTDR` bits 7:0 get the byte.
//!
//! While the channel runs, its live `TRANS_COUNT` says how many bytes are
//! still to go (Table 1149, p1124), so ring space frees up as bytes leave
//! and `write` can keep queuing. But a running channel cannot be extended,
//! so bytes queued while it runs are started by the next call to
//! [`write`](Rp2350UartDma::write), [`poll`](Rp2350UartDma::poll) or
//! [`flush`](Rp2350UartDma::flush) after it finishes. Call one of them
//! regularly, at least once per transmit-ring's worth of time (89 ms at
//! 115 200 baud).
//!
//! # Memory ordering
//!
//! The rings are ordinary RAM that the DMA reads and writes behind the
//! compiler's back, so every access goes through raw pointers with
//! volatile reads and writes, and no reference to a ring is kept. The
//! processor must also not reorder RAM accesses across the DMA register
//! accesses that hand a slot over: Device-nGnRE accesses are not ordered
//! against Normal-memory accesses (rule RNDHC,
//! [Armv8-M ARM §B7.12.2, p239](../../../docs/DDI0553B_z_armv8m_arm.pdf#page=239)),
//! so a [`fence`] (a `DMB` on this core) sits between them: after reading
//! the RX channel's `WRITE_ADDR` and before reading the ring, and after
//! filling the TX ring and before triggering the channel.
//!
//! # Bring-up requirements
//!
//! * A configured [`Rp2350Uart`] (so the UART, its clock and its pins are
//!   up) and two [`Rp2350DmaChannel`]s from [`crate::dma`], which reset the
//!   DMA block.
//! * The bus lets the DMA reach the UARTs out of reset: `ACCESSCTRL.UART0`
//!   and `UART1` reset with `DMA` = 1 and Secure-Privileged access
//!   allowed ([Tables 951–952, p853–854](../../../docs/rp2350-datasheet.pdf#page=853)),
//!   which is the level DMA channels run at out of reset (see
//!   [`crate::dma`]), and SRAM is open to the DMA (Table 918, p831).
//! * `UARTDMACR`: `RXDMAE` and `TXDMAE` set, `DMAONERR` clear, so erroneous
//!   characters still reach the ring with their error bits
//!   ([Table 1041, p979](../../../docs/rp2350-datasheet.pdf#page=979)).
//!
//! No interrupt is enabled anywhere; nothing here waits except
//! [`flush`](Rp2350UartDma::flush).
//!
//! **Untested on hardware.** In particular, characters already in the
//! receive FIFO when `into_dma` runs are moved into the ring by the
//! processor first; whether a character arriving in the few cycles between
//! that and `RXDMAE` being set is still requested by the UART is not stated
//! in the datasheet.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer), so the `#page=`
//! links open the cited page.

use core::sync::atomic::{Ordering, fence};

use api::common::{ErrorType, Read, Write};

use crate::clocks::CLK_PERI_HZ;
use crate::dma::{
    self, CTRL_AHB_ERROR, CTRL_BUSY, CTRL_CHAIN_TO_SHIFT, CTRL_DATA_SIZE_SHIFT, CTRL_EN,
    CTRL_INCR_READ, CTRL_INCR_WRITE, CTRL_RING_SEL_READ, CTRL_RING_SEL_WRITE, CTRL_RING_SIZE_SHIFT,
    CTRL_TREQ_SEL_SHIFT, DATA_SIZE_BYTE, DATA_SIZE_HALFWORD, MODE_ENDLESS, MODE_NORMAL,
    Rp2350DmaChannel, TRANS_COUNT_MODE_SHIFT, dreq_uart_rx, dreq_uart_tx,
};
use crate::uart::{self, BaudDivisor, FR_BUSY, FR_RXFE, LineErrors, Rp2350Uart, UartError};

// --- Rings ---------------------------------------------------------------

/// Entries in a [`UartRxRing`]: one per received character.
pub const RX_RING_ENTRIES: usize = 2048;
/// `log2` of a [`UartRxRing`]'s size in bytes: the RX channel's `RING_SIZE`.
const RX_RING_SIZE_LOG2: u32 = 12;
/// Bytes in a [`UartTxRing`].
pub const TX_RING_BYTES: usize = 1024;
/// `log2` of a [`UartTxRing`]'s size in bytes: the TX channel's `RING_SIZE`.
const TX_RING_SIZE_LOG2: u32 = 10;

/// How far ahead of the RX channel [`Rp2350UartDma::read`] looks for the
/// sentinel, and how many entries an overrun's clearing leaves alone. 64
/// characters take 0.69 ms even at 921 600 baud, against a few instructions
/// between two reads, and against clearing the rest of the ring, about
/// 2000 stores (roughly 15–30 µs at 150 MHz).
pub const RX_GUARD_ENTRIES: usize = 64;

/// What an RX ring slot holds when no character is waiting in it. See
/// [Receive](self#receive-an-endless-channel-into-a-ring).
const SENTINEL: u16 = 0xffff;

// Each ring is exactly its power-of-two size (Table 1150, p1126: "Ring
// sizes between 2 and 32768 bytes are possible").
const _: () = assert!(RX_RING_ENTRIES * 2 == 1 << RX_RING_SIZE_LOG2);
const _: () = assert!(TX_RING_BYTES == 1 << TX_RING_SIZE_LOG2);
const _: () = assert!(RX_RING_SIZE_LOG2 >= 1 && RX_RING_SIZE_LOG2 <= 15);
const _: () = assert!(TX_RING_SIZE_LOG2 >= 1 && TX_RING_SIZE_LOG2 <= 15);
// The guard must leave most of the ring usable.
const _: () = assert!(RX_GUARD_ENTRIES < RX_RING_ENTRIES / 2);
// No UARTDR value can equal the sentinel: bits 15:12 are reserved.
const _: () = assert!(SENTINEL & 0xf000 != 0);

/// The ring the RX channel fills: [`RX_RING_ENTRIES`] halfword entries,
/// 4 kB, aligned to its own size as the DMA's address wrap requires
/// (Table 1150, p1126).
///
/// Make one `static mut` with [`UartRxRing::new`] (zeros, so it lives in
/// `.bss`) and pass [`Rp2350Uart::into_dma`] the one `&'static mut` ever
/// made from it: the DMA writes it for the rest of the boot.
#[repr(C, align(4096))]
pub struct UartRxRing {
    /// One FIFO entry per slot: character in bits 7:0, errors in 11:8.
    entries: [u16; RX_RING_ENTRIES],
}

/// The ring the TX channel drains: [`TX_RING_BYTES`] bytes, 1 kB, aligned
/// to its own size. Made and handed over like [`UartRxRing`].
#[repr(C, align(1024))]
pub struct UartTxRing {
    /// Bytes waiting to be sent, and bytes already sent.
    bytes: [u8; TX_RING_BYTES],
}

// The alignment the wrap needs is the size itself.
const _: () = assert!(core::mem::align_of::<UartRxRing>() == 1 << RX_RING_SIZE_LOG2);
const _: () = assert!(core::mem::size_of::<UartRxRing>() == 1 << RX_RING_SIZE_LOG2);
const _: () = assert!(core::mem::align_of::<UartTxRing>() == 1 << TX_RING_SIZE_LOG2);
const _: () = assert!(core::mem::size_of::<UartTxRing>() == 1 << TX_RING_SIZE_LOG2);

impl UartRxRing {
    /// A zeroed ring; `into_dma` fills it with the sentinel before use.
    pub const fn new() -> Self {
        // All zeros: placed in .bss, no flash cost.
        Self { entries: [0; RX_RING_ENTRIES] }
    }
}

impl Default for UartRxRing {
    fn default() -> Self {
        // Same as `new`.
        Self::new()
    }
}

impl UartTxRing {
    /// A zeroed ring.
    pub const fn new() -> Self {
        // All zeros: placed in .bss, no flash cost.
        Self { bytes: [0; TX_RING_BYTES] }
    }
}

impl Default for UartTxRing {
    fn default() -> Self {
        // Same as `new`.
        Self::new()
    }
}

/// Distance from ring index `from` forward to ring index `to`, in RX
/// entries.
const fn rx_distance(from: usize, to: usize) -> usize {
    // Add a whole ring before the modulo so the subtraction cannot wrap.
    (to + RX_RING_ENTRIES - from) % RX_RING_ENTRIES
}

// Spot checks of the ring arithmetic.
const _: () = assert!(rx_distance(5, 5) == 0);
const _: () = assert!(rx_distance(RX_RING_ENTRIES - 1, 0) == 1);
const _: () = assert!(rx_distance(1, 0) == RX_RING_ENTRIES - 1);

/// `UARTDMACR.RXDMAE`, bit 0 (Table 1041, p979).
const DMACR_RXDMAE: u32 = 1 << 0;
/// `UARTDMACR.TXDMAE`, bit 1 (Table 1041, p979).
const DMACR_TXDMAE: u32 = 1 << 1;

// --- Conversion ----------------------------------------------------------

impl<const I: usize> Rp2350Uart<I> {
    /// Hand this UART's FIFOs to DMA channel `TXC` (transmit) and DMA
    /// channel `RXC` (receive), with `rx_ring` and `tx_ring` as their
    /// buffers. Line settings, pins and the receive-error counts carry
    /// over.
    ///
    /// In order: every RX slot set to the sentinel; any characters already
    /// in the receive FIFO moved into the start of the ring by the
    /// processor; the RX channel started after them; then `UARTDMACR` set,
    /// which lets the UART request transfers. The TX channel starts with
    /// the first [`write`](Rp2350UartDma::write).
    ///
    /// The channels, the rings and the polled driver are all consumed: the
    /// channels and rings are the DMA's for the rest of the boot.
    pub fn into_dma<const TXC: usize, const RXC: usize>(
        self,
        _tx_channel: Rp2350DmaChannel<TXC>,
        _rx_channel: Rp2350DmaChannel<RXC>,
        rx_ring: &'static mut UartRxRing,
        tx_ring: &'static mut UartTxRing,
    ) -> Rp2350UartDma<I> {
        // A channel number that does not exist is a build error.
        let () = Rp2350DmaChannel::<TXC>::EXISTS;
        let () = Rp2350DmaChannel::<RXC>::EXISTS;
        // From here on the rings are only reached through raw pointers.
        let rx = rx_ring.entries.as_mut_ptr();
        let tx = tx_ring.bytes.as_mut_ptr();
        // This UART's registers.
        let regs = Self::regs();
        // Ring slots filled by the processor before the channel starts.
        let mut pending = 0;
        unsafe {
            // Every slot empty (volatile: the DMA will write these).
            for i in 0..RX_RING_ENTRIES {
                rx.add(i).write_volatile(SENTINEL);
            }
            // Move characters already waiting into the ring, with their
            // error bits, while the processor may still read UARTDR: stop
            // at RXFE (Table 1030, p973) or a full ring less the guard.
            while (&raw const (*regs).fr).read_volatile() & FR_RXFE == 0
                && pending < RX_RING_ENTRIES - RX_GUARD_ENTRIES
            {
                // Bits 15:0 of UARTDR: character and error bits (Table 1028).
                let entry = (&raw const (*regs).dr).read_volatile() as u16;
                rx.add(pending).write_volatile(entry);
                pending += 1;
            }
            // Ring stores done before the channel can see the ring.
            fence(Ordering::SeqCst);
            // RX: UARTDR → ring, halfwords, endless, wrapping its writes.
            dma::start(
                RXC,
                // Read the data register, never incrementing.
                &raw const (*regs).dr as usize as u32,
                // Write from the first free slot.
                rx.add(pending) as usize as u32,
                // ENDLESS with a count of 1: a count of 0 would halt at once
                // (§12.6.2.2.1, p1096); in this mode it never decrements.
                (MODE_ENDLESS << TRANS_COUNT_MODE_SHIFT) | 1,
                CTRL_EN
                    | (DATA_SIZE_HALFWORD << CTRL_DATA_SIZE_SHIFT)
                    | CTRL_INCR_WRITE
                    | (RX_RING_SIZE_LOG2 << CTRL_RING_SIZE_SHIFT)
                    | CTRL_RING_SEL_WRITE
                    // CHAIN_TO = own number disables chaining (Table 1150, p1126).
                    | ((RXC as u32) << CTRL_CHAIN_TO_SHIFT)
                    | (dreq_uart_rx(I) << CTRL_TREQ_SEL_SHIFT),
            );
            // Let the UART raise DMA requests in both directions; DMAONERR
            // = 0 so errored characters still flow (Table 1041, p979).
            (&raw mut (*regs).dmacr).write_volatile(DMACR_RXDMAE | DMACR_TXDMAE);
        }
        // The polled driver's state carries over; the DMA state starts empty.
        Rp2350UartDma {
            tx_channel: TXC,
            rx_channel: RXC,
            rx_ring: rx,
            rx_read: 0,
            tx_ring: tx,
            tx_queued: 0,
            tx_started: 0,
            divisor: self.divisor,
            errors: self.errors,
            rx_halted: false,
            tx_halted: false,
        }
    }
}

// --- Driver --------------------------------------------------------------

/// UART `I` with its FIFOs serviced by DMA. Made by
/// [`Rp2350Uart::into_dma`]; see the [module documentation](self).
///
/// Dropping it stops both channels and turns `UARTDMACR` off; the UART
/// itself stays configured and enabled.
pub struct Rp2350UartDma<const I: usize> {
    /// The TX channel's number.
    tx_channel: usize,
    /// The RX channel's number.
    rx_channel: usize,
    /// First RX slot. Accessed only with volatile reads and writes.
    rx_ring: *mut u16,
    /// The next RX slot to read.
    rx_read: usize,
    /// First TX byte. Accessed only with volatile reads and writes.
    tx_ring: *mut u8,
    /// Bytes ever copied into the TX ring (wrapping).
    tx_queued: u32,
    /// Bytes ever handed to the TX channel, i.e. up to the end of its
    /// current or last transfer (wrapping).
    tx_started: u32,
    /// The UART's baud divisor, for [`baud`](Self::baud).
    divisor: BaudDivisor,
    /// Receive errors since the last [`take_line_errors`](Self::take_line_errors).
    errors: LineErrors,
    /// The RX channel has reported a bus error.
    rx_halted: bool,
    /// The TX channel has reported a bus error.
    tx_halted: bool,
}

// SAFETY: the ring pointers come from the `&'static mut` rings handed to
// `into_dma`, so this value is the only thing that touches them, and no
// interrupt is enabled, so nothing on the core it leaves reaches them after
// it moves. The UART and DMA registers are shared bus peripherals either
// core may drive. `Send`, not `Sync`: one owner at a time, since two cores
// using one ring at once would corrupt its read and write counts.
unsafe impl<const I: usize> Send for Rp2350UartDma<I> {}

impl<const I: usize> Rp2350UartDma<I> {
    /// The baud rate actually produced, as [`Rp2350Uart::baud`].
    pub fn baud(&self) -> u32 {
        // Same divisor, same clock.
        self.divisor.baud(CLK_PERI_HZ)
    }

    /// The receive errors counted since the last call, and reset the
    /// counts to zero.
    pub fn take_line_errors(&mut self) -> LineErrors {
        // Swap the counts out for zeros.
        core::mem::take(&mut self.errors)
    }

    // --- Receive ---------------------------------------------------------

    /// Move received characters into `buf`, up to `buf.len()`; returns how
    /// many. `Ok(0)` means nothing is waiting. Never blocks.
    ///
    /// Characters with framing, parity or break errors are dropped and
    /// counted, as by [`Rp2350Uart::read`]. Errors, checked first:
    /// [`UartError::DmaError`] (the RX channel halted) and
    /// [`UartError::RingOverrun`] (the ring is discarded; the next call
    /// returns characters from now on).
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, UartError> {
        // Where the channel has got to, or an error.
        let written = self.rx_written()?;
        // Good bytes stored in `buf`.
        let mut n = 0;
        // Take entries until `buf` is full or the reader catches up.
        while n < buf.len() && self.rx_read != written {
            // Pop the oldest entry and sort it.
            let entry = self.rx_pop();
            if let Ok(byte) = uart::classify(entry as u32, &mut self.errors) {
                buf[n] = byte;
                n += 1;
            }
        }
        // Good bytes delivered.
        Ok(n)
    }

    /// The RX ring index the channel writes next, after checking for a
    /// halted channel and for an overrun. Every slot before it holds a
    /// character.
    fn rx_written(&mut self) -> Result<usize, UartError> {
        // A bus error halts the channel for good (Table 1150, p1125).
        if self.rx_halted || unsafe { dma::ctrl(self.rx_channel) } & CTRL_AHB_ERROR != 0 {
            self.rx_halted = true;
            return Err(UartError::DmaError);
        }
        // Where the channel writes next.
        let written = self.rx_write_index();
        // A slot ahead of the channel that is not the sentinel was written
        // a lap ago and never read: reading has fallen a ring behind.
        let ahead = (written + RX_GUARD_ENTRIES) % RX_RING_ENTRIES;
        if unsafe { self.rx_ring.add(ahead).read_volatile() } != SENTINEL {
            self.rx_resync();
            return Err(UartError::RingOverrun);
        }
        // No error: the reader may consume up to `written`.
        Ok(written)
    }

    /// The RX ring index of the next slot the channel writes, from its
    /// `WRITE_ADDR`. RP2350 reports the completed position even across a
    /// wrap (§12.6.1, p1094, as relied on by [`crate::adc`]). The fence
    /// keeps the ring reads that follow after this register read.
    fn rx_write_index(&self) -> usize {
        // Live WRITE_ADDR (Table 1148, p1124).
        let addr = unsafe { dma::write_addr(self.rx_channel) };
        // No ring read may move before the register read (rule RNDHC, p239).
        fence(Ordering::Acquire);
        // Byte offset into the ring, in halfword entries.
        (addr as usize - self.rx_ring as usize) / 2 % RX_RING_ENTRIES
    }

    /// Take the oldest entry and mark its slot empty again.
    fn rx_pop(&mut self) -> u16 {
        // The slot to read.
        let slot = unsafe { self.rx_ring.add(self.rx_read) };
        // Volatile: the DMA writes this memory.
        let entry = unsafe { slot.read_volatile() };
        // Back to the sentinel, so the next lap's write is noticed.
        unsafe { slot.write_volatile(SENTINEL) };
        // Advance, wrapping.
        self.rx_read = (self.rx_read + 1) % RX_RING_ENTRIES;
        entry
    }

    /// After an overrun: every slot except the [`RX_GUARD_ENTRIES`] from the
    /// channel's position back to the sentinel, and reading restarted from
    /// that position. If the channel has left that window by the end, it
    /// may have written a slot after it was cleared, so start again. The
    /// same procedure as the ADC stream's.
    fn rx_resync(&mut self) {
        loop {
            // Where the channel is now.
            let from = self.rx_write_index();
            // Clear everything outside the window it may write meanwhile.
            for i in RX_GUARD_ENTRIES..RX_RING_ENTRIES {
                let slot = (from + i) % RX_RING_ENTRIES;
                unsafe { self.rx_ring.add(slot).write_volatile(SENTINEL) };
            }
            // Still inside the window: consistent; resume from `from`.
            if rx_distance(from, self.rx_write_index()) < RX_GUARD_ENTRIES {
                self.rx_read = from;
                return;
            }
        }
    }

    // --- Transmit --------------------------------------------------------

    /// Queue bytes for transmission; returns how many fit in the transmit
    /// ring, which may be fewer than `data.len()`, or 0. Never blocks.
    /// Starts the TX channel if it is idle. [`UartError::DmaError`] if the
    /// TX channel has halted.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, UartError> {
        // Error check, and restart if the last transfer has finished.
        self.poll()?;
        // Bytes queued but not yet taken by the DMA.
        let in_ring = self.tx_queued.wrapping_sub(self.tx_completed()) as usize;
        // Room left, and how much of `data` fits.
        let n = data.len().min(TX_RING_BYTES - in_ring);
        for &byte in &data[..n] {
            // Next free slot, wrapping on the ring size.
            let slot = self.tx_queued as usize % TX_RING_BYTES;
            // Volatile: the DMA reads this memory.
            unsafe { self.tx_ring.add(slot).write_volatile(byte) };
            self.tx_queued = self.tx_queued.wrapping_add(1);
        }
        // Send them now if the channel is idle.
        self.poll()?;
        Ok(n)
    }

    /// Start the TX channel on everything queued, if it is idle and
    /// anything is queued; report a halted channel. Call it regularly so
    /// bytes queued while a transfer ran are not left waiting (see
    /// [Transmit](self#transmit-a-ring-the-channel-drains)). Never blocks.
    pub fn poll(&mut self) -> Result<(), UartError> {
        // CTRL: BUSY and the error flags (Table 1150, p1125).
        let ctrl = unsafe { dma::ctrl(self.tx_channel) };
        // A bus error halts the channel for good.
        if self.tx_halted || ctrl & CTRL_AHB_ERROR != 0 {
            self.tx_halted = true;
            return Err(UartError::DmaError);
        }
        // Still sending, or nothing new: nothing to do.
        if ctrl & CTRL_BUSY != 0 || self.tx_queued == self.tx_started {
            return Ok(());
        }
        // Bytes queued since the last transfer started.
        let count = self.tx_queued.wrapping_sub(self.tx_started);
        // Where they start in the ring.
        let first = self.tx_started as usize % TX_RING_BYTES;
        // This UART's registers.
        let regs = Rp2350Uart::<I>::regs();
        // Ring stores before the trigger (rule RNDHC, Armv8-M ARM p239).
        fence(Ordering::SeqCst);
        unsafe {
            dma::start(
                self.tx_channel,
                // Read from the first unsent byte; RING_SIZE wraps it.
                self.tx_ring.add(first) as usize as u32,
                // Always write the data register.
                &raw mut (*regs).dr as usize as u32,
                // NORMAL: stop after `count` bytes (Table 1149, p1124).
                (MODE_NORMAL << TRANS_COUNT_MODE_SHIFT) | count,
                CTRL_EN
                    | (DATA_SIZE_BYTE << CTRL_DATA_SIZE_SHIFT)
                    | CTRL_INCR_READ
                    | (TX_RING_SIZE_LOG2 << CTRL_RING_SIZE_SHIFT)
                    | CTRL_RING_SEL_READ
                    // CHAIN_TO = own number disables chaining (Table 1150, p1126).
                    | ((self.tx_channel as u32) << CTRL_CHAIN_TO_SHIFT)
                    | (dreq_uart_tx(I) << CTRL_TREQ_SEL_SHIFT),
            );
        }
        // Everything queued is now in flight.
        self.tx_started = self.tx_queued;
        Ok(())
    }

    /// Bytes the TX channel has finished writing to the UART, ever
    /// (wrapping). While it runs, that is the bytes started less those it
    /// still has to do; once it stops, all of them. If it stops between
    /// the two register reads, `TRANS_COUNT` reads 0 and the answer is
    /// still right.
    fn tx_completed(&self) -> u32 {
        // BUSY clear: the last transfer is complete (§12.6.3, p1097).
        if unsafe { dma::ctrl(self.tx_channel) } & CTRL_BUSY == 0 {
            return self.tx_started;
        }
        // Running: subtract what is left (Table 1149, p1124).
        self.tx_started.wrapping_sub(unsafe { dma::remaining(self.tx_channel) })
    }

    /// `true` once every queued byte has been handed to the UART and has
    /// left the pin (`UARTFR.BUSY` clear, Table 1030, p973).
    pub fn is_idle(&self) -> bool {
        // Everything queued has been started and the channel has stopped...
        self.tx_queued == self.tx_started
            && unsafe { dma::ctrl(self.tx_channel) } & CTRL_BUSY == 0
            // ...and the UART has shifted out the last stop bit.
            && unsafe { (&raw const (*Rp2350Uart::<I>::regs()).fr).read_volatile() } & FR_BUSY == 0
    }

    /// Wait until [`is_idle`](Self::is_idle), restarting the channel as
    /// needed. Ends within the time to send everything queued: at most
    /// 1024 + 32 characters, 1.1 s at 9600 baud. [`UartError::DmaError`]
    /// if the TX channel halts.
    pub fn flush(&mut self) -> Result<(), UartError> {
        // Keep the channel fed until everything is out.
        while !self.is_idle() {
            self.poll()?;
        }
        Ok(())
    }
}

/// Stops both channels (the abort sequence in [`crate::dma`], including
/// the RP2350-E5 workaround) and clears `UARTDMACR`. Bytes still queued
/// are not sent. The UART stays enabled; the rings are no longer touched.
impl<const I: usize> Drop for Rp2350UartDma<I> {
    fn drop(&mut self) {
        unsafe {
            // Stop transmitting from the ring.
            dma::abort(self.tx_channel);
            // Stop receiving into the ring.
            dma::abort(self.rx_channel);
            // No more DMA requests from the UART (Table 1041, p979).
            (&raw mut (*Rp2350Uart::<I>::regs()).dmacr).write_volatile(0);
        }
    }
}

/// Byte-at-a-time access through the portable traits, as for the polled
/// driver. `uart.read(&mut buf)` and `uart.write(&data)` name the slice
/// methods; the trait methods are `Read::read(&mut uart)` and
/// `Write::write(&mut uart, byte)`.
impl<const I: usize> ErrorType for Rp2350UartDma<I> {
    type Error = UartError;
}

impl<const I: usize> Read<u8> for Rp2350UartDma<I> {
    /// One received character; [`UartError::WouldBlock`] if none is
    /// waiting; the error that character carried; or a DMA error.
    fn read(&mut self) -> Result<u8, UartError> {
        // Errors first, then where the channel has got to.
        let written = self.rx_written()?;
        // Nothing new.
        if self.rx_read == written {
            return Err(UartError::WouldBlock);
        }
        // One entry, sorted into a character or its error.
        let entry = self.rx_pop();
        uart::classify(entry as u32, &mut self.errors)
    }
}

impl<const I: usize> Write<u8> for Rp2350UartDma<I> {
    /// Queue one byte; [`UartError::WouldBlock`] if the transmit ring is
    /// full.
    fn write(&mut self, byte: u8) -> Result<(), UartError> {
        // The slice method returns how many it queued: 0 or 1 here.
        if Rp2350UartDma::write(self, &[byte])? == 1 {
            Ok(())
        } else {
            Err(UartError::WouldBlock)
        }
    }
}
