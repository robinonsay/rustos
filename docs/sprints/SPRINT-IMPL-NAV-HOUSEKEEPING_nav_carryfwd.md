---
document_type: Sprint Closure Record
sprint_id: SPRINT-IMPL-NAV-HOUSEKEEPING
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
predecessor: SPRINT-IMPL-NAV-TUTORIAL closure 2026-05-08
successor: PM-owned USER-NAV-LIB implementation (out-of-band)
status: Closed
---

# Sprint Closure Record — SPRINT-IMPL-NAV-HOUSEKEEPING (NAV-A1 + NAV-A3)

## 1. Sprint Goal

Discharge the two near-term `NAV-A*` carry-forward action items left open by the EKF amendment + implementation-readiness remediation (2026-05-04) and the SPRINT-IMPL-NAV-TUTORIAL closure (2026-05-08):

- **NAV-A1**: bidirectional `child_ids` on `SW-REQ-SYS-013` (housekeeping; was target ≤2026-05-10).
- **NAV-A3**: file split / trim — `docs/design/nav/design.md` was 608 lines (108 over the 500-cap from `ai/memory/constraints.md`), and `docs/design/nav/algorithm.md` was 511 lines (11 over). Both needed to land under cap before nav_lib v1.0 freeze.

NAV-A4, NAV-A5, NAV-A6 deferred per the sprint plan (NAV-A4 needs a new dedicated nav-lifecycle parent requirement, which is amendment-shaped not re-pointing-shaped; NAV-A5/A6 are FT2-only).

The reviewer-tool-propagation issue surfaced in SPRINT-IMPL-NAV-TUTORIAL §8 of its closure record is logged separately as agent-infrastructure follow-up; not in this sprint.

## 2. Predecessor

- **SPRINT-IMPL-NAV-TUTORIAL closure 2026-05-08** (CE PASS unconditional). Its closure record §11 enumerated NAV-A1 and NAV-A3 as the open carry-forwards in scope for follow-on housekeeping.
- **EKF amendment + implementation-readiness remediation closure 2026-05-04** (CE iteration-2 PASS): originated NAV-A1 through NAV-A6. NAV-A1 and NAV-A3 are the items targeted by this sprint.

## 3. Scope

**In scope (delivered):**
- NAV-A1: `SW-REQ-SYS-013.child_ids` updated to include `SW-REQ-NAV-018` and `SW-REQ-NAV-APP-014`.
- NAV-A3: `docs/design/nav/design.md` reduced from 608 to 325 lines via §5-§11 relocation to a new sister file `docs/design/nav/contracts.md` (322 lines); `docs/design/nav/algorithm.md` reduced from 511 to 480 lines via banner-compression and prose-tightening.
- `docs/design/nav/index.md` updated to reflect the three-file split with refreshed Files table, How-to-read guidance, and Requirements-covered note.
- Cross-reference updates: 7 sites in `algorithm.md` pointing to `design.md §5/§8/§9/§10/§11` re-targeted to `contracts.md`; 2 sites in `docs/tutorials/nav_kalman/11_fsw_nav_mapping.md` (the FSW-mapping chapter) re-targeted from `design.md §5` to `contracts.md §5/§9`.
- This closure record.

**Out of scope (deferred):**
- NAV-A4 (re-point `SW-REQ-NAV-019` parent): requires a new dedicated nav-lifecycle/init parent requirement first; amendment-shaped.
- NAV-A5 (`NAV_APP_INIT_T` settling-window override): FT2-only.
- NAV-A6 (dead-reckoning timeout configurability): FT2-only.
- Reviewer tool-set runtime propagation investigation: agent-infrastructure track.

