# 04 — Register Map

[Back to IMU index](./index.md)

The MPU-6050 register map (source: RM section 3, pages 6 to 8). Reset values
are `0x00` for every register **except** `0x6B` (`0x40`) and `0x75`
(`0x68`), per the note on RM page 8.

## 4.1 Configuration and Self-Test Registers

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x0D` | 13 | `SELF_TEST_X`   | R/W | `0x00` | XA_TEST[4-2], XG_TEST[4-0] |
| `0x0E` | 14 | `SELF_TEST_Y`   | R/W | `0x00` | YA_TEST[4-2], YG_TEST[4-0] |
| `0x0F` | 15 | `SELF_TEST_Z`   | R/W | `0x00` | ZA_TEST[4-2], ZG_TEST[4-0] |
| `0x10` | 16 | `SELF_TEST_A`   | R/W | `0x00` | XA_TEST[1-0], YA_TEST[1-0], ZA_TEST[1-0] |
| `0x19` | 25 | `SMPLRT_DIV`    | R/W | `0x00` | Sample-rate divider (8-bit) |
| `0x1A` | 26 | `CONFIG`        | R/W | `0x00` | EXT_SYNC_SET[2:0], DLPF_CFG[2:0] |
| `0x1B` | 27 | `GYRO_CONFIG`   | R/W | `0x00` | XG_ST, YG_ST, ZG_ST, FS_SEL[1:0] |
| `0x1C` | 28 | `ACCEL_CONFIG`  | R/W | `0x00` | XA_ST, YA_ST, ZA_ST, AFS_SEL[1:0] |
| `0x1F` | 31 | `MOT_THR`       | R/W | `0x00` | Motion-detection threshold (8-bit) |

## 4.2 FIFO and I2C Master Configuration

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x23` | 35 | `FIFO_EN`        | R/W | `0x00` | Per-channel FIFO enable (TEMP, XG, YG, ZG, ACCEL, SLV2, SLV1, SLV0) |
| `0x24` | 36 | `I2C_MST_CTRL`   | R/W | `0x00` | Aux-I2C master config + SLV3 FIFO enable |
| `0x25` | 37 | `I2C_SLV0_ADDR`  | R/W | `0x00` | RW + 7-bit slave-0 I2C address |
| `0x26` | 38 | `I2C_SLV0_REG`   | R/W | `0x00` | Slave-0 register start address |
| `0x27` | 39 | `I2C_SLV0_CTRL`  | R/W | `0x00` | EN, BYTE_SW, REG_DIS, GRP, LEN[3:0] |
| `0x28` | 40 | `I2C_SLV1_ADDR`  | R/W | `0x00` | RW + 7-bit slave-1 I2C address |
| `0x29` | 41 | `I2C_SLV1_REG`   | R/W | `0x00` | Slave-1 register start address |
| `0x2A` | 42 | `I2C_SLV1_CTRL`  | R/W | `0x00` | EN, BYTE_SW, REG_DIS, GRP, LEN[3:0] |
| `0x2B` | 43 | `I2C_SLV2_ADDR`  | R/W | `0x00` | RW + 7-bit slave-2 I2C address |
| `0x2C` | 44 | `I2C_SLV2_REG`   | R/W | `0x00` | Slave-2 register start address |
| `0x2D` | 45 | `I2C_SLV2_CTRL`  | R/W | `0x00` | EN, BYTE_SW, REG_DIS, GRP, LEN[3:0] |
| `0x2E` | 46 | `I2C_SLV3_ADDR`  | R/W | `0x00` | RW + 7-bit slave-3 I2C address |
| `0x2F` | 47 | `I2C_SLV3_REG`   | R/W | `0x00` | Slave-3 register start address |
| `0x30` | 48 | `I2C_SLV3_CTRL`  | R/W | `0x00` | EN, BYTE_SW, REG_DIS, GRP, LEN[3:0] |
| `0x31` | 49 | `I2C_SLV4_ADDR`  | R/W | `0x00` | RW + 7-bit slave-4 I2C address |
| `0x32` | 50 | `I2C_SLV4_REG`   | R/W | `0x00` | Slave-4 register start address |
| `0x33` | 51 | `I2C_SLV4_DO`    | R/W | `0x00` | Slave-4 data-out byte |
| `0x34` | 52 | `I2C_SLV4_CTRL`  | R/W | `0x00` | EN, INT_EN, REG_DIS, MST_DLY[4:0] |
| `0x35` | 53 | `I2C_SLV4_DI`    | R   | `0x00` | Slave-4 data-in byte |
| `0x36` | 54 | `I2C_MST_STATUS` | R   | `0x00` | NACK / lost-arb / done / pass-through flags |

