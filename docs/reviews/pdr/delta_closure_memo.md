---
document_type: PDR Delta Closure Memo
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
predecessor_review: PDR Sections S1–S9 (2026-05-02 → 2026-05-03)
predecessor_corrective: PDR Corrective Action Sprint (2026-05-03)
predecessor_delta: PDR Delta Review (2026-05-03)
status: All 5 pre-implementation actions remediated 2026-05-03; awaiting CE final-gate verdict on remediation sprint
---

# PDR Delta Closure Memo — Juno FT1 FSW Implementation Go/No-Go

## 1. Purpose

Records the Chief Engineer's binding determination on whether the corrected
Juno FT1 FSW design and requirements baseline is **GO** for implementation,
following the holistic delta review documented in
[delta_review.md](delta_review.md).

## 2. Sprint Summary

The original PDR review board (2026-05-02 → 2026-05-03) reviewed all 10 sections
and produced ~237 RIDs / 140 RFAs clustering into 4 root causes. A corrective
sprint (`corrective_sprint_plan.md`) rewrote ~3,200 lines across 35 design and
requirement files. The Software Lead drafted [closure_memo.md](closure_memo.md)
claiming all 15 ACs MET. This delta-PDR sprint added the SSE-R and CE coverage
that the corrective sprint had skipped: two independent holistic reviewers
(MAE + SSE-R) walked the entire baseline as one integrated system.

| Phase | Activity | Agents |
|-------|----------|--------|
| 0 | Lead pre-flight: traceability.py + 13 AC grep checks | 0 |
| 1 | Holistic MAE review (IEEE 1016/29148, traceability) | 1 |
| 2 | Holistic SSE-R review (C++11/freestanding, LibJuno conformance, technical correctness) | 1 |
| 3 | Lead consolidation into delta_review.md + master log + this memo | 0 |
| 4 | Chief Engineer go/no-go determination | 1 |

**Total agent invocations: 3** (MAE + SSE-R + CE).

## 3. Delta Acceptance Criteria Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| DAC-1 | Holistic MAE review of entire baseline | MET | MAE returned a structured per-section report covering S1–S9 with PROCEED-WITH-MINOR advisory verdict |
| DAC-2 | Holistic SSE-R review of entire design baseline | MET | SSE-R returned a structured report with NEEDS-CHANGES advisory verdict (3 Major errors blocking implementation) |
| DAC-3 | Closure-memo §3 grep claims (AC-1..AC-13) re-verified | MET | All 15 ACs CONFIRMED via fresh grep evidence (delta_review.md §3) |
| DAC-4 | tools/traceability.py exits 0 with 371 reqs / ≥370 test specs | MET | TRACEABILITY CHECK PASSED — Valid: 371; Test specs: 370 |
| DAC-5 | New RIDs/RFAs logged in master log with proposed dispositions | MET | 15 new PDR-RID-S10-NNN entries (5 Major + 10 Minor) added to rid_rfa_log.md; all OPEN pending Chair disposition |
| DAC-6 | Single consolidated delta-PDR review record | MET | docs/reviews/pdr/delta_review.md (this delta) |
| DAC-7 | CE issues paragraph-long go/no-go determination | MET — see §6 below; verdict: GO-WITH-PRE-IMPLEMENTATION-ACTIONS with 5 named actions |

## 4. Reviewer Advisory Verdict Summary

| Reviewer | Verdict | Findings | Blocking Items |
|----------|---------|----------|----------------|
| MAE | PROCEED-WITH-MINOR | 2 Major + 5 Minor + 0 RFA-new | `tApi`/`ptApi` drift (imu_app); stale `sch_lib::Run` (sim_harness) |
| SSE-R | NEEDS CHANGES | 3 Major + 5 Minor + 0 RFA-new | telem field-name compile failure; sys_app UB cast; nav_app TimestampToMicros call-site |

Net 5 Major findings (no overlap between MAE and SSE-R) and 10 Minor findings.
Full descriptions and recommended resolutions in `delta_review.md` §4.

## 5. Carry-Forward RFAs (unchanged from corrective-sprint closure memo §5)

Both delta reviewers encountered and noted (per brief instruction did not re-raise):

1. `juno::app::AppInit(...)` LibJuno publication gap — backlog item.
2. `JUNO_MSG_BUS_VARIANT_T` placeholder — composition-root sprint deliverable.
3. Capacity placeholder pins (`kBrokerPipes`, `kBrokerRegistry`, `kDefaultWriteBufBlocks`, `kDefaultRingCap`) — L2 extensions.
4. Option C migration of `SIM_SENSORS_RAW_T` / `SIM_BARO_REGS_T` to imu_lib/baro_lib public headers — deferred future sprint.
5. NASA Trick `exec_get_sim_time()` symbol verification — Trick integration time.

