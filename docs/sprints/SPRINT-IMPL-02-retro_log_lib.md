# SPRINT-IMPL-02-retro — log_lib Pico2 Stub Coverage

| Field | Value |
|-------|-------|
| Sprint ID | SPRINT-IMPL-02-retro |
| Module | `log_lib` (Pico2-backend host-side test coverage retrofit) |
| Wave | 1 (Foundation Libraries) — methodological retrofit |
| Sprint type | Retro-coverage sprint discharging SDP Revision B (methodology.md §5.1) retro-applicability for log_lib |
| Start date | 2026-05-05 |
| End date | 2026-05-05 |
| Status | **CLOSED** |
| Predecessors | SPRINT-IMPL-02 (CLOSED 2026-05-04, original log_lib sprint with POSIX-only tests); SPRINT-IMPL-03 (CLOSED 2026-05-05, established the Revision B Pico2-stub-test convention) |
| Successor eligible | SPRINT-IMPL-04 (nmea_lib, Wave 1 — single-impl, exempt from §5.3 dual-impl rule) |
| PM approval | 2026-05-05 — sprint plan approved with Q1 disposition (UART path only; RTT path deferred to potential Wave 5 HIL Demonstration) |

## Sprint Goal

Retroactively apply [docs/sdp/methodology.md](../sdp/methodology.md) §5.1 / §5.2 / §5.3 (Revision B amendment 2026-05-05) to `log_lib`. Closes the Pico2 host-side test-coverage gap left by SPRINT-IMPL-02 (which delivered POSIX-only tests because the methodology at the time stated "Pico2 build cross-compiles but does not execute tests"). The Pico2 production source `libs/log_lib/src/pico2/log_pico2.cpp` is **unchanged** — only test infrastructure is added (linker substitution proves).

After this sprint closes, the Revision B retro-applicability clause is fully discharged for log_lib (and previously for time_lib via SPRINT-IMPL-03). Future dual-impl sprints (SPRINT-IMPL-05/06/07/08/09/10/11) will adopt the convention from sprint inception per methodology §5.3.

## Worker Invocations

### Phase 1

| # | Phase | File | Worker | Iter | Final Status |
|---|-------|------|--------|------|--------------|
| 1 | 1 | `libs/log_lib/tests/stubs/hardware/uart.h` (88 lines, stub mirror of pico-sdk hardware/uart.h) | senior-software-engineer | 1 + Lead-direct (uart_init type alignment uint32_t to match stub source) | APPROVED |
| 2 | 1 | `libs/log_lib/tests/stubs/log_pico2_stub.cpp` (214 lines, stub implementations + test-controllable state in juno::test::log_pico2_stub namespace) | senior-software-engineer | 1 (no edits — `noexcept`-on-stub Warning explicitly rejected per 2026-05-03 spot-verify-and-reject pattern) | APPROVED |
| 3 | 1 | `libs/log_lib/tests/log_pico2_test.cpp` (475 lines, 19 TEST_F: 12 SW-TC-LOG mirror + 7 EC) | senior-software-engineer (test author, distinct invocation) | 1 + Lead-direct (cross-worker sed name-rename + 2 ERROR fixes for FIFO-full simulation) | APPROVED |
| 4 | 1 | `libs/log_lib/CMakeLists.txt` (+40 lines, log_pico2_test target) | Lead-direct atomic edit | 1 | APPROVED |

## Reviewer Verdicts

