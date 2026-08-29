---
document_type: Tutorial Chapter — The RP2350 Memory Map
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 2 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapter 01
sources: RP2350 datasheet 2.2, 4.2, 4.4; Pico 2 datasheet 1
---

# Chapter 02 — The RP2350 Memory Map

Every number in the linker script comes from here.

## 2.1 Top-level decode

Address decode is first performed on **bits 31:28** — one nibble (2.2, Table 7):

| Bus segment | Base |
|---|---|
| ROM | `0x00000000` |
| **XIP** (external flash) | **`0x10000000`** |
| **SRAM** | **`0x20000000`** |
| APB peripherals | `0x40000000` |
| AHB peripherals | `0x50000000` |
| Core-local peripherals (SIO) | `0xd0000000` |
| Cortex-M33 private registers (PPB) | `0xe0000000` |

Unmapped ranges raise a bus error.

## 2.2 XIP — execute in place

### 2.2.1 The physical situation

**The RP2350 die has no program flash.** It has a boot ROM at `0x00000000` and
520 kB of SRAM at `0x20000000`. That is all.

Your program lives on a **separate chip**. The Pico 2 datasheet names it:

> Pico 2 provides minimal (yet flexible) external circuitry to support the
> RP2350 chip: flash (**Winbond W25Q32RV**), crystal (Abracon ABM8-272-T3),
> power supplies and decoupling, and USB connector.

That part speaks QSPI — six wires, a serial protocol. A Cortex-M33 cannot
instruction-fetch from it.

### 2.2.2 What XIP is

Section 4.4 states it directly:

> The term execute-in-place refers to external memory mapped directly into the
> chip's internal address space. [...] For example, a processor instruction
> fetch from AHB address `0x10001234` results in a QSPI memory interface fetch
> from address `0x001234` in an external flash device.

The **QMI** (QSPI Memory Interface) watches the AHB bus for accesses in the
`0x1` region, synthesises the serial transaction, and returns the bytes as
though they came from memory. The CPU never knows.

The contrast worth holding: on a conventional MCU, flash is genuinely
memory-mapped silicon on the same die. Here it is a hardware *emulation* of
memory-mapped flash, over a serial link.

### 2.2.3 The base address is arithmetic

`XIP_BASE = 0x10000000` is the origin of a *window*, not the location of any
memory. Its only job:

```
flash chip offset = XIP address - 0x10000000
```

So `0x10000000` is byte **0** of the flash chip — the first bytes on the
device, which is exactly where the bootrom goes looking.

### 2.2.4 Four bases, one memory

Table 9 lists four, selected by bits **27:26**:

| Base | Meaning |
|---|---|
| `0x10000000` | `XIP_BASE` — **cached** |
| `0x14000000` | uncached |
| `0x18000000` | cache maintenance |
| `0x1c000000` | uncached, untranslated (bypass QMI address translation) |

These are **not four memories**. They are four views of the same bytes; the
address you choose tells the XIP subsystem how to treat the cache on the way
through.

**Use `0x10000000` for code.** `0x18…` is a write-only maintenance mirror
(write data is ignored; the low address bits select the operation), and
`0x1c…` bypasses the address translation the bootrom's rolling window relies on.

### 2.2.5 The cache

16 kB, two-way set-associative, **1-cycle hit**, physically two 8 kB banks
interleaving odd and even cache lines so both can be accessed in the same cycle
(4.4.1).

Without it, every instruction fetch would be a serial round-trip to an external
chip. With it, a hot loop that fits in 16 kB runs at roughly SRAM speed.

Software need not consider coherence *unless performing flash programming*.

### 2.2.6 `.rodata` is read-only in hardware — but writes do not fault

The XIP window is write-protected by default, via `XIP_CTRL.CTRL.WRITABLE_M0`.
Read the mechanism carefully (Table 439):

> Note the read-only behaviour is implemented by **downgrading writes to
> reads**, so writes will still cause allocation of an address, but have no
> other effect.

> **Silent-failure trap.** A wild pointer write into `.rodata` is a **no-op**,
> not a fault. Do not expect a HardFault to catch it.

### 2.2.7 Why `LENGTH = 4M` and not `16M`

QMI chip select 0 owns a **16 MB** AHB window (`0x10000000`–`0x10ffffff`;
CS1 starts at `0x11000000`). But QMI drives a **fixed 24-bit address phase**,
so a 4 MB device **aliases four times** inside that window.

