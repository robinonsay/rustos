# Baro ICD — Freescale MPL3115A2

Interface Control Document for the Freescale Xtrinsic **MPL3115A2** I2C
Precision Altimeter (used as the Juno FSW barometric altimeter).

This document is derived from *Freescale Semiconductor Document Number
MPL3115A2, Data Sheet: Technical Data, Rev 3.0, 12/2013*. The PDF is
distributed under Adafruit product **#1893** (the breakout-board form
factor).

## Source Document

- Local PDF: [`../../1893_datasheet.pdf`](../../1893_datasheet.pdf)
- Extracted text fixture: `.icd_fixtures/1893_datasheet.txt`

## Parent Document

- Avionics ICD: [`../avionics.md`](../avionics.md)

## Key Specifications At-a-Glance

| Property | Value |
|----------|-------|
| Sensor type | MEMS absolute pressure / altitude / temperature |
| Pressure calibrated range | **50 to 110 kPa** |
| Pressure operational range | 20 to 110 kPa |
| Pressure absolute accuracy | **±0.4 kPa** (50–110 kPa, 0–50 °C) |
| Altitude derivable | Yes — via US Standard Atmosphere 1976 |
| Altitude resolution (typ.) | 0.3 m (128× OSR) |
| Pressure resolution (typ.) | 1.5 Pa (128× OSR) |
| Temperature range | -40 °C to +85 °C |
| Temperature accuracy | ±1 °C @ 25 °C, ±3 °C over range |
| ADC | 24-bit |
| Pressure output | 20-bit unsigned, **Q18.2** Pascals |
| Altitude output | 20-bit signed, **Q16.4** meters |
| Temperature output | 12-bit signed, **Q8.4** °C |
| Supply voltage VDD | **1.95 V to 3.6 V** (LDO regulated) |
| Interface supply VDDIO | 1.62 V to 3.6 V |
| Standby current | 2 µA (typ.) |
| Active current | 8.5 µA / 40 µA / 265 µA (1×/16×/128× OSR) |
| Digital interface | I2C, up to 400 kHz |
| **I2C 7-bit slave address** | **0x60** (write 0xC0 / read 0xC1) |
| **WHO_AM_I register** | 0x0C, expected value **0xC4** |
| Package | LGA, 5 × 3 × 1.1 mm, 8 pads |
| FIFO depth | 32 samples |
| Operating temperature | -40 °C to +85 °C |

## Table of Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Features, package, ordering information |
| [`02_electrical.md`](02_electrical.md) | Power, signal levels, pin descriptions, LGA pinout |
| [`03_i2c_interface.md`](03_i2c_interface.md) | I2C protocol, slave address, transactions, multi-byte reads |
| [`04_modes.md`](04_modes.md) | STANDBY/ACTIVE, polling vs interrupt, altimeter vs barometer |
| [`05_register_map.md`](05_register_map.md) | Full register map (address, name, R/W, reset) |
| [`06_register_details.md`](06_register_details.md) | Bit-level definitions of key registers |
| [`07_data_format.md`](07_data_format.md) | Q18.2 pressure, Q16.4 altitude, Q8.4 temperature |
| [`08_init_sequence.md`](08_init_sequence.md) | Recommended power-on initialization |
| [`09_appendix.md`](09_appendix.md) | Curves, errata pointers, environmental specs |

## Document Conventions

- All register addresses are 8-bit hex (e.g., `0x26`).
- Bit numbering: bit 7 is MSB, bit 0 is LSB unless otherwise noted.
- "Reset value" means power-on reset, unless explicitly noted as
  STBY-to-ACTIVE reset.
- "shall" language is used only in derived Juno requirements, not in this
  ICD body, which is descriptive.

## Provenance Notes

- Pin number for `INT2` and `INT1` in the source datasheet's Table 1
  conflicts with Figure 2; this ICD uses the **Table 1** mapping
  (INT2 = pin 5, INT1 = pin 6) as the authoritative source.
- Some register tables in the source datasheet reuse the OUT_T_xxx label
  on OUT_P_xxx fields; this ICD silently corrects those in
  [`07_data_format.md`](07_data_format.md).

## Cross-References

- Back to avionics ICD: [`../avionics.md`](../avionics.md)
- Source PDF: [`../../1893_datasheet.pdf`](../../1893_datasheet.pdf)
