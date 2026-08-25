# LoRa Radio ICD — REYAX RYLR896

Interface Control Document for the REYAX RYLR896 868/915 MHz LoRa Antenna
Transceiver Module. The RYLR896 wraps a Semtech SX1276 LoRa engine behind a
simple AT-command UART interface and an integrated antenna.

- **Source datasheet:** [`../../RYLR896_EN.pdf`](../../RYLR896_EN.pdf)
  (REYAX TECHNOLOGY CO., LTD., 01-Nov-2021, document 56312E37)
- **Parent ICD index:** [`../avionics.md`](../avionics.md)

> **Source-fidelity note.** The supplied PDF is image-heavy; the extracted
> text covers the cover page, product description, pin table, electrical
> specification, and certifications, but the AT command set, response format,
> and init sequence are **not** present in the extracted source text. Items
> drawn from the public RYLR896 AT-command reference are flagged inline as
> `[from RYLR896 module reference, not extracted source PDF]`.

## Key Specifications (at-a-glance)

| Item | Value |
|------|-------|
| Modulation engine | Semtech SX1276 LoRa |
| Frequency band | 862 – 1020 MHz (868/915 MHz typical) |
| RF output power | -4 to +15 dBm |
| RF sensitivity | -148 dBm |
| Communication range | 4.5 km typical, up to 15 km (env. dependent) |
| Host interface | UART (AT commands), 8N1 |
| Default UART baud | 115200 bps (range 300 – 115200) |
| I2C interface | **N/A** (UART-only module) |
| Supply voltage (VDD) | 2.0 / 3.3 / 3.6 V (min/typ/max) |
| TX current | 49.7 mA at +14 dBm |
| RX current | 16.5 mA (`AT+MODE=0`) |
| Sleep current | 0.5 µA (`AT+MODE=1`) |
| Encryption | AES-128 (data link) |
| Operating temperature | -40 °C to +85 °C |
| Certifications | FCC (QLY-RYLR896), NCC (CCAN18LP0920T8) |

## Pinout Summary

| Pin | Name | Direction | Role |
|-----|------|-----------|------|
| 1 | VDD  | I | 3.3 V supply |
| 2 | NRST | I | Active-low reset, 100 kΩ internal pull-up; assert low ≥100 ms |
| 3 | RXD  | I | UART data input (host TX → module RX) |
| 4 | TXD  | O | UART data output (module TX → host RX) |
| 5 | NC   | - | Not connected |
| 6 | GND  | - | Ground |

> **Note on NSS.** The brief mentions an "NSS optional" line. The RYLR896
> exposes the SX1276 only through its on-module MCU + UART; **there is no
> external NSS/SPI chip-select pin** on the 6-pin module. Treat NSS as not
> applicable for RYLR896. (See `02_electrical.md`.)

## Document Map

1. [`01_overview.md`](01_overview.md) — Product description, features,
   applications, certifications.
2. [`02_electrical.md`](02_electrical.md) — Power, signal levels, pin
   descriptions, DC characteristics.
3. [`03_uart_interface.md`](03_uart_interface.md) — UART configuration, AT
   command framing.
4. [`04_at_commands.md`](04_at_commands.md) — Full AT command list with
   syntax and example responses.
5. [`05_at_responses.md`](05_at_responses.md) — Response format, `+OK`,
   `+ERR=N` codes, `+RCV=...` async events.
6. [`06_init_sequence.md`](06_init_sequence.md) — Power-on probe and
   initialization order.
7. [`07_appendix.md`](07_appendix.md) — Regulatory, range, environmental
   notes.

## Cross-References

- Parent avionics ICD index: [`../avionics.md`](../avionics.md)
- Source PDF: [`../../RYLR896_EN.pdf`](../../RYLR896_EN.pdf)
- Sibling sensor ICDs (e.g., IMU, GPS) live alongside this directory under
  `docs/icd/`.

## Revision History

| Date       | Rev | Description                                |
|------------|-----|--------------------------------------------|
| 2026-05-01 | A   | Initial conversion from REYAX RYLR896 EN datasheet (01-Nov-2021, 56312E37). |
