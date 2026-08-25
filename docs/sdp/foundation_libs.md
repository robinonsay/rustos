---
document_type: SDP — Foundation Libraries (Wave 0, 1, 2)
program: Juno FT1 FSW
revision: B
effective_date: 2026-05-05
parent: index.md
sprints_covered: SPRINT-IMPL-00 through SPRINT-IMPL-06
status: Active (Revision B amendment 2026-05-05 — SPRINT-IMPL-03 file inventory expanded with three Pico2-test stub artifacts per methodology.md §5.1; SPRINT-IMPL-05/06 sprint cards must adopt the same convention)
---

# SDP — Wave 0, 1, 2: Foundation Libraries

## 1. Purpose

This file details seven implementation sprints that produce the project-wide
enabler headers (Wave 0), the foundation libraries with no inter-library
dependencies (Wave 1: kmat_lib, log_lib, time_lib, nmea_lib), and the
platform libraries that depend only on `time_lib` (Wave 2: device_lib,
sch_lib). After every sprint covered here closes successfully, every sensor
driver and domain library can begin (Wave 3+ in `sensor_libs.md` and
`domain_libs.md`). All sprints follow the per-sprint structure pinned in
[`methodology.md`](methodology.md) and gate on the standard test pipeline
documented in §5 of that file. The file references but does not redefine the
conventions captured in [`index.md`](index.md).

## 2. Wave Summary

| Wave | Sprints | Modules | Predecessor Wave | Successor Waves |
|------|---------|---------|------------------|-----------------|
| 0 | SPRINT-IMPL-00 | bus_variant + capacity pins | none (PDR baseline) | enables all Wave 5+ |
| 1 | SPRINT-IMPL-01..04 | kmat_lib, log_lib, time_lib, nmea_lib | none (LibJuno only) | Wave 2, Wave 3, Wave 4 |
| 2 | SPRINT-IMPL-05..06 | device_lib, sch_lib | Wave 1 (time_lib) | Wave 3, Wave 5+ |

Per-sprint files / requirements / unit-test totals across this file:
**59 SW-REQ-* IDs** retired (15 KMAT + 8 LOG + 7 TIME + 12 NMEA + 7 DEVICE +
10 SCH); **76 SW-TC-* test cases** authored (20 KMAT + 12 LOG + 7 TIME +
17 NMEA + 8 DEVICE + 12 SCH). The single TIME demonstration test
(`SW-TC-TIME-008`) is deferred to the FT1 hardware-bring-up demo procedure
in `sensor_apps.md` Wave 5 — it is **not** in scope for SPRINT-IMPL-03's
exit gate.

## 3. Per-Sprint Plans

### SPRINT-IMPL-00 — Bus Variant + Capacity Pins (Wave 0 enabler)

- **Module**: project-wide FSW headers (no L2 design dir; sources from
  `docs/design/system/system_design.md` §4 message catalog and the per-app
  / per-lib L2 designs that already reference these symbols).
- **Predecessors**: none (PDR baseline).
- **Carry-forward RFAs resolved**: SDP-R-02 (`JUNO_MSG_BUS_VARIANT_T`
  publication); SDP-R-03 (capacity placeholder pins
  `kBrokerPipes`, `kBrokerRegistry`,
  `juno::sd::kDefaultWriteBufBlocks`, `juno::device::kDefaultRingCap`).
- **Files to produce** (one per worker — 2 files, header-only):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `apps/include/juno_msg_bus_variant.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `apps/include/juno_fsw_capacities.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: 0 (header-only; coverage verified by inclusion in
  next-sprint compile of `kmat_test.cpp` and the SPRINT-IMPL-05/06 driver
  source files that consume `juno::device::kDefaultRingCap` and
  `juno::sd::kDefaultWriteBufBlocks`).