## 6. Chief Engineer Determination

**Rationale:** The corrective action sprint succeeded at the architectural level — all four root causes (C1 Option A app-lifecycle pivot, C2 status-code catalog sweep, C3 mlog @ 5 ms cascade, C4 LibJuno canonical type-name pivot) are closed, all 15 closure-memo acceptance criteria hold under independent re-verification with fresh grep evidence, the traceability tool is clean at 371 requirements with 370 test specs, and the five carry-forward RFAs are upstream/integration concerns already accepted as PASS-WITH-ACTIONS by the Chair rather than design-level defects. However, the holistic delta review surfaced five last-mile Major findings that the batched corrective-sprint reviewers missed, every one of which would block clean implementation: imu_app §4.1's `tApp.tRoot.tApi` mis-cite (canonical is `ptApi` per `libjuno/include/juno/module.h:131`), sim_harness/interfaces.md §TickFsw's stale `sch_lib::Run` reference, telem §6 packet-encoder row 56's reference to a non-existent `tNav.fAltMHae` member (NAV_STATE_T at nav/design.md:109 publishes `double tPosLla[3]` with `fAltMHae` only as a comment annotation), sys_app §3.3/§4.1's two-level `SYS_APP` → `APP_ROOT_T` + separate `SYS_APP_IMPL_T JUNO_MODULE_DERIVE(APP_ROOT_T,...)` embedding pattern that downcasts via `static_cast<SYS_APP_IMPL_T&>(tApp.tRoot)` and produces strict-aliasing UB given the `JUNO_MODULE_DERIVE` layout at `libjuno/include/juno/module.h:161`, and nav_app §4.4's free-function call `juno::time::TimestampToMicros(*_ptTime, ...)` against the canonical member-function declaration at `libjuno/include/juno/time/time_api.hpp:142`. Four of these are single-token or single-line point edits and the fifth (sys_app) is a single-section structural alignment to the pattern the other seven apps already use (Option (a) per delta_review.md §4.1: collapse `SYS_APP` and `SYS_APP_IMPL_T` into one `JUNO_MODULE_DERIVE(APP_ROOT_T, ...)` struct), with a Lead-estimated total remediation budget of ≤2 hours that does not warrant another corrective sprint. The 10 Minor findings are editorial/precision-narrowing notes that may be folded into the same touch-up pass or deferred to early CDR cleanup at the Chair's discretion. Charter §1 explicitly scopes the PDR to design-level artifacts (`.cpp`/`.hpp` source is reserved for CDR), so the design baseline is implementable as drawn contingent only on the five named edits landing before implementation kickoff; on that condition the Juno FT1 FSW design baseline is approved to proceed to coding. This determination supersedes the recommendation line in `closure_memo.md` §6.

**Verdict: GO-WITH-PRE-IMPLEMENTATION-ACTIONS**

**Pre-Implementation Actions (target deadline: `pre-implementation-kickoff`):**

1. **PDR-RID-S10-001 (Δ-MAJOR-1)** — `docs/design/imu_app/design.md` §4.1 line 146: change `tApp.tRoot.tApi` to `tApp.tRoot.ptApi` to match the `JUNO_MODULE_ROOT` macro publication (`libjuno/include/juno/module.h:131`) and the convention followed by every other app design.
2. **PDR-RID-S10-002 (Δ-MAJOR-2)** — `docs/design/sim_harness/interfaces.md` §TickFsw line 305 (and the §7.1:274 sequence-diagram label in `docs/design/sim_harness/design.md`): replace the stale `sch_lib::Run` reference with `juno::sch::SCH_API_T<8,200>::Execute()` per `system_design.md` §8.1/§8.2 and `conventions.md` §1.4.
3. **PDR-RID-S10-003 (Δ-MAJOR-3)** — `docs/design/telem/design.md` §6 packet-encoder table row 56: replace `tNav.fAltMHae` with `(float)tNav.tPosLla[2]` and add an inline comment citing `SW-REQ-SYS-042` documenting the intentional double→float narrowing for the wire-format precision contract (also subsumes Δ-MINOR-6).
4. **PDR-RID-S10-004 (Δ-MAJOR-4)** — `docs/design/sys_app/design.md` §3.3 / §4.1 lines 109, 124, 129, 146: restructure `SYS_APP` to be the single `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` struct directly (Option (a) per `delta_review.md` §4.1), eliminating the redundant outer wrapper and the UB-prone `static_cast<SYS_APP_IMPL_T&>(tApp.tRoot)`; align with the single-level pattern used by the other seven apps (e.g., `IMU_APP_T` at `imu_app/design.md` §3.3 line 102).
5. **PDR-RID-S10-005 (Δ-MAJOR-5)** — `docs/design/nav_app/design.md` §4.4 line 179: replace the free-function call `juno::time::TimestampToMicros(*_ptTime, _ptTime->ptApi->Now(*_ptTime).tOk).tOk` with the canonical member-function form `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` per `libjuno/include/juno/time/time_api.hpp:142` and the pattern already used by baro_app, imu_app, gps_app, and afm_app.

