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
| The flash driver never touches the watchdog. The window starts only right after a watchdog kick given by the application's main loop, and the longest time without a kick (the rest of that pass, the window, the next full pass) is at most half the watchdog load | the watchdog keeps counting while the core is in ROM; a kick from the driver would let a runaway caller keep the chip alive | cwht keyer host study §6.5, R-1 |
| Offsets are checked by the caller: sector or page aligned, and only inside the store sectors | the range functions do no checks | p387 |
| After step 5, re-apply any QMI timing the application set after boot (for example a faster `M0_TIMING.CLKDIV`) | step 5 restores the boot-time mode, not the application's | p388, p1239 |

A fault inside the window cannot run its handler (the handler is in flash),
so the core locks up; the running watchdog then resets the chip within its
load time of the last main-loop kick. This is the bounded outcome the driver
design relies on.

## cwht window design (WP-SW-08)

One RAM-resident function per operation, entered with the arguments already
checked by a pure decision function (`plan`, host-tested). The cwht scheduler
(`SW-SCHED`) declares each window as a state with a bound, so the stall is
neither a tick overrun nor a missed monitor deadline while it stays inside the
bound:

| Step | Action | Where it runs |
|------|--------|---------------|
| 0 | `plan(op, offset, len, conditions)`: reject anything that is not a whole sector (erase) or whole page (program) of a configuration or event-log sector, or that fails the write conditions below | flash, host-tested |
| 1 | The main-loop pass dispatches every due task; the kick decision finds every monitor on time and kicks the watchdog | flash, cwht main loop |
| 2 | In the same pass, the scheduler enters its declared store-window state (start time from TIMER0; bound 10 ms for a program window, 450 ms for an erase window); resolve `IF`, `EX`, `RE`/`RP`, `FC`, `XF`; copy the XIP setup function to an SRAM buffer; set PRIMASK | flash (then jump to SRAM) |
| 3 | `connect_internal_flash`, `flash_exit_xip`, the range function, `flash_flush_cache`, the SRAM copy of the XIP setup | SRAM and ROM only |
| 4 | Re-apply the application's QMI timing; clear PRIMASK | SRAM, then flash |
| 5 | The scheduler leaves the store-window state and reads TIMER0 (it counts through PRIMASK). Over the bound: a tick overrun, safe state. Within it: the missed ticks are counted as a declared stall, not as overruns | flash, cwht scheduler |
| 6 | A catch-up pass runs every monitor task once on fresh samples, then the kick decision | flash, cwht main loop |
| 7 | Later, as an ordinary step: read back through XIP and compare (program), or check for 0xFF (erase) | flash |

Write conditions (cwht, every `FlashStore` user, configuration and event
log): the mode is Receive or Fault-safe; PA_EN and TX_KEY are low and the key
inputs read open; the pass that ends in step 1 took a PA temperature sample;
at least 1 s has passed since the last window. Receive audio is muted, charge
enable is de-asserted and transmit is disarmed before step 2 and restored
after step 6.

Records are appended: each 256-byte record goes into the next blank page of
its sector (16 pages per sector), so almost every write is a program window.
A sector is erased only to reclaim it, when the other sector of its ring is
full and it holds only older records. Erase and program are always separate
windows.

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
  last sector. cwht places its event log in an 8-sector reserve at offsets
  `0x3F5000` to `0x3FCFFF`, configuration sector A at `0x3FD000`, sector B at
  `0x3FE000`, and leaves `0x3FF000` unused. The application `FLASH` region
  ends at `0x3F5000` (44 KiB below the device end).

## Time budget (cwht WP-SW-08, DML-3)

Part timing is not in the RP2350 datasheet. The values below are cwht
assumption A-7 (Winbond W25Q-family maxima, the W25Q32RV datasheet is not in
the repository) until the DML-5 dev-board measurement; they are marked (A).

| Item | Value | Source |
|------|-------|--------|
| 4 kB sector erase, maximum | 400 ms (A) | cwht A-7 |
| 256-byte page program, maximum | 3 ms (A) | cwht A-7 |
| Window overhead (connect, exit XIP, flush, XIP setup) | at most 1 ms (A) | cwht A-F2 |
| Program window | 4 ms | sum |
| Erase window | 401 ms | sum |
| Declared window bounds (cwht scheduler) | 10 ms program, 450 ms erase | cwht proposal |
| One main-loop pass with every task due | at most 1 ms (A) | cwht A-F8 |
| Longest time without a watchdog kick: rest of the pass, erase window, catch-up pass | 403 ms; 452 ms at the erase bound | sum |
| Watchdog load (cwht) | 1.0 s: 2.48 times 403 ms, 2.21 times 452 ms | cwht keyer host study §6.5 |
| Largest erase bound that keeps the factor 2 | 498 ms | 1.0 s / 2 - 2 x 1 ms |
| Largest sector erase before the bound declares an overrun | 449 ms | 450 ms - 1 ms |
| Read-back of one 256-byte record after the flush | under 1 ms | 03h at CLKDIV 12 worst case, [`03_xip_qmi.md`](03_xip_qmi.md) |

The figures are recomputed by cwht
`docs/design/analysis/fw-b1-dml3/check_fw_b1_dml3.py`.
