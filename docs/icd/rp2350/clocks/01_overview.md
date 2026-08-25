# Clocks: Overview & Functional Description

[Back to clocks index](index.md) | [Back to ICD index](../index.md)

## 8.1. Architecture

The clocks block provides independent clocks to on-chip and external
components. It uses multiple clock generators to derive the required clocks
from a small set of clock sources.

```
Clock sources                 Clock generators            Destinations
-----------------             ---------------------       --------------
XOSC (1-50 MHz)               clk_gpout0..3   ÷           GPIO Muxing
ROSC (~11 MHz)         ─►     clk_ref         ÷ en        Watchdog/Timers/OTP
LPOSC (32 kHz)                clk_sys         ÷ en        Cores/Bus/Memories
GPIN0..1                      clk_peri        ÷ en        UART + SPI
PLL_SYS                       clk_usb         ÷ en        USB
PLL_USB                       clk_adc         ÷ en        ADC
                              clk_hstx        ÷ en        HSTX
```

Always-on domain: `clk_pow` (Power Manager), `clk_ref` AON-tick.
Switched-core domain: everything else.

## 8.1.1. Clock Sources

| Source | Frequency | Startup | Notes |
|--------|-----------|---------|-------|
| LPOSC  | ~32 kHz | Auto on AON power | 1% tunable; varies with PVT |
| ROSC   | 4.6-19.6 MHz (nom 11 MHz) | Auto at power-on | Boots the chip; PVT-sensitive |
| XOSC   | 1-50 MHz (typ 12 MHz crystal) | Several ms; wait for `XOSC_STABLE` | Required for ±1000 ppm clocks |
| GPIN0/1 | ≤50 MHz | External | Replaces ROSC/XOSC if external clock available |
| PLL_SYS | up to spec | After XOSC stable | Drives `clk_sys` |
| PLL_USB | 48 MHz | After XOSC stable | Drives `clk_usb`, `clk_adc`, optional `clk_peri` |

**FSW rule:** Always wait for `XOSC_STABLE` before starting PLLs. Always wait
for `LOCK` in the PLL `STATUS` register before switching `clk_sys` to the PLL
output.

### LPOSC

Always-on 32 kHz oscillator. Used for the Always-On (AON) timer and the power
manager when switched-core is off. Frequency varies with V/T.

### ROSC

Boot oscillator. Available immediately after switched-core power-up. Stops in
DORMANT mode (auto-restart on exit). The FSW does not need to use the ROSC
beyond boot — switch to XOSC + PLL early in init.

### XOSC

Reference for the PLLs. Requires external crystal (12 MHz on the Pico 2 ref
design). Stays inactive until enabled in software. Startup takes several ms;
software must poll `XOSC_STABLE`.

### PLLs

| PLL | Default consumer | Notes |
|-----|------------------|-------|
| PLL_SYS | `clk_sys` (and `clk_peri` if not detached) | Variable frequency |
| PLL_USB | `clk_usb`, `clk_adc` (always 48 MHz) | Fixed 48 MHz |

After starting a PLL, wait for the `LOCK` bit in its `STATUS` register before
using its output. Output cannot be used during reference-divider or
output-divider changes, or during bypass changes; it can be used during
feedback-divisor changes (with possible over/undershoot).

## 8.1.2. Clock Generators

Each generator has:

- An auxiliary (aux) mux — glitches when select control changes.
- An optional glitchless mux (only on `clk_ref` and `clk_sys`).
- A fractional divider (1.0 to 2^16) — glitch-free divisor changes.
- Optional duty-cycle correction (DCC) — restores 50% duty when divisor is odd.
- Wake/Sleep enable bits.

### Generators on RP2350

