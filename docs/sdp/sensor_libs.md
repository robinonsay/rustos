---
document_type: SDP — Sensor Driver Libraries (Wave 3)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: index.md
sprints_covered: SPRINT-IMPL-07 through SPRINT-IMPL-11
status: Draft (awaiting Chief Engineer gate)
---

# SDP — Wave 3: Sensor Driver Libraries

## 1. Purpose

This file details five sensor-driver implementation sprints (SPRINT-IMPL-07 through SPRINT-IMPL-11). Each driver wraps a single hardware part behind the LibJuno C++ vtable pattern; the POSIX implementation uses simulated/test fixtures (typically backed by `sim_sensors` Trick globals or a backing-file image), and the Pico2 implementation drives the real peripheral over the bus owned by `device_lib`. Wave 3 closes the sensor-driver layer of the Controller stack so that Wave 4 (domain libraries that consume sensor message types — `nav_lib`, `afm_lib`, `telem_lib`, `mlog_lib`) and Wave 5 (sensor apps — `imu_app`, `baro_app`, `gps_app`) can begin in turn.

## 2. Wave Summary

| Wave | Sprints | Modules | Predecessor Wave | Successor Waves |
|------|---------|---------|------------------|-----------------|
| 3 | SPRINT-IMPL-07..11 | imu, baro, gps, lora, sd | Wave 1 only for `imu_lib` / `baro_lib` / `sd_lib` (these own their own I2C/SPI directly per their L2 designs); Wave 2 `device_lib` for `gps_lib` (UART) and `lora_lib` (UART); Wave 1 `nmea_lib` additionally for `gps_lib` | Wave 4 (`domain_libs.md`), Wave 5 (`sensor_apps.md`) |

After the Wave 1 exit gate PASS (per `foundation_libs.md`), `imu_lib` / `baro_lib` / `sd_lib` are eligible immediately — they own their own peripheral handles (I2C for imu/baro; SPI for sd) per their respective L2 designs and do not consume `device_lib` (which is UART1-only per `docs/design/device/design.md` §1). `gps_lib` and `lora_lib` are eligible after the Wave 2 exit gate PASS (they consume `device_lib::DEVICE_LIB_API_T` for UART byte transport); `gps_lib` additionally requires `nmea_lib` (Wave 1) for sentence parsing. Recommended sequencing: complete SPRINT-IMPL-07 (`imu_lib`) first to derisk the LibJuno C++ vtable pattern at scale (5 ms TDM cadence, 17 test cases, 14 requirements), then run SPRINT-IMPL-08 / -09 / -10 / -11 in parallel as their respective predecessor waves close.

All five sprints inherit the carry-forward RFAs from `methodology.md` — specifically RFA-3 (capacity pins) is reaffirmed here for `juno::device::kDefaultRingCap`, which is consumed by `gps_lib`'s UART config (`kGpsRxRingCap = 2048`) and by `lora_lib` / `sd_lib` defaults. RFA-4 (Option C `SIM_SENSORS_RAW_T`) remains deferred; the Wave 3 drivers compose against the Option D static-assert injection seam (in-place `const SIM_SENSORS_RAW_T *` dereferenced by the POSIX impl).

## 3. Per-Sprint Plans

### SPRINT-IMPL-07 — imu_lib

