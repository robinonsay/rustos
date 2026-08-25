# UART: Overview & Functional Description

[Back to UART index](index.md) | [Back to ICD index](../index.md)

## 12.1. Block Description

RP2350 has two identical UART instances based on the Arm PrimeCell PL011
(r1p5). Each instance includes:

- 32 × 8 TX FIFO
- 32 × 12 RX FIFO (data + framing/parity/break/overrun status)
- Programmable baud-rate generator clocked by `clk_peri`
- Programmable serial format (5/6/7/8 data bits, 1/2 stop, parity)
- Line-break detection
- Programmable hardware flow control (RTS/CTS)
- DMA interface (single + burst)

> Modem and IrDA features of PL011 are **not** supported on RP2350.

### Connection Model

| PL011 signal | Pin name on GPIO mux |
|--------------|----------------------|
| `UARTTXD` | `uartN_tx` |
| `UARTRXD` | `uartN_rx` |
| `nUARTRTS` | `uartN_rts` |
| `nUARTCTS` | `uartN_cts` |

`UARTCLK` ← `clk_peri`. `PCLK` ← `clk_sys`.

## 12.1.2. Internal Functional Blocks

| Block | Role |
|-------|------|
| AMBA APB interface | Decodes register/FIFO accesses |
| Register block | Holds control / status / FIFO pointers |
| Baud-rate generator | Generates `Baud16` (16× the line rate) |
| Transmit FIFO | 32 × 8-bit; behaves as 1-byte holding register when FIFO disabled |
| Receive FIFO | 32 × 12-bit (data + status) |
| Transmit logic | Parallel-to-serial, frames data |
| Receive logic | Serial-to-parallel, error detection |
| Interrupt logic | Generates 11 individual interrupts; combined IRQ output |
| DMA interface | Single + burst request/clear handshake |
| Synchronizers | PCLK ↔ UARTCLK domain crossing |

## 12.1.3.1. Clock Constraints

For a target baud rate range:

```
F_UARTCLK_min ≥ 16 × baud_rate_max
F_UARTCLK_max ≤ 16 × 65535 × baud_rate_min
F_UARTCLK    ≤ (5/3) × F_PCLK
```

Examples:

- 110 baud to 460 800 baud range → `clk_peri` between 7.37 MHz and
  115.34 MHz.
- 921 600 baud at `UARTCLK = 14.7456 MHz` → `PCLK ≥ 8.85 MHz`.
- 9600 baud (FT1 GPS) at `UARTCLK = 125 MHz` → easily satisfied.

## 12.1.3.2. Frame Layout

```
  Start  D0  D1  D2  D3  D4  D5  D6  D7  [Parity]  Stop1  [Stop2]
   0    LSB                                MSB
```

- Word length 5..8, programmed via `UARTLCR_H.WLEN`.
- One or two stop bits (`UARTLCR_H.STP2`).
- Parity enable + odd/even/stick (`UARTLCR_H.PEN`, `EPS`, `SPS`).
- Optional break (`UARTLCR_H.BRK` — drives line low for ≥2 frames).

### Receiver Sampling

For each bit, the receiver samples three times near the bit centre and
takes the majority. The start bit is verified at the 8th cycle of `Baud16`
(half-bit time). If the line is still low at sample time, the start is
accepted; otherwise it's a glitch and is ignored.

### Receive FIFO Status Bits

| FIFO bit | Meaning |
|----------|---------|
| 11 | Overrun |
| 10 | Break error |
| 9 | Parity error |
| 8 | Framing error |
| 7:0 | Received data |

The overrun bit, when raised in `UARTDR`, applies to the **next** character
that fits in the FIFO (not the one that was lost).

## 12.1.4. Hardware Flow Control

Optional RTS/CTS:

| Mode | `CTSEN` | `RTSEN` |
|------|---------|---------|
| Both RTS+CTS | 1 | 1 |
| CTS only | 1 | 0 |
| RTS only | 0 | 1 |
| Neither | 0 | 0 |

When RTS flow control is enabled, `nUARTRTS` is asserted until the RX FIFO
fills to the watermark (`UARTIFLS.RXIFLSEL`). On reaching the watermark,
RTS is de-asserted. CTS flow control gates each transmitted byte on the
`nUARTCTS` input.

> When RTS flow control is enabled in `UARTCR.RTSEN`, software cannot
> directly drive RTS via `UARTCR.RTS`.

### FT1 Notes

- GPS link has no RTS/CTS — keep both bits 0.
- For a future telemetry link with flow control, set `RXIFLSEL` to 1/2 or
  3/4 to give the remote side adequate headroom before RTS de-asserts.

## 12.1.5. DMA Interface

DMA signals (per direction):

| Signal | Description |
|--------|-------------|
| `UARTRXDMASREQ` | Single RX request — asserted when RX FIFO non-empty |
| `UARTRXDMABREQ` | Burst RX request — asserted when RX FIFO ≥ watermark |
| `UARTRXDMACLR` | DMA clears the RX request |
| `UARTTXDMASREQ` | Single TX request — asserted when TX FIFO not full |
| `UARTTXDMABREQ` | Burst TX request — asserted when TX FIFO ≤ watermark |
| `UARTTXDMACLR` | DMA clears the TX request |

DMA enables: `UARTDMACR.RXDMAE`, `UARTDMACR.TXDMAE`. The driver may also
set `UARTDMACR.DMAONERR` so that an RX error masks RX DMA requests until
software acks the error.

### DMA Trigger Watermarks

Selected via `UARTIFLS.RXIFLSEL` / `TXIFLSEL`:

| Watermark | TX (empty slots) | RX (filled slots) |
|-----------|------------------|-------------------|
| 1/8  | 28 | 4 |
| 1/4  | 24 | 8 |
| 1/2  | 16 | 16 |
| 3/4  | 8  | 24 |
| 7/8  | 4  | 28 |

When the FIFO is disabled (`UARTLCR_H.FEN=0`), only single transfer
requests can fire — burst requests stay low.

## See Also

- [`02_operation.md`](02_operation.md) — Init sequence, baud-rate calc.
- [`03_interrupts.md`](03_interrupts.md) — Interrupt sources.
- [`04_registers.md`](04_registers.md) — Register layouts.
- See source PDF Figure 64 for the character-frame timing diagram.
