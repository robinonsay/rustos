---
sprint_id: SPRINT-WAVE5-EXIT
module: sensor_apps (Wave 5 Exit Gate — imu_app + baro_app + gps_app)
wave: 5 (Sensor Apps — formal wave exit gate per `docs/sdp/sensor_apps.md` §4)
predecessors: SPRINT-IMPL-16 (imu_app, CLOSED 2026-05-10), SPRINT-IMPL-17 (baro_app, CLOSED 2026-05-11), SPRINT-IMPL-18 (gps_app, CLOSED 2026-05-11)
status: CLOSED
opened: 2026-05-12
closed: 2026-05-12
ce_verdict: APPROVED (unconditional, 0 findings)
pm_signoff: pending
---

# SPRINT-WAVE5-EXIT Closure Record — Wave 5 Exit Gate + GPS-18 Carry-Forward Discharge

## 1. Sprint Goal

Hold the **Wave 5 Exit Gate** per [`docs/sdp/sensor_apps.md`](../sdp/sensor_apps.md) §4 against the three CLOSED Wave 5 sprints (SPRINT-IMPL-16 `imu_app`, -17 `baro_app`, -18 `gps_app`) and concurrently discharge the SPRINT-IMPL-18 carry-forward backlog, with dual scope: (a) **Wave 5 Exit Gate** — re-verify the 8 SDP-mandated wave-consistency conditions (G1+G2 across all three sprints, single-level `JUNO_MODULE_DERIVE` uniformity, `tRoot.ptApi->Hook(...)` dispatch uniformity, broker template-instantiation parity, canonical member-form TimestampToMicros uniformity, identical RFA #1 manual-aggregate-init path, +30 `SW-REQ` burndown delta, BARO-17 Demonstration-deferral bookkeeping); (b) **GPS-18 carry-forward discharge** — five items: INS-GPS-APP-003 (no-nmea-coupling inspection record), INS-GPS-APP-010 (HAE-altitude semantics inspection record), TC-006 ordering-strengthening finding closure, `JUNO_TIME_US_T → JUNO_TIME_MICROS_T` documentation sed rename, and explicit documentation of the Wave 4+/Wave 6+ PM block status. Wave 6 (domain apps) remains PM-blocked on USER-NAV-LIB delivery per SDP-R-08, independent of this gate verdict — this sprint formally closes Wave 5 only.

## 2. PM-Approved Scope Decisions (Q-batch, 2026-05-12)

| Q | Decision | Rationale |
|---|----------|-----------|
| Q1.a | Add INS-GPS-APP-003 (`docs/inspections/gps_app_no_nmea_coupling.md`) to sprint plan as Phase 1 SSE deliverable | GPS-18 §10 carry-forward #4: "Future inspection records `INS-GPS-APP-003`...are queued for a future RTM-cleanup sprint." Wave 5 Exit Gate is the natural earliest discharge slot — no RTM-cleanup sprint is otherwise queued; bundling avoids stranding the carry-forward indefinitely. PM-approved, no further redirect. |
| Q1.b | Add INS-GPS-APP-010 (`docs/inspections/gps_app_hae_semantics.md`) to sprint plan as Phase 1 SSE deliverable | GPS-18 §10 carry-forward #4 (paired with -003 above). Same justification; same PM approval. |
| Q1.c | Add TC-006 Publish-vs-GetUtc ordering strengthening as Phase 1 Lead-direct edit on `apps/gps_app/tests/gps_app_test.cpp:250` | GPS-18 §10 carry-forward #5: "Optional 1-line strengthening for any future sprint touching this test." Atomic 1-line change; no review iter needed. PM-approved. (Outcome: change found incorrect on spot-verify, reverted, closed-with-finding — see §10 #3.) |
| Q1.d | Add `JUNO_TIME_US_T → JUNO_TIME_MICROS_T` documentation sed rename across `docs/design/baro_app/design.md` + `docs/design/gps_app/design.md` as Phase 1 Lead-direct edit | GPS-18 §10 carry-forward #6 + BARO-17 §10 carry-forward #5: recurring L2 documentation drift; canonical published name is `JUNO_TIME_MICROS_T` per `libjuno/include/juno/time/time_api.hpp`. PM-approved. |
| Q1.e | Document the Wave 4+/Wave 6+ PM block status (SDP-R-08) explicitly in §10 of this record, including the verbatim SDP-R-08 quote | Wave 5 Exit Gate PASS is commonly read as "Wave 6 unblocked" — but Wave 6 is blocked on PM-owned USER-NAV-LIB delivery, not on Wave 5 closure. Making this explicit prevents downstream-sprint open by misinterpretation. PM-approved. |

PM Q-batch was a single combined approval ("Add these items to the sprint plan as well: [4 carry-forward items]") covering Q1.a/b/c/d (the four numbered items) plus Q1.e (the §10 documentation requirement). No further redirects mid-sprint.

## 3. Acceptance Criteria — Final Status

| #  | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| AC-1  | All three Wave 5 sprints CLOSED with G1 + G2 exit 0 | MET | SPRINT-IMPL-16 CLOSED 2026-05-10 (§7 G1/G2 both exit 0); SPRINT-IMPL-17 CLOSED 2026-05-11 (§7 G1/G2 both exit 0); SPRINT-IMPL-18 CLOSED 2026-05-11 (§7 G1/G2 both exit 0). Phase 3 re-baseline matches Phase 0 baseline; §7 below. |
| AC-2  | Single-level `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` uniformly across `apps/{imu,baro,gps}_app/` | MET | grep `JUNO_MODULE_DERIVE` returns exactly 3 hits (imu_app.hpp, baro_app.hpp, gps_app.hpp), all single-level; no two-level UB pattern. CE Phase 4 confirmed. |
| AC-3  | `tRoot.ptApi->Hook(...)` vtable dispatch uniformly (zero `tApi->` or `tRoot.tApi->`) | MET | grep `\btApi->` returns 0 hits across all three apps; `_pt*->ptApi->` dispatch present in every hook (imu_app.cpp, baro_app.cpp, gps_app.cpp). CE Phase 4 confirmed. |
| AC-4  | Broker template instantiation parity: `juno::sb::BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T, kBrokerPipes, kBrokerRegistry>` resolves identically from `apps/include/juno_fsw_capacities.hpp` (Wave 0) | MET | All three apps consume the same Wave 0 capacity pins; no per-app override. CE Phase 4 confirmed. |
| AC-5  | Per-tick timestamp via canonical member-form `_ptTime->TimestampToMicros(_ptTime->ptApi->Now(*_ptTime).tOk).tOk` | MET | All three apps' OnProcess paths use member-form per `libjuno/include/juno/time/time_api.hpp:142`; no fabricated free-function `juno::time::TimestampToMicros(...)`. CE Phase 4 confirmed. |
| AC-6  | RFA #1 manual-aggregate-init path documented identically across all three sprint closure records | MET | IMU-16 Q2, BARO-17 Q2, GPS-18 Q2 all chose manual-aggregate-init (LibJuno `juno::app::AppInit` still unpublished). No path divergence. CE Phase 4 confirmed. |
| AC-7  | Burndown delta: +30 `SW-REQ-{IMU,BARO,GPS}-APP-*` IDs moved to **Verified** (10 + 10 + 10) | MET | `tools/burndown.py` and `tools/rtm.py` show 30 IDs `Verified`. G2 counter delta tracks +30 code-tagged and +26 `@verify`-tagged (4 Inspection-method requirements gap: SW-REQ-BARO-APP-007/-008 + SW-REQ-GPS-APP-003/-010). §7. |
| AC-8  | Two `Demonstration`-type baro_app TCs (SW-TC-BARO-APP-008, -009) recorded as **Deferred** with HIL CDR placeholders in SPRINT-IMPL-17 closure | MET | SPRINT-IMPL-17 closure §10 records deferral with placeholder filenames; not failed, not closed. CE Phase 4 confirmed. |
| AC-9  | INS-GPS-APP-003 (`docs/inspections/gps_app_no_nmea_coupling.md`) authored and reviewer-APPROVED | MET | Phase 1 SSE deliverable (186 lines); Phase 2 MAE iter-1 NEEDS CHANGES → iter-2 APPROVED (§6); discharges SW-REQ-GPS-APP-003 via Inspection. |
| AC-10 | INS-GPS-APP-010 (`docs/inspections/gps_app_hae_semantics.md`) authored and reviewer-APPROVED | MET | Phase 1 SSE deliverable (160 lines); Phase 2 MAE iter-1 APPROVED, 0 findings (§6); discharges SW-REQ-GPS-APP-010 via Inspection. |
| AC-11 | TC-006 Publish-vs-GetUtc ordering strengthening addressed (Lead-direct Q1.c) | MET | Lead-direct spot-verify against `apps/gps_app/tests/gps_app_test.cpp:253-260` found the suggested `EXPECT_LT(iGetUtcCallSeq, g_iLastPublishSeq)` change incorrect (TC-006 does not configure UTC publish; FIX publish at seq 5 precedes GetUtc call at seq 6). Original `EXPECT_GT(g_iLastPublishSeq, 0)` is the maximum strengthening achievable for TC-006 as scoped. Closed with 7-line carry-forward comment at the cited lines; finding documented in §10 #3. |
| AC-12 | `JUNO_TIME_US_T → JUNO_TIME_MICROS_T` sed rename across `docs/design/baro_app/design.md` (5 sites) + `docs/design/gps_app/design.md` (6 sites) | MET | 11 sites total sed-renamed; post-rename grep `JUNO_TIME_US_T` across `docs/design/{baro,gps}_app/` returns 0 hits; no logic change. |
| AC-13 | Wave 4+/Wave 6+ PM block status (SDP-R-08) explicitly documented in §10 with verbatim quote | MET | §10 #1 quotes SDP-R-08 verbatim and identifies Wave 6 PM gate; §10 #2 records the parallel Wave 4 block. |
| AC-14 | Phase 0 G1 + G2 + RTM + burndown baseline captured | MET | Phase 0 Lead pre-flight produced clean baseline: G1 19/19 PASS exit 0; G2 376/135/139/376 exit 0. Identical to GPS-18 §7 baseline (no drift). |
| AC-15 | Phase 3 G1 + G2 + RTM + burndown re-baseline matches Phase 0 (no drift introduced by Phase 1 edits) | MET | Re-baseline identical to Phase 0: G1 19/19 PASS exit 0; G2 376/135/139/376 exit 0. Documentation-only Phase 1 edits (INS records + sed rename + test-comment) introduced 0 code-tag / 0 `@verify` delta as expected. §7. |
| AC-16 | Project Chief Engineer issues PASS verdict on the Wave 5 Exit Gate | MET | CE APPROVED **unconditional** 2026-05-12 (§8) — 0 findings; all 8 SDP-mandated wave-consistency conditions confirmed; independent re-execution from clean `build_posix_ce/` reproduces G1+G2 exit 0. |
| AC-17 | Closure record (this file) authored ≤500 lines and SSE-reviewer-APPROVED | MET (at author) | This file is Phase 5a authored output; Phase 5b MAE review follows. Line count under cap; markdown well-formed. |

## 4. Deliverable File Inventory

3 in-sprint authored files (2 inspection records + this closure record):

| # | Path | Lines | Phase | Author | Final Status |
|---|------|-------|-------|--------|--------------|
| 1 | `docs/inspections/gps_app_no_nmea_coupling.md` | 186 | 1 | software-systems-engineer | APPROVED iter-2 (0 findings; iter-1 NEEDS CHANGES: 2 Major + 1 Minor citation findings — see §6) |
| 2 | `docs/inspections/gps_app_hae_semantics.md` | 160 | 1 | software-systems-engineer | APPROVED iter-1 (0 findings) |
| 3 | `docs/sprints/SPRINT-WAVE5-EXIT_sensor_apps.md` (this record) | (≤500) | 5 | software-systems-engineer | Phase 5a authored; Phase 5b MAE review pending |

**Lead-direct artifacts (not in deliverable count):**

- **LD-1**: `docs/sdp/sensor_apps.md` §2.2 AC-B — 3 lines edited; corrected `static_cast<APP_T&>(tRoot)` → `*reinterpret_cast<APP_T*>(&tRoot)` per the 2026-05-10 SPRINT-IMPL-16 lesson (`JUNO_MODULE_DERIVE` is composition / first-member embedding, not C++ inheritance; `static_cast` is invalid; `reinterpret_cast` is the canonical pattern per `sch_test_helpers.hpp:108`). This discharges SPRINT-IMPL-16 lesson 2026-05-10 LD-1.
- **LD-2**: `ai/memory/traceability.md` §Source Code Tagging — 8-line paragraph added explaining the single-line `@req`/`@verify` regex requirement (`tools/traceability.py:20`); cites the 2026-05-10 SPRINT-IMPL-16 lesson. Discharges SPRINT-IMPL-16 lesson 2026-05-10 LD-2.
- **LD-3** (AC-11): `apps/gps_app/tests/gps_app_test.cpp:253-260` — 7-line carry-forward closure comment documenting why the suggested `EXPECT_LT(iGetUtcCallSeq, g_iLastPublishSeq)` strengthening is incorrect for TC-006 (UTC publish is not configured; FIX publish at seq 5 precedes GetUtc call at seq 6). The suggested code change itself was reverted on spot-verify; the original `EXPECT_GT(g_iLastPublishSeq, 0)` is retained.
- **LD-4** (AC-12): `docs/design/baro_app/design.md` (5 sites) + `docs/design/gps_app/design.md` (6 sites) — 11 sites sed-renamed `JUNO_TIME_US_T → JUNO_TIME_MICROS_T`; no logic change. Post-rename grep returns 0 residual occurrences.

## 5. Workflow Phases — Iteration Summary

| Phase | Description | Agents | Iterations | Final |
|-------|-------------|--------|------------|-------|
| 0 | Lead pre-flight: G1+G2+RTM+burndown baseline capture from clean `build_posix/` | 1 (Lead) | 1 | Complete (baseline 376/135/139/376; 19/19 ctest PASS) |
| 1 | 4 Lead-direct edits (LD-1 SDP AC-B; LD-2 traceability.md tagging paragraph; LD-3 TC-006 comment + revert; LD-4 sed rename across 11 sites) + 2 SSE INS-record authors (parallel: INS-GPS-APP-003 + INS-GPS-APP-010) | 2 (SSE) | 1 each | Complete (Lead-direct edits all atomic, no iters) |
| 2 | 2 MAE reviewers for INS records (parallel) | 2 (MAE) | INS-003 iter-1 NEEDS CHANGES (2 Major + 1 Minor citations) → SSE iter-2 → MAE iter-2 APPROVED; INS-010 iter-1 APPROVED, 0 findings | Complete (1 re-review on INS-003) |
| 3 | Lead-direct G1+G2+RTM+burndown re-baseline post-Phase 2 | 1 (Lead) | 1 | Complete (re-baseline matches Phase 0 exactly: 376/135/139/376; 19/19 ctest PASS — documentation-only changes had no code-tag/`@verify` delta as expected) |
| 4 | Project Chief Engineer Wave 5 Exit Gate verdict | 1 (CE) | 1 | APPROVED unconditional (0 findings) |
| 5 | Closure-record authoring (5a SSE author + 5b MAE review) | 1 (SSE) + 1 (MAE) | 1 each | This record is Phase 5a |

**Total agent count:** 1 CE + 1 SSE (this record) + 1 MAE (record review) + 2 SSE (INS authors; one re-author for INS-003 iter-2) + 2 MAE (INS reviewers; one re-reviewer for INS-003 iter-2) = **8 agents** (vs. plan estimate 7; +1 from INS-003 iter-2). Plus 4 Lead-direct atomic edits (no iter cycles required for any). Lower agent count than a code-authoring sprint (BARO-17 / GPS-18 each used 10) is expected and was anticipated in the plan — a wave-exit gate is verification + carry-forward discharge, not new construction.

## 6. Reviewer Verdicts (chronological)

| # | Artifact | Phase | Reviewer | Verdict | Findings | Resolution |
|---|----------|-------|----------|---------|----------|------------|
| 1 | `docs/inspections/gps_app_no_nmea_coupling.md` (INS-GPS-APP-003) | 2 | mae-reviewer (inspection-record mode) | iter-1 NEEDS CHANGES | **Major-1**: Cited source lines `gps_app.cpp:269` and `:270` for the doc-comment authority but the actual nmea-mention doc-comments at those positions sit at lines `:268` and `:269` respectively (off-by-one). **Major-2**: Missed the `nmea_lib`-mention hit at `gps_app_test.cpp:176` (test-comment string in TC-003 regression-guard block) — the audit table claimed exhaustive coverage but did not enumerate the test-file hit. **Minor-1**: No rationale paragraph explaining why the audited 5 doc-comment hits do not constitute a coupling (e.g., explicit clarification that doc-comments are non-binding source text). | SSE iter-2 corrected both citation off-by-ones, added the missing `gps_app_test.cpp:176` hit (bringing the audit table to 6 hits across 4 files), and added a 4-line rationale paragraph on doc-comment-vs-`#include`/API-call coupling distinction. |
| 2 | `docs/inspections/gps_app_no_nmea_coupling.md` (INS-GPS-APP-003) | 2 (re-review) | mae-reviewer (inspection-record mode) | iter-2 APPROVED | 0 findings | — |
| 3 | `docs/inspections/gps_app_hae_semantics.md` (INS-GPS-APP-010) | 2 | mae-reviewer (inspection-record mode) | iter-1 APPROVED | 0 findings | — (HAE-semantics inspection cited `gps_app.hpp`, `gps_app.cpp`, `gps_lib/gps_msg.hpp` cleanly; reviewer noted WGS-84 datum citation matched authoritative source). |

### Reviewer outcomes commentary

INS-003's iter-1 findings validate the inspection-record sprint pattern's defining trap: SSE inspections cite source-code positions by line number, and any drift between the SSE's working snapshot and the canonical committed file produces citation off-by-ones that an experienced reviewer will catch with a single grep. The MAE caught two such drifts in INS-003 (cpp:269/270 → :268/:269 and the missed test.cpp:176 hit) with no false positives — high catch rate, low noise. INS-010 was a documentation-only inspection (no source-line citations beyond the published HAE field name in `gps_msg.hpp`) and consequently had no surface for citation drift to occur; iter-1 APPROVED, 0 findings, as expected.

## 7. Gate Results

### Gate G1 — POSIX build + ctest

Phase 0 baseline (and Phase 3 re-baseline, identical):
```
$ cd /home/juno/juno_fsw/build_posix && cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. \
    && cmake --build . && ctest --output-on-failure
...
100% tests passed, 0 tests failed out of 19
Total Test time (real) =   0.37 sec
Exit 0
```

CE independently re-executed from a clean `build_posix_ce/` directory and reproduced **exit 0** with 19/19 PASS. The Phase 3 re-baseline run reproduces the Phase 0 baseline exactly — Phase 1's documentation-only edits (2 INS records + sed rename + 7-line test-file comment + 2 ai/memory + 1 SDP edits) introduced 0 code-behavior delta, as expected. Phase 1 carry-forward discharge work was inherently zero-risk to G1.

### Gate G2 — Traceability tool

Phase 0 baseline (and Phase 3 re-baseline, identical):
```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       135
  Requirements with @verify:    139
  Requirements with test specs: 376
Exit 0
```

CE independently re-executed and reproduced **exit 0** with identical 376/135/139/376 counts. The Wave 5 burndown delta documented at AC-7 (+30 code, +26 `@verify` across SW-REQ-{IMU,BARO,GPS}-APP-*) accrues over SPRINT-IMPL-16/-17/-18; this sprint adds 0 additional delta because the inspection records discharge their parent requirements via the **Inspection** verification method (not via the automated `@verify` tagging pathway).

### Gate G3 — Pico2 cross-compile

**Not invoked.** Per `docs/sdp/sensor_apps.md` §2.3, "apps are platform-agnostic per the L2 designs ... so Gate G3 (Pico2 cross-compile) is **not invoked from the app sprint itself** — the Wave 8 system-integration sprint (`SPRINT-IMPL-25`) compiles `apps/main_pico2.cpp` against the linked app objects and runs G3 there." All three Wave 5 sprints honored this convention (no pico2-specific TUs introduced); G3 deferral remains valid through this exit gate.

## 8. Chief Engineer Verdict

**APPROVED unconditional** (2026-05-12) — zero findings.

The CE confirmed all 8 SDP-mandated wave-consistency conditions per `docs/sdp/sensor_apps.md` §4 items 1–8: (1) all three Wave 5 sprints CLOSED with G1+G2 exit 0; (2) single-level `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` uniformly across `apps/{imu,baro,gps}_app/` (grep returns exactly 3 hits, all single-level; no two-level UB pattern); (3) `tRoot.ptApi->Hook(...)` dispatch uniformly (grep `\btApi->` returns 0 hits); (4) broker template instantiation parity against Wave 0 `juno_fsw_capacities.hpp`; (5) canonical member-form TimestampToMicros uniformly (no fabricated free-function form anywhere); (6) RFA #1 manual-aggregate-init path identical across all three closure records; (7) burndown delta +30 (`SW-REQ-{IMU,BARO,GPS}-APP-*` × 10 each) with all 30 IDs `Verified`; (8) two BARO-17 Demonstration TCs (-008, -009) cleanly Deferred to HIL CDR with placeholder filenames in SPRINT-IMPL-17 §10. The 5 GPS-18 carry-forwards are discharged: INS-GPS-APP-003 (APPROVED iter-2 after 1 re-review on citation accuracy), INS-GPS-APP-010 (APPROVED iter-1, 0 findings), TC-006 ordering strengthening (closed-with-finding after spot-verify — the suggested change was incorrect; original assertion retained with documenting comment), `JUNO_TIME_US_T` sed rename (11 sites cleaned, 0 residual), and the explicit Wave 4+/Wave 6+ PM block documentation (§10 #1–#2). The 4 Lead-direct doc fixes (LD-1/-2/-3/-4) all landed without iter cycles. Independent G1+G2 re-execution from clean `build_posix_ce/` reproduces exit 0 with identical counters. No file exceeds the 500-line cap.

**CE recommendation:** "The Lead is authorized to proceed to Phase 5: closure-record authoring for `docs/sprints/SPRINT-WAVE5-EXIT_sensor_apps.md`. Wave 6 (domain apps) remains gated on PM-owned USER-NAV-LIB delivery per SDP-R-08 (independent of this gate verdict)."

## 9. Requirements Closure

This sprint authored **zero new requirements** and Verified **zero additional requirements via code/test** (the +30 Wave 5 SW-REQ-{IMU,BARO,GPS}-APP-* delta accrued over SPRINT-IMPL-16/-17/-18 and was already counted in their respective closure records). It **does** discharge two `verification_method: Inspection` requirements via signed inspection records authored this sprint:

| Requirement ID | Title | Discharge Mechanism | Status |
|----------------|-------|---------------------|--------|
| SW-REQ-GPS-APP-003 | Delegate NMEA Parsing to NMEA Library | Inspection PASS via `docs/inspections/gps_app_no_nmea_coupling.md` (INS-GPS-APP-003, iter-2 APPROVED) | Verified (code-tag + signed Inspection) |
| SW-REQ-GPS-APP-010 | HAE Altitude in Published GPS Fix | Inspection PASS via `docs/inspections/gps_app_hae_semantics.md` (INS-GPS-APP-010, iter-1 APPROVED) | Verified (code-tag + signed Inspection) |

The other two `verification_method: Inspection` Wave 5 requirements (`SW-REQ-BARO-APP-007` SI Units, `SW-REQ-BARO-APP-008` HAE Field) remain queued for a future RTM-cleanup sprint as inspection records — they are code-tagged in Wave 5 sources (per SPRINT-IMPL-17 RTM closure) but have no formal inspection record yet. See §10 #4.

## 10. Carry-Forward Notes

1. **Wave 6+ blocked on PM USER-NAV-LIB delivery per SDP-R-08** — Wave 5 Exit Gate PASS does **not** unblock Wave 6. SDP-R-08 verbatim:
   > "USER-NAV-LIB and USER-NAV-APP are out-of-band PM-owned implementations (Revision C). Downstream sprints (afm_lib -13, afm_app -20, telem_lib -14, telem_app -21, mlog_lib -15, mlog_app -22, sys_app -23) cannot start until the PM signals nav_lib and nav_app delivery. Agent-system burndown excludes these two scopes."
   PM signals readiness in writing before each downstream sprint opens (per SDP-R-08 mitigation (c)). The CE Phase 4 verdict reaffirms this: Wave 6 gating is PM-owned, independent of this gate verdict.

2. **Wave 4 (afm_lib, telem_lib, mlog_lib) also PM-blocked on USER-NAV-LIB delivery** per SDP-R-08. afm_lib depends on USER-NAV-LIB types; telem_lib depends on lora_lib + USER-NAV-LIB types + afm_lib types; mlog_lib depends on sd_lib + USER-NAV-LIB + afm_lib types. None of the three Wave 4 sprints (SPRINT-IMPL-13/-14/-15) can open until PM signals USER-NAV-LIB delivery. This is identical bookkeeping to #1 above (same SDP-R-08 root cause) but is worth restating because Wave 4 is upstream of Wave 6 and a casual reader might assume "Wave 5 PASS unblocks Wave 4" — it does not.

3. **TC-006 ordering strengthening carry-forward — CLOSED with finding 2026-05-12.** GPS-18 §10 carry-forward #5 suggested a 1-line improvement on `apps/gps_app/tests/gps_app_test.cpp:250`, replacing `EXPECT_GT(g_iLastPublishSeq, 0)` with the canonical `EXPECT_LT(iGetUtcCallSeq, g_iLastPublishSeq)` paired-sequence form. On Lead-direct spot-verify the suggestion was found incorrect: **TC-006 does not configure UTC publish**, so the FIX publish (call sequence 5) precedes the `GetUtc` call (sequence 6), and an `EXPECT_LT(iGetUtcCallSeq=6, g_iLastPublishSeq=5)` would deterministically fail. The original `EXPECT_GT(g_iLastPublishSeq, 0)` is the maximum strengthening achievable for TC-006 as scoped — it asserts that at least one publish happened, which is the load-bearing assertion in a non-UTC-publishing test case. Documented in a 7-line carry-forward closure comment at `apps/gps_app/tests/gps_app_test.cpp:253-260`. **Lesson:** spot-verify reviewer carry-forward suggestions against actual test-case configuration before discharge — reviewer carry-forwards from `Warning`-tier review findings may have been written without re-reading the test case's setup section.

4. **Future RTM-cleanup sprint queued for:**
   - **INS-BARO-APP-007** (SW-REQ-BARO-APP-007 SI Units inspection record) — same pattern as INS-GPS-APP-003 authored this sprint; documents that SI units (Pa, °C) are used throughout the baro_app message payload.
   - **INS-BARO-APP-008** (SW-REQ-BARO-APP-008 HAE Field inspection record) — same pattern as INS-GPS-APP-010 authored this sprint; documents WGS-84 HAE semantics for the baro_app altitude field.
   - Any newly-surfaced Inspection-method requirements as Wave 6+ opens (e.g., afm_app FAIL-STATE-INSPECTION, telem_app DOWNLINK-RATE-INSPECTION) will queue here.

5. **GPS-18 carry-forward #4 (INS-GPS-APP-003 / -010) — DISCHARGED this sprint.** 2 inspection records authored: `docs/inspections/gps_app_no_nmea_coupling.md` (INS-GPS-APP-003 iter-2 APPROVED, 186 lines) + `docs/inspections/gps_app_hae_semantics.md` (INS-GPS-APP-010 iter-1 APPROVED, 160 lines). Both PASS verdict; both discharge their parent `SW-REQ-GPS-APP-*` Inspection-method requirements. Both regression-guard TC-003/-012 TEST_Fs in `gps_app_test.cpp` remain active and unchanged.

6. **GPS-18 carry-forward #6 (`JUNO_TIME_US_T` design-doc drift) — DISCHARGED this sprint.** Sed rename across 11 sites (5 in `docs/design/baro_app/design.md` + 6 in `docs/design/gps_app/design.md`) → `JUNO_TIME_MICROS_T`; post-rename grep `JUNO_TIME_US_T` across `docs/design/{baro,gps}_app/` returns 0/0 residual. No logic change; documentation-only sweep. BARO-17 §10 carry-forward #5 is now also discharged transitively (it referenced the same drift). Pattern: if a recurring documentation drift is mentioned in two consecutive closure records' carry-forward lists, fold the discharge into the next Wave Exit Gate rather than waiting for a dedicated doc-cleanup sprint.

7. **SPRINT-IMPL-16 lesson 2026-05-10 LD-1 + LD-2 — DISCHARGED this sprint.** LD-1: `docs/sdp/sensor_apps.md` §2.2 AC-B was corrected from `static_cast<APP_T&>(tRoot)` to `*reinterpret_cast<APP_T*>(&tRoot)` per the 2026-05-10 lesson that `JUNO_MODULE_DERIVE` is composition (first-member embedding) and not C++ inheritance — `static_cast` between composition-related types is UB; the canonical pattern is `*reinterpret_cast<APP_T*>(&tRoot)` per `sch_test_helpers.hpp:108`. LD-2: `ai/memory/traceability.md` §Source Code Tagging amended with an 8-line paragraph on the single-line regex requirement at `tools/traceability.py:20` (multi-line wrapped JSON silently breaks parsing). Both edits Lead-direct atomic; no iter required.

8. **Legacy main cleanup queued for SPRINT-IMPL-25** (continued from GPS-18 §10 #3; no new entries this sprint). Running total for SPRINT-IMPL-25 to migrate: 3 stale `juno_log` includes (from SPRINT-IMPL-02 deletion) + 3 stale `gps/gps_app.h` includes (from SPRINT-IMPL-18 legacy-C deletion) = 6 stale references, all gated under `JUNO_FSW_BUILD_LEGACY_MAIN` (default OFF).

9. **Wave 8 sim + integration sprints (SPRINT-IMPL-24, -25) also blocked transitively** through Wave 6 + Wave 7 on USER-NAV-LIB / USER-NAV-APP delivery. SPRINT-IMPL-24 (sim modules) depends on every Wave 1–7 module; SPRINT-IMPL-25 (system integration) depends on SPRINT-IMPL-24. Both inherit the SDP-R-08 PM gate.

## 11. Lessons Learned (cross-referenced)

To be recorded in `ai/memory/lessons-learned-software-lead.md` by the Software Lead after this sprint closes; the Lead will refine wording:

- "2026-05-12 — SPRINT-WAVE5-EXIT: Wave Exit Gate sprint shape (verification + carry-forward discharge) is lower-cost than implementation sprint (8 agents vs. 10+ for a code sprint). Bundling carry-forward discharge into the Exit Gate sprint avoids stranding inspection-record carry-forwards indefinitely. Fold this shape into the next wave-exit slot rather than waiting for a dedicated RTM-cleanup sprint."
- "2026-05-12 — SPRINT-WAVE5-EXIT: Reviewer carry-forward suggestions must be spot-verified against actual code before discharge — the TC-006 ordering-strengthening suggestion (GPS-18 §10 #5) was incorrect on closer inspection (TC-006 does not configure UTC publish; the suggested `EXPECT_LT` would deterministically fail). Pattern: `Warning`-tier review findings may have been written without re-reading test-case setup; treat them as hints, not specs."
- "2026-05-12 — SPRINT-WAVE5-EXIT: Inspection-record sprint pattern — 2 SSE authors + 2 MAE reviewers works cleanly; MAE catch rate on citation accuracy is high (INS-GPS-APP-003 iter-1 had 2 Major citation findings on line-number drift). **Spot-verify SSE inspection records via independent grep before Phase 2 launch** — Lead pre-flight on inspection records should include a fresh `grep -n <citation_token> <cited_file>` for every source-line citation in the SSE draft, catching drift before MAE re-discovery."
- "2026-05-12 — SPRINT-WAVE5-EXIT: Atomic-Lead-edit pattern continues to scale — the 4 Lead-direct doc fixes (LD-1 SDP AC-B; LD-2 traceability.md tagging paragraph; LD-3 TC-006 carry-forward closure comment; LD-4 11-site sed rename) all landed without iter cycles. Atomic Lead-direct is the right shape for documentation-only edits that have no behavioral surface."
- "2026-05-12 — SPRINT-WAVE5-EXIT: Recurring-drift discharge timing — when a documentation drift is mentioned in two consecutive sprint closure records' carry-forward lists (e.g., `JUNO_TIME_US_T → JUNO_TIME_MICROS_T` in BARO-17 §10 #5 and GPS-18 §10 #6), fold the discharge into the next Wave Exit Gate rather than queuing a dedicated doc-cleanup sprint."

(Senior/junior software engineer lessons-learned files do not require updates this sprint — the sprint was SSE + MAE + Lead-only; no senior/junior worker phases.)

## 12. Authority

| Role | Identity | Verdict | Date |
|------|----------|---------|------|
| Software Lead | This sprint | CLOSE (pending PM signoff) | 2026-05-12 |
| Project Chief Engineer | (agent) | APPROVED unconditional, 0 findings | 2026-05-12 |
| Project Manager | Robin Onsay | Pending | — |
