# 07 — Appendix: Regulatory, Range, Environmental

[← Back to index](index.md)

This appendix contains regulatory text and environmental notes drawn from
the source datasheet, plus operational guidance on link budget and range.

## Regulatory Certifications

The RYLR896 is certified under:

- **FCC** — Contains TX FCC ID: `QLY-RYLR896` (source p.7)
- **NCC** (Taiwan) — `CCAN18LP0920T8` (source p.7)

### FCC Statement (verbatim, source p.7)

> This device complies with part 15 of the FCC Rules. Operation is subject
> to the following two conditions:
>
> (1) This device may not cause harmful interference, and
> (2) this device must accept any interference received, including
>     interference that may cause undesired operation.
>
> NOTE: This equipment has been tested and found to comply with the limits
> for a Class B digital device, pursuant to part 15 of the FCC Rules.
> These limits are designed to provide reasonable protection against
> harmful interference in a residential installation. This equipment
> generates, uses and can radiate radio frequency energy and, if not
> installed and used in accordance with the instructions, may cause
> harmful interference to radio communications. However, there is no
> guarantee that interference will not occur in a particular installation.
>
> If this equipment does cause harmful interference to radio or television
> reception, which can be determined by turning the equipment off and on,
> the user is encouraged to try to correct the interference by one or more
> of the following measures:
>
> - Reorient or relocate the receiving antenna.
> - Increase the separation between the equipment and receiver.
> - Connect the equipment into an outlet on a circuit different from that
>   to which the receiver is connected.
> - Consult the dealer or an experienced radio/TV technician for help.
>
> Changes or modifications not expressly approved by the party responsible
> for compliance could void the user's authority to operate the equipment.

### FCC End-Product Labelling (source p.7)

> The final end product must be labeled in a visible area with the
> following: `Contains TX FCC ID : QLY-RYLR896`. If the size of the end
> product is larger than 8x10 cm, then the FCC part 15.19 statement also
> has to be available on the label:
>
> "This device complies with Part 15 of FCC rules. Operation is subject to
> the following two conditions: (1) this device may not cause harmful
> interference and (2) this device must accept any interference received,
> including interference that may cause undesired operation."

### NCC Statement (Taiwan, verbatim, source p.7)

低功率電波輻射性電機管理辦法:

> 第十二條 經型式認證合格之低功率射頻電機，非經許可，公司、商號或使用者均不得擅自變更頻率、加大功率或變更原設計之特性及功能。
>
> 第十四條 低功率射頻電機之使用不得影響飛航安全及干擾合法通信；經發現有干擾現象時，應立即停用，並改善至無干擾時方得繼續使用。前項合法通信，指依電信法規定作業之無線電通信。低功率射頻電機須忍受合法通信或工業、科學及醫療用電波輻射性電機設備之干擾。

### Frequency Band Selection by Region

| Region        | Band       | Notes |
|---------------|------------|-------|
| EU (ETSI)     | 868 MHz    | RYLR896 supported via `AT+BAND=868000000` |
| US / Canada   | 915 MHz    | RYLR896 supported via `AT+BAND=915000000` |
| Asia (varies) | 868 / 915  | Verify local regulations before deployment |

The module's tunable range per the source datasheet (p.4) is **862 MHz to
1020 MHz**, but operators MUST restrict use to bands legal in their
jurisdiction.

## Range and Link Budget

From the source SPECIFICATION table (p.4):

- **Typical range:** 4.5 km
- **Maximum range:** 15 km — "Depend on RF parameter & environment."
- **Sensitivity:** -148 dBm (typ.)
- **TX power:** -4 to +15 dBm
- **Implied link budget:** 163 dB at +15 dBm TX / -148 dBm RX

Real-world range depends strongly on:

- Spreading factor (`SF`) and bandwidth (`BW`) — higher SF + lower BW
  trades data rate for range.
- Antenna orientation and polarisation alignment.
- Line-of-sight and Fresnel-zone clearance.
- Weather, ground reflections, and adjacent-channel interference.

For Juno FSW telemetry the recommended starting point is `SF=10, BW=125 kHz,
CR=4/5, Preamble=4` at +14 dBm TX power, which favours range over data
rate. `[recommendation, not from source PDF]`

## Environmental Specifications

From source p.4:

| Parameter | Value |
|-----------|-------|
| Operating temperature | -40 °C to +85 °C (typ. 25 °C) |
| EEPROM endurance      | 300 k erase/write cycles |
| Frequency accuracy    | ±2 ppm |
| Weight                | 3.07 g typ. |

The source PDF does not specify humidity, vibration, or shock limits in the
extracted text; consult REYAX directly if these are mission-critical.

## Antenna Note

The RYLR896 has an **integrated antenna** (source p.2). The mechanical
drawings on PDF pages 5–6 show the antenna keep-out zone (image-only,
not extracted). End-product designs MUST observe the keep-out region
specified by the original drawing to avoid detuning the antenna.

## Contact

- **Email:** `sales@reyax.com`
- **Web:** `http://reyax.com`

(Source p.7.)

## Source PDF

The original datasheet is available at:

- [`../../RYLR896_EN.pdf`](../../RYLR896_EN.pdf)

Document identifier: **56312E37**, dated **01-Nov-2021**.
Copyright © 2019, REYAX TECHNOLOGY CO., LTD.

[← 06 Init Sequence](06_init_sequence.md) | [Index](index.md)
