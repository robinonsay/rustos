---
document_type: "Tutorial Chapter — Boot Metadata and the Vector Table"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 5 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-04
sources: RP2350 datasheet §5.9.1 (PDF p417-418), §5.9.3.1 (PDF p420-421), §5.9.3.3 (PDF p423), §5.9.5 / §5.9.5.1 (PDF p427), §5.9.5.2 (PDF p428), §3.2 Table 94 (PDF p83-84), §3.7.4.5 (PDF p135-136), §3.7.4.6 / §3.7.4.7 (PDF p137), Table 200 ICSR (PDF p182), Table 206 SHPR2 / Table 207 SHPR3 (PDF p185-186), Table 249 SFSR (PDF p200-201)
creates: firmware/pico2/src/lib.rs — through §5.9's listing, with one placeholder
---

# Chapter 05 — Boot Metadata and the Vector Table

Two `static`s in `firmware/pico2/src/lib.rs` that the *hardware* reads before
any of your code runs. Neither is called from anywhere; both are load-bearing.
By the end of this chapter you will have pulled them out of the built image byte
by byte and confirmed the bytes are the ones the datasheet asks for.

One numbering collision to hold, because this chapter cannot avoid it: the
bootrom chapter of the datasheet is also chapter 5. Every `§5.9.x` below is a
**datasheet** section — the metadata-block specification — while this chapter's
own sections stop at `§5.9`, "The file so far". Nothing else in the tutorial is
numbered `5.9.1` or deeper, so a third number always means the datasheet.
Chapter 09 has the same problem with the datasheet's GPIO chapter and solves it
the other way, by writing `datasheet §9.4` in full.

## 5.1 What the bootrom needs

Chapter 04 §4.11.2 has the sections of the **finished** image — still the
target, not something you can build yet; §5.9 gives this chapter's own build
and its numbers, which are smaller:

```
Sections:
Idx Name            Size     VMA      Type
  1 .vector_table   00000110 10000000 DATA
  2 .boot_info      00000014 10000110 DATA
  3 .text           0000183c 10000124 TEXT
```

`.vector_table` and `.boot_info` are this chapter; `.text` is chapters 06 and
08. Two data blobs sit in front of every instruction in the image, and the
bootrom demands them in two separate ways.

**First, a metadata block.** §5.9.5 (PDF p427): "A minimum amount of metadata
(i.e. a valid IMAGE_DEF block) must be embedded in any binary for the bootrom to
recognise it as a valid program image, as opposed to, for example, blank flash
contents or a disconnected flash device. This must appear within the first 4 kB
of a flash image". `.boot_info` is at `0x10000110`, `0x110` bytes into flash —
inside the window with room to spare. Chapter 04 shows the `ASSERT` in `link.ld`
that enforces it.

**Second, a vector table at the base of the image.** The minimum block you are
about to write carries no `VECTOR_TABLE` or `ENTRY_POINT` item, and §5.9.5.1
(PDF p427) says what happens then:

> Since the above block does not specify an explicit entry point, the bootrom
> will assume the binary starts with a Cortex-M vector table, and enter via the
> reset handler and initial stack pointer specified in that table (offsets +4
> and +0 bytes into the table).

§5.9.3.3 (PDF p423) states the same default from the other direction: "if there
is no ENTRY_POINT or VECTOR_TABLE, Item, then a VECTOR_TABLE at the start of the
image is assumed". So `.vector_table` sits at `ORIGIN(FLASH)`, and the bootrom
reads word 0 into SP and word 1 into PC.

One thing the bootrom explicitly does **not** need (§5.9.5, PDF p427): "Unlike
RP2040, there is no requirement for flash binaries to have a checksummed
"boot2" flash setup function at flash address 0. The RP2350 bootrom performs a
simple best-effort XIP setup during flash scanning, and a flash-resident program
can continue executing in this state." There is no second-stage bootloader to
write, and your reset handler is already executing from flash when it starts.
(XIP — execute-in-place — is the hardware that maps the external flash chip
into the address space so the CPU can fetch instructions from it directly;
chapter 03 §3.2 is the full story.)

