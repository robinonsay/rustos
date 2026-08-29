---
document_type: "Tutorial Chapter — The Linker Script"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 4 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-03
sources: RP2350 datasheet §2.2.2 Table 9 and §2.2.3 Tables 10 & 11 (PDF p32); §3.2 (PDF p83-84); §3.7 Cortex-M33 configuration (PDF p125); §3.7 M33 VTOR, Table 201 (PDF p183); §5.9.1 (PDF p417); §5.9.3.3 (PDF p423); §5.9.5 and §5.9.5.1 (PDF p427); RP2350-E25 (PDF p1356). Armv8-M Architecture Reference Manual for the vector-table alignment rule, which the RP2350 datasheet does not restate.
creates: firmware/pico2/link.ld
---

# Chapter 04 — The Linker Script

Chapter 02 covered linker scripts in general and chapter 03 covered where the
RP2350's memory is. This is the script that falls out of those two, and it ends
with the symbol contract the reset handler in chapter 06 §6.4 consumes.

Write `firmware/pico2/link.ld` from §4.1, then keep it open beside the chapter:
§4.2 through §4.10 take it one line at a time and say what breaks when that line
is missing.

**Do not expect an image at the end of this chapter.** The two crates are
still chapter 01 §1.9's stubs — a `pico2` library that is only a panic
handler, a `demo` binary that is only two attributes and an import — so the
script is correct and has nothing to place. §4.11.1 gives the build you can
actually run once §4.1 is typed — an empty one, and the emptiness is
informative. §4.11.2 gives the finished firmware's layout, which you reach at
the end of chapter 08.

## 4.1 The script

This is `firmware/pico2/link.ld` with its comments stripped. The file in the
tree carries the same content plus long block comments; nothing else differs.

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}

ENTRY(OnReset)

_min_stack_size = 8K;

SECTIONS
{
    .vector_table ORIGIN(FLASH) : ALIGN(512) {
        KEEP(*(.vector_table));
        . = ALIGN(4);
    } > FLASH

    .boot_info : ALIGN(4) {
        KEEP(*(.boot_info));
        . = ALIGN(4);
    } > FLASH

    .text : {
        *(.text .text.*)
        . = ALIGN(4);
    } > FLASH

    .rodata : {
        *(.rodata .rodata.*)
        . = ALIGN(4);
    } > FLASH

    .data : ALIGN(4) {
        __sdata = .;
        *(.data .data.*)
        . = ALIGN(4);
        __edata = .;
    } > RAM AT > FLASH
    __sidata = LOADADDR(.data);

    .bss (NOLOAD) : ALIGN(4) {
        __sbss = .;
        *(.bss .bss.*)
        . = ALIGN(4);
        __ebss = .;
    } > RAM

    .stack (NOLOAD) : ALIGN(8) {
        . = . + _min_stack_size;
    } > RAM
    /DISCARD/ : { *(.ARM.exidx .ARM.exidx.*) }

    _stack_top = ORIGIN(RAM) + LENGTH(RAM);
}
ASSERT(ADDR(.vector_table) == ORIGIN(FLASH),
    "vector table must be at flash base (datasheet 5.9.3.3)");
ASSERT(ADDR(.vector_table) % 512 == 0,
    "vector table violates VTOR alignment");
ASSERT(ADDR(.boot_info) - ORIGIN(FLASH) < 4096,
    "IMAGE_DEF must be in the first 4kB (datasheet 5.9)");
ASSERT(ADDR(.boot_info) % 4 == 0,
    "boot block must start word-aligned (datasheet 5.9.1)");
ASSERT(_stack_top - __ebss >= _min_stack_size,
    "insufficient headroom between .bss and the stack");
