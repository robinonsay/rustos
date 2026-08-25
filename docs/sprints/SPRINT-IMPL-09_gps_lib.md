---
sprint_id: SPRINT-IMPL-09
module: gps_lib
wave: 3 (Sensor Driver Libraries)
predecessors: SPRINT-IMPL-04 (nmea_lib), SPRINT-IMPL-05 (device_lib), SPRINT-IMPL-06 (sch_lib), SPRINT-IMPL-08 (baro_lib closed)
status: CLOSED
opened: 2026-05-06
closed: 2026-05-06
ce_verdict: APPROVED
pm_signoff: pending
---

# SPRINT-IMPL-09 Closure Record — `gps_lib`

## 1. Sprint Goal

Implement the `gps_lib` Wave 3 sensor driver covering the GlobalTop FGPMMOPA6H GPS receiver (UART, 9600 baud, 5 Hz NMEA cadence) behind the LibJuno C++ vtable pattern. Per SDP §5 master sprint table: 10 SW-REQ-GPS-* requirements, 13 test cases (10 Unit + 3 Demonstration deferred to HIL).

## 2. PM-Approved Scope Decisions

| Q | Decision | Rationale |
|---|----------|-----------|
| Q1 | Per-platform IMPL split (`GPS_LIB_POSIX_T` / `GPS_LIB_PICO2_T`) per SPRINT-IMPL-05-retro-A canonical pattern (deviation from L2 §3.3 single-IMPL form) | Mirrors SPRINT-IMPL-07 imu_lib + SPRINT-IMPL-08 baro_lib precedent; avoids deprecated `void*`-handle anti-pattern |
| Q2 | Single test executable parameterized over both factories (no pico-sdk stubs) | gps_lib has no direct pico-sdk surface; UART access goes through injected `juno::device::DEVICE_LIB_API_T<2048>` vtable; matches SPRINT-IMPL-08 baro_lib precedent (callback-injected transport) |
| Q3 | Legacy gps_lib C files deleted; `apps/gps_app/` + legacy mains gated under existing `JUNO_FSW_BUILD_LEGACY_MAIN` | Mirrors SPRINT-IMPL-02 juno_log → log_lib precedent; gating already in place from SPRINT-IMPL-05; cleanest path |
| Q4 | Demonstration TCs SW-TC-GPS-011/-012/-013 deferred to HIL post-CDR | Standard methodology disposition; HIL hardware not yet available |
| Q5 | Mid-sprint: add `juno::time::TIME_ROOT_T *ptTime` to `GPS_LIB_ROOT_T`; `New()` factories take `&tTime` | Resolves L2 §4.1/§4.2.6/§5 staleness gap (SW-TC-GPS-006/-007 unimplementable without clock source); mirrors imu_lib canonical pattern (`IMU_LIB_POSIX_T::ptTime`); 4-file Lead-direct atomic edit |

PM approved Q1–Q4 at sprint open 2026-05-06 with all four recommendations; Q5 approved mid-Phase 2 after Lead spot-verify surfaced the gap.

