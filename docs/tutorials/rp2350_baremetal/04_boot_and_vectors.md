---
document_type: Tutorial Chapter — Boot Metadata and the Vector Table
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 4 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-03
sources: RP2350 datasheet 5.9, 5.9.1, 5.9.3.3, 5.9.5.1, 3.2, 3.7
---

# Chapter 04 — Boot Metadata and the Vector Table

Two `static`s that the *hardware* reads before any of your code runs.

## 4.1 What the bootrom needs

The RP2350 bootrom scans the first 4 kB of flash for a **metadata block loop**
containing a valid `IMAGE_DEF`, and refuses to boot without one — this is how it
distinguishes a program from blank flash or a disconnected device.

One thing it explicitly does **not** need (5.9.5):

> Unlike RP2040, there is no requirement for flash binaries to have a
> checksummed "boot2" flash setup function at flash address 0. The RP2350
> bootrom performs a simple best-effort XIP setup during flash scanning, and a
> flash-resident program can continue executing in this state.

So there is no second-stage bootloader to write. You are allowed to already be
executing from flash when your reset handler starts.

## 4.2 Block structure (5.9.1)

Every metadata block is:

```
header -> items -> link -> footer
```

All little-endian, word-aligned, total size a multiple of 4.

The **link** is the interesting field: a byte offset to the next block's header,
and the list must eventually loop back to the first block or *the whole chain is
ignored*. That is deliberate — it prevents a partially-overwritten image from
leaving orphaned blocks that still look valid. For a single block the link is
`0` (points at itself).

## 4.3 The minimum Arm IMAGE_DEF (5.9.5.1)

Five words, 20 bytes. Valid as long as `CRIT1.SECURE_BOOT_ENABLE` is clear,
which it is on a stock Pico 2.

| Word | LE value | Meaning |
|---|---|---|
| 0 | `0xffffded3` | `PICOBIN_BLOCK_MARKER_START` |
| 1 | `0x10210142` | IMAGE_TYPE item |
| 2 | `0x000001ff` | LAST item |
| 3 | `0x00000000` | link — 0 means loop to self |
| 4 | `0xab123579` | `PICOBIN_BLOCK_MARKER_END` |

Words 1 and 2 look like magic numbers but are structured. These are
**little-endian words**, so read the bytes right to left:

**Word 1 = `0x10210142`** -> bytes `42 01 21 10`

- `0x42` — item type: `size_flag=0` (1-byte size field) + `item_type = IMAGE_TYPE`
- `0x01` — this item is 1 word long
- `0x1021` — the payload

**Word 2 = `0x000001ff`** -> bytes `ff 01 00 00`

- `0xff` — `size_flag=1` + `item_type = LAST`
- `0x0001` — total word count of the *other* items
- `0x00` — pad

### 4.3.1 Decoding `0x1021`

| Bits | Value | Meaning |
|---|---|---|
| 3:0 | 1 | `IMAGE_TYPE_EXE` |
| 5:4 | 2 | `EXE_SECURITY_S` — runs in Secure mode |
| 10:8 | 0 | `EXE_CPU_ARM` |
| 14:12 | 1 | `EXE_CHIP_RP2350` |

> **This field is what selects Arm over RISC-V.** The RISC-V variant is
> `0x1101` in the same slot. Pick wrong and the bootrom hands your image to the
> Hazard3 cores.

## 4.4 Writing it in Rust

```rust
#[used]
#[unsafe(link_section = ".boot_info")]
static BOOT_INFO: [u32; 5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579,
];
```

Three attributes, all load-bearing:

| | Purpose |
|---|---|
| `#[unsafe(link_section = ".boot_info")]` | names the section; must match the linker script exactly |
| `#[used]` | stops **rustc/LLVM** dropping an unreferenced static |
| `KEEP()` in `link.ld` | stops **lld `--gc-sections`** dropping the section |

Declare it `[u32; 5]`, **not** a byte array — writing `u32` literals on a
little-endian target produces exactly the byte sequence the datasheet's "LE
Value" column specifies. A byte array means doing the endian swap by hand.

`link_section` is one of the attributes edition 2024 made `unsafe`, alongside
`no_mangle` and `export_name` — because a bad section name produces a broken
binary with no diagnostic.

## 4.5 Verifying the boot block

```
llvm-objdump -s -j .boot_info <elf>
```

```
 10000000 d3deffff 42012110 ff010000 00000000  ....B.!.........
 10000010 793512ab                             y5..
```

If the bytes come out reversed, a byte array was used instead of `u32`s.

Independent confirmation from Raspberry Pi's own tool (note picotool dispatches
on file extension, so copy the Cargo output to a `.elf` name first):

```
$ picotool info -a pico2.elf
 target chip:         RP2350
 image type:          ARM Secure

Metadata Block 1
 address:             0x10000110
 next block address:  0x10000110      <- self-loop
 block type:          image def
```

## 4.6 The vector table

### 4.6.1 Layout

68 entries = 16 ARM system slots + 52 RP2350 external interrupts
(3.2: *"an internal interrupt controller, with 52 interrupt inputs"*).

