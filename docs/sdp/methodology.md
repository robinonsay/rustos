---
document_type: SDP — Methodology
program: Juno FT1 FSW
revision: B
effective_date: 2026-05-05
parent: index.md
status: Active (Revision B major amendment per PM 2026-05-05 — Pico2 unit-test coverage via pico-sdk stubs)
---

# Juno FT1 FSW SDP — Methodology

## 1. Purpose

This document is the operational handbook the Software Lead consults at every
sprint kickoff. It defines the per-sprint structure that every implementation
sprint (SPRINT-IMPL-00 through SPRINT-IMPL-25) must follow: file inventory,
worker/reviewer assignments, test authoring policy, test execution gate,
build/CI conventions, traceability tagging, lessons-learned hook, and SDP
amendment process. All wave-specific files
([foundation_libs.md](foundation_libs.md), [sensor_libs.md](sensor_libs.md),
[domain_libs.md](domain_libs.md), [sensor_apps.md](sensor_apps.md),
[domain_apps.md](domain_apps.md),
[sim_and_integration.md](sim_and_integration.md)) inherit the conventions in
this file by reference. The master sprint table and risk register live in
[index.md](index.md).

## 2. Per-Sprint Lifecycle

Every implementation sprint executes the following five phases in order. No
phase may be skipped.

1. **Phase 0 — Lead pre-flight.** The Software Lead reads the predecessor
   sprint records (`docs/sprints/SPRINT-IMPL-NN-1.md`, etc.), reads the relevant
   `ai/memory/lessons-learned-*.md` files, confirms all `Predecessors` listed
   in the wave-file sprint card are closed, and runs `python3
   tools/traceability.py` to capture the pre-sprint counter baseline.
2. **Phase 1 — Worker fan-out.** The Lead spawns one worker per file in the
   sprint's file inventory. All worker invocations are dispatched in a single
   message (parallel agent calls). Worker assignments follow §4 below; brief
   contents follow §12 Worker Brief Template.
3. **Phase 2 — Reviewer fan-out.** Once all workers return, the Lead spawns
   one reviewer per worker output. Reviewers issue `APPROVED` or `NEEDS
   CHANGES`. The Lead iterates author→review up to **three times per file**;
   if a file is not APPROVED after the third pass, the Lead halts the sprint
   and escalates to the PM.
4. **Phase 3 — Test execution gate.** Lead-direct (no agent): run gate G1
   (POSIX build + ctest), gate G2 (`tools/traceability.py`), and (if
   applicable) gate G3 (Pico2 cross-compile). Both G1 and G2 must exit 0.
5. **Phase 4 — Project Chief Engineer gate.** The Lead spawns the
   `project-chief-engineer` agent to verify the sprint acceptance criteria
   (§8). The CE issues a PASS or FAIL verdict.
6. **Phase 5 — Sprint closure.** On CE PASS, the Lead writes
   `docs/sprints/SPRINT-IMPL-NN_<module>.md` per §9 and updates the relevant
   `ai/memory/lessons-learned-*.md` files per §10. The master sprint table in
   [index.md](index.md) is updated to mark the sprint Closed.

## 3. File Inventory Template

Every implementation sprint produces exactly the file set defined by its wave
file. Library sprints produce six files; app sprints produce four files (no
posix/pico2 split for apps — apps are platform-agnostic per the L2 designs).

### Library sprint file template (six files, one per worker)

| # | File path | Worker | Reviewer |
|---|-----------|--------|----------|
| 1 | `libs/<name>_lib/include/<name>_lib/<name>_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 2 | `libs/<name>_lib/src/<name>_impl.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 3 | `libs/<name>_lib/src/posix/<name>_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 4 | `libs/<name>_lib/src/pico2/<name>_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 5 | `libs/<name>_lib/tests/<name>_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
| 6 | `libs/<name>_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

Pure-compute libs (`kmat`, `nmea`, `telem`, `mlog`) and sim-only libs may omit
the `src/posix/` and `src/pico2/` split per their L2 designs (single shared
impl). In those cases the inventory reduces to four files: api header, impl,
test, CMake. The wave file is authoritative on which sprints follow which
template.

