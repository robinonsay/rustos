# Juno FSW Design Conventions

**Status:** Authoritative.
**Audience:** Every per-module design-doc worker.
**Purpose:** Single shared reference for cross-module names, idioms, and structure. Locking conventions BEFORE the per-module fan-out prevents drift (FT1 lesson: AFM phase enum vs. SYS phase enum disagreement).

If a per-module brief contradicts this file, treat the brief as authoritative ONLY for that module's domain content; for cross-module vocabulary the conventions doc wins. Flag the conflict to the Software Lead.

---

## 1. Module Pattern (LibJuno C++)

Every Juno FSW module's design **must** follow the LibJuno C++ template (`libjuno/templates/template_cpp/`). The legacy C pattern in `libs/*/src/*.c` is being replaced; do not present it as a positive example.

### 1.1 Header layout

```cpp
// MIT License header
#pragma once
#include "juno/module.h"
#include "juno/module.hpp"
#include "juno/status.h"
#include <cstddef>

namespace juno::<module>            // lowercase; e.g. juno::gps, juno::afm
{

struct <MODULE>_ROOT_T;             // forward declaration

struct <MODULE>_API_T               // function-reference vtable
{
    JUNO_STATUS_T (&Foo)(<MODULE>_ROOT_T &tRoot, ...) noexcept;
    RESULT_T<X>   (&Bar)(<MODULE>_ROOT_T &tRoot, ...) noexcept;
    OPTION_T<Y>   (&Baz)(const <MODULE>_ROOT_T &tRoot)  noexcept;
};

struct <MODULE>_ROOT_T JUNO_MODULE_ROOT(<MODULE>_API_T,
    // members shared by every implementation
);

} // namespace juno::<module>
```

Use the **templated** form `<MODULE>_ROOT_T<T, N>` and `JUNO_MODULE_ARG(<MODULE>_API_T<T,N>)` only when the module is generic over an element type or capacity (rare for FSW capabilities; common for utility containers).

### 1.2 Implementation layout

```cpp
namespace juno::<module>
{

struct <MODULE>_IMPL_T JUNO_MODULE_DERIVE(<MODULE>_ROOT_T,
    // platform-specific members (file descriptors, peripheral handles, ...)

    static JUNO_STATUS_T Foo(<MODULE>_ROOT_T &tRoot, ...) noexcept;
    static RESULT_T<X>   Bar(<MODULE>_ROOT_T &tRoot, ...) noexcept;

    static RESULT_T<<MODULE>_IMPL_T> New(
        JUNO_FAILURE_HANDLER_T pfcnFailureHandler,
        JUNO_USER_DATA_T      *pvUserData
    ) noexcept;
);

} // namespace juno::<module>
```

The vtable is wired **once** inside `New()` as a `static` local and **never reassigned**:

```cpp
RESULT_T<MODULE_IMPL_T> MODULE_IMPL_T::New(...) noexcept
{
    static <MODULE>_API_T tApi{ &MODULE_IMPL_T::Foo, &MODULE_IMPL_T::Bar, ... };
    return { JUNO_STATUS_SUCCESS, { &tApi, pfcnFailureHandler, pvUserData } };
}
```

Modules that consume LibJuno's already-published interfaces (e.g., `juno::time::TIME_ROOT_T`, `juno::sch::SCH_ROOT_T<NAppsPerFrame, NFrames>`, `juno::app::APP_ROOT_T`) **do not** redefine the ROOT/API types. Instead, they provide platform-specific function implementations (the `Now`/`SleepTo`/`Sleep` triple for time; the `Execute`/`GetMinorFramePeriod`/`GetMajorFramePeriod` triple for sch; the `OnStart`/`OnProcess`/`OnExit` triple for each app) and aggregate-initialize the LibJuno-published ROOT struct at the composition root. See per-module L2 designs for the published examples.

### 1.3 Mandatory rules

- All public functions are `noexcept`.
- No constructors or destructors on `ROOT_T` / `IMPL_T` — keep them trivially constructible (so `.bss` zero-init is safe).
- Init via explicit `New()` factory; teardown via explicit `Deinit()` if needed.
- No `virtual`, no `dynamic_cast`/`typeid`, no `throw`/`try`/`catch`, no `new`/`delete`/`malloc`.
- No global mutable state inside libraries.

### 1.4 App Lifecycle (canonical from `juno::app::APP_API_T`)

