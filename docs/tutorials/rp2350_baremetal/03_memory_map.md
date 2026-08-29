---
document_type: "Tutorial Chapter — The RP2350 Memory Map"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 3 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-02
sources: RP2350 datasheet §2.2 (Tables 7, 9, 10, 11, 12, 14, 15), §3.1 (PDF p39),
  §3.7 (Table 218), §4.1-4.2 (Table 433), §4.4 and §4.4.1, §4.4.5 (Table 439),
  §12.14 and §12.14.4; Pico 2 datasheet p4, p5
creates: nothing
describes: firmware/pico2/link.ld (where its MEMORY literals come from; chapter 04
  writes it), firmware/pico2/src/common/reg.rs (where its four base addresses come
  from; chapter 08 §8.3 writes it)
---

# Chapter 03 — The RP2350 Memory Map

Nothing in this chapter is typed into a file, and there is nothing to build
yet: `link.ld` does not exist until chapter 04 §4.1, and chapter 01 §1.9 shows
what a build without it does. This chapter is where chapter 04's four `MEMORY`
literals and chapter 08's four peripheral bases come from.

So start from a number you have already seen. This is the last thing the linker
says about the **finished** firmware — chapter 01 §1.8's memory report, as it
arrives in the build log:

```
warning: linker stdout: Memory region         Used Size  Region Size  %age Used
                    FLASH:        6908 B         4 MB      0.16%
                      RAM:          8 KB       520 KB      1.54%
```

That table appears because `.cargo/config.toml` passes `--print-memory-usage`
(chapter 01), and rustc surfaces the linker's stdout as a `linker_messages`
warning. It reports two regions, which the linker knows about only because
`firmware/pico2/link.ld` declares them:

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}
```

Four literals. Chapter 04 uses all four and defends none of them; this chapter
is where they come from — along with the firmware's other magic numbers, the
peripheral bases in `firmware/pico2/src/common/reg.rs`. The tree carries a
`///` doc comment on the enum and on every variant; those are elided here and
the code lines are quoted as they stand:

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

The `#[allow(non_camel_case_types)]` is there because the SCREAMING_SNAKE
variant names would otherwise trip a lint; the comment above it is the tree's
own statement of why the names are worth keeping anyway.

By the end you should be able to say where each of those eight numbers is
written down, and why the two `0x4002_…` values differ by exactly `0x8000` while
`SIO` sits in a different nibble entirely.

## 3.1 Top-level decode on bits 31:28

The chip splits the 4 GB address space by inspecting one nibble: "Rough address
decode is first performed on bits 31:28 of the address" (§2.2, PDF p31). Table 7
(§2.2, PDF p31) is the whole top level:

| Bus Segment | Base Address |
|---|---|
| ROM | `0x00000000` |
| XIP | `0x10000000` |
| SRAM | `0x20000000` |
| APB Peripherals | `0x40000000` |
| AHB Peripherals | `0x50000000` |
| Core-local Peripherals (SIO) | `0xd0000000` |
| Cortex-M33 private registers | `0xe0000000` |

Two properties of this table matter more than the addresses themselves.

**Unmapped ranges fault.** "Unmapped address ranges raise a bus error when
accessed" (§2.2, PDF p31). There is no quiet region to stumble into.

**Not every segment is executable.** APB peripheral registers "are accessible
to processor load/store and DMA only. Instruction fetch will always fail"
(§2.2.4, PDF p32), and the AHB segment says the same (§2.2.5, PDF p34). You can
only fetch instructions from ROM, XIP and SRAM. That is why the linker script
has exactly two regions: those are the only two places your program can live.

Each segment then gets its own sub-table: XIP is Table 9 (§2.2.2, PDF p32), used
in §3.2.4; SRAM is Tables 10 and 11 (§2.2.3, PDF p32), used in §3.3.2; APB is
Table 12 (§2.2.4, PDF p32-p33), used in §3.4. SIO is Table 14 (§2.2.6, PDF p35),
and has only two entries:

| Bus Endpoint | Base Address |
|---|---|
| `SIO_BASE` | `0xd0000000` |
| `SIO_NONSEC_BASE` | `0xd0020000` |

