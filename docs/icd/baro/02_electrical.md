# 02 — Electrical and Mechanical Interface

[← Back to Baro ICD index](index.md)

## 2.1 Pin Descriptions (LGA, top view)

The MPL3115A2 is an 8-pad LGA. The authoritative pin assignment is
Table 1 of the source datasheet:

| Pin | Name   | Direction | Function |
|-----|--------|-----------|----------|
| 1   | VDD    | Power     | Power supply (1.95–3.6 V) |
| 2   | CAP    | Analog    | External capacitor for internal LDO bypass |
| 3   | GND    | Power     | Ground |
| 4   | VDDIO  | Power     | Digital I/O supply (1.62–3.6 V) |
| 5   | INT2   | Output    | Programmable interrupt 2 (open-drain or push-pull) |
| 6   | INT1   | Output    | Programmable interrupt 1 (open-drain or push-pull) |
| 7   | SDA    | I/O       | I2C serial data (open-drain) |
| 8   | SCL    | Input     | I2C serial clock |

> **Note on Pin 1 marker.** The Pin 1 index area marker on the package
> top has no internal electrical connection. It is purely a mechanical
> orientation reference.

## 2.2 Power Supply

| Symbol | Parameter | Min | Typ | Max | Unit |
|--------|-----------|-----|-----|-----|------|
| VDD    | Operating supply voltage   | 1.95 | 2.5 | 3.6 | V |
| VDDIO  | I/O supply voltage         | 1.62 | 1.8 | 3.6 | V |

VDD and VDDIO may be the same rail or independent rails, allowing
level-shifted I2C operation against a 1.8 V host.

### 2.2.1 Decoupling

The datasheet recommends, placed as close as possible to the device:

- **100 nF ceramic** + **10 µF bulk** (or **10 µF ceramic**) on VDD.
- A second **100 nF** ceramic on the `CAP` pin (pin 2), bypassing the
  internal LDO regulator.
- A **100 nF** ceramic on VDDIO.

### 2.2.2 Current Consumption

@ VDD = 2.5 V, T = 25 °C, one update per second:

| Mode | OSR | I_DD (typ.) |
|------|-----|-------------|
| Highest speed mode | 1×   | **8.5 µA** |
| Standard mode      | 16×  | **40 µA**  |
| High resolution    | 128× | **265 µA** |
| Acquisition peak (I_DDMAX) | — | **2 mA** (max during conversion) |
| STANDBY (SBYB = 0, I_DDSTBY) | — | **2 µA** (typ.) |

### 2.2.3 Turn-On Time (T_ON)

Time from STANDBY to first valid data:

| OSR setting | T_ON (typ.) |
|-------------|-------------|
| 1×  (highest speed)      | 60 ms |
| 128× (highest resolution) | 1000 ms |

## 2.3 Digital Signal Levels

All thresholds reference VDDIO.

| Symbol | Parameter | Min | Max | Unit |
|--------|-----------|-----|-----|------|
| V_IH  | Digital high input (SCL, SDA) | 0.75 × VDDIO | — | V |
| V_IL  | Digital low input  (SCL, SDA) | — | 0.3 × VDDIO  | V |
| V_OH  | High-level output (INT1, INT2 push-pull, I_O = 500 µA) | 0.9 × VDDIO | — | V |
| V_OL  | Low-level output  (INT1, INT2, I_O = 500 µA) | — | 0.1 × VDDIO | V |
| V_OLS | Low-level output  (SDA, I_O = 500 µA) | — | 0.1 × VDDIO | V |

## 2.4 Absolute Maximum Ratings

Stress only; functional operation is not implied.

| Characteristic | Symbol | Value | Unit |
|----------------|--------|-------|------|
| Maximum applied pressure | P_max | 500 | kPa |
| Supply voltage           | VDD   | -0.3 to 3.6 | V |
| Interface supply voltage | VDDIO | -0.3 to 3.6 | V |
| Voltage on SCL/SDA       | V_IN  | -0.3 to VDDIO + 0.3 | V |
| Operating temperature    | T_OP  | -40 to +85 | °C |
| Storage temperature      | T_STG | -40 to +125 | °C |

## 2.5 ESD and Latch-Up

| Rating | Symbol | Value | Unit |
|--------|--------|-------|------|
| Human Body Model        | HBM | ±2000 | V |
| Machine Model           | MM  | ±200  | V |
| Charged Device Model    | CDM | ±500  | V |
| Latch-up current @ 85 °C | —  | ±100  | mA |

The MPL3115A2 is **mechanical-shock sensitive** (pressure die) **and
ESD sensitive**. Improper handling can cause permanent damage.

## 2.6 Mechanical

| Property | Value |
|----------|-------|
| Package style | LGA |
| Dimensions    | 5.0 mm × 3.0 mm × 1.1 mm |
| Pad count     | 8 |
| Lid material  | Stainless steel (with vent) |

Recommended PCB landing pattern: see source datasheet Figure 10
(Section 8). Pin 1 marker is mechanical only.

## 2.7 Typical Application Schematic (descriptive)

```
                    VDDIO
                      |
                    [100nF]
                      |
        VDD --[100nF]-+-[10uF]--+---- VDD pin 1
                                |
                              CAP pin 2 --[100nF]-- GND
        GND -------- pin 3
        VDDIO ------ pin 4
        INT2 ------- pin 5  ----[pull-up to VDDIO, optional]----
        INT1 ------- pin 6  ----[pull-up to VDDIO, optional]----
        SDA -------- pin 7  ----[4.7k pull-up to VDDIO]---- I2C
        SCL -------- pin 8  ----[4.7k pull-up to VDDIO]---- I2C
```

I2C bus pull-up resistor selection is a function of bus capacitance
and target SCL frequency; see [`03_i2c_interface.md`](03_i2c_interface.md).

## 2.8 LGA Top-View Pin Map

```
            +---------------------------+
            |     [Pin 1 index dot]     |
            |                           |
        1 --| VDD                  SCL |-- 8
        2 --| CAP                  SDA |-- 7
        3 --| GND                  INT1|-- 6
        4 --| VDDIO                INT2|-- 5
            |                           |
            +---------------------------+
                  TOP VIEW (LGA)
```

[← Back to Baro ICD index](index.md)
