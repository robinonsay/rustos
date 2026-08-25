---
sprint_id: SPRINT-IMPL-06
module: sch_lib
wave: 2
start_date: 2026-05-06
close_date: 2026-05-06
status: CLOSED
predecessors: SPRINT-IMPL-03 (time_lib)
successors: Wave 1+2 Exit Gate; then Wave 3 (sensor libs)
---

# Sprint Closure Record — SPRINT-IMPL-06 sch_lib

## 1. Sprint Goal

Implement the FT1 platform impls of LibJuno's `juno::sch::SCH_API_T<8, 200>` cyclic-executive scheduler per L2 design [`docs/design/sch/design.md`](../design/sch/design.md), retiring 9 of 10 SW-REQ-SCH-NNN IDs (SW-REQ-SCH-007 deferred to composition-root SPRINT-IMPL-25). Structurally dual-impl (POSIX + Pico2 vtable instances) but algorithmically single-impl (shared `Execute` body in `sch_common.cpp`); no pico-sdk surface (pacing delegated to time_lib's SleepTo backend per Q2=A PM disposition).

## 2. PM Approvals

PM approved 2026-05-06 with three Q&A dispositions:
- **Q1 → Option A**: `libs/sch_lib/` path used (matches all other FSW libs); SDP/design `libs/sch/` was treated as a minor naming inconsistency. Lead-direct fix to 12 stale `google_test_ref` entries in `docs/test_cases/sch/test_cases.json` (`libs/juno_sch/tests/juno_sch_test.cpp` → `libs/sch_lib/tests/sch_test.cpp`).
- **Q2 → Option A (skip §5.1 Pico2 stub bundle)**: sch_lib is structurally dual-impl but algorithmically single-impl per L2 §3 / §8.3. Zero direct pico-sdk surface — no stub bundle needed. SW-TC-SCH-009 byte-equivalence verified by linking BOTH platform .cpp files into one `sch_test` binary and `memcmp`'ing recorded invocation traces.
- **Q3 → Option A**: Single shared `Execute` body in `sch_common.cpp`; platform `.cpp` files hold only the `static const SCH_API_T<8, 200> tApi{...}` literal + a public `juno::fsw::sch::<platform>::GetApi()` accessor.

## 3. Worker Invocations

| # | Task | Worker | File | Iter | Verdict |
|---|------|--------|------|-----:|---------|
| 0a | Test-cases.json path rewrite + 2 procedure-prose updates | Lead-direct | `docs/test_cases/sch/test_cases.json` | — | clean |
| 1 | Shared Execute + GetMinor + GetMajor | senior-software-engineer | `libs/sch_lib/src/common/sch_common.cpp` (273→288 lines) | 1 | APPROVED iter-1 |
| 2 | POSIX vtable + seam header pair | senior-software-engineer | `libs/sch_lib/src/posix/sch_posix.{hpp,cpp}` (86 + 153) | 1 | APPROVED iter-1 |
| 3 | Pico2 vtable + seam header pair | senior-software-engineer | `libs/sch_lib/src/pico2/sch_pico2.{hpp,cpp}` (102 + 141) | 1 | APPROVED iter-1 |
| 4 | Google Test (initial 736-line monolithic) | senior-software-engineer (test author) | `libs/sch_lib/tests/sch_test.cpp` (736 lines) | 1 | NEEDS CHANGES (size cap + TC-004 + TC-009 tag) |
| 4-iter2 | 3-file split (helpers + main + equiv) per Option A+B | senior-software-engineer (test author iter-2) | `tests/sch_test_helpers.hpp` (171), `tests/sch_test.cpp` (460→473), `tests/sch_equiv_test.cpp` (109) | 2 | NEEDS CHANGES iter-2 (TC-007 SCOPE NOTE missing) → APPROVED via Lead-direct atomic close |
| 5 | CMakeLists.txt | junior-software-engineer | `libs/sch_lib/CMakeLists.txt` (131 lines) | 1 | APPROVED iter-1 |

