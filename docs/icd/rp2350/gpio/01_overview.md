# GPIO: Overview & Function Select

[Back to GPIO index](index.md) | [Back to ICD index](../index.md)

## 9.1. Overview

RP2350 has up to 54 multi-functional GPIO pins, divided into two banks:

| Bank | QFN-60 (RP2350A) | QFN-80 |
|------|------------------|--------|
| Bank 0 (User IO) | 30 user GPIOs | 48 user GPIOs |
| Bank 1 (QSPI + USB) | 6 QSPI IOs + USB DP/DM | same |

ADC-capable Bank 0 pins:
- QFN-60: GPIOs 26-29 (4 channels)
- QFN-80: GPIOs 40-47 (8 channels)

Each Bank-0 GPIO can be assigned to one of: SIO (software), PIO0/1/2,
SPI0/1, UART0/1, I2C0/1, PWM A/B, HSTX, CLOCK GPIN/GPOUT, USB control,
QMI CS1n (auxiliary chip select), CoreSight trace, or external interrupt.

Each Bank-1 GPIO supports: SIO, QMI (XIP), UART, I2C.

## 9.3. Reset State

At first power-up, all Bank-0 IOs are:

- Output buffer high-impedance.
- Input buffer **disabled** (`IE=0`).
- Pulled low.
- Isolation latches **set** (`ISO=1`).
- IO mux at the null function (`FUNCSEL=0x1f`).

Bank-1 IOs have the same state except `IE=1` and pull-up/down differs:
SCK, SD0, SD1 are pull-down; SD2, SD3, CSn are pull-up.

> **FSW must** set `IE=1` and clear `ISO=0` (in the pad register) before any
> digital I/O. `gpio_set_function()` does both.

Pads return to reset state on:

- Brownout reset
- RUN pin asserted low
- SW-DP CDBGRSTREQ
- RP-AP rescue reset

## 9.4. Function Select

Allocate a function by writing the `FUNCSEL` field in the `GPIOn_CTRL`
register. Each GPIO has its own CTRL register (see `04_registers.md`). Each
peripheral input must be selected by **only one** GPIO at a time; otherwise
the peripheral receives the OR of all sources.

### Bank 0 Function Table (FT1-relevant subset)

The full table covers F0..F11 for GPIOs 0-47. The subset below shows the
peripheral functions the FT1 FSW will use (SPIx, UARTx, I2Cx, SIO, CLOCK
GPIN/GPOUT). HSTX, PIO, PWM, USB, and Trace columns are omitted.

