---
document_type: PDR Charter
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-02
status: Issued
---

# Juno FT1 Flight Software — Preliminary Design Review (PDR) Charter

## 1. Purpose and Scope

This charter convenes the Preliminary Design Review (PDR) for the Juno FT1 Flight Software (FSW). The PDR evaluates the L1 system design and the per-module L2 designs against the SW-REQ requirements baseline, the cross-module conventions, and the project constraints, and authorizes the program to proceed to Critical Design Review (CDR) and implementation upon Chair signature.

The PDR is a **design-level** review. It does not evaluate implementation source code, hardware-in-the-loop (HIL) test results, or post-flight artifacts; those gates belong to CDR and to the FT1 / FT2 / FIDR closure activities. Reviewers shall not raise findings that depend on the existence of `.cpp` / `.hpp` files not yet written.

### 1.1 In Scope

- Architecture and the MVC layering described in [architecture.md](../../../ai/memory/architecture.md).
- Requirements coverage across all SW-REQ modules under [docs/requirements/](../../requirements/).
- Interface definitions and bus message catalog in the L1 system design and the per-module L2 designs.
- Traceability between requirements, design sections, and test cases (`tools/traceability.py` baseline).
- Error and safety handling strategy, lifecycle (POST / Init / Run / Safe / Recovery), and TDM scheduling.
- Memory ownership rules and the POSIX / Pico2 build split.
- Simulation integration strategy (NASA Trick) for the POSIX build.
- All design documents under [docs/design/](../../design/), all requirement JSONs under [docs/requirements/](../../requirements/), and all test-case JSONs under [docs/test_cases/](../../test_cases/).

### 1.2 Out of Scope

- Implementation source code (`.cpp` / `.hpp`) not yet written — reserved for CDR.
- Hardware-in-the-loop test results — reserved for post-CDR verification.
- FT2 mission and any Flight Investigation / Discrepancy Report (FIDR) activity, per [SW-REQ-SYS-037](../../requirements/sys/requirements.json).

---

## 2. Authoritative Documents

The following documents constitute the review baseline. The Software Lead presents from these files; reviewers shall not invent or substitute content.

### 2.1 Conventions and Architecture

- [conventions.md](../../design/conventions.md) — cross-module vocabulary lock (phase enum, time base, frames, naming, scheduler period units). Authoritative for any cross-module name conflict.
- [architecture.md](../../../ai/memory/architecture.md) — MVC layering, LibJuno C++ module pattern, dependency injection at the composition root.
- [constraints.md](../../../ai/memory/constraints.md) — hard technical constraints (no heap, no exceptions, no RTTI, no virtual dispatch, C++11 freestanding, `-Werror`).

### 2.2 L1 System Design

- [system_design.md](../../design/system/system_design.md) — top-level FSW design covering composition, scheduler, bus catalog, lifecycle, error handling, memory ownership, and POSIX / Pico2 split. Addresses [SW-REQ-SYS-001](../../requirements/sys/requirements.json) through [SW-REQ-SYS-062](../../requirements/sys/requirements.json).

### 2.3 L2 Per-Module Designs

The 27 per-module L2 designs under [docs/design/](../../design/) are reviewed section-by-section as enumerated in §4 below.

### 2.4 Requirements Baseline

- 371 valid SW-REQ identifiers across all modules in [docs/requirements/](../../requirements/), confirmed by `tools/traceability.py`.
- 62 SW-REQ-SYS identifiers in [docs/requirements/sys/requirements.json](../../requirements/sys/requirements.json) form the L1 baseline; every per-module design decomposes from this set.

### 2.5 Test-Case Baseline

- All test-case JSONs under [docs/test_cases/](../../test_cases/) are reviewed for traceability to SW-REQ identifiers and for compliance with the schema described in [test-case-schema.md](../../../ai/memory/test-case-schema.md).
- Pre-flight measurement: 370 of 371 requirements carry a test specification (see §9).

### 2.6 Review Records

- [_template_section.md](_template_section.md) — section-record template the Chair uses to capture each section's RID/RFA list, dispositions, and verdict.
- [rid_rfa_log.md](rid_rfa_log.md) — master log indexing every RID and RFA across sections; authoritative for ID format, severity codes, source-reviewer codes, and disposition codes (see §6).

