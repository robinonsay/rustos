---
sprint_id: SPRINT-IMPL-08
module: baro_lib
wave: 3 (Sensor Driver Libraries)
predecessors: SPRINT-IMPL-00..07 (Wave 0/1/2 + imu_lib closed)
status: CLOSED
opened: 2026-05-06
closed: 2026-05-06
ce_verdict: APPROVED
pm_signoff: pending
---

# SPRINT-IMPL-08 Closure Record — `baro_lib`

## 1. Sprint Goal

Implement the `baro_lib` Wave 3 sensor driver covering the NXP MPL3115A2
barometric altimeter behind the LibJuno C++ vtable pattern. Per SDP §5
master sprint table: 10 SW-REQ-BARO-* requirements, 12 test cases (10 Unit
+ 2 Demonstration deferred to HIL).

## 2. PM-Approved Scope Decisions

| Q | Decision | Rationale |
|---|----------|-----------|
| Q1 | Per-platform IMPL split (`BARO_LIB_POSIX_T` / `BARO_LIB_PICO2_T`) | Mirrors SPRINT-IMPL-07 imu_lib precedent / SPRINT-IMPL-05-retro-A canonical pattern |
| Q2 | Single `baro_lib_test.cpp` exercising both IMPLs via parameterized fixture | baro_lib has no pico-sdk dependency (callback-injected transport per L2 §3.2); no stubs needed; 19 agents vs imu_lib's 28 |
| Q3 | Demonstration TCs SW-TC-BARO-005 (POST probe) / SW-TC-BARO-010 (Pico2 lift stimulus) deferred to HIL bench | No hardware available; matches imu_lib carry-forward |

PM approved 2026-05-06 with all three recommendations.

