# SD Card SPI-Mode Protocol ICD

## Document Identification

| Field | Value |
|-------|-------|
| Document | SD Card SPI-Mode Protocol Interface Control Document |
| Subsystem | Mass Storage (SD Card on Pico 2 SPI0) |
| Mission | FT1 |
| Standard | IEEE 1016-style ICD, multi-file markdown |
| Status | Draft |

## Source Note

The authoritative reference for the SD SPI-mode wire protocol is the
**SD Physical Layer Simplified Specification** published by the SD Association.
**That PDF is not present in this repository.** The content in this ICD is
authored from the public, stable, and well-known SD SPI-mode standard
(CMD0 / CMD8 / CMD55+ACMD41 / CMD58 / CMD17 / CMD24 sequence is documented
identically across vendor application notes, Linux kernel `mmc_spi` driver
sources, ELM-ChaN FatFs port notes, and the SD Association simplified
specifications). Where SD-Association-only details (exact reserved bit
patterns, error-corner behaviors) might differ from this document, the SD
Physical Layer Simplified Spec shall be considered authoritative and shall
supersede this ICD.

The repository does contain two adjacent SD-Association documents that cover
the **host-controller (SDIO/SDHCI) hardware path**, not the SPI-mode protocol
the FSW uses:

- `../../PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf`
  (note the literal space in the filename) — SDIO host-controller register
  model. **Not used for FT1.**
- `../../SDUC-Host-Implementation-Guideline_Ver1.00.pdf` — SDUC capacity-class
  host hints. **Not used for FT1.**

FT1 uses the SD card in **SPI mode** driven directly by the Pico 2's SPI0
peripheral — not via an SDIO host controller. The SDIO/SDHCI documents are
retained only for historical reference and possible future host-controller
work; they are out of scope for this ICD.

## Scope

This ICD specifies the wire-level interface between the Pico 2 (RP2350) flight
computer and a removable SD card operating in **SPI mode**. It covers the
electrical signal mapping, command and response framing, the host-driven
initialization sequence, single-block read/write data transfer, and the
register layouts (CID, CSD) needed for capacity discovery.

The ICD is constrained to **SDHC and SDXC** cards up to 32 GB capacity for FT1
(the FAT32 filesystem layer used by the FSW is the limiting factor). SDSC (≤2
GB) is supported by the protocol described here but is not recommended for
flight. SDUC (>2 TB) is out of scope.

The FAT32 filesystem layer that sits on top of this protocol (used by the
mass-logging application) is **out of scope** for this ICD; it is mentioned
only contextually in the appendix.

## Table of Contents

| # | File | Topic |
|---|------|-------|
| 1 | [01_overview.md](01_overview.md) | SPI-mode positioning, supported cards, voltage, CS routing |
| 2 | [02_signal_interface.md](02_signal_interface.md) | Pin assignments, SPI mode 0, clock rates |
| 3 | [03_command_format.md](03_command_format.md) | 6-byte command frame layout |
| 4 | [04_response_formats.md](04_response_formats.md) | R1, R1b, R2, R3, R7 response bit layouts |
| 5 | [05_init_sequence.md](05_init_sequence.md) | Power-up to ready-state initialization sequence |
| 6 | [06_data_transfer.md](06_data_transfer.md) | CMD17 / CMD24 single-block read and write |
| 7 | [07_csd_cid.md](07_csd_cid.md) | CID and CSD register layouts, capacity calculation |
| 8 | [08_appendix.md](08_appendix.md) | FAT32 layer note, host-controller PDF cross-references |

## Cross-References

### Within this document

- [Overview](01_overview.md) — start here for context
- [Signal interface](02_signal_interface.md) — for hardware integration
- [Initialization sequence](05_init_sequence.md) — for driver bring-up

### Outside this document

- [`../avionics.md`](../avionics.md) — Pico 2 pin map, GP17/CS assignment, 3V3 power rail (link target may not yet exist; pin numbers in this ICD reflect the brief's stated assignment)
- [`../../PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf`](../../PartA2_SD%20Host_Controller_Simplified_Specification_Ver4.20.pdf) — SDIO host-controller spec (NOT used for FT1; reference only)
- [`../../SDUC-Host-Implementation-Guideline_Ver1.00.pdf`](../../SDUC-Host-Implementation-Guideline_Ver1.00.pdf) — SDUC host hints (NOT used for FT1; reference only)

## Conventions

- All multi-byte integers on the SD bus are **big-endian** (most significant
  byte first), unlike the Pico 2's native little-endian CPU. Drivers must
  byte-swap card-side fields explicitly.
- All bit numbering in this ICD follows SD-Association convention: bit 7 is
  the most significant bit of a byte, bit 0 is the least significant.
- "0x" prefix denotes hexadecimal; "0b" prefix denotes binary.
- Reserved bits in registers and responses shall be transmitted as zero by
  the host and shall be ignored on receipt unless explicitly noted.

## Revision History

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 0.1 | 2026-05-01 | Software Systems Engineer | Initial draft authored from public SD SPI-mode standard knowledge |
