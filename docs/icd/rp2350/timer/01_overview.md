# System Timers: Overview

[Back to timer index](index.md) | [Back to ICD index](../index.md)

## 12.8.1 Overview

Each system timer instance keeps one 64-bit count that increments once per
tick and has four alarms. At a 1 µs tick the count cannot overflow in any
practical lifetime, so software treats it as monotonic.

Two instances exist so that the Secure and Non-secure worlds can each own
one. Changes from RP2040: two instances instead of one; the tick now comes
from the system tick generators (§8.5); the `LOCKED` and `SOURCE` registers
are new.

## 12.8.2 Counter

The 32-bit bus reaches the 64-bit count through register pairs:

| Pair | Purpose | Rule |
|------|---------|------|
| `TIMELW`, `TIMEHW` | Write the time | Write `TIMELW` first; the value is applied when `TIMEHW` is written |
| `TIMELR`, `TIMEHR` | Latched read | Read `TIMELR` first; that read latches `TIMEHR` until `TIMEHR` is read |
| `TIMERAWL`, `TIMERAWH` | Raw read | No latch and no side effect |

The datasheet cautions against writing the time while other software relies
on it being monotonic.

## 12.8.3 Alarms

- Each alarm compares its 32-bit `ALARMn` value with the low 32 bits of the
  count (`TIMELR`) and fires on equality.
- Writing `ALARMn` arms the alarm (its `ARMED` bit sets). Firing disarms it.
  Writing 1 to its `ARMED` bit disarms it without firing.
- A fire sets the alarm's bit in `INTR` (raw status, write 1 to clear). With
  the matching `INTE` bit set, the alarm's IRQ line is asserted. `INTF`
  forces an interrupt; `INTS` is the status after masking and forcing.
- Because only 32 bits compare, the furthest alarm is 2^32 µs (about
  72 minutes) ahead. A target already passed when `ALARMn` is written does
  not fire until the low word comes round again.

## Tick source (§12.8.4 note, §8.5)

The timer counts only while its tick runs. The tick generator for TIMER0 is
`TICKS.TIMER0_CTRL`/`TIMER0_CYCLES`; with `clk_ref` from the 12 MHz crystal,
`CYCLES = 12` gives 1 µs. `SOURCE = 1` bypasses the tick and counts `clk_sys`.

## Debug pause

`DBGPAUSE` (reset `0x6`) pauses the count while either core is halted by a
debugger, so time does not jump across a breakpoint. `PAUSE` pauses it under
software control.
