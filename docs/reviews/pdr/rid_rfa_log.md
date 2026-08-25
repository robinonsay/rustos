# Juno FT1 PDR — RID and RFA Master Log

## Purpose

This file is the **master cross-section index** of every Review Item
Discrepancy (RID) and Request For Action (RFA) raised during the Juno FT1
Preliminary Design Review (PDR). It exists to give the Chair, the Software
Lead, and reviewers a single place to scan totals, status, and ownership
across all PDR sections.

Per-section disposition records under `docs/reviews/pdr/sections/SX_*.md`
are the **authoritative content sources** for each RID and RFA: full
descriptions, recommended resolutions, Chair rationales, and verification
notes live there. This master log mirrors only the index columns and the
roll-up statistics. If this log and a section record disagree, the section
record wins; this log must be corrected to match.

This log starts empty. Entries are appended by the Software Lead at the end
of each PDR section as items are formally logged in their section record.

## Conventions

### RID ID Format

```
PDR-RID-S<X>-<NNN>
```

- `S<X>` — section number, `S1` through `S10`
- `<NNN>` — zero-padded sequence within the section, starting at `001`

Example: `PDR-RID-S3-002` is the second RID raised in Section 3.

### RFA ID Format

```
PDR-RFA-S<X>-<NNN>
```

Same numbering rules as RIDs. RID and RFA sequences are independent
within a section (a section can have `PDR-RID-S3-001` and `PDR-RFA-S3-001`
simultaneously without collision).

### Severity (RIDs only)

| Severity | Meaning |
|----------|---------|
| Major | Defect that, if unresolved, blocks PDR exit or creates flight risk. |
| Minor | Defect that should be fixed but does not block PDR exit on its own. |
| Editorial | Wording, formatting, or typographical issue with no technical impact. |

RFAs are **advisory** and carry no severity field; they request an action
or investigation rather than declaring a defect.

### Source Reviewer Codes

| Code | Reviewer Role |
|------|---------------|
| MAE | Mission Assurance Engineer |
| SSE-R | Senior Software Engineer (Reviewer) |
| CE | Chief Engineer |
| CHAIR | Item raised by the PDR Chair directly |

### Disposition Codes

| Code | Meaning |
|------|---------|
| ACCEPT | Chair accepts the finding as-stated; corrective action will be taken. |
| ACCEPT-MOD | Chair accepts with modifications to scope, wording, or recommended resolution. |
| REJECT | Chair rejects the finding; no corrective action required. |
| DEFER | Chair defers the finding to a later review (e.g., CDR) or sprint. |
| CLOSE-NO-ACTION | Item is withdrawn or determined non-applicable; no action required. |
| OPEN | Default state; the Chair has not yet rendered a disposition. |

## Status Legend

| Status | Definition |
|--------|------------|
| OPEN | Logged but the Chair has not yet rendered a disposition. |
| DISPOSED | Chair has rendered a disposition. Corrective action may still be pending. |
| CLOSED | Disposition rendered AND the corrective action has been verified complete by the responsible reviewer or the Software Lead. |

A `CLOSE-NO-ACTION` or `REJECT` disposition transitions the item directly
from OPEN to CLOSED at the moment of disposition (no pending action). All
other dispositions go OPEN to DISPOSED, and require explicit verification
before transitioning to CLOSED.

## RID Master Table

<!-- Append one row per RID. Keep this table sorted by ID. -->

