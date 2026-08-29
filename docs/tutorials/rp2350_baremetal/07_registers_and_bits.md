---
document_type: "Tutorial Chapter — Registers, Bits, and Register Blocks"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 7 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-06
sources: RP2350 datasheet §2.1.3 (PDF p27), §2.1.5 (PDF p28), §3.1.11 (Tables 16/45, PDF p55-56, p69), §7.5 (Tables 533/534, PDF p504), §9.11.1 (Tables 648/700, PDF p603-606, p650), §9.11.3 (Tables 850/851/852, PDF p783-785)
creates: nothing
describes: firmware/pico2/src/gpio/mod.rs, firmware/pico2/src/common/reg.rs,
  firmware/pico2/src/common/reset.rs (chapter 08 §8.3-§8.5 write them),
  api/src/gpio/mod.rs, .cargo/config.toml
---

# Chapter 07 — Registers, Bits, and Register Blocks

This is the firmware's whole "drive the pin" path — the body of the
`Write<bool>` implementation in `firmware/pico2/src/gpio/gpio.rs`, quoted
verbatim (chapter 08 §8.11 explains the trait around it):

```rust
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
```

Five things happen there, none of them anything to do with GPIO: an integer
becomes a pointer, a `#[repr(C)]` struct turns a register name into an offset,
`&raw mut` produces a field address without producing a reference,
`1 << self.pin_no` builds a bit position, and `write_volatile` forces the store
to happen. This chapter is those five techniques. Chapter 08 is then pure
sequence — which register, in which order, and why.

Nothing in this chapter is typed into a file. It quotes `gpio.rs`,
`gpio/mod.rs`, `common/reg.rs` and `common/reset.rs` as worked examples of
those five techniques; chapter 08 §8.3 through §8.5 are where you actually
create those files, and creating them here would not work — `gpio/mod.rs`
opens with `pub mod gpio;`, naming a file that does not exist until §8.6.

Citation, callout and numbering conventions are the index's; so is the
convention that listings from the tree elide its `///` doc comments and quote
the code lines byte-for-byte. This chapter does not restate them.

## 7.1 `volatile`

A plain `*ptr` read is, to the compiler, a pure load from memory: it may cache
the value in a register, reorder it, duplicate it, merge two accesses into one,
or delete it entirely if the result looks unused. All valid for ordinary memory
— all catastrophic for a hardware register, where **the access itself is the
point**.

`read_volatile` / `write_volatile` tell LLVM: this access has effects you cannot
see, so emit exactly the accesses I wrote, exactly once, in the order I wrote
them.

The guarantee stops at the CPU boundary. `volatile` orders accesses in the
*instruction stream*, not through write buffers or the pipeline — that is what
`dsb` / `isb` are for, and it is why the reset handler in chapter 06 pairs them
with its `CPACR` and `VTOR` writes.

## 7.2 Read-modify-write, or plain write?

| Register shape | Correct access |
|---|---|
| several independent live fields (e.g. `CPACR`) | **read-modify-write** |
| the whole register is one value (e.g. `VTOR`) | **plain write** |
| freshly-reset register where all other fields want 0 | plain write is *safer* |
| one bit is yours and the other 31 belong to other peripherals (`RESETS.RESET`) | **read-modify-write**, always |

The fourth row is the one the firmware actually performs. `RESETS.RESET` at
`0x40020000` holds one reset bit per peripheral, all reset to `0x1`
(Table 534, datasheet §7.5, PDF p504). The firmware wants bits 6 and 9 cleared
and every other bit left exactly as it was, so `clr_reset_reg` in
`common/reset.rs` reads, masks, and writes back — verbatim, the function's
body:

```rust
    // Create pointer to reset addresses
    let reset_addr = RegAddr::RESET as usize as *mut Reset;
    unsafe{
        // read current registers
        let reset = &raw mut (*reset_addr).reset;
        let current = reset.read_volatile();
        // reset IO and PAD
        reset.write_volatile(current & mask);

    }
```

