# 07 — Other Register Details

[Back to IMU index](./index.md)

This file documents the remaining registers: self-test, signal-path reset,
motion-detection threshold and control, auxiliary-I2C master / slave
configuration, and the WHO_AM_I identity register.

Register reference: RM-MPU-6000A Rev 4.0, sections 4.1, 4.6 to 4.14, 4.22
to 4.28, and 4.34.

---

## SELF_TEST_X / Y / Z / A (`0x0D`-`0x10`)

R/W. Reset value `0x00` for each.

| Addr | Name | Bit 7 | Bit 6 | Bit 5 | Bit 4 | Bit 3 | Bit 2 | Bit 1 | Bit 0 |
|------|------|------:|------:|------:|------:|------:|------:|------:|------:|
| `0x0D` | `SELF_TEST_X` | XA_TEST[4] | XA_TEST[3] | XA_TEST[2] | XG_TEST[4] | XG_TEST[3] | XG_TEST[2] | XG_TEST[1] | XG_TEST[0] |
| `0x0E` | `SELF_TEST_Y` | YA_TEST[4] | YA_TEST[3] | YA_TEST[2] | YG_TEST[4] | YG_TEST[3] | YG_TEST[2] | YG_TEST[1] | YG_TEST[0] |
| `0x0F` | `SELF_TEST_Z` | ZA_TEST[4] | ZA_TEST[3] | ZA_TEST[2] | ZG_TEST[4] | ZG_TEST[3] | ZG_TEST[2] | ZG_TEST[1] | ZG_TEST[0] |
| `0x10` | `SELF_TEST_A` | reserved | reserved | XA_TEST[1] | XA_TEST[0] | YA_TEST[1] | YA_TEST[0] | ZA_TEST[1] | ZA_TEST[0] |

`xG_TEST[4:0]` (gyro) and `xA_TEST[4:0]` (accel, formed by concatenating
the upper 3 bits in `SELF_TEST_X/Y/Z` with the lower 2 bits in
`SELF_TEST_A`) are 5-bit unsigned factory-trim codes used to compute the
expected self-test response (RM section 4.1).

### Gyro factory-trim formula (RM section 4.1, page 10)

For X (Z analogous):

```
FT[Xg] = 25 * 131 * 1.046^(XG_TEST - 1)   if XG_TEST != 0
FT[Xg] = 0                                if XG_TEST == 0
```

For Y the formula is negated:

```
FT[Yg] = -25 * 131 * 1.046^(YG_TEST - 1)  if YG_TEST != 0
```

When performing gyro self-test, set `FS_SEL = 0` (±250 dps).

### Accel factory-trim formula (RM section 4.1, page 11)

For each axis (X, Y, Z), with the 5-bit `xA_TEST` formed by concatenating
the upper 3 bits and lower 2 bits as described above:

```
FT[xA] = 4096 * 0.34 * (0.92 / 0.34)^((xA_TEST - 1) / (2^5 - 2))
                                          if xA_TEST != 0
FT[xA] = 0                                if xA_TEST == 0
```

When performing accel self-test, set `AFS_SEL = 2` (±8 G).

### Pass/fail criterion (RM section 4.1)

```
Change-from-FT (%) = (STR - FT) / FT
```

where `STR = sensor_output_with_self_test - sensor_output_without_self_test`.
Pass/fail limits are in the Product Specification — **see PDF**.

The self-test trigger bits are in `GYRO_CONFIG` (`XG_ST`, `YG_ST`, `ZG_ST`)
and `ACCEL_CONFIG` (`XA_ST`, `YA_ST`, `ZA_ST`), documented in
[05_register_details_config.md](./05_register_details_config.md).

---

## MOT_THR (`0x1F`)

R/W. Reset `0x00`.

| Bit | Name | Description |
|----:|------|-------------|
| 7:0 | `MOT_THR[7:0]` | Motion-detection threshold; mg-per-LSB scaling in Product Specification (**see PDF**). |

Motion is flagged when the absolute value of any accel measurement exceeds
`MOT_THR`. The Motion interrupt status appears in `INT_STATUS.MOT_INT`
(`0x3A` bit 6). — RM section 4.6.

---

## I2C_MST_CTRL (`0x24`)

R/W. Reset `0x00`. Configures the **auxiliary** I2C master.

