# UART: Interrupts

[Back to UART index](index.md) | [Back to ICD index](../index.md)

## 12.1.6. Interrupt Sources

The PL011 generates 11 individually maskable interrupts. RP2350 routes only
the **combined** `UARTINTR` to the NVIC — individual sources are read via
`UARTRIS` / `UARTMIS`.

| Source | Description | Cleared By |
|--------|-------------|------------|
| `UARTRXINTR` | RX FIFO above watermark, or 1-byte holding register full | Read FIFO down to below watermark, or `UARTICR.RXIC` |
| `UARTTXINTR` | TX FIFO at/below watermark | Write data, or `UARTICR.TXIC` |
| `UARTRTINTR` | Receive timeout (no new RX for 32 bit-times) | Read FIFO empty, or `UARTICR.RTIC` |
| `UARTOEINTR` | Overrun error | `UARTICR.OEIC` (or write `UARTECR`) |
| `UARTBEINTR` | Break error | `UARTICR.BEIC` |
| `UARTPEINTR` | Parity error | `UARTICR.PEIC` |
| `UARTFEINTR` | Framing error | `UARTICR.FEIC` |
| `UARTRIINTR` | Modem RI change (unused on RP2350) | `UARTICR.RIMIC` |
| `UARTCTSINTR` | nUARTCTS change | `UARTICR.CTSMIC` |
| `UARTDCDINTR` | nUARTDCD change | `UARTICR.DCDMIC` |
| `UARTDSRINTR` | nUARTDSR change | `UARTICR.DSRMIC` |
| `UARTEINTR` | Combined error (FE/PE/BE/OE) | Individual error clears |
| `UARTMSINTR` | Combined modem-status (CTS/DCD/DSR/RI) | Individual modem clears |
| `UARTINTR` | Combined output to NVIC | All sub-sources resolved |

To enable an interrupt: set the matching bit in `UARTIMSC`. To disable:
clear the bit. `UARTRIS` shows the raw flag; `UARTMIS` shows the same
after the mask.

### Modem Interrupt (`UARTMSINTR`)

A change on any of `nUARTCTS`, `nUARTDCD`, `nUARTDSR`, or `nUARTRI` raises
`UARTMSINTR`. Clear by writing 1 to the relevant bit in `UARTICR`. RP2350
does not route most of the modem signals to GPIO, so only CTS is normally
relevant.

### Receive Interrupt (`UARTRXINTR`)

- FIFO mode (`FEN=1`): asserts when RX FIFO ≥ `RXIFLSEL` watermark.
  Cleared automatically by reading enough bytes; or manually via `RXIC`.
- Holding-register mode (`FEN=0`): asserts on any received byte.

### Transmit Interrupt (`UARTTXINTR`)

- FIFO mode: asserts when TX FIFO ≤ `TXIFLSEL` watermark.
- Holding-register mode: asserts when TX holding register empty.

> **Note (transition-based):** The TX interrupt fires on a **level
> transition**, not a steady level. If interrupts are enabled before any
> data is written, `UARTTXINTR` is **not** initially asserted. You must
> either prime the FIFO before enabling, or do an initial DMA push, or use
> a separate "kick" mechanism.

### Receive Timeout (`UARTRTINTR`)

Asserted when RX FIFO is non-empty and no new character has been received
in 32 bit-times. Cleared when:

- Software reads the FIFO empty.
- Software writes 1 to `UARTICR.RTIC`.

Useful for closing variable-length protocol frames (e.g., NMEA newlines).

### Error Interrupt (`UARTEINTR`)

Combined source — read `UARTRIS` / `UARTMIS` to discriminate among framing,
parity, break, and overrun. Clear via the corresponding bits in `UARTICR`
(bits 7-10).

## NVIC Connection

Each instance has one IRQ output:

| Symbolic | Vector |
|----------|--------|
| `UART0_IRQ` | UART0 combined |
| `UART1_IRQ` | UART1 combined |

Two parallel non-secure IRQ lines exist for TrustZone configurations
(`UART0_IRQ_NS`, `UART1_IRQ_NS`). FT1 FSW runs in a single-security context,
so use the secure variant.

## FT1 Driver Recommendation

For the GPS link (9600 baud, line-oriented NMEA):

1. Set `UARTIFLS.RXIFLSEL = 1/8` (interrupt at 4 bytes RX FIFO depth) so
   short NMEA fragments still wake the handler.
2. Enable `UARTIMSC.RXIM` and `UARTIMSC.RTIM` (RX FIFO + receive timeout).
3. Optionally enable error interrupts (`OEIM`, `FEIM`, `PEIM`, `BEIM`) and
   log them to a diagnostic counter.
4. Disable modem interrupts.
5. In the ISR, drain the RX FIFO until empty; write to `UARTICR.RTIC` to
   clear the timeout.

## See Also

- [`02_operation.md`](02_operation.md) — Init order.
- [`04_registers.md`](04_registers.md) — `UARTIMSC`, `UARTICR`, etc.
