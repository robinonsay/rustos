# 04 — Response Formats

[← Back to index](index.md)

> **Source note:** Authored from public SD SPI-mode protocol knowledge.
> The SD Physical Layer Simplified Specification PDF is not present in
> this repository.

## 4.1 General Response Framing

After the host sends a 6-byte command frame and continues clocking with
MOSI = 0xFF, the card replies on MISO. Every response in SPI mode begins
with a **byte whose MSB (bit 7) is 0**. While the card is "thinking" — has
not yet started its response — it transmits 0xFF on MISO (MSB = 1).

The host shall poll MISO byte-by-byte, looking for the first byte that
clears bit 7. That byte is the start of the response. The card guarantees
this byte will appear within **NCR** (Number of Clocks for Response) bytes
of the end of the command, where:

- NCR ≤ 8 bytes for SPI-mode commands.
- The host shall give up after polling **16 bytes** without seeing a
  response start, treating the situation as a card-not-present or
  card-fault error.

```
Host: <CMD bytes 0..5> 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF ...
Card:  ?    ?    ?    ?    ?    ?    0xFF 0xFF 0xFF <RESP> <RESP> ...
                                    \_____/      \_/
                                    NCR (≤ 8)    response begins
                                    waiting bytes
```

The format of `<RESP>` depends on the command. SPI-mode SD responses come
in five flavors: **R1, R1b, R2, R3, R7**.

## 4.2 R1 — Standard 1-byte Status Response

R1 is the most common response. It is a single byte whose bits report
top-level command-execution flags.

| Bit | Name | Meaning when set (=1) |
|-----|------|----------------------|
| 7   | (reserved) | Always 0 — used by the host to identify the start of the response |
| 6   | Parameter Error | Argument was out of the allowed range |
| 5   | Address Error | Misaligned address (e.g., CMD17 with non-512-aligned address on SDSC) |
| 4   | Erase Sequence Error | Erase command sequence violated |
| 3   | Com CRC Error | CRC7 of the most recent command failed (only meaningful when CRC checking is enabled) |
| 2   | Illegal Command | Command not supported in the card's current state |
| 1   | Erase Reset | A previous erase sequence was cleared by a non-erase command |
| 0   | In Idle State | Card is in idle (post-CMD0, pre-init-complete) state |

### 4.2.1 Notable R1 Values

| Hex | Meaning |
|-----|---------|
| 0x00 | Card ready (initialization complete, no errors) |
| 0x01 | In idle state (post-CMD0, ACMD41 not yet completed) |
| 0x04 | Illegal command (e.g., CMD8 on a v1.x card, treated by the host as a hint that the card is SDSC v1.x) |
| 0x05 | Idle + illegal command (CMD8 issued to a v1.x card while idle) |
| 0xFF | **Not** a valid R1 — indicates the card never responded; the host shall treat this as a timeout |

R1 is returned by: CMD0, CMD8 (as the leading byte of R7), CMD9, CMD10,
CMD13 (as the leading byte of R2), CMD16, CMD17, CMD24, CMD55, CMD58 (as
the leading byte of R3), ACMD41.

## 4.3 R1b — R1 with Busy Signal

R1b is identical to R1 in its content and framing, but the **card holds
MISO low** for as long as it needs to complete the operation. The host
must clock the bus and read MISO until it sees 0xFF, indicating the card
has released the line and the operation is complete.

```
Host: ... 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF 0xFF ...
Card: <R1> 0x00 0x00 0x00 0x00 0x00 0x00 0xFF 0xFF
           \________ busy ________/    ^ ready
```

The host shall apply a **busy-poll timeout**:

| Operation | Typical busy time | Driver timeout |
|-----------|-------------------|----------------|
| CMD24 (single-block write) | 1–10 ms | 250 ms |
| CMD25 (multi-block write) — not used in FT1 | up to 500 ms per block | n/a |
| Erase commands — not used in FT1 | seconds | n/a |

R1b is returned by: CMD12 (STOP_TRANSMISSION, not used in FT1), CMD28,
CMD29, CMD38 (erase commands, not used in FT1). The trailing busy phase
also follows the data-response token after a CMD24 write — see
[06_data_transfer.md](06_data_transfer.md).

## 4.4 R2 — 2-byte Status Response (CMD13)

R2 is two bytes long. Byte 0 is identical in format to R1; byte 1 carries
extended status flags specific to CMD13 (SEND_STATUS).

