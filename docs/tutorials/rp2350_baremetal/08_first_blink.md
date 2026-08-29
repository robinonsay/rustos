---
document_type: "Tutorial Chapter — First Blink"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 8 of 9
revision: B
effective_date: 2026-08-28
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-07
sources: RP2350 datasheet §2.2.4 (Table 12, PDF p33), §2.2.6 (Table 14, PDF p35),
  §3.1.11 (Table 16, PDF p55-56), §7.5.2 (PDF p503), §7.5.3 (Tables 533/534/536,
  PDF p504-506), §8.1.2.1 (Table 540, PDF p516), §8.3.1 (PDF p560), §8.5
  (Table 623, PDF p571), §9.3 (PDF p586-587), §9.10.1 (PDF p596-601),
  §9.11.1 (Tables 699/700, PDF p604, p649-651), §9.11.3 (Tables 850/877,
  PDF p783-785, p798); errata RP2350-E9 (PDF p1358-1359); Pico 2 datasheet p9, p10
creates: firmware/pico2/src/common/mod.rs, firmware/pico2/src/common/reg.rs,
  firmware/pico2/src/gpio/mod.rs, firmware/pico2/src/gpio/gpio.rs; finishes
  firmware/pico2/src/main.rs
---

# Chapter 08 — First Blink

Everything so far has been infrastructure: a linker script, a boot block, a
vector table, a reset handler that reaches `main` with a stack under it. None of
it is observable from outside the chip. This chapter makes one pin move, in six
steps, in the order the firmware performs them. You perform the first one about
sixty lines from here. Citation and callout conventions are the index's.

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
| 1 | `RESETS.RESET` | deassert bits 6 and 9, then poll `RESET_DONE` for both | §8.5 |
| 2 | `SIO.GPIO_OUT_CLR` | write `1 << 25` — start the pin low | §8.6 |
| 3 | `SIO.GPIO_OE_SET` | write `1 << 25` — make it an output | §8.6 |
| 4 | `PADS_BANK0.GPIO25` | `IE = 1`, `OD = 0` — wake the pad | §8.7 |
| 5 | `IO_BANK0.GPIO25_CTRL` | `FUNCSEL = 5` — hand the pin to SIO | §8.8 |
| 6 | `PADS_BANK0.GPIO25` | clear `ISO` — connect the pad to the outside | §8.9 |

Then toggle `SIO.GPIO_OUT_XOR` forever (§8.11).

One rule orders the list: **nothing responds until its reset is deasserted, and
nothing reaches the pin until `ISO` is cleared, so configuration happens in
between.** Step 1 opens the door, step 6 opens the shutter, and steps 2 through
5 run in the dark on purpose, where no half-configured state can appear on the
wire. That is the order `configure_gpio_pin_out` writes them in; §8.10.1 is
honest about why it is worth a second look.

## 8.3 The address constants

This chapter creates four files under `firmware/pico2/src/`, and two of them are
one line long. Make all four now, so nothing below has to stop and explain a
missing module:

```text
src/main.rs                  exists — chapters 05 and 06
src/common/mod.rs            new — §8.3
src/common/reg.rs            new — §8.3
src/gpio/mod.rs              new — §8.4
src/gpio/gpio.rs             new — §8.5 onward
```

Rust does not find a file because it is on disk; a parent module has to declare
it. `src/common/mod.rs` is exactly one line, and it is the whole file:

```rust
pub mod reg;
```

`src/gpio/mod.rs` opens the same way — `pub mod gpio;` — and §8.4 shows it in
place. The two `mod` declarations that pull `common` and `gpio` into the crate
go in `main.rs`, and §8.12.2 adds them there once both directories exist.

The four base addresses live in one enum — `firmware/pico2/src/common/reg.rs`,
in full:

```rust
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum RegAddr {
    RESET = 0x4002_0000 as usize,
    IO_BANK0 = 0x4002_8000 as usize,
    SIO = 0xd000_0000 as usize,
    PADS_BANK0 = 0x4003_8000 as usize,
}
```