## 5.2 Block structure (§5.9.1)

Every metadata block has the same shape (§5.9.1, PDF p417):

```text
header -> items -> link -> footer
```

All multi-byte values inside a block are little-endian, blocks start
word-aligned, and the total size is an exact number of words. The header is
always `0xffffded3` and the footer always `0xab123579`. Each item begins with a
byte packing `size_flag:1` and `item_type:7`, followed by its size in words; the
final item must be `PICOBIN_BLOCK_ITEM_LAST`, which "encodes the total word
count of the block's items" (§5.9.1, PDF p417-418). §5.3 shows all of that
concretely for the one block this firmware carries.

The **link** is the field that catches people. §5.9.1 (PDF p417):

> To be valid, this linked list must eventually link back to the first block in
> the list, forming a closed block loop; failure to close the loop results in
> the entire linked list being ignored. The loop rule is used to avoid treating
> orphaned blocks from partially overwritten images being treated as valid.

For a single block the link is `0` — "link to self", a loop of one. An
`IMAGE_DEF` is also capped at 384 bytes, anything larger being ignored (§5.9.1,
PDF p417). You are about to write 20.

## 5.3 The minimum Arm IMAGE_DEF

§5.9.5.1 (PDF p427) gives the whole thing as a 20-byte sequence, valid as long
as `CRIT1.SECURE_BOOT_ENABLE` is clear — which it is on a stock Pico 2.

| Word | LE Value | Description |
|---|---|---|
| 0 | `0xffffded3` | `PICOBIN_BLOCK_MARKER_START` |
| 1 | `0x10210142` | `IMAGE_TYPE` item, 1 word, payload `0x1021` |
| 2 | `0x000001ff` | `LAST` item, other items' size `0x0001` |
| 3 | `0x00000000` | link — `0x00000000` means link to self |
| 4 | `0xab123579` | `PICOBIN_BLOCK_MARKER_END` |

Words 1 and 2 look like magic numbers but are structured, and because these are
**little-endian words** you read their bytes right to left.

| Word | Bytes | Meaning |
|---|---|---|
| `0x10210142` | `42` | `size_flag == 0` (1-byte size field), `item_type == PICOBIN_BLOCK_ITEM_1BS_IMAGE_TYPE` |
| | `01` | this item is 1 word in size |
| | `21 10` | `image_type_flags` = `0x1021`, decoded next |
| `0x000001ff` | `ff` | `size_type == 1`, `item_type == PICOBIN_BLOCK_ITEM_2BS_LAST` |
| | `01 00` | size: the other items total one word |
| | `00` | pad |

### 5.3.1 Decoding `0x1021`

`image_type_flags` is a bitfield (§5.9.3.1, PDF p420-421). The datasheet's own
table, with the value this image selects in each field:

| Bits | Field | Values | In `0x1021` |
|---|---|---|---|
| 0-3 | Image Type | 0 `IMAGE_TYPE_INVALID`, 1 `IMAGE_TYPE_EXE`, 2 `IMAGE_TYPE_DATA` | 1 — executable |
| 4-5 | EXE Security | 0 `UNSPECIFIED`, 1 `EXE_SECURITY_NS`, 2 `EXE_SECURITY_S` | 2 — Secure mode |
| 6-7 | reserved | 0 | 0 |
| 8-10 | EXE CPU | 0 `EXE_CPU_ARM`, 1 `EXE_CPU_RISCV` | 0 — Arm |
| 11 | reserved | 0 | 0 |
| 12-14 | EXE CHIP | 0 `EXE_CHIP_RP2040`, 1 `EXE_CHIP_RP2350` | 1 — RP2350 |
| 15 | EXE TBYB | 0 not set, 1 `EXE_TBYB` | 0 |

