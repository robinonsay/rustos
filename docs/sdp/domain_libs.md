---
document_type: SDP — Domain Libraries (Wave 4)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: index.md
sprints_covered: SPRINT-IMPL-12 through SPRINT-IMPL-15
status: Draft (awaiting Chief Engineer gate)
---

# SDP — Wave 4: Domain Libraries

## 1. Purpose

Wave 4 covers the four domain-library sprints that sit between the foundation/sensor library waves and the application waves. `nav_lib` runs the navigation EKF that fuses IMU/baro/GPS into a 16-state estimate; `afm_lib` runs the autonomous flight phase state machine that classifies pre-launch/boost/apogee/descent/landing from sensor and nav inputs; `telem_lib` encodes the wire packets that `lora_lib` will transmit at 2 Hz; `mlog_lib` encodes the binary records that `sd_lib` will persist to the SD card. After Wave 4 closes, every domain encoder/computer the FSW needs is in place and Wave 5 (sensor apps) and Wave 6 (domain apps) can begin composing them onto the bus.

## 2. Wave Summary

| Wave | Sprints | Modules | Predecessor Wave | Successor Waves |
|------|---------|---------|------------------|-----------------|
| 4 | SPRINT-IMPL-12..15 | nav, afm, telem, mlog | Wave 1 (kmat_lib), Wave 3 (sensor message types from imu/baro/gps; lora_lib for telem; sd_lib for mlog) | Wave 5 (sensor apps), Wave 6 (domain apps) |

Wave 4 sprints have **inter-sprint dependencies** that pin a partial order:

- **SPRINT-IMPL-12 (nav)** must close first. It exports `juno::nav::NAV_STATE_T` and pins the authoritative `JUNO_MSG_NAV_STATE_T` field-shape table that all downstream Wave-4 modules consume.
- **SPRINT-IMPL-13 (afm)** depends on SPRINT-IMPL-12 (consumes `JUNO_MSG_NAV_STATE_T` in `Update()`) and exports `juno::afm::JUNO_PHASE_T` declared in `libs/afm_lib/include/afm_lib/afm_api.hpp`.
- **SPRINT-IMPL-14 (telem)** depends on SPRINT-IMPL-12 (nav state field shape, esp. row 56 narrowing) AND SPRINT-IMPL-13 (phase enum) AND SPRINT-IMPL-10 (lora_lib payload bound).
- **SPRINT-IMPL-15 (mlog)** depends on SPRINT-IMPL-12 + SPRINT-IMPL-13 (message types) AND SPRINT-IMPL-11 (sd_lib write API surface).

Recommended scheduling: **12 → 13 → (14 ∥ 15)**. SPRINT-IMPL-14 and SPRINT-IMPL-15 can run in parallel once 12 and 13 are CLOSED because they share no source files and only consume already-frozen header types.

## 3. Per-Sprint Plans

### SPRINT-IMPL-12 — nav_lib

