# PWM: Overview and Programming Model

[Back to PWM index](index.md) | [Back to ICD index](../index.md)

## 12.5.1 Overview

Each of the 12 slices has a 16-bit counter, an 8.4 fractional clock divider,
two output channels (A, B) with independent compare values, a configurable
wrap value, dual-slope (phase-correct) and trailing-edge modes, input modes
on the B pin for frequency or duty measurement, an interrupt and DMA request
at wrap, and single-count phase advance or retard. The global `EN` register
starts or stops several slices together. Changes from RP2040: 12 slices
instead of 8 (the extra four only on GPIO 32 to 47), and a second interrupt
line.

## 12.5.2 GPIO to channel map (Table 1129)

| GPIO | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|------|---|---|---|---|---|---|---|---|---|---|----|----|----|----|----|----|
| Channel | 0A | 0B | 1A | 1B | 2A | 2B | 3A | 3B | 4A | 4B | 5A | 5B | 6A | 6B | 7A | 7B |

| GPIO | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 |
|------|----|----|----|----|----|----|----|----|----|----|----|----|----|----|
| Channel | 0A | 0B | 1A | 1B | 2A | 2B | 3A | 3B | 4A | 4B | 5A | 5B | 6A | 6B |

GPIO 30 to 47 exist only in the QFN-80 package. Selecting the same output on
two GPIOs shows the same signal on both.

## 12.5.2.1 Pulse width modulation

The output compares the counter with the channel's `CC` value: high while
the counter is below `CC`, low otherwise (trailing-edge mode). The period is
`TOP + 1` counts.

## 12.5.2.2 0 % and 100 % duty

`CC = 0` gives a constant low and `CC = TOP + 1` a constant high, both without
toggling.

5. On and off: write `CC`, then `CTR = TOP`. The counter wraps at its next
   count and latches the new `CC` (see double buffering below) within one
   count, at most 256 `clk_sys` cycles, instead of at the end of the period.
   A duty change writes `CC` alone and takes effect at the next wrap.

## 12.5.2.3 Double buffering

Each slice holds two copies of `CC` and `TOP`. Software writes one copy at any
time; the counter uses the other, which is updated from the written copy when
the counter wraps to 0 (in phase-correct mode, at the 0-to-0 transition). A
change therefore never produces a partial pulse. `DIV` is not double-buffered.

## 12.5.2.6 Configuring the period

Period in `clk_sys` cycles:

    period = (TOP + 1) × (CSR_PH_CORRECT + 1) × (DIV_INT + DIV_FRAC / 16)

Output frequency:

    f = f_clk_sys / period

The divider slows counting to between one count per cycle and one count per
256 cycles. `DIV_INT = 0` divides by 256, and then no `DIV_FRAC` bit may be
set.

## 12.5.2.7 IRQ and DREQ

The block has two IRQ outputs. A slice raises its flag in `INTR` at each wrap;
`IRQ0_INTE` and `IRQ1_INTE` select which flags assert each IRQ, `IRQn_INTS`
shows the masked status, and writing a mask to `INTR` clears flags. Each slice
also has a DMA request at wrap.

## Bring-up order used by the cwht driver

1. Reset and release PWM: set `RESETS.RESET` bit 16 through the `+0x2000`
   alias, clear it through the `+0x3000` alias, wait for `RESET_DONE` bit 16;
   write `EN = 0`. Asserting the reset first makes every start, including a
   restart that did not reset PWM, begin with every slice stopped at `CC = 0`.
2. For an output pin: `CSR = 0`; `DIV`; `TOP`; `CC = 0`; `CTR = 0`;
   `CSR.EN = 1`. The slice now runs with the output low.
3. `IO_BANK0.GPIOn_CTRL = 4` (PWM, no overrides).
4. Clear `OD` and `ISO` of `PADS_BANK0.GPIOn` through the `+0x3000` alias,
   isolation last, so the pad comes up driven low.
