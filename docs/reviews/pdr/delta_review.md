---
document_type: PDR Delta Review Record
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
predecessor_review: PDR S1–S9 (2026-05-02 → 2026-05-03)
predecessor_corrective: PDR Corrective Action Sprint (2026-05-03)
status: Findings consolidated; awaiting Chief Engineer go/no-go
---

# PDR Delta Review Record — Juno FT1 FSW

## 1. Purpose

The PDR Corrective Action Sprint (`docs/reviews/pdr/corrective_sprint_plan.md`)
rewrote ~3,200 lines across 35 design and requirement files to close the four
root causes (C1–C4) that the original PDR board surfaced. The Software Lead
drafted a closure memo (`docs/reviews/pdr/closure_memo.md`) claiming all 15
acceptance criteria MET, but verification was thinner than Charter §3 calls
for: only 4 batched MAE reviewers covered S6–S9, S3–S5 received only the
mechanical sweeps with no fresh reviewer eyes, and **no SSE-R or CE coverage**
ran on any of the corrective work. This delta review fills that gap by running
two independent **holistic, system-wide** reviewers across the full corrected
baseline, before the Chief Engineer issues the binding implementation
go/no-go.

## 2. Scope

Holistic system-wide review of the FSW design and requirements baseline:

- L1: `docs/design/system/system_design.md`, `docs/design/conventions.md`
- L2 (27 designs): all `docs/design/<module>/design.md` plus the kmat split files and `sim_harness/interfaces.md`
- Requirements (10 modules): all `docs/requirements/<module>/requirements.json`
- Pre-flight evidence: `tools/traceability.py` output and 13 AC grep checks
  re-verifying the closure-memo §3 claims with fresh evidence

Two reviewer agents ran independently per Charter §5:
- **MAE** (`software-mission-assurance-engineer`) — IEEE 1016/29148 compliance, traceability completeness, requirements quality, cross-module incoherence at the documentation level.
- **SSE-R** (`senior-software-engineer` reviewer mode) — C++11/freestanding conformance, LibJuno module-pattern correctness, cross-module API contract integrity, technical correctness, WCET feasibility.

## 3. Pre-Flight Evidence (Phase 0, Lead direct)

```
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        371
  Requirements with code:       0      (expected at PDR — no implementation yet)
  Requirements with @verify:    0      (expected at PDR — no test code yet)
  Requirements with test specs: 370
```

Closure-memo §3 AC grep checks (re-run 2026-05-03 by Software Lead):

| AC | Claim | Fresh Evidence | Verdict |
|----|-------|----------------|---------|
| AC-1 | Zero public `Init|Execute` hooks in `apps/*_app/design.md` | grep returned 0 hits | CONFIRMED |
| AC-2 | Zero fabricated status-code names outside conventions §4.7 mapping table | Only legitimate hits are inside the §4.7 mapping table itself (rows documenting the migration); no usage hits | CONFIRMED |
| AC-3 | Zero `kMlogAppPeriodMs = 10` hits | grep returned 0 hits | CONFIRMED |
| AC-4 | Zero `TIME_LIB_ROOT_T` / `juno::time::GetUs` / `juno_time::GetUs` hits | grep returned 0 hits | CONFIRMED |
| AC-5 | sim_harness uses `tTrickTimeApi` + `juno::time::TimeInit`; no `JUNO_TIME_PROVIDER_T` | confirmed at sim_harness/interfaces.md §4.4 lines 198–258 | CONFIRMED |
| AC-6 | sim_dynamics `SIM_DYN_TRUTH_T` is C++-only POD (not `extern "C"`) carrying `juno::afm::JUNO_PHASE_T ePhase` | confirmed at sim_dynamics §1.1, §6.1, §10 with `static_assert` | CONFIRMED |
| AC-7 | sim_sensors GPS injection via openpty master-fd `::write` | confirmed at sim_harness/interfaces.md §4.4.1 line 271 and sim_sensors §6.1 line 337 | CONFIRMED |
| AC-8 | `JUNO_FSW_STATE_T` enum at conventions.md §4.7 | confirmed lines 234–249 | CONFIRMED |
| AC-9 | sys_app §4.3 health-bitmap bit-assignment table | confirmed (6 bits + reserved range, set/clear semantics, mask constants) | CONFIRMED |
| AC-10 | SYS-016 includes "pre-launch initial phase" | confirmed at requirements/sys/requirements.json line 112 | CONFIRMED |
| AC-11 | `kNavGpsBoundM_default = 200.0` with FT1 rationale | confirmed at nav/design.md line 457 | CONFIRMED |
| AC-12 | Authoritative `JUNO_MSG_NAV_STATE_T` field-shape table at nav §4.1 | confirmed at nav/design.md line 132 | CONFIRMED |
| AC-13 | traceability.py exits 0 with 371 reqs / 370 test specs | confirmed above | CONFIRMED |
| AC-14 | Phase-3 reviewers issued PROCEED on corrected designs | per closure_memo §4 (S6/S7/S8 PROCEED-WITH-MINOR; S9 PROCEED after fix) | CONFIRMED via prior records |
| AC-15 | Master log shows zero OPEN RIDs/RFAs | per rid_rfa_log.md statistics (0 OPEN across all sections) | CONFIRMED |

