# GPS ICD — Appendix

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Sections 2.1, 2.2, 2.7–2.9, 5,
6, and 7.

Back to: [index.md](index.md)

## Mechanical

| Parameter | Value |
|-----------|-------|
| Package | SMD module |
| Outline dimensions | 16 × 16 × 4.7 mm (tolerance ± 0.2 mm) |
| Patch antenna | 15 × 15 × 2.5 mm ceramic, top-mounted |
| Weight | 4 g |
| Pin count | 20 (see [02_electrical.md](02_electrical.md)) |

The vendor datasheet contains a mechanical-dimension drawing on page 10 and
a recommended PCB pad layout on page 11. **See Sections 2.1 and 2.2 of the
source PDF for the dimensional drawings (units mm, tolerance ± 0.1 mm for
the pad layout).**

## Environmental

| Parameter | Value |
|-----------|-------|
| Operating temperature | -40 °C to +85 °C |
| Storage shelf life | 6 months from bag-seal date |
| Storage condition | < 30 °C, < 60% relative humidity, non-condensing |
| Moisture sensitivity | Pre-bake required before reflow |

### Moisture Sensitivity and Floor Life

GlobalTop GPS modules are moisture-sensitive and must be pre-baked before
solder reflow. After pre-baking, the module's "floor life" is **72 hours**
in normal factory conditions (temperature 23 °C, 60 ± 5% RH). Reflow must
complete within 72 hours of pre-bake.

If the 72-hour window is exceeded, the module may suffer reflow damage
(cracks, SMD-pad delamination, "popcorn" effect).

### Pre-Bake Profiles

| Profile | Time | Temperature |
|---------|------|-------------|
| Standard pre-bake | 8–12 hours | 60 °C |
| Pre-reflow (Section 6) | 6 hours | 60 ± 5 °C |
| Pre-reflow (alternate) | 4 hours | 70 ± 5 °C |

Tray temperature must not exceed 100 °C. After baking, cool the tray to
≤ 35 °C before handling to prevent deformation.

**Repeat-bake limit:** cumulative bake time at > 90 °C and ≤ 125 °C must
not exceed 96 hours. Bake temperatures > 125 °C are not allowed. Excessive
baking causes oxidation and intermetallic growth on the SMD terminations,
degrading solderability.

## Reflow Soldering Temperature Profile

| Stage | Specification |
|-------|---------------|
| Average ramp-up rate (25 → 150 °C) | 3 °C/sec max. |
| Average ramp-up rate (217 °C to peak) | 3 °C/sec max. |
| Preheat | 175 ± 25 °C, 60 to 120 seconds |
| Time above 217 °C | 60 to 150 seconds |
| Peak temperature | 250 °C +0/-5, 20 to 40 seconds |
| Ramp-down rate | 6 °C/sec max. |
| Time from 25 °C to peak | 8 minutes max. |
| Process | Pb-free only |

The vendor profile diagram is on page 32 of the source PDF. **See Section
6.1 (page 32) of the source PDF for the reflow temperature/time graph.**

### Reflow Process Notes (vendor cautions)

1. Pre-bake required before SMT reflow.
2. Solder paste usage must follow first-in-first-out rotation.
3. Temperature and humidity must be controlled in the SMT line and storage
   area (recommended: 23 °C, 60 ± 5% RH).
4. Vacuum mouthpiece must support module weight to prevent positional shift
   on placement.
5. Eyesight check for module positional offset before reflow.
6. Reflow profile data must be measured and recorded before each SMT run.
7. For double-sided PCBA processes, run the GPS module on the **second
   pass only** to avoid repeated reflow exposure. Contact GlobalTop in
   advance if first-pass mounting is required.
8. Do not invert the module — patch-antenna side must face up during
   reflow.

### Manual Soldering

| Parameter | Specification |
|-----------|---------------|
| Soldering iron tip temperature | < 380 °C |
| Contact time | < 3 seconds |

