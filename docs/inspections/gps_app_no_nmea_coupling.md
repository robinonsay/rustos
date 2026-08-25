---
inspection_id: INS-GPS-APP-003
requirement: SW-REQ-GPS-APP-003
test_case: SW-TC-GPS-APP-003
verification_method: Inspection
inspection_date: 2026-05-12
inspector: Software Lead
verdict: PASS
sprint_context: SPRINT-WAVE5-EXIT Wave 5 carry-forward discharge
---

# Inspection Record — SW-REQ-GPS-APP-003 Delegate NMEA Parsing to NMEA Library

## 1. Requirement Under Inspection

**SW-REQ-GPS-APP-003** — *Delegate NMEA Parsing to NMEA Library*

> The GPS application shall delegate NMEA sentence parsing to the NMEA library.

- `verification_method`: **Inspection** (per [docs/requirements/gps_app/requirements.json](../requirements/gps_app/requirements.json))
- Rationale: layering declaration; keeps the view-layer `gps_app` decoupled from the parser. NMEA parsing is owned by `nmea_lib`, transitively consumed through `gps_lib`'s `GetRawNmea`/`GetFix`/`GetUtc` API surface. `gps_app` is a publisher that never touches NMEA byte syntax directly.
- Test case: **SW-TC-GPS-APP-003** (per [docs/test_cases/gps_app/test_cases.json](../test_cases/gps_app/test_cases.json))

## 2. Inspection Procedure

Per SW-TC-GPS-APP-003's `procedure` array (4 steps):

1. Inspect `apps/gps_app/CMakeLists.txt` for link or include dependencies on `nmea_lib` or `NMEA_LIB_*` build targets.
2. Inspect all `apps/gps_app/include/` and `apps/gps_app/src/` files for `#include` directives referencing `nmea_lib` headers.
3. Inspect all `apps/gps_app/include/` and `apps/gps_app/src/` files for API call references to `NMEA_LIB_*` types or `juno::nmea::*` symbols (excluding doc-comment narrative references).
4. Confirm `apps/gps_app/tests/` likewise contains no `nmea_lib` coupling (the test fixture must not reach around `gps_lib` to `nmea_lib` directly).

## 3. Acceptance Criterion

> `apps/gps_app/` declares no build dependency on, includes from, or API symbol references to `nmea_lib` or `juno::nmea::*`. Doc-comment narrative references in prose are allowed.

## 4. Evidence

Inspection performed by line-by-line review of every source file under `apps/gps_app/` at HEAD post-SPRINT-IMPL-18 closure (commit baseline: `c7ecd39ac` — sprint-closure commit for gps_app). Equivalent shell commands documented per step for reproducibility.

### 4.1 Step 1 — CMakeLists.txt link/include audit

**Command:**
```
grep -nE "nmea_lib|NMEA_LIB_" apps/gps_app/CMakeLists.txt
```

**Result:** **0 hits.**

**Link declarations** (lines 62, 65, 68, 104-112):
```
target_link_libraries(${PROJECT_NAME} PUBLIC juno)
target_link_libraries(${PROJECT_NAME} PUBLIC gps_lib)
target_link_libraries(${PROJECT_NAME} PUBLIC time_lib)
target_link_libraries(gps_app_test PRIVATE ${PROJECT_NAME} juno gps_lib time_lib gtest gtest_main pthread)
```

`juno` is LibJuno (no NMEA layer). `gps_lib` is the canonical GPS driver that transitively consumes `nmea_lib`; the dependency is mediated. `time_lib` provides timestamping. Neither the library target nor the test target links `nmea_lib` directly. **PASS**

### 4.2 Step 2 — Header/source `#include` audit

**Command:**
```
grep -rnE "#include[[:space:]]+[<\"]nmea_lib/" apps/gps_app/include apps/gps_app/src
```

**Result:** **0 hits.** No `#include` directive in `apps/gps_app/include/gps_app/gps_app.hpp` or `apps/gps_app/src/gps_app.cpp` references any `nmea_lib/*` header.

Actual include set in `apps/gps_app/include/gps_app/gps_app.hpp` (lines 61-70):
```
#include <stdint.h>
#include "juno/app/app_api.hpp"
#include "juno/module.h"
#include "juno/sb/broker_api.hpp"
#include "juno/status.h"
#include "juno/time/time_api.hpp"
#include "gps_lib/gps_api.hpp"
#include "gps_lib/gps_msg.hpp"
#include "juno_fsw_capacities.hpp"
#include "juno_msg_bus_variant.hpp"
```

