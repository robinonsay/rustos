//! Peripheral base addresses.

/// Base address of each peripheral this crate talks to.
///
/// Cast a variant to a pointer to the matching layout struct to reach its
/// registers:
///
/// ```ignore
/// let sio = RegAddr::SIO as usize as *mut Sio;
/// let gpio_in = unsafe { (&raw const (*sio).gpio_in).read_volatile() };
/// ```
///
/// The two-step `as usize as *mut _` is required: Rust will not cast an enum
/// straight to a raw pointer, so the discriminant is taken as an integer
/// first.
///
/// # Address space
///
/// These come from three different regions of the RP2350 memory map (Table 7,
/// p31), which is worth noticing because the difference is architectural, not
/// cosmetic:
///
/// * `0x4000_0000` — **APB peripherals** (Advanced Peripheral Bus), behind a
///   bridge. A read costs at least three cycles and a write four.
/// * `0x5000_0000` — AHB peripherals (Advanced High-performance Bus; DMA,
///   USB), zero-wait-state: reads and writes complete in one bus cycle.
/// * `0xd000_0000` — **core-local** peripherals, i.e. [`SIO`](RegAddr::SIO).
///
/// # Atomic aliases
///
/// Every peripheral register block is allocated 4 kB, and most are mirrored
/// three more times (§2.1.3, p27):
///
/// | Offset | Effect of a write |
/// |--------|-------------------|
/// | `+0x0000` | normal read/write |
/// | `+0x1000` | atomic XOR |
/// | `+0x2000` | atomic bitmask set |
/// | `+0x3000` | atomic bitmask clear |
///
/// These let you change one field without a read-modify-write, which matters
/// whenever an interrupt handler or the other core might touch the same
/// register in between. Because the layout structs are `#[repr(C)]` — fields
/// are laid out in declaration order at the offsets a C compiler would use,
/// so a struct field's address equals block base plus register offset — you
/// get an alias view for free by re-basing the same type — for example
/// `(RegAddr::RESET as usize + 0x3000) as *mut Reset` is a clear-alias view of
/// the whole reset controller.
///
/// [`SIO`](RegAddr::SIO) is explicitly **excluded** from this scheme; it
/// provides its own dedicated `SET`/`CLR`/`XOR` registers instead.
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Variant names deliberately match the datasheet's block names exactly, so
// code can be checked against the register listings without translation.
#[allow(non_camel_case_types)]
pub enum RegAddr {
    /// `RESETS` — subsystem reset controller. Holds every peripheral in reset
    /// until software releases it. See [`crate::common::reset`].
    RESET = 0x4002_0000,

    /// `IO_BANK0` — function select and interrupt control for GPIO0–47.
    /// Decides *which peripheral* a pin is connected to.
    IO_BANK0 = 0x4002_8000,

    /// `PADS_BANK0` — the physical pads for GPIO0–47: input enable, output
    /// disable, pull resistors, drive strength, Schmitt trigger, isolation.
    /// Decides *electrical behaviour* once `IO_BANK0` has chosen a function.
    PADS_BANK0 = 0x4003_8000,

    /// `SIO` — single-cycle I/O, the fast path for GPIO.
    ///
    /// Unusual in three ways. It sits at `0xd000_0000`, outside both
    /// peripheral regions; it is reached over two dedicated AHB ports, one per
    /// core, so accesses take a single cycle with no bus arbitration (p26);
    /// and it is **not banked per core** for GPIO — both cores see the same
    /// pins.
    ///
    /// On RP2040 this block hung off the Cortex-M0+ IOPORT, an Arm-defined
    /// port. On RP2350 it does not, which is part of why the same GPIO code
    /// works when the chip is booted with its RISC-V Hazard3 cores instead.
    SIO = 0xd000_0000,

    /// `CLOCKS`: the clock generators (`clk_ref`, `clk_sys`, `clk_peri`, ...)
    /// and the frequency counter `FC0`. Datasheet §8.1.6, Table 542 (p521).
    /// Not in `RESETS`; it is reset with the chip.
    CLOCKS = 0x4001_0000,

    /// `XOSC`: the crystal oscillator (12 MHz on the Pico 2). Datasheet
    /// §8.2.8, Table 597 (p555). Not in `RESETS`.
    XOSC = 0x4004_8000,