| Bit | Name | Description |
|----:|------|-------------|
| 7 | `MULT_MST_EN`     | 1 = auxiliary bus multi-master arbitration enabled (+~30 uA). |
| 6 | `WAIT_FOR_ES`     | 1 = delay Data-Ready interrupt until external-sensor data is loaded. |
| 5 | `SLV_3_FIFO_EN`   | 1 = `EXT_SENS_DATA` bytes for aux slave 3 are written to FIFO. |
| 4 | `I2C_MST_P_NSR`   | 0 = restart between aux reads; 1 = stop+start. |
| 3:0 | `I2C_MST_CLK[3:0]` | 4-bit divider on internal 8 MHz clock (see table). |

### I2C_MST_CLK table (RM section 4.8, page 19)

| `I2C_MST_CLK` | Aux clock | Divider |
|--------------:|-----------|---------|
| 0  | 348 kHz | 23 |
| 1  | 333 kHz | 24 |
| 2  | 320 kHz | 25 |
| 3  | 308 kHz | 26 |
| 4  | 296 kHz | 27 |
| 5  | 286 kHz | 28 |
| 6  | 276 kHz | 29 |
| 7  | 267 kHz | 30 |
| 8  | 258 kHz | 31 |
| 9  | 500 kHz | 16 |
| 10 | 471 kHz | 17 |
| 11 | 444 kHz | 18 |
| 12 | 421 kHz | 19 |
| 13 | 400 kHz | 20 |
| 14 | 381 kHz | 21 |
| 15 | 364 kHz | 22 |

---

## I2C_SLVn_ADDR / _REG / _CTRL (`0x25`-`0x30`, slaves 0-3)

For each slave **n** in 0…3 the three bytes follow the same layout
(RM section 4.9, pages 20-22):

| Reg | Bit 7 | Bits 6:0 |
|-----|-------|----------|
| `I2C_SLVn_ADDR` | `I2C_SLVn_RW` (0=write, 1=read) | `I2C_SLVn_ADDR[6:0]` (target 7-bit slave address) |

| Reg | Bits 7:0 |
|-----|----------|
| `I2C_SLVn_REG` | Target slave's internal start register |

| Reg | Bit 7 | Bit 6 | Bit 5 | Bit 4 | Bits 3:0 |
|-----|-------|-------|-------|-------|----------|
| `I2C_SLVn_CTRL` | `EN` | `BYTE_SW` | `REG_DIS` | `GRP` | `LEN[3:0]` |

Field meanings (RM section 4.9):

- `EN` — 1 enables this slave's transactions.
- `BYTE_SW` — 1 swaps high/low bytes within word pairs (pairing controlled
  by `GRP`).
- `REG_DIS` — 1 = transaction reads/writes data only (no register-address
  byte sent first).
- `GRP` — pairing convention for word swap: 0 = (even,odd), 1 = (odd,even).
- `LEN[3:0]` — number of bytes to transfer (0 = disabled).

---

## I2C_SLV4 (`0x31`-`0x35`)

Slave 4 is special: it carries a `_DO` (data-out) and `_DI` (data-in)
register and runs at most once per Sample Rate (RM section 4.13).

| Addr | Name | Layout |
|------|------|--------|
| `0x31` | `I2C_SLV4_ADDR` | `I2C_SLV4_RW [7]`, `ADDR[6:0]` |
| `0x32` | `I2C_SLV4_REG`  | `I2C_SLV4_REG[7:0]` |
| `0x33` | `I2C_SLV4_DO`   | Byte to write to slave 4 |
| `0x34` | `I2C_SLV4_CTRL` | `EN [7]`, `INT_EN [6]`, `REG_DIS [5]`, `I2C_MST_DLY[4:0]` |
| `0x35` | `I2C_SLV4_DI`   | Byte read from slave 4 (R only) |

`EN` auto-clears after the single transaction completes. Completion sets
`I2C_MST_STATUS.I2C_SLV4_DONE` (`0x36` bit 6) and, if `INT_EN = 1` and
`INT_ENABLE.I2C_MST_INT_EN = 1`, raises the host INT.

`I2C_MST_DLY[4:0]` controls the reduced access rate for any slave whose
`I2C_MST_DELAY_CTRL.I2C_SLVx_DLY_EN` is set: that slave is accessed every
`1 / (1 + I2C_MST_DLY)` Sample Rate periods. — RM section 4.13.

---

