# 03 — UART Interface and AT Framing

[← Back to index](index.md)

## UART Configuration

The RYLR896 host interface is a **3.3 V CMOS asynchronous UART**. Configuration
values are taken from the SPECIFICATION table on page 4 of the source PDF
(row "Baud rate ... 8, N, 1").

| Parameter      | Value (default)                  | Notes |
|----------------|----------------------------------|-------|
| Baud rate      | **115200 bps**                   | Range 300 – 115200, set by `AT+IPR`. `[default value from RYLR896 module reference, not extracted source PDF]` |
| Data bits      | **8**                            | Source p.4 ("8, N, 1") |
| Parity         | **None**                         | Source p.4 ("8, N, 1") |
| Stop bits      | **1**                            | Source p.4 ("8, N, 1") |
| Flow control   | **None** (no RTS/CTS lines)      | Module exposes only RXD / TXD |
| Logic level    | **3.3 V CMOS**                   | See `02_electrical.md` |
| Idle line      | High                             | Standard UART |

## Wiring

```
   Host MCU                         RYLR896
  ┌─────────┐                      ┌─────────┐
  │   TXD   ├─────────────────────►│  RXD (3)│
  │   RXD   │◄─────────────────────┤  TXD (4)│
  │  GPIO_x ├─────────────────────►│ NRST (2)│  (open-drain or push-pull)
  │   3V3   ├─────────────────────►│  VDD (1)│
  │   GND   ├─────────────────────┤  GND (6)│
  └─────────┘                      └─────────┘
```

The host's UART **TX** pin connects to the module's **RXD** (pin 3); the
module's **TXD** (pin 4) connects to the host's **UART RX**. NRST is optional
but recommended for deterministic recovery.

## AT Command Framing

The control protocol on the UART link is a line-oriented ASCII AT-command
set. Every host-to-module command and every module-to-host response is
terminated by a CR/LF pair.

`[The framing details below — terminator, payload format, byte-count
constraint — are from the RYLR896 module reference and are not present in
the extracted source PDF text. They follow the standard REYAX AT
implementation.]`

### Frame Structure (host → module)

```
AT[+CMD[=<param1>[,<param2>[,...]]]]<CR><LF>
```

Where:

- `AT` — fixed prefix, uppercase.
- `+CMD` — command keyword (uppercase), present for all commands except the
  bare probe `AT`.
- `=<params>` — optional parameter list, comma-separated, no whitespace.
- `<CR><LF>` — terminator: 0x0D 0x0A. Some hosts send LF-only successfully;
  CR/LF is the canonical form.

Examples:

```
AT\r\n
AT+ADDRESS=120\r\n
AT+SEND=50,5,HELLO\r\n
```

### Frame Structure (module → host)

```
+<TAG>[=<value>[,<value>[,...]]]<CR><LF>
```

Common tags:

- `+OK` — command accepted (no return data).
- `+ERR=<n>` — command rejected, see `05_at_responses.md`.
- `+<CMD>=<value>` — query response (e.g., `+ADDRESS=120`).
- `+RCV=<addr>,<len>,<data>,<RSSI>,<SNR>` — asynchronous receive event.

### Byte / Payload Limits

- The data payload of `AT+SEND` is restricted to **≤ 240 bytes** per
  transmission. `[from RYLR896 module reference, not extracted source PDF]`
- Commands are line-buffered. The module begins parsing on receipt of `\r\n`.
- Whitespace inside parameters is preserved; whitespace around commas is
  not allowed.

### Timing Considerations

- After power-on or reset, allow the module to boot (~100 ms) before
  issuing the first `AT`. `[from RYLR896 module reference]`
- Each command yields a response within ~100 ms for control commands.
  `AT+SEND` may take significantly longer depending on `AT+PARAMETER`
  (spreading factor, bandwidth, payload length); the host MUST wait for the
  trailing `+OK` (or `+ERR=N`) before issuing the next command.
- Asynchronous `+RCV=...` lines may arrive at any time when the module is
  in receive mode — the host parser MUST tolerate interleaving with
  command responses, although in practice a `+RCV` will not appear inside
  another response line.

### Parsing Recommendations (host side)

1. Read bytes until the next `\r\n`. Strip the terminator.
2. Match the leading character:
   - `+` → unsolicited or response line; dispatch on the tag.
   - any other character → echo (if enabled) or noise; discard.
3. Track an outstanding-command flag so that `+OK` / `+ERR=N` lines can be
   correlated with the most recent host command.

## Traceability

| Source PDF section     | Covered here |
|------------------------|--------------|
| SPECIFICATION (p.4) — UART row "Baud rate ... 8, N, 1" | UART configuration table |

[← 02 Electrical](02_electrical.md) | [Index](index.md) | [Next: 04 AT Commands →](04_at_commands.md)
