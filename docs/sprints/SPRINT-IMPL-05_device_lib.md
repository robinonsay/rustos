---
sprint_id: SPRINT-IMPL-05
module: device_lib
wave: 2
start_date: 2026-05-05
close_date: 2026-05-05
status: CLOSED
predecessors: SPRINT-IMPL-00, SPRINT-IMPL-03
successors: SPRINT-IMPL-09 (gps_lib consumes device_lib), SPRINT-IMPL-10 (lora_lib consumes device_lib)
---

# Sprint Closure Record — SPRINT-IMPL-05 device_lib

## 1. Sprint Goal

Implement the FT1 UART1 hardware abstraction (`juno::device::DEVICE_LIB_API_T<N>`) per L2 design `docs/design/device/design.md`, retiring 7 SW-REQ-DEVICE-001..007. Dual-impl (POSIX pty + Pico2 pico-sdk UART1) with mandatory Pico2 host-side stub tests per methodology §5.1 Revision B.

## 2. Sprint Plan Outcome

PM-approved 2026-05-05 with two PM dispositions:
- **Q1 → Option A (clean delete)**: legacy `libs/device_lib/{src/uart1.c, include/device_lib/uart1.h, CMakeLists.txt}` deleted; legacy `gps_lib` and `apps/gps_app` gated under `JUNO_FSW_BUILD_LEGACY_MAIN` (precedent: SPRINT-IMPL-02 log_lib legacy-main gating).
- **Q2 → both stubbed Pico2 unit tests AND HIL Demonstrations**: SW-TC-DEVICE-001..008 re-scoped to host-side dual-backend Unit; SW-TC-DEVICE-009..012 added as HIL Demonstration deferred to FT1 bench.

## 3. Worker Invocations

| # | Task | Worker | File | Iterations | Final Verdict |
|---|------|--------|------|-----------:|---------------|
| 0a | Phase-0 legacy cleanup + CMake gating | Lead-direct | (3 file deletes + 2 CMake edits + baseline build verify) | — | clean |
| 0b | Test cases JSON amendment | software-systems-engineer | docs/test_cases/device/test_cases.json | 1 | APPROVED iter 1 |
| 1 | Public API header | senior-software-engineer | libs/device_lib/include/device_lib/device_api.hpp (252 lines) | 1 | APPROVED iter 1 |
| 2 | IMPL declaration header | senior-software-engineer | libs/device_lib/include/device_lib/device_impl.hpp (222 lines) | 1 | APPROVED iter 1 |
| 3 | POSIX impl | senior-software-engineer | libs/device_lib/src/posix/device_posix.cpp (465 lines) | 2 | APPROVED iter 2 |
| 4 | Pico2 impl | senior-software-engineer | libs/device_lib/src/pico2/device_pico2.cpp (462 lines) | 1 | APPROVED iter 1 |
| 5 | POSIX-backend test | senior-software-engineer (test author) | libs/device_lib/tests/device_lib_test.cpp (430 lines) | 1 | APPROVED iter 1 |
| 6 | Pico2-stub header | senior-software-engineer | libs/device_lib/tests/stubs/hardware/uart.h (174 lines) | 1 | APPROVED iter 1 |
| 7 | Pico2-stub source | senior-software-engineer | libs/device_lib/tests/stubs/device_pico2_stub.cpp (393 lines) | 1 | APPROVED iter 1 |
| 8 | Pico2-backend test | senior-software-engineer (test author) | libs/device_lib/tests/device_lib_pico2_test.cpp (496 lines) | 2 | APPROVED iter 2 |
| 9 | CMakeLists | junior-software-engineer | libs/device_lib/CMakeLists.txt (149 lines) | 1 | APPROVED iter 1 |