## I2C_MST_STATUS (`0x36`)

R only. Reset `0x00`. Reading this register clears all bits.

| Bit | Name | Description |
|----:|------|-------------|
| 7 | `PASS_THROUGH` | FSYNC pass-through flag. |
| 6 | `I2C_SLV4_DONE` | Slave-4 transaction completed. |
| 5 | `I2C_LOST_ARB` | Aux master lost arbitration (error). |
| 4 | `I2C_SLV4_NACK` | Aux NACK in slave-4 transaction. |
| 3 | `I2C_SLV3_NACK` | Aux NACK in slave-3 transaction. |
| 2 | `I2C_SLV2_NACK` | Aux NACK in slave-2 transaction. |
| 1 | `I2C_SLV1_NACK` | Aux NACK in slave-1 transaction. |
| 0 | `I2C_SLV0_NACK` | Aux NACK in slave-0 transaction. |

Each bit drives the host INT pin through `INT_ENABLE.I2C_MST_INT_EN`.

---

## I2C_SLVn_DO (`0x63`-`0x66`)

R/W. Reset `0x00`. One byte each for slaves 0…3 — the data byte that the
auxiliary master writes when the corresponding slave is in write mode
(`I2C_SLVn_RW = 0`).

| Addr | Name |
|------|------|
| `0x63` | `I2C_SLV0_DO` |
| `0x64` | `I2C_SLV1_DO` |
| `0x65` | `I2C_SLV2_DO` |
| `0x66` | `I2C_SLV3_DO` |

---

## I2C_MST_DELAY_CTRL (`0x67`)

R/W. Reset `0x00`. RM section 4.26.

| Bit | Name | Description |
|----:|------|-------------|
| 7 | `DELAY_ES_SHADOW`  | 1 = delay shadowing of external sensor data until all data has been received. |
| 6 | reserved           | — |
| 5 | reserved           | — |
| 4 | `I2C_SLV4_DLY_EN`  | 1 = slave 4 accessed at the reduced rate (`1 / (1 + I2C_MST_DLY)`). |
| 3 | `I2C_SLV3_DLY_EN`  | 1 = slave 3 reduced rate. |
| 2 | `I2C_SLV2_DLY_EN`  | 1 = slave 2 reduced rate. |
| 1 | `I2C_SLV1_DLY_EN`  | 1 = slave 1 reduced rate. |
| 0 | `I2C_SLV0_DLY_EN`  | 1 = slave 0 reduced rate. |

---

## SIGNAL_PATH_RESET (`0x68`)

Write-only (per RM section 4.27 the type is shown as Write Only / R/W
depending on the table; treat as write-only). Reset `0x00`.

| Bit | Name | Description |
|----:|------|-------------|
| 7:3 | reserved | — |
| 2 | `GYRO_RESET`  | 1 resets gyro analog + digital signal path. |
| 1 | `ACCEL_RESET` | 1 resets accel analog + digital signal path. |
| 0 | `TEMP_RESET`  | 1 resets temperature signal path. |

> "This register does not clear the sensor registers." — RM section 4.27.
> To also clear the sensor registers, use `USER_CTRL.SIG_COND_RESET`.

---

## MOT_DETECT_CTRL (`0x69`)

R/W. Reset `0x00`. RM section 4.28.

| Bit | Name | Description |
|----:|------|-------------|
| 7:6 | reserved | — |
| 5:4 | `ACCEL_ON_DELAY[1:0]` | Additional accel-power-on delay, 1 LSB = 1 ms (added to the default 4 ms). |
| 3:0 | reserved | — |

---

## WHO_AM_I (`0x75`)

R only. Reset value **`0x68`** (per RM page 8).

| Bit | Name | Description |
|----:|------|-------------|
| 7   | reserved (hard-coded 0) | — |
| 6:1 | `WHO_AM_I[6:1]` | Upper 6 bits of the MPU's 7-bit I2C address. POR value `110100`. |
| 0   | reserved (hard-coded 0) | — |

The whole-byte default value is `0x68`, regardless of the AD0 pin. — RM
section 4.34. FSW shall verify `WHO_AM_I == 0x68` immediately after
power-up to confirm the part is reachable on I2C.

---

[Prev: 06 Data registers](./06_register_details_data.md) | [Back to IMU index](./index.md) | [Next: 08 Init sequence](./08_init_sequence.md)