| ID | Section | Severity | Source | Title | Disposition | Owner | Target | Status |
|----|---------|----------|--------|-------|-------------|-------|--------|--------|
| PDR-RID-S1-001 | S1 | Minor | MAE | §11 traceability table cites sections lacking matching design tags | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-002 | S1 | Minor | MAE | §4 bus catalog row for `JUNO_MSG_MLOG_RECORD_T` is internally inconsistent | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-003 | S1 | Minor | MAE | §3.3 module catalog header paths diverge from architecture.md | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-004 | S1 | Minor | MAE | §5 state-diagram shows Recovery as peer state contradicting prose | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-005 | S1 | Major | SSE-R | mlog@10ms cannot satisfy SW-REQ-SYS-011 full-rate IMU logging | ACCEPT-MOD | Software Lead | pre-S2 | DISPOSED |
| PDR-RID-S1-006 | S1 | Minor | SSE-R | §8.1 pseudocode demonstrates bare `.tOk` without `JUNO_ASSERT_OK` | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-007 | S1 | Minor | SSE-R | `JUNO_MSG_GPS_NMEA_RAW_T.acSentence[N]` array dimension unspecified | ACCEPT | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S1-008 | S1 | Minor | SSE-R | `JUNO_TIME_PROVIDER_T` named in §8.1 but type signature undefined | ACCEPT | Software Lead | pre-S2 | DISPOSED |
| PDR-RID-S1-009 | S1 | Minor | SSE-R | §8.1 step 5 prose names `&log` but pseudocode passes `&time` | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-010 | S1 | Editorial | SSE-R | §8.1 step 4 placement implies broker constructed after domain libs | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S1-011 | S1 | Major | CE | SW-REQ-SYS-016 phase-set is narrower than `JUNO_PHASE_T` enum | ACCEPT | Software Lead → Chair | pre-S5 | DISPOSED |
| PDR-RID-S1-012 | S1 | Minor | CE | `JUNO_TIME_PROVIDER_T` injection seam absent from `conventions.md` | ACCEPT | Software Lead | pre-S2 | DISPOSED |
| PDR-RID-S1-013 | S1 | Minor | CE | "Apps subscribe at `Init()`" rule appears only in system_design §8.1 | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RID-S2-001 | S2 | Major | MAE | `juno::app::APP_ROOT_T` forward-declared but never defined | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-002 | S2 | Minor | MAE | Status-code naming drift: NULL_POINTER vs NULLPTR_ERROR (light framing) | ACCEPT | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S2-003 | S2 | Minor | MAE | sch L2 uses SCH_LIB_IMPL_T<N> but L1 system_design §8.1 uses SCH_IMPL_T | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-004 | S2 | Minor | MAE | log LOG-007 rationale "stdout" vs design "stderr" | ACCEPT | Chair (PM) | batched-S2 | DISPOSED |
| PDR-RID-S2-005 | S2 | Editorial | MAE | sch §11 traceability table title disagrees with requirements.json title | ACCEPT | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S2-006 | S2 | Editorial | MAE | device template<const size_t N> redundant `const`; sch uses template<size_t N> | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-007 | S2 | Major | SSE-R | `APP_ROOT_T` undefined — sch::Register type contract unverifiable | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-008 | S2 | Major | SSE-R | `JUNO_MODULE_ROOT(SCH_LIB_API_T<N>, ...)` missing `JUNO_MODULE_ARG` | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-009 | S2 | Major | SSE-R | Fabricated status codes — 7 symbols not in `juno/status.h` | ACCEPT-MOD | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S2-010 | S2 | Major | SSE-R | `std::sqrt`/`<cmath>` freestanding-permitted claim unsubstantiated (kmat) | ACCEPT-MOD | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-011 | S2 | Minor | SSE-R | time `GetUs()` ROOT→IMPL upcast pattern undocumented | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-012 | S2 | Minor | SSE-R | log variadic `(&LogFmt)(...)` vtable well-formedness unstated | ACCEPT-MOD | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-013 | S2 | Minor | SSE-R | `vsnprintf` freestanding availability unaddressed (log) | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-014 | S2 | Minor | SSE-R | device ring overflow drop-oldest creates undocumented nmea hazard | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-015 | S2 | Minor | SSE-R | kmat Invert LU pivot tiebreak unspecified — KMAT-009 determinism gap | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RID-S2-016 | S2 | Major | CE | S1-AI-005 mlog@5ms not absorbed into time §8 / sch §7.1 | ACCEPT | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S2-017 | S2 | Major | CE | device_lib<N> templated; sys_app consumes non-templated form | ACCEPT-MOD | Software Lead → S8 | at-S8 | DISPOSED |
| PDR-RID-S2-018 | S2 | Major | CE | `juno::app::APP_ROOT_T` fictional — re-raise with cross-section evidence | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-019 | S2 | Major | CE | Status-code symbols don't exist — re-raise with project framing | ACCEPT-MOD | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S2-020 | S2 | Minor | CE | system_design §8.1 SCH_IMPL_T vs L2 SCH_LIB_IMPL_T<N> — re-raise w/ re-open | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RID-S2-021 | S2 | Major | CHAIR | FT1 time/sch designs reinvent LibJuno interfaces; app lifecycle hook names mismatch | ACCEPT-MOD (Option A) | Software Lead | pre-S3 | DISPOSED |
| PDR-RID-S10-001 | S10 | Major | MAE | imu_app §4.1:146 uses `tApp.tRoot.tApi` instead of canonical `tApp.tRoot.ptApi` (drift trap) | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-002 | S10 | Major | MAE | sim_harness/interfaces.md §4.5:305 cites legacy `sch_lib::Run`; canonical is `juno::sch::SCH_API_T<8,200>::Execute()` | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-003 | S10 | Major | SSE-R | telem/design.md §4.5:206 references `tNav.fAltMHae` — no such field on NAV_STATE_T (compile failure) | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-004 | S10 | Major | SSE-R | sys_app two-level SYS_APP / SYS_APP_IMPL_T embedding produces UB cast (strict aliasing); restructure as single JUNO_MODULE_DERIVE | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-005 | S10 | Major | SSE-R | nav_app §4.4:179 uses `juno::time::TimestampToMicros(*_ptTime, ...)` free-function — TimestampToMicros is non-static member of TIME_ROOT_T | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-006 | S10 | Minor | MAE | baro/design.md §1:17 says "the **app** owns bus access" — composition root owns it; baro_app/baro_lib don't touch I2C | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-007 | S10 | Minor | MAE | lora/design.md §3.2:65 mermaid label "5 ms tick" feeds telem_app — telem_app is 500 ms | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-008 | S10 | Minor | MAE | afm/design.md §4.1:108 JUNO_PHASE_T home self-loops; pin to libs/afm_lib/include/afm_lib/afm_api.hpp | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-009 | S10 | Minor | MAE | sim_sensors/design.md §4.3:168 references stale "tSensorCfg substructure" — SIM_SCENARIO_T is flat | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-010 | S10 | Minor | MAE | App aggregate naming drift: SYS_APP and GPS_APP lack `_T` suffix; 6 other apps carry it (conventions §3 inconsistency) | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-011 | S10 | Minor | SSE-R | telem/design.md §4.5 double→float altitude narrowing undocumented (relates to S10-003 fix) | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-012 | S10 | Minor | SSE-R | mlog/design.md §6.6 NAV record fAltHaeM double→float narrowing undocumented | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-013 | S10 | Minor | SSE-R | kMaxDropouts mismatch: sim_sensors §4.3 declares 16, sim_scenario §4.3 declares 8; pin in one header | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-014 | S10 | Minor | SSE-R | nav/design.md §4.5 extension status-code list skips offset +2 between kNavStatusGpsStale (+1) and kNavStatusConvergenceFail (+3); document gap | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |
| PDR-RID-S10-015 | S10 | Minor | SSE-R | sim_harness/interfaces.md §4.3 step 4 transcoding silently narrows double→float for sensor noise sigmas; document narrowing | ACCEPT | Software Lead | 2026-05-03 (delta-PDR remediation sprint) | CLOSED |

