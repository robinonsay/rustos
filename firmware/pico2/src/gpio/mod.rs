//! Memory-mapped register layouts for RP2350 GPIO bring-up.
//!
//! Every struct in this module is a *layout template*, not a value. None of
//! them are ever constructed. You take the peripheral's base address from
//! [`crate::common::reg::RegAddr`], cast it to a pointer to one of these
//! types, project to a field with `&raw mut` / `&raw const`, and access that
//! field with [`core::ptr::read_volatile`] / [`core::ptr::write_volatile`].
//!
//! ```ignore
//! let sio = RegAddr::SIO as usize as *mut Sio;
//! let set = &raw mut (*sio).gpio_out_set;   // pointer, no reference formed
//! set.write_volatile(1 << 25);              // the access is what is volatile
//! ```
//!
//! Two rules make this sound, and both are easy to violate by accident:
//!
//! 1. **Never form a `&` or `&mut` to these fields.** A reference asserts to
//!    the compiler that nothing else can change the memory, which is exactly
//!    false for hardware. `as_mut().unwrap()` looks harmless and lets LLVM
//!    hoist a load out of a poll loop, turning the wait into `b .`.
//! 2. **Volatility lives on the access, not the pointer.** `read_volatile` and
//!    `write_volatile` are the only things that guarantee the load or store
//!    actually happens, once, in program order.
//!
//! `#[repr(C)]` is load-bearing on every struct here. It is what pins field
//! order and offsets to the declaration order; without it the compiler is free
//! to reorder fields and the addresses stop matching the hardware. Explicit
//! `_reserved` padding is likewise mandatory — the holes in a peripheral's
//! address space have to be spelled out, because `#[repr(C)]` only inserts
//! padding for alignment, and every field here is already 4-byte aligned.
//!
//! ## Atomic register aliases
//!
//! Each APB peripheral gets 4 kB of address space, aliased four ways
//! (§2.1.3, p26):
//!
//! | Address     | Effect on write        |
//! |-------------|------------------------|
//! | `base + 0x0000` | normal read/write  |
//! | `base + 0x1000` | atomic XOR         |
//! | `base + 0x2000` | atomic bitmask set |
//! | `base + 0x3000` | atomic bitmask clear |
//!
//! This lets you change one field of a control register without a
//! read-modify-write, which matters when an interrupt handler or the other
//! core touches the same register. `IO_BANK0`, `PADS_BANK0`, and `RESETS` all
//! support it. **SIO does not** — it is not on the APB bus at all. SIO instead
//! provides set/clear/XOR as *separate registers* in its normal window, which
//! is why `Sio` below is four times longer than it looks like it needs to be.
//!
//! ## Datasheet sources
//!
//! All references are to the RP2350 datasheet (`docs/rp2350-datasheet.pdf`),
//! cited by PDF page number:
//!
//! - Table 16, p55–56 — SIO register map
//! - Tables 649/650, p607–609 — `GPIO0_STATUS`, `GPIO0_CTRL` bit fields
//! - Tables 745+, p687–697 — `IRQSUMMARY_*` and `INTR*` offsets
//! - Tables 851/852, p785 — `VOLTAGE_SELECT` and `GPIOn` pad control bits
//! - Tables 533/534, p504 — `RESETS` register map and reset bit assignments
//! - §2.1.3, p27 — atomic register aliases

pub mod gpio;

