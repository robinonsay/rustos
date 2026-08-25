# IMU ICD — InvenSense MPU-6050 (Register Map)

**Source document:** [RM-MPU-6000A-00 Rev 4.0 (2012-03-09)](../../RM-MPU-6000A.pdf)
**Parent ICD:** [Avionics ICD](../avionics.md)
**Part used in flight:** MPU-6050 over **I2C**
**Scope of this ICD:** This is the *register manual* for the MPU-6000 family.
The MPU-6000 (SPI + I2C) and MPU-6050 (I2C-only) share the same register map.
Wherever the source document distinguishes the two parts, this ICD calls it out.

---

## Quick Reference

| Item | Value |
|------|-------|
| Part | InvenSense MPU-6050 |
| Sensors | 3-axis gyroscope + 3-axis accelerometer + temperature |
| Gyroscope full-scale (programmable) | ±250, ±500, ±1000, ±2000 dps |
| Accelerometer full-scale (programmable) | ±2 G, ±4 G, ±8 G, ±16 G |
| ADC width (gyro and accel) | 16-bit, two's complement |
| FT1 selection | Accelerometer ±16 G, Gyroscope ±2000 dps |
| Bus | I2C up to 400 kHz (Fast Mode) |
| 7-bit I2C address (AD0 = 0) | `0x68` |
| 7-bit I2C address (AD0 = 1) | `0x69` |
| WHO_AM_I register | `0x75` |
| WHO_AM_I default value | `0x68` |
| Package | 4x4x0.9 mm QFN |
| Sample Rate equation | Gyro Output Rate / (1 + SMPLRT_DIV) |
| Gyro Output Rate | 8 kHz (DLPF = 0 or 7), else 1 kHz |
| Accelerometer output rate | 1 kHz |

---

## Table of Contents

| # | File | Topic |
|---|------|-------|
| 01 | [01_overview.md](./01_overview.md) | Features, MPU-6000 vs MPU-6050, applications |
| 02 | [02_electrical.md](./02_electrical.md) | Power, signal levels, pin descriptions, package |
| 03 | [03_i2c_interface.md](./03_i2c_interface.md) | I2C protocol, addressing, transactions |
| 04 | [04_register_map.md](./04_register_map.md) | Full register map (address, name, R/W, reset) |
| 05 | [05_register_details_config.md](./05_register_details_config.md) | Configuration registers in detail |
| 06 | [06_register_details_data.md](./06_register_details_data.md) | Data registers (accel, gyro, temp, FIFO, ext) |
| 07 | [07_register_details_other.md](./07_register_details_other.md) | Self-test, WHO_AM_I, signal path reset, etc. |
| 08 | [08_init_sequence.md](./08_init_sequence.md) | Recommended power-on initialization sequence |

---

## Notes on Faithfulness

- Every register address, default value, and bit-field name in this ICD is taken
  directly from the InvenSense register map document (RM-MPU-6000A Rev 4.0).
- Figures, signal-path block diagrams, and electrical-spec tables are NOT in
  the register-map document; references that appear in the source as
  "see Section X of the Product Specification document" are reproduced
  verbatim and marked **see PDF**.
- For any field where this ICD summarizes the source narrative, the source
  page is cited where helpful (e.g., "RM page 12").

---

## Links

- Back to [Avionics ICD](../avionics.md)
- Source PDF: [RM-MPU-6000A.pdf](../../RM-MPU-6000A.pdf)
