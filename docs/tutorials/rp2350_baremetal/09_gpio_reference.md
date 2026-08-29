---
document_type: "Tutorial Chapter — GPIO Reference"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 9 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-08
sources: RP2350 datasheet §2.1.3 (PDF p27), §2.2 (PDF p33), §3.1.11 (Table 16, PDF p55-56), §3.1.3 (Table 23, PDF p64), §3.6.1, §7.5.2 (PDF p503), §7.5.3 (Tables 533/534/536, PDF p504-506), §9.2 (PDF p586), §9.3 (PDF p586-587), §9.6 (PDF p593), §9.7 (PDF p594-595), §9.8, §9.9 (PDF p595), §9.11.1 (Tables 648/699/700, PDF p603-606, p649-651), §9.11.3 (Tables 850/851/852/877, PDF p783-785, p798), §10.6 ACCESSCTRL register list (PDF p826); errata RP2350-E9 (PDF p1358-1359); Pico 2 datasheet p9
creates: nothing
describes: firmware/pico2/src/gpio/mod.rs, firmware/pico2/src/gpio/gpio.rs,
  firmware/pico2/src/common/reg.rs, firmware/pico2/src/common/reset.rs —
  all written in chapter 08
reading: reference — skip on a first pass; chapter 08 is self-contained
---

# Chapter 09 — GPIO Reference

## 9.0 How to use this chapter

You already have a blink. Chapter 08 took you from a held-in-reset chip to
GP25 driven high and low through `SIO.GPIO_OUT_SET` and `SIO.GPIO_OUT_CLR`,
and it did that with the minimum number of register fields — one bit here, one
field there, each introduced at the moment it was needed. This chapter is the
rest of those registers: every field, every reset value, every offset, ordered
by block rather than by task, because that is how you look a number up.
Nothing here is required to get a blink; most of it is required to write a
driver that other code can trust.

Each section names the chapter 08 step it expands, so you can arrive here from
a forward pointer and leave again:

| Section | Expands | Block |
|---|---|---|
| §9.1 | chapter 08's block table | banks and naming |
| §9.2 | chapter 08 §8.5-§8.6 | `RESETS` `0x40020000` |
| §9.3 | chapter 08 §8.8 | `PADS_BANK0` `0x40038000` — the pad |
| §9.4 | chapter 08 §8.9 | `IO_BANK0` `0x40028000` — the mux |
| §9.5 | chapter 08 §8.7, §8.11 | `SIO` `0xd0000000` — the value |
| §9.6 | nothing — proposed | releasing a pin |
| §9.7 | nothing | what the firmware does not do |

The index's conventions apply, plus one this chapter needs and no other does:
the GPIO chapter of the datasheet is unfortunately also chapter 9, so a bare
`§9.4` means a section of *this chapter* and a datasheet section of the same
number is always written `datasheet §9.4`. As everywhere, **the firmware**
means the code in the tree — here mostly `firmware/pico2` — and **proposed**
means code that does not exist and is not compiled; §9.7 lists all of it.

For modelling any of these blocks as a `#[repr(C)]` struct — the reserved
holes, the alignment traps, why you never form a `&mut` over MMIO — see
chapter 07 §7.6. This chapter gives you the offsets; that one gives the layout
rules.

## 9.1 Banks and naming

**Two banks.** Bank 0 is user IO — 30 GPIOs on the RP2350A/QFN-60 in the
Pico 2, 48 on the QFN-80 (datasheet §9.3, PDF p586); the firmware records the
30 as `MAX_GPIO_PIN` in `firmware/pico2/src/common/mod.rs` and checks every
pin number against it (chapter 08 §8.11). Bank 1 is the six QSPI IOs plus the
USB DP/DM pins, which RP2350 can use as GPIOs (datasheet §9.2, PDF p586).

The naming is inconsistent between chapters of the datasheet:

| Hardware | Address map (§2.2, PDF p33) | ACCESSCTRL (PDF p826) |
|---|---|---|
| Bank 0 IO mux | `IO_BANK0` @ `0x40028000` | `IO_BANK0` @ `0x68` |
| Bank 1 IO mux | **`IO_QSPI`** @ `0x40030000` | **`IO_BANK1`** @ `0x6c` |
| Bank 0 pads | `PADS_BANK0` @ `0x40038000` | `PADS_BANK0` @ `0x70` |
| Bank 1 pads | `PADS_QSPI` @ `0x40040000` | `PADS_QSPI` @ `0x74` |

So `IO_BANK1` does exist as a name — it just never appears in the address map.
The `0` in `IO_BANK0` means "the user bank", not "the first of N".

The blocks are grouped **by layer, not by bank**: the two IO muxes are
adjacent at `0x40028000` and `0x40030000`, then the two pad blocks at
`0x40038000` and `0x40040000` (PDF p33). `PADS = IO + 0x10000` holds for both
banks. **Inferred:** that is an artefact of the allocation order, not a
documented rule, so do not build offset arithmetic on it.

Bank 1 is owned by the bootrom's XIP setup — the flash you are executing from
is on those pins. Leave it alone; see §9.2.3 for what happens if you do not.

### 9.1.1 Where the 32-bit limit bites

`IO_BANK0` and `PADS_BANK0` use **per-pin registers**, so nothing about a
32-bit register width constrains them. Bank 0 scales to 48 GPIOs by having a
longer array: 48 `STATUS`/`CTRL` pairs in the mux (§9.4), 48 pad words in the
pad block (§9.3).

