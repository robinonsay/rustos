---
document_type: "Tutorial Index — RP2350 Bare-Metal Rust"
program: rustos (Raspberry Pi Pico 2 / RP2350)
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/
---

# Bare-Metal Rust on the Raspberry Pi Pico 2

You build a `#![no_std]` / `#![no_main]` firmware image for the RP2350, from an
empty directory to an LED blinking on the board in front of you. Every linker
script line, every boot-metadata word and every register write is hand-written
and traced to a datasheet citation.

Deliberately excluded: `cortex-m`, `cortex-m-rt`, `rp-hal`, `embassy`, `defmt`,
and every other external crate. Nothing outside this repository is compiled
into the image. You write the reset handler, the vector table and the boot
block yourself, because that is the material.

The workspace you build is three crates, and the split is part of what the
tutorial teaches:

- **`api`** — portable hardware-abstraction traits (`Write`, `Read`, `Gpio`,
  `Block`, …) plus the take-once `Board` type. No register addresses; compiles
  for your laptop as well as the chip.
- **`firmware/pico2`** — a **library**: the boot metadata block, the vector
  table, the reset handler, the `entry!` macro that names the application's
  entry point, and a GPIO driver implementing `api`'s traits.
- **`demo`** — the one **binary**: the blinky application, wired to the
  runtime by `pico2::entry!(main)` and driving the pin through `api`'s traits.

## What you end up with

A single ELF at `target/thumbv8m.main-none-eabihf/release/demo`: **7044 bytes
of flash**, 8200 bytes of RAM, and the on-board user LED on **GP25** blinking
under a `spin_loop` delay. GP25 is the LED pin on this board — *"GPIO25 OP
Connected to user LED"* (Pico 2 datasheet p9).

The linker prints the budget on every build, because `.cargo/config.toml`
passes `--print-memory-usage`, which chapter 01 sets up:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        7044 B         4 MB      0.17%
             RAM:        8200 B       520 KB      1.54%
```

Four sections carry the flash bytes, and nothing else does. Real output from
`llvm-objdump --section-headers` on the release build, cut off after the last
line worth looking at:

```
Sections:
Idx Name            Size     VMA      Type
  0                 00000000 00000000
  1 .vector_table   00000110 10000000 DATA
  2 .boot_info      00000014 10000110 DATA
  3 .text           00001888 10000124 TEXT
  4 .rodata         000001d8 100019ac DATA
  5 .data           00000000 20000000 DATA
  6 .bss            00000004 20000000 BSS
  7 .stack          00002000 20000008 BSS
  8 .comment        00000099 00000000
  9 .ARM.attributes 0000003a 00000000
```

Index 8 and upward is ELF bookkeeping: no address, not loaded, not part of the
7044, and the symbol and string tables below it are cut from the listing. What
ships is 272 bytes of vector table (68 entries), a 20-byte `IMAGE_DEF` boot
block, 6280 bytes of code, and 472 bytes of read-only data — most of the last
two being `core`'s formatting and panic machinery, pulled in by the
application's `unwrap()` calls; chapter 08 §8.13 itemises that cost.

`.data` is empty, but `.bss` is not: it is exactly 4 bytes — `BOARD_CREATED`,
the `AtomicBool` behind `api`'s take-once `Board` — so the reset handler's
zero loop does real work in the shipping image, and the copy loop moves zero
bytes. Chapter 06 shows exactly what both compile to. The 8200 B of RAM is
that word, 4 bytes of alignment padding, and the 8 kB stack reservation.

The result is a valid RP2350 boot image, and `picotool` will confirm that about
the **file**, with no board attached:

```
File demo.elf:

Program Information
 target chip:         RP2350
 image type:          ARM Secure

Fixed Pin Information
 none

Build Information
 none

Metadata Block 1
 address:             0x10000110
 next block address:  0x10000110
 block type:          image def
 target chip:         RP2350
 image type:          ARM Secure
 extra security:      not enabled
