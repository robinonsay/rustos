# Juno FSW Requirements

Structured IEEE 29148 requirements for the Juno flight software, organized
by module. Each module has its own directory with a `requirements.json`
(authoritative source) and a `requirements.md` (human-readable rendering).

## System Level (L1)

| Module ID | Title | Path | Req Count | Description |
|-----------|-------|------|-----------|-------------|
| sys | System (FT1 mission L1) | [requirements.md](sys/requirements.md) / [json](sys/requirements.json) | 62 | Top-level FT1 mission requirements that decompose to all FSW modules. |

## FSW Libraries (L2)

| Module ID | Title | Path | Req Count | Description |
|-----------|-------|------|-----------|-------------|
| gps | GPS Library | [requirements.md](gps/requirements.md) / [json](gps/requirements.json) | 10 | GPS receiver driver providing position fixes from NMEA stream. |
| nmea | NMEA Parser Library | [requirements.md](nmea/requirements.md) / [json](nmea/requirements.json) | 12 | Parser for standard NMEA-0183 GPS sentences. |
| device | Device Library (Pico2 UART1) | [requirements.md](device/requirements.md) / [json](device/requirements.json) | 7 | Pico2 UART1 hardware abstraction for serial peripherals. |
| kmat | Matrix Math Library | [requirements.md](kmat/requirements.md) / [json](kmat/requirements.json) | 15 | Fixed-size matrix and vector math for nav/control. |
| log | Diagnostic Logger Library | [requirements.md](log/requirements.md) / [json](log/requirements.json) | 8 | Diagnostic logging facility with severity levels and sinks. |
| sch | TDM Scheduler Library | [requirements.md](sch/requirements.md) / [json](sch/requirements.json) | 10 | Time-Division-Multiplexed cooperative app scheduler. |
| time | Monotonic Time Library | [requirements.md](time/requirements.md) / [json](time/requirements.json) | 7 | Monotonic time source and elapsed-time measurement. |
| imu | IMU Driver Library (MPU-6050) | [requirements.md](imu/requirements.md) / [json](imu/requirements.json) | 14 | MPU-6050 inertial measurement unit driver. |
| baro | Barometric Altimeter Library (MPL3115A2) | [requirements.md](baro/requirements.md) / [json](baro/requirements.json) | 10 | MPL3115A2 barometric pressure/altitude sensor driver. |
| lora | LoRa Radio Library (RYLR896) | [requirements.md](lora/requirements.md) / [json](lora/requirements.json) | 12 | RYLR896 LoRa radio driver for downlink telemetry. |
| sd | SD Card Library | [requirements.md](sd/requirements.md) / [json](sd/requirements.json) | 12 | SD-card block storage and file write interface. |
| nav | Navigation Library (16-state, algorithm-agnostic) | [requirements.md](nav/requirements.md) / [json](nav/requirements.json) | 17 | 16-state navigation filter with algorithm-agnostic API. |
| afm | Automated Flight Manager Library (algorithm-agnostic) | [requirements.md](afm/requirements.md) / [json](afm/requirements.json) | 11 | Flight phase state machine and event manager. |
| telem | Telemetry Packet Library | [requirements.md](telem/requirements.md) / [json](telem/requirements.json) | 12 | Telemetry packet packing, framing, and CRC. |
| mlog | Mission Log Library | [requirements.md](mlog/requirements.md) / [json](mlog/requirements.json) | 14 | Onboard mission data logger with binary record format. |

## FSW Applications (L2)

| Module ID | Title | Path | Req Count | Description |
|-----------|-------|------|-----------|-------------|
| gps_app | GPS App | [requirements.md](gps_app/requirements.md) / [json](gps_app/requirements.json) | 10 | App that schedules the GPS library and publishes fixes to the bus. |
| imu_app | IMU App | [requirements.md](imu_app/requirements.md) / [json](imu_app/requirements.json) | 10 | App that schedules the IMU library and publishes samples to the bus. |
| baro_app | Barometric Altimeter App | [requirements.md](baro_app/requirements.md) / [json](baro_app/requirements.json) | 10 | App that schedules the barometer library and publishes altitude. |
| nav_app | Navigation App | [requirements.md](nav_app/requirements.md) / [json](nav_app/requirements.json) | 13 | App that runs the navigation filter and publishes the nav state. |
| afm_app | AFM App | [requirements.md](afm_app/requirements.md) / [json](afm_app/requirements.json) | 10 | App that runs the automated flight manager state machine. |
| telem_app | Telemetry App | [requirements.md](telem_app/requirements.md) / [json](telem_app/requirements.json) | 11 | App that assembles telemetry packets and dispatches to LoRa. |
| mlog_app | Mission Log App | [requirements.md](mlog_app/requirements.md) / [json](mlog_app/requirements.json) | 12 | App that records mission data to the SD card. |
| sys_app | System App (POST, health, LED, lifecycle) | [requirements.md](sys_app/requirements.md) / [json](sys_app/requirements.json) | 12 | Power-on self-test, health monitor, status LED, and lifecycle control. |

## Simulation Modules (L2)

| Module ID | Title | Path | Req Count | Description |
|-----------|-------|------|-----------|-------------|
| sim_dynamics | Trick 6-DOF Dynamics | [requirements.md](sim_dynamics/requirements.md) / [json](sim_dynamics/requirements.json) | 14 | Six-degree-of-freedom rigid-body rocket dynamics model. |
| sim_sensors | Trick Sensor Models | [requirements.md](sim_sensors/requirements.md) / [json](sim_sensors/requirements.json) | 14 | Sensor models (GPS, IMU, baro) producing realistic FSW inputs. |
| sim_scenario | Trick Scenario Configuration | [requirements.md](sim_scenario/requirements.md) / [json](sim_scenario/requirements.json) | 12 | Scenario, environment, and initial-condition configuration. |
| sim_harness | Trick Top-Level Integration | [requirements.md](sim_harness/requirements.md) / [json](sim_harness/requirements.json) | 10 | Top-level Trick S_define harness wiring FSW to sim models. |

## Schema

See [schema.json](schema.json) for the full JSON schema.

## ID Format

`SW-REQ-<MODULE>-<NNN>`

Examples: `SW-REQ-GPS-001`, `SW-REQ-NAV-003`, `SW-REQ-SYS-001`

## Tooling

```bash
# Search by ID
python3 tools/requirements_search.py --id SW-REQ-GPS-001

# Search by keyword
python3 tools/requirements_search.py --keyword "position fix"

# Generate RTM
python3 tools/rtm.py

# Check burndown
python3 tools/burndown.py

# Verify traceability
python3 tools/traceability.py

# Render markdown views
python3 tools/render_markdown.py
```

---

Total: 28 modules / 371 requirements. Generated for FT1 sprint.