`SIO` is the **bitmask** block — one bit per GPIO in a 32-bit register. That
is why the `GPIO_HI_*` companions exist: `GPIO_OUT` covers GPIO0-31 and
`GPIO_HI_OUT` covers GPIO32-47 *plus* the QSPI IOs and USB DP/DM in its upper
bits (datasheet §9.8, PDF p595). The consequences of that packing are §9.5.1,
and they are sharper than they look.

On the Pico 2 (GPIO0-29) everything fits in the low registers. GP25 is bit 25
and the high registers never come up.

## 9.2 RESETS in full

Expands chapter 08 §8.5-§8.6. Every peripheral on the chip powers up **held in
reset** — `RESET` resets to `0x1` in every defined bit (Table 534, PDF p504).
Before `IO_BANK0` or `PADS_BANK0` acknowledges a single write, you deassert its
bit. In the tree this lives in two places: the three helpers
`clr_reset_reg` / `set_reset_reg` / `wait_for_reset_done` in
`firmware/pico2/src/common/reset.rs`, and the `Block` implementation in
`gpio/gpio.rs` that calls them with the GPIO mask.

### 9.2.1 The register table and the polarity flip

| Offset | Name | Info |
|---|---|---|
| `0x0` | `RESET` | one bit per component, `1` = held in reset |
| `0x4` | `WDSEL` | one bit per component, `1` = also reset when the watchdog fires |
| `0x8` | `RESET_DONE` | read-only status, `1` = out of reset |

(Table 533, PDF p504.) The three registers all use the same bit numbering.
The bits this firmware touches:

| Bits | Field | Type | Reset |
|---|---|---|---|
| 10 | `PADS_QSPI` | RW | `0x1` |
| 9 | `PADS_BANK0` | RW | `0x1` |
| 7 | `IO_QSPI` | RW | `0x1` |
| 6 | `IO_BANK0` | RW | `0x1` |

(Table 534, PDF p504. `RESET_DONE` mirrors the same positions as `RO`,
Table 536, PDF p506.)

The polarity reads backwards the first time. From the SDK struct commentary in
§7.5.2 (PDF p503):

> `reset`: This register contains a bit for each component that can be reset.
> When set to 1, the reset is asserted. If the bit is cleared, the reset is
> deasserted.
>
> `reset_done`: This register contains a bit for each component that is
> automatically set when the component is out of reset. This allows software to
> wait for this status bit in case the component requires initialisation before
> use.

| Register | Bit = 1 means | You |
|---|---|---|
| `RESET` | held **in** reset | **write 0** to release |
| `RESET_DONE` | **out of** reset, ready | **wait for 1** |

Same bit position, opposite sense. You are not "setting a reset bit" — bits 6
and 9 are already `1` at power-on and you are clearing them. The verb is
**deassert**.

`RESET_DONE` is read-only, so there is nothing to clear there. There is no
interrupt for it and deassertion takes a few cycles to propagate, so a busy-wait
is the intended and only mechanism — which is exactly what the quoted sentence
above says it is for.

### 9.2.2 Atomic aliases beat read-modify-write

Chapter 07 §7.5 has the mechanism: every peripheral register block is decoded
four times, at `+0x0000` (normal), `+0x1000` (XOR), `+0x2000` (bitmask set) and
`+0x3000` (bitmask clear) (datasheet §2.1.3, PDF p27). What matters here is that
`RESETS` supports it — it is not on the exclusion list, unlike SIO (§9.5) — so
writing `0x240` to `+0x3000` clears exactly bits 6 and 9 and leaves the other 27
untouched, in one store, with no read. The four aliases occupy 16 kB in total,
so the window `0x40020000`-`0x40023fff` belongs to `RESETS` and is free:
`IO_BANK0` does not begin until `0x40028000` (PDF p33).

**The firmware does not use the alias.** `clr_reset_reg` does a plain
read-modify-write at offset `0x0`, and `Block::start` then polls `RESET_DONE`
at `0x8` through `wait_for_reset_done`; both are quoted and walked in
chapter 08 §8.5. `IO_PAD_BITMASK` there is `(1 << 6) | (1 << 9)` = `0x240`,
and `start` passes its **complement** — `clr_reset_reg(!IO_PAD_BITMASK)` —
because the helper computes `RESET &= mask`.

The alias version below is an **improvement over what is in the tree**, not a
description of it. It is one store instead of a load, an AND and a store, and it
cannot lose a concurrent write from the other core:

```rust
// PROPOSED — not in the tree today
pub const RESETS_BASE: usize = 0x4002_0000;
const RESETS_RESET_CLR:  *mut u32   = (RESETS_BASE + 0x3000) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x8) as *const u32;

const RESET_IO_BANK0:   u32 = 1 << 6;
const RESET_PADS_BANK0: u32 = 1 << 9;

unsafe fn unreset(mask: u32) {
    unsafe {
        RESETS_RESET_CLR.write_volatile(mask);                   // 0 = deassert
        while RESETS_RESET_DONE.read_volatile() & mask != mask {} // 1 = ready
    }
}
```

(Note the mask polarity difference from the shipping code: the `+0x3000` alias
takes the bits to clear directly, so no complement.) Two details carry weight,
and the firmware gets both of them right:

- **`read_volatile` in the loop condition.** A plain read is loop-invariant, so
  LLVM hoists it out and you spin on a stale value forever. The firmware's
  `reset_done.read_volatile()` in `wait_for_reset_done` is inside the `while`
  condition, which is where it has to be.
