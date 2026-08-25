---
document_type: SDP — Sensor Apps (Wave 5)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: index.md
sprints_covered: SPRINT-IMPL-16 through SPRINT-IMPL-18
status: Draft (awaiting Chief Engineer gate)
---

# SDP — Wave 5: Sensor Apps

## 1. Purpose

Wave 5 lands the three sensor-publishing applications (`imu_app`, `baro_app`,
`gps_app`) in three independently-schedulable sprints. Each app is a thin,
TDM-scheduled View-layer publisher with **no business logic**: per-tick work
delegates to the corresponding Wave 3 sensor library, attaches a monotonic-µs
timestamp, and publishes a typed message onto the LibJuno software broker.
After Wave 5 closes, every sensor stream is observable on the bus, enabling
Wave 6 domain apps (`nav_app`, `afm_app`, `telem_app`, `mlog_app`) to
subscribe and consume those messages without further sensor-app rework.

## 2. Wave Summary

| Wave | Sprints | Modules | Predecessor Wave | Successor Waves |
|------|---------|---------|------------------|-----------------|
| 5 | SPRINT-IMPL-16..18 | imu_app, baro_app, gps_app | Wave 0 (`JUNO_MSG_BUS_VARIANT_T` + `juno_fsw_capacities.hpp`), Wave 2 (`sch_lib`, `time_lib`, `device_lib`), Wave 3 (per-app sensor lib) | Wave 6 (domain apps), Wave 8 (system integration) |

All three Wave 5 sprints can run **in parallel** once their per-lib
predecessor (Wave 3 sensor lib) plus the Wave 0 (`bus_variant + capacities`)
and Wave 2 (`sch_lib`/`time_lib`) enablers are closed. The Lead must brief
every Wave 5 worker with the **current `juno::app::AppInit` LibJuno
publication status** (see RFA carry-forward below).

### 2.1 Carry-forward RFAs (apply to all three Wave 5 sprints)

