# Demonstration Procedure: SW-TC-IMU-016 — IMU Power-On Self-Test (Pico 2 Hardware)

## 1. Identification

| Field | Value |
|-------|-------|
| Test Case ID | SW-TC-IMU-016 |
| Verifies | SW-REQ-IMU-008 (POST probe), SW-REQ-IMU-011 (unhealthy on POST failure) |
| Verification Method | Demonstration |
| Test Type | Demonstration (operator-executed) |
| Prerequisite Sprint | SPRINT-IMPL-07 (`imu_lib`) closed; SPRINT-IMPL-07-retro (full BIT) closed |
| Estimated Duration | 15 minutes |
| Required Personnel | 1 operator (rocketry-experienced) |

## 2. Purpose

Verify on flight-target Pico 2 hardware that `imu_lib::PowerOnSelfTest()`:

1. Returns SUCCESS and reports HEALTHY when an MPU-6050 is correctly wired
   (passes WHO_AM_I=0x68 + full BIT register sequence per L2 design §9 item 4).
2. Returns INVALID_DATA_ERROR (or READ_ERROR) and reports FAULTED when the
   IMU is missing or its SDA line is disconnected (mid-bus failure proxy).

Together these two scenarios demonstrate the POST contract end-to-end on real
hardware in a way that simulation alone cannot. Per
`docs/requirements/imu/requirements.json`:

- **SW-REQ-IMU-008**: "The IMU library shall probe the IMU device once at
  startup and report a pass-or-fail result."
- **SW-REQ-IMU-011**: "The IMU library shall report the IMU as unhealthy
  when the startup probe fails."

## 3. Equipment

| Item | Specification | Purpose |
|------|---------------|---------|
| Raspberry Pi Pico 2 board | RP2350, USB-A or USB-C cable | Flight target |
| MPU-6050 breakout board | InvenSense MPU-6050, 3.3 V tolerant | Sensor under test |
| Jumper wires | Female-to-female, 4× minimum | Pico ↔ MPU-6050 wiring |
| USB cable (data) | Pico ↔ host computer | Power + serial CDC |
| Host computer with serial terminal | `minicom`, `picocom`, or PuTTY at 115200 8N1 | Observation + log capture |
| Camera (smartphone OK) | — | Setup photograph |
| Multimeter / ohmmeter | Continuity mode | Wiring continuity check |
| (Optional) 4.7 kΩ resistors × 2 | I2C pull-ups to 3V3 | If breakout lacks pull-ups |

## 4. Wiring

The MPU-6050 connects to the Pico 2 over I2C0 (per the FT1 composition
root convention — GP4/GP5 on the Pico 2):

```
Pico 2                  MPU-6050
------                  --------
3V3 (Pin 36)   ---->    VCC
GND (Pin 38)   ---->    GND
GP4 (Pin 6)    ---->    SDA          [I2C0 SDA]
GP5 (Pin 7)    ---->    SCL          [I2C0 SCL]
                        AD0  ---->   GND  (selects 0x68 device address;
                                          tie to VCC for 0x69)
                        INT  unused
                        XCL  unused
                        XDA  unused
```

ASCII wiring sketch:

```
   Pico 2                              MPU-6050
   +------+                            +------+
   |  3V3 |--------------------------->| VCC  |
   |  GND |--------------------------->| GND  |
   |  GP4 |--- I2C0 SDA -------------->| SDA  |
   |  GP5 |--- I2C0 SCL -------------->| SCL  |
   +------+                       GND->| AD0  | (selects 0x68)
                                       +------+
```

Note: Many MPU-6050 breakouts include onboard 4.7 kΩ pull-ups on SDA/SCL.
If the breakout in use does NOT have pull-ups, install 4.7 kΩ resistors
from SDA→3V3 and SCL→3V3 before powering on.

TODO (hardware-bench time): Replace ASCII sketch above with a labeled
photograph of the actual bench setup once hardware is available; save
as `imu_post_wiring_photo.jpg` adjacent to this document.

## 5. Pre-Conditions

1. Latest FT1 firmware built for `PLATFORM=PICO2` (`build_pico2/`) and flashed
   to the Pico 2 (`picotool load <fsw_app>.uf2`). Verify the firmware build
   includes SPRINT-IMPL-07-retro's full BIT (commit hash recorded in §11).
2. Wiring per §4 confirmed by visual inspection AND ohmmeter continuity
   check (3V3↔VCC, GND↔GND, GP4↔SDA, GP5↔SCL, AD0↔GND).
3. Serial terminal connected to the Pico 2 USB CDC port (typically
   `/dev/ttyACM0` on Linux/macOS, `COM<N>` on Windows). Settings: 115200
   baud, 8 data bits, no parity, 1 stop bit, no flow control.
4. Operator log book / electronic notebook available for procedure
   record-keeping (§11 sign-off).
5. No flight-critical hardware powered nearby; this is a bench activity.

## 6. Procedure — Part A: POST PASS (IMU connected)

| Step | Action | Expected Observation |
|------|--------|----------------------|
| A.1 | Power on the Pico 2 by plugging in USB | Boot banner appears on serial terminal within 1 second |
| A.2 | Observe the IMU POST log line during startup | Log line `[INFO] imu: POST PASS — WHO_AM_I=0x68, BIT response={0xXX,0xXX,0xXX,0xXX}` appears within 500 ms of boot |
| A.3 | Capture serial terminal output to a file | Save as `imu_post_pass_serial.log` |
| A.4 | Photograph the wiring as configured | Save as `imu_post_setup.jpg` |
| A.5 | Note the firmware commit hash printed in the boot banner | Record in operator log (§11) |
| A.6 | Verify the periodic IMU health publication shows HEALTHY | Look for `imu_health=HEALTHY` in subsequent log lines |

