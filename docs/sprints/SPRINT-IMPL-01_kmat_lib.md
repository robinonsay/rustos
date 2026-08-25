# SPRINT-IMPL-01 — kmat_lib (Wave 1, Kinematics Layer)

| Field | Value |
|-------|-------|
| Sprint ID | SPRINT-IMPL-01 |
| Module | `kmat_lib` (kinematics layer; layered on `juno::math`) |
| Wave | 1 (Foundation Libraries) |
| Revision | B (REV A → B at Phase 1A; PM directive 2026-05-04 to layer kmat on `juno::math` rather than duplicate primitives) |
| Start date | 2026-05-04 |
| End date | 2026-05-04 |
| Status | **CLOSED** |
| Predecessors | SPRINT-IMPL-00 (CLOSED); LibJuno `juno::math` upstream |
| Successor eligible | SPRINT-IMPL-02 (log_lib, Wave 1) |
| PM approval | 2026-05-04 — sprint plan approved; REV B layering approved; gtest infrastructure approved |

## Sprint Goal

Author the header-only `kmat_lib` providing the **kinematics layer** of the FSW math stack — fixed-size matrix algebra (`MAT_T<R,C>`, `MatMul`, `Transpose`, `Add`/`Sub`/`Mult`, `Invert`, `MatVecMul`) and quaternion-attitude operations (`QuatNormalize`, `QuatToMat3`, `QuatRotate`) layered on top of LibJuno's primitive `juno::math::VEC` and `juno::math::QUAT` types via `using`-declarations. Satisfies all 15 `SW-REQ-KMAT-*` requirements with 20 `SW-TC-KMAT-*` unit tests.

## REV B Architectural Pivot (Phase 1A)

PM directive 2026-05-04: kmat is the kinematics layer; LibJuno's `juno::math` (`libjuno/include/juno/math/juno_math.hpp`) provides the primitive vector/quaternion types and operations. The original L2 (REV A) duplicated `VEC`, `QUAT`, `Add`, `Sub`, `Mult`, `Dot`, `Cross`, `L2Norm2`, `HamProd`, `Conj`, `Recip` as kmat-side declarations — substantially overlapping with LibJuno upstream. REV B re-exports those symbols via `using juno::math::*;` declarations and reserves kmat-original status for: `MAT_T`, matrix algebra, `Invert` (LU partial pivoting + numeric-error status), kinematics ops (`QuatNormalize`/`QuatToMat3`/`QuatRotate`), `MatVecMul`, `kPivotEpsilon<T>()`, and `JUNO_FSW_STATUS_NUMERIC_ERROR`. Requirements unchanged.

## Worker Invocations

| # | Phase | File | Worker | Iter | Final Status |
|---|-------|------|--------|------|--------------|
| 1 | 1A | `docs/design/kmat/index.md` (354 lines) | software-systems-engineer | 1 | APPROVED |
| 2 | 1A | `docs/design/kmat/04_interface.md` (432 lines) | software-systems-engineer | 1 | APPROVED |
| 3 | 1B | `libs/kmat_lib/include/kmat_lib/kmat_api.hpp` (364 lines) | senior-software-engineer | 1 | APPROVED |
| 4 | 1B | `libs/kmat_lib/include/kmat_lib/kmat_impl.hpp` (490 lines) | senior-software-engineer | 1 | APPROVED |
| 5 | 1B | `libs/kmat_lib/tests/kmat_test.cpp` (464 lines) | senior-software-engineer (test author) | 1 | NEEDS CHANGES → Lead-direct atomic edits applied |
| 6 | 1B | `libs/kmat_lib/CMakeLists.txt` (69 lines) | junior-software-engineer | 1 | NEEDS CHANGES → Lead-direct atomic edit applied |

## Reviewer Verdicts