The `mask` parameter arrives already complemented — the caller writes
`clr_reset_reg(!IO_PAD_BITMASK)`, so `current & mask` clears exactly bits 6
and 9. A plain `write_volatile(0)` here would deassert every reset on the chip
at once, including bit 7 (`IO_QSPI`) and bit 10 (`PADS_QSPI`) — the pins the
image is executing from. Chapter 08 §8.5 walks this code as step 1 of the
bring-up sequence, and chapter 09 §9.2.1 gives the full bit list.

Applying RMW everywhere out of caution is its own bug: it makes
write-1-to-clear registers behave very strangely, and it can preserve stale
fields you meant to reset.

There is also no narrower escape hatch. §2.1.5 (PDF p28): *"The majority of
memory-mapped IO registers on RP2350 ignore the width of bus read/write
accesses. They treat all writes as though they were 32 bits in size."* A `strb`
to the low byte replicates the value across the bus and writes the whole
register. The only two ways to touch part of one are read-modify-write and the
atomic aliases in §7.5.

## 7.3 Rust has no bitfields

There is no equivalent of `struct { unsigned x : 3; }`, and that is a feature.
C bitfields leave allocation order, straddling, and underlying type all
implementation-defined, which is why most embedded C style guides ban them for
MMIO. Rust simply does not offer the footgun.

**Shift and mask explicitly.**

### 7.3.1 Single-bit flags

The firmware names its bit *positions*, function-locally, and shifts at the use
site. From `configure_gpio_pin_out`, verbatim:

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

Set with `|`, clear with `& !`. `IE` and `OD` are `u8`, so `1 << OD` is an
integer literal shifted by a `u8`; it compiles because `current_pad` is `u32`
and the literals infer to `u32` from the compound assignment.

The alternative is to name the masks rather than the positions, in one place:

```rust
// PROPOSED — not in the tree today
pub const PADS_ISO: u32 = 1 << 8;
pub const PADS_OD:  u32 = 1 << 7;
pub const PADS_IE:  u32 = 1 << 6;
pub const PADS_PDE: u32 = 1 << 2;
```

The firmware has no such module — `IE`, `OD`, `ISO`, `PUE` and `PDE` are
declared inside the two functions that use them, and there are no pad
constants anywhere else. The positions and reset values are Table 852
(PDF p785); chapter 09 has the table.

### 7.3.2 Multi-bit fields — clear, then set

A field wider than one bit cannot be set with `|` alone, because `|` cannot
clear. The general form is clear-then-set:

```rust
// PROPOSED — not in the tree today
pub const FUNCSEL_MASK: u32 = 0x1f;
pub const FUNCSEL_SIO:  u32 = 5;

#[inline]
pub const fn with_funcsel(reg: u32, funcsel: u32) -> u32 {
    (reg & !FUNCSEL_MASK) | (funcsel & FUNCSEL_MASK)
}
```

> **Silent-failure trap.** The clear-first step is load-bearing, and `FUNCSEL`
> is the perfect example. Its reset value is `0x1f` — *all five bits set*
> (Table 700, §9.11.1, PDF p650 — that is `GPIO25_CTRL`; every `GPIOn_CTRL`
> has the same shape):
>
> ```rust
> assert_eq!(with_funcsel(0x1f, FUNCSEL_SIO), 5);
> assert_eq!(0x1f | FUNCSEL_SIO, 0x1f);   // the naive version
> ```
>
> `reg | 5` leaves it at `0x1f`, which is `NULL` — the pin stays disconnected
> from every peripheral. No fault, no diagnostic, a dark LED and code that
> reads correctly.

For a field not at bit 0 the full form is:

```rust
(reg & !(MASK << SHIFT)) | ((val & MASK) << SHIFT)
```

The firmware sidesteps all of this for `FUNCSEL` by writing the register whole
(`io_ctrl.write_volatile(SIO)` with `const SIO: u32 = 5`), which is row three
of the table in §7.2: the pin is freshly out of reset and every other field in
`GPIOn_CTRL` wants `0` anyway. That is correct here and wrong in a general
`set_function()` that must preserve the four `OVER` fields — which is exactly
when `with_funcsel` earns its place.

