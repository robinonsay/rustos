# SPI (Section 12.3)

[Back to RP2350 ICD index](../index.md)

## Chapter Contents

| File | Topic |
|------|-------|
| [`01_overview.md`](01_overview.md) | Functional description, FIFOs, frame formats |
| [`02_operation.md`](02_operation.md) | Master/slave init, bit-rate, frame format selection |
| [`03_interrupts_dma.md`](03_interrupts_dma.md) | Interrupt sources, DMA |
| [`04_registers.md`](04_registers.md) | Register map and bit fields |

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| SPI0 | `SPI0_BASE` | `0x40080000` |
| SPI1 | `SPI1_BASE` | `0x40088000` |

## Peripheral Identity

PrimeCell PL022 SSP (Synchronous Serial Port), revision r1p4. Two
identical instances. Distinct from the QSPI memory interface (QMI,
Section 12.14).

## Key Specifications

| Property | Value |
|----------|-------|
| Frame formats | Motorola SPI, Texas Instruments SSI, National Microwire |
| Master / slave | Either (set at init; not dynamic) |
| Data size | 4-16 bits |
| TX FIFO | 8 × 16-bit |
| RX FIFO | 8 × 16-bit |
| DMA | Single + burst (watermark = 4) |
| Reference clock | `clk_peri` (SSPCLK) |
| Bus clock | `clk_sys` (PCLK) |
| Constraint | F_SSPCLK ≤ F_PCLK |
| Min bit rate | F_SSPCLK / (254 × 256) |
| Max bit rate | F_SSPCLK / 2 (master) |
| Pin signals | SCK, TX (MOSI/MISO), RX, CSn |

## FT1 Driver Notes

- The SD card uses **SPI master** with Motorola SPI frame format,
  `SPO=0, SPH=0` (Mode 0), 8-bit data.
- Recommended SD bus rate: ≤25 MHz for SDHC compatibility, but during
  init drop to 100-400 kHz (SD spec init clock range).
- The PL022 nSSPOE auto-tristates the TX line when deselected in slave
  mode — safe for multi-slave shared lines.
- For SD card chip-select use a separate GPIO in SIO mode and toggle it
  manually around each transaction (PL022's CSn does not match SD card
  protocol's continuous-CS-during-multi-block-read).

## Cross-References

- Pin muxing: see [`../gpio/01_overview.md`](../gpio/01_overview.md) (F1=SPI).
- Pad config: see [`../gpio/02_pads.md`](../gpio/02_pads.md).
- Clock setup: see [`../clocks/02_programming.md`](../clocks/02_programming.md).