## RFA Master Table

<!-- Append one row per RFA. Keep this table sorted by ID. -->

| ID | Section | Source | Title | Disposition | Owner | Target | Status |
|----|---------|--------|-------|-------------|-------|--------|--------|
| PDR-RFA-S1-001 | S1 | MAE | WCET claim "fits in 5 ms with margin" carries no measurement basis | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-002 | S1 | MAE | POSIX/Pico2 functional-equivalence verification artifact not specified | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-003 | S1 | MAE | Trick injection callback semantic contract not stated | ACCEPT | Software Lead | pre-S2 | DISPOSED |
| PDR-RFA-S1-004 | S1 | MAE | §3 over-tags SW-REQ-SYS-043 at System Overview level | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-005 | S1 | MAE | SYS-016 vs AFM-002 phase-set disagreement (PM action) | ACCEPT | Chair (PM) | pre-S5 | DISPOSED |
| PDR-RFA-S1-006 | S1 | MAE | §8.1 pseudocode contains project-history annotation | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-007 | S1 | SSE-R | Trick sensor-injection seam not sketched at L1 | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-008 | S1 | SSE-R | System lifecycle state enum (`JUNO_FSW_STATE_T`) undefined at L1 | ACCEPT | Software Lead | pre-S8 | DISPOSED |
| PDR-RFA-S1-009 | S1 | SSE-R | Health bitmap bit assignments not specified at L1 | ACCEPT | Software Lead → S8 board | at-S8 | DISPOSED |
| PDR-RFA-S1-010 | S1 | SSE-R | `NAV_STATE_T.bValid` single flag conflates two failure modes | ACCEPT | Software Lead → S7 board | at-S7 | DISPOSED |
| PDR-RFA-S1-011 | S1 | SSE-R | SYS-016 phase-text 4 vs AFM-002 5 (RTM impact) | ACCEPT | (closed by RID-S1-011) | pre-S5 | DISPOSED |
| PDR-RFA-S1-012 | S1 | CE | `requirements/index.md` count drift vs JSON reality | ACCEPT | Software Lead | pre-S2 | DISPOSED |
| PDR-RFA-S1-013 | S1 | CE | WCET §8.2 assertion lacks per-module budget table | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-014 | S1 | CE | POSIX-only test platform may mask Pico2-specific failure modes | ACCEPT | Software Lead | batched-S1 | DISPOSED |
| PDR-RFA-S1-015 | S1 | CE | IMU TBD has no decision-deadline trigger at S1 | ACCEPT | Chair (PM) | CDR | DISPOSED |
| PDR-RFA-S1-016 | S1 | CE | SYS-014 numeric threshold deferred to nav L2 with no system-level cap | ACCEPT | Software Lead → S5 board | at-S5 | DISPOSED |
| PDR-RFA-S2-001 | S2 | MAE | sch_lib has no overrun-detection requirement (FLAG-1) | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RFA-S2-002 | S2 | MAE | log_lib variadic API surface (FLAG-3) | ACCEPT | (closed by RID-S2-012) | batched-S2 | DISPOSED |
| PDR-RFA-S2-003 | S2 | MAE | POST-bitmap responsibility for foundation-lib New() failures asymmetric | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-004 | S2 | MAE | sch_lib POSIX vs Pico2 wait primitive divergence informally described | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RFA-S2-005 | S2 | MAE | kmat kPivotEpsilon defaults stated "tuned in nav L2" without forcing function | ACCEPT | Software Lead → S5 board | at-S5 | DISPOSED |
| PDR-RFA-S2-006 | S2 | MAE | time §4.2 references conventions §4.3 incorrectly (should be §1.3) | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RFA-S2-007 | S2 | SSE-R | time_lib no health bit — exemption rationale should cross-reference SYS-058 | ACCEPT | Software Lead | pre-S3 (in C-1) | DISPOSED |
| PDR-RFA-S2-008 | S2 | SSE-R | POST-bitmap New() failure attribution all five libs (same root as RFA-003) | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-009 | S2 | SSE-R | sch overrun detection diagnostic not promoted to requirement (same root as RFA-001) | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RFA-S2-010 | S2 | SSE-R | kmat bit-identical POSIX/Pico2 claim may be optimistic for sqrt/Invert | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-011 | S2 | SSE-R | log eMinLevel runtime non-mutability relies on convention, not enforcement | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-012 | S2 | SSE-R | sch static constexpr namespace-scope -Wunused-variable potential (informational) | CLOSE-NO-ACTION | — | — | CLOSED |
| PDR-RFA-S2-013 | S2 | CE | kmat header-only deviation should be promoted to project-level exception | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-014 | S2 | CE | kmat Invert/QuatNormalize libm linkage should be explicitly called out | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-015 | S2 | CE | device_lib POSIX openpty requires -lutil on Linux | ACCEPT | Software Lead | batched-S2 | DISPOSED |
| PDR-RFA-S2-016 | S2 | CE | sch::Register period-multiple precondition could be static_assert in templated overload | DEFER | — | CDR | DISPOSED |
| PDR-RFA-S2-017 | S2 | CE | Pico2 sch time_us_64 busy-wait power consumption | DEFER | — | CDR | DISPOSED |

