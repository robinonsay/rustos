# Flash Write: Bootrom Flash API

[Back to flash index](index.md) | [Back to ICD index](../index.md)

## 5.4.1 Locating the functions (p376-377)

Bootrom function addresses change between bootrom releases, so they are looked
up at run time through fixed words at the bottom of ROM (Table 452, Arm):

| Address | Contents |
|---------|----------|
| `0x00000010` | Magic `'M'`, `'u'`, `0x02` (3 bytes) |
| `0x00000013` | Bootrom version byte (2 on A2 silicon; informational only) |
| `0x00000014` | 16-bit pointer to the ROM entry table |
| `0x00000016` | 16-bit pointer to `rom_table_lookup_val()` |
| `0x00000018` | 16-bit pointer to `rom_table_lookup_entry()` |

The other fields are valid only if the magic reads `'M'`, `'u'`, `0x02`. On
Arm the function pointer is stored in the table, so the lookup used is the
one at `0x16`, called with `(code, RT_FLAG_FUNC_ARM_SEC)` from Secure code. The
code is `(c2 << 8) | c1` for the two-character code of each function.

## 5.4.2 Availability (p377-378)

The six low-level flash functions are available to Arm Secure code and RISC-V
only (§5.4.6.1, p380). rustos images are `ARM Secure` (IMAGE_DEF), so the
calls are made directly; no Non-secure permission (`set_ns_api_permission`)
is involved.

## 5.4.8 The functions the driver uses (p385-388)

| Code | Function | Signature | Effect |
|------|----------|-----------|--------|
| `'I','F'` | `connect_internal_flash` | `void (void)` | Restores the QSPI pad controls to default and connects QMI to the QSPI pads (p384) |
| `'E','X'` | `flash_exit_xip` | `void (void)` | Initialises QMI for direct mode, sets a basic 03h XIP mode at CLKDIV 12, and returns the device from continuous-read or QPI mode to serial-command state (p385) |
| `'R','E'` | `flash_range_erase` | `void (uint32_t addr, size_t count, uint32_t block_size, uint8_t block_cmd)` | Erases `count` bytes from flash offset `addr`; both multiples of 4096; optionally uses a larger block command where it fits (p387) |
| `'R','P'` | `flash_range_program` | `void (uint32_t addr, const uint8_t *data, size_t count)` | Programs `count` bytes at flash offset `addr`; both multiples of 256 (p387) |
| `'F','C'` | `flash_flush_cache` | `void (void)` | Invalidates every XIP cache line so later cached reads see the new data; also unpins pinned lines (p386) |
| `'X','F'` | `xip_setup_func_ptr` | data: pointer to the XIP setup function in boot RAM | The function that restores the XIP mode found at boot; copied to SRAM and called after the operation (p388, p398) |
| `'X','M'` | `flash_select_xip_read_mode` | `void (bootrom_xip_mode_t mode, uint8_t clkdiv)` | Sets one of four XIP read modes (0 = 03h, 1 = 0Bh, 2 = BBh, 3 = EBh) for both windows; the divisor also applies to direct mode (p388) |

`addr` for the two range functions is an offset from the start of flash (a
storage address), not an XIP address: offset `0x3FD000` is XIP address
`0x103FD000`.

Neither range function validates its arguments. An unaligned or out-of-range
request is not rejected by the bootrom; the caller must check it.

### `flash_op` (p386-387)

`int flash_op(uint32_t flags, uint32_t addr, uint32_t size_bytes, uint8_t *buf)`
(`'F','O'`) wraps the same operations with alignment checks
(`BOOTROM_ERROR_BAD_ALIGNMENT`), bounds checks against the boot RAM copy of
`FLASH_DEVINFO` (16 MB on CS0 by default, p385), partition permission checks
and optional runtime-to-storage translation. It does not add a timeout and
does not change the XIP-off window. The cwht driver uses the two range
functions with its own checks (a host-tested decision function), so that the
accepted ranges are exactly the store sectors and not the whole 16 MB default.

## 5.4.3 Return codes (p378)

The range functions and `flash_flush_cache` are `void`: the bootrom reports
no error for an erase or program. The driver verifies every operation by
reading back through XIP after the flush (cwht 07 section 14.2 row g). The
`int` functions return `BOOTROM_OK` (0) or a negative code, among them
`BOOTROM_ERROR_NOT_PERMITTED` (-4), `BOOTROM_ERROR_INVALID_ADDRESS` (-10),
`BOOTROM_ERROR_BAD_ALIGNMENT` (-11) and `BOOTROM_ERROR_LOCK_REQUIRED` (-19).

## 5.4.4 Boot locks (p379)

Boot lock 1 (`LOCK_FLASH_OP`) guards direct-mode flash use, but lock checking
is enabled only when boot lock 7 (`LOCK_ENABLE`) is claimed, and it is off by
default. rustos does not claim `LOCK_ENABLE`, so no lock is taken before the
calls. If a later design enables lock checking, lock 1 must be claimed (SIO
spinlock procedure, §3.1.4) before the window, or the bootrom returns
`BOOTROM_ERROR_LOCK_REQUIRED` from the functions that check it.