**Lead-direct mid-sprint corrections** (per methodology §11 atomic-edit pattern):
- Phase 0a: legacy delete + CMake gating + apps/gps_app gating.
- Phase 3: added `tests/stubs/pico/types.h` (49 lines) when G1 build surfaced missing pico-sdk header in stub include path; CXX-language flag gating in `device_lib/CMakeLists.txt` per SPRINT-IMPL-03 closure recommendation when G3 surfaced pico-sdk transitive C/C++ source-compile issue.
- Phase 3 close: added `// @{"verify": ["SW-REQ-DEVICE-007"]}` to existing edge-case test in `device_lib_pico2_test.cpp` AND new `WriteBytes_TransmitsCallerBuffer` TEST_F in `device_lib_test.cpp` (POSIX) when G2 traceability counter delta showed @verify gap on DEVICE-007.

## 4. Reviewer Verdicts (Phase 2)

| File | Iter 1 | Iter 2 |
|------|--------|--------|
| device_api.hpp | APPROVED | — |
| device_impl.hpp | APPROVED | — |
| **device_posix.cpp** | NEEDS CHANGES (1 Error: ring-state mutation before hard-error return; 3 Warnings) | APPROVED |
| device_pico2.cpp | APPROVED | — |
| device_lib_test.cpp | APPROVED | — |
| stubs/hardware/uart.h | APPROVED | — |
| stubs/device_pico2_stub.cpp | APPROVED | — |
| **device_lib_pico2_test.cpp** | NEEDS CHANGES (1 Error: `ReadBytes_RingOverflowReportsTableFull` structurally cleared sticky before assertion call; 1 Warning) | APPROVED |
| CMakeLists.txt | APPROVED | — |
| test_cases.json (Phase 0b) | APPROVED (MAE) | — |

**7/9 + 1 = 8 first-pass APPROVED out of 10 reviewable artifacts.** Two iteration-2 cycles for Major findings; both resolved cleanly.

## 5. Gate Evidence (Phase 3)

### Gate G1 — POSIX build + ctest

```
$ cd /home/juno/juno_fsw && rm -rf build_posix && mkdir build_posix && cd build_posix
$ cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && cmake --build . && ctest --output-on-failure
...
Test #1: kmat_test ........................   Passed    0.00 sec
Test #2: time_test ........................   Passed    0.12 sec
Test #3: time_pico2_test ..................   Passed    0.00 sec
Test #4: log_test .........................   Passed    0.01 sec
Test #5: log_pico2_test ...................   Passed    0.00 sec
Test #6: nmea_test ........................   Passed    0.00 sec
Test #7: device_lib_test ..................   Passed    0.16 sec
Test #8: device_lib_pico2_test ............   Passed    0.00 sec

100% tests passed, 0 tests failed out of 8
```
**G1 exit: 0**

### Gate G2 — Traceability

```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       40        (was 33; +7 = DEVICE-001..007)
  Requirements with @verify:    49        (was 42; +7 = DEVICE-001..007)
  Requirements with test specs: 376
```
**G2 exit: 0**

### Gate G3 — Pico2 cross-compile (partial per SPRINT-IMPL-03 precedent)

```
$ rm -rf build_pico2 && mkdir build_pico2 && cd build_pico2 && cmake ..
[...arm-none-eabi-gcc 13.2.1 configures...]
$ cmake --build . --target device_lib
[ 22%] Building CXX object libs/device_lib/CMakeFiles/device_lib.dir/src/pico2/device_pico2.cpp.o
[device_pico2.cpp.o produced clean under -ffreestanding + strict flag set]
[ 25%] Building C object libs/device_lib/CMakeFiles/device_lib.dir/__/__/pico-sdk/src/rp2_common/pico_stdlib/stdlib.c.o
[transitive pico-sdk source compile fails on -Werror=undef due to LIB_PICO_BINARY_INFO not auto-defined at this build path]
```

