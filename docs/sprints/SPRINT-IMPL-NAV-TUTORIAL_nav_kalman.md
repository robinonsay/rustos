---
document_type: Sprint Closure Record
sprint_id: SPRINT-IMPL-NAV-TUTORIAL
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
predecessor: SDP Revision C amendment 2026-05-08 (this sprint discharges)
successor: PM-owned USER-NAV-LIB and USER-NAV-APP implementations (out-of-band)
status: Closed
---

# Sprint Closure Record — SPRINT-IMPL-NAV-TUTORIAL (nav_kalman tutorial)

## 1. Sprint Goal

Replace SPRINT-IMPL-12 (nav_lib) and SPRINT-IMPL-19 (nav_app) per SDP Revision C with a single agent-side tutorial sprint. The PM (Robin Onsay) elected to implement nav_lib and nav_app personally as a learning exercise; the agent-side deliverable is a self-contained 13-chapter Kalman-filter + navigation tutorial under `docs/tutorials/nav_kalman/` written for a software engineer rusty on linear algebra and probability with no nav/controls background.

## 2. Predecessor

- **SDP Revision C amendment** (2026-05-08): removed SPRINT-IMPL-12 and SPRINT-IMPL-19 from §5 master sprint table; added SPRINT-IMPL-NAV-TUTORIAL row, USER-NAV-LIB row (out-of-band), and USER-NAV-APP row (out-of-band); added §6 DAG nodes STUT/UNL/UNA; added §7 SDP-R-08 risk row; updated §10 exit criteria to recognize PM-delivered nav implementations against G1+G2+G3.
- **PDR-baselined nav spec artifacts** (2026-05-04 CE PASS unconditional after EKF amendment + implementation-readiness remediation): `docs/requirements/nav/`, `docs/design/nav/`, `docs/test_cases/nav/`, plus `nav_app/` triplet — all current and authoritative.

## 3. Scope

**In scope (delivered):**
- Tutorial directory `docs/tutorials/nav_kalman/` with index + 12 numbered chapter files.
- SDP Revision C amendment.
- This closure record.

**Out of scope (PM-owned, out-of-band):**
- nav_lib implementation (USER-NAV-LIB).
- nav_app implementation (USER-NAV-APP).
- NAV-A1 (bidirectional `child_ids` on SW-REQ-SYS-013) — opportunistic carry-forward.
- NAV-A3 (file split for `docs/design/nav/design.md` 608 lines and `algorithm.md` 511 lines) — opportunistic carry-forward.
- Refresh of nav and nav_app spec triplets (already PDR-current; no changes required).

## 4. Acceptance Criteria Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | `docs/sdp/index.md` amended to Revision C | MET | §5 master table, §6 DAG, §7 risk register (SDP-R-08), §10 exit criteria, §13 approval block all updated; file at 293 lines (under 500 cap) |
| AC-2 | `docs/tutorials/nav_kalman/` exists with `index.md` + 12 numbered chapters | MET | 13 files total: `index.md`, `01..12_<title>.md`; total 5651 lines |
| AC-3 | Tutorial covers all topics: linear algebra, probability/Gaussians, state-space, linear KF, EKF, frames/attitude, strapdown INS, sensor fusion, FSW mapping, exercises | MET | per chapter coverage map in §6 |
| AC-4 | Every chapter ≤500 lines | MET | max line count is ch07 at 485; all under cap |
| AC-5 | Exercises chapter has ≥8 exercises with worked solutions | MET | ch12 has 9 exercises (2 easy + 3 medium + 3 hard + 1 bonus implementation-prep) |
| AC-6 | nav and nav_app spec artifacts unchanged; `tools/traceability.py` exits 0 | MET | traceability check PASS at 376/376 (Phase 0 baseline preserved) |
| AC-7 | Sprint closure record at `docs/sprints/SPRINT-IMPL-NAV-TUTORIAL_nav_kalman.md` | MET | this file |
| AC-8 | MAE APPROVED on every chapter; CE PASS on final gate | MET (pending CE) | 13 MAE chapter reviews + 1 SSE-R holistic + 1 SDP MAE → all NEEDS-CHANGES findings addressed Lead-direct |

