# SPI: Operation Modes & Initialisation

[Back to SPI index](index.md) | [Back to ICD index](../index.md)

## 12.3.4.1-3. Reset and Configuration

After reset:

- Peripheral logic is disabled (`SSPCR1.SSE = 0`).
- All FIFOs empty.
- Master mode default (`SSPCR1.MS = 0`).

Configure these before enabling:

1. `SSPCR0` — frame format, data size, clock polarity/phase, SCR.
2. `SSPCR1` — master/slave mode, slave-output disable.
3. `SSPCPSR` — clock prescale divisor.
4. `SSPDMACR` — DMA enables (if used).
5. `SSPIMSC` — interrupt enables (if used).
6. `SSPCR1.SSE = 1` — enable.

You may pre-fill the TX FIFO (up to 8 × 16-bit values) before enabling.
Once enabled and TX FIFO non-empty, transmission begins immediately.

## 12.3.4.5. Programming SSPCR0

| Field | Bits | Purpose |
|-------|------|---------|
| `SCR` | 15:8 | Serial clock rate (0..255). Final SCK = `SSPCLK / (CPSDVSR × (1+SCR))` |
| `SPH` | 7 | Phase (Motorola only) |
| `SPO` | 6 | Polarity (Motorola only) |
| `FRF` | 5:4 | Frame format (00=Motorola, 01=TI, 10=Microwire) |
| `DSS` | 3:0 | Data size: 0011..1111 = 4..16 bits |

## 12.3.4.6. Programming SSPCR1

| Field | Bits | Purpose |
|-------|------|---------|
| `SOD` | 3 | Slave-mode output disable (0=drive TX, 1=tristate) |
| `MS`  | 2 | 0=master, 1=slave (only writable when SSE=0) |
| `SSE` | 1 | Enable peripheral |
| `LBM` | 0 | Loopback mode for self-test |

## Master Init Recipe (FT1 SD Card Example)

Goal: Motorola SPI Mode 0, 8-bit, 1 MHz initial bit rate,
`SSPCLK = clk_peri = 125 MHz`.

```
1. Reset peripheral via RESETS block, then de-assert reset.
2. Wait for clk_peri stable (XOSC + PLL_USB or PLL_SYS).

3. Compute prescale + SCR for bit rate:
       target = 1 MHz
       prescale even, ≥ 2; pick smallest CPSDVSR such that
       125 MHz / (CPSDVSR × 256) ≤ target.
       For 1 MHz target: CPSDVSR=4, SCR=30 → 125 MHz/(4×31) ≈ 1.008 MHz.

4. SSPCPSR = 4
   SSPCR0  = (SCR=30) << 8
           | (SPH=0)  << 7
           | (SPO=0)  << 6
           | (FRF=0)  << 4
           | (DSS=7)        // 8-bit data
           = 0x1E07

5. SSPCR1 = 0    (master, no loopback, not enabled yet)

6. SSPDMACR = 0  (no DMA initially)
   SSPIMSC  = 0  (no interrupts initially)

7. SSPCR1 |= SSE_BIT   (enable)

8. gpio_set_function() on SCK, TX, RX pins (function 0x01 = SPI).
   Drive CSn manually via SIO on a separate GPIO.

9. After SD init complete, increase bit rate:
   - Disable peripheral first (clear SSE)
   - New SCR/CPSDVSR
   - Re-enable
```

## 12.3.4.7-13. Frame Formats (selection summary)

| Use case | FRF | SPO | SPH | DSS |
|----------|-----|-----|-----|-----|
| SD card | Motorola | 0 | 0 | 7 (8-bit) |
| Sensor with Mode 3 | Motorola | 1 | 1 | depends |
| TI SSI device | TI | n/a | n/a | 8-15 |
| Microwire EEPROM | Microwire | n/a | n/a | depends |

For Motorola Mode 0, the master drives data on the falling edge of SCK
and samples on the rising edge. SD cards expect this.

## Transmit / Receive Operation

Per byte:

1. Wait for `SSPSR.TNF = 1` (TX FIFO not full).
2. Write the byte to `SSPDR` (low byte; high bits unused for `DSS<16`).
3. Wait for `SSPSR.RNE = 1` (RX FIFO not empty).
4. Read `SSPDR` to get the simultaneously-clocked-in byte.

Because SPI is full-duplex, every TX byte produces one RX byte. For
write-only drivers, discard the RX byte. For read-only, write a dummy
(0x00 or 0xFF) for each byte you want to read.

### `SSPSR` Flags

| Bit | Field | Meaning |
|-----|-------|---------|
| 4 | `BSY` | SSP currently transmitting/receiving |
| 3 | `RFF` | RX FIFO full |
| 2 | `RNE` | RX FIFO not empty |
| 1 | `TNF` | TX FIFO not full |
| 0 | `TFE` | TX FIFO empty |

### Right-Justification

For `DSS < 16`, the data must be **right-justified** when written to
`SSPDR`. Unused upper bits are ignored on TX and zero on RX.

## Disabling

To re-program any "configured at init" field (master/slave mode), clear
`SSPCR1.SSE` first. Wait for `SSPSR.BSY = 0` to ensure all bits have
actually flushed.

## See Also

- [`03_interrupts_dma.md`](03_interrupts_dma.md) — Interrupt & DMA flows.
- [`04_registers.md`](04_registers.md) — Register details.
