# 04 — AT Command Set

[← Back to index](index.md)

> **Source-fidelity note.** The extracted source PDF does **not** contain the
> AT-command list (the relevant pages of the original document are graphical
> and were not captured by text extraction). The commands documented below
> are taken from the public REYAX RYLR896 AT-command reference, which is
> the authoritative implementation reference for the module.
>
> Every command in this section is flagged: `[from RYLR896 module reference,
> not extracted source PDF]`.

All commands and parameters are uppercase. Every command is terminated with
`<CR><LF>` (`\r\n`). Responses are described in
[`05_at_responses.md`](05_at_responses.md).

## Command Index

| # | Command          | Purpose                                  |
|---|------------------|------------------------------------------|
| 1 | `AT`             | Probe / liveness check                   |
| 2 | `AT+ADDRESS`     | Set / query module address (0–65535)     |
| 3 | `AT+NETWORKID`   | Set / query network group ID (0–16)      |
| 4 | `AT+BAND`        | Set / query RF carrier frequency         |
| 5 | `AT+PARAMETER`   | Set / query SF, BW, CR, preamble         |
| 6 | `AT+SEND`        | Transmit a data payload                  |
| 7 | `+RCV=` (async)  | Inbound data event (not a command)       |
| 8 | `AT+IPR`         | Set / query UART baud rate               |
| 9 | `AT+RESET`       | Software reset                           |
| 10| `AT+VER`         | Query firmware version                   |
| 11| `AT+UID`         | Query module unique ID                   |

## 1. `AT` — Probe

Liveness check. The module responds with `+OK` if it is ready to accept
further commands.

| Aspect    | Value |
|-----------|-------|
| Set form  | `AT\r\n` |
| Query form| (same — no parameters) |
| Response  | `+OK\r\n` |
| Example   | TX `AT\r\n` &nbsp; RX `+OK\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## 2. `AT+ADDRESS` — Module Address

Sets the 16-bit logical address used in `AT+SEND` and `+RCV` framing. Two
modules with different `ADDRESS` values may share the same `NETWORKID`.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+ADDRESS=<addr>\r\n` |
| Range      | `0` – `65535` |
| Query form | `AT+ADDRESS?\r\n` |
| Set resp.  | `+OK\r\n` |
| Query resp.| `+ADDRESS=<addr>\r\n` |
| Example    | TX `AT+ADDRESS=120\r\n` &nbsp; RX `+OK\r\n` |
|            | TX `AT+ADDRESS?\r\n` &nbsp; RX `+ADDRESS=120\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## 3. `AT+NETWORKID` — Network Group

All modules that wish to communicate must share the same `NETWORKID`.
Modules in different networks ignore each other's transmissions.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+NETWORKID=<id>\r\n` |
| Range      | `0` – `16` |
| Query form | `AT+NETWORKID?\r\n` |
| Set resp.  | `+OK\r\n` |
| Query resp.| `+NETWORKID=<id>\r\n` |
| Example    | TX `AT+NETWORKID=5\r\n` &nbsp; RX `+OK\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## 4. `AT+BAND` — RF Carrier Frequency

Sets the carrier frequency in Hz. Must be within the module's supported
range (**862 MHz – 1020 MHz**, source p.4).

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+BAND=<freq_hz>\r\n` |
| Typical    | `868000000` (EU), `915000000` (US/AS) |
| Query form | `AT+BAND?\r\n` |
| Set resp.  | `+OK\r\n` |
| Query resp.| `+BAND=<freq_hz>\r\n` |
| Example    | TX `AT+BAND=915000000\r\n` &nbsp; RX `+OK\r\n` |

The RYLR896 must operate within the regulatory band approved for the
deployment locale. See [`07_appendix.md`](07_appendix.md).

`[from RYLR896 module reference, not extracted source PDF]`

## 5. `AT+PARAMETER` — RF Modem Parameters

