# GPS ICD — Overview

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Section 1 (Functional Description).

Back to: [index.md](index.md)

## Module Description

The FGPMMOPA6H is a 4th-generation stand-alone GPS module built around the
MediaTek MT3339 chipset. Headline characteristics:

- Ultra-high tracking sensitivity: **-165 dBm**
- Instant Time-to-First-Fix (TTFF)
- Low power consumption (acquisition 82 mW, tracking 66 mW)
- Compact form factor: **16 × 16 × 4.7 mm**

The module is a Patch-On-Top (POT) design with an integrated 15 × 15 × 2.5 mm
ceramic patch antenna and an additional embedded path for an external active
antenna. Automatic antenna switching and short-circuit protection are built in.

The module supports up to **210 PRN channels** with **66 search channels** and
**22 simultaneous tracking channels**.

## Highlights and Features

- Built-in 15 × 15 × 2.5 mm ceramic patch antenna on top of module
- Ultra-high sensitivity: -165 dBm (without patch antenna), up to 45 dB C/N of
  SVs in open-sky reception
- High update rate: up to 10 Hz (note 1)
- 12 multi-tone active interference cancellers (ISSCC 2011 Award, Section 26.5)
- High-accuracy 1-PPS timing (10 ns jitter typical)
- AGPS support for fast TTFF via EPO (Extended Prediction Orbit, 7-day / 14-day
  data sets) — note 2
- EASY: self-generated orbit prediction for instant fix (note 2)
- AlwaysLocate intelligent algorithm for power saving (note 2)
- Embedded logger function (note 2)
- Automatic antenna switching
- Antenna Advisor (status detection and notification)
- GlobalTop firmware customization services
- Consumption current at 3.3 V: acquisition 25 mA typ., tracking 20 mA typ.
- Compliance: E911, RoHS, REACH

Note 1: SBAS can only be enabled when update rate is ≤ 5 Hz.

Note 2: Some features require special firmware or commands programmed by the
customer. Refer to GlobalTop "GPS Command List" (out-of-scope for this ICD;
not used by Juno FSW).

## Supported Augmentation and Constellations

- Autonomous GPS
- QZSS (ranging)
- SBAS — WAAS, EGNOS, GAGAN, MSAS (default-on; cannot be combined with
  >5 Hz update rate)
- AGPS (EPO)

## Applications (vendor-stated)

- Handheld devices
- Tablet PC / PLB / MID
- M2M applications
- Asset management
- Surveillance

## System Block Diagram

The vendor datasheet includes a system block diagram on page 6. The diagram is
not reproduced here. **See Section 1.3 (page 6) of the source PDF for the
block diagram.**

## Multi-Tone Active Interference Canceller (MTAIC)

Because Wi-Fi, GSM/GPRS, 3G/4G, and Bluetooth radios are commonly co-located
on the host PCB, their RF harmonics can degrade GPS reception. The PA6H
multi-tone active interference canceller (MTAIC) rejects external RF
interference from other active components on the main board, improving GPS
reception capacity without hardware changes. PA6H can cancel up to **12
independent continuous-wave (CW) interference channels**.

## 1PPS

A pulse-per-second (1PPS) signal precisely indicates the start of a second.
The PA6H 1PPS output has typical jitter of **10 ns** and is used for precise
timekeeping (e.g., NTP synchronization). PA6H emits the high-accuracy 1PPS
signal after a 3D fix is acquired. A power-on 1PPS output is also available
when configured via custom firmware.

See [03_interface.md](03_interface.md) for 1PPS pin behavior and
[06_timing.md](06_timing.md) for timing accuracy details.

## AGPS — EPO

The AGPS feature uses Extended Prediction Orbit (EPO) data to accelerate TTFF.
EPO data is downloaded from a GlobalTop FTP server over the host network and
fed to the GPS engine, which uses it when satellite ephemeris is weak or
unavailable.

**Note: Juno FSW does not use EPO; this section is documented for vendor
fidelity only.**

## EASY (Self-Generated Orbit Prediction)

EASY is an embedded assist system for quick positioning. The GPS engine
automatically calculates and predicts orbit data (max 3 days) on power-up and
saves it to internal memory. The engine then uses this prediction for
positioning when satellite information is insufficient, improving fix
performance under indoor or urban conditions.

The vendor datasheet contains "Figure 1.12-1 EASY System operation" on page 8;
**see Section 1.7 (page 8) of the source PDF for the EASY operation diagram.**

## AlwaysLocate (Advance Power Periodic Mode)

An intelligent power-management algorithm that reduces GPS engine duty cycle.
The trade-off is degraded positioning accuracy: reported location stays within
< 50 m CEP. Activation requires custom firmware/commands.

## Embedded Logger

The embedded logger function does NOT require a host CPU/MCU or external
flash. The GPS engine uses internal flash (in the GPS chipset) to log fixes.

Logged data format: UTC, Latitude, Longitude, Valid, Checksum.

Maximum log duration: up to **2 days** under AlwaysLocate condition.

Note: Per-record size shrunk from 24 bytes to 15 bytes in this revision.

## Antenna Advisor

"Antenna Advisor" is a software-driven antenna status system available
exclusively on the PA6H. It detects and notifies the host of antenna state
through a proprietary NMEA-style protocol (see [05_commands.md](05_commands.md),
PGTOP).

States reported by Antenna Advisor:

- Active Antenna Shorted
- Using Internal Antenna
- Using Active Antenna

## Traceability — Module Capability to Juno FSW Use

| Vendor Capability | Juno FSW Use |
|-------------------|--------------|
| NMEA 0183 over UART TTL @ 9600 baud | Yes (primary fix interface) |
| 1PPS output | Future (timing alignment, FT2+) |
| Antenna Advisor PGTOP messages | Optional (informational) |
| EPO / EASY / AlwaysLocate | Not used |
| Embedded logger | Not used |
| RTCM DGPS input | Not used |
| Binary Mode | Not used (NMEA only) |
