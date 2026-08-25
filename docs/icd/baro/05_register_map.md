# 05 — Register Map

[← Back to Baro ICD index](index.md)

This is the full register address map for the MPL3115A2, transcribed
from datasheet Table 9. Bit-level definitions for the most-used
registers are in [`06_register_details.md`](06_register_details.md).

## 5.1 Footnote Codes (used in the table)

| Code | Meaning |
|------|---------|
| (P)  | Contents preserved when transitioning ACTIVE → STANDBY. |
| (RA) | Contents reset when transitioning STANDBY → ACTIVE. |
| (M)  | Contents may be modified anytime in STANDBY or ACTIVE. |
| (S)  | Contents may be modified only in STANDBY (except SBYB, OST, RST in CTRL_REG1). |

## 5.2 Full Register Map

| Addr | Name | Reset | Reset on STBY→ACT | R/W | Auto-incr → | Comment / footnote |
|------|------|-------|-------------------|-----|-------------|--------------------|
| 0x00 | STATUS                | 0x00 | Yes | R   | 0x01 | Alias for DR_STATUS or F_STATUS, depending on F_MODE. (P)(RA) |
| 0x01 | OUT_P_MSB             | 0x00 | Yes | R   | 0x02 | Pressure bits 19..12. Doubles as F_DATA root pointer when FIFO enabled. (P)(RA) |
| 0x02 | OUT_P_CSB             | 0x00 | Yes | R   | 0x03 | Pressure bits 11..4. (P)(RA) |
| 0x03 | OUT_P_LSB             | 0x00 | Yes | R   | 0x04 | Pressure bits 3..0 (and Q18.2 fraction). (P)(RA) |
| 0x04 | OUT_T_MSB             | 0x00 | Yes | R   | 0x05 | Temperature bits 11..4. (P)(RA) |
| 0x05 | OUT_T_LSB             | 0x00 | Yes | R   | 0x00 | Temperature bits 3..0 (Q8.4 fraction). Wraps to STATUS. (P)(RA) |
| 0x06 | DR_STATUS             | 0x00 | Yes | R   | 0x07 | Data-ready status (alias of 0x00 when FIFO disabled). (P)(RA) |
| 0x07 | OUT_P_DELTA_MSB       | 0x00 | Yes | R   | 0x08 | Pressure delta bits 19..12. (P)(RA) |
| 0x08 | OUT_P_DELTA_CSB       | 0x00 | Yes | R   | 0x09 | Pressure delta bits 11..4. (P)(RA) |
| 0x09 | OUT_P_DELTA_LSB       | 0x00 | Yes | R   | 0x0A | Pressure delta bits 3..0. (P)(RA) |
| 0x0A | OUT_T_DELTA_MSB       | 0x00 | Yes | R   | 0x0B | Temperature delta bits 11..4. (P)(RA) |
| 0x0B | OUT_T_DELTA_LSB       | 0x00 | Yes | R   | 0x06 | Temperature delta bits 3..0. Wraps to DR_STATUS. (P)(RA) |
| 0x0C | **WHO_AM_I**          | **0xC4** | No | R | 0x0D | Fixed device ID. NVM-programmed. |
| 0x0D | F_STATUS              | 0x00 | Yes | R   | 0x0E | FIFO status (alias of 0x00 when FIFO enabled). (P)(RA) |
| 0x0E | F_DATA                | 0x00 | Yes | R   | 0x0E | FIFO 8-bit access. Pointer does not advance from this address. (P)(RA) |
| 0x0F | F_SETUP               | 0x00 | No  | R/W | 0x10 | FIFO mode + watermark. (M) |
| 0x10 | TIME_DLY              | 0x00 | Yes | R   | 0x11 | Ticks since last FIFO byte written; clears on FIFO drain. (P)(RA) |
| 0x11 | SYSMOD                | 0x00 | Yes | R   | 0x12 | Current system mode (0 = STANDBY, 1 = ACTIVE). (P)(RA) |
| 0x12 | INT_SOURCE            | 0x00 | No  | R   | 0x13 | Interrupt source flags. (P) |
| 0x13 | PT_DATA_CFG           | 0x00 | No  | R/W | 0x14 | Data-ready event flag enables. (M) |
| 0x14 | BAR_IN_MSB            | **0xC5** | No | R/W | 0x15 | Barometric input MSB (default = 101 326 Pa @ 2 Pa/LSB). (M) |
| 0x15 | BAR_IN_LSB            | **0xE7** | No | R/W | 0x16 | Barometric input LSB. (M) |
| 0x16 | P_TGT_MSB             | 0x00 | No  | R/W | 0x17 | Pressure/Altitude target MSB. (M) |
| 0x17 | P_TGT_LSB             | 0x00 | No  | R/W | 0x18 | Pressure/Altitude target LSB. (M) |
| 0x18 | T_TGT                 | 0x00 | No  | R/W | 0x19 | Temperature target (8-bit, 2's-complement °C). (M) |
| 0x19 | P_WND_MSB             | 0x00 | No  | R/W | 0x1A | Pressure/Altitude window MSB. (M) |
| 0x1A | P_WND_LSB             | 0x00 | No  | R/W | 0x1B | Pressure/Altitude window LSB. (M) |
| 0x1B | T_WND                 | 0x00 | No  | R/W | 0x1C | Temperature window (8-bit unsigned °C). (M) |
| 0x1C | P_MIN_MSB             | 0x00 | No  | R/W | 0x1D | Captured minimum pressure/altitude bits 19..12. |
| 0x1D | P_MIN_CSB             | 0x00 | No  | R/W | 0x1E | Bits 11..4. |
| 0x1E | P_MIN_LSB             | 0x00 | No  | R/W | 0x1F | Bits 3..0. |
| 0x1F | T_MIN_MSB             | 0x00 | No  | R/W | 0x20 | Captured minimum temperature bits 11..4. |
| 0x20 | T_MIN_LSB             | 0x00 | No  | R/W | 0x21 | Bits 3..0. |
| 0x21 | P_MAX_MSB             | 0x00 | No  | R/W | 0x22 | Captured maximum pressure/altitude bits 19..12. |
| 0x22 | P_MAX_CSB             | 0x00 | No  | R/W | 0x23 | Bits 11..4. |
| 0x23 | P_MAX_LSB             | 0x00 | No  | R/W | 0x24 | Bits 3..0. |
| 0x24 | T_MAX_MSB             | 0x00 | No  | R/W | 0x25 | Captured maximum temperature bits 11..4. |
| 0x25 | T_MAX_LSB             | 0x00 | No  | R/W | 0x26 | Bits 3..0. |
| 0x26 | **CTRL_REG1**         | 0x00 | No  | R/W | 0x27 | Mode (ALT/RAW), oversampling, OST, RST, SBYB. (S\*) |
| 0x27 | **CTRL_REG2**         | 0x00 | No  | R/W | 0x28 | Acquisition step (ST), ALARM_SEL, LOAD_OUTPUT. (S) |
| 0x28 | **CTRL_REG3**         | 0x00 | No  | R/W | 0x29 | INT1/INT2 polarity and push-pull / open-drain. (S) |
| 0x29 | **CTRL_REG4**         | 0x00 | No  | R/W | 0x2A | Interrupt enables (DRDY, FIFO, PW, TW, PTH, TTH, PCHG, TCHG). (S) |
| 0x2A | **CTRL_REG5**         | 0x00 | No  | R/W | 0x2B | INT1/INT2 routing per source. (S) |
| 0x2B | OFF_P                 | 0x00 | No  | R/W | 0x2C | Pressure user offset (8-bit s.c., 4 Pa/LSB, ±508 Pa). |
| 0x2C | OFF_T                 | 0x00 | No  | R/W | 0x2D | Temperature user offset (8-bit s.c., 0.0625 °C/LSB, -8 to +7.9375 °C). |
| 0x2D | OFF_H                 | 0x00 | No  | R/W | 0x0C | Altitude user offset (8-bit s.c., 1 m/LSB, -128 to +127 m). |

\* CTRL_REG1: SBYB, OST, and RST may be written in either STANDBY or
ACTIVE; all other bits require STANDBY.

## 5.3 FIFO-Mode Aliasing (Area A, F_MODE > 0)

When the FIFO is enabled, the bottom of the address space changes
behavior. From datasheet Table 10:

| Addr | F_MODE = 00 (FIFO disabled) | F_MODE > 00 (FIFO enabled) |
|------|------------------------------|----------------------------|
| 0x00 | DR_STATUS / STATUS           | F_STATUS                   |
| 0x01 | OUT_P_MSB                    | F_DATA (FIFO read pointer) |
| 0x02 | OUT_P_CSB                    | reserved (reads 0x00)      |
| 0x03 | OUT_P_LSB                    | reserved (reads 0x00)      |
| 0x04 | OUT_T_MSB                    | reserved (reads 0x00)      |
| 0x05 | OUT_T_LSB                    | reserved (reads 0x00)      |

The auto-increment pointer for FIFO mode wraps to `0x01` after each
F_DATA read so that successive reads continue draining the FIFO.

## 5.4 Most-Common Register Bursts

The following bursts are recommended by the datasheet quick-start
section (`F_RD` multi-byte reads):

| Burst | Length | Purpose |
|-------|--------|---------|
| 0x00 → 0x05 | 6 bytes | STATUS + Pressure + Temperature in one transaction (FIFO disabled). |
| 0x01 → 0x05 | 5 bytes | Pressure + Temperature without status (clears DRDY flag). |
| 0x07 → 0x0B | 5 bytes | Pressure delta + Temperature delta. |
| 0x01 (FIFO) | up to 160 bytes | Drain all 32 FIFO samples. |
| 0x14 → 0x15 | 2 bytes | Read/write barometric input. |
| 0x16 → 0x18 | 3 bytes | Read/write all targets. |
| 0x26 → 0x2A | 5 bytes | Read/write all five control registers. |
| 0x2B → 0x2D | 3 bytes | Read/write all user-offset trims. |

## 5.5 Quick Reference — Power-On Defaults

| Register | POR value | Notes |
|----------|-----------|-------|
| CTRL_REG1   | 0x00 | STANDBY, barometer mode, 1× OSR, no OST, no RST. |
| CTRL_REG2   | 0x00 | ST = 0 (1 s), no LOAD_OUTPUT, ALARM_SEL = 0. |
| CTRL_REG3   | 0x00 | INT1/INT2 active-low, push-pull. |
| CTRL_REG4   | 0x00 | All interrupt sources disabled. |
| CTRL_REG5   | 0x00 | All interrupts route to INT2. |
| F_SETUP     | 0x00 | FIFO disabled, watermark = 0. |
| PT_DATA_CFG | 0x00 | All data-event flags disabled. |
| BAR_IN      | 0xC5E7 | = 50 663 × 2 Pa = **101 326 Pa** (sea-level default). |
| OFF_P/T/H   | 0x00 | No user offsets. |
| WHO_AM_I    | 0xC4 | NVM-programmed. |

[← Back to Baro ICD index](index.md)
