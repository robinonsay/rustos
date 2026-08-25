# SPI: Overview & Functional Description

[Back to SPI index](index.md) | [Back to ICD index](../index.md)

## 12.3.2. Block Description

PrimeCell PL022 (r1p4) SSP. The peripheral performs serial-to-parallel
and parallel-to-serial conversion. Software accesses through the AMBA
APB interface (`PCLK = clk_sys`). The serial logic is clocked by
`SSPCLK = clk_peri`.

### Pins

| PL022 signal | GPIO mux name |
|--------------|---------------|
| `SSPCLKOUT` / `SSPCLKIN` | `spiN_sck` |
| `SSPTXD`    | `spiN_tx` |
| `SSPRXD`    | `spiN_rx` |
| `SSPFSSOUT` / `SSPFSSIN` | `spiN_csn` |

`SSPTXD` data direction is always output (regardless of master/slave —
the peripheral controls tristating internally via `nSSPOE`).
`SSPRXD` data direction is always input.

In master mode, `SSPCLKOUT` and `SSPFSSOUT` are outputs; in slave mode
they are inputs.

## 12.3.1. Changes from RP2040

The TX output enable is now controlled internally by `nSSPOE`. Software
no longer needs to manage TX tristating in slave mode — the peripheral
auto-tristates when deselected.

## 12.3.3. Internal Functional Blocks

| Block | Function |
|-------|----------|
| AMBA APB interface | Decodes register / FIFO accesses |
| Register block | Stores written/read data |
| Clock prescaler | `SSPCPSR` and `SSPCR0.SCR` divide `SSPCLK` to produce `SSPCLKOUT` (master) |
| TX FIFO | 8 × 16-bit |
| RX FIFO | 8 × 16-bit |
| TX/RX logic | Parallel-to-serial / serial-to-parallel |
| Interrupt logic | 4 individual maskable interrupts (combined for NVIC) |
| DMA interface | TX / RX request lines + clear |
| Synchronizers | PCLK ↔ SSPCLK domain crossing |

## 12.3.4.4. Clock Ratios

```
F_SSPCLK ≤ F_PCLK     (i.e., clk_peri ≤ clk_sys)
```

Master mode: max bit rate ≈ `SSPCLK / 2`.
Slave mode: max bit rate ≈ `SSPCLK / 12` (synchroniser delay).

Examples (at `clk_peri = 150 MHz`):

| Mode | Max bit rate |
|------|--------------|
| Master | 75 Mb/s (CPSDVSR=2, SCR=0) |
| Slave  | 12.5 Mb/s |

## 12.3.4.6.1. Bit-Rate Generation

```
SSPCLKOUT = SSPCLK / (CPSDVSR × (1 + SCR))
```

- `CPSDVSR` (`SSPCPSR`): even, 2-254 (LSB always reads 0).
- `SCR` (`SSPCR0[15:8]`): 0-255.

Example (master, `SSPCLK=125 MHz`, `CPSDVSR=2`):
- Min bit rate: 125 MHz / (2 × 256) ≈ 244 kHz.
- Max bit rate: 125 MHz / (2 × 1) = 62.5 MHz.

## 12.3.4.7. Frame Formats

Selected via `SSPCR0.FRF`:

| FRF | Format |
|-----|--------|
| 0b00 | Motorola SPI |
| 0b01 | Texas Instruments SSI |
| 0b10 | National Microwire |
| 0b11 | Reserved |

Data size 4-16 bits via `SSPCR0.DSS`:

| DSS | Data bits |
|-----|-----------|
| 0011 | 4 |
| 0100 | 5 |
| ... | ... |
| 0111 | 8 |
| ... | ... |
| 1111 | 16 |

(0000-0010 reserved.)

### Motorola SPI Modes

Programmed via `SSPCR0.SPO` (clock polarity) and `SSPCR0.SPH` (clock
phase):

| SPO | SPH | Mode | Idle SCK | Sampling Edge |
|-----|-----|------|----------|---------------|
| 0 | 0 | Mode 0 | low | rising |
| 0 | 1 | Mode 1 | low | falling |
| 1 | 0 | Mode 2 | high | falling |
| 1 | 1 | Mode 3 | high | rising |

> SD card requires Mode 0.

> See source PDF Figures 93-98 for waveform diagrams.

### Texas Instruments SSI

`SSPFSSOUT` is pulsed high for one SCK period at the start of each frame.
Both ends drive on rising edges, sample on falling edges. Idle: SCK low,
FSS low.

> See source PDF Figures 91-92.

### National Microwire

Half-duplex: master sends 8-bit control, slave responds with 4-16 bit
data. Total frame 13-25 bits. Used for some EEPROMs.

> See source PDF Figures 99-100. Not used in FT1.

## 12.3.4.5. Slave Mode SSPFSSIN Setup/Hold (informational)

When operating as a slave with a free-running SCK from the master,
`SSPFSSIN` must have ≥2 SSPCLK periods of setup and ≥1 SSPCLK period of
hold relative to the rising edge of SSPCLK. See source PDF Figure 101.
Not used in FT1.

## 12.3.4.15. Connection Examples (informational)

The PL022 instance must be configured at init as either master or slave
(no dynamic switch). Multi-slave configurations require external CSn
control (or use of the GPIO-driven CS approach for SD cards).

## See Also

- [`02_operation.md`](02_operation.md) — Init, bit-rate, register
  programming.
- [`03_interrupts_dma.md`](03_interrupts_dma.md) — Interrupts & DMA.
- [`04_registers.md`](04_registers.md) — Register details.