- **`& mask != mask` waits for both bits.** `!= 0` falls through as soon as
  either one is ready, which on a good day means you configure the pad block
  while it is still in reset and every store is discarded.

### 9.2.3 Bit 7 and bit 10 will kill the running image

| Bit | Block | |
|---|---|---|
| 6 | `IO_BANK0` | yours |
| **7** | **`IO_QSPI`** | **flash pins** |
| 9 | `PADS_BANK0` | yours |
| **10** | **`PADS_QSPI`** | **flash pads** |

(Table 534, PDF p504.)

> **Hardware-destructive.** You are executing from flash over
> `IO_QSPI`/`PADS_QSPI`. Assert either and the XIP window stops answering
> mid-instruction-fetch, with no recoverable fault — fetching the fault
> handler also goes through flash. `0x240` is correct; `0x480` is a power
> cycle. They are one nibble apart, which is reason enough to name the masks
> rather than write a literal at the call site — which the firmware does:
> `IO_PAD_BITMASK` is built from two named bit positions, and the doc comment
> on `set_reset_reg` carries exactly this warning.

### 9.2.4 The two directions the firmware does implement

The reset helpers come in a symmetric pair, and `Block` uses both directions
(chapter 08 §8.6):

- **`start`** releases: `clr_reset_reg(!IO_PAD_BITMASK)` then
  `wait_for_reset_done(IO_PAD_BITMASK)`.
- **`stop`** re-asserts: `set_reset_reg(IO_PAD_BITMASK)` puts both GPIO blocks
  back into reset. There is no `wait` counterpart on this path — `RESET_DONE`
  reports readiness, and a block held in reset simply never reports ready.

What the firmware does **not** do is the SDK's assert-then-deassert *init*
idiom — `reset_block()` / `unreset_block_wait()` used back-to-back at startup
(§7.5.2, PDF p503):

```rust
// PROPOSED — not in the tree today
unsafe fn reinit(mask: u32) {
    RESETS_RESET_SET.write_volatile(mask);   // +0x2000 alias — assert
    unreset(mask);                           // clear, then poll
}
```

That is not cleanup. It forces a peripheral to known defaults *before* you
configure it, so you do not inherit whatever the bootrom or a previous run left
behind — which matters after a soft reset that did not clear the block.
`Block::start` only clears, on the assumption that the image runs from a cold
boot where the bits are already `1`.

`stop` has a caveat the tree's own doc comment states: any pin handle handed
out earlier keeps compiling after `stop`, but the block behind it is in reset,
so writes through it are accepted by the bus and discarded. Nothing in the
type system connects a `Rp2350GpioPin` to the `Block` that made it usable.

### 9.2.5 Reset is the wrong granularity for a resource allocator

If you are building an RTOS where tasks acquire and release GPIO, reasserting
the reset bit on release looks like hygiene. It does not work: **bit 6 is one
bit for all 30 pins of Bank 0.** Releasing GP25 that way yanks GP2 out from
under another task, and bit 9 takes every pad in the bank with it.

The per-pin park primitive is `ISO` (§9.3.1) — set it and the pad's control
inputs are latched at their current values, holding the pin's state until the
latch is cleared again. That is what a per-pin release should drive; see §9.6.

Block reset would not restore the pin anyway. Datasheet §9.7 (PDF p595):

> The ISO control bits are not reset by the PADS register block reset driven by
> the RESETS control registers: resetting the PADS register block returns
> non-isolated pads to their reset state, but has no effect on isolated pads.

So a per-pin release must set `ISO = 1` explicitly regardless — at which point
it has already restored the full power-on state and the reset bit adds nothing.

If you refcount the bank and want to reassert once the count hits zero, weigh
one thing first: a block in reset discards writes silently, so a
use-after-release through a stale pointer becomes an invisible no-op instead of
a visibly wrong pin. Leaving the block live and catching the error in the type
system is the better failure mode. (And if the kernel owns any pin itself — a
status LED, a console UART — the count never reaches zero and the branch is
dead code.)

## 9.3 PADS_BANK0 in full

Expands chapter 08 §8.8. Base `0x40038000` (PDF p33). This is **the pad**:
drive strength, slew, pulls, input enable, isolation.

| Offset | Name | Info |
|---|---|---|
| `0x00` | `VOLTAGE_SELECT` | Voltage select. Per bank control |
| `0x04` | `GPIO0` | |
| `0x08` | `GPIO1` | |
| … | … | pad `n` at `0x04 + 4n` |
| `0x68` | `GPIO25` | the user LED pad — absolute `0x40038068` |
| … | … | |
| `0xc0` | `GPIO47` | |
| `0xc4` | `SWCLK` | debug-port pad |
| `0xc8` | `SWD` | debug-port pad |

(Table 850, PDF p783-785.)

The `+0x04` in `0x04 + 4n` is not slack. Offset `0x00` is `VOLTAGE_SELECT`, a
per-bank control that has nothing to do with any one pin. `IO_BANK0` does
**not** do this — it opens directly with `GPIO0_STATUS` at offset `0` (§9.4).
The two blocks are not parallel in shape, which is exactly why the mistake is
easy to make: index a pads array from offset `0` and every pin is off by one,
with `pads[0]` landing on `VOLTAGE_SELECT`.

