# 08 — Recommended Initialization Sequences

[← Back to Baro ICD index](index.md)

The following sequences are derived from Section 4 ("Quick Start") of
the source datasheet (Figures 5 and 6) and adapted for the Juno FSW
software-bus driver model.

All examples use the I2C 8-bit slave write address `0xC0` (read
address `0xC1`, 7-bit address `0x60`).

## 8.1 Universal Power-On Sequence

```
1. Wait for VDD to settle (at least the LDO turn-on time).
2. Read WHO_AM_I (0x0C). Expect 0xC4. On mismatch, abort.
3. Issue a soft reset: write CTRL_REG1 (0x26) = 0x04 (RST = 1).
4. Wait at least 20 ms for the reset to complete (RST self-clears).
5. Read CTRL_REG1 and verify it reads 0x00.
6. Proceed to one of the mode-specific configuration sequences below.
```

After a soft reset the device is in STANDBY with all configuration
registers at their POR defaults; BAR_IN is reloaded to 0xC5E7
(101 326 Pa).

## 8.2 Altimeter, Polling, OSR = 128 (preferred Juno FT1 default)

Mirrors datasheet Figure 5, polling branch:

| # | Action | Bus transaction |
|---|--------|------------------|
| 1 | Set Altimeter mode + OSR = 128, remain in STANDBY  | W 0x26 = `0xB8` |
| 2 | Enable data event flags (DREM | PDEFE | TDEFE)     | W 0x13 = `0x07` |
| 3 | Move to ACTIVE                                      | W 0x26 = `0xB9` |
| 4 | (loop) Read STATUS                                  | R 0x00 |
| 5 | If `(STATUS & 0x08) != 0`, burst-read 5 bytes       | R 0x01..0x05 |
| 6 | Convert OUT_P (Q16.4 m), OUT_T (Q8.4 °C)            | (host arithmetic) |

The first valid sample is available after one full conversion period
(≈ 512 ms at 128× OSR; allow up to 1 s to be safe). Subsequent
samples follow the auto-acquisition step in `CTRL_REG2.ST` (default
`0` = 1 s).

## 8.3 Altimeter, Interrupt-Driven, OSR = 128

Mirrors datasheet Figure 5, interrupt branch:

| # | Action | Bus transaction |
|---|--------|------------------|
| 1 | Altimeter, OSR = 128, STANDBY                     | W 0x26 = `0xB8` |
| 2 | Configure INT pins active-low, open-drain         | W 0x28 = `0x11` |
| 3 | Enable data-ready interrupt                       | W 0x29 = `0x80` |
| 4 | Enable PT data flags                              | W 0x13 = `0x07` |
| 5 | Move to ACTIVE                                     | W 0x26 = `0xB9` |
| 6 | (ISR) on INT2 edge, read INT_SOURCE               | R 0x12 |
| 7 | If `SRC_DRDY` set, burst-read OUT_P/OUT_T         | R 0x01..0x05 |

By default DRDY routes to **INT2**; route to INT1 by writing
`CTRL_REG5 (0x2A) = 0x80` before going ACTIVE.

## 8.4 Altimeter, FIFO Watermark = 10, OSR = 128

Mirrors datasheet Figure 6:

| # | Action | Bus transaction |
|---|--------|------------------|
| 1 | Altimeter, OSR = 128, STANDBY                     | W 0x26 = `0xB8` |
| 2 | INT pin active-low, open-drain                    | W 0x28 = `0x11` |
| 3 | Enable FIFO interrupt                             | W 0x29 = `0x40` |
| 4 | Enable PT data flags                              | W 0x13 = `0x07` |
| 5 | Set FIFO mode = 01 (circular), watermark = 10     | W 0x0F = `0x4A` |
| 6 | Move to ACTIVE                                     | W 0x26 = `0xB9` |
| 7 | (ISR) on INT2 edge, read INT_SOURCE               | R 0x12 |
| 8 | If `SRC_FIFO`, read F_STATUS (clears interrupt)   | R 0x0D |
| 9 | Burst-read 32 bytes via F_DATA (`0x01`)           | R 0x01 ×32 |

Each 5-byte block in the burst is one sample (P_MSB, P_CSB, P_LSB,
T_MSB, T_LSB). 32 samples = 160 bytes max per drain.

