# SPRINT-IMPL-04 — nmea_lib (Wave 1, Foundation Libraries)

| Field | Value |
|-------|-------|
| Sprint ID | SPRINT-IMPL-04 |
| Module | `nmea_lib` (single-impl pure-compute NMEA-0183 parser — GGA / RMC / GSA / VTG) |
| Wave | 1 (Foundation Libraries) |
| Revision | A |
| Start date | 2026-05-05 |
| End date | 2026-05-05 |
| Status | **CLOSED** |
| Predecessors | SPRINT-IMPL-00 (CLOSED), SPRINT-IMPL-01 (CLOSED), SPRINT-IMPL-02 (CLOSED), SPRINT-IMPL-03 (CLOSED); LibJuno upstream only |
| Successor eligible | SPRINT-IMPL-05 (device_lib, Wave 2 — depends on SPRINT-IMPL-03 only); SPRINT-IMPL-09 (gps_lib, Wave 3 — depends on this sprint + 05) |
| PM approval | 2026-05-05 — sprint plan approved with PM concurrence on Q1 (use plural `tests/` and `nmea_test.cpp` consistent with kmat/log/time precedent; SDP wave file + test_cases.json `google_test_ref` to be amended at closure) and Q2 (TC-017 implemented as deterministic golden-byte assertion; reviewer + CE confirm by inspection that `nmea_impl.cpp` uses no platform-conditional code, satisfying `SW-REQ-NMEA-010` byte-equivalence by construction) |
| Pico2 stub coverage | **EXEMPT** per methodology §5.3 (single-impl pure compute) |

## Sprint Goal

Implement `nmea_lib` per [docs/design/nmea/design.md](../design/nmea/design.md) and [SDP foundation_libs.md §3 SPRINT-IMPL-04](../sdp/foundation_libs.md): a single-impl pure-compute NMEA-0183 parser (GGA / RMC / GSA / VTG) consumed by `gps_lib` in Wave 3. Closes 12 `SW-REQ-NMEA-*` requirements with 17 `SW-TC-NMEA-*` Unit tests on the POSIX host backend; byte-equivalence to Pico2 cross-compile is guaranteed by construction (single shared `nmea_impl.cpp` linked into both targets, no platform-conditional code).

## Worker Invocations

| # | Phase | File | Worker | Iter | Final Status |
|---|-------|------|--------|------|--------------|
| 1 | 1 | `libs/nmea_lib/include/nmea_lib/nmea_api.hpp` (224 lines) | senior-software-engineer | 1 | APPROVED |
| 2 | 1 | `libs/nmea_lib/include/nmea_lib/nmea_types.hpp` (191 lines) | senior-software-engineer | 1 | APPROVED |
| 3 | 1 | `libs/nmea_lib/include/nmea_lib/nmea_impl.hpp` (149 lines) + `libs/nmea_lib/src/nmea_impl.cpp` (413 lines) — paired worker per SDP §3 file 3 | senior-software-engineer | 1 + Lead-direct atomic remediation (4 edits: 1 Major correctness guard, 3 traceability tags) | APPROVED |
| 4 | 1 | `libs/nmea_lib/tests/nmea_test.cpp` (467 lines after banner-compression sweep; was 477 from worker, peaked at 499 after carryover-assertion remediation, compressed to 467) | senior-software-engineer (test author, distinct invocation) | 1 + Lead-direct atomic remediation (2 carryover-assertion edits + banner-compression sweep) | APPROVED |
| 5 | 1 | `libs/nmea_lib/CMakeLists.txt` (78 lines) | junior-software-engineer | 1 | APPROVED |

**Total Phase-1 worker invocations: 5.** All workers correctly grep-verified the brief's LibJuno symbol citations against `libjuno/include/juno/...` per the 2026-05-05 SPRINT-IMPL-03 lesson; no LibJuno symbol deviations reported.

## Reviewer Verdicts (Phase 2)

