# System Timers: Register Map

[Back to timer index](index.md) | [Back to ICD index](../index.md)

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| TIMER0 | `TIMER0_BASE` | `0x400b0000` |
| TIMER1 | `TIMER1_BASE` | `0x400b8000` |

The block has the `+0x1000` XOR, `+0x2000` set and `+0x3000` clear aliases of
§2.1.3.

## 12.8.5 Register List (Table 1226)

| Offset | Name | Type | Reset | Description |
|--------|------|------|-------|-------------|
| 0x00 | `TIMEHW` | WF | `0x00000000` | Write bits 63:32 of time; write `TIMELW` first |
| 0x04 | `TIMELW` | WF | `0x00000000` | Write bits 31:0 of time; takes effect when `TIMEHW` is written |
| 0x08 | `TIMEHR` | RO | `0x00000000` | Read bits 63:32 of time; read `TIMELR` first |
| 0x0c | `TIMELR` | RO | `0x00000000` | Read bits 31:0 of time; latches `TIMEHR` |
| 0x10 | `ALARM0` | RW | `0x00000000` | Arm alarm 0 for this low-word value |
| 0x14 | `ALARM1` | RW | `0x00000000` | Arm alarm 1 |
| 0x18 | `ALARM2` | RW | `0x00000000` | Arm alarm 2 |
| 0x1c | `ALARM3` | RW | `0x00000000` | Arm alarm 3 |
| 0x20 | `ARMED` | WC | `0x0` | Bits 3:0 armed status; write 1 to disarm |
| 0x24 | `TIMERAWH` | RO | `0x00000000` | Raw bits 63:32, no side effect |
| 0x28 | `TIMERAWL` | RO | `0x00000000` | Raw bits 31:0, no side effect |
| 0x2c | `DBGPAUSE` | RW | `0x6` | Pause while a debugger halts a core |
| 0x30 | `PAUSE` | RW | `0x0` | Pause the timer |
| 0x34 | `LOCKED` | RW | `0x0` | Set to disable write access until reset |
| 0x38 | `SOURCE` | RW | `0x0` | Tick or `clk_sys` |
| 0x3c | `INTR` | WC | `0x0` | Raw interrupts |
| 0x40 | `INTE` | RW | `0x0` | Interrupt enable |
| 0x44 | `INTF` | RW | `0x0` | Interrupt force |
| 0x48 | `INTS` | RO | `0x0` | Interrupt status after masking and forcing |

## ARMED (0x20), Table 1235

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:4 | reserved | - | - | |
| 3:0 | `ARMED` | WC | `0x0` | One bit per alarm; a write to `ALARMn` sets it, firing clears it, writing 1 clears it without firing |

## DBGPAUSE (0x2c), Table 1238

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 2 | `DBG1` | RW | `0x1` | Pause while processor 1 is in debug mode |
| 1 | `DBG0` | RW | `0x1` | Pause while processor 0 is in debug mode |
| 0 | reserved | - | - | |

## PAUSE (0x30), Table 1239 and LOCKED (0x34), Table 1240

| Register | Bit | Type | Reset | Description |
|----------|-----|------|-------|-------------|
| `PAUSE` | 0 | RW | `0x0` | 1 pauses the timer |
| `LOCKED` | 0 | RW | `0x0` | 1 disables write access to the timer; cleared only by reset |

## SOURCE (0x38), Table 1241

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 0 | `CLK_SYS` | RW | `0x0` | `0x0` `TICK`: count the tick from `TICKS`. `0x1` `CLK_SYS`: count system clock cycles |

## INTR (0x3c), INTE (0x40), INTF (0x44), INTS (0x48), Tables 1242 to 1245

| Bits | Field | `INTR` | `INTE` | `INTF` | `INTS` |
|------|-------|--------|--------|--------|--------|
| 3 | `ALARM_3` | WC | RW | RW | RO |
| 2 | `ALARM_2` | WC | RW | RW | RO |
| 1 | `ALARM_1` | WC | RW | RW | RO |
| 0 | `ALARM_0` | WC | RW | RW | RO |

All reset to 0. Bits 31:4 reserved.
