---
document_type: PDR Corrective Action Sprint Plan
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
status: Draft (awaiting Chair approval)
predecessor_review: PDR Sections S1-S9 review (2026-05-02 → 2026-05-03)
successor_milestone: PDR Closure (S10)
---

# PDR Corrective Action Sprint Plan

## 1. Background

The Preliminary Design Review (PDR) review board reviewed all 10 sections (S1 architecture; S2 foundation libraries; S3 sensor drivers; S4 comm/storage; S5 domain libraries; S6 sensor apps; S7 domain apps; S8 system app; S9 simulation modules; S10 closure). Sections S1 and S2 closed CHAIR PROCEED with action items. Sections S3-S9 were reviewed in parallel by 21 reviewer agents (3 per section: Mission Assurance, Senior Engineer, Chief Engineer) and produced approximately **237 RIDs and 140 RFAs**.

The findings cluster heavily into **four root causes**, each traceable to upstream decisions made during S1/S2 that did not propagate to L2 designs authored before those decisions landed:

| Root Cause | Origin | Manifestation in S3-S9 |
|------------|--------|------------------------|
| **C1. Option A app-lifecycle pivot** (apps must embed `juno::app::APP_ROOT_T` and implement `OnStart/OnProcess/OnExit`) | S2-RID-S2-021, accepted 2026-05-03 | All 8 apps (S6: gps_app, imu_app, baro_app; S7: nav_app, afm_app, telem_app, mlog_app; S8: sys_app) still expose `Init/Execute` instead of canonical hooks |
| **C2. Status-code catalog sweep** (`conventions.md` §4.7 added 19-code canonical catalog + fabricated→canonical mapping) | S2-RID-S2-009 / S2-AI-005, target rolling | ~21 of 27 L2 designs use fabricated codes (`JUNO_STATUS_IO_ERROR`, `_INVALID_ARG_ERROR`, `_OVERFLOW_ERROR`, `_NUMERIC_ERROR`, `_FORMAT_ERROR`, `_BOUNDS_ERROR`, `_INVALID_INPUT_ERROR`, `_INVALID_ARGUMENT_ERROR`, `_MEMORY_SIZE_ERROR`) |
| **C3. mlog @ 5 ms cascade** (`kMlogAppPeriodMs = 5` for SW-REQ-SYS-011 no-downsampling) | S1-AI-005, executed in `system_design.md` only | mlog/design.md still 10 ms; nav §8, afm §8, telem §8, sd §8, sys_app §8.2 all cite 10 ms |
| **C4. LibJuno type-name canonical pivot** (use `juno::time::TIME_ROOT_T`, `juno::sb::BROKER_ROOT_T<MsgT,…>`, etc., per Option A) | S2-RID-S2-021, accepted 2026-05-03 | Most apps + sim_harness reference fabricated `TIME_LIB_ROOT_T` / non-templated `BROKER_ROOT` / `juno::time::GetUs()` |

The corrective work is dominantly **mechanical** (sweeps + structural transforms with known patterns), not architectural rework. This sprint plan organizes the corrective work into 6 batched workstreams (B1–B6) that, executed in two phases, close the bulk of S3-S9 RIDs and the carry-forward action items from S1/S2.

## 2. Sprint Goal

Drive every Major and Minor RID from S3-S9 to CLOSED status (or DEFERRED-with-Chair-approval) so that the Project Chief Engineer can issue a PDR closure verdict (PASS / PASS-WITH-ACTIONS / FAIL) per Charter §7. Specifically:

- **Close every S1/S2 carry-forward action** that has a pre-S10 target.
- **Resolve C1-C4 root causes** across all 27 L2 designs.
- **Update conventions.md, system_design.md, and `requirements/sys/requirements.json`** to reflect Option A and the SYS-016 amendment.
- **Produce a clean traceability gate** (`tools/traceability.py` exit 0; 371 reqs valid; coverage preserved).
- **Re-spawn reviewers** on the corrected designs for verification before S10.

## 3. Sprint Acceptance Criteria

