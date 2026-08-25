# GPS ICD — Interface Protocol

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Sections 2.5, 2.6, and 3.

Back to: [index.md](index.md)

## UART Serial Interface

The primary host interface is a UART TTL serial link with separate TX and RX
lines.

| Property              | Value                                  |
|-----------------------|-----------------------------------------|
| Lines                 | TX (Pin 9, output), RX (Pin 10, input)  |
| Logic level           | TTL (see [02_electrical.md](02_electrical.md)) |
| Default baud rate     | 9600 bps                                |
| Data bits             | 8                                       |
| Parity                | None                                    |
| Stop bits             | 1                                       |
| Flow control          | None                                    |
| Default content       | NMEA 0183 sentences on TX               |
| RX usage              | MTK NMEA commands, firmware update      |

The vendor datasheet does not state non-default baud rates explicitly in the
specification list, but baud-rate change commands are supported through the
PMTK command set (out of scope of this datasheet — see vendor "GPS Command
List"; not used by Juno FSW which runs at the 9600 baud default).

## NMEA Sentence Frame Format

All output sentences follow the standard NMEA 0183 frame:

```
$<TalkerID><Sentence>,<field1>,<field2>,...,<fieldN>*<CC><CR><LF>
```

Frame elements:

| Element     | Description |
|-------------|-------------|
| `$`         | Sentence start delimiter (ASCII 0x24) |
| Talker ID   | 2-character source: `GP` for GPS sentences, `PG` / `PMTK` for proprietary |
| Sentence ID | 3-character message type (e.g., `GGA`, `RMC`) |
| `,`         | Field delimiter (ASCII 0x2C) |
| Fields      | Comma-separated ASCII fields; empty fields are still delimited |
| `*`         | Checksum delimiter (ASCII 0x2A) |
| `CC`        | 2-character ASCII hex checksum (uppercase) |
| `<CR><LF>`  | Sentence terminator (ASCII 0x0D 0x0A) |

### Checksum Computation

The checksum is the **XOR of all bytes between (but not including) the `$`
and the `*`**, expressed as two uppercase hexadecimal ASCII characters.

```
checksum = 0
for each byte b in payload (between $ and *):
    checksum = checksum XOR b
emit two-hex-digit ASCII representation
```

### Output Sentence Set

By default the module emits the following NMEA sentences each fix interval:

| Sentence | Subject |
|----------|---------|
| GGA | Time, position, fix type |
| GSA | DOP and active satellites |
| GSV | Satellites in view |
| RMC | Recommended minimum navigation info |
| VTG | Course and speed over ground |

Field-level definitions: see [04_nmea_sentences.md](04_nmea_sentences.md).

### Proprietary Sentences

| Talker / Sentence | Direction | Purpose |
|-------------------|-----------|---------|
| `$PMTK<NNN>` | Host → Module | MTK NMEA command (configuration / control) |
| `$PGTOP,11,...` | Module → Host | Antenna Advisor status |

See [05_commands.md](05_commands.md) for command-level detail.

> **Errata cross-link:** the source PDF Table-12 mistakenly labels the
> Antenna Advisor format as "PGACK Data Format"; the actual emitted message
> ID is `$PGTOP`. See the
> [PGTOP/PGACK errata note in 05_commands.md](05_commands.md#antenna-status--pgtop-antenna-advisor)
> and the
> [errata table entry in 07_appendix.md](07_appendix.md#errata-and-vendor-customization-pointers).

## 1PPS Time Mark

| Property | Value |
|----------|-------|
| Pin | 13 |
| Logic level | 2.8 V CMOS |
| Jitter (typical) | 10 ns |
| Activation | After 3D fix |
| Synchronization | GPS time, start-of-second edge |

The 1PPS rising edge marks the start of a UTC second. Following the rising
edge, the corresponding NMEA sentences (e.g., GGA, RMC) for that second are
emitted on the UART. The exact UART-vs-1PPS latency depends on baud rate and
sentence length; the vendor datasheet does not specify this offset.

A power-on 1PPS output (i.e., before fix) can be enabled via custom firmware
(out of scope of this datasheet).

## 3D-FIX Indicator

| Property | Value |
|----------|-------|
| Pin | 5 |
| Logic | Square-wave heartbeat before 2D, low after 2D/3D |
| Pre-fix waveform | 1 s high / 1 s low (50% duty, 0.5 Hz) |
| Post-fix waveform | Continuous low |

The 3D-FIX line provides a hardware-observable fix indication independent of
NMEA parsing.

## External Antenna RF Input (EX_ANT, Pin 11)

| Property | Value |
|----------|-------|
| Frequency | 1575.42 MHz (L1) |
| Bias supply | Derived from VCC |
| Detection threshold | ≥ 4 mA antenna current → external antenna selected |
| Short-circuit response | Current limited to safe level (see [02_electrical.md](02_electrical.md)) |

The module switches automatically between the internal patch antenna and the
external antenna based on detection of antenna current ≥ 4 mA on EX_ANT.

## RTCM Differential GPS Input (Pin 14)

| Property | Value |
|----------|-------|
| Direction | Input |
| Logic level | TTL |
| Protocol | RTCM SC-104 |
| Default state | Disabled |

RTCM streaming is disabled by default and requires GlobalTop firmware enable.
**Not used by Juno FSW.**

## NRESET Reset Behavior

Driving NRESET (Pin 2) low resets the module. The vendor datasheet does not
specify minimum reset pulse width or post-reset boot timing.

After reset, if VBACKUP is unpowered, the module performs a cold start (see
[06_timing.md](06_timing.md)).

## Antenna Status Notification

When the Antenna Advisor feature is active, the module emits a `$PGTOP,11,V*CC`
sentence on TX whenever antenna state changes. See
[05_commands.md](05_commands.md).

> **Errata cross-link:** vendor source PDF Table-12 labels this format as
> "PGACK"; the actual emitted message ID is `$PGTOP`. See the
> [PGTOP/PGACK errata note in 05_commands.md](05_commands.md#antenna-status--pgtop-antenna-advisor)
> and the
> [errata table entry in 07_appendix.md](07_appendix.md#errata-and-vendor-customization-pointers).

## Update Rate

| Property | Value |
|----------|-------|
| Default | 1 Hz |
| Maximum | 10 Hz |
| SBAS-compatible maximum | 5 Hz |

The update rate determines how frequently the full default NMEA sentence set
is emitted. Update rate above 5 Hz disables SBAS augmentation. Update rate
configuration is performed via PMTK commands (out of scope of this
datasheet).

## Bandwidth Considerations at 9600 Baud

Empirical bandwidth budget (informational, not specified by vendor): a full
default sentence set (GGA + GSA + GSV × up to 3 + RMC + VTG) at 1 Hz fits
comfortably within 9600 bps (≈960 bytes/s). For 5 Hz operation the budget is
tight; for 10 Hz operation a higher baud rate (configured via PMTK) is
typically required. **Juno FSW operates at 1 Hz default and stays within
9600 baud.**
