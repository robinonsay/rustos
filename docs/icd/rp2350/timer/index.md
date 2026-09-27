# System Timers (Section 12.8)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Counter, alarms, tick source, interrupts |
| [`02_programming.md`](02_programming.md) | Reading the 64-bit time, arming and clearing alarms, bring-up order |
| [`03_registers.md`](03_registers.md) | Register map and bit fields |

## Base Addresses

| Instance | Symbol | Address | `RESETS.RESET` bit | IRQ lines |
|----------|--------|---------|--------------------|-----------|
| TIMER0 | `TIMER0_BASE` | `0x400b0000` | 23 | `TIMER0_IRQ_0..3` = 0 to 3 |
| TIMER1 | `TIMER1_BASE` | `0x400b8000` | 24 | `TIMER1_IRQ_0..3` = 4 to 7 |

Sources: datasheet §12.8.5 (p1184), §7.5 Table 534 (p504), §3.2 Table 94 (p82).

## Key Specifications

| Property | Value |
|----------|-------|
| Counter | 64 bits, one count per tick |
| Tick | from the `TICKS` block (§8.5): `TIMER0_CYCLES` periods of `clk_ref`; 12 at 12 MHz for 1 µs |
| Alternative source | `SOURCE.CLK_SYS = 1` counts `clk_sys` cycles instead of the tick |
| Alarms | 4 per instance, one IRQ each, compare on the low 32 bits |
| Longest alarm | 2^32 − 1 µs, about 71.6 minutes |
| Security | TIMER0 and TIMER1 can sit in different security domains |

## cwht Driver Notes (WP-SW-01)

- The `pico2` driver `timer::timer::Rp2350Timer0` uses TIMER0 only, with the
  tick at 1 µs started by `clocks::clocks::Rp2350Clocks::init` (WP-SW-11).
- Time is read from `TIMERAWH`/`TIMERAWL` (no latch), never from
  `TIMEHR`/`TIMELR`, so reads from thread code and handlers cannot corrupt
  each other.
- The time is never written (`TIMEHW`/`TIMELW`) and `LOCKED` is never set.
- cwht uses ALARM0 for keyer element timing and ALARM1 for the 1 kHz input
  sampling tick (cwht 07 section 19).

## Cross-References

- Tick generators and `clk_ref`: [`../clocks/01_overview.md`](../clocks/01_overview.md) and datasheet §8.5.
- Interrupt numbering and NVIC: datasheet §3.2 and §3.7.5.
