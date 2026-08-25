# 02 - Peripheral Alternate Functions

> Authored from public Raspberry Pi Pico 2 board pinout. The source PDF
> (`../../Pico-2-Pinout.pdf`) is image-based; this content is reconstructed
> textually. Alt-function assignments that are not certain from the public
> reference card are annotated
> `[verify against rp2350-datasheet.pdf chapter 9.11]`.

The RP2350 routes each peripheral to multiple GPIO pins via the GPIO function
mux. This file lists, **per peripheral**, which **header-exposed** GPx pins
can carry which signal of that peripheral. On-board reserved GPIOs (GP23 -
GP25) and unrouted GPIOs (e.g. GP47) are excluded.

The convention is: **a peripheral signal can appear on any GPIO whose number
modulo 4 matches the signal's row offset for that peripheral type.** This is
a property of the RP2040/RP2350 IO mux. Specifically:

- UART: TX is on GPIOs where `GP mod 4 == 0`; RX on `GP mod 4 == 1`; CTS on
  `GP mod 4 == 2`; RTS on `GP mod 4 == 3`. UART0 vs UART1 selection is by
  GPIO number range.
- I2C: SDA on even-numbered GPIOs; SCL on odd-numbered GPIOs.
- SPI: RX on `GP mod 4 == 0`; CSn on `GP mod 4 == 1`; SCK on `GP mod 4 == 2`;
  TX on `GP mod 4 == 3`. SPI0 vs SPI1 selection is by GPIO number range.

The tables below list the **specific** pin/peripheral combinations exposed
on the Pico 2 board header.

---

## UART0

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP0  | 1  | UART0 TX | **Default Pico convention; FT1: LoRa TX** |
| GP1  | 2  | UART0 RX | **Default Pico convention; FT1: LoRa RX** |
| GP2  | 4  | UART0 CTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP3  | 5  | UART0 RTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP12 | 16 | UART0 TX | Alt mapping |
| GP13 | 17 | UART0 RX | Alt mapping |
| GP14 | 19 | UART0 CTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP15 | 20 | UART0 RTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP16 | 21 | UART0 TX | Alt mapping (collides with FT1 SPI0 MISO use) |
| GP17 | 22 | UART0 RX | Alt mapping (collides with FT1 SPI0 CS use) |

## UART1

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP4  | 6  | UART1 TX | **Default Pico convention; FT1: GPS TX** |
| GP5  | 7  | UART1 RX | **Default Pico convention; FT1: GPS RX** |
| GP6  | 9  | UART1 CTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP7  | 10 | UART1 RTS | `[verify against rp2350-datasheet.pdf chapter 9.11]` |
| GP8  | 11 | UART1 TX | Alt mapping (collides with FT1 I2C0 SDA use) |
| GP9  | 12 | UART1 RX | Alt mapping (collides with FT1 I2C0 SCL use) |
| GP20 | 26 | UART1 TX | Alt mapping |
| GP21 | 27 | UART1 RX | Alt mapping |

---

## I2C0

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP0  | 1  | I2C0 SDA | Collides with FT1 LoRa UART0 TX |
| GP1  | 2  | I2C0 SCL | Collides with FT1 LoRa UART0 RX |
| GP4  | 6  | I2C0 SDA | Collides with FT1 GPS UART1 TX |
| GP5  | 7  | I2C0 SCL | Collides with FT1 GPS UART1 RX |
| GP8  | 11 | I2C0 SDA | **FT1: baro + IMU SDA** |
| GP9  | 12 | I2C0 SCL | **FT1: baro + IMU SCL** |
| GP12 | 16 | I2C0 SDA | Alt mapping |
| GP13 | 17 | I2C0 SCL | Alt mapping |
| GP16 | 21 | I2C0 SDA | Alt mapping |
| GP17 | 22 | I2C0 SCL | Alt mapping |
| GP20 | 26 | I2C0 SDA | Alt mapping |
| GP21 | 27 | I2C0 SCL | Alt mapping |
| GP28 | 34 | I2C0 SDA | Shared with ADC2 - choose one function |