`#[repr(usize)]` pins each discriminant to pointer width, which makes the cast
at every use site lossless:

```rust
let reset_addr = RegAddr::RESET as usize as *mut Reset;
```

Two casts, not one: `as usize` extracts the discriminant, `as *mut Reset`
reinterprets it as a pointer. The `as usize` on each initialiser is redundant
under `#[repr(usize)]` and is in the tree, so it is quoted. The SCREAMING_SNAKE
names produce warnings; §8.13 says which, and why they are expected.

## 8.4 The register structs

`firmware/pico2/src/gpio/mod.rs` is five `#[repr(C)]` structs and nothing else.
They are maps: point one at a base address and index it. Chapter 07 §7.6
explains why they are shaped this way and chapter 09 keeps the field-by-field
reference; here they are only the thing you index.

```rust
pub mod gpio;

#[repr(C)]
struct GpioRegs{
    pub status: u32,  // Status Register
    pub ctrl: u32,    // Ctrl Register
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
    pub voltage_select: u32,   // 0x00  bank-wide input threshold
    pub pads: [u32; 48],
    pub swclk: u32,            // 0xc4
    pub swd: u32,              // 0xc8
}

#[repr(C)]
struct Reset{
    pub reset: u32,
    pub wdsel: u32,
    pub reset_done: u32
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

Every field is a `u32` or an array of them, so alignment is 4 and `#[repr(C)]`
inserts no padding. The offsets this chapter uses:

| Offset | Name | Info |
|---|---|---|
| `RESETS + 0x0` | `RESET` | `Reset.reset` — Table 533, PDF p504 |
| `RESETS + 0x8` | `RESET_DONE` | `Reset.reset_done` — Table 533, PDF p504 |
| `SIO + 0x020` | `GPIO_OUT_CLR` | `Sio.gpio_out_clr` — Table 16, PDF p55 |
| `SIO + 0x028` | `GPIO_OUT_XOR` | `Sio.gpio_out_xor` — Table 16, PDF p55 |
| `SIO + 0x038` | `GPIO_OE_SET` | `Sio.gpio_oe_set` — Table 16, PDF p56 |
| `PADS_BANK0 + 0x04 + 4n` | `GPIOn` | `PadsBank.pads[n]` — Table 850, PDF p783 |
| `IO_BANK0 + 0x000 + 8n` | `GPIOn_STATUS` | `IoBank.gpio[n].status` — Table 648, PDF p603-604 |
| `IO_BANK0 + 0x004 + 8n` | `GPIOn_CTRL` | `IoBank.gpio[n].ctrl` — Table 648, PDF p603-604 |

The firmware's SIO field names transpose the datasheet's — the code writes
`gpio_out_hi` where the datasheet says `GPIO_HI_OUT`. Same layout, different
spelling; chapter 09 §9.5 uses the datasheet's.

## 8.5 Step 1 — release the two resets

The four functions from here to §8.12 are the whole of
`firmware/pico2/src/gpio/gpio.rs`, in the order the file has them. Start the
file with its head — three imports, verbatim, and nothing else above them:

```rust
use core::hint::spin_loop;

use crate::gpio::{IoBank, PadsBank, Reset, Sio};
use crate::common::reg::RegAddr;
```

`spin_loop` is the delay in §8.12 and `RegAddr` is §8.3's enum. The middle line
is the one worth a sentence, because it looks like it should not compile. Look
back at §8.4: those five structs are declared `struct`, not `pub struct` — they
are **private to `gpio/mod.rs`**. A private item is visible inside the module
that declares it *and inside that module's descendants*, and `gpio::gpio` is a
child of `gpio`, so `gpio.rs` can name `IoBank` and `PadsBank` while nothing
outside the `gpio` module can. That is what keeps the register maps from leaking
into the rest of the crate while still letting the driver use them. `super::` in
place of `crate::gpio::` would resolve to the same module; the tree spells it
out, so this does too.

