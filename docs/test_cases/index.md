# Juno FSW Test Cases

Test cases are structured JSON files validated against `schema.json`, with
human-readable Markdown summaries (`test_cases.md`) per module. Demonstration
procedures live alongside the JSON as separate Markdown files.

This index provides a complete table of contents for all 28 test case modules
covering the System layer, FSW Libraries, FSW Applications, and the Simulation
environment.

## Modules

| Module ID | Title | Path | TC Count | Coverage |
|-----------|-------|------|----------|----------|
| SYS | System | [sys/](sys/) | 62 | 100% |
| GPS | GPS Library | [gps/](gps/) | 13 | 100% |
| NMEA | NMEA Parser Library | [nmea/](nmea/) | 17 | 100% |
| DEVICE | Device Abstraction Library | [device/](device/) | 8 | 100% |
| KMAT | Kalman Matrix Math Library | [kmat/](kmat/) | 20 | 100% |
| LOG | Logging Library | [log/](log/) | 12 | 100% |
| SCH | Scheduler Library | [sch/](sch/) | 12 | 100% |
| TIME | Time Library | [time/](time/) | 8 | 100% |
| IMU | IMU Library | [imu/](imu/) | 17 | 100% |
| BARO | Barometer Library | [baro/](baro/) | 12 | 100% |
| LORA | LoRa Radio Library | [lora/](lora/) | 15 | 100% |
| SD | SD Card Library | [sd/](sd/) | 15 | 100% |
| NAV | Navigation Library | [nav/](nav/) | 20 | 100% |
| AFM | Active Flight Management Library | [afm/](afm/) | 16 | 100% |
| TELEM | Telemetry Library | [telem/](telem/) | 16 | 100% |
| MLOG | Mission Log Library | [mlog/](mlog/) | 17 | 100% |
| GPS_APP | GPS Application | [gps_app/](gps_app/) | 12 | 100% |
| IMU_APP | IMU Application | [imu_app/](imu_app/) | 12 | 100% |
| BARO_APP | Barometer Application | [baro_app/](baro_app/) | 14 | 100% |
| NAV_APP | Navigation Application | [nav_app/](nav_app/) | 16 | 100% |
| AFM_APP | Active Flight Management Application | [afm_app/](afm_app/) | 13 | 100% |
| TELEM_APP | Telemetry Application | [telem_app/](telem_app/) | 13 | 100% |
| MLOG_APP | Mission Log Application | [mlog_app/](mlog_app/) | 14 | 100% |
| SYS_APP | System Application | [sys_app/](sys_app/) | 14 | 100% |
| SIM_DYNAMICS | Simulation Dynamics | [sim_dynamics/](sim_dynamics/) | 18 | 100% |
| SIM_SENSORS | Simulation Sensors | [sim_sensors/](sim_sensors/) | 16 | 100% |
| SIM_SCENARIO | Simulation Scenarios | [sim_scenario/](sim_scenario/) | 14 | 100% |
| SIM_HARNESS | Simulation Harness | [sim_harness/](sim_harness/) | 13 | 100% |

## System

Cross-cutting test cases that exercise the full FSW stack and project-wide
properties (build, traceability, scheduling envelope, memory model).

- **SYS — System** ([test_cases.md](sys/test_cases.md) | [test_cases.json](sys/test_cases.json)) — 62 test cases

## FSW Libraries

Hardware abstraction and capability libraries (Controller layer). Each library
exposes a public C++11 API used by one or more FSW applications.

- **GPS — GPS Library** ([test_cases.md](gps/test_cases.md) | [test_cases.json](gps/test_cases.json)) — 13 test cases
- **NMEA — NMEA Parser Library** ([test_cases.md](nmea/test_cases.md) | [test_cases.json](nmea/test_cases.json)) — 17 test cases
- **DEVICE — Device Abstraction Library** ([test_cases.md](device/test_cases.md) | [test_cases.json](device/test_cases.json)) — 8 test cases
- **KMAT — Kalman Matrix Math Library** ([test_cases.md](kmat/test_cases.md) | [test_cases.json](kmat/test_cases.json)) — 20 test cases
- **LOG — Logging Library** ([test_cases.md](log/test_cases.md) | [test_cases.json](log/test_cases.json)) — 12 test cases
- **SCH — Scheduler Library** ([test_cases.md](sch/test_cases.md) | [test_cases.json](sch/test_cases.json)) — 12 test cases
- **TIME — Time Library** ([test_cases.md](time/test_cases.md) | [test_cases.json](time/test_cases.json)) — 8 test cases
- **IMU — IMU Library** ([test_cases.md](imu/test_cases.md) | [test_cases.json](imu/test_cases.json)) — 17 test cases
- **BARO — Barometer Library** ([test_cases.md](baro/test_cases.md) | [test_cases.json](baro/test_cases.json)) — 12 test cases
- **LORA — LoRa Radio Library** ([test_cases.md](lora/test_cases.md) | [test_cases.json](lora/test_cases.json)) — 15 test cases
- **SD — SD Card Library** ([test_cases.md](sd/test_cases.md) | [test_cases.json](sd/test_cases.json)) — 15 test cases
- **NAV — Navigation Library** ([test_cases.md](nav/test_cases.md) | [test_cases.json](nav/test_cases.json)) — 20 test cases
- **AFM — Active Flight Management Library** ([test_cases.md](afm/test_cases.md) | [test_cases.json](afm/test_cases.json)) — 16 test cases
- **TELEM — Telemetry Library** ([test_cases.md](telem/test_cases.md) | [test_cases.json](telem/test_cases.json)) — 16 test cases
- **MLOG — Mission Log Library** ([test_cases.md](mlog/test_cases.md) | [test_cases.json](mlog/test_cases.json)) — 17 test cases

