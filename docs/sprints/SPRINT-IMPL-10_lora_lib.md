---
sprint_id: SPRINT-IMPL-10
module: lora_lib
wave: 3 (Sensor Driver Libraries)
predecessors: SPRINT-IMPL-05 (device_lib)
status: CLOSED
opened: 2026-05-06
closed: 2026-05-06
ce_verdict: APPROVED (unconditional, 0 findings)
pm_signoff: pending
---

# SPRINT-IMPL-10 Closure Record — `lora_lib`

## 1. Sprint Goal

Implement the `lora_lib` Wave 3 sensor driver wrapping the REYAX RYLR896 LoRa
radio (UART, AT-command framing, ≤240 B MTU, downlink-only for FT1) behind the
LibJuno C++ vtable pattern with both POSIX and Pico2 implementations. Per SDP
§5 master sprint table: 12 SW-REQ-LORA-* requirements, 15 test cases (13 Unit
+ 2 Demonstration deferred to HIL).

## 2. PM-Approved Scope Decisions

| Q | Decision | Rationale |
|---|----------|-----------|
| Q1 | Per-platform IMPL split (`LORA_LIB_POSIX_T` / `LORA_LIB_PICO2_T`) per SPRINT-IMPL-05-retro-A canonical pattern (deviation from L2 §10.1 single-IMPL form) | Mirrors imu_lib (07) / baro_lib (08) / gps_lib (09) precedent; avoids deprecated `void*`-handle anti-pattern |
| Q2 | Single test executable parameterized over both factories (no pico-sdk stubs) | lora_lib has zero direct pico-sdk surface; UART access is fully via injected `juno::device::DEVICE_LIB_API_T<256>` vtable; matches gps_lib precedent |
| Q3 | Demonstration TCs `SW-TC-LORA-014`/`-015` deferred to HIL post-CDR | Standard methodology disposition; HIL hardware not yet available |
| Q4 | Add `juno::time::TIME_ROOT_T *ptTime` to `LORA_LIB_ROOT_T` upfront (before Phase 1) | **Preempts** the gps_lib Q5 mid-sprint amendment cycle; `Tick()` per-step timeout (L2 §7.3, §8) requires monotonic-µs from juno::time per `SW-REQ-LORA-008/-011`; L2 minor amendment recorded |

PM approved Q1–Q4 at sprint open 2026-05-06 with all four recommendations.

