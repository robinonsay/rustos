# 08 — Initialization Sequence

[Back to IMU index](./index.md)

This file describes the recommended power-on initialization sequence for
the MPU-6050 over I2C. Every step is justified by a citation back to the
register-map document; nothing is fabricated.

---

## 8.1 Preconditions

- VDD and VLOGIC are stable.
- The host MCU's primary I2C controller is configured for ≤ 400 kHz.
- AD0 is hard-wired to GND, giving 7-bit slave address `0x68`.
- At least one Sample Rate period has elapsed since power-on (so the
  device's internal startup sequence has completed).

> "The device will come up in sleep mode upon power-up."
> — RM section 4 (page 9).

## 8.2 Recommended Sequence

The order below is the conservative sequence that the register-map
document supports across sections 4.1, 4.2, 4.4, 4.5, 4.15, 4.16, 4.17,
4.27, 4.29, 4.30, and 4.34.

### Step 1 — Sanity-check identity

| Action | Register | Value |
|--------|----------|-------|
| Read `WHO_AM_I` | `0x75` | Expect `0x68` |

If the read does not return `0x68`, abort initialization and report a
hardware fault. — RM section 4.34.

### Step 2 — Wake from sleep and select clock source

The device powers on with `PWR_MGMT_1 = 0x40` (i.e., `SLEEP = 1`,
`CLKSEL = 0`). To wake it and switch the clock to a gyro-PLL reference
(recommended for stability per RM section 4.30):

| Action | Register | Value | Effect |
|--------|----------|-------|--------|
| Write `PWR_MGMT_1` | `0x6B` | `0x01` | `DEVICE_RESET = 0`, `SLEEP = 0`, `CYCLE = 0`, `TEMP_DIS = 0`, `CLKSEL = 1` (PLL with X-axis gyro reference). |

> "It is highly recommended that the device be configured to use one of
> the gyroscopes (or an external clock source) as the clock reference for
> improved stability." — RM section 4.30.

After wake-up, allow the gyro PLL to lock before proceeding. The
register-map document does not specify a fixed lock time; FSW shall wait
at least one Sample Rate period (worst case 1 ms at default settings)
before issuing further configuration writes.

### Step 3 — (Optional) Soft-reset and re-wake

If a clean state is required (e.g., warm restart, or recovering from
unknown configuration), issue a device reset and re-execute Step 2:

| Action | Register | Value |
|--------|----------|-------|
| Write `PWR_MGMT_1` | `0x6B` | `0x80` (`DEVICE_RESET = 1`) |
| Wait | — | Until the bit auto-clears (RM section 4.30) |
| Write `SIGNAL_PATH_RESET` | `0x68` | `0x07` (gyro + accel + temp reset) |
| Wait | — | One Sample Rate period |
| Re-do Step 2 | — | — |

`DEVICE_RESET` reverts all internal registers to their reset values
(RM section 4.30). `SIGNAL_PATH_RESET` flushes the analog/digital paths
without clearing the sensor data registers (RM section 4.27).

### Step 4 — Configure DLPF and Sample Rate

Choose `DLPF_CFG` first, since it determines the gyro output rate that
`SMPLRT_DIV` divides:

| Action | Register | Value (example) | Notes |
|--------|----------|-----------------|-------|
| Write `CONFIG` | `0x1A` | `0x03` | `EXT_SYNC_SET = 0`, `DLPF_CFG = 3` → 42 Hz gyro / 44 Hz accel BW, gyro Fs = 1 kHz. |
| Write `SMPLRT_DIV` | `0x19` | `0x04` | Sample Rate = 1 kHz / (1 + 4) = 200 Hz. |

The exact `DLPF_CFG` and `SMPLRT_DIV` values are mission-specific and
shall be set by the application; this ICD only specifies the
write-ordering. — RM sections 4.2 and 4.3.

### Step 5 — Set full-scale ranges (FT1: ±16 G / ±2000 dps)

| Action | Register | Value | Effect |
|--------|----------|-------|--------|
| Write `GYRO_CONFIG`  | `0x1B` | `0x18` | `FS_SEL = 3` → ±2000 dps, no self-test. |
| Write `ACCEL_CONFIG` | `0x1C` | `0x18` | `AFS_SEL = 3` → ±16 G, no self-test. |

— RM sections 4.4 and 4.5. The corresponding sensitivity is 16.4 LSB/dps
for the gyro and 2048 LSB/G for the accel (sections 4.18, 4.20).

### Step 6 — Configure interrupts

Disable the auxiliary-I2C master and Bypass mode (Juno does not use
either), then choose INT pin behavior and enable the Data-Ready source:

| Action | Register | Value | Effect |
|--------|----------|-------|--------|
| Write `USER_CTRL`   | `0x6A` | `0x00` | All FIFO / aux-master / SPI features disabled. |
| Write `INT_PIN_CFG` | `0x37` | `0x10` | Active-high, push-pull, 50 us pulse, INT_RD_CLEAR = 1, no FSYNC, no bypass. |
| Write `INT_ENABLE`  | `0x38` | `0x01` | `DATA_RDY_EN = 1`; all other sources disabled. |

— RM sections 4.15, 4.16, 4.29.

### Step 7 — Clear stale interrupt status

| Action | Register | Effect |
|--------|----------|--------|
| Read `INT_STATUS`  | `0x3A` | Clears all latched interrupt-status bits. |
| Read `I2C_MST_STATUS` | `0x36` | Clears NACK / lost-arb / done / pass-through bits. |

— RM sections 4.14, 4.17.

### Step 8 — Wait for first Data-Ready

Either poll `INT_STATUS.DATA_RDY_INT` (`0x3A` bit 0) or wait for the INT
pin assertion. The first valid sample is available one Sample Rate period
after Step 5. — RM section 4.16.

## 8.3 Steady-State Sample Read

Once initialized, the canonical steady-state read is a 14-byte burst
starting at `ACCEL_XOUT_H`:

| Action | Register | Length | Notes |
|--------|----------|--------|-------|
| Burst read | `0x3B` | 14 bytes | Yields accel(X,Y,Z), temp, gyro(X,Y,Z) from one sampling instant. |

— RM section 4.18 (atomicity guarantee), reproduced in
[03_i2c_interface.md](./03_i2c_interface.md#3-4-burst-read-atomicity-guarantee).

## 8.4 Init Sequence Diagram

```
+------+   I2C    +------------+
| Host |--------->| MPU-6050   |
+------+          +------------+
   |                    |
   |--- read 0x75 ----->|
   |<------ 0x68 -------|       Step 1: WHO_AM_I check
   |                    |
   |--- write 0x6B 0x01->|      Step 2: wake + CLKSEL=1
   |                    |
   |--- write 0x1A 0x03->|      Step 4a: CONFIG (DLPF)
   |--- write 0x19 0x04->|      Step 4b: SMPLRT_DIV
   |                    |
   |--- write 0x1B 0x18->|      Step 5a: GYRO_CONFIG ±2000 dps
   |--- write 0x1C 0x18->|      Step 5b: ACCEL_CONFIG ±16 G
   |                    |
   |--- write 0x6A 0x00->|      Step 6a: USER_CTRL
   |--- write 0x37 0x10->|      Step 6b: INT_PIN_CFG
   |--- write 0x38 0x01->|      Step 6c: INT_ENABLE (DATA_RDY)
   |                    |
   |--- read 0x3A ------>|      Step 7a: clear INT_STATUS
   |<------ 0x?? -------|
   |--- read 0x36 ------>|      Step 7b: clear I2C_MST_STATUS
   |<------ 0x?? -------|
   |                    |
   |    (wait 1 / Sample Rate)  Step 8: first sample ready
   |                    |
   |--- burst read 0x3B (14)-->|  Steady state
   |<--- 14 bytes -------|
```

## 8.5 Failure Handling

Per project constraints (`ai/memory/constraints.md`, "Determinism"
section), every step above shall report a status code on failure rather
than silently continuing:

- I2C NACK on any write → `JUNO_STATUS_<I2C-error>`; do not proceed.
- `WHO_AM_I != 0x68` → identity-mismatch error; do not proceed.
- First Data-Ready not asserted within `2 / SampleRate` → timeout error.

The exact status-code mapping is the responsibility of the IMU library
design document; this ICD specifies only the register-level contract.

---

[Prev: 07 Other registers](./07_register_details_other.md) | [Back to IMU index](./index.md)
