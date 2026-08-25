# Juno FT1 PDR — Section S1: High-Level Architecture & Conventions

## 1. Header

| Field | Value |
|-------|-------|
| Section Number | S1 |
| Section Title | High-Level Architecture & Conventions |
| Date Convened | 2026-05-02 |
| Chair | Project Manager |
| Software Lead (Presenter) | Software Lead (orchestrator) |
| Attendees | Chair, MAE (`software-mission-assurance-engineer`), SSE-R (`senior-software-engineer`), CE (`project-chief-engineer`), Software Lead (non-voting) |

### Documents Under Review

- [docs/design/system/system_design.md](../../../design/system/system_design.md) (496 lines)
- [docs/design/conventions.md](../../../design/conventions.md) (308 lines)

Authoritative baselines referenced (not under review): `ai/memory/architecture.md`, `ai/memory/constraints.md`, `ai/memory/coding-standards.md`, `docs/requirements/sys/requirements.json` (62 SYS reqs).

## 2. Section Summary

**Key Decisions Presented**

- D-1: MVC layering — App = View, Lib = Controller, Broker = Model.
- D-2: LibJuno C++ module pattern for every lib (vtable DI, no virtual).
- D-3: Composition root is `apps/main.cpp` with per-platform aliasing.
- D-4: Static TDM schedule, 5 ms base tick, 1000 ms hyperperiod, 537 invocations/sec.
- D-5: Bus message catalog of 11 message types, all POD with leading `tTimestampUs`.
- D-6: Lifecycle POST → Init → Run → Safe ↔ Run → Recovery; no FSW-initiated reboot.
- D-7: Phase enum locked to {PRE_LAUNCH, BOOST, APOGEE, DESCENT, LANDING}; no COAST, no LANDED.
- D-8: Time base = monotonic `uint64_t` µs; type alias `JUNO_TIME_US_T`.
- D-9: Frames — geodetic position, HAE altitude, NED velocity, body→NED quaternion, body X-fwd/Y-right/Z-down, SI units; ECEF excluded.
- D-10: Memory ownership — caller-owned, static/`.bss`, `BlockAlloc<T,N>` for pools, zero heap.
- D-11: POSIX/Pico2 functional equivalence via identical composition graph.
- D-12: Trick SITL via `pfcnTimeProvider` callback injection.
- D-13: Error handling — all `noexcept`, diagnostic-only failure handlers, per-sensor health bit.
- D-14: POST result downlinked once at boot; health bitmap published 10 Hz by `sys_app`.

**Key Requirements Covered**

- All 62 `SW-REQ-SYS-001` … `SW-REQ-SYS-062` (the L1 system baseline).
- Coverage audit: every ID has at least one in-section `<!-- @{"design": [...]} -->` tag in `system_design.md` (verified by MAE).

**Key Interfaces Covered**

- 11 bus message types: `JUNO_MSG_IMU_SAMPLE_T`, `_BARO_SAMPLE_T`, `_GPS_FIX_T`, `_GPS_UTC_T`, `_GPS_NMEA_RAW_T`, `_NAV_STATE_T`, `_AFM_PHASE_T`, `_SYS_HEALTH_T`, `_SYS_POST_T`, `_TELEM_PACKET_T`, `_MLOG_RECORD_T`.
- Composition root order (7-step rule set in §8.1).
- TDM scheduler offset table (8 apps, periods 5/10/50/100/200/500 ms).
- `JUNO_TIME_PROVIDER_T` callback for sim-time injection.

**Risks and Open Issues Flagged by Lead** (carried into review)

- L-F1: SYS-016 lists 4 phases vs AFM-002's 5 (`pre-launch` mismatch).
- L-F2: Trick wallclock vs monotonic-µs equivalence — handled via `pfcnTimeProvider`.
- L-F3: Apps subscribe at `Init()`, broker must exist first.
- L-F4: `JUNO_MSG_MLOG_RECORD_T` is sink-only.
- L-F5: WCET claim "fits in 5 ms with margin" unmeasured.
- L-F6: POSIX/Pico2 equivalence has no integration-test artifact yet.
- L-F7: SYS-014 numeric threshold deferred to nav L2.

## 3. RID List

ID format: `PDR-RID-S1-NNN`. Numbered sequentially across all sources.

