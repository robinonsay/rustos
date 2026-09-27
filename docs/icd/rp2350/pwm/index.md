# PWM (Section 12.5)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Slices, channels, GPIO map, duty and period, double buffering, IRQ |
| [`02_registers.md`](02_registers.md) | Register map and bit fields |

## Base Address

| Instance | Symbol | Address | `RESETS.RESET` bit | IRQ lines |
|----------|--------|---------|--------------------|-----------|
| PWM | `PWM_BASE` | `0x400a8000` | 16 | `PWM_IRQ_WRAP_0` = 8, `PWM_IRQ_WRAP_1` = 9 |

Sources: datasheet §12.5.3 (p1083), §7.5 Table 534 (p504), §3.2 Table 94 (p82).

## Key Specifications

| Property | Value |
|----------|-------|
| Slices | 12 (8 to 11 only on GPIO 32 to 47, QFN-80) |
| Channels | A and B per slice; same period, independent duty |
| Counter | 16 bits, wraps at `TOP` (or counts back down in phase-correct mode) |
| Divider | 8.4 fixed point, 1 to 255 + 15/16 (`INT = 0` means 256) |
| Count clock | `clk_sys` |
| Duty range | 0 % (`CC = 0`) to 100 % (`CC = TOP + 1`), glitch-free at both ends |
| Double buffering | `CC` and `TOP` latch at the counter wrap |
| GPIO function | `FUNCSEL = 4` |

## cwht Driver Notes (WP-SW-03)

- The `pico2` driver `pwm::pwm::Rp2350Pwm` uses slices 0 to 7 only (the
  RP2350A reach them), one output pin per slice, trailing-edge mode, no
  inversion (cwht coding standard CS-36).
- Frequency is set by the smallest divider that fits the period in 16 bits,
  with `TOP` at most 65534 so that `CC = TOP + 1` still fits the `CC` field.
- Off is `CC = 0` with the slice still running, so off takes effect at the
  next wrap and the pin is held low.
- PWM interrupts are not used.

## Cross-References

- GPIO function select and pad control: [`../gpio/01_overview.md`](../gpio/01_overview.md), [`../gpio/02_pads.md`](../gpio/02_pads.md).
- `clk_sys`: [`../clocks/01_overview.md`](../clocks/01_overview.md).
