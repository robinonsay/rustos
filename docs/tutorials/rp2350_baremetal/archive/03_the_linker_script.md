---
document_type: Tutorial Chapter — The Linker Script
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 3 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-02
artifact: firmware/pico2/link.ld
---

# Chapter 03 — The Linker Script

## 3.1 The script

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

## 3.2 Section order is flash order

The linker walks `SECTIONS` top to bottom handing out addresses. The two
boot-critical sections must therefore come **first**.

> **Silent-failure trap.** `.vector_table` carries an explicit address
> (`ORIGIN(FLASH)`). Declare it *after* `.text` and `.text` claims
> `0x10000000` first, producing:
>
> ```
> rust-lld: error: section .text virtual address range overlaps with .vector_table
> ```
>
> This stays latent while `.vector_table` is empty and appears the moment Rust
> code claims the section — i.e. long after you wrote the bug.

## 3.3 `.vector_table`

```ld
.vector_table ORIGIN(FLASH) : ALIGN(512) {
    KEEP(*(.vector_table));
    . = ALIGN(4);
} > FLASH
```

- **`ORIGIN(FLASH)`** pins it to image offset 0. With no `ENTRY_POINT` or
  `VECTOR_TABLE` item in the boot metadata, the bootrom *assumes* a vector table
  at the image start (5.9.3.3) and reads word 0 into SP, word 1 into PC.
- **`KEEP`** — nothing in Rust *calls* a vector table, so `--gc-sections` would
  delete it. Non-negotiable. (`#[used]` is also required on the Rust side; see
  1.5.)
- **`ALIGN(512)`** — ARMv8-M requires VTOR alignment of (entry count rounded up
  to a power of two) x 4. 68 entries -> 128 -> x4 = **512**.

  This is *stricter than this chip needs*: RP2350's VTOR only implements
  128-byte granularity (`TBLOFF` is bits 31:7, Table 201). The 512 comes from
  the architecture, not the part. Keep it — architecture wins.

## 3.4 `.boot_info`

```ld
.boot_info : ALIGN(4) {
    KEEP(*(.boot_info));
    . = ALIGN(4);
} > FLASH
```

RP2350-specific and mandatory (chapter 04). `ALIGN(4)` because 5.9.1 requires
blocks to start word-aligned; without it the section inherits alignment from
whatever input turns up, and a `[u8; 20]` declaration would land unaligned with
no diagnostic.

## 3.5 `.data` — the VMA/LMA split

```ld
.data : ALIGN(4) {
    __sdata = .;
    *(.data .data.*)
    . = ALIGN(4);
    __edata = .;
} > RAM AT > FLASH
__sidata = LOADADDR(.data);
```

`> RAM` sets the VMA; `AT > FLASH` sets the LMA. The value must survive
power-off (so it ships in flash) but must be writable (so it runs from RAM).

The three symbols are the **contract with the reset handler**:

| Symbol | Meaning |
|---|---|
| `__sidata` | source — where the bytes sit in flash (LMA) |
| `__sdata` | destination — where they must end up in RAM (VMA) |
| `__edata` | end — so you know how many |

The `. = ALIGN(4)` before `__edata` is load-bearing: it guarantees the length is
a whole number of words, which is what makes a `u32`-at-a-time copy safe.

## 3.6 `.bss`

```ld
.bss (NOLOAD) : ALIGN(4) {
    __sbss = .;
    *(.bss .bss.*)
    . = ALIGN(4);
    __ebss = .;
} > RAM
```

- **`(NOLOAD)`** — occupies address space, contributes no bytes to the image.
  Without it you would ship kilobytes of literal zeros.
- **No `AT >`** — there is no load address because there is nothing to load.
- Placed **after** `.data` so the two are contiguous in RAM.

`*(COMMON)` appears in many scripts; it is for C tentative definitions and Rust
never emits them.

## 3.7 The stack reservation

```ld
_min_stack_size = 8K;

.stack (NOLOAD) : ALIGN(8) {
    . = . + _min_stack_size;
} > RAM
```

The *real* stack descends from `_stack_top` at the top of RAM. This section
books the same number of bytes at the **bottom**, purely so the linker's region
check and `--print-memory-usage` account for it.