Every peripheral powers up **held in reset**. Before `IO_BANK0` or `PADS_BANK0`
acknowledge a single write, you deassert their reset bits and wait for the
hardware to confirm.

```rust
unsafe fn reset_gpio(){
    // Bit for IOBANK
    const IOBANK_RESET_BIT:u8 = 6;
    // Bit for Pad
    const PADBANK_RESET_BIT:u8 = 9;
    // IO Pad bitmask
    const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current & !IO_PAD_BITMASK);
        let reset_done = &raw const (*reset_addr).reset_done;
        // Wait for it to be done
        while reset_done.read_volatile() & IO_PAD_BITMASK != IO_PAD_BITMASK
        {}
    }
}
```

The two bits, from `RESETS.RESET` at `0x40020000` (Table 534, PDF p504):

| Bits | Field | Type | Reset |
|---|---|---|---|
| 9 | `PADS_BANK0` | RW | `0x1` |
| 6 | `IO_BANK0` | RW | `0x1` |

`RESET_DONE` at `0x40020008` uses the same bit numbering and is read-only
(Table 536, PDF p506). `IO_PAD_BITMASK` is `(1 << 6) | (1 << 9)` = **`0x240`**;
`<<` binds tighter than `|`, so the unparenthesised constant is correct.

| Register | Bit = 1 means | You |
|---|---|---|
| `RESET` | held **in** reset | write `0` to release |
| `RESET_DONE` | **out of** reset, ready | wait for `1` |

Same bit position, opposite sense. Both bits are already `1` at power-on, so you
clear them — the verb is **deassert**. `current & !0x240` is a read-modify-write
on a register holding 27 other components' reset bits — `RESET` defines bits
28:0 (Table 534, PDF p504-505) — and the read is what keeps you from resetting
those 27. The poll then spins until *both* bits appear:
`& mask != mask` is false only when the masked read equals the full mask. There
is no interrupt for this; datasheet §7.5.2 states the intent — *"This allows
software to wait for this status bit in case the component requires
initialisation before use."* (PDF p503).

> **Silent-failure trap.** While a block is in reset its registers do not
> respond. Writes are discarded — no error, no bus fault, no diagnostic — and
> reads return reset values, so a debugger shows you a plausible register block
> that is simply not listening. Every store to `0x40028000` or `0x40038000`
> before this function returns goes nowhere. Step 1 is not housekeeping; it is
> the moment those two peripherals become addressable.

> **Hardware-destructive.** In the same register, bit 7 is `IO_QSPI` and bit 10
> is `PADS_QSPI` (Table 534, PDF p504). Those are the pins your flash is on, and
> you are executing from that flash. Assert either and the image running the
> instruction that asserted it stops existing. The mask is `0x240` — not
> `0x2c0`, not `0x6c0`.
> This is the reset-controller face of the hazard chapter 03 §3.4 raised for
> `IO_QSPI` and `PADS_QSPI` themselves; chapter 09 §9.2.3 has the bit table.

Everything else about `RESETS` — the `+0x3000` atomic-clear alias as an
alternative to the read-modify-write above (the alias mechanism itself is
chapter 07 §7.5), assert-then-deassert as an init idiom, and why reset is the
wrong granularity for a resource allocator — is chapter 09 §9.2, and none of it
is needed to blink.

## 8.6 Steps 2 and 3 — initial level and direction, before the mux

Two stores to SIO, from `configure_gpio_pin_out` (whole function in §8.10):

```rust
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
```

`GPIO_OUT_CLR` is `SIO + 0x020` and `GPIO_OE_SET` is `SIO + 0x038` (Table 16,
§3.1.11, PDF p55-56); for pin 25 both take `1 << 25` = `0x02000000`. These are
write-one-to-act registers — the bits you write are the bits that change — so
neither store needs a read first and the other 31 GPIOs are untouched. The pin
is now an output driving low, decided before anything connects it to the world.