| # | Criterion | Verifiable Artifact |
|---|-----------|---------------------|
| AC-1 | All 8 apps embed `juno::app::APP_ROOT_T` and expose `OnStart/OnProcess/OnExit` per `conventions.md` §1.4 | grep across `docs/design/{*_app}/design.md` shows zero `(Init|Execute)\b` as public lifecycle hooks; aggregate-init pattern present |
| AC-2 | All 27 L2 designs use only the 19 canonical status codes from `juno/status.h` (or FSW-extension `JUNO_STATUS_CUSTOM_ERROR + N` with documented offsets) | grep for fabricated names (`JUNO_STATUS_IO_ERROR`, `_INVALID_ARG_ERROR`, `_OVERFLOW_ERROR`, `_NUMERIC_ERROR`, `_FORMAT_ERROR`, `_BOUNDS_ERROR`, `_INVALID_INPUT_ERROR`, `_INVALID_ARGUMENT_ERROR`, `_MEMORY_SIZE_ERROR`, `_NULL_POINTER`, `_OVERFLOW`) returns zero hits |
| AC-3 | `kMlogAppPeriodMs = 5` reflected in every L2 design that references the mlog period | grep for `kMlogAppPeriodMs = 10` returns zero hits across `docs/design/` |
| AC-4 | LibJuno canonical types used: `juno::time::TIME_ROOT_T` (not `TIME_LIB_ROOT_T`); `juno::sb::BROKER_ROOT_T<…>` (not unqualified `BROKER_ROOT`); `juno::time::TIME_API_T::Now` + `TimestampToMicros` (not `GetUs`/`Now()` free functions) | grep for `TIME_LIB_ROOT_T`, `juno::time::GetUs`, `juno_time::GetUs` returns zero hits |
| AC-5 | sim_harness implements `juno::time::TIME_API_T { Now, SleepTo, Sleep }` and aggregate-initializes via `juno::time::TimeInit(...)` per Option A; no `JUNO_TIME_PROVIDER_T` callback; no `TIME_LIB_IMPL_T::New(...)` factory | sim_harness §4.4 shows the canonical pattern; `interfaces.md` updated |
| AC-6 | sim_dynamics `SIM_DYN_TRUTH_T` is C++-only (drops `extern "C"`) OR uses C-compatible types throughout (no namespaced enum inside `extern "C"` block) | Header `.hpp` rename or `uint8_t u8Phase` field; static_assert holds |
| AC-7 | sim_sensors GPS injection seam matches device_lib's actual POSIX impl (pty write-fd, not fictional `device_lib::posix::Inject`) | Cross-reference resolves to a real symbol or device_lib design extended with `Inject` |
| AC-8 | `conventions.md` §4.x adds canonical `JUNO_FSW_STATE_T` enum (5-value lifecycle) | enum present, sys_app references it |
| AC-9 | sys_app §4 includes authoritative health-bitmap bit-assignment table | table present with bit positions, owner, set/clear semantics |
| AC-10 | SYS-016 amended to include `pre-launch` as initial phase (Chair action) | requirements/sys/requirements.json shows pre-launch in SYS-016 description or rationale |
| AC-11 | nav L2 §4 or §9 pins a numeric default for `kNavGpsBoundM` (SW-REQ-SYS-014) | nav/design.md contains the value with rationale |
| AC-12 | telem ↔ nav field-precision reconciled (`fAltMHae`/`tVelNed`); bus-message field shape pinned | system_design.md §4 or nav §4.1 contains explicit field list with types |
| AC-13 | `tools/traceability.py` exits 0 with 371 reqs and ≥370 with test specs | gate output captured |
| AC-14 | Re-spawned reviewer agents (3 per section, S3-S9) issue PROCEED on corrected designs | 21 reviewer reports with PROCEED recommendations |
| AC-15 | Master log (`rid_rfa_log.md`) shows zero OPEN RIDs/RFAs (all DISPOSED or CLOSED) | log statistics counters |

## 4. Workstreams

### B1 — Option A App Lifecycle Migration (8 apps)

**Scope:** Rewrite `§3.3`, `§4.1`, `§4.2`, `§7` (sequence diagrams), `§10` (memory ownership), and `§11` (traceability) of all 8 app L2 designs to:
- Embed `juno::app::APP_ROOT_T tRoot;` as the first member of the concrete app struct.
- Replace bespoke `<APP>_API_T` (where present, e.g., `IMU_APP_API_T`, `TELEM_APP_API_T`, `SYS_APP_API_T`) with the canonical `juno::app::APP_API_T { OnStart, OnProcess, OnExit }`.
- Move what was `Init(...)` into a free `<App>AppInit(...)` function called by the composition root before the scheduler starts; rename the lifecycle entry that the scheduler dispatches to `OnProcess`; rename the one-shot init hook to `OnStart`; add `OnExit` (POSIX-only, no-op on Pico2 per SW-REQ-SYS-047).
- Replace any reference to `sch_lib::Run()` with the canonical `juno::sch::SCH_API_T<8, 200>::Execute()`.
- Show composition-root aggregate-initialization of the static `juno::app::APP_API_T tApi{...}` and how `&tApp.tRoot` is placed into `SCH_ROOT_T<8, 200>::tArrSchTable[i][j]`.

