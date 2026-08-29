---
document_type: "Tutorial Chapter — First Blink"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 8 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-07
sources: RP2350 datasheet §2.2.4 (Table 12, PDF p33), §2.2.6 (Table 14, PDF p35),
  §3.1.11 (Table 16, PDF p55-56), §7.5.2 (PDF p503), §7.5.3 (Tables 533/534/536,
  PDF p504-506), §8.1.2.1 (Table 540, PDF p516), §8.3.1 (PDF p560), §8.5
  (Table 623, PDF p571), §9.3 (PDF p586-587), §9.6.1 (PDF p594), §9.10.1
  (PDF p596-601), §9.11.1 (Tables 699/700, PDF p604, p649-651), §9.11.3
  (Tables 850/877, PDF p783-785, p798); errata RP2350-E9 (PDF p1358-1359);
  Pico 2 datasheet p9, p10
creates: firmware/pico2/src/common/mod.rs, firmware/pico2/src/common/reg.rs,
  firmware/pico2/src/common/reset.rs, firmware/pico2/src/gpio/mod.rs,
  firmware/pico2/src/gpio/gpio.rs; adds two lines to firmware/pico2/src/lib.rs;
  finishes demo/src/main.rs
---

# Chapter 08 — First Blink

Everything so far has been infrastructure: a linker script, a boot block, a
vector table, a reset handler that reaches the application with a stack under
it. None of it is observable from outside the chip. This chapter makes one pin
move — six register-level steps, wrapped in the driver and trait layer the
tree actually ships, ending with the application that blinks. Citation and
callout conventions are the index's; so is the rule that listings from the
tree elide its `///` doc comments and quote code lines byte-for-byte.

## 8.1 The goal — GP25, the on-board user LED

The Pico 2 wires one LED to a chip GPIO: *"GPIO25 OP — Connected to user LED"*
(Pico 2 datasheet p9). It is not on a header — GP23 through GP25 are
board-reserved — and its only external access is a test point,
*"TP5 GPIO25/LED (not recommended to be used)"* (p10). You do not need it. The
LED is on the board and you drive it from the inside.

Four blocks stand between a store instruction and that LED. They are not peers;
they are layers a signal passes through:

```text
CPU --> SIO --------> IO_BANK0 --------> PADS_BANK0 --------> pin
        the value      the mux            the pad
        (level and     (which peripheral  (drive, pulls, input
         direction)     owns the pin)      enable, isolation)
```

A fifth block, `RESETS`, sits outside that path and decides whether the others
answer at all.

| Block | Base | Role |
|---|---|---|
| `RESETS` | `0x40020000` | gates whether the next three respond to anything |
| `SIO` | `0xd0000000` | the value and the direction |
| `IO_BANK0` | `0x40028000` | the mux — which peripheral is connected |
| `PADS_BANK0` | `0x40038000` | the pad — drive, pulls, input enable, isolation |

The first, third and fourth come from the APB address map (§2.2.4, Table 12,
PDF p33). `SIO` is not on APB and comes from Table 14 (§2.2.6, PDF p35).

## 8.2 The whole sequence, up front

Six steps. Nothing else is required to light the LED.

| # | Block | Action | Section |
|---|---|---|---|
| 1 | `RESETS.RESET` | deassert bits 6 and 9, then poll `RESET_DONE` for both | §8.5, §8.6 |
| 2 | `SIO.GPIO_OUT_CLR` | write `1 << 25` — start the pin low | §8.7 |
| 3 | `SIO.GPIO_OE_SET` | write `1 << 25` — make it an output | §8.7 |
| 4 | `PADS_BANK0.GPIO25` | `IE = 1`, `OD = 0` — wake the pad | §8.8 |
| 5 | `IO_BANK0.GPIO25_CTRL` | `FUNCSEL = 5` — hand the pin to SIO | §8.9 |
| 6 | `PADS_BANK0.GPIO25` | clear `ISO` — connect the pad to the outside | §8.10 |

Then drive the pin high and low forever through `SIO.GPIO_OUT_SET` and
`SIO.GPIO_OUT_CLR` (§8.11, §8.12).

One rule orders the list. A block whose reset is asserted discards every bus
write, so nothing before step 1 can have any effect; and while `ISO` is set,
the pad's control inputs are latched at their old values, so nothing between
steps 2 and 5 reaches the physical pin. Steps 2 through 5 therefore configure
the pin while it is provably disconnected, and step 6 — the `ISO` clear — is
the single moment the finished configuration propagates to the pad. No
half-configured state can ever appear on the wire. That is the order
`configure_gpio_pin_out` writes them in; §8.10.1 is honest about one subtlety
inside it.

In the tree, step 1 belongs to the *port* — it runs once, in `Block::start`,
for the whole GPIO bank — and steps 2 through 6 belong to the *pin*, running
once per configured pin. The chapter follows that split.

## 8.3 The files, and the address constants

This chapter creates five files under `firmware/pico2/src/` and finishes
`demo/src/main.rs`:

```text
firmware/pico2/src/lib.rs        exists — chapters 05, 06; gains two lines below
firmware/pico2/src/common/mod.rs new — §8.3
firmware/pico2/src/common/reg.rs new — §8.3
firmware/pico2/src/common/reset.rs new — §8.5
firmware/pico2/src/gpio/mod.rs   new — §8.4
firmware/pico2/src/gpio/gpio.rs  new — §8.6 onward
demo/src/main.rs                 exists — chapter 06; finished in §8.12
```

Rust does not find a file because it is on disk; a parent module has to declare
it. The two declarations that pull `common` and `gpio` into the `pico2` crate
go into `lib.rs`, directly under its `use` line — the last edit `lib.rs` ever
gets:

```rust
pub mod common;
pub mod gpio;
```

`src/common/mod.rs` declares its two children and holds one constant — code
lines verbatim (the tree documents the constant at length):

```rust
pub mod reg;
pub mod reset;

pub const MAX_GPIO_PIN: usize = 30;
```

