---
sprint_id: SPRINT-IMPL-06-retro-A
module: libjuno (juno_sch_api.hpp) + sch_lib consumer ripple
wave: 2 (retrofit)
start_date: 2026-05-06
close_date: 2026-05-06
status: CLOSED
predecessors: SPRINT-IMPL-06 (sch_lib)
successors: Wave 3 entry (SPRINT-IMPL-07 imu_lib already authorized; this retro does not block)
---

# Sprint Closure Record — SPRINT-IMPL-06-retro-A LibJuno SCH App-Type Bug Fix

## 1. Sprint Goal

Rectify a LibJuno bug surfaced post-SPRINT-IMPL-06: `libjuno/include/juno/sch/juno_sch_api.hpp` declared its embedded schedule-table as `JUNO_APP_ROOT_T *tArrSchTable[NFrames][NAppsPerFrame]` (the **C** type from `app_api.h`), inconsistent with the surrounding C++-only header (templates, namespaces, `juno::time::TIME_ROOT_T &tTime` reference field). The C `JUNO_APP_API_T::OnProcess` signature `(*)(JUNO_APP_ROOT_T*)` is NOT memory-compatible with the C++ `juno::app::APP_API_T::OnProcess` shape `(&)(APP_ROOT_T &) noexcept`. After the fix, the scheduler's table holds C++ APP_ROOT_T pointers; dispatch is type-safe; the consumer pattern aligns with all FSW apps (which use `JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` per SDP §11 canonical names).

## 2. PM Approvals

PM approved 2026-05-06 with three Q&A dispositions, plus one mid-sprint clarification:
- **Q1 → Option A**: patch LibJuno locally; commit inside libjuno; bump submodule pointer.
- **Q2 → Option A**: sprint id = SPRINT-IMPL-06-retro-A.
- **Q3 → Option A**: replace `extern "C" { #include "juno/app/app_api.h" }` with C++ `#include "juno/app/app_api.hpp"`. C include becomes dead.
- **Mid-sprint correction (Phase 5)**: PM clarified that "libjuno is no longer a submodule and is managed directly by this repo." The .git/config still had stale `submodule.libjuno.*` entries from a prior submodule setup; libjuno's `.git` directory and build artifacts had been deleted by the user. Phase 5 reduced from "submodule commit + pointer bump" to "convert gitlink to vendored + clean stale config + stage the modified file as a regular tracked file."

## 3. Worker Invocations

| # | Task | Worker | File | Iter | Verdict |
|---|------|--------|------|-----:|---------|
| 1 | LibJuno header fix | senior-software-engineer | `libjuno/include/juno/sch/juno_sch_api.hpp` | 1 | APPROVED iter-1 |
| 2-6 | FSW consumer ripple (5 files) | Lead-direct (atomic edits per 2026-05-03 lesson) | `libs/sch_lib/src/common/sch_common.cpp`, `libs/sch_lib/tests/sch_test_helpers.hpp`, `libs/sch_lib/tests/sch_test.cpp`, `libs/sch_lib/src/posix/sch_posix.cpp`, `libs/sch_lib/src/pico2/sch_pico2.cpp` | 1 | senior-software-engineer integrated reviewer iter-1: NEEDS CHANGES (1 Major) → Lead-direct atomic close |

**Worker count**: 1 (LibJuno header).
**Reviewer count**: 1 LibJuno reviewer iter-1 (APPROVED) + 1 FSW integrated reviewer iter-1 (NEEDS CHANGES).
**Lead-direct edits**: 5 FSW consumer files in Phase 3; 1 atomic fix in iter-2 (sch_test.cpp:340 pointer→reference); Phase 5 git operations (rm cached gitlink + clean config + add modified file).
**CE invocations**: 1.
**Total**: 3 agents.

## 4. Reviewer Verdicts

| File / Phase | Iter 1 | Iter 2 |
|--------------|--------|--------|
| `juno_sch_api.hpp` (LibJuno header) | APPROVED — confirmed include cleanup, type rename, docstring, 5 LibJuno req-tags untouched, zero internal-LibJuno consumers depend on the removed C include | — |
| FSW consumer integrated review | NEEDS CHANGES — 1 Major: `sch_test.cpp:340` `OnStart(&tApp.tRoot)` passes pointer to a function reference declared as `(&OnStart)(APP_ROOT_T &)` (compile error). | n/a — Lead-direct atomic close (single-line edit `OnStart(&tApp.tRoot)` → `OnStart(tApp.tRoot)`); G1 build verified |

