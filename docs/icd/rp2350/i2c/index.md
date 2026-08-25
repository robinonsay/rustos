# I2C (Section 12.2)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Features, IP config, terminology |
| [`02_modes.md`](02_modes.md) | Master/slave operation, init, abort, disable |
| [`03_timing.md`](03_timing.md) | IC_CLK calculation, *CNT registers, spike suppression |
| [`04_interrupts_dma.md`](04_interrupts_dma.md) | Interrupt sources, DMA |
| [`05_registers_a.md`](05_registers_a.md) | Core control & data registers |
| [`06_registers_b.md`](06_registers_b.md) | Interrupt, FIFO, status, clear, ID registers |

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| I2C0 | `I2C0_BASE` | `0x40090000` |
| I2C1 | `I2C1_BASE` | `0x40098000` |

## Peripheral Identity

Synopsys DesignWare DW_apb_i2c v2.03a. Two identical instances. All
clocking is from `clk_sys` (`ic_clk` = `clk_sys`).

## Key Specifications

| Property | Value |
|----------|-------|
| Modes | Standard (≤100 kb/s), Fast (≤400 kb/s), Fast Mode Plus (≤1000 kb/s) |
| Addressing | 7-bit (default) or 10-bit (master only) |
| TX FIFO | 16 entries |
| RX FIFO | 16 entries |
| Default slave address | 0x055 |
| Master/slave | Either, but **not simultaneously** |
| DMA | Single transfers (low bandwidth) |
| Interrupts | Single combined output |
| Clock domain | `ic_clk` = `clk_sys` |

> **Modes not supported:** High-speed (3.4 Mb/s), Ultra-Fast (5 Mb/s).
> **Protocols not supported:** SMBus, PMBus.

## FT1 Driver Notes

- IMU and barometer (TBD) will both share an I2C bus, configured as
  **master** in **Fast Mode** (400 kb/s) by default.
- Both controllers must use master XOR slave (configurable) — never both.
- Pads must be configured with internal pull-up enabled, slew-rate slow,
  Schmitt trigger on. **External pull-ups are still required** on each line
  (typ. 4.7 kΩ at 3.3 V) — the internal pull-ups are not strong enough.
- For ≥32 MHz `ic_clk`, FM+ (1 Mb/s) is allowed. With the FT1 default
  `clk_sys = 125 MHz` or `150 MHz`, all three modes are achievable.
- The driver should follow the documented disable procedure
  (poll `IC_ENABLE_STATUS`) before reconfiguration.

## Cross-References

- Pin muxing: see [`../gpio/01_overview.md`](../gpio/01_overview.md) (F3=I2C).
- Pad config: see [`../gpio/02_pads.md`](../gpio/02_pads.md).
- `clk_sys` setup: see [`../clocks/02_programming.md`](../clocks/02_programming.md).
