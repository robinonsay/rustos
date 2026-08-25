# UART: Register Map

[Back to UART index](index.md) | [Back to ICD index](../index.md)

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| UART0 | `UART0_BASE` | `0x40070000` |
| UART1 | `UART1_BASE` | `0x40078000` |

## 12.1.8. Register List

| Offset | Name | Description |
|--------|------|-------------|
| 0x000 | `UARTDR` | Data register |
| 0x004 | `UARTRSR/UARTECR` | Receive status / error clear |
| 0x018 | `UARTFR` | Flag register |
| 0x020 | `UARTILPR` | IrDA low-power counter (unused) |
| 0x024 | `UARTIBRD` | Integer baud rate divisor |
| 0x028 | `UARTFBRD` | Fractional baud rate divisor |
| 0x02c | `UARTLCR_H` | Line control |
| 0x030 | `UARTCR` | Control register |
| 0x034 | `UARTIFLS` | Interrupt FIFO level select |
| 0x038 | `UARTIMSC` | Interrupt mask set/clear |
| 0x03c | `UARTRIS` | Raw interrupt status |
| 0x040 | `UARTMIS` | Masked interrupt status |
| 0x044 | `UARTICR` | Interrupt clear |
| 0x048 | `UARTDMACR` | DMA control |
| 0xfe0..0xfec | `UARTPERIPHID0..3` | RO PrimeCell ID |
| 0xff0..0xffc | `UARTPCELLID0..3` | RO PrimeCell ID |

## UARTDR (0x000)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 11 | `OE` | RO | Overrun error |
| 10 | `BE` | RO | Break error |
| 9 | `PE` | RO | Parity error |
| 8 | `FE` | RO | Framing error |
| 7:0 | `DATA` | RWF | Read=RX, Write=TX |

## UARTRSR / UARTECR (0x004)

Read returns receive-status. Write to this address acts as the error
clear (`UARTECR`).

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3 | `OE` | WC | 0 | Overrun |
| 2 | `BE` | WC | 0 | Break |
| 1 | `PE` | WC | 0 | Parity |
| 0 | `FE` | WC | 0 | Framing |

## UARTFR — Flag Register (0x018)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 8 | `RI` | RO | — | Ring indicator (complement of nUARTRI) |
| 7 | `TXFE` | RO | 1 | TX FIFO empty (or holding register empty) |
| 6 | `RXFF` | RO | 0 | RX FIFO full |
| 5 | `TXFF` | RO | 0 | TX FIFO full |
| 4 | `RXFE` | RO | 1 | RX FIFO empty |
| 3 | `BUSY` | RO | 0 | UART busy transmitting |
| 2 | `DCD` | RO | — | Data carrier detect |
| 1 | `DSR` | RO | — | Data set ready |
| 0 | `CTS` | RO | — | Clear to send |

## UARTIBRD (0x024)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15:0 | `BAUD_DIVINT` | RW | 0x0000 | Integer baud divisor |

## UARTFBRD (0x028)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 5:0 | `BAUD_DIVFRAC` | RW | 0x00 | Fractional baud divisor |

## UARTLCR_H — Line Control (0x02c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7 | `SPS` | RW | 0 | Stick parity select |
| 6:5 | `WLEN` | RW | 0 | Word length: 0=5b, 1=6b, 2=7b, 3=8b |
| 4 | `FEN` | RW | 0 | Enable FIFOs (1 = 32-deep, 0 = 1-byte holding) |
| 3 | `STP2` | RW | 0 | 0 = 1 stop bit, 1 = 2 stop bits |
| 2 | `EPS` | RW | 0 | Even parity (when `PEN=1`) |
| 1 | `PEN` | RW | 0 | Parity enable |
| 0 | `BRK` | RW | 0 | Send break (drives line low) |

> Writing `UARTLCR_H` latches the baud-rate divisors written in `UARTIBRD`
> and `UARTFBRD`. Always do an LCR_H write after changing baud divisors.