Why `0xd0000000` and not the APB range? Because SIO is not on APB: §2.2.6 calls
it core-local, *"accessible to processor load/store only… Access is always
zero-wait-state"* (PDF p34), while an APB write costs *"a minimum of four
cycles"* (§2.2.4, PDF p33). The value and the direction, written on every
toggle, live off the peripheral bus; the mux and the pad, written once, stay on
it. The full SIO tour is chapter 09 §9.5.

## 8.7 Step 4 — the pad: `IE = 1`, `OD = 0`

```rust
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        current_pad &= !(1 << OD);
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
```

```text
PADS_BANK0 + 0x04 + 4*25 = 0x40038000 + 0x04 + 0x64 = 0x40038068
```

Table 850 lists `GPIO25` at offset `0x68` (datasheet §9.11.3, PDF p784), so the arithmetic
and the table agree. The `+0x04` is not decoration: offset `0x00` is
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
`GPIOn_STATUS.INFROMPAD`, which is how §8.15 reads back what the pin is doing.
The read-modify-write earns its keep — this register resets to `0x116` (`ISO=1`,
`DRIVE=0x1`, `PDE=1`, `SCHMITT=1`), so a plain write of `0x40` would silently
drop the pull-down and the Schmitt trigger. After this step it holds `0x156`.
The remaining pad fields are chapter 09 §9.3.

## 8.8 Step 5 — the mux: `FUNCSEL = 5`

```rust
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
indexed by the same `pin` three lines apart. This is the most error-prone
arithmetic in the chapter: get it wrong and you configure a different pin, or
land on a `STATUS` register and write read-only bits, which succeeds and does
nothing. In the firmware both are structural, `gpio[pin].ctrl` and `pads[pin]`,
and `#[repr(C)]` does the multiply.

`FUNCSEL = 5` selects SIO: Table 700 enumerates it for this pin specifically,
`0x05 → SIO_25`, and the field resets to `0x1f` (NULL) (PDF p650-651).

The write is a **plain write of `5`**, not a read-modify-write, and that is
correct here. `GPIO25_CTRL` also carries `OUTOVER` (bits 13:12), `OEOVER`
(15:14), `INOVER` (17:16) and `IRQOVER` (29:28); all four reset to `0x0`, and
`0x0` in each means "take the peripheral's signal unmodified". Writing bare `5`
sets `FUNCSEL` and forces every override to pass-through in one store — which is
also how you recover a known state on a pin other code has touched. The SDK does
the same: *"Zero all fields apart from fsel; we want this IO to do what the
peripheral tells it."* (datasheet §9.10.1, PDF p600). Full field table is
chapter 09 §9.4.

## 8.9 Step 6 — clear `ISO`

```rust
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
```

Bit 8, cleared with a read-modify-write so the `IE` you just set and the pad's
reset-value fields survive. The register goes from `0x156` to `0x056`.

This is last because it is the shutter. While `ISO` is set the pad is
disconnected from everything upstream of it, so steps 2 through 5 configured a
pin that could not affect the outside world whatever order they ran in. Clearing
it once, at the end, connects a pin that is already fully configured: the LED
sees one transition into driven-low, never a glitch from a half-set-up pin.

> **Silent-failure trap.** Leave `ISO` set and every register reads back exactly
> as written — `FUNCSEL` says 5, `GPIO_OE` says output — and the pin does
> nothing, because *"the isolation latches prevent upstream signals from
> propagating to the pad"* (datasheet §9.3, PDF p587). Resetting the block does
> not rescue you: *"If a pad's isolation latches are in the latched state then
> resetting the PADS and IO registers does not physically return the pad to its
> reset state."* (datasheet §9.3, PDF p587). Step 1 cannot undo a missing step 6.

The power-domain rationale, what the latches capture, the three ways they trap
you, and the correct teardown order are chapter 09 §9.3.1.

## 8.10 The finished function

