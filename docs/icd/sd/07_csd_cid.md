# 07 — CID and CSD Registers

[← Back to index](index.md)

> **Source note:** Authored from public SD SPI-mode protocol knowledge.
> The SD Physical Layer Simplified Specification PDF is not present in
> this repository.

## 7.1 Overview

Two card-resident registers carry identification and capacity metadata
that the host can read at init time:

| Register | Length | Read by | Used for |
|----------|--------|---------|----------|
| CID (Card IDentification) | 16 bytes | CMD10 | Manufacturer / OEM / serial logging |
| CSD (Card Specific Data) | 16 bytes | CMD9 | Capacity computation, version detection, max clock rate |

Both registers are returned in SPI mode as a **data block**: an R1 byte
followed by the 0xFE start token followed by 16 bytes of payload
followed by 2 bytes of CRC. The framing is identical to a CMD17 read
except the payload is 16 bytes instead of 512.

## 7.2 CID — Card Identification (CMD10)

### 7.2.1 Wire Format

```
Host: <CMD10> 0xFF ... 0xFF 0xFF ... 0xFF 0xFF 0xFF
Card:         <R1>  0xFF ... 0xFE <16 bytes CID> <CRC16>
```

### 7.2.2 CID Layout (16 bytes, MSB first on the wire)

