# SPI: Register Map

[Back to SPI index](index.md) | [Back to ICD index](../index.md)

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| SPI0 | `SPI0_BASE` | `0x40080000` |
| SPI1 | `SPI1_BASE` | `0x40088000` |

## 12.3.5. Register List

| Offset | Name | Description |
|--------|------|-------------|
| 0x000 | `SSPCR0` | Control register 0 (format, rate, data size) |
| 0x004 | `SSPCR1` | Control register 1 (master/slave, enable) |
| 0x008 | `SSPDR` | Data register (TX/RX FIFO) |
| 0x00c | `SSPSR` | Status register |
| 0x010 | `SSPCPSR` | Clock prescale register |
| 0x014 | `SSPIMSC` | Interrupt mask set/clear |
| 0x018 | `SSPRIS` | Raw interrupt status |
| 0x01c | `SSPMIS` | Masked interrupt status |
| 0x020 | `SSPICR` | Interrupt clear |
| 0x024 | `SSPDMACR` | DMA control |
| 0xfe0..0xfec | `SSPPERIPHID0..3` | RO PrimeCell ID |
| 0xff0..0xffc | `SSPPCELLID0..3` | RO PrimeCell ID |

## SSPCR0 — Control Register 0 (0x000)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15:8 | `SCR` | RW | 0x00 | Serial clock rate. SCK = `SSPCLK / (CPSDVSR × (1+SCR))` |
| 7 | `SPH` | RW | 0 | Clock phase (Motorola only): 0=sample first edge, 1=sample second edge |
| 6 | `SPO` | RW | 0 | Clock polarity (Motorola only): 0=idle low, 1=idle high |
| 5:4 | `FRF` | RW | 0 | Frame format: 00=Motorola, 01=TI SSI, 10=Microwire, 11=reserved |
| 3:0 | `DSS` | RW | 0 | Data size: 0011=4b ... 1111=16b. Values 0000-0010 reserved |

## SSPCR1 — Control Register 1 (0x004)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3 | `SOD` | RW | 0 | Slave-mode output disable. 0=drive TXD, 1=tristate |
| 2 | `MS`  | RW | 0 | Mode: 0=master, 1=slave (only writable when SSE=0) |
| 1 | `SSE` | RW | 0 | Synchronous serial port enable |
| 0 | `LBM` | RW | 0 | Loopback mode |

## SSPDR — Data Register (0x008)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 15:0 | `DATA` | RWF | Read = pop RX FIFO; write = push TX FIFO |

For `DSS<16` (most uses), data is right-justified. Top bits are ignored
on TX and read back as 0 on RX.

## SSPSR — Status Register (0x00c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 4 | `BSY` | RO | 0 | Currently transmitting/receiving or TX FIFO non-empty |
| 3 | `RFF` | RO | 0 | RX FIFO full |
| 2 | `RNE` | RO | 0 | RX FIFO not empty |
| 1 | `TNF` | RO | 1 | TX FIFO not full |
| 0 | `TFE` | RO | 1 | TX FIFO empty |

## SSPCPSR — Clock Prescale (0x010)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7:0 | `CPSDVSR` | RW | 0x00 | Even divisor 2-254. LSB always reads 0 |

## SSPIMSC — Interrupt Mask Set/Clear (0x014)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3 | `TXIM` | RW | 0 | Unmask TX FIFO interrupt |
| 2 | `RXIM` | RW | 0 | Unmask RX FIFO interrupt |
| 1 | `RTIM` | RW | 0 | Unmask receive timeout interrupt |
| 0 | `RORIM` | RW | 0 | Unmask receive overrun interrupt |

## SSPRIS — Raw Interrupt Status (0x018)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3 | `TXRIS` | RO | 1 | TX FIFO half-empty (raw) |
| 2 | `RXRIS` | RO | 0 | RX FIFO half-full (raw) |
| 1 | `RTRIS` | RO | 0 | Receive timeout (raw) |
| 0 | `RORRIS` | RO | 0 | Receive overrun (raw) |

> Note: `TXRIS` reset value is 1 because the TX FIFO is empty at reset.

## SSPMIS — Masked Interrupt Status (0x01c)

Same layout as `SSPRIS`, but each bit is RO and represents the post-mask
state. Reset all 0.

## SSPICR — Interrupt Clear (0x020)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 1 | `RTIC` | WC | Clear receive-timeout latch |
| 0 | `RORIC` | WC | Clear receive-overrun latch |

> TX and RX FIFO interrupts have no clear bit — they track FIFO state.

## SSPDMACR — DMA Control (0x024)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 1 | `TXDMAE` | RW | 0 | Enable TX DMA request line |
| 0 | `RXDMAE` | RW | 0 | Enable RX DMA request line |

## Identification Registers

### SSPPERIPHID0..3 (0xfe0..0xfec)

| Register | Reset | Field |
|----------|-------|-------|
| `SSPPERIPHID0` | 0x22 | PartNumber0 |
| `SSPPERIPHID1` | 0x10 | Designer0=0x1, PartNumber1=0x0 |
| `SSPPERIPHID2` | 0x34 | Revision=0x3, Designer1=0x4 |
| `SSPPERIPHID3` | 0x00 | Configuration |

### SSPPCELLID0..3 (0xff0..0xffc)

| Register | Reset |
|----------|-------|
| `SSPPCELLID0` | 0x0d |
| `SSPPCELLID1` | 0xf0 |
| `SSPPCELLID2` | 0x05 |
| `SSPPCELLID3` | 0xb1 |

These IDs are useful as a sanity check at driver bring-up:

```c
assert(ssp->periphid0 == 0x22);
assert(ssp->pcellid0  == 0x0d);
```

## See Also

- [`02_operation.md`](02_operation.md) — Init flow using these registers.
- [`03_interrupts_dma.md`](03_interrupts_dma.md) — Interrupt + DMA usage.