```

Both `MEMORY` entries come from chapter 03: the XIP window at `0x10000000`
(§2.2.2, Table 9, PDF p32), with `LENGTH = 4M` for the reason in chapter 03
§3.2.7, and SRAM at `0x20000000` running to `SRAM_END = 0x20082000` — 520 kB
(§2.2.3, Tables 10 and 11, PDF p32). The `(rx)` / `(rwx)` attributes document
intent and enforce nothing. `ENTRY(OnReset)` records the entry point in the ELF
header and gives `--gc-sections` a root to trace from; `rustc` passes
`--gc-sections` by default.

## 4.2 Section order is flash order

The linker walks `SECTIONS` top to bottom handing out addresses, so the order
of the output-section declarations *is* the order of bytes in the image. The
two boot-critical sections have to come first: the bootrom reads the vector
table from the image start (§5.9.3.3, PDF p423) and scans only the first 4 kB
for the metadata block (§5.9.5, PDF p427).

`.vector_table` is the one section with an explicit address (`ORIGIN(FLASH)`).
Declare it below `.text` and `.text` reaches `0x10000000` first. Measured, by
moving the `.vector_table` block below `.text` in a copy of this script and
linking an object that puts a `[u32; 68]` in `.vector_table`:

```
rust-lld: error: section .text virtual address range overlaps with .vector_table
rust-lld: error: section .text load address range overlaps with .vector_table
```

> **Silent-failure trap.** That error only appears once `.vector_table` has
> content. With the same reordered script and an object that claims no
> `.vector_table`, the link succeeds and `llvm-objdump` reports:
>
> ```
>   1 .boot_info      00000000 10000000 10000000 DATA
>   2 .text           00000010 10000000 10000000 TEXT
>   3 .vector_table   00000000 10000000 10000000 DATA
> ```
>
> Three sections at one address, no diagnostic. The bug is latent for as long
> as the Rust side is a stub, and surfaces on the day you add the real table —
> long after you wrote it.

## 4.3 `.vector_table`

```ld
.vector_table ORIGIN(FLASH) : ALIGN(512) {
    KEEP(*(.vector_table));
    . = ALIGN(4);
} > FLASH
```

**`ORIGIN(FLASH)`** pins the section to image offset 0. The firmware's
`IMAGE_DEF` carries neither an `ENTRY_POINT` nor a `VECTOR_TABLE` item, and in
that case "a VECTOR_TABLE at the start of the image is assumed" (§5.9.3.3, PDF
p423): the bootrom takes the initial SP from offset +0 and the reset address
from offset +4 (§5.9.5.1, PDF p427). Chapter 05 §5.3 covers the block itself.

**`KEEP`** — nothing in Rust *calls* a vector table, so `--gc-sections` would
delete it as unreachable. `KEEP` and `#[used]` are both required and neither
substitutes for the other; chapter 02 §2.5 has the measurement.

**`ALIGN(512)`** — the table has 68 entries: 16 Arm system exceptions plus 52
external interrupt lines (§3.2, PDF p83-84; the Cortex-M33 configuration list
in §3.7 gives the same count as "IRQ: 52 external interrupts", PDF p125). The
Armv8-M architecture requires the vector table to be aligned to the entry count
rounded up to a power of two, times four: 68 → 128 → 512 bytes.

That alignment rule is an Arm architecture rule. The RP2350 datasheet does not
restate it; what the datasheet documents is the implemented register, and the
implemented register is *weaker*: `VTOR.TBLOFF` is bits 31:7 (Table 201, PDF
p183), so this chip can only place a table on a 128-byte boundary, and the low
seven bits of anything you write are dropped.

**Inferred:** keeping `ALIGN(512)` costs nothing today — the section sits at a
64 kB-aligned flash base either way — and keeps the script correct if the table
ever moves. Architecture wins over part.

## 4.4 `.boot_info`

```ld
.boot_info : ALIGN(4) {
    KEEP(*(.boot_info));
    . = ALIGN(4);
} > FLASH
```

RP2350-specific and mandatory: a flash image without a valid `IMAGE_DEF` in its
first 4 kB is not a program as far as the bootrom is concerned (§5.9.5, PDF
p427). It replaces RP2040's checksummed second-stage bootloader — the same page
states there is no `boot2` requirement on RP2350.

`ALIGN(4)` implements the block rule directly: "Blocks must start on a
word-aligned boundary, and the total size is always an exact number of words"
(§5.9.1, PDF p417). Without it the section inherits whatever alignment the
input object happens to carry, and a `[u8; 20]` declaration would land
unaligned with nothing to say so.

The section is empty until Rust claims it with
`#[unsafe(link_section = ".boot_info")]`, and the string must match this name
exactly.