## 7.4 Building addresses

Thumb-2 instructions are at most 32 bits, so there is no room for a full 32-bit
immediate. ARM materialises constants in halfword pairs. This is the head of
`OnReset` in the current release build
(`llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=OnReset`, real
output):

```asm
10000306 <OnReset>:
10000306:      	push	{r7, lr}
10000308:      	mov	r7, sp
1000030a:      	movw	r0, #0xed88
1000030e:      	movt	r0, #0xe000
10000312:      	ldr	r1, [r0]
10000314:      	orr	r1, r1, #0xf00000
10000318:      	str	r1, [r0]
```

`movw` loads the low halfword and zeroes the top; `movt` loads the high
halfword and leaves the low alone. Two instructions, eight bytes, to say
`0xe000ed88`. (Chapter 06 §6.5 prints the rest of this function and reads it
as startup code; here it is only an instruction-selection example.)

The last three instructions are the whole of §7.2's read-modify-write: `ldr`
READ, `orr` MODIFY, `str` WRITE. ARM is a **load/store architecture** —
arithmetic works only on registers, memory only via `ldr`/`str`, and there is
no instruction that ORs a value into a memory location. A read-modify-write is
structurally three instructions, and your Rust maps onto them one-to-one
because it has to. The same shape appears for `RESETS.RESET`, inlined into
`demo::main` in the finished build at `0x1000014c`, with `bic` — bit clear —
standing in for the `& mask`:

```asm
1000014c:      	ldr	r1, [r0, #-8]
10000150:      	bic	r1, r1, #0x240
10000154:      	str	r1, [r0, #-8]
```

(`r0` holds `0x40020008`, the `RESET_DONE` address the poll loop below it
needs, so LLVM reaches `RESET` at offset `-8` from it — one materialised base
serving two registers, the same trick as `VTOR` in chapter 06 §6.5.)

One optimiser habit is worth recognising in a disassembly: it merges adjacent
field updates. The clear-then-set pair of §7.3.1 touches bits 7 and 6 of the
same word, and LLVM turns the whole read-modify-write into a single bitfield
insert — from `init_output` in the finished build:

```asm
100003fa:      	ldr.w	r3, [r1, r2, lsl #2]
100003fe:      	bfi	r3, r12, #6, #2
...
10000406:      	str.w	r3, [r1, r2, lsl #2]
```

`r12` holds `1`, so `bfi r3, r12, #6, #2` writes `0b01` into bits 7:6 — `IE = 1`
and `OD = 0` in one instruction, from source that clears one bit and sets the
other separately. The two volatile calls are still exactly two bus accesses,
which is the guarantee that matters; what happens between them is not your
business.

## 7.5 Atomic register aliases

Every peripheral register block is decoded four times. §2.1.3 (PDF p27), quoted:

> Each peripheral register block is allocated 4 kB of address space, with
> registers accessed using one of 4 methods, selected by address decode.
>
> - Addr + 0x0000 : normal read write access
> - Addr + 0x1000 : atomic XOR on write
> - Addr + 0x2000 : atomic bitmask set on write
> - Addr + 0x3000 : atomic bitmask clear on write

Writing a mask to the `+0x3000` alias clears exactly those bits and leaves the
rest untouched, in one store, with no read — the clean way to release two
`RESETS` bits without the three-instruction sequence of §7.4. The four aliases
occupy 16 kB per block, and native atomic writes cost the same cycles as normal
ones.

**The firmware does not use them.** `clr_reset_reg` and `set_reset_reg` do a
plain read-modify-write at `RESETS + 0x0`. Both approaches are correct here;
the alias version is interrupt- and core-safe with no window between the read
and the write, the RMW version is what is in the tree — and the tree's own doc
comment on `clr_reset_reg` says as much, naming the single store to
`RESET_CLR` at `0x40023000` as the alternative. Do not read the alias table as
a description of the shipping code.

