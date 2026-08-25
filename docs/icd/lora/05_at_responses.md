# 05 — AT Response Format and Error Codes

[← Back to index](index.md)

> **Source-fidelity note.** Response framing and error-code values below are
> from the public RYLR896 AT-command reference, not from the extracted
> source PDF text. Tagged inline with `[from RYLR896 module reference,
> not extracted source PDF]`.

## General Response Form

Every module-to-host line begins with `+`, ends with `<CR><LF>`, and has the
form:

```
+<TAG>[=<value>[,<value>[,...]]]<CR><LF>
```

The host parser SHOULD treat any line not beginning with `+` as noise (or
as a command echo, if echo is enabled).

## Standard Response Tags

| Tag                            | Meaning |
|--------------------------------|---------|
| `+OK`                          | Command accepted, no return data |
| `+ERR=<n>`                     | Command rejected, error code `n` |
| `+<CMD>=<value>`               | Query response (e.g., `+ADDRESS=120`) |
| `+RCV=<addr>,<len>,<data>,<RSSI>,<SNR>` | Asynchronous receive event |
| `+RESET`                       | Emitted on receipt of `AT+RESET` |
| `+READY`                       | Emitted after the module has finished booting |
| `+IPR=<baud>`                  | Echoed before `+OK` when `AT+IPR` is set |
| `+VER=<string>`                | Firmware version |
| `+UID=<hex>`                   | Module unique ID |

## Error Codes (`+ERR=N`)

`[All error codes below from RYLR896 module reference, not extracted source PDF]`

| Code   | Meaning |
|--------|---------|
| `+ERR=1`  | No `<CR><LF>` at end of AT command |
| `+ERR=2`  | The head of the AT command is not `AT` |
| `+ERR=4`  | Unknown command |
| `+ERR=5`  | Data length specified by `AT+SEND` does not match the actual data length |
| `+ERR=10` | Transmit time exceeded |
| `+ERR=12` | CRC error on receive |
| `+ERR=13` | TX data exceeds 240 bytes |
| `+ERR=14` | Unknown error |
| `+ERR=15` | Unknown error |

> Codes `3`, `6`–`9`, `11` are reserved / not emitted by current firmware.

### Recommended Host Handling

| Error  | Recovery action |
|--------|-----------------|
| `+ERR=1` | Verify the host serial driver appends `\r\n`; retransmit the command. |
| `+ERR=2` | Buffer corruption; flush UART, reissue probe `AT`. |
| `+ERR=4` | Check command spelling / firmware version (`AT+VER?`). |
| `+ERR=5` | Length field disagrees with payload; recompute and retransmit. |
| `+ERR=10`| Air-time too long for current `AT+PARAMETER`; reduce SF or payload size. |
| `+ERR=12`| Link layer CRC failure; transient — application-level retry. |
| `+ERR=13`| Payload too large; segment to ≤240 byte chunks. |
| `+ERR=14`/`15` | Treat as transient; reissue `AT+RESET` if persistent. |

## Asynchronous `+RCV=` Detail

```
+RCV=<addr>,<len>,<data>,<RSSI>,<SNR>\r\n
```

| Field   | Type   | Notes |
|---------|--------|-------|
| `addr`  | uint16 | Sender address (0 – 65535). 0 indicates broadcast. |
| `len`   | uint8  | Payload length in bytes, 0 – 240. |
| `data`  | bytes  | Exactly `<len>` payload bytes. May contain commas; consumers MUST use `len` to delimit, not split-on-comma. |
| `RSSI`  | int    | Received signal strength in dBm (signed). |
| `SNR`   | int    | Signal-to-noise ratio in dB (signed). |

### Parsing Caveat

Because the payload itself may contain commas, a naïve `split(',')` parser
will break for arbitrary binary data. The recommended approach:

1. Read the line up to `\r\n`.
2. Locate the first comma — fields before it are `+RCV=<addr>`.
3. Locate the second comma — between the first and second commas is `<len>`.
4. Read exactly `<len>` bytes after the second comma — that is `<data>`.
5. The remaining `,<RSSI>,<SNR>` follows the payload.

## Boot / Power-On Banner

After a power-on or `AT+RESET`, the module emits:

```
+RESET\r\n
+READY\r\n
```

The host MUST wait for `+READY` (or a fixed boot delay) before issuing
further commands. `[from RYLR896 module reference, not extracted source PDF]`

## Response Timing

- Control commands (`AT`, `AT+ADDRESS`, `AT+NETWORKID`, `AT+PARAMETER`,
  `AT+BAND`, query forms): response within ~100 ms.
- `AT+SEND`: response is delivered **after the on-air transmission
  completes**. Air time depends on `SF`, `BW`, `CR`, and payload length and
  may exceed 1 s for long-range profiles. Hosts MUST NOT issue another
  command until the trailing `+OK` (or `+ERR=N`) is received.
- `AT+RESET`: `+RESET` immediately, `+READY` after the internal boot
  (~100 ms).

[← 04 AT Commands](04_at_commands.md) | [Index](index.md) | [Next: 06 Init Sequence →](06_init_sequence.md)