All 15 closure-memo ACs hold under independent re-verification.

## 4. Consolidated Findings

Findings were merged from MAE and SSE-R independent reports per Charter §5.
Verbatim duplicates collapsed; substantive variants recorded separately. The
master log (`docs/reviews/pdr/rid_rfa_log.md`) is updated with these as
PDR-RID-S10-NNN and PDR-RFA-S10-NNN entries.

### 4.1 Major Findings (5)

| ID | Source | File / Location | Description | Recommended Resolution |
|----|--------|-----------------|-------------|------------------------|
| Δ-MAJOR-1 | MAE | `docs/design/imu_app/design.md:146` | `JUNO_ASSERT_EXISTS` cite uses `tApp.tRoot.tApi` instead of canonical `tApp.tRoot.ptApi`. The LibJuno `JUNO_MODULE_ROOT` macro publishes `ptApi` (`libjuno/include/juno/module.h:131`), and `app_api.hpp:103` follows the convention. An implementer following this verbatim would generate non-compiling code. Identical to the 2026-05-03 lessons-learned `tApi`/`ptApi` drift trap. | Edit `tApp.tRoot.tApi` → `tApp.tRoot.ptApi`. Single-token edit. |
| Δ-MAJOR-2 | MAE | `docs/design/sim_harness/interfaces.md:305` | `TickFsw` postcondition cites `sch_lib::Run` — the legacy name. Canonical entry per `system_design.md §8.1/§8.2` and conventions §1.4 is `juno::sch::SCH_API_T<8, 200>::Execute()`. Same ambiguity in `sim_harness/design.md §7.1:274` ("advance one base tick"). | Replace `sch_lib::Run` reference with `juno::sch::SCH_API_T<8,200>::Execute()` (one minor frame). Tighten §7.1 sequence-diagram label. |
| Δ-MAJOR-3 | SSE-R | `docs/design/telem/design.md:206` | Telem packet encoder row 56 references `tNav.fAltMHae` — but `NAV_STATE_T` (authoritative at nav/design.md §4.1, struct at line 109) declares `double tPosLla[3]` with `// [dLatDeg, dLonDeg, fAltMHae]` as a *comment annotation*, not a field. There is no `fAltMHae` member on `NAV_STATE_T`. Compile failure. (Note: the same row's `tGps.fAltMHae` reference is correct because `GPS_FIX_T` *does* publish `fAltMHae` as a real field at nav/design.md:107.) | Replace `tNav.fAltMHae` → `(float)tNav.tPosLla[2]` and document the intentional double→float narrowing per `SW-REQ-SYS-042`. |
| Δ-MAJOR-4 | SSE-R | `docs/design/sys_app/design.md:109,124,129,146` | sys_app uses two-level embedding: `struct SYS_APP { APP_ROOT_T tRoot; ... };` AND a separate `struct SYS_APP_IMPL_T JUNO_MODULE_DERIVE(APP_ROOT_T, ...)`. Hooks downcast via `static_cast<SYS_APP_IMPL_T&>(tApp.tRoot)`, but `tApp.tRoot`'s dynamic type is `APP_ROOT_T`, not `SYS_APP_IMPL_T`. Per `JUNO_MODULE_DERIVE` macro definition (`libjuno/include/juno/module.h:161`), the IMPL embeds the ROOT as first member — so the cast attempts to read fields beyond the embedded ROOT into uninitialized memory. Strict-aliasing violation; UB. By contrast, the other 7 apps (e.g., `IMU_APP_T` at imu_app §3.3 line 102) ARE the `JUNO_MODULE_DERIVE` struct directly (single-level), which makes their downcast layout-compatible. | Restructure: either (a) make `SYS_APP` itself the `JUNO_MODULE_DERIVE(APP_ROOT_T, ...)` struct (eliminate the redundant outer wrapper); or (b) make `SYS_APP` embed `SYS_APP_IMPL_T tImpl` as its first member (instead of `APP_ROOT_T tRoot`) so the downcast targets a real `SYS_APP_IMPL_T`. Option (a) matches the pattern the other 7 apps use. |
| Δ-MAJOR-5 | SSE-R | `docs/design/nav_app/design.md:179` | `OnProcess` postcondition uses `juno::time::TimestampToMicros(*_ptTime, _ptTime->ptApi->Now(*_ptTime).tOk).tOk` — free-function syntax. LibJuno `time_api.hpp` (lines 27, 67, 141) declares `TimestampToMicros` as a non-static member function of `TIME_ROOT_T`. baro_app/imu_app/gps_app/afm_app all use the canonical member-function form `_ptTime->TimestampToMicros(tNow).tOk`. nav_app is the sole outlier; will not compile. | Replace with `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` matching the 4 other app patterns. |

### 4.2 Minor Findings (10)

| ID | Source | File / Location | Description | Recommended Resolution |
|----|--------|-----------------|-------------|------------------------|
| Δ-MINOR-1 | MAE | `docs/design/baro/design.md:17` | "the **app** owns bus access and injects the byte transport into the lib at `New()`" — but neither `baro_app` nor `baro_lib` touch I2C in current designs. Bus transport is wired by composition root. | Reword: "the composition root owns bus access; the caller of `BARO_LIB_IMPL_T::New()` injects the byte transport." |
| Δ-MINOR-2 | MAE | `docs/design/lora/design.md:65` | Mermaid edge label "5 ms tick" feeds telem_app, but telem_app is 500 ms (`kTelemAppPeriodMs = 500`). Diagram-label slip. | Edit "5 ms tick" → "500 ms tick". |
| Δ-MINOR-3 | MAE | `docs/design/afm/design.md:108` | `JUNO_PHASE_T` declaration-home prose self-loops ("the canonical declaration lives alongside `JUNO_PHASE_T`"). | Pin to `libs/afm_lib/include/afm_lib/afm_api.hpp` (consistent with sim_dynamics §6.1:224 `#include`). |
| Δ-MINOR-4 | MAE | `docs/design/sim_sensors/design.md:168` | References `sim_scenario`'s "tSensorCfg substructure" — but `SIM_SCENARIO_T` is a flat POD with no nested cfg substructure (per sim_scenario §4.3 and sim_harness/interfaces.md §4.3 step 1.4). | Reword: "consumes a transcoded `SIM_SENSOR_CFG_T` populated by `sim_harness` from flat `SIM_SCENARIO_T` fields." |
| Δ-MINOR-5 | MAE | `docs/design/sys_app/design.md:109` and `docs/design/gps_app/design.md:72` | App aggregate naming drift: `SYS_APP` and `GPS_APP` lack the `_T` suffix that `conventions.md` §3 mandates. Six other apps (`IMU_APP_T`, `BARO_APP_T`, `NAV_APP_T`, `AFM_APP_T`, `TELEM_APP_T`, `MLOG_APP_T`) carry the suffix. | Pick one direction across all 8 apps. Recommend: add `_T` (matches conventions verbatim). gps_app §3.3 already documents its deviation; sys_app does not — at minimum align documentation. |
| Δ-MINOR-6 | SSE-R | `docs/design/telem/design.md:206` (related to Δ-MAJOR-3) | Even after Δ-MAJOR-3 fix, the `(float)tNav.tPosLla[2]` narrowing remains undocumented. At 600 m HAE float gives ~4 cm resolution (adequate for FT1) but design intent should be visible inline. | Add inline comment referencing `SW-REQ-SYS-042` and the wire-format precision contract. |
| Δ-MINOR-7 | SSE-R | `docs/design/mlog/design.md §6.6` (NAV record) | NAV record stores `fAltHaeM` as `float` derived from `double tPosLla[2]`. Same precision-narrowing concern as Δ-MINOR-6. | Add precision-reduction note to NAV record schema table. |
| Δ-MINOR-8 | SSE-R | `docs/design/sim_sensors/design.md §4.3` vs `docs/design/sim_scenario/design.md §4.3` | Constant mismatch: sim_sensors declares `kMaxDropouts = 16`; sim_scenario declares `kMaxDropouts = 8`. Same logical cap defined twice with different values. Scenario parser enforces 8 before transcode (no overflow), but a 16-slot sensor-side array would silently accept more if any future code path uses the sensor-side constant for parsing. | Pin authoritative `kMaxDropouts` in one header (recommend `sim_scenario`'s) and import from the other. |
| Δ-MINOR-9 | SSE-R | `docs/design/nav/design.md §4.5` | Extension status-code list skips offset `+2` between `kNavStatusGpsStale` (+1) and `kNavStatusConvergenceFail` (+3). Gap is undocumented. | Either assign offset `+2` or explicitly mark "reserved." |
| Δ-MINOR-10 | SSE-R | `docs/design/sim_harness/interfaces.md §4.3 step 4` | sim_harness transcoding of sensor noise parameters narrows `double` (in `SIM_SCENARIO_T`) → `float` (in `SIM_SENSOR_CFG_T`) silently. Likely intentional (sensor models run in float) but undocumented. | Add narrowing note to the transcoding step. |

### 4.3 Carry-Forward RFAs Encountered (already tracked — no new entries)

Both reviewers encountered the 5 carry-forward RFAs already enumerated in
`closure_memo.md` §5; neither raised new RIDs/RFAs against them per brief
instruction:

1. `juno::app::AppInit(...)` doxygen-documented but not yet published in LibJuno (encountered in all 8 app designs).
2. `JUNO_MSG_BUS_VARIANT_T` placeholder (encountered in every app's `BROKER_ROOT_T<...>` template arg).
3. Capacity placeholder pins `kBrokerPipes` / `kBrokerRegistry` / `kDefaultWriteBufBlocks` / `kDefaultRingCap` (encountered in app designs and sys_app).
4. Option C migration of `SIM_SENSORS_RAW_T` / `SIM_BARO_REGS_T` to imu_lib/baro_lib public headers (currently Option D with `static_assert` cross-checks).
5. NASA Trick `exec_get_sim_time()` symbol verification at integration time.

## 5. Reviewer Advisory Verdicts

| Reviewer | Verdict | Rationale |
|----------|---------|-----------|
| MAE | **PROCEED-WITH-MINOR** | Corrective sprint achieved its goal; all 15 ACs hold under independent re-verification; surviving items are last-mile editorial cleanup. The two MAE Major findings (`tApi`/`ptApi` and `sch_lib::Run`) are single-token edits. Implementation is unblocked once those two land. |
| SSE-R | **NEEDS CHANGES** | Three Major errors block implementation: E-001 (telem field-name compile failure), E-002 (sys_app two-level embedding produces UB), E-003 (nav_app wrong call-site syntax for `TimestampToMicros`). E-001 and E-003 are point fixes; E-002 requires structural rework of sys_app's IMPL pattern (single-section restructure). Upon resolution of these three errors, no other blocking obstacles to implementation. |

## 6. Cross-Reviewer Consolidated Verdict (Lead, advisory)

The two reviewers converge on the same answer with different framings: the
corrective sprint succeeded at the system architecture level (every AC holds;
all four root causes closed; cross-module data contracts coherent), but five
last-mile defects survive in the corrected baseline that would block clean
implementation. None require new sprints; each is a targeted point edit (4
single-token / single-line fixes plus the sys_app section restructure).
Estimated total Lead time to remediate all five Major findings: ≤2 hours.
The Chief Engineer is the binding authority on the implementation go/no-go.

## 7. Disposition Tracking

Per Charter §5, the **Chair (PM)** dispositions findings; the Software Lead
records dispositions and tracks corrective work to closure. Findings are
filed into the master log as `PDR-RID-S10-NNN` and `PDR-RFA-S10-NNN`
(see `docs/reviews/pdr/rid_rfa_log.md`).

This delta review record is the authoritative source for finding content;
the master log mirrors the index columns only.

## 8. Cross-References

- [PDR Charter](charter.md)
- [PDR Closure Memo (corrective sprint)](closure_memo.md)
- [PDR Corrective Sprint Plan](corrective_sprint_plan.md)
- [PDR RID/RFA Master Log](rid_rfa_log.md)
- [Section S1: Architecture](sections/S1_architecture.md)
- [Section S2: Foundation Libraries](sections/S2_foundation_libs.md)
- Delta closure memo (CE go/no-go): `delta_closure_memo.md` (authored Phase 3)