That misfire is worse than a wrong pin. `VOLTAGE_SELECT` bit 0 sets the input
threshold for the whole bank: `0x0` → 3V3 (DVDD ≥ 2V5), `0x1` → 1V8
(DVDD ≤ 1V8), reset `0x0` (Table 851, PDF p785). Datasheet §9.6 (PDF p593)
states the consequence plainly — *"By default, the pad input thresholds are
valid for an IOVDD voltage between 2.5V and 3.3V"*. Write a pad configuration
word with an odd value there and you have told the chip the board runs at
1.8 V when it does not. No fault, no warning; just marginal receive thresholds
bank-wide.

The pad word itself, identical for every pin:

| Bits | Field | Type | Reset |
|---|---|---|---|
| 31:9 | *Reserved* | - | - |
| 8 | `ISO`: Pad isolation control | RW | **`0x1`** |
| 7 | `OD`: Output disable. Has priority over output enable from peripherals | RW | `0x0` |
| 6 | `IE`: Input enable | RW | **`0x0`** |
| 5:4 | `DRIVE`: Drive strength (`0x0` 2MA, `0x1` 4MA, `0x2` 8MA, `0x3` 12MA) | RW | `0x1` |
| 3 | `PUE`: Pull up enable | RW | `0x0` |
| 2 | `PDE`: Pull down enable | RW | **`0x1`** |
| 1 | `SCHMITT`: Enable schmitt trigger | RW | `0x1` |
| 0 | `SLEWFAST`: Slew rate control. 1 = Fast, 0 = Slow | RW | `0x0` |

(Table 852, `GPIO0`, PDF p785. The datasheet prints one identical table per
pin; GP25's own copy is Table 877, PDF p798, which is the one chapter 08 §8.8
cites.) The full reset word is therefore `0x116`:
`ISO` `0x100` | `DRIVE=0x1` `0x010` | `PDE` `0x004` | `SCHMITT` `0x002`. That
number comes back in §9.6.

Datasheet §9.3 (PDF p587) states the requirement that costs the most
people the most time:

> Applications must enable the pad input (`GPIO0.IE = 1`) and disable pad
> isolation latches (`GPIO0.ISO = 0`) before using the pads for digital I/O.

Two of those three defaults are therefore wrong for output: `IE` resets to `0`
and `ISO` resets to `1`. `OD` already resets to `0`, which is why chapter 08's
bring-up clears a bit that was already clear — the firmware writes both in one
read-modify-write and the compiler folds them into a single `bfi`.

One field pair the datasheet flags separately: setting `PUE` and `PDE`
together does not enable both resistors, it enables **bus keeper mode**, where
the pad is pulled toward whatever level it currently reads — a weak latch that
holds the last driven level (datasheet §9.6.1, PDF p594). In a `set_pull()`
helper, `Up | Down` is therefore not a nonsense argument — it is a third mode
with its own name, and it stops working when the core is powered down. This is
why the firmware's `configure_gpio_pin_in` writes **both** `PUE` and `PDE` on
every path (chapter 08 §8.11): setting one without clearing the other leaves
the pad in bus-keeper mode, which is legal and almost never intended.

### 9.3.1 The isolation latch

Datasheet §9.7 (PDF p594):

> To ensure that pad states are well-defined at all times, all signals passing
> from the switched core power domain to the pads pass through isolation
> latches. In normal operation, the latches are transparent [...] However, when
> the ISO bit for each pad is set (e.g. `GPIO0.ISO`) or the switched core domain
> is powered down, the control signals currently presented to that pad are
> latched until the isolation is disabled. **This includes the output enable
> state, output high/low level, and pull-up/pull-down resistor enable.** The
> input signal from the pad back into the switched core domain is not isolated.

So the mechanism is a latch on every control signal crossing from the core
domain to the pad: while `ISO` is `1`, the pad keeps acting on the *latched*
values, and nothing you write to the mux, to SIO, or to the other pad fields
propagates through. The purpose is power cycling. RP2350 can power down the
entire switched core domain — everything except POWMAN and some CoreSight
logic — and without the latches every pad would glitch on the way in and out.
Pads hold their pre-power-down state, and on power-up *"all the GPIO ISO bits
reset to 1, so the pre-power down state continues to be maintained until user
software starts up and clears the ISO bit to indicate it is ready to use the
pad again"* (datasheet §9.7, PDF p594).

The lifecycle: `ISO` resets to `1` → software configures the mux and the pad —
none of it visible at the pin, because the latch is holding the old control
values → software clears `ISO`, and the pad switches once, from its held state
directly to the fully configured one. No intermediate state ever reaches the
pin. That is why clearing `ISO` is the last store in both configuration
functions (chapter 08 §8.10), not the first.

> **Silent-failure trap — three of them, all producing a dark pin.**
>
> 1. **Resetting PADS does not clear `ISO`.** Datasheet §9.7 (PDF p595):
>    *"The ISO control bits are not reset by the PADS register block reset
>    driven by the RESETS control registers: resetting the PADS register block
>    returns non-isolated pads to their reset state, but has no effect on
>    isolated pads."* Cycling the peripheral reset will not revive a pin left
>    isolated. Only an always-on domain reset clears the latches: power-on
>    reset, brownout, `RUN` asserted low, SW-DP `CDBGRSTREQ`, or an RP-AP
>    rescue reset (datasheet §9.7, PDF p594).
> 2. **Setting `ISO` freezes whatever the pad is doing right now.** It is not
>    a "park safely" button. Isolate a pin mid-drive and it holds that level
>    indefinitely, including through a block reset. Drop `OE` first — §9.6.
> 3. **None of this existed on RP2040.** Datasheet §9.7 (PDF p594):
>    *"Non-SDK applications ported from RP2040 must clear the ISO bit before
>    using a GPIO, as this feature was not present on RP2040."* Nearly every
>    blink tutorial online is RP2040-era and omits both `ISO` and `IE`, which
>    is how you end up with a correctly configured pin that stays dark and no
>    way to tell.