> **Silent-failure trap.** §2.1.3 lists the blocks that do **not** support
> atomic register access, and `SIO` is first: *"SIO (Section 3.1), though some
> individual registers (e.g. GPIO) have set, clear, and XOR aliases"*. SIO's
> `_SET`/`_CLR`/`_XOR` are real registers at real offsets (§7.6.2), not address
> aliases — the chapter-opening `write` targets two of them. Table 16 ends at
> offset `0x1e4` (`TMDS_POP_DOUBLE_L2`, PDF p61), so `SIO_BASE + 0x3000` is
> past every SIO register there is. **Inferred:** the store goes nowhere useful
> and the bits you meant to clear stay set, with no fault. The exclusion list
> also covers the CoreSight window, the Cortex-M33 PPB, and the OTP SBPI
> bridge.

## 7.6 Modelling a register block as a `#[repr(C)]` struct

A `#[repr(C)]` struct over a peripheral base is the cheapest way to get named
registers with no runtime cost: field order is declaration order, and the
offsets fall out of the layout algorithm. The catch is that the layout
algorithm is doing arithmetic on your behalf, and it does not know what the
datasheet says. Three ways that goes wrong — §7.6.1, §7.6.2 and §7.6.3 — all of
them silent.

The firmware declares five such structs: `GpioRegs`, `IoBank`, `PadsBank` and
`Sio` in `firmware/pico2/src/gpio/mod.rs`, and `Reset` in
`firmware/pico2/src/common/reset.rs`, next to the three functions that use it.
All five are private to their modules with `pub` fields. The two simple ones,
code lines verbatim (each field carries a long doc comment in the tree):

```rust
#[repr(C)]
struct GpioRegs{
    pub status: u32,
    pub ctrl: u32,
}

#[repr(C)]
struct Reset{
    pub reset: u32,
    pub wdsel: u32,
    pub reset_done: u32
}
```

`Reset` matches Table 533 (datasheet §7.5, PDF p504) exactly — `RESET` at
`0x0`, `WDSEL` at `0x4`, `RESET_DONE` at `0x8` — and `GpioRegs` is the 8-byte
per-pin pair that `IoBank` is an array of. The base addresses come from an enum
in `firmware/pico2/src/common/reg.rs`:

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

The two are joined with a double cast, `RegAddr::RESET as usize as *mut Reset`:
the first cast turns the variant into its discriminant (`#[repr(usize)]` is
what makes that well-defined), the second turns the integer into a pointer.
The SCREAMING_SNAKE variant names would trip the `non_camel_case_types` lint —
`IO_BANK0` and `PADS_BANK0`; `RESET` and `SIO` are single words and would
not — which is why the enum carries `#[allow(non_camel_case_types)]` and the
comment above it saying what the names buy.

### 7.6.1 Leading per-bank registers

