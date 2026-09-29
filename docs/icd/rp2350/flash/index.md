# Flash Write: Bootrom Flash API, XIP and QMI (Sections 4.4, 5.4, 5.5, 12.14)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_bootrom_api.md`](01_bootrom_api.md) | Locating ROM functions, the six low-level flash functions, `flash_op`, return codes, boot locks |
| [`02_programming.md`](02_programming.md) | Erase and program sequence, the window with XIP off, UF2 downloads and RP2350-E10, time budget |
| [`03_xip_qmi.md`](03_xip_qmi.md) | XIP address windows and cache, QMI direct mode, the boot XIP read mode, the registers the sequence touches |

## Source Pages

Page numbers are the printed page numbers (page footer) of `../../rp2350-datasheet.pdf`,
build 2025-02-20 (PDF page = printed page + 1 in this build).

| Datasheet section | Printed pages | Content used here |
|-------------------|---------------|-------------------|
| 4.4 External Flash and PSRAM (XIP) | 340-351 | XIP windows, 16 kB cache, coherence rule |
| 5.2.7 Flash Boot, Table 451 | 372-373 | Read modes and SCK divisors the bootrom tries |
| 5.4.1 to 5.4.4 Bootrom APIs | 376-379 | ROM table, availability, return codes, boot locks |
| 5.4.6 to 5.4.8 API listings | 380-398 | `connect_internal_flash` to `flash_select_xip_read_mode` |
| 5.5.2 UF2 Format Details | 399-400 | Sector-granular erase of UF2 downloads |
| 12.14.4 to 12.14.6 QMI | 1231-1246 | Address translation, direct mode, register list |
| Appendix E, RP2350-E10 | 1350-1351 | Absolute block at the end of flash |

## Base Addresses

| Block | Symbol | Address | Source |
|-------|--------|---------|--------|
| Cached XIP window | `XIP_BASE` | `0x10000000` | §4.4.1 (p341) |
| Uncached XIP window | `XIP_NOCACHE_BASE` | `0x14000000` | §4.4.1 (p341) |
| QMI registers | `XIP_QMI_BASE` | `0x400d0000` | §12.14.6 (p1233) |
| ROM table pointers | - | `0x00000014` to `0x00000019` | §5.4.1 Table 452 (p376) |

## Key Specifications

| Property | Value | Source |
|----------|-------|--------|
| Erase unit | 4096-byte sector; address and count multiples of 4096 | §5.4.8.10 (p387) |
| Program unit | 256-byte page; address and count multiples of 256 | §5.4.8.11 (p387) |
| Argument checks | none in `flash_range_erase` / `flash_range_program`; `flash_op` checks alignment, bounds and partitions | §5.4.8.9 to 5.4.8.11 (p386-387) |
| XIP during an operation | QMI in direct mode; any XIP access from a core, DMA or the debugger returns a bus fault | §5.4.8.9 (p387), §12.14.5 (p1232) |
| XIP after an operation | basic 03h serial XIP at CLKDIV 12 until the saved XIP setup is re-run | §5.4.8.6, 5.4.8.7 (p385), §5.4.8.10 (p387) |
| Cache coherence | only a concern after programming or erasing: flush with `flash_flush_cache` | §4.4.1 (p341), §5.4.8.8 (p386) |
| Availability | all six low-level functions are Arm Secure and RISC-V only | §5.4.6.1 (p380) |
| Default flash size seen by the bootrom | 16 MB on CS0 unless OTP `FLASH_DEVINFO` is enabled | §5.4.8.5 (p385) |
| Pico 2 device | Winbond W25Q32RV, 4 MB | Pico 2 datasheet ch. 1 (p4) |

## cwht Driver Notes (WP-SW-08)

- WP-SW-08 is the `FlashStore` work package of cwht 07 section 19 (erase sector,
  program page, read) for the configuration sectors A and B and the event log.
  Both users write under the same conditions (Receive or Fault-safe, PA_EN and
  TX_KEY low, key inputs open), one declared scheduler window at a time. See
  [`02_programming.md`](02_programming.md) "cwht window design".
- The flash driver holds no watchdog handle: only the cwht main loop kicks the
  watchdog, after every monitor task has run.
  This chapter is its DML-3 register ICD; the analytical proof and the time
  budget are in cwht `docs/design/analysis/fw-b1-dml3-wp-sw-08-10-12.md`
  section 4. The driver itself is FW-B2 work.
- Only core 0 runs rustos code (core 1 stays in the bootrom wait state), so
  masking interrupts on core 0 stops every flash fetch except the debugger's.
- The whole sequence from `connect_internal_flash` to the re-run XIP setup
  runs from SRAM (`.data.ramfunc`, copied by `reset_data()`), with PRIMASK set,
  calling only ROM functions resolved before the window. See
  [`02_programming.md`](02_programming.md).
- cwht keeps its store and its event log out of the last sector of the 4 MB
  device (RP2350-E10, [`02_programming.md`](02_programming.md) "UF2 downloads").

## Cross-References

- Critical section (PRIMASK): cwht WP-SW-09, `pico2::irq`.
- Watchdog kick by the main loop before and after the window: cwht WP-SW-07 (`watchdog/`, not yet extracted).
- CRC-32 of each record: cwht `cwht-core` (WP-SW-12), not a rustos item.