> **Silent-failure trap.** Mistype that string — `".bootinfo"`, `".boot-info"` —
> and the section stays empty. All four flash-side `ASSERT`s still pass: they
> test `ADDR(.boot_info)`, and an empty section still has an address (measured:
> a link with an empty `.boot_info` produces `.boot_info 00000000 10000000` and
> no diagnostic). `cargo build` is clean, `picotool info -a` reports no metadata
> block, and the board comes up in BOOTSEL as if the flash were blank. Verify
> with the size, not the address — chapter 05 §5.5.

## 4.5 `.data` — the VMA/LMA split

```ld
.data : ALIGN(4) {
    __sdata = .;
    *(.data .data.*)
    . = ALIGN(4);
    __edata = .;
} > RAM AT > FLASH
__sidata = LOADADDR(.data);
```

This is the only section with two addresses. `> RAM` sets the VMA (virtual
memory address — where the code refers to it at run time); `AT > FLASH` sets
the LMA (load memory address — where the bytes ship in the image). A mutable
static with a non-zero initialiser needs both: the value must survive power-off,
so it ships in flash, and it must be writable, so it lives in RAM. Chapter 02
§2.3 has the general form.

Nothing moves those bytes for you. The reset handler does, and these three
symbols are the contract it links against:

| Symbol | Meaning |
|---|---|
| `__sidata` | source — where the bytes sit in flash (LMA) |
| `__sdata` | destination — where they must end up in RAM (VMA) |
| `__edata` | end of the destination — so you know how many |

`__sidata` is assigned outside the braces because `LOADADDR` needs a completed
output section to report on. The `. = ALIGN(4)` before `__edata` is load-bearing:
it guarantees the span is a whole number of words, which is what makes the
`u32`-at-a-time copy in chapter 06 §6.4.3 sound. All three symbols have
addresses and no values — read one as a `u32` and you get whatever byte pattern
lives there (chapter 02 §2.6).

## 4.6 `.bss`

```ld
.bss (NOLOAD) : ALIGN(4) {
    __sbss = .;
    *(.bss .bss.*)
    . = ALIGN(4);
    __ebss = .;
} > RAM
```

- **`(NOLOAD)`** — occupies address space, contributes no bytes to the image.
  Without it you would ship kilobytes of literal zeros in flash.
- **No `AT >`** — there is no load address, because there is nothing to load.
- Placed immediately after `.data`, so the two are contiguous in RAM.

`__sbss` / `__ebss` bracket the region the reset handler zeroes, on the same
terms as the `.data` triple. `*(COMMON)` appears in most scripts and is absent
here deliberately: it collects C tentative definitions, and Rust never emits
them.

## 4.7 The stack reservation

```ld
_min_stack_size = 8K;

.stack (NOLOAD) : ALIGN(8) {
    . = . + _min_stack_size;
} > RAM
```

The *real* stack descends from `_stack_top` at the top of RAM (§4.9). This
section books the same number of bytes at the **bottom**, immediately above
`.bss`, and never contains anything. `ALIGN(8)` because AAPCS requires 8-byte
stack alignment at a public interface.

### 4.7.1 Why it exists

`--print-memory-usage` reports the space used by output sections, and it knows
nothing about a stack that is only implied by word 0 of the vector table.
Measured on this toolchain, linking an object with 515 kB of `.bss` against
three variants of this script:

```
neither .stack nor the headroom ASSERT:
    RAM:  515 KB / 520 KB   99.04%      links clean, exit 0

headroom ASSERT only:
    rust-lld: error: insufficient headroom between .bss and the stack
    RAM:  515 KB / 520 KB   99.04%      still reports 99%

both, as shipped:
    rust-lld: error: insufficient headroom between .bss and the stack
    rust-lld: error: section '.stack' will not fit in region 'RAM': overflowed by 3072 bytes
    RAM:  523 KB / 520 KB  100.58%
```

The first line is the one to look at. With no `.stack` section, a build that
cannot possibly run reports 99% and exits successfully. The reservation is what
makes the *number* honest; the `ASSERT` is what makes the *build* fail.

### 4.7.2 What it does not do

It is not a guard band. `.bss` grows up from `0x20000000`, the stack descends
from `0x20082000`, the `.stack` section sits in between as pure bookkeeping,
and nothing in the hardware watches the boundary.

