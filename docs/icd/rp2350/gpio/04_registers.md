# GPIO: Register Map

[Back to GPIO index](index.md) | [Back to ICD index](../index.md)

## Base Addresses

| Block | Symbol | Address |
|-------|--------|---------|
| User IO bank | `IO_BANK0_BASE` | `0x40028000` |
| User pad bank | `PADS_BANK0_BASE` | `0x40038000` |

(QSPI bank base addresses are `IO_QSPI_BASE = 0x40030000` and
`PADS_QSPI_BASE = 0x40040000`. QSPI register dumps are intentionally
trimmed — refer to the source PDF Section 9.4 / 9.11.2 / 9.11.4 if needed.)

## 9.11.1. IO_BANK0 Register Layout

For each GPIO `n` (n=0..47), there is a `STATUS` and `CTRL` register pair:

| Offset | Register |
|--------|----------|
| `0x000 + 8*n` | `GPIOn_STATUS` |
| `0x004 + 8*n` | `GPIOn_CTRL` |

Followed by interrupt summary and control banks:

| Offset | Register |
|--------|----------|
| 0x200 | `IRQSUMMARY_PROC0_SECURE0` |
| 0x204 | `IRQSUMMARY_PROC0_SECURE1` |
| 0x208 | `IRQSUMMARY_PROC0_NONSECURE0` |
| 0x20c | `IRQSUMMARY_PROC0_NONSECURE1` |
| 0x210 | `IRQSUMMARY_PROC1_SECURE0` |
| 0x214 | `IRQSUMMARY_PROC1_SECURE1` |
| 0x218 | `IRQSUMMARY_PROC1_NONSECURE0` |
| 0x21c | `IRQSUMMARY_PROC1_NONSECURE1` |
| 0x220 | `IRQSUMMARY_COMA_WAKE_SECURE0` |
| 0x224 | `IRQSUMMARY_COMA_WAKE_SECURE1` |
| 0x228 | `IRQSUMMARY_COMA_WAKE_NONSECURE0` |
| 0x22c | `IRQSUMMARY_COMA_WAKE_NONSECURE1` |
| 0x230..0x244 | `INTR0..INTR5` (raw) |
| 0x248..0x4ff | `proc0_irq_ctrl`, `proc1_irq_ctrl`, `dormant_wake_irq_ctrl` (each containing `INTE0..5`, `INTF0..5`, `INTS0..5`) |

## GPIOn_STATUS Register

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 26 | `IRQTOPROC` | RO | 0 | Interrupt to processor (after override) |
| 17 | `INFROMPAD` | RO | 0 | Raw input from pad |
| 13 | `OETOPAD` | RO | 0 | Output enable to pad after override |
| 9  | `OUTTOPAD` | RO | 0 | Output level to pad after override |

## GPIOn_CTRL Register

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 29:28 | `IRQOVER` | RW | 0 | 0=normal, 1=invert, 2=force LOW, 3=force HIGH |
| 17:16 | `INOVER` | RW | 0 | 0=normal, 1=invert peri input, 2=force LOW, 3=force HIGH |
| 15:14 | `OEOVER` | RW | 0 | 0=peri OE, 1=invert peri OE, 2=disable, 3=enable |
| 13:12 | `OUTOVER` | RW | 0 | 0=peri OUT, 1=invert peri OUT, 2=force LOW, 3=force HIGH |
| 4:0 | `FUNCSEL` | RW | 0x1f | Pin function (0..30, 31 = NULL) |