### 3.1 MAE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S1-001 | Minor | §11 traceability table cites sections that lack matching design tags | Several rows in §11 attribute coverage to a section whose `<!-- @{"design": [...]} -->` tag does not list that requirement, contradicting the conventions §8 rule that per-section tags are authoritative. Examples: SYS-004 → §1,§9 but §9 tag lacks 004; SYS-016 → §7.3,§8 but §8 tag lacks 016; SYS-017 → §3,§4,§8 but §8 tag lacks 017; SYS-018 → §7.3,§8 but §8 tag lacks 018; SYS-023 → §4,§6 but §4 tag lacks 023. | Either add missing IDs to the cited sections' design tags, or correct the §11 table to list only sections that carry an authoritative tag for each requirement. | OPEN | — | — | OPEN |
| PDR-RID-S1-002 | Minor | §4 bus catalog row for `JUNO_MSG_MLOG_RECORD_T` is internally inconsistent | Row 11 lists publisher `mlog_app` with "n/a (sink)" subscribers while the trailing note states "mlog is a terminal sink, not a publisher to other apps." If the record is sink-only it does not belong in the cross-app bus catalog. §7.3 sequence diagram further muddies this by showing `mlog_app->>broker: Publish/Persist(MLOG_RECORD_T)`. | Remove `JUNO_MSG_MLOG_RECORD_T` from the §4 bus catalog and document it instead in §6 as an SD-card persistence record kind owned by `mlog_app`. Correct the §7.3 arrow to "Persist" only (no broker publish). | OPEN | — | — | OPEN |
| PDR-RID-S1-003 | Minor | §3.3 module catalog header paths diverge from architecture.md | §3.3 lists app header paths as `apps/<name>_app/include/<name>_app/<name>_app.hpp`. `architecture.md` Directory Structure specifies `apps/<name>_app/include/<name>/<name>_app.hpp` (single `<name>` subdirectory, no `_app` suffix). The L1 design states "Header paths are mandatory" — elevating divergence to a binding rule that contradicts architecture.md. | Pick one form and align both documents. Recommended: keep architecture.md's `include/<name>/<name>_app.hpp` form, update §3.3 accordingly. | OPEN | — | — | OPEN |
| PDR-RID-S1-004 | Minor | §5 state-diagram shows Recovery as a peer state contradicting prose | §5 prose: "The Recovery sub-state is a function of AFM phase = `JUNO_PHASE_LANDING`... not a separate scheduler mode." The Mermaid `stateDiagram-v2` immediately above renders Recovery as a top-level node with explicit `Run --> Recovery` and `Safe --> Recovery` transitions and a self-loop, identical in graphical weight to Run and Safe. | Either model Recovery as an annotation on the LANDING-phase observable (and remove from state diagram), or remove the prose claim that Recovery is "not a separate scheduler mode." Pick one and apply consistently across §5 and §11. | OPEN | — | — | OPEN |

### 3.2 SSE-R Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S1-005 | **Major** | mlog@10 ms cannot satisfy SW-REQ-SYS-011 full-rate IMU logging | §3.3 sets `kMlogAppPeriodMs = 10`. IMU publishes at 5 ms. §10.1 defines subscriber message buffers as a single-slot, latest-value store; §6 confirms "broker copies on publish, subscriber sees its own immutable view" — single-slot. At each 10 ms mlog tick, the mlog-side IMU buffer holds only the most-recent IMU sample; the sample published 5 ms earlier has been overwritten. mlog therefore discards 1 of every 2 IMU samples, directly violating SW-REQ-SYS-011 (No Sensor Downsampling for Logging). The §3.3 rationale ("drains the bus before the next nav tick") is orthogonal to publisher rate. | Pick one and state explicitly: (A) Change `kMlogAppPeriodMs` to 5 ms, co-schedule mlog with imu_app; update §3.3 rationale and §8.2 offset table. (B) Add a per-subscriber ring buffer in the broker for IMU messages (redesigns §10.1 buffer ownership; cost: N-slot static array per subscription). (C) Have imu_app double-buffer its mlog output (`JUNO_MSG_IMU_BATCH_T` variant; updates §4 catalog). Option A is simplest and preserves the broker model. | OPEN | — | — | OPEN |
| PDR-RID-S1-006 | Minor | §8.1 pseudocode demonstrates bare `.tOk` without `JUNO_ASSERT_OK` | `juno::time::TIME_LIB_IMPL_T tTime = ...::New(...).tOk;` — `.tOk` is accessed without checking `RESULT_T::tStatus`. This pattern silently accepts a zeroed/invalid `_IMPL_T` if `New()` fails, contradicting `conventions.md` §4.3: "callers use `JUNO_ASSERT_*`; bare `if`-return is a review failure." Because this pseudocode is the exemplar for 27 L2 designs, it propagates a non-compliant idiom downstream. | Replace bare `.tOk` with the correct status-checking idiom. For composition root context (where failure marks POST bitmap and continues per §9.7), use: `RESULT_T<TIME_LIB_IMPL_T> tTimeResult = TIME_LIB_IMPL_T::New(...);` then check `tTimeResult.tStatus` before using `.tOk`. Add an exemplar comment showing POST-bitmap handling. | OPEN | — | — | OPEN |
| PDR-RID-S1-007 | Minor | `JUNO_MSG_GPS_NMEA_RAW_T.acSentence[N]` array dimension unspecified | §4 catalog entry lists fields as `tTimestampUs, acSentence[N], zLen` with N unspecified. All bus messages must be POD aggregates with zero constructors; an unbound dimension cannot be instantiated as a POD member of mlog_app. Static memory budget at composition root (§10.1) is incomplete. NMEA standard sentence cap is 82 printable + CRLF = 84 bytes. | Lock the maximum at L1. Add `static constexpr size_t kNmeaMaxSentenceBytes = 84;` to `conventions.md` §4.4 or `system_design.md` §4. Change catalog entry to `acSentence[kNmeaMaxSentenceBytes]`. | OPEN | — | — | OPEN |
| PDR-RID-S1-008 | Minor | `JUNO_TIME_PROVIDER_T` named in §8.1 but type signature undefined | §8.1 PM-Decision comment names the type alias but neither system_design nor conventions §4.2 defines its C++ type signature (return type, parameter list, noexcept). Without `using JUNO_TIME_PROVIDER_T = JUNO_TIME_US_T (*)(JUNO_USER_DATA_T*) noexcept;` (or similar), `time_lib`, `sim_harness`, and every module accepting `time_lib::ROOT_T &` cannot complete §4 or §9. | Add formal type alias to `conventions.md` §4.2: return type `JUNO_TIME_US_T`, parameter list (at minimum `void`, ideally `JUNO_USER_DATA_T *pvUserData` for Trick context), `noexcept` qualifier. Reference from system_design §8.1 and time_lib L2. | OPEN | — | — | OPEN |
| PDR-RID-S1-009 | Minor | §8.1 step 5 prose names `&log` but pseudocode passes `&time` | Step 5 prose: "Apps `Init(&libRoot, &broker, &log)` — DI by reference." Pseudocode: `tImuApp.Init(tImuLib.tRoot, tBus.tRoot, tTime.tRoot)` — passes time, not log. Because imu_app timestamps samples (SYS-027), `time_lib` injection is correct and necessary; it is omitted from step 5 prose. L2 app designers using step 5 as canonical DI will either add an unnecessary `log_lib` parameter or miss `time_lib`. | Reconcile prose with pseudocode. Recommended: "Apps `Init(&libRoot, &broker, &time)` — DI by reference; some apps also receive `&log` if they write diagnostic records directly," or note step 5 is schematic and direct readers to each app's L2 §4 for the authoritative signature. `time_lib` must appear explicitly in the example. | OPEN | — | — | OPEN |
| PDR-RID-S1-010 | Editorial | §8.1 step 4 placement implies broker is constructed after domain libs | Step 4 ("Broker is constructed before any app `Init()`") appears after step 3 (domain libs), suggesting broker is fourth in time. The Mermaid flowchart shows broker as a parallel root-level dependency. Reading ambiguity between sequencing rule vs. invariant. No bug today (broker has no lib deps), but wording departs from flowchart intent. | Rephrase step 4 as an invariant: move broker construction to step 1 or add a parenthetical noting step 4 is positional-independent. Clarify the 7 rules are invariants, not strict time-ordering; flowchart is authoritative for sequencing. | OPEN | — | — | OPEN |