That second entry is not a second SIO. Accesses to `0xd0000000` "are mapped to
the SIO bank which matches the security attribute of the bus access", so Secure
state reaches the Secure bank and Non-secure state the Non-secure bank through
one address; `0xd0020000` is a mirror that lets **Secure** code reach the
Non-secure view deliberately, and "attempting to access this address range from
Non-secure code will generate a bus fault" (datasheet §3.1.1, PDF p39). This
firmware is a Secure image — `picotool info -a` on the built ELF reports
`image type: ARM Secure` — so `RegAddr::SIO = 0xd000_0000` is the Secure bank.

The distinction is not academic for GPIO, because the GPIO registers are one of
the blocks the datasheet lists as **not** duplicated: "The GPIO registers are
shared, and Non-secure accesses are filtered on a per-GPIO basis by the
Non-secure GPIO mask defined in the ACCESSCTRL `GPIO_NSMASK0` and `GPIO_NSMASK1`
registers" (datasheet §3.1.1, PDF p39). Chapter 09 leans on that when it lays
out the SIO offset map.

The PPB is Table 15 (§2.2.7, PDF p35): `PPB_BASE` `0xe0000000`,
`PPB_NONSEC_BASE` `0xe0020000`, `EPPB_BASE` `0xe0080000` — the same `0x20000`
Secure-to-Non-secure offset, deliberately (datasheet §3.1.1, PDF p39, NOTE).
Chapter 06 uses `CPACR` at `0xe000ed88` and `VTOR` at `0xe000ed08` from that
segment.

## 3.2 XIP — where your code actually is

### 3.2.1 The physical situation

**The RP2350 die has no program flash.** It has a 32 kB ROM at `0x00000000`
(§4.1, PDF p338) and 520 kB of SRAM at `0x20000000` (§4.2, PDF p338). That is
the on-chip memory, in full.

Your program lives on a separate chip. The board datasheet names it: the Pico 2
provides "minimal (yet flexible) external circuitry to support the RP2350 chip:
flash (**Winbond W25Q32RV**), crystal (Abracon ABM8-272-T3), power supplies and
decoupling, and USB connector" (Pico 2 datasheet p5), and the headline spec is
"RP2350A microcontroller with **4 MB flash**" (Pico 2 datasheet p4).

That part speaks QSPI: six wires and a serial command protocol. A Cortex-M33
cannot instruction-fetch from a serial protocol.

### 3.2.2 What XIP is

§4.4 (PDF p341-342) states the trick without ceremony: "The term
execute-in-place refers to external memory mapped directly into the chip's
internal address space. This enables you to execute code as-is from the external
memory without explicitly copying into on-chip SRAM. For example, a processor
instruction fetch from AHB address `0x10001234` results in a QSPI memory
interface fetch from address `0x001234` in an external flash device."

The **QMI** (QSPI Memory Interface) watches the bus for accesses in the `0x1…`
segment, synthesises the serial transaction, and returns the bytes as though
they had come from memory. The core never finds out.

Hold on to the contrast: on a conventional microcontroller, flash is genuinely
memory-mapped silicon on the same die, whereas here it is a hardware *emulation*
of memory-mapped flash over a serial link. That is why the read-only behaviour
in §3.2.6 is so odd, and why the address space in §3.2.7 is not storage.

### 3.2.3 The base address is arithmetic

`XIP_BASE = 0x10000000` is the origin of a *window*; nothing is stored there.
Its entire job is the subtraction in that quoted example:

```text
flash device address = XIP address - 0x10000000
```

`0x10001234` becomes device address `0x001234`. So `0x10000000` is byte **0** of
the flash chip — the first byte the bootrom scans for a valid image, which is
why chapter 05's `IMAGE_DEF` block must sit near the start of the region.

### 3.2.4 Four bases, one memory

Table 9 (§2.2.2, PDF p32) lists four bases, and §4.4.1 (PDF p342) says they are
decoded on **bits 27:26**:

| Bus Endpoint | Base Address |
|---|---|
| `XIP_BASE` | `0x10000000` |
| `XIP_NOCACHE_NOALLOC_BASE` | `0x14000000` |
| `XIP_MAINTENANCE_BASE` | `0x18000000` |
| `XIP_NOCACHE_NOALLOC_NOTRANSLATE_BASE` | `0x1c000000` |

These are not four memories. They are four views of the same bytes, and the
address you choose tells the XIP subsystem how to treat the cache and the
address translation on the way through.

