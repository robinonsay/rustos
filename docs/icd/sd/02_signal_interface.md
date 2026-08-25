# 02 — Signal Interface

[← Back to index](index.md)

> **Source note:** Pin assignments are taken from the FT1 avionics
> definition (see `../avionics.md`). SPI mode and timing parameters
> are authored from public SD SPI-mode protocol knowledge; the SD
> Physical Layer Simplified Specification PDF is not present in this
> repository.

## 2.1 Signal Inventory

The SD card SPI-mode interface uses exactly four signals plus power and
ground. There are no card-to-host interrupt or card-detect signals carried
over the SPI bus itself — the FSW polls the card via CMD13 (SEND_STATUS)
when liveness checking is required.

| Signal | Direction (host view) | SD Pad # | Pico 2 Pin | Pico 2 GPIO | Notes |
|--------|----------------------|----------|------------|-------------|-------|
| `CS` (Chip Select, active low) | Host → Card | 1 | 22 | GP17 | Software-driven, not hardware-driven by SPI0 |
| `MOSI` / `DI` (Data In, host-to-card) | Host → Card | 2 | 25 | GP19 | Sampled by card on CLK rising edge |
| `VSS` (Ground) | — | 3, 6 | GND | — | Common ground with Pico 2 |
| `VDD` (Power) | Host → Card | 4 | 3V3 | — | 3.3 V supply |
| `CLK` / `SCLK` (Serial Clock) | Host → Card | 5 | 24 | GP18 | Continuous clock for both directions |
| `MISO` / `DO` (Data Out, card-to-host) | Card → Host | 7 | 21 | GP16 | Driven by card on CLK falling edge |

The Pico 2 SPI peripheral used is **SPI0**. Pin assignments above place CLK,
MOSI, and MISO on the SPI0 hardware function for those GPIOs. CS is on a
GPIO that is **not** the SPI0 hardware-CS pin so that the firmware can hold
CS asserted across multi-byte transactions without the peripheral
auto-deasserting between bytes (see [section 1.5](01_overview.md#15-chip-select-routing)).

## 2.2 Pinout Diagram (Host View)

The card's micro-SD form factor presents pads in the following order, with
SPI-mode role labels:

```
            +------------------------------------+
            |                                    |
SD pad #:   8     7     6     5     4     3   2   1
SPI role:  RSV   DO    GND   CLK   VDD  GND  DI  CS
            |                                    |
Pico GPIO:  -    GP16   -   GP18   3V3   -  GP19 GP17
Pico pin:   -     21    -    24    3V3   -   25   22
            |                                    |
            +------------------------------------+
                       (contacts up)
```

Pad 8 ("RSV" — reserved) is unconnected in SPI mode.

## 2.3 SPI Mode and Polarity

The SD card requires **SPI Mode 0**:

| Parameter | Value | Meaning |
|-----------|-------|---------|
| `CPOL` (Clock Polarity) | 0 | Clock idles low between transactions |
| `CPHA` (Clock Phase) | 0 | Data sampled on the rising edge, shifted on the falling edge |
| Bit order | MSB-first | Bit 7 of every byte is shifted out first |
| Word size | 8 bits | All transfers are byte-aligned |

Using any other CPOL/CPHA combination produces undefined card behavior.

## 2.4 Clock Rate Plan

The SD card requires the host to use a **slow clock during initialization**
and may then be switched to a higher operating clock once the card is in the
data-transfer state.

| Phase | Clock Rate | Justification |
|-------|-----------|---------------|
| Power-up dummy-clock training (≥ 74 cycles with CS high, MOSI high) | 100 kHz to 400 kHz | Card internal regulator stabilization; SD spec requires the host to provide ≥ 74 clocks before the first command at a frequency between 100 kHz and 400 kHz |
| Initialization (CMD0 through CMD58 / ACMD41 ready) | ≤ 400 kHz | Required by SD spec for legacy compatibility with low-end cards |
| Post-initialization data transfer (CMD17, CMD24, etc.) | ≤ 25 MHz | SD specification ceiling for SPI mode; vendor cards may guarantee 25 MHz, some legacy cards only 20 MHz |
| Juno FSW recommended operating clock | 10–12 MHz | Headroom under the 25 MHz ceiling, fits cleanly into RP2350 SPI0 dividers from the 125 MHz peripheral clock |

The driver shall switch the SPI0 baud-rate divisor only **after** the card
returns ready (R1 = 0x00 from the final ACMD41) and **before** issuing the
first data-transfer command. Issuing CMD0 at >400 kHz produces undefined
behavior on some cards and shall be avoided.

## 2.5 Signal Timing

The SD card is synchronous to CLK in both directions:

```
CLK    ___       ___       ___       ___
      |   |     |   |     |   |     |   |
______|   |_____|   |_____|   |_____|   |____    (Mode 0: idle low)
      ^ rising  ^ rising  ^ rising  ^ rising
      |         |         |         |
      sample    sample    sample    sample

MOSI   ===X=========X=========X=========X===     (host updates on falling)
          B7        B6        B5        B4

MISO   ===X=========X=========X=========X===     (card updates on falling)
          B7        B6        B5        B4
```

There is no separate "data ready" handshake. Whenever CS is low and CLK is
toggling, the card is actively shifting MISO and sampling MOSI. The host
discovers card-side framing (e.g., the start of an R1 response, the start
of a data token) by reading MISO byte-by-byte and looking for the bytes
defined in [04_response_formats.md](04_response_formats.md) and
[06_data_transfer.md](06_data_transfer.md).

## 2.6 Idle Behavior of MOSI

When the host has no command to send but is clocking the bus to read a
response or wait on busy, **MOSI shall be held high (0xFF)**. This is a
firm requirement of the SD SPI-mode protocol: the card uses MOSI = 0xFF as
the "no command" idle pattern and may misinterpret a stuck-low MOSI as the
start bit of a malformed command.

## 2.7 CS Hold and Release

| Action | CS State | Notes |
|--------|----------|-------|
| Pre-init dummy-clock training | High | ≥ 74 clocks with MOSI = 0xFF |
| Issuing a command + reading response | Low (asserted) | Held for the entire command + response window |
| Between commands when no transaction is active | High (deasserted) | Host should send 8 dummy clocks (one byte of 0xFF) after deasserting CS to allow the card to release MISO |
| During CMD17 read (command → data token → 512 B + CRC) | Low | Held continuously |
| During CMD24 write (command → R1 → data token + 512 B + CRC → data response → busy) | Low | Held continuously, including during the busy-wait poll |

Some cards require an extra eight clocks **after** CS goes high to fully
release the MISO line; the driver shall always send one trailing 0xFF byte
with CS high after every transaction.

## 2.8 Cabling and Layout Constraints

- The SD card socket shall sit within 50 mm trace length of the Pico 2 SPI0
  pins. Longer traces require source-termination resistors.
- A 100 nF ceramic decoupling capacitor shall be placed within 5 mm of the
  card's VDD pad.
- Signal traces shall be impedance-matched to roughly 50 Ω single-ended
  for clock rates above 10 MHz.

These are layout-level constraints carried in the avionics document; this
ICD records them only to clarify what the SPI-mode protocol assumes about
the physical channel.
