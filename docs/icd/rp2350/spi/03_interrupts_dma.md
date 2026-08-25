# SPI: Interrupts & DMA

[Back to SPI index](index.md) | [Back to ICD index](../index.md)

## Interrupt Sources

Four individually maskable interrupts, plus a single combined output to
the NVIC.

| Source | Description | Mask Bit (SSPIMSC) | Clear |
|--------|-------------|--------------------|-------|
| `SSPTXINTR` | TX FIFO half-empty (≤4 bytes) | `TXIM` (bit 3) | Auto: write data until above watermark |
| `SSPRXINTR` | RX FIFO half-full (≥4 bytes) | `RXIM` (bit 2) | Auto: read until below watermark |
| `SSPRTINTR` | RX timeout — RX FIFO has data, no read in 32 SCK periods | `RTIM` (bit 1) | `SSPICR.RTIC` (write 1) |
| `SSPRORINTR` | RX overrun — RX FIFO written while full | `RORIM` (bit 0) | `SSPICR.RORIC` (write 1) |

`SSPRIS` (raw) and `SSPMIS` (masked) report current state. `SSPICR` clears
RT and ROR latches; TX/RX track FIFO state automatically.

### TX/RX Watermarks

Fixed at 4 (half of 8). Not programmable on PL022.

### NVIC Connection

| Symbolic | Vector |
|----------|--------|
| `SPI0_IRQ` | SPI0 combined |
| `SPI1_IRQ` | SPI1 combined |

## 12.3.4.16. DMA Interface

Each instance has 4 DMA handshake signals (per direction). Signals are
synchronous to PCLK.

| Signal | Description |
|--------|-------------|
| `SSPRXDMASREQ` | Single RX request — RX FIFO non-empty |
| `SSPRXDMABREQ` | Burst RX request — RX FIFO ≥ 4 |
| `SSPRXDMACLR` | DMA acks RX request |
| `SSPTXDMASREQ` | Single TX request — TX FIFO has space |
| `SSPTXDMABREQ` | Burst TX request — TX FIFO ≤ 4 used |
| `SSPTXDMACLR` | DMA acks TX request |

Burst threshold (watermark) is **fixed at 4**.

### DMA Enable (`SSPDMACR`)

| Bit | Field | Description |
|-----|-------|-------------|
| 1 | `TXDMAE` | Enable TX DMA request line |
| 0 | `RXDMAE` | Enable RX DMA request line |

When the SSP is disabled (`SSE=0`) or `TXDMAE`/`RXDMAE` is cleared, the
corresponding request line de-asserts.

### Connecting DMA Channels

| Direction | DREQ |
|-----------|------|
| SPI0 TX | `DREQ_SPI0_TX` |
| SPI0 RX | `DREQ_SPI0_RX` |
| SPI1 TX | `DREQ_SPI1_TX` |
| SPI1 RX | `DREQ_SPI1_RX` |

### Full-Duplex DMA Pattern

Allocate two channels (one TX, one RX). For an N-byte transfer:

1. Configure RX channel:
   - DREQ = `DREQ_SPIn_RX`.
   - Source = `&SSP->dr`.
   - Destination = caller buffer (or null sink for write-only).
   - Transfer count = N.
2. Configure TX channel:
   - DREQ = `DREQ_SPIn_TX`.
   - Source = caller buffer (or null source for read-only).
   - Destination = `&SSP->dr`.
   - Transfer count = N.
3. Set `SSPDMACR = TXDMAE | RXDMAE`.
4. Enable both channels.
5. RX channel completion = transfer done. (TX completes earlier because
   FIFO is shallow.)

### Read-Only or Write-Only

- **Read-only:** TX channel's source is a fixed dummy byte (0xFF for SD
  card reads). RX channel writes to the caller buffer.
- **Write-only:** RX channel's destination is a discard sink (e.g., a
  scratch byte with `write_increment = 0`).

The PL022 does not support disabling either direction independently —
every TX byte produces one RX byte, so a parallel RX channel is always
required.

## FT1 Driver Recommendations

- For SD card block reads (512 bytes at a time), DMA both directions
  with the dummy-byte TX pattern.
- For small register reads/writes (sensors, radio), CPU polling on
  `SSPSR.TNF` / `SSPSR.RNE` is simpler and lower overhead than setting up
  DMA channels.
- Enable `SSPRORINTR` for diagnostics — overruns indicate the driver
  isn't draining RX fast enough.
- `SSPRTINTR` is useful to close variable-length transactions (e.g., when
  the slave's response length isn't known a priori).

## See Also

- [`02_operation.md`](02_operation.md) — Init and TX/RX flow.
- [`04_registers.md`](04_registers.md) — `SSPIMSC`, `SSPICR`, `SSPDMACR`
  details.