> **Silent-failure trap.** A stack overflow on this firmware is not a fault. The
> Cortex-M33 has `MSPLIM` for exactly this and the firmware never writes it
> (chapter 06 §6.9), so a deep call chain walks the stack pointer straight down
> through `.stack`, through `.bss`, and into your statics — corrupting them and
> continuing to run. The `ASSERT` in §4.10 checks the *static* worst case at
> link time and cannot see the dynamic one. `flip-link` — which puts the stack
> below `.bss` so an overflow hits unmapped memory and faults — is the real fix,
> and it is not in this project.

## 4.8 `/DISCARD/`

```ld
/DISCARD/ : { *(.ARM.exidx .ARM.exidx.*) }
```

`.ARM.exidx` is stack-unwinding metadata. Both profiles set `panic = "abort"`
and there is no unwinder in the image, so it is dead weight — but it is emitted
anyway: the `libcore` rlib for `thumbv8m.main-none-eabihf` on this toolchain
contains 547 `.ARM.exidx` input sections, and `rustc` emits them for the
firmware's own functions with `-C panic=abort` too.

Kept, it is an *orphan* — a section no rule claims — and the linker puts it
wherever it likes. Measured, by removing this one line from a copy of the script
and linking a minimal image:

```
  3 .text           00000010 10000000 10000000 TEXT
  4 .ARM.exidx      00000010 10000010 10000010 DATA
  5 .rodata         00000000 10000020 10000020 DATA
```

lld put it between `.text` and `.rodata`, which is harmless here — but that is
the linker's choice, not the script's guarantee, and the only way to stop caring
what it chooses is to discard the input.

The rule sits between the `.stack` section and the `_stack_top` assignment,
which is cosmetically odd and functionally irrelevant: `/DISCARD/` matches
inputs and consumes no address space, so its position has no effect.

## 4.9 `_stack_top`

```ld
_stack_top = ORIGIN(RAM) + LENGTH(RAM);   /* 0x20082000 */
```

This is word 0 of the vector table (chapter 05 §5.6.1), and the value the
bootrom loads into SP before branching to the reset handler.

`0x20000000 + 520K = 0x20082000`, which is exactly `SRAM_END` in Table 11 (PDF
p32) — one past the last valid RAM byte. That is correct rather than
off-by-one: the Arm stack is full-descending, so the first push writes to
`_stack_top - 4` and the address itself is never dereferenced. Chapter 03
§3.3.3 covers why it matters that it is never dereferenced.

Confirmed in the image — the first word of `.vector_table` is `00 20 08 20`,
little-endian for `0x20082000`:

```
Contents of section .vector_table:
 10000000 00200820 db020010 d5020010 d5020010  . . ............
```

## 4.10 The assertions

Each one guards a failure whose symptom is a board that boots to nothing:

| Assertion | Prevents |
|---|---|
| `ADDR(.vector_table) == ORIGIN(FLASH)` | the bootrom reading whatever is at offset 0 as SP and PC (§5.9.3.3) |
| `ADDR(.vector_table) % 512 == 0` | a table the architecture cannot address; on this chip `VTOR` would drop the low 7 bits and point below the real table (Table 201) |
| `ADDR(.boot_info) - ORIGIN(FLASH) < 4096` | the bootrom never finding the `IMAGE_DEF` and rejecting the image (§5.9.5) |
| `ADDR(.boot_info) % 4 == 0` | a block that violates the word-alignment rule (§5.9.1) |
| `_stack_top - __ebss >= _min_stack_size` | `.bss` growing into the stack's static budget (§4.7.1) |

All five are link-time arithmetic over symbols the script itself defines, so
they cost nothing at run time and cannot be skipped by a profile.

`ASSERT` must sit **outside** the `SECTIONS` block (chapter 02 §2.7): inside, it
is a location-counter-sensitive statement and the addresses it reads are not
final yet.

## 4.11 Verified output

### 4.11.1 What this chapter's build actually produces

Type §4.1 into `firmware/pico2/link.ld` and build, with both crates still
chapter 01 §1.9's stubs. This is the whole of it, staged — real output, and not
what §4.11.2 shows:

```
$ cargo build --release
warning: linker stderr: rust-lld: cannot find entry symbol OnReset; not setting start address
  |
  = note: `#[warn(linker_messages)]` on by default

warning: linker stdout: Memory region         Used Size  Region Size  %age Used
                    FLASH:          0 GB         4 MB      0.00%
                      RAM:          8 KB       520 KB      1.54%

