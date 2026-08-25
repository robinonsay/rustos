# GPS ICD — Electrical

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Sections 2.3–2.8.

Back to: [index.md](index.md)

## Pin Configuration (Top View)

The vendor datasheet includes a pin configuration drawing on page 12 (Top
View). **See Section 2.3 (page 12) of the source PDF for the physical pin
layout drawing.** Pin numbers below correspond to that drawing.

## Pin Assignment

| Pin | Name    | I/O   | Description |
|-----|---------|-------|-------------|
| 1   | VCC     | PI    | Main DC power input |
| 2   | NRESET  | I     | Reset input, low active |
| 3   | GND     | P     | Ground |
| 4   | VBACKUP | PI    | Backup power input for RTC and navigation data retention |
| 5   | 3D-FIX  | O     | 3D-fix indicator |
| 6   | NC      | --    | Not connected |
| 7   | NC      | --    | Not connected |
| 8   | GND     | P     | Ground |
| 9   | TX      | O     | Serial data output for NMEA output (UART TTL) |
| 10  | RX      | I     | Serial data input for firmware update / commands (UART TTL) |
| 11  | EX_ANT  | I/PO  | External active antenna RF input. DC power from VCC supplied to antenna. |
| 12  | GND     | P     | Ground |
| 13  | 1PPS    | O     | 1PPS time-mark output, 2.8 V CMOS level |
| 14  | RTCM    | I     | Serial data input for DGPS RTCM data streaming |
| 15  | NC      | --    | Not connected |
| 16  | NC      | --    | Not connected |
| 17  | NC      | --    | Not connected |
| 18  | NC      | --    | Not connected |
| 19  | GND     | P     | Ground |
| 20  | NC      | --    | Not connected |

I/O legend: PI = Power Input, P = Power (Ground), I = Input, O = Output,
PO = Power Output.

## I/O Pin Descriptions

### VCC (Pin 1)

Main DC power supply. Voltage range: **3.0 V to 4.3 V**, typical **3.3 V**.
Ripple must be controlled below **50 mV peak-to-peak**.

### NRESET (Pin 2)

Active-low reset input. Driving this pin low resets the module. **If not
used, leave floating.**

### GND (Pins 3, 8, 12, 19)

Ground reference.

### VBACKUP (Pin 4)

Backup power for the GPS chipset internal RTC and last-known-position memory.
Voltage range: **2.0 V to 4.3 V**, typical **3.0 V**.

If VBACKUP is not powered, the module will perform a lengthy **cold start**
on every power-up because previous satellite information (almanac/ephemeris)
is not retained. **If not used, leave open.**

### 3D-FIX (Pin 5)

Fix-status indicator output. Default behavior:

- **Before 2D fix:** continuous square wave — 1 s high, 1 s low.
- **After 2D or 3D fix:** continuous low level.

Timing behavior is configurable via custom firmware (e.g., to wake a host
MCU). **If not used, leave floating.**

### TX (Pin 9)

UART transmitter, TTL level. Outputs NMEA sentences to host.

### RX (Pin 10)

UART receiver, TTL level. Accepts software commands and firmware update
streams from host.

### EX_ANT (Pin 11)

External active-antenna RF input with built-in DC bias derived from VCC
(recommended 3.3 V supply for external antennas). Detection logic:

- When ≥ 4 mA is drawn through this pin, the module recognizes an external
  antenna and switches reception to it.
- On short-circuit, the module limits drawn current to a safe level.

Antenna DC current limits (vendor-specified):

| VCC | EX_ANT current limit |
|-----|----------------------|
| 3.0 V | 25 mA |
| 3.3 V | 28 mA |
| 3.6 V | 31 mA |

### 1PPS (Pin 13)

Pulse-per-second output, **2.8 V CMOS level**, synchronized to GPS time after
3D fix. **If not used, leave floating.**

### RTCM (Pin 14)

DGPS RTCM data input, TTL level. **Disabled by default.** Contact GlobalTop
support to enable. **If not used, leave floating.**

### NC (Pins 6, 7, 15, 16, 17, 18, 20)

Not connected — leave open.

## Absolute Maximum Ratings

| Symbol   | Min | Typ | Max | Unit |
|----------|-----|-----|-----|------|
| VCC      | 3.0 | 3.3 | 4.3 | V    |
| VBACKUP  | 2.0 | 3.0 | 4.3 | V    |

**VCC must not exceed 4.3 VDC.**

## Operating Conditions

| Parameter                              | Condition         | Min | Typ | Max | Unit  |
|----------------------------------------|-------------------|-----|-----|-----|-------|
| Operation supply ripple voltage        | —                 | —   | —   | 50  | mVpp  |
| RX0 TTL high-level input voltage       | VCC = 3.0–4.3 V   | 2.0 | —   | VCC | V     |
| RX0 TTL low-level input voltage        | VCC = 3.0–4.3 V   | 0   | —   | 0.8 | V     |
| TX0 TTL high-level output voltage      | VCC = 3.0–4.3 V   | 2.4 | —   | 2.8 | V     |
| TX0 TTL low-level output voltage       | VCC = 3.0–4.3 V   | 0   | —   | 0.4 | V     |
| Current — acquisition (3.3 V, 1 Hz)    | —                 | —   | 25  | —   | mA    |
| Current — tracking (3.3 V, 1 Hz)       | —                 | —   | 20  | —   | mA    |
| Backup power consumption               | 3.0 V, 25 °C      | —   | 7   | —   | µA    |
| Operating temperature                  | —                 | -40 | —   | +85 | °C    |

## Host Interface Voltage Compatibility (Pico 2 Integration Note)

The TX output high level is 2.4 V min / 2.8 V max. The RP2350 (Pico 2) GPIO
input high threshold (Vih) is approximately 2.0 V at 3.3 V IO supply, so the
GPS TX line is directly compatible with the Pico 2 UART RX input.

The GPS RX input requires Vih ≥ 2.0 V; the Pico 2 GPIO output high at 3.3 V
satisfies this directly. **No level shifter required.**

## External Antenna Specification (Recommended)

| Characteristic     | Specification                       |
|--------------------|--------------------------------------|
| Polarization       | Right-hand circular polarized        |
| Frequency          | 1.57542 GHz ± 1.023 MHz              |
| Power supply       | 3 V to 3.6 V                         |
| DC current         | 4 mA to 20 mA at 3.3 V               |
| Total gain         | > +15 dBi (two-stage LNA)            |
| Output VSWR        | < 2.5                                |
| Impedance          | 50 Ω                                 |
| Noise figure       | < 1.5 dB                             |

The antenna must have a clear view of the sky and be positioned on a surface
level to the horizon for best results.

## Reference Design Notes

The vendor datasheet contains a reference schematic on page 26. **See
Section 4.1 (page 26) of the source PDF for the reference circuit.**

Vendor design notes:

1. Ferrite bead L1 added for power-noise reduction.
2. Bypass capacitors C1 and C2 must be placed close to the module.
3. Damping resistors R2, R3, R4 may be modified based on EMI requirements.
4. Contact GlobalTop sales for antenna-implementation support.
