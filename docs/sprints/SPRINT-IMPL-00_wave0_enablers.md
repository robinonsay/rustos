# SPRINT-IMPL-00 — Wave 0 Enablers (Bus Variant + Capacity Pins)

| Field | Value |
|-------|-------|
| Sprint ID | SPRINT-IMPL-00 |
| Module | Project-wide FSW headers (Wave 0 enabler) |
| Wave | 0 |
| Start date | 2026-05-04 |
| End date | 2026-05-04 |
| Status | **CLOSED** |
| Predecessors | none (PDR-closed baseline + LibJuno) |
| Successor eligible | SPRINT-IMPL-01 (kmat_lib, Wave 1) |
| PM approval | 2026-05-04 (sprint plan + SDP sign-off in same message) |

## Sprint Goal

Author the two project-wide enabler headers that every Wave 5+ application
sprint depends on, resolving SDP risk-register items SDP-R-02
(`JUNO_MSG_BUS_VARIANT_T` publication) and SDP-R-03 (capacity placeholder
pins).

## Worker Invocations

| # | File | Worker | Iterations | Final Status |
|---|------|--------|------------|--------------|
| 1 | `apps/include/juno_msg_bus_variant.hpp` (188 lines) | senior-software-engineer | 1 | APPROVED |
| 2 | `apps/include/juno_fsw_capacities.hpp` (144 lines) | senior-software-engineer | 1 | APPROVED |

## Reviewer Verdicts

| # | File Reviewed | Reviewer | Iteration | Verdict |
|---|--------------|----------|-----------|---------|
| 1 | `apps/include/juno_msg_bus_variant.hpp` | senior-software-engineer (reviewer mode) | 1 | APPROVED — 0 errors, 0 warnings; smoke compile passed both POSIX and Pico2; `BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T, 8, 64>` instantiation verified |
| 2 | `apps/include/juno_fsw_capacities.hpp` | senior-software-engineer (reviewer mode) | 1 | APPROVED — 0 errors, 0 warnings; 6 static_asserts; smoke compile passed both targets |

## Phase 3 Gate Output

```
=== POSIX smoke compile ===
g++ -std=c++11 -Wall -Wextra -Werror -pedantic -Wshadow -Wcast-align -Wundef
    -Wswitch -Wswitch-default -fno-rtti -fno-exceptions -fno-common
    -fno-strict-aliasing -I libjuno/include -I . /tmp/sprint00_smoke.cpp
POSIX exit: 0

=== Pico2 cross-compile (freestanding ARM Cortex-M33) ===
arm-none-eabi-g++ -std=c++11 -Wall -Wextra -Werror -pedantic -Wshadow
    -Wcast-align -Wundef -Wswitch -Wswitch-default -fno-rtti -fno-exceptions
    -fno-common -fno-strict-aliasing -ffreestanding -mcpu=cortex-m33 -mthumb
    -I libjuno/include -I . -c /tmp/sprint00_smoke.cpp
Pico2 exit: 0

=== Phase 3 Gate G2 — traceability.py ===
TRACEABILITY CHECK PASSED
  Valid requirement IDs:        376
  Requirements with code:       0
  Requirements with @verify:    0
  Requirements with test specs: 375
G2 exit: 0
```

The smoke `.cpp` exercises the canonical Wave-5+ pattern:
```cpp
juno::sb::BROKER_ROOT_T<JUNO_MSG_BUS_VARIANT_T,
                        juno::broker::kBrokerPipes,
                        juno::broker::kBrokerRegistry> tBroker;
```
proving cross-header integration of the variant + capacity pins under both
POSIX and Pico2 toolchains.

## Acceptance Criteria — Final Status

