# 01 - Physical Pinout (40-pin Header + Debug)

> Authored from public Raspberry Pi Pico 2 board pinout. Source PDF
> (`../../Pico-2-Pinout.pdf`) is image-based and could not be text-extracted
> beyond the color legend. Where an alternate-function assignment is not
> directly visible on the public reference card, it is annotated
> `[verify against rp2350-datasheet.pdf chapter 9.11]`.

## Coordinate System

Pin numbering follows the standard Pico convention:

- Physical pins **1 - 20** run **down the left edge** (USB connector at top).
- Physical pins **21 - 40** run **up the right edge** (pin 21 = bottom-right,
  pin 40 = top-right next to USB).
- Pin 1 is at the top-left, indicated by a chamfered/square pad on the PCB
  silkscreen.

The 3-pin **debug header** sits at the bottom edge of the board, between the
two 20-pin rows.

## 40-Pin Through-Hole Header

| Phy Pin | Pico Label | GP / Function | Default UART/SPI/I2C alt | Notes |
|---------|------------|---------------|--------------------------|-------|
| 1  | GP0  | GP0  | UART0 TX / I2C0 SDA / SPI0 RX  | FT1: LoRa UART0 TX |
| 2  | GP1  | GP1  | UART0 RX / I2C0 SCL / SPI0 CSn | FT1: LoRa UART0 RX |
| 3  | GND  | GND  | -                              | Ground |
| 4  | GP2  | GP2  | I2C1 SDA / SPI0 SCK / PIO       | General-purpose |
| 5  | GP3  | GP3  | I2C1 SCL / SPI0 TX / PIO        | General-purpose |
| 6  | GP4  | GP4  | UART1 TX / I2C0 SDA / SPI0 RX   | FT1: GPS UART1 TX |
| 7  | GP5  | GP5  | UART1 RX / I2C0 SCL / SPI0 CSn  | FT1: GPS UART1 RX |
| 8  | GND  | GND  | -                               | Ground |
| 9  | GP6  | GP6  | I2C1 SDA / SPI0 SCK / PIO       | General-purpose |
| 10 | GP7  | GP7  | I2C1 SCL / SPI0 TX / PIO        | General-purpose |
| 11 | GP8  | GP8  | UART1 TX / I2C0 SDA / SPI1 RX   | FT1: I2C0 SDA (baro + IMU) |
| 12 | GP9  | GP9  | UART1 RX / I2C0 SCL / SPI1 CSn  | FT1: I2C0 SCL (baro + IMU) |
| 13 | GND  | GND  | -                               | Ground |
| 14 | GP10 | GP10 | I2C1 SDA / SPI1 SCK / PIO       | General-purpose |
| 15 | GP11 | GP11 | I2C1 SCL / SPI1 TX / PIO        | General-purpose |
| 16 | GP12 | GP12 | UART0 TX / I2C0 SDA / SPI1 RX   | UART0 alt mapping |
| 17 | GP13 | GP13 | UART0 RX / I2C0 SCL / SPI1 CSn  | UART0 alt mapping |
| 18 | GND  | GND  | -                               | Ground |
| 19 | GP14 | GP14 | I2C1 SDA / SPI1 SCK / PIO       | General-purpose |
| 20 | GP15 | GP15 | I2C1 SCL / SPI1 TX / PIO        | General-purpose |
| 21 | GP16 | GP16 | UART0 TX / I2C0 SDA / SPI0 RX   | FT1: SPI0 MISO (SD card) |
| 22 | GP17 | GP17 | UART0 RX / I2C0 SCL / SPI0 CSn  | FT1: SPI0 CS (SD card) |
| 23 | GND  | GND  | -                               | Ground |
| 24 | GP18 | GP18 | I2C1 SDA / SPI0 SCK / PIO       | FT1: SPI0 SCK (SD card) |
| 25 | GP19 | GP19 | I2C1 SCL / SPI0 TX / PIO        | FT1: SPI0 MOSI (SD card) |
| 26 | GP20 | GP20 | UART1 TX / I2C0 SDA / SPI0 RX   | General-purpose |
| 27 | GP21 | GP21 | UART1 RX / I2C0 SCL / SPI0 CSn  | General-purpose |
| 28 | GND  | GND  | -                               | Ground |
| 29 | GP22 | GP22 | I2C1 SDA / SPI0 SCK / PIO       | General-purpose |
| 30 | RUN  | RUN (active-low reset, system control) | -    | Pull low to reset RP2350; tie to button or leave floating |
| 31 | GP26 | GP26 / ADC0 | I2C1 SDA / SPI1 SCK / PIO / ADC0 | ADC channel 0 |
| 32 | GP27 | GP27 / ADC1 | I2C1 SCL / SPI1 TX  / PIO / ADC1 | ADC channel 1 |
| 33 | AGND | AGND | -                               | Analog ground (separate from digital GND for ADC quiet path) |
| 34 | GP28 | GP28 / ADC2 | I2C0 SDA / SPI1 RX  / PIO / ADC2 | ADC channel 2 |
| 35 | ADC_VREF | ADC_VREF | -                          | ADC reference voltage (3.3 V via on-board filter) |
| 36 | 3V3(OUT) | 3V3_OUT | -                           | Regulated 3.3 V output (max draw ~300 mA) |
| 37 | 3V3_EN   | 3V3_EN  | -                           | Pull low to disable on-board 3.3 V regulator (system control) |
| 38 | GND      | GND     | -                           | Ground |
| 39 | VSYS     | VSYS    | -                           | 1.8 - 5.5 V system input (Schottky-OR'd with VBUS) |
| 40 | VBUS     | VBUS    | -                           | USB +5 V from micro-USB connector |

### Quick-Reference Pin Class Counts

| Class | Count | Pins |
|-------|-------|------|
| User GPIO (also ADC) | 3  | GP26, GP27, GP28 |
| User GPIO (digital only on header) | 23 | GP0 - GP22 |
| Ground (digital) | 8 | Phy 3, 8, 13, 18, 23, 28, 38, plus debug GND |
| Analog ground | 1 | Phy 33 (AGND) |
| Power | 4 | 3V3_OUT (36), 3V3_EN (37), VSYS (39), VBUS (40) |
| ADC reference | 1 | ADC_VREF (35) |
| System control | 1 | RUN (30) |

## 3-Pin Debug Header

The debug header is a separate 3-pin 0.1" connector on the bottom edge of the
board (between the two 20-pin rows). It is **not** part of the 40-pin DIP.

| Debug Pin | Label | Function |
|-----------|-------|----------|
| 1 | SWCLK | Serial Wire Debug clock |
| 2 | GND   | Ground (debug return) |
| 3 | SWDIO | Serial Wire Debug data I/O |

Use with a Picoprobe / Debug Probe for SWD-based flashing and live debugging
(OpenOCD, gdb-multiarch).

## Reserved / On-Board GPIOs (NOT exposed on header)

The RP2350 die has more GPIO bank pins than the Pico 2 board exposes. The
following are reserved for on-board functions and must not be assumed
available externally:

| GPIO | On-board Use |
|------|--------------|
| GP23 | On-board power-supply control (SMPS PS pin) |
| GP24 | VBUS sense (high when USB is plugged in) |
| GP25 | On-board user LED |

### PSRAM Pin Caveat (GP47)

The RP2350 SoC has a dedicated **QSPI / PSRAM** pin set, and on the bare die
**GP47** can be configured as a chip-select for an external PSRAM IC. The
**Pico 2 board does not populate a PSRAM IC**, and **GP47 is not routed to
the 40-pin header** - it exists in the chip's GPIO map only. Firmware must
not attempt to use GP47 as a PSRAM CS on a stock Pico 2 board.
`[verify against rp2350-datasheet.pdf chapter 9.11]`

## Electrical Notes

- Per-pin source/sink current default: 4 mA (configurable up to 12 mA per
  pad). `[verify against rp2350-datasheet.pdf chapter 9.11]`
- Inputs are **not 5 V tolerant**. Level-shift any 5 V signal before
  connecting to a GPIO.
- ADC inputs are 12-bit, 0 V to ADC_VREF (nominal 3.3 V).
- Use **AGND** (phy 33) and a clean **ADC_VREF** (phy 35) when sampling
  analog inputs to minimize digital coupling.

## Back-Links

- [Index](./index.md)
- [02 Peripheral Alt Functions](./02_peripheral_alt_functions.md)
- [03 FT1 Assignment](./03_ft1_assignment.md)
- [Source PDF](../../Pico-2-Pinout.pdf)
