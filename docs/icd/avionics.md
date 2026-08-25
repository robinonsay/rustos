# FT1 Avionics Interface Control Document

This document defines the wiring between the Raspberry Pi Pico 2 (RP2350)
microcontroller and the four FT1 sensor and communication devices: the
FGPMMOPA6H GPS receiver, the MPU-6050 inertial measurement unit, the
MPL3115A2 barometric altimeter, and the RYLR896 LoRa radio. It also covers
the SD card used for onboard storage. All bus assignments, GPIO allocations,
voltage levels, and power expectations for the FT1 flight stack are captured
here, with cross-links to per-device ICDs and to the underlying RP2350
peripheral chapters.

## 1. Bus Wiring Table

| Bus | RP2350 Pins | Device | Notes |
|---|---|---|---|
| UART0 | TX=GP0/pin 1, RX=GP1/pin 2 | RYLR896 LoRa radio | AT-command interface |
| UART1 | TX=GP4/pin 6, RX=GP5/pin 7 | FGPMMOPA6H GPS | NMEA, 9600 baud default |
| I2C0 | SDA=GP8/pin 11, SCL=GP9/pin 12 | MPL3115A2 (baro, 0x60) + MPU-6050 (IMU, 0x68) | Shared bus, 400 kHz |
| SPI0 | SCK=GP18/pin 24, MOSI=GP19/pin 25, MISO=GP16/pin 21 | SD card (SPI mode) | Mode 0 |
| SPI0 CS | **GP17/pin 22** | SD card chip-select | Software-controlled |

## 2. Per-Bus Details

### 2.1 UART0 - RYLR896 LoRa Radio

- **Bus role:** Serial command and telemetry link to the RYLR896 LoRa
  module. UART0 was selected because the RYLR896 exposes a simple
  AT-command interface intended to be driven by a host UART, and routing it
  to the RP2350's first UART keeps the LoRa link on a dedicated, lowest-
  numbered peripheral.
- **Voltage levels:** 3.3 V CMOS logic on TX/RX. No level translation is
  required; the RYLR896 is a 3.3 V module.
- **Baud-rate source:** Both ends of the link derive their baud rate from
  internal references: the RP2350 uses its peripheral clock (clk_peri,
  default 48 MHz) divided down to the configured baud rate, and the RYLR896
  uses its on-module oscillator. No external crystal or reference is shared
  between the two.
- **Pull-up/pull-down requirements:** None required for normal full-duplex
  UART operation. TX is driven push-pull. RX should not be left floating
  when the radio is unpowered; if power-sequencing the radio independently
  of the MCU, an internal pull-up on RX is recommended.
- **Reference:** [rp2350/uart/index.md](rp2350/uart/index.md) and the RYLR896 ICD at
  [lora/index.md](lora/index.md).

### 2.2 UART1 - FGPMMOPA6H GPS

- **Bus role:** NMEA-0183 serial input from the GlobalTop FGPMMOPA6H GPS
  receiver. UART1 was chosen so that the GPS link is electrically
  independent from the LoRa link on UART0; this avoids contention if either
  device is reconfigured at a non-default baud and lets each peripheral run
  its own DMA channel.
- **Voltage levels:** 3.3 V CMOS logic on TX/RX. The FGPMMOPA6H module is
  3.3 V native; no level translation required.
- **Baud-rate source:** RP2350 derives the 9600 baud line rate from
  clk_peri. The FGPMMOPA6H uses its own internal oscillator. The default
  configured rate after power-on is 9600 8N1.
- **Pull-up/pull-down requirements:** None required. As with UART0, an
  internal pull-up on RX is recommended if the GPS is power-sequenced
  separately from the MCU so that idle-high is enforced before the GPS
  asserts the line.
- **Reference:** [rp2350/uart/index.md](rp2350/uart/index.md) and the GPS ICD at
  [gps/index.md](gps/index.md).

