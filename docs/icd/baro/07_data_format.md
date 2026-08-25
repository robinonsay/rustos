# 07 — Data Output Formats

[← Back to Baro ICD index](index.md)

The MPL3115A2 always reports samples in fixed-point Q-format. The
specific Q-format depends on the measurement mode (`CTRL_REG1.ALT`,
`CTRL_REG1.RAW`).

## 7.1 Pressure Output — Barometer Mode (Q18.2 unsigned)

When `ALT = 0` and `RAW = 0`, OUT_P is a **20-bit unsigned** Q18.2
fixed-point value in Pascals.

Layout across the three OUT_P bytes:

```
   OUT_P_MSB (0x01)         OUT_P_CSB (0x02)         OUT_P_LSB (0x03)
   PD19 ... PD12            PD11 ...   PD4           PD3 PD2 PD1 PD0 0 0 0 0
   |<-- integer ------------------------------>|<-frac->|
   18 integer bits                              2 frac bits  (and 4 zero bits)
```

Numerical reconstruction in C:

```c
uint32_t raw_p = ((uint32_t)msb << 16)
               | ((uint32_t)csb <<  8)
               |  (uint32_t)lsb;          // raw_p has fraction in bits 5..4
uint32_t p_x64 = raw_p >> 4;              // shift to align Q18.2 in lower bits
double pressure_pa = (double)p_x64 / 4.0; // Q18.2 -> Pa
```

Equivalent: `pressure_pa = raw_p / 64.0` because the bottom four bits
of OUT_P_LSB are zero, the device-emitted 24-bit value equals
`(integer_pa << 6) | (fraction_2bits << 4)`, i.e. `pressure_pa × 64`.

| Property | Value |
|----------|-------|
| Range          | 0 to 262 143.75 Pa (~262 kPa)  |
| LSB resolution | 0.25 Pa |
| Calibrated     | 50 to 110 kPa |

## 7.2 Altitude Output — Altimeter Mode (Q16.4 signed)

When `ALT = 1` and `RAW = 0`, OUT_P is a **20-bit signed (two's
complement)** Q16.4 fixed-point value in meters.

Layout:

```
   OUT_P_MSB (0x01)         OUT_P_CSB (0x02)         OUT_P_LSB (0x03)
   PD19 ... PD12            PD11 ...   PD4           PD3 PD2 PD1 PD0 0 0 0 0
   |<-- signed integer (16 bits) ------------>|<- fraction (4 bits) ->|
```

Numerical reconstruction:

```c
int32_t raw_alt = ((int32_t)(int8_t)msb << 16) // sign-extend MSB
                | ((int32_t)csb <<  8)
                | ((int32_t)lsb);
int32_t alt_x65536 = raw_alt;                  // 24-bit value = m × 65536
double altitude_m  = (double)alt_x65536 / 65536.0;
```

Note: the source datasheet describes the construction by left-shifting
into a 32-bit integer; equivalently, the integer meters are the
concatenation of OUT_P_MSB and OUT_P_CSB, and the fraction is the
upper nibble of OUT_P_LSB.

| Property | Value |
|----------|-------|
| Range          | -32 768.0 to +32 767.9375 m |
| LSB resolution | 0.0625 m (1/16 m) |
| Reference      | Sea level via BAR_IN (default 101 326 Pa) |

The altitude-from-pressure formula used internally is:

```
h = 44 330.77 × (1 - (P / P0)^0.1902632) + OFF_H
```