## 3. Acceptance Criteria — Final Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | Public API per L2 §4.1 | MET | [libs/baro_lib/include/baro_lib/baro_api.hpp](../../libs/baro_lib/include/baro_lib/baro_api.hpp) — 5 vtable refs (Configure/Probe/Sample/SetSlp/IsHealthy); BUS_T; SAMPLE_T; constants `kMpl3115a2I2cAddr=0x60`, `kSampleRateHz=20`, `kDefaultSlpPa=101325.0f` |
| AC-2 | Per-platform IMPL split | MET | `BARO_LIB_POSIX_T` at [baro_posix.hpp:199](../../libs/baro_lib/include/baro_lib/baro_posix.hpp#L199); `BARO_LIB_PICO2_T` at [baro_pico2.hpp:171](../../libs/baro_lib/include/baro_lib/baro_pico2.hpp#L171); both `JUNO_MODULE_DERIVE`-d |
| AC-3 | Common decode + `tTimestampUs == tNowUs` invariant | MET | [baro_common.cpp](../../libs/baro_lib/src/common/baro_common.cpp) Q18.2/Q12.4 decoders; success and failure paths both write `tNowUs` to `tTimestampUs` (Tc008/Tc011 verify) |
| AC-4 | Configure register sequence | MET | Mock-callback assertion in Tc009: 5 writes (0x26=0xB8, 0x13=0x07, 0x14, 0x15, 0x26=0xB9) |
| AC-5 | Probe WHO_AM_I (0x0C → 0xC4) | MET | Tc012 verifies mismatch path returns `JUNO_STATUS_DNE_ERROR` |
| AC-6 | Sample timeout / failure semantics | MET | Tc007 (read-fail → bValid=false, bHealthy=false), Tc008 (timeout passthrough), Tc011 (intermittent failures sustained) |
| AC-7 | `BARO_SAMPLE_T = {}` value-init | MET | All three return paths in `BaroCommon_DoSample` use the value-init pattern (Sprint 7 lesson) |
| AC-8 | All 10 SW-REQ-BARO-* `@req`-tagged | MET | `grep` confirms 10 unique IDs; @verify covers SW-REQ-BARO-001..009 (BARO-010 Demonstration-only and deferred per Q3) |
| AC-9 | Vtable wired once via `static const BARO_LIB_API_T tApi{...}` local | MET | [baro_posix.cpp:117](../../libs/baro_lib/src/posix/baro_posix.cpp#L117), [baro_pico2.cpp:109](../../libs/baro_lib/src/pico2/baro_pico2.cpp#L109); `tImpl.tRoot.ptApi = &tApi` |
| AC-10 | All API entries `noexcept`; freestanding-clean | MET | No `virtual`/`new`/`delete`/`throw`/`try`/`catch` anywhere; -Werror clean |
| AC-11 | Failure handler diagnostic-only | MET | Every IO-failure path in baro_common.cpp invokes the handler if non-null; never aborts |
| **AC-12** | **G1 PASS — POSIX build + ctest** | **MET** | `12/12 tests passed, 0 failed` (baro_lib_test ran 20 parametric instances, all PASS — 10 Unit-type SW-TC × 2 IMPL params) |
| **AC-13** | **G2 PASS — `tools/traceability.py` exit 0** | **MET** | `TRACEABILITY CHECK PASSED — 376 valid req IDs, 81 with @verify, 376 with test specs` |
| **AC-14** | **G3 PASS — Pico2 cross-compile clean** | **MET** | `cmake --build build_pico2 --target baro_lib` → `[100%] Built target baro_lib` (exit 0) |
| AC-15 | `add_subdirectory(baro_lib)` registered | MET | [libs/CMakeLists.txt:10](../../libs/CMakeLists.txt#L10) |
| AC-16 | Per-source `set_source_files_properties` COMPILE_OPTIONS pattern | MET | [libs/baro_lib/CMakeLists.txt](../../libs/baro_lib/CMakeLists.txt) — no legacy `target_compile_options(${PROJECT_NAME} PRIVATE …)` |
| AC-17 | CE issues APPROVED | **MET** | See §6 below |

## 4. Deliverable File Inventory

10 production files (vs SDP §5's 6 — expansion documented as PM-approved Q1):

| # | Path | Lines | Phase | Author | Final Status |
|---|------|-------|-------|--------|--------------|
| 1 | `libs/baro_lib/include/baro_lib/baro_api.hpp` | 499 | 1 | senior-software-engineer | APPROVED iter-1 + Lead-direct (BUS_T `&`→`*` pointer fix; bHasRead field added; docstring tightening) |
| 2 | `libs/baro_lib/CMakeLists.txt` | 138 | 1 | junior-software-engineer | APPROVED iter-1 + Lead-direct (-Wmissing-field-initializers / -fPIC / -O1 alignment with imu_lib; cross-IMPL test compile) |
| 3 | `libs/baro_lib/include/baro_lib/baro_posix.hpp` | 497 | 2 | senior-software-engineer | APPROVED iter-1 + Lead-direct (stale JUNO_STATUS_PRECONDITION_ERROR doc fix) |
| 4 | `libs/baro_lib/include/baro_lib/baro_pico2.hpp` | 388 | 2 | senior-software-engineer | APPROVED iter-1 |
| 5 | `libs/baro_lib/src/common/baro_common.hpp` | 218 | 2 | junior-software-engineer | APPROVED iter-1 |
| 6 | `libs/baro_lib/src/common/baro_common.cpp` | 490 | 3 | senior-software-engineer | APPROVED iter-1 + Lead-direct (`bConfigured=false` on Configure failure; bHasRead semantic for IsHealthy) |
| 7 | `libs/baro_lib/src/posix/baro_posix.cpp` | 149 | 3 | senior-software-engineer | APPROVED iter-1 + Lead-direct (BUS_T pointer comment refresh) |
| 8 | `libs/baro_lib/src/pico2/baro_pico2.cpp` | 158 | 3 | senior-software-engineer | APPROVED iter-1 |
| 9 | `libs/baro_lib/tests/baro_lib_test.cpp` | 497 | 3 | senior-software-engineer | APPROVED iter-1 + Lead-direct (kFakeNowUs odr-use definition; Tc003 raw values × 256 instead of × 16 to match MPL3115A2 4-bit padding; Configure write-sequence assertion in Tc009) |

**Lead-direct artifacts (not in deliverable count):**
- `docs/design/baro/design.md` §4.3 (per-platform IMPL split language) + §4.1 BUS_T pointer amendment
- `libs/CMakeLists.txt` registration (`add_subdirectory(baro_lib)`)

## 5. Worker / Reviewer Summary

### Workers (9 invocations, 3 phases)

| Phase | File | Worker | Iter | Final |
|-------|------|--------|------|-------|
| 1 | baro_api.hpp | senior-software-engineer | 1 | APPROVED |
| 1 | CMakeLists.txt | junior-software-engineer | 1 | APPROVED |
| 2 | baro_posix.hpp | senior-software-engineer | 1 | APPROVED |
| 2 | baro_pico2.hpp | senior-software-engineer | 1 | APPROVED |
| 2 | baro_common.hpp | junior-software-engineer | 1 | APPROVED |
| 3 | baro_common.cpp | senior-software-engineer | 1 | APPROVED + Lead-direct |
| 3 | baro_posix.cpp | senior-software-engineer | 1 | APPROVED |
| 3 | baro_pico2.cpp | senior-software-engineer | 1 | APPROVED |
| 3 | baro_lib_test.cpp | senior-software-engineer (test author) | 1 | APPROVED + Lead-direct |

### Reviewers (9 invocations, 3 phases)

All Phase 1/2/3 reviewers were senior-software-engineer in reviewer mode.
Phase 3 baro_lib_test.cpp reviewer ran ctest before issuing verdict
(Sprint 7 lesson — "Test Reviewer Must Run the Test, Not Just Inspect It").
The reviewer caught a structural build failure (BARO_LIB_BUS_T reference
members blocking default-construct + copy-assign) that the four Phase-2
reviewers did not flag — confirming the Sprint-7 lesson applies.

### Project Chief Engineer (2 invocations)

- **Iter 1:** REJECTED on AC-17 only — `baro_api.hpp` = 501 lines (single-line overrun on the 500-line hard cap). 16/17 ACs MET.
- **Iter 2:** APPROVED after Lead-direct docstring tightening reduced `baro_api.hpp` to 499 lines. All 17 ACs MET.

## 6. Chief Engineer Verdict

**APPROVED** (re-gate iteration). All 17 acceptance criteria MET. Gate
evidence: G1 (12/12 tests pass), G2 (traceability exit 0, 81 @verify
tags), G3 (Pico2 cross-compile clean). Cross-sprint consistency holds —
per-platform DERIVE pattern, per-source COMPILE_OPTIONS, ptApi dispatch,
and failure handler invocation all honor the canonical SPRINT-IMPL-05-retro-A
/ SPRINT-IMPL-07 patterns. No ID conflicts; no broken references. File-size
compliance verified across all 9 deliverables (max = 499 lines on baro_api.hpp).

## 7. Carry-Forwards

1. **HIL Demonstration TCs** — SW-TC-BARO-005 (POST present/absent on real
   MPL3115A2) and SW-TC-BARO-010 (Pico2 lift stimulus) deferred per PM Q3.
   Track in HIL backlog alongside imu_lib HIL deferrals.
2. **`JUNO_TIME_US_T` alias** — when LibJuno publishes the alias (currently
   only `JUNO_TIME_MICROS_T` exists), retroactively update `BARO_SAMPLE_T::tTimestampUs`
   and bus-callback timeout types per the file-header deviation note in
   baro_api.hpp.
3. **File-size headroom** — `baro_posix.hpp` (497), `baro_common.cpp` (490),
   and `baro_lib_test.cpp` (497) are within budget but have ≤10 lines of
   headroom. Any future addition to those files needs file-size headroom
   recheck.
4. **L2 §4.1 BUS_T pointer amendment** — the L2 design originally specified
   reference-typed BUS_T members. The amendment (Phase 3 Lead-direct) changed
   them to function pointers per LibJuno-canonical callback shape. The L2
   amendment is documented inline; PM may wish to formalize it as a Revision
   C of the baro design doc.

## 8. Lessons Captured (added to `lessons-learned-software-lead.md`)

- **2026-05-06 — SPRINT-IMPL-08: Reference-Typed Members Block Default-Init Pattern**
  L2 specified `JUNO_STATUS_T (&WriteReg)(...)` reference-typed BUS_T members.
  References make the struct non-default-constructible and non-copy-assignable,
  breaking the canonical `tImpl = {}` + field-assignment factory pattern that
  imu_lib and the LibJuno template use. Convert callback members to function
  pointers (matching `JUNO_FAILURE_HANDLER_T` and the broader LibJuno callback
  shape) when the holding struct is part of a value-typed ROOT/IMPL.
- **2026-05-06 — SPRINT-IMPL-08: Single Test File Cross-Compiles Both IMPL TUs**
  When a Wave-3 sensor lib has zero pico-sdk dependency, the unit-test target
  can compile BOTH the platform-current IMPL TU (via the lib's standard build)
  AND the platform-other IMPL TU (added directly to the test executable's
  source list). This satisfies methodology §5.1 Revision B's Pico2 unit-test
  coverage mandate WITHOUT a stub-driven secondary test target — saving
  ~6 agent invocations vs the imu_lib pattern.
- **2026-05-06 — SPRINT-IMPL-08: Static-constexpr ODR-Use in C++11 Fixtures**
  Google Test fixtures with `static constexpr` members consumed by reference
  (e.g., `EXPECT_EQ(kFakeNowUs, ...)`) are odr-used and require an out-of-class
  definition under C++11 (made redundant by C++17 inline-variable rules, but
  the project standard is C++11). Add `constexpr T ClassName::kMember;`
  directly after the class declaration. Apply to any future test file using
  the static-constexpr-fixture-member pattern.
- **2026-05-06 — SPRINT-IMPL-08: Q-Format with Padding ≠ Q-Format Without**
  MPL3115A2 OUT_T_LSB lower nibble is reserved/zero (per datasheet §5.2),
  pushing the int16 composite to `value × 256` rather than the unpadded
  `value × 16`. Test injection helpers MUST account for the padding when
  encoding raw values. Document the padding convention in the lib's common
  header alongside the decode constant. Apply to any future MPL-class sensor
  with similar Q-format conventions (baro/temperature sensors typically have
  4-bit padding).
- **2026-05-06 — SPRINT-IMPL-08: IsHealthy "Never Read" Sentinel Cannot Reuse `bConfigured`**
  L2 §4.2.5 says IsHealthy returns None "if never read." `bConfigured` is
  NOT the right gate because Probe() can run without Configure() and a failed
  Probe should produce Some(false), not None. Add a dedicated `bHasRead` flag
  set on Probe/Sample entry. Apply to any future driver that distinguishes
  "no read attempted" from "read attempted, healthy" / "read attempted, failed."