### 3.3 CE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S1-011 | **Major** | SW-REQ-SYS-016 phase-set is narrower than `JUNO_PHASE_T` enum | `system_design.md` §4 (`JUNO_MSG_AFM_PHASE_T`) and §5 ("Run → Recovery: AFM phase = LANDING") rely on the 5-value `JUNO_PHASE_T` from `conventions.md` §4.1. The parent requirement SYS-016 only requires detection of `boost, apogee, descent, landing` — `pre-launch` is not a detected phase. `conventions.md` FLAG-2 notes this but takes the "AFM-002 wins" position without a Chair disposition. The system L1 has effectively adopted a 5-value enum that one of its parent requirements does not list. | Either (a) Chair amends SW-REQ-SYS-016 to include `pre-launch` as the at-power-on initial phase (with rationale that pre-launch is the initial state, not a "detected" phase), or (b) `conventions.md` §4.1 records a Chair-signed disposition that the 5-value enum is canonical while SYS-016's wording is intentional. Preferred: (a). | OPEN | — | — | OPEN |
| PDR-RID-S1-012 | Minor | `JUNO_TIME_PROVIDER_T` injection seam absent from `conventions.md` | system_design §8.1 introduces the `pfcnTimeProvider` callback as the canonical Trick injection point. `conventions.md` §4.2 (Time base) and §6 (POSIX vs Pico2) document clock sources but make no mention of `JUNO_TIME_PROVIDER_T` as a vocabulary item. Risk: any L2 author touching time integration reads §4.2 and §6 of conventions and does not see the seam — only documented at system-design layer and time-lib L2. | Add a single bullet under `conventions.md` §4.2 (or §6) stating that `JUNO_TIME_PROVIDER_T` is the canonical sim-time injection seam, pointing to `time/design.md` for the full contract. Editorial-scope amendment; no L2 changes. | OPEN | — | — | OPEN |
| PDR-RID-S1-013 | Minor | "Apps subscribe at `Init()`" rule appears only in system_design §8.1 | system_design §8.1 rule 4 is the only place this ordering rule is captured. `conventions.md` §7 IEEE-1016 structure does not call this out as a §3/§7 expectation; §5 (memory ownership) addresses ownership but not subscription timing. Per-module L2 designs that publish/subscribe must follow this rule but the single source of truth is the system design, not the conventions doc. | Add the subscribe-at-`Init()` rule to `conventions.md` §1.3 (Mandatory rules) or §5 so per-module L2 designers see it without reading system_design §8.1. Editorial scope. | OPEN | — | — | OPEN |

## 4. RFA List

ID format: `PDR-RFA-S1-NNN`. RFAs do not carry a severity field.

