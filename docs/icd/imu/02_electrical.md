# 02 — Electrical Interface

[Back to IMU index](./index.md)

> **Note on scope.** The InvenSense register-map document (RM-MPU-6000A) does
> not contain electrical absolute-maximum ratings, DC characteristics, AC
> characteristics, or pin-out figures. Those sections live in the
> *MPU-6000/MPU-6050 Product Specification*, which is a separate document.
> The information below is what can be extracted faithfully from the
> register-map document, plus references where the data must be looked up
> from the Product Specification.

## 2.1 Package

| Attribute | Value | Source |
|-----------|-------|--------|
| Package | QFN, 24-pin | RM section 2 (page 5) |
| Body | 4 mm x 4 mm x 0.9 mm | RM section 2 (page 5) |
| Footprint | Compatible with MPU-3000 family | RM section 2 (page 5) |

## 2.2 Power Supplies (MPU-6050)

| Pin | Name | Function | Notes |
|-----|------|----------|-------|
| VDD | Analog + digital core supply | Powers the analog front-end and digital core | See PDF for voltage range |
| VLOGIC | Digital I/O reference | Sets the logic level of SDA, SCL, INT, FSYNC, AD0 | MPU-6050 only |
| GND | Ground | Common return | — |

> The MPU-6000 does **not** have a VLOGIC pin; its digital I/O references VDD.
> The presence of VLOGIC on the MPU-6050 is the primary electrical
> distinction between the two parts (RM section 2, page 5).
>
> Absolute voltage ranges and current consumption: **see PDF** (Product
> Specification, electrical-characteristics tables).

## 2.3 Signal-Level Reference

All digital I/O on the MPU-6050 (SDA, SCL, INT, FSYNC, CLKIN, AD0) is
referenced to **VLOGIC**, not VDD. This allows the MPU-6050 to interface to
host MCUs whose I/O voltage differs from VDD (e.g., a 1.8 V host MCU with a
2.5 V or 3.3 V VDD on the MPU). The exact VLOGIC range is in the Product
Specification (**see PDF**).

## 2.4 Pin Description (relevant to register-map ICD)

The register-map document references the following pins by name. The
complete pinout (including supply pins, RegOut, CPOUT, and reserved pins) is
in the Product Specification (**see PDF**).

| Pin | Direction | Function | Referenced in RM section |
|-----|-----------|----------|--------------------------|
| SDA / SDI | I/O | Primary I2C data line (also SPI MOSI on MPU-6000) | section 4.29 |
| SCL / SCLK | I | Primary I2C clock (also SPI clock on MPU-6000) | section 4.29 |
| AD0 / SDO | I (MPU-6050) | I2C address LSB (sets bit 0 of 7-bit address); on MPU-6000 in SPI mode this is SDO | section 4.34 |
| AUX_DA | I/O | Auxiliary I2C data line (master mode toward external slaves) | section 4.8, 4.15 |
| AUX_CL | O | Auxiliary I2C clock (master mode) | section 4.8, 4.15 |
| INT | O | Interrupt output to host | section 4.15 |
| FSYNC | I | Frame-sync input; can be sampled into a sensor data LSB or used as a host interrupt pass-through | section 4.3, 4.14, 4.15 |
| CLKIN | I | External reference clock input (selectable via CLKSEL) | section 4.30 |
| nCS | I | SPI chip-select (MPU-6000 only) | section 4.29 |

## 2.5 Logic Levels

The register-map document does not enumerate VIH/VIL/VOH/VOL values.
**See PDF** (Product Specification, DC characteristics) for absolute values.
What the register-map document *does* state about logic-level configuration
that is firmware-controllable:

- The `INT` pin polarity is selectable (active-high or active-low) via
  `INT_PIN_CFG.INT_LEVEL`. — RM section 4.15.
- The `INT` pin drive type is selectable (push-pull or open-drain) via
  `INT_PIN_CFG.INT_OPEN`. — RM section 4.15.
- The `INT` pin pulse behavior is selectable (50 us pulse vs. latched until
  cleared) via `INT_PIN_CFG.LATCH_INT_EN`. — RM section 4.15.
- The `FSYNC` pin polarity (when used as an interrupt pass-through) is
  selectable via `INT_PIN_CFG.FSYNC_INT_LEVEL`. — RM section 4.15.

These bits are documented in detail in
[05_register_details_config.md](./05_register_details_config.md#int_pin_cfg).

## 2.6 Reset

| Mechanism | Effect | Notes |
|-----------|--------|-------|
| Power-on | All registers return to reset values; device starts in **sleep mode**. | RM section 4 (page 9): "The device will come up in sleep mode upon power-up." |
| `PWR_MGMT_1.DEVICE_RESET` (bit 7) | Resets all internal registers to their default values. Bit auto-clears. | RM section 4.30. |
| `USER_CTRL.SIG_COND_RESET` (bit 0) | Resets signal paths for all sensors **and** clears the sensor registers. Bit auto-clears. | RM section 4.29. |
| `SIGNAL_PATH_RESET` (register 0x68) | Resets analog + digital signal paths only; does **not** clear the sensor registers. | RM section 4.27. |
| `USER_CTRL.FIFO_RESET` (bit 2) | Resets the FIFO when set while `FIFO_EN = 0`. Bit auto-clears. | RM section 4.29. |
| `USER_CTRL.I2C_MST_RESET` (bit 1) | Resets the auxiliary I2C master when set while `I2C_MST_EN = 0`. Bit auto-clears. | RM section 4.29. |

## 2.7 Reset Values (summary)

The register-map document states (RM page 8):

- All registers reset to `0x00` **except**:
  - Register `0x6B` (PWR_MGMT_1) → `0x40` (i.e., SLEEP = 1).
  - Register `0x75` (WHO_AM_I) → `0x68`.

Per-register reset values are repeated in
[04_register_map.md](./04_register_map.md).

---

[Prev: 01 Overview](./01_overview.md) | [Back to IMU index](./index.md) | [Next: 03 I2C Interface](./03_i2c_interface.md)