- **Module**: `nav_lib` (algorithm-agnostic 16-state navigation library; consumes IMU/baro/GPS samples; produces `NAV_STATE_T`).
- **Predecessors**: SPRINT-IMPL-01 (kmat_lib for matrix math), SPRINT-IMPL-07 (imu_lib message types), SPRINT-IMPL-08 (baro_lib message types), SPRINT-IMPL-09 (gps_lib message types).
- **L2 design**: `docs/design/nav/design.md` (single shared impl per §3.3; pure-compute — no posix/pico2 split).
- **Requirements**: 17 SW-REQ-NAV-* (`SW-REQ-NAV-001` through `SW-REQ-NAV-017`).
- **Files to produce** (5 files; pure-compute single-impl per L2 §3.3 — no posix/pico2 split):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/nav_lib/include/nav_lib/nav_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/nav_lib/include/nav_lib/nav_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/nav_lib/src/nav_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/nav_lib/tests/nav_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/nav_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 18 Unit-type SW-TC-NAV-* entries (`SW-TC-NAV-001..013, 015, 017, 018, 019, 020`) plus 2 Demonstration entries (`SW-TC-NAV-014` GPS-bound, `SW-TC-NAV-016` POSIX/Pico2 equivalence). The 18 Unit cases drive `libs/nav_lib/tests/nav_lib_test.cpp`.
- **Acceptance criteria**: per `methodology.md` §8 (1–9), plus:
  - `NAV_STATE_T` field shape matches the **authoritative table at `nav/design.md` §4.1** (use `static_assert(sizeof(juno::nav::NAV_STATE_T) == sizeof(JUNO_MSG_NAV_STATE_T))` and `static_assert(offsetof(...) == offsetof(...))` for every member to enforce byte equivalence with `JUNO_MSG_NAV_STATE_T`). All floating-point fields are `double` (no `float` substitution permitted on the bus message).
  - FSW status code extensions defined in `nav_api.hpp` inside `namespace juno::nav`: `JUNO_FSW_STATUS_DIVERGED_ERROR = JUNO_STATUS_CUSTOM_ERROR + 3` and `JUNO_FSW_STATUS_OUT_OF_ORDER_ERROR = JUNO_STATUS_CUSTOM_ERROR + 4` per `nav/design.md` §4.5. Both must be `static constexpr JUNO_STATUS_T` with internal linkage.
  - `static constexpr double kNavGpsBoundM_default = 200.0;` declared in `nav_api.hpp` in `namespace juno::nav` (closes S1-AI-023; per `nav/design.md` §9). The composition root must source `NAV_INIT_T.fGpsBoundM` from this constant.
  - **Determinism (SW-REQ-NAV-015)**: identical input sequences produce **bit-identical** `NAV_STATE_T` outputs across two independent process invocations (Run A vs Run B byte-equality across all 16 fields plus `bValid` and timestamp).
  - Tests cover `Aligned → Diverged` transition path on numerical instability (covariance unbounded / non-finite component) AND on GPS bound violation; `Diverged` continues propagating per `SW-REQ-NAV-013` (no halt, no auto-recovery).
  - LibJuno conventions: `JUNO_MODULE_ROOT(NAV_LIB_API_T, ...)` and `JUNO_MODULE_DERIVE(NAV_LIB_ROOT_T, ...)`; `tRoot.ptApi->Hook(...)` dispatch (never `tApi->`); all five API entry points are `noexcept`.
- **Test gate**: G1 (POSIX build + ctest) and G2 (`tools/traceability.py` exit 0); **G3 (Pico 2 cross-compile)** required because `SW-REQ-NAV-016` claims POSIX/Pico2 equivalence even with single-shared impl.
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

### SPRINT-IMPL-13 — afm_lib

- **Module**: `afm_lib` (autonomous flight-manager phase state machine; consumes IMU/baro/GPS/NAV; produces `JUNO_PHASE_T` plus per-transition timestamp).
- **Predecessors**: SPRINT-IMPL-12 (`juno::nav::NAV_STATE_T` and `JUNO_MSG_NAV_STATE_T` field shape), SPRINT-IMPL-07/08/09 (sensor message types). No kmat dependency — afm is pure logic, no matrix math.
- **L2 design**: `docs/design/afm/design.md` (single-file impl per §3.3; pure state machine — no posix/pico2 split).
- **Requirements**: 11 SW-REQ-AFM-* (`SW-REQ-AFM-001` through `SW-REQ-AFM-011`).
- **Files to produce** (5 files; pure-compute single-impl per L2 §3.3):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/afm_lib/include/afm_lib/afm_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/afm_lib/include/afm_lib/afm_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/afm_lib/src/afm_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/afm_lib/tests/afm_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/afm_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 14 Unit-type + 1 Integration-type SW-TC-AFM-* entries (`SW-TC-AFM-001..009, 011, 012, 013, 015, 016` Unit; `SW-TC-AFM-014` Integration POSIX-vs-Pico2 cross-build) plus 1 Demonstration (`SW-TC-AFM-010` HIL replay). The 14 Unit + 1 Integration cases drive `libs/afm_lib/tests/afm_lib_test.cpp`.
- **Acceptance criteria**: per `methodology.md` §8 (1–9), plus:
  - **`JUNO_PHASE_T` declaration home**: per `afm/design.md` §4 (delta-PDR Δ-MINOR-3 fix), the `JUNO_PHASE_T` enum is declared in `libs/afm_lib/include/afm_lib/afm_api.hpp` inside `namespace juno::afm` — **not** in `system/conventions` headers and **not** duplicated elsewhere. `telem_lib` and `mlog_lib` consume it via `juno::afm::JUNO_PHASE_T`.
  - Phase enum value set must be exactly `{PRE_LAUNCH, BOOST, APOGEE, DESCENT, LANDING}` per `SW-REQ-AFM-002` and `SW-REQ-SYS-016` (cross-module name-locked per the 2026-05-02 lessons-learned entry on cross-module enum spelling).
  - **Monotonic-forward transitions (SW-REQ-AFM-004, AFM-005)**: state machine refuses regressive or skip-ahead transitions; verified by `SW-TC-AFM-005`, `-006`, `-007`.
  - **Bounded latency (SW-REQ-AFM-007)**: per-transition detection within 1.0 s of ground-truth on synthetic profile (`SW-TC-AFM-009` Unit); HIL latency (`SW-TC-AFM-010`) is a Demonstration deferred to flight ops.
  - **AFM-loss tolerance (SW-REQ-AFM-009)**: failure paths return `JUNO_STATUS_T` error codes; never `abort()`, never throw, never call into the bus (afm_lib has no broker handle).
  - **Determinism + POSIX/Pico2 equivalence (SW-REQ-AFM-010, AFM-011)**: identical inputs yield bit-identical phase trace AND identical four transition timestamps across (a) repeated POSIX runs and (b) POSIX vs Pico2 cross-compiled runs.
  - LibJuno conventions: same as SPRINT-IMPL-12 (root/api/impl triple, `noexcept`, `JUNO_MODULE_ROOT`/`DERIVE` macros).