| # | Reviewer | File Reviewed | Initial Verdict | Iterations | Final Verdict |
|---|----------|---------------|-----------------|------------|---------------|
| 1 | senior-software-engineer (reviewer mode) | `nmea_api.hpp` | APPROVED | 1 | APPROVED — zero findings |
| 2 | senior-software-engineer (reviewer mode) | `nmea_types.hpp` | APPROVED | 1 | APPROVED — zero findings; all 6 structs + 1 enum class match L2 §4.1 verbatim |
| 3 | senior-software-engineer (reviewer mode) | `nmea_impl.{hpp,cpp}` paired | NEEDS CHANGES (1 Major + 3 Warnings) | 1 + Lead-direct remediation | APPROVED equivalent — see remediation log below |
| 4 | senior-software-engineer (reviewer mode) | `nmea_test.cpp` | NEEDS CHANGES (2 Majors) | 1 + Lead-direct remediation | APPROVED equivalent — see remediation log below |
| 5 | senior-software-engineer (reviewer mode) | `CMakeLists.txt` | APPROVED | 1 | APPROVED — STATIC lib, JUNO_FSW_TESTS gate, no libm, no platform switch, all flags match kmat precedent verbatim |

**Total Phase-2 reviewer invocations: 5.**

## Lead-Direct Atomic Remediation (per 2026-05-04 SPRINT-IMPL-02 atomic-Lead-edit pattern)

Reviewer findings on `nmea_impl.cpp` and `nmea_test.cpp` were classified as atomic-edit-complexity per the 2026-05-04 lesson and applied Lead-direct rather than spawning iteration-2 worker invocations. This collapsed 6 individual findings into 6 batched Lead Edit calls + 1 banner-compression sweep, saving 2 worker re-invocations + 2 reviewer re-invocations.

**`nmea_impl.cpp` — 4 atomic edits:**

