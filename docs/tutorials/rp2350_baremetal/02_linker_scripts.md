---
document_type: "Tutorial Chapter — Linker Scripts"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 2 of 9
revision: B
effective_date: 2026-08-28
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapter 01
sources: none from the RP2350 datasheet — this chapter makes no chip-specific
  claim. Every behaviour below was measured on 2026-08-28 with rustc 1.98.0
  (88d9e12ae 2026-08-18), LLVM/LLD 22.1.8, targeting thumbv8m.main-none-eabihf.
  Chip addresses, offsets and datasheet citations begin in chapter 03.
creates: nothing
describes: firmware/pico2/link.ld — grammar only; chapter 04 writes it line by line
---

# Chapter 02 — Linker Scripts

## 2.1 What the linker actually does

`rustc` compiles each crate into an **object file**: machine code and data with
no idea where any of it will live. Every internal address is a placeholder plus
a relocation entry saying "patch me once you know."

The linker answers exactly one question: **what address does each byte get?**
It concatenates matching pieces, assigns real addresses, patches every
relocation, and emits an ELF.

On a hosted OS you never think about this because the OS supplies the answer —
a loader, virtual memory, a stack set up for you — through a built-in default
script encoding "put it anywhere, the MMU will sort it out." On a Cortex-M33
there is no loader. Nothing sets up a stack. Nothing copies initialised
variables anywhere. If you do not state where things go, nobody does.
**The linker script is you filling that vacuum.**

Everything in this chapter is generic linker behaviour. No RP2350 number
appears here except the two `MEMORY` lines quoted in §2.4.3, and those are
there only as grammar. The chip's real addresses start in chapter 03; the
project's own script is read line by line in chapter 04.

## 2.2 Sections

Compilers group code and data into *sections* by how they must be treated.

| Section | Contains | Needs flash? | Needs RAM? |
|---|---|---|---|
| `.text` | executable code | yes | no |
| `.rodata` | `const`s, string literals, immutable statics | yes | no |
| `.data` | mutable statics with a **non-zero** initialiser | yes (for the values) | yes |
| `.bss` | mutable statics initialised to **zero** | **no** | yes |

`.bss` is the elegant one. A 4096-byte zeroed buffer costs **zero** bytes of
flash — you ship the *instruction* "zero this range at startup," expressed as a
start and an end address, not 4096 zeros.

### 2.2.1 Measured behaviour

Declare several statics, compile for the target with `--emit=obj` (no linking,
so no script is needed yet), and ask the object where each one landed:

```rust
// PROPOSED — not in the tree today; a scratch file used only to measure
#![no_std]
#[unsafe(no_mangle)] pub static mut BIG_ZERO: [u8;4096] = [0;4096];
#[unsafe(no_mangle)] pub static mut BIG_NONZERO: [u8;4096] = [1;4096];
#[unsafe(no_mangle)] pub static mut ONE_BYTE_SET: [u8;4096] = {let mut a=[0u8;4096]; a[7]=1; a};
#[unsafe(no_mangle)] pub static RO_ZERO: u32 = 0;
#[unsafe(no_mangle)] pub static mut NONE_OPT: Option<u32> = None;
```

`--crate-type lib` is required and is not optional politeness. `rustc` invoked
by hand does not infer a crate type from the filename — only `cargo` does that.
Leave the flag off and `rustc` compiles this as a **binary**, which needs things
a library does not, and you get two errors about concepts this section has not
reached yet:

```
error: `#[panic_handler]` function required, but not found
error[E0601]: `main` function not found in crate `lib`
```

Neither error is about sections. Add the flag and the command below runs.

```
$ rustc --edition 2024 --crate-type lib --target thumbv8m.main-none-eabihf --emit=obj -O lib.rs
$ llvm-objdump --section-headers lib.o
Idx Name               Size     VMA      Type
  2 .text              00000000 00000000 TEXT
  3 .data.BIG_NONZERO  00001000 00000000 DATA
  4 .bss.BIG_ZERO      00001000 00000000 BSS
  5 .data.ONE_BYTE_SET 00001000 00000000 DATA
  6 .bss.NONE_OPT      00000008 00000000 BSS
  7 .rodata.RO_ZERO    00000004 00000000 DATA