**Apps in scope:**
| Section | App | Worker |
|---------|-----|--------|
| S6 | `gps_app` | software-systems-engineer |
| S6 | `imu_app` | software-systems-engineer |
| S6 | `baro_app` | software-systems-engineer |
| S7 | `nav_app` | software-systems-engineer |
| S7 | `afm_app` | software-systems-engineer |
| S7 | `telem_app` | software-systems-engineer |
| S7 | `mlog_app` | software-systems-engineer |
| S8 | `sys_app` | software-systems-engineer |

**Workers:** 8 in parallel.

**Acceptance:** AC-1 (Option A lifecycle) plus per-app traceability tag preservation (every SW-REQ-*-APP-NNN still tagged in at least one §, §11 traceability table preserved).

**Dependencies:** B6 must complete first if `JUNO_FSW_STATE_T` enum is consumed by `sys_app`. Otherwise B1 is independent and parallelizable.

### B2 — Status-Code Catalog Sweep (≈21 L2 designs)

**Scope:** Replace every fabricated status-code name with its canonical replacement per `conventions.md` §4.7 mapping table:

| Fabricated (do not use) | Canonical replacement | Notes |
|-------------------------|----------------------|-------|
| `JUNO_STATUS_NULL_POINTER` | `JUNO_STATUS_NULLPTR_ERROR` | already in catalog (value 2) |
| `JUNO_STATUS_OVERFLOW` | `JUNO_STATUS_TABLE_FULL_ERROR` | capacity exceeded |
| `JUNO_STATUS_OVERFLOW_ERROR` | `JUNO_STATUS_TABLE_FULL_ERROR` or `_OOB_ERROR` | choose by semantic |
| `JUNO_STATUS_IO_ERROR` | `JUNO_STATUS_READ_ERROR` or `_WRITE_ERROR` | pick by direction |
| `JUNO_STATUS_INVALID_STATE_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` | bad-state precondition |
| `JUNO_STATUS_INVALID_ARG_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` | |
| `JUNO_STATUS_INVALID_ARGUMENT_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` | |
| `JUNO_STATUS_INVALID_INPUT_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` | |
| `JUNO_STATUS_NUMERIC_ERROR` | FSW extension: `JUNO_FSW_STATUS_NUMERIC_ERROR = JUNO_STATUS_CUSTOM_ERROR + 1` in `juno::kmat` | document offset |
| `JUNO_STATUS_FORMAT_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` or FSW extension | sim_scenario context |
| `JUNO_STATUS_BOUNDS_ERROR` | `JUNO_STATUS_OOB_ERROR` (canonical) | already in catalog (value 17) |
| `JUNO_STATUS_MEMORY_SIZE_ERROR` | `JUNO_STATUS_INVALID_SIZE_ERROR` | already in catalog (value 6) |
| `JUNO_STATUS_BUSY_ERROR` | FSW extension: `JUNO_FSW_STATUS_BUSY_ERROR = JUNO_STATUS_CUSTOM_ERROR + N` in consuming namespace | lora_lib context |
| `JUNO_STATUS_DIVERGED_ERROR`, `_OUT_OF_ORDER_ERROR`, `_NOT_INITIALIZED_ERROR` | FSW extensions in `juno::nav` namespace | document offsets |

**Designs in scope:** All 27 L2 designs under `docs/design/` (excluding kmat 04/05 split files which are sub-files of one design). The sweep also touches `requirements/log/requirements.json` (LOG-007 rationale `stdout` → `stderr` per S2-RID-S2-004).

**Workers:** 1 sweep worker (script-assisted via grep + targeted Edit) OR 7 per-section workers (parallel). Recommend single sweep worker for atomicity.

**Acceptance:** AC-2 — grep returns zero fabricated hits.

**Dependencies:** None. Can run concurrent with all other Bs.