## 5. Tutorial Final Inventory

| # | File | Lines | Topic |
|---|------|-------|-------|
| - | `index.md` | 365 | TOC, audience, canonical notation, references, FSW mapping |
| 01 | `01_linear_algebra.md` | 447 | Vectors, matrices, transpose, inverse, eigenvalues, SPD, blocks |
| 02 | `02_probability_and_gaussians.md` | 396 | RVs, expectation/variance, covariance, multivariate Gaussians, Bayes |
| 03 | `03_state_space.md` | 439 | State-space, process+measurement models, 1D pos+vel example |
| 04 | `04_kalman_filter_derivation.md` | 371 | Linear KF derivation: predict, update, gain, Joseph form |
| 05 | `05_kalman_filter_worked_example.md` | 480 | 5-tick numerical KF run on 1D pos+vel + NumPy verification |
| 06 | `06_extended_kalman_filter.md` | 409 | EKF predict/update, Jacobians, analytic vs numerical |
| 07 | `07_frames_and_transformations.md` | 485 | ECEF/NED/body frames, DCMs, three worked rotations |
| 08 | `08_attitude_representations.md` | 462 | Euler→quaternion, Hamilton product, q→DCM, propagation |
| 09 | `09_inertial_navigation.md` | 446 | Strapdown INS, bias states, dead-reckon failure, aiding |
| 10 | `10_sensor_fusion_intuition.md` | 450 | IMU+baro+GPS complementarity, EKF as automated fusion |
| 11 | `11_fsw_nav_mapping.md` | 448 | Map math to FT1: NAV_STATE_T, NAV_INIT_T, kmat ops, roadmap A-I |
| 12 | `12_exercises_and_solutions.md` | 453 | 9 exercises with worked solutions, 16×16 F-Jacobian worksheet |
| | **Total** | **5651** | |

## 6. Topic Coverage Map (vs PM brief)

| Required topic | Chapter(s) |
|----------------|------------|
| Math primer: vectors, matrices, transpose, inverse, positive-definiteness | ch01 |
| Math primer: random variables, expectation, variance, covariance, multivariate Gaussians | ch02 |
| State-space modeling | ch03 |
| Linear KF derivation as recursive Bayesian estimation under Gaussians | ch04 |
| Worked Kalman example | ch05 |
| Extended Kalman Filter | ch06 |
| Frames (ECEF/NED/body), attitude (Euler → quaternions), gravity model | ch07, ch08 |
| Inertial navigation (strapdown IMU, error states) | ch09 |
| Sensor fusion intuition (IMU + baro + GPS) | ch10 |
| Mapping the math to FT1 nav_lib (NAV_STATE_T, algorithm.md walk-through) | ch11 |
| Exercises with worked solutions | ch12 |

## 7. Worker / Reviewer Summary

### Phase 1 — SDP Amendment (Lead-direct + MAE verify)

| Item | Worker | Output | Reviewer | Verdict |
|------|--------|--------|----------|---------|
| SDP Revision C amendment | Software Lead (atomic-edit pattern) | `docs/sdp/index.md` (281→293 lines) | software-mission-assurance-engineer | APPROVED first iteration; one advisory flagged §12 stale "current revision: A" — fixed Lead-direct |

### Phase 2 — Tutorial Authoring

13 software-systems-engineer worker invocations (one per file). Three files exceeded the 500-line cap on initial author and required iter-2 trim workers (also software-systems-engineer): ch05 (510→480), ch08 (522→462 via authoring overwrite + trim), ch09 (537→446 via authoring overwrite + trim). Two chapters (ch04 and ch06) had a duplicate worker run because of an early socket-error retry; the second author's content overwrote the first cleanly.