## 9.4 IO_BANK0 in full

Expands chapter 08 §8.9. Base `0x40028000` (PDF p33). This is **the mux**:
which peripheral is connected to the pin.

| Offset | Name | Info |
|---|---|---|
| `0x000` | `GPIO0_STATUS` | `STATUS` at `0x000 + 8n` |
| `0x004` | `GPIO0_CTRL` | `CTRL` at `0x004 + 8n` |
| `0x008` | `GPIO1_STATUS` | |
| … | … | 8-byte stride, 48 pins |
| `0x0c8` | `GPIO25_STATUS` | absolute `0x400280c8` |
| `0x0cc` | `GPIO25_CTRL` | absolute `0x400280cc` |
| … | … | |
| `0x178` | `GPIO47_STATUS` | |
| `0x17c` | `GPIO47_CTRL` | pin array ends at `0x180` |
| `0x180` | *reserved* | **128 bytes, 32 words, no registers** |
| `0x200` | `IRQSUMMARY_PROC0_SECURE0` | 12 `IRQSUMMARY` registers, `0x200`-`0x22c` |
| `0x230` | `INTR0` | Raw Interrupts — 6 registers, `0x230`-`0x244` |
| `0x248` | `PROC0_INTE0` | Interrupt Enable for proc0 |

(Table 648, PDF p603-606. `GPIO25_STATUS` at `0x0c8` and `GPIO25_CTRL` at
`0x0cc` appear on PDF p604 and again, with their field maps, on PDF p649-650.)

Note the **8-byte stride** — each pin has a `STATUS` and a `CTRL` — against
the pad block's 4-byte stride, and note that unlike `PADS_BANK0` (§9.3) there
is no leading per-bank register: the pin array starts at offset `0`.

The 128-byte hole at `0x180` is the one to get right in a struct. `48 * 8 =
0x180` ends the pin array, and `(0x200 - 0x180) / 4 = 32` reserved words follow
before `IRQSUMMARY`. The SDK's own header agrees — `uint32_t _pad0[32]`
between `io[48]` and the summary registers (datasheet §9.10.1, PDF p596).
Miscount it and every interrupt register silently addresses something else;
chapter 07 §7.6 shows how to prove the count with a host-side offset check.

There are 12 `IRQSUMMARY` registers rather than RP2040's none because RP2350
doubles the GPIO interrupt set to separate Secure from Non-secure, and adds
summaries so you can see what is pending without reading six `INTR` words
(datasheet §9.2, PDF p586).

### 9.4.1 `GPIOn_CTRL`

| Bits | Field | Type | Reset |
|---|---|---|---|
| 31:30 | *Reserved* | - | - |
| 29:28 | `IRQOVER` | RW | `0x0` |
| 27:18 | *Reserved* | - | - |
| 17:16 | `INOVER` | RW | `0x0` |
| 15:14 | `OEOVER` | RW | `0x0` |
| 13:12 | `OUTOVER` | RW | `0x0` |
| 11:5 | *Reserved* | - | - |
| 4:0 | `FUNCSEL`: 0-31 → selects pin function according to the gpio table, 31 == NULL | RW | **`0x1f`** |

(Table 700, `GPIO25_CTRL`, PDF p650. The layout is identical for every pin; only
the `FUNCSEL` enumeration differs — see §9.4.2.)

The four `OVER` fields are override muxes sitting between the selected
peripheral and the pad. All are 2 bits, and three of the four share an
encoding:

| Value | `OUTOVER` / `INOVER` / `IRQOVER` | `OEOVER` |
|---|---|---|
| `0x0` | NORMAL — pass the peripheral signal | NORMAL — drive OE from the peripheral signal |
| `0x1` | INVERT | INVERT — drive OE from the inverse |
| `0x2` | LOW — drive low | **DISABLE: disable output** |
| `0x3` | HIGH — drive high | **ENABLE: enable output** |

(Table 700, PDF p650.) `OEOVER`'s `0x2`/`0x3` are DISABLE/ENABLE rather than
LOW/HIGH — the same two-bit encoding with a different meaning, because it is a
direction control and "drive the direction low" would be a strange thing to
write.

For a freshly reset pin all four `OVER` fields already want `0` and the
reserved bits want `0`, so the entire correct value is the `FUNCSEL` number.
That is why the firmware's

```rust
        const SIO: u32 = 5;
        io_ctrl.write_volatile(SIO);
```

is a plain store rather than a read-modify-write, and why the plain store is
the *safer* of the two here: it guarantees the overrides are NORMAL instead of
inheriting whatever a previous owner left. Save the read-modify-write form for
a general `set_function()` that must preserve overrides.

**Debug technique:** forcing `OUTOVER = 0x3` drives the pin high regardless of
what the peripheral says, without touching SIO at all. That isolates "is the
mux and pad path working?" from "is my SIO code right?" in one store.

### 9.4.2 `FUNCSEL` is a per-pin enumeration

`FUNCSEL` values are **not** a global table. Each pin's `CTRL` register carries
its own enumeration. For GPIO25 (Table 700, PDF p650-651):