| # | Phase | Reviewer | File Reviewed | Iter | Verdict |
|---|-------|----------|--------------|------|---------|
| 1 | 1A | software-mission-assurance-engineer | `docs/design/kmat/index.md` | 1 | APPROVED — all AC PASS |
| 2 | 1A | software-mission-assurance-engineer | `docs/design/kmat/04_interface.md` | 1 | APPROVED — all AC PASS; LibJuno symbol authority verified end-to-end |
| 3 | 1B | senior-software-engineer (reviewer mode) | `kmat_api.hpp` | 1 | APPROVED — 0 errors, smoke compile clean |
| 4 | 1B | senior-software-engineer (reviewer mode) | `kmat_impl.hpp` | 1 | APPROVED — 0 errors, QuatToMat3 matrix verified element-by-element |
| 5 | 1B | senior-software-engineer (reviewer mode) | `kmat_test.cpp` | 1 | NEEDS CHANGES — 1 Error (TC-010 magnitude) + 2 Warnings (TC-008 trivial inverse, TC-012 missing ops) |
| 6 | 1B | senior-software-engineer (reviewer mode) | `CMakeLists.txt` | 1 | NEEDS CHANGES — `-pedantic` + `-Wundef` missing |
| 7 | 3.5 | software-mission-assurance-engineer (test-audit) | `kmat_test.cpp` (post-fix) | 1 | APPROVED — 20/20 SW-TC-KMAT-* validity gate; per-test audit table all OK |
| 8 | 4 | project-chief-engineer | sprint deliverable | 1 | **PASS** |

## Lead-Direct Atomic Edits Applied During Sprint

Per the 2026-05-03 atomic-Lead-edit pattern (mechanical 1-line / 1-token / 1-paragraph fixes coordinated across files):

1. **`kPivotEpsilon` C++14 variable template → C++11 function template** — primary `template<typename T> static constexpr T kPivotEpsilon() noexcept;` + `<float>` and `<double>` specializations; call sites in impl gain `()`. (4 sites)
2. **`CMakeLists.txt` `-pedantic` + `-Wundef`** added to `JUNO_COMPILE_OPTIONS` per `coding-standards.md` mandate (closes Reviewer 4 finding).
3. **Test fixes per Reviewer B3:**
   - TC-008 KnownNumericInverse: identity matrix → `diag(2,3,4,5)` with hand-computed expected `diag(0.5, 1/3, 0.25, 0.2)`
   - TC-010 NearSingular: `diag(1e-15)` → `diag(1e-31)` (must be strictly below `kPivotEpsilon<double>() = 1e-30` for LU partial pivoting to flag singular)
   - TC-012 AllOps_RepeatedRuns_BitIdentical: added `Sub`, `Transpose`, `Mult` to the 3-run determinism sweep so all 5 ops the JSON procedure mentions are covered
4. **Pico2 freestanding fix:** drop `#include <cmath>` from both api and impl headers; add `KmatSqrt<T>()` (using `__builtin_sqrt`/`__builtin_sqrtf`) and `KmatAbs<T>()` helpers in `kmat_impl.hpp`. `<cmath>` is forbidden under `-ffreestanding` (triggers `#error "This header is not available in freestanding mode."` from newlib).
5. **`EXPECT_TRUE` template-comma preprocessor trap:** wrap `EXPECT_TRUE(MatNear<T,R,C>(...))` and `EXPECT_TRUE(MatBitEqual<T,R,C>(...))` in extra parens. The preprocessor splits `<double, 4, 4>` on commas before C++ semantic analysis; the macro receives 3 args instead of 1. (sed sweep + 1 manual fix)
6. **FSW-side test-flag rename `JUNO_TESTS` → `JUNO_FSW_TESTS`** — avoids cascading into LibJuno's own `JUNO_TESTS` flag which builds LibJuno's pre-existing broken-includes test target.
7. **gtest wired at FSW top-level** via `FetchContent_Declare(googletest URL https://github.com/google/googletest/archive/refs/tags/v1.14.0.tar.gz)` (pattern adapted from `libjuno/CMakeLists.txt:166-173`).
8. **`add_subdirectory(kmat_lib)` added to `libs/CMakeLists.txt`.**
9. **L2 amendments:** `docs/design/kmat/05_through_11.md` §10 Memory Ownership table `tData → arr` rename; revision letter A → B with REV B note; `docs/sdp/foundation_libs.md` §3 SPRINT-IMPL-01 AC-5 wording updated to record `arr[0..3]` mapping (w,x,y,z).
10. **`docs/test_cases/kmat/test_cases.json`** all 20 `google_test_ref` paths and 3 procedure references patched from legacy `libs/juno_kmat/...` → `libs/kmat_lib/tests/kmat_test.cpp` and `libkmat_lib.a`.
11. **`docs/sdp/index.md`** master sprint table updated to mark SPRINT-IMPL-01 CLOSED 2026-05-04.