Every Juno FSW app exposes its functionality through a `juno::app::APP_ROOT_T` aggregate (defined in `libjuno/include/juno/app/app_api.hpp`) carrying an `APP_API_T*` vtable with three function references:

| Hook | Signature | Purpose |
|------|-----------|---------|
| `OnStart` | `JUNO_STATUS_T (&)(APP_ROOT_T &tApp) noexcept` | Called once per app before the first `OnProcess` (init resources, subscribe to bus messages). |
| `OnProcess` | `JUNO_STATUS_T (&)(APP_ROOT_T &tApp) noexcept` | Called by the cyclic-executive scheduler at each scheduled tick. |
| `OnExit` | `JUNO_STATUS_T (&)(APP_ROOT_T &tApp) noexcept` | Called on graceful shutdown (POSIX tests only; Pico2 flight never invokes per `SW-REQ-SYS-047`). |

Each per-app L2 design (`docs/design/<name>_app/design.md`) provides:
- A concrete struct (e.g., `IMU_APP_T`) whose first member is `juno::app::APP_ROOT_T tRoot;` (or which embeds the ROOT via aggregate initialization).
- Static functions implementing `OnStart`, `OnProcess`, `OnExit`.
- A composition-root section showing the aggregate initialization that wires the static `APP_API_T` vtable into the ROOT.

Per-app TDM period constants stay as `k<App>AppPeriodMs` (per §4.5). The composition root populates LibJuno's `juno::sch::SCH_ROOT_T<NAppsPerFrame, NFrames>` 2D schedule table by placing each app's `APP_ROOT_T*` in the minor-frame indices that match its period.

---

## 2. C-Pattern Idioms That Are Forbidden in C++ Modules

Designs targeting C++ modules **must not** specify these C idioms. Use the C++ replacement on the right.

| Forbidden (C) | Required replacement (C++) |
|---|---|
| `JUNO_POINTER_T` in public API | Typed `T*` or `RESULT_T<T*>` |
| `JUNO_RESULT_POINTER_T` / `JUNO_OPTION_POINTER_T` | `RESULT_T<T*>` / `OPTION_T<T*>` |
| `JUNO_MEMORY_BLOCK` macro + `BlockGetT` / `BlockPutT` | Typed wrapper `BlockAlloc<T,N>` template |
| `void*` in API surface | Typed function references |
| Manual `sizeof` / `alignof` arguments | Deduced from `template <typename T>` |
| Runtime `if (zLength == 0) return ERR;` | `static_assert(N > 0, "...")` |
| `#define KMAX 32` | `static constexpr size_t kMax = 32;` in namespace |
| `if (status != SUCCESS) return ...` | `JUNO_ASSERT_SUCCESS(...)` macro |
| Null-check `if (!ptr) return ERR;` | `JUNO_ASSERT_EXISTS(ptr);` |
| Unwrap `if (!opt.bIsSome) return DNE;` | `JUNO_ASSERT_SOME(opt, return ...);` |

A single **"Bridging to C"** subsection is permitted in any design that consumes a C-only LibJuno facility (e.g., `JunoMemory_BlockInit`, `juno/sb/broker_api.h`); construct `JUNO_POINTER_T` inline as an implementation detail only.

---

## 3. Naming and Hungarian Conventions

Verbatim from `ai/memory/coding-standards.md` — every per-module design uses these unchanged.

| Element | Convention | Example |
|---|---|---|
| Types / Structs | `SCREAMING_SNAKE_CASE_T` | `GPS_LIB_ROOT_T` |
| Struct tags | `SCREAMING_SNAKE_CASE_TAG` | `GPS_LIB_ROOT_TAG` |
| Public functions | `PascalCase` with module prefix | `GpsLib_GetFix` |
| Static helpers | `PascalCase` (shorter) | `Verify` |
| Macros | `SCREAMING_SNAKE_CASE` | `JUNO_ASSERT_EXISTS` |
| Private members | Leading underscore | `_pfcnFailureHandler` |
| Namespaces | `lowercase` | `juno::gps` |
| `constexpr` constants | `kCamelCase` | `kMaxItems` |

| Variable prefix | Type |
|---|---|
| `t` | struct / typed value |
| `pt` | pointer to typed value |
| `z` | `size_t` |
| `i` | index / integer |
| `b` | `bool` |
| `pv` | `void *` |
| `pc` | `char *` |
| `pfcn` | function pointer / reference |
| `k` | `constexpr` constant |

