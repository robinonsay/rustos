---
document_type: SDP — Domain Apps + System App (Wave 6+7)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: index.md
sprints_covered: SPRINT-IMPL-19 through SPRINT-IMPL-23
status: Draft (awaiting Chief Engineer gate)
---

# SDP — Wave 6+7: Domain Apps + System App

## 1. Purpose

This file plans the five implementation sprints that close the FSW vertical
between the message-bus publishers (sensor apps) and the system-level
orchestrator. Wave 6 (four sprints, SPRINT-IMPL-19..22) covers the four
domain View-layer apps that subscribe to sensor publications, drive their
respective Controller libraries, and publish derived bus messages
(`JUNO_MSG_NAV_STATE_T`, `JUNO_MSG_AFM_PHASE_T`) or terminate the data path
on radio / SD storage. Wave 7 (one sprint, SPRINT-IMPL-23) covers `sys_app`
— the system-level orchestrator that owns POST, the lifecycle state
machine (`juno::JUNO_FSW_STATE_T`), the per-sensor health bitmap, and the
operator LED. After Wave 6+7 closes, every Controller and View artifact
needed for a runnable Trick simulation exists; Wave 8
(`sim_and_integration.md`) can then compose every lib + app into the FT1
flight image and the POSIX-target Trick scenario.

## 2. Wave Summary

| Wave | Sprints | Modules | Predecessor Wave | Successor Waves |
|------|---------|---------|------------------|-----------------|
| 6 | SPRINT-IMPL-19..22 | nav_app, afm_app, telem_app, mlog_app | Wave 4 (domain libs), Wave 5 (sensor apps) | Wave 7, Wave 8 |
| 7 | SPRINT-IMPL-23 | sys_app | All Wave 5+6 (every lib + every other app) | Wave 8 |

Wave 6 sprints have an inter-sprint subscription DAG: `nav_app` first (only
subscribes to sensor messages); `afm_app` second (subscribes to NAV_STATE +
sensor messages); `telem_app` and `mlog_app` last (subscribe to NAV_STATE,
AFM_PHASE, and the full sensor set). Recommended serialization: **nav →
afm → telem → mlog**. The two terminal apps (telem_app, mlog_app) have no
mutual dependency and may be parallelized if reviewer/CE bandwidth is
available, but the Wave 6+7 exit gate requires all four CLOSED before
SPRINT-IMPL-23 begins. Wave 7 (`sys_app`) is the orchestrator and depends
on every published message produced by every other app, plus `sd_lib` and
`lora_lib` health-query verbs.

## 3. Per-Sprint Plans

### SPRINT-IMPL-19 — nav_app

