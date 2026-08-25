---
document_type: nav_lib L2 Design — Index
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
status: Active (post-PDR + post-EKF-amendment)
---

# nav_lib — L2 Design (Index)

The `nav_lib` L2 design is split across three sister files for readability and to keep each under the 500-line constraint per `ai/memory/constraints.md`. This index is the canonical entry point. (Three-file split landed 2026-05-08 per the `NAV-A3` carry-forward, sprint `SPRINT-IMPL-NAV-HOUSEKEEPING`.)

## Files

| File | Scope |
|------|-------|
| [design.md](design.md) | **Public API contract:** `NAV_LIB_API_T` vtable, `NAV_LIB_ROOT_T` shape, `NAV_INIT_T` configuration schema (incl. all noise/covariance fields and `fInitialCovDiag[16]`), per-call contracts, FSW-extension status codes (`juno::nav` namespace). Covers `SW-REQ-NAV-001..017` (API-surface aspects). |
| [contracts.md](contracts.md) | **Runtime contracts:** state machine + alignment criteria (§5), data flow (§6), sequence diagrams (§7), timing and scheduling analysis (§8), error handling strategy (§9), memory ownership (§10), full per-requirement traceability table (§11). Covers `SW-REQ-NAV-001..016` (runtime/lifecycle aspects). |
| [algorithm.md](algorithm.md) | **EKF algorithm specification:** state vector composition, process model (8-step strapdown), baro and GPS measurement models, GPS divergence-bound check, noise/covariance load-time configuration surface, reference covariance values, numerical stability mandates (quaternion renorm, Joseph form, symmetry enforcement, pivot guarding), phase-aware behavior cross-reference. Covers `SW-REQ-NAV-018..020`. |

## How to read

- **Implementer of `nav_lib`** (PM-owned USER-NAV-LIB per SDP Revision C): read `algorithm.md` first for the algorithm specification, then `design.md` for the public API contract, then `contracts.md` for the runtime state machine and error handling that the implementation must satisfy.
- **Caller of `nav_lib`** (e.g., `nav_app` author): read `design.md` for the public surface and `contracts.md` for state machine + sequence diagrams; refer to `algorithm.md` only when reasoning about filter-tuning behavior.
- **Reviewer (MAE / SSE-R / CE)**: all three files are authoritative; cross-reference the per-section design tags `<!-- @{"design": [...]} -->` to confirm requirement coverage. `contracts.md §11` is the consolidated traceability table for `SW-REQ-NAV-001..016`; `algorithm.md §11` is the delta for `SW-REQ-NAV-018..020`.

## Cross-references

- [conventions.md](../conventions.md) — cross-module vocabulary, time base, frames, status codes
- [system_design.md](../system/system_design.md) — L1 system design (composition root, bus catalog, scheduler)
- [nav_app/design.md](../nav_app/design.md) — View-layer caller; owns the phase-aware gating logic per `SW-REQ-NAV-APP-014`/`-015`
- [kmat/04_interface.md](../kmat/04_interface.md) — published kmat types this lib uses for state and covariance storage
- `libjuno/include/juno/result.hpp` — `RESULT_T<T>` returned by every nav_lib function
- `libjuno/include/juno/status.h` — canonical 19-code status catalog and `JUNO_STATUS_CUSTOM_ERROR` base for FSW extensions

## Requirements covered

`docs/requirements/nav/requirements.json` — `SW-REQ-NAV-001` through `SW-REQ-NAV-020` (20 total; `-018` and `-019` added in the 2026-05-03 EKF amendment sprint, `-020` added in the 2026-05-04 implementation-readiness remediation; `-018..-020` covered exclusively by [algorithm.md](algorithm.md), `-001..-017` split between [design.md](design.md) (API surface) and [contracts.md](contracts.md) (runtime contracts) per the 2026-05-08 NAV-A3 split).

## Approval

**Predecessor.** PDR Closure — CE issued unconditional GO 2026-05-03 after the Delta-PDR Remediation Sprint. This delta-PDR amendment was directed by the PM the same day and must land before SPRINT-IMPL-12 (nav_lib) opens per the SDP.