## 5. Gate Evidence (Phase 4)

### Gate G1 — POSIX build + ctest
```
$ cd build_posix && cmake --build . && ctest --output-on-failure
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
Test #9: device_lib_pico2_test ............   Passed    0.00 sec

100% tests passed, 0 tests failed out of 9
```

Inside `sch_test`: `[==========] 12 tests from 2 test suites ran. [  PASSED  ] 12 tests.` (11 SchTest + 1 SchEquivTest — coverage unchanged from SPRINT-IMPL-06).

### Gate G2 — Traceability + zero-delta counter check
```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       49   (delta from SPRINT-IMPL-06 close = 0)
  Requirements with @verify:    58   (delta from SPRINT-IMPL-06 close = 0)
```

**Pure type-rename confirmed**: zero traceability impact (49/58 unchanged from SPRINT-IMPL-06 close baseline). This is the AC-9 success criterion — the fix changes static type-checking only, not requirement coverage.

### Gate G3 — Pico2 own-object cross-compile
```
$ arm-none-eabi-g++ -c -std=c++11 [strict-flag-set] -ffreestanding -nostdlib \
    libs/sch_lib/src/common/sch_common.cpp -o /tmp/sch_common.o
RC=0; output: 2032 B (identical to SPRINT-IMPL-06 close)

$ arm-none-eabi-g++ -c <same flags> libs/sch_lib/src/pico2/sch_pico2.cpp -o /tmp/sch_pico2.o
RC=0; output: 1324 B (identical to SPRINT-IMPL-06 close)
```

**Identical .o byte counts** vs SPRINT-IMPL-06 close confirms type-rename is a no-op at the machine level (vtable indirection generates identical code; only the static type-checking changed).

## 6. Phase 5 Infrastructure Conversion

Mid-sprint, Phase 5 was redesigned per PM clarification ("libjuno is managed directly"):
- `git rm --cached libjuno` — removed broken gitlink (was at upstream commit `f71128e524d4aa27d1b62788d4fe481a14556e3d`).
- `git config --local --remove-section submodule.libjuno` — cleaned stale `.git/config` entries.
- `git add libjuno/include/juno/sch/juno_sch_api.hpp` — staged the sch fix as a regular tracked file.
- The user committed the result as `99b3f3b9c` (":sparkles: Fix sch lib") on `develop`.

**Pre-Phase-5 broken state surfaced**:
- `submodule.libjuno.url git@github.com:robinonsay/libjuno` and `submodule.libjuno.active true` in `.git/config`.
- `.gitmodules` listed only `picotool`, `pico-sdk`, `trick` — NO `libjuno` entry.
- `.git/modules/libjuno` did not exist.
- `libjuno/.git` did not exist.
- `git submodule status` errored with "no submodule mapping found in .gitmodules for path 'libjuno'".

The conversion (gitlink removal + config cleanup) resolves the broken state and reflects the true vendored model.

## 7. Chief Engineer Verdict

**PASS** (issued 2026-05-06).

> All AC met. LibJuno header fix at `juno_sch_api.hpp:167` confirmed (`juno::app::APP_ROOT_T *`); include block at line 74 (`app_api.hpp`); 5 LibJuno req-tags untouched. FSW consumer dispatch `OnProcess(*ptApp)` (reference) at `sch_common.cpp:135`. C++ vtable brace-list at `sch_test_helpers.hpp:135` lawfully binds to `juno::app::APP_API_T`'s reference-bound function-ref members (verified by clean build under `-Wall -Wextra -Werror -Wmissing-field-initializers`). 9/9 ctest PASS (12/12 gtest). Traceability counter delta 0/0 vs SPRINT-IMPL-06. libjuno gitlink removed; sch fix committed at `99b3f3b9c` as a regular tracked file.

## 8. Carry-Forwards