```

`Type: BSS` is the whole point: a `BSS` section occupies no bytes in the file,
only an address range. `DATA` sections cost their full size in the image.

Three things fall out:

- **One non-zero byte disqualifies the whole array.** `BIG_ZERO` and
  `ONE_BYTE_SET` are the same 4096 bytes of RAM, but the second costs 4096
  bytes of flash. `.bss` can only be described as "a range to zero" — there is
  no room for exceptions. An array that is *almost* all zeros belongs in `.bss`
  with the exceptions written by code at startup.
- **The rule is about bytes, not intent.** `Option<u32> = None` landed in
  `.bss` because `None`'s representation happens to be all-zero — 8 bytes of
  it, discriminant included.
- **Immutable zero goes to `.rodata`, not `.bss`.** `.bss` is for data that is
  *both* zero-initialised *and* writable.

### 2.2.2 `const` versus `static` in Rust

These are not the same thing and only one of them has a section.

- **`const`** is a compile-time substitution. It has no address; every use site
  gets a copy inlined. It appears in **no section**.
- **`static`** is a thing at an address. That is what gets a section.

`pub const K: u32 = 0xdead_beef;` used once produces no `.rodata` at all — the
value shows up as immediates inside the function that used it:

```asm
       4: f64b 61ef    	movw	r1, #0xbeef
       8: f6cd 61ad    	movt	r1, #0xdead
```

(Caveat: taking a reference to a `const` can promote it to an anonymous
`.rodata` allocation. You cannot name or rely on that.)

### 2.2.3 Per-item sections

Rust emits **one section per item** so that unused ones can be garbage
collected (§2.5). A `#![no_main]` binary compiled to an object at
`-C opt-level=3`:

```
Idx Name                                                            Size     Type
  2 .text                                                           00000000 TEXT
  3 .text._RNvCs6rREvFdRhLb_7___rustc17rust_begin_unwind            00000006 TEXT
  4 .ARM.exidx.text._RNvCs6rREvFdRhLb_7___rustc17rust_begin_unwind  00000008 DATA
  6 .text._start                                                    00000006 TEXT
  7 .ARM.exidx.text._start                                          00000008 DATA
  9 .boot_info                                                      00000014 DATA
```

Note the size of the bare `.text` section: **zero**. Every byte of code is in a
`.text.<something>` section, which is why input patterns need the `.*` suffix —
`*(.text .text.*)`. Match only the bare `.text` and you link an empty binary,
successfully and with no error. The same applies to `.rodata.*`, `.data.*` and
`.bss.*`.

Note also the `.ARM.exidx.*` sections trailing every function: unwind tables
that a `panic = "abort"` firmware never uses. Chapter 04 shows the `/DISCARD/`
rule that throws them away.

## 2.3 VMA vs LMA — the concept everything hinges on

Every section has **two** addresses:

- **VMA** (virtual memory address, the runtime address) — where the code
  *thinks* it lives; where relocations point.
- **LMA** (load memory address) — where the bytes are physically stored in the
  image.

For `.text` and `.rodata` they are the same: stored in flash, executed from
flash. `.bss` has no LMA at all. `.data` is the interesting case — a mutable
static must live in RAM, but its initial value must survive power-off, so it
must ship in flash. **VMA in RAM, LMA in flash.** The linker proving it:

```
Idx Name       Size     VMA      LMA
  1 .text      0000008c 00100000 00100000     <- same
  2 .rodata    00000079 0010008c 0010008c     <- same
  3 .data      00000004 00200000 001000f0     <- SPLIT
  4 .bss       00000040 00200004 00200004     <- no flash at all
```

Those addresses are illustrative — round numbers chosen to make the split
visible, not this project's build. The real section table, with the RP2350's
own flash and RAM addresses, arrives in chapter 04.

Nothing moves those bytes from the LMA to the VMA. **The reset handler does**
(chapter 06). The linker's only contribution is telling you the three addresses
involved.

## 2.4 Script grammar

### 2.4.1 Lexical rules

- **Comments are `/* */` only.** `//` is a hard error:

  ```
  rust-lld: error: slash.ld:1: unknown directive: //
  >>> // a comment
  >>> ^
  ```

- Whitespace-insensitive; newlines are not significant.
- Numbers: decimal, `0x` hex, leading-`0` octal. Suffixes `K` and `M` mean
  x1024 and x1024^2 — not 1000. The project's `_min_stack_size = 8K;` resolves
  to `00002000 A _min_stack_size` in `llvm-nm`, which is 8192.
- Symbol assignments end with `;`. Block commands (`MEMORY { }`, `SECTIONS { }`)
  do not.
- Symbol names may contain `.`, `_`, `$`.

### 2.4.2 Top-level commands

| Command | Purpose |
|---|---|
| `ENTRY(sym)` | sets the ELF entry point; also roots `--gc-sections` |
| `MEMORY { … }` | declares physical regions |
| `SECTIONS { … }` | placement rules |
| `sym = expr;` | define a symbol |
| `PROVIDE(sym = expr);` | define only if nothing else did |
| `ASSERT(expr, "msg");` | link-time check |
| `INCLUDE file` | textual include |

