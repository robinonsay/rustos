# 06 — Data Register Details

[Back to IMU index](./index.md)

This file documents the read-only sensor data registers, the FIFO data
registers, and the external-sensor-data registers populated by the auxiliary
I2C master.

Register reference: RM-MPU-6000A Rev 4.0, sections 4.18 through 4.21,
section 4.32, and section 4.33.

---

## ACCEL_xOUT (`0x3B` to `0x40`)

Reset value: `0x00` for each byte. **Read-only.**

| Addr | Name | Bits | Description |
|------|------|------|-------------|
| `0x3B` | `ACCEL_XOUT_H` | `[15:8]` | Accel X high byte (signed) |
| `0x3C` | `ACCEL_XOUT_L` | `[7:0]`  | Accel X low byte |
| `0x3D` | `ACCEL_YOUT_H` | `[15:8]` | Accel Y high byte |
| `0x3E` | `ACCEL_YOUT_L` | `[7:0]`  | Accel Y low byte |
| `0x3F` | `ACCEL_ZOUT_H` | `[15:8]` | Accel Z high byte |
| `0x40` | `ACCEL_ZOUT_L` | `[7:0]`  | Accel Z low byte |

Each axis is a **16-bit two's-complement** value. Sensitivity depends on
`AFS_SEL` (RM section 4.18, page 30):

| `AFS_SEL` | Full-scale range | LSB Sensitivity |
|----------:|------------------|-----------------|
| 0 | ±2 G  | 16384 LSB / G |
| 1 | ±4 G  | 8192 LSB / G  |
| 2 | ±8 G  | 4096 LSB / G  |
| 3 | ±16 G | 2048 LSB / G  |

To convert: `accel_g = (int16_t)((H << 8) | L) / sensitivity_LSB_per_G`.

The internal register set is updated at the Sample Rate; the user-facing
register set is updated whenever the serial bus is idle, so a burst read
returns a coherent sample (RM section 4.18).

---

## TEMP_OUT (`0x41`, `0x42`)

Reset value: `0x00`. **Read-only.**

| Addr | Name | Bits |
|------|------|------|
| `0x41` | `TEMP_OUT_H` | `[15:8]` |
| `0x42` | `TEMP_OUT_L` | `[7:0]`  |

`TEMP_OUT` is a **16-bit signed** value. The on-chip conversion formula is:

```
temperature_C = (int16_t TEMP_OUT) / 340.0 + 36.53
```

— RM section 4.19 (page 31). The exact scale factor and offset are also
in the Product Specification (electrical specs); **see PDF** for tolerances.

The temperature sensor can be disabled by setting
`PWR_MGMT_1.TEMP_DIS = 1`.

---

## GYRO_xOUT (`0x43` to `0x48`)

Reset value: `0x00`. **Read-only.**

| Addr | Name | Bits | Description |
|------|------|------|-------------|
| `0x43` | `GYRO_XOUT_H` | `[15:8]` | Gyro X high byte (signed) |
| `0x44` | `GYRO_XOUT_L` | `[7:0]`  | Gyro X low byte |
| `0x45` | `GYRO_YOUT_H` | `[15:8]` | Gyro Y high byte |
| `0x46` | `GYRO_YOUT_L` | `[7:0]`  | Gyro Y low byte |
| `0x47` | `GYRO_ZOUT_H` | `[15:8]` | Gyro Z high byte |
| `0x48` | `GYRO_ZOUT_L` | `[7:0]`  | Gyro Z low byte |

Each axis is **16-bit two's-complement**. Sensitivity depends on `FS_SEL`
(RM section 4.20, page 32):

| `FS_SEL` | Full-scale range | LSB Sensitivity |
|---------:|------------------|-----------------|
| 0 | ±250 dps  | 131 LSB / (deg/s)  |
| 1 | ±500 dps  | 65.5 LSB / (deg/s) |
| 2 | ±1000 dps | 32.8 LSB / (deg/s) |
| 3 | ±2000 dps | 16.4 LSB / (deg/s) |

To convert: `omega_dps = (int16_t)((H << 8) | L) / sensitivity_LSB_per_dps`.

---

## EXT_SENS_DATA_xx (`0x49` to `0x60`)

Reset value: `0x00` for each byte. **Read-only.** 24 sequential 8-bit
registers populated by the auxiliary I2C master from external slaves 0
through 3 (slave 4 has its own `I2C_SLV4_DI` register at `0x35`).

| Range | Count |
|-------|-------|
| `0x49` (`EXT_SENS_DATA_00`) … `0x60` (`EXT_SENS_DATA_23`) | 24 bytes |

Allocation rules (RM section 4.21, pages 33-34):

- Each enabled slave is associated with `I2C_SLVx_LEN` consecutive
  `EXT_SENS_DATA` registers, in slave-number order, starting at
  `EXT_SENS_DATA_00`.