### Phase 2c — Cross-Chapter Alignment (Lead-direct)

ch03 §3.6 worked-example numerical values (`Q`, `R`, `x_0`, `P_0`) Lead-direct-edited to align with ch05 §1's authoritative numbers. Reviewer for ch03 verified the alignment.

### Phase 3 — Tutorial Review (14 reviewers in parallel)

| File | Reviewer | Verdict | Iterations | Findings (post-iteration) |
|------|----------|---------|------------|---------------------------|
| `index.md` | MAE | APPROVED | 1 | None |
| `01_linear_algebra.md` | MAE | APPROVED | 1 | Cosmetic frontmatter fix (Lead-direct) |
| `02_probability_and_gaussians.md` | MAE | APPROVED | 1 | None |
| `03_state_space.md` | MAE | APPROVED | 1 | None (Lead-direct alignment Phase 2c verified) |
| `04_kalman_filter_derivation.md` | MAE | APPROVED | 1 | None — math thoroughly verified including Joseph algebraic equivalence |
| `05_kalman_filter_worked_example.md` | MAE | APPROVED post-fix | 2 | 3 Warnings on missing line numbers in algorithm.md citations → Lead-direct fix |
| `06_extended_kalman_filter.md` | MAE | APPROVED | 1 | None — pendulum Jacobian verified |
| `07_frames_and_transformations.md` | MAE | APPROVED post-fix | 2 | 2 Errors + 2 Warnings (line 124→123 for gravity, lines 222-230 cite missing, 3rd worked example, ECI undefined) → second-author overwrite addressed AC-3/AC-5; Lead-direct fixed line 124→123 |
| `08_attitude_representations.md` | MAE | APPROVED | 1 | None — Hamilton product + q→DCM verified |
| `09_inertial_navigation.md` | MAE | APPROVED | 1 | None — specific-force convention + bias unit verified; dead-reckon arithmetic correct |
| `10_sensor_fusion_intuition.md` | MAE | APPROVED post-fix | 2 | 5 Errors on citation off-by-ones (`line 197` → `line 198` for h baro, etc.) → Lead-direct fix |
| `11_fsw_nav_mapping.md` | MAE | APPROVED post-fix | 2 | 2 Errors + 3 Warnings (state-machine miscite §8→design.md §5, 8-step recipe lines 113→174, kmat REV B name drift `Scale`→`Mult`, `QuatMul`→`HamProd`, `VEC_T`→`VEC`, `QUAT_T`→`QUAT`, `kPivotEpsilon` missing parens) → Lead-direct fix |
| `11_fsw_nav_mapping.md` | senior-software-engineer (holistic) | APPROVED post-fix | 1 | 4 BLOCKER + 1 MINOR; BLOCKERs are the same kmat REV B drift caught by MAE (already addressed Lead-direct); MINOR `Aligned (Navigating)` undocumented alias → Lead-direct fix |
| `12_exercises_and_solutions.md` | MAE | APPROVED | 1 | 2 cosmetic notes (not Warnings); accepted as-is per reviewer-bar drift lesson |

### Lead-Direct Iter-2 Edits Applied

- ch01: `chapter: 01 of 11` → `01 of 12` (frontmatter cosmetic).
- ch05: 3 citation sites updated with algorithm.md line numbers (§3.1 lines 70-82; §6 lines 351-357 + 359-363; §3.2/§4.1/§4.2 line ranges in FSW Anchor).
- ch07: `algorithm.md line 124` → `line 123` for gravity citation.
- ch10: 4 citation off-by-ones fixed (`line 197`→`198`, `198`→`199`, `199`→`201`); `line 325` for 10 Hz baro rate replaced with datasheet attribution.
- ch11: state-machine cite corrected (§8 lines 420-440 → design.md §5 lines 350-361 + 500-509); 8-step recipe range 113-144 → 113-174; `QuatMul`→`HamProd` (2 sites); `Scale`→`Mult` (2 sites); `kPivotEpsilon<double>`→`kPivotEpsilon<double>()` (3 sites); `VEC_T`/`QUAT_T`→`VEC`/`QUAT` (1 site); §9 `Aligned (Navigating)` → `Aligned` with note.