## 3. Acceptance Criteria — Final Status

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| AC-1 | Public API per L2 §4.1 in `lora_api.hpp` (vtable, ROOT, CONFIG, state enum, constants, AT timeout) | MET | [lora_api.hpp:106](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L106) (`kMaxPayloadBytes=240`), [:115](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L115) (`kMaxAtLineBytes=512`), [:125](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L125) (`kLoraUartRxCap=256`), [:133](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L133) (`kDefaultBaud=115200`), [:145](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L145) (`kAtTimeoutUs=1500000`), [:169-199](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L169-L199) (CONFIG 9-field POD), [:221-229](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L221-L229) (state enum), [:257-347](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L257-L347) (6-method vtable), [:387-472](../../libs/lora_lib/include/lora_lib/lora_api.hpp#L387-L472) (ROOT_T with `ptUart` + `ptTime` Q4) |
| AC-2 | Per-platform IMPL split via `JUNO_MODULE_DERIVE`; vtable wired once | MET | [lora_posix.hpp:206](../../libs/lora_lib/include/lora_lib/lora_posix.hpp#L206) (POSIX DERIVE), [lora_pico2.hpp:211](../../libs/lora_lib/include/lora_lib/lora_pico2.hpp#L211) (PICO2 DERIVE); [lora_posix.cpp:136-143](../../libs/lora_lib/src/posix/lora_posix.cpp#L136-L143), [lora_pico2.cpp:142-149](../../libs/lora_lib/src/pico2/lora_pico2.cpp#L142-L149) (`static const tApi`) |
| AC-3 | Common impl: 6 `Do*` helpers + 12 platform forwarders | MET | [lora_common.cpp:175,252,295,319,328,337](../../libs/lora_lib/src/common/lora_common.cpp) (6 `Do*`); 6 `Posix*` forwarders + 6 `Pico2*` forwarders |
| AC-4 | AT-command suite (`AT+ADDRESS`, `AT+NETWORKID`, `AT+BAND`, `AT+PARAMETER`) issued in Configure | MET | [lora_common.cpp:198,208,218,227](../../libs/lora_lib/src/common/lora_common.cpp); each gated by `IssueAtCommand` for `+OK\r\n` |
| AC-5 | POST `Probe()` issues bare `AT\r\n` and verifies `+OK\r\n` | MET | [lora_common.cpp:339-345](../../libs/lora_lib/src/common/lora_common.cpp#L339-L345) |
| AC-6 | Payload bytes verbatim through `AT+SEND` framing | MET | [lora_common.cpp:270](../../libs/lora_lib/src/common/lora_common.cpp#L270) (`memcpy` with no transform); [lora_lib_test.cpp:223-236](../../libs/lora_lib/tests/lora_lib_test.cpp#L223-L236) tests all 256 byte values + embedded `AT+SEND`/`\r\n` substrings |
| AC-7 | `IsHealthy()` and `IsBusy()` polling APIs | MET | [lora_common.cpp:319-325,328-334](../../libs/lora_lib/src/common/lora_common.cpp#L319-L334) |
| AC-8 | Configurable UART baud rate honored end-to-end | MET | [lora_common.cpp:187-189](../../libs/lora_lib/src/common/lora_common.cpp#L187-L189) (UART Configure with `tCfg.u32BaudRate`); test [`ConfigurableUartBaudRate_Honored`](../../libs/lora_lib/tests/lora_lib_test.cpp) verifies 9600 + 115200 |
| AC-9 | Non-blocking Send within 500 ms; Tick drains; 1500 ms per-step timeout via `ptTime` | MET | `kAtTimeoutUs=1500000` at lora_api.hpp:145; DoSend records `tSendStartUs = GetNowUs()` and returns immediately; DoTick computes `(tNow - tSendStartUs) >= kAtTimeoutUs` |
| AC-10 | All 12 SW-REQ-LORA-* code-tagged; 12 @verify-tagged; 014/015 deferred (Q3) | MET | tools/traceability.py PASS — counter delta +12 code, +12 @verify |
| **AC-11** | **G1 PASS — POSIX build + ctest** | **MET** | `100% tests passed, 0 tests failed out of 14`; lora_lib_test internal: 26/26 (13 TEST_P × 2 IMPL params) |
| **AC-12** | **G2 PASS — `tools/traceability.py` exit 0** | **MET** | Phase-0 baseline 376/81/89/376 → post-sprint 376/93/101/376 (delta +12 code, +12 @verify exactly matching 12 SW-REQ-LORA-*) |
| **AC-13** | **G3 PASS — Pico2 cross-compile (`arm-none-eabi-g++`)** | **MET** | `[100%] Built target lora_lib` |
| AC-14 | All 9 deliverable files ≤500 lines (constraints.md hard cap); CE APPROVED | MET | All ≤500; lora_posix.hpp at exactly 500 (at-cap, no headroom — flagged for future amendment care) |

## 4. Deliverable File Inventory

9 production files (vs SDP §5's 6 — expansion documented as PM-approved Q1 + Q4):

| # | Path | Lines | Phase | Author | Final Status |
|---|------|-------|-------|--------|--------------|
| 1 | `libs/lora_lib/include/lora_lib/lora_api.hpp` | 475 | 1 | senior | APPROVED iter-1 |
| 2 | `libs/lora_lib/CMakeLists.txt` | 163 | 1 | junior | APPROVED iter-1 |
| 3 | `libs/lora_lib/include/lora_lib/lora_posix.hpp` | 500 | 2 | senior | APPROVED iter-1 (at-cap; banner-compression deferred to future need) |
| 4 | `libs/lora_lib/include/lora_lib/lora_pico2.hpp` | 487 | 2 | senior | APPROVED after Lead-direct hook-rename `LoraLibPico2_*` → `Pico2*` |
| 5 | `libs/lora_lib/src/common/lora_common.hpp` | 195 | 2 | junior | APPROVED iter-1 |
| 6 | `libs/lora_lib/src/common/lora_common.cpp` | 425 | 3 | senior (re-spawn after socket disconnect) | APPROVED after Lead-direct comment additions (status-code substitution rationale) |
| 7 | `libs/lora_lib/src/posix/lora_posix.cpp` | 172 | 3 | senior | APPROVED iter-1 |
| 8 | `libs/lora_lib/src/pico2/lora_pico2.cpp` | 187 | 3 | senior | APPROVED iter-1 |
| 9 | `libs/lora_lib/tests/lora_lib_test.cpp` | 422 | 3 | senior (test-author) | APPROVED iter-1 (26/26 ctest PASS) |

**Lead-direct artifacts (not in deliverable count):**
- Phase 0: `libs/lora_lib/{include/lora_lib,src/{common,posix,pico2},tests}` directory tree created; 4 placeholder `.cpp` stubs + 1 placeholder CMakeLists.txt to satisfy cmake configure (overwritten in Phases 1–3); `add_subdirectory(lora_lib)` added to `libs/CMakeLists.txt`
- Phase 2 Lead-direct: `LoraLibPico2_*` → `Pico2*` rename via sed `\b` word-boundary pattern (8 hook sites) + 1 doxygen comment Edit (lessons-learned 2026-05-03 sed-rename pattern)
- Phase 3 Lead-direct: 3 comment additions to `lora_common.cpp` (lines 191, 254, 271) explaining `JUNO_STATUS_WRITE_ERROR`/`JUNO_STATUS_ERR` substitutions for unpublished `IO_ERROR`/`BUSY_ERROR` codes, and the L2 §4.2.2 FAILED→IDLE deviation per L2 §5 transient-state semantics

## 5. Worker / Reviewer Summary

### Workers (10 invocations across 3 phases — Phase 3 lora_common.cpp re-spawned once after socket disconnect)

| Phase | File | Worker | Iter | Final |
|-------|------|--------|------|-------|
| 1 | lora_api.hpp | senior | 1 | APPROVED |
| 1 | CMakeLists.txt | junior | 1 | APPROVED |
| 2 | lora_posix.hpp | senior | 1 | APPROVED |
| 2 | lora_pico2.hpp | senior | 1 | APPROVED + Lead-direct rename |
| 2 | lora_common.hpp | junior | 1 | APPROVED |
| 3 | lora_common.cpp | senior | 1 (after re-spawn) | APPROVED + Lead-direct comments |
| 3 | lora_posix.cpp | senior | 1 | APPROVED |
| 3 | lora_pico2.cpp | senior | 1 | APPROVED |
| 3 | lora_lib_test.cpp | senior (test author) | 1 | APPROVED |

### Reviewers (9 invocations)

All Phase 1/2/3 reviewers were senior-software-engineer in reviewer mode. Phase 3 test reviewer ran ctest binary directly (per Sprint 7 lesson) and confirmed 26/26 internal cases PASS.

### Project Chief Engineer (1 invocation)

CE issued **APPROVED** unconditional on first iteration. All 14 ACs MET, no findings.

## 6. Gate Results (Phase 4)

### G1 — POSIX build + ctest

```
100% tests passed, 0 tests failed out of 14
14/14 Test #14: lora_lib_test ............ Passed   0.00 sec
```
Internal lora_lib_test parametric coverage: 13 TEST_P × 2 IMPL params = 26 cases all PASS.

### G2 — Traceability

```
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       93   (Phase 0 baseline 81; delta +12)
  Requirements with @verify:    101  (Phase 0 baseline 89; delta +12)
  Requirements with test specs: 376
```

Counter delta exactly matches expected:
- +12 code-tagged: SW-REQ-LORA-001..012 (each carried by `Do*` body + corresponding `Posix*`/`Pico2*` forwarder; counter measures unique-ID coverage)
- +12 @verify-tagged: SW-REQ-LORA-001..012 (all 12 covered by Unit TCs; SW-TC-LORA-014/-015 are Demonstration-only and don't contribute)

### G3 — Pico2 cross-compile

```
[100%] Built target lora_lib
```

Toolchain: arm-none-eabi-gcc/g++. Clean cross-compile of the lora_lib static library against pico-sdk for the RP2350 target. **Zero direct pico-sdk surface in lora_lib itself** — UART access is fully via injected device_lib vtable.

## 7. Chief Engineer Verdict

**APPROVED** unconditional. All 14 ACs MET. No blocking issues. No findings.

CE notes:
- L2 amendments pending closure (per Q1 + Q4): per-platform IMPL split deviation from L2 §10.1; `ptTime` injection in `LORA_LIB_ROOT_T` (L2 §4.1 omits). Lead applies as minor amendments per methodology §11.
- Demonstration TCs SW-TC-LORA-014/-015 remain deferred to HIL post-CDR per Q3.
- File-size compliance: all 9 files ≤500 lines hard cap. **`lora_posix.hpp` at exactly 500 lines (zero headroom)** — flagged for future-amendment care; banner-compression playbook from lessons-learned 2026-05-04 SPRINT-IMPL-02 available if needed.
- Cross-sprint consistency: no duplicate IDs across all 376 requirements; no broken references; zero `pico/...` or `hardware/...` includes in lora_lib (cross-compile passes purely via device_lib vtable injection per Q2 architecture).

## 8. Sprint Statistics

- Workers spawned: 10 (Phase 1: 2; Phase 2: 3; Phase 3: 5 incl. 1 re-spawn for socket disconnect)
- Reviewers spawned: 9 (one per worker output)
- Lead-direct atomic edits: 4 (1 hook rename via sed + 3 comment additions)
- Iterations to APPROVED: all files iter-1 (with Lead-direct supplementary fixes)
- Total agent invocations: 10 workers + 9 reviewers + 1 CE = 20 (vs sprint plan's 19; the re-spawn accounted for the +1)

## 9. Lessons Learned (cross-references)

Updates appended to:
- `ai/memory/lessons-learned-software-lead.md` — preemptive Q-batch foresight (Q4 ptTime preempted gps_lib's Q5 mid-sprint amendment); Phase 0 placeholder + greenfield directory pattern; hook-naming-divergence Lead-direct sed rename; status-code-substitution comment-addition pattern (atomic-Lead-edit triage rule)
- `ai/memory/lessons-learned-senior-software-engineer.md` — `juno/status.h` does not publish `BUSY_ERROR` or `IO_ERROR` semantic codes; substitute with closest available (`JUNO_STATUS_ERR` / `JUNO_STATUS_WRITE_ERROR`) and document with inline comment citing the unpublished name; cross-file hook-naming consistency requirement (sibling Posix/Pico2 IMPL headers must use symmetric short prefixes per gps precedent)