1. **Major correctness fix** (Reviewer Finding #1, `nmea_impl.cpp:302-308`). FSM `kCsumPend → '\n'` branch was missing a "exactly 2 hex digits collected" guard. A truncated frame with one hex digit (e.g., `*4\n`) could spuriously match if the computed XOR happened to equal the partial declared value (`0x40`), violating SW-REQ-NMEA-003/004. Added `if (tRoot._au8Buf[kDC] != 2u) return fail("nmea_lib: truncated checksum field");` immediately before the XOR comparison. Verified by reviewing the inline guard ordering and re-running ctest.
2. **Traceability** (Reviewer Finding #2): added `// @{"req": ["SW-REQ-NMEA-003", "SW-REQ-NMEA-011"]}` above `NMEA_LIB_IMPL_T::Reset` (initialization is part of checksum-validation precondition + determinism).
3. **Traceability** (Reviewer Finding #3): added `// @{"req": ["SW-REQ-NMEA-009"]}` above `static DGsa` (sentence-type dispatch participant).
4. **Traceability** (Reviewer Finding #4): added `// @{"req": ["SW-REQ-NMEA-009"]}` above `NMEA_LIB_IMPL_T::AsGsa` (consistency with AsGga/AsRmc/AsVtg tagging).

**`nmea_test.cpp` — 2 atomic edits + 1 banner-compression sweep:**

1. **TC-014 carryover assertion** (Reviewer Finding #1): after the malformed-field rejection, feed a subsequent valid GGA `*47` sentence and assert `tStatus == SUCCESS && eType == NMEA_TYPE_GGA && u8FixQuality == 1u`. Proves rejection did not corrupt parser state.
2. **TC-015 carryover assertion** (Reviewer Finding #2): same pattern as TC-014, applied after out-of-bounds-lat rejection.
3. **Banner-compression sweep** (per 2026-05-04 SPRINT-IMPL-02 lesson): file approached 499 lines after carryover assertions (1-line buffer under cap). Compressed 16 SW-TC-NMEA banners from 3-line `/* === * SW-TC-... * === */` to single-line `// SW-TC-...`. TC-017's prose-style 3-line comment intentionally preserved (not a banner). Net 499 → 467 lines (33-line buffer). No test logic, `@verify` tag, or assertion changed; all 17/17 tests still pass.

## Gate Output (Phase 3 — Lead-direct)

### Gate G1 — POSIX build + ctest

```
$ rm -rf build_posix && mkdir build_posix && cd build_posix
$ cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && cmake --build .
[ 79%] Building CXX object libs/nmea_lib/CMakeFiles/nmea_lib.dir/src/nmea_impl.cpp.o
[ 81%] Linking CXX static library libnmea_lib.a
[ 81%] Built target nmea_lib
[ 83%] Building CXX object libs/nmea_lib/CMakeFiles/nmea_test.dir/tests/nmea_test.cpp.o
[ 85%] Linking CXX executable nmea_test
[ 85%] Built target nmea_test
$ ctest --output-on-failure
1/6 Test #1: kmat_test ............ Passed   0.00 sec
2/6 Test #2: time_test ............ Passed   0.12 sec
3/6 Test #3: time_pico2_test ...... Passed   0.00 sec
4/6 Test #4: log_test ............. Passed   0.00 sec
5/6 Test #5: log_pico2_test ....... Passed   0.00 sec
6/6 Test #6: nmea_test ............ Passed   0.00 sec
100% tests passed, 0 tests failed out of 6
```

`nmea_test` per-case breakdown (verbose ctest output): **17/17 PASSED** — `GgaValidFix_ParsesAllFields`, `GgaSouthWest_LatLonAreNegative`, `RmcValidFix_ParsesAllFields`, `ValidChecksum_SentenceAccepted_AndPopulated`, `BadChecksum_FeedByteReturnsError_NoCarryover`, `MissingChecksum_SentenceRejected`, `LatConversion_4807038N_Equals48p1173`, `LonConversion_01131000E_Equals11p5167`, `Altitude_5454M_ReportedAsMeters`, `SpeedKnotsToMps_10kt_Equals5p14444`, `UtcFromGga_123519_ParsesHoursMinutesSeconds`, `UtcFromRmc_225446_ParsesHoursMinutesSeconds`, `SentenceType_GgaRmcGsv_CorrectlyIdentified`, `MalformedLatField_ABCDEFG_Rejected`, `OutOfBoundsLat_9901000N_Rejected`, `Determinism_ThreeIdenticalParses_ByteIdentical`, `PosixPico2Equivalence_GoldenBytesForFixedInput`.

### Gate G2 — Traceability

```
$ python3 tools/traceability.py
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       33   (was 21; delta +12 = 12 new SW-REQ-NMEA tags)
  Requirements with @verify:    42   (was 30; delta +12 = 12 SW-REQ-NMEA verified)
  Requirements with test specs: 375  (no change; test-spec coverage was already in baseline)
```

Counter delta `+12 with code, +12 with @verify` matches exactly the 12 SW-REQ-NMEA-001..012 requirements covered by this sprint.

### Gate G3 — Pico2 cross-compile

```
$ rm -rf build_pico2 && mkdir build_pico2 && cd build_pico2
$ cmake .. && cmake --build . --target nmea_lib
Compiler: /usr/bin/arm-none-eabi-gcc
CPP Compiler: /usr/bin/arm-none-eabi-g++
[ 83%] Building CXX object libs/nmea_lib/CMakeFiles/nmea_lib.dir/src/nmea_impl.cpp.o
[100%] Linking CXX static library libnmea_lib.a
[100%] Built target nmea_lib
```

Pico2 cross-compile clean. The single `nmea_impl.cpp` TU compiled identically under `arm-none-eabi-g++` with no platform-conditional preprocessor — confirming `SW-REQ-NMEA-010` byte-equivalence by construction.

## Chief Engineer Verdict

**PASS** — issued by `project-chief-engineer` agent 2026-05-05.

CE re-verified all 10 acceptance criteria with independent grep / wc / ctest spot-checks:

| AC | Status | Evidence (per CE) |
|----|--------|-------------------|
| AC-1 | MET | 12 unique `SW-REQ-NMEA-*` IDs in `nmea_impl.cpp`; 17 `TEST_F(NmeaTest, ...)` blocks; 12 unique `@verify` IDs in tests |
| AC-2 | MET | `nmea_api.hpp:74` `static constexpr size_t kMaxSentenceLen = 128`; `_au8Buf[kMaxSentenceLen]` inline at line 210 |
| AC-3 | MET | `nmea_impl.cpp:308-311` truncated-checksum guard precedes XOR compare; `_tLastParsed` only mutated post-completion |
| AC-4 | MET | `nmea_impl.cpp:157` `d = deg + dmin/60.0`; line 110 single `0.514444f` literal; SW-TC-NMEA-007/008/009/010 PASS |
| AC-5 | MET | `add_library(... STATIC src/nmea_impl.cpp)`; zero hits for `JUNO_FSW_POSIX|posix|pico2` and zero `#ifdef\|#if` in source; SW-TC-NMEA-017 PASS |
| AC-6 | MET | No forbidden headers, no `new`/`delete`/`malloc`/`throw`, no `atof`/`memcpy`; only `static const NMEA_LIB_API_T tApi` + `static constexpr` file-scope datums |
| AC-7 | MET | No C++17 nested-namespace form; no C++14 digit separators; 8 `noexcept` annotations in api header (≥7 expected) |
| AC-8 | MET | G1: `100% tests passed, 0 failed out of 6`; G2: counter delta `+12/+12`; G3: `libnmea_lib.a` cross-compiled clean |
| AC-9 | MET | 224 / 191 / 149 / 413 / 467 / 78 — all under 500-line cap; tightest buffer is 33 lines on test file |
| AC-10 | MET | This verdict |

CE non-blocking follow-up notes:
1. Sprint brief said `nmea_impl.cpp` is 414 lines post-remediation; actual is 413 (1-line cosmetic).
2. The "no internal hook calls in nmea_lib" property (intentional for a pure-compute leaf library) is worth a one-line note in the L2 design rationale at the next refresh.
3. `nmea_test.cpp` 33-line buffer is the tightest in Wave 1; if TC-016/-017 ever need additional fixture refactoring, splitting test fixtures (e.g., state vs decode) would be a clean follow-up — not required now.

## SDP Amendment Required at Closure (per PM Q1 disposition)

Two minor SDP amendments to align with the as-built file paths (Lead-direct per methodology §11 minor-amendment provision):

1. **`docs/sdp/foundation_libs.md` §SPRINT-IMPL-04**: amend the file-inventory `Note: per L2 §3.3 the path is libs/nmea_lib/test/ (singular)` to use plural `tests/` consistent with kmat/log/time precedent.
2. **`docs/test_cases/nmea/test_cases.json`**: amend all 17 entries' `google_test_ref` from `libs/nmea_lib/tests/nmea_lib_test.cpp` to `libs/nmea_lib/tests/nmea_test.cpp` (filename was `nmea_lib_test.cpp` in the spec but `nmea_test.cpp` matches the kmat/log/time naming pattern and the file actually authored).
3. **`docs/design/nmea/design.md` §3.3**: amend the file-layout table `libs/nmea_lib/test/nmea_test.cpp` to `libs/nmea_lib/tests/nmea_test.cpp`.

These amendments are descriptive corrections only — no requirement, test-case logic, or design-section content change. Per methodology §11 they are minor amendments dischargeable Lead-direct with PM notification.

## Lessons Learned

Two new entries to record this sprint:

### `ai/memory/lessons-learned-software-lead.md`

#### 2026-05-05 — SPRINT-IMPL-04: Brief Test-Data Pre-Verification Saves a Reviewer Round

**What happened:** The Phase 1 worker brief for `nmea_test.cpp` cited a sample GGA sentence with a `*5C` checksum for the southern-hemisphere TC-002 case. The worker independently computed the XOR for the brief's sentence body, found `*48` (not `*5C`), used `*48` in the test, and flagged the deviation in their report. The reviewer then independently re-verified `*48` and concurred. No iteration cost — but if the worker had trusted the brief literally, the test would have failed at G1 with a checksum-rejected sentence and required a debug-fix-rebuild cycle.

**Root cause (positive lesson):** The worker correctly applied the 2026-05-05 SPRINT-IMPL-03 brief-template-grep lesson — extending its scope from "verify cited symbols against headers" to "verify cited test data against the project's own helpers." This is the worker behavior we want; the lesson generalizes naturally.

**Corrective action:** When a brief includes test-data with computed values (checksums, hashes, hex-encoded constants, fixed-point scaled integers), Lead MUST verify the values via the project's own helper or a one-line Python script before pasting into the brief. For NMEA the verification is `python3 -c "x=0; [x:=x^c for c in b'GPGGA,...']; print(f'*{x:02X}')"`. Cost: 30 seconds per non-trivial test data block. Avoided cost: a worker who blindly trusts the brief produces a broken test that fails at G1, requiring a debug cycle. The 2026-05-05 lesson now extends from "verify cited symbols" to "verify cited test-data computed values too."

#### 2026-05-05 — SPRINT-IMPL-04: Single-Impl Pure-Compute Sprints Run at 11 Agents Total — Below the 13-Agent Median

**What happened:** SPRINT-IMPL-04 closed at 11 agent invocations (5 workers + 5 reviewers + 1 CE), zero iteration-2 cycles via the Lead-direct atomic-edit pattern (6 reviewer findings → 6 batched Lead Edits). Compare to dual-impl precedents: SPRINT-IMPL-02 log_lib was 13 + 7 retro = 20 (with the Pico2-stub retrofit); SPRINT-IMPL-03 time_lib was 18 (initial 11 + Revision B 7). Single-impl pure-compute Wave-1 libs (nmea_lib here, kmat_lib at SPRINT-IMPL-01) consistently land at 9-11 agents.

**Root cause (positive lesson):** Methodology §5.3's single-impl exemption is load-bearing for cost — pure-compute libraries skip the 3-artifact Pico2-stub overhead (stub header + stub source + Pico2 test) and the 3-worker / 3-reviewer / 1-CE-re-gate cycle that comes with it. The exemption rule (kmat / nmea / telem / mlog) is correctly identified in SDP §5.3 and saves ~7 agents per affected sprint.

**Corrective action:** When sprint-planning future single-impl pure-compute sprints (Wave 4: telem_lib SPRINT-IMPL-14, mlog_lib SPRINT-IMPL-15), confirm exemption status via methodology §5.3 in Phase 0 pre-flight and budget 9-11 agents — not the 13+ baseline used for dual-impl sprints. Underbudgeting agents wastes no resources but overbudgeting can prompt unnecessary scope expansion. Dual-impl sprints (Wave 2: device_lib SPRINT-IMPL-05, sch_lib SPRINT-IMPL-06; Wave 3 sensor libs) should continue to budget 13-18 agents with the Pico2-stub artifacts in scope.

### `ai/memory/lessons-learned-senior-software-engineer.md`

#### 2026-05-05 — SPRINT-IMPL-04: FSM Multi-Predicate Branches Need Explicit Precondition Guards

**What happened:** `nmea_impl.cpp` `FeedByte`'s `kCsumPend → '\n'` branch was authored as `if (XOR != declared) return ERR;` — assuming the declared-checksum value had been fully populated by two prior hex-digit transitions. The FSM's design left the `ChecksumPending` state writable on a single hex digit (which is correct, since the state needs to accept one digit before the second arrives), but the `'\n'` transition didn't verify the digit count had actually reached 2 before evaluating the comparison. Reviewer caught this as a Major correctness defect; Lead-direct fix added `if (tRoot._au8Buf[kDC] != 2u) return fail("...");` immediately before the XOR compare.

**Root cause:** State-machine branches with multi-predicate preconditions (here: state == ChecksumPending AND digit-count == 2 AND newline byte) need each predicate explicitly guarded. The "digit-count == 2" predicate was implicit in the design (the LF should not arrive before two hex digits) but not enforced in the implementation. A misbehaving NMEA source emitting `*4\n` instead of `*48\n` could hit the bug; defensive validation rejects it.

**Corrective action:** When implementing FSM transitions where a state can be entered with partial data and exited via a terminator byte, the terminator-handler MUST explicitly assert the partial-data fields have reached their final populated state before evaluating any logical operations on them. Pattern:

```cpp
case kIntermediateState:
    if (terminator) {
        if (partial_data_count != EXPECTED_FINAL_COUNT)
            return reject("partial data — terminator arrived too early");
        // ... now safe to evaluate
    }
```

Add this to the worker brief's "common review traps" section for any FSM-implementing sprint (next applicable: device_lib UART RX ring SPRINT-IMPL-05).

## Files Produced

| Path | Lines | Type |
|------|-------|------|
| `libs/nmea_lib/include/nmea_lib/nmea_api.hpp` | 224 | Public API header |
| `libs/nmea_lib/include/nmea_lib/nmea_types.hpp` | 191 | Public POD types header |
| `libs/nmea_lib/include/nmea_lib/nmea_impl.hpp` | 149 | IMPL declaration header |
| `libs/nmea_lib/src/nmea_impl.cpp` | 413 | Implementation source |
| `libs/nmea_lib/tests/nmea_test.cpp` | 467 | Google Test source (17 SW-TC-NMEA cases) |
| `libs/nmea_lib/CMakeLists.txt` | 78 | Build registration |

## Sprint Metrics

| Metric | Value |
|--------|-------|
| Worker invocations (Phase 1) | 5 |
| Reviewer invocations (Phase 2) | 5 |
| Lead-direct atomic edits | 6 |
| Banner-compression sweeps | 1 |
| Iteration-2 cycles needed | 0 (Lead-direct collapsed all NEEDS CHANGES findings) |
| Chief Engineer invocations | 1 |
| **Total agent invocations** | **11** |
| SW-REQ-NMEA-* closed (Active → Verified) | 12 |
| SW-TC-NMEA-* implemented | 17 (out of 17 in baseline; 0 deferred) |
| Gate G1 result | PASSED (17/17 nmea_test, 6/6 ctest suite) |
| Gate G2 result | PASSED (counter delta +12/+12) |
| Gate G3 result | PASSED (Pico2 `libnmea_lib.a` clean) |
| Wall-clock | One day (2026-05-05) |

## Wave 1 Status After SPRINT-IMPL-04

| Sprint | Module | Status |
|--------|--------|--------|
| SPRINT-IMPL-00 | bus_variant + capacities | CLOSED 2026-05-04 |
| SPRINT-IMPL-01 | kmat_lib | CLOSED 2026-05-04 |
| SPRINT-IMPL-02 | log_lib | CLOSED 2026-05-04 |
| SPRINT-IMPL-02-retro | log_lib (Pico2 stub coverage) | CLOSED 2026-05-05 |
| SPRINT-IMPL-03 | time_lib | CLOSED 2026-05-05 |
| **SPRINT-IMPL-04** | **nmea_lib** | **CLOSED 2026-05-05** |

**Wave 1 complete.** Wave 2 sprints (SPRINT-IMPL-05 device_lib, SPRINT-IMPL-06 sch_lib) are now eligible to begin — both depend only on SPRINT-IMPL-03 (time_lib), which is closed.