| Byte offset | Bits (in 128-bit register) | Width | Field |
|-------------|----------------------------|-------|-------|
| 0           | 127..120 | 8 | MID — Manufacturer ID |
| 1–2         | 119..104 | 16 | OID — OEM / Application ID (two ASCII characters) |
| 3–7         | 103..64  | 40 | PNM — Product Name (5 ASCII characters) |
| 8           | 63..56   | 8 | PRV — Product Revision (BCD: upper 4 bits = major, lower 4 bits = minor) |
| 9–12        | 55..24   | 32 | PSN — Product Serial Number |
| 13          | 23..20   | 4 | (reserved) |
| 13–14       | 19..8    | 12 | MDT — Manufacturing Date (4-bit year offset from 2000 in upper nibble of byte 13's lower 4 bits + byte 14 upper 4 bits; 4-bit month in byte 14 lower 4 bits) |
| 15          | 7..1     | 7 | CRC7 of the CID |
| 15          | 0        | 1 | Always 1 (end bit) |

### 7.2.3 Field Notes

- **MID, OID, PNM** are vendor strings; record them in the boot log for
  forensic value but do not branch firmware behavior on their values.
- **PSN** is unique per card per vendor; the FSW shall log it on every
  boot so post-flight analysis can confirm which card flew.
- **MDT** allows the host to detect very old cards; the FSW does not
  enforce a date check.
- The CRC7 in byte 15 covers the first 15 bytes of the register and is
  validated by the **card** when written at the factory; the host
  ignores it.

### 7.2.4 Example (illustrative only — exact bytes vary by vendor)

```
Byte: 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F
Hex:  03 53 44 53 44 30 35 47 80 12 34 56 78 01 4E 0F
      \  \____/ \____________/ \  \____________/ \__/
       \  OID    PNM = "SD05G"  \  PSN = 0x12345678  MDT (year=2024,
        MID="SanDisk"            PRV=8.0                  month=April)
```

## 7.3 CSD — Card Specific Data (CMD9)

### 7.3.1 Wire Format

```
Host: <CMD9> 0xFF ... 0xFF 0xFF ... 0xFF 0xFF 0xFF
Card:         <R1>  0xFF ... 0xFE <16 bytes CSD> <CRC16>
```

### 7.3.2 CSD Versions

The first 2 bits of the CSD register identify its layout version:

| Bits 127..126 | CSD Structure | Card class | Layout |
|---------------|---------------|------------|--------|
| 0b00 | 1.0 | SDSC | "v1" layout (capacity from C_SIZE, C_SIZE_MULT, READ_BL_LEN) |
| 0b01 | 2.0 | SDHC, SDXC | "v2" layout (capacity from a single 22-bit C_SIZE field) |
| 0b10 | 3.0 | SDUC | "v3" layout — out of scope for FT1 |

For FT1, the driver should encounter only CSD v2.0. If it sees v1.0
that means the card is SDSC; the driver may compute the capacity using
the v1 formula or, more simply, treat any CSD v1.0 card as "small
enough" and just use the card without recomputing capacity.

### 7.3.3 CSD v2.0 Field Map (SDHC / SDXC)

| Byte offset | Bits in register | Width | Field |
|-------------|------------------|-------|-------|
| 0 | 127..126 | 2 | CSD_STRUCTURE = 0b01 |
| 0 | 125..120 | 6 | (reserved) |
| 1 | 119..112 | 8 | TAAC — data read access-time (fixed at 0x0E for v2 cards) |
| 2 | 111..104 | 8 | NSAC — data read access-time in CLK cycles (fixed at 0x00) |
| 3 | 103..96 | 8 | TRAN_SPEED — max clock (0x32 = 25 MHz; 0x5A = 50 MHz UHS) |
| 4–5 | 95..84 | 12 | CCC — Card Command Classes |
| 5 | 83..80 | 4 | READ_BL_LEN — max read data block length (fixed at 9 = 512 bytes for v2) |
| 6 | 79 | 1 | READ_BL_PARTIAL = 0 (v2 forces full-block reads) |
| 6 | 78 | 1 | WRITE_BLK_MISALIGN |
| 6 | 77 | 1 | READ_BLK_MISALIGN |
| 6 | 76 | 1 | DSR_IMP |
| 6 | 75..70 | 6 | (reserved) |
| 7–9 | 69..48 | 22 | **C_SIZE** — capacity (in 512 KB units) |
| 9 | 47 | 1 | (reserved) |
| 10 | 46 | 1 | ERASE_BLK_EN |
| 10 | 45..39 | 7 | SECTOR_SIZE |
| 11 | 38..32 | 7 | WP_GRP_SIZE |
| 12 | 31 | 1 | WP_GRP_ENABLE |
| 12 | 30..29 | 2 | (reserved) |
| 12 | 28..26 | 3 | R2W_FACTOR |
| 12–13 | 25..22 | 4 | WRITE_BL_LEN |
| 13 | 21 | 1 | WRITE_BL_PARTIAL |
| 13 | 20..16 | 5 | (reserved) |
| 14 | 15 | 1 | FILE_FORMAT_GRP |
| 14 | 14 | 1 | COPY |
| 14 | 13 | 1 | PERM_WRITE_PROTECT |
| 14 | 12 | 1 | TMP_WRITE_PROTECT |
| 14 | 11..10 | 2 | FILE_FORMAT |
| 14 | 9..8 | 2 | (reserved) |
| 15 | 7..1 | 7 | CRC7 |
| 15 | 0 | 1 | Always 1 (end bit) |

### 7.3.4 Capacity Calculation (CSD v2)

The 22-bit `C_SIZE` field straddles bytes 7..9 of the CSD:

```
C_SIZE = ((CSD[7] & 0x3F) << 16) | (CSD[8] << 8) | CSD[9]
```

The capacity in bytes is:

```
capacity_bytes = (C_SIZE + 1) * 512 KiB
                = (C_SIZE + 1) * 524288
```

The capacity in 512-byte blocks is:

```
capacity_blocks = (C_SIZE + 1) * 1024
```

#### Worked Examples

| Nominal card size | C_SIZE (decimal) | Capacity (bytes) | Capacity (512-byte blocks) |
|-------------------|------------------|------------------|------------------|
| 8 GB SDHC | 0x3B5F (15199) | 7 969 177 600 | 15 564 800 |
| 16 GB SDHC | 0x76FF (30463) | 15 971 909 632 | 31 195 136 |
| 32 GB SDHC | 0xEEFF (61183) | 32 086 425 600 | 62 668 800 |
| 64 GB SDXC | 0x1DDFF (122879) | 64 424 509 440 | 125 829 120 |

(Values are illustrative; actual cards may report slightly different
C_SIZE values that account for spare reserved blocks.)

### 7.3.5 CSD v1.0 Field Map (SDSC) — Reference Only

For completeness, the v1 capacity formula is:

```
capacity_bytes = (C_SIZE + 1) * 2^(C_SIZE_MULT + 2) * 2^READ_BL_LEN
```

with `C_SIZE` (12 bits), `C_SIZE_MULT` (3 bits), and `READ_BL_LEN` (4
bits) at well-defined positions in the v1 CSD layout. FT1 does not
exercise this path because v1.x SDSC cards are rejected during init
(see [section 5.6](05_init_sequence.md#56-failure-modes)).

## 7.4 Reading the Registers — Driver Pseudocode

```
read_csd_or_cid(cmd, buffer16):
    cs_low()
    send_command(cmd, 0)
    r1 = wait_for_r1()
    if r1 != 0x00: cs_high(); return ERROR
    if !wait_for_token(0xFE, timeout_ms=100):
        cs_high(); return ERROR
    spi_read_bytes(buffer16, 16)
    spi_read_bytes(crc, 2)            # ignored
    cs_high()
    spi_send_byte(0xFF)               # release MISO
    return SUCCESS

read_cid(buf): return read_csd_or_cid(CMD10, buf)
read_csd(buf): return read_csd_or_cid(CMD9,  buf)

compute_capacity_blocks(csd):
    if (csd[0] >> 6) == 0b01:        # CSD v2
        c_size = ((csd[7] & 0x3F) << 16) | (csd[8] << 8) | csd[9]
        return (c_size + 1) * 1024
    elif (csd[0] >> 6) == 0b00:      # CSD v1 (SDSC) — not used in FT1
        ...
    else:
        return 0   # unsupported (CSD v3 / SDUC)
```

## 7.5 Use of CSD Capacity by FT1

The mass-logging app uses the CSD-derived capacity to:

1. Reject mass-storage initialization if total capacity is below a
   minimum threshold (e.g., 1 GB), to guard against a missing or
   counterfeit card.
2. Reject total capacity above 32 GB only if the FAT32 layer cannot
   address it (driver may simply cap usable capacity at 32 GB and
   warn).
3. Pre-allocate a flight log file size proportional to capacity.

These behaviors are app-level concerns and are not part of this ICD;
they are mentioned here only to motivate why CSD capacity is read.
