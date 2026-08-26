---
document_type: Tutorial Chapter — Linker Scripts
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 1 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: a workspace that builds for thumbv8m.main-none-eabihf
---

# Chapter 01 — Linker Scripts

## 1.1 What the linker actually does

`rustc` compiles each crate into an **object file**: machine code and data with
no idea where any of it will live. Every internal address is a placeholder plus
a relocation entry saying "patch me once you know."

The linker answers exactly one question: **what address does each byte get?**
It concatenates matching pieces, assigns real addresses, patches every
relocation, and emits an ELF.

On a hosted OS you never think about this because the OS supplies the answer —
a loader, virtual memory, a stack set up for you. There is a built-in default
script encoding "put it anywhere, the MMU will sort it out."

On a Cortex-M33 there is no loader. Nothing sets up a stack. Nothing copies
initialised variables anywhere. If you do not state where things go, nobody
does. **The linker script is you filling that vacuum.**

## 1.2 Sections

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

### 1.2.1 Measured behaviour

Declaring several statics and asking the linker where each landed:

| Declaration | Section | RAM | Flash |
|---|---|---|---|
| `static mut BIG_ZERO: [u8;4096] = [0;4096]` | `.bss` | 4096 | **0** |
| `static mut BIG_NONZERO: [u8;4096] = [1;4096]` | `.data` | 4096 | **4096** |
| `static mut ONE_BYTE_SET: [u8;4096]` *(one byte = 1)* | `.data` | 4096 | **4096** |
| `static RO_ZERO: u32 = 0` | `.rodata` | 0 | 4 |
| `static mut NONE_OPT: Option<u32> = None` | `.bss` | 8 | **0** |

Three things fall out:

- **One non-zero byte disqualifies the whole array.** `.bss` can only be
  described as "a range to zero"; there is no room for exceptions. An array
  that is *almost* all zeros belongs in `.bss` with the exceptions written by
  code at startup.
- **The rule is about bytes, not intent.** `Option<u32> = None` landed in
  `.bss` because `None`'s representation happens to be all-zero.
- **Immutable zero goes to `.rodata`, not `.bss`.** `.bss` is for data that is
  *both* zero-initialised *and* writable.

### 1.2.2 `const` versus `static` in Rust

These are not the same thing and only one of them has a section.

- **`const`** is a compile-time substitution. It has no address; every use site
  gets a copy inlined. It appears in **no section** — a `const` used once
  becomes an immediate operand in `.text`.
- **`static`** is a thing at an address. That is what gets a section.

(Caveat: taking a reference to a `const` can promote it to an anonymous
`.rodata` allocation. You cannot name or rely on that.)

### 1.2.3 Per-item sections

Rust emits **one section per item** — `.text.OnReset`, `.data._RNvC..COUNTER.0`
— so that unused ones can be garbage collected. This is why input patterns need
the `.*` suffix:

```ld
*(.text .text.*)
```

Match only the bare `.text` and you will link an empty binary.

## 1.3 VMA vs LMA — the concept everything hinges on

Every section has **two** addresses:

- **VMA** (virtual / runtime address) — where the code *thinks* it lives; where
  relocations point.
- **LMA** (load address) — where the bytes are physically stored in the image.

For `.text` and `.rodata` they are the same: stored in flash, executed from
flash. `.bss` has no LMA at all.

`.data` is the interesting case. A mutable static must live in RAM, but its
initial value must survive power-off, so it must ship in flash.
**VMA in RAM, LMA in flash.** The linker proving it:

```
Idx Name       Size     VMA      LMA
  1 .text      0000008c 00100000 00100000     <- same
  2 .rodata    00000079 0010008c 0010008c     <- same
  3 .data      00000004 00200000 001000f0     <- SPLIT
  4 .bss       00000040 00200004 00200004     <- no flash at all
```

Nothing moves those bytes from the LMA to the VMA. **The reset handler does**
(chapter 05). The linker's only contribution is telling you the three addresses
involved.

## 1.4 Script grammar

### 1.4.1 Lexical rules

- **Comments are `/* */` only.** `//` is a hard error (`unknown directive: //`).
- Whitespace-insensitive; newlines are not significant.
- Numbers: decimal, `0x` hex, leading-`0` octal. Suffixes `K` and `M` mean
  x1024 and x1024^2 — not 1000.
- Symbol assignments end with `;`. Block commands (`MEMORY { }`, `SECTIONS { }`)
  do not.
- Symbol names may contain `.`, `_`, `$`.

### 1.4.2 Top-level commands

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

### 1.4.3 MEMORY

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}
```

- Region names are arbitrary labels. Only the addresses are real.
- No commas *between* entries; commas separate `ORIGIN` and `LENGTH`.
- Attributes (`r` read, `w` write, `x` execute, `a` allocatable,
  `i`/`l` initialised) are **advisory**. They influence orphan placement and
  produce warnings; they enforce nothing. Write them for documentation value.

### 1.4.4 SECTIONS

```
<name> [<addr>] [(<type>)] : [ALIGN(<n>)]
{
  <contents>
} [> <vma-region>] [AT> <lma-region>]
```

Contents may mix:

- **input section descriptions** — `*(.text .text.*)`, where `*` globs
  *input files* and the parenthesised list gives *section name patterns*.
- **symbol assignments** — `__sdata = .;`
- **location-counter moves** — `. = ALIGN(4);`
- **`KEEP(...)`** — exempt from garbage collection
- **`BYTE(x)` / `SHORT(x)` / `LONG(x)`** — emit literal data into the output

Section types in parentheses: `(NOLOAD)` is the one you need, for `.bss`.

### 1.4.5 `>` versus `AT >`

This is how the VMA/LMA split is expressed:

```ld
.data : ALIGN(4) {
  __sdata = .;
  *(.data .data.*)
  . = ALIGN(4);
  __edata = .;
} > RAM AT > FLASH          /* VMA in RAM, LMA in FLASH */
__sidata = LOADADDR(.data); /* ask where it ended up */
```

### 1.4.6 The location counter

`.` is the current address being handed out. Reading it gives the address at
that point; assigning to it moves forward, creating padding. `. = ALIGN(4)`
bumps to the next multiple of 4.

That is how `__sdata`/`__edata` capture a section's boundaries: place a symbol,
emit content, place another symbol.

### 1.4.7 Expressions

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

## 1.5 Garbage collection

**rustc passes `--gc-sections` by default.** With no `ENTRY()` and no `KEEP()`,
the linker concludes your entire program is unreachable and deletes it. An
empty script produces a binary with *zero* sections and the warning
`cannot find entry symbol _start`.

Two independent collectors must both be satisfied for a table nothing calls:

| Who might delete it | What stops them |
|---|---|
| **rustc / LLVM** — drops an unreferenced static before the linker sees it | `#[used]` |
| **lld `--gc-sections`** — drops a section nothing references | `KEEP()` |