`OUTPUT_FORMAT`/`OUTPUT_ARCH` are unnecessary — rustc passes the architecture.

### 2.4.3 MEMORY

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}
```

That block is quoted from `firmware/pico2/link.ld`. Where those four numbers
come from is chapter 03's subject, not this one. As grammar:

- Region names are arbitrary labels. Only the addresses are real.
- No commas *between* entries; commas separate `ORIGIN` and `LENGTH`.
- Attributes (`r` read, `w` write, `x` execute, `a` allocatable,
  `i`/`l` initialised) are **advisory**. They influence orphan placement and
  can produce warnings; they enforce nothing.

Measured: a script declaring only `FLASH (rx)` and directing `.text`, `.data`
**and** `.bss` into it links clean — no error, no warning — with writable
sections sitting in a region marked read-execute:

```
  1 .text           0000003c 10000000 TEXT
  2 .data           00000004 1000003c DATA
  3 .bss            00000010 10000040 BSS
```

Write the attributes for their documentation value; do not expect them to catch
a mistake.

### 2.4.4 SECTIONS

```text
<name> [<addr>] [(<type>)] : [ALIGN(<n>)]
{
  <contents>
} [> <vma-region>] [AT> <lma-region>]
```

Contents may mix **input section descriptions** — `*(.text .text.*)`, where
`*` globs *input files* and the parenthesised list gives *section name
patterns* — with **symbol assignments** (`__sdata = .;`), **location-counter
moves** (`. = ALIGN(4);`), **`KEEP(...)`** (exempt from garbage collection,
§2.5) and **`BYTE(x)` / `SHORT(x)` / `LONG(x)`**, which emit literal data into
the output. Section types go in parentheses: `(NOLOAD)` is the one you need,
for `.bss`.

### 2.4.5 `>` versus `AT >`

This is how the VMA/LMA split of §2.3 is expressed:

```ld
.data : ALIGN(4) {
  __sdata = .;
  *(.data .data.*)
  . = ALIGN(4);
  __edata = .;
} > RAM AT > FLASH          /* VMA in RAM, LMA in FLASH */
__sidata = LOADADDR(.data); /* ask where it ended up */
```

`>` names the region the section's VMA comes from; `AT >` names the region its
LMA comes from. Give only `>` and the two are the same.

### 2.4.6 The location counter

`.` is the current address being handed out. Reading it gives the address at
that point; assigning to it moves forward, creating padding. `. = ALIGN(4)`
bumps to the next multiple of 4. That is how `__sdata`/`__edata` capture a
section's boundaries: place a symbol, emit content, place another symbol.

### 2.4.7 Expressions

C-like operators throughout (arithmetic, bitwise, comparison, `&&`/`||`,
ternary `?:`), plus:

| Function | Returns |
|---|---|
| `.` | location counter |
| `ADDR(sec)` | a section's VMA |
| `LOADADDR(sec)` | a section's LMA |
| `SIZEOF(sec)` | a section's size |
| `ALIGN(n)` | `.` rounded up to a multiple of `n` |
| `ORIGIN(rgn)` / `LENGTH(rgn)` | from the `MEMORY` block |
| `DEFINED(sym)` | whether a symbol exists |
| `MAX(a,b)` / `MIN(a,b)` | as expected |

## 2.5 Garbage collection — `#[used]` versus `KEEP()`

**rustc passes `--gc-sections` by default.** You do not have to take that on
trust; make a link fail and rustc prints the command line it used, tail only:

```
"rust-lld" "-flavor" "gnu" … "-o" "slash.elf" "--gc-sections" "-O1" "-Tslash.ld"
```

With no `ENTRY()` and no `KEEP()`, the linker therefore concludes your entire
program is unreachable and deletes it. Linking a firmware whose entry point is
called `OnReset` against an **empty** script produces a binary with no
allocatable sections at all — only `.comment`, `.symtab` and friends — and one
warning:

```
warning: linker stderr: rust-lld: cannot find entry symbol _start; not setting start address
```

For a table nothing calls — a vector table, a boot header — **two independent
collectors** stand between you and a working image, running at different times:

| Who might delete it | When | What stops them |
|---|---|---|
| **rustc / LLVM** — drops an unreferenced static before the linker ever sees it | codegen | `#[used]` |
| **lld `--gc-sections`** — drops a section nothing references | link | `KEEP()` |

All four combinations, measured on an unreferenced
`#[unsafe(link_section = ".boot_info")] static BOOT_INFO: [u32;5]`:

| `#[used]`? | `KEEP()`? | opt-level | Result |
|---|---|---|---|
| no | no | 0 | reaches the linker, then dropped by `--gc-sections` |
| no | yes | 0 | survives — `.boot_info 00000014 10000008 DATA` |
| no | either | 3 | **never reaches the linker**; `KEEP()` cannot save it |
| yes | either | 3 | survives |