**Sprint summary.** Nav EKF + Phase-Aware Fusion delta-PDR amendment: pinned the algorithm to EKF, exposed eight load-time noise/covariance fields on `NAV_INIT_T`, and placed the boost-phase sensor-update gating with 1-second settling window in `nav_app` via a new `JUNO_MSG_AFM_PHASE_T` subscription.

**Reviewer verdicts.**

| Phase | Reviewer | Verdict |
|-------|----------|---------|
| Phase 2 (4 JSON deltas) | Module Architecture Engineer (MAE) | APPROVED 13/13 first iteration |
| Phase 5 (holistic design review) | Module Architecture Engineer (MAE) | APPROVED post-remediation (3 atomic findings closed Lead-direct) |
| Phase 5 (technical review) | Software Systems Engineer — Reviewer (SSE-R) | APPROVED post-remediation (4 findings closed Lead-direct: digit separator, predicate ordering, afm_lib include, JUNO_PHASE_T qualification) |
| Phase 6 (final gate) | Project Chief Engineer (CE) | **PASS-WITH-ACTIONS** — see rationale below |

**CE verdict and rationale.** The amendment is internally consistent and IEEE-29148/1016/829 compliant: all four new requirements (`SW-REQ-NAV-018/-019`, `SW-REQ-NAV-APP-014/-015`) are observable, atomic, "shall"-form, and within the 20-word target (`docs/requirements/nav/requirements.json` lines 192-212; `docs/requirements/nav_app/requirements.json` lines 148-168); both new test cases exist with parents (`SW-TC-NAV-021/-022`, `SW-TC-NAV-APP-017/-018`) and `tools/traceability.py` exits 0 with 375 reqs / 374 test specs as the brief stated. The pinned EKF specification in `docs/design/nav/algorithm.md` (486 lines, under the 500-cap) covers all 11 IEEE-1016 sections including state vector composition (§3.1), process model (§3.2), baro and GPS measurement models (§4.1-4.3), the eight load-time `NAV_INIT_T` noise/covariance fields (§5.1) with installation-only reference values (§5.2), and numerical-stability hazards (§6 — quaternion renormalization, Joseph form, symmetry enforcement, pivot guarding); §8 correctly delegates phase-aware behavior to `nav_app` and §11 carries the new `SW-REQ-NAV-018/-019` traceability rows. `docs/design/nav/design.md` cleanly pins the algorithm at lines 14, 20, 49, 60, 66, 254 and adds the eight noise/covariance fields to `NAV_INIT_T` at lines 128-138 with `SW-REQ-NAV-019` as the rationale, and the §11 traceability table is extended (lines 571-572). `docs/design/nav_app/design.md` (490 lines) correctly places the gating in the View layer with the SSE-R-mandated short-circuit predicate `_tBoostExitUs > 0 && (tNowUs - _tBoostExitUs) < kNavAppBoostSettlingUs` (line 194), `kNavAppBoostSettlingUs = 1000000` (no C++14 digit separator, line 144), the `JUNO_PHASE_BOOST` enumerator qualified through `juno::afm::JUNO_PHASE_T`, and both `afm_lib/afm_api.hpp` + `afm_lib/afm_msg.hpp` includes restored (lines 104, 109). The nav/algorithm.md uses only kmat published types referenced through `docs/design/kmat/04_interface.md` per SSE-R, and freestanding-compliance is asserted in §7. Three independent action items are tracked as PASS-WITH-ACTIONS rather than blockers because they are inherited from outside this sprint's scope, are explicitly enumerated in the closure_memo carry-forward, and do not invalidate the EKF pin or the phase-aware fusion contract. SPRINT-IMPL-12 (nav_lib) and SPRINT-IMPL-19 (nav_app) may open with this amendment as their authoritative anchor.

**Final Verdict: PASS-WITH-ACTIONS**

**Action items (non-blocking, tracked as future-sprint inputs).**