- **Acceptance criteria**:
  1. `juno_msg_bus_variant.hpp` defines `JUNO_MSG_BUS_VARIANT_T` and
     publishes per-MID member tags covering every bus message enumerated
     in `docs/design/system/system_design.md` §4 plus the lifecycle-state
     message from `conventions.md` §4.7 (12 total):
     `JUNO_MSG_IMU_SAMPLE_T`, `JUNO_MSG_BARO_SAMPLE_T`,
     `JUNO_MSG_GPS_FIX_T`, `JUNO_MSG_GPS_NMEA_RAW_T`,
     `JUNO_MSG_GPS_UTC_T`, `JUNO_MSG_NAV_STATE_T`,
     `JUNO_MSG_AFM_PHASE_T`, `JUNO_MSG_SYS_HEALTH_T`,
     `JUNO_MSG_SYS_POST_T`, `JUNO_MSG_TELEM_PACKET_T`,
     `JUNO_MSG_MLOG_RECORD_T`, `JUNO_MSG_SYS_STATE_T`.
     (Minor SDP amendment 2026-05-04 per SPRINT-IMPL-00 CE PASS verdict:
     the original draft listed 11 names with `JUNO_MSG_SYS_STATE_T`
     substituted for `JUNO_MSG_TELEM_PACKET_T`; both messages are real and
     are now both listed.)
  2. `juno_fsw_capacities.hpp` defines all four capacity constants
     (`juno::broker::kBrokerPipes = 8`,
     `juno::broker::kBrokerRegistry = 64`,
     `juno::sd::kDefaultWriteBufBlocks = 4`,
     `juno::device::kDefaultRingCap = 2048`) with values matching the
     L2 design references; each constant carries a `static_assert` for
     non-zero / power-of-two as appropriate.
  3. Both headers are `#pragma once`-guarded; freestanding-compatible
     (no STL containers, no heap, no exceptions); use only LibJuno and
     `<cstdint>` / `<cstddef>` includes.
  4. Headers parse cleanly when included into a smoke-test `.cpp` under
     both POSIX (`-DPLATFORM=POSIX`) and Pico2 (`-DPLATFORM=PICO2`)
     CMake configurations; no symbol collision with LibJuno's
     `juno::broker::*` template parameters.
  5. **Gate G1** (build): both POSIX and Pico2 cross-compile of the
     smoke-test `.cpp` succeed.
  6. **Gate G2** (traceability): `python3 tools/traceability.py`
     exits 0 (no requirement coverage delta — these are infrastructure
     headers that introduce no SW-REQ tags).
- **Test gate**: header parse only (no ctest entry yet; inclusion-based
  validation closed by SPRINT-IMPL-01 first build).
- **Estimated agent count**: 2 workers + 2 reviewers + 1 CE = 5 agents.

### SPRINT-IMPL-01 — kmat_lib (Wave 1)

- **Module**: `kmat_lib` — pure-compute kinematic / matrix /
  quaternion math; **header-only** per L2 (no posix/pico2 split).
- **Predecessors**: none (LibJuno + SPRINT-IMPL-00 headers only).
- **L2 design**: split file —
  [`docs/design/kmat/index.md`](../design/kmat/index.md),
  [`docs/design/kmat/04_interface.md`](../design/kmat/04_interface.md),
  [`docs/design/kmat/05_through_11.md`](../design/kmat/05_through_11.md).
- **Requirements**: **15** SW-REQ-KMAT-* IDs (`SW-REQ-KMAT-001` through
  `SW-REQ-KMAT-015`).
