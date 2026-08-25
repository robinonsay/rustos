# I2C: Interrupts & DMA

[Back to I2C index](index.md) | [Back to ICD index](../index.md)

## Interrupt Sources (`IC_RAW_INTR_STAT`)

| Bit | Name | Cause |
|-----|------|-------|
| 12 | `RESTART_DET` | Slave detected RESTART (slave-mode only) |
| 11 | `GEN_CALL` | General-call address received |
| 10 | `START_DET` | START or RESTART detected on bus |
| 9 | `STOP_DET` | STOP detected |
| 8 | `ACTIVITY` | Bus activity detected |
| 7 | `RX_DONE` | Slave-tx: master NACKed last byte (transfer ended) |
| 6 | `TX_ABRT` | Master-mode abort condition; check `IC_TX_ABRT_SOURCE` |
| 5 | `RD_REQ` | Slave-tx: master requests data (slave-mode) |
| 4 | `TX_EMPTY` | TX FIFO ≤ `IC_TX_TL` |
| 3 | `TX_OVER` | TX FIFO overflow on write |
| 2 | `RX_FULL` | RX FIFO ≥ `IC_RX_TL` |
| 1 | `RX_OVER` | RX FIFO overflow (data lost) |
| 0 | `RX_UNDER` | Software read from empty RX FIFO |

`IC_INTR_MASK` enables / disables individual sources.
`IC_INTR_STAT` shows masked status (what the CPU sees).
`IC_RAW_INTR_STAT` shows pre-mask state.

### Clearing Interrupts

| Source | Clear Register |
|--------|---------------|
| All     | `IC_CLR_INTR` (read-once clears all clearable bits) |
| `RX_UNDER` | `IC_CLR_RX_UNDER` |
| `RX_OVER`  | `IC_CLR_RX_OVER` |
| `TX_OVER`  | `IC_CLR_TX_OVER` |
| `RD_REQ`   | `IC_CLR_RD_REQ` |
| `TX_ABRT`  | `IC_CLR_TX_ABRT` |
| `RX_DONE`  | `IC_CLR_RX_DONE` |
| `ACTIVITY` | `IC_CLR_ACTIVITY` |
| `STOP_DET` | `IC_CLR_STOP_DET` |
| `START_DET`| `IC_CLR_START_DET` |
| `GEN_CALL` | `IC_CLR_GEN_CALL` |
| `RESTART_DET` | `IC_CLR_RESTART_DET` |

`TX_EMPTY` and `RX_FULL` have no separate clear register — they track FIFO
levels automatically.

### TX_ABRT Sources (`IC_TX_ABRT_SOURCE`)

Read this register after any `TX_ABRT` to determine cause. Notable bits
(non-exhaustive):

| Bit | Source |
|-----|--------|
| 0 | `ABRT_7B_ADDR_NOACK` — 7-bit address NACKed |
| 1 | `ABRT_10ADDR1_NOACK` — first byte of 10-bit address NACKed |
| 2 | `ABRT_10ADDR2_NOACK` — second byte NACKed |
| 3 | `ABRT_TXDATA_NOACK` — data byte NACKed |
| 4 | `ABRT_GCALL_NOACK` |
| 5 | `ABRT_GCALL_READ` |
| 6 | `ABRT_HS_ACKDET` |
| 7 | `ABRT_SBYTE_ACKDET` |
| 8 | `ABRT_HS_NORSTRT` |
| 9 | `ABRT_SBYTE_NORSTRT` |
| 10 | `ABRT_10B_RD_NORSTRT` |
| 11 | `ABRT_MASTER_DIS` |
| 12 | `ARB_LOST` |
| 13 | `ABRT_SLVFLUSH_TXFIFO` |
| 14 | `ABRT_SLV_ARBLOST` |
| 15 | `ABRT_SLVRD_INTX` |
| 16 | `ABRT_USER_ABRT` |

After reading `IC_TX_ABRT_SOURCE`, write to `IC_CLR_TX_ABRT` to clear the
flag and re-enable the TX FIFO.

## NVIC Connection

| Symbolic | Vector |
|----------|--------|
| `I2C0_IRQ` | I2C0 combined |
| `I2C1_IRQ` | I2C1 combined |

## 12.2.15. DMA Controller Interface

Each I2C controller has built-in DMA handshakes via `IC_DMA_CR`:

| Bit | Field | Description |
|-----|-------|-------------|
| 1 | `TDMAE` | Enable TX DMA request line |
| 0 | `RDMAE` | Enable RX DMA request line |

DMA transfer model: single-byte transfers. The DMA controller is
programmed with the byte count; each request transfers one byte.

Watermark levels (`IC_DMA_TDLR` for TX, `IC_DMA_RDLR` for RX) can be left
at their reset value (0). I2C is low-bandwidth so DMA bursts are
unnecessary.

### Using DMA for a Master Read of N Bytes

1. Configure the master per the standard init sequence.
2. Set `IC_DMA_CR.RDMAE = 1`.
3. Configure a DMA channel:
   - DREQ = `DREQ_I2C0_RX` (or `_I2C1_RX`).
   - Source = `IC_DATA_CMD` (read).
   - Destination = caller buffer.
   - Transfer count = N.
4. Push N read commands into `IC_DATA_CMD` (`CMD=1`); set `STOP=1` on the
   last command and any RESTART flags as needed. (You can either push them
   from the CPU or use a separate TX DMA channel with DREQ = `_I2C0_TX`.)
5. Start the RX DMA. Bytes flow into the caller's buffer as the master
   issues the bus reads.

### Using DMA for a Master Write of N Bytes

1. Set `IC_DMA_CR.TDMAE = 1`.
2. Build the command stream in memory: each entry is a 16-bit value
   (`IC_DATA_CMD` is 32-bit but only the low half-word is meaningful).
3. DMA channel: DREQ = `DREQ_I2C0_TX`, dest = `IC_DATA_CMD`.
4. Start DMA; the master issues bytes as TX FIFO has space.
5. Mark the last entry with `STOP=1`.

## FT1 Driver Recommendations

- For periodic IMU reads (≤16 bytes), CPU-driven FIFO push is simpler than
  DMA and avoids cross-channel coordination.
- Enable interrupts: `M_TX_ABRT`, `M_STOP_DET`, optionally `M_RX_FULL`.
- Disable: `M_RX_UNDER`, `M_RX_OVER`, `M_RD_REQ`, `M_RESTART_DET`,
  `M_GEN_CALL`, `M_START_DET`, `M_ACTIVITY` for normal operation. Enable
  selectively for debugging.
- Always check `IC_TX_ABRT_SOURCE` before clearing `TX_ABRT`.

## See Also

- [`02_modes.md`](02_modes.md) — Init flow that configures these masks.
- [`06_registers_b.md`](06_registers_b.md) — `IC_INTR_*`, `IC_DMA_*`
  register details.