1. **`build_posix_ce/` build artifacts in commit `99b3f3b9c`**. The "Fix sch lib" commit accidentally included an entire 3000+-file `build_posix_ce/` directory (CMake-generated build artifacts from the SPRINT-IMPL-06 CE gate). Recommend a follow-up cleanup commit to: (a) `git rm -r build_posix_ce/`, (b) add `build*/` to the parent `.gitignore`. Out of this sprint's scope (history-rewriting territory; user discretion).

2. **libjuno vendoring scope**. This sprint vendored ONLY `libjuno/include/juno/sch/juno_sch_api.hpp` (the modified file). The rest of libjuno's source tree (~5 MB / ~1,353 source files; ~`include/`, `tests/`, `cmake/`, `templates/`, top-level CMake/Doxyfile/LICENSE/README) remains untracked in the parent repo. Recommend a dedicated infrastructure sprint to: (a) decide which libjuno paths to vendor (source-only vs source+docs vs everything), (b) add `.gitignore` rules for libjuno's build artifacts (`build/`, `build_cpp_test/`, `build_smoke/`, `vscode-extension/node_modules/`, `CMakeFiles/`, `Testing/`), (c) bulk `git add` the curated set.

3. **Pre-existing carry-forwards from SPRINT-IMPL-06** remain open and unchanged:
   - SW-REQ-SCH-007 source `@req` tag deferred to SPRINT-IMPL-25 composition root.
   - test_cases.json TC-004 + TC-007 acceptance-criteria wording inherited misalignment (SCOPE NOTE comments document the proportional-projection adaptation).
   - time_lib + log_lib pico-sdk flag-leak (G3 static-library link blocker; per-object compile is the documented evidence form).
   - NMEA_STATUS_INVALID_MSG_ERROR == JUNO_FSW_STATUS_NUMERIC_ERROR == 1001 numeric collision (pre-existing, disjoint domains, non-functional).

## 9. Lessons Learned

Updated `ai/memory/lessons-learned-software-lead.md` with three new entries:
- **2026-05-06 — LibJuno C++/C type mismatches in C++-only headers**: When auditing LibJuno C++ headers, grep for any `JUNO_*_T` (C-prefix) symbols and verify the surrounding context is genuinely C-friendly. Pure C++-only headers (templates, namespaces, references) should use the `juno::*::*_T` C++ types throughout.
- **2026-05-06 — Submodule infrastructure can be silently broken**: `.gitmodules` missing + `.git/modules/<name>` missing + `<name>/.git` missing + `submodule.<name>.*` still in `.git/config` → git treats the path as a gitlink but no operations on the submodule succeed. Surface the inconsistency early (Phase 0 pre-flight: `git submodule status` should exit 0).
- **2026-05-06 — C-vs-C++ vtable function-pointer signature mismatches catch only one call site at a time**: When converting a C-stub vtable to C++ reference-bound function refs, the reviewer caught one missed call site (`sch_test.cpp:340 OnStart(&tApp.tRoot)`) but might have missed others if there were more. Worker briefs converting C→C++ vtables MUST include a grep for ALL call sites in the calling TU and an explicit ref-vs-pointer audit.

## 10. Files Touched

| File | Type | Disposition |
|------|------|-------------|
| `libjuno/include/juno/sch/juno_sch_api.hpp` | LibJuno upstream | Worker iter-1 APPROVED; staged as regular file (vendored) |
| `libs/sch_lib/src/common/sch_common.cpp` | FSW impl | Lead-direct atomic edits; integrated review APPROVED |
| `libs/sch_lib/tests/sch_test_helpers.hpp` | FSW test infra | Lead-direct atomic edits + `static`→`inline` for `BuildTime`; integrated review APPROVED |
| `libs/sch_lib/tests/sch_test.cpp` | FSW test | Lead-direct atomic edits + iter-2 fix on line 340 (pointer→reference) |
| `libs/sch_lib/src/posix/sch_posix.cpp` | FSW POSIX impl | Lead-direct atomic edit (removed dead C-include block) |
| `libs/sch_lib/src/pico2/sch_pico2.cpp` | FSW Pico2 impl | Lead-direct atomic edit (removed dead C-include block) |

## 11. Wave 3 Authorization Status

Unchanged from SPRINT-IMPL-06 close (Wave 1+2 Exit Gate already issued PASS 2026-05-06). Wave 3 (SPRINT-IMPL-07 imu_lib) remains AUTHORIZED. This retro does not block Wave 3 entry.