### App sprint file template (four files, one per worker)

| # | File path | Worker | Reviewer |
|---|-----------|--------|----------|
| 1 | `apps/<name>_app/include/<name>_app/<name>_app.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 2 | `apps/<name>_app/src/<name>_app.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
| 3 | `apps/<name>_app/tests/<name>_app_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
| 4 | `apps/<name>_app/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

The composition root (`apps/main.cpp`) is authored exactly once in
SPRINT-IMPL-25 (system integration), not per app. Apps follow the single-level
`JUNO_MODULE_DERIVE(juno::app::APP_ROOT_T, ...)` pattern from the sys_app
remediation (2026-05-03).

## 4. Worker / Reviewer Assignment Policy

Per `ai/skills/software-lead.md` worker-assignment table:

| Output type | Worker | Reviewer |
|-------------|--------|----------|
| Complex `.hpp`/`.cpp` (LibJuno module pattern, algorithm, hook implementation) | `senior-software-engineer` (Sonnet) | `senior-software-engineer` reviewer mode |
| Google Test `.cpp` (test author, distinct invocation from impl) | `senior-software-engineer` (Sonnet) | `senior-software-engineer` reviewer mode |
| Boilerplate `CMakeLists.txt` and simple utility `.cpp` | `junior-software-engineer` (Haiku) | `senior-software-engineer` reviewer mode |
| Test case JSON edits (if any) | `software-systems-engineer` (Opus) | `software-mission-assurance-engineer` |

Each reviewer reviews exactly one file: one worker = one reviewer = one file.
Test authors and impl authors are always **distinct invocations** even when
both are `senior-software-engineer` — this preserves the test-as-independent-
specification property required by IEEE 1012.

## 5. Test Authoring Policy (every sprint)

- **Test framework**: Google Test (`gtest`). Tests execute on the host (POSIX
  build) under `ctest`. The Pico2 build itself still cross-compiles only — it
  does not execute tests on flight hardware — but for **dual-impl libraries**
  (libraries whose `src/pico2/<name>_pico2.cpp` differs from
  `src/posix/<name>_posix.cpp` in non-trivial algorithmic content) a separate
  `<name>_pico2_test` Google Test target is **mandatory** per §5.1 below
  (Revision B amendment, 2026-05-05).
- **Test file naming**: `<module>_test.cpp` for libs (POSIX-backend);
  `<module>_pico2_test.cpp` for the Pico2-backend host-side tests when the lib
  is dual-impl; `<module>_app_test.cpp` for apps.
- **Test fixture pattern**: Inject all dependencies via the LibJuno DI vtable
  pattern. **No mock framework.** Author static-const test vtables (e.g.,
  `static const juno::time::TIME_API_T tStubTimeApi { &StubNow, &StubSleepTo,
  &StubSleep };`) and wire them into the unit-under-test's ROOT inside the
  fixture's `SetUp()`.
- **Test naming**: `TEST_F(<Suite>, <Scenario>_<ExpectedBehavior>)`. One
  `TEST_F` per scenario; no monolithic tests that exercise multiple
  acceptance criteria.
- **Traceability tagging**: every `TEST_F` carries `// @{"verify": ["SW-REQ-
  <MODULE>-NNN"]}` immediately above the macro. See §7 for full convention.
- **Coverage**: implement every test case in
  `docs/test_cases/<module>/test_cases.json` whose `type` is `Unit`.
  Integration-type test cases are also implemented in the sprint when the
  module's L2 design or `methodology.md` exception list designates them as
  in-scope (e.g., `sim_harness` whose only authored cases are
  Integration-type since unit-level testing of the harness is meaningless
  outside a full FSW composition; covered in SPRINT-IMPL-24).
  Demonstration-type test cases are out of scope for the implementation
  sprint and are handled in HIL post-CDR.
- **Determinism**: no `sleep()`-based timing. Use the injected time vtable to
  advance simulated time. No real file-system or network I/O without an
  injected stub.
- **Acceptance criteria strength**: per the 2026-05-02 lessons-learned entry,
  every test must check observable outputs/state changes beyond status
  codes. Apply the stub-replacement mental check: if a `return SUCCESS;` stub
  would pass, the assertion is too weak.
