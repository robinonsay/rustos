# GPS ICD — Timing and Performance

Source: GlobalTop FGPMMOPA6H Datasheet V0A, Section 2.6 and Sections 1.5,
1.7, 1.8.

Back to: [index.md](index.md)

## Time-to-First-Fix (TTFF)

| Start type | Typical TTFF | Description |
|------------|--------------|-------------|
| Hot start  | **1 s** | Time, position, almanac, and ephemeris all cached and valid |
| Warm start | **33 s** | Time and position cached; ephemeris stale or missing |
| Cold start | **35 s** | No cached data; module must acquire all SV info from scratch |

TTFF measurement conditions (vendor): number of SVs > 4, C/N > 40 dB,
PDOP < 1.5.

### Cold Start Triggers

A cold start occurs when:

- VBACKUP power is absent (battery removed or never connected) on power-up.
- The host issues `$PMTK103` (PMTK_CMD_COLD_START) — see
  [05_commands.md](05_commands.md).
- The host issues a full cold start / factory reset (PMTK104, vendor command
  list).

### Warm / Hot Start Behavior

Warm and hot starts rely on data preserved in the GPS chipset RTC and memory
backed by VBACKUP (Pin 4). To enable warm/hot starts, supply VBACKUP at all
times — see [02_electrical.md](02_electrical.md).

### EASY (Self-Generated Orbit Prediction)

When EASY is enabled (custom firmware), the module pre-calculates predicted
orbit data (up to **3 days**) on power-up and saves it to internal memory.
The engine uses this prediction when satellite information is insufficient,
improving fix performance under indoor or urban conditions.

**Not used by Juno FSW** — documented for vendor fidelity only.

## Update Rate

| Property | Value |
|----------|-------|
| Default rate | **1 Hz** |
| Maximum rate | **10 Hz** |
| Maximum rate with SBAS | **5 Hz** |

The update rate determines how often the full default NMEA sentence set is
emitted on the UART. Higher update rates require:

- Higher UART baud (configured via PMTK251 — vendor command list).
- Disabling SBAS if rate > 5 Hz.

The default 1 Hz rate fits comfortably within the 9600 baud default UART.

## 1PPS Timing

| Property | Value |
|----------|-------|
| Pin | 13 |
| Logic level | 2.8 V CMOS |
| Typical accuracy / jitter | **10 ns** |
| Activation | After 3D fix |
| Source | Synchronized to GPS time |
| Pre-fix output | None by default; available via custom firmware |

The 1PPS rising edge marks the start of each UTC second after a 3D fix.
Typical applications include:

- Computer timekeeping (NTP synchronization)
- Time-tagging external sensor samples
- Aligning multiple sensor time bases

The vendor datasheet does not specify the offset between the 1PPS edge and
the corresponding UART NMEA sentence. Implementations that require precise
1PPS-to-NMEA correlation must measure this offset empirically and account
for UART transmission latency at the operating baud rate.

## 3D-FIX Indicator Timing

| State | Pre-2D fix | Post-2D / 3D fix |
|-------|------------|-------------------|
| Pin 5 waveform | 1 s high / 1 s low | Continuous low |
| Effective rate | 0.5 Hz square wave | DC low |

The 3D-FIX line provides hardware-observable fix indication independent of
NMEA parsing. It is suitable for driving an LED or interrupting a host MCU.
Custom firmware can re-purpose this line for wake-on-fix or other timing
applications.

## Acquisition vs Tracking Power Profile

| Mode | Current @ 3.3 V | Power |
|------|-----------------|-------|
| Acquisition | 25 mA typ. | ≈ 82 mW |
| Tracking | 20 mA typ. | ≈ 66 mW |
| Backup (VBACKUP only) | 7 µA typ. @ 3.0 V, 25 °C | ≈ 21 µW |

Acquisition mode is active during cold/warm start and reacquisition; tracking
mode is active once stable lock is achieved.

## Performance Limits

| Limit | Value | Notes |
|-------|-------|-------|
| Maximum altitude | 18,000 m (60,000 ft) | Above this, fix may be lost |
| Maximum velocity | 515 m/s (1000 knots) | — |
| Maximum acceleration | 4 G | Sustained acceleration beyond this may cause loss of lock |

**Juno FSW relevance — FT1/FT2:** the planned amateur-rocket flight profile
(G to L1 motor) operates well within altitude (< 1500 m typical) and velocity
limits (< 200 m/s peak). Peak acceleration during motor burn may approach or
exceed 4 G briefly; loss of GPS lock during the boost phase is expected and
must be handled by the navigation/sensor-fusion software.

## Position and Velocity Accuracy

| Mode | Position (50% CEP) | Velocity |
|------|---------------------|----------|
| Without aid | 3.0 m | 0.1 m/s |
| With SBAS (WAAS / EGNOS / MSAS / GAGAN) | 2.5 m | 0.05 m/s |

SBAS is enabled by default but is automatically disabled at update rates
> 5 Hz.

## Sensitivity

| Phase | Sensitivity |
|-------|-------------|
| Acquisition (cold start) | -148 dBm |
| Reacquisition (hot start) | -163 dBm |
| Tracking | -165 dBm |

Open-sky reception can reach up to **45 dB C/N** for visible SVs (without
patch antenna).

## Antenna and RF

The module operates exclusively at L1 (1575.42 MHz). The integrated patch
antenna (15 × 15 × 2.5 mm ceramic) is suitable for default operation. For
constrained RF environments or in-vehicle integrations, an external active
antenna can be connected to EX_ANT (Pin 11); the module switches
automatically when ≥ 4 mA antenna current is detected.

See [02_electrical.md](02_electrical.md) for external antenna specifications
and current limits.
