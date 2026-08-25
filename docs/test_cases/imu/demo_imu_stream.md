# Demonstration Procedure: SW-TC-IMU-017 — IMU Stream Cadence (Pico 2 Hardware)

## 1. Identification

| Field | Value |
|-------|-------|
| Test Case ID | SW-TC-IMU-017 |
| Verifies | SW-REQ-IMU-001 (200 Hz sample rate), SW-REQ-IMU-006 (per-sample timestamp), SW-REQ-IMU-013 (POSIX/Pico2 equivalence — by demonstrating Pico2 path produces the same SI values as POSIX simulation) |
| Verification Method | Demonstration |
| Test Type | Demonstration (operator-executed) |
| Prerequisite Sprint | SPRINT-IMPL-07 (`imu_lib`) closed + SW-TC-IMU-016 demo passed (POST works first) |
| Estimated Duration | 30 minutes (10 s capture + analysis time) |
| Required Personnel | 1 operator with Python/plotting skills |

## 2. Purpose

Demonstrate on flight-target Pico 2 hardware that the IMU pipeline:

1. Produces approximately 200 samples per second (±1% over 10 s)
2. Reports a populated, monotonic timestamp on every sample
3. Reads accel-Z ≈ +9.80665 m/s² (gravity) on a stationary, level board
4. Reports gyro magnitudes ≤ 0.1 rad/s on a stationary board
5. Reports HEALTHY on every sample row

Together these confirm the IMU pipeline behaves correctly end-to-end on
real hardware against the contractual 200 Hz cadence and SI-units output.

## 3. Equipment

| Item | Specification | Purpose |
|------|--------------|---------|
| Raspberry Pi Pico 2 board | RP2350, USB cable | Flight target |
| MPU-6050 breakout board | InvenSense MPU-6050 | Sensor under test |
| Jumper wires | F-F, 4× | Wiring (per `demo_imu_post.md` §4) |
| USB cable | Pico ↔ host computer | Power + serial |
| Host computer with serial capture + Python | `minicom`/`picocom` to capture 115200 8N1; Python 3 with `pandas`, `matplotlib` for plotting | Observation + analysis |
| Level surface (machinist level OK) | ± 0.5° to vertical | Stationary platform with Z-axis aligned to gravity |
| Stopwatch / phone timer | — | Confirms capture window |

## 4. Wiring

Identical to `demo_imu_post.md` §4. See that document for the full diagram.

## 5. Pre-Conditions

1. SW-TC-IMU-016 (POST hardware demo) PASSED for this exact firmware build
   (verifies the IMU is wired and POST works).
2. Firmware built with IMU sample logging enabled. The expected serial format:

   ```
   imu_sample,<timestamp_us>,<ax>,<ay>,<az>,<gx>,<gy>,<gz>,<health>
   ```

   Where timestamps are µs, accels are m/s², gyros are rad/s, health is
   `HEALTHY` or `FAULTED`.
3. Pico 2 placed on a level surface with the board's +Z axis pointing UP
   (so gravity reads +9.80665 on accel-Z).
4. Serial terminal connected, capture-to-file ready.
5. Python environment with `pandas` and `matplotlib` installed:

   ```bash
   python3 -m pip install pandas matplotlib
   ```

## 6. Procedure

| Step | Action | Expected Observation |
|------|--------|----------------------|
| 1 | Verify Pico 2 is on a level surface, +Z up | Visual + bubble-level check |
| 2 | Power on; wait for `POST PASS` log line | Per SW-TC-IMU-016 expected output |
| 3 | Start serial capture: `picocom -b 115200 /dev/ttyACM0 \| tee imu_stream.csv.raw` (or platform equivalent) | Capture begins |
| 4 | Wait 10.0 seconds (use stopwatch) without touching the board | ~2000 IMU sample lines stream in |
| 5 | Stop capture | Capture ends |
| 6 | Strip non-`imu_sample,*` lines and convert to CSV: `grep '^imu_sample,' imu_stream.csv.raw \| sed 's/^imu_sample,//' > imu_stream.csv` and prepend a header row `timestamp_us,ax,ay,az,gx,gy,gz,health` | `imu_stream.csv` ready for analysis |
| 7 | Analyze cadence + SI values with the analysis script (§7) | Plots and pass/fail printout produced |
| 8 | Save the CSV, plots, and analysis script output as artifacts | Complete artifact set |

## 7. Analysis Script