`ALIGN(8)` because AAPCS requires 8-byte stack alignment.

### 3.7.1 Why it exists

`--print-memory-usage` **excludes the stack**. Without this reservation it
reports `0%` RAM no matter how close `.bss` gets to collision. Measured, with
515 kB of `.bss` in 520 kB of RAM:

```
without .stack:   RAM: 515 KB / 520 KB   99.04%   ... links clean
with .stack:      rust-lld: error: section '.stack' will not fit in region 'RAM'
                  rust-lld: error: insufficient headroom between .bss and the stack
```

### 3.7.2 What it does not do

It is **not** a guard band. `.bss` grows up from the bottom, the stack descends
from the top, and nothing sits between them; a stack overflow still silently
eats statics. The load-bearing check is the
`ASSERT(_stack_top - __ebss >= _min_stack_size)`. The section only fixes the
*accounting*. (`flip-link` is the real fix, later.)

## 3.8 `/DISCARD/`

```ld
/DISCARD/ : { *(.ARM.exidx .ARM.exidx.*) }
```

`.ARM.exidx` is stack-unwinding metadata for `panic = "unwind"`. With
`panic = "abort"` and no unwinder it is dead weight — and as an *orphan* the
linker places it wherever it likes. Before this rule it was observed landing at
`0x20000000`, directly on top of `.data`.

## 3.9 `_stack_top`

```ld
_stack_top = ORIGIN(RAM) + LENGTH(RAM);   /* 0x20082000 */
```

Word 0 of the vector table. One past the last valid byte — correct because the
M33 stack is full-descending (2.3.3).

## 3.10 The assertions

Each guards a failure that is otherwise **a board that boots to nothing with no
diagnostic**:

| Assertion | Prevents |
|---|---|
| vector table at flash base | bootrom reading garbage as SP/PC |
| vector table % 512 | VTOR silently dropping the low bits, pointing below the real table |
| boot_info within 4 kB | bootrom never finding the IMAGE_DEF; image rejected |
| boot_info % 4 | malformed metadata block |
| stack headroom | `.bss`/stack collision at runtime |

`ASSERT` must sit **outside** the `SECTIONS` block (1.7).

## 3.11 Verified output

```
Idx Name            Size     VMA       LMA       Type
  1 .vector_table   00000110  10000000  10000000  DATA    <- 0x110 = 68 x 4
  2 .boot_info      00000014  10000110  10000110  DATA    <- 20 bytes
  3 .text           0000044c  10000124  10000124  TEXT
  4 .rodata         00000000  10000570  10000570  DATA
  5 .data           00000004  20000000  10000570  DATA    <- VMA != LMA
  6 .bss            00000010  20000004  20000004  BSS
  7 .stack          00002000  20000018  20000018  BSS

  __sidata = 10000570   == .data's LMA
  __sdata  = 20000000  ---+-- 4 bytes  = 1 word to copy
  __edata  = 20000004  ---+
  __sbss   = 20000004  ---+-- 16 bytes = 4 words to zero
  __ebss   = 20000014  ---+
  _stack_top = 20082000
```

Checks to run after any change:

1. `.vector_table` is `0x110` bytes at `0x10000000`
2. `.boot_info` is 4-aligned and inside the first 4 kB
3. `.data` has **two different** addresses, and `__sidata` equals its LMA
4. `.bss` immediately follows `.data`; `.stack` follows `.bss`
5. `.ARM.exidx` is absent
6. `_stack_top` is `0x20082000`

## 3.12 Known deviations

- The four `/* RP2350-E25 */` comments overstate the erratum. E25 is
  specifically about a `LOAD_MAP` with non-word sizes not raising an error, and
  this image contains no `LOAD_MAP`. The erratum's *workaround* text does
  recommend word-sized, word-aligned segments, so the `ALIGN(4)`s are good
  practice — they are just not E25 mitigations.
- No orphan-section guard (`ASSERT(SIZEOF(.got) == 0, ...)`) and no
  `ASSERT(__sidata % 4 == 0, ...)`. Both currently hold by construction; neither
  is enforced.
