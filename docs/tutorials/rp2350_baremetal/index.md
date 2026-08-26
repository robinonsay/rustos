---
document_type: Tutorial Index — RP2350 Bare-Metal Rust
program: rustos (Raspberry Pi Pico 2 / RP2350)
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/
---

# Bare-Metal Rust on the Raspberry Pi Pico 2

Building a `#![no_std]` / `#![no_main]` firmware image for the RP2350 from
scratch — no HAL, no `cortex-m-rt`, no external crates. Every linker script
line, every table, every register write is hand-written and traced to a
datasheet citation.

## Scope

Starts at the linker script and runs through first light on the on-board LED.
Toolchain and workspace setup are assumed (see chapter 01 preamble).

**Hardware:** Raspberry Pi Pico 2 — RP2350A in QFN-60, dual Cortex-M33,
520 kB SRAM, 4 MB external QSPI flash (Winbond W25Q32RV).
**Target triple:** `thumbv8m.main-none-eabihf`

## Chapters

| # | Chapter | Covers |
|---|---------|--------|
| 01 | [Linker Scripts](01_linker_scripts.md) | What the linker does, sections, VMA vs LMA, script grammar, inspection tooling |
| 02 | [The RP2350 Memory Map](02_memory_map.md) | XIP/flash, SRAM banking, why the numbers are what they are |
| 03 | [The Linker Script](03_the_linker_script.md) | The annotated script, symbol contract, `ASSERT`s, stack accounting |
| 04 | [Boot Metadata and the Vector Table](04_boot_and_vectors.md) | IMAGE_DEF block, ARMv8-M vector layout, the Thumb bit |
| 05 | [The Reset Handler](05_reset_handler.md) | FPU enable, VTOR, `.data` copy, `.bss` zero |
| 06 | [Registers and Bit Manipulation](06_registers_and_bits.md) | `volatile`, read-modify-write, why Rust has no bitfields |
| 07 | [GPIO and IO_BANK0](07_gpio.md) | The four blocks, bring-up sequence, `GPIOn_CTRL` reference |

## Primary Sources

- `docs/rp2350-datasheet.pdf` — authoritative for everything chip-level
- `docs/pico-2-datasheet.pdf` — board-level (flash size, LED pin)
- `docs/icd/rp2350/gpio/` — project ICD, covers Clocks/GPIO/UART/I2C/SPI only

Section numbers throughout refer to the RP2350 datasheet unless stated.
Where a PDF page is cited it is the **PDF page**; the datasheet's own printed
number is one lower.

## Conventions

- Verified facts are cited. Anything inferred rather than cited is marked.
- Traps that produce **silent** failure are called out in blockquotes — those
  are the ones that cost days.
