# GPIO: Pads, Isolation, SIO

[Back to GPIO index](index.md) | [Back to ICD index](../index.md)

## 9.6. Pads

Each GPIO connects off-chip via a pad. Adjustable electrical parameters:

| Parameter | Options |
|-----------|---------|
| Output drive strength | 2 mA / 4 mA / 8 mA / 12 mA |
| Output slew rate | slow or fast |
| Input hysteresis | Schmitt trigger on/off |
| Pull-up / pull-down | up, down, neither, or "bus keeper" |
| Input buffer | enabled or disabled |

`IOVDD` may be 1.8 V to 3.3 V. If powered at 1.8 V, set the
`VOLTAGE_SELECT` register for the bank — otherwise input thresholds will not
meet specification. Default thresholds are valid for 2.5-3.3 V.

> **Warning:** Using IOVDD > 1.8 V with thresholds set for 1.8 V may damage
> the chip.

### Pad Control Register Fields (per pad, e.g. `PADS_BANK0.GPIO0`)

| Bits | Field | Type | Reset | Description |
|------|-------|------|-------|-------------|
| 8 | `ISO` | RW | 1 | Isolation latch. 1 = latched (signals frozen). 0 = transparent. |
| 7 | `OD` | RW | 0 | Output disable (overrides peripheral output enable) |
| 6 | `IE` | RW | 0 | Input enable. **Must be 1 to receive any digital input.** |
| 5:4 | `DRIVE` | RW | 0x1 | 0=2mA, 1=4mA, 2=8mA, 3=12mA |
| 3 | `PUE` | RW | 0 | Pull-up enable |
| 2 | `PDE` | RW | 1 | Pull-down enable (Bank 0 default) |
| 1 | `SCHMITT` | RW | 1 | Schmitt trigger enable |
| 0 | `SLEWFAST` | RW | 0 | 0=slow slew, 1=fast |

> **Caution (RP2350-E9):** Pull-down does not function as expected under
> certain conditions. Refer to the official errata before relying on
> internal pull-down for level-sensitive inputs.

### Bus Keeper Mode

Setting **both** `PUE=1` and `PDE=1` enables bus-keeper mode: the pad weakly
holds whichever logic level was last present. Use this when the pin is
floated by an external driver and a defined level must be retained.

### Bank Voltage Select

| Register | Bit | Effect |
|----------|-----|--------|
| `PADS_BANK0.VOLTAGE_SELECT` | 0 | 0=3.3V thresholds, 1=1.8V thresholds |
| `PADS_QSPI.VOLTAGE_SELECT` | 0 | same |

Both registers must agree (single shared IOVDD).

### Recommended Pad Settings per Peripheral

| Peripheral | Pad Settings |
|------------|--------------|
| UART RX/TX | `IE=1`, `OD=0`, slew slow, Schmitt on, no pulls |
| I2C SDA/SCL | `IE=1`, `OD=0`, slew slow, Schmitt on, `PUE=1` (plus external pull-up) |
| SPI SCK/TX  | `IE=1`, `OD=0`, drive ≥4 mA, slew fast acceptable for high baud |
| SPI RX/CSn  | `IE=1`, Schmitt on, weak pull-up if floating |
| Clock IN    | `IE=1`, Schmitt on, slow slew |
| Clock OUT   | `IE=0` (optional), drive ≥4 mA |

## 9.7. Pad Isolation Latches

The isolation latches sit between the switched-core domain and the pad. In
normal operation they are transparent; when set, they freeze the pad's
output enable, output level, and pull control. They are also automatically
latched whenever the switched-core domain powers down.

### Lifecycle

1. After any reset of the always-on domain, all `ISO` bits reset to **1**.
2. Software configures the IO mux (`gpio_set_function()` writes `FUNCSEL`).
3. Software clears `ISO` to allow signals to propagate to the pad.
4. If the chip later powers down switched-core (deep sleep), the latches
   capture the current pad outputs.
5. On wake, `ISO=1` is re-asserted; software must clear `ISO` again to
   release the pad.

> Non-SDK code ported from RP2040 must clear `ISO` explicitly. RP2040 had no
> isolation latches.

The `PADS` register block reset (driven by `RESETS` control registers) does
**not** affect isolation latches. Only an always-on-domain reset clears
them.

## 9.8. Processor GPIO Controls (SIO)

The single-cycle IO subsystem (SIO, base `0xd0000000`) provides
memory-mapped registers for software-driven GPIO:

| Register | Width | Function |
|----------|-------|----------|
| `GPIO_OUT`, `GPIO_HI_OUT` | 32 bits each | Output level, GPIOs 0-31 / 32-47+ |
| `GPIO_OE`, `GPIO_HI_OE` | 32 bits each | Output enable |
| `GPIO_IN`, `GPIO_HI_IN` | 32 bits each | Read input level |

Notes:

- The `_OUT`/`_OE` registers take effect only when the GPIO's `FUNCSEL=5`
  (SIO) is selected.
- `_IN` registers can be read at any time, regardless of FUNCSEL — software
  can always observe the pin state.
- `_IN_HI` covers GPIOs 32-47 plus QSPI and USB DP/DM pins.
- Atomic set/clear/xor aliases (`GPIO_OUT_SET`, `GPIO_OUT_CLR`,
  `GPIO_OUT_XOR`) are at standard +0x1000/+0x2000/+0x3000 offsets relative
  to SIO_BASE.

The SIO GPIO registers are shared between the two cores. Non-secure access
is restricted by the masks in `GPIO_NSMASK0` / `GPIO_NSMASK1`.

> The DMA cannot access SIO. Use a PIO program for DMA-driven GPIO output.

## 9.9. GPIO Coprocessor Port (informational)

Coprocessor port 0 on each Cortex-M33 provides single-instruction access to
SIO GPIO registers. This is useful for low-overhead GPIO toggling in
interrupt handlers but not required by the FT1 driver layer.

## 9.10. Software Examples

### 9.10.1. Setting an IO function (paraphrased SDK)

```c
void gpio_set_function(uint gpio, gpio_function_t fn) {
    // 1. Enable input, clear output disable on the pad.
    pads_bank0_hw->io[gpio] =
        (pads_bank0_hw->io[gpio] & ~OD_BITS) | IE_BITS;
    // 2. Write the function select. Clears overrides as a side effect.
    io_bank0_hw->io[gpio].ctrl = fn << FUNCSEL_LSB;
    // 3. Drop pad isolation now that the peripheral owns the pad.
    pads_bank0_hw->io[gpio] &= ~ISO_BITS;
}
```

The same routine for the QSPI bank uses `pads_qspi_hw` and `io_qspi_hw`.

## See Also

- [`01_overview.md`](01_overview.md) — Function table.
- [`03_interrupts.md`](03_interrupts.md) — GPIO interrupt configuration.
- [`04_registers.md`](04_registers.md) — Pad and IO register layouts.
