# Flash Write: XIP Windows, Cache and QMI

[Back to flash index](index.md) | [Back to ICD index](../index.md)

## 4.4 XIP address windows (p341)

An XIP fetch from `0x10001234` reads device address `0x001234`. The 26-bit
XIP space is mirrored on bits 27:26 of the system address:

| Prefix | Access |
|--------|--------|
| `0x10...` | Cached XIP |
| `0x14...` | Uncached XIP |
| `0x18...` | Cache maintenance writes |
| `0x1c...` | Uncached, untranslated (bypasses QMI address translation) |

The lower half of the space holds two 16 MB windows, one per QMI chip select.
The Pico 2 has one device, on CS0.

## 4.4.1 XIP cache (p341)

- 16 kB, two-way set-associative, two 8 kB banks interleaving 8-byte lines.
- Software need not consider coherence except around flash programming:
  after an erase or program, `flash_flush_cache()` (p386) invalidates every
  line. It also unpins pinned lines, so cache-as-SRAM must not be in use.
- Address translation (`ATRANS0` to `ATRANS7`) sits downstream of the cache;
  a translation change also needs a flush (§12.14.4.2, p1232). The Pico 2
  image is unpartitioned, so translation is the identity map.

## 5.2.7 XIP read mode after boot (Table 451, p372-373)

The bootrom tries up to 16 read-mode and SCK-divisor pairs, starting with EBh
quad at divisor 3, then BBh dual, 0Bh serial and 03h serial at 3, and the
same four at 6, 12 and 24. The first pair that finds a valid image is kept,
and its setup function is written to boot RAM (p388). A device that answers
EBh quad is left at EBh quad, divisor 3.

All bootrom XIP modes send an 8-bit serial command prefix before each access
(p388). Miss cost per 8-byte cache line, in SCK cycles:

| Mode | Command | Address | Wait | Data (64 bits) | Total SCK | `clk_sys` cycles at the divisor | Cycles per byte |
|------|---------|---------|------|----------------|-----------|------------------------------|-----------------|
| EBh quad, divisor 3 (after boot) | 8 | 6 | 6 | 16 | 36 | 108 | 13.5 |
| EBh quad, divisor 2 (cwht C4 proposal) | 8 | 6 | 6 | 16 | 36 | 72 | 9 |
| 03h serial, divisor 12 (after `flash_exit_xip`) | 8 | 24 | 0 | 64 | 96 | 1152 | 144 |

Chip-select and turnaround cycles are not included; the DML-5 dev-board check
measures them.

## 12.14.5 Direct mode (p1232-1233)

In direct mode the XIP window is disconnected from the QSPI bus and the bus is
driven through the `DIRECT_TX` / `DIRECT_RX` FIFO pair; an XIP access then
returns a bus fault. The bootrom range functions enter and leave direct mode
themselves; the rustos driver never writes these registers. They are listed
so that a register trace or a debugger view can be read.

| Offset | Register | Fields the bootrom uses |
|--------|----------|-------------------------|
| `0x00` | `DIRECT_CSR` | `EN` (bit 0, enable direct mode), `BUSY` (bit 1), `ASSERT_CS0N` (bit 2), `AUTO_CS0N` (bit 6), `TXFULL` (10), `TXEMPTY` (11), `RXEMPTY` (16), `RXFULL` (17), `CLKDIV` (29:22, reset 6), `RXDELAY` (31:30) (Table 1293, p1234-1236) |
| `0x04` | `DIRECT_TX` | data in bits 15:0 with `NOPUSH`, `DWIDTH`, `IWIDTH`, `OE` controls (§12.14.5.1, p1233) |
| `0x08` | `DIRECT_RX` | received data |
| `0x0c` | `M0_TIMING` | `CLKDIV` bits 7:0 (reset 4): SCK period in `clk_sys` cycles for window 0; may change on the fly (p1237-1239) |
| `0x10`, `0x14` | `M0_RFMT`, `M0_RCMD` | read format and command of window 0 |
| `0x34` to `0x50` | `ATRANS0` to `ATRANS7` | address translation, 4 MiB each (Table 1292, p1233-1234) |

`ASSERT_CS0N` drives the chip select low even with `EN` clear (p1233), so no
code other than the bootrom functions writes `DIRECT_CSR`.

## What changes for the application after a flash operation

The saved XIP setup function (p388) restores the boot-time mode (for the
Pico 2, EBh quad at divisor 3). If the application later programs its own
`M0_TIMING.CLKDIV` (cwht proposal C4: divisor 2 at `clk_sys` 96 MHz, 48 MHz
SCK), it must write it again after each operation, inside the window, before
PRIMASK is cleared.