`MAX_GPIO_PIN` is a *package* fact, not a chip fact. RP2350 comes in two
packages: the RP2350A (QFN-60) bonds out 30 user GPIOs, the RP2350B (QFN-80)
bonds out 48 (datasheet §9.3, PDF p586), and the Pico 2 carries the RP2350A.
The register maps are identical either way — `IO_BANK0` and `PADS_BANK0` are
48 entries wide in both packages — so writing to pin 40 on this board succeeds
at the bus level, reports no error, and drives a pad connected to no leg of
the chip. The hardware will never tell you; this constant is how the driver
does (§8.11).

The four base addresses live in one enum — `firmware/pico2/src/common/reg.rs`,
code lines verbatim:

```rust
#[repr(usize)]
#[derive(Clone, Copy)]
// Variant names deliberately match the datasheet's block names exactly, so
// code can be checked against the register listings without translation.
#[allow(non_camel_case_types)]
pub enum RegAddr {
    RESET = 0x4002_0000,
    IO_BANK0 = 0x4002_8000,
    PADS_BANK0 = 0x4003_8000,
    SIO = 0xd000_0000,
}
```

`#[repr(usize)]` pins each discriminant to pointer width, which makes the cast
at every use site lossless:

```rust
let reset_addr = RegAddr::RESET as usize as *mut Reset;
```

Two casts, not one: `as usize` extracts the discriminant, `as *mut Reset`
reinterprets it as a pointer. The `#[allow(non_camel_case_types)]` silences
the lint the SCREAMING_SNAKE names would otherwise trip; chapter 03 §3.4
checks all four values against the datasheet's address map.

## 8.4 The register structs

`firmware/pico2/src/gpio/mod.rs` is one module declaration and four
`#[repr(C)]` structs. They are maps: point one at a base address and index it.
Chapter 07 §7.6 explains why they are shaped this way and chapter 09 keeps the
field-by-field reference; here they are only the thing you index. Code lines
verbatim — in the tree, every field carries a doc comment quoting its
datasheet table:

```rust
pub mod gpio;

#[repr(C)]
struct GpioRegs{
    pub status: u32,
    pub ctrl: u32,
}

#[repr(C)]
struct IoBank{
    pub gpio: [GpioRegs; 48],
    _reserved: [u32; 32],
    pub irqsummary: [u32; 12],
    pub intr: [u32; 6]
}

#[repr(C)]
struct PadsBank{
    pub voltage_select: u32,
    pub pads: [u32; 48],
    pub swclk: u32,
    pub swd: u32,
}

#[repr(C)]
struct Sio{
    pub cpuid:           u32,  // 0x000
    pub gpio_in:         u32,  // 0x004
    pub gpio_in_hi:      u32,  // 0x008
    _reserved:           u32,  // 0x00c  (FIFO_ST is at 0x050; 0x00c is a hole)
    pub gpio_out:        u32,  // 0x010
    pub gpio_out_hi:     u32,  // 0x014
    pub gpio_out_set:    u32,  // 0x018
    pub gpio_out_set_hi: u32,  // 0x01c
    pub gpio_out_clr:    u32,  // 0x020
    pub gpio_out_clr_hi: u32,  // 0x024
    pub gpio_out_xor:    u32,  // 0x028
    pub gpio_out_xor_hi: u32,  // 0x02c
    pub gpio_oe:         u32,  // 0x030
    pub gpio_oe_hi:      u32,  // 0x034
    pub gpio_oe_set:     u32,  // 0x038
    pub gpio_oe_set_hi:  u32,  // 0x03c
    pub gpio_oe_clr:     u32,  // 0x040
    pub gpio_oe_clr_hi:  u32,  // 0x044
    pub gpio_oe_xor:     u32,  // 0x048
    pub gpio_oe_xor_hi:  u32,  // 0x04c
}
```

The fifth struct, `Reset`, lives beside the code that uses it in
`common/reset.rs` (§8.5). Every field everywhere is a `u32` or an array of
them, so alignment is 4 and `#[repr(C)]` inserts no padding. The offsets this
chapter uses:

| Offset | Name | Info |
|---|---|---|
| `RESETS + 0x0` | `RESET` | `Reset.reset` — Table 533, PDF p504 |
| `RESETS + 0x8` | `RESET_DONE` | `Reset.reset_done` — Table 533, PDF p504 |
| `SIO + 0x018` | `GPIO_OUT_SET` | `Sio.gpio_out_set` — Table 16, PDF p55 |
| `SIO + 0x020` | `GPIO_OUT_CLR` | `Sio.gpio_out_clr` — Table 16, PDF p55 |
| `SIO + 0x038` | `GPIO_OE_SET` | `Sio.gpio_oe_set` — Table 16, PDF p56 |
| `SIO + 0x040` | `GPIO_OE_CLR` | `Sio.gpio_oe_clr` — Table 16, PDF p56 |
| `PADS_BANK0 + 0x04 + 4n` | `GPIOn` | `PadsBank.pads[n]` — Table 850, PDF p783 |
| `IO_BANK0 + 0x000 + 8n` | `GPIOn_STATUS` | `IoBank.gpio[n].status` — Table 648, PDF p603-604 |
| `IO_BANK0 + 0x004 + 8n` | `GPIOn_CTRL` | `IoBank.gpio[n].ctrl` — Table 648, PDF p603-604 |

The firmware's SIO field names transpose the datasheet's — the code writes
`gpio_out_hi` where the datasheet says `GPIO_HI_OUT`. Same layout, different
spelling; chapter 09 §9.5 uses the datasheet's.

## 8.5 Step 1, part one — the reset helpers

Every peripheral powers up **held in reset**. Before `IO_BANK0` or
`PADS_BANK0` acknowledge a single write, their reset bits must be deasserted
and the hardware must confirm. The machinery for that is chip-wide, not
GPIO-specific, so it lives in its own file: `firmware/pico2/src/common/reset.rs`
— the `Reset` struct (§8.4) and three free functions. Code lines verbatim,
tree comments kept:

```rust
use crate::common::reg::RegAddr;

#[repr(C)]
struct Reset{
    pub reset: u32,
    pub wdsel: u32,
    pub reset_done: u32
}

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
```