- **Module**: `imu_lib` — MPU-6050 6-DoF IMU driver (IMU model locked 2026-05-03 per S1-AI-022 / FLAG-4)
- **Predecessors**: none from Wave 2 — `device_lib` is UART1-only per its L2; `imu_lib` Pico2 impl owns the I2C peripheral directly via Pico SDK (`i2c_inst_t *ptI2C` factory arg per L2 §4.1); POSIX impl reads `SIM_SENSORS_RAW_T` injected by sim_sensors. Composition root supplies the I2C handle. (Wave 0 `JUNO_MSG_BUS_VARIANT_T` not consumed by libs.)
- **L2 design**: `docs/design/imu/design.md`
- **Requirements**: 14 SW-REQ-IMU-* IDs (`SW-REQ-IMU-001` through `SW-REQ-IMU-014`)
- **Files to produce** (6 files, posix/pico2 split applies):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/imu_lib/include/imu_lib/imu_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/imu_lib/src/imu_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/imu_lib/src/posix/imu_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/imu_lib/src/pico2/imu_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/imu_lib/tests/imu_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/imu_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 15 Unit-type entries (`SW-TC-IMU-001`..`-015`); 2 Demonstration-type (`SW-TC-IMU-016`, `-017`) deferred to Pico2 hardware bring-up
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - MPU-6050 register map honored per L2 §4.1 (WHO_AM_I `0x75` → `0x68`; `ACCEL_CONFIG.AFS_SEL = 0b11` for ±16 g; `GYRO_CONFIG.FS_SEL = 0b11` for ±2000 dps; `SMPLRT_DIV` programmed for 200 Hz output)
  - `SIM_SENSORS_RAW_T` injection seam (Option D static_assert) implemented in `imu_posix.cpp` using a `const SIM_SENSORS_RAW_T *` pointer member
  - All `IMU_SAMPLE_T` fields populated on success: `tAccel[3]` (m/s²), `tGyro[3]` (rad/s), `tTimestampUs`, `bValid`
  - Body-axis permutation matrix (X-fwd/Y-right/Z-down per `SW-REQ-SYS-057`) is `static constexpr` and bit-identical between POSIX and Pico2 builds (`SW-REQ-IMU-013`/`-014`)
  - `Sample()` worst-case bound: ≤ 500 µs Pico2, ≤ 50 µs POSIX (per L2 §8)
  - Failure handler is diagnostic-only on read/POST failure; library never aborts (`SW-REQ-IMU-012`)
- **Test gate**: G1 (POSIX unit tests pass) + G2 (traceability tool exit code 0) + G3 (Pico2 cross-compile clean)
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents

### SPRINT-IMPL-08 — baro_lib

- **Module**: `baro_lib` — NXP MPL3115A2 barometric altimeter driver (I2C address `0x60`)
- **Predecessors**: none from Wave 2 — `baro_lib` does not touch I2C directly per its L2 §1; transport is a `BARO_LIB_BUS_T` `WriteReg`/`ReadReg` callback pair injected at `BARO_LIB_IMPL_T::New()` by the composition root (Pico2: `apps/main.cpp` provides the I2C-backed callbacks; POSIX: `sim_harness/interfaces.md` §4.4.1 supplies the sim-backed callbacks). No `device_lib` dependency.
- **L2 design**: `docs/design/baro/design.md`
- **Requirements**: 10 SW-REQ-BARO-* IDs (`SW-REQ-BARO-001` through `SW-REQ-BARO-010`)
- **Files to produce** (6 files, posix/pico2 split applies):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/baro_lib/include/baro_lib/baro_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/baro_lib/src/baro_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/baro_lib/src/posix/baro_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/baro_lib/src/pico2/baro_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/baro_lib/tests/baro_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/baro_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 10 Unit-type entries (`SW-TC-BARO-001`..`-004`, `-006`..`-009`, `-011`, `-012`); 2 Demonstration-type (`SW-TC-BARO-005` POST present/absent, `SW-TC-BARO-010` Pico2 lift stimulus)
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - MPL3115A2 register sequence honored per L2 §4.2 (WHO_AM_I `0x0C` → `0xC4`; `CTRL_REG1 = 0xB8` OSR=128 altimeter standby; `PT_DATA_CFG = 0x07` DREADY enable; BAR_IN written from `tRoot.fSlpPa / 2.0`; `CTRL_REG1.SBYB = 1` to begin sampling)
  - **`BARO_LIB_BUS_T` callback transport per L2 §4.1** — `baro_lib` does NOT touch the I2C bus directly; the `WriteReg`/`ReadReg` callback pair is injected by the composition root at `New()` (`AC-10` cross-check)
  - `BARO_SAMPLE_T` fields populated: `tTimestampUs`, `fPressurePa`, `fTempC`, `fAltMHae`, `bValid` — the timestamp is **caller-supplied** (`tNowUs` argument) per L2 §4.2.3
  - `fAltMHae` field name is the locked canonical name per `system_design.md` §4 (semantic divergence from WGS-84 HAE is documented in L2 §11; `nav_lib` reconciles)
  - `Sample()` honors `tTimeoutUs` non-blocking guarantee (`SW-REQ-BARO-008`); worst-case 1.5 ms Pico2 (per L2 §8)
  - sim_sensors injection cross-check: POSIX impl reads from `sim_sensors`-backed `BARO_LIB_BUS_T` callbacks (RFA-4 deferred — composition root is responsible for the shim)