### 2.3 I2C0 - MPL3115A2 (Baro) + MPU-6050 (IMU)

- **Bus role:** Shared two-wire interface to the on-board environmental
  sensors. I2C0 was selected because both sensors expose I2C natively at
  distinct, fixed addresses (MPL3115A2 at 0x60, MPU-6050 at 0x68), so they
  can coexist on a single bus and consume only two GPIO lines total.
- **Voltage levels:** 3.3 V logic. Both sensors are 3.3 V parts. Bus speed
  is 400 kHz (Fast-mode) per the lower of the two sensors' rated maxima.
- **Pull-up requirements:** External pull-up resistors on SDA (GP8) and
  SCL (GP9) are **required**. Recommended value 4.7 kohm to 3.3 V; the
  exact value should be tuned to the bus capacitance of the assembled
  board, but 4.7 kohm is the FT1 default. The RP2350 internal pull-ups are
  too weak (~50 kohm) to be relied upon for I2C and must not be the only
  pull-up source.
- **Pull-down requirements:** None.
- **Reference:** [rp2350/i2c/index.md](rp2350/i2c/index.md), the baro ICD at
  [baro/index.md](baro/index.md), and the IMU ICD at
  [imu/index.md](imu/index.md).

### 2.4 SPI0 - SD Card

- **Bus role:** Block-oriented data link to the onboard SD card running
  in SPI mode. SPI0 was chosen because the SD card host-controller spec
  defines an SPI fallback mode, and the RP2350's SPI0 peripheral provides
  hardware-managed SCK/MOSI/MISO with DMA support that is well suited to
  block writes from the mission logger.
- **Voltage levels:** 3.3 V CMOS on SCK, MOSI, MISO, and CS. SD cards in
  SPI mode are 3.3 V parts.
- **SPI mode:** Mode 0 (CPOL=0, CPHA=0).
- **Pull-up/pull-down requirements:** A weak pull-up (~10 kohm to 3.3 V)
  on the CS line (GP17) is recommended so that the card is deselected
  during MCU reset before firmware drives the line. MOSI may benefit from
  a pull-up during initialization to keep the line idle-high while the
  card boots into SPI mode. SCK and MISO require no external pulls.
- **Reference:** [rp2350/spi/index.md](rp2350/spi/index.md) and the SD card ICD at
  [sd/index.md](sd/index.md).

### 2.5 SPI0 CS - SD Card Chip-Select

- **Bus role:** Software-driven chip-select for the SD card on SPI0.
- **Voltage levels:** 3.3 V CMOS, active-low.
- **Pull-up/pull-down requirements:** External 10 kohm pull-up to 3.3 V
  is recommended (see 2.4).
- **Reference:** [rp2350/gpio/index.md](rp2350/gpio/index.md) and
  [sd/index.md](sd/index.md).

## 3. Device Index

| Device | Role | ICD |
|---|---|---|
| FGPMMOPA6H | GPS receiver | [docs/icd/gps/index.md](gps/index.md) |
| MPU-6050 | IMU (6-DOF) | [docs/icd/imu/index.md](imu/index.md) |
| MPL3115A2 | Barometric altimeter | [docs/icd/baro/index.md](baro/index.md) |
| RYLR896 | LoRa radio | [docs/icd/lora/index.md](lora/index.md) |
| Pi Pico 2 (RP2350) | MCU | [docs/icd/rp2350/index.md](rp2350/index.md) and [docs/icd/pico2_pinout/index.md](pico2_pinout/index.md) |
| SD card | Storage (SPI mode) | [docs/icd/sd/index.md](sd/index.md) |

## 4. Pin Conflict Check

The following RP2350 GPIO pins are allocated by FT1 avionics. Each pin has
exactly one function; no pin is dual-used.