### B3 — Mlog @ 5 ms Cascade (S1-AI-005 re-execution)

**Scope:** Update every L2 design that references the mlog period to use `kMlogAppPeriodMs = 5`:

| File | Locations to fix |
|------|------------------|
| `docs/design/mlog/design.md` | §8 timing table, §3.3, §11 |
| `docs/design/mlog_app/design.md` | §4.1 constants, §3.3, §3.4, §6.1, §7.1, §8, §11 |
| `docs/design/nav/design.md` | §8 downstream-consumer footer |
| `docs/design/afm/design.md` | §8 downstream-consumer footer |
| `docs/design/telem/design.md` | §8 downstream-consumer footer |
| `docs/design/sd/design.md` | §3.2, §7.1, §8, §8.1 |
| `docs/design/sys_app/design.md` | §8.2 |
| `docs/design/nav_app/design.md` | §8 downstream-consumer footer |

**Workers:** 1 worker, sweep across files.

**Acceptance:** AC-3.

**Dependencies:** None. Independent of other Bs.

### B4 — LibJuno Canonical Type-Name Cascade

**Scope:** Replace incorrect type names with canonical:

| Fabricated / Old | Canonical | Files affected |
|-------------------|-----------|----------------|
| `juno::time::TIME_LIB_ROOT_T` | `juno::time::TIME_ROOT_T` | telem_app, mlog_app, gps_app (path), and any other |
| `juno::time::GetUs()`, `juno_time::GetUs()` | `tTime.tApi->Now(tTime).tOk` + `tTime.TimestampToMicros(...).tOk` | baro_app, baro_lib, gps_lib (text refs) |
| `juno::sb::BROKER_ROOT` (untemplated) | `juno::sb::BROKER_ROOT_T<MsgT, PipeN, RegCapacity>` (templated) — define a project-wide alias in a shared header | gps_app, imu_app, baro_app, nav_app, afm_app, telem_app, mlog_app, sys_app |
| `#include "juno_time/time_api.hpp"` | `#include "juno/time/time_api.hpp"` | gps_lib §4.1 |
| `juno::sd::SD_LIB_ROOT_T` (untemplated) | `juno::sd::SD_LIB_ROOT_T<N>` or `SD_LIB_ROOT_DEFAULT_T = SD_LIB_ROOT_T<kDefaultWriteBufBlocks>` | sys_app (S2-RID-S2-017 cascade) |
| `juno::device::DEVICE_LIB_ROOT_T` (untemplated) | `juno::device::DEVICE_LIB_ROOT_T<N>` per use site | sys_app, lora_lib references where untemplated |

**Workers:** 1 worker, multi-file Edit pass.

**Acceptance:** AC-4.

**Dependencies:** None.

### B5 — sim_harness Option A Rewrite + Fictional Symbols Cleanup (S9)

**Scope:** Substantial rewrite of sim_harness §4.4 / §10.2 / §10.3 / §11 + interfaces.md §4.4 / §4.5 to:
- Remove every `JUNO_TIME_PROVIDER_T` reference and the entire `TIME_LIB_IMPL_T::New(pfcnTimeProvider, …)` factory pattern.
- Add a `static const juno::time::TIME_API_T tTrickTimeApi { TrickNow, TrickSleepTo, TrickSleep };` aggregate-initialization in `sim/sim_harness/src/time_trick_source.cpp`.
- Show the composition-root call `juno::time::TimeInit(tTime, tTrickTimeApi, &TrickFailureHandler, this)`.
- `TrickNow` returns `RESULT_T<JUNO_TIMESTAMP_T>` derived from `exec_get_sim_time()` via `TIME_ROOT_T::DoubleToTimestamp`.
- Replace `device_lib::posix::Inject` references with the canonical `openpty()`-master-fd `write()` pattern (per device_lib §11.1) OR file an S3 cross-section re-open requesting `device_lib::posix::Inject` as an explicit API addition (Lead recommendation: pty write-fd pattern).
- Fix sim_harness `Init` ordering deadlock (sim_harness §4.3 calls `Init(tArgs, scen.tScenario, …)` before `scen.LoadScenario` runs) by either splitting `harness.Init` into two Trick jobs (arg parsing + composition) or by relocating `LoadScenario` into `harness.Init`.

