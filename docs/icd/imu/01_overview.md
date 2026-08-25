# 01 — MPU-6050 Overview

[Back to IMU index](./index.md)

## 1.1 Purpose

The MPU-6050 is a 6-axis Motion Processing Unit (MPU) integrating a 3-axis
gyroscope, a 3-axis accelerometer, and an on-chip Digital Motion Processor
(DMP) on a single silicon die. The DMP is capable of running 9-axis sensor
fusion algorithms (MotionFusion) when an external magnetometer is connected
on the auxiliary I2C bus.

This ICD covers only the **register-level interface** for the MPU-6050 over
the **primary I2C bus**. The DMP firmware interface is out of scope.

## 1.2 Headline Features (from RM section 2)

- 3-axis MEMS gyroscope, programmable full-scale: ±250, ±500, ±1000, ±2000 dps.
- 3-axis MEMS accelerometer, programmable full-scale: ±2 G, ±4 G, ±8 G, ±16 G.
- On-chip 16-bit ADCs for each gyro and accel axis.
- On-chip temperature sensor.
- Programmable Digital Low-Pass Filter (DLPF) shared by gyro and accel.
- Programmable sample-rate divider.
- 1024-byte FIFO buffer for low system overhead burst reads.
- Auxiliary I2C master interface for connecting up to 4 external slaves
  (e.g., a 3-axis magnetometer for 9-axis MotionFusion).
- Hardware self-test for both gyro and accel.
- INT pin with configurable polarity, drive type, and latch behavior.
- Motion-detection interrupt.
- Cycle (low-power) mode where the device wakes periodically to take a single
  accelerometer sample.

## 1.3 MPU-6000 vs MPU-6050

The MPU-6000 family contains two pin-compatible parts that share the same
register map. The differences relevant to firmware are:

| Aspect | MPU-6000 | MPU-6050 |
|--------|----------|----------|
| Primary serial interface | SPI **or** I2C | I2C only |
| SPI maximum clock | 20 MHz | n/a |
| I2C maximum clock | 400 kHz | 400 kHz |
| Logic interface voltage | Same as VDD (single supply) | VLOGIC pin (separate I/O level) |
| `USER_CTRL.I2C_IF_DIS` (bit 4) | Set to 1 to use SPI | **Always write 0** |

> Source: RM section 2 (page 5) and section 4.29 (page 39).

**Juno FSW uses the MPU-6050 over I2C.** Throughout this ICD, anything
specific to the MPU-6000 is called out; otherwise the description applies to
the MPU-6050.

## 1.4 Block-level Architecture

The MPU-6050 contains the following functional blocks (per RM section 2):

```
+---------------------------------------------------------------+
|                          MPU-6050                             |
|                                                               |
|  +-------------+    +-------------+    +-------------------+  |
|  | 3-axis gyro |--->|   16-bit    |--->|  Sensor data      |  |
|  +-------------+    |    ADCs     |    |  registers        |  |
|                     +-------------+    |  (0x3B..0x48)     |  |
|  +-------------+    +-------------+    |                   |  |
|  | 3-axis accel|--->|   16-bit    |--->|                   |  |
|  +-------------+    |    ADCs     |    |                   |  |
|                     +-------------+    |                   |  |
|  +-------------+    +-------------+    |                   |  |
|  | temp sensor |--->|   16-bit    |--->|                   |  |
|  +-------------+    |    ADC      |    +---------+---------+  |
|                     +-------------+              |            |
|                                                  v            |
|              +------------+              +---------------+    |
|              | DLPF + DHPF|<-------------|    FIFO       |    |
|              +------------+              |  (1024 bytes) |    |
|                                          +---------------+    |
|              +------------+              +---------------+    |
|              | Aux I2C    |<------------>|  Primary I2C  |--+ |
|              | master     |              |  + INT pin    |  | |
|              +------------+              +---------------+  | |
+---------------------------------------------------------------+
                                                              |
                                                              v
                                                       Host MCU bus
```

> Note: the source register map document does not include a labeled block
> diagram; the figure above is a textual summary of section 2 of the
> register manual. For the canonical signal-path diagram, **see PDF**
> (the diagram resides in the MPU-6000/6050 *Product Specification*, not in
> the register manual).

## 1.5 Typical Applications (from RM section 2)

- Smartphones, tablets, wearables.
- Game controllers, 3D mice, remote controls.
- Robotics and motion-tracked toys.
- Inertial measurement / attitude estimation for small UAVs and rockets.

## 1.6 References

- RM-MPU-6000A Rev 4.0 — Register Map and Descriptions
  (the source document for this ICD).
- MPU-6000/MPU-6050 Product Specification — referenced by RM for electrical
  specs, signal-path diagrams, motion-detection behavior, and self-test
  pass/fail limits. **Not included in this ICD; see PDF.**

---

[Back to IMU index](./index.md) | [Next: 02 Electrical](./02_electrical.md)
