# Juno FSW Interface Control Documents (ICDs)

> **Status (2026-08-29):** Only the `rp2350/` and `pico2_pinout/` ICDs relate
> to implemented code (the GPIO driver in `firmware/pico2` and the bare-metal
> tutorial cite them). The device ICDs (GPS, IMU, baro, LoRa, SD) and the
> avionics wiring ICD describe planned integrations; no driver for any of
> those devices exists in this repository's code yet.

This directory contains the Interface Control Documents for the FT1 avionics:
the top-level wiring document plus per-device ICDs converted from the source
PDFs in `docs/`.

For the wiring summary (UART/I2C/SPI bus assignments and pin map) start at the
[Avionics ICD](avionics.md). Each per-device ICD is a directory with its own
`index.md` plus chunked content files (each ≤500 lines).

## Top-Level

| ICD | Scope |
|---|---|
| [Avionics ICD](avionics.md) | FT1 wiring overview: UART0, UART1, I2C0, SPI0 + GP17 CS; pin conflict check; power; cross-links to all device ICDs |

## Device ICDs (Sensors / Comm / Storage)

| Device | Role | ICD |
|---|---|---|
| GlobalTop FGPMMOPA6H | GPS receiver (UART, NMEA) | [gps/](gps/index.md) |
| InvenSense MPU-6050 | IMU (I2C, 6-DOF accel + gyro) | [imu/](imu/index.md) |
| Freescale MPL3115A2 | Barometric altimeter (I2C) | [baro/](baro/index.md) |
| REYAX RYLR896 | LoRa radio (UART, AT commands) | [lora/](lora/index.md) |
| SD card (SPI mode) | Mission log storage | [sd/](sd/index.md) |

## MCU / Board ICDs

| ICD | Scope |
|---|---|
| [Pico 2 Pinout](pico2_pinout/index.md) | 40-pin physical pinout, peripheral alt functions, FT1 pin assignment |
| [RP2350 Datasheet (Tailored)](rp2350/index.md) | FT1-relevant chapters only: Clocks, GPIO, UART, I2C, SPI |

## Per-Device File Counts

| ICD | Files | Notes |
|---|---|---|
| `avionics.md` | 1 | Top-level wiring |
| `gps/` | 8 | index + 7 chunks |
| `imu/` | 9 | index + 8 chunks |
| `baro/` | 10 | index + 9 chunks |
| `lora/` | 8 | index + 7 chunks |
| `pico2_pinout/` | 4 | index + 3 chunks |
| `rp2350/` | 24 | top index + 5 peripheral subdirectories (clocks, gpio, uart, i2c, spi), each with its own index + 4–6 chunk files |
| `sd/` | 9 | index + 8 chunks |
| **Total** | **73** markdown files (+ this index = 74; + 3 sub-indexes within rp2350 already counted) | |

## Source PDFs

All ICDs link back to their source PDFs in `docs/`:

| Source | Used by |
|---|---|
| [GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf](../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf) | gps/ |
| [RM-MPU-6000A.pdf](../RM-MPU-6000A.pdf) | imu/ |
| [1893_datasheet.pdf](../1893_datasheet.pdf) (Adafruit product 1893 = MPL3115A2) | baro/ |
| [RYLR896_EN.pdf](../RYLR896_EN.pdf) | lora/ |
| [Pico-2-Pinout.pdf](../Pico-2-Pinout.pdf) | pico2_pinout/ (image-based; ICD authored from authoritative public Pico 2 reference) |
| [rp2350-datasheet.pdf](../rp2350-datasheet.pdf) | rp2350/ (tailored extract: chapters 8, 9, 12.1, 12.2, 12.3) |
| [PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf](../PartA2_SD%20Host_Controller_Simplified_Specification_Ver4.20.pdf), [SDUC-Host-Implementation-Guideline_Ver1.00.pdf](../SDUC-Host-Implementation-Guideline_Ver1.00.pdf) | sd/ (SPI-mode protocol ICD authored from public SD Physical Layer spec; PartA2 is for host-controller hardware, not the SPI client mode used) |

## Authoring Conventions

- Every ICD is multi-file: directory contains `index.md` plus numbered chunk files (`01_*.md`, `02_*.md`, ...).
- Each file ≤500 lines per `ai/memory/constraints.md`.
- All cross-links use relative paths.
- Items not directly extracted from source PDFs are inline-flagged (e.g., `[from RYLR896 module reference, not extracted source PDF]`, `[verify against rp2350-datasheet.pdf chapter 9.11]`).
- Where source PDFs are image-based or absent (Pico 2 pinout, SD Physical Layer spec), the ICD is authored from authoritative public references and clearly notes the source-fidelity status.

## Sprint Provenance

These ICDs were authored as the FT1 ICD sprint immediately following the FT1 Requirements & Test Cases sprint. They support the L2 driver-library requirements in [`../requirements/`](../requirements/).
