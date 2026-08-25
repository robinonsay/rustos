# Clocks (Chapter 8)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Clock architecture, sources, generators |
| [`02_programming.md`](02_programming.md) | Programmer's model, init sequence, frequency counter, resus |
| [`03_registers_a.md`](03_registers_a.md) | CLOCKS register list and per-generator CTRL/DIV/SELECTED |
| [`04_registers_b.md`](04_registers_b.md) | Frequency counter, sleep enables, interrupt registers |

## Base Address

| Block | Symbol | Address |
|-------|--------|---------|
| Clocks | `CLOCKS_BASE` | `0x40010000` |

## FSW Driver Notes

The FT1 init sequence requires:

1. Bring up XOSC (12 MHz crystal) before clearing PLL bypass.
2. Configure the system PLL for 150 MHz (typical) — `clk_sys` driven from
   `CLKSRC_PLL_SYS`.
3. Configure `clk_peri` from the system PLL or from the USB PLL (48 MHz) so
   that UART/SPI/I2C clock ratios are stable when `clk_sys` is changed.
4. Leave `clk_usb` and `clk_adc` at 48 MHz (sourced from the USB PLL).
5. Avoid relying on the ROSC for any timing-sensitive path (NMEA decode,
   sensor sampling) — its PVT variance is too large for FT1 nav requirements.

## Cross-References

- See [`../gpio/index.md`](../gpio/index.md) for `CLOCK GPIN/GPOUT` pin functions.
- See [`../uart/index.md`](../uart/index.md), [`../i2c/index.md`](../i2c/index.md),
  [`../spi/index.md`](../spi/index.md) for peripheral-specific clock requirements.