## Phase 3 Gate Evidence (verified independently by CE)

```
=== G1: POSIX build + ctest ===
$ cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON ..
Configure exit: 0
$ cmake --build .
Build exit: 0
$ ctest --output-on-failure
1/1 Test #1: kmat_test ........................   Passed    0.00 sec
100% tests passed, 0 tests failed out of 1
G1 exit: 0

$ ./libs/kmat_lib/kmat_test
[==========] 22 tests from 1 test suite ran. (0 ms total)
[  PASSED  ] 22 tests.

=== G2: tools/traceability.py ===
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       10
  Requirements with @verify:    15
  Requirements with test specs: 375
G2 exit: 0

(Counter delta from pre-sprint baseline: +10 with code, +15 with @verify.
 The 5-req gap on "with code" is for file-scope inspection-only requirements
 SW-REQ-KMAT-008/-009/-010/-011/-012/-014/-015 tagged at the top-of-file
 @req block in kmat_api.hpp lines 48-50 rather than per-function. The
 traceability tool's "with code" counter recognizes function-scope tags only.)

=== G3: Pico2 cross-compile freestanding ===
$ arm-none-eabi-g++ -std=c++11 -Wall -Wextra -Werror -pedantic -Wshadow
    -Wcast-align -Wundef -Wswitch -Wswitch-default -fno-rtti -fno-exceptions
    -fno-common -fno-strict-aliasing -ffreestanding -mcpu=cortex-m33 -mthumb
    -I libjuno/include -I libs/kmat_lib/include -c <smoke>.cpp
G3 exit: 0
```

## Acceptance Criteria — Final Status

| AC | Description | Status | Evidence |
|----|-------------|--------|----------|
| AC-1 | All 15 SW-REQ-KMAT-* tagged in kmat_api/impl.hpp; all 20 SW-TC-KMAT-* tagged in kmat_test.cpp | MET | `grep -c "@\\{\"req\":"` covers all 15; `grep -c "@\\{\"verify\":"` = 20 |
| AC-2 | `Invert` LU partial-pivoting `kPivotEpsilon<T>()` exercised by SW-TC-KMAT-009 (singular row of zeros) and -010 (`diag(1e-31) < 1e-30`) | MET | `kmat_impl.hpp:262`, `kmat_test.cpp:202-208` and `:211-218` |
| AC-3 | `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR = JUNO_STATUS_CUSTOM_ERROR + 1` declared; returned by `Invert` and `QuatNormalize` | MET | `kmat_api.hpp:103-104`; `kmat_impl.hpp:277` (Invert) and `:431` (QuatNormalize) |
| AC-4 | CMakeLists handles libm on POSIX; Pico2 inherits via pico-sdk newlib | MET | `CMakeLists.txt:44-46` `if(JUNO_FSW_POSIX) target_link_libraries(... INTERFACE m) endif()` |
| AC-5 (REV B) | Quaternion storage `juno::math::QUAT<T>::arr[4]` re-exported, scalar-first Hamilton `arr[0..3]=(w,x,y,z)` per SW-REQ-SYS-041 | MET | `kmat_api.hpp:71` `using juno::math::QUAT;`; upstream `libjuno/include/juno/math/juno_math.hpp:114-124`; tests use `{{w,x,y,z}}` |
| AC-6 | `-fno-rtti` and `-fno-exceptions` enforced on every kmat TU | MET | `CMakeLists.txt:27-30` `JUNO_COMPILE_CXX_OPTIONS`; SW-TC-KMAT-014/-017/-018 inspection-style asserts |
| AC-7 | Gates G1 + G2 + G3 all exit 0 | MET | See gate evidence above |

## Risk Resolution

