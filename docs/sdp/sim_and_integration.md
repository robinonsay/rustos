---
document_type: SDP — Sim Modules + System Integration (Wave 8)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: index.md
sprints_covered: SPRINT-IMPL-24, SPRINT-IMPL-25
status: Draft (awaiting Chief Engineer gate)
---

# SDP — Wave 8: Sim Modules + System Integration

## 1. Purpose

This file plans two sprints. **SPRINT-IMPL-24** lands the four sim modules
(`sim_dynamics`, `sim_sensors`, `sim_scenario`, `sim_harness`) as one
coordinated sprint because their L2 designs interlock — `sim_dynamics`
produces `SIM_DYN_TRUTH_T` consumed by `sim_sensors`; `sim_scenario`
produces `SIM_SCENARIO_T` consumed by both; `sim_harness` wires all three
plus the FSW composition into a Trick S_define and drives the system on a
fixed step. **SPRINT-IMPL-25** lands the FSW composition root
(`apps/main.cpp`) and the first full-FSW Trick integration test that
exercises every prior sprint's deliverable end-to-end. After Wave 8
closes, the FT1 FSW is ready for HIL bring-up at the CDR-track level and
this file's exit gate doubles as the FSW SDP exit gate.

## 2. Wave Summary

| Wave | Sprint | Modules | Predecessor | Successor |
|------|--------|---------|-------------|-----------|
| 8 | SPRINT-IMPL-24 | `sim_dynamics` + `sim_sensors` + `sim_scenario` + `sim_harness` (coordinated) | All Wave 1-7 sprints | SPRINT-IMPL-25 |
| 8 | SPRINT-IMPL-25 | system integration (composition root + Trick smoke test) | SPRINT-IMPL-24 | HIL CDR phase (out of SDP scope) |

SPRINT-IMPL-24 is **one sprint covering 4 modules** because the sim
module interfaces are tightly coupled — splitting into 4 separate sprints
would force per-sprint cross-stubbing that's larger than the coordinated
work. PM-approved scope per the SDP-authoring sprint plan.

## 3. Per-Sprint Plans

### SPRINT-IMPL-24 — Sim Modules (coordinated)

- **Modules**: `sim_dynamics`, `sim_sensors`, `sim_scenario`, `sim_harness`
- **Predecessors**: All Wave 1-7 sprints (every lib + every app must
  exist before the sim harness can compose them)
- **Entry gate**: NASA Trick `exec_get_sim_time()` symbol verified
  present in the linked Trick distribution (carry-forward RFA #5;
  closes SDP-R-05). Lead runs `nm $(TRICK_LIB) | grep exec_get_sim_time`
  before authorizing Phase 1 worker fanout.
- **L2 designs**:
  - `docs/design/sim_dynamics/design.md`
  - `docs/design/sim_sensors/design.md`
  - `docs/design/sim_scenario/design.md`
  - `docs/design/sim_harness/design.md`
  - `docs/design/sim_harness/interfaces.md`
- **Requirements**: **50 SW-REQ-SIM-\* IDs** total
  (14 SIM-DYN + 14 SIM-SENS + 12 SIM-SCEN + 10 SIM-HARN)
