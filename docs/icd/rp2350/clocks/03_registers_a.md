# Clocks: Register Map (Part A — Generators)

[Back to clocks index](index.md) | [Back to ICD index](../index.md)

## Base Address

`CLOCKS_BASE = 0x40010000`

## 8.1.6. Register List

| Offset | Name | Description |
|--------|------|-------------|
| 0x00 | `CLK_GPOUT0_CTRL` | clk_gpout0 control |
| 0x04 | `CLK_GPOUT0_DIV` | clk_gpout0 divisor |
| 0x08 | `CLK_GPOUT0_SELECTED` | one-hot, src currently selected |
| 0x0c | `CLK_GPOUT1_CTRL` | clk_gpout1 control |
| 0x10 | `CLK_GPOUT1_DIV` | |
| 0x14 | `CLK_GPOUT1_SELECTED` | |
| 0x18 | `CLK_GPOUT2_CTRL` | |
| 0x1c | `CLK_GPOUT2_DIV` | |
| 0x20 | `CLK_GPOUT2_SELECTED` | |
| 0x24 | `CLK_GPOUT3_CTRL` | |
| 0x28 | `CLK_GPOUT3_DIV` | |
| 0x2c | `CLK_GPOUT3_SELECTED` | |
| 0x30 | `CLK_REF_CTRL` | clk_ref control (glitchless) |
| 0x34 | `CLK_REF_DIV` | |
| 0x38 | `CLK_REF_SELECTED` | one-hot |
| 0x3c | `CLK_SYS_CTRL` | clk_sys control (glitchless) |
| 0x40 | `CLK_SYS_DIV` | |
| 0x44 | `CLK_SYS_SELECTED` | one-hot |
| 0x48 | `CLK_PERI_CTRL` | clk_peri control |
| 0x4c | `CLK_PERI_DIV` | |
| 0x50 | `CLK_PERI_SELECTED` | one-hot |
| 0x54 | `CLK_HSTX_CTRL` | clk_hstx control |
| 0x58 | `CLK_HSTX_DIV` | |
| 0x5c | `CLK_HSTX_SELECTED` | |
| 0x60 | `CLK_USB_CTRL` | clk_usb control |
| 0x64 | `CLK_USB_DIV` | |
| 0x68 | `CLK_USB_SELECTED` | |
| 0x6c | `CLK_ADC_CTRL` | clk_adc control |
| 0x70 | `CLK_ADC_DIV` | |
| 0x74 | `CLK_ADC_SELECTED` | |
| 0x78 | `DFTCLK_XOSC_CTRL` | DFT (debug) |
| 0x7c | `DFTCLK_ROSC_CTRL` | DFT (debug) |
| 0x80 | `DFTCLK_LPOSC_CTRL` | DFT (debug) |
| 0x84 | `CLK_SYS_RESUS_CTRL` | resus control |
| 0x88 | `CLK_SYS_RESUS_STATUS` | resus status |
| 0x8c..0xa8 | `FC0_*` | frequency counter (see part B) |
| 0xac..0xc0 | `WAKE_EN0/1`, `SLEEP_EN0/1`, `ENABLED0/1` | per-destination enables |
| 0xc4..0xd0 | `INTR`, `INTE`, `INTF`, `INTS` | resus interrupt |

## CLK_GPOUTn_CTRL (n = 0..3)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 28 | `ENABLED` | RO | 0 | Generator currently enabled |
| 20 | `NUDGE` | RW | 0 | Edge here shifts output phase by 1 input cycle |
| 17:16 | `PHASE` | RW | 0 | Delays enable signal by 0-3 input cycles (set before enable) |
| 12 | `DC50` | RW | 0 | Enable duty-cycle correction for odd divisors |
| 11 | `ENABLE` | RW | 0 | Start/stop generator cleanly |
| 10 | `KILL` | RW | 0 | Async kill (do not use in normal flow) |
| 8:5 | `AUXSRC` | RW | 0 | Aux source select (see enum below) |

`AUXSRC` enum (CLK_GPOUTn):

| Value | Source |
|-------|--------|
| 0x0 | `CLKSRC_PLL_SYS` |
| 0x1 | `CLKSRC_GPIN0` |
| 0x2 | `CLKSRC_GPIN1` |
| 0x3 | `CLKSRC_PLL_USB` |
| 0x4 | `CLKSRC_PLL_USB_PRIMARY_REF_OPCG` |
| 0x5 | `ROSC_CLKSRC` |
| 0x6 | `XOSC_CLKSRC` |
| 0x7 | `LPOSC_CLKSRC` |
| 0x8 | `CLK_SYS` |
| 0x9 | `CLK_USB` |
| 0xa | `CLK_ADC` |
| 0xb | `CLK_REF` |
| 0xc | `CLK_PERI` |
| 0xd | `CLK_HSTX` |
| 0xe | `OTP_CLK2FC` |

