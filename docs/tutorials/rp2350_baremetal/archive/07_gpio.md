---
document_type: Tutorial Chapter — GPIO and IO_BANK0
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 7 of 7
revision: B
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-06
sources: RP2350 datasheet 2.1.3, 2.2.4, 3.1, 3.1.11 (Table 16), 3.1.3 (Table 23), 3.6.1, 7.5, 7.5.3 (Tables 534/536), 9.1, 9.3, 9.6, 9.6.1, 9.7, 9.8, 9.9, 9.11 (Tables 700/850/852), 12.9.2; errata E9; docs/icd/rp2350/gpio/
---

# Chapter 07 — GPIO and IO_BANK0

Target: the on-board user LED on **GP25** (Pico 2 datasheet: *"GPIO25 OP —
Connected to user LED"*). It is a real chip GPIO wired on the PCB but
deliberately not brought out to a header — GP23-GP25 are reserved on-board.

## 7.1 The signal path

The blocks involved are not peers; they are **layers a signal passes through**:

```
CPU --> SIO --------> IO_BANK0 --------> PADS_BANK0 --------> physical pin
        "what          "which             "electrical
         value"         driver"            behaviour"
```

| Block | Base | Role |
|---|---|---|
| `RESETS` | `0x40020000` | gates whether the next two respond at all |
| `SIO` | `0xd0000000` | the value and direction |
| `IO_BANK0` | `0x40028000` | which peripheral is connected (the mux) |
| `PADS_BANK0` | `0x40038000` | drive, slew, pulls, input enable, isolation |

## 7.2 How many banks

**Two.** Bank 0 is user IO (30 GPIOs on the RP2350A/QFN-60 in the Pico 2, 48 on
the QFN-80); Bank 1 is the six QSPI IOs plus USB DP/DM.

The naming is inconsistent between chapters:

| Hardware | Address map (2.2.4) | ACCESSCTRL |
|---|---|---|
| Bank 0 IO mux | `IO_BANK0` @ `0x40028000` | `IO_BANK0` @ `0x68` |
| Bank 1 IO mux | **`IO_QSPI`** @ `0x40030000` | **`IO_BANK1`** @ `0x6c` |
| Bank 0 pads | `PADS_BANK0` @ `0x40038000` | `PADS_BANK0` @ `0x70` |
| Bank 1 pads | `PADS_QSPI` @ `0x40040000` | `PADS_QSPI` @ `0x74` |

So `IO_BANK1` *does* exist as a name — it just never appears in the address map.
The `0` in `IO_BANK0` means "the user bank", not "the first of N".

The blocks are grouped **by layer, not by bank**: the two IO muxes are adjacent,
then the two pad blocks. (`PADS = IO + 0x10000` holds for both banks; do not
build code on that coincidence.)

Bank 1 is owned by the bootrom's XIP setup — the flash you are executing from is
on those pins. Leave it alone.

## 7.3 Where the 32-bit limit actually bites

`IO_BANK0` and `PADS_BANK0` use **per-pin registers**, so nothing about a 32-bit
width constrains them; Bank 0 scales to 48 GPIOs by having a longer array.

**`SIO` is the bitmask block** — one bit per GPIO in a 32-bit register. That is
why `GPIO_HI_*` companions exist: `GPIO_OUT` covers GPIO 0-31, `GPIO_HI_OUT`
covers 32-47 *plus* the QSPI IOs and USB DP/DM in its upper bits.

On the Pico 2 (GPIO 0-29) everything fits in the low registers. GP25 is simply
bit 25.

## 7.4 RESETS — `0x40020000`

Every peripheral on the chip powers up **held in reset**. Before `IO_BANK0` or
`PADS_BANK0` will acknowledge a single write, you have to release them.

| Offset | Register | Type | Reset |
|---|---|---|---|
| `0x0` | `RESET` | RW | **`0x1` per bit** |
| `0x4` | `WDSEL` | RW | `0x0` |
| `0x8` | `RESET_DONE` | **RO** | `0x0` |

`IO_BANK0` = **bit 6**, `PADS_BANK0` = **bit 9** (Table 534).

### 7.4.1 The polarity flips between the two registers

This is the part that reads backwards the first time. From the SDK struct
commentary in 7.5.3:

> `reset`: When set to 1, the reset is asserted. If the bit is cleared, the
> reset is deasserted.
>
> `reset_done`: This register contains a bit for each component that is
> automatically set when the component is out of reset.

| Register | Bit = 1 means | You |
|---|---|---|
| `RESET` | held **in** reset | **write 0** to release |
| `RESET_DONE` | **out of** reset, ready | **wait for 1** |

Same bit position, opposite sense. You are not "setting a reset bit" — bits 6
and 9 are *already* 1 at power-on and you are clearing them. The verb is
**deassert**.

`RESET_DONE` is read-only (Table 536); there is nothing to clear there. It is a
status mirror, and the datasheet gives its purpose plainly: *"This allows
software to wait for this status bit in case the component requires
initialisation before use."* There is no interrupt for it, and deassertion takes
a few cycles to propagate. A busy-wait is the intended and only mechanism.

> **Silent-failure trap.** While a block is in reset its registers do not
> respond — writes are discarded, with no error and no fault. Every store to
> `0x40028000` before this point goes nowhere. Step 1 below is not housekeeping;
> it is the moment those two peripherals first become addressable at all.

### 7.4.2 Atomic aliases beat read-modify-write

§2.1.3 — every peripheral block is decoded four times:

```
+0x0000  normal read/write
+0x1000  atomic XOR on write
+0x2000  atomic bitmask SET on write
+0x3000  atomic bitmask CLEAR on write
```

Writing a mask to the `+0x3000` alias clears exactly those bits and leaves the
other 27 untouched, in one store, with no read. RESETS supports this natively —
the exclusion list is SIO, the CoreSight window, the PPB, and OTP-SBPI. The
16 kB alias window `0x40020000`-`0x40023fff` is free, since `IO_BANK0` does not
begin until `0x40028000`.

```rust
pub const RESETS_BASE: usize = 0x4002_0000;
const RESETS_RESET_CLR:  *mut u32   = (RESETS_BASE + 0x3000) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x8) as *const u32;

const RESET_IO_BANK0:   u32 = 1 << 6;
const RESET_PADS_BANK0: u32 = 1 << 9;

unsafe fn unreset(mask: u32) {
    unsafe {
        RESETS_RESET_CLR.write_volatile(mask);                      // 0 = deassert
        while RESETS_RESET_DONE.read_volatile() & mask != mask {}   // 1 = ready
    }
}
```

Two details carry weight:

- `read_volatile` in the loop condition. A plain read is hoisted out of the loop
  and you spin on a stale value forever.
- `& mask != mask` waits for **both** bits. `!= 0` falls through as soon as
  either one is ready.

### 7.4.3 Bit 7 and bit 10 will kill the running image

| Bit | Block | |
|---|---|---|
| 6 | `IO_BANK0` | yours |
| **7** | **`IO_QSPI`** | **flash pins** |
| 9 | `PADS_BANK0` | yours |
| **10** | **`PADS_QSPI`** | **flash pads** |

You are executing from flash over `IO_QSPI`/`PADS_QSPI`. Assert either and the
XIP window stops answering mid-instruction-fetch — with no recoverable fault,
because fetching the fault handler also goes through flash. `0x240` is correct;
`0x480` is a power cycle. They are one nibble apart, which is reason enough to
name the masks rather than write literals at the call site.

### 7.4.4 Assert-then-deassert is an *init* idiom, not teardown

The SDK's `reset_block()` / `unreset_block_wait()` pair (7.5.3) is used
back-to-back at startup:

```rust
unsafe fn reinit(mask: u32) {
    RESETS_RESET_SET.write_volatile(mask);   // +0x2000 alias — assert
    unreset(mask);                           // clear, then poll
}
```

That is not cleanup. It forces a peripheral to known defaults *before*
configuring it, so you do not inherit whatever the bootrom or a previous run
left behind — which matters after a soft reset that did not clear the block.

### 7.4.5 Reset is the wrong granularity for a resource allocator

If you are building an RTOS where tasks acquire and release GPIO, reasserting
the reset bit on release looks like good hygiene. It does not work: **bit 6 is
one bit for all 30 pins of Bank 0.** Releasing GP25 that way would yank GP2 out
from under another task, and bit 9 takes every pad in the bank with it.

The per-pin park primitive is `ISO` (§7.5.1) — freeze the pad, hold its state,
re-engage it later. That is what a per-pin `release` should drive; see §7.8.1.

Block reset would not restore the pin anyway. §9.7:

> The ISO control bits are not reset by the PADS register block reset driven by
> the RESETS control registers: resetting the PADS register block returns
> non-isolated pads to their reset state, but has no effect on isolated pads.

So a per-pin release must set `ISO = 1` explicitly regardless — at which point
it has already restored full power-on state and the reset bit adds nothing.

If you refcount the bank and want to reassert once the count hits zero, weigh
one thing first: a block in reset **discards writes silently**, so a
use-after-release through a stale pointer becomes an invisible no-op instead of
a visibly wrong pin. Leaving the block live and catching the error in the type
system is the better failure mode. (And if the kernel owns any pin itself — a
status LED, console UART — the count never reaches zero and the branch is dead
code.)

## 7.5 PADS_BANK0 — `0x04 + 4*n` (GP25 -> `+0x68`)

The `+0x04` in that formula is not slack. Offset `0x00` of this block is
`VOLTAGE_SELECT`, a **per-bank** control that has nothing to do with any one
pin (Table 850). The pin array starts one word later:

```
0x00  VOLTAGE_SELECT     per-bank input threshold
0x04  GPIO0
0x08  GPIO1
...
0xc0  GPIO47
0xc4  SWCLK
0xc8  SWD
```

`IO_BANK0` does **not** do this — it opens directly with `GPIO0_STATUS` at
offset `0`. The two banks are not parallel in shape, which is exactly why the
mistake is easy to make: index a pads array from offset `0` and every pin is
off by one, with `pads[0]` landing on `VOLTAGE_SELECT`.

That misfire is worse than a wrong pin. `VOLTAGE_SELECT` bit 0 sets the input
threshold for **all 30 pins at once** — 0 for IOVDD 2.5-3.3 V, 1 for 1.8 V
(9.6, Table 851). Write a pad config word with an odd value there and you have told the
chip the board runs at 1.8 V when it does not. No fault, no warning; just
marginal receive thresholds bank-wide.

| Bits | Field | Reset | Note |
|---|---|---|---|
| 8 | `ISO` | **1** | isolation latch — 1 = latched |
| 7 | `OD` | 0 | output disable (overrides peripheral OE) |
| 6 | `IE` | **0** | input enable |
| 5:4 | `DRIVE` | `0x1` | 00=2 mA, 01=4 mA, 10=8 mA, 11=12 mA |
| 3 | `PUE` | 0 | pull-up enable |
| 2 | `PDE` | **1** | pull-down enable |
| 1 | `SCHMITT` | 1 | Schmitt trigger |
| 0 | `SLEWFAST` | 0 | 0 = slow |

Section 9.3 gives the full reset state and the requirement:

> Applications must enable the pad input (`GPIO0.IE = 1`) and disable pad
> isolation latches (`GPIO0.ISO = 0`) before using the pads for digital I/O.

### 7.5.1 The isolation latch

Section 9.7 — the latch is a **configuration shield**. Verbatim:

> To ensure that pad states are well-defined at all times, all signals passing
> from the switched core power domain to the pads pass through isolation
> latches. In normal operation, the latches are transparent [...] However, when
> the ISO bit for each pad is set (e.g. `GPIO0.ISO`) or the switched core domain
> is powered down, the control signals currently presented to that pad are
> latched until the isolation is disabled. **This includes the output enable
> state, output high/low level, and pull-up/pull-down resistor enable.** The
> input signal from the pad back into the switched core domain is not isolated.

The purpose is power cycling: RP2350 can power down the entire switched core
domain, and without the latches every pad would glitch on the way in and out.
Pads hold their pre-power-down state, and on power-up *"all the GPIO ISO bits
reset to 1, so the pre-power down state continues to be maintained until user
software starts up and clears the ISO bit to indicate it is ready to use the pad
again."*

Lifecycle: `ISO` resets to 1 -> software configures the mux -> software clears
`ISO` to let signals reach the pad. Configure behind the shield, then open it
once, so the pin never glitches through intermediate states.

> **Three traps.**
>
> 1. **Resetting PADS does not clear `ISO`.** 9.7: *"The ISO control bits are
>    not reset by the PADS register block reset driven by the RESETS control
>    registers: resetting the PADS register block returns non-isolated pads to
>    their reset state, but has no effect on isolated pads."* Cycling the
>    peripheral reset will not revive a pin left isolated. Only an always-on
>    domain reset clears the latches — power-on reset, brownout, `RUN` asserted
>    low, SW-DP `CDBGRSTREQ`, or an RP-AP rescue reset.
> 2. **Setting `ISO` freezes whatever the pad is doing right now.** It is not a
>    "park safely" button. Isolate a pin mid-drive and it holds that level
>    indefinitely. Drop `OE` first — see §7.8.1.
> 3. **None of this existed on RP2040.** 9.7: *"Non-SDK applications ported from
>    RP2040 must clear the ISO bit before using a GPIO, as this feature was not
>    present on RP2040."* Nearly every blink tutorial online is RP2040-era and
>    omits both `ISO` and `IE` — you get a correctly-configured pin that stays
>    dark.

## 7.6 IO_BANK0 — `0x004 + 8*n` (GP25_CTRL -> `+0x0cc`)

Note the **8-byte stride** (each pin has `STATUS` and `CTRL`) versus PADS'
4-byte stride. Unlike `PADS_BANK0` (§7.5) there is no leading per-bank
register: the pin array starts at offset `0`.

```
0x000  GPIO0_STATUS       stride 8: STATUS at 0x000 + 8*n
0x004  GPIO0_CTRL                  CTRL   at 0x004 + 8*n
0x008  GPIO1_STATUS
...
0x178  GPIO47_STATUS
0x17c  GPIO47_CTRL
0x180  --- reserved ---   128 bytes, no registers
0x200  IRQSUMMARY_PROC0_SECURE0     12 IRQSUMMARY registers, 0x200-0x22c
...
0x230  INTR0                        6 INTR registers, 0x230-0x244
0x244  INTR5
0x248  PROC0_INTE0        interrupt enables begin
```

The 128-byte hole at `0x180` is the one to get right in a struct — `48*8 =
0x180` ends the pin array, and `(0x200 - 0x180) / 4 = 32` reserved words follow
before `IRQSUMMARY`. Miscount it and every interrupt register silently
addresses something else (§7.11.2).

### 7.6.1 `GPIOn_CTRL` (Table 700)

| Bits | Field | Type | Reset |
|---|---|---|---|
| 31:30 | *Reserved* | | |
| **29:28** | `IRQOVER` | RW | `0x0` |
| 27:18 | *Reserved* | | |
| **17:16** | `INOVER` | RW | `0x0` |
| **15:14** | `OEOVER` | RW | `0x0` |
| **13:12** | `OUTOVER` | RW | `0x0` |
| 11:5 | *Reserved* | | |
| **4:0** | `FUNCSEL` | RW | **`0x1f`** |

The four `OVER` fields are override muxes between the selected peripheral and
the pad. All 2 bits, all the same shape:

| Value | `OUTOVER` / `INOVER` / `IRQOVER` | `OEOVER` |
|---|---|---|
| `0x0` | NORMAL — pass the peripheral signal | NORMAL — OE from peripheral |
| `0x1` | INVERT | INVERT |
| `0x2` | drive LOW | **DISABLE** output |
| `0x3` | drive HIGH | **ENABLE** output |

`OEOVER`'s 2/3 are DISABLE/ENABLE rather than LOW/HIGH — same encoding,
different meaning, because it is a direction control.

**Debug technique:** forcing `OUTOVER = 0x3` drives the pin high regardless of
what the peripheral says, without touching SIO. That isolates "is the mux and
pad path working?" from "is my SIO code right?"

### 7.6.2 `FUNCSEL` is a per-pin enumeration

For GPIO25 specifically:

```
0x01 SPI1_SS_N     0x06 PIO0_25      0x09 CLOCKS_GPOUT_3
0x02 UART1_RX      0x07 PIO1_25      0x0a USB_MUXING_VBUS_DETECT
0x03 I2C0_SCL      0x08 PIO2_25      0x1f NULL
0x04 PWM_B_4       0x05 SIO_25  <-- what you want
```

`0x05 = SIO` is consistent across pins; the others are **not**. `0x02` is
`UART1_RX` on GP25 and something else on GP0.

### 7.6.3 Writing it

For a freshly-reset pin all four `OVER` fields already want `0` and the reserved
bits want `0`, so the whole correct value is just `FUNCSEL_SIO`:

```rust
ctrl_ptr.write_volatile(FUNCSEL_SIO);   // 0x0000_0005 - one store
```

That is *safer* than RMW here: it guarantees the overrides are NORMAL rather
than inheriting whatever was there. Save the RMW form for a general
`set_function()` that must preserve overrides.

### 7.6.4 `GPIOn_STATUS` — the debug register

Read-only, at `0x000 + 8*n` (`+0x0c8` for GP25):

| Bit | Field |
|---|---|
| 26 | `IRQTOPROC` |
| 17 | `INFROMPAD` — raw input from pad |
| 13 | `OETOPAD` — output enable to pad, after override |
| 9 | `OUTTOPAD` — output level to pad, after override |

> When the LED stays dark, read this first. If `OUTTOPAD` and `OETOPAD` are both
> 1, IO_BANK0 is doing its job and the problem is downstream — the pad, meaning
> `ISO` or `IE`. One read halves the search space.

## 7.7 SIO — `0xd0000000`

Not in the `0x4xxxxxxx` peripheral range: SIO hangs directly off the core,
outside the bus fabric — no arbitration, no APB wait states, hence
single-cycle. It is also **per-core**.

`0xd0000000` is the **Secure** bank; `0xd0020000` is Non-secure. A Secure image
uses `0xd0000000`.

Table 16 lists the block. Transcribed in full through the GPIO registers, with
nothing elided — the two things a condensed listing tends to drop are exactly
the two that break a struct:

```
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

`CPUID` at `0x000` reads back the index of the core executing the load. It is
not GPIO, but it is the first register in the block, and an RTOS wants it from
its first line — do not model it as reserved.

`0x00c` is the only hole in this range. Nothing in the datasheet flags it
beyond the jump in the offset column, and it is the reason the block is *not*
a clean sequence of lo/hi pairs.

> **The LO and HI registers interleave**, and the offsets differ from RP2040.
> `GPIO_OE` is `0x030` here; on RP2040 it was `0x020`, which is `GPIO_OUT_CLR`
> on RP2350. Port RP2040 code verbatim and you clear an output where you meant
> to enable one — silently.

> **Interleaved is not paired.** Each LO/HI pair is adjacent, but they are two
> independent 32-bit registers, and the `0x00c` hole means the pairs do not
> tile the block evenly. Modelling a pair as one 64-bit field is wrong twice
> over — see §7.7.1 for the hardware reason and §7.11.3 for what the compiler
> does to the offsets if you try.

`GPIO_OUT_XOR` toggles an LED in one write, with no read-modify-write and no
race.

### 7.7.1 They are 32-bit registers, not a 64-bit pin vector

The pairing is tempting to collapse. Section 9.8 forecloses it:

> The `GPIO_OUT` and `GPIO_HI_OUT` registers set the output level: 1 = high,
> 0 = low [...] **These registers are all 32 bits in size.** The low registers
> (e.g. `GPIO_OUT`) connect to GPIOs 0 through 31, and the high registers (e.g.
> `GPIO_HI_OUT`) connect to GPIOs 32 through 47, the QSPI pads, and the USB
> DM/DP pads.

Two consequences.

**The high register is not a continuation of the low one.** Only its bottom 16
bits are GPIOs. Table 23 gives the rest:

| Bits | Field |
|---|---|
| 31:28 | `QSPI_SD[3:0]` |
| 27 | `QSPI_CSN` |
| 26 | `QSPI_SCK` |
| 25 | `USB_DM` |
| 24 | `USB_DP` |
| 23:16 | reserved |
| 15:0 | GPIO32-47 |

Treat the pair as a flat 64-bit pin vector and "bit 58" is `QSPI_SCK` — the
same class of hazard as `RESETS` bit 7 (§7.4.3), and just as unrecoverable
while running from XIP. A single `1 << n` mask with `n < 48` is fine; a blanket
write is not.

**There is no 64-bit memory-mapped access.** If you want the whole 64 bits in
one instruction, the mechanism is the GPIO coprocessor on coprocessor port 0,
not the MMIO window (9.9):

> The equivalent of any SIO GPIO register access is a single instruction,
> without having to materialise a 32-bit register address beforehand [...]
> 64 bits can be read/written in a single instruction

That is `mrrc`/`mcrr` against `p0` (3.6.1) — a different mechanism entirely
from the register block at `0xd0000000`.

Two more facts worth keeping: SIO has **no atomic alias windows** (2.1.3
excludes it), which is why the SET/CLR/XOR registers exist as real registers
here instead. And the DMA cannot reach SIO at all (9.8) — the recommended path
to DMA-driven GPIO is a PIO program.

## 7.8 The bring-up sequence

| # | Block | Action |
|---|---|---|
| 1 | `RESETS.RESET` | **deassert** bits 6 and 9 — write `0` — then poll `RESET_DONE` for `1` (§7.4) |
| 2 | `IO_BANK0.GPIO25_CTRL` | `FUNCSEL = 5` (SIO) |
| 3 | `SIO.GPIO_OE_SET` | bit 25 — make it an output |
| 4 | `SIO.GPIO_OUT_CLR/SET` | choose the initial level |
| 5 | `PADS_BANK0.GPIO25` | `IE = 1`, `OD = 0` |
| 6 | `PADS_BANK0.GPIO25` | **clear `ISO = 0`** — the release |

Then toggle with `SIO.GPIO_OUT_XOR`, bit 25.

> **Why `FUNCSEL`/`OE` before `IE`** — errata **RP2350-E9**: with `IE=1`,
> `OE=0` and function NULL, a floating Bank-0 pad leaks ~120 µA and self-latches
> around 2.2 V. Select SIO and drive OE first, then enable the input, then
> release the isolation latch last (9.7).

### 7.8.1 Releasing a pin

The reverse of bring-up, and the order is load-bearing:

```rust
unsafe fn release_pin(n: u32) { unsafe {
    SIO_GPIO_OE_CLR.write_volatile(1 << n);        // 1. stop driving
    pads_bank0(n).write_volatile(0x016);           // 2. pad defaults, ISO transparent
    io_bank0_ctrl(n).write_volatile(FUNCSEL_NULL); // 3. disconnect mux (0x1f)
    pads_bank0(n).write_volatile(0x116);           // 4. re-isolate — parked
}}
```

`0x116` is the `PADS_BANK0.GPIOn` power-on value read straight off Table 852:
`ISO=1 | DRIVE=0x1 | PDE=1 | SCHMITT=1`. `0x016` is the same word with `ISO`
still clear.

Two ordering constraints:

- **Clear `OE` before setting `ISO`.** §9.7 latches "the output enable state,
  output high/low level, and pull-up/pull-down resistor enable". Isolate a pin
  that is still driving the LED and it stays lit permanently — surviving even a
  block reset, since block reset does not touch `ISO`.
- **Clear `IE` before `FUNCSEL = NULL`.** Errata E9 is specifically the `IE=1` +
  `OE=0` + function-NULL combination (§7.8). Nulling the mux first walks the pad
  straight through it.

Steps 2 and 4 are split rather than folded into one `0x116` store so `ISO` rises
strictly after the configuration it latches has settled. A single store is
probably fine; the split costs one APB write and removes the question.

In Rust this is what belongs in `Drop`:

```rust
pub struct Pin { n: u32 }

impl Drop for Pin {
    fn drop(&mut self) {
        unsafe { release_pin(self.n) }
        CLAIMED.fetch_and(!(1 << self.n), Ordering::Release);
    }
}
```

A move-only `Pin` makes handing it to a task an ownership transfer, so two tasks
cannot hold GP25 without someone writing `unsafe`. The `CLAIMED: AtomicU32`
claim mask needs real atomics rather than a plain `static mut` — the M33 is
dual-core and both cores share SIO. `compare_exchange` lowers to
`ldrex`/`strex` on ARMv8-M and works across cores without a spinlock.

## 7.9 The delay-loop trap

The prerequisite most likely to make *working* GPIO code look broken:

1. **`TIMER0` will not count.** `TICKS_TIMER0_CTRL.ENABLE` resets to `0x0` —
   new on RP2350; on RP2040 the watchdog produced the tick (12.9.2).
2. **A busy loop gets deleted.** At `opt-level=3` LLVM removes it entirely
   unless each iteration contains `core::hint::black_box` or
   `asm!("nop", options(nomem, nostack))`.
3. **The rate is uncalibrated anyway.** Nothing has started XOSC or the PLLs, so
   `clk_sys` runs from the ring oscillator (7.4, PSM step 6) and drifts with
   voltage and temperature.

Expect a first blink that is either instant or invisible. It is not the GPIO
code.

## 7.10 First-boot protocol

Success and failure look identical — a dark, silent board. There is exactly one
discriminator:

> If the bootrom **rejects** the image it falls through to USB Boot and the
> board re-enumerates as a mass-storage device.
>
> **Re-enumerates = rejected. Stays dark and does not enumerate = accepted and
> running.**

Flashing:

```
cp target/thumbv8m.main-none-eabihf/release/pico2 pico2.elf
picotool uf2 convert pico2.elf pico2.uf2 --family rp2350-arm-s
```

(picotool dispatches on file extension, so the extensionless Cargo output is
rejected.)

Since the firmware never enumerates USB, `picotool` cannot reboot the device —
**every reflash needs a physical BOOTSEL hold**. There is no brick risk: this
code writes two CPU registers and a handful of RAM bytes, and touches no OTP, no
QMI configuration, no partition table.

## 7.11 Modelling a register block as a `#[repr(C)]` struct

A `#[repr(C)]` struct over a peripheral base is the cheapest way to get named
registers with no runtime cost: field order is declaration order, and the
offsets fall out of the layout algorithm. The catch is that the layout
algorithm is doing arithmetic on your behalf, and it does not know what the
datasheet says. Three ways it goes wrong, all of them silent.

### 7.11.1 Leading per-bank registers

Covered in §7.5: `PADS_BANK0` opens with `VOLTAGE_SELECT`, `IO_BANK0` does not.
The rule is to transcribe offset `0x00` from the register list rather than
assume the block starts with the thing you came for.

```rust
#[repr(C)]
struct PadsBank {
    pub voltage_select: u32,   // 0x00  per-bank, not a pad
    pub pads: [u32; 48],       // 0x04 .. 0xc0
    pub swclk: u32,            // 0xc4
    pub swd: u32,              // 0xc8
}
```

`swclk`/`swd` are the debug-port pads. Naming them is correct — they are part
of the block — but think twice before letting a general pin allocator index
them, because a bug that reconfigures them disconnects your own debugger.

### 7.11.2 Reserved gaps

`IO_BANK0` has a 128-byte hole between `GPIO47_CTRL` (ending `0x180`) and
`IRQSUMMARY_PROC0_SECURE0` (`0x200`); `SIO` has a single reserved word at
`0x00c`. Neither is announced by anything except a jump in the offset column of
the register list. A `_reserved: [u32; N]` field is not padding-for-neatness,
it is a load-bearing part of the address arithmetic: get `N` wrong and every
field after it silently addresses a different register.

The way to check is to compile the struct for the **host** and print the
offsets against a null base, comparing each against the datasheet table:

```rust
let p: *const IoBank = core::ptr::null();
println!("{:#05x}", unsafe { &raw const (*p).irqsummary } as usize);  // want 0x200
```

`&raw const` is essential here — `&(*p).field` on a null pointer is instant UB.
This is a good use of the dual-target build from Chapter 02: the layout is a
property of `#[repr(C)]`, not of the target, so a host test proves it.

### 7.11.3 Alignment padding from wide fields

The most dangerous of the three, because it inserts bytes you never wrote.

`SIO` pairs a low and a high register for each operation, and it is tempting to
model each pair as one `u64`. Under `#[repr(C)]`, `u64` carries 8-byte
alignment, so the compiler inserts a 4-byte hole wherever a `u64` follows an
odd number of `u32`s:

```rust
#[repr(C)]
struct Sio {
    _reserved: u32,        // 0x00
    pub gpio_in: u64,      // intended 0x04 — ACTUALLY 0x08
    pub gpio_out: u64,     // 0x10
    // ...
}
```

`gpio_in` lands at `0x08`. It reads `GPIO_HI_IN` in its low half and the
reserved word at `0x00c` in its high half — never `GPIO_IN`, which is now
buried inside compiler-inserted padding and unreachable through the struct.

What makes this instructive is that every *other* field is at the right
address. The 4 bytes of alignment padding and the 4-byte reserved word at
`0x00c` happen to cancel, so `gpio_out` through `gpio_oe_set` all land
correctly. The struct half-works. It would pass a blink test and fail the first
time you read a pin — and any "tidy-up" (packing it, reordering it, giving
`cpuid` its real name) moves everything.

Two independent reasons not to reach for `u64` here at all:

* §7.7.1 — the datasheet says these are 32-bit registers, the high half is not
  a continuation of the low half, and the real 64-bit path is the GPIO
  coprocessor.
* Rust guarantees a `read_volatile::<u64>()` is not split or elided, but not
  that it becomes one bus transaction. On the AHB it is two 32-bit transfers
  regardless, in an order the source does not state.

Transcribe the register list literally instead:

```rust
#[repr(C)]
struct Sio {
    pub cpuid:           u32,  // 0x000  which core am I
    pub gpio_in:         u32,  // 0x004
    pub gpio_hi_in:      u32,  // 0x008
    _reserved0:          u32,  // 0x00c
    pub gpio_out:        u32,  // 0x010
    pub gpio_hi_out:     u32,  // 0x014
    pub gpio_out_set:    u32,  // 0x018
    pub gpio_hi_out_set: u32,  // 0x01c
    pub gpio_out_clr:    u32,  // 0x020
    pub gpio_hi_out_clr: u32,  // 0x024
    pub gpio_out_xor:    u32,  // 0x028
    pub gpio_hi_out_xor: u32,  // 0x02c
    pub gpio_oe:         u32,  // 0x030
    pub gpio_hi_oe:      u32,  // 0x034
    pub gpio_oe_set:     u32,  // 0x038
    pub gpio_hi_oe_set:  u32,  // 0x03c
    pub gpio_oe_clr:     u32,  // 0x040
    pub gpio_hi_oe_clr:  u32,  // 0x044
    pub gpio_oe_xor:     u32,  // 0x048
    pub gpio_hi_oe_xor:  u32,  // 0x04c
}                              // ends at 0x050 = FIFO_ST
```

Every field is `u32`, so the struct's alignment is 4 and no padding exists
anywhere. Ending the struct on a named register (`FIFO_ST`) rather than
mid-block makes it obvious that this is a deliberate partial view — the same
convention as `IoBank` stopping at `PROC0_INTE0`.

`CPUID` is worth naming rather than reserving. It is how a core identifies
itself, which an RTOS scheduler needs from its first line.

### 7.11.4 Two attributes to leave off

**`#[derive(Clone, Copy)]`.** It makes `let snapshot = *SIO;` compile, and that
expression is a non-volatile bulk read of every register in the block. LLVM may
reorder, coalesce or delete those loads, and some registers have read side
effects — `INTERP0_POP_LANE0` advances hardware state when read. An operation
you never want should not be expressible.

**`unsafe impl Sync`.** `u32` and arrays of `u32` are already `Sync`, so these
structs derive it. Writing it by hand costs nothing but spends the reader's
attention, and it dilutes the one place it is genuinely load-bearing: the
`Vector` union in Chapter 04, which holds raw pointers and does not get `Sync`
for free.

### 7.11.5 Never form a reference

The rule that makes the whole approach sound:

```rust
// NO — a real reference
let sio: &mut Sio = unsafe { &mut *(SIO_BASE as *mut Sio) };
sio.gpio_out_xor = 1 << 25;

// YES — raw pointer plus an explicit volatile access
let p = SIO_BASE as *mut Sio;
unsafe { (&raw mut (*p).gpio_out_xor).write_volatile(1 << 25) };
```

`&mut T` promises LLVM the memory is uniquely owned and does not change
underneath. Both halves are false for MMIO: the other core writes these
registers, and so does the hardware. The plain assignment is also a normal
store, which the optimiser may sink, hoist, merge with a neighbouring store, or
drop entirely as dead. `write_volatile` on a raw pointer is what pins it to one
access at one address in program order.