Bits 10:8 pick the architecture. The minimum **RISC-V** `IMAGE_DEF` (§5.9.5.2,
PDF p428) is the same 20 bytes with word 1 as `0x11010142`: `0x1021` becomes
`0x1101`, EXE CPU going from 0 to 1 and EXE Security dropping to unspecified.

> **Silent-failure trap.** Get bits 10:8 wrong and the bootrom hands your Arm
> image to the Hazard3 RISC-V cores. No diagnostic, no fault, no LED: the block
> is *valid*, so the bootrom boots it, at a core that cannot decode a single one
> of your instructions.

## 5.4 Writing it in Rust

### 5.4.1 Where it goes

This is the chapter where `firmware/pico2/src/lib.rs` starts becoming the real
runtime. Everything you type in this chapter and the next goes into that one
file; chapter 08 then adds the driver modules beside it. The file already has
its head from chapter 01 §1.9:

```rust
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

One convention before the first listing, because it holds for the rest of the
tutorial: the tree's `lib.rs` carries long `//!` and `///` documentation
comments on nearly every item. The listings here **elide those comments and
quote the code lines byte-for-byte**; where a listing keeps a comment, it is
the tree's. So your file and the tree's will differ in comments and in nothing
else. `demo/src/main.rs` stays as chapter 01 left it — two attributes and
`use pico2 as _;` — until chapter 06 gives it a job.

### 5.4.2 The boot block

Five words, verbatim from `firmware/pico2/src/lib.rs`:

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

Nothing reads `BOOT_INFO`, so three mechanisms must agree that it reaches the
image:

| Mechanism | Stops |
|---|---|
| `#[unsafe(link_section = ".boot_info")]` | nothing — it *names* the section, and the name must match `link.ld` character for character |
| `#[used]` | **rustc/LLVM** dropping an unreferenced static before the linker ever sees it |
| `KEEP()` in `link.ld` | **lld `--gc-sections`** dropping the input section |

Chapter 02 §2.5 covers the `#[used]` / `KEEP()` split; the short version is that
they defend against two different tools at two different times, and you need
both.

Declare it `[u32; 5]`, **not** `[u8; 20]`: writing `u32` literals on a
little-endian target produces exactly the byte sequence the datasheet's "LE
Value" column specifies, whereas a byte array means doing the endian swap by
hand, and a swapped header is not a header. `link_section` is one of the
attributes edition 2024 made `unsafe`, alongside `no_mangle` and `export_name`,
because a mistyped section name produces a broken binary and no diagnostic.

## 5.5 Verifying the boot block

Add `BOOT_INFO` above the panic handler, build, and dump the section out of the
ELF with
`llvm-objdump -s -j .boot_info target/thumbv8m.main-none-eabihf/release/demo`.
Staged — this is your build as it stands right now:

```
Contents of section .boot_info:
 10000000 d3deffff 42012110 ff010000 00000000  ....B.!.........
 10000010 793512ab                             y5..
```

Read the first group as bytes: `d3 de ff ff` is `0xffffded3` little-endian.
Every word matches §5.3. If your bytes come out reversed, you used a byte array.
The linker still reports `cannot find entry symbol OnReset` (chapter 04
§4.11.1) and the memory report says `FLASH: 20 B` — the block is the only
content in the image.

The address is `0x10000000` rather than the finished image's `0x10000110`,
because `.vector_table` is still empty until §5.6 puts 272 bytes in front of
it. The bytes are the part under test here, and they do not move.

Then get independent confirmation from Raspberry Pi's own tool. `picotool`
dispatches on the file extension, so copy the Cargo output to a `.elf` name
first and point it at the **file** — no board required, and none touched:

```
$ picotool info -a demo.elf
File demo.elf:

Program Information
 target chip:         RP2350
 image type:          ARM Secure

Metadata Block 1
 address:             0x10000000
 next block address:  0x10000000
 block type:          image def
 extra security:      not enabled
```

