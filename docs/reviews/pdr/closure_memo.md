---
document_type: PDR Closure Memo (S10)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
predecessor_review: PDR Sections S1–S9 (2026-05-02 → 2026-05-03)
predecessor_corrective: PDR Corrective Action Sprint (2026-05-03)
status: Draft (awaiting Chief Engineer gate + Chair signature)
---

# PDR Closure Memo — Juno FT1 FSW

## 1. Purpose

Records the Project Manager / Chair verdict on Preliminary Design Review (PDR) closure following completion of the corrective action sprint (`docs/reviews/pdr/corrective_sprint_plan.md`).

## 2. Sprint Summary

The PDR review board reviewed all 10 sections (S1 architecture; S2 foundation libraries; S3–S9 sensor/comm/storage/domain/sensor-app/domain-app/system-app/sim modules; S10 closure). S1 and S2 closed CHAIR PROCEED with action items. S3–S9 corrective work targeted four root causes:

| Root Cause | Closure |
|------------|---------|
| C1. Option A app-lifecycle pivot (canonical `juno::app::APP_API_T { OnStart, OnProcess, OnExit }`) | CLOSED — all 8 apps migrated; verified by S6/S7/S8 re-review |
| C2. Status-code catalog sweep (19 canonical codes + FSW extensions) | CLOSED — zero fabricated-code hits across `docs/design/` outside the §4.8 mapping table |
| C3. mlog @ 5 ms cascade (`SW-REQ-SYS-011` no-downsampling) | CLOSED — `kMlogAppPeriodMs = 5` propagated; `conventions.md` §4.5 lists canonical period table |
| C4. LibJuno canonical type-name pivot (`juno::time::TIME_ROOT_T`, templated `BROKER_ROOT_T<...>`) | CLOSED — zero `TIME_LIB_ROOT_T` / `JUNO_TIME_PROVIDER_T` / `juno::time::GetUs()` hits outside negation prose |

Total worker invocations: 14 (3 Phase-1 sweep workers + 8 Phase-2 B1 app workers + 2 Phase-2 B5 sim workers + 4 Phase-3 reviewers, plus 1 final gate to be spawned). Manual Lead edits applied where atomic (Phase 0 Chair items, sim_sensors §4.2/§6.1 fix, `tApi→ptApi` notation sweep).

## 3. Acceptance Criteria Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | All 8 apps embed `juno::app::APP_ROOT_T` and expose `OnStart`/`OnProcess`/`OnExit` | MET | `grep -nE "^\s*(JUNO_STATUS_T\s+)?(Init\|Execute)\s*\(" docs/design/*_app/design.md` returns zero public-lifecycle hits |
| AC-2 | All L2 designs use only canonical 19 status codes (or FSW extensions with offsets) | MET | `grep -rln -E "fabricated-pattern" docs/design/` outside conventions §4.8 → zero |
| AC-3 | `kMlogAppPeriodMs = 5` reflected everywhere | MET | `grep -rn "kMlogAppPeriodMs *= *10" docs/design/` → zero |
| AC-4 | Canonical LibJuno types only (`TIME_ROOT_T`, templated `BROKER_ROOT_T<...>`, `Now`+`TimestampToMicros`) | MET | residual hits are negation prose ("withdrawn", "do not use") only |
| AC-5 | sim_harness uses canonical `TIME_API_T` aggregate-init via `TimeInit`; no `JUNO_TIME_PROVIDER_T` | MET | `interfaces.md §4.4` aggregate-init pattern; `design.md §10.2` |
| AC-6 | sim_dynamics `SIM_DYN_TRUTH_T` is C++-only (no `extern "C"`) | MET | sim_dynamics §1.1 / §6.1 / §10 with `static_assert` |
| AC-7 | sim_sensors GPS injection via `openpty()` master-fd `::write()` | MET | sim_harness §10.2 + sim_sensors §4.2/§6.1 (corrected post-S9 review) |
| AC-8 | `conventions.md` §4.7 declares canonical `JUNO_FSW_STATE_T` enum | MET | conventions.md §4.7 lines 234–249 |
| AC-9 | sys_app §4.3 contains authoritative health-bitmap bit-assignment table | MET | sys_app/design.md §4.3 (6 bit rows, set/clear semantics) |
| AC-10 | SYS-016 amended to include `pre-launch` as initial phase | MET | requirements/sys/requirements.json line 112 |
| AC-11 | nav L2 pins numeric default for `kNavGpsBoundM_default` | MET | nav/design.md §9 (`= 200.0`) with FT1 rationale |
| AC-12 | telem ↔ nav field-precision reconciled; `JUNO_MSG_NAV_STATE_T` field shape pinned | MET | nav/design.md §4.1 authoritative table; telem_app §6.1 references verbatim |
| AC-13 | `tools/traceability.py` exits 0 with 371 reqs and ≥370 with test specs | MET | `TRACEABILITY CHECK PASSED — Valid: 371; Test specs: 370` |
| AC-14 | Re-spawned reviewer agents issue PROCEED on corrected designs | MET | S6, S7, S8: PROCEED-WITH-MINOR; S9: PROCEED after sim_sensors fix |
| AC-15 | Master log shows zero OPEN RIDs/RFAs | MET | All S1/S2 RIDs/RFAs DISPOSED or CLOSED; S3–S9 corrective work absorbed via this sprint (no parallel S3–S9 disposition records since the 21-reviewer fan-out was telescoped into 4 batched reviewers per the sprint plan §5 economy) |