| Value | Function |
|---|---|
| `0x01` | `SPI1_SS_N` |
| `0x02` | `UART1_RX` |
| `0x03` | `I2C0_SCL` |
| `0x04` | `PWM_B_4` |
| `0x05` | **`SIO_25`** — what the firmware writes |
| `0x06` | `PIO0_25` |
| `0x07` | `PIO1_25` |
| `0x08` | `PIO2_25` |
| `0x09` | `CLOCKS_GPOUT_3` |
| `0x0a` | `USB_MUXING_VBUS_DETECT` |
| `0x1f` | `NULL` |

`0x05 = SIO` is consistent across pins — `SIO_1` on GPIO1, `SIO_25` on GPIO25
(PDF p611, p651) — but the others are not. `0x02` is `UART1_RX` on GPIO25 and
`UART0_RX` on GPIO1. A named `FUNCSEL_UART_RX` shared across pins would be
wrong; `FUNCSEL_SIO` is the only value that generalises safely.

Note also that GPIO25 has no `0x00` entry. On GPIO1 that value is `JTAG_TMS`
(PDF p610); on GPIO25 the enumeration simply starts at `0x01`.

### 9.4.3 `GPIOn_STATUS` — the debug register

Read-only, at `0x000 + 8n` (`0x0c8` for GPIO25):

| Bits | Field | Type | Reset |
|---|---|---|---|
| 31:27 | *Reserved* | - | - |
| 26 | `IRQTOPROC`: interrupt to processors, after override is applied | RO | `0x0` |
| 25:18 | *Reserved* | - | - |
| 17 | `INFROMPAD`: input signal from pad, before filtering and override are applied | RO | `0x0` |
| 16:14 | *Reserved* | - | - |
| 13 | `OETOPAD`: output enable to pad after register override is applied | RO | `0x0` |
| 12:10 | *Reserved* | - | - |
| 9 | `OUTTOPAD`: output signal to pad after register override is applied | RO | `0x0` |
| 8:0 | *Reserved* | - | - |

(Table 699, `GPIO25_STATUS`, offset `0x0c8`, PDF p649. The layout is identical
for every pin.)

When the LED stays dark, read this register first. `OUTTOPAD` and `OETOPAD` are
sampled *after* the overrides, so if both are `1` the mux is doing its job and
the problem is downstream in the pad — meaning `ISO` or `IE` (§9.3). If they
are `0`, the problem is upstream in `FUNCSEL` or in SIO. One read halves the
search space, and it is the only observation point this firmware has: there is
no UART and no logging of any kind (§9.7).

## 9.5 SIO in full

Expands chapter 08 §8.7 and §8.11. Base `0xd0000000` (PDF p35). This is **the
value**: output level, direction, input.

SIO is not in the `0x4xxxxxxx` peripheral range because it does not hang off
the APB at all — it is attached directly to each core, which is what makes it
single-cycle. Parts of it are per-core (`CPUID` reads back the core that
issued the load, and each core has its own FIFOs), but the GPIO registers
specifically are shared between both cores — see the end of §9.5.1.
`0xd0000000` is the **Secure** bank; `0xd0020000` is Non-secure (PDF p35). A
Secure image — which this one is, per its `IMAGE_DEF` (chapter 05) — uses
`0xd0000000`.

Table 16 transcribed in full through the GPIO registers, with nothing elided.
The two entries a condensed listing tends to drop are exactly the two that
break a struct:

```text
0x000  CPUID              <-- a real register, not padding
0x004  GPIO_IN
0x008  GPIO_HI_IN
0x00c  --- reserved ---   <-- a real hole, no register here
0x010  GPIO_OUT
0x014  GPIO_HI_OUT
0x018  GPIO_OUT_SET
0x01c  GPIO_HI_OUT_SET
0x020  GPIO_OUT_CLR
0x024  GPIO_HI_OUT_CLR
0x028  GPIO_OUT_XOR
0x02c  GPIO_HI_OUT_XOR
0x030  GPIO_OE
0x034  GPIO_HI_OE
0x038  GPIO_OE_SET
0x03c  GPIO_HI_OE_SET
0x040  GPIO_OE_CLR
0x044  GPIO_HI_OE_CLR
0x048  GPIO_OE_XOR
0x04c  GPIO_HI_OE_XOR
0x050  FIFO_ST            <-- inter-core FIFOs; end of the GPIO registers
```

(Table 16, §3.1.11, PDF p55-56.)

`CPUID` at `0x000` reads back the index of the core executing the load —
"Processor core identifier" in the register list. It is not GPIO, but it is
the first register in the block and an RTOS wants it from its first line, so do
not model it as reserved. The firmware names it even though it never reads it.

`0x00c` is the only hole in this range. Nothing in the datasheet flags it
beyond the jump in the offset column, and it is the reason the block is *not*
a clean sequence of LO/HI pairs. Chapter 07 §7.6 shows what a `u64` field does
to the offsets on either side of it.

The SET/CLR/XOR registers exist as real registers here precisely because SIO
is on the exclusion list for the `+0x1000`/`+0x2000`/`+0x3000` atomic aliases
(§2.1.3, PDF p27). The firmware's `Write<bool>` implementation is built on two
of them: `write(true)` stores `1 << pin` to `GPIO_OUT_SET`, `write(false)`
stores the same mask to `GPIO_OUT_CLR` — one store either way, no
read-modify-write, no race with the other core (chapter 08 §8.11).
`GPIO_OUT_XOR` would invert a pin in one store the same way; the tree does not
use it today — the blink is written as explicit set and clear, not as a
toggle.