- **Reviewed cross-module overlap with LibJuno math:** caught at Phase 2 by PM observation; resolved via REV B layering (SDP minor amendment 2026-05-04).
- **Phase-0 gate-tool verification gap:** gtest infrastructure absent at FSW top-level; resolved Lead-direct mid-sprint by wiring FetchContent. Captured as lessons-learned (`ai/memory/lessons-learned-software-lead.md` 2026-05-04).
- **Pico2 freestanding `<cmath>` block:** `<cmath>` forbidden under `-ffreestanding`; resolved Lead-direct via `__builtin_sqrt`/`KmatAbs` helpers. Captured as lessons-learned (`ai/memory/lessons-learned-senior-software-engineer.md` 2026-05-04).
- **`EXPECT_TRUE` template-comma preprocessor trap:** classic C macro splitting on `<double, R, C>`; resolved Lead-direct via extra-paren wrapping. Captured as lessons-learned.

## Files Touched (created / edited / amended)

**Created:**
- `libs/kmat_lib/include/kmat_lib/kmat_api.hpp` (364 lines)
- `libs/kmat_lib/include/kmat_lib/kmat_impl.hpp` (490 lines)
- `libs/kmat_lib/tests/kmat_test.cpp` (464 lines)
- `libs/kmat_lib/CMakeLists.txt` (69 lines)
- `docs/sprints/SPRINT-IMPL-01_kmat_lib.md` (this file)

**Amended (REV A → REV B Lead-direct + worker authoring):**
- `docs/design/kmat/index.md` (354 lines, REV B — §1, §2, §3.1-§3.4 reorganized for layering)
- `docs/design/kmat/04_interface.md` (432 lines, REV B — §4 substantially rewritten; §4.1 `arr` rename; §4.2 matrix-only; §4.3 MAT operators; §4.6 re-exports; §4.7 status code; §4.8 NEW kPivotEpsilon)
- `docs/design/kmat/05_through_11.md` (REV B — §10 storage table `tData → arr`; revision letter bumped)
- `docs/sdp/foundation_libs.md` (§3 SPRINT-IMPL-01 AC-5 wording updated for `arr[0..3]` mapping)
- `docs/sdp/index.md` (master sprint table — SPRINT-IMPL-01 marked CLOSED)
- `docs/test_cases/kmat/test_cases.json` (all 20 `google_test_ref` paths patched to `libs/kmat_lib/tests/kmat_test.cpp`)
- `CMakeLists.txt` (top-level — gtest FetchContent wired under `JUNO_FSW_TESTS`)
- `libs/CMakeLists.txt` (`add_subdirectory(kmat_lib)` added)

## Lessons Learned (this sprint)

Captured in:
- `ai/memory/lessons-learned-software-lead.md` (2026-05-04 Phase-0 gate-tool verification; 2026-05-04 cross-module overlap audit before launching design sprints)
- `ai/memory/lessons-learned-senior-software-engineer.md` (2026-05-04 EXPECT_TRUE template-comma preprocessor trap; 2026-05-04 freestanding-safe sqrt/abs via builtins)
- `ai/memory/lessons-learned-software-systems-engineer.md` (2026-05-04 layering on LibJuno upstream — re-export rather than duplicate primitive types/ops)

## Agent Count

12 agents (matches REV B revised SDP estimate):
- Phase 1A: 2 SSE workers + 2 MAE reviewers = 4
- Phase 1B: 3 SSE authors + 4 SSE reviewers = 7
- Phase 3.5: 1 MAE test-audit reviewer = 1
- Phase 4: 1 CE final gate = 1
- Total: **12 agents**, 8 Lead-direct atomic edit cascades

## Notable Worker Deviations (Approved)

- **`kPivotEpsilon` form** authored as C++14 variable template by Worker B1 first iteration; converted Lead-direct to C++11 function template per project standard.
- **`MatNear<T,R,C>` helper** authored with `std::memcmp` for byte-equality (`MatBitEqual` separate helper) — clean separation per author's discretion.
- **2 extra kinematics interop tests** beyond the 20 SW-TC-KMAT-* (covering `QuatNormalize+QuatToMat3` and `QuatRotate+MatVecMul` composition) — author judgment to strengthen MAT_T+QUAT+VEC interop coverage; non-blocking, captured as informational note in MAE test-audit.

## Successor Eligibility

**SPRINT-IMPL-02 (log_lib, Wave 1) is eligible to launch** per CE PASS verdict.