### 4.1 MAE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S1-001 | WCET claim "fits in 5 ms with margin" carries no measurement basis | §8.2 asserts the worst-case 5 ms tick (8 apps co-dispatched at t=0) "fits in 5 ms with margin" without measurement, analytical bound, or per-module sub-budget allocation. Per Charter §1.2, code-level WCET measurement is out of PDR scope, but the claim has no traceable basis. | Add a sub-budget table to §8.2 (per-app µs allocations as engineering estimates) with forward reference indicating each per-module L2 §8 owns enforcement; or downgrade wording to "expected to fit; per-module L2 designs hold sub-budgets, measured WCET is a CDR deliverable." | OPEN | — | — | OPEN |
| PDR-RFA-S1-002 | POSIX/Pico2 functional-equivalence verification artifact not specified | §10.2 and §11 assert equivalence (SW-REQ-SYS-043) is exercised by Trick SITL feeding the same `*_ROOT_T` API, but neither identifies a concrete pass/fail criterion (e.g., a delta tolerance on logged outputs between POSIX and Pico2 builds running an identical scenario). | Add a one-paragraph criterion to §10.2 (e.g., "Equivalence is met when both builds produce bit-identical NAV_STATE and AFM_PHASE outputs given identical input recording") deferring the test artifact itself to CDR. | OPEN | — | — | OPEN |
| PDR-RFA-S1-003 | Trick injection callback semantic contract not stated | §8.1 pseudocode comments name `pfcnTimeProvider` but do not state the contract: callback must be monotonic non-decreasing, µs resolution, and is the sole time source seen by all libs in that build. | Promote the injection contract from a pseudocode comment to a §10.2 paragraph stating: callback must be monotonic non-decreasing, µs resolution, and is the sole time source seen by all libs in that build. | OPEN | — | — | OPEN |
| PDR-RFA-S1-004 | §3 over-tags SW-REQ-SYS-043 (POSIX/Pico2 equivalence) at the System Overview level | §3's design tag includes SW-REQ-SYS-043, but §3 only mentions "POSIX + Pico2 impls" parenthetically in the MVC table. Substantive coverage of -043 lives in §10.2 and §11. The over-tag is harmless but could mislead RTM tooling. | Remove SW-REQ-SYS-043 from §3's design tag; coverage by §10.2 and §11 is sufficient. | OPEN | — | — | OPEN |
| PDR-RFA-S1-005 | SYS-016 vs AFM-002 phase-set disagreement (PM action) | conventions FLAG-2 documents that SYS-016 enumerates 4 phases while AFM-002 enumerates 5. system_design inherits AFM-002 — internally consistent — but the underlying SYS-016 wording is unchanged, leaving the requirements baseline asymmetric. (Related to RID-S1-011.) | Confirm with Chair whether SYS-016 should be amended to mention pre-launch as the at-power-on initial state, or whether the 4-phase wording is intentional. Action belongs in the requirements baseline. | OPEN | — | — | OPEN |
| PDR-RFA-S1-006 | §8.1 pseudocode contains project-history annotation | Pseudocode comment block embeds sprint/decision metadata ("PM Decision 2 (sprint 2026-05-02)") that does not belong in an authoritative IEEE 1016 design document; belongs in commit messages or a decision log. | Replace the comment with a citation to `conventions.md` §4.2 (time base) and/or the relevant SW-REQ ID; move PM-decision provenance to a separate decisions log. | OPEN | — | — | OPEN |

### 4.2 SSE-R RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S1-007 | Trick sensor-injection seam not sketched at L1 | §10.2 and conventions §6 assert POSIX driver impls "replace hardware I/O with Trick variables" but neither specifies the mechanism: shared memory, FIFO, Trick `extern`, or callback. Without an L1 sketch, sim_sensors L2 (S9) and POSIX driver L2s (S3) cannot design consistently against the freestanding constraint. | Add a paragraph to §10.2 (or §10.3) naming the injection mechanism — e.g., "POSIX driver impls accept an optional data-provider callback (analogous to `pfcnTimeProvider`) through which `sim_sensors` writes simulated readings" — and noting which files are POSIX-only (not compiled under PLATFORM_PICO2). | OPEN | — | — | OPEN |
| PDR-RFA-S1-008 | System lifecycle state enum (`JUNO_FSW_STATE_T`) undefined at L1 | §5 describes the system lifecycle (POST, Init, Run, Safe, Recovery) as a state machine. No C++ enum type is defined for it. `sys_app` must track current lifecycle state. Without a locked enum, sys_app L2 will invent state-type names — risking the same drift `conventions.md` was created to prevent. `JUNO_PHASE_T` is locked; analogous `JUNO_FSW_STATE_T` is absent. | Add `enum class JUNO_FSW_STATE_T : uint8_t { JUNO_FSW_STATE_POST, _INIT, _RUN, _SAFE, _RECOVERY };` to `conventions.md` §4. Reference from `system_design.md` §5. Prevents drift in S8 sys_app L2. | OPEN | — | — | OPEN |
| PDR-RFA-S1-009 | Health bitmap bit assignments not specified at L1 | §4 defines `JUNO_MSG_SYS_HEALTH_T.u32HealthBitmap` and "per-sensor flags." §9.3 assigns responsibility per driver/app. No bit-position table exists. Telem packet (SYS-020) and mlog binary record (SYS-023) require bitmap encoding parseable on the ground. Independent L2 bit assignments → ground-segment collision. | Make `sys_app` L2 §4 the authoritative location for the bitmap bit-assignment table, with all sensor owners cross-referencing it. Alternatively, promote the table to `system_design.md` §4 under `JUNO_MSG_SYS_HEALTH_T` even if exact values are TBD. | OPEN | — | — | OPEN |
| PDR-RFA-S1-010 | `NAV_STATE_T.bValid` single flag conflates two distinct failure modes | §9.9: `bValid=false` published when nav-vs-GPS bound exceeded. §9.5 + SYS-059: `bValid` cleared on missing/degraded inputs. A single boolean cannot distinguish "no input" vs "converged but diverging from GPS" — afm_app gating may legitimately differ between modes. | Have nav_app L2 §4 evaluate whether an `eNavValidityReason` sub-field is needed alongside `bValid`, or explicitly confirm that all downstream consumers treat both modes identically (in which case add a statement to §9.9). | OPEN | — | — | OPEN |
| PDR-RFA-S1-011 | SYS-016 phase-text 4 vs AFM-002 5 (RTM impact) | Same root as RID-S1-011 / RFA-S1-005 but viewed through the future-RTM lens: code implementing PRE_LAUNCH handling will not trace cleanly to SYS-016 since SYS-016 does not mention that phase. | Chair: accept as a CDR-deferred action to amend SW-REQ-SYS-016, or document SYS-016's intent as "detected phases only" in its rationale field. | OPEN | — | — | OPEN |