## 3. Acceptance Criteria — Final Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | Public API per L2 §4.1 in `gps_api.hpp` | MET | [libs/gps_lib/include/gps_lib/gps_api.hpp](../../libs/gps_lib/include/gps_lib/gps_api.hpp) — `JUNO_MODULE_ROOT(GPS_LIB_API_T,...)` line 309; 6 vtable refs (Poll/GetFix/GetUtc/GetRawNmea/Probe/IsHealthy) lines 230-280; PODs `GPS_FIX_T`/`GPS_UTC_T`/`NMEA_RAW_T` lines 144/167/191; `kGpsRxRingCap=2048` (86), `kHealthStaleUs=600000` (93), `kNmeaRawMaxLen=96` (99) |
| AC-2 | Per-platform IMPL split | MET | `GPS_LIB_POSIX_T JUNO_MODULE_DERIVE(GPS_LIB_ROOT_T,...)` at [gps_posix.hpp:209](../../libs/gps_lib/include/gps_lib/gps_posix.hpp#L209); `GPS_LIB_PICO2_T` at [gps_pico2.hpp:207](../../libs/gps_lib/include/gps_lib/gps_pico2.hpp#L207); vtable wired via `static const GPS_LIB_API_T tApi = {...}` at [gps_posix.cpp:137](../../libs/gps_lib/src/posix/gps_posix.cpp#L137) and [gps_pico2.cpp:139](../../libs/gps_lib/src/pico2/gps_pico2.cpp#L139) |
| AC-3 | Common impl: 6 helpers + 12 forwarders single TU | MET | [gps_common.cpp](../../libs/gps_lib/src/common/gps_common.cpp): `DoPoll`(111), `DoGetFix`(243), `DoGetUtc`(264), `DoGetRawNmea`(285), `DoProbe`(308), `DoIsHealthy`(336); 6 Posix forwarders (380-401) + 6 Pico2 forwarders (408-429) |
| AC-4 | Poll byte-streaming via FeedByte/GetParsed; tLastRaw verbatim | MET | gps_common.cpp:151 `ptNmea->ptApi->FeedByte`; line 174 `GetParsed`; line 182 `memcpy(...)` of `au8RawBytes` per L2 §6.2 |
| AC-5 | Poll returns TABLE_FULL_ERROR on overflow; partial bytes still fed | MET | gps_common.cpp:123 detects TABLE_FULL_ERROR; loops at 151 still feeds returned bytes; `RxRingOverflow_*` TEST_P at gps_lib_test.cpp:412 PASSES |
| AC-6 | All 10 Unit-type SW-TC-GPS-* implemented; @verify tags; both platforms PASS | MET | 11 TEST_P × 2 instantiations = 22 ctest cases all PASS; raw runner: `[ PASSED ] 22 tests` |
| AC-7 | No dynamic alloc; noexcept; no virtual/RTTI/exceptions | MET | grep across all gps_lib production files — zero hits |
| AC-8 | Vtable dispatch via `tRoot.ptApi->`; failure handler inline (no JUNO_FAIL_ROOT) | MET | All collaborator dispatches via `tRoot.ptDevice/ptNmea/ptTime->ptApi->`; zero `JUNO_FAIL_ROOT` macro calls |
| AC-9 | All 10 SW-REQ-GPS @req-tagged; @verify covers 8 (excludes -005 and -009 Demo) | MET | tools/traceability.py PASSED with delta +10 code-tagged, +8 @verify-tagged |
| AC-10 | Legacy gps_lib C files removed; gps_app gated | MET | `find libs/gps_lib -name "*.c"` returns 0; libs/CMakeLists.txt:12 `add_subdirectory(gps_lib)` unconditional; apps/CMakeLists.txt:7 gates legacy `gps_app` under `JUNO_FSW_BUILD_LEGACY_MAIN` |
| **AC-11** | **G1 PASS** — POSIX build + ctest | **MET** | `100% tests passed, 0 tests failed out of 13`; gps_lib_test internal: 22/22 |
| **AC-12** | **G2 PASS** — traceability.py exit 0 | **MET** | `TRACEABILITY CHECK PASSED — 376 valid req IDs, 81 with code, 89 with @verify, 376 with test specs` (delta vs Phase 0 baseline 376/71/81/376: +10 code, +8 @verify) |
| **AC-13** | **G3 PASS** — Pico2 cross-compile (arm-none-eabi-g++) | **MET** | `[100%] Built target gps_lib` |

## 4. Deliverable File Inventory

9 production files (vs SDP §5's 6 — expansion documented as PM-approved Q1 + Q5):

| # | Path | Lines | Phase | Author | Final Status |
|---|------|-------|-------|--------|--------------|
| 1 | `libs/gps_lib/include/gps_lib/gps_api.hpp` | 376 | 1 | senior-software-engineer | APPROVED iter-1 + Lead-direct (ptTime member added per Q5) |
| 2 | `libs/gps_lib/CMakeLists.txt` | 157 | 1 | junior-software-engineer | APPROVED iter-1 + Lead-direct (time_lib link added per Q5; Phase 1 placeholder source guard) |
| 3 | `libs/gps_lib/include/gps_lib/gps_posix.hpp` | 485 | 2 | senior-software-engineer | APPROVED iter-1 + Lead-direct (tTime parameter doxygen drift fix) |
| 4 | `libs/gps_lib/include/gps_lib/gps_pico2.hpp` | 470 | 2 | senior-software-engineer | APPROVED iter-1 + Lead-direct (tTime parameter doxygen drift fix) |
| 5 | `libs/gps_lib/src/common/gps_common.hpp` | 212 | 2 | junior-software-engineer | APPROVED iter-1 + Lead-direct (tag format conversion `@req` → `// @{"req":...}`; Now() call form correction) |
| 6 | `libs/gps_lib/src/common/gps_common.cpp` | 432 | 3 | senior-software-engineer | APPROVED iter-1 |
| 7 | `libs/gps_lib/src/posix/gps_posix.cpp` | 171 | 3 | senior-software-engineer | APPROVED iter-1 + Lead-direct (tag placement: moved above signature) |
| 8 | `libs/gps_lib/src/pico2/gps_pico2.cpp` | 186 | 3 | senior-software-engineer | APPROVED iter-1 |
| 9 | `libs/gps_lib/tests/gps_lib_test.cpp` | 442 | 3 | senior-software-engineer (test author) | APPROVED iter-1 (worker self-fixed `\r`-drop kCsumPend issue during authoring) |

**Lead-direct artifacts (not in deliverable count):**
- Phase 0: deletion of legacy `libs/gps_lib/{include/gps_lib/gps_api.h,gps_pico2.h, src/gps.c, src/posix/gps_posix.c}` + legacy CMakeLists; `libs/CMakeLists.txt` gate removed (now unconditional `add_subdirectory(gps_lib)`)
- Phase 0: 4 placeholder `.cpp` files written to satisfy cmake configure (overwritten in Phase 2/3)
- Phase 2 Q5 amendment: `gps_api.hpp` (+`ptTime` member, +`time_api.hpp` include); `gps_posix.hpp`/`gps_pico2.hpp` (+`tTime` parameter on `New()`); `gps_common.hpp` (DoIsHealthy strict-staleness doxygen); `CMakeLists.txt` (+`time_lib` PUBLIC link)
- Phase 2 review fixes: doxygen drift fixes on posix/pico2 headers (4 sites tTime added); gps_common.hpp tag format conversion (6 sites) + Now() call form fix
- Phase 3 review fix: gps_posix.cpp tag placement (moved between doxygen and signature)

## 5. Worker / Reviewer Summary

### Workers (10 invocations across 3 phases — Phase 3 test author re-spawned once after socket disconnect)

| Phase | File | Worker | Iter | Final |
|-------|------|--------|------|-------|
| 1 | gps_api.hpp | senior | 1 | APPROVED |
| 1 | CMakeLists.txt | junior | 1 | APPROVED |
| 2 | gps_posix.hpp | senior | 1 | APPROVED |
| 2 | gps_pico2.hpp | senior | 1 | APPROVED |
| 2 | gps_common.hpp | junior | 1 | APPROVED |
| 3 | gps_common.cpp | senior | 1 | APPROVED |
| 3 | gps_posix.cpp | senior | 1 | APPROVED |
| 3 | gps_pico2.cpp | senior | 1 | APPROVED |
| 3 | gps_lib_test.cpp | senior (test author) | 1 (after retry — first invocation hit socket disconnect) | APPROVED |

### Reviewers (9 invocations across 3 phases)

All Phase 1/2/3 reviewers were senior-software-engineer in reviewer mode. Phase 3 test reviewer ran ctest binary directly (per Sprint 7 lesson "Test Reviewer Must Run the Test, Not Just Inspect It") and confirmed 22/22 internal cases PASS.

### Project Chief Engineer (1 invocation)

CE issued **APPROVED** unconditional on first iteration. All 13 ACs MET.

## 6. Gate Results (Phase 4)

### G1 — POSIX build + ctest

```
===G1 build===
[100%] Built target sample_app
===G1 ctest===
13/13 Test #13: gps_lib_test ... Passed    0.00 sec
100% tests passed, 0 tests failed out of 13
Total Test time (real) = 0.31 sec
```

### G2 — Traceability

```
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       81  (Phase 0 baseline: 71; delta +10)
  Requirements with @verify:    89  (Phase 0 baseline: 81; delta +8)
  Requirements with test specs: 376
```

Counter delta matches expected:
- +10 code-tagged: SW-REQ-GPS-001..010 (helpers cover all 10; forwarders re-cover; SW-REQ-GPS-008 carried by Posix factory; SW-REQ-GPS-009 carried by Pico2 factory)
- +8 @verify-tagged: SW-REQ-GPS-001/002/003/004/006/007/008/010 (excludes 005 and 009 — Demonstration-only, deferred per Q4)

### G3 — Pico2 cross-compile

```
[100%] Built target gps_lib
```

Toolchain: arm-none-eabi-gcc/g++. Clean cross-compile of the gps_lib static library against pico-sdk for the RP2350 target.

## 7. Chief Engineer Verdict

**APPROVED** unconditional. All 13 ACs MET. No blocking issues. Sprint ready for closure record + PM presentation.

CE notes:
- L2 amendments pending closure (per Q1 + Q5): per-platform IMPL split deviation from L2 §3.3 + ptTime injection in `GPS_LIB_ROOT_T`. Lead applies as minor amendments per methodology §11.
- Demonstration TCs SW-TC-GPS-011/-012/-013 remain deferred to HIL post-CDR per Q4.
- File-size compliance: all 9 files ≤ 500 lines hard cap (largest: gps_posix.hpp at 485, gps_pico2.hpp at 470, gps_lib_test.cpp at 442, gps_common.cpp at 432).
- Cross-sprint consistency: no duplicate IDs across all 376 requirements / 458 test cases; no broken references.

## 8. Lessons Learned (cross-references)

Updates appended to:
- `ai/memory/lessons-learned-software-lead.md` — PM-Q-batch escalation pattern surfaced an L2 design ambiguity (staleness gap) before workers committed to a wrong interpretation; Phase 0 placeholder-source pattern resolved cmake configure chicken-and-egg
- `ai/memory/lessons-learned-senior-software-engineer.md` — `\r\n` `kCsumPend` parser quirk in nmea_lib (test author caught and resolved during authoring); IDE clangd false-positives are not real compile failures (always trust cmake build)