## 4. Acceptance Criteria Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | `docs/design/nav/design.md` ≤480 lines (≤500 hard) | MET | 325 lines (was 608) |
| AC-2 | `docs/design/nav/algorithm.md` ≤480 lines | MET | 480 lines (was 511) |
| AC-3 | New sister file `docs/design/nav/contracts.md` ≤480 lines, all design tags `<!-- @{"design": [...]} -->` preserved verbatim | MET | 322 lines; 6 design tags relocated byte-identically per MAE byte-identity check (§5/§6/§7/§8/§9/§10; §11 has no tag in source) |
| AC-4 | `docs/design/nav/index.md` TOC updated to reference `contracts.md` | MET | three-file Files table with scope-per-file, How-to-read updated, Requirements-covered note refreshed |
| AC-5 | `SW-REQ-SYS-013.child_ids` includes `SW-REQ-NAV-018` AND `SW-REQ-NAV-APP-014` | MET | `jq` confirms `["SW-REQ-NAV-018", "SW-REQ-NAV-APP-014"]` |
| AC-6 | `tools/traceability.py` exits 0 | MET | 376/376 PASS |
| AC-7 | Sprint closure record at `docs/sprints/SPRINT-IMPL-NAV-HOUSEKEEPING_nav_carryfwd.md` | MET | this file |

## 5. Final Inventory

| File | Before | After | Δ |
|------|--------|-------|---|
| `docs/design/nav/design.md` | 608 | 325 | −283 |
| `docs/design/nav/algorithm.md` | 511 | 480 | −31 |
| `docs/design/nav/contracts.md` (NEW) | 0 | 322 | +322 |
| `docs/design/nav/index.md` | 85 | 86 | +1 |
| `docs/requirements/sys/requirements.json` | (unchanged size) | (unchanged size) | 0 (1 field updated) |
| **Total nav L2 design** | 1204 | 1213 | +9 (~1% growth from new front-matter + see-also footers; net win is every file ≤500) |

## 6. Worker / Reviewer Summary

### Phase 0 — Pre-flight (Lead-direct, ~5 min)

- Identified split boundary in `design.md` at line 320 (§4 ends, §5 begins).
- Confirmed traceability baseline 376/376 PASS pre-sprint.

### Phase 1 — Workers in parallel + Lead-direct

| Item | Worker | Output | Reviewer | Verdict |
|------|--------|--------|----------|---------|
| W-1 | software-systems-engineer | `docs/design/nav/contracts.md` (NEW, 322 lines, byte-identical relocation of design.md §5-§11 + new front-matter) | software-mission-assurance-engineer | APPROVED first iteration; byte-identity verified per-section against source range |
| W-2 | software-systems-engineer | `docs/design/nav/algorithm.md` (511 → 480 lines via banner-compression + prose-tighten across §1 intro, §3.2 lead, §3.2 closing, §5.1 closing, §7 Implementation Notes, §10 Memory Ownership delta, §11 leading paragraph) | software-mission-assurance-engineer | APPROVED first iteration; all 11 sections + design tag + 8-step process model + NAV_INIT_T 10-row table + reference-σ 8-row table + Groves+Trawny normative refs + Trawny URL preserved |
| W-3 (Lead-direct) | Software Lead | `docs/requirements/sys/requirements.json` SW-REQ-SYS-013 `child_ids` field | n/a (G2 traceability check) | confirmed via jq + traceability.py PASS |

### Phase 1.5 — Lead-direct atomic transition

- Truncated `docs/design/nav/design.md` from 608 to 325 lines (kept §1-§4, added "See also" footer pointing to contracts.md and algorithm.md).
- Updated `docs/design/nav/index.md` Files table and How-to-read guidance to reflect the three-file split.
- Updated 7 cross-references in `docs/design/nav/algorithm.md` (lines 98, 231, 294, 448, 450, 456, 465) from `design.md §X` to `contracts.md §X` where X ∈ {5, 8, 9, 10, 11}.
- Updated 2 cross-references in `docs/tutorials/nav_kalman/11_fsw_nav_mapping.md` (the FSW-mapping chapter) from `design.md §5` to `contracts.md §5/§9`.
- One Lead self-catch: an initial ch11 edit accidentally wrote `design.md §9` for divergence-handling; corrected Lead-direct to `contracts.md §9` after the residual-stale-reference grep audit. Lesson: even routine cross-reference renames need a post-edit grep before declaring done.

## 7. Build & Traceability Gates