- **Test gate**: G1 + G2; **G3 (Pico 2 cross-compile)** required for `SW-TC-AFM-014` Integration test.
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

### SPRINT-IMPL-14 — telem_lib

- **Module**: `telem_lib` (pure-compute telemetry packet encoder; CRC-16-CCITT integrity; consumed by `telem_app` at 2 Hz; bytes handed to `lora_lib` for transmission).
- **Predecessors**: SPRINT-IMPL-10 (lora_lib transport — establishes RYLR896 MTU bound `kMaxPacketBytes = 240`), SPRINT-IMPL-12 (`JUNO_MSG_NAV_STATE_T` field shape, especially row 56 — `tNav.tPosLla[2]` altitude precision), SPRINT-IMPL-13 (`juno::afm::JUNO_PHASE_T` enum), SPRINT-IMPL-08/09 (baro/gps message types).
- **L2 design**: `docs/design/telem/design.md` (single shared impl per §3.3; pure-compute — no posix/pico2 split).
- **Requirements**: 12 SW-REQ-TELEM-* (`SW-REQ-TELEM-001` through `SW-REQ-TELEM-012`).
- **Files to produce** (5 files; pure-compute single-impl per L2 §3.3):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/telem_lib/include/telem_lib/telem_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/telem_lib/include/telem_lib/telem_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/telem_lib/src/telem_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/telem_lib/tests/telem_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/telem_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 16 Unit-type SW-TC-TELEM-* entries (`SW-TC-TELEM-001` through `SW-TC-TELEM-016`); zero Demonstration. All drive `libs/telem_lib/tests/telem_lib_test.cpp`.
- **Acceptance criteria**: per `methodology.md` §8 (1–9), plus:
  - **Row 56 narrowing (Δ-MAJOR-3 fix per `telem/design.md` §6)**: when packing the nav-altitude payload field, the implementation must use `static_cast<float>(tNav.tPosLla[2])` for the explicit double→float narrowing. The reviewer must grep for `tNav.tPosLla[2]` in `telem_impl.cpp` and confirm the cast is present at row 56 of the packing table.
  - **CRC-16-CCITT golden-vector test (SW-REQ-TELEM-009)**: `SW-TC-TELEM-013` embeds a fixed `TELEM_INPUTS_T` struct and a fixed reference golden byte vector (including the trailing 2-byte CRC) in the test fixture; the produced bytes must match exactly on **both POSIX and Pico2 builds**. CRC variant locked at polynomial `0x1021`, init `0xFFFF`, no reflection, no final XOR (per `telem/design.md` §2).
  - **Bounded packet size (SW-REQ-TELEM-004)**: `static_assert(kPacketBytes <= kMaxPacketBytes, ...)` in `telem_api.hpp` enforces compile-time the 240-byte RYLR896 MTU bound (the `kMaxPacketBytes` value sourced from `lora_lib` headers).
  - **No I/O (SW-REQ-TELEM-011)**: `SW-TC-TELEM-014` Inspection must grep `libs/telem_lib/src/` for `uart`, `lora`, `radio`, `printf` and find zero matches; the test target must link with zero unresolved I/O symbols.
  - **Big-endian wire format**: all multi-byte fields packed big-endian per `telem/design.md` §2; verified by `SW-TC-TELEM-013` golden vector.
  - **Phase enum source**: `telem_inputs_t.ePhase` is `juno::afm::JUNO_PHASE_T` (the SPRINT-IMPL-13 declaration home); `telem_lib` must `#include "afm_lib/afm_api.hpp"` rather than re-declaring.
  - LibJuno conventions: same as SPRINT-IMPL-12.
