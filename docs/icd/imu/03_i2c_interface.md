# 03 — I2C Interface

[Back to IMU index](./index.md)

> **Scope.** This section describes the **primary I2C interface** that the
> host MCU uses to talk to the MPU-6050. The MPU-6050 also has an
> *auxiliary* I2C bus (AUX_DA / AUX_CL) over which it acts as a *master*
> toward external sensor slaves; that auxiliary bus is configured by the
> registers documented in
> [05_register_details_config.md](./05_register_details_config.md) and
> [07_register_details_other.md](./07_register_details_other.md), but it is
> not the bus the host uses.
>
> The InvenSense register-map document does not include a full I2C protocol
> tutorial; the items below cover what the document states about MPU-6050
> I2C addressing, transaction structure, and bus-level constraints. For
> protocol timing diagrams (tHD;DAT, tSU;STA, etc.) **see PDF** (Product
> Specification).

## 3.1 Bus Mode and Speed

| Attribute | Value | Source |
|-----------|-------|--------|
| Bus standard | I2C | RM section 2 |
| Maximum SCL frequency | 400 kHz (Fast Mode) | RM section 2 |
| Multi-master capable on primary bus | Not specified | — |
| Pull-ups | External (host board); not on chip | Implied by I2C |

## 3.2 Slave Addressing

The MPU-6050 is an I2C **slave** on the primary bus. Its 7-bit address is
formed as follows:

| 7-bit address bits | Source |
|--------------------|--------|
| `b6:b1` (upper six bits) | Hard-coded to `110100` (binary) = `0x34 << 1` upper part |
| `b0` (LSB) | Determined by the **AD0 pin** logic level |

Therefore the two possible 7-bit addresses are:

| AD0 pin | 7-bit address | Write byte (R/W=0) | Read byte (R/W=1) |
|---------|---------------|--------------------|--------------------|
| 0 (GND) | `0x68` | `0xD0` | `0xD1` |
| 1 (VLOGIC) | `0x69` | `0xD2` | `0xD3` |

> Source: RM section 4.34 (page 46): "The contents of WHO_AM_I are the
> upper 6 bits of the MPU-60X0's 7-bit I2C address. The least significant
> bit of the MPU-60X0's I2C address is determined by the value of the AD0
> pin."

**Juno FSW configuration:** AD0 is tied low → address `0x68`.

## 3.3 Register Access Model

The MPU-6050 follows the standard "register-pointer" I2C model used by most
MEMS sensors:

1. A **write** transaction sends a register address as the first data byte.
   Subsequent data bytes are written into successive registers (the internal
   register pointer auto-increments).
2. A **read** transaction first writes the register address (with a repeated
   start), then re-issues the slave address with R/W = 1 and reads one or
   more bytes. The internal register pointer auto-increments on each byte
   read.

The register-map document does not formally specify auto-increment, but the
register-map note on page 8 states:

> "Register Names ending in _H and _L contain the high and low bytes,
> respectively, of an internal register value."

…which implies that a multi-byte read of (for example) `ACCEL_XOUT_H` and
`ACCEL_XOUT_L` returns the high then low byte of the same 16-bit sample.
Multiple sections (e.g., RM section 4.18 page 30) describe **burst reads**
of the sensor-data registers as the recommended access pattern, confirming
the auto-increment behavior.

### 3.3.1 Single-byte write

```
START | 7-bit addr | W=0 | ACK | reg_addr | ACK | data | ACK | STOP
```

### 3.3.2 Multi-byte (burst) write

```
START | 7-bit addr | W=0 | ACK | reg_addr | ACK | d0 | ACK | d1 | ACK | ... | STOP
```

`reg_addr` is the starting register; data bytes go to `reg_addr`,
`reg_addr+1`, ….

### 3.3.3 Single-byte read

```
START | 7-bit addr | W=0 | ACK | reg_addr | ACK |
RSTART | 7-bit addr | R=1 | ACK | data | NACK | STOP
```

### 3.3.4 Multi-byte (burst) read

```
START | 7-bit addr | W=0 | ACK | reg_addr | ACK |
RSTART | 7-bit addr | R=1 | ACK | d0 | ACK | d1 | ACK | ... | dN | NACK | STOP
```

The host issues ACK after every received byte except the last; the final
byte is terminated by NACK + STOP per I2C convention.

## 3.4 Burst-Read Atomicity Guarantee

The register-map document explicitly states (RM section 4.18, page 30, and
repeated for temperature, gyro, and external-sensor data):

> "The data within the accelerometer sensors' internal register set is
> always updated at the Sample Rate. Meanwhile, the user-facing read
> register set duplicates the internal register set's data values whenever
> the serial interface is idle. This guarantees that a burst read of sensor
> registers will read measurements from the same sampling instant. Note
> that if burst reads are not used, the user is responsible for ensuring a
> set of single byte reads correspond to a single sampling instant by
> checking the Data Ready interrupt."

**Implication for FSW:** the canonical way to read a full sample is a single
14-byte burst read starting at `ACCEL_XOUT_H` (`0x3B`), which yields:

| Offset | Register | Content |
|--------|----------|---------|
| 0 | `0x3B` ACCEL_XOUT_H | accel X high byte |
| 1 | `0x3C` ACCEL_XOUT_L | accel X low byte |
| 2 | `0x3D` ACCEL_YOUT_H | accel Y high byte |
| 3 | `0x3E` ACCEL_YOUT_L | accel Y low byte |
| 4 | `0x3F` ACCEL_ZOUT_H | accel Z high byte |
| 5 | `0x40` ACCEL_ZOUT_L | accel Z low byte |
| 6 | `0x41` TEMP_OUT_H   | temp high byte |
| 7 | `0x42` TEMP_OUT_L   | temp low byte |
| 8 | `0x43` GYRO_XOUT_H  | gyro X high byte |
| 9 | `0x44` GYRO_XOUT_L  | gyro X low byte |
| 10 | `0x45` GYRO_YOUT_H | gyro Y high byte |
| 11 | `0x46` GYRO_YOUT_L | gyro Y low byte |
| 12 | `0x47` GYRO_ZOUT_H | gyro Z high byte |
| 13 | `0x48` GYRO_ZOUT_L | gyro Z low byte |

All 14 bytes are guaranteed to come from the same internal sampling instant.

## 3.5 Auxiliary I2C Bus and Bypass Mode

The MPU-6050 can be programmed to act as an I2C *master* on its auxiliary
bus (AUX_DA / AUX_CL) to read up to four external slaves and copy their
data into `EXT_SENS_DATA_*` registers. Alternatively it can be put into
**Bypass Mode** (`INT_PIN_CFG.I2C_BYPASS_EN = 1` and
`USER_CTRL.I2C_MST_EN = 0`) so that the host MCU directly drives the
auxiliary bus through the chip. — RM section 4.15 (page 27) and 4.29
(page 39).

> Bypass mode and auxiliary-master mode are not used by Juno FSW; the
> external magnetometer, if any, is read directly by the host MCU on a
> separate I2C bus.

## 3.6 Data-Ready Synchronization

The chip drives an `INT` pin pulse (or latch) every time a new sensor
sample has been written to the data registers, when
`INT_ENABLE.DATA_RDY_EN = 1` (RM section 4.16). Polling
`INT_STATUS.DATA_RDY_INT` is also valid (RM section 4.17).

---

[Prev: 02 Electrical](./02_electrical.md) | [Back to IMU index](./index.md) | [Next: 04 Register Map](./04_register_map.md)