/// One GPIO's pair of control registers in `IO_BANK0`.
///
/// The bank is an array of 48 of these, one per GPIO, so `GPIO*n*_STATUS*`
/// lives at `IO_BANK0 + n * 8` and `GPIO*n*_CTRL` at `IO_BANK0 + n * 8 + 4`.
/// For the on-board LED on GPIO25 that puts `CTRL` at offset `0x0cc`.
///
/// This block routes *signals*: it decides which peripheral inside the chip is
/// connected to the pin. It says nothing about the electrical behaviour of the
/// pin itself — that is [`PadsBank`].
#[repr(C)]
struct GpioRegs{
    /// `GPIOn_STATUS` — offset `0x0` within the pair. **Read-only.**
    ///
    /// Reports the live signal values at four observation points along the
    /// pin's signal path, *after* the override fields in [`ctrl`](Self::ctrl)
    /// have been applied. Purely diagnostic; writing it does nothing.
    ///
    /// | Bit | Name         | Meaning |
    /// |-----|--------------|---------|
    /// | 26  | `IRQTOPROC`  | interrupt signal being delivered to the processors |
    /// | 17  | `INFROMPAD`  | raw input from the pad, before filtering and override |
    /// | 13  | `OETOPAD`    | output-enable being driven to the pad |
    /// |  9  | `OUTTOPAD`   | output level being driven to the pad |
    ///
    /// All other bits read as zero. Reset value `0x0`.
    ///
    /// This is the register to read when a pin is not doing what you expect
    /// and you need to know *where* the signal stops. If `OETOPAD` is 0 after
    /// you set the pin as an output, the problem is upstream in SIO or in
    /// `OEOVER`; if it is 1 but the pin measures low, the problem is
    /// downstream in the pad — `OD` or `ISO`.
    ///
    /// Table 649, p607.
    pub status: u32,
    /// `GPIOn_CTRL` — offset `0x4` within the pair. Read/write, reset `0x1f`.
    ///
    /// Selects which internal peripheral drives this pin, plus four override
    /// fields that can force a signal without involving that peripheral.
    ///
    /// | Bits  | Name      | Meaning |
    /// |-------|-----------|---------|
    /// | 29:28 | `IRQOVER` | 0 normal, 1 invert, 2 force low, 3 force high |
    /// | 17:16 | `INOVER`  | same encoding, applied to the input to the peripheral |
    /// | 15:14 | `OEOVER`  | 0 normal, 1 invert, 2 disable output, 3 enable output |
    /// | 13:12 | `OUTOVER` | 0 normal, 1 invert, 2 drive low, 3 drive high |
    /// |  4:0  | `FUNCSEL` | which peripheral is connected; 31 = disconnected |
    ///
    /// `FUNCSEL` values are per-pin and read out of the pin function table,
    /// but the *slot* assignments are uniform across the bank: 0 JTAG,
    /// 1 SPI, 2 UART, 3 I2C, 4 PWM, **5 SIO**, 6/7/8 PIO0/1/2, 9 XIP,
    /// 10 USB, `0x1f` NULL. Function 5 for GPIO25 is `SIO_25` (p609).
    ///
    /// Writing `5` here is what hands the pin to `Sio`, and it is the only
    /// reason `gpio_out_set` and friends have any effect on it. The reset
    /// value `0x1f` (NULL) means every pin comes out of reset connected to
    /// nothing.
    ///
    /// A whole-register write of `5` is safe during bring-up because it also
    /// zeroes all four override fields, which is what you want. Once the pin
    /// is live, prefer the `+0x2000`/`+0x3000` aliases so you do not clobber
    /// an override some other code set.
    ///
    /// Table 650, p608–609.
    pub ctrl: u32,
}