## CLK_GPOUTn_DIV

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:16 | `INT` | RW | 0x0001 | Integer divisor; 0 → max+1; on-the-fly OK |
| 15:0  | `FRAC` | RW | 0x0000 | Fractional divisor |

## CLK_GPOUTn_SELECTED

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 0 | `SELECTED` | RO | 0x1 | No glitchless mux on gpout; hardwired to 1 |

## CLK_REF_CTRL

`clk_ref` has both a glitchless mux (`SRC`) and an aux mux (`AUXSRC`).

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 8:5 | `AUXSRC` | RW | 0 | Aux source: 0=PLL_USB, 1=GPIN0, 2=GPIN1, 3=PLL_USB_PRIMARY_REF_OPCG |
| 1:0 | `SRC` | RW | 0 | Glitchless source: 0=ROSC_CLKSRC_PH, 1=CLKSRC_CLK_REF_AUX, 2=XOSC_CLKSRC, 3=LPOSC_CLKSRC |

## CLK_REF_DIV

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 17:16 | `INT` | RW | 0x1 | Integer divisor (limited width on REF) |

## CLK_REF_SELECTED

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 3:0 | `SELECTED` | RO | One-hot: bit `n` set when `SRC=n` is active |

## CLK_SYS_CTRL

`clk_sys` has both a glitchless mux and an aux mux. Most common runtime
configuration: `SRC=1` (AUX), `AUXSRC=PLL_SYS`.

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 8:5 | `AUXSRC` | RW | 0 | Aux source: 0=PLL_SYS, 1=PLL_USB, 2=ROSC_CLKSRC, 3=XOSC_CLKSRC, 4=GPIN0, 5=GPIN1 |
| 0 | `SRC` | RW | 0 | Glitchless source: 0=CLK_REF, 1=CLKSRC_CLK_SYS_AUX |

## CLK_SYS_DIV

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:16 | `INT` | RW | 0x0001 | Integer divisor |
| 15:0  | `FRAC` | RW | 0x0000 | Fractional divisor |

## CLK_SYS_SELECTED

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 1:0 | `SELECTED` | RO | One-hot for current `SRC` |

## CLK_PERI_CTRL / CLK_USB_CTRL / CLK_ADC_CTRL / CLK_HSTX_CTRL

These are aux-only generators (no glitchless mux). Same field layout as
`CLK_GPOUTn_CTRL` but with restricted `AUXSRC` enums:

| Generator | Allowed `AUXSRC` |
|-----------|------------------|
| `clk_peri` | 0=CLK_SYS, 1=PLL_SYS, 2=PLL_USB, 3=ROSC_CLKSRC, 4=XOSC_CLKSRC, 5=GPIN0, 6=GPIN1 |
| `clk_usb`  | 0=PLL_USB, 1=PLL_SYS, 2=ROSC_CLKSRC, 3=XOSC_CLKSRC, 4=GPIN0, 5=GPIN1 |
| `clk_adc`  | 0=PLL_USB, 1=PLL_SYS, 2=ROSC_CLKSRC, 3=XOSC_CLKSRC, 4=GPIN0, 5=GPIN1 |
| `clk_hstx` | 0=PLL_SYS, 1=PLL_USB, 2=ROSC_CLKSRC, 3=XOSC_CLKSRC, 4=GPIN0, 5=GPIN1 |

> See the source PDF for the authoritative enumeration of each generator.

`CLK_PERI_DIV`, `CLK_USB_DIV`, `CLK_ADC_DIV`, `CLK_HSTX_DIV`: same layout
as `CLK_GPOUTn_DIV`.

## CLK_SYS_RESUS_CTRL (offset 0x84)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 16 | `CLEAR` | WC | 0 | Write 1 to clear a resus event |
| 12 | `FRCE` | RW | 0 | Force a resus event (debug) |
| 8 | `ENABLE` | RW | 0 | Resus enable |
| 7:0 | `TIMEOUT` | RW | 0xff | Timeout in clk_ref ticks (0 disables) |

## CLK_SYS_RESUS_STATUS (offset 0x88)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 0 | `RESUSSED` | RO | clk_sys was resuscitated since last clear |

## See Also

- [`04_registers_b.md`](04_registers_b.md) — Frequency counter, enables, interrupts.
- [`02_programming.md`](02_programming.md) — Init sequences using these registers.