Sets four LoRa modem parameters in a single command.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+PARAMETER=<SF>,<BW>,<CR>,<PreambleLen>\r\n` |
| Query form | `AT+PARAMETER?\r\n` |
| Set resp.  | `+OK\r\n` |
| Query resp.| `+PARAMETER=<SF>,<BW>,<CR>,<PreambleLen>\r\n` |

Parameter ranges:

| Field        | Range / Codes                                     |
|--------------|---------------------------------------------------|
| `SF` (spreading factor) | 7 – 12                                |
| `BW` (bandwidth)        | 0 = 7.8 kHz, 1 = 10.4, 2 = 15.6, 3 = 20.8, 4 = 31.25, 5 = 41.7, 6 = 62.5, 7 = 125, 8 = 250, **9 = 500 kHz** |
| `CR` (coding rate)      | 1 = 4/5, 2 = 4/6, 3 = 4/7, 4 = 4/8 |
| `Preamble Length`       | 4 – 7 (typical 4)                  |

Example — 868 MHz EU "fast" profile:

```
TX: AT+PARAMETER=10,7,1,4\r\n
RX: +OK\r\n
```

Example query:

```
TX: AT+PARAMETER?\r\n
RX: +PARAMETER=10,7,1,4\r\n
```

`[from RYLR896 module reference, not extracted source PDF]`

## 6. `AT+SEND` — Transmit Payload

Transmits a data payload to a remote address on the same `NETWORKID`.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+SEND=<addr>,<len>,<data>\r\n` |
| `addr`     | Target address, 0 – 65535. `0` = broadcast |
| `len`      | Payload length in bytes, **0 – 240** |
| `data`     | Raw ASCII / binary payload, exactly `<len>` bytes |
| Response   | `+OK\r\n` (after on-air completion) |

Example:

```
TX: AT+SEND=120,5,HELLO\r\n
RX: +OK\r\n
```

The receiving module emits an asynchronous `+RCV=...` (see §7).

`[from RYLR896 module reference, not extracted source PDF]`

## 7. `+RCV=` — Asynchronous Receive Event

Not a host-issued command — emitted by the module when a frame is received.

| Aspect     | Value |
|------------|-------|
| Format     | `+RCV=<addr>,<len>,<data>,<RSSI>,<SNR>\r\n` |
| `addr`     | Sender's `AT+ADDRESS` |
| `len`      | Payload length, 0 – 240 |
| `data`     | Payload bytes (exactly `<len>`) |
| `RSSI`     | Received signal strength, dBm (signed integer) |
| `SNR`      | Signal-to-noise ratio, dB (signed integer) |

Example:

```
RX: +RCV=50,5,HELLO,-99,40\r\n
```

`[from RYLR896 module reference, not extracted source PDF]`

## 8. `AT+IPR` — UART Baud Rate

Sets the UART baud rate. The change takes effect on the next character; the
host must reconfigure its UART to match before issuing further commands.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+IPR=<baud>\r\n` |
| Allowed    | `300, 1200, 4800, 9600, 19200, 28800, 38400, 57600, 115200` |
| Query form | `AT+IPR?\r\n` |
| Set resp.  | `+IPR=<baud>\r\n` then `+OK\r\n` |
| Default    | **115200** |
| Example    | TX `AT+IPR=9600\r\n` &nbsp; RX `+IPR=9600\r\n` `+OK\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## 9. `AT+RESET` — Software Reset

Resets the module's internal MCU. Equivalent to a NRST pulse.

| Aspect     | Value |
|------------|-------|
| Set form   | `AT+RESET\r\n` |
| Response   | `+RESET\r\n` then `+READY\r\n` (after boot) |
| Example    | TX `AT+RESET\r\n` &nbsp; RX `+RESET\r\n` ... `+READY\r\n` |

The host MUST treat all stored parameters as still applied after reset
(the module persists `ADDRESS`, `NETWORKID`, `BAND`, `PARAMETER`, `IPR` in
EEPROM).

`[from RYLR896 module reference, not extracted source PDF]`

## 10. `AT+VER` — Firmware Version

Returns the module's firmware version string.

| Aspect     | Value |
|------------|-------|
| Query form | `AT+VER?\r\n` |
| Response   | `+VER=<string>\r\n` |
| Example    | TX `AT+VER?\r\n` &nbsp; RX `+VER=RYLR89C_V1.2.5\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## 11. `AT+UID` — Unique ID

Returns the module's factory-programmed unique identifier (hexadecimal
string).

| Aspect     | Value |
|------------|-------|
| Query form | `AT+UID?\r\n` |
| Response   | `+UID=<hex_string>\r\n` |
| Example    | TX `AT+UID?\r\n` &nbsp; RX `+UID=0123456789ABCDEF\r\n` |

`[from RYLR896 module reference, not extracted source PDF]`

## Persistence

The following parameters are stored in EEPROM and survive power-cycle:

- `AT+ADDRESS`
- `AT+NETWORKID`
- `AT+BAND`
- `AT+PARAMETER`
- `AT+IPR`

EEPROM endurance is **300 k erase/write cycles** (source p.4). Avoid writing
these parameters every boot if defaults already match the desired state.

[← 03 UART Interface](03_uart_interface.md) | [Index](index.md) | [Next: 05 AT Responses →](05_at_responses.md)