One naming note, because it will bite you when you grep. The datasheet writes
`GPIO_HI_IN`, `GPIO_HI_OUT_SET`, `GPIO_HI_OE`; the firmware's `Sio` struct
writes `gpio_in_hi`, `gpio_out_set_hi`, `gpio_oe_hi` — the `HI` is a suffix in
the code and an infix in the datasheet. The offsets are right either way (the
struct is verified field by field in chapter 07 §7.6); only the search string
differs.

### 9.5.1 They are 32-bit registers, not a 64-bit pin vector

The LO/HI pairing is tempting to collapse into one 64-bit value. Datasheet
§9.8 (PDF p595) forecloses it:

> The `GPIO_OUT` and `GPIO_HI_OUT` registers set the output level: 1 = high,
> 0 = low [...] **These registers are all 32 bits in size.** The low registers
> (e.g. `GPIO_OUT`) connect to GPIOs 0 through 31, and the high registers (e.g.
> `GPIO_HI_OUT`) connect to GPIOs 32 through 47, the QSPI pads, and the USB
> DM/DP pads.

Two consequences follow.

**The high register is not a continuation of the low one.** Only its bottom 16
bits are GPIOs. Table 23 (PDF p64) gives the rest. It is printed for
`GPIO_HI_OUT_SET`; Table 25, `GPIO_HI_OUT_CLR`, repeats it identically on the
same page, and **inferred from those two:** the layout is common to every
`GPIO_HI_*` register.

| Bits | Field | Type | Reset |
|---|---|---|---|
| 31:28 | `QSPI_SD` | WO | `0x0` |
| 27 | `QSPI_CSN` | WO | `0x0` |
| 26 | `QSPI_SCK` | WO | `0x0` |
| 25 | `USB_DM` | WO | `0x0` |
| 24 | `USB_DP` | WO | `0x0` |
| 23:16 | *Reserved* | - | - |
| 15:0 | `GPIO` | WO | `0x0000` |

> **Hardware-destructive.** Treat the LO/HI pair as a flat 64-bit pin vector
> and "bit 58" is `QSPI_SCK` — bit 26 of the high register, not GPIO58, which
> does not exist. That is the same hazard as `RESETS` bit 7 (§9.2.3) and just as
> unrecoverable while running from XIP: you have taken the clock away from the
> flash you are fetching instructions out of. A single `1 << n` mask with
> `n < 48` written to the correct half is fine. A blanket write to a `u64` view,
> or any mask formed by shifting past bit 15 of the high register, is not.

**There is no 64-bit memory-mapped access.** If you want all 64 bits in one
instruction, the mechanism is the GPIO coprocessor on coprocessor port 0, not
the MMIO window. Datasheet §9.9 (PDF p595):

> The equivalent of any SIO GPIO register access is a single instruction,
> without having to materialise a 32-bit register address beforehand [...]
> 64 bits can be read/written in a single instruction

That is `mrrc`/`mcrr` against `p0`, documented in datasheet §3.6.1 — a
different mechanism entirely from the register block at `0xd0000000`, and one
this firmware does not use.

Two more facts worth keeping, both from datasheet §9.8 (PDF p595). The DMA
cannot access SIO at all; the recommended path to DMA-driven GPIO is a PIO
program that continuously transfers TX FIFO data to the GPIO outputs. And the
SIO GPIO registers are shared between both processors and both security
domains deliberately, which "avoids programming errors introduced by selecting
multiple GPIO functions for access from different contexts". There is no
per-core `GPIO_OUT`.

> **Silent-failure trap — the offsets moved from RP2040.** `GPIO_OE` is
> `0x030` on RP2350 (Table 16, PDF p55). The RP2040 offset — `0x020`, which on
> RP2350 is `GPIO_OUT_CLR` — is **not cited**: it belongs to the RP2040
> datasheet, which this tutorial has not consulted. Check it there before
> relying on it; the point below stands whatever the old number was. Port an
> RP2040 driver verbatim and your "enable the output" store clears an output
> level instead: the pin stays an input, the LED stays dark, and every register
> address in your code is a valid, writable, silently wrong one. The
> interleaved LO/HI layout is what moved them — RP2040 had no high registers to
> interleave.

## 9.6 Releasing a pin

The firmware has half of this story. What exists (chapter 08 §8.11): an owned
pin type, `Rp2350GpioPin`, whose private field and private constructors mean
the only way to obtain one is through the `Gpio` factory, and whose
construction is where the pin number is validated and the hardware configured.
What does **not** exist: any way to give a pin back. There is no
`release_pin`, no `Drop` implementation, and no claim mask — and nothing stops
you calling `init_output(25)` twice, either: the driver keeps no per-pin
bookkeeping, so a second call happily reconfigures the same pad and returns a
second owning handle. `demo` takes GP25 at boot and never gives it back, so
none of this bites today.

This section is the design sketch for the release path, kept here because
§9.2.5 and §9.3.1 between them determine most of it and those constraints are
easy to lose.

Release is the reverse of bring-up, and the order is load-bearing:

```rust
// PROPOSED — not in the tree today
unsafe fn release_pin(n: u32) { unsafe {
    SIO_GPIO_OE_CLR.write_volatile(1 << n);        // 1. stop driving
    pads_bank0(n).write_volatile(0x016);           // 2. pad defaults, ISO clear
    io_bank0_ctrl(n).write_volatile(FUNCSEL_NULL); // 3. disconnect mux (0x1f)
    pads_bank0(n).write_volatile(0x116);           // 4. re-isolate — parked
}}
```