- **Files to produce** (12 module files + 4 CMakeLists boilerplate;
  sim modules are POSIX-only with sim-specific build dependencies on
  Trick + yaml-cpp):

  | # | File path | Module | Worker | Reviewer |
  |---|-----------|--------|--------|----------|
  | 1 | `sim/sim_dynamics/include/sim_dynamics/sim_dynamics.hpp` | sim_dynamics | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `sim/sim_dynamics/src/sim_dynamics.cpp` | sim_dynamics | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `sim/sim_dynamics/tests/sim_dynamics_test.cpp` | sim_dynamics | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `sim/sim_sensors/include/sim_sensors/sim_sensors.hpp` | sim_sensors | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `sim/sim_sensors/src/sim_sensors.cpp` | sim_sensors | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 6 | `sim/sim_sensors/tests/sim_sensors_test.cpp` | sim_sensors | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 7 | `sim/sim_scenario/include/sim_scenario/sim_scenario.hpp` | sim_scenario | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 8 | `sim/sim_scenario/src/sim_scenario.cpp` | sim_scenario | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 9 | `sim/sim_scenario/tests/sim_scenario_test.cpp` | sim_scenario | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 10 | `sim/sim_harness/include/sim_harness/sim_harness.hpp` | sim_harness | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 11 | `sim/sim_harness/src/sim_harness.cpp` | sim_harness | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 12 | `sim/sim_harness/tests/sim_harness_test.cpp` | sim_harness | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |

  Plus 4 boilerplate `sim/<module>/CMakeLists.txt` files authored by
  `junior-software-engineer` (reviewed by `senior-software-engineer`).
  These are tracked as workers 13-16 but not counted in the per-module
  module ledger above.

- **Test cases**: **61 SW-TC-SIM-\* IDs** total (18 SIM-DYN + 16 SIM-SENS
  + 14 SIM-SCEN + 13 SIM-HARN); of which **36 are Unit-type** Google
  Test cases (16 SIM-DYN + 14 SIM-SENS + 6 SIM-SCEN + 0 SIM-HARN), 21
  are Integration-type, and 4 are Demonstration-type.

- **Acceptance criteria**: per `methodology.md` §8 (criteria 1-9),
  plus the following module-specific items:
  - **sim_dynamics**: `SIM_DYN_TRUTH_T` is a C++-only POD (not
    `extern "C"`) carrying `juno::afm::JUNO_PHASE_T ePhase` per
    delta-PDR Δ-AC-6 closure; source asserts
    `static_assert(std::is_trivially_copyable<SIM_DYN_TRUTH_T>::value, ...)`.
    All truth-state fields documented in SI units (m, m/s, rad/s,
    dimensionless quaternion); NED frame convention enforced
    (gravity along +Down, ground impact at Down ≥ 0).
  - **sim_sensors**: GPS injection routes through openpty master-fd
    `::write` (per delta-PDR Δ-AC-7 closure — **no fictional
    `device_lib::posix::Inject` symbol may appear**); IMU and baro use
    Option D `SIM_SENSORS_RAW_T` / `SIM_BARO_REGS_T` with
    `static_assert` layout cross-checks against the FSW driver-side
    register types; `kMaxDropouts = 8` matches
    `sim_scenario::kMaxDropouts` with `static_assert` cross-check
    (per delta-PDR Δ-MINOR-8 closure); IMU 16-bit signed quantization
    over configured ±16 G / ±2000 dps full scale.
  - **sim_scenario**: `SIM_SCENARIO_T` is a flat POD with no nested
    cfg substructures (per §4.3); the YAML loader wraps `yaml-cpp`
    in a single `try`/`catch` — this is the **only permitted
    exception handler in the codebase** (FSW remains exception-free).
    Default FT1 baseline scenario produces ~600 m apogee within ±50 m
    per SW-TC-SIM-SCEN-011.
  - **sim_harness**: composition uses canonical aggregate-init
    `juno::time::TIME_API_T tTrickTimeApi { TrickNow, TrickSleepTo,
    TrickSleep }` and `juno::time::TimeInit(tTime, tTrickTimeApi,
    nullptr, nullptr)` per delta-PDR Δ-AC-5 closure (no
    `JUNO_TIME_PROVIDER_T`, no
    `TIME_LIB_IMPL_T::New(pfcn, ...)` factory); `TickFsw` invokes
    `juno::sch::SCH_API_T<8, 200>::Execute(tSch)` (NOT the fictional
    `sch_lib::Run`) per delta-PDR Δ-MAJOR-2 closure;
    `interfaces.md` §4.3 step 4 documents the `double`→`float` sigma
    narrowing in transcoding per delta-PDR Δ-MINOR-10 closure.