---

## 4. Shared Cross-Module Vocabulary

Every per-module design **must** use these names verbatim. Do not paraphrase.

### 4.1 Flight phase enum (canonical from `SW-REQ-AFM-002`)

The five values (in order) are:

```cpp
namespace juno::afm
{
enum class JUNO_PHASE_T : uint8_t
{
    JUNO_PHASE_PRE_LAUNCH = 0,
    JUNO_PHASE_BOOST      = 1,
    JUNO_PHASE_APOGEE     = 2,
    JUNO_PHASE_DESCENT    = 3,
    JUNO_PHASE_LANDING    = 4,
};
}
```

Source of truth: `SW-REQ-AFM-002` (`pre-launch, boost, apogee, descent, or landing`). `SW-REQ-AFM-004` and `SW-REQ-AFM-005` constrain transitions to monotonic forward progress through this exact ordering.

Designs **must not** use `COAST` (does not appear in any requirement) and **must not** use `LANDED` (the canonical value is `LANDING`). See FLAGs at the end of this document.

### 4.2 Time base (canonical from `SW-REQ-SYS-026`)

- Single FSW time base: **monotonic `uint64_t` microseconds since startup**.
- Source library: `juno_time` (POSIX impl uses `clock_gettime(CLOCK_MONOTONIC)`; Pico2 impl uses RP2350 timer).
- Type alias used throughout: `using JUNO_TIME_US_T = uint64_t;` declared by `juno_time`.
- FSW message-field timestamps (`tTimestampUs`) are obtained by callers as `juno::time::TIME_ROOT_T::TimestampToMicros(tTime, tTime.ptApi->Now(tTime).tOk).tOk` — i.e., one `Now()` call followed by `TimestampToMicros()`.
- Per-record timestamping is mandated by `SW-REQ-SYS-027` — every sample-level record carries a `JUNO_TIME_US_T`.
- GPS UTC time is a **separate** record kind (`SW-REQ-SYS-028`); it is logged when GPS provides it but never replaces the monotonic time base.
- **Sim-time injection.** Trick SITL provides its own concrete implementation of `juno::time::TIME_API_T { Now, SleepTo, Sleep }` (the canonical LibJuno vtable from `libjuno/include/juno/time/time_api.hpp`). The simulator's `Now` impl returns the current Trick simulation time as a `JUNO_TIMESTAMP_T`; `SleepTo` and `Sleep` may be no-ops or accelerated. The flight POSIX build supplies a different `TIME_API_T` impl that uses `clock_gettime(CLOCK_MONOTONIC, ...)` and `clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, ...)`. The Pico2 build supplies a third impl using the RP2350 hardware timer. There is no `JUNO_TIME_PROVIDER_T` callback parameter — Trick injection is by replacing the entire `TIME_API_T` vtable at composition time.

### 4.3 Status semantics

- All fallible calls return `JUNO_STATUS_T` (signed integer; `0 = JUNO_STATUS_SUCCESS`).
- Value-bearing calls return `RESULT_T<T>` (status + payload). Optional reads return `OPTION_T<T>`.
- Propagation rule: callers **must** use `JUNO_ASSERT_SUCCESS` / `JUNO_ASSERT_OK` / `JUNO_ASSERT_SOME` / `JUNO_ASSERT_EXISTS`. Bare `if`-return is a review failure.
- Failure handlers (`JUNO_FAILURE_HANDLER_T`) are **diagnostic only** — they never alter control flow. Designs must state this explicitly when documenting error paths.
- A health-bitmap update is the canonical observable side effect of a sensor read failure (`SW-REQ-SYS-058`, `SW-REQ-SYS-060`, `SW-REQ-SYS-061`).
- **POST-bitmap attribution for foundation-lib `New()`/init failures.** Each foundation library specifies in its L2 §9 whether `New()`-time failures contribute to the POST result bitmap (`SW-REQ-SYS-029`/`-030`). Sensor and storage libraries (`gps_lib`, `imu_lib`, `baro_lib`, `lora_lib`, `sd_lib`, `device_lib`) DO contribute — the composition root records the failure in the per-sensor POST bit and continues with the sensor marked unhealthy (`SW-REQ-SYS-058`). Pure infrastructure libraries (`log_lib`, `time_lib`, `sch_lib`, `kmat_lib`) do NOT contribute — their `New()` failures are diagnostics-only via the failure handler and the composition root's normal status propagation. Per-module L2 §9 must explicitly state which class the module is in.