**Companion fixes (sim_dynamics, sim_scenario):**
- sim_dynamics §6.1: drop `extern "C"` on `SIM_DYN_TRUTH_T` (or replace `juno::afm::JUNO_PHASE_T ePhase` with `uint8_t u8Phase`); remove `#include "juno/afm/juno_phase.h"` (file does not exist; AFM design doesn't author it).
- sim_scenario §4.2 / §9.2: status-code sweep (covered by B2).
- sim_sensors `SIM_SENSORS_RAW_T` cross-module ownership: state explicitly that it is defined in `imu_lib`'s public header (or add `static_assert` layout equivalence).

**Workers:** 2 workers (1 for sim_harness, 1 for sim_dynamics + sim_scenario).

**Acceptance:** AC-5, AC-6, AC-7.

**Dependencies:** None.

### B6 — Conventions.md and System Baseline Closure (S1/S2 carry-forwards)

**Scope:**
- Add `JUNO_FSW_STATE_T` enum to `conventions.md` §4 (closes S1-AI-018):
  ```cpp
  namespace juno
  {
  enum class JUNO_FSW_STATE_T : uint8_t {
      JUNO_FSW_STATE_POST     = 0,
      JUNO_FSW_STATE_INIT     = 1,
      JUNO_FSW_STATE_RUN      = 2,
      JUNO_FSW_STATE_SAFE     = 3,
      JUNO_FSW_STATE_RECOVERY = 4,
  };
  }
  ```
- Update `sys_app/design.md` §3.3 to reference the canonical enum (drop local `LIFECYCLE_T`).
- Add authoritative health-bitmap bit-assignment table to sys_app §4 (closes S1-AI-019). Table format:
  | Bit | Sensor | Set by | Cleared by | Mask constant |
  |-----|--------|--------|------------|---------------|
  | 0 | IMU | imu_app on `bValid==false` | imu_app on `bValid==true` | `kHealthBitImu = 1u<<0` |
  | … | … | … | … | … |
- Add `nmea_lib` and `sim_harness` documented exceptions to `conventions.md` §6.1 (alongside `kmat_lib`).
- Pin `kNavGpsBoundM_default` numeric value in `nav/design.md` §4 or §9 (closes S1-AI-023; SW-REQ-SYS-014).
- Telem ↔ nav field-precision reconciliation: pin `JUNO_MSG_NAV_STATE_T` field shape in either `system_design.md` §4 or `nav/design.md` §4.1 (closes telem field-precision RID and `fAltMHae` ambiguity).
- **Chair actions (cannot be Lead-executed):**
  - **SYS-016 amendment**: amend `requirements/sys/requirements.json` SW-REQ-SYS-016 to include `pre-launch` as the at-power-on initial phase (closes S1-AI-011).
  - **LOG-007 rationale**: update LOG-007 rationale prose from "stdout" to "stderr" (closes S2-RID-S2-004).
  - **IMU model selection**: lock the IMU part choice (conventions FLAG-4 / S1-AI-022) — recommend MPU-6050 already proposed.

**Workers:** 1 worker for conventions/sys_app/nav L2 edits. Chair handles requirement-text amendments directly.

**Acceptance:** AC-8, AC-9, AC-10, AC-11, AC-12.

**Dependencies:** Must complete before B1 worker for `sys_app` (which consumes `JUNO_FSW_STATE_T`).

## 5. Execution Phases

### Phase 0 — Carry-forwards and conventions (Chair + Lead)

| Item | Owner | Output |
|------|-------|--------|
| SYS-016 amendment | Chair | requirements/sys/requirements.json |
| LOG-007 rationale | Chair | requirements/log/requirements.json |
| IMU model lock | Chair | conventions FLAG-4 → resolved |
| B6 conventions edit (`JUNO_FSW_STATE_T`, §6.1 exceptions) | Lead | conventions.md |
| B6 sys_app §4 health-bitmap table | Lead | sys_app/design.md (partial; full B1 rewrite later) |
| B6 nav numeric `kNavGpsBoundM_default` | Lead | nav/design.md |
| B6 `JUNO_MSG_NAV_STATE_T` field-shape pin | Lead | system_design.md or nav/design.md |

### Phase 1 — Mechanical sweeps (parallel)

| Workstream | Workers | Files |
|-----------|---------|-------|
| B2 status-code sweep | 1 (script-assisted) | ~21 L2 designs |
| B3 mlog 5 ms cascade | 1 | 8 L2 designs |
| B4 LibJuno type-name cascade | 1 | 8+ L2 designs |

3 workers in parallel. Estimated wall time: one parallel agent batch.

### Phase 2 — Structural rework (parallel)

| Workstream | Workers | Files |
|-----------|---------|-------|
| B1 Option A app migration | 8 | gps_app, imu_app, baro_app, nav_app, afm_app, telem_app, mlog_app, sys_app |
| B5 sim_harness Option A + fictional symbols | 2 | sim_harness/design.md + interfaces.md, sim_dynamics/design.md, sim_scenario/design.md |

10 workers in parallel.

### Phase 3 — Verification and re-review

1. Run `tools/traceability.py`; gate must pass with 371 reqs.
2. Re-spawn 21 reviewer agents (3 per section, S3-S9) to confirm corrective actions closed each Major/Minor RID. Reviewer briefs must include the corrected baseline and explicit instruction to verify each prior finding's closure.
3. Update each section record (`docs/reviews/pdr/sections/SX_*.md`) with final dispositions and CHAIR PROCEED verdict.
4. Update master log (`rid_rfa_log.md`) so all RIDs/RFAs are CLOSED or DISPOSED-with-CDR-deferred.

### Phase 4 — S10 closure

1. Author S10 closure draft (`docs/reviews/pdr/closure_memo.md`).
2. Spawn `project-chief-engineer` for the final-gate verdict (PASS / PASS-WITH-ACTIONS / FAIL).
3. Update `ai/memory/lessons-learned-software-lead.md` with three lessons:
   - LibJuno include tree must be in reviewer briefs (already saved 2026-05-03).
   - Upstream pivots (Option A, status code catalog, period cascade) must propagate via project-wide sweeps before downstream review convenes.
   - 21-agent parallel fan-out is tractable but the consolidation step is the bottleneck — plan for batched dispositions when findings cluster.

## 6. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| B1 Option A migration produces inconsistent app patterns across the 8 workers | Cross-section drift; re-review fails | Single shared brief template; spot-check by Lead before re-review |
| B5 device_lib `Inject` decision (recast vs. extend API) creates S3 cross-section re-open | Re-review of device_lib design | Lead recommends pty write-fd recast; no S3 re-open needed |
| Phase 1 sweeps miss occurrences in code blocks / mermaid diagrams (grep limitations) | RIDs survive into re-review | Use line-number-grep verification before re-spawning reviewers |
| `JUNO_MSG_NAV_STATE_T` field-shape decision between `tPosLla[3]` packed vs `dLat/dLon/fAltMHae` flat affects telem AND mlog AND nav | Cross-module coordination | B6 pins this once at L1 baseline; all downstreams reference verbatim |
| Re-spawned reviewers raise *new* findings beyond the corrective scope | Sprint scope creep | Brief reviewers explicitly to verify closure of *prior* RIDs only; new findings flagged as RFA-CDR |
| Chair-action items (SYS-016, LOG-007, IMU model) not closed timely | Phase 3 cannot complete | Phase 0 explicitly Chair-led; track via dedicated comms |

## 7. Lessons Carried Into This Sprint

From `lessons-learned-software-lead.md`:
- Cross-module API drift (the AFM phase enum lesson, 2026-05-02) → **C1-C4 are exact recurrences at scale**. Mitigation: project-wide sweep workstreams (B1-B5) rather than per-section iteration.
- LibJuno include tree must be in reviewer briefs (2026-05-03 lesson) → reviewer briefs for Phase 3 will include `libjuno/include/juno/` baseline references.
- PM escalation pattern (2026-05-02) → Phase 0 batches Chair actions into a single ask rather than letting them block per-section iteration.

## 8. Sprint Plan Approval

| Field | Value |
|-------|-------|
| Plan author | Software Lead |
| Plan date | 2026-05-03 |
| Predecessor | PDR S1-S9 review consolidated 2026-05-03 |
| Successor | PDR S10 closure |
| Estimated duration | 4 phases; Phases 0+1 in one parallel batch; Phase 2 in one parallel batch (≥10 workers); Phase 3 single re-review batch; Phase 4 closure |
| Estimated worker invocations | Phase 0: 1 worker; Phase 1: 3 workers; Phase 2: 10 workers; Phase 3: 21 reviewers; Total ≈ 35 agent invocations |
| Chair approval line | Project Manager — _____________________ — _____________________ |