Actual include set in `apps/gps_app/src/gps_app.cpp` (lines 60-63):
```
#include "gps_app/gps_app.hpp"
#include "gps_lib/gps_msg.hpp"
#include "juno/macros.h"
#include <string.h>
```

NMEA parsing is reached only transitively through `gps_lib/gps_api.hpp`. **PASS**

### 4.3 Step 3 — Header/source symbol audit (API references)

**Command:**
```
grep -rnE "NMEA_LIB_|juno::nmea::|nmea_lib::" apps/gps_app/include apps/gps_app/src
```

**Result:** **0 hits** for forbidden API symbols.

For completeness, a broader audit (case-sensitive token `nmea_lib` anywhere in source, including comments):
```
grep -rnE "nmea_lib|NMEA_LIB_" apps/gps_app/include apps/gps_app/src
```

| File | Line | Lexical context | Classification |
|------|------|-----------------|----------------|
| `apps/gps_app/include/gps_app/gps_app.hpp` | 151 | `*        NMEA parsing is delegated to gps_lib/nmea_lib transitively — gps_app` | doc-comment narrative (allowed) |
| `apps/gps_app/include/gps_app/gps_app.hpp` | 152 | `*        has no direct nmea_lib dependency (SW-REQ-GPS-APP-003, L2 §1).` | doc-comment narrative (allowed) |
| `apps/gps_app/src/gps_app.cpp` | 40 | ` *  - No direct nmea_lib include or call (SW-REQ-GPS-APP-003, L2 §1 / AC-9).` | doc-comment narrative (allowed) |
| `apps/gps_app/src/gps_app.cpp` | 244 | ` *     (gps_lib internal; no nmea_lib call here — AC-9 / SW-REQ-GPS-APP-003).` | doc-comment narrative (allowed) |
| `apps/gps_app/src/gps_app.cpp` | 268 | ` *  gps_lib/nmea_lib transitively.  This file includes NO nmea_lib header` | doc-comment narrative (allowed) |
| `apps/gps_app/src/gps_app.cpp` | 269 | ` *  and makes NO nmea_lib API calls; SW-REQ-GPS-APP-003 is satisfied by` | doc-comment narrative (allowed) |

All 6 lexical hits in the gps_app library sources are within `/** ... */` doc-comment blocks; **zero** are `#include` directives, `using` declarations, type references, or function-call expressions. Every hit asserts the absence of coupling — they are observational prose only. **PASS**

### 4.4 Step 4 — Test source audit

**Command:**
```
grep -rnE "nmea_lib|NMEA_LIB_|juno::nmea::" apps/gps_app/tests
```

**Result:** **2 hits**, both classified narrative-only.

| File | Line | Lexical context | Classification |
|------|------|-----------------|----------------|
| `apps/gps_app/tests/gps_app_test.cpp` | 176 | `// (delegate NMEA parsing to nmea_lib) has verification_method=Inspection; the` | doc-comment narrative (allowed) |
| `apps/gps_app/tests/gps_app_test.cpp` | 178 | `// confirming zero `nmea_lib`/`NMEA_LIB_*`/`#include "nmea_lib/..."` references` | doc-comment narrative (allowed) |

Both hits are in the TC-003 regression-guard preamble comment block (lines 175-181) that explains why the TEST_F carries no `@verify` tag and points future inspectors at this very record. Neither is a `#include`, a using-declaration, or a symbol reference. The test fixture itself wires `gps_lib` stubs only — no `nmea_lib` types appear in the StubGps* function signatures, fixture members, or message payload extractors.

**Hit-count drift note vs. GPS-18 closure record:** The SPRINT-IMPL-18 closure record §10 carry-forward #4 reported a 5-hit count for the `nmea_lib`/`NMEA_LIB_*` doc-comment narrative survey. This inspection's complete enumeration returns **8 hit-lines** across all four files (CMake: 0; hpp: 2; cpp: 4; test: 2). All 8 are narrative-only and the substantive PASS verdict (zero forbidden coupling) is unchanged. The 5-vs-8 delta is an accounting recount — GPS-18 likely undercounted cpp:268, cpp:269, and test.cpp:176, all of which are narrative-only.

