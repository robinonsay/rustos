# GPS ICD — Commands and Status Protocols

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Sections 3.2 and 3.3.

Back to: [index.md](index.md)

This file documents the proprietary command and status sentence sets
referenced in the datasheet. The datasheet itself only documents one PMTK
command and one PGTOP status message in detail; full command-list coverage
is delegated to the separate vendor "GPS Command List" document, which is
**not part of this ICD**.

## Frame Structure

All proprietary commands and status messages reuse the NMEA 0183 frame
format (start delimiter `$`, comma-separated payload, `*`, two-character
hex checksum, `<CR><LF>`). See [03_interface.md](03_interface.md) for the
generic frame definition and checksum algorithm.

## Antenna Status — `$PGTOP` (Antenna Advisor)

Direction: **Module → Host** (status notification).

This sentence is emitted by the Antenna Advisor feature. It is only relevant
to configurations using an external active antenna.

### Frame

```
$PGTOP,11,<value>*<CC><CR><LF>
```

Example:

```
$PGTOP,11,3*6F
```

### Field Definitions

| # | Name | Example | Description |
|---|------|---------|-------------|
| 1 | Message ID | `$PGTOP` | Proprietary protocol header |
| 2 | Command ID | `11` | Function type — Antenna Advisor status |
| 3 | Value | `3` | Antenna status code (see table below) |
| 4 | Checksum | `*6F` | XOR checksum |
| 5 | Terminator | `<CR><LF>` | End of message |

### Antenna Status Values

| Value | Meaning |
|-------|---------|
| 1 | Active antenna shorted |
| 2 | Using internal antenna |
| 3 | Using active antenna |

The vendor datasheet labels the format table "PGACK Data Format" in the
source PDF (Table-12); this is a vendor typo — the message ID is `$PGTOP`,
not `$PGACK`. The example sentence and field semantics are unambiguous.

---

## MTK NMEA Command Protocols — `$PMTK`

Direction: **Host → Module** (command).

The PMTK command family is used to configure and control the GPS engine
(cold/warm/hot start, baud rate, update rate, output sentence selection,
etc.). The datasheet documents only one command in full detail and refers
the reader to the separate "GPS Command List" for the complete catalog.

### Documented Command — PMTK_CMD_COLD_START (Packet Type 103)

| Field | Value |
|-------|-------|
| Packet Type | 103 |
| Symbol | `PMTK_CMD_COLD_START` |
| Direction | Host → Module |
| Behavior | Cold start: do not use Time, Position, Almanacs, or Ephemeris data at restart |

Example sentence:

```
$PMTK103*30<CR><LF>
```

This command instructs the GPS engine to discard cached time, position,
almanac, and ephemeris on restart. After issuing PMTK103 the module
performs a full cold start (see [06_timing.md](06_timing.md) for cold-start
TTFF).

### Other PMTK Commands

The vendor datasheet does **not** enumerate the remainder of the PMTK
command set. Commands typically referenced in vendor application notes (and
common to MT3339-based modules) include — for context only, not authoritative
in this ICD:

- PMTK101 — hot start
- PMTK102 — warm start
- PMTK103 — cold start (documented above)
- PMTK104 — full cold start (factory reset)
- PMTK220 — set position-fix interval (update rate)
- PMTK251 — set baud rate
- PMTK314 — select output sentences and rates
- PMTK313 — enable/disable SBAS
- PMTK353 — set GNSS search mode

**Authoritative reference:** GlobalTop "GPS Command List" (separate vendor
document, not included in this ICD).

**Juno FSW note:** the FT1/FT2 configuration runs the module at default
settings (1 Hz, 9600 baud, default sentence set). No PMTK commands are
issued at runtime. The PMTK channel is therefore unused by Juno FSW for
the planned missions.

---

## Firmware Customization Services

Section 3.4 of the source datasheet states that GlobalTop offers firmware
customization services that may add or modify the command and output set,
including:

- Binary mode
- 1-sentence output
- Geo-fencing
- Last position retention

These customizations require pre-flashed firmware variants and are
**out of scope of this ICD**.

---

## Acknowledgement and Response Behavior

The datasheet does not describe a standardized PMTK acknowledgement format.
In practice, MT3339-family modules emit `$PMTK001,<cmd>,<result>*<CC>`
acknowledgements, but this format is not documented in the source PDF and
must be confirmed against the vendor "GPS Command List" before relying on
it. Treat as **vendor-confirmation-required** when integrating new commands.