1. **`juno::app::AppInit(...)` RFA #1** — Each Wave 5 sprint must check
   whether LibJuno has published `juno::app::AppInit` by the sprint's start.
   - If **yes**: the per-app `<App>AppInit` setup function calls
     `juno::app::AppInit(tApp.tRoot, tApi, pfcnFailureHandler, pvUserData)`
     directly (see L2 design §4 in each module's `design.md`).
   - If **no**: the per-app `<App>AppInit` setup function manually
     aggregate-inits `tApp.tRoot` (set `tApp.tRoot.ptApi = &tApi;`,
     `tApp.tRoot.pfcnFailureHandler = pfcnFailureHandler;`,
     `tApp.tRoot.pvUserData = pvUserData;`) instead of calling the
     unpublished function.
   - The Lead briefs the worker with the current LibJuno status; the chosen
     path is documented in the sprint closure record.
2. **`JUNO_MSG_BUS_VARIANT_T`** — defined in Wave 0 (`SPRINT-IMPL-00`);
   consumed verbatim by every Wave 5 broker template instantiation.
3. **`kBrokerPipes` / `kBrokerRegistry`** — defined in Wave 0
   (`apps/include/juno_fsw_capacities.hpp`); consumed as the second/third
   template parameters of `juno::sb::BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T,
   kBrokerPipes, kBrokerRegistry>`.

### 2.2 Cross-sprint conventions (apply uniformly)

- **App aggregate**: `<MODULE>_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T,
  ...)` — single-level pattern from the sys_app delta-PDR remediation
  (2026-05-03). The L2 designs confirm `IMU_APP_T`, `BARO_APP_T`, and
  `GPS_APP_T` all follow this pattern with `juno::app::APP_ROOT_T tRoot;` as
  the first member.
- **Hook signatures**: `static JUNO_STATUS_T <App>_OnStart/OnProcess/OnExit
  (juno::app::APP_ROOT_T &tRoot) noexcept` — file-scope statics in the `.cpp`.
- **Hook downcast**: `auto &tApp = *reinterpret_cast<<APP>_T*>(&tRoot);` —
  layout-compatible because `<APP>_T`'s first member is `tRoot` via
  `JUNO_MODULE_SUPER` (`libjuno/include/juno/module.h:131`). `JUNO_MODULE_DERIVE`
  is composition (first-member embedding), not C++ inheritance, so
  `static_cast` is invalid; the canonical pattern is `*reinterpret_cast<<APP>_T*
  >(&tRoot)` per `sch_test_helpers.hpp:108` (2026-05-10 SPRINT-IMPL-16 lesson).
- **Per-tick timestamp**: `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(
  *_ptTime).tOk).tOk` — member-function form per the delta-PDR Δ-MAJOR-5 fix
  (`libjuno/include/juno/time/time_api.hpp:142`). Never invent a free
  `juno::time::TimestampToMicros(...)`.
- **Vtable wired**: `static const juno::app::APP_API_T tApi { &<App>_OnStart,
  &<App>_OnProcess, &<App>_OnExit };` — file-scope `static const` local
  inside `<App>AppInit`. This is the **sole** file-scope datum in each app's
  `.cpp` translation unit.
- **Setup function**: free function
  `<App>AppInit(<APP>_T &tApp, /* DI refs */, JUNO_FAILURE_HANDLER_T
  pfcnFailureHandler, JUNO_USER_DATA_T *pvUserData) noexcept` — see RFA #1
  for branch-on-LibJuno behavior.
- **Vtable dispatch**: `tRoot.ptApi->Hook(...)` — never `tApi->...` or
  `tRoot.tApi->...`. The pointer name is `ptApi` per `JUNO_MODULE_ROOT`.
- **Broker template instantiation**:
  `juno::sb::BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T, kBrokerPipes,
  kBrokerRegistry>` — capacities resolved from
  `apps/include/juno_fsw_capacities.hpp` (Wave 0).

### 2.3 Test gate boilerplate (per [methodology.md](methodology.md) §9)

```bash
mkdir -p build_posix && cd build_posix && cmake -DPLATFORM=POSIX .. \
    && cmake --build . && ctest --output-on-failure
python3 tools/traceability.py
```

Apps are platform-agnostic per the L2 designs (`imu_app/design.md` §4.6,
`baro_app/design.md` §3.3, `gps_app/design.md` §11), so Gate G3 (Pico2
cross-compile) is **not invoked from the app sprint itself** — the Wave 8
system-integration sprint (`SPRINT-IMPL-25`) compiles `apps/main_pico2.cpp`
against the linked app objects and runs G3 there. If a Wave 5 worker is
forced to add a `pico2`-specific TU (not expected per the L2 designs), the
Lead notes the deviation in the sprint record and adds G3 explicitly.

## 3. Per-Sprint Plans

### SPRINT-IMPL-16 — imu_app

- **Module**: `imu_app` (5 ms / 200 Hz publisher; consumes `imu_lib`)
- **Predecessors**: SPRINT-IMPL-00 (`bus_variant + capacities`), SPRINT-IMPL-03
  (`time_lib`), SPRINT-IMPL-06 (`sch_lib`), SPRINT-IMPL-07 (`imu_lib`)
- **L2 design**: `docs/design/imu_app/design.md`
- **Canonical type**: `IMU_APP_T` (single-level
  `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` per L2 §3.3 line 102;
  embeds `juno::app::APP_ROOT_T tRoot;` as first member)
- **Period constant**: `static constexpr uint32_t kImuAppPeriodMs = 5;`
  (200 Hz; L2 §3.3 line 100; `SW-REQ-SYS-005`)
- **Requirements**: 10 × `SW-REQ-IMU-APP-001` through `SW-REQ-IMU-APP-010`
  (`docs/requirements/imu_app/requirements.json`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/imu_app/include/imu_app/imu_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/imu_app/src/imu_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/imu_app/tests/imu_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/imu_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 12 × `SW-TC-IMU-APP-001` through `SW-TC-IMU-APP-012` from
  `docs/test_cases/imu_app/test_cases.json`. All twelve are `Unit`/
  `Integration` type with `google_test_ref:
  apps/imu_app/tests/imu_app_test.cpp`; the test-author worker implements
  every one.
- **Test stub strategy**: Inject stub `imu_lib` (vtable of test functions
  matching `juno::imu::IMU_LIB_API_T`) plus a fake `juno::time::TIME_ROOT_T`
  with controllable `Now`/`TimestampToMicros` and a recording broker that
  captures `JUNO_MSG_IMU_SAMPLE_T` publishes. Per the lessons-learned
  2026-05-02 entry, every test setup must enumerate **all** dependencies the
  app pulls from the composition root (scheduler driver, broker, imu stub,
  time stub) — not just the focal collaborator.
- **Acceptance criteria**: per [methodology.md](methodology.md) §8 (1–11),
  plus the following imu_app-specific items:
  - **AC-A**: `IMU_APP_T` declared via
    `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` — single-level pattern
    (delta-PDR S10 closure); first member is `juno::app::APP_ROOT_T tRoot;`
  - **AC-B**: Hook downcast uses `auto &tApp = static_cast<IMU_APP_T&>(
    tRoot);` — layout-compatible per `JUNO_MODULE_SUPER`
  - **AC-C**: Vtable dispatch in `_pt*->ptApi->Hook(...)` form — never
    `tApi->`
  - **AC-D**: Per-tick timestamp via `_ptTime->TimestampToMicros(
    _ptTime->ptApi->Now(*_ptTime).tOk).tOk` (member-form,
    `libjuno/include/juno/time/time_api.hpp:142`)
  - **AC-E**: `ImuAppInit` follows RFA #1 — uses `juno::app::AppInit` if
    LibJuno has published it, otherwise manual aggregate-init of `tApp.tRoot`
  - **AC-F**: Per-tick `Sample` call **and** broker `Publish` of
    `JUNO_MSG_IMU_SAMPLE_T` are both observable from the recording subscriber
    (verifies `SW-REQ-IMU-APP-002` + `SW-REQ-IMU-APP-003`)
  - **AC-G**: 5 ms period constant `kImuAppPeriodMs = 5` is a
    `static constexpr uint32_t` in the public header
  - **AC-H**: `OnStart` calls `imu_lib::Configure(±16 G, ±2000 dps)` and
    success constitutes the verification evidence for `SW-REQ-IMU-APP-009`
    and `SW-REQ-IMU-APP-010` (per L2 §4.2)
- **Test gate**: G1 + G2. (G3 deferred to `SPRINT-IMPL-25`; this app has no
  pico2-specific TU per L2 §4.6.)
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = 9 agents.

### SPRINT-IMPL-17 — baro_app

- **Module**: `baro_app` (50 ms / 20 Hz publisher; consumes `baro_lib`)
- **Predecessors**: SPRINT-IMPL-00 (`bus_variant + capacities`), SPRINT-IMPL-03
  (`time_lib`), SPRINT-IMPL-06 (`sch_lib`), SPRINT-IMPL-08 (`baro_lib`)
- **L2 design**: `docs/design/baro_app/design.md`
- **Canonical type**: `BARO_APP_T` (single-level
  `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` per L2 §3.3 / §4.1;
  embeds `juno::app::APP_ROOT_T tRoot;` as first member)
- **Period constant**: `static constexpr uint32_t kBaroAppPeriodMs = 50;`
  (20 Hz; L2 §4.1 line 107; `SW-REQ-SYS-008`)
- **Requirements**: 10 × `SW-REQ-BARO-APP-001` through `SW-REQ-BARO-APP-010`
  (`docs/requirements/baro_app/requirements.json`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/baro_app/include/baro_app/baro_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/baro_app/src/baro_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/baro_app/tests/baro_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/baro_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 14 × `SW-TC-BARO-APP-001` through `SW-TC-BARO-APP-014`
  from `docs/test_cases/baro_app/test_cases.json`. Twelve are
  `Unit`/`Integration` type with `google_test_ref:
  apps/baro_app/tests/baro_app_test.cpp`; two (`SW-TC-BARO-APP-008` and
  `-009`) are `Demonstration` type with `google_test_ref: null` and
  `expected_artifacts` for signed inspection records (HAE altitude
  documentation and SI units documentation respectively) — those two
  Demonstration test cases are deferred to the HIL CDR phase per
  [index.md](index.md) §10.4 and are **not** implemented in this sprint.
- **Test stub strategy**: Inject stub `baro_lib` exposing a programmable
  sample sequence and an injectable health bitmap, a fake
  `juno::time::TIME_ROOT_T` with controllable µs cadence, and a recording
  broker capturing `JUNO_MSG_BARO_SAMPLE_T` publishes. Per L2 §4.1, the
  broker advertise/subscriber wiring happens in `BaroApp_OnStart`, **not**
  in `BaroAppInit`; tests must invoke `OnStart` before any `OnProcess`.
- **`BARO_LIB_BUS_T` callback wiring (delta-PDR Δ-MINOR-1 wording fix)**:
  the L2 `baro_lib` design owns the I²C transport callback type
  (`BARO_LIB_BUS_T`) inside the `baro_lib::BARO_LIB_IMPL_T::New()`
  composition root — **not** inside `baro_app`. The `baro_app` test fixture
  therefore receives a fully-constructed `juno::baro::BARO_LIB_ROOT_T &`
  reference and never sees the I²C transport directly. AC-I below makes this
  explicit so the worker does not introduce a transport seam in the app.
- **Acceptance criteria**: per [methodology.md](methodology.md) §8 (1–11),
  plus the following baro_app-specific items:
  - **AC-A**: `BARO_APP_T` declared via
    `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` — single-level pattern;
    first member is `juno::app::APP_ROOT_T tRoot;`
  - **AC-B**: Hook downcast uses `auto &tApp = static_cast<BARO_APP_T&>(
    tRoot);` — layout-compatible per `JUNO_MODULE_SUPER`
  - **AC-C**: Vtable dispatch in `_pt*->ptApi->Hook(...)` form — never
    `tApi->`
  - **AC-D**: Per-tick timestamp via `_ptTime->TimestampToMicros(
    _ptTime->ptApi->Now(*_ptTime).tOk).tOk` (member-form,
    `libjuno/include/juno/time/time_api.hpp:142`)
  - **AC-E**: `BaroAppInit` follows RFA #1 — uses `juno::app::AppInit` if
    LibJuno has published it, otherwise manual aggregate-init of `tApp.tRoot`
  - **AC-F**: Per-tick `Sample` call followed by `Publish` of
    `JUNO_MSG_BARO_SAMPLE_T`; the sample-precedes-publish ordering is
    explicitly verified per `SW-TC-BARO-APP-013`
  - **AC-G**: 50 ms period constant `kBaroAppPeriodMs = 50` is a
    `static constexpr uint32_t` in the public header
  - **AC-H**: `OnStart` performs broker advertise of
    `JUNO_MSG_BARO_SAMPLE_T` (per L2 §4.3); tests verify failure path leaves
    `_eState == UNINITIALIZED`
  - **AC-I**: `baro_app` source contains **no** I²C transport seam —
    `BARO_LIB_BUS_T` is owned by `baro_lib::BARO_LIB_IMPL_T::New()`
    upstream; the app receives an already-constructed `BARO_LIB_ROOT_T &`
    via `BaroAppInit` (delta-PDR Δ-MINOR-1 wording)
  - **AC-J**: Two Demonstration-type test cases (`SW-TC-BARO-APP-008`,
    `SW-TC-BARO-APP-009`) are explicitly **deferred** in the sprint record
    with the inspection-record placeholder filenames recorded for HIL CDR
- **Test gate**: G1 + G2. (G3 deferred to `SPRINT-IMPL-25`.)
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = 9 agents.

### SPRINT-IMPL-18 — gps_app

- **Module**: `gps_app` (200 ms / 5 Hz publisher; consumes `gps_lib`)
- **Predecessors**: SPRINT-IMPL-00 (`bus_variant + capacities`), SPRINT-IMPL-03
  (`time_lib`), SPRINT-IMPL-06 (`sch_lib`), SPRINT-IMPL-09 (`gps_lib`)
- **L2 design**: `docs/design/gps_app/design.md`
- **Canonical type**: `GPS_APP_T` (single-level
  `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` per L2 §3.3; embeds
  `juno::app::APP_ROOT_T tRoot;` as first member). Per the delta-PDR S10
  closure the canonical name is `GPS_APP_T` (it was `GPS_APP` pre-delta-PDR;
  workers must use `GPS_APP_T` verbatim).
- **Period constant**: `static constexpr uint32_t kGpsAppPeriodMs = 200;`
  (5 Hz; L2 §3.3 line 94; `SW-REQ-SYS-009`)
- **Requirements**: 10 × `SW-REQ-GPS-APP-001` through `SW-REQ-GPS-APP-010`
  (`docs/requirements/gps_app/requirements.json`)
- **Files to produce** (4 files):
  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/gps_app/include/gps_app/gps_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/gps_app/src/gps_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `apps/gps_app/tests/gps_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `apps/gps_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
- **Test cases**: 12 × `SW-TC-GPS-APP-001` through `SW-TC-GPS-APP-012` from
  `docs/test_cases/gps_app/test_cases.json`. All twelve are `Unit` type with
  `google_test_ref: apps/gps_app/tests/gps_app_test.cpp`; the test-author
  worker implements every one.
- **Test stub strategy** (POSIX-host openpty seam): per L2 §4.6 and the
  `gps_lib` design, `gps_lib` reads from the **slave** end of an `openpty()`
  pty pair on POSIX; the test fixture writes scripted NMEA sentence bytes
  to the **master** end. The fixture wires:
  - A `juno::gps::GPS_LIB_ROOT_T &` constructed against the pty slave fd
    (one fixture per test case; `gps_lib::GPS_LIB_IMPL_T::New()` accepts the
    fd via its DI seam).
  - A fake `juno::time::TIME_ROOT_T` with controllable
    `Now`/`TimestampToMicros`.
  - A recording broker capturing `JUNO_MSG_GPS_FIX_T`,
    `JUNO_MSG_GPS_NMEA_RAW_T`, and `JUNO_MSG_GPS_UTC_T` publishes
    separately (three topics).
  - The fixture writes deterministic NMEA byte streams (`$GPRMC`/`$GPGGA`
    sentences with known lat/lon/HAE/UTC) to the master fd before
    invoking `OnProcess`; the per-tick call sequence
    `Poll → GetRawNmea → GetFix → GetUtc` is exercised against the
    pre-loaded byte stream.
- **Acceptance criteria**: per [methodology.md](methodology.md) §8 (1–11),
  plus the following gps_app-specific items:
  - **AC-A**: `GPS_APP_T` (NOT `GPS_APP`) declared via
    `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` — single-level pattern
    (delta-PDR S10 closure); first member is `juno::app::APP_ROOT_T tRoot;`
  - **AC-B**: Hook downcast uses `auto &tApp = static_cast<GPS_APP_T&>(
    tRoot);` — layout-compatible per `JUNO_MODULE_SUPER`
  - **AC-C**: Vtable dispatch in `_pt*->ptApi->Hook(...)` form — never
    `tApi->`
  - **AC-D**: Per-tick timestamp via `_ptTime->TimestampToMicros(
    _ptTime->ptApi->Now(*_ptTime).tOk).tOk` (member-form,
    `libjuno/include/juno/time/time_api.hpp:142`)
  - **AC-E**: `GpsAppInit` follows RFA #1 — uses `juno::app::AppInit` if
    LibJuno has published it, otherwise manual aggregate-init of `tApp.tRoot`
  - **AC-F**: Per-tick `Poll → GetRawNmea → GetFix → GetUtc` call sequence
    (per L2 §6.1.1) followed by up to three broker `Publish` calls
    (`JUNO_MSG_GPS_FIX_T`, `JUNO_MSG_GPS_NMEA_RAW_T`, optional
    `JUNO_MSG_GPS_UTC_T`)
  - **AC-G**: 200 ms period constant `kGpsAppPeriodMs = 200` is a
    `static constexpr uint32_t` in the public header
  - **AC-H**: `OnStart` calls `gps_lib::Probe` and translates failure into
    a POST-bitmap bit (per L2 §4.2); `Probe` failure must **not** halt
    composition (`SW-REQ-SYS-029` / `-058`)
  - **AC-I**: gps_app source contains **no** direct `nmea_lib` dependency —
    NMEA parsing is delegated transitively through `gps_lib` per L2 §11
    `SW-REQ-GPS-APP-003` row (verified by `SW-TC-GPS-APP-003` source
    inspection)
  - **AC-J**: POSIX test fixture uses an `openpty()` pty seam: tests write
    scripted NMEA bytes to the master end and `gps_lib` (constructed
    against the slave fd) reads them — no direct UART access in the fixture
- **Test gate**: G1 + G2. (G3 deferred to `SPRINT-IMPL-25`; `gps_app`
  itself is platform-agnostic per L2 §11. The pty seam is a POSIX-host
  test artifact, not a pico2 build concern.)
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = 9 agents.

## 4. Wave Exit Gate

After SPRINT-IMPL-18 closes, the **Wave 5 Exit Gate** is held. The Lead
spawns a `project-chief-engineer` agent to confirm:

1. All three Wave 5 sprints (`SPRINT-IMPL-16`, `SPRINT-IMPL-17`,
   `SPRINT-IMPL-18`) are **CLOSED** with G1 (POSIX build + ctest) and G2
   (`tools/traceability.py`) both exit-0.
2. All three apps consistently use the **single-level
   `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)`** pattern from the
   sys_app delta-PDR remediation (2026-05-03). No two-level UB pattern
   appears anywhere in `apps/{imu,baro,gps}_app/`.
3. All three apps reference `tRoot.ptApi->Hook(...)` for vtable dispatch —
   never `tApi->...` or `tRoot.tApi->...`. The pointer name is `ptApi` per
   `JUNO_MODULE_ROOT` (`libjuno/include/juno/module.h:97`).
4. Broker template instantiations are consistent across all three apps:
   `juno::sb::BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T, kBrokerPipes,
   kBrokerRegistry>` resolves identically against
   `apps/include/juno_fsw_capacities.hpp` (Wave 0).
5. Per-tick timestamps in all three apps use the canonical member-form
   `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` per
   `libjuno/include/juno/time/time_api.hpp:142`. No fabricated free-function
   `juno::time::TimestampToMicros(...)` form anywhere.
6. The `juno::app::AppInit` carry-forward path (RFA #1) is documented
   identically across all three sprint closure records (either all three
   used the LibJuno-published `AppInit`, or all three used the manual
   aggregate-init workaround — mixing is a divergence the CE must flag).
7. Burndown delta: the three-app `SW-REQ` closure delta matches the per-app
   counts (10 + 10 + 10 = 30 requirements moved to **Verified**); RTM
   regenerated (`tools/rtm.py`) shows every `SW-REQ-IMU-APP-*`,
   `SW-REQ-BARO-APP-*`, `SW-REQ-GPS-APP-*` ID linked to at least one tagged
   code function and at least one tagged passing test.
8. Two `Demonstration`-type baro_app test cases (`SW-TC-BARO-APP-008`,
   `SW-TC-BARO-APP-009`) recorded as **Deferred** with HIL CDR placeholder
   filenames in the SPRINT-IMPL-17 closure record — not failed, not closed.

Only after the Wave 5 Exit Gate issues a CE PASS may **Wave 6** begin
(see [domain_apps.md](domain_apps.md)). Wave 8 system integration
(`SPRINT-IMPL-25`) holds the first end-to-end Pico2 cross-compile gate
(G3) for the linked apps.

## 5. Cross-References

- [SDP Index](index.md)
- [Methodology](methodology.md)
- [Foundation Libraries (Wave 0+1+2)](foundation_libs.md)
- [Sensor Libraries (Wave 3)](sensor_libs.md)
- [Domain Libraries (Wave 4)](domain_libs.md)
- [Domain Apps (Wave 6+7)](domain_apps.md)
- [Sim and Integration (Wave 8)](sim_and_integration.md)
- L2 designs: `docs/design/imu_app/design.md`,
  `docs/design/baro_app/design.md`, `docs/design/gps_app/design.md`
- Requirements: `docs/requirements/{imu_app,baro_app,gps_app}/requirements.json`
- Test cases: `docs/test_cases/{imu_app,baro_app,gps_app}/test_cases.json`
- LibJuno headers: `libjuno/include/juno/app/app_api.hpp`,
  `libjuno/include/juno/sb/broker_api.hpp`,
  `libjuno/include/juno/time/time_api.hpp`,
  `libjuno/include/juno/module.h`