## UARTCR — Control (0x030)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15 | `CTSEN` | RW | 0 | Enable CTS hardware flow control |
| 14 | `RTSEN` | RW | 0 | Enable RTS hardware flow control |
| 13 | `OUT2` | RW | 0 | Modem OUT2 (RI for DTE) |
| 12 | `OUT1` | RW | 0 | Modem OUT1 (DCD for DTE) |
| 11 | `RTS` | RW | 0 | Software RTS (when not in flow-control mode) |
| 10 | `DTR` | RW | 0 | DTR |
| 9 | `RXE` | RW | 1 | RX enable |
| 8 | `TXE` | RW | 1 | TX enable |
| 7 | `LBE` | RW | 0 | Loopback enable |
| 2 | `SIRLP` | RW | 0 | IrDA SIR low-power (unused on RP2350) |
| 1 | `SIREN` | RW | 0 | IrDA SIR enable (unused) |
| 0 | `UARTEN` | RW | 0 | UART enable |

## UARTIFLS — FIFO Level Select (0x034)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 5:3 | `RXIFLSEL` | RW | 2 | RX trigger: 0=⅛, 1=¼, 2=½, 3=¾, 4=⅞ |
| 2:0 | `TXIFLSEL` | RW | 2 | TX trigger: 0=⅛, 1=¼, 2=½, 3=¾, 4=⅞ |

## UARTIMSC / UARTRIS / UARTMIS / UARTICR (0x038, 0x03c, 0x040, 0x044)

All four registers share the same bit layout:

| Bit | Field | Description |
|-----|-------|-------------|
| 10 | `OEIM/OERIS/OEMIS/OEIC` | Overrun error |
| 9 | `BEIM/BERIS/BEMIS/BEIC` | Break error |
| 8 | `PEIM/PERIS/PEMIS/PEIC` | Parity error |
| 7 | `FEIM/FERIS/FEMIS/FEIC` | Framing error |
| 6 | `RTIM/RTRIS/RTMIS/RTIC` | Receive timeout |
| 5 | `TXIM/TXRIS/TXMIS/TXIC` | Transmit FIFO trigger |
| 4 | `RXIM/RXRIS/RXMIS/RXIC` | Receive FIFO trigger |
| 3 | `DSRMIM/DSRRMIS/DSRMMIS/DSRMIC` | Modem DSR change |
| 2 | `DCDMIM/DCDRMIS/DCDMMIS/DCDMIC` | Modem DCD change |
| 1 | `CTSMIM/CTSRMIS/CTSMMIS/CTSMIC` | Modem CTS change |
| 0 | `RIMIM/RIRMIS/RIMMIS/RIMIC` | Modem RI change |

- `UARTIMSC`: RW (1 = unmasked, fires CPU IRQ).
- `UARTRIS`: RO (raw status).
- `UARTMIS`: RO (status after masking).
- `UARTICR`: WC (write 1 to clear the latched flag).

## UARTDMACR (0x048)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 2 | `DMAONERR` | RW | 0 | Mask RX DMA req when error interrupt asserted |
| 1 | `TXDMAE` | RW | 0 | Enable TX DMA request lines |
| 0 | `RXDMAE` | RW | 0 | Enable RX DMA request lines |

## UARTPERIPHID / UARTPCELLID (0xfe0–0xffc)

Read-only PrimeCell identification block. Reads of:

| Register | Value |
|----------|-------|
| `UARTPERIPHID0` | 0x11 |
| `UARTPERIPHID1` | 0x10 |
| `UARTPERIPHID2` | 0x34 (rev r1p5 → revision field 0x3) |
| `UARTPERIPHID3` | 0x00 |
| `UARTPCELLID0` | 0x0d |
| `UARTPCELLID1` | 0xf0 |
| `UARTPCELLID2` | 0x05 |
| `UARTPCELLID3` | 0xb1 |

These are useful as a sanity check during driver bring-up.

## See Also

- [`02_operation.md`](02_operation.md) — Init sequence using these registers.
- [`03_interrupts.md`](03_interrupts.md) — Interrupt handling.