Use `0x10000000` for code. `0x18…` is a maintenance window whose writes operate
the cache rather than storing anything, and `0x1c…` bypasses the QMI address
translation the bootrom's image relocation relies on (§5.1.19, PDF p364-365).

### 3.2.5 The cache

The cache "is 16 kB, two-way set-associative, 1 cycle hit" (§4.4.1, PDF p342) —
physically two 8 kB banks "interleaving odd and even cache lines of 8-byte
granularity", so two lines can be reached in one cycle, but "logically, the XIP
cache behaves as a single 16 kB cache" (§4.4, PDF p342). Without it every
instruction fetch would be a serial round trip to another chip; with it, a loop
that fits in 16 kB runs at roughly on-chip speed. It "is internal to the XIP
subsystem […] so software does not have to consider cache coherence unless
performing flash programming operations" (§4.4.1, PDF p342), and this firmware
never programs flash.

### 3.2.6 Writes into `.rodata` are a silent no-op

The XIP window is read-only by default, controlled by `XIP_CTRL.CTRL.WRITABLE_M0`
for window 0 (`0x10000000` through `0x10ffffff`), reset value `0x0` (Table 439,
§4.4.5, PDF p348). Read *how* it is read-only in that same field description:
"Note the read-only behaviour is implemented by downgrading writes to reads, so
writes will still cause allocation of an address, but have no other effect."

> **Silent-failure trap.** A stray pointer write into `.rodata`, or into any
> other flash address, is downgraded to a read. It does not store, and it does
> not fault. No HardFault, no bus error, no diagnostic — the write simply did
> not happen. If you are hunting a constant that "changes" and does not, this is
> not the bug; if you are relying on a fault to catch a wild pointer into flash,
> there will not be one.

The reason for the default is in the same field description (PDF p348): a write
would appear to succeed via the cache, be lost on eviction, and the eviction
would issue a write command that "can break the flash out of its continuous read
mode. After this point, flash reads will return garbage."

### 3.2.7 Why `LENGTH = 4M` and not `16M`

The window and the device are different sizes.

The window is 16 MB: `WRITABLE_M0` names its extent as "addresses `0x10000000`
through `0x10ffffff`, and their uncached mirrors" (Table 439, PDF p348), chip
select 1 starts immediately after at `0x11000000` (`FLASH_DEVINFO.CS1_SIZE`,
PDF p1309), and address translation is "performed separately for each of the
16 MB chip select windows", four 4 MB panes each, defaulting on QMI reset to "a
1:1 identity mapping […] the entire 16 MB address space of the external QSPI
device is mapped directly into the system address space" (§12.14.4, PDF p1232).

The device is 4 MB (Pico 2 datasheet p4). The QMI addresses it with a fixed
24-bit address phase — "Only 24-bit addresses are supported" (§12.14, PDF p1240)
— covering 16 MB of device address space that this device does not have.

So the top 12 MB of the window has no storage behind it, and no bus error either:
a pane faults only "beyond the currently configured `SIZE`" (§12.14.4,
PDF p1232), and after reset every pane is a full 4 MB identity map. The access
goes out on the QSPI wire and the device answers however it answers.

**Inferred:** a 32 Mbit device decodes 22 address bits, so the top two address
bits are surplus and a 4 MB part aliases four times inside the window. That is
the reasoning in `link.ld`'s own comment and it matches the arithmetic, but the
RP2350 datasheet does not state it — that behaviour belongs to the Winbond
datasheet, which this tutorial has not consulted. Nothing below depends on it.

> **Silent-failure trap.** Set `LENGTH = 16M` and the linker will happily place
> code and data above the 4 MB mark, and report a healthy `%age Used`. Nothing
> in the toolchain knows how big the flash chip is. `picotool` will still see a
> valid image, because the boot block is at the bottom. The failure arrives at
> run time, as a jump into whatever the flash device returns for an address it
> does not have. Use the **device** size, not the window size.

## 3.3 SRAM

### 3.3.1 On-chip, and not part of the core

§4.2 (PDF p338): "There is a total of **520 kB** (520 × 1024 bytes) of on-chip
SRAM. For performance reasons, this memory is physically partitioned into ten
banks, but logically it still behaves as a single, flat 520 kB memory."