## Statistics

The Software Lead updates these counters at the end of each section. All
counters start at zero. Sum of `OPEN + DISPOSED + CLOSED` must equal the
total RID count and total RFA count respectively.

### RID Counters

| Metric | Count |
|--------|-------|
| Total RIDs | 49 |
| Major | 17 |
| Minor | 29 |
| Editorial | 3 |
| OPEN | 0 |
| DISPOSED | 26 |
| CLOSED | 23 |

### RFA Counters

| Metric | Count |
|--------|-------|
| Total RFAs | 33 |
| OPEN | 0 |
| DISPOSED | 28 |
| CLOSED | 5 |

### By Section

| Section | RIDs | RFAs | Open Items |
|---------|------|------|------------|
| S1 | 13 | 16 | 0 |
| S2 | 21 | 17 | 0 |
| S3 | 0 | 0 | 0 |
| S4 | 0 | 0 | 0 |
| S5 | 0 | 0 | 0 |
| S6 | 0 | 0 | 0 |
| S7 | 0 | 0 | 0 |
| S8 | 0 | 0 | 0 |
| S9 | 0 | 0 | 0 |
| S10 | 15 | 5 | 0 |

**S10 RFAs (carry-forward to LibJuno / future sprints):**
1. `juno::app::AppInit(...)` documented in app_api.hpp doxygen but not published as a function — LibJuno backlog.
2. `JUNO_MSG_BUS_VARIANT_T` placeholder definition needed before code lands — composition-root sprint.
3. Broker / SD / device template placeholder values (`kBrokerPipes`, `kBrokerRegistry`, `kDefaultWriteBufBlocks`, `kDefaultRingCap`) need authoritative pins — L2 design extensions.
4. Option C migration of `SIM_SENSORS_RAW_T` / `SIM_BARO_REGS_T` into imu_lib / baro_lib public headers — deferred future sprint.
5. NASA Trick `exec_get_sim_time()` symbol/header verified at Trick-environment integration time.