## Packaging

- Modules placed individually on tray; trays stacked and packaged together.
- Each pack includes:
  - Two desiccant packs.
  - One moisture-color-coded humidity card.
- Each pack placed in an antistatic (PE) bag.
- Bagged packs placed in two levels of cardboard cartons.
- Outside the antistatic bag: moisture-sensitive-device caution label.
- Acceptable RH on opening: ≤ 30%.

The vendor datasheet shows packaging photos on pages 28 and 29 of the source
PDF. **See Section 5.2 (pages 28–29) of the source PDF for packaging
photographs.**

## ESD Handling

GPS modules are Electrostatic Sensitive Devices (ESD). Required precautions
for handling, especially around the patch antenna and the EX_ANT (RF input)
pin:

- First point of contact must be between local GND and PCB GND (unless
  galvanic GND coupling exists).
- GND must be connected before working on the EX_ANT pin.
- Do not touch the mounted patch antenna (electrostatic discharge through
  the RF input is possible).
- Do not contact charged capacitors, coax cables, soldering irons, or other
  charge-storing materials while working on EX_ANT.
- Use an ESD-safe soldering-iron tip when soldering EX_ANT.

## Regulatory and Compliance

| Standard | Status |
|----------|--------|
| E911 | Compliant |
| RoHS | Compliant |
| REACH | Compliant |

## Errata and Vendor-Customization Pointers

| Item | Pointer |
|------|---------|
| Magnetic Variation field (RMC) | Empty by default; requires GlobalTop customization |
| Magnetic Course field (VTG) | Empty by default; requires GlobalTop customization |
| RTCM DGPS streaming (Pin 14) | Disabled by default; contact GlobalTop support |
| EPO / AGPS | Out of scope; refer to GlobalTop website |
| EASY orbit prediction | Custom firmware required |
| AlwaysLocate periodic mode | Custom firmware required |
| Embedded logger | Custom firmware required |
| Antenna Advisor message ID | Source PDF Table-12 mistakenly labels format as "PGACK"; actual sentence is `$PGTOP,11,...` |
| Power-on 1PPS (before fix) | Custom firmware required |
| Binary mode, 1-sentence output, geo-fencing, last-position retention | Custom firmware services |
| Full PMTK command catalog | Not in datasheet — see GlobalTop "GPS Command List" |
| Block diagram (Section 1.3, p. 6) | Not reproduced — see source PDF |
| EASY operation diagram (Fig 1.12-1, p. 8) | Not reproduced — see source PDF |
| Mechanical drawing (Section 2.1, p. 10) | Not reproduced — see source PDF |
| PCB pad layout (Section 2.2, p. 11) | Not reproduced — see source PDF |
| Pin configuration drawing (Section 2.3, p. 12) | Not reproduced — see source PDF |
| Reference schematic (Section 4.1, p. 26) | Not reproduced — see source PDF |
| Reflow profile graph (Section 6.1, p. 32) | Not reproduced — see source PDF |

## Vendor Contact

| Field | Value |
|-------|-------|
| Company | GlobalTop Technology Inc. |
| Address | No. 16 Nan-ke 9 th Rd., Science-Based Industrial Park, Tainan 741, Taiwan, R.O.C. |
| Tel | +886-6-5051268 |
| Fax | +886-6-5053381 |
| Web | www.gtop-tech.com |
| Email | sales@gtop-tech.com |

## Source Document Reference

| Field | Value |
|-------|-------|
| Title | GlobalTop FGPMMOPA6H Datasheet |
| Subtitle | GPS Module |
| Doc Type | Datasheet |
| Revision | V0A |
| Revision Date | 2012-01-31 |
| Vendor Author | Delano (GlobalTop) |
| Pages | 37 |
| Source PDF | [../../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf](../../GlobalTop-FGPMMOPA6H-Datasheet-V0A.pdf) |