- **Test gate**: G1 + G2; **G3 (Pico 2 cross-compile)** required to satisfy `SW-REQ-TELEM-010` POSIX/Pico2 byte-identical output (golden vector must match on both targets).
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

### SPRINT-IMPL-15 — mlog_lib

- **Module**: `mlog_lib` (pure-compute mission-log binary record encoder; serializes 8 record kinds — IMU, baro, GPS NMEA raw, GPS UTC, NAV, AFM phase, sys health, sys POST — into caller-supplied byte buffers).
- **Predecessors**: SPRINT-IMPL-11 (sd_lib write API — establishes the buffer-write contract `mlog_app` will use to persist the bytes), SPRINT-IMPL-12 (`JUNO_MSG_NAV_STATE_T`), SPRINT-IMPL-13 (`JUNO_MSG_AFM_PHASE_T`), SPRINT-IMPL-07/08/09 (sensor message types). Also depends on SYS message types (`JUNO_MSG_SYS_HEALTH_T`, `JUNO_MSG_SYS_POST_T`) which are pinned by the system_design.md catalog (frozen pre-Wave-4).
- **L2 design**: `docs/design/mlog/design.md` (single shared impl per §3.1/§3.3; pure-compute — no posix/pico2 split).
- **Requirements**: 14 SW-REQ-MLOG-* (`SW-REQ-MLOG-001` through `SW-REQ-MLOG-014`).
- **Files to produce** (5 files; pure-compute single-impl per L2 §3.3):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/mlog_lib/include/mlog_lib/mlog_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/mlog_lib/include/mlog_lib/mlog_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/mlog_lib/src/mlog_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/mlog_lib/tests/mlog_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/mlog_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 16 Unit-type + 1 Integration-type SW-TC-MLOG-* entries (`SW-TC-MLOG-001..012, 014..017` Unit; `SW-TC-MLOG-013` Integration POSIX-vs-Pico2 cross-build). All drive `libs/mlog_lib/tests/mlog_lib_test.cpp`.
- **Acceptance criteria**: per `methodology.md` §8 (1–9), plus:
  - **NAV record `fAltMHae` narrowing (Δ-MINOR-7 fix per `mlog/design.md` §6.6)**: in the NAV-record encoder, `fAltMHae` must be produced via an explicit `static_cast<float>(tNav.tPosLla[2])` from the double on the bus message. The reviewer must locate the cast in `mlog_impl.cpp` and document it. Comment must reference Δ-MINOR-7.
  - **Little-endian byte order on both POSIX and Pico2 (SW-REQ-MLOG-013, MLOG-014)**: encoder writes all multi-byte scalars little-endian regardless of host endianness; `SW-TC-MLOG-013` Integration verifies byte-level diff between POSIX and Pico2 outputs is zero across a fixed scripted record sequence.
  - **Self-describing record format (SW-REQ-MLOG-010)**: every record begins with `[kind:1 byte][tTimestampUs:8 bytes][payload:variable]` per `mlog/design.md` §3.2. Round-trip parser test (`SW-TC-MLOG-010`, `SW-TC-MLOG-015`) must recover record kind tag and dispatch to matching decoder for all 8 kinds.
  - **Per-record monotonic timestamp (SW-REQ-MLOG-007)**: caller passes `tTimestampUs` (encoder does not call any clock); `SW-TC-MLOG-007` injects a monotonic time source and verifies every record's embedded timestamp matches the source value at the moment of the write call.
  - **Schema version record-0 (`MLOG_KIND_HEADER`)**: per `mlog/design.md` §2 / §3.2, each new run begins with a header record carrying `kMlogSchemaVersion`. `mlog_app` (Wave 6) will issue this; `mlog_lib` must export `EncodeHeader()`.
  - **NAV state field shape consumption**: `mlog_lib` consumes `JUNO_MSG_NAV_STATE_T` whose field shape is locked by SPRINT-IMPL-12 §4.1 — `mlog_lib` must `#include` the bus message header rather than redeclare.
  - **AFM phase enum consumption**: `mlog_lib` consumes `juno::afm::JUNO_PHASE_T` from `afm_lib/afm_api.hpp` (SPRINT-IMPL-13 declaration home).
  - **Determinism (SW-REQ-MLOG-014)**: `SW-TC-MLOG-014` runs the same scripted sequence twice and SHA-256-hashes both byte streams; hashes must match exactly.
  - LibJuno conventions: same as SPRINT-IMPL-12. All 9 `Encode*` entry points are `noexcept`; encoder is purely functional with **no shared mutable state** in `MLOG_LIB_ROOT_T`.