`PADS_BANK0` does not open with a pad. Offset `0x00` is `VOLTAGE_SELECT`, a
per-bank input-threshold control (Table 850, §9.11.3, PDF p783). The pin array
starts one word later, and the struct says so (code lines verbatim; the tree
documents each field's offset in its doc comment):

```rust
#[repr(C)]
struct PadsBank{
    pub voltage_select: u32,
    pub pads: [u32; 48],
    pub swclk: u32,
    pub swd: u32,
}
```

`voltage_select` is offset `0x00`, `pads[n]` is `0x04 + 4n`, `swclk` is `0xc4`
and `swd` is `0xc8`. `IO_BANK0` does **not** do this — it opens directly with
`GPIO0_STATUS` at offset `0x000` (Table 648, §9.11.1, PDF p603). The two blocks
are not parallel in shape, which is why the mistake is easy: index a pads array
from offset `0` and every pin is off by one, with `pads[0]` landing on
`VOLTAGE_SELECT`, whose bit 0 sets the input threshold for the whole bank
(Table 851, PDF p785). Transcribe offset `0x00` from the register list rather
than assuming the block starts with the thing you came for.

`swclk` and `swd` are the debug-port pads. Naming them is correct — they are
part of the block — but think twice before letting a general pin allocator
index them, because a bug that reconfigures them disconnects your debugger.

### 7.6.2 Reserved gaps

Two of the four GPIO-path blocks have holes, announced by nothing except a jump
in the offset column of the register list.

`IO_BANK0` has 128 bytes of nothing between `GPIO47_CTRL` (ending at `0x180`)
and `IRQSUMMARY_PROC0_SECURE0` at `0x200` (Table 648, PDF p605):

```rust
#[repr(C)]
struct IoBank{
    pub gpio: [GpioRegs; 48],
    _reserved: [u32; 32],
    pub irqsummary: [u32; 12],
    pub intr: [u32; 6]
}
```

`48 * 8 = 0x180` ends the pin array and `(0x200 - 0x180) / 4 = 32` reserved
words follow. There are exactly 12 `IRQSUMMARY` registers (`0x200`-`0x22c`) and
6 `INTR` (`0x230`-`0x244`), so the struct ends at `0x248` = `PROC0_INTE0`, the
first register it deliberately does not model.

`SIO` has a single reserved word at `0x00c`. Table 16 (§3.1.11, PDF p55) runs
`0x008 GPIO_HI_IN` straight to `0x010 GPIO_OUT` with nothing between. Here the
tree's own trailing offset comments are kept, because they are the point:

```rust
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

The field names transpose the datasheet's: `0x008` is `GPIO_HI_IN` in Table 16
and `gpio_in_hi` in the code, and the same swap applies to every `_hi` field.
The datasheet name is authoritative when you cross-reference; the code name is
what compiles. Note also `CPUID` at `0x000` — a real register, the index of the
core executing the load, not padding. An RTOS wants it from its first line.
The struct stops at `0x050` = `FIFO_ST`, so ending on a named register makes it
obvious this is a deliberate partial view.

> **Silent-failure trap.** A `_reserved: [u32; N]` field is not
> padding-for-neatness, it is load-bearing address arithmetic. Get `N` wrong
> and every field after it addresses a different register — the compiler is
> perfectly happy, the pointer arithmetic is perfectly valid, and the wrong
> register is written with no fault.

The way to check is to compile the struct for the **host** and print each field
offset against a null base (§7.8 explains the idiom). Real output, abridged:

```
IoBank     irqsummary     0x200   Table 648
IoBank     intr           0x230   Table 648
IoBank     size_of        0x248   = PROC0_INTE0
PadsBank   pads           0x004   Table 850
PadsBank   swd            0x0c8   Table 850
Reset      reset_done     0x008   Table 533
Sio        gpio_out_xor   0x028   Table 16
Sio        size_of        0x050   = FIFO_ST
pads[25]      = 0x068
gpio[25].ctrl = 0x0cc
```

The last two lines are the addresses chapter 08 uses:
`0x40038000 + 0x068 = 0x40038068` for the GP25 pad, `0x40028000 + 0x0cc =
0x400280cc` for the GP25 mux. Layout under `#[repr(C)]` is a property of the
ABI's struct rules, not of the target, so a host check proves the firmware's
offsets — see §7.7 for where this belongs.

### 7.6.3 Alignment padding from wide fields

The most dangerous of the three, because it inserts bytes you never wrote.

`SIO` pairs a low and a high register for each operation, and it is tempting to
model each pair as one `u64`. Under `#[repr(C)]`, `u64` carries 8-byte
alignment, so the compiler inserts a 4-byte hole wherever a `u64` follows an
odd number of `u32`s:

```rust
// PROPOSED — not in the tree today
// (and wrong on purpose)
#[repr(C)]
struct SioWrong{
    _reserved: u32,          // 0x00
    pub gpio_in: u64,        // intended 0x04
    pub gpio_out: u64,       // intended 0x10
    pub gpio_out_set: u64,   // intended 0x18
}
```

Real offsets, from the same host program:

```
--- the u64 version ---
SioWrong   gpio_in        0x008   want 0x004
SioWrong   gpio_out       0x010   want 0x010
SioWrong   gpio_out_set   0x018   want 0x018
```

> **Silent-failure trap.** `gpio_in` lands at `0x008`. It reads `GPIO_HI_IN` in
> its low half and the reserved word at `0x00c` in its high half — never
> `GPIO_IN`, which is now buried inside compiler-inserted padding and
> unreachable through the struct. And every *other* field is at the right
> address: the 4 bytes of alignment padding and the 4-byte hole at `0x00c`
> cancel exactly. The struct half-works. It passes a blink test and fails the
> first time you read a pin — and any tidy-up (packing it, reordering it,
> giving `cpuid` its real name) moves everything.

Two independent reasons not to reach for `u64` here at all. The datasheet says
these are 32-bit registers, and the high register is not a continuation of the
low one — its upper bits are the QSPI pads and USB DP/DM (Table 16, §3.1.11,
PDF p55; chapter 09 has the bit map). And Rust guarantees a
`read_volatile::<u64>()` is not split or elided, but not that it becomes one
bus transaction. **Inferred:** on a 32-bit AHB it is two 32-bit transfers
regardless, in an order the source does not state.

Transcribe the register list literally instead, one `u32` per register, as the
firmware does. Every field is then `u32`, the struct's alignment is 4, and no
padding exists anywhere — which is why all five structs are padding-free by
construction.

### 7.6.4 Two attributes to leave off

**`#[derive(Clone, Copy)]`.** It makes `let snapshot = *sio_addr;` compile, and
that expression is a non-volatile bulk read of every register in the block.
LLVM may reorder, coalesce or delete those loads, and some SIO registers have
read side effects: `INTERP0_POP_LANE0` at offset `0x094` is *"Read LANE0
result, and simultaneously write lane results to both accumulators (POP)"*
(Table 45, PDF p69). An operation you never want should not be expressible.
None of the five register structs derives `Clone` or `Copy` (the `RegAddr`
*enum* does, and that is fine — copying an address is not reading a register).

**`unsafe impl Sync`.** `u32` and arrays of `u32` are already `Sync`, so these
structs get it for free, and none of them declares it. Writing it by hand
spends the reader's attention and dilutes the one place in this firmware where
it is load-bearing: the `Vector` union in chapter 05, which holds raw pointers
and does not get `Sync` for free.

### 7.6.5 Never form a reference

The rule that makes the whole approach sound:

```rust
// PROPOSED — not in the tree today
// NO — a real reference
let sio: &mut Sio = unsafe { &mut *(SIO_BASE as *mut Sio) };
sio.gpio_out_set = 1 << 25;

// YES — raw pointer plus an explicit volatile access
let p = SIO_BASE as *mut Sio;
unsafe { (&raw mut (*p).gpio_out_set).write_volatile(1 << 25) };
```

The second form is what the `write` implementation at the top of this chapter
does. `&mut T` promises LLVM the memory is uniquely owned and does not change
underneath. Both halves are false for MMIO: the other core writes these
registers, and so does the hardware. The plain assignment is also a normal
store, which the optimiser may sink, hoist, merge with a neighbour, or drop as
dead. `write_volatile` on a raw pointer pins it to one access, at one address,
in program order.

Every access in `gpio.rs` and `reset.rs` follows this shape, and no `&` or
`&mut` is ever formed over a peripheral.

## 7.7 The host-testable seam

Register *addresses*, *masks*, *shifts* and *offset arithmetic* are pure
computation. Only the `_volatile` calls touch hardware. That is a seam: the
arithmetic can be tested on your laptop, and only the pokes need a board.

The workspace draws that seam as a crate boundary, and — unlike earlier
revisions of this project — the seam is now wired in. The `api` crate declares
the portable vocabulary:

- `api::common` — `ErrorType` (one associated error type per peripheral),
  `Write<T>` and `Read<T>` (value in, value out), and `Block` (release a
  peripheral's reset bits / put them back);
- `api::common::board` — `Board`, the one concrete type: a take-once holder
  that starts every `Block` it is given (chapter 08 §8.12);
- `api::gpio` — `Pull` (which pull resistor an input gets), the `Gpio` factory
  trait (validate a pin number, hand back a configured pin), and the marker
  that ties it together, verbatim from `api/src/gpio/mod.rs`:

```rust
pub trait GpioPin: Write<bool> + Read<bool> {}
```

The firmware consumes it. `firmware/pico2/src/gpio/gpio.rs` opens with

```rust
use api::common::{Block, ErrorType, Read, Write};
use api::gpio::{Gpio, Pull};
```

and everything chapter 08 builds — the port type, the pin type, the
configuration functions — is an implementation of those traits. `api` itself
contains no register address and no `unsafe` *code* (its two `Block` methods
are declared `unsafe fn`, an obligation passed to implementors, not an unsafe
body), which is what lets it compile for the host: the `xtest` alias in
`.cargo/config.toml` overrides the global `[build] target` and builds `api`
for `aarch64-apple-darwin`. Chapter 01 §1.5.1 sets that up and shows it
running.

What the seam does not yet hold is tests. `cargo xtest` currently runs **zero
unit tests** — `api` has no `#[test]` anywhere — and the register arithmetic
that would benefit most is still function-local inside `gpio.rs`, on the wrong
side of the boundary. A `#![no_std]` library is still host-testable: `no_std`
only suppresses the implicit `extern crate std`; built for the host, the
library compiles unchanged and the *test harness* links `std`. This is the
slot the arithmetic would drop into:

```rust
// PROPOSED — not in the tree today
#[inline] pub const fn io_bank0_ctrl_offset(pin: u32) -> u32 { 0x004 + 8 * pin }
#[inline] pub const fn pads_bank0_offset(pin: u32)    -> u32 { 0x004 + 4 * pin }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn funcsel_replaces_not_ors() {
        assert_eq!(with_funcsel(0x1f, FUNCSEL_SIO), 5);
    }
    #[test] fn funcsel_preserves_upper_bits() {
        assert_eq!(with_funcsel(0xdead_be00 | 0x1f, FUNCSEL_SIO), 0xdead_be00 | 5);
    }
    #[test] fn gp25_offsets() {
        assert_eq!(io_bank0_ctrl_offset(25), 0xcc);   // 8-byte stride
        assert_eq!(pads_bank0_offset(25),    0x68);   // 4-byte stride
    }
}
```

Two different strides for the same pin number, both `0x004 + k*n`, differing
only in `k` — exactly the kind of thing worth a test, and exactly the kind that
produces a working blink and a broken everything-else when it is wrong. The
`&raw`-offset check of §7.6.2 belongs in the same module for the same reason.

## 7.8 Appendix — the address-of-a-field idiom

`&raw mut (*p).field` and `&raw const (*p).field` compute a field address
without ever creating a reference. They are what makes §7.6.5's rule followable:
before `&raw` was stabilised you wrote `addr_of_mut!`, and before *that*, people
wrote `&mut (*p).field as *mut _` and had already committed the UB by the time
the cast ran.

The host offset check in §7.6.2 leans on this hard — it takes field addresses
on a **null** base:

```rust
// PROPOSED — not in the tree today
let p: *const IoBank = core::ptr::null();
println!("{:#05x}", unsafe { &raw const (*p).irqsummary } as usize);  // want 0x200
```

`&raw const (*p).irqsummary` is a pure offset computation — it never loads
through `p`, so a null base yields the offset directly. Writing
`&(*p).irqsummary` instead is instant undefined behaviour: forming a reference
requires the referent to be dereferenceable and non-null, and the compiler may
assume both from the moment the reference exists, whether or not you read
through it. One prints `0x200`; the other has no defined meaning and will
probably also print `0x200`, right up until the day it does not.

The idiom composes through arrays and nested structs:
`&raw mut (*io_addr).gpio[pin].ctrl`, from `configure_gpio_pin_out`, walks a
struct field, an array index and a second struct field, and produces one
address with no reference formed anywhere along the way.

---

That is the whole vocabulary: `volatile`, read-modify-write versus plain write,
shift-and-mask instead of bitfields, `#[repr(C)]` over a base address, `&raw`
instead of `&`. Every one of them appears in this chapter attached to a
fragment, out of order and out of context. **Chapter 08** puts them back in
order — the writes to four peripherals, in the sequence the hardware requires,
ending with a lit LED.
