# I2C: Register Map (Part A — Control & Data)

[Back to I2C index](index.md) | [Back to ICD index](../index.md)

## Base Addresses

| Instance | Symbol | Address |
|----------|--------|---------|
| I2C0 | `I2C0_BASE` | `0x40090000` |
| I2C1 | `I2C1_BASE` | `0x40098000` |

## 12.2.17. Register List (offsets 0x00–0x40)

| Offset | Name | Description |
|--------|------|-------------|
| 0x00 | `IC_CON` | Control |
| 0x04 | `IC_TAR` | Target address (master) |
| 0x08 | `IC_SAR` | Slave address |
| 0x10 | `IC_DATA_CMD` | TX/RX data + command |
| 0x14 | `IC_SS_SCL_HCNT` | SS SCL high count |
| 0x18 | `IC_SS_SCL_LCNT` | SS SCL low count |
| 0x1c | `IC_FS_SCL_HCNT` | FS/FM+ SCL high count |
| 0x20 | `IC_FS_SCL_LCNT` | FS/FM+ SCL low count |
| 0x2c | `IC_INTR_STAT` | Masked interrupt status (RO) |
| 0x30 | `IC_INTR_MASK` | Interrupt mask (RW) |
| 0x34 | `IC_RAW_INTR_STAT` | Raw interrupt status (RO) |
| 0x38 | `IC_RX_TL` | RX FIFO threshold |
| 0x3c | `IC_TX_TL` | TX FIFO threshold |
| 0x40 | `IC_CLR_INTR` | Read to clear all clearable interrupts |

(Continued in [`06_registers_b.md`](06_registers_b.md): individual clear
registers, status, FIFO levels, ID registers.)

## IC_CON — Control (0x00)

> Many fields can be written **only** while the controller is disabled
> (`IC_ENABLE.ENABLE = 0`).

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 10 | `STOP_DET_IF_MASTER_ACTIVE` | RO | 0 | Master always issues STOP_DET |
| 9 | `RX_FIFO_FULL_HLD_CTRL` | RW | 0 | 1 = hold bus when RX FIFO full |
| 8 | `TX_EMPTY_CTRL` | RW | 0 | 1 = controlled `TX_EMPTY` (asserts only when both shifter and FIFO empty) |
| 7 | `STOP_DET_IFADDRESSED` | RW | 0 | Slave: STOP_DET only on addressed transfers |
| 6 | `IC_SLAVE_DISABLE` | RW | 1 | 1 = slave disabled |
| 5 | `IC_RESTART_EN` | RW | 1 | 1 = master may issue RESTART |
| 4 | `IC_10BITADDR_MASTER` | RW | 0 | 1 = 10-bit master addressing |
| 3 | `IC_10BITADDR_SLAVE` | RW | 0 | 1 = 10-bit slave addressing |
| 2:1 | `SPEED` | RW | 2 | 1=Standard, 2=Fast/FM+, 3=High-speed (unsupported) |
| 0 | `MASTER_MODE` | RW | 1 | 1 = master mode enabled |

For FT1 master at FM: `IC_CON = 0b01100011 = 0x63`
(`IC_SLAVE_DISABLE | IC_RESTART_EN | SPEED=Fast | MASTER_MODE`).

## IC_TAR — Target Address (0x04)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 11 | `SPECIAL` | RW | 0 | 1 = perform special command |
| 10 | `GC_OR_START` | RW | 0 | When SPECIAL=1: 0=general call, 1=START byte |
| 9:0 | `IC_TAR` | RW | 0x055 | Target slave address |

> Most fields require disable to write.

## IC_SAR — Slave Address (0x08)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 9:0 | `IC_SAR` | RW | 0x055 | Slave address (this controller's own) |

## IC_DATA_CMD — Data + Command (0x10)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 11 | `RESTART` | WO | 1 = issue RESTART before this transfer |
| 10 | `STOP` | WO | 1 = issue STOP after this transfer |
| 9 | `FIRST_DATA_BYTE` | RO | (RX) 1 if this is the first byte of a transfer |
| 8 | `CMD` | RW | (Master) 0=write, 1=read |
| 7:0 | `DAT` | RWF | Data byte (write); read returns RX byte |

Write: queues a command + optional data. Read: pops a byte from the RX
FIFO.

## IC_SS_SCL_HCNT (0x14), IC_SS_SCL_LCNT (0x18)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15:0 | `IC_SS_SCL_xCNT` | RW | per spec | Standard-mode SCL high/low count |

## IC_FS_SCL_HCNT (0x1c), IC_FS_SCL_LCNT (0x20)

Same shape as SS variants but for Fast / FM+.

## IC_RX_TL — RX FIFO Threshold (0x38)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7:0 | `RX_TL` | RW | 0 | RX_FULL fires when RX FIFO depth ≥ RX_TL+1 (0 → 1) |

## IC_TX_TL — TX FIFO Threshold (0x3c)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 7:0 | `TX_TL` | RW | 0 | TX_EMPTY fires when TX FIFO ≤ TX_TL |

## IC_INTR_STAT (0x2c) / IC_INTR_MASK (0x30) / IC_RAW_INTR_STAT (0x34)

Same bit layout as listed in [`04_interrupts_dma.md`](04_interrupts_dma.md):

| Bit | Field |
|-----|-------|
| 12 | `R_RESTART_DET` / `M_RESTART_DET` |
| 11 | `R_GEN_CALL` / `M_GEN_CALL` |
| 10 | `R_START_DET` / `M_START_DET` |
| 9 | `R_STOP_DET` / `M_STOP_DET` |
| 8 | `R_ACTIVITY` / `M_ACTIVITY` |
| 7 | `R_RX_DONE` / `M_RX_DONE` |
| 6 | `R_TX_ABRT` / `M_TX_ABRT` |
| 5 | `R_RD_REQ` / `M_RD_REQ` |
| 4 | `R_TX_EMPTY` / `M_TX_EMPTY` |
| 3 | `R_TX_OVER` / `M_TX_OVER` |
| 2 | `R_RX_FULL` / `M_RX_FULL` |
| 1 | `R_RX_OVER` / `M_RX_OVER` |
| 0 | `R_RX_UNDER` / `M_RX_UNDER` |

`IC_INTR_STAT` returns the masked status; `IC_RAW_INTR_STAT` returns
unmasked. Both RO.

`IC_INTR_MASK` is RW; reset values are vendor defaults — set explicitly
during init.

## IC_CLR_INTR (0x40)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 0 | `CLR_INTR` | RO | Reading clears: RX_UNDER, RX_OVER, TX_OVER, RD_REQ, TX_ABRT, RX_DONE, ACTIVITY, STOP_DET, START_DET, GEN_CALL |

`TX_EMPTY` and `RX_FULL` are not cleared by this register (they track FIFO
state).

## See Also

- [`06_registers_b.md`](06_registers_b.md) — Per-source clear registers,
  status, FIFO levels, abort source, ID registers.