Read the pair as two directions on one register. `clr_reset_reg` computes
`RESET &= mask` — it can only move bits from 1 to 0, so it can only *release*
resets, and a caller releasing blocks passes the **complement** of the bits it
wants freed. `set_reset_reg` computes `RESET |= mask` — the mask is passed
uncomplemented, and it can only *assert* resets. The relevant registers, from
`RESETS` at `0x40020000` (Table 533, PDF p504):

| Register | Bit = 1 means | You |
|---|---|---|
| `RESET` | held **in** reset | write `0` to release |
| `RESET_DONE` | **out of** reset, ready | wait for `1` |

Same bit numbering, opposite sense. Both GPIO bits are `1` at power-on, so
releasing them means clearing — the verb is **deassert**. The read-modify-write
in `clr_reset_reg` is what protects the other 27 components' bits — `RESET`
defines bits 28:0 (Table 534, PDF p504-505) — and the poll in
`wait_for_reset_done` spins until *both* requested bits appear:
`& mask != mask` is false only when the masked read equals the full mask
(`!= 0` would fall through when the first block wakes). There is no interrupt
for this; datasheet §7.5.2 states the intent — *"This allows software to wait
for this status bit in case the component requires initialisation before
use."* (PDF p503). The `read_volatile` inside the loop condition is mandatory:
through a plain reference, LLVM may hoist the load out of the loop — nothing
in the language says ordinary memory changes behind your back — and the wait
compiles to a branch-to-self.

> **Silent-failure trap.** While a block is in reset its registers do not
> respond. Writes are discarded — no error, no bus fault, no diagnostic — and
> reads return reset values, so a debugger shows you a plausible register block
> that is simply not listening. Every store to `0x40028000` or `0x40038000`
> before step 1 completes goes nowhere. Step 1 is not housekeeping; it is the
> moment those two peripherals become addressable.

> **Hardware-destructive.** In the same register, bit 7 is `IO_QSPI` and bit 10
> is `PADS_QSPI` (Table 534, PDF p504). Those are the pins your flash is on,
> and you are executing from that flash. Assert either — one wrong bit in a
> `set_reset_reg` mask — and the image running the instruction stops existing:
> execution halts mid-instruction-fetch with no fault and no output. This is
> the reset-controller face of the hazard chapter 03 §3.4 raised for `IO_QSPI`
> and `PADS_QSPI` themselves; chapter 09 §9.2.3 has the bit table.

Everything else about `RESETS` — the `+0x3000` atomic-clear alias as an
alternative to the read-modify-write (the alias mechanism itself is chapter 07
§7.5), assert-then-deassert as an init idiom, and why reset is the wrong
granularity for a resource allocator — is chapter 09 §9.2, and none of it is
needed to blink.

## 8.6 Step 1, part two — the port and its `Block` impl

Now start `firmware/pico2/src/gpio/gpio.rs`, the driver proper. Its head —
six imports, verbatim, and nothing else above them:

```rust
use core::fmt::Debug;

use crate::common::MAX_GPIO_PIN;
use crate::common::reset::{clr_reset_reg, set_reset_reg, wait_for_reset_done};
use crate::gpio::{IoBank, PadsBank, Sio};
use crate::common::reg::RegAddr;
use api::common::{Block, ErrorType, Read, Write};
use api::gpio::{Gpio, Pull};
```

The third line is the one worth a sentence, because it looks like it should
not compile. Look back at §8.4: those structs are declared `struct`, not
`pub struct` — they are **private to `gpio/mod.rs`**. A private item is
visible inside the module that declares it *and inside that module's
descendants*, and `gpio::gpio` is a child of `gpio`, so `gpio.rs` can name
`IoBank`, `PadsBank` and `Sio` while nothing outside the `gpio` module can.
That is what keeps the register maps from leaking into the rest of the crate
while still letting the driver use them. The last two lines are the `api`
traits from chapter 07 §7.7 — this file exists to implement them.

The port itself is a zero-sized type. There is no state to hold — the
registers live at fixed addresses — so the struct exists to give the trait
implementations somewhere to live:

```rust
pub struct Rp2350Gpio
{

}
```

