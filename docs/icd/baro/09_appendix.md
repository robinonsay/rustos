# 09 — Appendix

[← Back to Baro ICD index](index.md)

## 9.1 Environmental Specifications

| Parameter | Min | Typ | Max | Unit | Notes |
|-----------|-----|-----|-----|------|-------|
| Operating temperature                | -40 | 25 | +85  | °C  | T_OP |
| Storage temperature                  | -40 | —  | +125 | °C  | T_STG |
| Calibrated pressure range            | 50  | —  | 110  | kPa | Specified accuracy |
| Operational pressure range           | 20  | —  | 110  | kPa | Functional |
| Maximum applied pressure (stress)    | —   | —  | 500  | kPa | P_max |
| Long-term drift (1 year)             | —   | —  | ±0.1 | kPa | |
| Board-mount drift (post-reflow)      | —   | —  | ±0.15| kPa | |
| Pressure noise @ 1× OSR              | —   | 19 | —    | Pa RMS | |
| Pressure noise @ 128× OSR            | —   | 1.5| —    | Pa RMS | |

## 9.2 Typical Curves and Figures

The source datasheet contains typical-performance figures that are
not transcribed here (they are graphical and are best read from the
PDF directly). See:

- **Figure 1** — Block diagram. (Source PDF, page 3.)
- **Figure 2** — Pin connections / typical decoupling. (Page 4.)
- **Figure 3** — I2C slave timing diagram. (Page 7.)
- **Figure 4** — I2C bus transmission signals. (Page 8.)
- **Figure 5** — Polling/Interrupt quick-start flow. (Page 10.)
- **Figure 6** — FIFO quick-start flow. (Page 11.)
- **Figure 7** — Mode transition diagram (OFF/STANDBY/ACTIVE). (Page 13.)
- **Figure 8** — Interrupt controller block diagram. (Page 34.)
- **Figure 9** — INT1/INT2 OR-routing logic. (Page 35.)
- **Figure 10** — Recommended PCB landing pattern. (Page 37.)
- **Figures 11–12** — Tape-and-reel dimensions and orientation. (Page 40.)

For all numerical and graphical detail, refer to the source PDF:
[`../../1893_datasheet.pdf`](../../1893_datasheet.pdf).

## 9.3 Errata and Known Quirks

The Rev 3.0 datasheet does not contain a separate errata sheet. The
following ambiguities or quirks were observed while converting the
datasheet to this ICD; FSW developers should be aware:

1. **Pressure data tables labeled "OUT_T_xxx".** In datasheet
   Section 7.1.3 the byte-construction text refers to
   `OUT_T_MSB / OUT_T_CSB / OUT_T_LSB` where the context clearly means
   `OUT_P_MSB / OUT_P_CSB / OUT_P_LSB`. This is a typo in the source.
   Use the field positions in
   [`07_data_format.md`](07_data_format.md) §7.1–§7.2.

2. **OUT_P_DELTA_LSB bit-name typo.** Datasheet Table 21 labels the
   register's bits `TDD3..TDD0` (temperature names) but the surrounding
   text and address (`0x09`) confirm this is the *pressure* delta LSB.
   Treat as `PDD3..PDD0`.

3. **Q-format wording.** The datasheet describes temperature as
   "Q12.4". This refers to total bit-width (12) and fraction (4). The
   integer portion is 8 bits including sign — i.e., a more
   conventional notation is **Q8.4 in 12 bits**. This ICD uses the
   conventional notation.

4. **Bit-3 reserved in DR_STATUS.** Datasheet Table 12 shows bit 3 as
   the PTDR flag and bit 0 as reserved; some quick-start example code
   in Figure 5 uses the mask `0x08` to test PTDR. Both descriptions
   are self-consistent with bit-3 = PTDR.

5. **Pin 1 and Figure 2 vs Table 1.** The datasheet's Figure 2
   ("Pin Connections") visually swaps INT1/INT2 placement on the
   diagram. Table 1 is authoritative: INT2 = pin 5, INT1 = pin 6.

6. **F_MODE state-change rule.** F_MODE cannot be switched directly
   between `01` (circular) and `10` (stop-on-full); the host must
   first write `00` (disabled) and then the new mode.

7. **OST behavior depends on SBYB.** When SBYB = 1, OST is "sticky"
   — it does not auto-clear and must be cleared and re-set to fire
   again. When SBYB = 0, OST auto-clears after one conversion. This
   asymmetry is required reading for any one-shot driver code.