- **App test setups**: per the 2026-05-02 lessons-learned entry on app DI
  enumeration, app-level fixtures must wire **every** dependency the app
  pulls from the composition root (scheduler, bus broker, every library used
  directly or indirectly during the cycle). Omitting a downstream
  collaborator makes the test non-reproducible.

### 5.1 Pico2-Impl Host-Side Coverage Convention (Revision B, 2026-05-05)

For every library whose Pico2 implementation TU (`src/pico2/<name>_pico2.cpp`)
contains non-trivial algorithmic content — i.e., it does anything beyond a
single platform-call passthrough — the sprint MUST produce a Pico2-backend
host test target alongside the POSIX one. The Pico2 implementation TU is
compiled into the test target unchanged; the pico-sdk free functions it
references (`time_us_64`, `sleep_until`, `sleep_us`, `from_us_since_boot`,
`uart_init`, `uart_putc_raw`, `i2c_write_blocking`, `spi_write_blocking`,
etc.) are **stubbed at link time** by host-side stub objects so the test
can drive the Pico2 algorithmic logic deterministically and assert against
both nominal and edge-case scenarios.

The convention has three artifacts per dual-impl library:

| # | Path pattern | Purpose |
|---|-------------|---------|
| 1 | `libs/<name>_lib/tests/stubs/<sdk-header-path>.h` | Stub header(s) mirroring the pico-sdk surface used by the Pico2 TU. Use the same `extern "C"` linkage and same struct layout as pico-sdk so the Pico2 source compiles against the stub identically to compiling against pico-sdk. The test target's include path **prepends** `tests/stubs/` so `pico/time.h` (or equivalent) resolves to the stub. |
| 2 | `libs/<name>_lib/tests/stubs/<name>_pico2_stub.cpp` | Stub implementations of the pico-sdk free functions, in `extern "C"` linkage. Each stub maintains test-controllable state (current-time counter, last-sleep-target, last-write-bytes, call counts) inside a `juno::test::<name>_pico2_stub` namespace. Provide a `Reset()` helper that the test fixture calls in `SetUp()`. |
| 3 | `libs/<name>_lib/tests/<name>_pico2_test.cpp` | Google Test source. Fixture wires the canonical `juno::time::TIME_ROOT_T` (or equivalent) via the existing `BindTime`/`<NAME>_LIB_IMPL_T::New` helper from the Pico2 namespace. `TEST_F` cases exercise every `SW-TC-<MODULE>-NNN` that the POSIX backend covers, **plus** Pico2-specific edge cases: stub-controlled clock advances, max-uint64 wrap, zero-duration sleep, past-target SleepTo, deterministic call-count assertions on the stub. |

CMake additions (per the SPRINT-IMPL-03 time_lib precedent):

```cmake
if(JUNO_FSW_TESTS)
    add_executable(<name>_pico2_test
        ${PROJECT_SOURCE_DIR}/src/pico2/<name>_pico2.cpp
        ${PROJECT_SOURCE_DIR}/tests/stubs/<name>_pico2_stub.cpp
        ${PROJECT_SOURCE_DIR}/tests/<name>_pico2_test.cpp
    )
    target_include_directories(<name>_pico2_test PRIVATE
        ${PROJECT_SOURCE_DIR}/tests/stubs   # PREPENDED so pico/* resolves to stub
        ${PROJECT_SOURCE_DIR}/src/pico2
    )
    target_link_libraries(<name>_pico2_test PRIVATE juno gtest gtest_main)
    target_compile_options(<name>_pico2_test PRIVATE
        ${JUNO_COMPILE_OPTIONS} ${JUNO_COMPILE_CXX_OPTIONS})
    if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
        target_compile_options(<name>_pico2_test PRIVATE -Wno-nonnull-compare)
    endif()
    add_test(NAME <name>_pico2_test COMMAND <name>_pico2_test)
endif()
```

The Pico2 production source `<name>_pico2.cpp` is **not** modified — only test
infrastructure is added. The linker chooses the stub object over pico-sdk's
real symbols at the test target only; the production Pico2 firmware build
(`PLATFORM=PICO2`) links against real pico-sdk and is unaffected.