Which bits the port owns are module-level constants, verbatim (the `//` lines
are the tree's):

```rust
// Bit for IOBANK
const IOBANK_RESET_BIT:u8 = 6;
// Bit for Pad
const PADBANK_RESET_BIT:u8 = 9;
// IO Pad bitmask
const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;
```

`IO_BANK0` is bit 6 and `PADS_BANK0` is bit 9 of `RESETS.RESET` (Table 534,
PDF p504); `IO_PAD_BITMASK` is `(1 << 6) | (1 << 9)` = **`0x240`** — `<<`
binds tighter than `|`, so the unparenthesised constant is correct. Neither
bit alone is sufficient: `IO_BANK0` routes the signal and `PADS_BANK0`
connects it to a physical leg of the package.

Step 1 is then the `Block` implementation — `api`'s bring-up lifecycle trait,
with `start` releasing the blocks and `stop` putting them back:

```rust
impl Block for Rp2350Gpio
{
    unsafe fn start(&mut self) {
        unsafe{
            clr_reset_reg(!IO_PAD_BITMASK);
            wait_for_reset_done(IO_PAD_BITMASK);
        }
    }

    unsafe fn stop(&mut self) {
        unsafe{
            set_reset_reg(IO_PAD_BITMASK);
        }
    }
}
```

Note the complement in `start` and its absence in `stop` — the polarity
gymnastics of §8.5 in two lines. `clr_reset_reg` ANDs, so the zeros in
`!IO_PAD_BITMASK` are the bits being freed; `set_reset_reg` ORs, so
`IO_PAD_BITMASK` is passed straight. `stop` has no
`wait_for_reset_done` counterpart because `RESET_DONE` reports readiness, and
a block held in reset simply never reports ready. Who calls `start`? The
application does, exactly once, through `Board::take` — §8.12. Nothing in this
crate calls it.

## 8.7 Steps 2 and 3 — initial level and direction, before the mux

Two stores to SIO, from `configure_gpio_pin_out` (whole function in §8.10):

```rust
        // Drive low first, then enable the driver: no glitch high.
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
```

`GPIO_OUT_CLR` is `SIO + 0x020` and `GPIO_OE_SET` is `SIO + 0x038` (Table 16,
§3.1.11, PDF p55-56); for pin 25 both take `1 << 25` = `0x02000000`. These are
write-one-to-act registers — the bits you write are the bits that change — so
neither store needs a read first and the other 31 GPIOs are untouched. The pin
is now an output driving low, decided before anything connects it to the world:
level first, then enable, so the pin's first driven state is a known low
rather than whatever `GPIO_OUT` happened to hold.

Why `0xd0000000` and not the APB range? Because SIO is not on APB: §2.2.6 calls
it core-local, *"accessible to processor load/store only… Access is always
zero-wait-state"* (PDF p34), while an APB write costs *"a minimum of four
cycles"* (§2.2.4, PDF p33). The value and the direction, written on every
blink, live off the peripheral bus; the mux and the pad, written once, stay on
it. The full SIO tour is chapter 09 §9.5.

## 8.8 Step 4 — the pad: `IE = 1`, `OD = 0`

```rust
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        // OD must be clear or the pad refuses to drive regardless of SIO.
        current_pad &= !(1 << OD);
        // IE on so the pin can also be read back; see the doc comment.
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
```

```text
PADS_BANK0 + 0x04 + 4*25 = 0x40038000 + 0x04 + 0x64 = 0x40038068
```

Table 850 lists `GPIO25` at offset `0x68` (datasheet §9.11.3, PDF p784), so the
arithmetic and the table agree. The `+0x04` is not decoration: offset `0x00` is
`VOLTAGE_SELECT`, a bank-wide control, so a pad array indexed from the block
base lands on it — changing the input threshold for the whole bank and
configuring no pin at all. `PadsBank` encodes the `+0x04` structurally by
declaring `voltage_select` before `pads`, which is why `pads[25]` is right.

The three bits this chapter touches, from `PADS_BANK0: GPIO25` (Table 877,
PDF p798 — the datasheet prints one table per pin and they are identical;
chapter 09 §9.3 quotes the same layout from `GPIO0`, Table 852, PDF p785):

| Bits | Field | Type | Reset |
|---|---|---|---|
| 8 | `ISO: Pad isolation control. Remove this once the pad is configured by software.` | RW | `0x1` |
| 7 | `OD: Output disable. Has priority over output enable from peripherals` | RW | `0x0` |
| 6 | `IE: Input enable` | RW | `0x0` |

`OD` already resets to `0`, so clearing it is a no-op on a fresh pin and costs
nothing to make explicit. `IE` resets to `0` and must be set — a requirement,
not a preference, from datasheet §9.3, Reset State:

> *"Applications must enable the pad input (`GPIO0.IE` = 1) and disable pad
> isolation latches (`GPIO0.ISO` = 0) before using the pads for digital I/O."*
> (datasheet §9.3, PDF p587)

An output-only pin still wants `IE = 1`: the input buffer feeds `GPIO_IN` and
`GPIOn_STATUS.INFROMPAD`, which is how `Read::read` (§8.11) and §8.15's
debugging read back what the pin is actually doing. The read-modify-write
earns its keep — this register resets to `0x116` (`ISO=1`, `DRIVE=0x1`,
`PDE=1`, `SCHMITT=1`), so a plain write of `0x40` would silently drop the
pull-down and the Schmitt trigger. After this step it holds `0x156`. The
remaining pad fields are chapter 09 §9.3.

## 8.9 Step 5 — the mux: `FUNCSEL = 5`

```rust
        // FUNCSEL = 5 connects the pin to SIO. Writing the whole register also
        // clears IRQOVER/INOVER/OEOVER/OUTOVER to "normal", which is wanted.
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
```

```text
IO_BANK0 + 0x004 + 8*25 = 0x40028000 + 0x004 + 0xc8 = 0x400280cc
```

The register list confirms both halves of the pair: `GPIO25_STATUS` at `0x0c8`,
`GPIO25_CTRL` at `0x0cc` (Table 648, PDF p604, and again with their field maps
as Tables 699 and 700 on PDF p649-650).

**The stride is 8, not 4.** `IO_BANK0` gives every pin *two* registers, `STATUS`
then `CTRL`, so a pin's control register is at `0x004 + 8n` while its pad
register is at `0x04 + 4n` — two arrays, two strides, two leading offsets,
indexed by the same `pin` a few lines apart. This is the most error-prone
arithmetic in the chapter: get it wrong and you configure a different pin, or
land on a `STATUS` register and write read-only bits, which succeeds and does
nothing. In the firmware both are structural, `gpio[pin].ctrl` and `pads[pin]`,
and `#[repr(C)]` does the multiply.

`FUNCSEL = 5` selects SIO: Table 700 enumerates it for this pin specifically,
`0x05 → SIO_25`, and the field resets to `0x1f` (NULL) (PDF p650-651).

The write is a **plain write of `5`**, not a read-modify-write, and that is
correct here — the tree's comment says why. `GPIO25_CTRL` also carries
`OUTOVER` (bits 13:12), `OEOVER` (15:14), `INOVER` (17:16) and `IRQOVER`
(29:28); all four reset to `0x0`, and `0x0` in each means "take the
peripheral's signal unmodified". Writing bare `5` sets `FUNCSEL` and forces
every override to pass-through in one store — which is also how you recover a
known state on a pin other code has touched. The SDK does the same: *"Zero all
fields apart from fsel; we want this IO to do what the peripheral tells it."*
(datasheet §9.10.1, PDF p600). Full field table is chapter 09 §9.4.

## 8.10 Step 6 — clear `ISO`, and the finished function

```rust
        // Release the isolation latch now that mux and pad are both set.
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
```

Bit 8, cleared with a read-modify-write so the `IE` you just set and the pad's
reset-value fields survive. The register goes from `0x156` to `0x056`.

This is last because of what the latch does. While `ISO` is set, every control
signal crossing from the core domain to the pad — output enable, output level,
pull enables — is held at its latched value, and nothing written upstream
propagates through (datasheet §9.3, PDF p587; the full mechanism is chapter 09
§9.3.1). So steps 2 through 5 configured a pin whose pad was still acting on
its power-on state, whatever order they ran in; clearing `ISO` once, at the
end, lets the pad switch directly from that held state to the complete
configuration. The LED sees one transition into driven-low, never an
intermediate state.

> **Silent-failure trap.** Leave `ISO` set and every register reads back exactly
> as written — `FUNCSEL` says 5, `GPIO_OE` says output — and the pin does
> nothing, because *"the isolation latches prevent upstream signals from
> propagating to the pad"* (datasheet §9.3, PDF p587). Resetting the block does
> not rescue you: *"If a pad's isolation latches are in the latched state then
> resetting the PADS and IO registers does not physically return the pad to its
> reset state."* (datasheet §9.3, PDF p587). Step 1 cannot undo a missing step 6.

The power-domain rationale, what the latches capture, the three ways they trap
you, and the correct teardown order are chapter 09 §9.3.1.

Here is the whole function — verbatim from `firmware/pico2/src/gpio/gpio.rs`,
comments and all; §8.7 through §8.10 quoted pieces of exactly this:

```rust
unsafe fn configure_gpio_pin_out(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        // Drive low first, then enable the driver: no glitch high.
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        // OD must be clear or the pad refuses to drive regardless of SIO.
        current_pad &= !(1 << OD);
        // IE on so the pin can also be read back; see the doc comment.
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
        // Release the isolation latch last.
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
    }
}
```

Statement by statement, with the address each hits for `pin = 25`:

| Statement | Step | Address | Effect |
|---|---|---|---|
| `gpio_out_clr.write_volatile(1 << pin)` | 2 | `0xd0000020` | output value low |
| `gpio_oe_set.write_volatile(1 << pin)` | 3 | `0xd0000038` | output enable on |
| `current_pad &= !(1 << OD)` / `\|= 1 << IE` / `pad.write_volatile` | 4 | `0x40038068` | `OD=0`, `IE=1` (`0x116` → `0x156`) |
| `io_ctrl.write_volatile(SIO)` | 5 | `0x400280cc` | `FUNCSEL = 5`, overrides zeroed |
| `current_pad &= !(1 << ISO)` / `pad.write_volatile` | 6 | `0x40038068` | `ISO=0` (`0x156` → `0x056`) |

Step 1 is not in this function; it ran once for the whole bank in
`Block::start` (§8.6).

The tree also carries the input twin, `configure_gpio_pin_in` — the same
four-phase shape (stop driving via `GPIO_OE_CLR`, set up the pad, `FUNCSEL`,
`ISO` last) with the pad phase configuring the pull resistors instead of the
driver. Its pull logic writes **both** `PUE` (bit 3) and `PDE` (bit 2) on
every arm, verbatim:

```rust
        // Pull resistors, Table 852 p785. PDE resets to 1 and PUE to 0, so a
        // fresh pad already has a pull-down. Every arm therefore writes BOTH
        // bits: setting one without clearing the other leaves both enabled,
        // which is a legal bus-keeper configuration and almost never intended.
        const PUE: u8 = 3;
        const PDE: u8 = 2;
        match pull {
            Pull::Up   => { current_pad |=  1 << PUE; current_pad &= !(1 << PDE); }
            Pull::Down => { current_pad &= !(1 << PUE); current_pad |=  1 << PDE; }
            Pull::None => { current_pad &= !(1 << PUE); current_pad &= !(1 << PDE); }
        }
```

"Bus keeper" is a hardware mode, not a mistake the chip rejects: with both
pull enables set, the pad applies a weak pull *toward whatever logic level the
pin currently reads*, so the pin holds its last driven level instead of
sitting at a defined high or low (datasheet §9.6.1, PDF p594; chapter 09 §9.3
has more). Setting one pull without clearing the other therefore does not
produce an error — it produces that mode, silently, which is why every arm
writes both bits. The blink never calls this function; it is quoted so the
input path does not remain a mystery, and chapter 09 §9.3 holds the rest.

### 8.10.1 The window this order opens, and why it is closed

The firmware sets `IE = 1` in step 4 while `FUNCSEL` is still `0x1f` (NULL),
because the mux is not written until step 5. Errata **RP2350-E9**, *"Increased
leakage current on Bank 0 GPIO when pad input is enabled"* (affects RP2350 A2,
PDF p1358), names exactly that combination. Its four conditions, quoted
(PDF p1359):