| Index | Exception |
|---|---|
| 0 | **Initial stack pointer** (not a handler) |
| 1 | Reset |
| 2 | NMI |
| 3 | HardFault |
| 4 | MemManage |
| 5 | BusFault |
| 6 | UsageFault |
| 7 | SecureFault |
| 8-10 | Reserved — must be zero |
| 11 | SVCall |
| 12 | DebugMonitor |
| 13 | Reserved — must be zero |
| 14 | PendSV |
| 15 | SysTick |
| 16-67 | IRQ0 - IRQ51 |

68 x 4 = 272 bytes = `0x110`.

> Only the lower **46** IRQs are wired to peripherals. IRQ46-51 are
> `SPAREIRQ_IRQ_0..5`, *"hardwired to zero (never firing)"*, reserved for a core
> to interrupt itself. The table is still 68 entries.

### 4.6.2 The Rust type problem

The table mixes three types: a **stack pointer** at index 0, **function
pointers** at 1-67, and **zeros** in the reserved slots. Rust arrays are
homogeneous, so this needs a union:

```rust
#[repr(C)]
#[derive(Clone, Copy)]
union Vector {
    handler:   unsafe extern "C" fn(),
    reset:     unsafe extern "C" fn() -> !,
    stack_top: *const u32,
    reserved:  u32,
}
unsafe impl Sync for Vector {}
```

Four details:

- **`#[repr(C)]` is required.** `repr(Rust)` union layout is *explicitly
  unspecified*. It happens to give 4/4 today — but this is a structure the
  silicon reads. `repr(C)` makes it a guarantee rather than an observation.
- **`reset` needs its own field.** A diverging `fn() -> !` will not coerce to
  `fn()` in union-field position:
  `error[E0308]: expected fn pointer 'unsafe extern "C" fn() -> ()'`.
- **`unsafe impl Sync`** is required and sound — the `*const u32` field is what
  makes the type non-`Sync`; it is a 4-byte POD with no interior mutability.
- **`&raw const` on an extern static const-evaluates**, so the stack pointer can
  come straight from the linker symbol:
  `Vector { stack_top: &raw const _stack_top }`. No hardcoded `0x20082000`, and
  no need for `LONG(_stack_start)` in the script (which is how `cortex-m-rt`
  sidesteps the same problem).

### 4.6.3 Building 68 entries

A `const` block with mutation works in a static initialiser — fill defaults
first, then override:

```rust
#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0]  = Vector { stack_top: &raw const _stack_top };
    t[1]  = Vector { reset: OnReset };
    t[3]  = Vector { handler: OnHardFault };
    t[8]  = Vector { reserved: 0 };
    t[9]  = Vector { reserved: 0 };
    t[10] = Vector { reserved: 0 };
    t[13] = Vector { reserved: 0 };
    t
};
```

### 4.6.4 The Thumb bit

```
 10000000 00200820 31010010 25010010 2b010010
```

- Word 0 -> `0x20082000` = `_stack_top`
- Word 1 -> `0x1000_0131`, while `nm` reports `OnReset` at `0x1000_0130`

**Bit 0 is set.** On ARM, bit 0 of a branch target selects the instruction set
and must be 1 for Thumb. From 3.7:

> All populated vectors in the vector table entries must have bit[0] set.
> Creating a table entry with bit[0] clear generates an INVSTATE fault on the
> first instruction of the handler.

Rust function pointers carry this automatically — which is the payoff for using
typed `fn` pointers rather than `u32` literals. **If you ever hand-write an
address as a `u32`, you own the `+1`.**

## 4.7 Handlers must not return

Exception return on ARM is not a normal branch to a code address: on entry `LR`
holds a magic `EXC_RETURN` value (`0xFFFFFFxx`), and branching to it triggers
the unstacking sequence.

> **Silent-failure trap.** An **empty** `extern "C" fn` compiles to `bx lr`,
> which is a *perfectly valid exception return*. It does not crash — it resumes
> at the faulting instruction, which faults again, forever, leaving no trace.

The correct bodies:

| Handler | Behaviour |
|---|---|
| a real peripheral ISR you wrote | clear the source, do the work, **return** |
| `DefaultHandler` (catch-all) | **spin** — it cannot clear what it cannot identify |
| fault handlers | **spin** — returning re-executes the faulting instruction |

Returning does *not* clear an interrupt; you clear it at the peripheral. A
default handler that returns produces an infinite interrupt storm in which the
foreground never advances but the chip looks busy — strictly harder to diagnose
than a clean freeze.

### 4.7.1 Identifying the trap

`IPSR` bits 8:0 hold the exception number currently executing, and that number
*is* the vector table index:

```rust
let ipsr: u32;
unsafe { core::arch::asm!("mrs {}, ipsr", out(reg) ipsr, options(nomem, nostack)); }
```

> **Release-build trap.** Two handlers with identical `loop{}` bodies are
> **folded to one address** by LLVM, so NMI and HardFault vector to the same
> instruction and become indistinguishable. Give them different bodies if you
> want to tell them apart.