All 26 lines, verbatim from `firmware/pico2/src/gpio/gpio.rs`:

```rust
unsafe fn configure_gpio_pin_out(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        current_pad &= !(1 << OD);
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
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

Step 1 is not in this function; `gpio_demo` calls `reset_gpio()` first (§8.12).

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
disassembly in §8.13 and the register dumps in §8.15 are taken from, and it
works on the board. Do not reorder it while following this chapter.

The recommendation below is for code you write later, and it is deliberately not
applied here: moving `io_ctrl.write_volatile(SIO)` above the pad `IE`/`OD`
read-modify-write costs nothing and closes the overlap of conditions 2 and 3
outright rather than relying on `ISO` to cover it. It is the shape to reach for
in a driver of your own. **The firmware has not been changed** — the tutorial
documents the tree as it is, and marks the improvement rather than quietly
shipping a listing that no build in this chapter matches.

## 8.11 Toggling

```rust
unsafe fn toggle_gpio_pin(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    unsafe{
        let toggle = &raw mut (*sio_addr).gpio_out_xor;
        toggle.write_volatile(1 << pin);
    }
}
```

`GPIO_OUT_XOR` is `SIO + 0x028` = `0xd0000028`, *"GPIO0…31 output value XOR"*
(Table 16, PDF p55). Writing `1 << 25` inverts bit 25 of `GPIO_OUT` and leaves
the other 31 alone. One store: no read, no compare, no branch, and no window in
which a second core's update to `GPIO_OUT` could be lost, because the XOR
happens in the SIO hardware.

## 8.12 The demo loop

```rust
pub unsafe fn gpio_demo(){
    unsafe{
        reset_gpio();
        configure_gpio_pin_out(25);
    }
    loop{
        unsafe{
            toggle_gpio_pin(25);
        }
        for _ in 0..500_000 {spin_loop();}
    }
}
```

Step 1, then steps 2 through 6, then forever. Pin `25` is hard-coded at both
call sites. `gpio_demo` is the only `pub` item in the module, and that closes
`gpio.rs`: the four functions of §8.5 through §8.12, under the three imports of
§8.5, are the complete file. §8.12.3 prints all 75 lines of it in one block, to
check yours against.

### 8.12.1 The delay loop survives, and here is the proof

`for _ in 0..500_000 {spin_loop();}` looks exactly like the kind of loop an
optimiser deletes. It is not, because `core::hint::spin_loop()` is inline
assembly — it emits a `yield` on ARM — and LLVM does not delete instructions it
cannot see through. Disassemble `main` out of the release build:

```
llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=pico2::main \
  target/thumbv8m.main-none-eabihf/release/pico2
```

`pico2::main`, not `main`. `main` here is an ordinary private Rust function with
a mangled symbol, so `--disassemble-symbols=main` fails with
`failed to disassemble missing symbol main` — which sounds like the function was
optimised away and is not. Chapter 01 §1.7.1 has the `-C` rule. Real output,
abridged:

```asm
10000216:      	mov	r3, r2         ; r3 = 500000
10000218:      	str	r1, [r0]       ; 0xd0000028 GPIO_OUT_XOR, r1 = 1 << 25
1000021a:      	yield
1000021c:      	subs	r3, #0x32      ; decrement by 50
1000021e:      	yield
...                                    ; 48 more yields
10000280:      	bne	0x1000021a
```

Fifty `yield` instructions per iteration, counted from the objdump, and
`subs r3, #0x32` confirms the unroll factor of 50. LLVM unrolled the outer blink
loop twice as well, so the compiled shape is toggle-delay-toggle-delay branching
back to `0x10000216`. The loop is real code and it executes.

> **Release-build trap.** Take `spin_loop()` out and the danger becomes real: an
> empty `for _ in 0..500_000 {}` has no observable effect and at
> `opt-level = 3` LLVM removes it entirely. The delay works in a debug build,
> vanishes under `--release`, and the LED toggles at tens of megahertz — a dim,
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