### 4.3 CE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S1-012 | `requirements/index.md` count drift vs JSON reality | `requirements/index.md` lists `device = 6 reqs` and per-module sum of 370. Actual `requirements/device/requirements.json` has 7 IDs and repo total is 371 (matches pre-flight `traceability.py`). Not an S1-doc defect but a baseline drift the L1 design implicitly relies on. | Software Lead updates `requirements/index.md` device row to 7 and per-module sum to 371 before S2 convenes. No impact on system_design or conventions. | OPEN | — | — | OPEN |
| PDR-RFA-S1-013 | WCET §8.2 assertion lacks per-module budget table (related to MAE-RFA-001) | §8.2 asserts "any single 5 ms tick … fits in 5 ms with margin" without an L1 µs/app budget table. The claim then propagates into 27 per-module L2 §8 sections without substantiation. | Add an L1 indicative budget table (engineering estimate, not measured) and explicitly mark "to be measured at CDR per SW-REQ-SYS-044". Defers measurement to CDR with explicit Chair approval per Charter §7 #3. | OPEN | — | — | OPEN |
| PDR-RFA-S1-014 | POSIX-only test platform may mask Pico2-specific failure modes | §10.2 makes POSIX the gate for SW-REQ-SYS-054 (100% line coverage); Pico2 equivalence is asserted by construction. Real-time / cache / interrupt issues unique to RP2350 not exercised by gate. | At CDR, add a HIL or on-target smoke test gate exercising the Pico2 build at the level of one full hyperperiod with sensors driven by a bench harness; record in §10.2 that POSIX gate is necessary but not sufficient for SYS-043. | OPEN | — | — | OPEN |
| PDR-RFA-S1-015 | IMU TBD has no decision-deadline trigger at S1 | conventions FLAG-4 carries IMU-model TBD forward. system_design §3.3 hard-codes 5 ms IMU period via SYS-005 independent of part choice (acceptable). However, `imu/design.md` cannot complete §4 without part choice. No system-level deadline recorded. | PM-level action: set a part-selection deadline before CDR. S1 documents do not need amendment. | OPEN | — | — | OPEN |
| PDR-RFA-S1-016 | SYS-014 numeric threshold deferred to nav L2 with no system-level cap | system_design §1 (out-of-scope) and §9.9 defer SYS-014 numeric horizontal bound to nav L2. The L1 contract is "nav publishes `bValid=false` when bound exceeded; FSW takes no further action." Consistent with SYS-014 rationale and SYS-034 continuation. | Defer to S5 nav L2 review. Confirm at nav L2 review that the numeric value in `nav/design.md` is the only authoritative numeric bound for SYS-014. | OPEN | — | — | OPEN |

## 5. Disposition Decisions

> Chair statement, 2026-05-02: "I accept the findings of the board."
> Software Lead recorded each item below as ACCEPT (or ACCEPT-MOD where the
> reviewer presented an option set). For the two Major RIDs the
> reviewer-preferred option is recorded; the Chair retains the right to
> revise option selections before S10 closure.

### RIDs