> 1. The voltage on the pad is in the undefined logic region.
> 2. Input buffer is enabled in `GPIO0.IE`
> 3. Output buffer is disabled (e.g. selecting the NULL GPIO function)
> 4. Isolation is clear in `GPIO0.ISO`, or the previous were true at the point
>    isolation was set

Conditions 2 and 3 both hold between the step-4 write and the step-5 write.
About 120 µA of leakage then holds a floating pad near 2.2 V at `IOVDD` 3.3 V,
and the internal pull-down is too weak to overcome it.

**Inferred:** condition 4 is not met here, so the errata does not fire. `ISO` is
`1` from power-on through that whole window and is not cleared until step 6,
after `FUNCSEL = 5` has landed — by which time the output buffer is enabled and
condition 3 is false. The pad is isolated for the entire time the dangerous pair
holds, and condition 4's "at the point isolation was set" clause covers
isolation being *set*, not cleared. That is reasoning from the errata text and
the write order; the datasheet does not state it, and it is not a measurement.
Two things make it more comfortable: GP25 drives an LED rather than a floating
net, so condition 1 is doubtful too; and the SDK's `gpio_set_function` writes
the same three registers in the same order (datasheet §9.10.1, PDF p600-601).

**Type it as printed in §8.10.** That is what is in the tree, it is what the
build in §8.13 is measured from, and it works on the board. Do not reorder it
while following this chapter.

The recommendation below is for code you write later, and it is deliberately
not applied here: moving `io_ctrl.write_volatile(SIO)` above the pad `IE`/`OD`
read-modify-write costs nothing and closes the overlap of conditions 2 and 3
outright rather than relying on `ISO` to cover it. It is the shape to reach for
in a driver of your own. **The firmware has not been changed** — the tutorial
documents the tree as it is, and marks the improvement rather than quietly
shipping a listing that no build in this chapter matches.

