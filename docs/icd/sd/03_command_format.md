# 03 — Command Format

[← Back to index](index.md)

> **Source note:** Authored from public SD SPI-mode protocol knowledge.
> The SD Physical Layer Simplified Specification PDF is not present in
> this repository.

## 3.1 Command Frame Length

Every command sent over the SD SPI bus is exactly **6 bytes**, transmitted
MSB-first, while CS is held low. The frame layout is fixed:

| Offset | Length | Field | Description |
|--------|--------|-------|-------------|
| 0      | 1 byte | Command index byte | Start bit + transmission bit + 6-bit command index |
| 1–4    | 4 bytes | Argument | 32-bit, big-endian command-specific argument |
| 5      | 1 byte | CRC + end bit | 7-bit CRC7 + end bit |

There is no variable-length command form in SPI mode. Commands that take no
argument (e.g., CMD0) still send all four argument bytes as 0x00000000.

## 3.2 Byte-by-Byte Layout

```
 Byte 0          Byte 1     Byte 2     Byte 3     Byte 4     Byte 5
+--------+      +--------+ +--------+ +--------+ +--------+ +--------+
|01iiiiii|      |aaaaaaaa| |aaaaaaaa| |aaaaaaaa| |aaaaaaaa| |ccccccc1|
+--------+      +--------+ +--------+ +--------+ +--------+ +--------+
 ||\____/        \________________________________________/  \_____/|
 || command       32-bit argument (big-endian, MSB first)    7-bit  end
 || index 0..63                                              CRC7   bit
 ||
 |+--- transmission bit, always 1 (host-to-card)
 +---- start bit, always 0
```

## 3.3 Field Definitions

### 3.3.1 Command Index Byte (Byte 0)

| Bit | Value | Name | Meaning |
|-----|-------|------|---------|
| 7   | 0     | Start bit | Always 0 for a command frame |
| 6   | 1     | Transmission bit | 1 = host-to-card direction |
| 5..0 | iiiiii | Command index | 6-bit command number, 0–63 |

The two fixed bits make the command-index byte **always start with binary
`01`**. So byte 0 of any command is `0x40 | command_index`. Examples:

| Command | Index | Byte 0 |
|---------|-------|--------|
| CMD0    | 0     | 0x40 |
| CMD1    | 1     | 0x41 |
| CMD8    | 8     | 0x48 |
| CMD9    | 9     | 0x49 |
| CMD10   | 10    | 0x4A |
| CMD13   | 13    | 0x4D |
| CMD16   | 16    | 0x50 |
| CMD17   | 17    | 0x51 |
| CMD24   | 24    | 0x58 |
| CMD55   | 55    | 0x77 |
| CMD58   | 58    | 0x7A |
| ACMD41  | 41    | 0x69 (sent as a normal command, prefixed by CMD55) |

### 3.3.2 Argument (Bytes 1–4)

The 32-bit argument is transmitted **most significant byte first** (bytes
1, 2, 3, 4 carry bits 31..24, 23..16, 15..8, 7..0 respectively).

The semantics of the argument are command-specific:

| Command | Argument Encoding |
|---------|-------------------|
| CMD0 (GO_IDLE_STATE) | 0x00000000 (stuffed, ignored by card) |
| CMD8 (SEND_IF_COND) | bits 11..8 = VHS (voltage host supplied) = 0x1; bits 7..0 = check pattern (host-chosen, conventionally 0xAA); upper bits 0 |
| CMD9 (SEND_CSD) | 0x00000000 (stuffed) |
| CMD10 (SEND_CID) | 0x00000000 (stuffed) |
| CMD13 (SEND_STATUS) | 0x00000000 (stuffed) |
| CMD16 (SET_BLOCKLEN) | block length in bytes, big-endian (typically 0x00000200 = 512) |
| CMD17 (READ_SINGLE_BLOCK) | block address: byte offset for SDSC, 512-byte block index for SDHC/SDXC |
| CMD24 (WRITE_BLOCK) | block address: byte offset for SDSC, 512-byte block index for SDHC/SDXC |
| CMD55 (APP_CMD) | 0x00000000 (stuffed) |
| CMD58 (READ_OCR) | 0x00000000 (stuffed) |
| ACMD41 (SD_SEND_OP_COND) | bit 30 = HCS (Host Capacity Support); set to 1 to declare host can address SDHC/SDXC. Other bits 0 |

### 3.3.3 CRC + End Bit (Byte 5)

| Bit | Value | Meaning |
|-----|-------|---------|
| 7..1 | ccccccc | 7-bit CRC7 of bytes 0..4 |
| 0 | 1 | End bit, always 1 |