/// `IO_BANK0` — the user-GPIO function-select block. Base `0x4002_8000`.
///
/// Held in reset until you clear bit 6 of `RESETS.RESET`; see
/// [`crate::common::reset`].
///
/// The QSPI pins have their own separate `IO_QSPI` block at a different base;
/// this struct covers only the 48 general-purpose pins.
#[repr(C)]
struct IoBank{
    /// `GPIO0_STATUS` … `GPIO47_CTRL` — `0x000`–`0x17f`.
    ///
    /// Index by GPIO number: `gpio[25].ctrl` is `GPIO25_CTRL` at `0x0cc`.
    ///
    /// The array is 48 entries even on packages that bond out fewer pins. The
    /// Pico 2 exposes GPIO0–29; 30–47 exist in the register map but have no
    /// package pin. Indexing past 47 is out-of-bounds on the struct and will
    /// silently walk into the reserved region below, so bounds-check the pin
    /// number at the API boundary rather than trusting the caller.
    pub gpio: [GpioRegs; 48],
    /// Reserved, `0x180`–`0x1ff`. 32 words of address hole.
    ///
    /// Present only so the fields after it land at the right offsets. Do not
    /// read or write it.
    _reserved: [u32; 32],
    /// `IRQSUMMARY_*` — `0x200`–`0x22f`. **Read-only.**
    ///
    /// Twelve words: for each of six interrupt destinations, a pair of words
    /// covering GPIO0–31 and GPIO32–47.
    ///
    /// | Index | Register | Offset |
    /// |-------|----------|--------|
    /// | 0, 1  | `IRQSUMMARY_PROC0_SECURE0/1`      | `0x200`, `0x204` |
    /// | 2, 3  | `IRQSUMMARY_PROC0_NONSECURE0/1`   | `0x208`, `0x20c` |
    /// | 4, 5  | `IRQSUMMARY_PROC1_SECURE0/1`      | `0x210`, `0x214` |
    /// | 6, 7  | `IRQSUMMARY_PROC1_NONSECURE0/1`   | `0x218`, `0x21c` |
    /// | 8, 9  | `IRQSUMMARY_COMA_WAKE_SECURE0/1`  | `0x220`, `0x224` |
    /// | 10,11 | `IRQSUMMARY_COMA_WAKE_NONSECURE0/1` | `0x228`, `0x22c` |
    ///
    /// One bit per GPIO, set when that pin has *any* pending, enabled
    /// interrupt for that destination. This is the register an interrupt
    /// handler reads first to find out which pin fired, before going to
    /// [`intr`](Self::intr) to find out which of the four edge/level events
    /// it was. Unused until you enable GPIO interrupts.
    ///
    /// p687–697.
    pub irqsummary: [u32; 12],
    /// `INTR0` … `INTR5` — `0x230`–`0x247`. Raw interrupt status, write-1-to-clear.
    ///
    /// Each GPIO gets **four** bits — level-low, level-high, edge-low,
    /// edge-high — so one 32-bit word covers eight pins, and six words cover
    /// all 48. GPIO *n*'s bits live in `intr[n / 8]` at bit positions
    /// `(n % 8) * 4 + event`.
    ///
    /// "Raw" means pre-mask: a bit sets here whether or not the corresponding
    /// interrupt is enabled. The two edge bits latch and must be cleared by
    /// writing a 1 back to them; the two level bits follow the pin and clear
    /// themselves when the condition goes away.
    ///
    /// p697+.
    pub intr: [u32; 6]
}

