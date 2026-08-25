# Raspberry Pi Pico 2 Pinout — Interface Control Document

## Document Status

| Field | Value |
|-------|-------|
| Document Type | Interface Control Document (ICD) |
| Subject | Raspberry Pi Pico 2 board pinout (RP2350) |
| Source | `../../Pico-2-Pinout.pdf` (image-based; text extraction yielded ~153 chars) |
| Authoring Basis | Publicly documented Raspberry Pi Pico 2 board pinout (same physical 40-pin footprint as Pico 1, with RP2350 alt-function set) |
| Verification Reference | `docs/rp2350-datasheet.pdf` chapter 9.11 (GPIO function table) |

> **Note on source fidelity:** The original `Pico-2-Pinout.pdf` is a graphical
> reference card. Text extraction recovered only the title and color-legend
> labels (`Power`, `Ground`, `UART / UART (default)`, `System Control`,
> `Debugging`, `ADC`). The detailed mappings in this ICD are authored from
> publicly documented Pi Pico 2 board pinout information. Where an
> alternate-function assignment is not certain from the public reference card,
> it is annotated `[verify against rp2350-datasheet.pdf chapter 9.11]`.

## Board Summary

The Raspberry Pi Pico 2 is the second-generation Raspberry Pi microcontroller
board, built around the **RP2350** SoC. It exposes a 40-pin DIP-compatible
through-hole header in the same physical footprint as the Pico 1, plus a
3-pin debug header (SWCLK / GND / SWDIO).

| Property | Value |
|----------|-------|
| MCU | RP2350 (dual Cortex-M33 / dual Hazard3 RISC-V, selectable) |
| Board GPIOs exposed | 26 (GP0 - GP22, GP26 - GP28; GP23 - GP25 reserved on-board) |
| ADC channels exposed | 3 (GP26 / ADC0, GP27 / ADC1, GP28 / ADC2) |
| Operating logic level | 3.3 V |
| 5 V tolerant inputs | No |
| Header pitch | 0.1" (2.54 mm) |
| Header layout | 2 x 20 (40 pins) plus 1 x 3 debug |
| USB | Micro-USB B (VBUS available on pin 40) |
| On-board PSRAM | None (the RP2350 die has a PSRAM-capable bank `[verify against rp2350-datasheet.pdf chapter 9.11]` but the Pico 2 board does not populate it; see physical pinout note for GP47) |

## Color / Function Legend (preserved from source)

The source PDF uses a color legend identifying the following pin classes:

- Power
- Ground
- UART / UART (default)
- System Control
- Debugging
- ADC
- (additionally, by convention) GPIO, I2C, SPI, PWM, PIO, Clocks

This ICD reproduces the same classes textually in the per-pin tables.

## Table of Contents

| File | Contents |
|------|----------|
| [`01_physical_pinout.md`](./01_physical_pinout.md) | Physical pin number to GPIO / function mapping for all 40 header pins plus the 3-pin debug header |
| [`02_peripheral_alt_functions.md`](./02_peripheral_alt_functions.md) | Alternate-function tables for UART0, UART1, I2C0, I2C1, SPI0, SPI1, PIO0, PIO1 |
| [`03_ft1_assignment.md`](./03_ft1_assignment.md) | FT1 pin assignment (GP0/1 LoRa, GP4/5 GPS, GP8/9 baro+IMU, GP16-19 SD) cross-referenced to avionics doc |

## External References

- **Source PDF (image-based reference card):** [`../../Pico-2-Pinout.pdf`](../../Pico-2-Pinout.pdf)
- **Avionics ICD (FT1 wiring):** [`../avionics.md`](../avionics.md)
- **RP2350 datasheet:** `docs/rp2350-datasheet.pdf` (chapter 9.11 = GPIO function table; chapter 12 = peripherals)

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-05-01 | Initial multi-file ICD authored from public Pico 2 pinout (source PDF is image-based) | Software Systems Engineer |

## Back-Links

- [Avionics ICD](../avionics.md)
- [Pico-2-Pinout.pdf (source)](../../Pico-2-Pinout.pdf)
