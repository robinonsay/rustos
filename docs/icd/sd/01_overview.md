# 01 — Overview

[← Back to index](index.md)

> **Source note:** Content authored from public SD SPI-mode protocol
> knowledge. The SD Physical Layer Simplified Specification PDF is not
> present in this repository.

## 1.1 Purpose

This section positions the SD card SPI-mode interface within the Juno FSW
mass-storage stack and identifies the supported card classes, electrical
characteristics, and host-side resources committed to the interface.

## 1.2 SPI Mode vs. SDIO Mode

An SD card supports two mutually exclusive command modes after power-up:

| Mode | Wire Count | Host Side | Used by Juno FSW |
|------|-----------|-----------|------------------|
| SDIO / SD bus mode (1-bit or 4-bit) | 4–6 (CMD, DAT0..DAT3, CLK) | SDHCI host controller (a dedicated peripheral) | No |
| SPI mode | 4 (CS, CLK, MOSI, MISO) | Generic SPI master | **Yes — FT1** |

Selection is **automatic and irreversible per power cycle**: if the host
asserts CS low while issuing the first CMD0 (GO_IDLE_STATE) the card enters
SPI mode and remains there until the next power cycle. If CS is high during
the first CMD0 the card enters SD bus mode.

Juno FSW uses SPI mode because:

1. The Pico 2 (RP2350) does not expose an SDHCI host-controller peripheral;
   its SD-card support is realized over the generic SPI0 peripheral.
2. SPI mode trades raw throughput (which it caps near 25 MHz × 1 bit) for a
   dramatically simpler state machine and a wire format that can be driven
   by any SPI master without specialized DMA descriptors.
3. The flight workload (mass logging at < 1 MB/s sustained) is comfortably
   below the SPI-mode throughput ceiling.

The two PDFs in `docs/` (PartA2 host controller, SDUC host implementation
guideline) describe the SDIO/SDHCI host hardware path. They are **not
applicable** to the SPI-mode protocol described in the rest of this ICD.

## 1.3 Supported Card Classes for FT1

| Class | Capacity Range | Block Addressing | Supported for FT1 |
|-------|---------------|------------------|-------------------|
| SDSC (Standard Capacity) | up to 2 GB | Byte-addressed | Tolerated; not recommended |
| SDHC (High Capacity) | > 2 GB to 32 GB | 512-byte block-addressed | **Recommended** |
| SDXC (Extended Capacity) | > 32 GB to 2 TB | 512-byte block-addressed | Supported up to 32 GB only (FAT32 limit) |
| SDUC (Ultra Capacity) | > 2 TB | 512-byte block-addressed | **Not supported** |

The 32 GB upper bound for FT1 is imposed by the FAT32 filesystem layer
(out of scope for this ICD; see [appendix](08_appendix.md)) and not by the
SPI-mode protocol itself, which can address any SDHC/SDXC card.

The protocol behaves identically on SDHC and SDXC cards: both report
capacity via the v2.0 CSD layout (see [07_csd_cid.md](07_csd_cid.md)) and
both accept block-indexed addresses for CMD17/CMD24. The driver shall
treat SDHC and SDXC as a single "block-addressed" code path.

SDSC support requires an additional CMD16 (SET_BLOCKLEN) step during
initialization to force a 512-byte block size, because SDSC cards default
to byte-addressing with a card-defined block length. Drivers may omit
this step when the [init sequence](05_init_sequence.md) confirms the card
is SDHC/SDXC via the CCS bit of the OCR register.

## 1.4 Electrical Summary

| Parameter | Value |
|-----------|-------|
| Supply voltage (VDD) | 3.3 V (nominal) |
| Logic levels | 3.3 V CMOS (LVCMOS33), Pico 2 native |
| Card-side pull-ups | Required on CS, MOSI, MISO per SD spec; the card provides internal weak pull-ups but a 10 kΩ external pull-up on MISO is conventional |
| Inrush at power-up | The SD card draws a transient current of up to ~100 mA during power-on; the rocket's 3V3 rail must support this without browning out the Pico 2 |
| Power-on settling | Host shall wait at least 1 ms after the 3V3 rail crosses 2.7 V before issuing the first clock edge |

The SD card and the Pico 2 share the same 3V3 rail. There is no level
shifter between them.

## 1.5 Chip-Select Routing

The chip-select line for the SD card is dedicated to one Pico 2 GPIO and is
**not multiplexed** with any other SPI peripheral on SPI0. This guarantees
the SD card can be deselected (CS high) without disturbing other SPI traffic
and vice versa.

| Pin (Pico 2) | GPIO | Direction | Role |
|--------------|------|-----------|------|
| Pin 22 | GP17 | Output (host) | SD card CS / SS, active-low |

The CS line is driven directly by the host firmware, not by the SPI0
peripheral's hardware-managed CS pin, so that the firmware can hold CS
asserted across multi-byte transactions (e.g., command + R1 wait + data
token + 512-byte payload + CRC + busy-wait) without the SPI0 hardware
deasserting between bytes.

See [`../avionics.md`](../avionics.md) for the full Pico 2 pin map and
the rationale for choosing GP17 specifically.

## 1.6 Out of Scope

The following topics are explicitly out of scope for this ICD:

- The FAT32 filesystem layer (volume boot record, FAT, directory entries,
  cluster chains) — see [appendix](08_appendix.md).
- The SDIO 4-bit bus interface and SDHCI host-controller register set —
  see the PartA2 PDF referenced in the [appendix](08_appendix.md).
- SD card UHS-I / UHS-II / SD Express signaling.
- Card-side write-protect-tab handling — the protocol carries no
  notion of the mechanical write-protect slider; that is a host-board
  concern.
- Power-removal lifetime and write-endurance budgeting.

## 1.7 Traceability

This section will be updated when SW-REQ IDs for the SD subsystem are
assigned. Placeholder mapping:

- Card class support → mass-storage capacity requirements (TBD)
- 3.3 V single-supply operation → power architecture requirements (TBD)
- GP17 CS assignment → avionics pin-map requirements (TBD)