- **Test gate**:
  - **G1**: POSIX build + ctest must pass with sim modules linked.
    Configure command:
    ```bash
    mkdir -p build_posix && cd build_posix && \
      cmake -DPLATFORM=POSIX -DJUNO_FSW_BUILD_SIM=ON .. && \
      cmake --build . && ctest --output-on-failure
    ```
    The CMake configuration must `find_package(Trick)` and
    `find_package(yaml-cpp)` and report missing-dep failures cleanly
    at configure time (not at link time).
  - **G2**: `python3 tools/traceability.py` exits 0; every
    `SW-REQ-SIM-*` ID has at least one code tag and one test tag.
  - **G3**: not applicable — sim modules are POSIX-only, no Pico2
    dual-impl.

- **Estimated agent count**: 16 workers (12 module files + 4
  CMakeLists) + 16 reviewers + 1 CE = **33 agents**. **This is the
  largest sprint in the SDP** — the Lead may sub-stage by module
  within Phase 1 (e.g., dynamics → sensors → scenario → harness, each
  with 4 parallel workers per module) to reduce concurrent-review
  load. Sub-staging is a Lead operational decision; the per-module
  acceptance criteria above remain authoritative regardless of
  staging.

### SPRINT-IMPL-25 — System Integration

- **Module**: composition root + first full-FSW Trick integration test
- **Predecessors**: SPRINT-IMPL-24 (all sim modules) + every prior
  sprint (every lib + every app must be CLOSED with G1+G2 PASS)
- **Entry gate**: All 24 prior sprints CLOSED;
  `python3 tools/burndown.py` reports the expected pre-integration
  closure level across the entire FSW. Carry-forward RFA #1
  (`juno::app::AppInit` FSW workaround) verified consistent across
  every per-app `<App>AppInit` call site by reading
  `apps/*/src/*_app.cpp` in pre-flight inspection.
- **L2 design**: `docs/design/system/system_design.md` §8.1
  (composition-root invariants and scheduler table population)