The window is not the device. Set `LENGTH = 16M` and the linker will place code
at addresses that alias back onto itself, with no diagnostic. Use the device
size: **4 MB**, per the Pico 2 datasheet.

## 2.3 SRAM

### 2.3.1 On-chip, and not part of the core

Section 4.2:

> There is a total of **520 kB** (520 x 1024 bytes) of on-chip SRAM. For
> performance reasons, this memory is physically partitioned into ten banks,
> but logically it still behaves as a single, flat 520 kB memory.

Eight 64 kB banks + two 4 kB banks = 520 kB.

It is on the same die as the cores but is **not part of** them. It sits behind
the AHB5 fabric as ten independently-arbitrated slaves shared by both M33s, DMA,
and every other bus manager:

> Each SRAM bank is accessed via a dedicated AHB5 arbiter. This means different
> bus managers can access different SRAM banks in parallel, so up to six 32-bit
> SRAM accesses can take place every system clock cycle.

That is what the banking is *for*.

**The M33 has no tightly-coupled memory.** The chip says so directly — M33
`ID_MMFR0`, Table 218:

```
19:16  TCM: Indicates support for tightly coupled memories (TCMs)   RO  0x0
```

So the core owns its general-purpose and system registers, and nothing else.
Every other byte a program touches — including every stack push — is a bus
transaction.

### 2.3.2 Layout

Table 10 — SRAM0-7, striped on address bits 3:2:

```
SRAM_BASE / SRAM_STRIPED_BASE / SRAM0_BASE   0x20000000
SRAM4_BASE                                   0x20040000
SRAM_STRIPED_END                             0x20080000
```

Table 11 — SRAM8-9, never striped:

```
SRAM8_BASE   0x20080000
SRAM9_BASE   0x20081000
SRAM_END     0x20082000
```

`0x20082000 - 0x20000000 = 0x82000 = 532480 = 520 kB` exactly, and the striped
region ends precisely where SRAM8 begins. **Contiguous, no gap, no aliasing** —
so a single `RAM` region in the linker script is correct.

The `0x20040000` watermark is also the boundary between the **SRAM0 and SRAM1
power domains** — irrelevant until you care about low-power modes.

Table 11's note is worth remembering for later:

> These smaller blocks of SRAM are useful for hoisting high-bandwidth data
> structures like the processor stacks.

### 2.3.3 The stack top is deliberately outside RAM

```ld
_stack_top = ORIGIN(RAM) + LENGTH(RAM);   /* 0x20082000 */
```

The last valid byte is `0x20081fff`. `0x20082000` is **not decoded** by the
system AHB crossbar and would raise a bus fault if accessed.

That is safe *only* because the M33 stack is **full-descending**: the initial SP
is pre-decremented before the first push, so the first pushed word lands at
`_stack_top - 4` and `_stack_top` itself is never dereferenced.

(Full-descending is an ARMv8-M architecture guarantee, not stated in this
datasheet.)

## 2.4 APB peripherals

Every APB peripheral sits on a uniform **32 kB (`0x8000`) grid** — an address
decode convenience, not a reflection of register counts (`IO_BANK0` uses about
`0x180` bytes of its 32 kB window).

```
0x40000000  SYSINFO      0x40028000  IO_BANK0     0x40070000  UART0
0x40008000  SYSCFG       0x40030000  IO_QSPI      0x40078000  UART1
0x40010000  CLOCKS       0x40038000  PADS_BANK0   0x40080000  SPI0
0x40018000  PSM          0x40040000  PADS_QSPI    0x40090000  I2C0
0x40020000  RESETS       0x40048000  XOSC         0x400a8000  PWM
                         0x40050000  PLL_SYS      0x400b0000  TIMER0
```

Note the grouping is **by layer, not by bank**: the two IO muxes are adjacent,
then the two pad blocks. See chapter 07.

## 2.5 The resulting MEMORY block

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}
```

Four numbers, each traceable:

| Value | Source |
|---|---|
| `0x10000000` | 2.2.2 Table 9, `XIP_BASE` (cached mirror) |
| `4M` | Pico 2 datasheet 1 — W25Q32RV device size, **not** the 16 MB window |
| `0x20000000` | 2.2 Table 7 / 2.2.3 Table 10, `SRAM_BASE` |
| `520K` | 4.2, and `SRAM_END - SRAM_BASE` from Table 11 |