- Total allocation across all enabled slaves cannot exceed 24 bytes; excess
  bytes are dropped.
- Allocation is recomputed only when (1) all slaves are disabled, or
  (2) `USER_CTRL.I2C_MST_RESET` is set. Disabling a single slave does NOT
  reclaim its slot for higher-numbered slaves.

> Auxiliary-I2C master operation is not used by Juno FSW; this section is
> documented for completeness.

---

## FIFO_COUNTH / FIFO_COUNTL (`0x72`, `0x73`)

Reset value: `0x00`. R/W (treated as read-only by FSW for current count).

| Addr | Name | Bits | Description |
|------|------|------|-------------|
| `0x72` | `FIFO_COUNTH` | `[15:8]` | High byte of FIFO sample count |
| `0x73` | `FIFO_COUNTL` | `[7:0]`  | Low byte of FIFO sample count |

`FIFO_COUNT` is a 16-bit unsigned value indicating the **number of bytes**
currently buffered in the FIFO.

> "Reading only `FIFO_COUNT_L` will not update the registers to the current
> sample count. `FIFO_COUNT_H` must be accessed first to update the
> contents of both these registers." — RM section 4.32 (page 44).

Therefore FSW must always burst-read `0x72` then `0x73` together (or
re-read both if the H byte changes between accesses).

---

## FIFO_R_W (`0x74`)

Reset value: `0x00`. R/W. The FIFO data port.

| Bit | Name | Description |
|----:|------|-------------|
| 7:0 | `FIFO_DATA[7:0]` | Byte popped from (read) or pushed into (write) the FIFO. |

Behavioral notes (RM section 4.33, page 45):

- Data is written to the FIFO in **register-number order** (from lowest to
  highest) for whichever channels are enabled in `FIFO_EN` (`0x23`) and
  `I2C_MST_CTRL.SLV_3_FIFO_EN`.
- On overflow, `INT_STATUS.FIFO_OFLOW_INT` sets to 1, the **oldest data is
  lost**, and new data continues to be written.
- If the FIFO is empty, reading `FIFO_R_W` returns the last byte previously
  read until new data arrives. FSW must check `FIFO_COUNT` before each
  read.

### FIFO content layout (when all standard channels enabled)

If `FIFO_EN = 0xF8` (TEMP, XG, YG, ZG, ACCEL all enabled) and no aux slaves
are FIFO-enabled, each sample-rate tick pushes 14 bytes into the FIFO in
this order (RM sections 4.7 + 4.33):

| Byte | Source register |
|-----:|-----------------|
| 0 | `ACCEL_XOUT_H` |
| 1 | `ACCEL_XOUT_L` |
| 2 | `ACCEL_YOUT_H` |
| 3 | `ACCEL_YOUT_L` |
| 4 | `ACCEL_ZOUT_H` |
| 5 | `ACCEL_ZOUT_L` |
| 6 | `TEMP_OUT_H`   |
| 7 | `TEMP_OUT_L`   |
| 8 | `GYRO_XOUT_H`  |
| 9 | `GYRO_XOUT_L`  |
| 10 | `GYRO_YOUT_H` |
| 11 | `GYRO_YOUT_L` |
| 12 | `GYRO_ZOUT_H` |
| 13 | `GYRO_ZOUT_L` |

---

## FIFO_EN (`0x23`)

Reset value: `0x00`. Selects which sensor channels feed the FIFO.

| Bit | Name | Description |
|----:|------|-------------|
| 7 | `TEMP_FIFO_EN`  | 1 = `TEMP_OUT_H/L` written to FIFO each Sample Rate tick. |
| 6 | `XG_FIFO_EN`    | 1 = `GYRO_XOUT_H/L` written to FIFO. |
| 5 | `YG_FIFO_EN`    | 1 = `GYRO_YOUT_H/L` written to FIFO. |
| 4 | `ZG_FIFO_EN`    | 1 = `GYRO_ZOUT_H/L` written to FIFO. |
| 3 | `ACCEL_FIFO_EN` | 1 = all six `ACCEL_*OUT` bytes written to FIFO. |
| 2 | `SLV2_FIFO_EN`  | 1 = `EXT_SENS_DATA` bytes for aux-I2C slave 2 written to FIFO. |
| 1 | `SLV1_FIFO_EN`  | 1 = `EXT_SENS_DATA` bytes for aux-I2C slave 1 written to FIFO. |
| 0 | `SLV0_FIFO_EN`  | 1 = `EXT_SENS_DATA` bytes for aux-I2C slave 0 written to FIFO. |

> The FIFO enable bit for aux-I2C slave 3 is in `I2C_MST_CTRL`
> (`SLV_3_FIFO_EN`), not here. — RM section 4.7.

---

[Prev: 05 Config registers](./05_register_details_config.md) | [Back to IMU index](./index.md) | [Next: 07 Other registers](./07_register_details_other.md)
