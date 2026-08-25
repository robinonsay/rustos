# I2C: Timing & Clock Configuration

[Back to I2C index](index.md) | [Back to ICD index](../index.md)

## 12.2.11. Spike Suppression

A digital filter rejects glitches on SCL/SDA. The filter passes a level
change only after the input has remained stable for `IC_FS_SPKLEN` cycles
of `ic_clk`. The minimum value is 1 (so the minimum spike length depends
on the `ic_clk` period — at 10 MHz `ic_clk`, minimum spike = 100 ns).

For Fast/FM+ modes the I2C spec requires up to 50 ns spike suppression;
this is feasible at `ic_clk ≥ 20 MHz`.

The `IC_FS_SPKLEN` register defaults to a value derived from a 100 ns
`ic_clk` period — it must be updated for the actual `clk_sys` in use.

## 12.2.14. IC_CLK and *CNT Registers

Programming `IC_*_SCL_HCNT` and `IC_*_SCL_LCNT` is required before any I2C
bus transaction. The four count registers are:

| Register | Used by |
|----------|---------|
| `IC_SS_SCL_HCNT` | Standard mode (≤100 kb/s) |
| `IC_SS_SCL_LCNT` | Standard mode |
| `IC_FS_SCL_HCNT` | Fast / Fast Mode Plus |
| `IC_FS_SCL_LCNT` | Fast / Fast Mode Plus |

> When operating only as a slave, the `*CNT` registers are not used and
> can be left at default.

### Derivation of I2C Timing Parameters

| Parameter | Symbol | SS | FS / FM+ |
|-----------|--------|----|----------|
| LOW period of SCL | tLOW | `IC_SS_SCL_LCNT` | `IC_FS_SCL_LCNT` |
| HIGH period of SCL | tHIGH | `IC_SS_SCL_HCNT` | `IC_FS_SCL_HCNT` |
| Setup for repeated START | tSU;STA | `IC_SS_SCL_LCNT` | `IC_FS_SCL_HCNT` |
| Hold for repeated START | tHD;STA | `IC_SS_SCL_HCNT` | `IC_FS_SCL_HCNT` |
| Setup for STOP | tSU;STO | `IC_SS_SCL_HCNT` | `IC_FS_SCL_HCNT` |
| Bus free between STOP/START | tBUF | `IC_SS_SCL_LCNT` | `IC_FS_SCL_LCNT` |
| Spike length | tSP | `IC_FS_SPKLEN` | `IC_FS_SPKLEN` |
| Data hold | tHD;DAT | `IC_SDA_HOLD` | `IC_SDA_HOLD` |
| Data setup | tSU;DAT | `IC_SDA_SETUP` | `IC_SDA_SETUP` |

### Minimum HCNT / LCNT Values (master mode)

```
LCNT > IC_FS_SPKLEN + 7
HCNT > IC_FS_SPKLEN + 5
```

Internal logic adds:

```
SCL_high_time = (HCNT + IC_FS_SPKLEN + 7) × t_ic_clk + SCL_fall_time
SCL_low_time  = (LCNT + 1) × t_ic_clk - SCL_fall_time + SCL_rise_time
```

Equivalently, for a desired SCL frequency:

```
IC_HCNT = ROUNDUP(MIN_SCL_HIGHtime × ic_clk_freq) − (IC_FS_SPKLEN + 7)
IC_LCNT = ROUNDUP(MIN_SCL_LOWtime  × ic_clk_freq) − 1
```

### Minimum SCL Periods (per I2C spec)

| Mode | tHIGH min | tLOW min |
|------|-----------|----------|
| 100 kb/s (SS) | 4000 ns | 4700 ns |
| 400 kb/s (FS) | 600 ns  | 1300 ns |
| 1000 kb/s (FM+) | 260 ns | 500 ns |

### Minimum `ic_clk` Frequency

| Mode | min `ic_clk` | min SPKLEN | LCNT (program) | HCNT (program) |
|------|--------------|------------|----------------|----------------|
| SS   | 2.7 MHz      | 1          | 12 (LCNT 13) | 6 (HCNT 14) |
| FS   | 12.0 MHz     | 1          | 15 (LCNT 16) | 6 (HCNT 14) |
| FM+  | 32 MHz       | 2          | 15 (LCNT 16) | 7 (HCNT 16) |

## Worked Examples

### 100 kb/s at `clk_sys = 125 MHz`

```
HCNT = ROUNDUP(4000ns × 125 MHz) − (SPKLEN + 7)
     = 500 − 8 = 492     # for SPKLEN=1
LCNT = ROUNDUP(4700ns × 125 MHz) − 1
     = 588 − 1 = 587
```

(Round up to nearest integer; tweak after measuring real SCL waveform.)

### 400 kb/s at `clk_sys = 125 MHz`

```
HCNT = ROUNDUP(600ns × 125 MHz) − 8 = 75 − 8 = 67
LCNT = ROUNDUP(1300ns × 125 MHz) − 1 = 163 − 1 = 162
```

The pico-sdk uses these forms in `i2c_set_baudrate`:

```c
period = (clk_sys + bauds/2) / bauds;
lcnt   = period * 3 / 5;        // 60% of period for LCNT
hcnt   = period - lcnt;
```

## 12.2.15. DMA Watermark Levels

The DMA Controller sees only single transfer requests for I2C; the
`IC_DMA_TDLR` and `IC_DMA_RDLR` watermark registers can be left at their
reset value (0) because I2C bandwidth is low relative to system bandwidth.

## 12.2.16. Interrupt Register Behaviour

| Interrupt | Set by HW / Cleared by SW | Set+Cleared by HW |
|-----------|---------------------------|-------------------|
| `RESTART_DET`, `GEN_CALL`, `START_DET`, `STOP_DET`, `ACTIVITY`, `RX_DONE`, `TX_ABRT`, `RD_REQ`, `TX_OVER`, `RX_OVER`, `RX_UNDER` | Yes | — |
| `TX_EMPTY`, `RX_FULL` | — | Yes |

Hardware-set/SW-cleared interrupts have a corresponding `IC_CLR_*`
register. Hardware-managed (`TX_EMPTY`, `RX_FULL`) follow the FIFO state
automatically and have no separate clear register.

## See Also

- [`02_modes.md`](02_modes.md) — Initialisation context for these timings.
- [`05_registers_a.md`](05_registers_a.md) — `IC_*_SCL_HCNT/_LCNT`,
  `IC_FS_SPKLEN`, `IC_SDA_HOLD`, `IC_SDA_SETUP` register details.
