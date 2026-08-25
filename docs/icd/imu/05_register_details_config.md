# 05 — Configuration Register Details

[Back to IMU index](./index.md)

This file documents the bit-level definition of every configuration register
the host MCU writes during normal operation: power management, sample-rate
divider, DLPF, gyro/accel range, interrupts, and the master-level user
control register.

Register reference: RM-MPU-6000A Rev 4.0, sections 4.2 through 4.30.

---

## PWR_MGMT_1 (`0x6B`)

Reset value: **`0x40`** (i.e., `SLEEP = 1`).

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | `DEVICE_RESET` | R/W | Set to 1 to reset all internal registers to defaults. Auto-clears. |
| 6 | `SLEEP`        | R/W | 1 = device in low-power sleep mode (default after power-on). |
| 5 | `CYCLE`        | R/W | When 1 with `SLEEP = 0`, device cycles between sleep and a single accel sample at the rate set by `LP_WAKE_CTRL`. |
| 4 | reserved       | -   | Always read 0; do not write 1. |
| 3 | `TEMP_DIS`     | R/W | 1 = disable temperature sensor. |
| 2:0 | `CLKSEL[2:0]` | R/W | Clock source selection (see table below). |

### CLKSEL table (RM section 4.30)

| `CLKSEL` | Clock source |
|---------:|--------------|
| 0 | Internal 8 MHz oscillator |
| 1 | PLL with X-axis gyroscope reference |
| 2 | PLL with Y-axis gyroscope reference |
| 3 | PLL with Z-axis gyroscope reference |
| 4 | PLL with external 32.768 kHz reference |
| 5 | PLL with external 19.2 MHz reference |
| 6 | Reserved |
| 7 | Stops the clock; keeps timing generator in reset |

> "Upon power up, the MPU-60X0 clock source defaults to the internal
> oscillator. However, it is highly recommended that the device be
> configured to use one of the gyroscopes (or an external clock source) as
> the clock reference for improved stability." — RM section 4.30.

---

## PWR_MGMT_2 (`0x6C`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7:6 | `LP_WAKE_CTRL[1:0]` | R/W | Wake-up frequency in accelerometer-only low-power mode. |
| 5 | `STBY_XA` | R/W | 1 = X-axis accelerometer in standby. |
| 4 | `STBY_YA` | R/W | 1 = Y-axis accelerometer in standby. |
| 3 | `STBY_ZA` | R/W | 1 = Z-axis accelerometer in standby. |
| 2 | `STBY_XG` | R/W | 1 = X-axis gyroscope in standby. |
| 1 | `STBY_YG` | R/W | 1 = Y-axis gyroscope in standby. |
| 0 | `STBY_ZG` | R/W | 1 = Z-axis gyroscope in standby. |

### LP_WAKE_CTRL table (RM section 4.31)

| `LP_WAKE_CTRL` | Wake-up frequency |
|---------------:|-------------------|
| 0 | 1.25 Hz |
| 1 | 5 Hz |
| 2 | 20 Hz |
| 3 | 40 Hz |

To enter Accelerometer-Only Low Power Mode (RM section 4.31): set
`PWR_MGMT_1.CYCLE = 1`, `SLEEP = 0`, `TEMP_DIS = 1`, then set
`STBY_XG = STBY_YG = STBY_ZG = 1`.

---

## SMPLRT_DIV (`0x19`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7:0 | `SMPLRT_DIV[7:0]` | R/W | 8-bit unsigned divider applied to the gyro output rate. |

```
Sample Rate = Gyroscope Output Rate / (1 + SMPLRT_DIV)
```

where Gyro Output Rate is **8 kHz** when `DLPF_CFG = 0 or 7` and **1 kHz**
otherwise. (RM section 4.2.)

> Accelerometer output rate is fixed at 1 kHz. If the configured Sample
> Rate exceeds 1 kHz, the same accelerometer sample may be repeated in the
> FIFO and DMP. — RM section 4.2.

---

## CONFIG (`0x1A`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | reserved | - | — |
| 6 | reserved | - | — |
| 5:3 | `EXT_SYNC_SET[2:0]` | R/W | Selects which sensor LSB receives the latched FSYNC bit. |
| 2:0 | `DLPF_CFG[2:0]`     | R/W | Digital low-pass filter setting (shared by gyro + accel). |

### EXT_SYNC_SET (RM section 4.3)

| `EXT_SYNC_SET` | FSYNC bit location |
|---------------:|--------------------|
| 0 | Input disabled |
| 1 | `TEMP_OUT_L[0]` |
| 2 | `GYRO_XOUT_L[0]` |
| 3 | `GYRO_YOUT_L[0]` |
| 4 | `GYRO_ZOUT_L[0]` |
| 5 | `ACCEL_XOUT_L[0]` |
| 6 | `ACCEL_YOUT_L[0]` |
| 7 | `ACCEL_ZOUT_L[0]` |

### DLPF_CFG (RM section 4.3)

| `DLPF_CFG` | Accel BW (Hz) | Accel delay (ms) | Gyro BW (Hz) | Gyro delay (ms) | Gyro Fs (kHz) |
|-----------:|--------------:|-----------------:|-------------:|----------------:|--------------:|
| 0 | 260 | 0    | 256 | 0.98 | 8 |
| 1 | 184 | 2.0  | 188 | 1.9  | 1 |
| 2 | 94  | 3.0  | 98  | 2.8  | 1 |
| 3 | 44  | 4.9  | 42  | 4.8  | 1 |
| 4 | 21  | 8.5  | 20  | 8.3  | 1 |
| 5 | 10  | 13.8 | 10  | 13.4 | 1 |
| 6 | 5   | 19.0 | 5   | 18.6 | 1 |
| 7 | reserved | reserved | reserved | reserved | 8 |