- **Module**: `nav_app` (10 ms; subscribes IMU/baro/GPS samples; runs `nav_lib` EKF; publishes `JUNO_MSG_NAV_STATE_T`)
- **Predecessors**: SPRINT-IMPL-12 (nav_lib), SPRINT-IMPL-16 (imu_app), SPRINT-IMPL-17 (baro_app), SPRINT-IMPL-18 (gps_app)
- **L2 design**: `docs/design/nav_app/design.md`
- **Requirements**: 13 SW-REQ-NAV-APP-* IDs (`-001` … `-013`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/nav_app/include/nav_app/nav_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/nav_app/src/nav_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/nav_app/tests/nav_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/nav_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 15 Unit/Integration SW-TC-NAV-APP-* (`-001` … `-011`, `-013` … `-016`). One Demonstration (`-012`) is ground-test only and not in scope of this sprint's gate.
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - `NAV_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` single-level aggregate (per delta-PDR S10; no parallel `NAV_APP_ROOT_T` / `NAV_APP_API_T`).
  - Lifecycle hooks are static free functions in `nav_app.cpp` taking `juno::app::APP_ROOT_T &` and downcasting via `JUNO_MODULE_DERIVE` to `NAV_APP_T &`; vtable dispatch goes through `tRoot.ptApi->Hook(...)` (never `tApi->`).
  - Time-stamping uses canonical member-form: `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` (per delta-PDR Δ-MAJOR-5 fix; matches the form already in baro_app/imu_app/gps_app/afm_app).
  - `JUNO_MSG_NAV_STATE_T` publish references `nav/design.md` §4.1 authoritative field-shape table verbatim — no field renames; doubles throughout per `SW-REQ-SYS-040` / `SW-REQ-SYS-043`.
  - Subscriptions to IMU/baro/GPS happen in `OnStart` (per `conventions.md` §1.4), not at composition.
  - One `JUNO_MSG_NAV_STATE_T` published per `OnProcess` tick regardless of input availability (`SW-REQ-NAV-APP-010` / parent `SW-REQ-SYS-034`); `bValid` forwarded verbatim from `nav_lib::GetState`.
  - Tests inject a stub broker (delivers fake IMU/baro/GPS samples on demand) and a stub `nav_lib` (test vtable returning canned `NAV_STATE_T`); per-tick publish is verified by inspecting recorded broker `Publish` calls.
- **Test gate**: G1 (POSIX build + ctest) + G2 (`tools/traceability.py`).
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = **9 agents**.

### SPRINT-IMPL-20 — afm_app

- **Module**: `afm_app` (10 ms; subscribes `NAV_STATE` + sensor samples; runs `afm_lib` phase machine; publishes `JUNO_MSG_AFM_PHASE_T`)
- **Predecessors**: SPRINT-IMPL-13 (afm_lib), SPRINT-IMPL-19 (nav_app NAV_STATE publisher), SPRINT-IMPL-16/17/18 (sensor apps)
- **L2 design**: `docs/design/afm_app/design.md`
- **Requirements**: 10 SW-REQ-AFM-APP-* IDs (`-001` … `-010`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/afm_app/include/afm_app/afm_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/afm_app/src/afm_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/afm_app/tests/afm_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/afm_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 12 Unit/Integration SW-TC-AFM-APP-* (`-001` … `-012`). One Demonstration (`-013`, FT1 phase-accuracy) is post-flight only.
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - `AFM_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` single-level pattern; same hook-downcast and `ptApi->` dispatch invariants as nav_app.
  - Time-stamping uses canonical member-form `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk`.
  - Phase enum spelling matches `afm/design.md` and `SW-REQ-SYS-016` exactly: `pre-launch`, `boost`, `apogee`, `descent`, `landing` (per LL 2026-05-02 — cross-module enum source-of-truth).
  - Tests inject a stub broker + stub `nav_lib` (publishes a canned `JUNO_MSG_NAV_STATE_T` timeline per tick) + stub `afm_lib` (test vtable returning canned phase / transition timestamps); the canned `NAV_STATE` timeline drives the SW-TC-AFM-APP-012 phase-monotonicity test.
  - Per-tick `JUNO_MSG_AFM_PHASE_T` publish (`SW-REQ-AFM-APP-005`) and per-transition timestamp publish (`SW-REQ-AFM-APP-006`) verified via broker `Publish` recording.
  - Fault-isolation test (SW-TC-AFM-APP-008) confirms `OnProcess` returning a non-success status does not block peer apps under the scheduler harness.
- **Test gate**: G1 + G2.
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = **9 agents**.

### SPRINT-IMPL-21 — telem_app

- **Module**: `telem_app` (500 ms; subscribes everything; encodes via `telem_lib`; transmits via `lora_lib`; backpressure via `lora_lib::IsBusy()`)
- **Predecessors**: SPRINT-IMPL-14 (telem_lib), SPRINT-IMPL-10 (lora_lib), SPRINT-IMPL-19 (NAV_STATE), SPRINT-IMPL-20 (AFM_PHASE), SPRINT-IMPL-16/17/18 (sensor apps)
- **L2 design**: `docs/design/telem_app/design.md`
- **Requirements**: 11 SW-REQ-TELEM-APP-* IDs (`-001` … `-011`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/telem_app/include/telem_app/telem_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/telem_app/src/telem_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/telem_app/tests/telem_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/telem_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 11 Unit/Integration SW-TC-TELEM-APP-* (`-001` … `-005`, `-007` … `-009`, `-011` … `-013`). Two Demonstrations (`-006` continuous coverage, `-010` post-landing beacon) are bench/HW only and not in this sprint's gate.
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - `TELEM_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` single-level pattern.
  - Time-stamping uses canonical member-form `_ptTime->TimestampToMicros(...)`.
  - **Backpressure**: per-tick check via stub `lora_lib::IsBusy()` returning `true`; verifies `telem_app` does not invoke `lora_lib::Transmit()` while busy and does not stall the scheduler slot.
  - **CRC golden vector**: the CRC byte ranges produced by `telem_lib::Compose()` are reused verbatim from `telem_lib`'s sprint test vectors (no re-derivation in this sprint).
  - **Continue after transmit failure** (SW-REQ-TELEM-APP-007 / SW-TC-TELEM-APP-007): stub `lora_lib::Transmit` returns failure on cycle N; the next 5 ticks invoke `Transmit` again with no skip / no backoff.
  - **Radio-health publish** every cycle (SW-TC-TELEM-APP-008/-013): broker records exactly one `JUNO_MSG_*_RADIO_HEALTH_*` publish per tick whose status field tracks the prior tick's `Transmit` outcome.
  - Tests follow LL 2026-05-02 setup-completeness rule: every setup enumerates the full DI triple (broker stub + telem_lib stub + lora_lib stub + time stub).
- **Test gate**: G1 + G2.
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = **9 agents**.

### SPRINT-IMPL-22 — mlog_app

- **Module**: `mlog_app` (5 ms — matches IMU rate per delta-PDR S1-AI-005 cascade fix and `SW-REQ-SYS-011` no-downsampling; subscribes everything; encodes via `mlog_lib`; writes via `sd_lib`)
- **Predecessors**: SPRINT-IMPL-15 (mlog_lib), SPRINT-IMPL-11 (sd_lib), SPRINT-IMPL-16/17/18 (sensor apps), SPRINT-IMPL-19 (NAV_STATE), SPRINT-IMPL-20 (AFM_PHASE)
- **L2 design**: `docs/design/mlog_app/design.md`
- **Requirements**: 12 SW-REQ-MLOG-APP-* IDs (`-001` … `-012`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/mlog_app/include/mlog_app/mlog_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/mlog_app/src/mlog_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/mlog_app/tests/mlog_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/mlog_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 14 Unit/Integration SW-TC-MLOG-APP-* (`-001` … `-014`). No Demonstrations in scope.
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - `MLOG_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` single-level pattern.
  - **5 ms slot completion**: stub `sd_lib::WriteBlock` measures call latency; per-tick total drain + encode + write completes within the 5 ms TDM slot for the worst-case drain count.
  - **No downsampling** (SW-REQ-SYS-011 / SW-REQ-MLOG-APP-002 / SW-TC-MLOG-APP-002): every IMU sample queued on the broker results in exactly one encoded NAV/IMU mlog record at the IMU 5 ms cadence; the test publishes 100 IMU samples and asserts the stub `mlog_lib::EncodeImu` recorded exactly 100 calls in publish order.
  - **NewRun before any forward** (SW-TC-MLOG-APP-003/-013): stub `mlog_lib` records call ordering; the first recorded call is `NewRun()`, every `Encode*` call follows.
  - **Per-record monotonic-µs timestamps** (SW-REQ-MLOG-APP-005/-006): stub time source seeded with a known monotonic sequence; captured timestamps equal the seeded sequence exactly.
  - **Continue after SD failure** (SW-TC-MLOG-APP-009): stub `sd_lib::WriteBlock` returns failure on call N; subsequent calls still attempted; `mlog_app` does not abort.
  - **Zero bus publication** (PM Decision 3): test asserts the broker stub's `Publish` recorder is empty after N ticks (mlog_app is pure sink; SD health is polled by sys_app via `sd_lib::IsHealthy`).
- **Test gate**: G1 + G2.
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = **9 agents**.

### SPRINT-IMPL-23 — sys_app

- **Module**: `sys_app` (100 ms; POST + lifecycle state machine + health bitmap aggregation + operator LED + SD/LoRa health polling)
- **Predecessors**: All Wave 5+6 sprints + every Wave 0..4 lib (subscribes to every other app's published messages; polls `sd_lib::IsHealthy` and `lora_lib::IsHealthy` directly; drives LED via `device_lib`)
- **L2 design**: `docs/design/sys_app/design.md` — post-delta-PDR S10 restructured: single-level `SYS_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` (canonical `_T` suffix matches the other 7 apps per `conventions.md` §3; the prior pre-delta-PDR `SYS_APP` two-level UB pattern is eliminated and must not return).
- **Requirements**: 12 SW-REQ-SYS-APP-* IDs (`-001` … `-012`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/sys_app/include/sys_app/sys_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/sys_app/src/sys_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/sys_app/tests/sys_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/sys_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 10 Unit/Integration SW-TC-SYS-APP-* (`-001` … `-007`, `-010`, `-011`, `-014`). Four Demonstrations (`-008` LED green, `-009` LED error, `-012` power-on, `-013` endurance) are HW/bench only.
- **Acceptance criteria**: per `methodology.md` §8 (1-9), plus:
  - **Canonical aggregate**: `SYS_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, …)` — single-level only. Any return of the prior two-level `SYS_APP` ROOT-wrapper pattern is a regression of PDR-RID-S10-004 and a hard-fail at review.
  - **Health bitmap**: declare the six bit constants per `sys_app/design.md` §4.3 authoritative table:
    - `kHealthBitImu   = 1u << 0` (bit 0, imu_lib via imu_app)
    - `kHealthBitBaro  = 1u << 1` (bit 1, baro_lib via baro_app)
    - `kHealthBitGps   = 1u << 2` (bit 2, gps_lib via gps_app)
    - `kHealthBitSd    = 1u << 3` (bit 3, sd_lib polled directly via `sd_lib::IsHealthy()`)
    - `kHealthBitRadio = 1u << 4` (bit 4, lora_lib polled directly via `lora_lib::IsHealthy()`)
    - `kHealthBitNav   = 1u << 5` (bit 5, nav_app via NAV_STATE.bValid)
    - bits 6..31 reserved (zero) per the §4.3 table.
  - **Lifecycle**: consume `juno::JUNO_FSW_STATE_T` from `conventions.md` §4.7 verbatim — values `JUNO_FSW_STATE_POST`, `JUNO_FSW_STATE_INIT`, `JUNO_FSW_STATE_RUN`, `JUNO_FSW_STATE_SAFE`, `JUNO_FSW_STATE_RECOVERY`. **No parallel local enum** in `sys_app`.
  - **POST in `OnStart`**: exercise every sensor lib's POST verb once per `sys_app/design.md` §2 verb table — `imu_lib::PowerOnSelfTest()`, `baro_lib::Probe()`, `gps_lib::Probe()`, `lora_lib::Probe()`, `sd_lib::Mount()`. Per-sensor pass/fail recorded into `tPostResult.u32PostBitmap`; one `JUNO_MSG_SYS_POST_T` published once at startup; no further POST publications on subsequent ticks (SW-TC-SYS-APP-001/-002/-003/-004/-005).
  - **LED bit pattern**: drives onboard LED per `sys_app/design.md` §5.2 table — solid green when `u32HealthBitmap == 0` (all-healthy); error pattern otherwise (SW-REQ-SYS-APP-007/-008).
  - **No reboot / no watchdog kick** (SW-REQ-SYS-APP-009 / SW-TC-SYS-APP-010): `apps/sys_app/` source contains no calls to reboot, reset, watchdog, `abort()`, `exit()`, or RP2350 reset/watchdog register writes; verified by grep-style inspection test.
  - **Test coverage** must cover (a) clean `POST → INIT → RUN`; (b) one bit set on `bValid=false` from a sensor → `RUN → SAFE` transition; (c) bit cleared → `SAFE → RUN`; (d) `JUNO_MSG_AFM_PHASE_T.ePhase == landing` → `RUN → RECOVERY`; (e) `sys_app` never invokes reboot/watchdog (the inspection above).
  - Tests follow LL 2026-05-02 setup-completeness rule: setup enumerates the broker, time, every sensor lib stub (5 libs × stub), `device_lib` stub for LED, and the failure handler.
  - POSIX/Pico2 functional equivalence test (SW-TC-SYS-APP-014) replays a recorded mock-sensor scenario through both targets and field-compares POST result, SD log record, and a 100-tick window of `JUNO_MSG_SYS_HEALTH_T` publishes.
- **Test gate**: G1 + G2.
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = **9 agents**.

## 4. Wave Exit Gate

After SPRINT-IMPL-23 closes, the **Wave 6+7 Exit Gate** spawned by the
Software Lead engages the project Chief Engineer to confirm:

- All 5 sprints (SPRINT-IMPL-19..23) closed with G1 (POSIX build + ctest)
  and G2 (`tools/traceability.py`) both exit-0.
- **Health-bitmap consistency**: every app that sets or clears a bit cites
  `sys_app/design.md` §4.3 by name (no duplicated literal `1u<<N`
  expressions outside `sys_app`). A grep over `apps/` confirms only
  `sys_app` defines the `kHealthBit*` constants.
- **Lifecycle-state consistency**: every app that references the FSW
  lifecycle uses `juno::JUNO_FSW_STATE_T` from `conventions.md` §4.7. A
  grep over `apps/` confirms no parallel `LIFECYCLE_T` / `FSW_STATE_T`
  enums exist outside the canonical declaration.
- **Single-level `JUNO_MODULE_DERIVE`**: every app aggregate
  (`NAV_APP_T`, `AFM_APP_T`, `TELEM_APP_T`, `MLOG_APP_T`, `SYS_APP_T`)
  uses the single-level pattern from `conventions.md` §1.4 with
  `juno::app::APP_ROOT_T tRoot` as first member; no parallel
  `<APP>_APP_ROOT_T` / `<APP>_APP_API_T` redeclarations.
- **Burndown delta**: `python3 tools/burndown.py` shows the expected
  closure of SW-REQ-NAV-APP, -AFM-APP, -TELEM-APP, -MLOG-APP, -SYS-APP
  Active → Verified for every Test/Inspection-method requirement; the
  remaining Demonstration-method requirements (`SW-REQ-NAV-APP-011`,
  `-TELEM-APP-006`, `-TELEM-APP-010`, `-AFM-APP-006` demo, `-SYS-APP-007`,
  `-SYS-APP-008`, `-SYS-APP-010`, `-SYS-APP-011`) remain Active pending
  ground-test / FT1 flight execution.

Only after the Wave 6+7 exit gate PASSes can Wave 8 begin
(`sim_and_integration.md`, SPRINT-IMPL-24..25 — sim modules + system
integration / Trick scenario).

## 5. Cross-References

- [SDP Index](index.md)
- [Methodology](methodology.md)
- [Foundation Libraries](foundation_libs.md)
- [Sensor Libraries](sensor_libs.md)
- [Domain Libraries](domain_libs.md)
- [Sensor Apps](sensor_apps.md)
- [Sim & Integration](sim_and_integration.md)