/// `PADS_BANK0` — the electrical properties of each user GPIO pad.
/// Base `0x4003_8000`.
///
/// Held in reset until you clear bit 9 of `RESETS.RESET`; see
/// [`crate::common::reset`].
///
/// Where [`IoBank`] decides *which signal* reaches the pin, this block decides
/// *how the pin behaves electrically*: whether the input buffer is powered,
/// whether the output driver is allowed to drive, how hard it drives, and
/// which pull resistor is attached. Both blocks must be configured; getting
/// one right and the other wrong produces a pin that looks correct in the
/// register dump and does nothing in the real world.
#[repr(C)]
struct PadsBank{
    /// `VOLTAGE_SELECT` — `0x00`. Bank-wide, reset `0x0`.
    ///
    /// Bit 0 sets the IO voltage threshold for **every** pad in the bank:
    /// `0` = 3.3 V (requires `DVDD >= 2.5 V`), `1` = 1.8 V (requires
    /// `DVDD <= 1.8 V`). All other bits reserved.
    ///
    /// The Pico 2 runs its IO rail at 3.3 V, so the reset value is already
    /// correct and this register should be left alone. Setting it to 1 on a
    /// 3.3 V board misconfigures the input thresholds for all 48 pads at
    /// once. Table 851, p785.
    pub voltage_select: u32,
    /// `GPIO0` … `GPIO47` — `0x04`–`0xc0`. One pad-control word per GPIO.
    ///
    /// Note the offset: the array starts at `0x04`, *after*
    /// [`voltage_select`](Self::voltage_select), so `pads[25]` (GPIO25) is at
    /// `0x68` — index by GPIO number, and the struct handles the `+4`.
    ///
    /// | Bits | Name       | Reset | Meaning |
    /// |------|------------|-------|---------|
    /// | 8    | `ISO`      | `1`   | pad isolation latch; **1 = pad cut off from the chip** |
    /// | 7    | `OD`       | `0`   | output disable; overrides any peripheral output enable |
    /// | 6    | `IE`       | `0`   | input enable; powers the input buffer |
    /// | 5:4  | `DRIVE`    | `0x1` | 0 = 2 mA, 1 = 4 mA, 2 = 8 mA, 3 = 12 mA |
    /// | 3    | `PUE`      | `0`   | pull-up enable |
    /// | 2    | `PDE`      | `1`   | pull-down enable |
    /// | 1    | `SCHMITT`  | `1`   | Schmitt-trigger (hysteresis) on the input |
    /// | 0    | `SLEWFAST` | `0`   | 1 = fast slew, 0 = slow |
    ///
    /// Reset value is therefore `0x116`: isolated, pulled down, hysteresis on,
    /// 4 mA, input buffer off.
    ///
    /// Two of these bits are the classic reasons a freshly configured pin does
    /// nothing:
    ///
    /// - **`ISO` starts at 1.** The pad is latched into a safe state until
    ///   software says otherwise. Clear it *last*, after `FUNCSEL` is set, so
    ///   the pin never briefly drives a value from a half-configured state.
    /// - **`IE` must be set even for an output.** The input buffer is what
    ///   feeds `GPIO_IN`, so with `IE` clear you can drive the pin but never
    ///   read back what it is actually doing (§9.3, p586).
    ///
    /// **Always read-modify-write this register.** A whole-register write is
    /// how you accidentally clear `ISO` too early, or drop `DRIVE` and
    /// `SCHMITT` back to zero while intending only to touch `IE`. The
    /// alternative is the `+0x2000`/`+0x3000` atomic aliases, which change
    /// exactly the bits you name.
    ///
    /// Table 852, p785.
    pub pads: [u32; 48],
    /// `SWCLK` — `0xc4`. Pad control for the SWD clock pin.
    ///
    /// Same bit layout as [`pads`](Self::pads), but a different reset value,
    /// because the debug pins come up already usable. Touching this while a
    /// debugger is attached will drop the connection; on a board you flash
    /// over USB it is simply not yours to configure.
    pub swclk: u32,
    /// `SWD` — `0xc8`. Pad control for the SWD data pin. See
    /// [`swclk`](Self::swclk).
    pub swd: u32,
}