| # | Phase | Reviewer | File | Iter | Verdict |
|---|-------|----------|------|------|---------|
| 1 | 2 | senior-software-engineer (reviewer) | `tests/stubs/hardware/uart.h` | 1 | NEEDS CHANGES — 1 Error: signature type mismatch (`unsigned int` vs `uint32_t`) between header and sibling stub source |
| 2 | 2 | senior-software-engineer (reviewer) | `tests/stubs/log_pico2_stub.cpp` | 1 | NEEDS CHANGES — 2 Warnings: (a) signature mismatch with header (covered by Reviewer 1's fix); (b) missing `noexcept` on extern "C" functions |
| 3 | 2 | senior-software-engineer (reviewer) | `tests/log_pico2_test.cpp` | 1 | NEEDS CHANGES — 2 Errors (FIFO-full simulation broken — would not catch production bugs the test claims to verify) + 4 Warnings |
| 4 | 4 | project-chief-engineer | sprint deliverable | 1 | **PASS** (no remediation required; CE issued 2 non-blocking recommendations for future dual-impl sprints) |

## Lead-Direct Atomic Edits Applied

Per the 2026-05-04 atomic-Lead-edit-pattern lesson, all reviewer findings classified atomic were applied Lead-direct without iteration-2 worker dispatch.

### Phase 1 (cross-worker name alignment, applied before Phase 2 review)

1. **Cross-worker helper-name canonicalization** — test author worker independently chose state-variable names (`g_acBytesCaptured`, `g_zBytesCaptured`, `g_iLastInitBaud`, `g_iWritableUntilCount`) that diverged from the stub source's actual definitions (`g_acBytes`, `g_zBytesWritten`, `g_iLastUartInitBaud`, `g_zWritableBudget`). Applied 4-rename `sed` across the test file to align with the stub's actual names. (Same lesson surfaced by SPRINT-IMPL-02 original log_lib sprint — cross-worker shared-state names need brief-template alignment.)

### Phase 2 (after reviewer fan-out, atomic-edit cascade)

2. **`uart.h:68` type alignment** — header declared `unsigned int uart_init(...)` but stub source defined `uint32_t uart_init(...)`. On LP64 hosts these alias, but the type mismatch is non-portable and sibling-source-divergent. Lead-direct edit changed the header to `uint32_t uart_init(uart_inst_t *uart, uint32_t baudrate);` matching the stub source AND the production `kLogUartBaud = 115200u` (uint32_t).

3. **TC-009 FIFO-full simulation ERROR fix (`log_pico2_test.cpp:275`)** — original test set `g_bWritableNext = false` but left `g_zWritableBudget = SIZE_MAX` (Reset default). Stub's `uart_is_writable()` returns true while budget>0 ignoring the flag, so the test would never simulate FIFO-full and would always fail `EXPECT_EQ(tS, JUNO_STATUS_WRITE_ERROR)` against a correct production implementation. Lead-direct added `stub::g_zWritableBudget = 0U;` before setting `g_bWritableNext = false;` so FIFO-full-from-byte-one is actually simulated.

4. **EC-3 FIFO-full simulation ERROR fix (`log_pico2_test.cpp:372`)** — original test set `g_zWritableBudget = 5U` but didn't set `g_bWritableNext = false`. After 5 budget units consumed, the stub returns the default `g_bWritableNext = true`, never simulating the FIFO-full transition. Lead-direct added `stub::g_bWritableNext = false;` after the budget setting so the 6th `uart_is_writable` call returns false.

### Reviewer findings explicitly rejected (per 2026-05-03 spot-verify-and-reject pattern)

5. **Reviewer's `noexcept`-on-stub Warning (log_pico2_stub.cpp lines 162, 179, 199)** — REJECTED as inapplicable. Real pico-sdk's `hardware/uart.h` does NOT mark its functions `noexcept`. The FSW coding-standards rule "all functions noexcept" applies to FSW production code, not to pico-sdk-surrogate stubs whose entire purpose is to mirror the real C-library API surface. Stub fidelity to real pico-sdk semantics is the load-bearing concern for surrogate code. CE concurred with the rejection in its PASS rationale.

6. **Reviewer's MakeImpl() helper Warnings (lines 67-74)** — accepted as minor style observations but not blocking; documented as a follow-up cleanup if the same pattern recurs in future sprints.

## Phase 3 Gate Evidence

```
=== G1: POSIX build + ctest ===
$ cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON ..
Configure exit: 0
$ cmake --build .
[100%] Built target gps_app
Build exit: 0
$ ctest --output-on-failure
1/5 Test #1: kmat_test ........................   Passed    0.00 sec
2/5 Test #2: time_test ........................   Passed    0.12 sec
3/5 Test #3: time_pico2_test ..................   Passed    0.00 sec
4/5 Test #4: log_test .........................   Passed    0.01 sec
5/5 Test #5: log_pico2_test ...................   Passed    0.00 sec
100% tests passed, 0 tests failed out of 5

log_pico2_test detail:
[==========] 19 tests from LogLibPico2Test (0 ms total)
[  PASSED  ] 19 tests.
   - 12 SW-TC-LOG-001..012 mirror tests (Pico2 backend via stub)
   - 7 Pico2-specific edge cases:
     • EC-1 NewInvokesUartInit_ExactlyOnce_AtBaud115200
     • EC-2 Log_EmitsFormattedRecordViaUartPutcRaw_MessageVisible
     • EC-3 FifoFullDropNewest_ReturnsWriteError_OnlyNBytesEmitted
     • EC-4 BelowMinLevel_GatedOut_NoUartCalls_ReturnsSuccess
     • EC-5 LogFmt_FormatsAndWritesViaUart_VariadicPathVerified
     • EC-6 BoundedRecord_LongMessageTruncatesAtKLogMaxRecord
     • EC-7 SeverityPrefixBytes_PerLevel_RecordStartsWithLabel

G1 exit: 0

=== G2: tools/traceability.py ===
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       21    (unchanged — no new SW-REQ-* tagged in code)
  Requirements with @verify:    30    (unchanged — log_lib's 8 SW-REQ-LOG-* were
                                       already @verify-tagged in log_test.cpp;
                                       the new pico2 test re-tags the same IDs,
                                       which is correct per the methodology
                                       — duplicates don't increment unique-ID count)
  Requirements with test specs: 375
G2 exit: 0

=== G3: Pico2 freestanding cross-compile (smoke) ===
$ arm-none-eabi-g++ -std=c++11 -Wall -Wextra -Werror -pedantic -Wshadow
    -Wcast-align -Wundef -Wswitch -Wswitch-default -fno-rtti -fno-exceptions
    -fno-common -fno-strict-aliasing -ffreestanding -mcpu=cortex-m33 -mthumb
    -I libjuno/include -I libs/log_lib/tests/stubs -fsyntax-only -x c++
    -include libs/log_lib/tests/stubs/hardware/uart.h /dev/null
G3 (stub header parse) exit: 0
```

## Acceptance Criteria — Final Status

| AC | Description | Status | Evidence |
|----|-------------|--------|----------|
| AC-1 | `log_pico2_test` ctest target builds + 12 SW-TC-LOG-001..012 PASS | MET | ctest 5/5 PASS; gtest 19/19 PASS (12 mirror + 7 EC) |
| AC-2 | Stub-state observability per §5.2 (state, last-args, counters, Reset) | MET | All four contract items present in `log_pico2_stub.cpp:71,74,77,82,88,95,98,101,109` |
| AC-3 | `log_pico2.cpp` unmodified | MET | mtime predates this sprint's stub artifacts; CMakeLists comment documents linker-substitution-only contract |
| AC-4 | 7 Pico2-specific edge cases delivered (≥6 required) | MET | EC-1..EC-7 all PASS |
| AC-5 | Compiler flags clean; zero warnings; no dynamic alloc | MET | full strict flag set; clean rebuild zero warnings |
| AC-6 | Gates G1+G2+G3 all exit 0 | MET | see evidence above |
| AC-7 | CE PASS verdict | MET | CE iter 1 PASS |
| AC-8 | All file lengths ≤500 lines | MET | 88 / 214 / 475 / 133 (all under cap) |
| AC-9 | RTT path explicitly out of scope per Q1 | MET | UART-only stub set; CMakeLists `LOG_LIB_PICO2_USE_RTT` option preserved but unused by the test target |

## Risk Resolution

- **Cross-worker shared-state-name drift** — surfaced again in this sprint (test author chose different names than stub source). Resolved by Lead-direct sed rename. **Future sprints**: brief templates for parallel stub+test workers must explicitly enumerate the canonical state-variable names so all parallel workers see the same contract. Captured as lessons-learned.
- **FIFO-full simulation requires dual-knob discipline** — the stub's `g_zWritableBudget` + `g_bWritableNext` dual-knob design caused two test-author errors that needed Lead-direct fixes. CE recommended a `SimulateFifoFullAfter(size_t N)` helper for future sprints (atomically sets both knobs); not applied in this sprint to avoid post-CE diff but logged for SPRINT-IMPL-05+.
- **`noexcept` on extern "C" pico-sdk-surrogate stubs** — reviewer flagged as missing per FSW coding-standards. Lead rejected per 2026-05-03 spot-verify pattern: real pico-sdk doesn't use `noexcept`; stub fidelity to the real C-library API is the load-bearing concern. CE concurred. Captured as lessons-learned ("scope of FSW coding standards: production code only, not surrogate stubs").

## Files Touched (created / amended)

**Created:**
- `libs/log_lib/tests/stubs/hardware/uart.h` (88 lines)
- `libs/log_lib/tests/stubs/log_pico2_stub.cpp` (214 lines)
- `libs/log_lib/tests/log_pico2_test.cpp` (475 lines)
- `docs/sprints/SPRINT-IMPL-02-retro_log_lib.md` (this file)

**Amended (Lead-direct):**
- `libs/log_lib/CMakeLists.txt` (+40 lines: `log_pico2_test` target gated on `JUNO_FSW_TESTS`, links `tests/stubs/log_pico2_stub.cpp`, prepends `tests/stubs/` to include path)

**Unmodified (verified):**
- `libs/log_lib/src/pico2/log_pico2.cpp` (production source — no diff against SPRINT-IMPL-02 closure)
- All other log_lib source files
- All other libs/

## Lessons Learned (this sprint)

Captured in:
- `ai/memory/lessons-learned-software-lead.md` (2026-05-05 — see entries below)

## Agent Count

7 agents (matches original sprint plan estimate):
- Phase 1: 3 workers (3 SSE) = 3
- Phase 2: 3 reviewers (all SSE-R) = 3
- Phase 4: 1 project-chief-engineer = 1
- Total: **7 agents**, 4 Lead-direct atomic edit cascades

## Notable Worker Deviations (Approved)

- **Test-author worker independently grep-verified `LOG_LIB_IMPL_T::New()` signature** before authoring the fixture's call — caught and avoided a brief-spec ambiguity (3-arg production form vs 4-arg POSIX-mode test seam). The `New()` call uses the 3-arg production form correctly.
- **Test-author worker omitted `g_zWritableBudgetUsed` from the extern declarations** — the stub source defines it but no test uses it. Reviewer flagged this as a Warning ("symbol declared but unused"); Lead concurred this is benign (the symbol exists in the stub for future tests that need to assert "FIFO accepted exactly N bytes") and deferred the cleanup.
- **CE recommended a `SimulateFifoFullAfter(size_t N)` helper** in the stub source to atomically set both `g_zWritableBudget` and `g_bWritableNext` (which the dual-knob design requires for FIFO-full simulation). Logged for SPRINT-IMPL-05+ adoption — not applied this sprint to avoid post-CE diff.

## Successor Eligibility

**SPRINT-IMPL-04 (nmea_lib, Wave 1) is eligible to launch** per CE PASS verdict. Wave 1 progress: **3/4 sprints CLOSED** (kmat, log+log-retro, time); nmea_lib remains. nmea_lib is single-impl pure-compute and is **exempt** from the §5.3 dual-impl rule (no Pico2 stub coverage required).

**Methodology §5.1 retro-applicability is fully discharged** by this sprint (and SPRINT-IMPL-03 for time_lib). The Revision B convention's remaining scope is exclusively forward-looking dual-impl sprints (SPRINT-IMPL-05 device_lib, SPRINT-IMPL-06 sch_lib, SPRINT-IMPL-07 imu_lib, SPRINT-IMPL-08 baro_lib, SPRINT-IMPL-09 gps_lib, SPRINT-IMPL-10 lora_lib, SPRINT-IMPL-11 sd_lib).

## Pre-Closure CE Recommendations Logged for Follow-Up

1. **`SimulateFifoFullAfter(size_t N)` helper** in `log_pico2_stub.cpp` — would atomically set both `g_zWritableBudget` and `g_bWritableNext` so future test authors cannot get the FIFO-full simulation wrong. Pattern can be generalized to other dual-knob stub designs (device_lib UART loopback, sd_lib SPI, etc.). Recommend adopting in SPRINT-IMPL-05 as an additional methodology §5.2 helper-function convention.
2. **PM record retro-applicability closure** — methodology §5.1 retro-applicability is now fully discharged for both pre-Revision-B dual-impl sprints (SPRINT-IMPL-02 log_lib via this retro + SPRINT-IMPL-03 time_lib in-sprint). The Revision B compliance scope going forward is unambiguously forward-only: SPRINT-IMPL-05/06/07/08/09/10/11 must adopt the convention from sprint inception.
