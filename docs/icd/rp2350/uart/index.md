# UART (Section 12.1)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Functional description, FIFOs, framing, flow control |
| [`02_operation.md`](02_operation.md) | Operation modes (TX/RX), baud rate, init sequence, DMA |
| [`03_interrupts.md`](03_interrupts.md) | Interrupt sources & combined IRQ |
| [`04_registers.md`](04_registers.md) | Register map and bit fields |

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| UART0 | `UART0_BASE` | `0x40070000` |
| UART1 | `UART1_BASE` | `0x40078000` |

## Peripheral Identity

PrimeCell PL011 (Revision r1p5). Two identical instances. PL011 modem mode
and IrDA mode are **not supported** on RP2350.

## Key Specifications

| Property | Value |
|----------|-------|
| TX FIFO | 32 × 8-bit |
| RX FIFO | 32 × 12-bit (8-bit data + 4 status bits) |
| Word length | 5 / 6 / 7 / 8 bits |
| Stop bits | 1 or 2 |
| Parity | none / even / odd / stick |
| Hardware flow control | optional RTS / CTS |
| Max baud rate | UARTCLK / 16 (≈7.8 Mbaud at 125 MHz `clk_peri`) |
| Reference clock | `clk_peri` (UARTCLK) |
| Bus clock | `clk_sys` (PCLK) |

## FT1 Driver Notes

- The GPS receiver (FGPMMOPA6H) outputs NMEA at 9600 baud. The UART driver
  must support `UART_PARITY_NONE`, 8 data bits, 1 stop bit.
- Hardware flow control is **not** used on the GPS link (the GPS module has
  no flow-control lines). Configure `CTSEN=0`, `RTSEN=0`.
- DMA is recommended for RX to avoid losing characters under TDM jitter,
  but FT1 may use FIFO-poll if scheduler latency is bounded.
- Set FIFO trigger levels conservatively (`RXIFLSEL=1/4` or `1/8`) for
  low-latency responsiveness on slow links.

## Cross-References

- Pin muxing: see [`../gpio/01_overview.md`](../gpio/01_overview.md).
- Clock setup (`clk_peri`): see [`../clocks/02_programming.md`](../clocks/02_programming.md).