- **Files to produce** (4 files; kmat is header-only per L2 §4):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/kmat_lib/include/kmat_lib/kmat_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/kmat_lib/include/kmat_lib/kmat_impl.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/kmat_lib/tests/kmat_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 4 | `libs/kmat_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: **20** Unit-type entries
  (`SW-TC-KMAT-001` through `SW-TC-KMAT-020`).
- **Acceptance criteria** (methodology.md §8 baseline plus
  module-specifics):
  1. All 15 SW-REQ-KMAT-* tagged in `kmat_impl.hpp` (`@{"req":[...]}`)
     and 20 SW-TC-KMAT-* tagged in `kmat_test.cpp` (`@{"verify":[...]}`).
  2. Pivot-tiebreak rule for `Invert` documented (matches L2 §4.2.6
     `kPivotEpsilon<T>`) and exercised by `SW-TC-KMAT-009` /
     `-010` (singular and near-singular).
  3. `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` declared as
     `JUNO_STATUS_CUSTOM_ERROR + 1` in `kmat_api.hpp`
     (matches L2 §4.7); returned by `Invert` and `QuatNormalize`.
  4. CMakeLists explicitly handles libm linkage on POSIX
     (`target_link_libraries(... PUBLIC m)` guarded by
     `if(NOT PLATFORM STREQUAL "PICO2")`); Pico2 inherits libm via
     pico-sdk newlib.
  5. Quaternion storage is `juno::math::QUAT<T>::arr[4]` (re-exported from
     `juno::math` per `docs/design/kmat/index.md` §3.4 REV B), with
     scalar-first Hamilton mapping `arr[0]=w, arr[1]=x, arr[2]=y, arr[3]=z`
     per L2 §4.6 / `SW-REQ-SYS-041`; documented in `kmat_api.hpp` and
     verified by inspection.
  6. `-fno-rtti` and `-fno-exceptions` enforced on every kmat TU
     (verified by `SW-TC-KMAT-017` / `-018` symbol grep).
  7. **Gate G1** (build), **G2** (traceability), **G3** (Pico2
     cross-compile freestanding) all exit 0.
- **Test gate**: G1 + G2 mandatory; G3 confirms freestanding compliance
  (header-only template instantiation under `-ffreestanding`).
- **Estimated agent count**: 4 workers + 4 reviewers + 1 CE = 9 agents.

### SPRINT-IMPL-02 — log_lib (Wave 1)

- **Module**: `log_lib` — diagnostic severity-tagged logger (DEBUG / INFO
  / WARN / ERROR); **dual-impl** (POSIX `stderr` + Pico2 UART/RTT) per
  L2 §3.
- **Predecessors**: none (LibJuno only; SPRINT-IMPL-00 not strictly
  required because log_lib carries no bus dependency).
- **L2 design**: [`docs/design/log/design.md`](../design/log/design.md).
- **Requirements**: **8** SW-REQ-LOG-* IDs (`SW-REQ-LOG-001` through
  `SW-REQ-LOG-008`).
- **Files to produce** (6 files; dual-impl):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/log_lib/include/log_lib/log_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/log_lib/src/common/log_common.cpp` (severity-prefix lookup, vsnprintf format scratch buffer, gating) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/log_lib/src/posix/log_posix.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/log_lib/src/pico2/log_pico2.cpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/log_lib/tests/log_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/log_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: **12** Unit-type entries
  (`SW-TC-LOG-001` through `SW-TC-LOG-012`).
- **Acceptance criteria** (methodology §8 baseline plus):
  1. All 8 SW-REQ-LOG-* tagged in `log_common.cpp` /
     `log_posix.cpp` / `log_pico2.cpp` and all 12 SW-TC-LOG-* tagged
     in `log_test.cpp`.
  2. `kLogMaxRecord = 256` pinned as `static constexpr` in `log_api.hpp`
     (matches L2 §4.2); `vsnprintf` is the only stdlib formatting
     primitive used; no `vasprintf` / `asprintf` / `open_memstream`
     anywhere (verified by `SW-TC-LOG-006` symbol inspection).
  3. POSIX sink wires to `stderr` fd (resolves L2 FLAG-1: design
     adopted `stderr`; rationale prose update was deferred and is **not**
     blocking for this sprint).
  4. Pico2 sink defaults to UART; RTT is opt-in via build flag
     `LOG_LIB_PICO2_USE_RTT` (matches L2 FLAG-4). Both impls
     non-blocking: drop-newest on FIFO-full returning
     `JUNO_STATUS_WRITE_ERROR`.
  5. Vtable construction once at `New()` as file-scope `static const`;
     no global mutable state (verified by `SW-TC-LOG-010` /
     `-011` source inspection).
  6. `log_lib` does **not** include `mlog_lib`, SD, or filesystem
     headers (verified by `SW-TC-LOG-012`).
  7. **Gate G1, G2, G3** all exit 0.