warning: `demo` (bin "demo") generated 2 warnings
    Finished `release` profile [optimized] target(s) in 0.23s
```

`FLASH: 0 GB` is not a typo and not a mistyped script. It is the correct answer,
and it is `ENTRY(OnReset)` (§4.1) producing it. Chapter 02 §2.5 established that
rustc passes `--gc-sections`; `ENTRY` is what names the root that collection
keeps things from. `OnReset` does not exist yet, so there is no root, nothing is
reachable, and nothing is kept — §2.5's mechanism with its input set to empty.
Measured: add `#[unsafe(no_mangle)] pub extern "C" fn Foo() { loop{} }`
to the `pico2` stub and flash stays at `0 GB`; add
`-C link-arg=--no-gc-sections` on top of that and it becomes `12 B` with
`10000000 T Foo` in `llvm-nm`. Every flash section is empty and collapses onto
`ORIGIN(FLASH)`:

```
$ llvm-objdump --section-headers target/thumbv8m.main-none-eabihf/release/demo
Idx Name            Size     VMA      Type
  1 .vector_table   00000000 10000000 DATA
  2 .boot_info      00000000 10000000 DATA
  3 .text           00000000 10000000 DATA
  4 .rodata         00000000 10000000 DATA
  5 .data           00000000 20000000 DATA
  6 .bss            00000000 20000000 BSS
  7 .stack          00002000 20000000 BSS
$ llvm-readobj --file-headers target/thumbv8m.main-none-eabihf/release/demo | grep Entry
  Entry: 0x0
```

The only non-empty section is `.stack`, which is 8 kB of nothing (§4.7), and
that is the whole `RAM: 8 KB` line. `llvm-nm -n` shows the linker symbols are
already right — `_min_stack_size = 0x2000`, `_stack_top = 0x20082000` — and
`__sidata` sits at `0x10000000` because `.data`'s load address is one past an
empty `.rodata` at `ORIGIN(FLASH)`.

Three things should be true of your staged build, and none of them mention
`.text`: the two `MEMORY` regions report `4 MB` and `520 KB`; `RAM` reports
`8 KB`; and the only linker complaint is the missing `OnReset`. If instead the
build fails with `cannot find linker script link.ld`, `build.rs` or
`.cargo/config.toml` is the problem, not the script — chapter 01 §1.6 and §1.9.
Chapter 05 §5.9 is the first build in this tutorial with bytes in it: 300 B,
once `firmware/pico2/src/lib.rs` defines `OnReset` and the two `static`s.

### 4.11.2 The finished firmware

Everything below is the **finished** image, at the end of chapter 08. It is the
target, not this chapter's output. From `cargo build --release`,
`llvm-objdump --section-headers` on
`target/thumbv8m.main-none-eabihf/release/demo`:

```
Idx Name            Size     VMA      Type
  1 .vector_table   00000110 10000000 DATA
  2 .boot_info      00000014 10000110 DATA
  3 .text           0000183c 10000124 TEXT
  4 .rodata         0000019c 10001960 DATA
  5 .data           00000000 20000000 DATA
  6 .bss            00000000 20000000 BSS
  7 .stack          00002000 20000000 BSS
```

`llvm-nm -n`, filtered to the linker symbols:

```
00002000 A _min_stack_size
10001afc A __sidata
20000000 B __ebss
20000000 R __edata
20000000 B __sbss
20000000 R __sdata
20082000 A _stack_top
```

and the linker's own memory report, which arrives as a `linker_messages`
warning because of `--print-memory-usage`:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        6908 B         4 MB      0.16%
             RAM:          8 KB       520 KB      1.54%