---

## GYRO_CONFIG (`0x1B`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | `XG_ST` | R/W | 1 = X-axis gyro performs self-test. |
| 6 | `YG_ST` | R/W | 1 = Y-axis gyro performs self-test. |
| 5 | `ZG_ST` | R/W | 1 = Z-axis gyro performs self-test. |
| 4:3 | `FS_SEL[1:0]` | R/W | Gyro full-scale range. |
| 2:0 | reserved | - | — |

### FS_SEL table (RM section 4.4)

| `FS_SEL` | Full-scale range | Sensitivity (LSB / (deg/s)) |
|---------:|------------------|-----------------------------|
| 0 | ±250 dps  | 131    |
| 1 | ±500 dps  | 65.5   |
| 2 | ±1000 dps | 32.8   |
| 3 | ±2000 dps | 16.4   |

> Sensitivity values from RM section 4.20 (page 32).

---

## ACCEL_CONFIG (`0x1C`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | `XA_ST` | R/W | 1 = X-axis accel performs self-test. |
| 6 | `YA_ST` | R/W | 1 = Y-axis accel performs self-test. |
| 5 | `ZA_ST` | R/W | 1 = Z-axis accel performs self-test. |
| 4:3 | `AFS_SEL[1:0]` | R/W | Accel full-scale range. |
| 2:0 | reserved | - | — |

### AFS_SEL table (RM section 4.5 + section 4.18)

| `AFS_SEL` | Full-scale range | Sensitivity (LSB / G) |
|----------:|------------------|-----------------------|
| 0 | ±2 G  | 16384 |
| 1 | ±4 G  | 8192  |
| 2 | ±8 G  | 4096  |
| 3 | ±16 G | 2048  |

> The register-map document references a "Digital High-Pass Filter (DHPF)"
> in this register's description (RM section 4.5) but the bit allocation
> for DHPF is not enumerated in the register table on page 6; bits 2:0 are
> shown as a single dash. **See PDF** for the DHPF bit layout if needed.

---

## INT_PIN_CFG (`0x37`) {#int_pin_cfg}

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | `INT_LEVEL`        | R/W | 0 = INT active high; 1 = INT active low. |
| 6 | `INT_OPEN`         | R/W | 0 = push-pull; 1 = open-drain. |
| 5 | `LATCH_INT_EN`     | R/W | 0 = INT pulses 50 us; 1 = INT held high until cleared. |
| 4 | `INT_RD_CLEAR`     | R/W | 0 = clear `INT_STATUS` bits only by reading `INT_STATUS`; 1 = clear on any read. |
| 3 | `FSYNC_INT_LEVEL`  | R/W | 0 = FSYNC interrupt active high; 1 = active low. |
| 2 | `FSYNC_INT_EN`     | R/W | 1 = FSYNC pin generates an interrupt to the host. |
| 1 | `I2C_BYPASS_EN`    | R/W | 1 (with `USER_CTRL.I2C_MST_EN = 0`) = host directly accesses aux I2C bus. |
| 0 | reserved           | -   | — |

---

## INT_ENABLE (`0x38`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | reserved          | -   | — |
| 6 | `MOT_EN`          | R/W | 1 = motion-detection interrupt enabled. |
| 5 | reserved          | -   | — |
| 4 | `FIFO_OFLOW_EN`   | R/W | 1 = FIFO-overflow interrupt enabled. |
| 3 | `I2C_MST_INT_EN`  | R/W | 1 = any auxiliary-I2C-master source can raise INT. |
| 2 | reserved          | -   | — |
| 1 | reserved          | -   | — |
| 0 | `DATA_RDY_EN`     | R/W | 1 = Data-Ready interrupt enabled (fires when sensor regs updated). |

---

## INT_STATUS (`0x3A`)

Reset value: `0x00`. **Read-only.** Each bit clears after the register is read.

| Bit | Name | Description |
|----:|------|-------------|
| 7 | reserved        | — |
| 6 | `MOT_INT`       | 1 = motion-detection interrupt occurred. |
| 5 | reserved        | — |
| 4 | `FIFO_OFLOW_INT`| 1 = FIFO overflow occurred. |
| 3 | `I2C_MST_INT`   | 1 = aux-I2C master interrupt; see `I2C_MST_STATUS`. |
| 2 | reserved        | — |
| 1 | reserved        | — |
| 0 | `DATA_RDY_INT`  | 1 = new sensor sample available. |

---

## USER_CTRL (`0x6A`)

Reset value: `0x00`.

| Bit | Name | R/W | Description |
|----:|------|-----|-------------|
| 7 | reserved          | -   | — |
| 6 | `FIFO_EN`         | R/W | 1 = enable FIFO operations. |
| 5 | `I2C_MST_EN`      | R/W | 1 = enable aux-I2C master mode. |
| 4 | `I2C_IF_DIS`      | R/W | MPU-6000: 1 disables I2C in favor of SPI. **MPU-6050: always write 0.** |
| 3 | reserved          | -   | — |
| 2 | `FIFO_RESET`      | R/W | 1 (with `FIFO_EN = 0`) resets FIFO; auto-clears. |
| 1 | `I2C_MST_RESET`   | R/W | 1 (with `I2C_MST_EN = 0`) resets aux-I2C master; auto-clears. |
| 0 | `SIG_COND_RESET`  | R/W | 1 resets all signal paths **and clears sensor registers**; auto-clears. |

---

[Prev: 04 Register Map](./04_register_map.md) | [Back to IMU index](./index.md) | [Next: 06 Data registers](./06_register_details_data.md)
