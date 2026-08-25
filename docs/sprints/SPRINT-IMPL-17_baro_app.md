---
sprint_id: SPRINT-IMPL-17
module: baro_app
wave: 5 (Sensor Apps — second)
predecessors: SPRINT-IMPL-00 (bus_variant + capacities), SPRINT-IMPL-03 (time_lib), SPRINT-IMPL-06 (sch_lib), SPRINT-IMPL-08 (baro_lib)
status: CLOSED
opened: 2026-05-11
closed: 2026-05-11
ce_verdict: APPROVED (unconditional, 0 findings)
pm_signoff: pending
---

# SPRINT-IMPL-17 Closure Record — `baro_app`

## 1. Sprint Goal

Implement the `baro_app` Wave 5 sensor-publishing application per [`docs/design/baro_app/design.md`](../design/baro_app/design.md): a thin TDM-scheduled View-layer publisher that, on every 50 ms tick (20 Hz), reads one barometric sample from `baro_lib`, attaches a monotonic-µs timestamp, and publishes a `JUNO_MSG_BARO_SAMPLE_T` on the LibJuno software broker. Wave 5 progress: 2 of 3 sensor apps now CLOSED (after SPRINT-IMPL-16 `imu_app`); SPRINT-IMPL-18 (`gps_app`) remains.

## 2. PM-Approved Scope Decisions (Q-batch, 2026-05-11)