```python
#!/usr/bin/env python3
"""SW-TC-IMU-017 analysis — cadence + stationary-state SI value verification."""
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import sys

df = pd.read_csv('imu_stream.csv')

# 1. Sample count
n = len(df)
print(f"Total samples: {n}  (expected: 2000 ±1% = [1980, 2020])")
assert 1980 <= n <= 2020, f"Sample count {n} outside expected range"

# 2. Cadence: average inter-sample period
dt_us = np.diff(df['timestamp_us'].values)
mean_dt_ms = dt_us.mean() / 1000.0
print(f"Mean inter-sample period: {mean_dt_ms:.3f} ms  (expected: 5.000 ±1%)")
assert 4.95 <= mean_dt_ms <= 5.05, f"Cadence {mean_dt_ms} ms out of tolerance"

# 3. Timestamp monotonicity
assert (dt_us > 0).all(), "Non-monotonic timestamps detected"

# 4. Accel-Z = gravity
mean_az = df['az'].mean()
print(f"Mean accel-Z: {mean_az:.4f} m/s²  (expected: 9.80665 ±0.5)")
assert 9.30 <= mean_az <= 10.30, f"Accel-Z {mean_az} not gravity-like"

# 5. Gyro magnitudes
gyro_mag = np.sqrt(df['gx']**2 + df['gy']**2 + df['gz']**2)
print(f"Max gyro magnitude: {gyro_mag.max():.4f} rad/s  (expected: ≤ 0.1)")
assert gyro_mag.max() <= 0.1, "Gyro magnitudes too high — board not stationary?"

# 6. Health
unhealthy = (df['health'] != 'HEALTHY').sum()
print(f"Unhealthy samples: {unhealthy}  (expected: 0)")
assert unhealthy == 0, f"{unhealthy} unhealthy samples found"

# 7. Plot dt distribution
plt.figure(figsize=(10, 4))
plt.hist(dt_us / 1000.0, bins=50)
plt.axvline(5.0, color='r', linestyle='--', label='5 ms target')
plt.xlabel('Inter-sample period (ms)')
plt.ylabel('Count')
plt.title('SW-TC-IMU-017 — Inter-Sample Period Distribution (200 Hz target)')
plt.legend()
plt.savefig('imu_dt_plot.png', dpi=120)
print("Saved imu_dt_plot.png")

print("ALL CHECKS PASSED")
```

Save this script as `analyze_imu_stream.py` alongside the captured CSV and run:

```bash
python3 analyze_imu_stream.py
```

## 8. Pass/Fail Criteria

The demonstration **passes** when ALL of:

- Sample count is 2000 ±1% (1980 to 2020 samples in the 10 s window)
- Mean inter-sample period is 5.000 ms ±1% (4.95 to 5.05 ms)
- All timestamps are strictly monotonically increasing
- Mean accel-Z is between 9.30 and 10.30 m/s² (gravity within ±5% to allow for bias)
- Maximum gyro magnitude ≤ 0.1 rad/s (stationary)
- Zero unhealthy samples in the 2000-sample window
- The dt-distribution histogram shows a tight peak at 5 ms with no outliers >10 ms

The demonstration **fails** if any of:

- Sample count is outside [1980, 2020] (cadence regression)
- Any timestamp goes backwards (clock bug)
- Mean accel-Z is outside [9.30, 10.30] m/s² (calibration / wiring problem)
- Any gyro reading exceeds 0.1 rad/s (board moved during capture, OR gyro bias)
- Any sample reports FAULTED health (intermittent I2C / chip issue)
- The dt-distribution shows a bimodal histogram (jitter / scheduler stall)

## 9. Expected Artifacts

| Artifact | File Name | Format | Purpose |
|----------|-----------|--------|---------|
| Captured 10-s IMU stream | `imu_stream.csv` | CSV (8 columns, ~2000 rows) | Raw evidence |
| Inter-sample period plot | `imu_dt_plot.png` | PNG image | Cadence visualization |
| Analysis script output | `imu_stream_analysis.log` | UTF-8 text (stdout from analyze script) | Pass/fail printout |
| Operator log | `imu_stream_demo_log.md` | Markdown | Narrative + firmware hash + setup notes |

## 10. Troubleshooting

| Symptom | Possible Cause | Action |
|---------|----------------|--------|
| Sample count ≪ 2000 (e.g., 1500) | Scheduler stall or serial buffer overrun | Reduce serial bandwidth (drop debug logs); verify imu_app period = 5 ms |
| Sample count ≫ 2020 | Capture window > 10 s | Re-run with stopwatch precision |
| Mean dt > 5.05 ms | imu_app not running at 200 Hz | Verify scheduler config; check sch_lib period for imu_app |
| Mean accel-Z far from 9.80665 | Wrong axis orientation, or accel range mismatch | Confirm board level + Z up; verify Configure(PLUS_MINUS_16G) at boot |
| Gyro magnitudes > 0.1 rad/s | Board not stationary, or untested gyro bias | Re-run with absolutely still board; if bias persists, log it for nav_lib calibration |
| Bimodal dt histogram | Co-running app preempts imu_app TDM slot | Reduce co-running app load; verify TDM major-frame budget |

## 11. Sign-Off

| Field | Value |
|-------|-------|
| Operator name | _____________________ |
| Operator signature | _____________________ |
| Date | _____________________ |
| Firmware commit hash | _____________________ |
| Pico 2 board serial | _____________________ |
| MPU-6050 part marking | _____________________ |
| Sample count captured | _____________________ |
| Mean dt (ms) | _____________________ |
| Mean accel-Z (m/s²) | _____________________ |
| Max gyro magnitude (rad/s) | _____________________ |
| Outcome | [ ] Pass     [ ] Fail     [ ] Inconclusive |
| Notes | _____________________ |

## 12. Cross-References

- `docs/requirements/imu/requirements.json` — SW-REQ-IMU-001, -006, -013
- `docs/test_cases/imu/test_cases.json` — SW-TC-IMU-017 entry
- `docs/design/imu/design.md` §8 (timing analysis: 200 Hz cadence; 5 ms TDM slot)
- `apps/imu_app/` — TDM scheduling for Sample() (when imu_app sprint closes)
- `docs/test_cases/imu/demo_imu_post.md` — prerequisite POST demo