### 8.12.2 Finishing `main.rs`

`main.rs` still holds chapter 06 §6.5's placeholder. Two things change, and they
are the last edits the tutorial asks for.

**The head of the file.** The two modules you created in §8.3 and §8.4 now
exist, so declare them, and import the one function they export:

```rust
#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::copy_nonoverlapping};

use crate::gpio::gpio::gpio_demo;
mod common;
mod gpio;
```

`gpio::gpio` is not a typo. `mod gpio;` names the directory module in
`gpio/mod.rs`, and that module's `pub mod gpio;` names `gpio.rs` inside it — a
module called `gpio` containing a module called `gpio`. `mod common;` earns its
place even though nothing in `main.rs` names `common`: without it `common/reg.rs`
is not part of the crate at all, and `gpio.rs`'s `use crate::common::reg::RegAddr`
fails to resolve.

**The body of `main`.** Replace the placeholder with the real thing:

```rust
fn main() -> !{
    unsafe{
        gpio_demo();
    }
    loop{}
}
```

Still `-> !`, for the reason chapter 06 §6.5 gives: `OnReset` ends in `main()`
and is itself `-> !`. `gpio_demo` never returns, so the trailing `loop{}` is
unreachable — it is there to satisfy the type, not the hardware. `OnReset` is
unchanged from chapter 06, and reaches `main` after the FPU, `VTOR`, `.data` and
`.bss` work:

```rust
#[unsafe(no_mangle)] pub extern "C" fn OnReset() -> ! {
    unsafe{
        enable_fpu();
        reset_vtor();
        reset_data();
        reset_bss();
    }
    main();
}
```

That is the whole firmware. Every `PLACEHOLDER` from §5.9 and §6.5 is now gone,
and `main.rs` is byte-for-byte the tree's — 126 lines, in the order §5.9 and
§6.5 built them up: the head above, `BOOT_INFO`, the six `extern` symbols, the
panic handler, `union Vector`, the PPB constants, the four `#[inline]` helpers,
`OnReset`, `DefaultHandler`, `OnHardFault`, `VECTOR_TABLE`, and `main` last.

### 8.12.3 `gpio.rs` in full

`main.rs` got a whole-file listing at the end of chapter 05 and again at the end
of chapter 06; `gpio.rs` was written a function at a time, so here it is once,
whole. This is `firmware/pico2/src/gpio/gpio.rs` verbatim — 75 lines, nothing
elided, no placeholders left. Check yours against it before building.

The four function bodies are §8.5, §8.10, §8.11 and §8.12 in that order, under
§8.5's three `use` lines. Note in particular that §8.6 through §8.9 add nothing:
they are commentary on pieces of `configure_gpio_pin_out`, which appears here
exactly once.

```rust
use core::hint::spin_loop;

use crate::gpio::{IoBank, PadsBank, Reset, Sio};
use crate::common::reg::RegAddr;

unsafe fn reset_gpio(){
    // Bit for IOBANK
    const IOBANK_RESET_BIT:u8 = 6;
    // Bit for Pad
    const PADBANK_RESET_BIT:u8 = 9;
    // IO Pad bitmask
    const IO_PAD_BITMASK: u32 = 1 << IOBANK_RESET_BIT | 1 << PADBANK_RESET_BIT;
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current & !IO_PAD_BITMASK);
        let reset_done = &raw const (*reset_addr).reset_done;
        // Wait for it to be done
        while reset_done.read_volatile() & IO_PAD_BITMASK != IO_PAD_BITMASK
        {}
    }
}

unsafe fn configure_gpio_pin_out(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe{
        let gpio_out_clr = &raw mut (*sio_addr).gpio_out_clr;
        gpio_out_clr.write_volatile(1 << pin);
        let gpio_oe_set = &raw mut (*sio_addr).gpio_oe_set;
        gpio_oe_set.write_volatile(1 << pin);
        let pad= &raw mut (*pads_addr).pads[pin];
        let mut current_pad = pad.read_volatile();
        const IE: u8 = 6;
        const OD: u8 = 7;
        current_pad &= !(1 << OD);
        current_pad |= 1 << IE;
        pad.write_volatile( current_pad);
        let io_ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
        let mut current_pad = pad.read_volatile();
        const ISO: u8 = 8;
        current_pad &= !(1 << ISO);
        pad.write_volatile(current_pad);
    }
}

unsafe fn toggle_gpio_pin(pin: usize)
{
    let sio_addr = RegAddr::SIO as usize as *mut Sio;
    unsafe{
        let toggle = &raw mut (*sio_addr).gpio_out_xor;
        toggle.write_volatile(1 << pin);
    }
}

pub unsafe fn gpio_demo(){
    unsafe{
        reset_gpio();
        configure_gpio_pin_out(25);
    }
    loop{
        unsafe{
            toggle_gpio_pin(25);
        }
        for _ in 0..500_000 {spin_loop();}
    }
}
```