**Lead-direct edits** (per 2026-05-03 atomic-edit pattern):
- Phase 0a: 12 google_test_ref path rewrites + 2 inspection-procedure prose updates in `docs/test_cases/sch/test_cases.json`.
- Phase 2 close: added TC-007 SCOPE NOTE comment block above `AppFailure_ContinuesOtherApps` (mirrors TC-004 pattern; documents the same FT1 1000ms major-frame proportional-projection rationale per 2026-05-05 inherited-misalignment lesson).
- Phase 3 G1: replaced `JUNO_ASSERT_EXISTS(tSch.ptApi)` in `SchGetMinor/MajorFramePeriod` with explicit RESULT_T-shaped null-checks (the macro returns a bare `JUNO_STATUS_NULLPTR_ERROR` int, which mismatches the `RESULT_T<JUNO_TIMESTAMP_T>` return type).
- Phase 3 G1: registered `add_subdirectory(sch_lib)` in `libs/CMakeLists.txt` (was missing — sch_lib wasn't being built).
- Phase 3 G1: `sed` rename `static` → `inline` for 5 helpers in `sch_test_helpers.hpp` (CountingAppOnStart/OnProcess/OnExit, InitCountingApp, TestFailureHandler) — `static` linkage in a header included by two test TUs trips `-Werror=unused-function` in TUs that don't reference each helper. `inline` in headers is the canonical C++11 portable solution.
- Phase 3 CMakeLists update: added `tests/sch_equiv_test.cpp` to `SCH_TEST_SOURCES`.

## 4. Reviewer Verdicts

| File | Iter 1 | Iter 2 |
|------|--------|--------|
| sch_common.cpp | APPROVED | — |
| sch_posix.{hpp,cpp} | APPROVED | — |
| sch_pico2.{hpp,cpp} | APPROVED | — |
| **sch_test.cpp (736 lines)** | NEEDS CHANGES (3 findings: size, TC-004 dead code, TC-009 tag misplacement) | n/a (worker iter-2 produced 3 split files) |
| **sch_test split (helpers + test + equiv)** | n/a | NEEDS CHANGES (1 finding: TC-007 SCOPE NOTE missing) → Lead-direct close |
| CMakeLists.txt | APPROVED | — |

**Total reviewer agent count**: 5 iter-1 + 1 iter-2 = 6.
**Total worker agent count**: 5 iter-1 + 1 iter-2 (test) = 6.
**Total CE invocations**: 1.
**Total Phase 0/3 Lead-direct atomic edits**: ~20 (test_cases.json path/prose, RESULT_T null-checks, libs/CMakeLists.txt registration, helpers static→inline, CMake test-source list, TC-007 SCOPE NOTE).

## 5. Gate Evidence

### Gate G1 — POSIX build + ctest
```
$ cd /home/juno/juno_fsw && rm -rf build_posix && mkdir build_posix && cd build_posix
$ cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && cmake --build . && ctest --output-on-failure
...
[ 53%] Built target sch_test
...
Test #1: kmat_test ........................   Passed    0.00 sec
Test #2: sch_test .........................   Passed    0.00 sec
Test #3: time_test ........................   Passed    0.12 sec
Test #4: time_pico2_test ..................   Passed    0.00 sec
Test #5: log_test .........................   Passed    0.01 sec
Test #6: log_pico2_test ...................   Passed    0.00 sec
Test #7: nmea_test ........................   Passed    0.00 sec
Test #8: device_lib_test ..................   Passed    0.15 sec
Test #9: device_lib_pico2_test ............   Passed    0.01 sec

100% tests passed, 0 tests failed out of 9
```

Inside `sch_test`: `[==========] 12 tests from 2 test suites ran. [  PASSED  ] 12 tests.` (11 SchTest covering SW-TC-SCH-001..-008, -010..-012 + 1 SchEquivTest covering SW-TC-SCH-009).

### Gate G2 — Traceability check
```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       49   (delta +9 from baseline 40)
  Requirements with @verify:    58   (delta +10 from baseline 48)
  Requirements with test specs: 376
```

**Counter-delta sanity check** (per 2026-05-05 SPRINT-IMPL-05 lesson): expected +10 code / +10 @verify; actual +9 / +10. The single-ID code gap is **structurally correct**: SW-REQ-SCH-007 (Application Lifecycle Start Invocation) is implemented in the composition root (apps/main.cpp authored at SPRINT-IMPL-25), not in sch_lib. The scheduler's `SchExecute` enforces the contract by precondition (the table's apps must have OnStart called before first Execute) but does not itself invoke OnStart. Test contract is verified (TC-008 has `@verify` for SW-REQ-SCH-007). See carry-forward §7.

### Gate G3 — Pico2 cross-compile (per-object evidence)
```
$ arm-none-eabi-g++ -c -std=c++11 -O1 -fPIC -Wall -Wextra -Werror -pedantic -Wshadow \
    -Wcast-align -Wundef -Wswitch -Wswitch-default -Wmissing-field-initializers \
    -fno-common -fno-strict-aliasing -fno-rtti -fno-exceptions -ffreestanding -nostdlib \
    -I libjuno/include -I libs/sch_lib/src/common -I libs/sch_lib/src/pico2 \
    -I libs/time_lib/src/pico2 \
    libs/sch_lib/src/common/sch_common.cpp -o /tmp/sch_common.o
RC=0; output: /tmp/sch_common.o (2032 B)

$ arm-none-eabi-g++ -c <same flags> libs/sch_lib/src/pico2/sch_pico2.cpp -o /tmp/sch_pico2.o
RC=0; output: /tmp/sch_pico2.o (1324 B)
```

Static-library link via `cmake --build build_pico2 --target sch_lib` fails on the pre-existing time_lib pico-sdk-flag-leak issue (`stdlib.c: -fno-rtti is valid for C++ but not for C`) — see carry-forward §7. Per the 2026-05-05 SPRINT-IMPL-05 lesson, per-object compile of the sprint's own TUs is sufficient G3 evidence.

