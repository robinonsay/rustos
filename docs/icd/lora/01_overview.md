# 01 — Overview

[← Back to index](index.md)

## Product Description

The REYAX **RYLR896** is an **868/915 MHz LoRa Antenna Transceiver Module**.
It is built around the **Semtech SX1276** LoRa modem and provides ultra-long
range spread-spectrum communication with high interference immunity while
minimising current consumption.

The RYLR896 is **certified by NCC and FCC**.

> Source: RYLR896 EN datasheet, p.2 ("PRODUCT DESCRIPTION").

## Features

Verbatim from the source datasheet (p.2):

- Semtech SX1276 Engine
- Excellent blocking immunity
- Low receive current
- High sensitivity
- Control easily by AT commands
- 127 dB Dynamic Range RSSI
- Designed with integrated antenna
- AES128 Data encryption

## Applications

Verbatim from the source datasheet (p.2):

- IoT Applications
- Mobile Equipment
- Home Security
- Industrial Monitoring and Control Equipment
- Car Alarm

For Juno FSW the RYLR896 is used as the **primary point-to-point telemetry
downlink** between the airborne flight computer and the ground station, and
as a low-rate uplink command channel.

## Package and Form Factor

- 6-pin SMD module with **integrated antenna** (no external antenna required).
- Mechanical drawings appear on pages 5–6 of the source PDF; the extracted
  text does not include numerical dimensions (the figures are bitmap-only).
- **Weight:** 3.07 g (typ.) — see `02_electrical.md` SPECIFICATION table.

> The detailed mechanical drawing (`Unit: mm`) on pages 5–6 of the source
> PDF was not extracted as text. Refer to the original PDF for footprint and
> outline dimensions.

## Certifications

- **FCC** — Contains TX FCC ID: `QLY-RYLR896`
- **NCC** (Taiwan) — `CCAN18LP0920T8`

See [`07_appendix.md`](07_appendix.md) for the verbatim FCC and NCC
statements that must accompany end-product labelling.

## Traceability

| Source PDF section            | Target file |
|-------------------------------|-------------|
| PRODUCT DESCRIPTION (p.2)     | this file   |
| FEATURES (p.2)                | this file   |
| APPLICATIONS (p.2)            | this file   |
| CERTIFICATION (p.2, p.7)      | this file, `07_appendix.md` |
| PIN DESCRIPTION (p.3)         | `02_electrical.md` |
| SPECIFICATION (p.4)           | `02_electrical.md` |
| DIMENSIONS (p.5–6)            | (figures only — see PDF) |

[← Back to index](index.md) | [Next: 02 Electrical →](02_electrical.md)
