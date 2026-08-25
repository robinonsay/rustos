# 01 — Overview

[← Back to Baro ICD index](index.md)

## 1.1 Device Identity

| Item | Value |
|------|-------|
| Manufacturer | Freescale Semiconductor (now NXP) |
| Family | Xtrinsic |
| Part number | **MPL3115A2** |
| Type | I2C Precision Altimeter |
| Datasheet rev | 3.0 (12/2013) |
| Document number | MPL3115A2 |

## 1.2 Functional Summary

The MPL3115A2 employs a MEMS absolute pressure sensor coupled with a
24-bit ADC and an internal digital signal processor. It outputs:

- **Pressure** in Pascals (20-bit, Q18.2 unsigned).
- **Altitude** in meters (20-bit, Q16.4 signed two's complement),
  derived from pressure via the US Standard Atmosphere 1976 model.
- **Temperature** in degrees Celsius (12-bit, Q8.4 signed two's
  complement).

Internal processing removes pressure-temperature compensation work
from the host MCU. The device is factory-calibrated; trim values are
stored in on-chip NVM and applied automatically.

## 1.3 Features (verbatim from datasheet)

- 1.95 V to 3.6 V supply voltage, internally regulated by LDO.
- 1.6 V to 3.6 V digital interface supply voltage.
- Fully compensated internally.
- Direct reading, compensated outputs:
  - Pressure: 20-bit measurement (Pascals).
  - Altitude: 20-bit measurement (meters).
  - Temperature: 12-bit measurement (degrees Celsius).
- Programmable events (interrupt-driven thresholds and windows).
- Autonomous data acquisition (1 s to 9 h period).
- Resolution down to 0.1 m.
- 32-sample FIFO.
- Ability to log data up to 12 days using the FIFO.
- I2C digital output interface (operates up to 400 kHz).

## 1.4 Application Examples (from datasheet)

- High-accuracy altimetry.
- Smartphones / tablets.
- Personal electronics altimetry.
- GPS dead-reckoning.
- GPS enhancement for emergency services.
- Map assist, navigation.
- Weather station equipment.

For Juno FSW, the relevant role is **flight altimetry** for
apogee detection, AGL altitude logging, and barometric input to the
navigation Kalman filter.

## 1.5 Package

| Property | Value |
|----------|-------|
| Package style | LGA (Land Grid Array) |
| Dimensions | 5.0 mm × 3.0 mm × 1.1 mm |
| Pad count | 8 |
| Lid | Stainless steel (with vent) |
| Mount | Surface mount |
| RoHS | Compliant |
| Case number | 2053-01 |

The device has an external port to atmosphere through the lid; PCB
layout must avoid covering or obstructing this port.

## 1.6 Operating Range Summary

| Item | Min | Typ | Max | Unit |
|------|-----|-----|-----|------|
| Calibrated pressure range | 50 | — | 110 | kPa |
| Operational pressure range | 20 | — | 110 | kPa |
| Operating temperature | -40 | 25 | +85 | °C |
| Storage temperature | -40 | — | +125 | °C |
| Maximum applied pressure | — | — | 500 | kPa |

## 1.7 Performance Highlights

| Item | Value |
|------|-------|
| Pressure noise (1× OSR) | 19 Pa RMS |
| Pressure noise (128× OSR) | 1.5 Pa RMS |
| Pressure absolute accuracy | ±0.4 kPa (0–50 °C) |
| Pressure relative accuracy (constant T) | ±0.05 kPa |
| Pressure relative accuracy (changing T) | ±0.1 kPa |
| Pressure resolution (barometer mode, 128× OSR) | 0.25 Pa |
| Altitude resolution (altimeter mode, 128× OSR) | 0.0625 m |
| Output data rate (OST mode) | up to 100 Hz |
| FIFO data rate | up to 1 Hz |
| Long-term drift | ±0.1 kPa / year |
| Board-mount drift (post-reflow) | ±0.15 kPa |

## 1.8 Ordering Information

| Device Name | Package Options | Case # | Type |
|-------------|------------------|--------|------|
| MPL3115A2   | Tray             | 2053   | Single port, absolute |
| MPL3115A2R1 | Tape & Reel (1000) | 2053 | Single port, absolute |

Pressure type details: **Single port**, **absolute**, **gauge =
none**, **differential = none**. Calibrated for 50–110 kPa.

## 1.9 Block Diagram (paraphrased)

The device internal structure consists of:

1. MEMS pressure sensing element with sense amplifier.
2. Temperature reference element.
3. Analog mux feeding a 24-bit ADC.
4. Digital signal processing block (compensation, OSR, mode logic,
   FIFO, alarms, registers).
5. I2C interface.
6. LDO regulator (with external bypass cap on `CAP` pin).
7. Two interrupt outputs (`INT1`, `INT2`).
8. Clock oscillator + trim logic.

For the full pictorial block diagram see Figure 1 of the source PDF.

## 1.10 Related Documents (per datasheet)

- AN3150 — Soldering and handling of pressure sensors.
- AN4519 — Data manipulation and basic settings of the MPL3115A2 CLI.
- AN4481 — Sensor I2C setup and FAQ.

These are not reproduced in this ICD; consult the originals for
hands-on integration tips.

[← Back to Baro ICD index](index.md)
