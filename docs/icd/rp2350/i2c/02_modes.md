# I2C: Operation Modes

[Back to I2C index](index.md) | [Back to ICD index](../index.md)

## 12.2.10.1. Slave Mode (informational)

FT1 does **not** use the controllers as slaves. Documented here for
completeness.

### Initial Configuration

1. Disable: write `IC_ENABLE.ENABLE = 0`.
2. Set slave address in `IC_SAR` (bits 9:0).
3. Configure `IC_CON`: clear `MASTER_MODE`, clear `IC_SLAVE_DISABLE`,
   choose 7- or 10-bit addressing (`IC_10BITADDR_SLAVE`).
4. Enable: `IC_ENABLE.ENABLE = 1`.

> **Warning:** Only de-assert reset of the slave when the bus is IDLE; the
> internal SDA/SCL synchronizer flip-flops can produce a false START
> otherwise. Alternatively, enable in master-only mode first then switch.

### Slave-Transmitter / Slave-Receiver

See source PDF Sections 12.2.10.1.2-12.2.10.1.4 for the per-byte handshake
sequences (RD_REQ, RX_FULL, TX_ABRT). Not used by FT1.

## 12.2.10.2. Master Mode

### 12.2.10.2.1. Initial Configuration

```
1. Disable      : IC_ENABLE.ENABLE = 0
2. IC_CON       : MASTER_MODE = 1, IC_SLAVE_DISABLE = 1,
                  SPEED = 1 (Standard) | 2 (Fast/FM+),
                  IC_10BITADDR_MASTER = 0 or 1
3. IC_TAR       : target slave address (bits 9:0); also chooses general
                  call / start byte
4. *_HCNT/_LCNT : program SCL high/low counts (see 03_timing.md)
5. IC_FS_SPKLEN : program spike length (see 03_timing.md)
6. IC_SDA_HOLD  : program SDA hold time (typ. left at default)
7. IC_RX_TL     : RX FIFO threshold (default 0 → fire on every byte)
8. IC_TX_TL     : TX FIFO threshold
9. IC_INTR_MASK : enable required interrupts
10. IC_DMA_CR   : enable DMA if used
11. Enable      : IC_ENABLE.ENABLE = 1
```

After enable, write to `IC_DATA_CMD` initiates the transfer.

### 12.2.10.2.2. Master Transmit and Master Receive

Each `IC_DATA_CMD` write encodes one bus byte:

| Bit | Meaning |
|-----|---------|
| 8 (CMD)  | 0 = master-write, 1 = master-read |
| 9 (STOP) | 1 = issue STOP after this transfer |
| 10 (RESTART) | 1 = issue RESTART before this transfer |
| 7:0 (DAT) | Data byte for write; ignored for read |

The master continues transferring as long as commands are queued. If TX
FIFO empties:

- If the just-completed command had `STOP=1`, the master issues STOP.
- Otherwise, the master holds SCL low until the next `IC_DATA_CMD` write.

### Repeated Start

To switch direction or address mid-transaction, set the `RESTART` bit on
the next command. (Make sure restarts are not disabled in `IC_CON`.)

## 12.2.10.3. Disabling DW_apb_i2c

Write `IC_ENABLE.ENABLE = 0` to start shutdown. The master only fully
disables once the current command has `STOP=1` and the bus has been
released. Use `IC_ENABLE_STATUS.IC_EN` to detect completion.

### Procedure (`ti2c_poll`)

```
1. ti2c_poll = 10 / max_bus_speed   # ≈ 25 µs at 400 kb/s
2. POLL_COUNT = 0
3. IC_ENABLE.ENABLE = 0
4. while IC_ENABLE_STATUS.IC_EN == 1:
       sleep ti2c_poll
       POLL_COUNT += 1
       if POLL_COUNT > MAX_T_POLL_COUNT:
           return ERROR
5. return SUCCESS
```

> Earlier IP versions required polling `IC_STATUS` and
> `IC_RAW_INTR_STAT`. RP2350's IP only needs `IC_ENABLE_STATUS`.

## 12.2.10.4. Aborting I2C Transfers

To bail out of a queued sequence (e.g., to break free of a misbehaving
slave) without waiting for `STOP`:

```
1. Stop pushing to TX FIFO.
2. If using DMA, clear IC_DMA_CR.TDMAE.
3. Set IC_ENABLE.ABORT = 1.
4. Wait for the M_TX_ABRT interrupt.
5. Read IC_TX_ABRT_SOURCE — the cause is ABRT_USER_ABRT.
6. (Optional) Clear the abort latch via IC_CLR_TX_ABRT.
```

The hardware issues STOP and flushes the TX FIFO automatically. After
`TX_ABRT`, the TX FIFO stays in a flushed/reset state; software must read
`IC_CLR_TX_ABRT` before pushing new data.

## 12.2.12. Fast Mode Plus Operation

Steps before any FM+ transfer:

1. Set `ic_clk` ≥ 32 MHz (i.e., `clk_sys ≥ 32 MHz`).
2. `IC_CON[2:1] = 0b10` (Fast / FM+).
3. Program `IC_FS_SCL_LCNT` and `IC_FS_SCL_HCNT` for FM+ timing.
4. Program `IC_FS_SPKLEN` to suppress 50 ns spikes.
5. Program `IC_SDA_SETUP` to meet `tSU;DAT`.

See [`03_timing.md`](03_timing.md) for HCNT/LCNT computation.

## 12.2.13. Bus Clear (informational)

If `SDA` is stuck low, the master sends up to 9 SCL pulses to free a hung
slave. If `SCL` is stuck low, hardware reset is the only recovery.

## FT1 Driver Operation Recommendations

- Always queue **all** bytes for a single transaction in the TX FIFO before
  enabling, OR use DMA, to avoid empty-FIFO holds.
- Set `STOP=1` on the last byte of each transaction.
- For register reads from a sensor: write the register address (no STOP),
  RESTART, then read with `CMD=1` and `STOP=1` on the last byte.
- After a `TX_ABRT`, log the source from `IC_TX_ABRT_SOURCE` and clear via
  `IC_CLR_TX_ABRT`.
- Detect bus hang by checking `IC_STATUS.ACTIVITY` and `IC_RAW_INTR_STAT`
  after a watchdog timeout.

## See Also

- [`03_timing.md`](03_timing.md) — Timing register calculation.
- [`04_interrupts_dma.md`](04_interrupts_dma.md) — Interrupts & DMA.
- [`05_registers_a.md`](05_registers_a.md), [`06_registers_b.md`](06_registers_b.md) — Register details.