## 6. Chief Engineer Verdict

**PASS** (issued 2026-05-06).

> All 9 deliverables present at expected paths; line counts within budget (largest file `sch_test.cpp` = 473, all ≤ 500). Canonical dispatch confirmed at `sch_common.cpp:109` (`tSch.tTime.ptApi->Now`), `:138` (`ptApp->ptApi->OnProcess(ptApp)`), `:172` (`tSch.tTime.ptApi->SleepTo(tSch.tTime, tNextMinor)`). `AddTime` 2-arg form at `sch_common.cpp:158, 273` matches `time_api.hpp:137`. All 12 `@verify` tags on standalone `//` lines immediately above TEST_F. Source `@req` tags use `SW-REQ-SCH-NNN` consistently. Sprint cleared for closure and presentation to the Project Manager.

## 7. Carry-Forwards

1. **SW-REQ-SCH-007 source `@req` tag → SPRINT-IMPL-25**. The Application Lifecycle Start Invocation requirement is implemented in the composition root (`apps/main.cpp`), not in sch_lib. The scheduler's contract for SCH-007 is enforced by precondition (Doxygen-documented in `SchExecute`'s preconditions). Test `@verify` already in place at `tests/sch_test.cpp:325`. SPRINT-IMPL-25 will tag the composition root's OnStart loop with `SW-REQ-SCH-007`, closing the +10/+10 expected counter-delta.

2. **test_cases.json TC-004 + TC-007 acceptance-criteria wording**. Both TCs say "after 100 ms, exactly 10 invocations." That phrasing predates the LibJuno `SCH_ROOT_T<NAppsPerFrame, NFrames>` template finalization at `<8, 200>`, which makes `Execute()` a non-interruptible single 1000 ms major frame. SCOPE NOTE comment blocks present at `tests/sch_test.cpp:180-189` (TC-004) and `:278-291` (TC-007) document the proportional-projection adaptation. **Recommended action**: future RTM-cleanup sprint amends test_cases.json wording to "after 1000 ms major frame, exactly 100 invocations" or templatizes `SchExecute` for `<8, 20>` to enable the exact 100ms/10-invocation case. Per 2026-05-05 inherited-misalignment lesson, mid-sprint scope creep avoided.

3. **time_lib pre-existing pico-sdk flag-leak (carry-forward from SPRINT-IMPL-05 / 2026-05-05 lesson)**. time_lib's `target_compile_options` leaks `-fno-rtti` onto pico-sdk's transitive C sources (`stdlib.c`), breaking full G3 static-library link of any consumer (sch_lib reproduces this). **Recommended action**: dedicated infrastructure-cleanup sprint to retroactively patch `time_lib/CMakeLists.txt` and `log_lib/CMakeLists.txt` to use the `set_source_files_properties` per-source pattern adopted in SPRINT-IMPL-05-retro-A and SPRINT-IMPL-06. Not blocking for sch_lib closure since the sprint's own object compile is clean.

4. **Inherited stale `JUNO_SCH_T` / `juno_sch.h` references in test_cases.json prose**. Phase 0a fixed 2 of these (lines 170, 188); 2 more remain at lines 10, 186 in informal `setup` prose ("initialize a JUNO_SCH_T scheduler root"). These are descriptive-only (not file paths) and do not affect implementation; flagged for the same future RTM-cleanup sprint as carry-forward #2.

## 8. Lessons Learned

Updated `ai/memory/lessons-learned-software-lead.md` with one new entry:
- **2026-05-06 — SPRINT-IMPL-06**: brief-template-grep extends to LibJuno macro-vs-return-type compatibility — `JUNO_ASSERT_EXISTS` returns `JUNO_STATUS_NULLPTR_ERROR` (a bare int), which is incompatible with functions returning `RESULT_T<...>`. Future briefs should explicitly check the return type of every function calling `JUNO_ASSERT_EXISTS` and substitute an explicit `RESULT_T`-shaped null-check when the function returns `RESULT_T<>`. Add to common review traps.

## 9. Agent Count

| Phase | Agents |
|-------|-------:|
| 0 (Lead-direct test_cases.json) | 0 |
| 1 (workers iter-1) | 5 |
| 2 (reviewers iter-1) | 5 |
| Iter-2 (test author + reviewer) | 2 |
| Lead-direct (Phase 2 close + Phase 3 fixes) | 0 |
| 4 (CE) | 1 |
| **Total** | **13** |

In line with the 2026-05-05 lesson's projected agent count (13-18 baseline for Wave 2 dual-impl libs that DON'T link pico_stdlib directly — sch_lib here, since pacing is delegated to time_lib).

## 10. Wave 1+2 Exit Gate

To be invoked separately as the next CE invocation per [foundation_libs.md §4](../sdp/foundation_libs.md).
