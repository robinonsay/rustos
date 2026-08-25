# 06 — Data Transfer

[← Back to index](index.md)

> **Source note:** Authored from public SD SPI-mode protocol knowledge.
> The SD Physical Layer Simplified Specification PDF is not present in
> this repository.

## 6.1 Scope of this Section

This ICD covers only the **single-block** read and write paths used by
FT1: CMD17 and CMD24. Multi-block read (CMD18) and multi-block write
(CMD25) are part of the SD spec but are **not used** by the FT1 driver
and are not specified here.

For all data transfers in this section:

- The block size is **fixed at 512 bytes** (SDHC/SDXC native; SDSC
  forced via CMD16 in [section 5.6](05_init_sequence.md#step-6--cmd16-set_blocklen-sdsc-only)).
- The address argument is a **block index** (each block is 512 bytes)
  on SDHC/SDXC cards or a **byte offset** (must be a multiple of 512) on
  SDSC cards. The driver records the CCS bit at init time and chooses
  the right encoding per command.
- A 16-bit CRC trails every data block on the wire. The CRC is **not
  validated** by the card on writes by default and is **not validated**
  by the host on reads by default. The driver shall send `0xFFFF` as
  the CRC on writes and shall ignore the CRC bytes on reads. CMD59
  (CRC_ON_OFF) can enable validation; FT1 leaves it off.

## 6.2 Data Token Bytes

A handful of single-byte "data tokens" frame the data phases of CMD17
and CMD24. The host and the card both use these to delimit blocks:

| Token (binary) | Hex | Direction | Meaning |
|---------------|-----|-----------|---------|
| 11111110 | 0xFE | Both ways | "Start of single-block read or write data" |
| 11111100 | 0xFC | Host → Card | "Start of multi-block write data" (NOT used in FT1) |
| 11111101 | 0xFD | Host → Card | "Stop of multi-block write" (NOT used in FT1) |

Bit 0 of the start token is always 0 and the upper 7 bits are 1; this
makes the token easy to find while polling MISO with MOSI = 0xFF.

## 6.3 CMD17 — READ_SINGLE_BLOCK

### 6.3.1 Wire Sequence

```
Host: <CMD17 6 bytes> 0xFF 0xFF ... 0xFF 0xFF 0xFF ... 0xFF 0xFF 0xFF 0xFF
Card:                <R1>  ... wait ...  0xFE <512 data bytes> <CRC hi> <CRC lo>
                      ^                  ^
                      response start     data start token

(CS held low for the entire window above.)
```

### 6.3.2 Step-by-Step

1. Pull **CS low**.
2. Send the CMD17 command frame. Argument = block address (block index
   for SDHC/SDXC; byte offset for SDSC).
3. Poll MISO for the R1 byte. It must be `0x00`. Any other value is a
   command-level rejection — pull CS high, send 0xFF, return error.
4. Continue polling MISO with MOSI = 0xFF for the data start token.
   Expect `0xFE` within the **read access timeout** (see
   [6.5](#65-timeouts-and-retries)).
5. After the 0xFE token, read exactly **512 bytes** of payload into the
   caller-supplied buffer.
6. Read **2 more bytes** — the 16-bit CRC. The driver discards them.
7. Pull **CS high** and send one byte of 0xFF.

### 6.3.3 Error Tokens (Read Path)

If the card cannot satisfy the read, instead of the 0xFE start token it
sends a **read-error token**. A read-error token has its upper 4 bits =
0 (so it is distinguishable from 0xFE, whose upper bits are 1):

| Bits | Field |
|------|-------|
| 7..4 | 0000 (signals "read error token") |
| 3 | Card locked / unlock failed |
| 2 | Out of range |
| 1 | Card ECC failed |
| 0 | Card error |

If the host sees a byte that is neither 0xFF (still waiting) nor 0xFE
(start token), and bit 7 is 0, the byte is a read-error token. The
driver shall pull CS high, send 0xFF, and return an error to the
caller, preserving the token bits in the diagnostic log.

## 6.4 CMD24 — WRITE_BLOCK

### 6.4.1 Wire Sequence

```
Host: <CMD24 6 bytes> 0xFF 0xFF ... 0xFF 0xFE <512 data bytes> 0xFF 0xFF 0xFF 0xFF ... 0xFF
Card:                <R1>  0xFF 0xFF ...      0xFF ... 0xFF    <DRT> 0x00 0x00 ... 0xFF
                                                                      \__ busy ___/^ ready

(CS held low for the entire window above.)
                                              ^                ^
                                              host now drives  card responds with
                                              token+payload+CRC data response token (DRT)
                                              with MOSI; MISO   then drives MISO low
                                              ignored           while busy
```

### 6.4.2 Step-by-Step

1. Pull **CS low**.
2. Send the CMD24 command frame. Argument = block address (block index
   for SDHC/SDXC; byte offset for SDSC).
3. Poll MISO for the R1 byte. It must be `0x00`. Any other value is a
   command-level rejection.
4. Send **at least one byte of 0xFF** (one byte of dwell after R1; the
   spec recommends "Nwr" of at least 1 byte before the data token).
5. Send the data start token: **0xFE**.
6. Send exactly **512 bytes** of payload.
7. Send the 16-bit CRC. With CRC validation off, send `0xFF 0xFF`.
8. Read **one byte** from MISO — the **data response token** (DRT).
9. Validate the DRT:

| DRT pattern | Meaning |
|-------------|---------|
| `xxx00101` (0x05 with arbitrary upper 3 bits) | **Data accepted** |
| `xxx01011` | Data rejected — CRC error |
| `xxx01101` | Data rejected — write error (e.g., over-range, write-protected) |

The fixed bit pattern is bits 4..0 = `00101` for accept; the upper 3
bits are reserved. The driver should mask with `0x1F` and compare to
`0x05`.

10. The card now drives **MISO low** while it programs the block.
    Continue polling MISO with MOSI = 0xFF until MISO returns to `0xFF`
    (or until the busy-poll timeout — see [6.5](#65-timeouts-and-retries)).
11. Pull **CS high**, send one byte of 0xFF.

### 6.4.3 Verifying the Write Reached the Card

The DRT confirms the card **accepted** the data into its buffer; the
busy phase ends when the card has **programmed** the data into its
flash. Both must complete successfully for the write to be considered
durable. After busy clears, the driver may issue **CMD13 (SEND_STATUS)**
to read R2 and confirm no programming error was detected by the card
post-flash. FT1 treats CMD13 after every CMD24 as optional (used during
ground tests, omitted in flight to save time).

## 6.5 Timeouts and Retries

| Wait | Typical | Driver timeout | Action on timeout |
|------|---------|----------------|-------------------|
| R1 after CMD17 / CMD24 | < 1 ms | 100 ms | Abort transaction; return error |
| Data start token (0xFE) after R1 of CMD17 | 1–5 ms | 100 ms | Abort transaction; return error |
| DRT after CMD24 payload | < 1 ms | 250 ms | Abort transaction; return error |
| Busy-clear after DRT (CMD24) | 1–10 ms | 250 ms | Pull CS high anyway, return error to caller; the card may still complete the write internally — the next operation will reflect that |

Single-block read and single-block write should not require retries at
the protocol level — a failed transaction propagates an error to the
mass-logging app, which decides whether to retry or surface to
telemetry.

## 6.6 Throughput Estimate

At a 12 MHz operating clock with 8 bits per byte:

| Phase | Bytes | Wire time @ 12 MHz |
|-------|-------|---------------------|
| CMD17/24 frame | 6 | 4 µs |
| R1 wait | up to 8 | 5 µs |
| Token | 1 | 0.7 µs |
| Payload | 512 | 341 µs |
| CRC | 2 | 1.3 µs |
| Inter-transaction CS-up byte | 1 | 0.7 µs |
| **Total per CMD17 read** | ≈ 530 | ≈ 353 µs |
| **Total per CMD24 write (no busy)** | ≈ 530 | ≈ 353 µs |

Add the worst-case 10 ms write-busy phase to the write side. Sustained
single-block-write throughput at 12 MHz with worst-case busy is
**512 B / 10.4 ms ≈ 49 KB/s**. The mass-logger workload (~ 20 KB/s peak)
sits comfortably under that ceiling. If higher throughput is needed
later, the upgrade path is to switch to multi-block writes (CMD25),
which amortizes the busy phase across many blocks.

## 6.7 Address Translation Summary

| Card type (CCS at init) | CMD17/CMD24 argument |
|-------------------------|----------------------|
| SDHC / SDXC (CCS = 1) | Block index = (logical byte offset) / 512 |
| SDSC (CCS = 0) | Byte offset = block index × 512 (must be 512-aligned) |

The driver shall enforce alignment at the public API boundary: only
512-byte-aligned block indices are accepted from upper layers. Any
non-aligned read/write request shall be rejected with a parameter error.