`FUNCSEL` codes: see [`01_overview.md`](01_overview.md#bank-0-function-numeric-encoding).
The exact enum values depend on the GPIO (per Table 644 of the datasheet);
common codes are listed below — for full per-pin mapping, refer to the
GPIOn_CTRL section in the source datasheet (pages 605-680):

| Code | Typical Function |
|------|------------------|
| 0x00 | HSTX |
| 0x01 | SPI (RX/CSn/SCK/TX depending on GPIO) |
| 0x02 | UART (TX/RX/CTS/RTS) |
| 0x03 | I2C (SDA/SCL) |
| 0x04 | PWM A or B |
| 0x05 | SIO (software) |
| 0x06 | PIO0 |
| 0x07 | PIO1 |
| 0x08 | PIO2 |
| 0x09 | CLOCK GPIN/GPOUT or QMI CS1n |
| 0x0a | USB (VBUS_EN, VBUS_DET, OVERCURR_DETECT) |
| 0x0b | UART (alternate mapping) |
| 0x1f | NULL (default) |

## INTRn / INTEn / INTFn / INTSn Register Layout

Each register packs 8 GPIOs * 4 event bits = 32 bits:

| Bit (within nibble for GPIO `g`) | Event |
|----------------------------------|-------|
| 0 | LEVEL_LOW |
| 1 | LEVEL_HIGH |
| 2 | EDGE_LOW (latched in INTR; write 1 to clear) |
| 3 | EDGE_HIGH (latched in INTR; write 1 to clear) |

Register `INTRn` covers GPIOs `8n..8n+7` (`n=0..5`).

`INTRn` access types:
- LEVEL bits are RO.
- EDGE bits are WC (write-1-to-clear in INTR).

`INTEn` and `INTFn` are RW, same layout.

`INTSn` is RO (status after enable mask + force).

## IRQSUMMARY_* Registers

Each bit `n` = "GPIO `n` has at least one enabled interrupt asserted to this
destination". `*0` registers cover GPIOs 0-31; `*1` cover 32-47.

## 9.11.3. PADS_BANK0 Register Layout

| Offset | Register |
|--------|----------|
| 0x00 | `VOLTAGE_SELECT` |
| 0x04 + 4*n | `GPIOn` (pad control, n=0..47) |
| ... | `SWCLK`, `SWD` pad controls |

### PADS_BANK0.VOLTAGE_SELECT

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 0 | `VOLTAGE_SELECT` | RW | 0 | 0=3.3V (default), 1=1.8V |

### PADS_BANK0.GPIOn Register

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 8 | `ISO` | RW | 1 | Isolation latch (1=latched, 0=transparent) |
| 7 | `OD` | RW | 0 | Output disable (override peri OE) |
| 6 | `IE` | RW | 0 | Input enable |
| 5:4 | `DRIVE` | RW | 0x1 | 00=2 mA, 01=4 mA, 10=8 mA, 11=12 mA |
| 3 | `PUE` | RW | 0 | Pull-up enable |
| 2 | `PDE` | RW | 1 | Pull-down enable |
| 1 | `SCHMITT` | RW | 1 | Schmitt trigger enable |
| 0 | `SLEWFAST` | RW | 0 | 0=slow, 1=fast slew |

> Bus keeper mode = `PUE=1` and `PDE=1` simultaneously.

### PADS_BANK0 SWCLK / SWD

The serial-wire-debug pads have the same register layout but reset values
optimised for debug (typically `IE=1`, `PDE=1`).

## 9.11.4. PADS_QSPI Register Layout (summary)

| Offset | Register |
|--------|----------|
| 0x00 | `VOLTAGE_SELECT` |
| 0x04 + 4*n | `GPIO_QSPI_<SCK\|CSn\|SD0..SD3>` |

Each per-pad register has the same field layout as `PADS_BANK0.GPIOn`.
Reset values differ:
- `IE` resets to 1.
- Pull configuration: SCK/SD0/SD1 pull-down; SD2/SD3/CSn pull-up.

The FT1 FSW does not modify these registers — they are owned by the QMI/XIP
boot ROM/SDK.

## See Also

- [`01_overview.md`](01_overview.md) — Function table.
- [`02_pads.md`](02_pads.md) — Pad behaviour and SIO.
- [`03_interrupts.md`](03_interrupts.md) — Interrupt programming.