### 4.4 Software-bus message naming

- Type names: `JUNO_MSG_<MODULE>_<NAME>_T` (e.g., `JUNO_MSG_GPS_FIX_T`, `JUNO_MSG_NAV_STATE_T`, `JUNO_MSG_AFM_PHASE_T`).
- ID constants: `kJunoMsg<Module><Name>Id` as `static constexpr` in the module's namespace.
- Headers live with the publishing library: `libs/<module>_lib/include/<module>_lib/<module>_msg.hpp`.
- Each message type is a POD aggregate (trivially constructible). No constructors. Carry a `JUNO_TIME_US_T tTimestampUs` as the first field.
- Inter-app data only flows through the LibJuno broker (`libjuno/sb/broker_api.h`) — direct app-to-app coupling is forbidden by `architecture.md`.

### 4.5 TDM scheduler period units

- All period values are in **milliseconds** and are **compile-time constants** (`SW-REQ-SYS-010`).
- Naming: `k<App>AppPeriodMs` (e.g., `kGpsAppPeriodMs = 200`, `kImuAppPeriodMs = 5`, `kNavAppPeriodMs = 10`, `kMlogAppPeriodMs = 5`).
- Declared as `static constexpr uint32_t` in the app's public header.
- Periods derive directly from SYS sample-rate requirements: IMU 200 Hz → 5 ms (`SYS-005`); baro 20 Hz → 50 ms (`SYS-008`); GPS 5 Hz → 200 ms (`SYS-009`); nav 100 Hz → 10 ms (`SYS-012`); telemetry 2 Hz → 500 ms (`SYS-019`).
- **Canonical period table:** `kImuAppPeriodMs = 5`, `kMlogAppPeriodMs = 5`, `kNavAppPeriodMs = 10`, `kAfmAppPeriodMs = 10`, `kBaroAppPeriodMs = 50`, `kSysAppPeriodMs = 100`, `kGpsAppPeriodMs = 200`, `kTelemAppPeriodMs = 500`.
- `kMlogAppPeriodMs = 5` matches the IMU sample rate to satisfy `SW-REQ-SYS-011` (full-rate IMU logging — 200 Hz IMU → 5 ms mlog cadence; no downsampling, no batching delay). `mlog_app` therefore shares the 5 ms minor-frame slot pattern with `imu_app`.

### 4.6 Frame and unit conventions

Canonical from SYS frame requirements; quote these IDs verbatim in any per-module design that touches geometry.

| Quantity | Convention | Source |
|---|---|---|
| Position | WGS-84 geodetic latitude/longitude (deg), altitude (m) | `SW-REQ-SYS-038` |
| Altitude reference | WGS-84 ellipsoid (HAE) | `SW-REQ-SYS-039` |
| Velocity frame | NED (North-East-Down) | `SW-REQ-SYS-040` |
| Attitude | Unit quaternion (w, x, y, z), body→NED | `SW-REQ-SYS-041` |
| Body axes | X-forward, Y-right, Z-down | `SW-REQ-SYS-057` |
| Units | SI throughout | `SW-REQ-SYS-042` |

ECEF is **not** part of the FT1 vocabulary; do not introduce it without a requirement. Designs that need an intermediate frame (e.g., for IMU integration) must declare it in a Definitions table and not on the public API surface.

### 4.7 FSW Lifecycle State Enum (canonical)

The system-level lifecycle state is enumerated by `JUNO_FSW_STATE_T`, declared in the project-wide `conventions` namespace:

```cpp
namespace juno
{
enum class JUNO_FSW_STATE_T : uint8_t
{
    JUNO_FSW_STATE_POST     = 0,  // Power-on self-test in progress.
    JUNO_FSW_STATE_INIT     = 1,  // Composition root running; sensors/comm warming up.
    JUNO_FSW_STATE_RUN      = 2,  // Cyclic-executive scheduler dispatching apps.
    JUNO_FSW_STATE_SAFE     = 3,  // Degraded operation; nav invalid or sensors unhealthy.
    JUNO_FSW_STATE_RECOVERY = 4,  // Post-landing recovery beaconing (SW-REQ-SYS-048).
};
}
```