## 8.11 Owning the pin: the error, the handle, and the traits

The two configuration functions are `unsafe fn` taking a bare `pin: usize` —
nothing about them stops a caller passing 40. The rest of `gpio.rs` is the
layer that makes that impossible to reach, and it is where the `api` traits
earn their place.

**The error.** Validating a pin number is the one thing here that can
genuinely fail, so the port's error type has one variant (doc comments
elided):

```rust
pub enum GpioError
{
    PinOOB {
        pin: usize,
        count: usize,
    },
}
```

`PinOOB` — pin out of bounds — exists because the hardware will not produce an
error itself: the register arrays are 48 wide in every package (§8.3), so a
write for pin 40 succeeds at the bus level and drives a pad bonded to nothing.
Worse, `1 << pin` in the SIO path is a shift, and for `pin >= 32` a shift
overflow: in a release build the shift amount is silently masked to 5 bits, so
pin 33 would toggle pin 1. The tree implements `Debug` for `GpioError` by hand
(a `debug_struct` over the two fields) so `unwrap()` can print it — a decision
with a visible price in §8.13.

**The handle.** A configured pin is a value (doc comments elided):

```rust
pub struct Rp2350GpioPin
{
    pin_no: usize,
}
```

The field is private and both constructors are private to the module, so the
only way to obtain one is through the port's `Gpio` implementation below —
which means holding a `Rp2350GpioPin` is proof that the pin number was
validated and the hardware configured, because construction is the only place
either happens. The constructors check first, configure second — `new_output`
verbatim:

```rust
    fn new_output(pin_no: usize) -> Result<Self, GpioError>
    {
        if pin_no >= MAX_GPIO_PIN
        {
            return Err(GpioError::PinOOB { pin: pin_no, count: MAX_GPIO_PIN })
        }
        unsafe{
            configure_gpio_pin_out(pin_no);
        }
        return Ok(Self{pin_no: pin_no})
    }
```

The bounds check happens *before* any register write, so a bad pin number
leaves the hardware untouched. (`new_input` is identical in shape, with a
`Pull` argument passed through to `configure_gpio_pin_in`.) One honest limit:
the driver keeps no per-pin bookkeeping, so nothing stops two calls to
`init_output(25)` from each returning a handle to the same pin — the handle
proves configuration happened, not that it happened once. Chapter 09 §9.6
sketches the claim mask that would close that.

**The traits.** Everything else is wiring the types into `api`'s vocabulary:

```rust
impl ErrorType for Rp2350Gpio
{
    type Error = GpioError;
}

impl Gpio<Rp2350GpioPin> for Rp2350Gpio
{
    fn init_input(&mut self, pin_no: usize, pull: Pull) -> Result<Rp2350GpioPin, Self::Error> {
        return Rp2350GpioPin::new_input(pin_no, pull);
    }

    fn init_output(&mut self, pin_no: usize) -> Result<Rp2350GpioPin, Self::Error>
    {
        return Rp2350GpioPin::new_output(pin_no);
    }
}

impl ErrorType for Rp2350GpioPin
{
    type Error = core::convert::Infallible;
}
```

The two `ErrorType` impls differ on purpose. Port-level operations can fail —
a pin number is caller input — so the port's error is `GpioError`. Driving a
pin that already exists cannot: every failure mode was handled at
construction. `Infallible` is an empty enum, so it has no values,
`Result<(), Infallible>` is the same size as `()`, and the error branch is
eliminated at compile time.

**Writing the pin** is the chapter-07 opener, now in context:

```rust
impl Write<bool> for Rp2350GpioPin
{
    fn write(&mut self, value: bool) -> Result<(), Self::Error> {
        let sio_addr = RegAddr::SIO as usize as *mut Sio;
        unsafe
        {
            let set_reg = match value{
                true => &raw mut (*sio_addr).gpio_out_set,
                false => &raw mut (*sio_addr).gpio_out_clr
            };
            set_reg.write_volatile(1 << self.pin_no);
        }
        return  Ok(());
    }
}
```

Note what varies with `value`: the **register**, not the data. `GPIO_OUT_SET`
(`0xd0000018`) and `GPIO_OUT_CLR` (`0xd0000020`) both act on the bits written
as `1` and ignore zeros, so the tempting
`gpio_out_set.write_volatile((value as u32) << pin)` writes `0` for `false` —
a no-op, leaving the pin stuck high forever. Selecting the register makes both
directions a single store: no read, no modify, no window in which an interrupt
or the other core could lose an update to a neighbouring pin. (SIO provides
these dedicated set/clear registers precisely because it is excluded from the
`+0x2000`/`+0x3000` atomic aliases — chapter 07 §7.5. There is also a
`GPIO_OUT_XOR` that would invert the pin in one store; the tree does not use
it — the blink is an explicit set and clear, not a toggle.)

**Reading it back** completes the `GpioPin` bounds:

```rust
impl Read<bool> for Rp2350GpioPin
{
    fn read(&self) -> Result<bool, Self::Error> {
        let sio_addr = RegAddr::SIO as usize as *mut Sio;
        unsafe{
            let in_reg = &raw const (*sio_addr).gpio_in;
            return Ok(((in_reg.read_volatile() & (1 << self.pin_no))) == 1 << self.pin_no)
        }
    }
}
```