## 4.3 Interrupts

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x37` | 55 | `INT_PIN_CFG`  | R/W | `0x00` | INT pin config + FSYNC int + I2C bypass |
| `0x38` | 56 | `INT_ENABLE`   | R/W | `0x00` | Per-source interrupt enable |
| `0x3A` | 58 | `INT_STATUS`   | R   | `0x00` | Per-source interrupt status (clears on read) |

## 4.4 Sensor Data (Read-Only)

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x3B` | 59 | `ACCEL_XOUT_H` | R | `0x00` | Accel X high byte |
| `0x3C` | 60 | `ACCEL_XOUT_L` | R | `0x00` | Accel X low byte |
| `0x3D` | 61 | `ACCEL_YOUT_H` | R | `0x00` | Accel Y high byte |
| `0x3E` | 62 | `ACCEL_YOUT_L` | R | `0x00` | Accel Y low byte |
| `0x3F` | 63 | `ACCEL_ZOUT_H` | R | `0x00` | Accel Z high byte |
| `0x40` | 64 | `ACCEL_ZOUT_L` | R | `0x00` | Accel Z low byte |
| `0x41` | 65 | `TEMP_OUT_H`   | R | `0x00` | Temperature high byte |
| `0x42` | 66 | `TEMP_OUT_L`   | R | `0x00` | Temperature low byte |
| `0x43` | 67 | `GYRO_XOUT_H`  | R | `0x00` | Gyro X high byte |
| `0x44` | 68 | `GYRO_XOUT_L`  | R | `0x00` | Gyro X low byte |
| `0x45` | 69 | `GYRO_YOUT_H`  | R | `0x00` | Gyro Y high byte |
| `0x46` | 70 | `GYRO_YOUT_L`  | R | `0x00` | Gyro Y low byte |
| `0x47` | 71 | `GYRO_ZOUT_H`  | R | `0x00` | Gyro Z high byte |
| `0x48` | 72 | `GYRO_ZOUT_L`  | R | `0x00` | Gyro Z low byte |

## 4.5 External Sensor Data (Read-Only)

`EXT_SENS_DATA_00` through `EXT_SENS_DATA_23` populated by the auxiliary
I2C master from external slaves 0-3 (slave 4 has its own `_DI` register).