## I2C1

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP2  | 4  | I2C1 SDA | |
| GP3  | 5  | I2C1 SCL | |
| GP6  | 9  | I2C1 SDA | |
| GP7  | 10 | I2C1 SCL | |
| GP10 | 14 | I2C1 SDA | |
| GP11 | 15 | I2C1 SCL | |
| GP14 | 19 | I2C1 SDA | |
| GP15 | 20 | I2C1 SCL | |
| GP18 | 24 | I2C1 SDA | Collides with FT1 SPI0 SCK use |
| GP19 | 25 | I2C1 SCL | Collides with FT1 SPI0 MOSI use |
| GP22 | 29 | I2C1 SDA | |
| GP26 | 31 | I2C1 SDA | Shared with ADC0 |
| GP27 | 32 | I2C1 SCL | Shared with ADC1 |

---

## SPI0

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP0  | 1  | SPI0 RX (MISO)  | Collides with FT1 LoRa UART0 TX |
| GP1  | 2  | SPI0 CSn        | Collides with FT1 LoRa UART0 RX |
| GP2  | 4  | SPI0 SCK        | |
| GP3  | 5  | SPI0 TX (MOSI)  | |
| GP4  | 6  | SPI0 RX (MISO)  | Collides with FT1 GPS UART1 TX |
| GP5  | 7  | SPI0 CSn        | Collides with FT1 GPS UART1 RX |
| GP6  | 9  | SPI0 SCK        | |
| GP7  | 10 | SPI0 TX (MOSI)  | |
| GP16 | 21 | SPI0 RX (MISO)  | **FT1: SD MISO** |
| GP17 | 22 | SPI0 CSn        | **FT1: SD CS** |
| GP18 | 24 | SPI0 SCK        | **FT1: SD SCK** |
| GP19 | 25 | SPI0 TX (MOSI)  | **FT1: SD MOSI** |
| GP20 | 26 | SPI0 RX (MISO)  | Alt mapping |
| GP21 | 27 | SPI0 CSn        | Alt mapping |
| GP22 | 29 | SPI0 SCK        | Alt mapping |

## SPI1

| GP   | Phy Pin | Signal | Notes |
|------|---------|--------|-------|
| GP8  | 11 | SPI1 RX (MISO)  | Collides with FT1 I2C0 SDA |
| GP9  | 12 | SPI1 CSn        | Collides with FT1 I2C0 SCL |
| GP10 | 14 | SPI1 SCK        | |
| GP11 | 15 | SPI1 TX (MOSI)  | |
| GP12 | 16 | SPI1 RX (MISO)  | |
| GP13 | 17 | SPI1 CSn        | |
| GP14 | 19 | SPI1 SCK        | |
| GP15 | 20 | SPI1 TX (MOSI)  | |
| GP26 | 31 | SPI1 SCK        | Shared with ADC0 |
| GP27 | 32 | SPI1 TX (MOSI)  | Shared with ADC1 |
| GP28 | 34 | SPI1 RX (MISO)  | Shared with ADC2 |

---

## PIO0 and PIO1

The RP2350 has **two PIO blocks** (PIO0 and PIO1), each with **4 state
machines** (one more block / SM count vs RP2040's three blocks of 4).
`[verify against rp2350-datasheet.pdf chapter 9.11]`

PIO is **fully programmable**: any state machine in either PIO block can drive
or sample **any GPIO from GP0 through GP47** (subject to the chip's pin mux),
limited on the Pico 2 board to the GPIOs actually routed to the header
(GP0 - GP22, GP26 - GP28).

| Peripheral | Available GPIOs (Pico 2 header) |
|------------|---------------------------------|
| PIO0 | GP0 - GP22, GP26 - GP28 |
| PIO1 | GP0 - GP22, GP26 - GP28 |

Practical guidance:

- PIO is the right choice for non-standard protocols (WS2812, custom serial,
  parallel buses, precise pulse generation, software-defined SPI/UART).
- PIO0 and PIO1 can address the **same** GPIO but only one block / SM should
  drive a given pin at a time.
- When a GPIO is allocated to PIO, it cannot simultaneously be in use by the
  fixed-function UART/SPI/I2C peripherals.

---

## Function-Conflict Summary

The Pico 2 IO mux means a single GPIO often appears in many peripheral tables.
At runtime, **only one function is muxed onto a pin at a time**. The
"Notes" columns above flag the FT1 collisions; engineers must consult both
this file and `03_ft1_assignment.md` before reassigning any FT1 pin.

## Back-Links

- [Index](./index.md)
- [01 Physical Pinout](./01_physical_pinout.md)
- [03 FT1 Assignment](./03_ft1_assignment.md)
- [Source PDF](../../Pico-2-Pinout.pdf)