- `[PDR-RID-S1-001]: ACCEPT` — §11 traceability table to be reconciled with per-section design tags. Owner: Software Lead.
- `[PDR-RID-S1-002]: ACCEPT` — Remove `JUNO_MSG_MLOG_RECORD_T` from §4 bus catalog and document as SD-card persistence record kind in §6. Correct §7.3 arrow to "Persist". Owner: Software Lead.
- `[PDR-RID-S1-003]: ACCEPT` — Align §3.3 header paths to architecture.md baseline (`apps/<name>_app/include/<name>/<name>_app.hpp`). Owner: Software Lead.
- `[PDR-RID-S1-004]: ACCEPT` — Reconcile §5 Recovery state-vs-prose by removing Recovery from the state diagram and treating it as an annotation on the LANDING-phase observable; update §11 accordingly. Owner: Software Lead.
- `[PDR-RID-S1-005]: ACCEPT-MOD` — Option A: change `kMlogAppPeriodMs` to 5 ms; co-schedule mlog with imu_app. Update §3.3 rationale, §8.2 offset table, §11. Hyperperiod count for mlog rises from 100 to 200; worst-case 5 ms tick budget set unchanged. Owner: Software Lead.
- `[PDR-RID-S1-006]: ACCEPT` — Replace §8.1 pseudocode bare `.tOk` with explicit `RESULT_T`/`tStatus`/`JUNO_ASSERT_*` exemplar showing POST-bitmap handling. Owner: Software Lead.
- `[PDR-RID-S1-007]: ACCEPT` — Add `static constexpr size_t kNmeaMaxSentenceBytes = 84;` to `conventions.md` §4.4 (or `system_design.md` §4); change catalog entry to `acSentence[kNmeaMaxSentenceBytes]`. Owner: Software Lead.
- `[PDR-RID-S1-008]: ACCEPT` — Add formal `using JUNO_TIME_PROVIDER_T = JUNO_TIME_US_T (*)(JUNO_USER_DATA_T*) noexcept;` to `conventions.md` §4.2; reference from `system_design.md` §8.1 and time_lib L2 design. Owner: Software Lead.
- `[PDR-RID-S1-009]: ACCEPT` — Reconcile §8.1 step 5 prose with pseudocode; surface `time_lib` injection explicitly; mark step 5 as schematic and direct readers to per-app L2 §4 for authoritative signature. Owner: Software Lead.
- `[PDR-RID-S1-010]: ACCEPT` — Rephrase §8.1 step 4 as an invariant; clarify that the 7 rules are invariants (not strict time-ordering); flowchart is authoritative for sequencing. Owner: Software Lead.
- `[PDR-RID-S1-011]: ACCEPT` — Option (a): amend SW-REQ-SYS-016 to include `pre-launch` as the at-power-on initial phase, with rationale that pre-launch is the initial state (not a "detected" phase). Owner: Software Lead (drafts amendment) → Chair (approves requirement edit).
- `[PDR-RID-S1-012]: ACCEPT` — Add a bullet to `conventions.md` §4.2 (or §6) referencing `JUNO_TIME_PROVIDER_T`; addressed jointly with RID-S1-008. Owner: Software Lead.
- `[PDR-RID-S1-013]: ACCEPT` — Add subscribe-at-`Init()` rule to `conventions.md` §1.3 (Mandatory rules) or §5. Owner: Software Lead.

### RFAs

- `[PDR-RFA-S1-001]: ACCEPT` — Add per-module µs sub-budget table to `system_design.md` §8.2; mark as engineering estimate, defer measurement to CDR per SW-REQ-SYS-044. Combined execution with RFA-S1-013. Owner: Software Lead.
- `[PDR-RFA-S1-002]: ACCEPT` — Add equivalence pass/fail criterion (e.g., "bit-identical NAV_STATE and AFM_PHASE outputs given identical input recording") to §10.2; defer test artifact itself to CDR. Combined with RFA-S1-014. Owner: Software Lead.
- `[PDR-RFA-S1-003]: ACCEPT` — Promote Trick injection callback contract (monotonic non-decreasing, µs resolution, sole time source for libs in that build) from comment to §10.2 paragraph. Combined with RID-S1-008/-012. Owner: Software Lead.
- `[PDR-RFA-S1-004]: ACCEPT` — Remove SW-REQ-SYS-043 from §3 design tag (coverage by §10.2 + §11 sufficient). Owner: Software Lead.
- `[PDR-RFA-S1-005]: ACCEPT` — PM-level requirements-baseline action; addressed jointly with RID-S1-011 Option (a). Owner: Chair (PM).
- `[PDR-RFA-S1-006]: ACCEPT` — Replace §8.1 PM-Decision comment with citation to `conventions.md` §4.2; move PM-decision provenance to a separate decisions log. Owner: Software Lead.
- `[PDR-RFA-S1-007]: ACCEPT` — Add §10.2 paragraph naming the POSIX driver Trick-injection mechanism (e.g., optional data-provider callback analogous to `pfcnTimeProvider`); identify POSIX-only files. Owner: Software Lead.
- `[PDR-RFA-S1-008]: ACCEPT` — Add `enum class JUNO_FSW_STATE_T : uint8_t { JUNO_FSW_STATE_POST, _INIT, _RUN, _SAFE, _RECOVERY };` to `conventions.md` §4; reference from §5. Owner: Software Lead.
- `[PDR-RFA-S1-009]: ACCEPT` — Make `sys_app` L2 §4 the authoritative location for the `u32HealthBitmap` bit-assignment table; cross-referenced from sensor owners. Action carried into S8 sys_app review. Owner: Software Lead → S8 review board.
- `[PDR-RFA-S1-010]: ACCEPT` — Have nav_app L2 §4 evaluate `eNavValidityReason` sub-field vs. a single `bValid` and document the decision; carried into S7 nav_app review. Owner: Software Lead → S7 review board.
- `[PDR-RFA-S1-011]: ACCEPT` — Same root as RID-S1-011 Option (a). No separate action; closed by RID-S1-011 disposition.
- `[PDR-RFA-S1-012]: ACCEPT` — Update `requirements/index.md`: device row to 7, per-module sum to 371. Must complete before S2 convenes. Owner: Software Lead.
- `[PDR-RFA-S1-013]: ACCEPT` — Combined with RFA-S1-001. Owner: Software Lead.
- `[PDR-RFA-S1-014]: ACCEPT` — Combined with RFA-S1-002. Owner: Software Lead.
- `[PDR-RFA-S1-015]: ACCEPT` — IMU part-selection deadline before CDR; PM-level action. Owner: Chair (PM).
- `[PDR-RFA-S1-016]: ACCEPT` — Carried into S5 nav L2 review; confirm the numeric value in `nav/design.md` is the only authoritative bound for SW-REQ-SYS-014. Owner: Software Lead → S5 review board.