with `P0` = BAR_IN (in Pa) and `OFF_H` an integer-meter user offset
(see [§6.13](06_register_details.md#613-off_h-0x2d)).

## 7.3 Temperature Output (Q8.4 signed)

OUT_T is a **12-bit signed (two's complement)** Q8.4 fixed-point
value in °C (also written "Q12.4" in the datasheet, which describes
the total bit width — the integer part is 8 bits including sign).

```
   OUT_T_MSB (0x04)        OUT_T_LSB (0x05)
   TD11 ... TD4            TD3 TD2 TD1 TD0 0 0 0 0
   |<- signed int (8 bits) ->|<- fraction (4 bits) ->|
```

Numerical reconstruction:

```c
int16_t raw_t = ((int16_t)(int8_t)msb << 8) | (int16_t)lsb;
double temperature_c = (double)raw_t / 256.0;   // raw is °C × 256
```

| Property | Value |
|----------|-------|
| Range          | -128.0 to +127.9375 °C |
| LSB resolution | 0.0625 °C (1/16 °C) |
| Specified accuracy | ±1 °C @ 25 °C, ±3 °C over -40..+85 °C |

## 7.4 Delta Registers (OUT_P_DELTA, OUT_T_DELTA)

The delta registers (`0x07..0x0B`) carry the **change** between the
most recent sample and the previous sample. Format mirrors OUT_P /
OUT_T except:

- Pressure delta is always **20-bit signed** (Q18.2), regardless of
  Barometer vs. Altimeter mode (both give a meaningful negative delta
  on falling pressure / rising altitude).
- Altitude delta is 20-bit signed Q16.4 meters.
- Temperature delta is 12-bit signed Q8.4 °C.

In RAW mode these registers are not updated.

## 7.5 RAW Mode

When `CTRL_REG1.RAW = 1`:

- OUT_P holds 24 bits of raw ADC counts (no compensation, no scaling,
  no offset).
- OUT_T holds 16 bits of raw ADC counts.
- OUT_P_DELTA / OUT_T_DELTA are **not** updated.
- FIFO, alarms, change interrupts, and OFF_P/T/H are inactive.

## 7.6 BAR_IN Format

`BAR_IN` is **16-bit unsigned** with **2 Pa per LSB**.

```
P_sea_level_pa = BAR_IN_value × 2
```

Range: 0 to 131 070 Pa. Default after reset: `0xC5E7` = 50 663 →
**101 326 Pa**.

## 7.7 P_TGT / P_WND / T_TGT / T_WND Formats

| Register | Width | Sign | Units (Altimeter mode) | Units (Barometer mode) |
|----------|-------|------|------------------------|------------------------|
| P_TGT    | 16    | signed (alt) / unsigned (bar) | meters | 2 Pa/LSB |
| P_WND    | 16    | unsigned | meters | 2 Pa/LSB |
| T_TGT    | 8     | signed | °C (1 °C/LSB) | °C (1 °C/LSB) |
| T_WND    | 8     | unsigned | °C (1 °C/LSB) | °C (1 °C/LSB) |

## 7.8 P_MIN / P_MAX / T_MIN / T_MAX

Same Q-format as the corresponding live OUT_P / OUT_T registers (see
[§7.1](#71-pressure-output--barometer-mode-q182-unsigned),
[§7.2](#72-altitude-output--altimeter-mode-q164-signed),
[§7.3](#73-temperature-output-q84-signed)). Cleared on power-up or
by writing `0` to the register.

## 7.9 Format Summary Table

| Quantity | Source register(s) | Width | Sign | Q-format | LSB |
|----------|--------------------|-------|------|----------|-----|
| Pressure (Bar mode) | OUT_P_MSB/CSB/LSB | 20 | unsigned | Q18.2 | 0.25 Pa |
| Altitude (Alt mode) | OUT_P_MSB/CSB/LSB | 20 | signed   | Q16.4 | 0.0625 m |
| Temperature         | OUT_T_MSB/LSB     | 12 | signed   | Q8.4  | 0.0625 °C |
| Pressure delta      | OUT_P_DELTA_*     | 20 | signed   | Q18.2 | 0.25 Pa |
| Altitude delta      | OUT_P_DELTA_*     | 20 | signed   | Q16.4 | 0.0625 m |
| Temperature delta   | OUT_T_DELTA_*     | 12 | signed   | Q8.4  | 0.0625 °C |
| BAR_IN              | BAR_IN_MSB/LSB    | 16 | unsigned | Q16.0 (×2 Pa) | 2 Pa |
| OFF_P               | OFF_P             | 8  | signed   | Q8.0 (×4 Pa)  | 4 Pa |
| OFF_T               | OFF_T             | 8  | signed   | Q4.4  | 0.0625 °C |
| OFF_H               | OFF_H             | 8  | signed   | Q8.0  | 1 m |

## 7.10 Endianness

All multi-byte registers are **MSB-first** in the I2C wire order; the
device emits MSB at the lowest address.

[← Back to Baro ICD index](index.md)