### 5.2 Stub-state observability requirements (Revision B, 2026-05-05)

Every Pico2 stub object must expose, at minimum:

1. A current-time / current-counter variable readable & writable by tests.
2. Last-call-argument capture for any state-mutating sdk function
   (`sleep_until` target, `sleep_us` duration, `uart_putc_raw` byte,
   `i2c_write_blocking` payload, etc.).
3. A monotonic call counter per sdk function so tests can assert exact
   invocation counts (catches `Sleep` being skipped, double-sleep bugs,
   etc.).
4. A `Reset()` free function that zeroes all stub state; called by every
   `SetUp()` in the test fixture to guarantee per-test isolation.

This is the *stub-state observability* contract — without all four items,
tests cannot distinguish a stub's correct behavior from coincidental return
values.

### 5.3 Dual-impl identification rule (Revision B, 2026-05-05)

A library is "dual-impl" for the purposes of §5.1 / §5.2 if and only if both
`src/posix/<name>_posix.cpp` AND `src/pico2/<name>_pico2.cpp` exist with
**non-trivial divergent algorithmic content**. Pure-compute single-impl
libraries (`kmat_lib`, `nmea_lib` in Wave 1; the algorithmic libs in Wave 4)
are exempt. The wave-file sprint card is authoritative on which sprints are
dual-impl; a sprint card stating "dual-impl" implicitly mandates §5.1
compliance.

## 6. Test Execution Gate (per-sprint exit criterion)

Two gates run by the Lead in Phase 3 must pass before sprint closure. A third
runs when the sprint touches `src/pico2/`.

### Gate G1 — POSIX build + tests pass

```bash
mkdir -p build_posix && cd build_posix && \
  cmake -DJUNO_FSW_POSIX=ON -DJUNO_FSW_TESTS=ON .. && \
  cmake --build . && \
  ctest --output-on-failure
```

Must exit 0. All `SW-TC-<MODULE>-*` test cases for the sprint's module pass.
For dual-impl libraries (per §5.3), **both** the POSIX-backend
(`<name>_test`) and the Pico2-backend host-side (`<name>_pico2_test`) ctest
targets must run and PASS — exact same `SW-TC-<MODULE>-*` coverage on each
side, plus the Pico2-specific edge cases enumerated in §5.1 row 3
(Revision B amendment, 2026-05-05).

### Gate G2 — Traceability clean

```bash
python3 tools/traceability.py
```

Must exit 0. The counter delta versus the Phase 0 baseline shows
`Requirements with code` and `Requirements with @verify` increased by the
count of requirements covered by this sprint.

### Optional Gate G3 — Pico2 cross-compile passes

Required for any sprint that authors or modifies `libs/<name>_lib/src/pico2/`
files.

```bash
mkdir -p build_pico2 && cd build_pico2 && \
  cmake .. && \
  cmake --build .
```

(Pico2 is the default target when `JUNO_FSW_POSIX` is unset; see top-level
`CMakeLists.txt`.)

Must exit 0. No test execution on Pico2 (cross-compile constraint).

## 7. Traceability Tagging Conventions

Per `ai/memory/traceability.md`:

### Source code tagging

Place immediately above the function that implements the requirement:

```cpp
// @{"req": ["SW-REQ-<MODULE>-NNN"]}
JUNO_STATUS_T <Module>_<Function>(<MODULE>_LIB_ROOT_T &tRoot, ...) noexcept
```

Multiple requirements per function:

```cpp
// @{"req": ["SW-REQ-<MODULE>-001", "SW-REQ-<MODULE>-002"]}
```

### Test tagging

Immediately above each `TEST_F`:

```cpp
// @{"verify": ["SW-REQ-<MODULE>-NNN"]}
TEST_F(<Suite>, <Scenario>_<ExpectedBehavior>) { ... }
```

### Design doc tagging

Already present in the baseline as HTML comments above design sections.
Implementation sprints **do not modify** design documents — any divergence
discovered during implementation is logged as an SDP amendment per §11.