- **Test gate**: G1 + G2; **G3 (Pico 2 cross-compile)** required for `SW-TC-MLOG-013` Integration test.
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

## 4. Wave Exit Gate

After SPRINT-IMPL-15 closes, the **"Wave 4 Exit Gate"** (Chief Engineer review) confirms:

1. All four sprints (SPRINT-IMPL-12, -13, -14, -15) are **CLOSED** with G1+G2+G3 passing on every sprint, and `tools/traceability.py` exits 0 across the four module trees.
2. **Cross-lib message-shape consistency**: byte-equivalent layouts asserted at compile time via `static_assert(sizeof(juno::nav::NAV_STATE_T) == sizeof(JUNO_MSG_NAV_STATE_T))` (and per-field `offsetof` checks) in `nav_api.hpp`. Equivalent assertions where `telem_lib` / `mlog_lib` cast bus messages into encoder inputs.
3. **Cross-lib enum consistency**: `juno::afm::JUNO_PHASE_T` is referenced (not redeclared) by `telem_lib` and `mlog_lib`. Reviewer greps both modules for any local re-declaration of phase tokens — must find none.
4. **Carry-forward RFA closure for S1-AI-023**: `kNavGpsBoundM_default = 200.0` is present in `libs/nav_lib/include/nav_lib/nav_api.hpp` and the comment cites the FT1 ground-track rationale documented in `nav/design.md` §9.
5. **Burndown delta**: `tools/burndown.py` shows requirement closure delta matching the expected count (17 NAV + 11 AFM + 12 TELEM + 14 MLOG = 54 SW-REQ-* IDs newly Verified).
6. **No regressions** in Wave 0/1/2/3 module test suites (full ctest still green).

Only after the Wave 4 exit gate **PASS** can Wave 5 begin (`sensor_apps.md`).

## 5. Cross-References

- [SDP Index](index.md)
- [Methodology](methodology.md)
- [Foundation Libraries (Wave 0–1)](foundation_libs.md)
- [Sensor Libraries (Wave 2–3)](sensor_libs.md)
- [Sensor Apps (Wave 5)](sensor_apps.md)
- [Domain Apps (Wave 6)](domain_apps.md)
- [Sim and Integration (Waves 7–8)](sim_and_integration.md)
- L2 designs: `docs/design/nav/design.md`, `docs/design/afm/design.md`, `docs/design/telem/design.md`, `docs/design/mlog/design.md`
- Requirements: `docs/requirements/{nav,afm,telem,mlog}/requirements.json`
- Test cases: `docs/test_cases/{nav,afm,telem,mlog}/test_cases.json`