## 6. Action Items Created

Action IDs format: `S1-AI-NNN`. Targets reference PDR milestones: **pre-S2** = before S2 convenes; **batched-S1** = batched corrective edits to S1 documents executed before PDR closure (S10); **CDR** = deferred to Critical Design Review per Charter §1.2.

| Action ID | Source RID/RFA | Description | Owner | Target | Status |
|-----------|----------------|-------------|-------|--------|--------|
| S1-AI-001 | RID-S1-001 | Reconcile §11 traceability table with per-section design tags | Software Lead | batched-S1 | OPEN |
| S1-AI-002 | RID-S1-002 | Remove `JUNO_MSG_MLOG_RECORD_T` from §4 bus catalog; document in §6 as persistence record; correct §7.3 arrow | Software Lead | batched-S1 | OPEN |
| S1-AI-003 | RID-S1-003 | Align §3.3 header paths to `apps/<name>_app/include/<name>/<name>_app.hpp` form | Software Lead | batched-S1 | OPEN |
| S1-AI-004 | RID-S1-004 | Remove Recovery from §5 state diagram; treat as annotation on LANDING-phase observable; update §11 | Software Lead | batched-S1 | OPEN |
| S1-AI-005 | RID-S1-005 | Change `kMlogAppPeriodMs` 10 → 5; update §3.3 rationale, §8.2 offset table, §11 | Software Lead | **pre-S2** (affects S5 mlog and S7 mlog_app reviews) | DONE 2026-05-02 |
| S1-AI-006 | RID-S1-006 | Replace §8.1 pseudocode bare `.tOk` with `JUNO_ASSERT_*` exemplar showing POST-bitmap handling | Software Lead | batched-S1 | OPEN |
| S1-AI-007 | RID-S1-007 | Lock NMEA buffer size: add `kNmeaMaxSentenceBytes = 84` to conventions §4.4; update catalog entry | Software Lead | **pre-S3** (affects gps L2 review) | OPEN |
| S1-AI-008 | RID-S1-008, -012, RFA-001-003 | Add `using JUNO_TIME_PROVIDER_T = JUNO_TIME_US_T (*)(JUNO_USER_DATA_T*) noexcept;` to conventions §4.2; promote callback contract to §10.2 paragraph | Software Lead | **pre-S2** (affects S2 time_lib review) | DONE 2026-05-02 (typedef + contract added to conventions §4.2; system_design §10.2 paragraph deferred to batched-S1) |
| S1-AI-009 | RID-S1-009 | Reconcile §8.1 step 5 prose/pseudocode; surface `time_lib` injection; mark step 5 schematic | Software Lead | batched-S1 | OPEN |
| S1-AI-010 | RID-S1-010 | Rephrase §8.1 step 4 as invariant; clarify 7 rules are invariants not time-order | Software Lead | batched-S1 | OPEN |
| S1-AI-011 | RID-S1-011, RFA-005, RFA-011 | Draft amendment to SW-REQ-SYS-016 adding `pre-launch` as initial phase; route to Chair for approval | Software Lead → Chair | **pre-S5** (affects afm L2 review) | OPEN |
| S1-AI-012 | RID-S1-013 | Add subscribe-at-`Init()` rule to conventions §1.3 (or §5) | Software Lead | batched-S1 | OPEN |
| S1-AI-013 | RFA-S1-001, -013 | Add per-module µs sub-budget table to §8.2; mark engineering estimate; defer measurement to CDR | Software Lead | batched-S1 | OPEN |
| S1-AI-014 | RFA-S1-002, -014 | Add SW-REQ-SYS-043 equivalence pass/fail criterion to §10.2; defer test artifact to CDR | Software Lead | batched-S1 | OPEN |
| S1-AI-015 | RFA-S1-004 | Remove SW-REQ-SYS-043 from §3 design tag | Software Lead | batched-S1 | OPEN |
| S1-AI-016 | RFA-S1-006 | Replace §8.1 PM-Decision comment with conventions §4.2 citation; move provenance to decisions log | Software Lead | batched-S1 | OPEN |
| S1-AI-017 | RFA-S1-007 | Add §10.2 paragraph naming POSIX-driver Trick injection mechanism; identify POSIX-only files | Software Lead | batched-S1 | OPEN |
| S1-AI-018 | RFA-S1-008 | Add `JUNO_FSW_STATE_T` enum to conventions §4; reference from system_design §5 | Software Lead | **pre-S8** (affects sys_app review) | OPEN |
| S1-AI-019 | RFA-S1-009 | Adopt sys_app L2 §4 as authoritative location for health-bitmap bit-assignment table; cross-reference sensor owners | Software Lead → S8 board | **at S8 review** | OPEN |
| S1-AI-020 | RFA-S1-010 | Evaluate `eNavValidityReason` sub-field vs single `bValid` in nav_app L2 §4; document decision | Software Lead → S7 board | **at S7 review** | OPEN |
| S1-AI-021 | RFA-S1-012 | Update `requirements/index.md`: device row 6→7; per-module sum 370→371 | Software Lead | **pre-S2** | DONE 2026-05-02 |
| S1-AI-022 | RFA-S1-015 | Set IMU part-selection deadline before CDR | Chair (PM) | CDR | OPEN |
| S1-AI-023 | RFA-S1-016 | At S5 nav L2 review, confirm numeric value in `nav/design.md` is the only authoritative bound for SW-REQ-SYS-014 | Software Lead → S5 board | **at S5 review** | OPEN |