It reads `GPIO_IN`, not `GPIO_OUT` — the level actually present on the pad,
not the last value written. On an output pin those differ whenever the outside
world wins (a short to ground, a load beyond the pad's drive strength), which
makes this the cheapest fault detection available, and it is why step 4 set
`IE` even on an output: with the input buffer off, `GPIO_IN` reads `0`
regardless of the voltage on the leg.

With `Write<bool>` and `Read<bool>` both implemented, `api`'s blanket impl
makes `Rp2350GpioPin` a `GpioPin` automatically — no `impl GpioPin for ...`
line exists or is needed (chapter 07 §7.7).

## 8.12 The application

One piece remains between `OnReset` and a blinking LED: somebody has to call
`Block::start`, and the `api` crate's one concrete type exists to be that
somebody. `Board` (in `api/src/common/board.rs`) is a take-once holder for
the application's peripherals — the essential lines:

```rust
static BOARD_CREATED: AtomicBool = AtomicBool::new(false);

pub struct Board<'a, const N:usize>
{
    blocks: [&'a mut dyn Block; N]
}
```

`Board::take(blocks)` performs a `compare_exchange` on `BOARD_CREATED` —
`false` to `true`, atomically — and only the caller that wins proceeds to call
`unsafe { block.start(); }` on each entry, returning `Some(Board)`. Every
later call finds the flag already `true` and gets `None`. That once-only latch
is the justification for the `unsafe` bring-up call: each driver touches only
its own reset bits, and it happens exactly once, from one place.
`BOARD_CREATED` is also the finished image's entire `.bss` — the 4 bytes
chapter 04 §4.11.2 accounts for — so its "starts at `false`" guarantee is
provided by `reset_bss` in chapter 06. The atomicity matters even on one core
the moment interrupts or the second core exist; `compare_exchange` lowers to
`ldaexb`/`stlexb`, and you can see that pair at the very top of `main`'s
disassembly.

Now finish `demo/src/main.rs`. Replace chapter 06's placeholder `main` and add
the imports and the delay — the whole file, code lines verbatim, comments
elided:

```rust
#![no_std]
#![no_main]

use core::hint::spin_loop;

use api::{common::{Write, board::Board}, gpio::Gpio};
use pico2::gpio::gpio::Rp2350Gpio;


pico2::entry!(main);

fn main() -> ! {
    let mut gpio: Rp2350Gpio = Rp2350Gpio{};
    let _board = Board::take([&mut gpio]).unwrap();
    let mut pin25_o = gpio.init_output(25).unwrap();
    loop {
        pin25_o.write(true);
        delay();
        pin25_o.write(false);
        delay();
    }
}

fn delay()
{
    for _ in 0 .. 5_000_000
    {
        spin_loop();
    }
}
```

Read `main` against the six steps. `Board::take([&mut gpio]).unwrap()` is
step 1 — it starts the one registered `Block`, and the `unwrap` converts "the
board was already taken" into a panic instead of a silently dead bank.
`gpio.init_output(25).unwrap()` is steps 2 through 6 — bounds check, then
`configure_gpio_pin_out(25)`, returning the owning `pin25_o` handle. The loop
is then two single-store writes separated by delays. Everything hardware
touches goes through the `api` traits (`Gpio::init_output`, `Write::write`);
what stays chip-specific in this file is the driver type it names —
`Rp2350Gpio` — and the hard-coded pin 25, which are precisely the lines that
would change on different hardware.

### 8.12.1 The delay loop survives, and here is the proof

`for _ in 0 .. 5_000_000 { spin_loop(); }` looks exactly like the kind of loop
an optimiser deletes. It is not, because `core::hint::spin_loop()` emits a
real instruction — `yield` on ARM — and LLVM does not delete instructions with
architectural effects. Disassemble `main` out of the release build:

```
llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=demo::main \
  target/thumbv8m.main-none-eabihf/release/demo
```

`demo::main`, not `main`. `main` here is an ordinary private Rust function with
a mangled symbol, so `--disassemble-symbols=main` fails with
`failed to disassemble missing symbol main` — which sounds like the function
was optimised away and is not. Chapter 01 §1.7.1 has the `-C` rule. Real
output, abridged to one delay:

```asm
100001a4:      	movw	r1, #0x4b40    ; r1 = 0x4c4b40 = 5000000
100001a8:      	movt	r1, #0x4c
100001ac:      	mov	r3, r1
100001ae:      	str	r0, [r2]       ; 0xd0000018 GPIO_OUT_SET, r0 = 1 << 25
100001b0:      	yield
100001b2:      	subs	r3, #0x40      ; decrement by 64
100001b4:      	yield
...                                    ; 62 more yields
10000232:      	bne	0x100001b0
```

Sixty-four `yield` instructions per iteration, counted from the objdump, and
`subs r3, #0x40` confirms the unroll factor of 64. The compiled shape is
set - delay - clear (a `str r0, [r2, #0x8]`, `GPIO_OUT_CLR` at `+0x8` from the
same base) - delay, branching back. The loop is real code and it executes.

> **Release-build trap.** Take `spin_loop()` out and the danger becomes real: an
> empty `for _ in 0..5_000_000 {}` has no observable effect and at
> `opt-level = 3` LLVM removes it entirely. The delay works in a debug build,
> vanishes under `--release`, and the LED toggles at megahertz rates — a dim,
> steady glow that reads as a hardware fault. Three things keep a delay loop
> alive: `core::hint::spin_loop()`, `core::hint::black_box`, or
> `asm!("nop", options(nomem, nostack))`. The firmware uses the first.

Two other warnings about time on this chip still stand. **`TIMER0` will not
count:** the tick generator feeding it is disabled at reset —
`TICKS: TIMER0_CTRL` bit 0 `ENABLE` resets to `0x0` (datasheet §8.5, Table 623,
PDF p571-572), new relative to RP2040, where the watchdog produced the tick — so
a timer-based delay blocks forever. **The rate is uncalibrated:** nothing here
starts XOSC or a PLL, so `clk_sys` *"Runs from clk_ref at power-up"* and
`clk_ref` *"Runs from Ring Oscillator (ROSC) at power-up"* (Table 540,
datasheet §8.1.2.1, PDF p516). Table 540's `6 - 12MHz` for `clk_ref` is a
nominal design figure, not what an uncalibrated ROSC does: during boot the ROSC
*"runs at a nominal 11MHz and is guaranteed to be in the range 4.6MHz to
19.6MHz without randomisation"* (datasheet §8.3.1, PDF p560), and it drifts with
process, voltage and temperature. **Inferred:** the blink period is therefore a
consequence of that unspecified frequency and LLVM's unroll factor, not a number
anyone chose, and it will differ between boards.

## 8.13 Build it

```
cargo build --release
```

The linker prints the budget on every build, via the `--print-memory-usage`
link-arg in `.cargo/config.toml`:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        7044 B         4 MB      0.17%
             RAM:        8200 B       520 KB      1.54%
```

7044 bytes is `.vector_table` (272) + `.boot_info` (20) + `.text` (`0x1888` =
6280) + `.rodata` (`0x1d8` = 472). The RAM line is the `.stack` reservation
plus the 4-byte `.bss` holding `BOARD_CREATED` and 4 bytes of alignment
padding (chapter 04 §4.11.2 decomposes it). The build emits exactly **one**
warning — the `linker_messages` warning carrying the table above; rustc
routes linker stdout through the warning channel (chapter 01 §1.7).

Where did five kilobytes over chapter 06's 1400 B go? Not into the driver —
the configuration and write paths are a few hundred bytes. `llvm-nm -C` on the
image shows the bulk: `core::fmt::Formatter::pad`, `pad_integral`,
`core::fmt::write`, the `Debug` impls for integers, `core::str::count`,
`core::option::unwrap_failed`, `core::result::unwrap_failed`,
`core::panicking::panic_fmt`. That is the price of the two `unwrap()` calls in
`main` plus `GpioError`'s `Debug` impl: unwrap must be able to *format* the
error into a panic message, so the formatting machinery rides along, and
`.rodata` gains the panic strings. It is dead weight on a board with no
console — nothing can ever display the message — and replacing the `unwrap`s
with `loop {}` matches or a custom trap would drop most of it. The tree keeps
the `unwrap`s because they are the ordinary Rust idiom and the budget is
0.17% of flash; know what they cost, and that the choice is reversible.

## 8.14 Flash it

`picotool` dispatches on file extension and Cargo's output has none, so copy it
first:

```
cp target/thumbv8m.main-none-eabihf/release/demo demo.elf
picotool uf2 convert demo.elf demo.uf2 --family rp2350-arm-s
```

`rp2350-arm-s` is the Arm Secure family — the image type the boot block
declares, and the one `picotool info -a demo.elf` reports back about the
**file**, with no board attached. Chapter 05 §5.5 prints that output and reads
it word by word; run it now as a check that the boot block is well-formed
before you go near hardware. (The names are yours to choose — the repository
root's `blinky.elf` and `blinky.uf2` are exactly this pair of commands run
with a different name.)

To get it onto the board, hold **BOOTSEL** while plugging in the USB cable, wait
for the mass-storage volume, then either drag `demo.uf2` onto it or run
`picotool load demo.uf2`. The hold is not a one-time thing: this firmware never
enumerates USB, so `picotool` cannot ask the running image to reboot into the
bootloader. **Every reflash needs a physical BOOTSEL hold.**

There is no brick risk, and the argument is worth stating rather than taking on
faith: this image writes two CPU registers (`CPACR`, `VTOR`), some RAM, and a
handful of peripheral registers across `RESETS`, `SIO`, `IO_BANK0` and
`PADS_BANK0`. It touches no OTP, no QMI flash-interface configuration, no
partition table and no boot key, so the bootrom's BOOTSEL path depends on
nothing it changes. Holding BOOTSEL always returns you to a mass-storage device
you can drop a working UF2 onto.

## 8.15 Did it work?

Success and failure look nearly identical from across the desk: a board with no
LED on. There is exactly one discriminator, and it is binary. If the bootrom
**rejects** the image — malformed or missing `IMAGE_DEF`, bad vector table — it
falls through to USB Boot and the board re-enumerates as a mass-storage device.

**Re-enumerates as mass storage = the image was rejected. Stays dark and does
not enumerate = the image was accepted and is running.** A dark board that does
*not* appear on your desktop is a running image with a GPIO problem, which is a
completely different investigation. Establish which one you have before changing
a line.

If the image is accepted and the LED is dark, run these checks in order — each
one halves the remaining possibilities.

**1. Read `GPIO25_STATUS` at `0x400280c8`.** It is the mux's own account of what
it is sending to the pad, and it is read-only (Table 699, PDF p649):

| Bits | Field | Type | Reset |
|---|---|---|---|
| 17 | `INFROMPAD: input signal from pad, before filtering and override are applied` | RO | `0x0` |
| 13 | `OETOPAD: output enable to pad after register override is applied` | RO | `0x0` |
| 9 | `OUTTOPAD: output signal to pad after register override is applied` | RO | `0x0` |

If `OETOPAD` (bit 13) is `0`, step 3 or step 5 did not take — either the output
enable never got set, or `FUNCSEL` is not 5 and SIO does not own the pin. If
both bit 9 and bit 13 are `1`, the mux is doing its job and the fault is
downstream, in the pad: re-read `0x40038068` and confirm bit 8 (`ISO`) is `0`
and bit 6 (`IE`) is `1`. `OUTTOPAD` flipping while the LED stays dark is the
signature of a still-isolated pad.

**2. Force the output high from the mux.** To prove the pad and the board wiring
independently of SIO, set `OUTOVER` to `0x3` — *"HIGH: drive output high"*
(Table 700, PDF p650) — which overrides the peripheral's output signal entirely:

```rust
// PROPOSED — not in the tree today
// GPIO25_CTRL = OUTOVER(0x3) | FUNCSEL(5): drive the pin high from the mux,
// ignoring whatever SIO says.
let io_ctrl = 0x4002_80cc as *mut u32;
unsafe { io_ctrl.write_volatile((0x3 << 12) | 5) };
```

That word is `0x3005`. If the LED lights with it and not with the SIO path, the
fault is in steps 2 and 3. If it stays dark even then, `ISO` is still set, or
`IE`/`OD` are wrong, or step 1 never completed — and step 1 is the one that
makes everything above it silently do nothing, which is why it is first.

---

Next: **chapter 09**, the GPIO register reference. It holds what this chapter
skipped — `RESETS` in full, including the atomic aliases (§9.2); every pad field
and the isolation latches (§9.3, §9.3.1); the complete `GPIOn_CTRL` and
`GPIOn_STATUS` maps (§9.4); and the SIO offset map (§9.5). It is reference: skip
it on a first pass and come back when you want a number rather than an
explanation.