| GPIO | Pico 2 Pin | Function | Bus | Device |
|---|---|---|---|---|
| GP0  | 1  | UART0 TX  | UART0    | RYLR896 LoRa |
| GP1  | 2  | UART0 RX  | UART0    | RYLR896 LoRa |
| GP4  | 6  | UART1 TX  | UART1    | FGPMMOPA6H GPS |
| GP5  | 7  | UART1 RX  | UART1    | FGPMMOPA6H GPS |
| GP8  | 11 | I2C0 SDA  | I2C0     | MPL3115A2 + MPU-6050 |
| GP9  | 12 | I2C0 SCL  | I2C0     | MPL3115A2 + MPU-6050 |
| GP16 | 21 | SPI0 MISO | SPI0     | SD card |
| GP17 | 22 | SPI0 CS   | SPI0 CS  | SD card |
| GP18 | 24 | SPI0 SCK  | SPI0     | SD card |
| GP19 | 25 | SPI0 MOSI | SPI0     | SD card |

Conflict review:

- Each GPIO appears exactly once in the table above.
- UART0 and UART1 occupy disjoint GPIOs (GP0/GP1 vs GP4/GP5).
- I2C0 GPIOs (GP8, GP9) are disjoint from all UART and SPI GPIOs.
- SPI0 GPIOs (GP16, GP17, GP18, GP19) are disjoint from all UART and I2C
  GPIOs.
- The two I2C0 devices share the same pair of GPIOs by design (one bus, two
  addresses); this is not a pin conflict because I2C is a multi-drop bus.

No pin conflicts exist.

## 5. Power

All FT1 avionics devices operate from a single 3.3 V regulated supply rail
sourced through the Pico 2's 3V3(OUT) pin or, where greater headroom is
required, through a board-level 3.3 V regulator fed from VSYS. All four
sensor/comm devices and the SD card accept 3.3 V CMOS logic, so no
level-shifting is needed on any bus.

Per-device supply voltage ranges:

| Device | Supply Rail | Logic Level | Range |
|---|---|---|---|
| Raspberry Pi Pico 2 (RP2350) | VSYS in / 3V3 out | 3.3 V | VSYS 1.8-5.5 V; 3V3 fixed at 3.3 V |
| FGPMMOPA6H GPS | VCC | 3.3 V | TBD - confirm against datasheet (typ. 3.0-4.3 V) |
| MPU-6050 IMU | VDD | 3.3 V | TBD - confirm against datasheet (typ. 2.375-3.46 V) |
| MPL3115A2 Baro | VDD | 3.3 V | TBD - confirm against datasheet (typ. 1.95-3.6 V) |
| RYLR896 LoRa | VCC | 3.3 V | TBD - confirm against datasheet (typ. 1.8-3.7 V) |
| SD card (SPI mode) | VDD | 3.3 V | 2.7-3.6 V per SD physical layer spec |

The TBD entries should be verified against the source PDFs listed in
section 6 before flight readiness review. Each per-device ICD captures the
authoritative supply range for that part; this table is informational only.

## 6. Source PDF References

The following datasheets and specifications are the authoritative sources
for the bus assignments and electrical characteristics summarized here.
Links are relative to this file (`docs/icd/avionics.md`).

- GPS (FGPMMOPA6H): [../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf](../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf)
- IMU (MPU-6000/6050 register map): [../RM-MPU-6000A.pdf](../RM-MPU-6000A.pdf)
- Baro (MPL3115A2): [../1893_datasheet.pdf](../1893_datasheet.pdf)
- LoRa (RYLR896): [../RYLR896_EN.pdf](../RYLR896_EN.pdf)
- Pico 2 pinout: [../Pico-2-Pinout.pdf](../Pico-2-Pinout.pdf)
- RP2350 datasheet: [../rp2350-datasheet.pdf](../rp2350-datasheet.pdf)
- SD host controller (simplified, v4.20): [../PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf](../PartA2_SD%20Host_Controller_Simplified_Specification_Ver4.20.pdf)