**23 action items total.** Targets summarize as:
- **pre-S2** (must clear before next section): S1-AI-005, -008, -021 — three items
- **pre-S3** (before sensor-driver libs review): S1-AI-007 — one item
- **pre-S5** (before domain libs review): S1-AI-011 — one item
- **pre-S8** (before system-app review): S1-AI-018 — one item
- **at-section** (raised during a future section's disposition): S1-AI-019, -020, -023 — three items
- **batched-S1** (executed in a single S1-corrective-action mini-sprint before S10): S1-AI-001, -002, -003, -004, -006, -009, -010, -012, -013, -014, -015, -016, -017 — thirteen items
- **CDR** (deferred per Charter §1.2): S1-AI-022 — one item

## 7. Section Verdict

The Chair selects exactly one of the three verdicts below by removing the
square brackets from the chosen line and deleting the other two. The
verdict line must be filled in before the section record is considered
final.

- **CHAIR PROCEED** — Section content is acceptable; PDR may proceed to the next section. Open action items will be tracked but do not block progression.

**Verdict Notes**

The Chair accepted all 13 RIDs and 16 RFAs. The two Major RIDs (PDR-RID-S1-005, PDR-RID-S1-011) carry agreed corrective actions with explicit owners and pre-section targets (S1-AI-005, S1-AI-011); per Charter §7, this satisfies the Major-RID disposition requirement for section advancement (corrective action assigned with explicit Chair approval, target dated). PDR may proceed to S2.

Three pre-S2 action items must close before S2 convenes:
- S1-AI-005 (kMlogAppPeriodMs 10 → 5; affects S5 mlog & S7 mlog_app)
- S1-AI-008 (`JUNO_TIME_PROVIDER_T` type alias in conventions; affects S2 time_lib)
- S1-AI-021 (requirements/index.md device count drift)

The Software Lead will execute these three before convening S2; the remaining batched-S1 items (13) and at-section items (3) will be executed before S10 closure or at the relevant downstream sections, respectively.

**Chair Signature**: Project Manager — 2026-05-02

## 8. Cross-References

### Documents Reviewed

- [docs/design/system/system_design.md](../../../design/system/system_design.md)
- [docs/design/conventions.md](../../../design/conventions.md)

### Master Log

- [PDR RID/RFA Master Log](../rid_rfa_log.md)

### Related Section Records

(none — first section)

## 9. Reviewer Recommendations Summary

| Reviewer | Total RIDs | Major | Minor | Editorial | RFAs | Recommendation |
|----------|-----------|-------|-------|-----------|------|----------------|
| MAE | 4 | 0 | 4 | 0 | 6 | PROCEED |
| SSE-R | 6 | 1 | 4 | 1 | 5 | **HOLD** (Major: PDR-RID-S1-005) |
| CE | 3 | 1 | 2 | 0 | 5 | PROCEED with disposition (Major resolvable as ACCEPT-MOD) |
| **Section Total** | **13** | **2** | **10** | **1** | **16** | Mixed — Chair decides |

The two **Major** RIDs (PDR-RID-S1-005, PDR-RID-S1-011) and the four pairs of related items below should be considered as groups during disposition:

- **Phase enum scope:** RID-S1-011 (Major), RFA-S1-005, RFA-S1-011 — same root.
- **WCET budget:** RFA-S1-001, RFA-S1-013 — same root.
- **POSIX/Pico2 equivalence verification:** RFA-S1-002, RFA-S1-014 — same root.
- **Trick time injection contract:** RID-S1-008, RID-S1-012, RFA-S1-003 — same root.