## 8. Per-Sprint Acceptance Criteria Checklist

Every sprint must satisfy all eleven of the following before Phase 4 (Chief
Engineer gate). The reviewer of each file checks the per-file projection of
this list (see §12 Reviewer Brief Template).

1. All files in the sprint's file inventory are authored and reviewer-
   APPROVED.
2. Compiler flags clean: `-std=c++11 -Wall -Wextra -Werror -pedantic
   -Wshadow -Wcast-align -Wundef -Wswitch -Wswitch-default -fno-rtti
   -fno-exceptions -fno-common -fno-strict-aliasing`. No warnings.
3. Memory model clean: zero dynamic allocation; no `new`, `delete`, `malloc`,
   `free`, or heap-backed STL containers.
4. LibJuno module pattern compliance: ROOT/API/IMPL split; vtable wired once
   in `New()`; `noexcept` on every entry point; `JUNO_MODULE_ROOT` /
   `JUNO_MODULE_DERIVE` macros used per `libjuno/include/juno/module.h`
   (lines 97, 131, 161).
5. Cross-module API references resolved against actual peer headers — no
   fabricated symbol names. Per the 2026-05-03 lessons-learned entry, any
   LibJuno symbol referenced in a brief must be cross-checked against
   `libjuno/include/juno/...` before authoring.
6. POSIX/Pico2 dual implementation present where applicable per the L2
   design; single-impl libs (`kmat`, `nmea`, `telem`, `mlog`) document the
   deviation per their L2 design.
7. Test cases: every `Unit`-type `SW-TC-<MODULE>-*` from
   `docs/test_cases/<module>/test_cases.json` is implemented; every test
   passes; every test is tagged with the matching `SW-REQ-*` ID.
8. Vtable dispatch idiom: `tRoot.ptApi->Hook(...)` — never
   `tRoot.tApi->...`. The pointer name is `ptApi` per `JUNO_MODULE_ROOT`.
9. Time conversions use the published member-function shape:
   `_ptTime->TimestampToMicros(<JUNO_TIMESTAMP_T>).tOk` per
   `libjuno/include/juno/time/time_api.hpp:142`. Never invent a free-function
   `juno::time::TimestampToMicros(...)` form.
10. Gate G1 + G2 + G3 (where applicable) all exit 0.
11. Project Chief Engineer issues a PASS verdict on the sprint.

## 9. Sprint Closure Record

Each sprint produces a record at `docs/sprints/SPRINT-IMPL-NN_<module>.md`
(amended 2026-05-04 from `ai/sprints/` per PM direction at SPRINT-IMPL-00 closure)
containing:

- Sprint ID, module, start/end dates
- Worker invocations: count and iterations per file
- Reviewer verdicts: file, iteration count, final verdict
- Gate output: stdout snippets for G1, G2, and (if applicable) G3
- Chief Engineer verdict and rationale
- Lessons learned: cross-referenced into the relevant
  `ai/memory/lessons-learned-*.md` files per §10

## 10. Lessons-Learned Hook

After every sprint (Phase 5), the Lead updates the relevant lessons-learned
file per the `ai/skills/software-lead.md` Lessons Learned Protocol:

| Issue category | Target file |
|----------------|-------------|
| Planning / decomposition issues | `ai/memory/lessons-learned-software-lead.md` |
| Code implementation issues | `ai/memory/lessons-learned-senior-software-engineer.md` |
| Boilerplate / scaffolding issues | `ai/memory/lessons-learned-junior-software-engineer.md` |
| Final-gate issues | `ai/memory/lessons-learned-project-chief-engineer.md` |
| Requirement / test-case authoring issues | `ai/memory/lessons-learned-software-systems-engineer.md` |

Format per `ai/skills/software-lead.md`:

```markdown
### YYYY-MM-DD — <Short Title>
**What happened:** <description>
**Root cause:** <why>
**Corrective action:** <what to do differently>
```

Each entry must be concise (target ≤6 lines) and actionable. Future sprint
briefs cite these entries by date so workers do not re-raise resolved issues.

## 10.1 Revision History