- **Test gate**: G1 + G2 + G3 (Pico2 cross-compile)
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents

### SPRINT-IMPL-09 — gps_lib

- **Module**: `gps_lib` — GlobalTop FGPMMOPA6H GPS receiver driver (UART 9600 baud, 5 Hz NMEA cadence)
- **Predecessors**: **SPRINT-IMPL-05** (`device_lib` for UART transport), **SPRINT-IMPL-04** (`nmea_lib` for sentence framing/parsing)
- **L2 design**: `docs/design/gps/design.md`
- **Requirements**: 10 SW-REQ-GPS-* IDs (`SW-REQ-GPS-001` through `SW-REQ-GPS-010`)
- **Files to produce** (6 files — `gps_impl.hpp` is split out per L2 §3.3, posix/pico2 split applies; the common `gps_impl.cpp` is folded into the `gps_impl.hpp` header per L2's header-only common-impl pattern):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/gps_lib/include/gps_lib/gps_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/gps_lib/include/gps_lib/gps_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/gps_lib/src/posix/gps_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/gps_lib/src/pico2/gps_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/gps_lib/tests/gps_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/gps_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 10 Unit-type entries (`SW-TC-GPS-001`..`-010`); 3 Demonstration-type (`SW-TC-GPS-011`, `-012`, `-013`) for Pico2 hardware probe present/absent and flight-UART NMEA stream
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - **POSIX impl uses the `openpty` pty-fd seam** per the delta-PDR fix, with the slave-side fd writeable by the test harness; the lib reads via `device_lib::ReadBytes` regardless of platform
  - RX ring capacity pinned at `kGpsRxRingCap = 2048` (RFA-3 reaffirmed); composes `juno::device::DEVICE_LIB_ROOT_T<2048>` so the `static_assert(N >= 256)` from `device_lib` is satisfied with 8× margin
  - `nmea_lib` byte-streaming contract (`FeedByte` + `GetParsed`) used per L2 §6.2 — `gps_lib` never accumulates a sentence buffer of its own; verbatim `au8RawBytes` sourced from `nmea_lib` for `tLastRaw` (`SW-REQ-GPS-002`)
  - `GPS_FIX_T` carries WGS-84 geodetic + HAE per `SW-REQ-GPS-010`; `eFixQuality` mirrors NMEA GGA field 6
  - `IsHealthy()` staleness threshold `kHealthStaleUs = 600'000` (3× the 200 ms `gps_app` period) tolerates a single missed sentence
  - `Poll()` returns `JUNO_STATUS_TABLE_FULL_ERROR` on RX ring overflow per L2 §9 item 9 (PM Decision 2026-05-02 amended `DEVICE-004`); partial bytes are still fed to `nmea_lib` for resync
- **Test gate**: G1 + G2 + G3 (Pico2 cross-compile)
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents

### SPRINT-IMPL-10 — lora_lib

- **Module**: `lora_lib` — RYLR896 LoRa radio transport driver (UART, AT-command framing, ≤ 240 byte MTU)
- **Predecessors**: SPRINT-IMPL-05 (`device_lib` for UART transport)
- **L2 design**: `docs/design/lora/design.md`
- **Requirements**: 12 SW-REQ-LORA-* IDs (`SW-REQ-LORA-001` through `SW-REQ-LORA-012`)
- **Files to produce** (6 files, posix/pico2 split applies):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/lora_lib/include/lora_lib/lora_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/lora_lib/src/lora_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/lora_lib/src/posix/lora_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/lora_lib/src/pico2/lora_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/lora_lib/tests/lora_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/lora_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 13 Unit-type entries (`SW-TC-LORA-001`..`-013`); 2 Demonstration-type (`SW-TC-LORA-014` Pico2 POST, `SW-TC-LORA-015` Pico2 2 Hz downlink with paired RX module)
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - AT-command configuration suite issued at `Init()` per `SW-REQ-LORA-004`: `AT+ADDRESS`, `AT+NETWORKID`, `AT+BAND`, `AT+PARAMETER` (each populated from caller-supplied config struct fields per `SW-TC-LORA-004`)
  - POST probe issues bare `AT` and verifies `+OK\r\n` response from RYLR896 within startup window (`SW-REQ-LORA-005`)
  - Payload bytes pass through unmodified — driver embeds caller bytes inside `AT+SEND` framing without interpreting them, including all 256 byte values and embedded `AT+SEND` / `\r\n` substrings (`SW-REQ-LORA-002` / `SW-TC-LORA-002`)
  - **`IsHealthy()` and `IsBusy()` polling APIs** that `telem_app` and `sys_app` consume — `IsHealthy()` reflects the most recent transmit / module-response outcome (`SW-REQ-LORA-006`/`-007`/`-008`); `IsBusy()` (or equivalent state accessor) lets `telem_app` skip a tick if the radio is mid-transmission rather than backpressuring the 500 ms TDM slot
  - Configurable UART baud rate parameter accepted at `New()` (`SW-REQ-LORA-012`)
  - Each transmit completes within the 500 ms `kTelemAppPeriodMs` budget (`SW-REQ-LORA-003`)
- **Test gate**: G1 + G2 + G3 (Pico2 cross-compile)
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents

### SPRINT-IMPL-11 — sd_lib

- **Module**: `sd_lib` — SD-card block storage driver (SPI on Pico2; backing-file image on POSIX); raw 512 B block append, no FAT (per L2 §3.4 FT1 decision)
- **Predecessors**: none from Wave 2 — `sd_lib` owns SPI directly per L2 §3.1 ("Owns SPI bus to the SD card on Pico2; owns the scratch image FD on POSIX"); the `device_lib` UART1 abstraction does not cover SPI. SPI peripheral handle (Pico2) or scratch-image fd (POSIX) is opened inside `sd_lib::*::New()`.
- **L2 design**: `docs/design/sd/design.md`
- **Requirements**: 12 SW-REQ-SD-* IDs (`SW-REQ-SD-001` through `SW-REQ-SD-012`)
- **Files to produce** (6 files, posix/pico2 split applies):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/sd_lib/include/sd_lib/sd_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/sd_lib/src/sd_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/sd_lib/src/posix/sd_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/sd_lib/src/pico2/sd_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/sd_lib/tests/sd_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/sd_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 12 Unit-type entries (`SW-TC-SD-001`..`-008`, `-010`..`-012`, `-015`); 2 Integration-type (`SW-TC-SD-013`, `-014` POSIX/flight parity); 1 Demonstration-type (`SW-TC-SD-009` Pico2 sustained throughput on Class 10+ card)
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - **Templated `SD_LIB_ROOT_T<kDefaultWriteBufBlocks>` consumed by `mlog_app` and `sys_app`** per L2 §4.1; the default `N = 4` (2 KiB staging buffer) covers `mlog_lib`'s ≤ 131 B per-NMEA-record worst case with margin; `system_design.md` composition root pins one `N` for the flight build
  - Brief lifecycle naming crosswalk (Init/Write/Flush/Close → `Mount`/`WriteBlock`/`Sync`/`Deinit`) honored per L2 §2.1; the SD-idiomatic names ship in the public API and the crosswalk is documented for AC-8 audit
  - Raw 512 B block append on Pico2 — no FAT — per `SW-REQ-SD-005`/`-006` and L2 §3.4; ground-side analysis uses `tools/sd_dump.py` (separate ticket per FLAG-1)
  - Run-header block writes (one per power-on) implement `SW-REQ-SD-003` (new run dir per power-on) and `SW-REQ-SD-004` (prior runs preserved by never rewriting older headers)
  - **`IsHealthy()` polled by `sys_app`** per PM Decision 3 (no broker publish from `sd_lib`); `_u32ConsecFailures` health latch threshold `kMaxConsecFailures = 8` (≈ 40 ms at 5 ms `mlog_app` cadence) — design choice per L2 FLAG-3
  - Determinism guarantee — identical input streams produce byte-identical on-disk output (`SW-REQ-SD-012`); zero-padding with `0x00` on `Sync()` of the final partial block; deterministic LBA assignment (run_base + cursor/512); no clock-derived bytes in the on-disk stream
- **Test gate**: G1 + G2 + G3 (Pico2 cross-compile)
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents

## 4. Wave Exit Gate

After SPRINT-IMPL-11 closes, the Software Lead spawns a "Wave 3 Exit Gate" `project-chief-engineer` invocation that confirms:

- All 5 sprints CLOSED with G1 + G2 + G3 passing
- **Cross-driver consistency check**:
  - Every driver publishes a `New()` factory returning `RESULT_T<<MODULE>_LIB_IMPL_T>` (per the LibJuno C++ pattern from `conventions.md` §1.2)
  - Every driver implements `IsHealthy()` if its lib L2 specifies one (`baro_lib`, `gps_lib`, `lora_lib`, `sd_lib` all do; `imu_lib` exposes `Health()` returning `OPTION_T<IMU_HEALTH_T>` per L2 §4.2 — accepted as the moral equivalent under naming divergence with documented note)
  - Every driver's vtable is wired exactly once via a `static <MODULE>_LIB_API_T tApi{...}` local inside `New()` (per `conventions.md` §1.2, never reassigned)
  - No driver allocates dynamic memory; no driver throws; every API entry is `noexcept` (per `SW-REQ-SYS-050` / `-053`)
- **Burndown check**: 5 modules' SW-REQ closure delta matches the per-module counts above (14 + 10 + 10 + 12 + 12 = **58 SW-REQ-* IDs** newly closable). Verified by `python3 tools/burndown.py` exit code 0.
- **Traceability check**: `python3 tools/traceability.py` exits 0; every Unit-type test case in the per-sprint tables has a `// @{"verify": ["SW-TC-..."]}` tag in the corresponding `*_lib_test.cpp` file.

Only after the Wave 3 exit gate PASS can Wave 4 begin (`domain_libs.md`).

## 5. Cross-References

- [SDP Index](index.md)
- [Methodology](methodology.md)
- [Foundation Libraries (Wave 0/1/2)](foundation_libs.md)
- [Domain Libraries (Wave 4)](domain_libs.md)
- [Sensor Apps (Wave 5)](sensor_apps.md)
- [Domain Apps (Wave 5/6)](domain_apps.md)
- [Sim and Integration (Wave 7/8)](sim_and_integration.md)

L2 design references (each sprint's authoritative design):

- `docs/design/imu/design.md` — SPRINT-IMPL-07
- `docs/design/baro/design.md` — SPRINT-IMPL-08
- `docs/design/gps/design.md` — SPRINT-IMPL-09
- `docs/design/sd/design.md` — SPRINT-IMPL-11
- `docs/design/lora/design.md` — SPRINT-IMPL-10

Requirements and test case JSONs:

- `docs/requirements/imu/requirements.json`, `docs/test_cases/imu/test_cases.json`
- `docs/requirements/baro/requirements.json`, `docs/test_cases/baro/test_cases.json`
- `docs/requirements/gps/requirements.json`, `docs/test_cases/gps/test_cases.json`
- `docs/requirements/lora/requirements.json`, `docs/test_cases/lora/test_cases.json`
- `docs/requirements/sd/requirements.json`, `docs/test_cases/sd/test_cases.json`

LibJuno headers (authoritative per shape; cross-checked per lessons-learned 2026-05-03):

- `libjuno/include/juno/module.h` — `JUNO_MODULE_ROOT`, `JUNO_MODULE_DERIVE`
- `libjuno/include/juno/status.h` — 19 canonical status codes
- `libjuno/include/juno/time/time_api.hpp` — `TIME_ROOT_T::TimestampToMicros` member function (used by `baro_lib::Sample` caller-supplied timestamp path)
