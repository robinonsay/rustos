# GPIO (Chapter 9)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | GPIO architecture, banks, reset state, function-select tables |
| [`02_pads.md`](02_pads.md) | Pad electrical controls, isolation latches, SIO, pad register details |
| [`03_interrupts.md`](03_interrupts.md) | GPIO interrupts, summary registers, programming examples |
| [`04_registers.md`](04_registers.md) | IO_BANK0 + PADS_BANK0 register maps |

## Base Addresses

| Block | Symbol | Address |
|-------|--------|---------|
| User IO bank registers | `IO_BANK0_BASE` | `0x40028000` |
| User pad bank registers | `PADS_BANK0_BASE` | `0x40038000` |
| QSPI IO bank registers | `IO_QSPI_BASE` | `0x40030000` |
| QSPI pad bank registers | `PADS_QSPI_BASE` | `0x40040000` |
| SIO (single-cycle IO) | `SIO_BASE` | `0xd0000000` |

## FT1 Driver Notes

- The FSW uses **Bank 0** for all FT1 peripheral pin muxing (UART, I2C, SPI,
  optional clock GPIN/GPOUT). Bank 1 (QSPI) is used by the boot flash and is
  not touched by application code; QSPI register details are intentionally
  trimmed in this ICD.
- Key fact: pads come out of reset with input disabled (`IE=0`) and isolation
  latched (`ISO=1`). Software **must** set `IE=1` and clear `ISO=0` before
  using the pad. The SDK helper `gpio_set_function()` does both automatically.
- I2C pads must be configured with pull-up enabled, slew-rate limited, and
  Schmitt trigger enabled. External pull-ups are still required on the bus.
- For SPI used with an SD card on Bank 0, refer to the Bank 0 function table
  to choose pins (SPI0/SPI1, SCK/RX/TX/CSn).

## Cross-References

- Pin muxing tables for UART/I2C/SPI peripherals: see [`01_overview.md`](01_overview.md).
- For interrupt vector wiring see [`03_interrupts.md`](03_interrupts.md).