> **Silent-failure trap.** In a *debug* build an unreferenced
> `#[link_section]` static often survives without `#[used]`. In `--release`
> with LTO it is dropped. The result is firmware that boots from
> `cargo build` and bricks from `cargo build --release`.

## 1.6 Linker symbols have addresses, not values

```
00200000 D __sdata
00200004 D __edata
001000f0 A __sidata
```

`__sidata` **is** `0x001000f0`. There is no variable at that location
containing the number `0x001000f0`.

In Rust, declare them opaque and take the **address**:

```rust
unsafe extern "C" { static __sidata: u32; }
let src = &raw const __sidata;     // correct
// reading __sidata gives whatever bytes happen to be there
```

Declare them `static`, not `static mut` — you only ever take addresses, and
`static mut` drags in edition-2024's `static_mut_refs` rules for no benefit.

> This is the single most common first bug in a hand-written reset handler.

## 1.7 `ASSERT` placement

`ASSERT` **cannot go inside the `SECTIONS` block.** lld parses it as the start
of an output-section definition and demands a `:`.

| Placement | Result |
|---|---|
| inside `SECTIONS { }` | `error: : expected, but got ;` |
| after the closing `}` | works, and still fires |

Assertions are the closest thing to a linter for linker scripts. Encode
invariants there and the *linker* enforces them on every build:

```ld
ASSERT(ADDR(.vector_table) == ORIGIN(FLASH), "vector table must be at flash base");
ASSERT(ADDR(.boot_info) % 4 == 0,            "boot block must be word-aligned");
```

Each of those, unasserted, is a board that boots to nothing with no diagnostic.

## 1.8 Tooling

There is no linter, LSP, or type-checker for linker scripts — editor support
tops out at syntax highlighting. The checking you want lives in two places:
`ASSERT` (above) and linker flags.

### 1.8.1 Flags

```toml
# .cargo/config.toml
[target.thumbv8m.main-none-eabihf]
rustflags = [
  "-C", "link-arg=-Tlink.ld",
  "-C", "link-arg=--print-memory-usage",
]
```

- **`--print-memory-usage`** — a region utilisation table every build.
- **`--orphan-handling=warn|error`** — catches input sections matching no rule
  (the "I forgot `.rodata.*` and my constants vanished" class). Start at `warn`;
  debug builds also report `.debug_*` and `.comment` as orphans.
- **`-Map=out.map`** — full map file: which input file contributed each byte.

> **Path gotcha.** Cargo invokes `rustc` with the working directory set to the
> **workspace root**, not the crate directory. A bare `-Tlink.ld` resolves at
> the workspace root. To keep the script beside its crate, add a `build.rs`:
>
> ```rust
> fn main() {
>     let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
>     println!("cargo:rustc-link-search={dir}");
>     println!("cargo:rerun-if-changed=link.ld");
> }
> ```
>
> The `rerun-if-changed` line matters day to day: without it, Cargo does not
> know the script exists, and editing it produces a "Finished" with a **stale
> binary**.

### 1.8.2 Inspection

The rustup `llvm-tools` component provides these; add to `PATH` with:

```
export PATH="$PATH:$(rustc --print sysroot)/lib/rustlib/<host-triple>/bin"
```

| Command | Answers |
|---|---|
| `llvm-objdump --section-headers <elf>` | did my sections land where I said? (VMA/LMA/size) |
| `llvm-nm <elf>` | where did each symbol go? (`T`=.text `R`=.rodata `D`=.data `B`=.bss `A`=absolute) |
| `llvm-objdump -s -j .data <elf>` | what bytes are actually in this section? |
| `llvm-objdump -d <elf>` | disassembly |
| `llvm-objdump -f <elf>` | entry point |
| `llvm-size <elf>` | text/data/bss totals |

Note `llvm-readelf` is **not** in the rustup set — use `llvm-objdump -f` or
`llvm-readobj`.

## 1.9 What the ecosystem does instead

No tool generates a linker script from a part number. What exists is a
hand-written script someone else already wrote, which you parameterise:

- **`cortex-m-rt`** ships a complete `link.x` — vector table, section
  definitions, the symbol contract, the reset handler. Your entire contribution
  is a `memory.x` containing **only the `MEMORY` block**.
- **`rp235x-hal`** ships that plus the IMAGE_DEF handling.
- **`flip-link`** is a linker *wrapper* that inverts the layout so stack
  overflow hits a guard instead of silently eating `.bss`.

Writing one by hand is not how you would ship. It is how you become able to
read `link.x` and know what every line does.