---

## 3. Review Board

| Role | Agent / Identifier | Responsibilities | Voting |
|------|-------------------|------------------|--------|
| Chair | Project Manager (the user) | Convenes the review, dispositions every RID and RFA, signs each section verdict, signs final convene and closure statements. | Yes |
| Mission Assurance Engineer (MAE) | `software-mission-assurance-engineer` agent | Reviews each section through the lens of IEEE 1016 and IEEE 29148 compliance, traceability completeness, requirements coverage, and test-case integrity. | Advisory |
| Senior Software Engineer Reviewer (SSE-R) | `senior-software-engineer` agent (reviewer mode) | Reviews each section through the lens of technical correctness, feasibility, C++11 / freestanding conformance, LibJuno module-pattern conformance, and interface coherence. | Advisory |
| Chief Engineer (CE) | `project-chief-engineer` agent | Reviews each section through the lens of overall system integrity, sprint acceptance-criteria alignment, cross-section coherence, and PDR exit-criteria progress. Issues final closure verdict in §7. | Advisory |
| Software Lead (Presenter, non-voting) | Orchestrator | Presents each section, consolidates RIDs and RFAs from the three reviewers into the section record, updates the master log, captures Chair dispositions, and authors the closure memo at §10. | No |

Reviewer findings are advisory; **only the Chair dispositions findings**. The Software Lead does not approve their own work. The Chief Engineer's verdict in [closure_memo.md](closure_memo.md) is required for PDR exit but does not substitute for the Chair's section signatures.

---

## 4. Section Plan

The PDR proceeds section-by-section in the order below. Each section convenes once; if a Major RID forces re-work, the affected section reconvenes after corrective action and is re-signed by the Chair.

| Section | Title | Documents Under Review |
|---------|-------|------------------------|
| S1 | High-Level Architecture and Conventions | [system_design.md](../../design/system/system_design.md), [conventions.md](../../design/conventions.md) |
| S2 | Foundation Libraries | L2 designs for `time_lib`, `log_lib`, `sch_lib`, `device_lib`, `kmat_lib` |
| S3 | Sensor Driver Libraries | L2 designs for `gps_lib`, `nmea_lib`, `imu_lib`, `baro_lib` |
| S4 | Comm and Storage Libraries | L2 designs for `lora_lib`, `sd_lib` |
| S5 | Domain Libraries | L2 designs for `nav_lib`, `afm_lib`, `telem_lib`, `mlog_lib` |
| S6 | Sensor Apps | L2 designs for `gps_app`, `imu_app`, `baro_app` |
| S7 | Domain Apps | L2 designs for `nav_app`, `afm_app`, `telem_app`, `mlog_app` |
| S8 | System App | L2 design for `sys_app` |
| S9 | Simulation Modules | L2 designs for `sim_dynamics`, `sim_sensors`, `sim_scenario`, `sim_harness` |
| S10 | PDR Closure | Chief Engineer closure verdict in [closure_memo.md](closure_memo.md); Chair signs final convene exit. |

Each section's record is filed at `docs/reviews/pdr/sections/S<N>_<short_name>.md` using the [_template_section.md](_template_section.md) template.

---

## 5. Per-Section Process

For sections S1 through S9 the flow is:

1. **Present.** The Software Lead presents the documents under review for the section, summarizing key decisions, requirements covered, interfaces, and any risks the Lead has already flagged. The presentation content is captured in §2 of the section record.
2. **Review.** The MAE, SSE-R, and CE run **in parallel** as independent agents, each producing a list of RIDs and RFAs scoped to their lens (see §3). Reviewers shall not coordinate findings; duplication across reviewers is acceptable and is consolidated downstream.
3. **Consolidate.** The Software Lead merges the three reviewer outputs into the section's RID and RFA tables (§3 and §4 of the section record), assigning IDs per the conventions in [rid_rfa_log.md](rid_rfa_log.md). Verbatim duplicates collapse to a single entry citing all source reviewers; substantive variants are recorded separately.
4. **Disposition.** The Chair reviews each RID and RFA in turn and renders a disposition (`ACCEPT`, `ACCEPT-MOD`, `REJECT`, `DEFER`, `CLOSE-NO-ACTION`). Disposition rationale is captured in §5 of the section record for any disposition other than a plain `ACCEPT` of an unmodified recommendation.
5. **Record.** The Software Lead transcribes each RID and RFA into the master log ([rid_rfa_log.md](rid_rfa_log.md)) and creates action items (§6 of the section record) for accepted findings that require follow-up work.
6. **Verdict.** The Chair selects one of the three verdicts in §7 of the section record (`CHAIR PROCEED`, `CHAIR HOLD`, `BLOCKED`) and signs the section. A `CHAIR HOLD` blocks PDR progression past the section; a `BLOCKED` verdict requires resolution of the listed external dependency before the review can resume.

For section S10, the flow is reduced: the Chief Engineer reviews the consolidated state of the master log and the action-item burndown, issues the closure verdict in [closure_memo.md](closure_memo.md), and the Chair countersigns.

---

## 6. RID and RFA Conventions

A **Review Item Discrepancy (RID)** documents a defect: a place where the design contradicts a requirement, a convention, a constraint, or a previously accepted decision. RIDs carry a severity (Major, Minor, Editorial) per [rid_rfa_log.md](rid_rfa_log.md). A **Request For Action (RFA)** documents an advisory action or investigation: a place where the reviewer believes follow-up work is warranted but no defect is alleged. RFAs are advisory and carry no severity.

This Charter does **not** restate ID format, severity codes, source-reviewer codes, or disposition codes. The master log [rid_rfa_log.md](rid_rfa_log.md) is the authoritative reference for that vocabulary, and any future edit to those conventions shall be made in that file. Reviewers and the Software Lead reference the master log when assigning IDs and dispositions.

---

## 7. Exit Criteria

The PDR is complete when **all four** of the following criteria are met. These are captured verbatim from the sprint acceptance criteria the Software Lead is operating under.

- All 10 section records exist and carry a Chair signature.
- Every RID and RFA in the master log has a disposition other than OPEN.
- All Major RIDs are either CLOSED or assigned a CDR-deferred action with explicit Chair approval.
- Chief Engineer issues a closure verdict in `closure_memo.md` of PASS, PASS-WITH-ACTIONS, or FAIL.

| # | Exit Criterion | Owner | Verification |
|---|---------------|-------|--------------|
| 1 | All 10 section records exist and carry a Chair signature. | Software Lead (filing); Chair (signing) | Inspection of `docs/reviews/pdr/sections/`. |
| 2 | Every RID and RFA in the master log has a disposition other than OPEN. | Chair (disposing); Software Lead (logging) | Inspection of [rid_rfa_log.md](rid_rfa_log.md) RID and RFA tables. |
| 3 | All Major RIDs are CLOSED or carry a CDR-deferred action with explicit Chair approval. | Chair (approving deferrals); Software Lead (tracking) | Inspection of master log and action-item table. |
| 4 | Chief Engineer issues a closure verdict in `closure_memo.md` of PASS, PASS-WITH-ACTIONS, or FAIL. | Chief Engineer (issuing); Chair (countersigning) | Inspection of [closure_memo.md](closure_memo.md). |

A FAIL verdict from the Chief Engineer reopens the PDR; the program does not proceed to CDR until the verdict is PASS or PASS-WITH-ACTIONS.

---

## 8. Roles and Responsibilities

### 8.1 Chair (Project Manager)

Convenes the PDR by signing this Charter; presides over each section's disposition phase; signs each section verdict; approves any CDR-deferral of a Major RID; countersigns the Chief Engineer's closure verdict. The Chair is the **only** role authorized to disposition findings.

### 8.2 Software Lead

Presents each section; consolidates parallel reviewer outputs into the section record; assigns RID and RFA identifiers per [rid_rfa_log.md](rid_rfa_log.md); transcribes each finding into the master log; tracks action items to closure; authors the section verdict line for the Chair to sign. The Software Lead is **non-voting** and does not approve their own work. Per [constraints.md](../../../ai/memory/constraints.md), the Software Lead escalates to the Chair after at most three feedback loops on a single finding.