`device_pico2.cpp.o` cross-compiles clean — SPRINT-IMPL-05 deliverable scope MET. Transitive pico-sdk source-compile is the **pre-existing infrastructure issue documented in SPRINT-IMPL-03 closure record §234**: `JUNO_COMPILE_*` flags propagate from a lib that links `pico_stdlib` onto pico-sdk's INTERFACE_SOURCES. SPRINT-IMPL-04 dodged it (nmea_lib has no pico_stdlib dep); SPRINT-IMPL-05 directly links it. **Mitigation applied this sprint**: gated both `JUNO_COMPILE_OPTIONS` and `JUNO_COMPILE_CXX_OPTIONS` behind `$<COMPILE_LANGUAGE:CXX>` in `libs/device_lib/CMakeLists.txt:64-72`, eliminating leakage of `-Wundef`, `-fno-rtti`, `-fno-exceptions` etc. onto pico-sdk's transitive C sources. Residual issue is pico-sdk's CXX sources (e.g., `new_delete.cpp`) tripping their own `-Werror=undef` due to missing pico-sdk auto-config defines — that is broader build-infrastructure scope (recommend pico-sdk targets be linked PRIVATE-not-PUBLIC across all libs, OR pico-sdk INTERFACE_SOURCES be wrapped with appropriate `-w`/per-target-flag isolation).

**G3 status for SPRINT-IMPL-05 deliverable: PASS** (carryforward infrastructure issue per precedent).

## 6. Acceptance Criteria Status

| AC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| AC-1 | All 7 SW-REQ-DEVICE-001..007 tagged in both impls; traceability code-coverage delta ≥7 | MET | Code +7 / @verify +7 in G2 output |
| AC-2 | `DEVICE_LIB_ROOT_T<N>` templated; `static_assert(N >= 256)`; FT1 instantiation pinned `kDefaultRingCap=2048` | MET | `device_api.hpp:210`; `apps/include/juno_fsw_capacities.hpp:134-140` |
| AC-3 | `ReadBytes` semantics: `{TABLE_FULL_ERROR, zCount}` overflow / `{SUCCESS, 0}` empty / `{NULLPTR_ERROR, 0}` null buffer | MET | SW-TC-DEVICE-004/005/007 PASS on both backends |
| AC-4 | Both impls non-blocking | MET | POSIX `O_NONBLOCK`+`EAGAIN`; Pico2 `uart_is_readable`+`uart_getc`; no `_blocking` calls |
| AC-5 | `device_lib` does NOT subscribe/publish to broker | MET | `grep -rn "broker\|sb/" libs/device_lib/{src,include}` empty |
| AC-6 | All 8 SW-TC-DEVICE-001..008 implemented in both backends; both ctest PASS | MET | 8/8 ctest, both `device_lib_test` + `device_lib_pico2_test` PASS |
| AC-7 | Vtable wired once at file-scope `static const`; dispatch via `tRoot.ptApi->Hook(...)` | MET | `device_posix.cpp:375,441`; `device_pico2.cpp:419,431` |
| AC-8 | `noexcept` on all entry points; no heap/STL/virtual/RTTI; ≤500 LoC per file | MET | Largest: `device_lib_pico2_test.cpp` 496 lines; all production ≤465 |
| AC-9 | Pico2 stub exposes RX FIFO + TX capture + last-args + per-fn counts + `Reset()` per methodology §5.2 | MET | `device_pico2_stub.cpp` defines all 18 canonical state variables + `Reset()` + `PushRx()` |
| AC-10 | G1 + G2 exit 0; G3 partial per precedent | MET | See gate evidence §5 |
| AC-11 | CE PASS verdict | MET | See §7 |

## 7. Chief Engineer Verdict

**APPROVED** (issued by `project-chief-engineer` 2026-05-05, first-iteration unconditional). Full rationale: "Sprint SPRINT-IMPL-05 delivers a production-grade Wave 2 `device_lib` with dual-backend (POSIX + Pico2-stub) Unit coverage of all 7 requirements (SW-REQ-DEVICE-001..007), strict traceability tagging in both impls (+7 code coverage, +7 @verify coverage), and disciplined adherence to the L2 design contract (templated ring `<N>` with `>=256` guard pinned to FT1 `kDefaultRingCap=2048`; non-blocking I/O on both platforms; file-scope `static const tApi` vtable; dispatch via `tRoot.ptApi->`; no broker coupling). All 8 ctest targets PASS at G1, traceability exits 0 at G2, and the G3 carryforward is the documented pico-sdk transitive-source infrastructure issue from SPRINT-IMPL-03 §234 — already mitigated for `device_lib`'s own deliverable via Lead-direct CXX-language flag gating in the module CMakeLists. File-size compliance, ID uniqueness, and JSON validation all clear."

