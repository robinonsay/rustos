# GPS ICD — NMEA Output Sentences

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Section 3.1.

Back to: [index.md](index.md)

This file enumerates the supported NMEA 0183 output sentences with their
field-level definitions. Frame structure (start delimiter, checksum,
terminator) is defined in [03_interface.md](03_interface.md).

Proprietary status sentences (notably `$PGTOP,11,...` for Antenna Advisor)
are out of scope of this file and are documented in
[05_commands.md](05_commands.md#antenna-status--pgtop-antenna-advisor).
The vendor source PDF Table-12 labels the `$PGTOP` format as "PGACK" — see
the [PGTOP/PGACK errata cross-link in 05_commands.md](05_commands.md#antenna-status--pgtop-antenna-advisor)
and the
[errata table entry in 07_appendix.md](07_appendix.md#errata-and-vendor-customization-pointers).

## Supported Sentences (Summary)

| Sentence | Description |
|----------|-------------|
| GGA | Time, position, and fix-type data |
| GSA | GPS receiver operating mode, active satellites used in the position solution, and DOP values |
| GSV | Number of satellites in view, satellite IDs, elevation, azimuth, and SNR values |
| RMC | Time, date, position, course, and speed (Recommended Minimum Navigation Information) |
| VTG | Course and speed information relative to the ground |

---

## GGA — Global Positioning System Fixed Data

Time, position, and fix-related data.

Example:

```
$GPGGA,064951.000,2307.1256,N,12016.4438,E,1,8,0.95,39.9,M,17.8,M,,*65
```

### GGA Field Definitions

| # | Name | Example | Units | Description |
|---|------|---------|-------|-------------|
| 1 | Message ID | `$GPGGA` | — | GGA protocol header |
| 2 | UTC Time | `064951.000` | hhmmss.sss | UTC time of fix |
| 3 | Latitude | `2307.1256` | ddmm.mmmm | Latitude |
| 4 | N/S Indicator | `N` | — | `N` = north, `S` = south |
| 5 | Longitude | `12016.4438` | dddmm.mmmm | Longitude |
| 6 | E/W Indicator | `E` | — | `E` = east, `W` = west |
| 7 | Position Fix Indicator | `1` | — | See *Position Fix Indicator* table below |
| 8 | Satellites Used | `8` | count | Range 0 to 14 |
| 9 | HDOP | `0.95` | — | Horizontal Dilution of Precision |
| 10 | MSL Altitude | `39.9` | meters | Antenna altitude above/below mean sea level |
| 11 | Units | `M` | — | Units of antenna altitude (meters) |
| 12 | Geoidal Separation | `17.8` | meters | Geoid separation |
| 13 | Units | `M` | — | Units of geoid separation (meters) |
| 14 | Age of Diff. Corr. | (empty) | seconds | Null when DGPS not used |
| 15 | (DGPS Station ID) | (empty) | — | Null when DGPS not used |
| 16 | Checksum | `*65` | — | XOR checksum |
| 17 | Terminator | `<CR><LF>` | — | End of message |

### Position Fix Indicator

| Value | Description |
|-------|-------------|
| 0 | Fix not available |
| 1 | GPS fix |
| 2 | Differential GPS fix |

---

## GSA — GNSS DOP and Active Satellites

Operating mode, active satellites in the position solution, and DOP values.

Example:

```
$GPGSA,A,3,29,21,26,15,18,09,06,10,,,,,2.32,0.95,2.11*00
```

### GSA Field Definitions

| # | Name | Example | Description |
|---|------|---------|-------------|
| 1 | Message ID | `$GPGSA` | GSA protocol header |
| 2 | Mode 1 | `A` | See *Mode 1* table below |
| 3 | Mode 2 | `3` | See *Mode 2* table below |
| 4–15 | Satellite Used | `29`, `21`, …, (empty) | SVs on Channels 1 through 12 |
| 16 | PDOP | `2.32` | Position Dilution of Precision |
| 17 | HDOP | `0.95` | Horizontal Dilution of Precision |
| 18 | VDOP | `2.11` | Vertical Dilution of Precision |
| 19 | Checksum | `*00` | XOR checksum |
| 20 | Terminator | `<CR><LF>` | End of message |

### Mode 1

| Value | Description |
|-------|-------------|
| `M` | Manual — forced to operate in 2D or 3D mode |
| `A` | Automatic — allowed to switch 2D/3D automatically |

### Mode 2

| Value | Description |
|-------|-------------|
| 1 | Fix not available |
| 2 | 2D fix (< 4 SVs used) |
| 3 | 3D fix (≥ 4 SVs used) |

---

## GSV — GNSS Satellites in View

Number of satellites in view with per-SV elevation, azimuth, and SNR.

Multiple GSV sentences may be emitted per fix; the **Number of Messages** and
**Message Number** fields tie the multi-sentence sequence together.

Example (3-message sequence):

```
$GPGSV,3,1,09,29,36,029,42,21,46,314,43,26,44,020,43,15,21,321,39*7D
$GPGSV,3,2,09,18,26,314,40,09,57,170,44,06,20,229,37,10,26,084,37*77
$GPGSV,3,3,09,07,,,26*73
```

### GSV Field Definitions

| # | Name | Example | Units | Description |
|---|------|---------|-------|-------------|
| 1 | Message ID | `$GPGSV` | — | GSV protocol header |
| 2 | Number of Messages | `3` | — | Range 1 to 3 (depending on SV count) |
| 3 | Message Number | `1` | — | Range 1 to *Number of Messages* |
| 4 | Satellites in View | `09` | count | Total SVs in view |
| 5 | Satellite ID (Ch 1) | `29` | — | PRN, range 1 to 32 |
| 6 | Elevation (Ch 1) | `36` | degrees | Maximum 90 |
| 7 | Azimuth (Ch 1) | `029` | degrees | True heading, range 0 to 359 |
| 8 | SNR / C/N₀ (Ch 1) | `42` | dB-Hz | Range 0 to 99, null when not tracking |
| … | Channels 2, 3, 4 | … | — | Same four-field pattern, up to 4 SVs per sentence |
| Last | Checksum | `*7D` | — | XOR checksum |
| Last+1 | Terminator | `<CR><LF>` | — | End of message |

Each GSV sentence reports up to **4 satellites** (16 satellite-related
fields). With 9 satellites visible the receiver emits 3 GSV sentences as in
the example.

---

## RMC — Recommended Minimum Navigation Information

Time, date, position, course, and speed.

Example:

```
$GPRMC,064951.000,A,2307.1256,N,12016.4438,E,0.03,165.48,260406,3.05,W,A*2C
```

### RMC Field Definitions

> **Field-numbering note:** This ICD splits **Magnetic Variation** into two
> separate fields — field 11 (numeric magnitude) and field 12 (E/W
> indicator) — to align with the canonical NMEA 0183 RMC layout used
> elsewhere in Juno FSW documentation. The source vendor PDF presents these
> as a single combined "Magnetic Variation" field. Both representations
> describe the same wire encoding (the two on-the-wire fields exist either
> way, separated by a comma); the renumbering is purely an editorial
> choice for cross-PDF consistency.

| # | Name | Example | Units | Description |
|---|------|---------|-------|-------------|
| 1 | Message ID | `$GPRMC` | — | RMC protocol header |
| 2 | UTC Time | `064951.000` | hhmmss.sss | UTC time of fix |
| 3 | Status | `A` | — | `A` = data valid, `V` = data not valid |
| 4 | Latitude | `2307.1256` | ddmm.mmmm | Latitude |
| 5 | N/S Indicator | `N` | — | `N` = north, `S` = south |
| 6 | Longitude | `12016.4438` | dddmm.mmmm | Longitude |
| 7 | E/W Indicator | `E` | — | `E` = east, `W` = west |
| 8 | Speed Over Ground | `0.03` | knots | Speed |
| 9 | Course Over Ground | `165.48` | degrees | True heading |
| 10 | Date | `260406` | ddmmyy | UTC date |
| 11 | Magnetic Variation | `3.05` | degrees | Requires GlobalTop customization |
| 12 | Magnetic Variation E/W | `W` | — | `E` = east, `W` = west; requires customization |
| 13 | Mode | `A` | — | See *RMC Mode* table below |
| 14 | Checksum | `*2C` | — | XOR checksum |
| 15 | Terminator | `<CR><LF>` | — | End of message |

### RMC Mode

| Value | Description |
|-------|-------------|
| `A` | Autonomous mode |
| `D` | Differential mode |
| `E` | Estimated mode |

---

## VTG — Course and Speed Over Ground

Course and speed information relative to the ground.

Example:

```
$GPVTG,165.48,T,,M,0.03,N,0.06,K,A*37
```

### VTG Field Definitions

| # | Name | Example | Units | Description |
|---|------|---------|-------|-------------|
| 1 | Message ID | `$GPVTG` | — | VTG protocol header |
| 2 | Course (True) | `165.48` | degrees | Measured heading |
| 3 | Reference (True) | `T` | — | Indicates true heading |
| 4 | Course (Magnetic) | (empty) | degrees | Numeric magnetic-course value; emitted only on units with GlobalTop customization service applied (default firmware emits this field as empty) |
| 5 | Reference (Magnetic) | `M` | — | Literal reference indicator (`M` = magnetic). Always emitted regardless of customization; only the field-4 numeric value is gated by customization |
| 6 | Speed (knots) | `0.03` | knots | Measured horizontal speed |
| 7 | Units (knots) | `N` | — | Knots |
| 8 | Speed (km/h) | `0.06` | km/h | Measured horizontal speed |
| 9 | Units (km/h) | `K` | — | Kilometers per hour |
| 10 | Mode | `A` | — | Same values as RMC Mode (`A`/`D`/`E`) |
| 11 | Checksum | `*06` | — | XOR checksum |
| 12 | Terminator | `<CR><LF>` | — | End of message |

---

## Field Encoding Notes

- **Latitude format:** `ddmm.mmmm` — 2-digit degrees, 2-digit minutes, 4
  decimal places of minutes.
- **Longitude format:** `dddmm.mmmm` — 3-digit degrees, 2-digit minutes, 4
  decimal places of minutes.
- **UTC Time format:** `hhmmss.sss` — 2-digit hours, 2-digit minutes,
  2-digit seconds, 3 decimal places of seconds.
- **Date format (RMC):** `ddmmyy` — 2-digit day, 2-digit month, 2-digit
  year (last two digits of calendar year).
- **Empty fields:** an empty field is still bracketed by commas
  (e.g., `,,`). Parsers must accept null fields and not require numeric
  defaults.
- **Magnetic Variation (RMC) and Magnetic Course (VTG)** are populated
  only on units with GlobalTop customization service applied; default
  firmware emits these as empty.