Eight 64 kB banks plus two 4 kB banks (§4.2, PDF p338): 8 × 64 + 2 × 4 = 520.

It is on the same die as the cores but not *part of* them. Each bank sits behind
the bus fabric as an independently arbitrated slave: "Each SRAM bank is accessed
via a dedicated AHB5 arbiter. This means different bus managers can access
different SRAM banks in parallel, so up to six 32-bit SRAM accesses can take
place every system clock cycle (one per manager)" (§4.2, PDF p338).

The M33 on this chip has no tightly coupled memory to fall back on: `ID_MMFR0`
bits 19:16, "TCM: Indicates support for tightly coupled memories (TCMs)", read
`0x0` (Table 218, datasheet §3.7, PDF p190). The core owns its registers and
nothing else, so every other byte your program touches — including every stack
push — is a bus transaction into one of those ten banks.

### 3.3.2 Layout

Table 10 (§2.2.3, PDF p32), SRAM0-7, "always striped on bits 3:2 of the
address":

| Bus Endpoint | Base Address |
|---|---|
| `SRAM_BASE` | `0x20000000` |
| `SRAM_STRIPED_BASE` | `0x20000000` |
| `SRAM0_BASE` | `0x20000000` |
| `SRAM4_BASE` | `0x20040000` |
| `SRAM_STRIPED_END` | `0x20080000` |

Table 11 (§2.2.3, PDF p32), SRAM8-9, "always non-striped":

| Bus Endpoint | Base Address |
|---|---|
| `SRAM8_BASE` | `0x20080000` |
| `SRAM9_BASE` | `0x20081000` |
| `SRAM_END` | `0x20082000` |

Striping means consecutive words go to different banks — `0x20000000` to bank 0,
`0x20000004` to bank 1, and so on, wrapping to bank 0 at `0x20000010`
(Table 433, §4.2, PDF p339). It is invisible to software.

The arithmetic that licenses a single `RAM` region:

```text
0x20082000 - 0x20000000 = 0x82000 = 532480 = 520 × 1024
```

The striped region ends at `0x20080000` and `SRAM8_BASE` begins at `0x20080000`.
Contiguous, no gap, no alias — RP2350 dropped RP2040's non-striped mirror "to
avoid mapping the same SRAM location as both Secure and Non-secure" (§4.2 NOTE,
PDF p339). One region of 520K is correct.

Two details to file away: the `0x20040000` watermark "marks the boundary between
the SRAM0 and SRAM1 power domains" (§4.2, PDF p338), and the two 4 kB banks at
the top "are useful for hoisting high-bandwidth data structures like the
processor stacks" (§2.2.3, PDF p32) — a hint this firmware does not take.

### 3.3.3 The stack top is deliberately outside RAM

The last assignment in `link.ld`'s `SECTIONS` block, before the assertions:

```ld
_stack_top = ORIGIN(RAM) + LENGTH(RAM);
```

`0x20000000 + 0x82000 = 0x20082000`, and the linker agrees — `llvm-nm -n` on
the release build, filtered to the three absolute symbols:

```
00002000 A _min_stack_size
10001afc A __sidata
20082000 A _stack_top
```

`0x20082000` is `SRAM_END`. It is one past the last valid byte, `0x20081fff`,
and by §2.2 (PDF p31) it is an unmapped address that raises a bus error.

Putting the initial stack pointer at an address that faults is correct only
because the Cortex-M33 stack is **full-descending**: SP is decremented before the
first store, so the first pushed word lands at `_stack_top - 4` and `_stack_top`
itself is never dereferenced. Full-descending is an ARMv8-M architectural
property, not something this datasheet states; the datasheet's contribution is
only that the address above `SRAM_END` is unmapped. Chapter 05 puts this word
into vector table slot 0; chapter 04 shows the `ASSERT` that keeps `.bss` from
growing into the stack.

## 3.4 APB peripherals — a 32 kB grid

Every APB peripheral is allocated a **32 kB (`0x8000`) window**, whether it needs
one or not. `RESETS` has three registers and uses `0xc` bytes of its 32 kB
(Table 533, PDF p504); `IO_BANK0` uses a fraction of a kilobyte. The grid
is an address-decode convenience, not a statement about register counts.

From Table 12 (§2.2.4, PDF p32-p33) — the entries this tutorial refers to, in the
datasheet's order:

| Bus Endpoint | Base Address |
|---|---|
| `SYSINFO_BASE` | `0x40000000` |
| `SYSCFG_BASE` | `0x40008000` |
| `CLOCKS_BASE` | `0x40010000` |
| `PSM_BASE` | `0x40018000` |
| `RESETS_BASE` | `0x40020000` |
| `IO_BANK0_BASE` | `0x40028000` |
| `IO_QSPI_BASE` | `0x40030000` |
| `PADS_BANK0_BASE` | `0x40038000` |
| `PADS_QSPI_BASE` | `0x40040000` |
| `XOSC_BASE` | `0x40048000` |
| `PLL_SYS_BASE` | `0x40050000` |
| `UART0_BASE` | `0x40070000` |
| `UART1_BASE` | `0x40078000` |
| `SPI0_BASE` | `0x40080000` |
| `I2C0_BASE` | `0x40090000` |
| `PWM_BASE` | `0x400a8000` |
| `TIMER0_BASE` | `0x400b0000` |

The full table continues to `TBMAN_BASE` at `0x40160000` (PDF p33-p34); nothing
this firmware touches lives above `PADS_BANK0`.

Read the ordering. Peripherals are grouped **by layer, not by bank**: the two IO
muxes are adjacent (`IO_BANK0`, then `IO_QSPI`), then the two pad blocks
(`PADS_BANK0`, then `PADS_QSPI`). Bank 0 is the user GPIOs; QSPI is the six pins
wired to the flash chip of §3.2.1. That adjacency is why a one-nibble typo is a
bad day: `0x40030000` instead of `0x40028000` still points at a live, writable
mux — the one driving your flash.

> **Hardware-destructive.** `IO_QSPI` at `0x40030000` and `PADS_QSPI` at
> `0x40040000` control the six pins the RP2350 uses to fetch every instruction
> it executes. Reconfiguring them from code that is itself running out of XIP
> stops the instruction stream mid-write. The same goes for the `RESETS` bits
> holding those two blocks, which sit immediately beside the two this firmware
> clears; chapter 08 returns to them.

The four bases in `RegAddr`, checked against this table and against Table 14:

| `RegAddr` variant | Value | Datasheet name | Source |
|---|---|---|---|
| `RESET` | `0x4002_0000` | `RESETS_BASE` | Table 12, PDF p33 |
| `IO_BANK0` | `0x4002_8000` | `IO_BANK0_BASE` | Table 12, PDF p33 |
| `PADS_BANK0` | `0x4003_8000` | `PADS_BANK0_BASE` | Table 12, PDF p33 |
| `SIO` | `0xd000_0000` | `SIO_BASE` | Table 14, PDF p35 |

All four are correct. Note that the enum names the first one `RESET` while the
datasheet block is `RESETS` — a cosmetic mismatch that costs a search when you
grep the datasheet for it. Chapter 08 uses all four to bring up GP25; chapter 09
holds the offset maps inside each block.

## 3.5 The resulting MEMORY block

```ld
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x10000000, LENGTH = 4M
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 520K
}
```

Four numbers, each traceable:

| Value | Provenance |
|---|---|
| `0x10000000` | `XIP_BASE`, the **cached** mirror — Table 9 (§2.2.2, PDF p32). The other three bases in §3.2.4 are the same bytes with different cache and translation behaviour. |
| `4M` | The **device** size: Winbond W25Q32RV, 4 MB (Pico 2 datasheet p4, p5). Not the 16 MB chip-select-0 window (Table 439, PDF p348) — see §3.2.7. |
| `0x20000000` | `SRAM_BASE` — Table 7 (§2.2, PDF p31) and Table 10 (§2.2.3, PDF p32). |
| `520K` | §4.2 (PDF p338), confirmed by `SRAM_END - SRAM_BASE = 0x82000` from Table 11 (§2.2.3, PDF p32). |

The `(rx)` and `(rwx)` attributes are absent from that table because they are not
hardware facts. They are advisory: they enforce nothing, and only steer where
unclaimed sections land. Chapter 04 takes that apart along with the rest of the
script — `ENTRY`, the section order, `AT >` for VMA (virtual memory address, the
address a section runs at) versus LMA (load memory address, where its bytes are
stored), and the five `ASSERT`s that turn this chapter's constraints into link
errors.
