# 06 — Power-On Initialization Sequence

[← Back to index](index.md)

> **Source-fidelity note.** This procedure is assembled from the public
> RYLR896 AT-command reference and standard practice. Specific timings and
> parameter defaults below are flagged
> `[from RYLR896 module reference, not extracted source PDF]` where they
> are not present in the extracted source datasheet text.

## Goals of the Init Sequence

1. Confirm the module is alive and the UART link is correct.
2. Configure the radio (band, SF/BW/CR, preamble) for the network.
3. Configure the address and network ID.
4. Leave the module in receive-ready state.

## Step-by-Step

```
┌────────┐   ┌──────────┐   ┌──────────────┐   ┌──────────────┐   ┌────────────┐
│ Power  │──►│ NRST low │──►│ wait ≥100 ms │──►│ probe `AT`   │──►│ configure  │──►ready
│ apply  │   │ ≥100 ms  │   │ for +READY   │   │ expect +OK   │   │ params     │
└────────┘   └──────────┘   └──────────────┘   └──────────────┘   └────────────┘
```

| Step | Host action | Module response | Notes |
|------|-------------|-----------------|-------|
| 1 | Apply VDD (3.3 V). | — | Source p.4 |
| 2 | Hold NRST low ≥100 ms, then release. (Optional if cold-booting.) | `+RESET\r\n` then `+READY\r\n` after boot. | Source p.3 (NRST timing); banner `[from module reference]` |
| 3 | Wait for `+READY\r\n` or 100 ms timeout. | — | `[from module reference]` |
| 4 | TX `AT\r\n` (probe). | `+OK\r\n` | If no response, retry up to 3 times, then escalate as init failure. |
| 5 | TX `AT+ADDRESS=<N>\r\n`. | `+OK\r\n` | `<N>` from mission config. EEPROM-persisted. |
| 6 | TX `AT+NETWORKID=<G>\r\n`. | `+OK\r\n` | `<G>` from mission config (0–16). EEPROM-persisted. |
| 7 | TX `AT+BAND=<freq>\r\n`. | `+OK\r\n` | 868 MHz EU, 915 MHz US/AS. EEPROM-persisted. |
| 8 | TX `AT+PARAMETER=<SF>,<BW>,<CR>,<Preamble>\r\n`. | `+OK\r\n` | EEPROM-persisted. |
| 9 | (Optional) TX `AT+VER?\r\n` and `AT+UID?\r\n`; log results. | `+VER=...\r\n`, `+UID=...\r\n` | For boot diagnostics. |
| 10 | Module is now in receive-ready state — emit `+RCV=...` for any inbound frames; `AT+SEND` for outbound. | — | No explicit "enter RX" command; default mode is RX-active. |

## Pseudocode (host side)

```text
init_lora(addr, network_id, band_hz, sf, bw, cr, preamble):
    pulse_nrst_low(>= 100 ms)
    wait_for_line("+READY", timeout = 500 ms)        // or ignore if not seen
    for attempt in 1..3:
        send("AT\r\n")
        if expect_line("+OK", timeout = 200 ms): break
    else:
        return INIT_FAIL_PROBE

    if not configure("AT+ADDRESS=" + addr):       return INIT_FAIL_ADDR
    if not configure("AT+NETWORKID=" + network_id): return INIT_FAIL_NET
    if not configure("AT+BAND=" + band_hz):       return INIT_FAIL_BAND
    if not configure("AT+PARAMETER=" + sf + "," + bw + "," + cr + "," + preamble):
        return INIT_FAIL_PARAM

    return INIT_OK

configure(cmd):
    send(cmd + "\r\n")
    return expect_line("+OK", timeout = 200 ms)
```

## EEPROM-Save Optimization

Because `ADDRESS`, `NETWORKID`, `BAND`, `PARAMETER`, and `IPR` are
persisted to EEPROM with **300 k cycle** endurance (source p.4), the host
SHOULD:

1. Query the current value first (e.g., `AT+ADDRESS?`).
2. Only issue the `=<value>` set form when the current value differs from
   the desired value.

This avoids unnecessary EEPROM wear when the same firmware reboots many
times with identical configuration.

## Receive-Mode Note

The RYLR896 is **continuously in receive mode** when not transmitting (this
is the default behaviour after init). No explicit "enter RX" AT command is
required. `+RCV=...` events arrive asynchronously. `[from RYLR896 module
reference, not extracted source PDF]`

## Sleep / Wake (Out of Scope)

`AT+MODE=1` (sleep, 0.5 µA per source p.4) and `AT+MODE=0` (normal RX) are
referenced in the SPECIFICATION table but are not part of the standard
init sequence and are documented in the broader command reference. For
nominal Juno FSW use the module remains in `AT+MODE=0` continuously.

## Failure Modes During Init

| Symptom | Likely cause | Recovery |
|---------|--------------|----------|
| No `+READY` after reset | NRST not pulsed long enough; UART baud mismatch. | Verify NRST ≥100 ms; verify host UART = 115200 8N1. |
| `+ERR=2` on first `AT` | Garbled bytes during boot; line noise. | Discard; reissue `AT`. |
| `+ERR=4` on `AT+PARAMETER` | Out-of-range field. | Validate SF (7–12), BW (0–9), CR (1–4), preamble (4–7). |
| Probe `AT` times out | Module unpowered, swapped TX/RX, wrong baud. | Check VDD, swap RXD/TXD, try 9600 baud (factory fallback if `AT+IPR` was previously changed). |

[← 05 AT Responses](05_at_responses.md) | [Index](index.md) | [Next: 07 Appendix →](07_appendix.md)
