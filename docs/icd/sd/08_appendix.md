# 08 — Appendix

[← Back to index](index.md)

> **Source note:** Authored from public SD SPI-mode protocol knowledge.
> The SD Physical Layer Simplified Specification PDF is not present in
> this repository.

## 8.1 FAT32 Filesystem Layer (Out of ICD Scope)

The Juno FSW mass-logging application stores flight data on the SD card
as files in a FAT32 filesystem. The FAT32 layer sits **on top of** the
SPI-mode block protocol described in this ICD: every FAT32 sector read
or write decomposes into one or more CMD17 / CMD24 transactions. The
SPI-mode protocol does not know or care about FAT32; from the
SPI-mode perspective the card is just a 512-byte-block-addressed array.

### 8.1.1 Why FAT32 specifically

| Reason | Note |
|--------|------|
| Universal cross-platform readability | Ground crew can read flight logs on any laptop without driver installation |
| Well-understood failure modes | FAT-corruption recovery procedures are public |
| Public-domain implementations exist | ELM-ChaN FatFs is widely used in embedded SD-card stacks |
| No journaling overhead | Write-amplification is bounded |

### 8.1.2 What FT1 needs from the SPI block layer

For the FAT32 layer to function correctly on top of this ICD, the SPI
driver must guarantee:

1. **512-byte block size** in every read and write — see
   [section 6.1](06_data_transfer.md#61-scope-of-this-section).
2. **Linear block address space** — every block index from 0 to
   capacity-1 (inclusive) is readable and writable.
3. **Atomic single-block writes** — a CMD24 either completes (DRT
   accepted, busy clears) or fails as a whole; partial-block writes
   are not visible.
4. **CCS-aware addressing** — the SPI driver internally translates
   logical block indices into the right argument format for SDHC/SDXC
   (block index) vs. SDSC (byte offset).

The SPI driver does **not** need to be aware of FAT32 boot sectors,
file allocation tables, directory entries, or cluster chains; those are
the FAT32 layer's responsibility.

### 8.1.3 Where the FAT32 layer documentation lives

Out of scope for this ICD. The FAT32 layer is documented in the
mass-logging app's design document (TBD location:
`docs/design/mlog_app/`).

## 8.2 Cross-References to In-Repo SD-Association PDFs

Two SD-Association PDFs live in `docs/`. They cover different parts of
the SD ecosystem from the SPI-mode protocol of this ICD and are
**not** referenced by the FT1 driver. They are retained for engineering
reference and possible future use.

### 8.2.1 PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf

| Field | Value |
|-------|-------|
| Path | [`../../PartA2_SD Host_Controller_Simplified_Specification_Ver4.20.pdf`](../../PartA2_SD%20Host_Controller_Simplified_Specification_Ver4.20.pdf) (note literal space in filename) |
| Subject | SDHCI host-controller register interface (the dedicated host hardware that drives the SD bus in SDIO mode) |
| Applicability to FT1 | **None.** The Pico 2 (RP2350) does not expose an SDHCI host controller. FT1 uses SPI mode over generic SPI0. |
| Possible future use | If a follow-on flight computer integrates an SDHCI peripheral, this PDF defines the register layout to be programmed. The SPI-mode ICD here would still apply if that board fell back to SPI mode. |

### 8.2.2 SDUC-Host-Implementation-Guideline_Ver1.00.pdf

| Field | Value |
|-------|-------|
| Path | [`../../SDUC-Host-Implementation-Guideline_Ver1.00.pdf`](../../SDUC-Host-Implementation-Guideline_Ver1.00.pdf) |
| Subject | Implementation hints for hosts that wish to support SDUC (Ultra Capacity, > 2 TB) cards |
| Applicability to FT1 | **None.** SDUC cards are explicitly out of scope (see [section 1.3](01_overview.md#13-supported-card-classes-for-ft1)). The FAT32 filesystem layer cannot address > 32 GB anyway. |
| Possible future use | If FT-N introduces a > 2 TB mass-storage class with an exFAT layer, this PDF would inform host-side capacity arithmetic. |

## 8.3 Source PDF Acquisition Note

The authoritative wire-protocol document for this ICD is the **SD
Physical Layer Simplified Specification** (most recent v9.0+ at the
time of authoring), available free of charge from the SD Association
at https://www.sdcard.org/. That PDF is **not** committed to this
repository.

If a future revision of this ICD needs to reconcile a corner case
(e.g., a specific reserved-bit pattern in OCR or CSD), the maintainer
should:

1. Download the current SD Physical Layer Simplified Specification.
2. Place it under `docs/` with a versioned filename (e.g.,
   `Physical_Layer_Simplified_Specification_VerX.YZ.pdf`).
3. Cite the relevant section number when amending this ICD.
4. Update each markdown file's source note to point to the in-repo
   copy.

## 8.4 Glossary

| Term | Definition |
|------|------------|
| ACMD | Application command — a command that must be preceded by CMD55 |
| CCS | Card Capacity Status (OCR bit 30) — 1 = SDHC/SDXC, 0 = SDSC |
| CID | Card Identification register (16 bytes, read with CMD10) |
| CMD | Command — a 6-byte SPI-mode message from host to card |
| CRC7 | 7-bit cyclic redundancy code used in command frames |
| CRC16 | 16-bit cyclic redundancy code used in data blocks |
| CSD | Card Specific Data register (16 bytes, read with CMD9) |
| DRT | Data Response Token — 1-byte status sent by card after a CMD24 write payload |
| HCS | Host Capacity Support (ACMD41 argument bit 30) |
| ICD | Interface Control Document |
| MISO / DO | Master In, Slave Out / Data Out (card to host) |
| MOSI / DI | Master Out, Slave In / Data In (host to card) |
| NCR | Number of Clocks for Response — host wait window for response start |
| OCR | Operating Conditions Register (32 bits, read with CMD58) |
| PSN | Product Serial Number (CID field) |
| R1, R1b, R2, R3, R7 | Response types — see [04_response_formats.md](04_response_formats.md) |
| SDHC | Secure Digital High Capacity (> 2 GB to 32 GB) |
| SDHCI | SD Host Controller Interface (host-side hardware peripheral) |
| SDIO | SD Input / Output (4-bit parallel bus mode, also covers SDIO peripheral cards) |
| SDSC | Secure Digital Standard Capacity (≤ 2 GB) |
| SDUC | Secure Digital Ultra Capacity (> 2 TB) |
| SDXC | Secure Digital Extended Capacity (> 32 GB to 2 TB) |
| SPI | Serial Peripheral Interface |
| VDD | Card supply voltage |
| VHS | Voltage Host Supplied (CMD8 argument field) |

## 8.5 Open Items

The following items are deferred to follow-on work:

| # | Item | Owner |
|---|------|-------|
| 1 | Define SW-REQ IDs for the SD subsystem and back-link them into each section of this ICD | Software Systems Engineer |
| 2 | Add a worked Mermaid sequence diagram for a complete CMD24 write transaction including the busy-poll phase | Software Systems Engineer |
| 3 | Add a CRC7 reference implementation appendix (the dynamic CRC7 path) | Software Systems Engineer |
| 4 | Cross-link this ICD from `docs/avionics.md` once the avionics doc is created | Avionics |
| 5 | Specify the SD-card-not-present detection strategy at the FSW level (likely a CMD13 ping cadence) | Software Lead |