```

The blink itself is a pair of single stores: `SIO.GPIO_OUT_SET` at
`0xd0000018` and `SIO.GPIO_OUT_CLR` at `0xd0000020`, each written with
`1 << 25` and separated by `for _ in 0 .. 5_000_000 { spin_loop(); }`. They
are reached through the `api` traits — `main` calls
`pin25_o.write(true)` / `write(false)`, and the driver's `Write<bool>`
implementation picks the register.

There is no clock setup anywhere in this firmware — no XOSC, no PLL. `clk_sys`
runs from `clk_ref` at power-up (Table 540, PDF p516), and `clk_ref` runs from
the ring oscillator, which during boot "runs at a nominal 11MHz and is
guaranteed to be in the range 4.6MHz to 19.6MHz without randomisation"
(§8.3.1, PDF p560). **Inferred:** the blink period is therefore an uncalibrated
consequence of that unspecified frequency and of how LLVM unrolls the delay
loop, not a number anyone chose. Chapter 08 shows the delay loop that survives
optimisation, and the instructions it actually emits.

## Scope

Chapter 01 builds the workspace from an empty directory: `rustup`, the
`thumbv8m.main-none-eabihf` target, the four `Cargo.toml`s, `panic = "abort"`,
`.cargo/config.toml`, `build.rs`, and the build-and-inspect loop every later
chapter uses. Nothing is assumed except that you know Rust and have never done
embedded work. From there the tutorial runs straight through to a blinking LED;
getting the image onto the board is chapter 08 §8.14, at the point where there
is something worth flashing.

Not covered, because the firmware does not do it: clocks and PLL bring-up,
interrupts and the NVIC, UART/USB/logging of any kind, the second core, and the
Hazard3 RISC-V boot path.

**Hardware:** Raspberry Pi Pico 2 — RP2350A in QFN-60, 30 GPIO (Table 1,
PDF p14), dual Cortex-M33, 520 kB SRAM in 10 banks (PDF p14), 4 MB external QSPI
flash, a Winbond W25Q32RV (Pico 2 datasheet p4, p5).
**Target triple:** `thumbv8m.main-none-eabihf`

## Chapters

| # | Chapter | Covers |
|---|---------|--------|
| 01 | [Toolchain and Workspace](01_setup_and_workspace.md) | `rustup`, the target triple, the three crates and four `Cargo.toml`s, `.cargo/config.toml`, `build.rs`, the build-and-inspect loop |
| 02 | [Linker Scripts](02_linker_scripts.md) | what the linker does, sections, VMA vs LMA, script grammar, `#[used]` vs `KEEP()`, inspection tooling |
| 03 | [The RP2350 Memory Map](03_memory_map.md) | XIP/flash, SRAM banking, APB base addresses, why the four `MEMORY` numbers are what they are |
| 04 | [The Linker Script](04_the_linker_script.md) | the annotated script, the exported symbol contract, the `ASSERT`s, stack accounting |
| 05 | [Boot Metadata and the Vector Table](05_boot_and_vectors.md) | the `IMAGE_DEF` block, ARMv8-M vector layout, the Thumb bit, identical-handler folding, `lib.rs` so far (§5.9) |
| 06 | [The Reset Handler](06_reset_handler.md) | FPU enable, VTOR, `.data` copy, `.bss` zero, the `entry!` macro and `__rustos_main`, `lib.rs` so far (§6.5) |
| 07 | [Registers, Bits, and Register Blocks](07_registers_and_bits.md) | `volatile`, read-modify-write versus plain write, no bitfields in Rust, `#[repr(C)]` register blocks, the atomic aliases, the `api` seam |
| 08 | [First Blink](08_first_blink.md) | RESETS, the pad, the mux, the value; the bring-up order, the driver and its traits, the finished `demo`, flashing, the blink |
| 09 | [GPIO Reference](09_gpio_reference.md) — **reference; skip on a first pass** | `GPIOn_CTRL` / `GPIOn_STATUS`, the pad bits and the isolation latch, the SIO offset map, releasing a pin |

The directory also holds `archive/`, which is **revision A of this tutorial and
is superseded** — seven files numbered 01-07 that collide with the numbering
above. `archive/README.md` says what each was replaced by and where it was
wrong. Nothing in it is current; nothing above links into it.

## Two reading paths

**Start to finish, for a working blink: 01 → 08.** Read them in order. Each
chapter ends with something you can build, disassemble, or see on the board, and
each assumes only the chapters before it. Chapter 09 is not on this path.