```

Read three things off that. `.vector_table` is `0x110` = 272 bytes = 68 × 4 and
`.boot_info` is `0x14` = 20 bytes, the minimum `IMAGE_DEF` of §5.9.5.1. The
whole 8 kB of RAM in use is the `.stack` reservation. And — the point chapter 06
§6.7 makes from the other side — **`.data` and `.bss` are both zero-length**:
`__sdata`, `__edata`, `__sbss` and `__ebss` are all `0x20000000`, so both loops
in the reset handler run zero iterations. The release codegen computes the
count, gets zero, and calls `__aeabi_memcpy4` with it anyway.

Note there is no LMA column at all. `.data` is empty, so no section has an LMA
that differs from its VMA and `llvm-objdump` drops the column; the only
surviving evidence of `AT > FLASH` is `__sidata = 0x10001afc`, one past the end
of `.rodata` in flash (`0x10001960 + 0x19c`), exactly where the first byte of
`.data` would ship.

> **Silent-failure trap.** Do not read these values out of `firmware.map`. lld's
> map prints, for a symbol-assignment line, the *location counter* at that point
> rather than the symbol's value. The shipping map file contains
> `20002000 20002000 0 1 _stack_top = ORIGIN(RAM) + LENGTH(RAM)` and
> `20000000 20000000 0 1 __sidata = LOADADDR(.data)` — neither number is the
> symbol's value (`0x20082000` and `0x10001afc` respectively, per `llvm-nm`).
> The map is right about sections and misleading about assignments. Use
> `llvm-nm` for symbols.

The same layout with the chapter-06 §6.7 test statics added — the same tree,
same date, one `u32` in `.data` and a `[u32; 4]` in `.bss`:

```
Idx Name            Size     VMA       LMA       Type
  1 .vector_table   00000110  10000000  10000000  DATA
  2 .boot_info      00000014  10000110  10000110  DATA
  3 .text           0000183c  10000124  10000124  TEXT
  4 .rodata         0000019c  10001960  10001960  DATA
  5 .data           00000004  20000000  10001afc  DATA    <- VMA != LMA
  6 .bss            00000010  20000004  20000004  BSS
  7 .stack          00002000  20000018  20000018  BSS

  __sidata = 10001afc  == .data's LMA
  __sdata  = 20000000  ---+-- 4 bytes  = 1 word to copy
  __edata  = 20000004  ---+
  __sbss   = 20000004  ---+-- 16 bytes = 4 words to zero
  __ebss   = 20000014  ---+
  _stack_top = 20082000
```

That is the arrangement the script is designed for and the only one in which
the reset handler's loops do any work. The LMA column reappears the moment
`.data` is non-empty.

Checks to run after any change to `link.ld`:

1. `.vector_table` is `0x110` bytes at `0x10000000`
2. `.boot_info` is 4-aligned, non-zero in size, and inside the first 4 kB
3. `.data`, when non-empty, has **two different** addresses, and `__sidata`
   equals its LMA
4. `.bss` immediately follows `.data`; `.stack` follows `.bss`
5. `.ARM.exidx` is absent
6. `_stack_top` is `0x20082000`, read from `llvm-nm` and not from the map
7. in the shipping firmware specifically, `.data` and `.bss` are both
   zero-length and `__sdata == __edata == __sbss == __ebss == 0x20000000`. If
   either grows, something acquired a mutable static, the reset handler's loops
   stop being no-ops, and both are now on the critical path to boot

## 4.12 Known deviations

- The three `/* RP2350-E25 */` comments overstate the erratum. E25 is
  specifically that "a `LOAD_MAP` which uses non-word sizes does not cause an
  error" (PDF p1356), and this image contains no `LOAD_MAP` — its `IMAGE_DEF`
  is the 20-byte minimum. The erratum's workaround text does say "a best
  practice is to make sure that linker memory segments are both word-sized and
  word-aligned", so the `ALIGN(4)`s are good practice; they are not E25
  mitigations.
- There is no orphan-section guard. `ASSERT(SIZEOF(.got) == 0, ...)` is the
  usual one, and §4.8 showed that an unclaimed section gets placed silently.
  Nothing in this script would catch a `.got` appearing.
- There is no `ASSERT(SIZEOF(.boot_info) == 20, ...)`, which is what would
  actually catch the mistyped `link_section` of §4.4.
- There is no `ASSERT(__sidata % 4 == 0, ...)`. It holds by construction —
  `.rodata` ends on `. = ALIGN(4)` — but nothing enforces it.

---

The script now says where every byte goes, and the `ASSERT`s say so out loud on
every build. What it does **not** do is put anything in `.vector_table` or
`.boot_info`: both are still empty output sections waiting for a Rust `static`
to claim them. **Chapter 05** writes those two statics — the 20-byte `IMAGE_DEF`
the bootrom scans for, and the 68-entry vector table whose word 0 is the
`_stack_top` this chapter just computed.
