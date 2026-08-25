---
inspection_id: INS-LOG-001
requirement: SW-REQ-LOG-008
test_case: SW-TC-LOG-012
verification_method: Inspection
inspection_date: 2026-05-05
inspector: Software Lead
verdict: PASS
sprint_context: SPRINT-IMPL-05-retro-B RTM-cleanup amendment
---

# Inspection Record — SW-REQ-LOG-008 Separation From Mission Logging

## 1. Requirement Under Inspection

**SW-REQ-LOG-008** — *Separation From Mission Logging*

> The log library shall not write diagnostic records to the SD card mission log.

- `verification_method`: **Inspection** (per [docs/requirements/log/requirements.json](../requirements/log/requirements.json))
- Rationale: scope-separation declaration; keeps `juno_log` distinct from `mlog_lib`. juno_log is the diagnostic logger; mission logging is owned by `mlog_lib`.
- Test case: **SW-TC-LOG-012** (per [docs/test_cases/log/test_cases.json](../test_cases/log/test_cases.json))

## 2. Inspection Procedure

Per SW-TC-LOG-012's `procedure` array (4 steps):

1. Inspect `libs/log_lib/CMakeLists.txt` for link or include dependencies on `mlog_lib`, `sd_lib`, SD card drivers, or FatFs.
2. Inspect all `libs/log_lib` source files for include directives referencing `mlog_lib`, `sd_lib`, FatFs, or filesystem headers.
3. Inspect all `libs/log_lib` source files for symbol references to SD, mlog, or file write APIs.
4. Confirm none of the above dependencies, includes, or references are present.

## 3. Acceptance Criterion

> `libs/log_lib` declares no build dependency on, includes from, or symbol references to `mlog_lib`, `sd_lib`, SD card, or filesystem APIs.

## 4. Evidence

### 4.1 Step 1 — CMakeLists.txt link/include audit

**Command:**
```
grep -nE "mlog_lib|sd_lib|FatFs|ff\.h|<filesystem>|<fstream>|f_open|f_write|mlog_" \
  libs/log_lib/CMakeLists.txt
```

**Result:** **0 hits** of forbidden tokens.

The two non-forbidden hits at lines 118, 162 reference the substring "inspection-style tests" in explanatory comments — **not** mlog_lib or sd_lib references.

**Link declarations** (lines 68, 92, 155):
```
target_link_libraries(log_lib PUBLIC juno)
target_link_libraries(log_lib PUBLIC pico_stdlib hardware_uart)   # Pico2 only
target_link_libraries(log_pico2_test PRIVATE juno gtest gtest_main pthread)
```

`juno` is LibJuno (no SD/mlog). `pico_stdlib + hardware_uart` are the Pico2 UART-output dependencies for the diagnostic sink. Neither links `mlog_lib`, `sd_lib`, or any filesystem layer. ✅ **PASS**

### 4.2 Step 2 — Source-file include audit

**Command:**
```
grep -rnE "mlog_lib|sd_lib|FatFs|ff\.h|<filesystem>|<fstream>" \
  libs/log_lib/include libs/log_lib/src
```

**Result:** **0 hits.** No `#include` directive in any `log_lib` header or source file references `mlog_lib`, `sd_lib`, FatFs, or any C++ filesystem header. ✅ **PASS**

### 4.3 Step 3 — Source-file symbol audit

**Command:**
```
grep -rnE "f_open|f_write|f_close|f_read|f_sync|sd_init|sd_write|mlog_emit|mlog_write" \
  libs/log_lib/include libs/log_lib/src
```

**Result:** **0 hits.** No FatFs `f_*` symbols, no `sd_*` symbols, no `mlog_*` symbols are referenced anywhere in `log_lib` source. ✅ **PASS**

### 4.4 Step 4 — Confirmation

All three audit steps return zero hits. The acceptance criterion ("declares no build dependency on, includes from, or symbol references to mlog_lib, sd_lib, SD card, or filesystem APIs") is satisfied. ✅ **PASS**

## 5. Verdict

**PASS** — `libs/log_lib` is genuinely separated from `mlog_lib`, `sd_lib`, SD card drivers, and filesystem APIs. SW-REQ-LOG-008 is verified by Inspection per its declared method.

## 6. Related Test Artifact (informational)

[`libs/log_lib/tests/log_pico2_test.cpp`](../../libs/log_lib/tests/log_pico2_test.cpp) contains a `TEST_F` named `Pico2Impl_NoISinkFd_NoPosixFileCoupling` that uses `static_assert(sizeof(LOG_LIB_PICO2_T) == sizeof(LOG_LIB_ROOT_T))` plus a runtime ASSERT on the vtable. This is a **narrow regression guard** preventing reintroduction of the SPRINT-IMPL-02 `int iSinkFd` POSIX-coupling field — it verifies struct layout only, NOT the broader "no link/include/symbol coupling" assertion which this inspection record covers. The TEST_F does NOT carry a `@verify` tag for SW-REQ-LOG-008 (intentionally — Test artifacts must not claim verification of Inspection-method requirements per IEEE 829).

## 7. Re-Inspection Triggers

This inspection record must be re-executed (and re-signed) when any of the following changes:

- `libs/log_lib/CMakeLists.txt` link/include declarations change.
- A new `.cpp` or `.hpp` file is added to `libs/log_lib/` that this record's grep didn't cover.
- Any future sprint touches log_lib's I/O-sink logic (FT1 RTT/UART config or hypothetical SD logging).

Re-inspection follows the same 4-step procedure and records a new entry below the Approval section (or supersedes this record with a new `INS-LOG-001-REV-B` document).

## 8. Approval

| Field | Value |
|-------|-------|
| Inspector | Software Lead |
| Date | 2026-05-05 |
| Sprint | SPRINT-IMPL-05-retro-B (RTM-cleanup amendment) |
| Verdict | **PASS** |
| Predecessor inspection | None (first inspection record for SW-REQ-LOG-008) |
| Tooling used | `grep -rnE` against working-tree at HEAD post-SPRINT-IMPL-05-retro-B closure |
