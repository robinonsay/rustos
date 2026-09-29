# Flash Write: Programming Model

[Back to flash index](index.md) | [Back to ICD index](../index.md)

## Datasheet sequence (§5.4.8.9 to 5.4.8.11, p386-387)

For an erase from Secure code the datasheet gives this order; a program
operation is the same with `flash_range_program` in step 3:

1. `connect_internal_flash()`
2. `flash_exit_xip()`
3. `flash_range_erase(offset, 4096 * n, block_size, block_cmd)` (or `flash_op`)
4. `flash_flush_cache()`
5. Copy the XIP setup function from boot RAM (`xip_setup_func_ptr`) into SRAM
   and call it, to restore the XIP mode found at boot.

From step 2 until the range function returns, QMI is in direct mode (§12.14.5,
p1232): any XIP access from either core, DMA or the debugger returns a bus
fault. The RP2350 functions leave the device in a basic 03h serial XIP state
(CLKDIV 12) between operations (p385, p387), so reads work after step 3, but
slowly, and step 4 is needed before cached reads see the new data (§4.4.1,
p341).

## Constraints that follow for the caller

| Constraint | Why | Source |
|------------|-----|--------|
| Every instruction from step 1 to step 5 is fetched from SRAM or ROM | an XIP fetch in the window is a bus fault | p387, p1232 |
| Every data access in the window is to SRAM, ROM or peripherals: no `.rodata`, no `const` table, no literal pool in flash | same | p387, p1232 |
| The ROM function pointers and the XIP setup copy are resolved and made before step 1 | the lookup code of the caller lives in flash | p376-377 |
| Interrupts masked on core 0 (PRIMASK) for the whole window | the vector table and the handlers are in flash | rustos `link.ld`; p387 |
| `NMI_MASK0` and `NMI_MASK1` stay 0 (their reset value) | an NMI ignores PRIMASK; the only other NMI source is a failed RCP integrity check | §3.2.1 (p83), Table 362 (p232) |
| Core 1 not running (it stays in the bootrom wait state) | it would fetch from XIP | p387 |
| No DMA channel reads or writes the XIP windows during the window | DMA XIP access faults | p387 |
| No debugger memory access to XIP during the window | it returns a bus fault | p387 |
| The watchdog is fed immediately before masking, and the window is shorter than half the watchdog load | the watchdog keeps counting while the core is in ROM | cwht keyer host study §6.5 |
| Offsets are checked by the caller: sector or page aligned, and only inside the store sectors | the range functions do no checks | p387 |
| After step 5, re-apply any QMI timing the application set after boot (for example a faster `M0_TIMING.CLKDIV`) | step 5 restores the boot-time mode, not the application's | p388, p1239 |

A fault inside the window cannot run its handler (the handler is in flash),
so the core locks up; the running watchdog then resets the chip within its
load time. This is the bounded outcome the driver design relies on.

## cwht window design (WP-SW-08)

One RAM-resident function per operation, entered with the arguments already
checked by a pure decision function (`plan`, host-tested):

| Step | Action | Where it runs |
|------|--------|---------------|
| 0 | `plan(op, offset, len)`: reject anything that is not a whole sector (erase) or whole page (program) inside copy A or copy B | flash, host-tested |
| 1 | Resolve `IF`, `EX`, `RE`/`RP`, `FC`, `XF`; copy the XIP setup function to an SRAM buffer | flash |
| 2 | Feed the watchdog; set PRIMASK | flash (then jump to SRAM) |
| 3 | `connect_internal_flash`, `flash_exit_xip`, the range function, `flash_flush_cache`, the SRAM copy of the XIP setup | SRAM and ROM only |
| 4 | Re-apply the application's QMI timing; clear PRIMASK | SRAM, then flash |
| 5 | Read back through XIP and compare (program) or check for 0xFF (erase) | flash |

Erase and program are separate windows, so the longest window is one sector
erase. The record is one 256-byte page, so each commit is one erase window and
one program window.

## UF2 downloads and RP2350-E10 (§5.5.2, p399-400; Appendix E, p1350-1351)

- A UF2 download always erases flash a whole 4 kB sector at a time: pages of a
  sector the UF2 does not carry are left erased (p400). Any sector a UF2 block
  touches loses its old contents.
- The RP2350-E10 workaround (A2 silicon: drag-and-drop fails with a partition
  table) adds one absolute-family block at the start of the UF2 that targets
  the end of flash and is written first (p1351). picotool `--abs-block`
  places it at `0x10ffff00` (cwht research F5).
- On the 4 MB Pico 2 device that address is offset `0xFFFF00`, inside the
  16 MB default CS0 size (p385), so the bootrom writes it. If the device
  decodes 22 address bits (4 MB; the part datasheet is not in this
  repository, cwht assumption A-F3), the block lands at `0x3FFF00`, and the
  sector erase clears the whole last sector `0x3FF000`.
- Consequence: nothing that must survive a firmware update may live in the
  last sector. cwht places copy A at offset `0x3FD000`, copy B at `0x3FE000`
  and leaves `0x3FF000` unused. The application `FLASH` region ends at
  `0x3FD000` (12 KiB below the device end).

## Time budget (cwht WP-SW-08, DML-3)

Part timing is not in the RP2350 datasheet. The values below are cwht
assumption A-7 (Winbond W25Q-family maxima, the W25Q32RV datasheet is not in
the repository) until the DML-5 dev-board measurement; they are marked (A).

| Item | Value | Source |
|------|-------|--------|
| 4 kB sector erase, maximum | 400 ms (A) | cwht A-7 |
| 256-byte page program, maximum | 3 ms (A) | cwht A-7 |
| Window overhead (connect, exit XIP, flush, XIP setup) | at most 1 ms (A) | cwht A-F2 |
| Longest masked window: erase | 401 ms | sum |
| Longest masked window if erase and program share one window | 404 ms | sum |
| Watchdog load (cwht) | 1.0 s: 2.48 times the 404 ms window | cwht keyer host study §6.5 |
| Largest sector erase that keeps the factor 2 | 496 ms | 1.0 s / 2 - 3 ms - 1 ms |
| Read-back of one 256-byte record after the flush | under 1 ms | 03h at CLKDIV 12 worst case, [`03_xip_qmi.md`](03_xip_qmi.md) |

The figures are recomputed by cwht
`docs/design/analysis/fw-b1-dml3/check_fw_b1_dml3.py`.
