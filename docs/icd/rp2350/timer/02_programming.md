# System Timers: Programming Model

[Back to timer index](index.md) | [Back to ICD index](../index.md)

## Bring-up order

1. Start the tick: `TICKS.TIMER0_CTRL.ENABLE = 0`, `TIMER0_CYCLES = clk_ref MHz`,
   `ENABLE = 1`, wait for `RUNNING` (§8.5; cwht WP-SW-11 does this).
2. Release TIMER0 from reset: clear `RESETS.RESET` bit 23 (the `+0x3000`
   atomic-clear alias avoids a read-modify-write) and wait for bit 23 of
   `RESET_DONE` (§7.5).
3. Set `SOURCE = 0` (tick), `PAUSE = 0`, `DBGPAUSE` as required, `INTE = 0`;
   write `0xf` to `ARMED` and to `INTR` to start with every alarm disarmed and
   every latch clear.

## 12.8.4.1 Reading the 64-bit time

- Simple form: read `TIMELR`, then `TIMEHR`. Unsafe when two readers can
  interleave (two cores, or thread code and a handler), because the second
  `TIMELR` read replaces the first reader's latch.
- Race-free form (the SDK's `timer_time_us_64`): read `TIMERAWH`, then
  `TIMERAWL`, then `TIMERAWH` again; if the two high reads differ, read
  `TIMERAWL` again and use it with the second high value.

The cwht driver reads high, low, high, low unconditionally and selects the
consistent pair in a pure function, so the register sequence is fixed and the
decision is host-tested.

## 12.8.4.2 Arming an alarm

Datasheet steps: enable the alarm's bit in `INTE`; enable the IRQ line at the
processor (§3.2); write the target (current `TIMERAWL` plus the delay) to
`ALARMn`, which sets `ARMED`. After the fire, `ARMED` reads 0; clear the latch
by writing 1 to the alarm's `INTR` bit.

cwht driver sequence for `schedule_at(at)` on alarm `n`:

| Step | Access | Purpose |
|------|--------|---------|
| 1 | write `1 << n` to `ARMED`, then to `INTR` | Disarm and drop any latch, so no fire of an old target can be reported |
| 2 | read the time | Decide: target not in the future, more than 2^32 − 1 µs ahead, or armable |
| 3 | write `1 << n` to `INTE + 0x2000` | Enable the alarm's interrupt without touching the others |
| 4 | write the low 32 bits of `at` to `ALARMn` | Arm |
| 5 | read the time, `ARMED`, `INTR` | Still armed and time past target: missed the match, disarm and report due. Not armed and not latched: the arm failed, disarm and report an error. Otherwise pending |

## Clearing a fire

Read `INTR`; if the alarm's bit is set, write that bit back to clear it. The
write is made only after a read saw the bit, so a fire that latches between a
read and a clear is never lost.

## 12.8.4.3 Busy wait

The SDK busy-waits by comparing the raw time with a target. cwht does not
busy-wait on the timer; delays are alarms.
