# GPS ICD — GlobalTop FGPMMOPA6H

Interface Control Document for the GlobalTop FGPMMOPA6H GPS Standalone Module
(MediaTek MT3339 chipset). This ICD organizes the vendor datasheet
(Revision V0A, 2012-01-31) by integration topic for use by Juno FSW.

- Source PDF: [GlobalTop FGPMMOPA6H Datasheet (V0A)](../../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf)
- Parent ICD index: [../avionics.md](../avionics.md)

## Key Specifications At-a-Glance

| Parameter | Value |
|-----------|-------|
| GPS chipset | MediaTek MT3339 |
| Frequency | L1, 1575.42 MHz |
| Channels | 66 search, 22 simultaneous tracking, up to 210 PRN |
| Sensitivity (tracking) | -165 dBm |
| Sensitivity (acquisition / cold start) | -148 dBm |
| Sensitivity (reacquisition / hot start) | -163 dBm |
| TTFF — hot start | 1 s typical |
| TTFF — warm start | 33 s typical |
| TTFF — cold start | 35 s typical |
| Position accuracy (no aid) | 3.0 m (50% CEP) |
| Position accuracy (SBAS) | 2.5 m (50% CEP) |
| Velocity accuracy (no aid) | 0.1 m/s |
| Velocity accuracy (SBAS) | 0.05 m/s |
| 1PPS timing accuracy | 10 ns typical |
| Update rate | 1 Hz default, up to 10 Hz |
| Maximum altitude | 18,000 m (60,000 ft) |
| Maximum velocity | 515 m/s (1000 knots) |
| Maximum acceleration | 4 G |
| Default UART baud rate | 9600 bps |
| Supply voltage VCC | 3.0 V to 4.3 V (typ. 3.3 V) |
| Backup voltage VBACKUP | 2.0 V to 4.3 V (typ. 3.0 V) |
| Current — acquisition | 25 mA typ. @ 3.3 V |
| Current — tracking | 20 mA typ. @ 3.3 V |
| Backup current | 7 µA typ. @ 3.0 V, 25 °C |
| Operating temperature | -40 °C to +85 °C |
| Package | 16 × 16 × 4.7 mm SMD |
| Weight | 4 g |
| Compliance | E911, RoHS, REACH |

## Document Map

| File | Topic |
|------|-------|
| [01_overview.md](01_overview.md) | Functional description, features, applications, block diagram |
| [02_electrical.md](02_electrical.md) | Power, signal levels, pin assignment and pin descriptions |
| [03_interface.md](03_interface.md) | UART protocol, baud rate, NMEA frame format, 1PPS, antenna I/O |
| [04_nmea_sentences.md](04_nmea_sentences.md) | GGA, GSA, GSV, RMC, VTG sentence field tables |
| [05_commands.md](05_commands.md) | PMTK and PGTOP command/status protocols |
| [06_timing.md](06_timing.md) | TTFF, update rate, 1PPS timing |
| [07_appendix.md](07_appendix.md) | Mechanical, environmental, regulatory, errata pointers |

## Features Beyond Scope of This ICD

The following datasheet features require GlobalTop firmware customization
services and are NOT used by Juno FSW in the FT1/FT2 flight configurations.
They are documented for completeness but flagged for downstream omission:

- EPO (AGPS) extended-prediction orbit data via FTP
- EASY self-generated orbit prediction
- AlwaysLocate periodic power-saving mode
- Embedded Logger (uses internal flash)
- Binary Mode, 1-Sentence Output, Geo-fencing, Last Position Retention
- Magnetic Variation field in RMC (vendor customization required)
- Magnetic Course reference in VTG (vendor customization required)
- RTCM DGPS streaming on Pin 14 (disabled by default)

See [07_appendix.md](07_appendix.md) for errata and unconvertible figure
references.

## Source Document

| Attribute | Value |
|-----------|-------|
| Title | GlobalTop FGPMMOPA6H Datasheet |
| Subtitle | GPS Module |
| Doc Type | Datasheet |
| Revision | V0A |
| Revision Date | 2012-01-31 |
| Author (vendor) | Delano (GlobalTop) |
| Pages | 37 |
| Source PDF | [GlobalTop FGPMMOPA6H Datasheet (V0A)](../../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf) |