The third row is the trap. At `-C opt-level=3` the section is simply absent
from the object file, so there is nothing left for the script to keep:

```
$ rustc … --emit=obj -C opt-level=0 …    9 .boot_info  00000014 00000000 DATA
$ rustc … --emit=obj -C opt-level=3 …    (no .boot_info section)
```

> **Release-build trap.** In a *debug* build an unreferenced
> `#[link_section]` static survives codegen, and `KEEP()` alone carries it into
> the image. At `-C opt-level=3` rustc deletes it first, and the linker cannot
> keep what it never received. The result is firmware that boots from
> `cargo build` and bricks from `cargo build --release`, with no error from
> either. Write both `#[used]` and `KEEP()`, always.

On this toolchain `#[used]` marks the section `SHF_GNU_RETAIN` — visible in
`llvm-readobj --section-headers` as `Flags [ (0x200002) SHF_ALLOC,
SHF_GNU_RETAIN ]` — and lld honours that flag under `--gc-sections`, which is
why the fourth row survives without `KEEP()`. That is an implementation detail
of one linker version, not a contract; `KEEP()` also states the intent to a
human reader and protects sections coming from assembly or C, which carry no
such flag.

## 2.6 Linker symbols have addresses, not values

```
00200000 D __sdata
00200004 D __edata
001000f0 A __sidata
```

(Illustrative addresses again — the shape is what matters.) `__sidata` **is**
`0x001000f0`. There is no variable at that location containing the number
`0x001000f0`.

In Rust, declare them opaque and take the **address**:

```rust
unsafe extern "C" { static __sidata: u32; }
let src = &raw const __sidata;     // correct
// reading __sidata gives whatever bytes happen to be there
```

Declare them `static`, not `static mut` — you only ever take addresses, and
`static mut` drags in edition-2024's `static_mut_refs` rules for nothing.

> **Silent-failure trap.** Reading the symbol instead of taking its address
> compiles, links, and runs. You get whatever bytes happen to sit at the start
> of `.data` as your source pointer. This is the single most common first bug
> in a hand-written reset handler, and it presents as a HardFault far away from
> the mistake, or as no fault at all and a variable that is quietly wrong.

## 2.7 `ASSERT` placement

`ASSERT` **cannot go inside the `SECTIONS` block.** lld tries to parse it as
the start of an output-section definition, and the diagnostic depends on where
in the block you put it — none of them say "ASSERT belongs outside":

| Placement | Observed |
|---|---|
| last statement in `SECTIONS { }` | `rust-lld: error: a1.ld:3: malformed number: }` |
| first statement in `SECTIONS { }` | `rust-lld: error: a2.ld:3: symbol not found: .text` |
| after the closing `}` | works |

Outside the block it works, and it fires:

```ld
SECTIONS { .text : { *(.text .text.*) } }
ASSERT(ADDR(.text) == 0x1000, "text must be at 0x1000");
```

```
rust-lld: error: text must be at 0x1000
```

Assertions are the closest thing to a linter for linker scripts — there is no
type-checker and no LSP for this language, and editor support tops out at
syntax highlighting. Encode invariants there and the *linker* checks them on
every build:

```ld
ASSERT(ADDR(.vector_table) == ORIGIN(FLASH), "vector table must be at flash base");
ASSERT(ADDR(.boot_info) % 4 == 0,            "boot block must be word-aligned");
```

Each of those, unasserted, is a board that boots to nothing with no diagnostic.
The project's script carries five of them, all after the `SECTIONS` block;
chapter 04 reads them one at a time.

## 2.8 What the ecosystem does instead

No tool generates a linker script from a part number. What exists is a
hand-written script someone else already wrote, which you parameterise:

- **`cortex-m-rt`** ships a complete `link.x` — vector table, section
  definitions, the symbol contract, the reset handler. Your entire contribution
  is a `memory.x` containing **only the `MEMORY` block**.
- **`rp235x-hal`** ships that plus the `IMAGE_DEF` handling that chapter 05
  builds by hand.
- **`flip-link`** is a linker *wrapper* that inverts the layout so stack
  overflow hits a guard instead of silently eating `.bss`.

Writing one by hand is not how you would ship. It is how you become able to
read `link.x` and know what every line does.

You now have the grammar and none of the addresses. Every number this chapter
used was illustrative: `0x00100000`, `0x00200000`, the round figures in §2.3
and §2.6. **Chapter 03** replaces them, and answers the one question a linker
script cannot: where the RP2350's flash and SRAM actually are, and why
`LENGTH = 4M` rather than `16M`. Chapter 04 then reads this project's own
script line by line with both halves in hand.

