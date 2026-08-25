# Clocks: Register Map (Part B — FC, Enables, Interrupts)

[Back to clocks index](index.md) | [Back to ICD index](../index.md)

## Frequency Counter Registers

### FC0_REF_KHZ (offset 0x8c)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 19:0 | `FC0_REF_KHZ` | RW | Reference clock frequency, in kHz |

### FC0_MIN_KHZ (offset 0x90)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 24:0 | `FC0_MIN_KHZ` | RW | Minimum pass frequency (set 0 if unused) |

### FC0_MAX_KHZ (offset 0x94)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 24:0 | `FC0_MAX_KHZ` | RW | Maximum pass frequency (set 0x1ffffff if unused) |

### FC0_DELAY (offset 0x98)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 2:0 | `FC0_DELAY` | RW | Delay (in clk_ref periods) before counting starts |

### FC0_INTERVAL (offset 0x9c)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 3:0 | `FC0_INTERVAL` | RW | Test interval = ~1µs * 2^interval (default 8 → 250µs) |

### FC0_SRC (offset 0xa0)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 7:0 | `FC0_SRC` | RW | Source select; writing this register starts a measurement; set 0 when not in use |

`FC0_SRC` enum:

| Value | Source |
|-------|--------|
| 0x00  | NULL (no source) |
| 0x01  | PLL_SYS_CLKSRC_PRIMARY |
| 0x02  | PLL_USB_CLKSRC_PRIMARY |
| 0x03  | ROSC_CLKSRC |
| 0x04  | ROSC_CLKSRC_PH |
| 0x05  | XOSC_CLKSRC |
| 0x06  | CLKSRC_GPIN0 |
| 0x07  | CLKSRC_GPIN1 |
| 0x08  | CLK_REF |
| 0x09  | CLK_SYS |
| 0x0a  | CLK_PERI |
| 0x0b  | CLK_USB |
| 0x0c  | CLK_ADC |
| 0x0d  | CLK_HSTX |
| 0x0e  | LPOSC_CLKSRC |
| 0x0f  | OTP_CLK2FC |
| 0x10  | PLL_USB_CLKSRC_PRIMARY_DFT |

### FC0_STATUS (offset 0xa4)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 28 | `DIED` | RO | Source clock stopped during measurement |
| 24 | `FAST` | RO | Frequency above MAX_KHZ |
| 20 | `SLOW` | RO | Frequency below MIN_KHZ |
| 16 | `FAIL` | RO | DIED or FAST or SLOW |
| 12 | `WAITING` | RO | Currently waiting for the source |
| 8  | `RUNNING` | RO | Counter currently running |
| 4  | `DONE` | RO | Measurement complete; result in FC0_RESULT |
| 0  | `PASS` | RO | Frequency within range |

### FC0_RESULT (offset 0xa8)

| Bits | Field | Type | Description |
|------|-------|------|-------------|
| 29:5 | `KHZ` | RO | Frequency in kHz (integer) |
| 4:0  | `FRAC` | RO | Fractional component (5 bits) |

## Wake/Sleep Enable Registers

Each bit in `WAKE_EN0/1` and `SLEEP_EN0/1` enables one clock destination.
The `ENABLED0/1` registers report the actual current enable state. Reset
value is all 1s (everything enabled in wake; everything enabled in sleep).

| Offset | Name | Description |
|--------|------|-------------|
| 0xac | `WAKE_EN0` | Wake-mode enables, group 0 |
| 0xb0 | `WAKE_EN1` | Wake-mode enables, group 1 |
| 0xb4 | `SLEEP_EN0` | Sleep-mode enables, group 0 |
| 0xb8 | `SLEEP_EN1` | Sleep-mode enables, group 1 |
| 0xbc | `ENABLED0` | Current state of group 0 enables (RO) |
| 0xc0 | `ENABLED1` | Current state of group 1 enables (RO) |

Bit assignments correspond to specific clock destinations (e.g.
`clk_ref_powman`, `clk_sys_uart0`, `clk_peri_spi0`). Refer to the SDK header
`CLOCKS_WAKE_EN0_*_BITS` macros for the full mapping. The FSW driver should:

- Leave `WAKE_EN*` at reset for FT1 (all-on; verified working).
- Set up `SLEEP_EN*` only if entering system sleep is required.

## Interrupt Registers

| Offset | Name | Description |
|--------|------|-------------|
| 0xc4 | `INTR` | Raw interrupt status |
| 0xc8 | `INTE` | Interrupt enable |
| 0xcc | `INTF` | Interrupt force |
| 0xd0 | `INTS` | Interrupt status after masking & forcing |

### INTR / INTE / INTF / INTS

| Bits | Field | Description |
|------|-------|-------------|
| 0 | `CLK_SYS_RESUS` | Resus event detected on clk_sys |

To handle a resus interrupt:

1. Service the IRQ (read `INTS`, confirm bit 0 set).
2. Reconfigure `clk_sys` if needed.
3. Write `CLEAR` bit in `CLK_SYS_RESUS_CTRL` to clear the latch.

## See Also

- [`02_programming.md`](02_programming.md) — Resus enable example using
  `INTE`, the resus control, and the IRQ handler.
- [`03_registers_a.md`](03_registers_a.md) — Generator register details.