- **Files to produce** (3 files; the entire system needs only a
  composition root, an integration test, and a top-level CMakeLists
  tweak):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/main.cpp` (POSIX entry; instantiates every lib + every app, calls every per-app `<App>AppInit`, registers app `tRoot` pointers into `juno::sch::SCH_ROOT_T<8, 200>::tArrSchTable`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `tests/integration/full_fsw_test.cpp` (Google Test integration suite that drives a smoke scenario through `sim_harness` against the composition root) | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 3 | Top-level `CMakeLists.txt` updates (link composition root against every lib + every app + `sim_harness`; declare integration test target) | junior-software-engineer | senior-software-engineer (reviewer mode) |

  For FT1 SDP scope the POSIX-only composition root is sufficient; a
  separate `apps/main_pico2.cpp` for flight composition is its own
  subsequent sprint outside the FSW SDP and is not produced here.

- **Test cases**: integration smoke test (no per-module SW-TC-* IDs
  are added in this sprint; the integration covers cross-module
  behavior and traces against `SW-REQ-SYS-*` integration-level
  requirements such as composition-root invariants per
  `system_design.md` §8.1, and against `SW-REQ-SIM-HARN-001`
  through `SW-REQ-SIM-HARN-010` exercised end-to-end).

- **Acceptance criteria**: per `methodology.md` §8 (criteria 1-9), plus:
  - **Composition root coverage**: `apps/main.cpp` instantiates every
    lib's `_IMPL_T::New()` factory, calls every per-app
    `<App>AppInit(tApp, /* DI args */, ...)`, and registers all app
    `APP_ROOT_T*` pointers into
    `juno::sch::SCH_ROOT_T<8, 200>::tArrSchTable[i][j]` per
    `system_design.md` §8.1. Vtable dispatch in the composition uses
    `tRoot.ptApi->Hook(...)` consistently — never `tApi->`.
  - **Time injection**: composition root calls
    `juno::time::TimeInit(tTime, tApi, pfcnFailureHandler, pvUserData)`
    exactly once with the Trick-backed `TIME_API_T` (when running
    inside `sim_harness`) or the POSIX `clock_gettime`-backed
    `TIME_API_T` (when running standalone). Time conversions in
    consuming code use `_ptTime->TimestampToMicros(...).tOk` per
    `libjuno/include/juno/time/time_api.hpp` (non-static member
    function).
  - **Scheduler driver**: a single `juno::sch::SCH_API_T<8,
    200>::Execute(tSch)` call drives every period (5 ms / 10 ms /
    50 ms / 100 ms / 200 ms / 500 ms) and dispatches every
    registered app exactly once per its scheduled period.
  - **Integration smoke test**: `full_fsw_test.cpp` runs the FT1
    baseline scenario through `sim_harness` and verifies:
    1. Every app's `OnStart` POST sequence runs (each app sets a
       POST-OK flag observable on the bus).
    2. The first `Execute()` cycle exercises every period band.
    3. Bus messages flow correctly between publishers and
       subscribers — e.g., `NAV_STATE` produced by `nav_app` reaches
       `afm_app`, `telem_app`, `mlog_app`, `sys_app` within one
       period of publication.
    4. `AFM` publishes a phase transition (PRE_LAUNCH → BOOST) within
       the SW-REQ-SYS-018 ±1 s tolerance vs. the
       `sim_scenario` true-phase transition record.
    5. `mlog_app` writes at least one record to the SD log artifact.
    6. `telem_app` emits at least one CCSDS packet to the telemetry
       capture artifact.
  - **No allocation during steady-state**: a `mallinfo` snapshot
    taken before `Execute()` and after 100 cycles shows zero net
    allocation, OR an `LD_PRELOAD` malloc-trap fixture aborts on any
    `malloc`/`calloc`/`new` after composition completes. (This
    enforces the SYS-level "no dynamic allocation post-init"
    contract.)
  - **Cross-cutting**: all sim module unit/integration tests still
    pass alongside the new integration test (no regressions).

- **Test gate**:
  - **G1**: full POSIX build + ctest including the integration suite:
    ```bash
    mkdir -p build_posix && cd build_posix && \
      cmake -DPLATFORM=POSIX -DJUNO_FSW_BUILD_SIM=ON \
            -DJUNO_FSW_BUILD_INTEGRATION=ON .. && \
      cmake --build . && ctest --output-on-failure
    ```
  - **G2**: `python3 tools/traceability.py` exits 0; final closure is
    expected to show **100% requirement closure** across all
    `SW-REQ-*` IDs in the repository.
  - **G3**: not applicable — composition root and integration test
    are POSIX-only.

- **Estimated agent count**: 3 workers + 3 reviewers + 1 CE = **7 agents**.

## 4. Wave Exit Gate (also serves as FSW SDP exit gate)

After SPRINT-IMPL-25 closes, the Lead spawns the **final FSW SDP exit
gate** `project-chief-engineer` invocation. The CE confirms:

1. All 26 sprints (SPRINT-IMPL-00..25) CLOSED with G1+G2 passing;
   sim sprint (24) and integration sprint (25) green.
2. **Composition root coverage**: every app registered into the
   scheduler table; every lib's `New()` called; every DI pointer
   wired; vtable dispatch convention `tRoot.ptApi->Hook(...)`
   honored throughout.
3. `python3 tools/burndown.py` shows **100% requirement closure** —
   no `SW-REQ-*` uncovered by code+tests.
4. Integration smoke test (`tests/integration/full_fsw_test.cpp`)
   passes on POSIX; sim modules still pass alongside.
5. All 5 carry-forward RFAs from `closure_memo.md` §5 closed (or
   formally accepted as out-of-scope / CDR-deferred with PM
   countersign):
   1. `juno::app::AppInit` FSW workaround consistency — verified
      across every `<App>AppInit` call site.
   2. `JUNO_MSG_BUS_VARIANT_T` definition — done in Wave 0.
   3. Capacity pins — done in Wave 0; per-lib reaffirmation
      collected during Waves 1-7.
   4. Option C → Option D `SIM_SENSORS_RAW_T` migration — Wave 8
      verifies Option D `static_assert` cross-checks succeed at
      integration time (sim_sensors AC item).
   5. NASA Trick `exec_get_sim_time()` symbol — verified at
      SPRINT-IMPL-24 entry gate.

CE issues PASS / PASS-WITH-ACTIONS / FAIL on the FSW design+
implementation. PASS unblocks HIL CDR phase (out of FSW SDP scope).

## 5. Cross-References

- [SDP Index](index.md), [Methodology](methodology.md)
- Wave files: [foundation_libs.md](foundation_libs.md),
  [sensor_libs.md](sensor_libs.md), [domain_libs.md](domain_libs.md),
  [sensor_apps.md](sensor_apps.md), [domain_apps.md](domain_apps.md)
- L2 designs:
  `docs/design/sim_dynamics/design.md`,
  `docs/design/sim_sensors/design.md`,
  `docs/design/sim_scenario/design.md`,
  `docs/design/sim_harness/design.md`,
  `docs/design/sim_harness/interfaces.md`,
  `docs/design/system/system_design.md`
- LibJuno upstream (authoritative for canonical names):
  - `libjuno/include/juno/module.h` (lines 97, 131, 161 —
    `JUNO_MODULE_ROOT` / `JUNO_MODULE_DERIVE`)
  - `libjuno/include/juno/time/time_api.hpp` (`TIME_ROOT_T`,
    `TIME_API_T`, `TimestampToMicros` member function)
  - `libjuno/include/juno/sch/juno_sch_api.hpp`
    (`SCH_API_T<8, 200>::Execute`)
  - `libjuno/include/juno/sb/broker_api.hpp`
    (`JUNO_MSG_BUS_VARIANT_T`)
  - `libjuno/include/juno/status.h` (19 canonical status codes)

<!-- @{"design": ["SW-REQ-SIM-DYN-001","SW-REQ-SIM-DYN-002","SW-REQ-SIM-DYN-003","SW-REQ-SIM-DYN-004","SW-REQ-SIM-DYN-005","SW-REQ-SIM-DYN-006","SW-REQ-SIM-DYN-007","SW-REQ-SIM-DYN-008","SW-REQ-SIM-DYN-009","SW-REQ-SIM-DYN-010","SW-REQ-SIM-DYN-011","SW-REQ-SIM-DYN-012","SW-REQ-SIM-DYN-013","SW-REQ-SIM-DYN-014","SW-REQ-SIM-SENS-001","SW-REQ-SIM-SENS-002","SW-REQ-SIM-SENS-003","SW-REQ-SIM-SENS-004","SW-REQ-SIM-SENS-005","SW-REQ-SIM-SENS-006","SW-REQ-SIM-SENS-007","SW-REQ-SIM-SENS-008","SW-REQ-SIM-SENS-009","SW-REQ-SIM-SENS-010","SW-REQ-SIM-SENS-011","SW-REQ-SIM-SENS-012","SW-REQ-SIM-SENS-013","SW-REQ-SIM-SENS-014","SW-REQ-SIM-SCEN-001","SW-REQ-SIM-SCEN-002","SW-REQ-SIM-SCEN-003","SW-REQ-SIM-SCEN-004","SW-REQ-SIM-SCEN-005","SW-REQ-SIM-SCEN-006","SW-REQ-SIM-SCEN-007","SW-REQ-SIM-SCEN-008","SW-REQ-SIM-SCEN-009","SW-REQ-SIM-SCEN-010","SW-REQ-SIM-SCEN-011","SW-REQ-SIM-SCEN-012","SW-REQ-SIM-HARN-001","SW-REQ-SIM-HARN-002","SW-REQ-SIM-HARN-003","SW-REQ-SIM-HARN-004","SW-REQ-SIM-HARN-005","SW-REQ-SIM-HARN-006","SW-REQ-SIM-HARN-007","SW-REQ-SIM-HARN-008","SW-REQ-SIM-HARN-009","SW-REQ-SIM-HARN-010"]} -->