| Q | Decision | Rationale |
|---|----------|-----------|
| Q1 | Lead-direct create `libs/baro_lib/include/baro_lib/baro_msg.hpp` (Option A) as Phase 0 prerequisite | `JUNO_MSG_BARO_SAMPLE_T` specified at this path in L2 §4.6 but not authored in any prior sprint (Wave 0 only created bus variant; SPRINT-IMPL-08 baro_lib only authored internal `BARO_SAMPLE_T`). Pure POD (~127 LoC, no logic), L2-canonical path, no Wave 3 reopen needed. Mirrors SPRINT-IMPL-16 PM Q2 imu_msg.hpp pattern. |
| Q2 | Manual aggregate-init for `BaroAppInit` (RFA #1) — `tApp.tRoot.ptApi = &tApi; tApp.tRoot.JUNO_FAILURE_HANDLER = ...;` | `juno::app::AppInit` still not published in `libjuno/include/juno/app/app_api.hpp` (only docstring-referenced at lines 58, 97). Same fallback as SPRINT-IMPL-16; Wave 5 consistency requirement per `sensor_apps.md §4` exit gate. |
| Q3 | TC-007 and TC-014 reinterpret legacy "baro health topic / fault bitmap" language as `JUNO_MSG_BARO_SAMPLE_T.bValid` boolean | L2 §4.6 has ONE published topic with `bValid` as health observable; "fault bitmap 0x00/0x01/0xFF" PDR-era wording reinterpreted as success-rate patterns 0/3, 1/3, 3/3. Q3 deviation cited in test file-header doxygen lines 16-20. |
| Q4 | TC-010 implemented as POSIX self-consistency over 200 scripted samples (no Pico2 reference vector required) | No Pico2 reference vector exists; per L2 §3.3, `baro_app` is platform-agnostic (single shared impl across POSIX/Pico2), so the deterministic-pass-through nature satisfies functional-equivalence intent. Q4 deviation cited in test file-header doxygen lines 21-24. |
| Q5 | TC-008 and TC-009 (Demonstration/Inspection) deferred to HIL CDR with placeholder filenames | Per SDP `sensor_apps.md §3 SPRINT-IMPL-17 AC-J`; Demo procedures executed post-CDR at HIL bench. |

## 3. Acceptance Criteria — Final Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | All 4 deliverable files authored and reviewer-APPROVED | MET | hpp/CMake/cpp/test — see §6 reviewer verdicts |
| AC-2 | `BARO_APP_T` declared via single-level `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` | MET | [baro_app.hpp:150](../../apps/baro_app/include/baro_app/baro_app.hpp#L150) |
| AC-3 | Hook downcast via first-member `reinterpret_cast`; vtable dispatch via `_pt*->ptApi->Hook(*_pt*)`; zero `tApi->` | MET | [baro_app.cpp:185](../../apps/baro_app/src/baro_app.cpp#L185), :245, :348 (downcasts); :188/:194/:249/:261/:319 (vtable dispatch); grep `\btApi\->` returns zero hits |
| AC-4 | Per-tick timestamp via canonical member-form `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` | MET | [baro_app.cpp:249-255](../../apps/baro_app/src/baro_app.cpp#L249-L255) |
| AC-5 | `BaroAppInit` uses manual aggregate-init per Q2 | MET | [baro_app.cpp:109-144](../../apps/baro_app/src/baro_app.cpp#L109-L144) mirrors imu_app:92-121 |
| AC-6 | Per-tick Sample + Publish observable; Sample precedes Publish | MET | TC-002, TC-003, TC-013 (iter-2 paired-seq pattern) — see §6 |
| AC-7 | `static constexpr uint32_t kBaroAppPeriodMs = 50;` in public header | MET | [baro_app.hpp:80](../../apps/baro_app/include/baro_app/baro_app.hpp#L80) |
| AC-8 | UNINITIALIZED → RUNNING → DEGRADED state machine; counters observable | MET | hpp:107-112 enum; hpp:172/175/178 fields; cpp:200/290/294/349 transitions |
| AC-9 | `baro_app` source contains NO I²C transport seam | MET | grep `BARO_LIB_BUS_T\|i2c\|I2C` yields only one doxygen-comment hit on hpp:85 (timeout bounds I²C inside baro_lib); zero code-level coupling |
| AC-10 | All 10 SW-REQ-BARO-APP-* code-tagged on single-line `@req`; all 12 implemented TCs are `TEST_F` with single-line `@verify`; 2 Demo deferred | MET | hpp:72,82,114,181 + cpp:86,150,208,330 cover SW-REQ-BARO-APP-001..010; 12 TEST_F with 12 single-line `@verify` tags; TC-008/-009 deferred per Q5 |
| AC-11 | Test fixture wires every DI dep (stub baro_lib + fake TIME + REAL broker + recording pipe) | MET | [baro_app_test.cpp:138-167](../../apps/baro_app/tests/baro_app_test.cpp#L138-L167) |
| **AC-12** | **Gate G1 PASS — POSIX build + ctest** | **MET** | `100% tests passed, 0 tests failed out of 18` (17 prior + 12-TEST_F baro_app_test) |
| **AC-13** | **Gate G2 PASS — `tools/traceability.py` exit 0** | **MET** | 376/125/131/376 (baseline 376/115/123/376 → delta +10 code, +8 @verify; SW-REQ-BARO-APP-007/-008 are Inspection-method, no automated test) |
| AC-14 | All files ≤500 lines; compiler-clean under FSW flag set | MET | hpp=227, cpp=354, test=499, cmake=116, msg=127 |
| AC-15 | Project Chief Engineer issues PASS verdict | MET | CE APPROVED unconditional 2026-05-11 (§8) |

## 4. Deliverable File Inventory

5 production files (4 in-sprint + 1 Phase 0 prerequisite):

| # | Path | Lines | Phase | Author | Final Status |
|---|------|-------|-------|--------|--------------|
| 1 | `apps/baro_app/include/baro_app/baro_app.hpp` | 227 | 1 | senior | APPROVED iter-1 (0 findings) |
| 2 | `apps/baro_app/CMakeLists.txt` | 116 | 1 | junior | APPROVED iter-1 (0 findings) |
| 3 | `apps/baro_app/src/baro_app.cpp` | 354 | 2 | senior | APPROVED iter-1 + Lead-direct atomic edit (per §6) |
| 4 | `apps/baro_app/tests/baro_app_test.cpp` | 499 | 2 | senior (test author) | APPROVED iter-2 (1 MAJOR addressed; per §6) |
| 5 | `libs/baro_lib/include/baro_lib/baro_msg.hpp` (Phase 0 prerequisite per PM Q1) | 127 | 0 | Lead-direct | N/A (Lead-direct; pure POD with no logic) |

**Lead-direct artifacts (not in deliverable count):**
- Phase 0: directory tree `apps/baro_app/{include/baro_app,src,tests}` + `add_subdirectory(baro_app)` in [`apps/CMakeLists.txt`](../../apps/CMakeLists.txt).
- Phase 0 (PM Q1 fix): `libs/baro_lib/include/baro_lib/baro_msg.hpp` — pure POD defining `JUNO_MSG_BARO_SAMPLE_T` per L2 §4.6.
- Phase 2 ADVISORY resolution: added `SW-REQ-BARO-APP-007` and `SW-REQ-BARO-APP-008` to `OnProcess` `@req` tag for completeness (one-line edit at [baro_app.cpp:208](../../apps/baro_app/src/baro_app.cpp#L208)).

## 5. Workflow Phases — Iteration Summary

| Phase | Description | Agents | Iterations | Final |
|-------|-------------|--------|------------|-------|
| 0 | Lead pre-flight: traceability baseline + directory tree + add_subdirectory wire-up + (PM-Q1) Lead-direct `baro_msg.hpp` | 1 (Lead) | 1 | Complete |
| 1 | Workers fan-out: baro_app.hpp (senior) + CMakeLists.txt (junior) — parallel | 2 | 1 each | Complete |
| 1-review | Reviewers fan-out: baro_app.hpp (APPROVED iter-1) + CMakeLists.txt (APPROVED iter-1) — parallel | 2 | 1 each | Complete (0 findings on both) |
| 2 | Workers fan-out: baro_app.cpp (senior) + baro_app_test.cpp (senior test author) — parallel | 2 | 1 each | Complete |
| 2-review | Reviewers fan-out: baro_app.cpp (APPROVED iter-1 + 1 ADVISORY Lead-direct) + baro_app_test.cpp (NEEDS CHANGES iter-1: 1 MAJOR on TC-013 → iter-2) | 2 | impl: 1; test: 2 | Complete |
| 3 | Lead-direct G1 + G2 gates | 1 (Lead) | 1 | Complete (both PASS) |
| 4 | Project Chief Engineer gate | 1 (CE) | 1 | APPROVED unconditional |

**Total agents:** 10 (4 workers across 2 phases + 5 reviewer invocations including 1 iter-2 + 1 CE).

## 6. Reviewer Verdicts (chronological)

| # | File | Phase | Reviewer | Verdict | Findings | Resolution |
|---|------|-------|----------|---------|----------|------------|
| 1 | baro_app.hpp | 1 | senior-software-engineer | APPROVED iter-1 | 0 findings | — |
| 2 | CMakeLists.txt | 1 | senior-software-engineer | APPROVED iter-1 | 0 findings | — |
| 3 | baro_app.cpp | 2 | senior-software-engineer | APPROVED iter-1 with 1 ADVISORY | SW-REQ-BARO-APP-007/-008 (Inspection-method) absent from `@req` tags | Lead-direct atomic edit: added IDs to OnProcess `@req` tag for completeness |
| 4 | baro_app_test.cpp | 2 | senior-software-engineer | NEEDS CHANGES iter-1 | 1 MAJOR (TC-013 vacuous ordering assertion) | Worker iter-2 |
| 5 | baro_app_test.cpp | 2 | senior-software-engineer | APPROVED iter-2 | 0 findings | — |

### Phase 1 reviewer outcomes
Both Phase 1 files passed on first iteration with zero findings — a significant improvement over SPRINT-IMPL-16 Phase 1 (which had 2 MAJOR CMake findings requiring Lead-direct fix). Attribution: the imu_app-precedent-matching brief eliminated the failure modes that had surfaced in IMU-16.

### Phase 2 impl reviewer finding (Lead-direct resolved)
- ADVISORY: `SW-REQ-BARO-APP-007` (HAE altitude — Inspection-method) and `SW-REQ-BARO-APP-008` (SI units — Inspection-method) absent from any `@req` annotation. `tools/traceability.py` does not error on Inspection-method requirements without code tags, but the brief's "all 10 IDs covered" completeness goal was unmet. Lead-direct atomic edit appended both IDs to the existing `OnProcess` single-line `@req` tag.

### Phase 2 test reviewer finding iter-1 (worker iter-2 resolved)
- MAJOR (TC-013 vacuous ordering): `EXPECT_GT(g_tBaroState.iSampleCallSeq, 0)` only proved Sample was called, not that Sample preceded Publish. A buggy impl publishing before sampling would pass.
  - **iter-2 fix (Option C — paired sequence numbers):** introduced a stub `BROKER_API_T` vtable used ONLY by TC-013 (other 11 tests untouched). Both `StubBaro_Sample` and `StubBroker_Publish` increment a shared `g_iCallSeq`; the first call gets the lower number. New assertions at test:460-462 are `EXPECT_GT(g_iLastPublishSeq, 0)` (Publish was called) AND `EXPECT_LT(g_tBaroState.iSampleCallSeq, g_iLastPublishSeq)` (Sample seq < Publish seq, proving order). A reversed-order regression would produce `g_iLastPublishSeq=1, iSampleCallSeq=2`, failing `EXPECT_LT`. iter-2 reviewer APPROVED with 0 findings.

## 7. Gate Results

### Gate G1 — POSIX build + ctest
```
$ cd /home/juno/juno_fsw/build_posix && cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && cmake --build . && ctest --output-on-failure
...
[100%] Built target baro_app_test
...
18/18 Test #18: baro_app_test ....................   Passed    0.00 sec
100% tests passed, 0 tests failed out of 18
Total Test time (real) =   0.34 sec
Exit 0
```

### Gate G2 — Traceability tool
```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       125   (Phase 0 baseline 115; delta +10 = SW-REQ-BARO-APP-001..010)
  Requirements with @verify:    131   (Phase 0 baseline 123; delta +8 = SW-REQ-BARO-APP-001..006 + -009 + -010; -007/-008 are Inspection-method)
  Requirements with test specs: 376
Exit 0
```

### Gate G3 — Pico2 cross-compile
**Not invoked.** Per L2 §3.3, `baro_app` is platform-agnostic (no `src/posix/`/`src/pico2/` split — single shared `src/baro_app.cpp` per CMakeLists:46-48). Per SDP `sensor_apps.md §2.3`, G3 deferred to SPRINT-IMPL-25 system integration.

## 8. Chief Engineer Verdict

**APPROVED unconditional** (2026-05-11).

> "All 15 acceptance criteria MET with independent evidence. Independent re-execution of the two automated gates reproduces the Lead's results exactly: traceability exits 0 with the +10 code / +8 @verify delta matching the 10 SW-REQ-BARO-APP-* code IDs and 8 unique @verify tags (SW-REQ-BARO-APP-007/-008 correctly absent as Inspection-method); ctest reports 18/18 PASS including the new `baro_app_test`. The L2 design contracts (single-level JUNO_MODULE_DERIVE, first-member reinterpret_cast downcast, vtable dispatch through `_pt*->ptApi->Method(*_pt*)`, member-form TimestampToMicros, manual aggregate-init for RFA #1, OnStart Probe+Configure, 50 ms public period constant, 3-state machine UNINIT→RUNNING→DEGRADED, value-init memcpy publish pattern) are honored verbatim in source. The Phase 0 Lead-direct authoring of `libs/baro_lib/include/baro_lib/baro_msg.hpp` correctly resolved the Wave 0→Wave 5 missed-link pattern and was concurrently PM-approved (Q1). The TC-013 iter-2 paired-sequence-number fix is logically sound: detects reversed-order regression, doesn't false-positive on omitted-Sample regression (caught by EXPECT_EQ on iSampleCallCount). No file exceeds the 500-line cap (test file at 499 — 1 line under hard limit). No cross-sprint ID conflicts or broken references; existing imu_app_test (test #17) still passes."

CE recommends the Lead proceed with SPRINT-IMPL-18 (`gps_app`) per the SDP §5 master sprint table, applying the Phase 0 mitigation upfront for `JUNO_MSG_GPS_FIX_T` / `JUNO_MSG_GPS_NMEA_RAW_T` / `JUNO_MSG_GPS_UTC_T` (per the carry-forward note in §10 below).

## 9. Requirements Closure

The 10 SW-REQ-BARO-APP-* IDs advance from `Active` to `Verified` per RTM `Verified` definition (≥1 code function tagged + ≥1 passing test tagged, except Inspection-method requirements which need only code-tagging).

| Requirement ID | Title | Code tagged | Test tagged | Status |
|----------------|-------|-------------|-------------|--------|
| SW-REQ-BARO-APP-001 | Execute Barometer App at 20 Hz | BaroAppInit + BaroApp_OnStart + BaroApp_OnProcess | TC-001 | Verified |
| SW-REQ-BARO-APP-002 | Acquire One Sample Each Cycle | BaroAppInit + BaroApp_OnProcess | TC-002 + TC-013 | Verified |
| SW-REQ-BARO-APP-003 | Publish Barometer Message on Software Bus | BaroAppInit + BaroApp_OnProcess | TC-003 + TC-013 | Verified |
| SW-REQ-BARO-APP-004 | Pure Pass-Through Without Filtering | BaroApp_OnProcess | TC-004 + TC-005 | Verified |
| SW-REQ-BARO-APP-005 | Timestamp Each Published Baro Message | BaroApp_OnProcess | TC-006 | Verified |
| SW-REQ-BARO-APP-006 | Publish Baro Health on Software Bus | BaroApp_OnProcess | TC-007 + TC-014 | Verified |
| SW-REQ-BARO-APP-007 | Report Altitude Referenced to WGS-84 Ellipsoid | BaroApp_OnProcess | (Inspection-method; HIL CDR TC-008) | Verified by code-tag + deferred Demo |
| SW-REQ-BARO-APP-008 | Publish Baro Quantities in SI Units | BaroApp_OnProcess | (Inspection-method; HIL CDR TC-009) | Verified by code-tag + deferred Demo |
| SW-REQ-BARO-APP-009 | POSIX Build Functional Equivalence | BaroApp_OnProcess | TC-010 (Q4 reinterpretation: POSIX self-consistency) | Verified |
| SW-REQ-BARO-APP-010 | Deterministic Baro Message Output | BaroApp_OnProcess | TC-011 + TC-012 | Verified |

## 10. Carry-Forward Notes (for SPRINT-IMPL-18 + -25)

1. **Wave 0 → Wave 5 missed-link pattern (now confirmed across two sprints)**: Both SPRINT-IMPL-16 (imu_msg.hpp) and SPRINT-IMPL-17 (baro_msg.hpp) required a Phase 0 Lead-direct prerequisite for the bus-message POD header. SPRINT-IMPL-18 should preemptively check for `JUNO_MSG_GPS_FIX_T`, `JUNO_MSG_GPS_NMEA_RAW_T`, and `JUNO_MSG_GPS_UTC_T` at `libs/gps_lib/include/gps_lib/gps_msg.hpp` and Lead-direct create the publisher-POD header(s) in Phase 0 if absent. The pattern is now reliable enough to fold into the SPRINT-IMPL-18 plan as a presumed Phase 0 task (not a contingent risk).

2. **RFA #1 — manual aggregate-init precedent (now established across two sprints)**: SPRINT-IMPL-16 set the precedent; SPRINT-IMPL-17 confirmed it. SPRINT-IMPL-18 should use the same manual aggregate-init pattern unless LibJuno publishes `juno::app::AppInit()` between sprints. The Wave 5 exit gate in `sensor_apps.md §4` enforces consistency across all three apps.

3. **TC-013 paired-sequence-number Sample-before-Publish ordering pattern**: The Option C fix (shared monotonic counter incremented by both stub Sample and stub Publish, with `EXPECT_LT(iSampleCallSeq, iLastPublishSeq)`) is now the canonical idiom for sensor-app ordering verification. Recommend SPRINT-IMPL-18 use it from iter-1 (don't rebuild the vacuous version IMU-16/BARO-17 iter-1 both produced). Document the pattern in the SPRINT-IMPL-18 test author brief upfront.

4. **L2 design drift for "broker advertise"**: L2 §4.3 mentions broker advertise but `BROKER_API_T` only publishes `Publish` and `RegisterSubscriber` (no Advertise). baro_app correctly omitted the advertise call and documented the deviation. Consider a minor SDP/L2 amendment in a future sprint to align L2 wording with the actual broker surface. Same drift exists in `juno::app::AppInit` references (L2 §4.2) — not blocking but worth a follow-up cleanup.

5. **L2 design references `JUNO_TIME_US_T`** which doesn't exist in LibJuno (canonical name is `JUNO_TIME_MICROS_T`). Recurring drift across baro_lib.hpp + baro_app L2 + baro_app.hpp + baro_app.cpp. Future sprints touching these L2 documents may want to do a documentation-only sweep.

## 11. Lessons Learned (cross-referenced)

Updated in `ai/memory/lessons-learned-software-lead.md` 2026-05-11 entries:
- "SPRINT-IMPL-17: imu_app-precedent-matching briefs eliminate Phase 1 iteration cost"
- "SPRINT-IMPL-17: Inspection-method requirement completeness — tag `@req` even though tool doesn't gate"
- "SPRINT-IMPL-17: Vacuous ordering assertions — pair monotonic-counter stubs for both producer and consumer"

Updated in `ai/memory/lessons-learned-senior-software-engineer.md` 2026-05-11 entry:
- "SPRINT-IMPL-17: TC-013 paired-sequence-number stub broker pattern for Sample-before-Publish ordering tests"

## 12. Authority

| Role | Identity | Verdict | Date |
|------|----------|---------|------|
| Software Lead | This sprint | CLOSE (pending PM signoff) | 2026-05-11 |
| Project Chief Engineer | (agent) | APPROVED unconditional, 0 findings | 2026-05-11 |
| Project Manager | Robin Onsay | Pending | — |