| Addr (hex) | Addr (dec) | Name | R/W | Reset |
|-----------:|-----------:|------|-----|-------|
| `0x49` | 73 | `EXT_SENS_DATA_00` | R | `0x00` |
| `0x4A` | 74 | `EXT_SENS_DATA_01` | R | `0x00` |
| `0x4B` | 75 | `EXT_SENS_DATA_02` | R | `0x00` |
| `0x4C` | 76 | `EXT_SENS_DATA_03` | R | `0x00` |
| `0x4D` | 77 | `EXT_SENS_DATA_04` | R | `0x00` |
| `0x4E` | 78 | `EXT_SENS_DATA_05` | R | `0x00` |
| `0x4F` | 79 | `EXT_SENS_DATA_06` | R | `0x00` |
| `0x50` | 80 | `EXT_SENS_DATA_07` | R | `0x00` |
| `0x51` | 81 | `EXT_SENS_DATA_08` | R | `0x00` |
| `0x52` | 82 | `EXT_SENS_DATA_09` | R | `0x00` |
| `0x53` | 83 | `EXT_SENS_DATA_10` | R | `0x00` |
| `0x54` | 84 | `EXT_SENS_DATA_11` | R | `0x00` |
| `0x55` | 85 | `EXT_SENS_DATA_12` | R | `0x00` |
| `0x56` | 86 | `EXT_SENS_DATA_13` | R | `0x00` |
| `0x57` | 87 | `EXT_SENS_DATA_14` | R | `0x00` |
| `0x58` | 88 | `EXT_SENS_DATA_15` | R | `0x00` |
| `0x59` | 89 | `EXT_SENS_DATA_16` | R | `0x00` |
| `0x5A` | 90 | `EXT_SENS_DATA_17` | R | `0x00` |
| `0x5B` | 91 | `EXT_SENS_DATA_18` | R | `0x00` |
| `0x5C` | 92 | `EXT_SENS_DATA_19` | R | `0x00` |
| `0x5D` | 93 | `EXT_SENS_DATA_20` | R | `0x00` |
| `0x5E` | 94 | `EXT_SENS_DATA_21` | R | `0x00` |
| `0x5F` | 95 | `EXT_SENS_DATA_22` | R | `0x00` |
| `0x60` | 96 | `EXT_SENS_DATA_23` | R | `0x00` |

## 4.6 Aux-I2C Slave Data-Out and Misc

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x63` | 99  | `I2C_SLV0_DO`        | R/W | `0x00` | Slave-0 data-out byte (write mode) |
| `0x64` | 100 | `I2C_SLV1_DO`        | R/W | `0x00` | Slave-1 data-out byte |
| `0x65` | 101 | `I2C_SLV2_DO`        | R/W | `0x00` | Slave-2 data-out byte |
| `0x66` | 102 | `I2C_SLV3_DO`        | R/W | `0x00` | Slave-3 data-out byte |
| `0x67` | 103 | `I2C_MST_DELAY_CTRL` | R/W | `0x00` | Per-slave delay-enable + ES_SHADOW |
| `0x68` | 104 | `SIGNAL_PATH_RESET`  | R/W | `0x00` | Gyro / Accel / Temp signal-path reset (write) |
| `0x69` | 105 | `MOT_DETECT_CTRL`    | R/W | `0x00` | Accelerometer wake-on-motion delay |
| `0x6A` | 106 | `USER_CTRL`          | R/W | `0x00` | FIFO + aux-I2C master + sig-cond reset |
| `0x6B` | 107 | `PWR_MGMT_1`         | R/W | **`0x40`** | DEVICE_RESET, SLEEP (default 1), CYCLE, TEMP_DIS, CLKSEL |
| `0x6C` | 108 | `PWR_MGMT_2`         | R/W | `0x00` | LP_WAKE_CTRL + per-axis standby |

## 4.7 FIFO and Identity

| Addr (hex) | Addr (dec) | Name | R/W | Reset | Function |
|-----------:|-----------:|------|-----|-------|----------|
| `0x72` | 114 | `FIFO_COUNTH` | R/W | `0x00` | FIFO count high byte (read H first, latches L) |
| `0x73` | 115 | `FIFO_COUNTL` | R/W | `0x00` | FIFO count low byte |
| `0x74` | 116 | `FIFO_R_W`    | R/W | `0x00` | FIFO data port |
| `0x75` | 117 | `WHO_AM_I`    | R   | **`0x68`** | Device identity (bits 6:1 of 7-bit I2C address) |

## 4.8 Reserved / Undocumented Addresses

The register-map document does not list registers at every address in the
range `0x00`-`0x75`. Addresses not appearing in the tables above are
either reserved or undocumented and **shall not be written by FSW**.

---

[Prev: 03 I2C Interface](./03_i2c_interface.md) | [Back to IMU index](./index.md) | [Next: 05 Config registers in detail](./05_register_details_config.md)
