# 03 - FT1 Pin Assignment

> Authored from public Raspberry Pi Pico 2 board pinout (source PDF
> `../../Pico-2-Pinout.pdf` is image-based) and from the FT1 avionics wiring
> captured in [`../avionics.md`](../avionics.md). For any alt-function detail
> not visible on the public reference card, see
> `[verify against rp2350-datasheet.pdf chapter 9.11]`.

This file is the **single source of truth** for which Pico 2 GPIO pins FT1
firmware is permitted to drive, and what each pin is wired to on the FT1
avionics PCB.

## FT1 Bus Allocation Summary

| Bus | Pins (GP) | Phy Pins | FT1 Use |
|-----|-----------|----------|---------|
| UART0 | GP0 (TX), GP1 (RX) | 1, 2 | LoRa radio (RYLR896) |
| UART1 | GP4 (TX), GP5 (RX) | 6, 7 | GPS module (FGPMMOPA6H) |
| I2C0  | GP8 (SDA), GP9 (SCL) | 11, 12 | Barometer + IMU (shared bus) |
| SPI0  | GP16 (MISO), GP17 (CS), GP18 (SCK), GP19 (MOSI) | 21, 22, 24, 25 | SD card |

## Detailed Pin Map (FT1)

### UART0 - LoRa Radio (RYLR896)

| Phy Pin | GP   | Signal     | Wired to RYLR896 | Direction (Pico view) | Notes |
|---------|------|------------|------------------|------------------------|-------|
| 1       | GP0  | UART0 TX   | RYLR896 RXD       | Output | Command/data to radio |
| 2       | GP1  | UART0 RX   | RYLR896 TXD       | Input  | Replies/notifications from radio |

- Default Pico convention places UART0 on GP0/GP1, so no alt-function mux
  reconfiguration beyond `GPIO_FUNC_UART` is required.
- LoRa baud rate per FT1 avionics doc.

### UART1 - GPS (GlobalTop FGPMMOPA6H)

| Phy Pin | GP   | Signal     | Wired to GPS module | Direction (Pico view) | Notes |
|---------|------|------------|----------------------|------------------------|-------|
| 6       | GP4  | UART1 TX   | GPS RX               | Output | Configuration / PMTK commands to GPS |
| 7       | GP5  | UART1 RX   | GPS TX               | Input  | NMEA stream from GPS |

- Default Pico convention places UART1 on GP4/GP5.
- GPS default baud rate 9600; raise via PMTK if higher rate required.

### I2C0 - Barometer + IMU (shared)

| Phy Pin | GP   | Signal   | Wired to | Direction | Notes |
|---------|------|----------|----------|-----------|-------|
| 11      | GP8  | I2C0 SDA | Baro + IMU SDA | Bidir | Pull-up resistor on FT1 PCB |
| 12      | GP9  | I2C0 SCL | Baro + IMU SCL | Output (open-drain) | Pull-up resistor on FT1 PCB |

- Barometer and IMU share the I2C0 bus; their I2C addresses must not collide
  (verified at avionics integration time).
- I2C0 on GP8/GP9 is an **alt mapping**, not the lowest-numbered Pico
  default (GP4/GP5). GP4/GP5 are reserved for GPS, so I2C0 is moved up.

### SPI0 - SD Card

| Phy Pin | GP   | Signal       | Wired to SD socket | Direction (Pico view) | Notes |
|---------|------|--------------|--------------------|------------------------|-------|
| 21      | GP16 | SPI0 RX (MISO) | SD DO (DATA0)     | Input  | |
| 22      | GP17 | SPI0 CSn       | SD CS             | Output | **CS is on GP17 specifically** |
| 24      | GP18 | SPI0 SCK       | SD CLK            | Output | |
| 25      | GP19 | SPI0 TX (MOSI) | SD DI (CMD)       | Output | |

- **CS = GP17** is fixed for FT1; do not relocate without an avionics ECN.
- SD card is operated in SPI mode (not SDIO).

## FT1 Pin Allocation Block Diagram

```
                          Pico 2 (RP2350)
                       +---------------------+
   LoRa RYLR896  RXD --|GP0  (UART0 TX)      |
                  TXD --|GP1  (UART0 RX)      |
                        |                     |
   GPS  FGPMMOPA6H RX --|GP4  (UART1 TX)      |
                  TX --|GP5  (UART1 RX)      |
                        |                     |
   Baro+IMU  SDA  <-->--|GP8  (I2C0 SDA)      |
            SCL  <-->--|GP9  (I2C0 SCL)      |
                        |                     |
   SD DO         <-----|GP16 (SPI0 MISO)     |
   SD CS         <-----|GP17 (SPI0 CSn)      |
   SD CLK        <-----|GP18 (SPI0 SCK)      |
   SD DI         <-----|GP19 (SPI0 MOSI)     |
                       +---------------------+
```

## Pins Reserved (do NOT reassign without ECN)

The following GPIOs are FT1-allocated and must not be repurposed in firmware
or test fixtures without a coordinated change with the avionics owner:

- GP0, GP1 (LoRa)
- GP4, GP5 (GPS)
- GP8, GP9 (I2C0 baro+IMU)
- GP16, GP17, GP18, GP19 (SPI0 SD card)

## Pins Available for Future FT1 Use

The following header GPIOs are unallocated on FT1 and available for future
expansion (sensors, debug headers, status LEDs, etc.):

- GP2, GP3, GP6, GP7
- GP10, GP11, GP12, GP13, GP14, GP15
- GP20, GP21, GP22
- GP26 (ADC0), GP27 (ADC1), GP28 (ADC2) - prefer for analog sensing

## Cross-Reference Checks

When changing FT1 wiring, every entry in this file must be cross-checked
against:

1. [`../avionics.md`](../avionics.md) - FT1 avionics wiring authoritative
   source for connector/header mappings.
2. [`./01_physical_pinout.md`](./01_physical_pinout.md) - confirm physical
   pin numbers match.
3. [`./02_peripheral_alt_functions.md`](./02_peripheral_alt_functions.md) -
   confirm the chosen GPIO can host the chosen peripheral signal.
4. `docs/rp2350-datasheet.pdf` chapter 9.11 - source-of-truth for any alt
   function flagged `[verify against rp2350-datasheet.pdf chapter 9.11]`.

## Back-Links

- [Index](./index.md)
- [01 Physical Pinout](./01_physical_pinout.md)
- [02 Peripheral Alt Functions](./02_peripheral_alt_functions.md)
- [Avionics ICD](../avionics.md)
- [Source PDF](../../Pico-2-Pinout.pdf)
