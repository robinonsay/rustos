# 02 — Electrical Interface

[← Back to index](index.md)

All values in this section are taken verbatim from the **SPECIFICATION**
table on page 4 of the source datasheet, and the **PIN DESCRIPTION** table
on page 3.

## Pin Description (source p.3)

| Pin | Name | I/O | Condition / Description |
|-----|------|-----|-------------------------|
| 1   | VDD  | I   | Power supply |
| 2   | NRST | I   | RESET (active low). 100 kΩ internal pull-up. Pull down at least **100 ms** to reset. |
| 3   | RXD  | I   | UART data input (driven by host TX) |
| 4   | TXD  | O   | UART data output (drives host RX) |
| 5   | NC   | -   | (no connect) |
| 6   | GND  | -   | Ground |

### Note on NSS / SPI Chip-Select

The RYLR896 module **does not expose an NSS / SPI-CS pin** on its 6-pin
header. The host interface is UART-only; the SX1276 SPI bus is internal to
the module's on-board MCU. Designs requiring a "NSS optional" wiring on the
LoRa interface should leave that signal unconnected — the RYLR896 cannot use
it.

## Specification Table (source p.4)

| Item | Min. | Typical | Max. | Unit | Condition |
|------|------|---------|------|------|-----------|
| VDD Power Supply        | 2    | 3.3      | 3.6  | V    | VDD |
| RF Output Power Range   | -4   |          | 15   | dBm  |  |
| Filter insertion loss   | 1    | 2        | 3    | dB   |  |
| RF Sensitivity          |      | -148     |      | dBm  |  |
| RF Input Level          |      |          | 10   | dBm  |  |
| Frequency Range         | 862  | 868/915  | 1020 | MHz  |  |
| Frequency Accuracy      |      | ±2       |      | ppm  |  |
| Communication Range     |      | 4.5      | 15   | km   | Depends on RF parameter & environment |
| Transmit Current        |      | 49.7     |      | mA   | RFOP = +14 dBm |
| Receive Current         |      | 16.5     |      | mA   | `AT+MODE=0` |
| Sleep Current           |      | 0.5      |      | µA   | `AT+MODE=1` |
| UART Baud Rate          | 300  | 115200   | 115200 | bps | 8, N, 1 |
| Digital Input Level High (V_IH) | 0.7·VDD |     | VDD  | V |  |
| Digital Input Level Low (V_IL)  | 0       |     | 0.3·VDD | V |  |
| Digital Output Level High (V_OH)| 0.9·VDD |     | VDD  | V |  |
| Digital Output Level Low (V_OL) |         |     | 0.1  | V |  |
| EEPROM cycling (erase/write)    |         | 300 |      | k cycles |  |
| Weight                  |      | 3.07     |      | g    |  |
| Operating Temperature   | -40  | 25       | +85  | °C   |  |

## Power Sequencing & Reset

- Apply **VDD** in the range **2.0 V – 3.6 V** (3.3 V nominal).
- **NRST** is internally pulled up via 100 kΩ.
  - To force a reset, drive NRST low for **≥100 ms**, then release. The
    internal pull-up returns the module to the de-asserted state.
  - If unused, NRST may be left floating (relies on the internal pull-up) or
    tied to VDD via an external resistor for noise immunity.
- After reset release, allow the module's internal MCU to boot before
  sending AT commands. A conservative settling time of **≥100 ms** after
  NRST release is recommended before issuing the first probe `AT`.
  `[from RYLR896 module reference, not extracted source PDF]`

## Signal-Level Compatibility

The digital I/O thresholds scale with VDD:

- At **VDD = 3.3 V**: V_IH ≥ 2.31 V, V_IL ≤ 0.99 V, V_OH ≥ 2.97 V,
  V_OL ≤ 0.1 V.
- The RYLR896 is therefore directly compatible with **3.3 V CMOS UART**
  hosts (e.g., RP2350 / Pico 2). It is **not 5 V tolerant**; level
  translation is required when interfacing with 5 V hosts.

## Recommended External Components

The extracted source PDF does not specify external decoupling or filtering
beyond the integrated module design. Standard practice
`[from RYLR896 module reference, not extracted source PDF]`:

- Place a **100 nF** decoupling capacitor close to VDD/GND.
- Add a bulk **10 µF** capacitor on the supply rail for transient TX bursts
  (peak ~50 mA at +14 dBm).
- Keep the RF antenna keep-out region per the mechanical drawing (PDF
  pages 5–6).

## Traceability

| Source PDF section     | Covered here |
|------------------------|--------------|
| PIN DESCRIPTION (p.3)  | Pin table    |
| SPECIFICATION (p.4)    | Specification table, signal-level compatibility |

[← 01 Overview](01_overview.md) | [Index](index.md) | [Next: 03 UART Interface →](03_uart_interface.md)