## Cross-Section Re-Open Log

Per lessons-learned: when a finding raised in a later section invalidates
or contradicts content already disposed in an earlier section, the earlier
section's affected items must be re-opened. Record those events here so
that the Chair has a clear chain-of-custody for the re-opening.

| Date | Original Section | New ID | Description | Original-Section Status After Re-Open |
|------|------------------|--------|-------------|---------------------------------------|
| 2026-05-03 | S1 | (S2-AI-004) | system_design.md §8.1 composition root pseudocode rewritten under Option A to use LibJuno's `juno::time::TIME_ROOT_T` and `juno::sch::SCH_ROOT_T<8, 200>`; replaces `kMlogAppPeriodMs = 5` cascade originally executed under S1-AI-005 with the LibJuno-impl form. | S1 verdict unchanged (CHAIR PROCEED); §8.1 content is regenerated and re-verified by S2-AI-019 traceability gate. |
| 2026-05-03 | S1 | (S10-Sprint) | Corrective Action Sprint executed: SYS-016 amended to include `pre-launch` as initial phase (closes S1-AI-011 / SYS-016 vs AFM-002 phase-set disagreement); LOG-007 rationale "stdout"→"stderr" (closes S2-RID-S2-004); IMU model locked to MPU-6050 (closes FLAG-4 / S1-AI-022); `JUNO_FSW_STATE_T` enum added to conventions §4.7 (closes S1-AI-018 / PDR-RFA-S1-008); sys_app §4.3 health-bitmap bit-assignment table added (closes S1-AI-019 / PDR-RFA-S1-009); nav numeric `kNavGpsBoundM_default = 200.0` pinned (closes S1-AI-023 / PDR-RFA-S1-016); `JUNO_MSG_NAV_STATE_T` field-shape table pinned in nav §4.1 (closes telem ↔ nav field-precision RID); §6.1 conventions exceptions extended for `nmea_lib` and `sim_harness`. | S1 + S2 verdicts unchanged (CHAIR PROCEED); affected items transition from DISPOSED to CLOSED. |
| 2026-05-03 | S2 | (S10-Sprint) | Phase 1 mechanical sweeps (B2 status codes; B3 mlog @ 5 ms; B4 LibJuno canonical types) and Phase 2 structural rework (B1 Option A app lifecycle for all 8 apps; B5 sim_harness Option A + sim_dynamics extern C drop + sim_sensors Option D + pty seam) applied across `docs/design/`. Closes Root Causes C1–C4 enumerated in `docs/reviews/pdr/corrective_sprint_plan.md` §1. Verified by 4 batched MAE reviewers (S6/S7/S8/S9), traceability.py exit 0 (371 reqs, 370 test specs), and AC-1..AC-15 grep verification. | S2 verdict unchanged (CHAIR PROCEED); affected items transition from DISPOSED to CLOSED. |
| 2026-05-03 | S10 | (Delta-PDR Remediation Sprint) | All 15 PDR-RID-S10-001 through PDR-RID-S10-015 findings remediated: Phase 1 Lead-direct atomic edits (4 Majors + 9 Minors across imu_app, sim_harness, telem, nav_app, baro, lora, afm, sim_sensors, gps_app, mlog, sim_scenario, nav, sim_harness/interfaces.md); Phase 2 sys_app structural restructure (Δ-MAJOR-4 + Δ-MINOR-5 sys_app portion) by software-systems-engineer worker (collapsed `SYS_APP` + `SYS_APP_IMPL_T` into single `SYS_APP_T JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` matching the canonical single-level pattern of the other 7 apps); Phase 3 MAE review APPROVED on first iteration (11/11 checks PASS); Phase 4 re-verification grep PASS for all 15 findings + traceability.py exit 0 (371 reqs, 370 test specs). | All 15 S10 RIDs transition from OPEN → CLOSED (ACCEPT disposition); CE GO-WITH-PRE-IMPLEMENTATION-ACTIONS conditions satisfied. |

## Maintenance Notes

- Do not edit per-item descriptions in this file; edit the section record
  and copy the title/disposition/status fields here.
- When a section record is closed (Chair signs the verdict), confirm every
  row sourced from that section is consistent with the section record
  before the next section convenes.
- This file is plain Markdown; no scripts currently parse it. If tooling
  is added later, preserve the table column order above.
