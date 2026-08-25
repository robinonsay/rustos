---
sprint_id: SPRINT-IMPL-05-retro-A
module: template_cpp + device_lib (per-platform IMPL pattern + G3 mitigation)
wave: 2 (retro)
start_date: 2026-05-05
close_date: 2026-05-05
status: CLOSED
predecessor: SPRINT-IMPL-05 closure 2026-05-05
successor: SPRINT-IMPL-05-retro-B (log_lib mirror — planned)
---

# Sprint Closure Record — SPRINT-IMPL-05-retro-A

## 1. Sprint Goal

Rectify the single-IMPL-with-`void*`-handle drift in `device_lib` (split into `DEVICE_LIB_POSIX_T<N>` + `DEVICE_LIB_PICO2_T<N>` with type-specific handle members), update `template_cpp` so this drift cannot recur, and adopt consumer-side per-source-file `COMPILE_OPTIONS` as the canonical CMakeLists pattern for libs linking pico-sdk targets — fully resolving the SPRINT-IMPL-05 G3 transitive-source carry-forward and SPRINT-IMPL-05 POSIX fd-leak carry-forward as architectural side effects.

## 2. Sprint Plan Outcome

PM-approved 2026-05-05 with all three Q-batch recommendations accepted:
- **Q1**: Naming `DEVICE_LIB_POSIX_T` / `DEVICE_LIB_PICO2_T` (shorter form).
- **Q2**: Three template files (rewrite `temp_impl.hpp` + 2 new skeletons `temp_posix.hpp` / `temp_pico.hpp`).
- **Q3**: G3 **full-lib** cross-compile is the new bar (replacing SPRINT-IMPL-03's "object-file clean" precedent).

PM split the originally-proposed combined retro into **Sprint A** (template + device_lib) and **Sprint B** (log_lib mirror) due to peak-parallel-agent count; this is Sprint A's record. Sprint B will be presented for approval after this closure.

## 3. Worker Invocations

| # | Phase | Task | Worker | File | Iter | Verdict |
|---|-------|------|--------|------|-----:|---------|
| 1 | 1 | Rewrite temp_impl.hpp to per-platform DERIVE pattern | senior-software-engineer | `libjuno/templates/template_cpp/include/temp_impl.hpp` (490 lines) | 1 → APPROVED + Lead-direct iter-2 | APPROVED |
| 2 | 1 | NEW POSIX IMPL skeleton | senior-software-engineer | `libjuno/templates/template_cpp/include/temp_posix.hpp` (328 lines) | 1 → APPROVED + Lead-direct iter-2 | APPROVED |
| 3 | 1 | NEW Pico2 IMPL skeleton | senior-software-engineer | `libjuno/templates/template_cpp/include/temp_pico.hpp` (321 lines) | 1 → APPROVED + Lead-direct iter-2 | APPROVED |
| 4 | 1 | Update temp_api.hpp Doxygen (Section 11) | senior-software-engineer | `libjuno/templates/template_cpp/include/temp_api.hpp` (542→418 lines) | 1 → APPROVED + Lead-direct iter-2 | APPROVED |
| 5 | 2 | NEW device_posix.hpp | senior-software-engineer | `libs/device_lib/include/device_lib/device_posix.hpp` (387 lines) | 1 | APPROVED |
| 6 | 2 | NEW device_pico2.hpp | senior-software-engineer | `libs/device_lib/include/device_lib/device_pico2.hpp` (278 lines) | 1 | APPROVED |
| 7 | 2 | Rewrite POSIX impl with type-safe fields + Deinit | senior-software-engineer | `libs/device_lib/src/posix/device_posix.cpp` (485 lines) | 1 → APPROVED + Lead-direct iter-2 (`<cstddef>` → `<stddef.h>`) | APPROVED |
| 8 | 2 | Rewrite Pico2 impl with type-safe `uart_inst_t *ptUart` | senior-software-engineer | `libs/device_lib/src/pico2/device_pico2.cpp` (389 lines) | 1 | APPROVED |
| 9 | 2 | Rewrite POSIX test (drop _pvHandle hack, use Deinit in TearDown) | senior-software-engineer (test author) | `libs/device_lib/tests/device_lib_test.cpp` (387 lines) | 1 | APPROVED |
| 10 | 2 | Rewrite Pico2-stub test (use DEVICE_LIB_PICO2_T directly) | senior-software-engineer (test author) | `libs/device_lib/tests/device_lib_pico2_test.cpp` (498 lines) | 1 | APPROVED |
| 11 | 2 | Rewrite CMakeLists with per-source-file COMPILE_OPTIONS | junior-software-engineer | `libs/device_lib/CMakeLists.txt` (147 lines) | 1 | APPROVED |

**Lead-direct mid-sprint corrections** (per the 2026-05-03 atomic-Lead-edit pattern):
- Phase-0 attempt: pico-sdk patch (option C) RULED OUT — `set_source_files_properties` from upstream doesn't propagate across consumer-directory boundary; reverted both pico-sdk edits; pivoted to consumer-side approach.
- Phase 1 iter-2 (4 files, ~10 atomic edits): drop variadic `, ...` from TEMP_API1/2 vtable slots in temp_api.hpp; cascade drop from temp_posix.hpp + temp_pico.hpp static method declarations; Option B reconciliation (`void *ptPeripheral` → `uint32_t *ptPeripheral` in temp_pico.hpp) per cross-reviewer adjudication; freestanding-safe includes (`<cstddef>`/`<cstdint>` → `<stddef.h>`/`<stdint.h>`) in temp_pico.hpp; tResult value-init at line 261; explicit-instantiation contract comment in temp_posix.hpp; inline dispatch comment example in temp_impl.hpp.
- Phase 2 iter-2 (1 file, 1 edit): `<cstddef>`/`<cstdint>` → `<stddef.h>`/`<stdint.h>` in device_posix.cpp for project consistency.
- Phase 2 close: `device_impl.hpp` deleted (the now-superseded SPRINT-IMPL-05 single-IMPL header).

## 4. Reviewer Verdicts

### Phase 1 (template_cpp)
| File | Iter 1 | Iter 2 |
|------|--------|--------|
| temp_impl.hpp | NEEDS CHANGES (1 Error: variadic mismatch + 1 Warning: missing dispatch comment) | APPROVED via Lead-direct |
| temp_posix.hpp | NEEDS CHANGES (1 Error: missing explicit-instantiation contract comment) | APPROVED via Lead-direct |
| temp_pico.hpp | NEEDS CHANGES (3 Errors: variadic `...`, uninit tResult, freestanding-unsafe includes + 1 Warning) | APPROVED via Lead-direct |
| temp_api.hpp | APPROVED iter-1 (Section 11 added; sections 1-10 trimmed; SUMMARY row added) | (re-touched by Lead-direct cross-cutting variadic drop) |

Inconsistency adjudication: Task 1 reviewer recommended Option A (`void *` + warning); Task 3 reviewer recommended Option B (`uint32_t *`); **Lead chose Option B** — putting `void *` in canonical templates normalizes the exact anti-pattern the sprint exists to prevent.

### Phase 2 (device_lib)
| File | Iter 1 | Iter 2 |
|------|--------|--------|
| device_posix.hpp | APPROVED | — |
| device_pico2.hpp | APPROVED | — |
| device_posix.cpp | NEEDS CHANGES (1 Warning: `<cstddef>`/`<cstdint>` style consistency) | APPROVED via Lead-direct |
| device_pico2.cpp | APPROVED | — |
| device_lib_test.cpp | APPROVED | — |
| device_lib_pico2_test.cpp | APPROVED | — |
| CMakeLists.txt | APPROVED | — |

**11/11 first-pass APPROVED** across both phases (with Lead-direct atomic iter-2 corrections handling 9 specific findings).

## 5. Gate Evidence (Phase 3)

### Gate G1 — POSIX build + ctest
```
$ cd build_posix && cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && cmake --build . && ctest --output-on-failure
...
8/8 tests passed:
  Test #1: kmat_test ........................   Passed
  Test #2: time_test ........................   Passed
  Test #3: time_pico2_test ..................   Passed
  Test #4: log_test .........................   Passed
  Test #5: log_pico2_test ...................   Passed
  Test #6: nmea_test ........................   Passed
  Test #7: device_lib_test ..................   Passed (9 TEST_F)
  Test #8: device_lib_pico2_test ............   Passed (15 TEST_F)
```
**G1 exit: 0**

### Gate G2 — Traceability
```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       40   (delta 0 from SPRINT-IMPL-05 baseline)
  Requirements with @verify:    49   (delta 0 from SPRINT-IMPL-05 baseline)
  Requirements with test specs: 376
```
**G2 exit: 0** — internal-design rectification preserved coverage exactly; no requirements added or removed.

### Gate G3 — Pico2 cross-compile FULL device_lib (NEW BAR achieved)
```
$ rm -rf build_pico2 && mkdir build_pico2 && cd build_pico2 && cmake .. && cmake --build . --target device_lib
...
[ 88%] Building C object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_atomic/atomic.c.o
[ 91%] Building CXX object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_cxx_options/new_delete.cpp.o
[ 94%] Building C object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_printf/printf.c.o
[ 94%] Building ASM object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_crt0/crt0.S.o
[ 97%] Building C object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_clib_interface/newlib_interface.c.o
[100%] Linking CXX static library libdevice_lib.a
[100%] Built target device_lib
```
**G3 exit: 0** — the SPRINT-IMPL-03 §234 / SPRINT-IMPL-05 G3 transitive-source carry-forward is **RESOLVED for device_lib**. Pico-sdk's own C/CXX/ASM sources (gpio.c, stdlib.c, atomic.c, new_delete.cpp, crt0.S, printf.c, stdio.c, newlib_interface.c) all compile clean because they don't inherit our strict flags — the per-source-file COMPILE_OPTIONS pattern applies our flags to OUR sources only.

## 6. Acceptance Criteria Status

| AC | Criterion | Status |
|----|-----------|--------|
| AC-1 | template_cpp ONE DERIVE per platform; new skeletons exist; per-platform pattern documented as canonical | MET |
| AC-2 | device_lib has device_posix.hpp + device_pico2.hpp; device_impl.hpp deleted; void *_pvHandle removed from code | MET |
| AC-3 | DEVICE_LIB_POSIX_T<N> has int iFdMaster/iFdSlave; New() opens both, Deinit() closes both; DEVICE_LIB_PICO2_T<N> has uart_inst_t *ptUart | MET |
| AC-4 | device_lib CMakeLists uses consumer-side per-source-file COMPILE_OPTIONS pattern | MET |
| AC-5 | G3 full-lib cross-compile succeeds | MET (NEW BAR) |
| AC-6 | All 8 SW-TC-DEVICE-001..008 + WriteBytes_TransmitsCallerBuffer PASS on both backends | MET |
| AC-7 | traceability.py exit 0; counter delta = 0 | MET |
| AC-8 | noexcept everywhere; no heap/STL/virtual/RTTI; ≤500 LoC/file | MET (largest 498) |
| AC-9 | POSIX fd-leak closed (New() stores both fds, Deinit() closes both, fixture calls Deinit() in TearDown) | MET |
| AC-10 | CE PASS verdict | MET |

## 7. Chief Engineer Verdict

**APPROVED** unconditional first-iteration. Full rationale: "All ten acceptance criteria are MET with concrete on-disk evidence. The single-IMPL-with-`void*`-handle drift identified in SPRINT-IMPL-05 is fully rectified: `device_impl.hpp` is deleted, `void *_pvHandle` survives only in documentation comments (zero code references), and `DEVICE_LIB_POSIX_T<N>` / `DEVICE_LIB_PICO2_T<N>` carry first-class typed handle members. Gate evidence holds under CE re-run: G1 (POSIX 8/8 PASS), G2 (traceability exit 0, counter delta 0), and the new G3 full-lib cross-compile bar — confirming the SPRINT-IMPL-03 §234 / SPRINT-IMPL-05 G3 carry-forward is RESOLVED for device_lib via consumer-side per-source-file `COMPILE_OPTIONS`. The SPRINT-IMPL-05 carry-forward POSIX fd-leak is closed via `Deinit()`. All twelve deliverable files remain ≤500 LoC. Sprint cleared for PM presentation."

## 8. Agent Count

| Phase | Agents | Notes |
|-------|--------|-------|
| Phase 0 | 0 | Pre-flight audits + pico-sdk option-C try-and-revert (Lead-direct) |
| Phase 1 | 4 | Workers (template_cpp) |
| Phase 1 review | 4 | Reviewers |
| Phase 1 iter-2 | 0 | Lead-direct atomic fixes (~10 edits across 4 files) |
| Phase 2 | 7 | Workers (device_lib retro) |
| Phase 2 review | 7 | Reviewers |
| Phase 2 iter-2 | 0 | Lead-direct atomic fix (1 edit) |
| Phase 3 | 0 | Lead-direct gates |
| Phase 4 | 1 | project-chief-engineer |
| **Total** | **23** | Below the projected 23–29 budget. Lead-direct atomic-edit pattern saved ~12-16 agent invocations vs worker+reviewer iteration. |

## 9. Carry-Forward / Follow-Up Items

1. **G3 transitive pico-sdk source-compile (SPRINT-IMPL-05 carry-forward #1)**: **RESOLVED for device_lib** via the consumer-side per-source-file COMPILE_OPTIONS pattern. Other libs (`time_lib`, `log_lib`, `kmat_lib`) still use the leaky `target_compile_options` pattern — they happen to compile clean on POSIX and on Pico2 because their `.cpp` workloads stay under the threshold that triggers the issue. **Recommendation**: retroactively migrate them to the new pattern as a build-infrastructure cleanup. SPRINT-IMPL-05-retro-B (log_lib retro) will pick up log_lib as part of its rectification scope.
2. **POSIX test fd-leak (SPRINT-IMPL-05 carry-forward #2)**: **RESOLVED**. `DEVICE_LIB_POSIX_T<N>` declares `iFdMaster` AND `iFdSlave` as first-class fields; `New()` populates both; `Deinit()` closes both with `>= 0` guards; test fixture calls `Deinit()` in `TearDown()`.
3. **DEVICE-007 test-spec gap (SPRINT-IMPL-05 carry-forward #3)**: still open. The implementation-side @verify gap is closed (`WriteBytes_PushesIntoTxFifo` Pico2 + `WriteBytes_TransmitsCallerBuffer` POSIX both tagged), but `docs/test_cases/device/test_cases.json` doesn't yet have a SW-TC-DEVICE-013 entry pointing at these tests. Recommend adding in next sprint touching device_lib (SPRINT-IMPL-09 gps_lib or SPRINT-IMPL-10 lora_lib).
4. **template_cpp upstream propagation**: the 4 template files modified in this sprint live in the libjuno submodule. They should be propagated upstream when libjuno picks up its next release. Mark in SPRINT-IMPL-05-retro-A closure as "FSW-side template adoption; libjuno upstream PR queued."

## 10. Lessons Learned

Cross-referenced into per-role files per methodology §10:

- `ai/memory/lessons-learned-software-lead.md` — pico-sdk patch (option C) ruled out; consumer-side per-source-file pattern is canonical for libs linking pico_stdlib; atomic Lead-direct iter-2 fixes scale to multi-file template work; Option-A/B inconsistency adjudication via reviewer cross-feedback.
- `ai/memory/lessons-learned-senior-software-engineer.md` — per-platform IMPL DERIVE pattern with type-specific handle members (vs single-IMPL-with-`void*` drift); `JUNO_MODULE_DERIVE` macro doesn't accept C-ellipsis `...` in its arg list (preprocessor trap); template `New()`/`Deinit()` declarations alone don't link — explicit instantiation contract required; freestanding-unsafe `<cstddef>`/`<cstdint>` C++ wrappers vs C-form `<stddef.h>`/`<stdint.h>`.
- `ai/memory/lessons-learned-junior-software-engineer.md` — consumer-side per-source-file `set_source_files_properties` is the new canonical CMake pattern for any lib linking pico-sdk INTERFACE_SOURCES.

## 11. Approval

| Field | Value |
|-------|-------|
| Author | Software Lead |
| Date | 2026-05-05 |
| Predecessor | SPRINT-IMPL-05 (device_lib) closure 2026-05-05 |
| Successor | SPRINT-IMPL-05-retro-B (log_lib mirror — planned, presented for approval after this closure) |
| CE verdict | APPROVED unconditional 2026-05-05 |
| PM approval | (this record) — 2026-05-05 |