| Byte | Bits | Field |
|------|------|-------|
| 0 | 7..0 | R1 status (see [4.2](#42-r1--standard-1-byte-status-response)) |
| 1 | 7 | Card is locked |
| 1 | 6 | Write protect erase skip / lock-unlock command failed |
| 1 | 5 | General error / unknown error |
| 1 | 4 | Card controller error |
| 1 | 3 | Card ECC failed |
| 1 | 2 | Write protect violation |
| 1 | 1 | Erase parameter |
| 1 | 0 | Out-of-range / CSD overwrite |

A healthy card responds to CMD13 with R2 = 0x0000.

R2 is returned only by CMD13.

## 4.5 R3 — R1 + 32-bit OCR (CMD58)

R3 is five bytes: a leading R1 byte followed by the 32-bit Operating
Conditions Register (OCR), most-significant-byte first.

```
Byte:    0    1    2    3    4
Field: <R1> <OCR[31:24]> <OCR[23:16]> <OCR[15:8]> <OCR[7:0]>
```

### 4.5.1 OCR Bit Map

| Bit | Name | Meaning |
|-----|------|---------|
| 31  | Card Power-Up Status (busy) | 1 = card has finished power-up; 0 = still busy |
| 30  | CCS (Card Capacity Status) | 1 = SDHC or SDXC (block-addressed); 0 = SDSC (byte-addressed). Only valid when bit 31 is 1 |
| 29  | UHS-II card status | Always 0 for SPI mode |
| 24  | Switching to 1.8 V accepted | Always 0 in SPI mode |
| 23  | 3.5–3.6 V supported | Typically 1 |
| 22  | 3.4–3.5 V supported | Typically 1 |
| 21  | 3.3–3.4 V supported | **Must be 1 for FT1 — the rocket runs on 3.3 V** |
| 20  | 3.2–3.3 V supported | Typically 1 |
| 19..15 | Lower voltage windows | Typically 0; not used for 3.3 V operation |
| Other bits | Reserved | Read as 0 |

### 4.5.2 Notable OCR Patterns

| 32-bit OCR | Meaning |
|------------|---------|
| 0xC0FF8000 | Powered-up, SDHC/SDXC, full 2.7–3.6 V range supported |
| 0x80FF8000 | Powered-up, SDSC (CCS=0), full voltage range |
| 0x00FF8000 | Still powering up — card not yet ready (host shall retry) |

R3 is returned only by CMD58.

## 4.6 R7 — R1 + 32-bit Voltage Echo (CMD8)

R7 is five bytes, identically framed to R3: a leading R1 byte followed by
a 32-bit payload, most-significant-byte first. The payload is the card's
**echo of the host's CMD8 argument** along with a voltage-acceptance
field.

```
Byte:    0    1    2    3    4
Field: <R1> 0x00       0x00       <VHS_echo>   <pattern_echo>
              (rsvd)   (rsvd)      bits 11..8    bits 7..0
```

### 4.6.1 R7 Payload Layout

| Bytes 1..4 (32-bit) | Bits | Field |
|---------------------|------|-------|
| 31..28 | 4   | Command version (0x1 for CMD8 v1) |
| 27..12 | 16  | Reserved (0) |
| 11..8  | 4   | Voltage Accepted (VHS echo): 0x1 = 2.7–3.6 V |
| 7..0   | 8   | Check pattern echo (echoes byte 4 of the command argument) |

### 4.6.2 R7 Validation

The host sent CMD8 with argument 0x000001AA. A valid R7 reply has:

- R1 byte = 0x01 (idle, no errors). A v1.x SDSC card replies with R1 byte
  = 0x05 (idle + illegal command); the host shall treat this as "card is
  v1.x SDSC, skip CMD8 path."
- VHS echo nibble = 0x1 (the host's VHS argument)
- Check pattern echo = 0xAA (the host's pattern byte)

If either echo field does not match the host-supplied value the card
shall be treated as non-functional and the init sequence aborted.

R7 is returned only by CMD8.

## 4.7 Response Selection Table

| Command sent | Response type | Response length |
|--------------|---------------|-----------------|
| CMD0 | R1 | 1 byte |
| CMD8 | R7 | 5 bytes |
| CMD9 | R1 + 16-byte CSD payload + 2-byte CRC (data block) | 1 + 16 + 2 bytes |
| CMD10 | R1 + 16-byte CID payload + 2-byte CRC (data block) | 1 + 16 + 2 bytes |
| CMD13 | R2 | 2 bytes |
| CMD16 | R1 | 1 byte |
| CMD17 | R1, then data token + 512 B + 2-byte CRC | 1 + 1 + 512 + 2 bytes |
| CMD24 | R1, then host writes; card replies with data-response token + busy | 1 + 1 + busy bytes |
| CMD55 | R1 | 1 byte |
| CMD58 | R3 | 5 bytes |
| ACMD41 | R1 | 1 byte |

## 4.8 Error Recovery from Unexpected Responses

| Symptom | Likely cause | Driver action |
|---------|--------------|---------------|
| 16 bytes of 0xFF after a command | Card not present, miswired CS/MISO, or card hung | Abort init; surface "card not present" error to telemetry |
| R1 bit 2 (Illegal Command) for CMD8 | Card is SDSC v1.x | Take the v1.x branch of the init sequence (use CMD1 instead of ACMD41; this branch is documented but not used by FT1) |
| R1 = 0x01 for several seconds during ACMD41 loop | Card slow to power up | Continue polling up to a 1-second total budget |
| R1 ≠ 0x00 after CMD17 / CMD24 | Address out of range, card not initialized, or write-protected | Surface error to caller; log argument and R1 byte |
| Data-response token != 0bxxx00101 after CMD24 | Card rejected the data block | See [06_data_transfer.md](06_data_transfer.md) |