### 6.1 Pre-Implementation Action Status (Delta-PDR Remediation Sprint, 2026-05-03)

All 5 pre-implementation actions and 10 Minor findings remediated by the Delta-PDR Remediation Sprint executed 2026-05-03 (3-agent sprint: 1 worker + 1 MAE + 1 CE final gate).

| Action / Finding | Status | Evidence |
|------------------|--------|----------|
| PDR-RID-S10-001 (Δ-MAJOR-1, imu_app `tApi`→`ptApi`) | CLOSED | `grep -nE "tRoot\.tApi" docs/design/imu_app/design.md` → 0 hits |
| PDR-RID-S10-002 (Δ-MAJOR-2, sim_harness `sch_lib::Run`→`SCH_API_T<8,200>::Execute`) | CLOSED | sim_harness/interfaces.md §TickFsw + sim_harness/design.md §7.1 sequence-diagram label updated; only residual `sch_lib::Run` is a deliberate negation reference ("supersedes the legacy ... name") |
| PDR-RID-S10-003 (Δ-MAJOR-3, telem `tNav.fAltMHae`→`(float)tNav.tPosLla[2]`) | CLOSED | `grep -nE "tNav\.fAltMHae" docs/design/telem/design.md` → 0 hits; new row 56 cites `static_cast<float>(tNav.tPosLla[2])` with `SW-REQ-SYS-042` narrowing comment |
| PDR-RID-S10-004 (Δ-MAJOR-4, sys_app structural restructure) | CLOSED | `SYS_APP_IMPL_T` eliminated; `SYS_APP_T` declared via `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)`; UB cast `static_cast<SYS_APP_IMPL_T&>(tApp.tRoot)` removed; canonical `static_cast<SYS_APP_T&>(tRoot)` (layout-compatible) replaces it. MAE APPROVED on first iteration (11/11 checks PASS) |
| PDR-RID-S10-005 (Δ-MAJOR-5, nav_app `TimestampToMicros` member-form) | CLOSED | nav_app/design.md §4.4 line 179 uses `_ptTime->TimestampToMicros(...)`; matches baro/imu/gps/afm pattern |
| PDR-RID-S10-006 (Δ-MINOR-1, baro "app owns bus") | CLOSED | baro/design.md §1 reworded to "composition root owns bus access; the caller of `BARO_LIB_IMPL_T::New()` injects the byte transport" |
| PDR-RID-S10-007 (Δ-MINOR-2, lora "5 ms tick"→"500 ms tick") | CLOSED | lora/design.md §3.2 mermaid label updated |
| PDR-RID-S10-008 (Δ-MINOR-3, afm `JUNO_PHASE_T` home pin) | CLOSED | afm/design.md §4.1 explicitly pins canonical declaration to `libs/afm_lib/include/afm_lib/afm_api.hpp` |
| PDR-RID-S10-009 (Δ-MINOR-4, sim_sensors stale "tSensorCfg substructure") | CLOSED | sim_sensors/design.md §4.3 reworded to "transcoded `SIM_SENSOR_CFG_T` populated by `sim_harness` from flat `SIM_SCENARIO_T` fields" |
| PDR-RID-S10-010 (Δ-MINOR-5, `SYS_APP`→`SYS_APP_T` and `GPS_APP`→`GPS_APP_T`) | CLOSED | sys_app rename folded into Δ-MAJOR-4 restructure; gps_app rename via word-boundary sed (19 sites; `GPS_APP_API_T` legacy reference preserved) |
| PDR-RID-S10-011 (Δ-MINOR-6, telem narrowing comment) | CLOSED | telem/design.md row 56 now cites `SW-REQ-SYS-042` wire-format precision contract |
| PDR-RID-S10-012 (Δ-MINOR-7, mlog NAV record narrowing note) | CLOSED | mlog/design.md §6.6 NAV record `fAltMHae` row notes "narrowed from `tNav.tPosLla[2]` (`double`) — intentional" |
| PDR-RID-S10-013 (Δ-MINOR-8, `kMaxDropouts` pin) | CLOSED | sim_sensors/design.md `kMaxDropouts = 8` (was 16); `static_assert` cross-check against `sim_scenario::kMaxDropouts` documented |
| PDR-RID-S10-014 (Δ-MINOR-9, nav §4.5 reserved offsets) | CLOSED | nav/design.md §4.5 documents `+0`, `+1`, `+2` as reserved with rationale |
| PDR-RID-S10-015 (Δ-MINOR-10, sim_harness transcoding narrowing) | CLOSED | sim_harness/interfaces.md §4.3 step 4 documents `double`→`float` sigma transcoding intent |
| traceability.py | PASS | 371 reqs, 370 test specs, exit 0 (re-run 2026-05-03 post-remediation) |

