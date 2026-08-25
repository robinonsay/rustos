# Clocks: Programming Model & Init

[Back to clocks index](index.md) | [Back to ICD index](../index.md)

## 8.1.5.1. Configuring a Clock Generator

Required inputs to configure a generator:

1. Source frequency (Hz).
2. Mux / aux-mux position of the source.
3. Desired output frequency.

### Algorithm (paraphrased from pico-sdk `clock_configure_internal`)

```
fn clock_configure(clock, src, auxsrc, src_freq, freq):
    # 1. If divisor is increasing, set divisor before source. This avoids
    #    a momentary overspeed when switching to a faster source.
    if new_div > current_div:
        write div register

    # 2. If glitchless slice (clk_ref/clk_sys) and switching to AUX,
    #    move glitchless mux away from AUX first to avoid passing glitches.
    if has_glitchless_mux(clock) and src == AUX:
        clear CTRL.SRC
        wait until SELECTED & 1

    # 3. Otherwise, disable the clock cleanly to avoid aux-mux glitches.
    else:
        clear CTRL.ENABLE
        busy_wait(3 * (clk_sys / clock_freq))

    # 4. Set aux mux first.
    write_masked CTRL, auxsrc << AUXSRC_LSB

    # 5. Then set glitchless src (if applicable) and poll SELECTED.
    if has_glitchless_mux(clock):
        write_masked CTRL, src << SRC_LSB
        wait until SELECTED & (1 << src)

    # 6. Re-enable.
    set CTRL.ENABLE

    # 7. Apply final divisor.
    write div register
    record configured_freq
```

### Computing the Divisor

```
div64 = (src_freq << DIV_INT_LSB) / freq
if div64 fits in 32 bits:
    div = div64
    actual_freq = (src_freq << DIV_INT_LSB) / div
else:
    div = 0    # max divider
    actual_freq = src_freq >> (32 - DIV_INT_LSB)
```

### Common Helper Forms

- `clock_configure_int_divider(clock, src, auxsrc, src_freq, int_div)` —
  forces an integer divisor.
- `clock_configure_undivided(clock, src, auxsrc, src_freq)` — divisor = 1.

### Example: clk_sys = PLL_SYS / 1 = 125 MHz

```
clock_configure_undivided(
    clk_sys,
    CLOCKS_CLK_SYS_CTRL_SRC_VALUE_CLKSRC_CLK_SYS_AUX,
    CLOCKS_CLK_SYS_CTRL_AUXSRC_VALUE_CLKSRC_PLL_SYS,
    SYS_CLK_HZ);
```

## 8.1.5.2. Using the Frequency Counter

Procedure:

1. Set `FC0_REF_KHZ` to the frequency of `clk_ref` in kHz.
2. Optionally set `FC0_MIN_KHZ` / `FC0_MAX_KHZ` for pass/fail testing
   (set MIN=0, MAX=0xffffffff to disable).
3. Set `FC0_INTERVAL` (test window selector).
4. Write `FC0_SRC` with the source mux value — this starts the measurement.
5. Poll `FC0_STATUS.DONE` until set.
6. Read `FC0_RESULT >> KHZ_LSB` for the frequency in kHz.

Always wait for `FC0_STATUS.RUNNING == 0` before starting a new measurement.

## 8.1.5.3. Configuring a GPIO Output Clock

Each `clk_gpout0..3` generator can be routed to a GPIO via the function
select. Default GPIO mappings (from SDK):

| GPIO | Clock |
|------|-------|
| 21 | `clk_gpout0` |
| 23 | `clk_gpout1` |
| 24 | `clk_gpout2` |
| 25 | `clk_gpout3` |
| 13 | `clk_gpout0` (alternate) |
| 15 | `clk_gpout1` (alternate) |

Steps:

1. Program the gpclk generator's `CTRL` (`AUXSRC | ENABLE`).
2. Program the divider (`INT << INT_LSB | FRAC << FRAC_LSB`).
3. Set the GPIO function to `GPIO_FUNC_GPCK`.

## 8.1.5.4. Configuring a GPIO Input Clock

Default GPIO → GPIN mapping:

| GPIO | GPIN |
|------|------|
| 20 | GPIN0 |
| 22 | GPIN1 |
| 12 | GPIN0 (alt) |
| 14 | GPIN1 (alt) |

`GPIN0/1` is always selected via the aux-mux. The glitchless source
(if applicable) must be set to `1` (AUX). The driver computes the auxsrc as
`gpin0_src[clock] + gpin_index`.

## 8.1.5.5. Enabling Resus

Initial setup:

```
irq_set_exclusive_handler(CLOCKS_IRQ, clocks_irq_handler);
clocks_hw->inte = CLOCKS_INTE_CLK_SYS_RESUS_BITS;
irq_set_enabled(CLOCKS_IRQ, true);
clocks_hw->resus.ctrl =
    CLOCKS_CLK_SYS_RESUS_CTRL_ENABLE_BITS | timeout_value;
```

Recommended `timeout_value`: choose so that
`timeout >= 2 * clk_ref_freq / clk_sys_min_freq`. Example: with 3 MHz `clk_ref`
and a 1 MHz floor for `clk_sys`, use a timeout of `2*3*1 = 6`.

## 8.1.5.6. Configuring Sleep Mode (informational)

System sleep is active when both cores plus DMA are idle. The clocks block
swaps `WAKE_EN` for `SLEEP_EN`. For maximum power savings:

- Stop unused PLLs.
- Increase divisors of generated clocks.
- Stop external clocks.

`clk_sys` is always sent to the cores during sleep so they can wake.

## FT1 FSW Recommended Init Sequence

1. **Boot** — chip starts on ROSC, `clk_sys` ≈ 11 MHz.
2. Enable XOSC, wait for `XOSC_STABLE`.
3. Switch `clk_ref` to XOSC.
4. Configure PLL_SYS for the desired `clk_sys` (e.g., 125 MHz or 150 MHz).
5. Wait for PLL_SYS `LOCK`.
6. Switch `clk_sys` glitchless mux to AUX, then aux to `CLKSRC_PLL_SYS`.
7. Configure PLL_USB for 48 MHz; wait for `LOCK`.
8. Configure `clk_usb` and `clk_adc` from PLL_USB at 48 MHz.
9. Configure `clk_peri` from PLL_SYS (or PLL_USB if peripherals must remain
   independent of `clk_sys` changes).
10. Optionally stop ROSC if power saving matters and XOSC is verified stable.

After init, `clock_get_hz(clock_handle)` returns the recorded frequency.

> **Note:** `clock_get_hz` returns the value cached when `clock_configure`
> was called. If the configured `src_freq` was wrong, this number is wrong.
> Use the frequency counter to verify in bring-up tests.

## See Also

- [`01_overview.md`](01_overview.md) — Architecture and switching rules.
- [`03_registers_a.md`](03_registers_a.md), [`04_registers_b.md`](04_registers_b.md) — Register bit fields.