    /// `PLL_SYS`: the system PLL that `clk_sys` runs from. Datasheet §8.6.5,
    /// Table 635 (p580). `RESETS.RESET` bit 14.
    PLL_SYS = 0x4005_0000,

    /// `TICKS`: the tick generators that turn `clk_ref` into the 1 µs
    /// timebase of TIMER0, TIMER1, the watchdog and `SysTick`. Datasheet §8.5.2,
    /// Table 616 (p567). Not in `RESETS`.
    TICKS = 0x4010_8000,

    /// The Cortex-M33 NVIC enable and pending registers, starting at
    /// `NVIC_ISER0` (PPB offset `0x0e100`, datasheet §3.7.5, Table 192, p179).
    /// Part of the Arm private peripheral bus: no `RESETS` bit and no atomic
    /// aliases; the set and clear registers are separate words instead.
    NVIC = 0xe000_e100,

    /// `TIMER0`: the 64-bit microsecond timer and its four alarms. Datasheet
    /// §12.8.5, Table 1226 (p1184). `RESETS.RESET` bit 23.
    TIMER0 = 0x400b_0000,

    /// `PWM`: twelve PWM slices (0 to 7 reachable on the RP2350A). Datasheet
    /// §12.5.3, Table 1130 (p1083). `RESETS.RESET` bit 16.
    PWM = 0x400a_8000,
}

/// Offset of the atomic **XOR** alias of an APB register (§2.1.3, p27).
///
/// A write to `register + ALIAS_XOR` flips the written bits of the register.
/// Not available on [`RegAddr::SIO`] (it has its own XOR registers). No
/// driver writes it yet; the test register file models it.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ALIAS_XOR: usize = 0x1000;

/// Offset of the atomic **bitmask set** alias of an APB register (§2.1.3).
///
/// A write to `register + ALIAS_SET` sets the written bits and leaves every
/// other bit alone, in one bus write with no read-modify-write window.
pub(crate) const ALIAS_SET: usize = 0x2000;

/// Offset of the atomic **bitmask clear** alias of an APB register (§2.1.3).
///
/// A write to `register + ALIAS_CLR` clears the written bits and leaves every
/// other bit alone.
pub(crate) const ALIAS_CLR: usize = 0x3000;

/// Every in-block offset is masked with this before it becomes an address:
/// word-aligned and inside the block's 16 KiB window (4 KiB of registers plus
/// the three alias views, §2.1.3). See [`Mmio`].
pub(crate) const BLOCK_WINDOW_MASK: usize = 0x3ffc;

/// Register access as a capability, so that a driver's *sequence* of register
/// reads and writes can run against the hardware on the target and against a
/// scripted register file in host tests.
///
/// A driver written against `Regs` keeps every decision (what to write, what a
/// status value means, when a wait has gone on too long) in code that compiles
/// and runs on the host, where unit tests exercise it (cwht coding standard
/// CS-38). The only target-only piece is [`Mmio`], which turns
/// `(block, offset)` into a volatile load or store and decides nothing.
///
/// `offset` is the register's byte offset from the block base as the
/// datasheet's "List of Registers" gives it, optionally plus one of the alias
/// offsets [`ALIAS_XOR`], [`ALIAS_SET`], [`ALIAS_CLR`].
pub(crate) trait Regs {
    /// Read the 32-bit register at `block + offset`.
    fn read(&mut self, block: RegAddr, offset: usize) -> u32;
    /// Write `value` to the 32-bit register at `block + offset`.
    fn write(&mut self, block: RegAddr, offset: usize, value: u32);
}

/// The hardware implementation of [`Regs`]: volatile loads and stores to the
/// peripheral windows of the RP2350 memory map. Zero-sized.
///
/// Only compiled for the bare-metal target; on the host the drivers run
/// against the test register file instead, so no host code can dereference a
/// peripheral address.
#[cfg(target_os = "none")]
pub(crate) struct Mmio;