Source of truth: declared once in a project header (e.g., `include/juno/fsw_state.hpp` or co-located with `sys_app`'s public API). Per-module designs **must not** define a parallel `LIFECYCLE_T` / `STATE_T` enum; consume `juno::JUNO_FSW_STATE_T` verbatim. `sys_app` is the sole publisher of state transitions on the bus (`JUNO_MSG_SYS_STATE_T`).

State semantics:
- `POST`: entered at power-on; exited when `sys_app` finishes the once-only self-test (`SW-REQ-SYS-029`).
- `INIT`: entered after POST; exited when the scheduler begins dispatching (`sch::Run`).
- `RUN`: nominal; the dominant state during ascent and descent.
- `SAFE`: entered when `nav_app.bValid==false` AND a sensor health bit is unhealthy; consumers (e.g., telem) gate use of nav output.
- `RECOVERY`: entered when AFM phase reaches `JUNO_PHASE_LANDING` and persists until power loss (`SW-REQ-SYS-047`, `SW-REQ-SYS-048`).

### 4.8 Status Code Catalog

`juno/status.h` defines 19 canonical status codes (consumed verbatim by every FSW module):

| Code | Value | Meaning |
|------|-------|---------|
| `JUNO_STATUS_SUCCESS` | 0 | Operation completed successfully. |
| `JUNO_STATUS_ERR` | 1 | Unspecified error. |
| `JUNO_STATUS_NULLPTR_ERROR` | 2 | A required pointer argument was NULL or invalid. |
| `JUNO_STATUS_MEMALLOC_ERROR` | 3 | Memory allocation failed (hosted only — never used in libs). |
| `JUNO_STATUS_MEMFREE_ERROR` | 4 | Memory free operation failed. |
| `JUNO_STATUS_INVALID_TYPE_ERROR` | 5 | Provided type or trait did not match. |
| `JUNO_STATUS_INVALID_SIZE_ERROR` | 6 | Provided size or alignment was invalid. |
| `JUNO_STATUS_TABLE_FULL_ERROR` | 7 | A fixed-capacity table/structure was full (use this for ring/queue overflow). |
| `JUNO_STATUS_DNE_ERROR` | 8 | Requested element or key did not exist. |
| `JUNO_STATUS_FILE_ERROR` | 9 | Generic file I/O error on hosted platforms. |
| `JUNO_STATUS_READ_ERROR` | 10 | Read operation failed. |
| `JUNO_STATUS_WRITE_ERROR` | 11 | Write operation failed. |
| `JUNO_STATUS_CRC_ERROR` | 12 | CRC check failed. |
| `JUNO_STATUS_INVALID_REF_ERROR` | 13 | Reference identifier or handle was invalid. |
| `JUNO_STATUS_REF_IN_USE_ERROR` | 14 | Resource cannot be freed while references active. |
| `JUNO_STATUS_INVALID_DATA_ERROR` | 15 | Input data failed validation (use this for bad args/state). |
| `JUNO_STATUS_TIMEOUT_ERROR` | 16 | Operation timed out. |
| `JUNO_STATUS_OOB_ERROR` | 17 | Index or pointer was out of bounds. |
| `JUNO_STATUS_CUSTOM_ERROR` | 1000 | Base offset for FSW-specific custom codes. |

#### FSW-specific extensions (offsets from `JUNO_STATUS_CUSTOM_ERROR`)

When a FSW-specific failure mode does not map cleanly to any LibJuno code, define an offset constant in the consuming module's namespace:

```cpp
namespace juno::kmat
{
    static constexpr JUNO_STATUS_T JUNO_FSW_STATUS_NUMERIC_ERROR =
        JUNO_STATUS_CUSTOM_ERROR + 1;  // singular matrix / underflowed pivot
}
```

Each FSW-specific code MUST cite its `+N` offset and the rationale; offsets `+1..+999` are available.

#### Mapping table for fabricated names found in earlier designs

Any FT1 design that previously used a fabricated name must be swept to the canonical mapping:

| Fabricated name (do NOT use) | Canonical replacement |
|------------------------------|-----------------------|
| `JUNO_STATUS_NULL_POINTER` | `JUNO_STATUS_NULLPTR_ERROR` |
| `JUNO_STATUS_OVERFLOW` | `JUNO_STATUS_TABLE_FULL_ERROR` (capacity exceeded) or `JUNO_STATUS_OOB_ERROR` (index out of range) |
| `JUNO_STATUS_OVERFLOW_ERROR` | Same as above |
| `JUNO_STATUS_IO_ERROR` | `JUNO_STATUS_READ_ERROR` (read path) or `JUNO_STATUS_WRITE_ERROR` (write path) |
| `JUNO_STATUS_INVALID_STATE_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` (typically for bad-state preconditions) |
| `JUNO_STATUS_INVALID_ARG_ERROR` | `JUNO_STATUS_INVALID_DATA_ERROR` |
| `JUNO_STATUS_NUMERIC_ERROR` | `JUNO_FSW_STATUS_NUMERIC_ERROR = JUNO_STATUS_CUSTOM_ERROR + 1` (FSW extension) |

This catalog is authoritative. Per-module L2 designs must use only the symbols above.

---

## 5. Memory Ownership Rules

Every design's "Memory Ownership" section (IEEE 1016 §10) **must** assert:

1. **Caller owns all storage.** Libraries never allocate.
2. **Buffers are passed at `New()` (factory) or `Init()` (rare).** Lifetime extends at least until `Deinit()` returns or the program ends.
3. **No global mutable state in libraries.** Static `tApi` vtable inside `New()` is the only acceptable file-scope data; it is read-only after construction.
4. **No `new`, `delete`, `malloc`, `calloc`, `realloc`, `free`, or heap-backed STL containers** (`SW-REQ-SYS-050`, `constraints.md`).
5. **Fixed-size pools** use the typed `BlockAlloc<T, N>` template wrapper from `template_cpp`; `T tPool[N]` lives inside the wrapper struct, owned by the caller.
6. **Apps own the messages they publish** until the broker copies them. Design must state buffer ownership for every published / subscribed message.

---

## 6. POSIX vs Pico2 Split

`SW-REQ-SYS-043` requires functional equivalence; `SW-REQ-AFM-010` and similar require bit-identical outputs across builds.

- One `<MODULE>_ROOT_T` (header) with two `<MODULE>_IMPL_T` derivations:
  - `libs/<module>_lib/src/posix/<module>_posix.cpp` — POSIX impl (used in unit tests and Trick).
  - `libs/<module>_lib/src/pico2/<module>_pico2.cpp` — Pico2 impl (flight hardware).
- The `IMPL_T` carries platform-specific members (file descriptors, peripheral handles); `ROOT_T` does not.
- The composition root (`apps/<app>/src/main.cpp`) selects the impl by `#if defined(PLATFORM_POSIX)` / `#if defined(PLATFORM_PICO2)` — no runtime selection.
- **Trick** integration uses the POSIX impl: Trick variables drive the simulated sensor inputs, the POSIX impls forward them to the same `ROOT_T` API the flight build uses (`SW-REQ-SYS-045`). Designs document the Trick variable names in §4 (Interface Definitions) when applicable.
- Designs **must** state the equivalence requirement in §11 (Traceability) and call out any deliberate platform divergence (e.g., timing source) explicitly with rationale.

### 6.1 Documented Exceptions

The POSIX/Pico2 source-split rule above admits the following documented exceptions:

- **`kmat_lib` is header-only** (no `src/posix/`, no `src/pico2/`). Rationale: pure templated compute over IEEE-754 floating-point storage; no platform-specific behavior; templated free functions require visible definitions; `-fno-fast-math` makes both POSIX and Pico2 builds produce IEEE-754-aligned results that are bit-identical for normal-range inputs (subnormal-range bit-identicality depends on libm provider — see `kmat_lib` design §11). This is a deliberate deviation from the §6 rule, accepted at PDR (2026-05-02). Any future utility-only module may follow the same pattern; non-utility modules must follow the standard split.

- **`nmea_lib` ships a single platform-agnostic `src/nmea_impl.cpp`** (no `src/posix/`, no `src/pico2/`). Rationale: the library is a pure byte-stream → typed-record transformer; it exposes no hardware handle and no platform call. The shared impl is linked unchanged by both POSIX and Pico2 composition roots and is byte-equivalent across builds by construction (`SW-REQ-NMEA-010`, `SW-REQ-NMEA-011`). The `IMPL_T` pattern is preserved (with no platform-specific members) for cross-module consistency.

- **`sim_harness` is a POSIX-only Trick artifact** (no `src/pico2/`; no Pico2 build participation). Rationale: the simulation harness exists only to drive the FSW POSIX build inside NASA Trick (`SW-REQ-SYS-045`); it is never built for the flight target. Sources live under `sim/sim_harness/src/` (`S_define` + `main.cpp`) and link only against the FSW POSIX library impls. This is a top-level scope exclusion, not a per-library exception. `sim_dynamics`, `sim_sensors`, and `sim_scenario` follow the same scope.

---

## 7. Design-Doc Structure (IEEE 1016, 11 Sections)

Every per-module design uses **these section headings, in this order**. Section numbers are part of the heading text. If a section is empty for the module, write "Not applicable for this module." rather than removing the heading.

1. **Purpose and Scope** — One paragraph stating the capability being designed, its requirement coverage range (e.g., "addresses `SW-REQ-GPS-001` through `SW-REQ-GPS-012`"), and what is explicitly out of scope. No implementation details.
2. **Definitions and Abbreviations** — Module-local terms only. Cross-module vocabulary (phase, NED, time base) is defined here; reference §4 of this conventions doc, do not redefine.
3. **System Overview** — MVC layer mapping for the module: which struct is the App (View), which struct is the Library (Controller), which messages it publishes/subscribes on the bus (Model). One Mermaid diagram showing the module in context with adjacent modules.
4. **Interface Definitions** — Every public function in `<MODULE>_API_T` documented as a contract table (signature, preconditions, postconditions, error conditions, thread safety). Use the format from `ai/skills/software-systems-engineer.md`. Include the Doxygen comment block that will appear in the header.
5. **State Machines** — Mermaid `stateDiagram-v2` for any stateful component (the AFM phase machine is the canonical example). For stateless modules, write "No internal state machine; module is functionally pure given inputs."
6. **Data Flow** — Bus message types published and subscribed, with type names from §4.4 of this conventions doc. Include a small ASCII or Mermaid diagram showing message direction.
7. **Sequence Diagrams** — Mermaid `sequenceDiagram` for at least one nominal cycle (TDM tick → app `Execute()` → lib calls → bus publish). Include at least one error path (e.g., sensor read failure → unhealthy bit set, no publish).
8. **Timing and Scheduling Analysis** — App's TDM period (from §4.5), worst-case execution budget within the slot, and an explicit statement that the app must complete within its slot. List downstream apps that consume this app's bus messages and their periods.
9. **Error Handling Strategy** — How `JUNO_STATUS_T` propagates, what `JUNO_ASSERT_*` macros are used where, what conditions set the failure handler, what conditions set the per-sensor health bit (see `SW-REQ-SYS-031`, `-058`, `-060`, `-061`). State explicitly: "Failure handlers are diagnostic-only and do not alter control flow."
10. **Memory Ownership** — Explicit table of every buffer and its owner / lifetime; reference §5 of this conventions doc. Confirm zero dynamic allocation and zero global mutable state.
11. **Traceability** — Table mapping every requirement ID this design addresses to the section number(s) that cover it. Include POSIX/Pico2 equivalence statement (§6).

Every per-module design follows the file layout `docs/design/<module>/<name>_design.md`. Use an `index.md` + section files only when content exceeds 500 lines (`constraints.md`).

---

## 8. Design Tag Format

Place the tag **on the line immediately above** the heading that addresses the requirement(s). Multiple IDs per tag allowed; multiple tags per heading **not** allowed (consolidate into one).

```markdown
<!-- @{"design": ["SW-REQ-GPS-001", "SW-REQ-GPS-002"]} -->
### 4.1 GpsLib_GetFix Contract
```

Rules:
- Every requirement ID inside a `design` tag **must exist** in `docs/requirements/<module>/requirements.json` (verified by `tools/traceability.py`).
- Every section that claims to "address" or "implement" a requirement **must** carry a tag — otherwise the link is invisible to RTM tooling (lesson learned: 2026-05-02, single `parent_id`).
- Tags are HTML comments; they render to nothing in viewers but are parsed by tooling.
- A single requirement ID may appear in multiple tags (e.g., once in §4 for the interface, once in §9 for the error path) — RTM tolerates this.
- Section 11 (Traceability) consolidates the per-section tags into one table — that table is **descriptive**, not authoritative; the per-section tags are authoritative.

---

## 9. Common Review Traps — Self-Check Before Submission

Run this checklist before reporting "Work Complete" to the Software Lead. A NO on any item is a review failure.

- [ ] Every requirement ID inside any `<!-- @{"design": [...]} -->` tag exists in `docs/requirements/<module>/requirements.json`.
- [ ] Every section that claims to address a requirement carries a `design` tag with that ID.
- [ ] Memory Ownership section asserts caller-owned, no `new`/`delete`/`malloc`, no global mutable state (matches §5).
- [ ] All sequence diagrams use module names exactly as defined in §4 of this conventions doc and the architecture doc (`gps_app`, `nav_lib`, etc., snake_case for app/lib references; `juno::<module>` for namespaces).
- [ ] Phase enum (if used) is `{PRE_LAUNCH, BOOST, APOGEE, DESCENT, LANDING}` — never `COAST`, never `LANDED`.
- [ ] Frames match §4.6 (NED for velocity, body X-fwd/Y-right/Z-down, geodetic for position) — no ECEF unless a future requirement adds it.
- [ ] Every `<MODULE>_API_T` function-reference is `noexcept`.
- [ ] No constructors / destructors on `<MODULE>_ROOT_T` or `<MODULE>_IMPL_T`.
- [ ] `New()` factory is the single init entry point; vtable wired once via `static` local.
- [ ] POSIX vs Pico2 §6 statement present; equivalence requirement (`SW-REQ-SYS-043`) called out in §11.
- [ ] File is ≤500 lines or split into `index.md` + section files (`constraints.md`).
- [ ] No "X and Y" compound behavior described as a single function contract — split into two contracts (lesson learned: 2026-05-02, atomicity).
- [ ] No fabricated rationale; every design choice cites either a `SW-REQ-*` ID or this conventions doc.

---

## FLAGs Raised

**FLAG-1: Phase enum proposed in this worker brief contradicts canonical requirements.**
The brief proposed `{JUNO_PHASE_PRELAUNCH, JUNO_PHASE_BOOST, JUNO_PHASE_COAST, JUNO_PHASE_APOGEE, JUNO_PHASE_DESCENT, JUNO_PHASE_LANDED}`. The canonical set per `SW-REQ-AFM-002` is `{pre-launch, boost, apogee, descent, landing}` — there is **no `coast`** phase, and the final state is **`landing`** not `landed`. This conventions doc adopts the requirements-grounded set and renders the brief's proposal non-authoritative. Software Lead should confirm with PM that no `coast` phase is intended for FT1 and no `landed` post-landing terminal state is needed beyond `landing` (recovery beaconing per `SW-REQ-SYS-048` continues during the `landing` phase).

**FLAG-2: SYS-016 lists four phases; AFM-002 lists five.**
`SW-REQ-SYS-016` describes "boost, apogee, descent, and landing" without `pre-launch`; `SW-REQ-AFM-002` adds `pre-launch` as the initial state. This conventions doc treats AFM-002 as the authoritative enum (it is the closer-to-implementation requirement and explicitly enumerates the phase set). Software Lead may want PM confirmation that SYS-016 should be amended to mention `pre-launch` as the at-power-on initial state, or that SYS-016's omission is intentional (pre-launch is not a "detected" phase, only the initial state).

**FLAG-3: Body axes spec not in original brief AC-6.**
The brief's AC-6 listed body / NED / ECEF as the example frame set. ECEF is **not** referenced by any FT1 requirement; body axes are pinned by `SW-REQ-SYS-057` (X-fwd, Y-right, Z-down). This conventions doc uses the requirements-grounded set (geodetic + HAE + NED + body X-fwd/Y-right/Z-down) and does **not** introduce ECEF. No PM action needed unless future missions add ECEF.

**FLAG-4: IMU model is "TBD" in `project-overview.md`.** **CLOSED 2026-05-03 — IMU locked to MPU-6050.**
Resolution per Chair action S1-AI-022: the FT1 IMU is **MPU-6050** (InvenSense). `project-overview.md` updated. Downstream `imu_lib` design §4 (Interface Definitions) is now unblocked: register map, accelerometer ±16 g range (`SW-REQ-SYS-006`), gyroscope ±2000 dps (`SW-REQ-SYS-007`), and 200 Hz sample rate (`SW-REQ-SYS-005`) are addressed by the MPU-6050 part. I²C bus selection and FIFO mode remain design-time decisions for `imu_lib`.
