# GPIO: Interrupts

[Back to GPIO index](index.md) | [Back to ICD index](../index.md)

## 9.5. Interrupt Sources

For each GPIO, four interrupt scenarios are supported:

| Bit Encoding | Event | Latched? |
|--------------|-------|----------|
| 0x1 | LEVEL_LOW | No (re-evaluated continuously) |
| 0x2 | LEVEL_HIGH | No |
| 0x4 | EDGE_LOW | Yes (in `INTR`) |
| 0x8 | EDGE_HIGH | Yes (in `INTR`) |

Edge events accumulate in the `INTR` register until cleared by writing 1 to
the corresponding bit. Level events are not latched — the interrupt
de-asserts as soon as the pin level changes.

## 9.5.1. Interrupt Destinations

There are three destinations and two security domains, giving twelve
distinct interrupt outputs:

| Bank | Destination | Security |
|------|-------------|----------|
| Bank 0 | proc0, proc1, dormant_wake | Secure + Non-secure |
| QSPI bank | proc0, proc1, dormant_wake | Secure + Non-secure |

Each destination has its own:

- `INTE0..5` — Interrupt enable (32 GPIOs * 4 events / 32 = 4 register
  groups; index `[gpio / 8]`, event-shifted by `4 * (gpio % 8)`).
- `INTS0..5` — Interrupt status after masking & forcing.
- `INTF0..5` — Force a specific interrupt (debug).

The shared raw status `INTR0..5` is at the IO bank level (not per-destination).

## 9.5.2. Encoding Layout

Each `INTE`/`INTS`/`INTF`/`INTR` register packs 8 GPIOs by 4 events each:

```
bit 31..28: GPIO[N+7] {EH, EL, LH, LL}
bit 27..24: GPIO[N+6] ...
...
bit 3..0  : GPIO[N+0] {EDGE_HIGH, EDGE_LOW, LEVEL_HIGH, LEVEL_LOW}
```

Where `N = 8 * register_index`. So `INTR0` covers GPIOs 0-7, `INTR1` covers
8-15, etc. (For QFN-80 the index can run to 5, covering up to GPIO 47.)

Within a 4-bit GPIO field:

| Bit (in nibble) | Event |
|-----------------|-------|
| 0 | LEVEL_LOW |
| 1 | LEVEL_HIGH |
| 2 | EDGE_LOW |
| 3 | EDGE_HIGH |

## 9.5.3. Summary Registers

To find which GPIOs have a pending interrupt without scanning all `INTSn`:

| Register | Per-GPIO Bit |
|----------|---------------|
| `IRQSUMMARY_PROC0_SECURE0/1` | bit `n` = GPIO `n` has any secure proc0 IRQ |
| `IRQSUMMARY_PROC0_NONSECURE0/1` | non-secure variant |
| `IRQSUMMARY_PROC1_SECURE0/1` | proc1 |
| `IRQSUMMARY_PROC1_NONSECURE0/1` | proc1 non-secure |
| `IRQSUMMARY_COMA_WAKE_SECURE0/1` | dormant_wake |
| `IRQSUMMARY_COMA_WAKE_NONSECURE0/1` | dormant_wake non-secure |

`*0` covers GPIOs 0-31, `*1` covers GPIOs 32-47.

## 9.5.4. Programming Pattern

Procedure (per-pin, for proc0):

1. Configure the pad and select the SIO function (`FUNCSEL=5`) if you want
   software input only — but the input register reads back regardless of
   FUNCSEL, so any function works.
2. Compute `event_mask = LEVEL_LOW | LEVEL_HIGH | EDGE_LOW | EDGE_HIGH`
   bits required.
3. Acknowledge any stale latched events: write to `INTR[gpio/8]` shifted
   bits at `4 * (gpio % 8)`.
4. Set the corresponding bits in `proc0_irq_ctrl.inte[gpio/8]`.
5. Install the IRQ handler for `IO_IRQ_BANK0` (proc0 secure) or the
   appropriate vector.
6. In the handler:
   - Read `proc0_irq_ctrl.ints[gpio/8]` to find which events fired.
   - Service them.
   - For edge events, write back the same bits to `INTR[gpio/8]` to clear.

### SDK Example (paraphrased)

```c
void gpio_set_irq_enabled(uint gpio, uint32_t events, bool enabled) {
    io_bank0_irq_ctrl_hw_t *ctrl = (get_core_num() == 0)
        ? &io_bank0_hw->proc0_irq_ctrl
        : &io_bank0_hw->proc1_irq_ctrl;
    gpio_acknowledge_irq(gpio, events);
    io_rw_32 *en_reg = &ctrl->inte[gpio / 8];
    events <<= 4 * (gpio % 8);
    if (enabled) hw_set_bits(en_reg, events);
    else         hw_clear_bits(en_reg, events);
}
```

`gpio_acknowledge_irq()` writes the event bits to the **raw** `INTR`
register, clearing any latched edges.

## 9.5.5. NVIC Vectors

| Symbolic | Description |
|----------|-------------|
| `IO_IRQ_BANK0` | Bank 0 to current core (combined of all GPIOs in bank 0) |
| `IO_IRQ_BANK0_NS` | Non-secure variant |
| `IO_IRQ_QSPI` | QSPI bank to current core |
| `IO_IRQ_QSPI_NS` | Non-secure variant |
| `IO_IRQ_DORMANT_WAKE` | Dormant-wake variant (per bank) |

Refer to Section 3.2 of the source datasheet for the full IRQ table.

## 9.5.6. FT1 Usage

FT1 FSW does **not** rely on GPIO interrupts in the nominal path — TDM
scheduling uses timer interrupts, and peripheral data flow uses
peripheral-specific (UART/SPI/I2C) interrupts or DMA. GPIO interrupts may
optionally be used for:

- A push-button "abort" input (level-low or edge-low).
- A radio "transmit done" flag from the LoRa module if not connected to a
  peripheral interrupt.

Document any GPIO interrupt use in the per-driver design document.

## See Also

- [`02_pads.md`](02_pads.md) — Pad input enable required to receive interrupts.
- [`04_registers.md`](04_registers.md) — Register-level details.
