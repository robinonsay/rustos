use crate::common::reg::RegAddr;

/// A peripheral that owns one or more bits in the `RESETS` register.
///
/// Implemented by drivers rather than by the reset controller, so each driver
/// keeps the knowledge of which bits it needs next to the code that needs
/// them. A driver that reaches pins generally owns more than one bit: `UART0`
/// alone does not produce a working UART, because the signal still has to get
/// through `IO_BANK0`'s function mux and a `PADS_BANK0` pad.
pub trait Block
{
    /// Release this peripheral's blocks from reset and block until the
    /// hardware reports them ready.
    ///
    /// Must be called before touching any of the peripheral's registers.
    /// Accesses to a block still in reset do not fault — they are accepted by
    /// the bus and discarded — so skipping this produces a peripheral that
    /// silently ignores every write.
    ///
    /// # Safety
    ///
    /// Writes a chip-wide control register shared with every other driver, and
    /// leaves hardware running. Implementations must touch only their own bits.
    unsafe fn start(&self);

    /// Return this peripheral's blocks to reset.
    ///
    /// # Safety
    ///
    /// Any handle to this peripheral becomes non-functional. As with
    /// [`start`](Block::start), implementations must confine themselves to
    /// their own bits: asserting reset on `IO_QSPI` or `PADS_QSPI` cuts the
    /// pins that XIP fetches instructions from, and execution stops mid-fetch
    /// with no fault and no output.
    unsafe fn reset(&self);
}


/// `RESETS` — the subsystem reset controller. Base `0x4002_0000`.
///
/// Almost every peripheral on the chip comes out of power-on reset **held in
/// reset**, and stays there until software releases it. Reads and writes to a
/// held peripheral do not fault; they just do nothing useful, which is why a
/// forgotten unreset presents as a peripheral that silently ignores you.
///
/// Bit assignments are the same in all three registers (Table 534, p504):
///
/// | Bit | Block | | Bit | Block | | Bit | Block |
/// |----|--------------|-|----|-----------|-|----|-----------|
/// | 28 | `USBCTRL`    | | 18 | `SPI0`    | |  8 | `JTAG`    |
/// | 27 | `UART1`      | | 17 | `SHA256`  | |  7 | `IO_QSPI` |
/// | 26 | `UART0`      | | 16 | `PWM`     | |  6 | `IO_BANK0` |
/// | 25 | `TRNG`       | | 15 | `PLL_USB` | |  5 | `I2C1`    |
/// | 24 | `TIMER1`     | | 14 | `PLL_SYS` | |  4 | `I2C0`    |
/// | 23 | `TIMER0`     | | 13 | `PIO2`    | |  3 | `HSTX`    |
/// | 22 | `TBMAN`      | | 12 | `PIO1`    | |  2 | `DMA`     |
/// | 21 | `SYSINFO`    | | 11 | `PIO0`    | |  1 | `BUSCTRL` |
/// | 20 | `SYSCFG`     | | 10 | `PADS_QSPI` | | 0 | `ADC`   |
/// | 19 | `SPI1`       | |  9 | `PADS_BANK0` | |   |         |
///
/// GPIO bring-up needs bits 6 (`IO_BANK0`) and 9 (`PADS_BANK0`).
#[repr(C)]
struct Reset{
    /// `RESET` — `0x0`. Read/write, **reset value `0x1fff_ffff`** — every bit set.
    ///
    /// One bit per resettable block. **1 asserts reset, 0 releases it.** The
    /// polarity is the opposite of what the name suggests: to *start* using a
    /// peripheral you *clear* its bit.
    ///
    /// Because every bit resets to 1, this must be a read-modify-write that
    /// clears only the bits you own:
    ///
    /// ```ignore
    /// let cur = reset.read_volatile();
    /// reset.write_volatile(cur & !MASK);   // release just MASK
    /// ```
    ///
    /// Writing `!MASK` directly instead of `cur & !MASK` writes 1s to all 27
    /// other bits and slams those peripherals *into* reset. On this chip that
    /// includes bits 7 and 10, `IO_QSPI` and `PADS_QSPI` — the pins the XIP
    /// flash is attached to, which is where your code is executing from. The
    /// program stops mid-instruction-fetch with no fault and no output.
    ///
    /// The `+0x3000` atomic-clear alias does the same job in one store with no
    /// window in which the read and the write can be interrupted.
    pub reset: u32,
    /// `WDSEL` — `0x4`. Read/write, reset `0x0`.
    ///
    /// One bit per block: 1 means "also reset this block when the watchdog
    /// fires." Independent of [`reset`](Self::reset) — this register selects
    /// what a *future* watchdog event tears down, it does not assert reset
    /// itself.
    ///
    /// Leave at zero unless you are building watchdog recovery. Note the
    /// warning in §7.5, p503: resetting the power-on state machine resets the
    /// entire reset controller, and with it every block.
    ///
    /// Table 535, p505.
    pub wdsel: u32,
    /// `RESET_DONE` — `0x8`. **Read-only**, reset `0x0`.
    ///
    /// One bit per block, set by hardware once that block has finished coming
    /// out of reset and is safe to talk to. Note that the polarity is the
    /// *inverse* of [`reset`](Self::reset): here **1 means ready**.
    ///
    /// Releasing a reset is not instantaneous, so after clearing bits in
    /// `RESET` you poll here until the same bits read back as 1:
    ///
    /// ```ignore
    /// while reset_done.read_volatile() & MASK != MASK {}
    /// ```
    ///
    /// The pointer must be read with `read_volatile` inside the loop. If you
    /// read through a plain `&u32`, LLVM is entitled to hoist the load out of
    /// the loop — nothing in the language says a `&u32` can change — and the
    /// wait compiles to an unconditional branch to itself.
    pub reset_done: u32
}