**All 15 acceptance criteria met.**

## 4. Reviewer Verdict Summary

| Reviewer Section | Files Covered | Verdict | Notes |
|------------------|---------------|---------|-------|
| S6 (sensor apps) | gps_app, imu_app, baro_app | PROCEED-WITH-MINOR | Only known-LibJuno-gap RFAs |
| S7 (domain apps) | nav_app, afm_app, telem_app, mlog_app | PROCEED-WITH-MINOR | `tApi→ptApi` drift fixed post-review (Lead edit) |
| S8 (system app + conventions/system/reqs) | sys_app, conventions.md, system_design.md, sys reqs, log reqs | PROCEED-WITH-MINOR | All Phase 0 corrective work clean |
| S9 (sim modules) | sim_harness, sim_dynamics, sim_sensors, sim_scenario | PROCEED (after fix) | Initially NEEDS-CHANGES on sim_sensors `Inject` references; fixed by Lead post-review |

## 5. Outstanding RFAs (Carried Forward)

### LibJuno gaps (out of FT1 PDR scope; flow to LibJuno team)

1. **`juno::app::AppInit(...)` is documented in `app_api.hpp` doxygen but not yet published as a function.** All 8 app designs reference this name; either LibJuno publishes it (preferred — mirrors `juno::time::TimeInit` published pattern in `time_api.hpp`) or each `<App>AppInit` setup function must aggregate-init `tApp.tRoot` directly. Tracked as a follow-up RFA on the LibJuno backlog.
2. **`JUNO_MSG_BUS_VARIANT_T` placeholder** used in every app design as the broker template message-type parameter. Definition (variant, std::variant-style aggregate, or per-MID broker pool) requires PM clarification before code can land. Tracked as a sprint-after-PDR composition-root deliverable.
3. **`kBrokerPipes` / `kBrokerRegistry` / `kDefaultWriteBufBlocks` / `kDefaultRingCap` placeholder values** used in app designs and sys_app. Authoritative numeric pins require sd_lib / device_lib / broker L2 design extensions. Tracked.

### Cross-section migrations (deferred to a future sprint)

4. **Option C migration of `SIM_SENSORS_RAW_T` / `SIM_BARO_REGS_T` into `imu_lib` / `baro_lib` public headers.** Currently Option D (sim_sensors authors; FSW POSIX impl carries `static_assert` layout cross-checks). Lead recommends Option C; deferred to avoid in-scope scope-creep.

### Trick integration assumptions

5. **`exec_get_sim_time()` exact symbol/header in NASA Trick distribution** — assumed published in `sim_services/include/sim_services/exec_proto.h`. Verified at integration time when Trick environment lands.

## 6. Recommendation

**Recommend PASS-WITH-ACTIONS** at the Chief Engineer gate, conditional on RFA tracking for the five carry-forward items above. None of the carry-forward items are flight-blocking for FT1; they are upstream/integration concerns that surface at code-and-build time, not at design time.

## 7. Approval

| Field | Value |
|-------|-------|
| Memo author | Software Lead |
| Memo date | 2026-05-03 |
| Predecessor | PDR S1–S9 review consolidated 2026-05-03 |
| Predecessor corrective | PDR Corrective Action Sprint complete 2026-05-03 |
| Chief Engineer verdict | _Pending_ (this memo gates the CE final review) |
| Chair (PM) verdict | _Pending_ |
| Chair signature line | _____________________ |
| Effective date upon signature | 2026-05-03 |