## FSW Apps

FSW applications (View layer). Each app composes one or more libraries and is
scheduled by the TDM scheduler.

- **GPS_APP — GPS Application** ([test_cases.md](gps_app/test_cases.md) | [test_cases.json](gps_app/test_cases.json)) — 12 test cases
- **IMU_APP — IMU Application** ([test_cases.md](imu_app/test_cases.md) | [test_cases.json](imu_app/test_cases.json)) — 12 test cases
- **BARO_APP — Barometer Application** ([test_cases.md](baro_app/test_cases.md) | [test_cases.json](baro_app/test_cases.json)) — 14 test cases
- **NAV_APP — Navigation Application** ([test_cases.md](nav_app/test_cases.md) | [test_cases.json](nav_app/test_cases.json)) — 16 test cases
- **AFM_APP — Active Flight Management Application** ([test_cases.md](afm_app/test_cases.md) | [test_cases.json](afm_app/test_cases.json)) — 13 test cases
- **TELEM_APP — Telemetry Application** ([test_cases.md](telem_app/test_cases.md) | [test_cases.json](telem_app/test_cases.json)) — 13 test cases
- **MLOG_APP — Mission Log Application** ([test_cases.md](mlog_app/test_cases.md) | [test_cases.json](mlog_app/test_cases.json)) — 14 test cases
- **SYS_APP — System Application** ([test_cases.md](sys_app/test_cases.md) | [test_cases.json](sys_app/test_cases.json)) — 14 test cases

## Simulation

NASA Trick simulation environment modules (POSIX builds): vehicle dynamics,
sensor models, scenario definitions, and the harness that drives FSW under
simulated conditions.

- **SIM_DYNAMICS — Simulation Dynamics** ([test_cases.md](sim_dynamics/test_cases.md) | [test_cases.json](sim_dynamics/test_cases.json)) — 18 test cases
- **SIM_SENSORS — Simulation Sensors** ([test_cases.md](sim_sensors/test_cases.md) | [test_cases.json](sim_sensors/test_cases.json)) — 16 test cases
- **SIM_SCENARIO — Simulation Scenarios** ([test_cases.md](sim_scenario/test_cases.md) | [test_cases.json](sim_scenario/test_cases.json)) — 14 test cases
- **SIM_HARNESS — Simulation Harness** ([test_cases.md](sim_harness/test_cases.md) | [test_cases.json](sim_harness/test_cases.json)) — 13 test cases

## Schema

See [schema.json](schema.json) for the full JSON schema.

## ID Format

`SW-TC-<MODULE>-<NNN>`

Examples: `SW-TC-GPS-001`, `SW-TC-NAV-003`, `SW-TC-SYS-042`

## Test Types

| Type | Verification Method | Execution |
|------|---------------------|-----------|
| Unit | Test | Automated (Google Test) |
| Integration | Test | Automated (Google Test) |
| Demonstration | Demonstration | Human-executed with artifacts |

## Google Test Tagging

Every `TEST_F` that implements a test case must be tagged:

```cpp
// @{"verify": ["SW-TC-GPS-001"]}
TEST_F(GpsLibTest, GetFix_ReturnsValidFix)
{
    ...
}
```

## Artifact Collection

Demonstration procedures produce artifacts logged in `expected_artifacts`:

- `serial_output` — terminal/serial logs
- `csv` — telemetry data files
- `plot` — data plots (PNG/PDF)
- `photo` — hardware photos
- `video` — test video recordings
- `log` — general log files

---

Total: 28 modules / 449 test cases / 100% requirement coverage.