## 8. Reviewer Tooling Note (carry-forward)

Reviewer agent definitions were updated mid-sprint to add `WebFetch` + `WebSearch` per PM directive. However, both the index.md MAE reviewer and the ch08 MAE reviewer reported that those tools were NOT exposed to them at runtime. They corroborated bibliographic claims against training-data knowledge instead of live HTTP. Reference URLs (Trawny TR-2005-002, Solà arXiv:1711.02508, ISBNs, DOI for Kalman 1960) were verified against widely cataloged values consistent with `algorithm.md` §3.2's normative reference list. Live HTTP verification deferred. Action item for follow-up: investigate why WebFetch/WebSearch tool changes did not propagate to the spawned agent runtime, and whether a fresh agent invocation (post-tool-update) would have access.

## 9. Cross-Chapter Consistency Audit

- **Canonical notation** from `index.md` §4 used consistently across all 12 chapters (bold vectors/matrices, $\hat{x}_{k|k-1}$ / $\hat{x}_{k|k}$ posterior notation, $\mathcal{N}(\boldsymbol{\mu}, \boldsymbol{\Sigma})$, Hamilton-convention scalar-first quaternion, $\mathbf{g}^{NED} = (0, 0, +9.80665)$).
- **1D position+velocity model** numerical values in ch03 §3.6 and ch05 §1 verified identical post Lead-direct alignment.
- **Quaternion convention** (Hamilton, scalar-first, body→NED) triple-stated in ch08, used consistently in ch11.
- **kmat REV B symbol names** (`HamProd`, `Mult`, `VEC`, `QUAT`, `QuatRotate`, `QuatNormalize`, `kPivotEpsilon<T>()`) used in ch08, ch11. Note: `algorithm.md` and `design.md` still use REV A names (`QuatMul`, `Scale`, `VEC_T`, `QUAT_T`); chapters bridge with explicit "kmat REV B; algorithm.md still uses legacy names narratively" footnotes.

## 10. Build & Traceability Gates

- **G1 (POSIX build + tests):** N/A for this sprint — tutorial is documentation only; no source/test code changes.
- **G2 (`tools/traceability.py`):** PASS — exit 0 with 376 valid requirement IDs / 376 with test specs (Phase 0 baseline preserved; sprint added no new requirements/tests, only tutorial files).
- **G3 (Pico2 cross-compile):** N/A for this sprint — no source code changes.

## 11. Risk Register

- **SDP-R-08** (added in this sprint, SDP Revision C §7): USER-NAV-LIB and USER-NAV-APP are PM-owned out-of-band implementations. Downstream sprints (afm_lib -13, afm_app -20, telem_lib -14, telem_app -21, mlog_lib -15, mlog_app -22, sys_app -23) cannot start until PM signals nav_lib + nav_app delivery in writing. Mitigation: tutorial complete, PDR-baselined spec authoritative, same G1+G2+G3 gates apply at integration.
- **NAV-A3** (carry-forward from EKF amendment 2026-05-04): `docs/design/nav/design.md` (608 lines) and `algorithm.md` (511 lines) over the 500-line cap. Not blocking implementation; can be split opportunistically.
- **NAV-A1** (carry-forward): bidirectional `child_ids` on SW-REQ-SYS-013. Housekeeping; not blocking.

## 12. Approval