1. **[CARRY-FORWARD-NAV-A1] Bidirectional `child_ids` update on `SW-REQ-SYS-013`.** Add `SW-REQ-NAV-018` and `SW-REQ-NAV-APP-014` to `SW-REQ-SYS-013.child_ids` in `docs/requirements/sys/requirements.json` so the parent-child links are bidirectional. Owner: next requirements-housekeeping sprint. Target: before SPRINT-IMPL-12 opens (≤2026-05-10).
2. **[CARRY-FORWARD-NAV-A2] `JUNO_MSG_BUS_VARIANT_T` must include `JUNO_MSG_AFM_PHASE_T`.** The nav_app `OnStart` subscription added in this sprint (`docs/design/nav_app/design.md` §4.3) requires the bus-variant catalog to carry the AFM phase message; this is a Wave-0 prerequisite already noted in the closure_memo §5 #2. Owner: composition-root / bus-catalog sprint. Target: before SPRINT-IMPL-19 opens.
3. **[CARRY-FORWARD-NAV-A3] nav/design.md split.** `docs/design/nav/design.md` is 574 lines (pre-existing 500-cap violation; +16 lines from this sprint's necessary edits to extend NAV_INIT_T and the §11 traceability table). Plan a Lead-direct refactor to split design.md into multi-file sister documents under the existing `nav/index.md` TOC. Owner: Software Lead. Target: before nav_lib v1.0 freeze.
4. **[CARRY-FORWARD-NAV-A4] `SW-REQ-NAV-019` parent re-pointing.** The current parent `SW-REQ-NAV-001` is imperfect (Phase 1 worker flagged); a future sprint may add a dedicated nav-lib lifecycle/init parent and re-point. Owner: requirements steward. Target: opportunistic, not blocking.
5. **[CARRY-FORWARD-NAV-A5] `NAV_APP_INIT_T` for caller-overridable settling window.** The 1-second settling window is currently a `constexpr` in nav_app; a future amendment may introduce `NAV_APP_INIT_T` to make it caller-overridable per the rationale on `SW-REQ-NAV-APP-015`. Owner: nav_app maintainer. Target: before FT2 if per-build retuning is required.

## Approval — Iteration 2 (post-remediation)

**Predecessor.** Prior `## Approval` section above (CE PASS-WITH-ACTIONS issued 2026-05-03 by the Project Chief Engineer immediately after the Nav EKF + Phase-Aware Fusion delta-PDR amendment landed). That verdict explicitly enumerated five carry-forward actions (`NAV-A1..A5`) and noted that the amendment was implementation-anchor for SPRINT-IMPL-12 (`nav_lib`) and SPRINT-IMPL-19 (`nav_app`). The PM then directed a targeted re-review which surfaced 4 BLOCKER + 3 RECOMMENDATION implementation-readiness gaps (G1–G7) that would have stopped the IMPL workers on day one; the Lead executed the remediation sprint that just landed.

**Sprint summary.** Implementation-readiness remediation sprint closed all 7 gaps via Lead-direct atomic edits: G1 (analytic F Jacobian normative literature anchors — Groves 2013 §14.2, Trawny & Roumeliotis 2005 TR-2005-002 — and explicit prohibition of finite-difference Jacobians in `algorithm.md` §3.2); G2 (new `SW-REQ-NAV-020` + `SW-TC-NAV-023` + `fInitialCovDiag[16]` field on `NAV_INIT_T` to make P_0 caller-supplied in `design.md` §4.1 and `algorithm.md` §5.1); G3 (new normative `design.md` §5.1 with three "shall" alignment conditions replacing the prior "e.g." wording); G4 (AFM_PHASE delivery semantics with cross-reference to `afm_app/design.md` §6 publish-on-every-tick and latest-value-retention pattern in `nav_app/design.md` §6.1); G5 (5-second dead-reckoning timeout with 3.3× safety margin in `design.md` §9); G6 (test reference path + quaternion tolerance unit fix in `SW-TC-NAV-021`); G7 (AFM_PHASE subscription line on `nav_app/design.md` §7.1 sequence diagram).

**SSE-R Phase 2 verdict.** APPROVED 7/7 — all four BLOCKERs and all three RECOMMENDATIONs closed; no new defects introduced; concurred on folding the `algorithm.md` 511-line expansion into the existing `NAV-A3` carry-forward.

**CE Iteration 2 verdict and rationale.** The four BLOCKER gaps that drove the prior PASS-WITH-ACTIONS rationale are now closed against verifiable evidence: (1) `docs/design/nav/algorithm.md` lines 147–167 mandate the analytic F derived from Groves 2013 §14.2 or Trawny & Roumeliotis TR-2005-002 and forbid numerical Jacobians; (2) `docs/requirements/nav/requirements.json` carries `SW-REQ-NAV-020`, `docs/test_cases/nav/test_cases.json` carries `SW-TC-NAV-023`, `docs/design/nav/design.md` lines 140–153 add `fInitialCovDiag[kNavStateDim]` to `NAV_INIT_T`, and `docs/design/nav/algorithm.md` §5.1 documents per-state P_0 indexing; (3) `docs/design/nav/design.md` §5.1 (lines 353–361) is an unambiguous "shall"-form alignment criterion set; (4) `docs/design/nav_app/design.md` lines 274 and 278 cross-reference `afm_app/design.md` §6/§11.2 publish-on-every-tick and pin the latest-value retention pattern. The three RECOMMENDATIONs are likewise closed (G5 in `design.md` §9 lines 504–511; G6/G7 verified in their respective files). `tools/traceability.py` exits 0 with 376 reqs / 375 test specs (delta of +1 each is exactly the new `SW-REQ-NAV-020` and `SW-TC-NAV-023`, matching brief expectations); `tools/requirements_search.py --validate docs/requirements/nav/requirements.json` PASSED. Two file-budget notes: `docs/design/nav/design.md` grew 574→608 and `docs/design/nav/algorithm.md` grew 486→511 (newly over-cap); both fold into the existing `NAV-A3` refactor scope per SSE-R concurrence and are not implementation blockers because the IMPL workers do not need a reformat to read these documents. The remaining carry-forward items (`NAV-A1` housekeeping bidirectional links, `NAV-A2` Wave-0 bus-variant prereq tracked outside this sprint's scope, `NAV-A3` file split, `NAV-A4` opportunistic parent re-pointing, `NAV-A5` FT2 settling-window override hook, and the new `NAV-A6` FT2 dead-reckoning timeout configurability) are bookkeeping/opportunistic items, not architectural ambiguities; SPRINT-IMPL-12 and SPRINT-IMPL-19 workers can now begin without waiting on any of them. This upgrades the iteration-1 PASS-WITH-ACTIONS to unconditional PASS.

**Final Verdict (Iteration 2): PASS**

**Updated carry-forward action list (post-remediation).**

1. **[CARRY-FORWARD-NAV-A1] Bidirectional `child_ids` on `SW-REQ-SYS-013`** — unchanged from iteration 1; still out of this sprint's scope; opportunistic housekeeping.
2. **[CARRY-FORWARD-NAV-A2] `JUNO_MSG_BUS_VARIANT_T` must include `JUNO_MSG_AFM_PHASE_T`** — unchanged from iteration 1; Wave-0 prerequisite for the composition-root / bus-catalog sprint; required before SPRINT-IMPL-19 opens.
3. **[CARRY-FORWARD-NAV-A3] nav file-budget split — SCOPE EXPANDED.** Now covers BOTH `docs/design/nav/design.md` (608 lines) AND `docs/design/nav/algorithm.md` (511 lines). The prior single-file refactor is rescoped to a multi-file split under the existing `nav/index.md` TOC; SSE-R Phase-2 explicitly concurred on folding `algorithm.md` into this existing action rather than opening a new one. Owner: Software Lead. Target: before nav_lib v1.0 freeze.
4. **[CARRY-FORWARD-NAV-A4] `SW-REQ-NAV-019` parent re-pointing** — unchanged from iteration 1; opportunistic, not blocking.
5. **[CARRY-FORWARD-NAV-A5] `NAV_APP_INIT_T` for caller-overridable settling window** — unchanged from iteration 1; FT2 retuning hook.
6. **[CARRY-FORWARD-NAV-A6] Dead-reckoning timeout configurability — NEW.** The 5-second dead-reckoning budget added to `docs/design/nav/design.md` §9 line 505 in this sprint is normative-pinned but not caller-configurable; FT2 may add a `NAV_INIT_T.fDeadReckoningTimeoutS` field to permit per-build retuning. Owner: nav_lib maintainer. Target: opportunistic, FT2 only if per-build retuning is required (not blocking for FT1).