Actual include set in `apps/gps_app/tests/gps_app_test.cpp` (lines 18-27):
```
#include <gtest/gtest.h>
#include <string.h>
#include <stdint.h>
#include "gps_app/gps_app.hpp"
#include "gps_lib/gps_api.hpp"
#include "gps_lib/gps_msg.hpp"
#include "juno/sb/broker_api.hpp"
#include "juno/time/time_api.hpp"
#include "juno_msg_bus_variant.hpp"
#include "juno_fsw_capacities.hpp"
```

The `juno::gps::NMEA_RAW_T` type used by the stub (gps_app_test.cpp:43, :69-73) is published by `libs/gps_lib/include/gps_lib/gps_api.hpp` — it is the `gps_lib` verbatim-NMEA-buffer DTO, **not** an `nmea_lib` type. `gps_lib` itself is the only translation unit that consumes `nmea_lib`. **PASS**

## 5. Verdict

**PASS** — `apps/gps_app/` is genuinely separated from `nmea_lib`. SW-REQ-GPS-APP-003 is verified by Inspection per its declared method:

- Build: zero `nmea_lib` link or include dependency in `apps/gps_app/CMakeLists.txt`.
- Source: zero `#include "nmea_lib/..."` directives in any `apps/gps_app/include/` or `apps/gps_app/src/` file.
- Symbols: zero `NMEA_LIB_*`, `juno::nmea::*`, or `nmea_lib::*` API references in any `apps/gps_app/` library or test source.
- Narrative: 8 doc-comment / regression-guard-preamble prose hit-lines (CMake: 0; hpp: 2; cpp: 4; test: 2), all classified as observational narrative (allowed by the acceptance criterion).

NMEA parsing is reached exclusively through `gps_lib`'s `Poll`/`GetRawNmea`/`GetFix`/`GetUtc` API surface, which is the architectural layering specified by the L2 design.

## 6. Related Test Artifact (informational)

[`apps/gps_app/tests/gps_app_test.cpp`](../../apps/gps_app/tests/gps_app_test.cpp) contains a `TEST_F` named `OnProcess_DelegatesParsingViaGpsLibOnly` (test.cpp:182) that asserts exactly one `Poll`, one `GetFix`, and one `GetRawNmea` stub-invocation per `OnProcess` tick — a **narrow regression guard** preventing reintroduction of any direct `nmea_lib` call path. It verifies dispatch shape only, NOT the broader "no link/include/symbol coupling" assertion which this inspection record covers. The TEST_F does NOT carry a `@verify` tag for SW-REQ-GPS-APP-003 (intentionally — Test artifacts must not claim verification of Inspection-method requirements per IEEE 829). The omission was applied as a Phase 3 Lead-direct atomic correction in SPRINT-IMPL-18 (see `docs/sprints/SPRINT-IMPL-18_gps_app.md` §6 "Phase 3 Lead-direct atomic correction") and cites the SPRINT-IMPL-05-retro-B RTM-cleanup lesson 2026-05-05.

## 7. Re-Inspection Triggers

This inspection record must be re-executed (and re-signed) when any of the following changes:

- A new `.cpp` or `.hpp` file is added to `apps/gps_app/` (under `include/`, `src/`, or `tests/`) that this record's grep did not cover.
- `apps/gps_app/CMakeLists.txt` link/include declarations change.
- Any future sprint touches `gps_app`'s NMEA handling, introduces a direct `nmea_lib` seam (e.g., a pty-driven integration test bypassing `gps_lib`), or modifies the `gps_lib` API surface in a way that exposes `nmea_lib` types into `apps/gps_app/`.

Re-inspection follows the same 4-step procedure and records a new entry below the Approval section (or supersedes this record with a new `INS-GPS-APP-003-REV-B` document).

## 8. Approval

| Field | Value |
|-------|-------|
| Inspector | Software Lead |
| Date | 2026-05-12 |
| Sprint | SPRINT-WAVE5-EXIT (Wave 5 Exit Gate + GPS-18 carry-forward discharge) |
| Verdict | **PASS** |
| Predecessor inspection | None (first inspection record for SW-REQ-GPS-APP-003) |
| Tooling used | Line-by-line source review with `grep -rnE` equivalents documented per step against working-tree at HEAD post-SPRINT-IMPL-18 closure |