- **G1 (POSIX build + tests):** N/A (documentation-only sprint).
- **G2 (`tools/traceability.py`):** PASS — exit 0 with 376 valid requirement IDs / 376 with test specs, identical to the pre-sprint baseline. No requirement coverage drift.
- **G3 (Pico2 cross-compile):** N/A (no source code changes).

## 8. Risk Register Update

- **NAV-A1**: CLOSED.
- **NAV-A3**: CLOSED.
- **NAV-A4**: still open; opportunistic; FT1 not blocking.
- **NAV-A5**: still open; FT2-only.
- **NAV-A6**: still open; FT2-only.
- **SDP-R-08**: still open; PM-owned USER-NAV-LIB and USER-NAV-APP implementations unchanged; this sprint preserved the spec artifacts the PM is implementing against.
- **Reviewer-tool-propagation**: still open; tracked separately as agent-infrastructure follow-up.

## 9. Approval

| Field | Value |
|-------|-------|
| Author | Software Lead |
| Date | 2026-05-08 |
| Predecessor | SPRINT-IMPL-NAV-TUTORIAL closure 2026-05-08 |
| W-1 MAE review | APPROVED first iteration (byte-identity check passed) |
| W-2 MAE review | APPROVED first iteration (all required content preserved) |
| W-3 Lead-direct verification | confirmed (jq + traceability.py PASS) |
| Chief Engineer verdict | **PASS unconditional** (2026-05-08; all 7 ACs MET; zero blocking findings; benign single grep false-positive on `system_design.md` substring documented and dismissed) |
| Chair (PM) approval | **TBD** (post-CE) |

## 10. Lessons Learned (preliminary; full text appended to `ai/memory/lessons-learned-software-lead.md` post-CE)

1. **Verbatim-relocation refactor pattern works for design-doc splits, not just code-file splits.** Per the 2026-05-06 SPRINT-IMPL-07-retro lesson, the verbatim-relocation + Lead-direct atomic transition pattern was developed for splitting C++ source files. It applies cleanly to markdown design-doc splits: worker authors NEW sister file with verbatim content, MAE diffs against source range, Lead-direct removes from original + updates index. Total agent invocations: 2 workers + 2 reviewers = 4 (plus Lead-direct + CE = 6 total). For documentation, the byte-identity check is also faster than for code (no semantic-equivalence concerns).

2. **Cross-reference updates need a post-edit grep audit.** The Lead's initial ch11 edit replaced `design.md §5 lines 350-361 (alignment criteria) and 500-509 (divergence)` with `contracts.md §5 (alignment criteria) and design.md §9 + contracts.md §5` — the `design.md §9` was a typo from incomplete substitution (§9 was relocated to contracts.md alongside §5). Caught only by the post-edit `grep -nE 'design\.md.*§(5|6|7|8|9|10|11)'` audit. Generalize the 2026-05-03 atomic-Lead-edit lesson: after any cross-reference rename across multiple files, a final `grep` for the OLD pattern is mandatory; the renaming is not done until grep returns zero (or only intentional historical refs).

3. **Historical carry-forward narratives may legitimately reference old document structure.** The `docs/design/nav/index.md` Approval section (lines 39-86) contains historical narratives from 2026-05-03 and 2026-05-04 PDR/EKF/remediation sprints that reference old design.md §5/§9/etc. line numbers. Those references are correct AS HISTORICAL RECORDS at the time those sprints closed. Do NOT update them to reflect post-2026-05-08 file structure — that would falsify the closure record. Only update LIVE cross-references (sister-file refs in algorithm.md, tutorial chapters consuming the design as authoritative spec). Distinguishing live refs from historical refs is a manual-judgment call during the residual-grep sweep.

## 11. Next Steps

1. Chief Engineer final gate.
2. PM presentation + sprint closure.
3. PM continues USER-NAV-LIB implementation against the now-clean three-file nav L2 design.
4. NAV-A4 / NAV-A5 / NAV-A6 stay open; revisit FT2 (or earlier if a related amendment opens the relevant scope).
5. Reviewer-tool-propagation investigation: track separately as an agent-infrastructure work item.
