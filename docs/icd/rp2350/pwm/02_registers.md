# PWM: Register Map

[Back to PWM index](index.md) | [Back to ICD index](../index.md)

## Base Address

`PWM_BASE = 0x400a8000`. The block has the `+0x1000` XOR, `+0x2000` set and
`+0x3000` clear aliases of §2.1.3.

## 12.5.3 Register List (Table 1130)

Slice `n` (0 to 11) occupies `0x14 × n` to `0x14 × n + 0x10`:

| Offset | Name | Description |
|--------|------|-------------|
| `0x14n + 0x00` | `CHn_CSR` | Control and status |
| `0x14n + 0x04` | `CHn_DIV` | Divider, `INT` and `FRAC` |
| `0x14n + 0x08` | `CHn_CTR` | Counter, direct access |
| `0x14n + 0x0c` | `CHn_CC` | Compare values A and B |
| `0x14n + 0x10` | `CHn_TOP` | Wrap value |
| `0x0f0` | `EN` | Enable bits of all slices (aliases each `CSR.EN`) |
| `0x0f4` | `INTR` | Raw interrupts |
| `0x0f8` | `IRQ0_INTE` | Interrupt enable for IRQ 0 |
| `0x0fc` | `IRQ0_INTF` | Interrupt force for IRQ 0 |
| `0x100` | `IRQ0_INTS` | Interrupt status for IRQ 0 |
| `0x104` | `IRQ1_INTE` | Interrupt enable for IRQ 1 |
| `0x108` | `IRQ1_INTF` | Interrupt force for IRQ 1 |
| `0x10c` | `IRQ1_INTS` | Interrupt status for IRQ 1 |

## CHn_CSR, Table 1131

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:8 | reserved | - | - | |
| 7 | `PH_ADV` | SC | `0x0` | Advance the phase by one count while running; write 1, poll until low; needs a divider above 1 |
| 6 | `PH_RET` | SC | `0x0` | Retard the phase by one count while running |
| 5:4 | `DIVMODE` | RW | `0x0` | `0` `DIV` free-running; `1` `LEVEL` gated by the B pin; `2` `RISE` count B rising edges; `3` `FALL` count B falling edges |
| 3 | `B_INV` | RW | `0x0` | Invert output B |
| 2 | `A_INV` | RW | `0x0` | Invert output A |
| 1 | `PH_CORRECT` | RW | `0x0` | 1 phase-correct, 0 trailing-edge |
| 0 | `EN` | RW | `0x0` | Enable the slice |

## CHn_DIV, Table 1132

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:12 | reserved | - | - | |
| 11:4 | `INT` | RW | `0x01` | Integer part of the divider |
| 3:0 | `FRAC` | RW | `0x0` | Fractional part in sixteenths (first-order sigma-delta) |

## CHn_CTR, Table 1133

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15:0 | `CTR` | RW | `0x0000` | The counter |

## CHn_CC, Table 1134

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 31:16 | `B` | RW | `0x0000` | Compare value, channel B |
| 15:0 | `A` | RW | `0x0000` | Compare value, channel A |

## CHn_TOP, Table 1135

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 15:0 | `TOP` | RW | `0xffff` | Counter wrap value |

## EN, Table 1136

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 11:0 | `CH11` .. `CH0` | RW | `0x0` | One enable bit per slice; the same physical bit as that slice's `CSR.EN` |
