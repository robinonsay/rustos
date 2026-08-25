# 06 — Register Bit-Level Details

[← Back to Baro ICD index](index.md)

This file expands the bit fields of every register that the Juno FSW
baro driver is expected to touch. Reset values are POR unless
otherwise noted.

## 6.1 STATUS / DR_STATUS (0x00, 0x06)

When `F_MODE = 00`, address `0x00` aliases `DR_STATUS` at `0x06`.
When `F_MODE > 00`, address `0x00` aliases `F_STATUS` at `0x0D`
(see [§6.7](#67-f_status-0x0d)).

| Bit | Name  | Description |
|-----|-------|-------------|
| 7   | PTOW  | Pressure/Altitude OR Temperature data overwritten before read. |
| 6   | POW   | Pressure/Altitude data overwritten before read. |
| 5   | TOW   | Temperature data overwritten before read. |
| 4   | 0     | Reserved. |
| 3   | PTDR  | Pressure/Altitude OR Temperature new data ready. |
| 2   | PDR   | New Pressure/Altitude data available. |
| 1   | TDR   | New Temperature data available. |
| 0   | 0     | Reserved. |

Clearing rules (F_MODE = 00):
- PTOW/POW/PDR clear when OUT_P_MSB is read.
- TOW/TDR clear when OUT_T_MSB is read.
- PTDR clears when *either* OUT_P_MSB or OUT_T_MSB is read.

When F_MODE > 0, all flags clear on F_DATA read.

## 6.2 OUT_P_MSB / CSB / LSB (0x01, 0x02, 0x03)

20-bit Pressure or Altitude sample. See
[`07_data_format.md`](07_data_format.md) for the full Q-format
description.

### 6.2.1 OUT_P_MSB (0x01)

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | PD19 | PD18 | PD17 | PD16 | PD15 | PD14 | PD13 | PD12 |

### 6.2.2 OUT_P_CSB (0x02)

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | PD11 | PD10 | PD9 | PD8 | PD7 | PD6 | PD5 | PD4 |

### 6.2.3 OUT_P_LSB (0x03)

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | PD3 | PD2 | PD1 | PD0 | 0 | 0 | 0 | 0 |

In Barometer mode, bits PD1 and PD0 are the **fractional** Pa
component (Q18.2). In Altimeter mode, all four PD3..PD0 are the
fractional meter component (Q16.4).

## 6.3 OUT_T_MSB / LSB (0x04, 0x05)

12-bit signed temperature sample, Q8.4 °C.

### 6.3.1 OUT_T_MSB (0x04)

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | TD11 | TD10 | TD9 | TD8 | TD7 | TD6 | TD5 | TD4 |

### 6.3.2 OUT_T_LSB (0x05)

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | TD3 | TD2 | TD1 | TD0 | 0 | 0 | 0 | 0 |

## 6.4 WHO_AM_I (0x0C)

Read-only 8-bit device identifier.

| Bit | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
|-----|---|---|---|---|---|---|---|---|
| Field | 1 | 1 | 0 | 0 | 0 | 1 | 0 | 0 |

Reset (NVM-programmed) = **0xC4**.

## 6.5 PT_DATA_CFG (0x13)

| Bit | Name  | Description |
|-----|-------|-------------|
| 7..3 | 0    | Reserved. |
| 2   | DREM  | Data ready event mode. 1 = generate event on new data only when data changes; 0 = event on every new data. |
| 1   | PDEFE | Enable event flag on new pressure/altitude data. |
| 0   | TDEFE | Enable event flag on new temperature data. |

The Juno FSW driver typically writes `0x07` (DREM | PDEFE | TDEFE).

## 6.6 INT_SOURCE (0x12)

Read-only. Bit cleared by reading the corresponding source register
(STATUS, F_STATUS, OUT_P, etc.).

| Bit | Name      | Source register |
|-----|-----------|-----------------|
| 7   | SRC_DRDY  | STATUS / OUT_P / OUT_T |
| 6   | SRC_FIFO  | F_STATUS |
| 5   | SRC_PW    | Pressure window event |
| 4   | SRC_TW    | Temperature window event |
| 3   | SRC_PTH   | Pressure/altitude threshold event |
| 2   | SRC_TTH   | Temperature threshold event |
| 1   | SRC_PCHG  | Pressure/altitude change event |
| 0   | SRC_TCHG  | Temperature change event |

## 6.7 F_STATUS (0x0D)

| Bit | Name | Description |
|-----|------|-------------|
| 7   | F_OVF        | FIFO overflow. Latches; cleared by F_STATUS read. |
| 6   | F_WMRK_FLAG  | F_CNT > F_WMRK. Cleared by F_STATUS read. |
| 5..0 | F_CNT[5:0] | Number of samples currently in FIFO (0..32). |

## 6.8 F_DATA (0x0E)

8-bit read-only access to the FIFO read pointer. See
[`04_modes.md`](04_modes.md) §4.5.3 for the read sequence.

## 6.9 BAR_IN_MSB / LSB (0x14, 0x15)

16-bit unsigned barometric input for altitude calculation, in
**2 Pa/LSB** units.

| Field | Bits |
|-------|------|
| BAR_IN_MSB | BAR[15..8] |
| BAR_IN_LSB | BAR[7..0]  |

Reset (NVM-programmed) = `0xC5E7` = 50 663 → **101 326 Pa**
(US Standard Atmosphere sea-level).

To set local sea-level pressure P_sl in Pa:

```
BAR_IN = round(P_sl / 2)   // 16-bit unsigned
BAR_IN_MSB = (BAR_IN >> 8) & 0xFF
BAR_IN_LSB = BAR_IN & 0xFF
```

## 6.10 P_TGT_MSB / LSB (0x16, 0x17)

Pressure or altitude target value used for SRC_PTH / SRC_PW events.

- In **Altimeter mode**: 16-bit *signed* (2's complement) meters.
- In **Barometer mode**: 16-bit *unsigned* in 2 Pa/LSB.

## 6.11 OFF_P (0x2B)

| Bit | 7..0 |
|-----|------|
| Field | OFF_P[7..0] |

8-bit signed (2's complement). **4 Pa per LSB.** Range -512 to +508 Pa.

Applied to compensated pressure output before scaling to Pascals or
to altitude. Not applied in RAW mode.

## 6.12 OFF_T (0x2C)

| Bit | 7..0 |
|-----|------|
| Field | OFF_T[7..0] |

8-bit signed (2's complement). **0.0625 °C per LSB.** Range -8 to
+7.9375 °C.

## 6.13 OFF_H (0x2D)

| Bit | 7..0 |
|-----|------|
| Field | OFF_H[7..0] |

8-bit signed (2's complement). **1 m per LSB.** Range -128 to +127 m.

Applied to altitude output (Altimeter mode only).

## 6.14 CTRL_REG1 (0x26)

Mode and oversampling. Most bits writable only in STANDBY; SBYB,
OST, and RST may be written in either state.

| Bit | Name | Description |
|-----|------|-------------|
| 7   | ALT  | Mode select. 0 = Barometer, 1 = Altimeter. |
| 6   | RAW  | RAW output mode (overrides ALT). 1 = no compensation. |
| 5   | OS2  | Oversample bit 2. |
| 4   | OS1  | Oversample bit 1. |
| 3   | OS0  | Oversample bit 0. See OSR table in [§4.3](04_modes.md#43-oversampling-osr). |
| 2   | RST  | Software reset (auto-clears). |
| 1   | OST  | One-shot trigger. |
| 0   | SBYB | 0 = STANDBY, 1 = ACTIVE. |

Common values:

| Hex | Meaning |
|-----|---------|
| 0xB8 | Altimeter, 128× OSR, STANDBY (configure-then-arm). |
| 0xB9 | Altimeter, 128× OSR, ACTIVE.                       |
| 0x38 | Barometer, 128× OSR, STANDBY. |
| 0x39 | Barometer, 128× OSR, ACTIVE. |

## 6.15 CTRL_REG2 (0x27)

| Bit | Name        | Description |
|-----|-------------|-------------|
| 7   | 0           | Reserved. |
| 6   | 0           | Reserved. |
| 5   | LOAD_OUTPUT | Load OUT_P/OUT_T as targets when ALARM_SEL = 1. |
| 4   | ALARM_SEL   | 0 = use P_TGT/T_TGT; 1 = use OUT_P/OUT_T. |
| 3..0 | ST[3..0]   | Auto-acquisition time step = 2^ST seconds (1 s … 9 h). |

## 6.16 CTRL_REG3 (0x28) — Interrupt Pin Configuration

| Bit | Name   | Description |
|-----|--------|-------------|
| 7   | 0      | Reserved. |
| 6   | 0      | Reserved. |
| 5   | IPOL1  | INT1 polarity. 0 = active low, 1 = active high. |
| 4   | PP_OD1 | INT1 output type. 0 = push-pull (with internal pull-up), 1 = open-drain. |
| 3   | 0      | Reserved. |
| 2   | 0      | Reserved. |
| 1   | IPOL2  | INT2 polarity. |
| 0   | PP_OD2 | INT2 output type. |

Writing `0x11` configures both INT1 and INT2 as active-low,
open-drain (the most common configuration for shared-interrupt
buses).

## 6.17 CTRL_REG4 (0x29) — Interrupt Enables

| Bit | Name           | Enables |
|-----|----------------|---------|
| 7   | INT_EN_DRDY    | Data ready interrupt. |
| 6   | INT_EN_FIFO    | FIFO interrupt (overflow or watermark). |
| 5   | INT_EN_PW      | Pressure/altitude window. |
| 4   | INT_EN_TW      | Temperature window. |
| 3   | INT_EN_PTH     | Pressure/altitude threshold. |
| 2   | INT_EN_TTH     | Temperature threshold. |
| 1   | INT_EN_PCHG    | Pressure/altitude change. |
| 0   | INT_EN_TCHG    | Temperature change. |

Writing `0x80` enables only data-ready (the Juno default).

## 6.18 CTRL_REG5 (0x2A) — Interrupt Pin Routing

Per-source routing. For each bit:

- 0 = route to INT2 pin (default).
- 1 = route to INT1 pin.

| Bit | Name           | Routes |
|-----|----------------|--------|
| 7   | INT_CFG_DRDY   | Data ready. |
| 6   | INT_CFG_FIFO   | FIFO. |
| 5   | INT_CFG_PW     | Pressure window. |
| 4   | INT_CFG_TW     | Temperature window. |
| 3   | INT_CFG_PTH    | Pressure threshold. |
| 2   | INT_CFG_TTH    | Temperature threshold. |
| 1   | INT_CFG_PCHG   | Pressure change. |
| 0   | INT_CFG_TCHG   | Temperature change. |

All sources routed to a given pin are logically OR'd. The host must
read INT_SOURCE to demultiplex.

## 6.19 SYSMOD (0x11)

| Bit | Name    | Description |
|-----|---------|-------------|
| 7..1 | 0      | Reserved (always read 0). |
| 0   | SYSMOD  | 0 = STANDBY, 1 = ACTIVE. |

Useful for confirming a STANDBY ↔ ACTIVE transition completed.

[← Back to Baro ICD index](index.md)