CRC7 is computed across the **first 5 bytes** of the command (the index
byte and all 4 argument bytes), most-significant-bit first, using the
generator polynomial:

```
G(x) = x^7 + x^3 + 1     (binary 10001001 = 0x89)
```

The CRC result is a 7-bit value placed in bits 7..1 of byte 5.

#### CRC Requirement Window

In SPI mode the card **only validates CRC7 on CMD0 and CMD8** by default;
all other commands accept any CRC7 value. As a practical consequence:

- **CMD0 must carry CRC7 = 0x4A** (so byte 5 = `0x95`). This is the only
  CRC7 value most cards will accept for CMD0 in SPI mode.
- **CMD8 must carry CRC7 = 0x43** (so byte 5 = `0x87`) when sent with the
  conventional argument 0x000001AA (VHS=1, check pattern=0xAA).
- For all other commands, drivers conventionally send a "stuff" CRC of
  `0xFF` in byte 5 (which is CRC7 = 0x7F + end bit 1) or `0x01` (CRC7 =
  0x00 + end bit 1). The card ignores the value.

A robust driver shall compute CRC7 dynamically for every command. The
SD specification permits the host to enable per-command CRC checking via
CMD59 (CRC_ON_OFF); FT1 firmware leaves CRC checking off after init for
performance.

## 3.4 Worked Examples

### 3.4.1 CMD0 (GO_IDLE_STATE) — wakes the card into SPI mode

```
Byte:      0    1    2    3    4    5
Hex:     0x40 0x00 0x00 0x00 0x00 0x95
Binary:  01000000 00000000 00000000 00000000 00000000 10010101
         ||\____/ \________________________________/ \_______/|
         || index=0           argument=0x00000000     CRC7    end=1
         ||                                          =0x4A
         |+ transmission=1
         + start=0
```

### 3.4.2 CMD8 (SEND_IF_COND) — voltage check

```
Byte:      0    1    2    3    4    5
Hex:     0x48 0x00 0x00 0x01 0xAA 0x87
                          ^^^^ ^^^^ ^^^^
                          VHS  check  CRC7=0x43, end=1
                          =0x1 pattern
                               =0xAA
```

The argument 0x000001AA encodes:
- bits 31..12 = 0 (reserved)
- bits 11..8 = 0x1 (VHS = "2.7–3.6 V supplied")
- bits 7..0 = 0xAA (check pattern, host-chosen)

### 3.4.3 CMD17 (READ_SINGLE_BLOCK) — read block index 0x00001234 from SDHC

```
Byte:      0    1    2    3    4    5
Hex:     0x51 0x00 0x00 0x12 0x34 0xFF
                    \_______ block index ______/  stuff CRC
```

Note that the argument is the **block index** (each block being 512 bytes)
for SDHC/SDXC, not a byte offset. So block index 0x00001234 corresponds to
byte offset 0x00001234 × 512 = 0x00246800. SDSC cards interpret the same
argument as a byte offset that must be a multiple of 512.

## 3.5 Command Table (Commands Used by FT1)

| Command | Hex | Name | Argument | Response | Used in |
|---------|-----|------|----------|----------|---------|
| CMD0    | 0x40 | GO_IDLE_STATE | 0x00000000 | R1 | Init |
| CMD8    | 0x48 | SEND_IF_COND | 0x000001AA | R7 | Init |
| CMD9    | 0x49 | SEND_CSD | 0x00000000 | R1 + 16-byte data | Capacity discovery |
| CMD10   | 0x4A | SEND_CID | 0x00000000 | R1 + 16-byte data | Card identification |
| CMD13   | 0x4D | SEND_STATUS | 0x00000000 | R2 | Liveness check |
| CMD16   | 0x50 | SET_BLOCKLEN | 0x00000200 | R1 | Init (SDSC only) |
| CMD17   | 0x51 | READ_SINGLE_BLOCK | block address | R1 + data token + 512 B + CRC | Data path |
| CMD24   | 0x58 | WRITE_BLOCK | block address | R1, then host sends data; card returns data response token | Data path |
| CMD55   | 0x77 | APP_CMD | 0x00000000 | R1 | Init (precedes ACMD41) |
| CMD58   | 0x7A | READ_OCR | 0x00000000 | R3 | Init |
| ACMD41  | 0x69 | SD_SEND_OP_COND | bit 30 = HCS | R1 | Init |

ACMD-numbered commands are sent on the wire as ordinary 6-byte command
frames whose index byte equals `0x40 | (ACMD_NUMBER & 0x3F)` and that are
**immediately preceded** by a CMD55 with the same CS-low window. The card
treats the command following CMD55 as an application command rather than
the standard command of the same number.