| Field | Value |
|-------|-------|
| Author | Software Lead |
| Date | 2026-05-08 |
| Predecessor | SDP Revision C amendment 2026-05-08 |
| Holistic MAE review (per chapter) | 13/13 APPROVED post-iteration; 4 chapters required iter-2 (Lead-direct fixes applied) |
| SSE-R holistic review (ch11) | APPROVED post-iteration (Lead-direct fix) |
| SDP MAE verification | APPROVED first iteration |
| Chief Engineer verdict | **PASS** (unconditional, 2026-05-08; all 8 ACs MET with concrete on-disk evidence per CE rationale) |
| Chair (PM) approval | **TBD** (awaiting PM presentation) |

## 13. Lessons Learned

(To be appended to `ai/memory/lessons-learned-software-lead.md` per the lessons-learned protocol.)

Key lessons surfaced during this sprint (full text in lessons-learned file):

1. **Cross-worker shared-state baselines need a Lead-controlled single source.** Two ch05 worker invocations (original + relaunch after socket-close) were briefed with subtly different numerical values (R=1.0 vs R=0.25, x_0=[0,0] vs [0,1]). Per the existing 2026-05-05 SPRINT-IMPL-02-retro lesson on cross-worker shared-state names, the same applies to shared-state numerical values: write a canonical-values block ONCE and paste verbatim. Cost of divergence: 1 Lead-direct alignment edit + 1 ch03 re-review iteration.

2. **Citation accuracy is the dominant tutorial review failure mode.** 4 of 4 chapter NEEDS-CHANGES verdicts (ch05, ch07, ch10, ch11) were citation-related: missing line numbers (ch05), off-by-one/two line numbers (ch07, ch10, ch11), miscited section purpose (ch11 §8 vs design.md §5). Worker briefs that direct line-citation discipline reduce this; for line ranges, briefs should specify "if your cite is more than 1 line off it will be flagged."

3. **Upstream document drift propagates downward.** Chapter 11 mirrored algorithm.md's REV A kmat names (`Scale`, `QuatMul`, `VEC_T`, `QUAT_T`) instead of kmat REV B's actual published names (`Mult`, `HamProd`, `VEC`, `QUAT`). The lessons-learned brief-prep grep extends naturally to kmat REV B verification: any worker citing `juno::kmat::*` symbols MUST grep against `libs/kmat_lib/include/kmat/kmat_api.hpp` (or `libjuno/include/juno/math/juno_math.hpp`) before pasting into the brief.

4. **Reviewer tool-set updates may not propagate to running agents.** Lead added `WebFetch` + `WebSearch` to MAE / SSE-R agent definitions mid-sprint per PM directive, but reviewers reported the tools were not actually exposed at runtime. Bibliographic verification fell back to training-data corroboration. Action: investigate runtime caching of agent tool sets; document the propagation timing in the agent infrastructure docs.

5. **Holistic SSE-R review on the FSW-mapping chapter caught BLOCKERs that the IEEE-lens MAE missed.** Per the 2026-05-03 delta-PDR lesson on holistic-vs-per-section reviewers, the SSE-R lens (technical correctness against actual published headers) caught 4 BLOCKERs (`Scale`, `QuatMul`, `VEC_T`, `QUAT_T`) that the per-section MAE flagged as Warnings. The two reviewers' findings overlapped (so I had already Lead-direct-fixed when the SSE-R returned), validating the parallel-review pattern: per-section + holistic catches strictly more than per-section alone.

## 14. Next Steps

1. **Chief Engineer final gate** (this sprint).
2. **PM presentation + sprint closure.**
3. **PM begins USER-NAV-LIB implementation** using the tutorial as a learning path and `docs/design/nav/algorithm.md` as authoritative spec. PM signals readiness in writing before each Wave 4 / 6+ downstream sprint opens (per SDP-R-08 mitigation).
4. **NAV-A3 file split** opportunistic; recommend bundling with any future amendment to `docs/design/nav/`.
5. **Reviewer-tool-propagation investigation** (carry-forward action item from §8).