| GPIO | F1 (SPI) | F2 (UART) | F3 (I2C) | F5 (SIO) | F9/F11 (Other) |
|------|----------|-----------|----------|----------|----------------|
| 0  | SPI0 RX  | UART0 TX  | I2C0 SDA | SIO | — |
| 1  | SPI0 CSn | UART0 RX  | I2C0 SCL | SIO | — |
| 2  | SPI0 SCK | UART0 CTS | I2C1 SDA | SIO | F11=UART0 TX |
| 3  | SPI0 TX  | UART0 RTS | I2C1 SCL | SIO | F11=UART0 RX |
| 4  | SPI0 RX  | UART1 TX  | I2C0 SDA | SIO | — |
| 5  | SPI0 CSn | UART1 RX  | I2C0 SCL | SIO | — |
| 6  | SPI0 SCK | UART1 CTS | I2C1 SDA | SIO | F11=UART1 TX |
| 7  | SPI0 TX  | UART1 RTS | I2C1 SCL | SIO | F11=UART1 RX |
| 8  | SPI1 RX  | UART1 TX  | I2C0 SDA | SIO | — |
| 9  | SPI1 CSn | UART1 RX  | I2C0 SCL | SIO | — |
| 10 | SPI1 SCK | UART1 CTS | I2C1 SDA | SIO | F11=UART1 TX |
| 11 | SPI1 TX  | UART1 RTS | I2C1 SCL | SIO | F11=UART1 RX |
| 12 | SPI1 RX  | UART0 TX  | I2C0 SDA | SIO | F9=CLOCK GPIN0 |
| 13 | SPI1 CSn | UART0 RX  | I2C0 SCL | SIO | F9=CLOCK GPOUT0 |
| 14 | SPI1 SCK | UART0 CTS | I2C1 SDA | SIO | F9=CLOCK GPIN1, F11=UART0 TX |
| 15 | SPI1 TX  | UART0 RTS | I2C1 SCL | SIO | F9=CLOCK GPOUT1, F11=UART0 RX |
| 16 | SPI0 RX  | UART0 TX  | I2C0 SDA | SIO | — |
| 17 | SPI0 CSn | UART0 RX  | I2C0 SCL | SIO | — |
| 18 | SPI0 SCK | UART0 CTS | I2C1 SDA | SIO | F11=UART0 TX |
| 19 | SPI0 TX  | UART0 RTS | I2C1 SCL | SIO | F11=UART0 RX |
| 20 | SPI0 RX  | UART1 TX  | I2C0 SDA | SIO | F9=CLOCK GPIN0 |
| 21 | SPI0 CSn | UART1 RX  | I2C0 SCL | SIO | F9=CLOCK GPOUT0 |
| 22 | SPI0 SCK | UART1 CTS | I2C1 SDA | SIO | F9=CLOCK GPIN1, F11=UART1 TX |
| 23 | SPI0 TX  | UART1 RTS | I2C1 SCL | SIO | F9=CLOCK GPOUT1, F11=UART1 RX |
| 24 | SPI1 RX  | UART1 TX  | I2C0 SDA | SIO | F9=CLOCK GPOUT2 |
| 25 | SPI1 CSn | UART1 RX  | I2C0 SCL | SIO | F9=CLOCK GPOUT3 |
| 26 | SPI1 SCK | UART1 CTS | I2C1 SDA | SIO | F11=UART1 TX |
| 27 | SPI1 TX  | UART1 RTS | I2C1 SCL | SIO | F11=UART1 RX |
| 28 | SPI1 RX  | UART0 TX  | I2C0 SDA | SIO | — |
| 29 | SPI1 CSn | UART0 RX  | I2C0 SCL | SIO | — |

(GPIOs 30-47 follow the same pattern; QFN-80 only. See source PDF table 644.)

### Bank 0 Function Numeric Encoding

| Code | Function |
|------|----------|
| 0 | HSTX (where applicable) |
| 1 | SPI |
| 2 | UART |
| 3 | I2C |
| 4 | PWM |
| 5 | SIO |
| 6 | PIO0 |
| 7 | PIO1 |
| 8 | PIO2 |
| 9 | CLOCK GPIN/GPOUT or QMI CS1n |
| 10 | USB control |
| 11 | UART (alt mapping for some pins) |
| 31 (`0x1f`) | NULL (default) |

### Bank 1 Function Table (informational)

The QSPI bank is normally driven by the QMI peripheral. Trimmed entries:

| Pin | F1 (peripheral) | F2 (UART) | F3 (I2C) | F5 (SIO) |
|-----|-----------------|-----------|----------|----------|
| USB DP    | UART1 TX | I2C0 SDA | — | SIO |
| USB DM    | UART1 RX | I2C0 SCL | — | SIO |
| QSPI SCK  | QMI SCK  | UART1 CTS | I2C1 SDA | SIO |
| QSPI CSn  | QMI CS0n | UART1 RTS | I2C1 SCL | SIO |
| QSPI SD0  | QMI SD0  | UART0 TX  | I2C0 SDA | SIO |
| QSPI SD1  | QMI SD1  | UART0 RX  | I2C0 SCL | SIO |
| QSPI SD2  | QMI SD2  | UART0 CTS | I2C1 SDA | SIO |
| QSPI SD3  | QMI SD3  | UART0 RTS | I2C1 SCL | SIO |

> The FT1 SD card is on **Bank 0 SPI**, not the QSPI bank. The QSPI bank
> remains under QMI for external boot flash.

## 9.5. Interrupts (Summary)

Four interrupt scenarios per GPIO: Level High, Level Low, Edge High, Edge
Low. Edge interrupts are latched in `INTR` and cleared by writing `INTR`.
Level interrupts are not latched.

Three destinations: proc0, proc1, dormant_wake. Each has its own
`INTE` / `INTS` / `INTF` arrays. See [`03_interrupts.md`](03_interrupts.md).

## See Also

- [`02_pads.md`](02_pads.md) — Pad electrical control, isolation, SIO.
- [`03_interrupts.md`](03_interrupts.md) — Interrupt configuration.
- [`04_registers.md`](04_registers.md) — Register layouts.