## 8.13 Build it

```
cargo build --release
```

The linker prints the budget on every build, via the `--print-memory-usage`
link-arg in `.cargo/config.toml`:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        1744 B         4 MB      0.04%
             RAM:          8 KB       520 KB      1.54%
```

1744 bytes is `.vector_table` (272) + `.boot_info` (20) + `.text` (1452). The
8 kB of RAM is the `.stack` reservation; `.data` and `.bss` are both zero-length
in this build.

The build is not silent, and none of the noise means you broke something.
Verified on 2026-08-28: `warning: pico2 (bin "pico2") generated 3 warnings` and
`warning: api (lib) generated 4 warnings`.

- **Two `non_camel_case_types` warnings**, on `RegAddr::IO_BANK0` and
  `RegAddr::PADS_BANK0` in `firmware/pico2/src/common/reg.rs`. The lint wants
  `IoBank0` and `PadsBank0`; the names are deliberately the datasheet's.
  `RESET` and `SIO` do not trip it — they are single words.
- **One `linker_messages` warning**, which is how the memory table above reaches
  your terminal: `rustc` treats anything the linker prints on stdout as a
  warning. Two plus one is the three.
- **Four dead-code warnings from the `api` crate** — `ErrorType`, `Write`,
  `Read` and `GpioPin` are never used. `firmware/pico2` depends on `api` and
  never calls into it.

## 8.14 Flash it

`picotool` dispatches on file extension and Cargo's output has none, so copy it
first:

```
cp target/thumbv8m.main-none-eabihf/release/pico2 pico2.elf
picotool uf2 convert pico2.elf pico2.uf2 --family rp2350-arm-s
```

`rp2350-arm-s` is the Arm Secure family — the image type the boot block
declares, and the one `picotool info -a pico2.elf` reports back about the
**file**, with no board attached. Chapter 05 §5.5 prints that output and reads
it word by word; run it now as a check that the boot block is well-formed
before you go near hardware.

To get it onto the board, hold **BOOTSEL** while plugging in the USB cable, wait
for the mass-storage volume, then either drag `pico2.uf2` onto it or run
`picotool load pico2.uf2`. The hold is not a one-time thing: this firmware never
enumerates USB, so `picotool` cannot ask the running image to reboot into the
bootloader. **Every reflash needs a physical BOOTSEL hold.**

There is no brick risk, and the argument is worth stating rather than taking on
faith: this image writes two CPU registers (`CPACR`, `VTOR`), some RAM, and five
peripheral registers across `RESETS`, `SIO`, `IO_BANK0` and `PADS_BANK0`. It
touches no OTP, no QMI flash-interface configuration, no partition table and no
boot key, so the bootrom's BOOTSEL path depends on nothing it changes. Holding
BOOTSEL always returns you to a mass-storage device you can drop a working UF2
onto.

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

If the image is accepted and the LED is dark, work down this ladder in order.

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