#[cfg(target_os = "none")]
impl Mmio {
    /// The address `block + (offset & BLOCK_WINDOW_MASK)`.
    ///
    /// The mask is the whole bounds argument: whatever `offset` a caller
    /// passes, the access lands word-aligned inside the 16 KiB window that
    /// the datasheet allocates to `block` (§2.1.3, p27), never outside it.
    #[inline]
    fn address(block: RegAddr, offset: usize) -> *mut u32 {
        (block as usize + (offset & BLOCK_WINDOW_MASK)) as *mut u32
    }
}

#[cfg(target_os = "none")]
impl Regs for Mmio {
    #[inline]
    fn read(&mut self, block: RegAddr, offset: usize) -> u32 {
        // SAFETY: `address` is a `RegAddr` peripheral base (RP2350 datasheet section 2.2,
        // Table 7) plus an offset masked to a word inside that block's 16 KiB register and
        // alias window (section 2.1.3), so the place is an aligned 32-bit MMIO word and never
        // Rust-owned memory. The load is volatile through a raw pointer, so no reference to
        // device memory is formed and the access is neither elided nor merged.
        unsafe { Self::address(block, offset).read_volatile() }
    }

    #[inline]
    fn write(&mut self, block: RegAddr, offset: usize, value: u32) {
        // SAFETY: as for `read`: an aligned 32-bit MMIO word inside `block`'s window
        // (datasheet sections 2.2 and 2.1.3), written with one volatile store through a raw
        // pointer. Which register is written, and with what, is the calling driver's
        // obligation; every caller cites the datasheet table of the register it writes.
        unsafe { Self::address(block, offset).write_volatile(value) }
    }
}

/// A bounded wait timed out: the condition did not hold within the budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PollTimeout;

/// Read `block + offset` until `value & mask == want`, at most `budget` times.
///
/// Returns the number of reads it took (1 when the condition already held),
/// or [`PollTimeout`] after `budget` reads without success. The budget is a
/// loop count, not a time, so the bound holds even when the clock the wait is
/// for never starts (cwht coding standard CS-37). A `budget` of 0 reads
/// nothing and times out.
pub(crate) fn poll<R: Regs>(
    regs: &mut R,
    block: RegAddr,
    offset: usize,
    mask: u32,
    want: u32,
    budget: u32,
) -> Result<u32, PollTimeout> {
    let mut reads: u32 = 0;
    while reads < budget {
        reads += 1;
        if regs.read(block, offset) & mask == want {
            return Ok(reads);
        }
    }
    Err(PollTimeout)
}

#[cfg(test)]
pub(crate) mod fake;

#[cfg(test)]
mod tests {
    use super::fake::FakeRegs;
    use super::*;

    #[test]
    fn poll_returns_one_when_the_condition_already_holds() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::XOSC, 0x04, 0x8000_0000);
        assert_eq!(
            poll(&mut regs, RegAddr::XOSC, 0x04, 1 << 31, 1 << 31, 5),
            Ok(1)
        );
    }

    #[test]
    fn poll_counts_reads_until_the_condition_holds() {
        let mut regs = FakeRegs::new();
        regs.script_reads(RegAddr::XOSC, 0x04, &[0, 0, 0x8000_0000]);
        assert_eq!(
            poll(&mut regs, RegAddr::XOSC, 0x04, 1 << 31, 1 << 31, 5),
            Ok(3)
        );
    }

    #[test]
    fn poll_times_out_after_exactly_budget_reads() {
        let mut regs = FakeRegs::new();
        assert_eq!(
            poll(&mut regs, RegAddr::XOSC, 0x04, 1 << 31, 1 << 31, 4),
            Err(PollTimeout)
        );
        assert_eq!(regs.reads_of(RegAddr::XOSC, 0x04), 4);
    }

    #[test]
    fn poll_with_zero_budget_reads_nothing() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::XOSC, 0x04, 0x8000_0000);
        assert_eq!(
            poll(&mut regs, RegAddr::XOSC, 0x04, 1 << 31, 1 << 31, 0),
            Err(PollTimeout)
        );
        assert_eq!(regs.reads_of(RegAddr::XOSC, 0x04), 0);
    }

    #[test]
    fn poll_compares_only_the_masked_bits() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::CLOCKS, 0x44, 0xffff_fff2);
        assert_eq!(poll(&mut regs, RegAddr::CLOCKS, 0x44, 0b11, 0b10, 1), Ok(1));
    }
}