/// `SIO` — Single-cycle IO. Base `0xd000_0000`.
///
/// SIO is not a peripheral on the APB bus; it is attached directly to each
/// core's IO port, so a read or write costs one cycle rather than the three or
/// four an APB access takes. That is the entire reason GPIO bit-banging on
/// this chip is fast, and it is also why the atomic `+0x1000`/`+0x2000`/
/// `+0x3000` aliases described in §2.1.3 **do not exist here**. SIO instead
/// bakes set/clear/XOR in as separate registers, which is why this struct has
/// four registers for output and four for output-enable.
///
/// SIO does not need to be released from reset — it is part of the core
/// complex, not a resettable subsystem, and has no bit in `RESETS.RESET`.
///
/// Each `*_hi` register is the datasheet's `GPIO_HI_*`, covering GPIO32–47
/// plus the QSPI and USB pins. On a Pico 2, which only bonds out GPIO0–29,
/// they are unused.
///
/// SIO only reaches a pin whose `GPIOn_CTRL.FUNCSEL` is 5. Until then, writes
/// here are accepted and have no visible effect.
///
/// Table 16, p55–56.
#[repr(C)]
struct Sio{
    /// `CPUID` — `0x000`. **Read-only.**
    ///
    /// Reads `0` on core 0 and `1` on core 1. The address is the same for both
    /// cores; the value differs by which core issued the read, which makes it
    /// the cheapest possible "which core am I?" — one load, no branch, no
    /// shared state. Single-core code can ignore it.
    pub cpuid:           u32,  // 0x000
    /// `GPIO_IN` — `0x004`. **Read-only.** Live input level of GPIO0–31.
    ///
    /// Bit *n* is the voltage actually present on pin *n*, sampled through the
    /// pad input buffer. This is the truth about the pin, as opposed to
    /// [`gpio_out`](Self::gpio_out), which only reports what you last asked
    /// for. On an output pin the two agree unless something external is
    /// fighting your driver — a short, or a load too heavy for the configured
    /// `DRIVE` strength. Comparing them is a real diagnostic.
    ///
    /// Reads `0` for any pin whose `IE` bit is clear in [`PadsBank::pads`],
    /// because the input buffer is unpowered. Also reads `0` from Non-secure
    /// code for pins marked Secure-only in `ACCESSCTRL`.
    pub gpio_in:         u32,  // 0x004
    /// `GPIO_HI_IN` — `0x008`. **Read-only.** As
    /// [`gpio_in`](Self::gpio_in), for GPIO32–47, the QSPI pins, and the USB
    /// pins. Unused on Pico 2.
    pub gpio_in_hi:      u32,  // 0x008
    /// Reserved — `0x00c`. An address hole; `FIFO_ST` is at `0x050`, not here.
    ///
    /// Declared only to push the following fields to their correct offsets.
    _reserved:           u32,  // 0x00c  (FIFO_ST is at 0x050; 0x00c is a hole)
    /// `GPIO_OUT` — `0x010`. Output *level* for GPIO0–31. 1 = high, 0 = low.
    ///
    /// **Reading this gives back the last value written, not the pin state.**
    /// For the pin state, read [`gpio_in`](Self::gpio_in).
    ///
    /// Writing this register sets all 32 pins at once, which is what you want
    /// for a parallel bus and what you almost never want for a single pin.
    /// Prefer [`gpio_out_set`](Self::gpio_out_set),
    /// [`gpio_out_clr`](Self::gpio_out_clr), and
    /// [`gpio_out_xor`](Self::gpio_out_xor): each is a single store that
    /// provably cannot disturb a pin you did not name, whereas a
    /// read-modify-write here can lose a concurrent update from an interrupt
    /// handler or the other core.
    ///
    /// If both cores write simultaneously — here or through any of the
    /// aliases — the result is as though core 0's write landed first and
    /// core 1's was applied on top.
    pub gpio_out:        u32,  // 0x010
    /// `GPIO_HI_OUT` — `0x014`. As [`gpio_out`](Self::gpio_out), for
    /// GPIO32–47, QSPI, and USB pins.
    pub gpio_out_hi:     u32,  // 0x014
    /// `GPIO_OUT_SET` — `0x018`. **Write-only in effect.** Atomic bit-set:
    /// `gpio_out |= wdata`.
    ///
    /// Writing `1 << n` drives pin *n* high. Zero bits leave their pins
    /// untouched, so this is the one-instruction way to raise a pin without a
    /// read-modify-write.
    pub gpio_out_set:    u32,  // 0x018
    /// `GPIO_HI_OUT_SET` — `0x01c`. Atomic bit-set on
    /// [`gpio_out_hi`](Self::gpio_out_hi).
    pub gpio_out_set_hi: u32,  // 0x01c
    /// `GPIO_OUT_CLR` — `0x020`. Atomic bit-clear: `gpio_out &= ~wdata`.
    ///
    /// Writing `1 << n` drives pin *n* low. Note the polarity — you write a
    /// **1** in the position you want cleared.
    ///
    /// Worth doing *before* enabling the output in
    /// [`gpio_oe_set`](Self::gpio_oe_set), so the pin's first driven state is
    /// a known low rather than whatever `gpio_out` happened to hold.
    pub gpio_out_clr:    u32,  // 0x020
    /// `GPIO_HI_OUT_CLR` — `0x024`. Atomic bit-clear on
    /// [`gpio_out_hi`](Self::gpio_out_hi).
    pub gpio_out_clr_hi: u32,  // 0x024
    /// `GPIO_OUT_XOR` — `0x028`. Atomic XOR: `gpio_out ^= wdata`.
    ///
    /// Writing `1 << n` inverts pin *n*. This is a whole blink cycle in one
    /// store — no read, no branch, no state kept in software. Toggling by
    /// reading [`gpio_out`](Self::gpio_out) and writing the complement costs
    /// three instructions and can race; this cannot.
    pub gpio_out_xor:    u32,  // 0x028
    /// `GPIO_HI_OUT_XOR` — `0x02c`. Atomic XOR on
    /// [`gpio_out_hi`](Self::gpio_out_hi).
    pub gpio_out_xor_hi: u32,  // 0x02c
    /// `GPIO_OE` — `0x030`. Output *enable* for GPIO0–31.
    /// 1 = drive the pin, 0 = leave it as an input.
    ///
    /// Reads back the last value written, like
    /// [`gpio_out`](Self::gpio_out).
    ///
    /// This is the SIO half of "make the pin an output." It is not sufficient
    /// on its own: `OD` must be clear in [`PadsBank::pads`] and `FUNCSEL` must
    /// be 5 in [`GpioRegs::ctrl`], or the enable never reaches the pad. `OD`
    /// in particular has priority over this register.
    ///
    /// As with `GPIO_OUT`, prefer the set/clear/XOR aliases below over
    /// read-modify-write.
    pub gpio_oe:         u32,  // 0x030
    /// `GPIO_HI_OE` — `0x034`. As [`gpio_oe`](Self::gpio_oe), for GPIO32–47,
    /// QSPI, and USB pins.
    pub gpio_oe_hi:      u32,  // 0x034
    /// `GPIO_OE_SET` — `0x038`. Atomic bit-set: `gpio_oe |= wdata`.
    ///
    /// Writing `1 << n` turns pin *n* into an output. Set the level first
    /// via [`gpio_out_clr`](Self::gpio_out_clr) or
    /// [`gpio_out_set`](Self::gpio_out_set) so the pin does not briefly drive
    /// a stale value at the moment it becomes an output.
    pub gpio_oe_set:     u32,  // 0x038
    /// `GPIO_HI_OE_SET` — `0x03c`. Atomic bit-set on
    /// [`gpio_oe_hi`](Self::gpio_oe_hi).
    pub gpio_oe_set_hi:  u32,  // 0x03c
    /// `GPIO_OE_CLR` — `0x040`. Atomic bit-clear: `gpio_oe &= ~wdata`.
    ///
    /// Writing `1 << n` releases pin *n* back to a high-impedance input. This
    /// is how you stop driving a pin without disturbing any other.
    pub gpio_oe_clr:     u32,  // 0x040
    /// `GPIO_HI_OE_CLR` — `0x044`. Atomic bit-clear on
    /// [`gpio_oe_hi`](Self::gpio_oe_hi).
    pub gpio_oe_clr_hi:  u32,  // 0x044
    /// `GPIO_OE_XOR` — `0x048`. Atomic XOR: `gpio_oe ^= wdata`.
    ///
    /// Writing `1 << n` flips pin *n* between driving and high-impedance.
    /// Useful for open-drain-style signalling done in software: hold
    /// `gpio_out` low permanently and toggle the enable, so the pin
    /// alternates between actively pulling low and floating to a pull-up.
    pub gpio_oe_xor:     u32,  // 0x048
    /// `GPIO_HI_OE_XOR` — `0x04c`. Atomic XOR on
    /// [`gpio_oe_hi`](Self::gpio_oe_hi).
    pub gpio_oe_xor_hi:  u32,  // 0x04c
}