- **Test gate**: G1 + G2 + G3.
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents.

### SPRINT-IMPL-03 — time_lib (Wave 1)

- **Module**: `time_lib` — FT1 platform implementations of LibJuno's
  `juno::time::TIME_API_T` (`Now`, `SleepTo`, `Sleep`); **dual-impl**
  (POSIX `clock_gettime(CLOCK_MONOTONIC)` + Pico2 `time_us_64()`) per
  L2 §4.1 / §4.2 / §4.3. **This is the time root that every Wave 2 and
  Wave 3+ sprint depends on.**
- **Predecessors**: none (consumes LibJuno's published `TIME_ROOT_T` /
  `TIME_API_T` / `TimeInit` only).
- **L2 design**: [`docs/design/time/design.md`](../design/time/design.md).
- **Requirements**: **7** SW-REQ-TIME-* IDs (`SW-REQ-TIME-001` through
  `SW-REQ-TIME-007`).
- **Files to produce** (8 files; LibJuno owns the canonical types so
  there is no FSW-side `_api.hpp` — platform impl headers and bodies, the
  POSIX test, the CMake, **and three new Pico2-test stub artifacts per
  methodology.md §5.1 Revision B amendment**):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/time_lib/src/posix/time_posix.hpp` (impl-private struct + free-function decls) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/time_lib/src/posix/time_posix.cpp` (`PosixNow` / `PosixSleepTo` / `PosixSleep` + `static const TIME_API_T tApi{...}`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/time_lib/src/pico2/time_pico2.hpp` + `libs/time_lib/src/pico2/time_pico2.cpp` (paired Pico2 impl) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/time_lib/tests/time_test.cpp` (POSIX-backend Google Test) | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/time_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |
  | 6 | `libs/time_lib/tests/stubs/pico/time.h` (host-side stub of pico-sdk `pico/time.h` — Revision B addition) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 7 | `libs/time_lib/tests/stubs/pico_time_stub.cpp` (host-side stub implementations + test-controllable state — Revision B addition) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 8 | `libs/time_lib/tests/time_pico2_test.cpp` (Pico2-backend Google Test exercising `time_pico2.cpp` against the stub — Revision B addition) | senior-software-engineer (test author, distinct invocation) | senior-software-engineer (reviewer mode) |

  Note: Item 3 bundles two physical files into one worker ticket
  because the Pico2 source is short and the `.hpp` is a private impl
  header (≤30 lines). Items 6, 7, 8 were added by the Revision B
  amendment after the PM identified the Pico2 host-side test-coverage
  gap during SPRINT-IMPL-03 closure (2026-05-05). The Pico2
  production source `time_pico2.cpp` is **not** modified by this
  expansion — only test infrastructure is added.

- **Test cases**: **7** Unit-type entries
  (`SW-TC-TIME-001` through `SW-TC-TIME-007`). The single
  Demonstration `SW-TC-TIME-008` (Pico2 hardware print-stream) is
  **deferred** to the Wave 5 hardware-bring-up demo procedure and is
  not in scope for this sprint.
- **Acceptance criteria** (methodology §8 baseline plus):
  1. All 7 SW-REQ-TIME-* tagged in `time_posix.cpp` and
     `time_pico2.cpp`; all 7 SW-TC-TIME-* tagged in `time_test.cpp`.
  2. **POSIX impl uses `CLOCK_MONOTONIC` only** — `CLOCK_REALTIME` /
     `CLOCK_MONOTONIC_RAW` / `CLOCK_BOOTTIME` are forbidden (L2 §9.1);
     reviewer greps the source.
  3. **Pico2 impl uses `time_us_64()` only** — the 32-bit
     `time_us_32()` is forbidden (L2 §9.2).
  4. `juno::time::TimeInit(tTime, tApi, /*pfh=*/nullptr,
     /*pud=*/nullptr)` is the **only** initialization call site
     pattern; no `JUNO_TIME_PROVIDER_T` callback typedef appears.
  5. Time conversion call sites in the test must use the canonical
     non-static member-function form (`tTime.TimestampToMicros(tTs).tOk`,
     `tTime.MicrosToTimestamp(u64Us).tOk`) per
     `ai/memory/lessons-learned-software-systems-engineer.md`
     (2026-05-03 entry).
  6. Each `tApi` is `static const` at file scope; no other file-scope
     mutable data (`SW-REQ-SYS-050`, conventions §5).
  7. **Gate G1, G2, G3** all exit 0.
  8. *(Revision B, 2026-05-05)* Pico2 host-side test target
     `time_pico2_test` builds under `JUNO_FSW_TESTS=ON`, links the
     stub object `pico_time_stub.cpp` (which provides `time_us_64`,
     `from_us_since_boot`, `sleep_until`, `sleep_us` with
     test-controllable state), and runs all `SW-TC-TIME-001..007`
     against `time_pico2.cpp` PASSING under ctest.
  9. *(Revision B)* Stub-state observability per methodology §5.2: the
     stub exposes a current-µs counter, last-`sleep_until`-target,
     last-`sleep_us`-duration, per-function call counts, and a
     `Reset()` helper called in fixture `SetUp()`.
  10. *(Revision B)* `time_pico2.cpp` is **unchanged** by the test
     addition — linker substitution proves the production source is
     unmodified.
- **Test gate**: G1 + G2 + G3, with G1 ctest running BOTH `time_test`
  (POSIX backend) AND `time_pico2_test` (Pico2 backend via stubs).
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11
  agents (initial), plus 3 workers + 3 reviewers + 1 CE re-gate = 7
  agents (Revision B addition) = **18 agents total**.

### SPRINT-IMPL-04 — nmea_lib (Wave 1)

- **Module**: `nmea_lib` — pure-compute NMEA-0183 parser
  (GGA / RMC / GSA / VTG); **single-impl** (no posix/pico2 split per
  L2 §3.3) — one `nmea_impl.cpp` linked by both targets.
- **Predecessors**: none (no time, no bus, no hardware).
- **L2 design**: [`docs/design/nmea/design.md`](../design/nmea/design.md).
- **Requirements**: **12** SW-REQ-NMEA-* IDs (`SW-REQ-NMEA-001` through
  `SW-REQ-NMEA-012`).
- **Files to produce** (5 files; pure-compute single-impl):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/nmea_lib/include/nmea_lib/nmea_api.hpp` | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/nmea_lib/include/nmea_lib/nmea_types.hpp` (POD records `NMEA_GGA_T`, `NMEA_RMC_T`, `NMEA_GSA_T`, `NMEA_VTG_T`, `NMEA_UTC_T`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/nmea_lib/include/nmea_lib/nmea_impl.hpp` + `libs/nmea_lib/src/nmea_impl.cpp` (paired single-impl) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/nmea_lib/tests/nmea_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/nmea_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

  Note: SDP minor amendment 2026-05-05 per SPRINT-IMPL-04 closure (PM Q1
  disposition): path uses plural `tests/` consistent with the kmat / log /
  time precedent, overriding the original L2 §3.3 singular form. The L2
  design §3.3 was amended in the same closure pass; the test_cases.json
  `google_test_ref` strings were also amended from `nmea_lib_test.cpp` to
  the as-built `nmea_test.cpp` filename.

- **Test cases**: **17** Unit-type entries
  (`SW-TC-NMEA-001` through `SW-TC-NMEA-017`).
- **Acceptance criteria** (methodology §8 baseline plus):
  1. All 12 SW-REQ-NMEA-* tagged in `nmea_impl.cpp`; all 17
     SW-TC-NMEA-* tagged in `nmea_test.cpp`.
  2. `kMaxSentenceLen = 128` pinned as `static constexpr` in
     `nmea_api.hpp` (matches L2 §4.1); the 128-byte sentence accumulator
     buffer lives inside the ROOT struct (caller-owned, conventions §5).
  3. Checksum is verified **before** any field decoding
     (`SW-REQ-NMEA-003` / `-004`); test `SW-TC-NMEA-005` proves
     the parsed-output struct is unmodified on checksum mismatch.
  4. Unit conversions at parser boundary: degrees-minutes →
     decimal degrees, knots → m/s, altitude in meters
     (`SW-TC-NMEA-007` / `-008` / `-009` / `-010`).
  5. **Pure compute / single source**: the same `nmea_impl.cpp` is
     compiled into both POSIX and Pico2 targets via a single CMake
     `add_library(nmea_lib STATIC ...)` call (no per-platform source
     switch). `SW-TC-NMEA-017` byte-compares parsed outputs across the
     two targets.
  6. No I/O, no time queries, no allocation, no global state — pure
     functions over inputs (verified by `nm --undefined-only` symbol
     grep in CMake post-build).
  7. **Gate G1, G2, G3** all exit 0.
- **Test gate**: G1 + G2 + G3.
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

### SPRINT-IMPL-05 — device_lib (Wave 2)

- **Module**: `device_lib` — UART1 hardware abstraction; templated on
  RX ring capacity `N` per L2 §4.1; **dual-impl** (POSIX pty / Pico2
  pico-sdk UART1) per L2 §3.
- **Predecessors**: SPRINT-IMPL-03 (`time_lib`) for the `Configure`
  bounded-init timestamping path; SPRINT-IMPL-00 capacity header
  (consumes `juno::device::kDefaultRingCap = 2048`).
- **L2 design**: [`docs/design/device/design.md`](../design/device/design.md).
- **Requirements**: **7** SW-REQ-DEVICE-* IDs (`SW-REQ-DEVICE-001`
  through `SW-REQ-DEVICE-007`).
- **Files to produce** (6 files; dual-impl):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/device_lib/include/device_lib/device_api.hpp` (templated `DEVICE_LIB_API_T<N>` + `DEVICE_LIB_ROOT_T<N>`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/device_lib/include/device_lib/device_impl.hpp` (templated `DEVICE_LIB_IMPL_T<N>` declaration + ring helpers) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/device_lib/src/posix/device_posix.cpp` (pty-backed `iFd` impl) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/device_lib/src/pico2/device_pico2.cpp` (pico-sdk `uart_inst_t* ptUart` impl) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 5 | `libs/device_lib/tests/device_lib_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 6 | `libs/device_lib/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: **8** Unit-type entries
  (`SW-TC-DEVICE-001` through `SW-TC-DEVICE-008`). Several require
  Pico2 hardware loopback (TX-to-RX jumper); the Pico2 portion runs
  on the bring-up bench fixture but is bound to `ctest` via the cross
  harness — the POSIX pty equivalent is the host-side gate.
- **Acceptance criteria** (methodology §8 baseline plus):
  1. All 7 SW-REQ-DEVICE-* tagged in the dual-impl source files; all
     8 SW-TC-DEVICE-* tagged in `device_lib_test.cpp`.
  2. `DEVICE_LIB_ROOT_T<N>` is templated on `N` per L2 §4.1; the FT1
     GPS instantiation pins `N = 2048` matching
     `juno::device::kDefaultRingCap` from the SPRINT-IMPL-00 header
     (cross-sprint pin verified by reviewer).
  3. `static_assert(N >= 256, ...)` guards the template at the ROOT
     declaration (matches L2 §4.1 line cited).
  4. `ReadBytes` returns `JUNO_STATUS_TABLE_FULL_ERROR` on ring
     overflow since previous read (L2 §4.2.3); empty-ring is
     `{SUCCESS, 0}` not an error.
  5. Both impls non-blocking; POSIX uses `O_NONBLOCK` on the pty fd;
     Pico2 uses `uart_is_readable` + `uart_getc` with no
     `uart_read_blocking`.
  6. Bus dependency: **none**. `device_lib` neither subscribes nor
     publishes (consumed directly by `gps_lib` per L2 §3.2).
  7. **Gate G1, G2, G3** all exit 0.
- **Test gate**: G1 + G2 + G3 (G3 also exercises the Pico2 cross-
  compile of the templated `DEVICE_LIB_IMPL_T<2048>` instantiation).
- **Estimated agent count**: 6 workers + 6 reviewers + 1 CE = 13 agents.

### SPRINT-IMPL-06 — sch_lib (Wave 2)

- **Module**: `sch` — FT1 cyclic-executive scheduler platform impls of
  LibJuno's `juno::sch::SCH_API_T<NAppsPerFrame, NFrames>` instantiated
  at `<8, 200>` per L2 §1; **dual-impl** (POSIX
  `clock_nanosleep(TIMER_ABSTIME)` + Pico2 timer-driven
  `busy_wait_until`) per L2 §3.
- **Predecessors**: SPRINT-IMPL-03 (`time_lib`) — the scheduler's
  pacing loop dispatches via the injected
  `juno::time::TIME_ROOT_T &tTime.ptApi->SleepTo`. **This is what
  every Wave 5+ app registers into.**
- **L2 design**: [`docs/design/sch/design.md`](../design/sch/design.md).
- **Requirements**: **10** SW-REQ-SCH-* IDs (`SW-REQ-SCH-001` through
  `SW-REQ-SCH-010`).
- **Files to produce** (5 files; LibJuno owns the canonical
  `SCH_ROOT_T<N,M>` / `SCH_API_T<N,M>` types per L2 §4.1, so no
  FSW-side `_api.hpp` — only the platform impl bodies, the test, and
  CMake):

  | # | File path | Worker | Reviewer |
  |---|-----------|--------|----------|
  | 1 | `libs/sch/src/posix/sch_posix.cpp` (`PosixExecute` body iterating `tArrSchTable[200][8]` + `static const SCH_API_T<8,200> tApi{...}`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 2 | `libs/sch/src/pico2/sch_pico2.cpp` (Pico2 `Execute` body, paired) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 3 | `libs/sch/src/common/sch_common.cpp` (shared `GetMinorFramePeriod` / `GetMajorFramePeriod` returning `tMinorFramePeriod * NFrames`) | senior-software-engineer | senior-software-engineer (reviewer mode) |
  | 4 | `libs/sch/tests/sch_test.cpp` | senior-software-engineer (test author) | senior-software-engineer (reviewer mode) |
  | 5 | `libs/sch/CMakeLists.txt` | junior-software-engineer | senior-software-engineer (reviewer mode) |

- **Test cases**: **12** entries — 11 Unit
  (`SW-TC-SCH-001`..`-008`, `-010`..`-012`) + 1 Integration
  (`SW-TC-SCH-009` — POSIX vs Pico2 invocation-sequence byte-
  comparison).
- **Acceptance criteria** (methodology §8 baseline plus):
  1. All 10 SW-REQ-SCH-* tagged in the impl source files; all 12
     SW-TC-SCH-* tagged in `sch_test.cpp` (11 with
     `verify` + 1 with `verify` for the integration test).
  2. Template instantiation pinned at `<8, 200>` (`NAppsPerFrame = 8`,
     `NFrames = 200`) per L2 §1; the embedded
     `tArrSchTable[200][8]` is caller-owned (`SW-REQ-SCH-010`).
  3. Minor-frame period = 5 ms; major frame = 1000 ms = hyperperiod
     of {5, 10, 50, 100, 200, 500} ms FT1 app rates per L2 §2.
  4. `Execute` invokes `tSch.tTime.ptApi->SleepTo(tSch.tTime,
     tNextMinorFrame)` to pace ticks (canonical LibJuno member-
     function dispatch — **not** `sch_lib::Run`); reviewer greps
     for the wrong call style.
  5. Each app's `OnStart` invoked **once** before its first
     `OnProcess` (`SW-REQ-SCH-007` / `SW-TC-SCH-008`); start-before-
     tick assertion never fires in the test.
  6. App-failure continuation: when one app's `OnProcess` returns a
     non-success status, the scheduler dispatches the remaining apps
     in the same minor frame and proceeds to the next minor frame
     without aborting (`SW-REQ-SCH-006` / `SW-TC-SCH-007`).
  7. POSIX/Pico2 byte-equivalence of dispatch sequence under fake
     time source (`SW-TC-SCH-009` Integration test).
  8. **Gate G1, G2, G3** all exit 0.
- **Test gate**: G1 + G2 + G3.
- **Estimated agent count**: 5 workers + 5 reviewers + 1 CE = 11 agents.

## 4. Wave Exit Gate

After SPRINT-IMPL-06 closes, the Lead spawns a **Wave 1+2 Exit Gate**
`project-chief-engineer` invocation that confirms:

- All 7 sprints (SPRINT-IMPL-00..06) are **CLOSED** with G1 (build) and
  G2 (traceability) passing on every sprint, and G3 (Pico2 cross-
  compile freestanding) passing on every sprint that produces
  compilable code (i.e., 01..06).
- `python3 tools/burndown.py` shows expected requirement closure: 59
  SW-REQ-* IDs moved from Active → Verified across the six L2-bearing
  sprints.
- **No cross-sprint API drift** — concrete checks the CE must run:
  - `time_lib`'s published `tTime.TimestampToMicros(tTs).tOk` form
    matches what `device_lib` and `sch_lib` consume at their `New()`
    call sites (canonical non-static member function — see lessons-
    learned 2026-05-03).
  - `juno::device::kDefaultRingCap` value in
    `apps/include/juno_fsw_capacities.hpp` (SPRINT-IMPL-00) equals the
    `N` template parameter that SPRINT-IMPL-05's `device_lib`
    instantiates for the FT1 GPS path.
  - `juno::sch::SCH_ROOT_T<8, 200>` template parameters in the
    SPRINT-IMPL-06 sources match the system_design.md §3.3 schedule
    (8 apps × 200 minor frames × 5 ms = 1000 ms hyperperiod).
  - `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` from SPRINT-IMPL-01 is
    the only FSW-extension status code introduced across all six
    library sprints (offset `+1` from `JUNO_STATUS_CUSTOM_ERROR`); no
    Wave 1+2 sprint introduces a colliding offset.
- The combined ctest suite for libs touched in this file
  (`kmat_test`, `log_test`, `time_test`, `nmea_test`, `device_lib_test`,
  `sch_test`) passes on POSIX, and the Pico2 cross-compile of all six
  libraries succeeds.

Only after the Wave 1+2 exit gate **PASS** can Wave 3 begin
(`sensor_libs.md`).

## 5. Cross-References

- [SDP Index](index.md) — master sprint table, cross-file conventions,
  carry-forward RFA register.
- [Methodology](methodology.md) — per-sprint structure (§7), gate
  definitions G1/G2/G3 (§5), AC baseline (§8).
- [Wave 3 Sensor Driver Libraries](sensor_libs.md) — successor sprints
  (SPRINT-IMPL-07..11: imu_lib, baro_lib, gps_lib, telem_lib /
  lora_lib, mlog_lib).
- [Wave 4 Domain Libraries](domain_libs.md) — successor sprints
  (SPRINT-IMPL-12..15).
- [Wave 5 Sensor Apps](sensor_apps.md) — Pico2 hardware demo procedures
  (consumes deferred `SW-TC-TIME-008` Pico2 print-stream
  demonstration).
- [Sim and Integration](sim_and_integration.md) — Trick SITL
  composition root that consumes the same `juno::time::TIME_API_T`
  vtable from this file's SPRINT-IMPL-03 by binding a
  `sim_harness`-provided impl (per `time/design.md` §4.4); no FT1
  callback typedef required.