That promise leans on a small set of complete listings. `link.ld` is written
once, whole, in chapter 04 §4.1. `firmware/pico2/src/lib.rs` grows across
three chapters, so chapters 05 and 06 each print **the whole file as it stands
at that point** — §5.9 and §6.5 — with the one body the next chapter writes
marked `PLACEHOLDER`. Both listings compile and link exactly as printed, and
each is followed by the `cargo build --release` output it produces, so you can
check yourself against a real number before moving on. `demo/src/main.rs` is
three lines in chapter 01, gains its `entry!` declaration and a placeholder
`main` in chapter 06 §6.4.4, and is printed finished in chapter 08 §8.12. The
driver files of chapter 08 are larger and are quoted item by item — every
struct, constant, function and trait impl appears, and the central
configuration function is printed whole in §8.10 — but the chapter does not
reprint the driver files end to end; the tree is the reference for those, and
the elision convention below says exactly how a listing may differ from it.

What a chapter does **not** do is in its frontmatter. `creates:` lists the
files that chapter tells you to write; `describes:` lists files it explains but
does not ask you to touch, and names the chapter that does. Chapters 02, 03, 07
and 09 create nothing at all.

Chapter 08 needs a handful of register fields you have not met before; it names
the exact section of chapter 09 that holds each one, so you never have to go
looking.

**Reference lookup: 09, plus the reference tails of 04 and 06.** Once you are
past the first blink and want a number rather than an explanation, three places
hold the tables:

- **chapter 09** — everything GPIO: the `GPIOn_CTRL` and `GPIOn_STATUS` field
  maps, the pad bits and their reset values, the isolation latch, and the SIO
  offset map. The rules for *modelling* one of those blocks as a `#[repr(C)]`
  struct live in chapter 07 §7.6, and chapter 09 points at them rather than
  repeating them.
- **chapter 04** — the linker's exported symbol contract (`__sdata`,
  `__edata`, `__sidata`, `__sbss`, `__ebss`, `_stack_top`, `_min_stack_size`)
  and the five `ASSERT`s that guard it.
- **chapter 06** — the PPB registers the reset handler touches: `CPACR` at
  `0xe000ed88` and `VTOR` at `0xe000ed08`.

Every register in those tables carries its datasheet table number. The tutorial
is a pointer into the datasheet, not a replacement for it.

## Primary sources

- `docs/rp2350-datasheet.pdf` — authoritative for everything chip-level
- `docs/pico-2-datasheet.pdf` — board-level: flash size and part, the LED pin,
  the test points
- `docs/icd/rp2350/gpio/` — this project's GPIO interface control document:
  `01_overview.md`, `02_pads.md`, `03_interrupts.md`, `04_registers.md`,
  `index.md`

Section numbers throughout refer to the RP2350 datasheet unless stated
otherwise. Where a PDF page is cited it is the **PDF page**; the datasheet's own
printed footer number is one lower.

## Conventions

These bind all nine chapters. Chapters point here instead of restating them.

**Citations.** Every factual claim about the hardware carries a datasheet
section (`§9.7`, `(9.11, Table 850)`), a table (`(Table 534)`), or a PDF page
(`(PDF p504)`) — usually a section or table and a PDF page together, so you can
find it either way. Board-level facts cite `Pico 2 datasheet p9`.

**Inference is marked.** Anything that is reasoning rather than quotation is
prefixed literally with **Inferred:** or set in a paragraph beginning "This is
inferred, not cited:". An inference never sits unmarked next to cited material.
A chapter that states a hardware fact without citing it and without marking it
has a bug in it.

**Callouts.** Blockquotes are reserved for failures that are *silent* — no
diagnostic, no fault, no clue. There are exactly three kinds:

> **Silent-failure trap.** Wrong behaviour, no diagnostic, no fault.

> **Release-build trap.** Works in debug, breaks under `--release`.

> **Hardware-destructive.** Bricks the running image.

Anything merely interesting stays as prose. A chapter with more than four
callouts is over-using them.

**Code blocks.** Rust quoted from the repository is verbatim in its **code
lines** — whitespace, brace style (`unsafe{`), odd spacing
(`pad.write_volatile( current_pad);`) and `//` comments included; it is not
tidied. One systematic elision is allowed and always applies: the tree's
`//!` and `///` **doc comments are elided from listings** unless a listing
says it keeps them, because the source files carry documentation many times
longer than the code. So your typed file and the tree may differ in doc
comments and in nothing else. Where a cleaner version of real code is worth
showing, the real one comes first and the second block is labelled "a cleaner
form, not what is in the tree". Any code that is **not** in the repository is
introduced by a sentence saying so, and carries a first-line comment:

```rust
// PROPOSED — not in the tree today
```

There is a third kind of block, and it exists so that every chapter can end in
a workspace that compiles. `firmware/pico2/src/lib.rs` is built up across
chapters 05 and 06 and `demo/src/main.rs` across 06 and 08, and a half-built
file is missing the body a later chapter writes. Where a chapter closes with a
complete file listing, the not-yet-written function bodies carry:

```rust
// PLACEHOLDER — chapter 08 §8.12 replaces this body
```

A `PLACEHOLDER` line always names the exact section that replaces it, and it is
always a body, never a signature: the signature is the real one from the tree,
so nothing you type has to be un-typed later. (One deliberate exception is
called out where it happens: chapter 01's `use pico2 as _;` line in `demo` is
replaced by `pico2::entry!(main);` in chapter 06.) By the end of chapter 08
every placeholder is gone.

Tool output — `objdump`, `nm`, `picotool`, the linker's memory table — is
quoted from a real run, never paraphrased. Language tags are `rust`, `ld`, `toml`,
`asm`, `text`; a plain fence means tool output. Every disassembly listing was
taken with `llvm-objdump -d -C`, and chapter 01 §1.7.1 explains why the `-C`
matters and which function names it changes. Two edits are made to disassembly
and no others: the trailing `@ imm = #0x…` that `llvm-objdump` appends to `b`
and `bl` lines is dropped as noise, and elided lines are marked `...`. Where a
listing carries `;` comments, those are the tutorial's and not the tool's.

Output comes in two kinds and they are always distinguished, because getting
this wrong is the fastest way to make a correct build look broken. Output from
a **staged** build — the workspace as it stands at the end of the chapter you
are reading — says so, and gives the numbers that differ from the finished
image; §1.9, §4.11.1, §5.5, §5.9, §6.5, §6.6 and §6.7 carry staged output, and
§6.5.1 tabulates the staged-versus-finished difference operand by operand.
Output from the **finished** firmware is labelled "finished" and is never
introduced by an instruction to go and build it, because before the end of
chapter 08 you cannot.

**Tables.** Register bit tables use the datasheet's own column order,
`| Bits | Field | Type | Reset |`. Offset maps use `| Offset | Name | Info |`.
Fields are never renamed or re-ordered.

**Numbers.** Hex is lowercase after `0x` (`0x40038068`), addresses at their
natural width — eight hex digits for a 32-bit address, three for a register
offset like `0x17c`. Rust literals keep the author's underscore style
(`0x4002_0000`, `5_000_000`). Bit positions are written `bit 6`, ranges
`bits 15:14`.

**Cross-references.** `§N.M` means a section in the same chapter; "chapter 08
§8.4" means a section in another one. You should never have to search for a
destination.

**Terminology,** fixed across all nine chapters:

- you **deassert** a reset — never "set the reset bit" for the release direction
- **the pad**, **the mux**, **the value** — PADS_BANK0, IO_BANK0, SIO
- **the bootrom** — one word, lowercase
- **VMA / LMA** — both expanded on first use in each chapter
- **GP25** is the board pin; **GPIO25** is the chip register name. The board
  datasheet uses both, and the distinction is worth holding.
- **the runtime** is the `pico2` library crate; **the application** is the
  `demo` binary crate; **the api crate** is the trait layer between them
- **the firmware** means the code in this repository as it exists in the tree,
  and **the image** is the binary built from it
  (`target/thumbv8m.main-none-eabihf/release/demo`); **proposed** means
  anything that does not exist in the tree

## A note on the numbers in this tutorial

Every address, offset, bit position, table number and PDF page was checked
against `docs/rp2350-datasheet.pdf`, and every block of tool output was pasted
from a run against the tree as it stood on 2026-08-29 (rustc 1.98.0,
LLD 22.1.8, picotool v2.3.0). If a number here disagrees with the datasheet,
the datasheet wins and the chapter is wrong. A tutorial with a wrong offset in
it is worse than no tutorial.