| Name | Description | Nominal Frequency |
|------|-------------|-------------------|
| `clk_gpout0..3` | Output to GPIO (debug/external) | N/A |
| `clk_ref` | Reference (always-on) | 6-12 MHz |
| `clk_sys` | System clock (always-on) | 150 MHz |
| `clk_peri` | Peripheral clock (UART, SPI, I2C) | 12-150 MHz |
| `clk_usb` | USB reference (must be 48 MHz) | 48 MHz |
| `clk_adc` | ADC reference (must be 48 MHz) | 48 MHz |
| `clk_hstx` | HSTX clock | 150 MHz |

### Mux Switching Procedures

**Glitchless mux (clk_ref / clk_sys):**

1. Switch the glitchless mux to an alternate source.
2. Poll `SELECTED` until the switch completes.

**Aux mux on a generator that has a glitchless mux:**

1. Switch glitchless mux away from aux.
2. Poll `SELECTED`.
3. Change aux-mux select.
4. Switch glitchless mux back to aux.
5. Poll `SELECTED`.

**Aux mux on a generator without a glitchless mux:**

1. Disable the divider (clear `CTRL_ENABLE`).
2. Wait two source-clock cycles.
3. Change aux-mux select.
4. Re-enable the divider.

> Failure to follow these sequences may glitch the destination clock and
> corrupt downstream logic.

### Divider

- Fractional, range 1.0 to 2^16.
- On-the-fly divisor changes synchronized to the end of each clock cycle.
- `INT=0` is interpreted as max+1.
- `KILL` exists to force-stop a stuck generator (do not use in normal flow).

### Duty Cycle Correction

Enable `DC50` in the generator's `CTRL` to restore 50% duty when dividing by
odd numbers. No-op for even divisors.

### Clock Enables

Each destination has a `WAKE_EN` bit and a `SLEEP_EN` bit. Reset value is
`0x1` (enabled). Exceptions (no enable):

- `clk_gpclk0..3`
- The processor cores (always need a clock for power management).
- `clk_sys_busfabric` and `clk_sys_clocks` in wake mode.

System sleep automatically swaps from `WAKE_EN` to `SLEEP_EN` when both cores
sleep and DMA is idle.

## 8.1.3. Frequency Counter

Measures the frequency of internal/external clocks by counting edges over a
test interval defined by `clk_ref` cycles. `clk_ref` must be stable (XOSC or
external).

| `FC0_INTERVAL` | Test Window | Accuracy |
|----------------|-------------|----------|
| 0  | 1 µs   | 2048 kHz |
| 5  | 32 µs  | 64 kHz |
| 10 | 1 ms   | 2 kHz |
| 15 | 32 ms  | 62.5 Hz |

Test mode also supports min/max range checking with `PASS`/`SLOW`/`FAST`/
`DIED` flags.

## 8.1.4. Resus (Resuscitate)

Automatic safety net: if `clk_sys` stops, the resus circuit switches `clk_sys`
to `clk_ref` after a programmable timeout. `clk_ref` must keep running (XOSC
or ROSC) for resus to work.

To enable:
- Program `CLK_SYS_RESUS_CTRL` timeout interval.
- Set `ENABLE` bit in `CLK_SYS_RESUS_CTRL`.
- Enable the `CLK_SYS_RESUS` interrupt in `INTE`.

To recover after a resus event:
- Reconfigure `clk_sys`.
- Write the `CLEAR` bit in `CLK_SYS_RESUS_CTRL`.

> **Warning:** Resus is a debug feature. Do not rely on it in flight code.
> A `clk_sys` running too slow can spuriously trigger it and cause a glitch.

## 8.1.5. DORMANT-mode Considerations (informational)

- ROSC/XOSC stop automatically in DORMANT, restart on exit.
- PLLs do **not** auto-restart — software must stop them before entering
  DORMANT and reconfigure on exit.
- LPOSC continues to run.

FSW for FT1 does not enter DORMANT, but the design must avoid leaving PLLs
running across a sleep transition.

## See Also

- [`02_programming.md`](02_programming.md) — Init sequence & code examples.
- [`03_registers_a.md`](03_registers_a.md) — Register details.