Delta-PDR Remediation Sprint agent invocations: 1 software-systems-engineer worker (sys_app restructure) + 1 MAE (review APPROVED) + 1 project-chief-engineer (final gate, this section). Lead-direct atomic edits handled the other 4 Majors and 9 Minors per the 2026-05-03 atomic-Lead-edit precedent.

### 6.2 Final-Gate Verdict on Remediation

**Rationale:** The Delta-PDR Remediation Sprint landed cleanly against every condition imposed by the prior `GO-WITH-PRE-IMPLEMENTATION-ACTIONS` verdict in §6, and independent grep re-verification by this Chief Engineer confirms each of the five Major closures: PDR-RID-S10-001 shows zero hits for `tRoot\.tApi` in `docs/design/imu_app/design.md` with the canonical `tRoot.ptApi` form now present at line 146; PDR-RID-S10-002 shows the only residual `sch_lib::Run` token in `sim_harness/interfaces.md:305` is a deliberate "supersedes the legacy ... name" negation reference paired with the canonical `juno::sch::SCH_API_T<8, 200>::Execute(tSch)` call, and `sim_harness/design.md:274` carries the matching sequence-diagram label; PDR-RID-S10-003 shows zero hits for `tNav\.fAltMHae` in `docs/design/telem/design.md` with row 56 now encoding `static_cast<float>(tNav.tPosLla[2])` annotated with the `SW-REQ-SYS-042` wire-format precision contract (subsuming Δ-MINOR-6); PDR-RID-S10-004 shows zero hits for the `SYS_APP_IMPL_T` symbol and zero hits for the UB-prone `static_cast<SYS_APP_IMPL_T&>` cast — `SYS_APP_T` is now declared via `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` at `sys_app/design.md:109` with the layout-compatible `static_cast<SYS_APP_T&>(tRoot)` downcast appearing in §3.3 / §4.1 / §4.2 narrative, and the structure exactly matches the `JUNO_MODULE_DERIVE` macro at `libjuno/include/juno/module.h:161` (which embeds `ROOT_T tRoot;` as the first member via the `JUNO_MODULE_SUPER` alias at module.h:97), so the strict-aliasing UB is genuinely eliminated rather than merely renamed; PDR-RID-S10-005 shows `nav_app/design.md:179` uses the canonical member-function form `_ptTime->TimestampToMicros(...)` aligned with the declaration in `libjuno/include/juno/time/time_api.hpp` and the pattern already used by baro_app, imu_app, gps_app, and afm_app. All 10 Minor findings (PDR-RID-S10-006 through PDR-RID-S10-015) are also CLOSED with evidence in §6.1 — spot-checks of lora 500 ms tick relabel (`lora/design.md:65`) and afm phase-enum home pin (`afm/design.md:108`) both confirm the asserted edits. The MAE second-look on the sys_app structural restructure returned APPROVED on first iteration with all 11 checks PASS, traceability.py re-runs at 371 requirements / 370 test specs / exit 0 (unchanged from pre-remediation baseline as expected since no requirements were touched), and the master log shows all 15 PDR-RID-S10-NNN entries transitioned OPEN→CLOSED with the "ACCEPT — 2026-05-03 (delta-PDR remediation sprint)" disposition note. The five carry-forward RFAs enumerated in §5 are upstream LibJuno publication / composition-root / NASA Trick integration concerns explicitly out of design-PDR scope per Charter §1, already accepted as PASS-WITH-ACTIONS by the Chair, and tracked separately for early CDR / integration-time closure — they do not count against this remediation sprint and do not block implementation kickoff. The Juno FT1 FSW design baseline now satisfies every condition of the prior conditional verdict and is unconditionally cleared for implementation.

**Final Verdict: GO**

## 7. Approval

| Field | Value |
|-------|-------|
| Memo author | Software Lead |
| Memo date | 2026-05-03 |
| Predecessor | PDR Corrective Action Sprint complete 2026-05-03 |
| Predecessor delta review | docs/reviews/pdr/delta_review.md |
| Chief Engineer determination | **GO** (issued by `project-chief-engineer` 2026-05-03 post-remediation sprint; rationale in §6.2) — supersedes the prior conditional verdict in §6 |
| Chair (PM) verdict | _Pending Chair countersign_ |
| Chair signature line | _____________________ |