| Revision | Date | Change | Approved by |
|----------|------|--------|-------------|
| A | 2026-05-03 | Initial SDP authoring sprint | CE 2026-05-03; PM 2026-05-04 |
| B | 2026-05-05 | Pico2 unit-test coverage via pico-sdk stubs (§5.1, §5.2, §5.3, §6 Gate G1 amendments). Surfaced by SPRINT-IMPL-03 closure when PM identified the gap. Establishes the convention for all future dual-impl sprints (SPRINT-IMPL-05/06/07/08/09/10/11). Retro-applies to log_lib via SPRINT-IMPL-02-retro (queued). | PM 2026-05-05 |

## 11. SDP Amendment Process

The SDP is a living document. Between sprints, the Software Lead may propose
amendments to address discovered gaps (e.g., circular deps, missing fixture
pattern, scope changes from the PM). The process:

1. Lead drafts an amendment proposal: which file changes, why, sprint impact.
2. PM reviews; if approved, Lead applies edits with the revision letter
   incremented in the file's front-matter (`revision: A` → `revision: B`).
3. The master sprint table in [index.md](index.md) is updated; cross-
   references in all wave files are repaired.
4. A note is logged in `ai/memory/lessons-learned-software-lead.md` capturing
   the amendment rationale.

**Major amendments** (e.g., wave reordering, new sprint added, file inventory
template change) require explicit PM signature.
**Minor amendments** (e.g., test-count corrections, file-path tweaks,
typo fixes) may be Lead-direct with PM notification only.

## 12. Cross-Cutting Policies

### LibJuno upstream change handling

If LibJuno publishes a previously-missing symbol (e.g., `juno::app::AppInit`),
the next implementation sprint that consumes apps may switch from the FSW
workaround to the LibJuno-published symbol. The Lead updates the closure memo
referenced from [index.md](index.md) §5 to reflect the change and revises the
affected app sprint's brief to use the published symbol. Per the 2026-05-03
PDR-review lesson, briefs must include `libjuno/include/juno/` paths (not
just template directories), and any "undefined type" reviewer finding is
cross-checked against upstream headers before escalation.

### Worker brief template

Every worker brief in every implementation sprint must include:

1. The exact output file path (exactly one file per worker).
2. The acceptance criteria for that file (per-file projection of §8).
3. The cross-file conventions snippet quoted in [index.md](index.md):
   canonical names (`JUNO_MODULE_ROOT`, `JUNO_MODULE_DERIVE`, `ptApi`
   dispatch, `juno::time::TimeInit`, `_ptTime->TimestampToMicros(...).tOk`,
   `juno::sch::SCH_API_T<8, 200>::Execute`), the 19 canonical status codes,
   and the C++11 freestanding constraints.
4. Common review traps (single `parent_id`, `tApi` vs `ptApi`, fabricated
   names, file-length 500-LoC limit, atomicity violations on "X and Y"
   compounds).
5. Skill-file reference and the relevant
   `ai/memory/lessons-learned-<role>.md` reference.
6. The carry-forward RFA list so workers do not re-raise resolved items.

### Reviewer brief template

Every reviewer brief must include:

1. The exact file under review.
2. The per-file projection of the eleven acceptance criteria from §8.
3. The cross-file conventions snippet from [index.md](index.md).
4. The common review traps list (mirroring the worker brief).
5. Verdict format: `APPROVED` or `NEEDS CHANGES` with `file:line` references
   for each finding.
6. Iteration cap: a file may be re-authored up to three times. After the
   third NEEDS CHANGES verdict the Lead halts the sprint per §2 Phase 2.

### File-length limit

Every authored file (source, header, test, design, requirements JSON, test-
case JSON) must remain ≤500 lines per `ai/memory/constraints.md`. Files that
exceed the limit must be split (for design docs, use an `index.md` with
linked sub-files; for source, refactor into helper translation units).

### Cross-module enum and unit conventions

Per the 2026-05-02 lessons-learned entry on cross-module conventions, any
shared enum (phase names, frame names, time-base format, units) is pinned in
the SYS L1 requirements with exact spelling. Every L2 worker brief must quote
the SYS L1 description **verbatim** for the relevant enum — no paraphrase, no
local invention.