## 9.4 Integration Checklist for Juno FSW

- [ ] VDD decoupling: 100 nF + 10 µF on VDD; 100 nF on CAP; 100 nF on VDDIO.
- [ ] I2C pull-ups sized for bus capacitance (start at 4.7 kΩ).
- [ ] INT1 and/or INT2 pulled up to VDDIO if open-drain mode used.
- [ ] WHO_AM_I check returns 0xC4 before any other transaction.
- [ ] Soft reset issued at boot.
- [ ] Altimeter + 128× OSR is the FT1 default; QNH (BAR_IN) override
      mechanism present in the baro library API.
- [ ] OFF_H ground calibration callable from CLI.
- [ ] Driver supports both polling and INT2-driven retrieval; FT1
      uses INT2.
- [ ] Conversion logic in driver matches Q-format described in
      [`07_data_format.md`](07_data_format.md).
- [ ] Bus transactions never block longer than the TDM slice.

## 9.5 Glossary

| Term | Meaning |
|------|---------|
| ACTIVE   | Power state in which the analog block is on and the device is acquiring data. |
| ADC      | Analog-to-digital converter (24-bit in this device). |
| ALT      | CTRL_REG1 bit selecting Altimeter (1) vs Barometer (0) mode. |
| AGL      | Above ground level (Juno-specific, computed by FSW). |
| BAR_IN   | 16-bit register holding sea-level reference pressure for altitude calc. |
| F_DATA   | FIFO read pointer register (0x0E). |
| F_RD     | "FIFO read" — multi-byte burst read mode (auto-incrementing pointer). |
| F_MODE   | F_SETUP[7:6]: FIFO mode select. |
| INT1/2   | Programmable interrupt output pins. |
| LDO      | Low-dropout regulator (internal to MPL3115A2). |
| LGA      | Land Grid Array (package style). |
| NVM      | Non-volatile memory (factory trim storage). |
| OSR      | Oversampling ratio (CTRL_REG1.OS[2:0]). |
| OST      | One-shot trigger bit (CTRL_REG1.OST). |
| OUT_P    | Pressure or altitude output register triplet (0x01..0x03). |
| OUT_T    | Temperature output register pair (0x04..0x05). |
| PDR/TDR  | Pressure / Temperature data ready flags. |
| PTDR     | Pressure OR Temperature data ready flag. |
| QNH      | Aviation term for sea-level reference pressure. |
| Q-format | Fixed-point notation: Qm.n = m integer bits, n fractional bits. |
| RAW      | CTRL_REG1.RAW = 1 → uncompensated ADC output. |
| SBYB     | "Standby bit". CTRL_REG1.SBYB: 0 = STANDBY, 1 = ACTIVE. |
| STANDBY  | Power state with analog off, registers writable. |
| ST       | CTRL_REG2.ST[3:0]: auto-acquisition step in 2^ST seconds. |
| T_ON     | Turn-on time from STANDBY to first valid data. |
| US Std Atm 1976 | NASA atmospheric model used internally for altitude. |
| WHO_AM_I | Device identification register (0x0C, value 0xC4). |

## 9.6 Revision History (of source datasheet)

| Rev | Date    | Notes (summary) |
|-----|---------|-----------------|
| 0   | 06/2011 | Initial release. |
| 1   | 12/2011 | Added bullets to ordering info; renamed registers ARM→TGT (0x16..0x1A); changes to bit names in 0x12 and 0x2A; min/max accuracy updates. |
| 2   | 04/2012 | Added 6.6.7 Data Ready and 6.6.8 FIFO Event sections; deleted some I2C timing rows; Figure 5/6 updates. |
| 2.1 | 05/2012 | Renamed ordering part MPL3115A2T1 → MPL3115A2R1. |
| 2.2 | 07/2012 | Updated Table 59 (8 → 18 ms entry); fixed OFF_H register title. |
| 3.0 | 12/2013 | Promoted to "Technical Information"; tightened resolution wording (0.1 m); updated Section 4 quick-start. |

## 9.7 Source Reference

- Freescale Semiconductor, **Document Number MPL3115A2**, *Data
  Sheet: Technical Data, Rev 3.0, 12/2013*.
- Local copy: [`../../1893_datasheet.pdf`](../../1893_datasheet.pdf)
  (Adafruit product 1893 packaging).

## 9.8 Cross-References

- [`index.md`](index.md) — TOC + key specs.
- [`../avionics.md`](../avionics.md) — parent avionics ICD.

[← Back to Baro ICD index](index.md)