(Elided from the real run: the empty `Fixed Pin Information` and
`Build Information` sections, and a repeat of the `target chip` / `image type`
pair inside the metadata block.)
`next block address` equals `address`: the self-loop of §5.2 closed. `ARM
Secure` is bits 10:8 and bits 5:4 of `0x1021` read back by someone else's
decoder, which is the point of running it. `picotool` reads the block, not the
vector table, so the twenty bytes are already enough for it to call the file an
RP2350 ARM Secure image. On the finished image the same command reports the
same four lines with `address` and `next block address` both `0x10000110`.

## 5.6 The vector table

### 5.6.1 Layout

The table is 68 entries: 16 Arm system slots plus one per external interrupt.
§3.2 (PDF p83) says how many of those there are — "Each core is equipped with an
internal interrupt controller, with 52 interrupt inputs", numbered IRQ0 to IRQ51
in Table 94 (PDF p83-84). 16 + 52 = 68, and 68 × 4 = 272 bytes = `0x110`,
exactly the `.vector_table` size in §5.1.

| Index | Exception | Firmware puts here |
|---|---|---|
| 0 | Initial stack pointer (not a handler) | `&raw const _stack_top` |
| 1 | Reset | `OnReset` |
| 2 | NMI | `DefaultHandler` |
| 3 | HardFault | `OnHardFault` |
| 4-7 | MemManage, BusFault, UsageFault, SecureFault | `DefaultHandler` |
| 8-10 | Reserved | `0` |
| 11-12 | SVCall, DebugMonitor | `DefaultHandler` |
| 13 | Reserved | `0` |
| 14-15 | PendSV, SysTick | `DefaultHandler` |
| 16-67 | IRQ0 - IRQ51 | `DefaultHandler` |

The firmware writes seven slots explicitly — 0, 1 and 3 populated; 8, 9, 10 and
13 zeroed — leaving the other 61, **slot 2 (NMI) included**, as `DefaultHandler`.
Hold that until §5.8.

Which indices are reserved is confirmed by the priority registers: SHPR2
(Table 206, PDF p185-186) documents `PRI_8`, `PRI_9` and `PRI_10` as "Reserved,
RES0" and SHPR3 (Table 207, PDF p186) documents `PRI_13` the same way, while
every other field in those registers names a real system handler. **Inferred:**
the *names* on indices 4-7, 11, 12, 14 and 15 are Armv8-M architectural rather
than quoted here — the RP2350 SHPR tables label every priority field
"SecureFault", a copy-paste error in the document, and §3.7.4.7 (PDF p137)
defers to the Armv8-M Architecture Reference Manual for the exception model.

Only the lower 46 IRQs reach peripherals: "only the lower 46 IRQ signals are
connected to system-level interrupt sources, and IRQs 46 to 51 are hardwired to
zero (never firing)" (PDF p84), reserved for a core to interrupt itself. The
table is still 68 entries — the slots exist whether or not anything drives them.

### 5.6.2 The Rust type problem

Slot 0 is the stack pointer, and its value is `_stack_top` — a symbol the
linker script defines (chapter 04 §4.9) and no Rust file defines. Declare it
before you can name it. Verbatim from the tree, doc comment included:

```rust
unsafe extern "C" {
    /// One past the last valid RAM byte; the initial stack pointer. The stack
    /// is full-descending, so the first push lands at `_stack_top - 4` and
    /// this address is never itself dereferenced — which matters, because it
    /// is outside the decoded SRAM range and would bus-fault.
    static _stack_top: u32;
}
```

In the tree that block sits immediately under `BOOT_INFO`, next to the vector
table that consumes it; §5.9 shows it in position. It is `static`, not
`static mut`, because you only ever take its *address* — the "value" of a linker
symbol is a fiction, and chapter 02 §2.6 is the section that explains why. The
five `.data` and `.bss` symbols get the same treatment in chapter 06 §6.4.2,
when there is code that needs them.

The table itself mixes three things: that **stack pointer** at index 0,
**function pointers** at 1-67, and **zeros** in the reserved slots. Rust arrays
are homogeneous, so this needs a union — verbatim, tree comments and all:

```rust
#[repr(C)]
#[derive(Clone, Copy)]
union Vector {
    /// An ordinary exception or interrupt handler.
    handler: unsafe extern "C" fn(),
    /// The reset handler. Diverges: there is nothing to return to.
    reset: unsafe extern "C" fn() -> !,
    /// Slot 0 only: the initial stack pointer value.
    stack_top: *const u32,
    /// An architecturally reserved slot, which must read as zero.
    reserved: u32,
}

// SAFETY: `Vector` contains a raw pointer, which is not `Sync`, so the
// compiler will not let a `static` hold one without this. It is sound here
// because the table is immutable, lives in read-only flash, and is only ever
// read by hardware performing vector fetches.
unsafe impl Sync for Vector {}
```

Four details, all of which you hit if you write this yourself:

- **`#[repr(C)]` is required.** `repr(Rust)` union layout is explicitly
  unspecified. It happens to give size 4 / align 4 today, but the silicon reads
  this structure; `repr(C)` turns an observation into a guarantee.
- **`reset` needs its own field.** A diverging `unsafe extern "C" fn() -> !`
  will not coerce to `unsafe extern "C" fn()` in union-field position; you get
  `error[E0308]: expected fn pointer 'unsafe extern "C" fn() -> ()'`.
- **`unsafe impl Sync`** is required and sound, for the reason the tree's
  `SAFETY:` comment states.
- **`&raw const` on an extern static const-evaluates**, so the initial stack
  pointer comes from the linker symbol rather than a hardcoded `0x20082000`.
  Chapter 02 §2.6 explains why it is `&raw const _stack_top` and not
  `_stack_top`. It is also why the script needs no `LONG(_stack_start)`, which
  is how `cortex-m-rt` sidesteps the same problem.

### 5.6.3 Building 68 entries

A `const` block with mutation is allowed in a static initialiser, so you fill
the defaults first and then override — verbatim from `lib.rs`:

```rust
#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0] = Vector { stack_top: &raw const _stack_top };
    t[1] = Vector { reset: OnReset };
    t[3] = Vector { handler: OnHardFault };
    t[8] = Vector { reserved: 0 };
    t[9] = Vector { reserved: 0 };
    t[10] = Vector { reserved: 0 };
    t[13] = Vector { reserved: 0 };
    t
};
```

The `[expr; 68]` repeat form is what makes `#[derive(Clone, Copy)]` on `Vector`
necessary; `#[used]` and `KEEP()` apply here exactly as in §5.4.

### 5.6.4 The Thumb bit

Same tool, different section —
`llvm-objdump -s -j .vector_table` on the **finished** image:

```
Contents of section .vector_table:
 10000000 00200820 db020010 d5020010 d5020010  . . ............
 10000010 d5020010 d5020010 d5020010 d5020010  ................
 10000020 00000000 00000000 00000000 d5020010  ................
 10000030 d5020010 00000000 d5020010 d5020010  ................
```

(The `file format` header line and the remaining 13 output lines — all
`d5020010` — are omitted.) Decode against §5.6.1:

- Word 0 → `0x20082000`; `nm` reports `_stack_top` at `20082000`. Exact.
- Word 1 → `0x100002db`; `nm` reports `OnReset` at `100002da`.
- Words 2 and 3 → `0x100002d5`; `nm` reports **both** `DefaultHandler` and
  `OnHardFault` at `100002d4`. Hold that thought until §5.8.
- Words 8, 9, 10 (offsets `0x20`, `0x24`, `0x28`) and word 13 (offset `0x34`) →
  `0x00000000`, the four reserved slots.

Every function pointer is one greater than its symbol address. **Bit 0 is set.**
On Arm, bit 0 of a vector selects the instruction set, and §3.7.4.6 (PDF p137)
is blunt: "All populated vectors in the vector table entries must have bit[0]
set. Creating a table entry with bit[0] clear generates an INVSTATE fault on the
first instruction of the handler corresponding to this vector." Bit 0 is what
loads into the EPSR T-bit on exception entry.