| AC | Description | Status | Evidence |
|----|-------------|--------|----------|
| AC-1 | `JUNO_MSG_BUS_VARIANT_T` covers all 12 bus message MIDs (11 from `system_design.md` §4 + `JUNO_MSG_SYS_STATE_T` from `conventions.md` §4.7) | MET | `apps/include/juno_msg_bus_variant.hpp:78-93` (enum) + `:148` (struct) |
| AC-2 | Four capacity pins with correct values + `static_assert` envelopes | MET | `juno_fsw_capacities.hpp:78,87,110,134` (pins) + `:89,92,112,115,136,139` (static_asserts) |
| AC-3 | `#pragma once`, freestanding-compatible, only LibJuno + `<cstdint>`/`<cstddef>` | MET | `juno_fsw_capacities.hpp:26,28`; `juno_msg_bus_variant.hpp:53-60` |
| AC-4 | Smoke-compile clean both targets; no `juno::sb::*` collision | MET | Phase 3 gate output above; broker template instantiation succeeds |
| AC-5 | Gate G1 (POSIX + Pico2) exit 0 | MET | Phase 3 gate output above |
| AC-6 | Gate G2 (`traceability.py`) exit 0; counter delta = 0 | MET | 376 / 0 / 0 / 375 unchanged from pre-sprint baseline |

## Risk Resolution

- **SDP-R-02** (`JUNO_MSG_BUS_VARIANT_T` publication): RESOLVED.
- **SDP-R-03** (capacity placeholder pins): RESOLVED.

Both retired from the live risk register.

## Chief Engineer Verdict

**PASS** — issued by `project-chief-engineer` 2026-05-04.

Rationale (excerpted): "Wave 0 enabler delivered. SDP-R-02 and SDP-R-03 are
retired from the live risk register. SPRINT-IMPL-01 (kmat_lib, Wave 1) is
now eligible to launch."

## Lead-Direct Edits Applied This Sprint

1. **SDP sign-off:** `docs/sdp/index.md:269` updated PM approval line from
   "_Pending PM signature_" to "**APPROVED 2026-05-04** — PM signed off SDP
   at SPRINT-IMPL-00 kickoff" with signature line populated.
2. **Minor SDP amendment** (per CE recommendation): `docs/sdp/foundation_libs.md`
   §3 SPRINT-IMPL-00 AC-1 amended to enumerate all 12 MID names (was 11
   with `JUNO_MSG_SYS_STATE_T` substituted for `JUNO_MSG_TELEM_PACKET_T`).
   Both messages are real; the amendment matches as-built. Per
   `methodology.md` §11 this qualifies as a Lead-direct minor amendment.
3. Created `apps/include/` directory (did not exist before this sprint).
4. Created `docs/sprints/` directory (did not exist before this sprint;
   PM redirected sprint records here from `ai/sprints/` at closure).

## Notable Worker Deviations (Approved)

- **`JUNO_TIME_MICROS_T` substituted for `JUNO_TIME_US_T`** in
  `apps/include/juno_msg_bus_variant.hpp` because `JUNO_TIME_US_T` is not
  yet published in LibJuno (it will land in Wave 1 `time_lib`).
  `JUNO_TIME_MICROS_T` is the actually-published canonical type at
  `libjuno/include/juno/time/time_api.h:77`. Worker documented the
  substitution at `:135-138` referencing `conventions.md §4.2`.
- **`uint8_t tArrPayload[256]` byte buffer** chosen over forward-declared
  union (which would be illegal C++ — `sizeof` of forward-declared
  incomplete types is undefined). The 256-byte fixed payload gives the
  variant a concrete `sizeof` (= 272 B) usable across all 26 sprints.

## Lessons Learned

Captured in `ai/memory/lessons-learned-software-lead.md` and
`ai/memory/lessons-learned-senior-software-engineer.md` (entries dated
2026-05-04).

## Files Touched

- **Created:** `apps/include/juno_msg_bus_variant.hpp`
- **Created:** `apps/include/juno_fsw_capacities.hpp`
- **Created:** `docs/sprints/SPRINT-IMPL-00_wave0_enablers.md` (this file)
- **Edited (minor amendment):** `docs/sdp/foundation_libs.md` §3 AC-1
- **Edited (PM sign-off):** `docs/sdp/index.md` §13 approval table

## Agent Count

5 (matches SDP estimate at `foundation_libs.md:96`):
- 2 senior-software-engineer workers (Phase 1)
- 2 senior-software-engineer reviewers (Phase 2, 1 iteration each)
- 1 project-chief-engineer (Phase 4 final gate)
