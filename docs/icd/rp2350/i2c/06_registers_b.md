# I2C: Register Map (Part B — Clears, Status, IDs)

[Back to I2C index](index.md) | [Back to ICD index](../index.md)

## Register List (offsets 0x44–0xfc)

| Offset | Name | Description |
|--------|------|-------------|
| 0x44 | `IC_CLR_RX_UNDER` | Clear RX_UNDER (RO read-clears) |
| 0x48 | `IC_CLR_RX_OVER` | Clear RX_OVER |
| 0x4c | `IC_CLR_TX_OVER` | Clear TX_OVER |
| 0x50 | `IC_CLR_RD_REQ` | Clear RD_REQ |
| 0x54 | `IC_CLR_TX_ABRT` | Clear TX_ABRT and release TX FIFO |
| 0x58 | `IC_CLR_RX_DONE` | Clear RX_DONE |
| 0x5c | `IC_CLR_ACTIVITY` | Clear ACTIVITY |
| 0x60 | `IC_CLR_STOP_DET` | Clear STOP_DET |
| 0x64 | `IC_CLR_START_DET` | Clear START_DET |
| 0x68 | `IC_CLR_GEN_CALL` | Clear GEN_CALL |
| 0x6c | `IC_ENABLE` | Enable + abort + control |
| 0x70 | `IC_STATUS` | Status |
| 0x74 | `IC_TXFLR` | TX FIFO level |
| 0x78 | `IC_RXFLR` | RX FIFO level |
| 0x7c | `IC_SDA_HOLD` | SDA hold time |
| 0x80 | `IC_TX_ABRT_SOURCE` | Source of last TX_ABRT |
| 0x84 | `IC_SLV_DATA_NACK_ONLY` | Slave NACK control |
| 0x88 | `IC_DMA_CR` | DMA enable |
| 0x8c | `IC_DMA_TDLR` | TX DMA level |
| 0x90 | `IC_DMA_RDLR` | RX DMA level |
| 0x94 | `IC_SDA_SETUP` | SDA setup time |
| 0x98 | `IC_ACK_GENERAL_CALL` | Slave ACK general call |
| 0x9c | `IC_ENABLE_STATUS` | Live state of enable |
| 0xa0 | `IC_FS_SPKLEN` | FS/FM+ spike length |
| 0xa8 | `IC_CLR_RESTART_DET` | Clear RESTART_DET |
| 0xf4 | `IC_COMP_PARAM_1` | Component parameters (RO) |
| 0xf8 | `IC_COMP_VERSION` | Component version |
| 0xfc | `IC_COMP_TYPE` | Component type ID |

## IC_CLR_* (0x44 .. 0x68, 0xa8)

Each `IC_CLR_*` register has a single bit at [0]; read clears the
corresponding interrupt latch. Reset values are 0.

## IC_ENABLE (0x6c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 2 | `TX_CMD_BLOCK` | RW | 0 | Block master commands until released (advanced flow control) |
| 1 | `ABORT` | RW | 0 | Issue STOP+flush; auto-clears when complete |
| 0 | `ENABLE` | RW | 0 | Enable controller |

## IC_STATUS (0x70)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 6 | `SLV_ACTIVITY` | RO | Slave engaged in transfer |
| 5 | `MST_ACTIVITY` | RO | Master engaged in transfer |
| 4 | `RFF` | RO | RX FIFO full |
| 3 | `RFNE` | RO | RX FIFO not empty |
| 2 | `TFE` | RO | TX FIFO empty |
| 1 | `TFNF` | RO | TX FIFO not full |
| 0 | `ACTIVITY` | RO | Bus activity (master OR slave) |

## IC_TXFLR (0x74)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 4:0 | `TXFLR` | RO | Number of bytes currently in the TX FIFO (0..16) |

## IC_RXFLR (0x78)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 4:0 | `RXFLR` | RO | Number of bytes currently in the RX FIFO (0..16) |

## IC_SDA_HOLD (0x7c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 23:16 | `IC_SDA_RX_HOLD` | RW | 0 | SDA hold time when receiving |
| 15:0 | `IC_SDA_TX_HOLD` | RW | 1 | SDA hold time when transmitting (in `ic_clk` cycles) |

## IC_TX_ABRT_SOURCE (0x80)

Read-only. Bits 0..16 enumerate abort causes (see
[`04_interrupts_dma.md`](04_interrupts_dma.md) for the table). Bits 22:18
report `TX_FLUSH_CNT` — number of TX FIFO entries flushed when the abort
occurred.

## IC_SLV_DATA_NACK_ONLY (0x84)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 0 | `NACK` | RW | 0 | Slave: NACK next received data byte |

## IC_DMA_CR (0x88)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 1 | `TDMAE` | RW | 0 | Enable TX DMA request |
| 0 | `RDMAE` | RW | 0 | Enable RX DMA request |

## IC_DMA_TDLR (0x8c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3:0 | `DMATDL` | RW | 0 | TX DMA watermark (request asserted when TX FIFO ≤ this) |

## IC_DMA_RDLR (0x90)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 3:0 | `DMARDL` | RW | 0 | RX DMA watermark (request asserted when RX FIFO ≥ DMARDL+1) |

> Both watermarks may be left at reset for typical use.

## IC_SDA_SETUP (0x94)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7:0 | `SDA_SETUP` | RW | 0x64 | SDA setup time, in `ic_clk` cycles |

## IC_ACK_GENERAL_CALL (0x98)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 0 | `ACK_GEN_CALL` | RW | 1 | Slave: ACK general call |

## IC_ENABLE_STATUS (0x9c)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 2 | `SLV_RX_DATA_LOST` | RO | Slave: data lost during disable |
| 1 | `SLV_DISABLED_WHILE_BUSY` | RO | Slave: disabled while addressed |
| 0 | `IC_EN` | RO | Live enable state — poll this during disable |

## IC_FS_SPKLEN (0xa0)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7:0 | `IC_FS_SPKLEN` | RW | 7 | FS/FM+ spike-suppression length, in `ic_clk` cycles. **Update for actual `clk_sys`.** |

## IC_COMP_PARAM_1 (0xf4) — Read-only Identification

(Datasheet notes this register is not implemented and reads as 0 on
RP2350. Use the SDK's compile-time constants for FIFO depth, etc.)

| Bits | Field | Description |
|------|-------|-------------|
| 23:16 | `TX_BUFFER_DEPTH` | TX FIFO depth = 16 |
| 15:8  | `RX_BUFFER_DEPTH` | RX FIFO depth = 16 |
| 3:2   | `MAX_SPEED_MODE` | Fast mode |
| 1:0   | `APB_DATA_WIDTH` | 32 bits |

## IC_COMP_VERSION (0xf8)

| Bits | Field | Type | Reset |
|------|-------|------|-------|
| 31:0 | `IC_COMP_VERSION` | RO | 0x3230312a |

## IC_COMP_TYPE (0xfc)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:0 | `IC_COMP_TYPE` | RO | 0x44570140 | DesignWare component type — ASCII "DW" + 0x0140 |

These two registers are useful for runtime sanity-checks (e.g.,
"is this controller present and accessible?").

## See Also

- [`05_registers_a.md`](05_registers_a.md) — Control & data registers.
- [`02_modes.md`](02_modes.md) — Disable / abort flows using
  `IC_ENABLE_STATUS` and `IC_ENABLE.ABORT`.
