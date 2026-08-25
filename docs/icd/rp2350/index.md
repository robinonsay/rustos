# RP2350 Interface Control Document (ICD)

## Scope

This ICD captures the FT1-relevant chapters of the RP2350 datasheet, tailored
for the Juno FSW driver code. Content is restricted to the five subsystems the
flight software touches directly during FT1:

1. **Clocks** — system, peripheral, and reference clock generators (Chapter 8)
2. **GPIO** — pad control and function-select muxing (Chapter 9, Bank 0 focus)
3. **UART** — PL011 controller used for the GPS link (Section 12.1)
4. **I2C** — DW_apb_i2c controller used for the IMU/baro buses (Section 12.2)
5. **SPI** — PL022 SSP controller used for SD card and other peripherals (Section 12.3)

Sections of the datasheet that do not directly inform driver behaviour
(verification methodology, internal block test logic, full timing waveforms,
QSPI-only register dumps) have been omitted or summarised. Where a timing
diagram is referenced, the reader is directed to the source PDF.

## Source Document

The canonical reference is the official Raspberry Pi RP2350 datasheet, stored
at `../../rp2350-datasheet.pdf` (relative to this index file).

| Chapter | Datasheet Pages | ICD Sub-directory |
|---------|-----------------|-------------------|
| 8. Clocks | 510-583 | [`clocks/`](clocks/index.md) |
| 9. GPIO | 584-700 | [`gpio/`](gpio/index.md) |
| 12.1 UART | 958-979 | [`uart/`](uart/index.md) |
| 12.2 I2C | 980-1042 | [`i2c/`](i2c/index.md) |
| 12.3 SPI | 1043-1090 | [`spi/`](spi/index.md) |

## Cross-References

- Avionics overview (board-level pin assignments, power tree): [`../avionics.md`](../avionics.md)
- Source PDF: [`../../rp2350-datasheet.pdf`](../../rp2350-datasheet.pdf)

## Hardware Context (FT1)

| Peripheral | Instance | Use |
|------------|----------|-----|
| UART0/UART1 | one of two PL011 | GPS NMEA stream (typ. 9600 baud) |
| I2C0/I2C1   | one of two DW_apb_i2c | IMU + baro sensors (Fast Mode 400 kb/s) |
| SPI0/SPI1   | one of two PL022 | SD card, optional radio control |
| GPIO Bank 0 | up to 30 pins (QFN-60) | All peripheral pin muxing |

## Conventions

- Register tables use the format: `Bits | Field | Type | Reset | Description`.
- Register access types: `RW` read-write, `RO` read-only, `WC` write-clear,
  `RWF` read/write FIFO.
- All register offsets are relative to the peripheral base address, listed at
  the top of each chapter's register file.
- All examples reference the pico-sdk; line numbers cited are from the SDK
  copies extracted in the source datasheet.

## Document Structure

Each peripheral subdirectory contains an `index.md` (chapter-level table of
contents) plus numbered chunk files for each section. Files are kept under
500 lines each per project constraint.
