# 03 — I2C Interface

[← Back to Baro ICD index](index.md)

## 3.1 Overview

The MPL3115A2 exposes its complete register file via a single
**I2C slave** interface (SCL = pin 8, SDA = pin 7). The interface
supports:

- Standard mode  — up to 100 kHz.
- Fast mode      — up to **400 kHz**.

Both lines are open-drain and require external pull-up resistors to
**VDDIO**. Typical values: 4.7 kΩ for short bus runs, smaller for
high-capacitance buses (max bus capacitance C_b = 400 pF for 1 kΩ
pull-up at 400 kHz).

## 3.2 Slave Address

| Form | Value |
|------|-------|
| 7-bit slave address          | **0x60** (binary `1100000`) |
| 8-bit write address (R/W=0)  | **0xC0** |
| 8-bit read  address (R/W=1)  | **0xC1** |

The slave address is **fixed in silicon**. The MPL3115A2 does *not*
acknowledge the I2C general-call address (`0000_000`). Alternate
addresses are available only via factory order (consult application
note AN4481).

## 3.3 Bus Timing

All values referenced to V_IH(min) and V_IL(max). Source: Table 4 of
the datasheet.

| Symbol | Parameter | Min | Max | Unit |
|--------|-----------|-----|-----|------|
| f_SCL  | SCL clock frequency (C_b ≤ 400 pF, R_p = 1 kΩ) | 0 | 400 | kHz |
| f_SCL  | SCL clock frequency (C_b ≤ 20 pF,  R_p = 1 kΩ) | 0 | 4   | MHz |
| t_BUF  | Bus free time STOP→START | 1.3 | — | µs |
| t_HD;STA | Repeated START hold time | 0.6 | — | µs |
| t_SU;STA | Repeated START setup time | 0.6 | — | µs |
| t_SU;STO | STOP condition setup time | 0.6 | — | µs |
| t_HD;DAT | SDA data hold time | 50 | — | ns |
| t_SU;DAT | SDA setup time | 100 | — | ns |
| t_LOW  | SCL low time | 1.3 | — | µs |
| t_HIGH | SCL high time | 0.6 | — | µs |
| t_r    | SDA, SCL rise time | 20 + 0.1·C_b | 300 | ns |
| t_f    | SDA, SCL fall time | 20 + 0.1·C_b | 300 | ns |
| t_SP   | Spike pulse width suppressed by filter | — | 50 | ns |

Notes:

- The device internally provides ≥ 300 ns hold time on SDA, sufficient
  to bridge the falling edge of SCL.
- The device **does not stretch** the LOW period of SCL.
- For fast-mode use within a standard-mode system, t_SU;DAT ≥ 250 ns
  must be met.

## 3.4 Bus Protocol Primitives

### 3.4.1 START / STOP

- **START** = high-to-low transition on SDA while SCL is high.
- **STOP**  = low-to-high transition on SDA while SCL is high.
- After START the bus is "busy" until a STOP is seen.

### 3.4.2 ACK / NACK

The 9th SCL pulse following each byte is the acknowledge bit. The
transmitter releases SDA; the receiver pulls SDA low for ACK or
leaves it high for NACK.

### 3.4.3 Repeated START

A master may issue a repeated START (Sr) without releasing the bus.
The MPL3115A2 expects repeated STARTs to be used when randomly
reading from a specific register (write-register-pointer, then
read-data).

## 3.5 Transaction Formats

### 3.5.1 Single-byte Write

```
S | 0xC0 | A | reg | A | data | A | P
```

Where `S` = START, `P` = STOP, `A` = ACK from slave, `0xC0` = 8-bit
slave write address, `reg` = target register address, `data` = value
to write.

### 3.5.2 Single-byte Read

```
S  | 0xC0 | A | reg | A | Sr | 0xC1 | A | data | NACK | P
```

The repeated START (`Sr`) avoids relinquishing the bus and re-issues
the slave address with the read bit set. The master sends NACK on the
final byte before STOP.

### 3.5.3 Multi-byte Read (Burst / "F_RD" mode)

The MPL3115A2 supports an **auto-incrementing address pointer**. A
single read transaction may pull multiple consecutive registers in
one go; after each data byte is acknowledged, the device advances its
internal pointer to the next register.

```
S | 0xC0 | A | reg_start | A | Sr | 0xC1 | A
                | data[reg_start]     | A
                | data[reg_start + 1] | A
                | ...                       
                | data[reg_start + N-1] | NACK | P
```

The auto-increment table is given in `05_register_map.md` (column
"Auto-Increment Address"). Notable wrap behavior:

- Pointer at `0x05` (OUT_T_LSB) wraps back to `0x00` (STATUS), so a
  6-byte burst from `0x00` reads STATUS + Pressure + Temperature in
  one shot — the canonical fast-poll pattern.
- Pointer at `0x0B` wraps to `0x06` for delta data.
- When `F_MODE > 0`, address `0x01` aliases to `F_DATA`, and reads
  pull queued FIFO samples; the pointer **does not advance** out of
  `0x01` for FIFO bursts (each read drains one FIFO byte).

This mode is referred to in this ICD and in the AN4519 application
note as "**F_RD multi-byte read**".

### 3.5.4 Multi-byte Write

Multi-byte writes also auto-increment the register pointer:

```
S | 0xC0 | A | reg_start | A | data0 | A | data1 | A | ... | dataN-1 | A | P
```

This is the recommended approach for writing the contiguous
CTRL_REGn block (`0x26`–`0x2A`) in one transaction.

### 3.5.5 Clock Stretching

The MPL3115A2 itself does not stretch SCL. A host that cannot accept
data fast enough may stretch SCL low to delay the next byte; the
device tolerates arbitrary stretch durations.

## 3.6 Read of WHO_AM_I (presence check)

A standard presence check is a single-byte read of register `0x0C`.

```
Master: S 0xC0 A 0x0C A Sr 0xC1 A
Slave:                            0xC4
Master:                                  NACK P
```

A correctly-wired, powered MPL3115A2 returns `0xC4`. Any other value
indicates either a wiring fault, a different sensor on the bus, or a
custom-programmed device.

## 3.7 Bus Utilization Notes for Juno FSW

- The Juno bus is shared with the IMU and other I2C peripherals;
  the baro driver must not hold the bus during the conversion period
  (up to 512 ms at 128× OSR). All MPL3115A2 reads are short
  transactions; conversion happens autonomously.
- Polling for STATUS.PTDR via repeated single-byte reads is bus-
  intensive at high SCL; prefer using the data-ready interrupt
  (INT2 default route, see [`04_modes.md`](04_modes.md)).

[← Back to Baro ICD index](index.md)