## 7. Procedure — Part B: POST FAIL (IMU SDA disconnected)

| Step | Action | Expected Observation |
|------|--------|----------------------|
| B.1 | Power off the Pico 2 (unplug USB) | LEDs off |
| B.2 | Disconnect the MPU-6050 SDA jumper wire from GP4 | Visual confirmation: SDA wire dangling, not touching any other pin |
| B.3 | Power on the Pico 2 | Boot banner appears |
| B.4 | Observe the IMU POST log line during startup | Log line `[ERROR] imu: POST FAIL — status=READ_ERROR` (or `INVALID_DATA_ERROR` if WHO_AM_I returns garbage) appears within 500 ms |
| B.5 | Verify subsequent log lines reflect FAULTED IMU health | Search for `imu_health=FAULTED` (or equivalent unhealthy state) in the stream |
| B.6 | Capture serial terminal output to a file | Save as `imu_post_fail_serial.log` |
| B.7 | Power off; reconnect SDA; verify Part A still passes when re-powered | Confirms the failure was solely attributable to the SDA disconnect, not a permanent fault |

## 8. Pass / Fail Criteria

The demonstration **passes** when ALL of the following hold:

- A.2 produces `POST PASS` and an HEALTHY health bit.
- B.4 produces `POST FAIL` and a FAULTED (unhealthy) health bit.
- B.7 confirms reversibility (the failure was caused by the disconnect,
  not by an unrelated fault).
- Boot-to-POST latency is ≤ 500 ms in both parts.
- The captured serial logs unambiguously show the POST status lines.

The demonstration **fails** if any of the following occur:

- POST status line never appears within 5 seconds of boot.
- POST PASS is reported with the IMU disconnected (false-pass — implies
  WHO_AM_I check is broken or stub-driven at FT1 build).
- POST FAIL is reported with the IMU correctly connected (false-fail —
  implies wiring or BIT logic regression).
- The board crashes / hangs during POST (any boot-banner absence after a
  10-second wait is classified as HANG and treated as a failure).

## 9. Expected Artifacts

| Artifact | File Name | Format | Purpose |
|----------|-----------|--------|---------|
| Part A serial log | `imu_post_pass_serial.log` | UTF-8 text | Evidence of POST PASS |
| Part B serial log | `imu_post_fail_serial.log` | UTF-8 text | Evidence of POST FAIL |
| Wiring photo | `imu_post_setup.jpg` | JPEG image | Reproducibility evidence |
| Operator log | `imu_post_demo_log.md` | Markdown | Operator narrative + firmware hash + observations |

All four artifacts are required to consider the demonstration complete and
to mark SW-TC-IMU-016 status as `Passed`. Artifacts are stored under
`docs/test_cases/imu/artifacts/SW-TC-IMU-016/<date>/` once collected.

## 10. Troubleshooting

| Symptom | Possible Cause | Action |
|---------|----------------|--------|
| Boot banner never appears | Pico 2 not flashed or USB cable is power-only | Reflash firmware; verify USB-data cable |
| `POST FAIL — INVALID_DATA_ERROR` with IMU connected | AD0 line not tied to GND (chip is at 0x69, FW expects 0x68) | Tie AD0 to GND or update FW config to 0x69 |
| `POST FAIL — WRITE_ERROR` with IMU connected | I2C pull-ups missing on the breakout | Add 4.7 kΩ pull-ups on SDA and SCL to 3V3 |
| All-zero SELF_TEST response (`POST FAIL — INVALID_DATA_ERROR` even with WHO_AM_I=0x68) | Chip is in a sleep state or not factory-trim-programmed | Power-cycle; verify VCC is 3.3 V (not a 1.8 V breakout variant) |
| Boot hangs at POST | I2C bus stuck (SDA held low by mis-wired device) | Disconnect MPU-6050; reboot; verify boot proceeds without IMU |
| Serial terminal shows garbage | Wrong baud rate | Reconfigure terminal to 115200 8N1, no flow control |
| Intermittent POST PASS / FAIL | Loose jumper wire on SDA or SCL | Reseat all jumpers; consider soldered harness |

## 11. Sign-Off

| Field | Value |
|-------|-------|
| Operator name | _____________________ |
| Operator signature | _____________________ |
| Date | _____________________ |
| Firmware commit hash | _____________________ |
| Pico 2 board serial | _____________________ |
| MPU-6050 part marking | _____________________ |
| Outcome | [ ] Pass     [ ] Fail     [ ] Inconclusive |
| Notes | _____________________ |

## 12. Cross-References

- `docs/requirements/imu/requirements.json` — SW-REQ-IMU-008, SW-REQ-IMU-011
- `docs/test_cases/imu/test_cases.json` — SW-TC-IMU-016 entry
- `docs/design/imu/design.md` §9 item 4 — POST contract + BIT sequence
  (amended 2026-05-06)
- `libs/imu_lib/src/pico2/imu_pico2.cpp` — `IMU_LIB_PICO2_T::PowerOnSelfTest`
  implementation
- `apps/sys_app/` — POST-bitmap publisher (when sys_app sprint closes;
  references the IMU POST status into the system health bitmap)
- `pico-sdk/src/boards/include/boards/pico2.h` — Pico 2 pinout reference
  for I2C0 SDA/SCL (GP4/GP5)