Rust function pointers carry the bit automatically, which is the payoff for
building the table out of typed `fn` pointers instead of `u32` literals. If you
ever hand-write a vector as a `u32`, you own the `+1` — that one at least fails
loudly, since INVSTATE escalates to HardFault. The reserved slots stay zero,
because "all populated vectors" does not include them: a reserved slot is not a
vector.

## 5.7 Handlers must not return

Exception return on Arm is not a normal branch to a code address. On exception
entry `LR` is loaded with a magic `EXC_RETURN` value of the form `0xffffffxx`,
and branching to it is what triggers the unstacking sequence. **Inferred:** that
encoding is Armv8-M architectural — the RP2350 datasheet mentions `EXC_RETURN`
only in passing, in the `SFSR.INVER` description (Table 249, PDF p201). The
consequence is a trap with no fault attached:

> **Silent-failure trap.** An **empty** `extern "C" fn` compiles to `bx lr`,
> which in a handler is a perfectly valid exception return. Nothing crashes.
> The core unstacks and resumes at the faulting instruction, which faults
> again, forever, leaving no fault status, no halt and no clue.

Each kind of handler therefore has exactly one correct body. A real peripheral
ISR clears the source at the peripheral, does the work, and **returns**. A
catch-all `DefaultHandler` **spins**, because it cannot clear a source it cannot
identify. Fault handlers **spin**, because returning re-executes the faulting
instruction. Returning never clears an interrupt on its own; a default handler
that returns produces an interrupt storm in which the foreground never advances
while the chip looks busy — harder to diagnose than a clean freeze. The firmware
takes the freeze — verbatim, doc comments elided:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn OnHardFault() {
    loop {}
}
```

Both compile to `b .`, a two-byte branch to itself: correct, and — as the next
section shows — identical.

## 5.8 Identifying the trap

Once you are stopped in a spinning handler, the question is *which* one. The
exception number currently executing **is** the vector table index, and the
datasheet exposes it in `ICSR.VECTACTIVE`, bits 8:0, "The exception number of
the current executing exception" (Table 200, PDF p182, offset `0x0ed04`).
**Inferred:** `IPSR` bits 8:0 hold the same value and are cheaper to read, but
that field width is Armv8-M architectural — the RP2350 datasheet lists IPSR only
as one of the three views of the PSR (§3.7.4.5, PDF p135-136). The firmware does
not read it today. Proposed, for a debugger:

```rust
// PROPOSED — not in the tree today
let ipsr: u32;
unsafe { core::arch::asm!("mrs {}, ipsr", out(reg) ipsr, options(nomem, nostack)); }
```

That gets you the number. It does not get you the *address* — and in the release
build the address has already been taken away from you.

> **Release-build trap.** `DefaultHandler` and `OnHardFault` have byte-identical
> bodies, so LLVM's identical code folding collapses them into one symbol under
> `--release`. Every vector that used to distinguish them now points at the same
> instruction. Under `--debug` they are two addresses and everything looks fine.

This is not hypothetical. `llvm-nm -n` on this chapter's staged build — where
`OnReset` is still a placeholder `loop {}` and folds too — release binary then
debug binary:

```
10000124 T DefaultHandler
10000124 T OnHardFault
10000124 T OnReset
```
```
10000124 T DefaultHandler
1000012c T OnHardFault
10000134 T OnReset
```

One address in release, three in debug, and it propagates straight into the
table. Debug `.vector_table`:

```
 10000000 00200820 35010010 25010010 2d010010  . . 5...%...-...