Acquisition rate is `2^ST` seconds (CTRL_REG2.ST). For 1 Hz FIFO
(the maximum), set `ST = 0`.

## 8.5 Barometer, Polling, OSR = 128

Identical to §8.2 except step 1 writes `0x38` (ALT cleared), step 3
writes `0x39`. Pressure is then in Q18.2 Pa.

## 8.6 One-Shot from STANDBY (lowest power)

For application contexts that read pressure infrequently
(e.g. ground checkout):

| # | Action | Bus transaction |
|---|--------|------------------|
| 1 | Configure mode + OSR, leave SBYB = 0              | W 0x26 = `0xB8` (Altimeter) |
| 2 | Enable PT flags                                   | W 0x13 = `0x07` |
| 3 | Trigger one shot                                  | W 0x26 = `0xBA` (OST = 1, SBYB = 0) |
| 4 | Wait T_ON (≤ 1 s @ 128× OSR)                      | (host delay) |
| 5 | Poll STATUS until PTDR = 1                        | R 0x00 |
| 6 | Burst-read OUT_P/OUT_T                            | R 0x01..0x05 |

The OST bit auto-clears when SBYB = 0; the device returns to STANDBY
immediately after the conversion completes.

## 8.7 Setting Local Sea-Level Pressure

When the local QNH is known (e.g. from a METAR or ground-station
barometer), program BAR_IN before going ACTIVE:

```
P_sl_pa = 101325                  // Pa (example)
BAR_IN  = P_sl_pa / 2 = 50662
W 0x14 = (BAR_IN >> 8) & 0xFF      // BAR_IN_MSB
W 0x15 =  BAR_IN       & 0xFF      // BAR_IN_LSB
```

Or use a single 2-byte burst write to `0x14`.

This affects only altitude calculation (Altimeter mode). Barometer
mode pressure values are unaffected.

## 8.8 Calibrating OFF_P / OFF_H at Ground Level

A typical pre-flight ground calibration:

1. Hold the rocket at known altitude `h_known` (e.g. launch-pad
   GPS altitude).
2. Configure Altimeter, OSR = 128, ACTIVE.
3. Read 8 samples; average to get `h_meas`.
4. Compute `delta = round(h_known - h_meas)` clamped to [-128, +127].
5. Write OFF_H (0x2D) = `delta` (8-bit signed).

## 8.9 Suggested Juno FSW Driver Init Pseudocode

```c
JUNO_STATUS_T baro_init(BARO_LIB_ROOT_T &root)
{
    // 1. Probe.
    uint8_t whoami;
    if (i2c_read1(0x60, 0x0C, &whoami) != OK) return ERR;
    if (whoami != 0xC4) return ERR;

    // 2. Soft reset.
    i2c_write1(0x60, 0x26, 0x04);
    sleep_ms(20);

    // 3. Configure altimeter, 128x OSR, STANDBY.
    i2c_write1(0x60, 0x26, 0xB8);

    // 4. Enable PT data event flags.
    i2c_write1(0x60, 0x13, 0x07);

    // 5. Set BAR_IN if QNH override is configured.
    if (root.qnh_override_pa != 0) {
        uint16_t bar = root.qnh_override_pa / 2;
        i2c_write1(0x60, 0x14, (bar >> 8) & 0xFF);
        i2c_write1(0x60, 0x15,  bar       & 0xFF);
    }

    // 6. Go ACTIVE.
    i2c_write1(0x60, 0x26, 0xB9);
    return OK;
}
```

(Pseudocode only — Juno driver-level details and JUNO_STATUS_T
mapping are specified in the baro library design document, not
this ICD.)

## 8.10 Mode-Change Recipe (running)

To change OSR, mode, or any other STANDBY-only field while the
device is running:

```
1. Write CTRL_REG1.SBYB = 0 (e.g. 0xB8 if currently Altimeter/128x).
2. Poll SYSMOD until it reads 0 (or wait 10 ms).
3. Write the new CTRL_REG1 / CTRL_REG2 / CTRL_REG3..5 values.
4. Write CTRL_REG1.SBYB = 1.
```

A subsequent first sample takes the configured T_ON to be valid.

[← Back to Baro ICD index](index.md)