### 8.3 Mission Assurance Engineer (MAE)

Reviews every section for IEEE 29148 (requirements) and IEEE 1016 (design) compliance, for traceability completeness across requirements and test cases, and for adherence to the requirements quality rules in [traceability.md](../../../ai/memory/traceability.md). Raises RIDs for missed coverage, broken trace links, and quality-rule violations.

### 8.4 Senior Software Engineer Reviewer (SSE-R)

Reviews every section for technical correctness, feasibility, and conformance with [constraints.md](../../../ai/memory/constraints.md) and the LibJuno C++ module pattern in [conventions.md](../../design/conventions.md) §1. Raises RIDs for design proposals that cannot be implemented within C++11 freestanding, that introduce dynamic allocation, that depend on virtual dispatch / RTTI / exceptions, or that violate cross-module API contracts.

### 8.5 Chief Engineer (CE)

Reviews every section for sprint acceptance-criteria alignment and cross-section coherence (e.g., whether a finding in S5 invalidates a disposition in S2). Owns the Cross-Section Re-Open Log in [rid_rfa_log.md](rid_rfa_log.md). Issues the final closure verdict in [closure_memo.md](closure_memo.md).

---

## 9. Tooling and Pre-Flight Gates

Before the PDR convenes, the Software Lead runs `tools/traceability.py` against the entire repository. The PDR shall not convene if the tool exits non-zero.

The recorded pre-flight result for this PDR is:

```
TRACEABILITY CHECK PASSED — Valid requirement IDs: 371; Requirements with test specs: 370
```

Reviewers may rely on these counts as authoritative for the PDR baseline. The single requirement without a test specification (`371 - 370 = 1`) is logged as a known item; reviewers may raise it as an RFA in S1 if appropriate, but it does not block convening.

The other traceability tools listed in [traceability.md](../../../ai/memory/traceability.md) (`requirements_search.py`, `rtm.py`, `burndown.py`) remain available to reviewers during the review for ad-hoc queries; their use is not required.

---

## 10. Lessons-Learned Mitigations

The following lessons from prior sprints have shaped the structure and execution of this PDR.

- **Cross-module API drift.** A prior sprint flagged that the AFM phase enum disagreed with the SYS phase enum because each module worker invented its own vocabulary. Mitigation: [conventions.md](../../design/conventions.md) was authored as a hard pre-fan-out vocabulary lock and is itself a document under review in S1. Reviewers shall raise an RID against any L2 design that paraphrases a §4 convention rather than referencing it verbatim.
- **Tool gates pre-validated.** A prior sprint discovered traceability breakage only at sprint close. Mitigation: `tools/traceability.py` is run before the PDR convenes (§9 above), and its result is recorded in this Charter rather than being asserted at closure.
- **PM escalation pattern for batched architectural decisions.** A prior sprint accumulated several architectural questions for the PM and resolved them in batches rather than one-at-a-time. Mitigation: section dispositions are **per-finding**, not per-batch, and the Chair signs each section before progression to avoid late-batch surprises.
- **Single `parent_id` is a hard constraint.** Reviewers historically flagged dual-parent rationale prose. Mitigation: every L2 design relies on the strongest parent link in JSON and captures secondary linkages as design narrative; reviewers shall not raise an RID against a missing JSON multi-parent field, but **shall** raise an RID against a design that lacks the secondary-linkage narrative where one is required by the brief.

---

## 11. Approval and Convene Statement

By signing below, the Chair convenes the Juno FT1 Flight Software Preliminary Design Review under the structure, process, and exit criteria defined in this Charter. Reviewers and the Software Lead shall execute Sections S1 through S10 in order, against the documents listed in §2 and enumerated per section in §4.

This Charter is Revision A and takes effect on the date below. Subsequent revisions, if any, are filed in this same path with an incremented revision letter and a new effective date.

**Chair Signature:** ____________________________________________

**Name:** Project Manager, Juno FT1

**Date:** ______________________