```

Word 1 (Reset) is `0x10000135` = `OnReset` + 1, word 2 (NMI) is `0x10000125` =
`DefaultHandler` + 1 and word 3 (HardFault) is `0x1000012d` = `OnHardFault` + 1
— distinct. In release all three are `0x10000125`.

The finished firmware keeps the two-way fold: `OnReset` grows a real body in
chapter 06 and separates, but `DefaultHandler` and `OnHardFault` stay
byte-identical forever, so the shipping `llvm-nm` reads

```
100002d4 T DefaultHandler
100002d4 T OnHardFault
100002da T OnReset
```

and `llvm-objdump -d` will only ever print one of the two names:

```asm
100002d4 <OnHardFault>:
100002d4: b580         	push	{r7, lr}
100002d6: 466f         	mov	r7, sp
100002d8: e7fe         	b	0x100002d8 <OnHardFault+0x4>
```

`DefaultHandler` is not missing from the release image; it *is* that. Run the
`llvm-nm` commands on your own builds and you will see the same collapse — it
follows from the source, not from anything about this machine.

The practical effect: a Pico 2 stopped at `0x100002d8` has told you nothing
about whether it took a HardFault or an unhandled interrupt, and because slot 2
was never overridden an NMI lands there too. To tell them apart, give the
handlers different bodies — read `IPSR` into a distinct local, spin on a
distinct constant — so there is nothing left for the folder to fold.

## 5.9 The file so far

Both statics are now checked against the image. Here is
`firmware/pico2/src/lib.rs` as this chapter leaves it — every listing above,
assembled in the order the tree has them, plus the one thing the chapter owes
you. Vector slot 1 names `OnReset`, and `OnReset` is chapter 06's subject, so it
stands here as a placeholder with its real signature and a spinning body. Per
§5.4.1's convention, the tree's doc comments are elided; every code line is the
tree's:

```rust
#![no_std]

use core::panic::PanicInfo;

#[used]
#[unsafe(link_section = ".boot_info")]
static BOOT_INFO: [u32; 5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579,
];

unsafe extern "C" {
    static _stack_top: u32;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Clone, Copy)]
union Vector {
    handler: unsafe extern "C" fn(),
    reset: unsafe extern "C" fn() -> !,
    stack_top: *const u32,
    reserved: u32,
}

unsafe impl Sync for Vector {}

// PLACEHOLDER — chapter 06 §6.1 replaces this body
#[unsafe(no_mangle)]
pub extern "C" fn OnReset() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn OnHardFault() {
    loop {}
}

#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0] = Vector { stack_top: &raw const _stack_top };
    t[1] = Vector { reset: OnReset };
    t[3] = Vector { handler: OnHardFault };
    t[8] = Vector { reserved: 0 };
    t[9] = Vector { reserved: 0 };
    t[10] = Vector { reserved: 0 };
    t[13] = Vector { reserved: 0 };
    t
};
```

`-> !` on the placeholder is not a detail to postpone. It is the real signature
from the tree, the `reset` union field will not accept anything else, and
chapter 06 replaces only what is between the braces.

That file builds and links as printed, with `demo/src/main.rs` still chapter
01's three lines. `cargo build --release`, staged:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:         300 B         4 MB      0.01%
             RAM:          8 KB       520 KB      1.54%
```

300 bytes: 272 of vector table, 20 of boot block, and 8 of `.text` — because
`OnReset`, `DefaultHandler` and `OnHardFault` are three functions with
byte-identical bodies, and §5.8's identical code folding collapses all three
into one three-instruction body: `push {r7, lr}` / `mov r7, sp` / `b .`, six
bytes, rounded up to eight by the `. = ALIGN(4)` at the end of `.text`
(chapter 04 §4.1). Dump the table and you can see it happen:

```
Contents of section .vector_table:
 10000000 00200820 25010010 25010010 25010010  . . %...%...%...
```

Word 1 is `0x10000125`, the same value as words 2 and 3 — at this stage the
reset vector and the fault vector are literally the same instruction. That is
correct for a placeholder and it goes away in chapter 06, where `OnReset` gets a
body of its own, exactly as §5.6.4's dump of the finished image shows.

Chapter 06 picks up at word 1: what `OnReset` does to the machine before it is
safe to hand control to an application.
