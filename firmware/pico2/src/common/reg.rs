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
#[derive(Clone, Copy)]
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
}