`0x116` is the `PADS_BANK0.GPIOn` power-on value read straight off Table 852
(PDF p785): `ISO` `0x100` | `DRIVE=0x1` `0x010` | `PDE` `0x004` | `SCHMITT`
`0x002`. `0x016` is the same word with `ISO` still clear. Writing the whole
word rather than masking bits is what makes "restore the reset state" a single
store you can check against one table.

Two ordering constraints:

- **Clear `OE` before setting `ISO`.** Datasheet §9.7 (PDF p594) latches "the
  output enable state, output high/low level, and pull-up/pull-down resistor
  enable". Isolate a pin that is still driving the LED and it stays lit
  permanently, surviving even a block reset — see trap 1 in §9.3.1.
- **Clear `IE` before `FUNCSEL = NULL`.** Errata RP2350-E9 (PDF p1359) names
  conditions 2 and 3 as "Input buffer is enabled in `GPIO0.IE`" and "Output
  buffer is disabled (e.g. selecting the NULL GPIO function)". Nulling the mux
  while `IE` is still set walks the pad straight through that pair. Step 2
  above writes `0x016`, which has `IE = 0`, before step 3 nulls the function.

Steps 2 and 4 are split rather than folded into a single `0x116` store so that
`ISO` rises strictly after the configuration it latches has settled.
**Inferred:** a single store is probably fine, since the pad sees one write
either way; the split costs one APB write and removes the question, which on a
release path that runs once per pin is a good trade.

In Rust this is what belongs in `Drop`, on the pin type the tree already has:

```rust
// PROPOSED — not in the tree today
impl Drop for Rp2350GpioPin {
    fn drop(&mut self) {
        unsafe { release_pin(self.pin_no as u32) }
        CLAIMED.fetch_and(!(1 << self.pin_no), Ordering::Release);
    }
}
```

A move-only pin makes handing it to a task an ownership transfer — that much
the tree's `Rp2350GpioPin` already provides. What the `CLAIMED: AtomicU32`
claim mask adds is uniqueness at *construction*: `init_output` would
`compare_exchange` the pin's bit before configuring, so two calls for pin 25
cannot both succeed. It needs real atomics rather than a plain `static mut`:
the M33 is dual-core and both cores share one SIO block (datasheet §9.8,
PDF p595), so a read-modify-write on a plain static loses claims.
`compare_exchange` lowers to `ldrex`/`strex` on ARMv8-M and works across cores
without a spinlock — the same primitive `api`'s `BOARD_CREATED` flag already
uses for `Board::take` (chapter 08 §8.12).

Note what this design does *not* do: it never touches `RESETS`. That is
§9.2.5 — bit 9 is one bit for the whole bank, and `ISO` is the only per-pin
park the hardware offers.

## 9.7 What is not implemented yet

An honest inventory, so you do not go looking through the tree for code that
was never written. Everything below is absent from the workspace as of this
revision:

| Missing | Where it would go | Discussed in |
|---|---|---|
| Named `FUNCSEL` / pad-bit constants | a `gpio::regs` module | §9.3, §9.4.2 |
| Offset helper functions (`pads_bank0(n)`, `io_bank0_ctrl(n)`) | same | §9.6 |
| `release_pin`, `Drop` for `Rp2350GpioPin` | `gpio::gpio` | §9.6 |
| A `CLAIMED` mask — per-pin uniqueness at construction | `gpio::gpio` | §9.6 |
| An atomic-alias helper (`unreset()` via `+0x3000`) | `common::reset` | §9.2.2 |
| Assert-then-deassert re-init (`reinit()`) | `common::reset` | §9.2.4 |
| Interrupts of any kind — the NVIC, `INTR`, `PROC0_INTE0` | — | §9.4 |
| Clock setup — XOSC, the PLLs | — | chapter 08 §8.12.1 |
| Unit tests in `api` | `api` | chapter 07 §7.7 |

The module-level constants in `gpio.rs` are exactly three, all for `RESETS`:
`IOBANK_RESET_BIT`, `PADBANK_RESET_BIT` and `IO_PAD_BITMASK`. Every other
magic number — `IE`, `OD`, `ISO`, `PUE`, `PDE`, `SIO` — is a function-local
`const` inside the two configuration functions. The `pub` items in the `gpio`
module are `Rp2350Gpio`, `Rp2350GpioPin` and `GpioError`, plus the trait
implementations on them; the register structs — `GpioRegs`, `IoBank`,
`PadsBank`, `Sio` in `gpio/mod.rs` and `Reset` in `common/reset.rs` — are
private to their modules.

Two more absences worth naming because they change what "reference" means
here. There is no UART, no USB and no logging, so `GPIOn_STATUS` (§9.4.3) is
the only way to observe the hardware from inside the running image. And while
the `api` crate's traits are now implemented and consumed — `gpio.rs` imports
`api::common` and `api::gpio`, and `demo` drives the pin through them — the
crate still carries **no tests**, so the host-testability that justifies the
boundary is a capability, not yet a practice (chapter 07 §7.7).

---

That is the end of the tutorial. Chapters 01 to 08 are the path from an empty
directory to a blinking LED; this chapter is the reference you return to for
the numbers. The [index](index.md) lists both, along with the conventions every
chapter holds to and the date on which each number here was checked.