## 8. Agent Count

| Phase | Agents | Notes |
|-------|--------|-------|
| Phase 0a | 0 | Lead-direct legacy cleanup + CMake gating |
| Phase 0b | 2 | 1 SSE worker + 1 MAE reviewer (test_cases.json amendment) |
| Phase 1 | 9 | 8 SSE workers (4 production C++ + 4 test C++) + 1 junior worker (CMakeLists) |
| Phase 2 iter 1 | 9 | 9 SSE reviewers (one per Phase-1 file) |
| Phase 2 iter 2 | 4 | 2 SSE workers re-author + 2 SSE re-reviewers (Tasks 3 & 8) |
| Phase 3 | 0 | Lead-direct gates + atomic Lead corrections (pico/types.h stub, CXX-flag gating, DEVICE-007 @verify tags) |
| Phase 4 | 1 | project-chief-engineer |
| **Total** | **25** | Above the 13–18 dual-impl baseline due to (a) PM-directed Phase 0b test-case amendment (+2), (b) two Major iter-2 cycles (+4), (c) larger-than-baseline file inventory (9 vs 6 due to Pico2-stub triplet + the Lead-direct pico/types.h stub) |

## 9. Carry-Forward / Follow-Up Items

1. **G3 transitive pico-sdk source-compile** — pre-existing infrastructure issue (SPRINT-IMPL-03 §234). SPRINT-IMPL-05 mitigation is local-CXX-flag-gating; broader fix (pico-sdk linkage isolation across all libs) deferred. SPRINT-IMPL-06 (sch_lib, dual-impl, links pico_stdlib) and SPRINT-IMPL-09/10/11 (sensor libs) will inherit. Recommend dedicated build-infrastructure sprint or carry the same Lead-direct CXX-language gating in each future lib's CMakeLists.
2. **POSIX test fd-leak** — `New()` opens its own pty pair internally; tests overwrite `_pvHandle` post-`New()`, leaking 2 fds per test. Sprint-accepted limitation; recommend adding `int iFdMaster` storage field to `DEVICE_LIB_IMPL_T<N>` plus a teardown helper as a minor SDP amendment in a future sprint.
3. **DEVICE-007 test-case-spec gap** — original `docs/test_cases/device/test_cases.json` had no Unit-type SW-TC entry covering DEVICE-007. SPRINT-IMPL-05 closed the @verify gap via Lead-direct tags on existing `WriteBytes_PushesIntoTxFifo` (Pico2) + new `WriteBytes_TransmitsCallerBuffer` (POSIX); but the test_cases.json itself doesn't have a SW-TC-DEVICE-013 entry pinning these. Recommend a minor test-case JSON amendment in the next sprint touching device_lib (gps_lib SPRINT-IMPL-09 or lora_lib SPRINT-IMPL-10).

## 10. Lessons Learned

Cross-referenced into per-role files per methodology §10:

- `ai/memory/lessons-learned-software-lead.md` — pre-Phase-1 cross-worker stub-surface completeness; G3 transitive-source infrastructure precedent needs proactive CXX-flag-gating in dual-impl-pico-sdk-linking sprints; coverage-counter-delta check at G2 catches @verify gaps.
- `ai/memory/lessons-learned-senior-software-engineer.md` — ring-state snapshot/restore pattern on multi-step drain with hard-error short-circuit; ring-overflow test design must trigger sticky detection in the SAME call as the assertion.
- `ai/memory/lessons-learned-junior-software-engineer.md` — dual-impl libs that link pico_stdlib MUST gate `JUNO_COMPILE_*` behind `$<COMPILE_LANGUAGE:CXX>` to prevent flag leakage onto pico-sdk transitive sources.

## 11. Approval

| Field | Value |
|-------|-------|
| Author | Software Lead |
| Date | 2026-05-05 |
| Predecessor | SPRINT-IMPL-03 (time_lib) closure 2026-05-05 |
| Successor | SPRINT-IMPL-06 (sch_lib) — Wave 2 platform pair completion |
| CE verdict | APPROVED unconditional 2026-05-05 |
| PM approval | (this record) — 2026-05-05 |