/// Read-modify-write `RESETS.RESET` with `RESET &= mask`.
///
/// **Clears** every bit that is `0` in `mask`, and leaves every bit that is
/// `1` at its current value. Since `0` means "released" in this register
/// (§7.5: "When set to 1, the reset is asserted"), the effect is to *release*
/// the blocks whose mask bits are zero. Note the two inversions stacked on top
/// of each other — the polarity is inverted, and so is the mask — so callers
/// releasing a block pass the complement:
///
/// ```ignore
/// set_reset_reg(!IO_PAD_BITMASK);   // release IO_BANK0 and PADS_BANK0
/// ```
///
/// Because the operation is an `AND`, this function can only ever move bits
/// from 1 to 0. It cannot assert a reset; that would require `current | mask`.
///
/// Releasing is not instantaneous — follow with [`wait_for_reset_done`].
///
/// # A note on the read-modify-write
///
/// The read and write are separate bus transactions, so an interrupt or the
/// other core can land in between and lose an update. `RESETS` is an APB
/// peripheral and is *not* on the exclusion list in §2.1.3, so it has the
/// atomic aliases: a single store to `RESET_CLR` at `0x4002_3000` would do the
/// same job with no window and no risk of writing the wrong value to the other
/// 27 bits.
///
/// # Safety
///
/// Writes a chip-wide control register. Passing a mask with zeros outside the
/// caller's own blocks releases peripherals belonging to other drivers.
pub unsafe fn clr_reset_reg(mask: u32){
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current & mask);

    }
}

pub unsafe fn set_reset_reg(mask: u32){
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current | mask);

    }
}

/// Spin until every block in `mask` reports ready.
///
/// `RESET_DONE` has the **inverse** polarity of `RESET`: here `1` means the
/// block has finished coming out of reset and is safe to talk to. So this
/// waits for the bits to become set, whereas releasing them meant clearing
/// bits in `RESET`.
///
/// The `read_volatile` inside the loop is mandatory, not stylistic. Read
/// through a plain `&u32` and LLVM is entitled to hoist the load out — nothing
/// in the language says a `&u32` can change behind your back — and the wait
/// compiles down to an unconditional branch to itself. A hang here is
/// therefore a very plausible symptom of dropping the `volatile`.
///
/// # Safety
///
/// Reads a hardware register. Loops forever if a block in `mask` never
/// reports ready, which happens if it was never released in the first place.
pub unsafe fn wait_for_reset_done(mask: u32){
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        let reset_done = &raw const (*reset_addr).reset_done;
        // Wait for it to be done
        while reset_done.read_volatile() & mask != mask
        {}
    }
}