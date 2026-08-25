# Juno FT1 PDR — Section S2: Foundation Libraries

## 1. Header

| Field | Value |
|-------|-------|
| Section Number | S2 |
| Section Title | Foundation Libraries (`time_lib`, `log_lib`, `sch_lib`, `device_lib`, `kmat_lib`) |
| Date Convened | 2026-05-02 |
| Chair | Project Manager |
| Software Lead (Presenter) | Software Lead (orchestrator) |
| Attendees | Chair, MAE, SSE-R, CE, Software Lead (non-voting) |

### Documents Under Review

- [docs/design/time/design.md](../../../design/time/design.md) (437 lines, 7 reqs)
- [docs/design/log/design.md](../../../design/log/design.md) (379 lines, 8 reqs)
- [docs/design/sch/design.md](../../../design/sch/design.md) (453 lines, 10 reqs)
- [docs/design/device/design.md](../../../design/device/design.md) (450 lines, 7 reqs)
- [docs/design/kmat/index.md](../../../design/kmat/index.md), [04_interface.md](../../../design/kmat/04_interface.md), [05_through_11.md](../../../design/kmat/05_through_11.md) (split per conventions §7; 15 reqs)

**Coverage scope:** 47 requirements (TIME-001..007, LOG-001..008, SCH-001..010, DEVICE-001..007, KMAT-001..015).

## 2. Section Summary

(Software Lead's presentation summary as briefed in chat — 14 module-level decisions across 5 libs; 9 Lead-pre-flagged concerns L-G1..L-G9. See chat transcript for the full presentation. Short form below.)

**Decisions of note**

- `time_lib` declares the canonical `JUNO_TIME_US_T = uint64_t` and accepts the `JUNO_TIME_PROVIDER_T` callback (Trick injection seam, S1-AI-008).
- `log_lib` is the diagnostic logger (NOT mission log); 4 severity levels (no TRACE); 256-byte `vsnprintf` stack buffer.
- `sch_lib` is templated on capacity `<N>`; 5 ms base tick; cooperative TDM; expects polymorphic `juno::app::APP_ROOT_T`.
- `device_lib` is templated on RX ring capacity `<N>` (FT1 GPS uses `<2048>`); non-blocking `ReadBytes`; ring overflow returns `JUNO_STATUS_OVERFLOW` (DEVICE-004 amended).
- `kmat_lib` is **header-only** (deviation from conventions §6, justified per §1.1 templated-form clause); `MAT_T<T,R,C>`, `VEC_T`, `QUAT_T` (Hamilton, w/x/y/z); LU `Invert`.

## 3. RID List

ID format: `PDR-RID-S2-NNN`. Numbered sequentially across all sources.

### 3.1 MAE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S2-001 | **Major** | `juno::app::APP_ROOT_T` forward-declared but never defined | sch §4.1 forward-declares `juno::app::APP_ROOT_T`; no S2 doc (and not S1 system_design.md) defines it. system_design §8.1 pseudocode passes `tImuApp.tRoot` (i.e., `IMU_APP_ROOT_T`). Without a defined base, every per-app L2 (S6/S7) cannot compile against `Register()`. sch FLAG-2 admits the gap but defers to per-app L2s, which inverts the dependency. **(Co-raised by SSE-R-007 and CE-018.)** | Define `juno::app::APP_ROOT_T` with `APP_API_T` vtable carrying `Execute()`/`Start()` references, with the contract that every `<APP>_ROOT_T` derives via `JUNO_MODULE_DERIVE`. Or raise system-level decision to insert `app_lib` base into conventions §4 before S6 convenes. The current state cannot exit S2. | OPEN | — | — | OPEN |
| PDR-RID-S2-002 | Minor | Status-code naming inconsistency: `JUNO_STATUS_NULL_POINTER` (device) vs `JUNO_STATUS_NULLPTR_ERROR` (log) | device §4.2.2/§4.2.3/§7.3/§9.2 use `JUNO_STATUS_NULL_POINTER`. log §4.4/§4.5/§9.2 use `JUNO_STATUS_NULLPTR_ERROR`. Same drift for `JUNO_STATUS_OVERFLOW` (device) vs `JUNO_STATUS_OVERFLOW_ERROR` (sch). conventions §4.3 doesn't lock the spelling. **(Lighter framing of SSE-R-009 / CE-019.)** | Pick one form (recommend aligning to existing `juno/status.h`). Update conventions §4.3 to enumerate canonical status names; reconcile all S2 designs in one batched pass. | OPEN | — | — | OPEN |
| PDR-RID-S2-003 | Minor | sch L2 uses `SCH_LIB_IMPL_T<N>` but L1 system_design §8.1 uses `SCH_IMPL_T` (no `_LIB_`, no `<N>`) | system_design §8.1 line 344 writes `juno::sch::SCH_IMPL_T tSch = juno::sch::SCH_IMPL_T::New(...).tOk;` and `Register(tSch.tRoot, ...)` — neither the `_LIB_` infix nor the `<N>` template parameter appears. sch §3/§4 mandates both. The composition root pseudocode is non-buildable against the L2 design. **(Co-raised by CE-020.)** | Amend system_design §8.1 pseudocode to `SCH_LIB_IMPL_T<8>` (matching FT1's 8 apps) and the matching template instantiation; ensure §8.1 uses `_LIB_` infixed names throughout. Record in master log Cross-Section Re-Open Log against S1. | OPEN | — | — | OPEN |
| PDR-RID-S2-004 | Minor | log_lib LOG-007 rationale prose says "stdout" while design and brief AC-5 mandate "stderr" | log requirements.json LOG-007 rationale: "POSIX sink is stdout"; log design §3.1/§6/§8/§10 specify `stderr`. Design FLAG-1 acknowledges the discrepancy. As long as the requirement-rationale text and the design-prose disagree, downstream verification cannot determine which is authoritative. | PM action: update LOG-007 rationale prose to "stderr" (no description change required). The design selection (stderr) is sound. | OPEN | — | — | OPEN |
| PDR-RID-S2-005 | Editorial | sch §11 traceability-table title for SW-REQ-SCH-004 disagrees with requirements.json title | sch design §11 line 438 titles SCH-004 as "Period Range 5–500 ms"; requirements.json title is "Period Range Covers FSW Application Rates". §11 is descriptive, but title mismatch hampers RTM cross-reference. | Replace "Period Range 5–500 ms" with "Period Range Covers FSW Application Rates" in sch §11 row. | OPEN | — | — | OPEN |
| PDR-RID-S2-006 | Editorial | device_lib `template<const size_t N>` uses redundant `const`; sch_lib uses `template<size_t N>` | device §4.1/§4.3 declare `template<const size_t N>` (the `const` qualifier on a non-type template parameter is redundant). sch_lib uses `template<size_t N>` consistently. Inconsistent style. | Standardize on `template<size_t N>` (drop redundant `const`) in device §4.1/§4.3 to match sch_lib and LibJuno reference. | OPEN | — | — | OPEN |

### 3.2 SSE-R Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S2-007 | **Major** | `APP_ROOT_T` undefined — `sch_lib::Register` type contract unverifiable | Same root as RID-S2-001; SSE-R framing focuses on type-safety verification under no-virtual / no-RTTI constraint. Without `APP_ROOT_T`'s definition, `Register(APP_ROOT_T &tApp)` accepting heterogeneous app root types is unverifiable at PDR. | Define `APP_ROOT_T` (with `APP_API_T` vtable carrying `Execute`/`Start` references) in a shared header; reference from sch §4.1 and each per-app L2 design; show one example concrete derivation. | OPEN | — | — | OPEN |
| PDR-RID-S2-008 | **Major** | `JUNO_MODULE_ROOT(SCH_LIB_API_T<N>, ...)` missing `JUNO_MODULE_ARG` — preprocessor defect | sch §4.1 header sketch writes `struct SCH_LIB_ROOT_T JUNO_MODULE_ROOT(SCH_LIB_API_T<N>, ...)`. `JUNO_MODULE_ROOT` is a variadic preprocessor macro; the comma in `SCH_LIB_API_T<N>` is interpreted as an argument separator, splitting into `API_T = SCH_LIB_API_T<N` and a separate `>`. Preprocessor error. The LibJuno template `temp_api.hpp` uses `JUNO_MODULE_ARG(API_T<T,N>)` for this case; sch must too. device_lib §4.1 already does this correctly. | Replace `JUNO_MODULE_ROOT(SCH_LIB_API_T<N>, ...)` with `JUNO_MODULE_ROOT(JUNO_MODULE_ARG(SCH_LIB_API_T<N>), ...)` throughout sch_lib. Apply same fix to `JUNO_MODULE_DERIVE(SCH_LIB_ROOT_T<N>, ...)` (§4.5) if applicable. | OPEN | — | — | OPEN |
| PDR-RID-S2-009 | **Major** | Fabricated status codes — seven `JUNO_STATUS_*` symbols not in `juno/status.h` | The following appear in S2 designs but do **not** exist in `libjuno/include/juno/status.h`: `JUNO_STATUS_NULL_POINTER` (device), `JUNO_STATUS_OVERFLOW` (device), `JUNO_STATUS_IO_ERROR` (log), `JUNO_STATUS_OVERFLOW_ERROR` (sch), `JUNO_STATUS_INVALID_STATE_ERROR` (sch), `JUNO_STATUS_INVALID_ARG_ERROR` (sch), `JUNO_STATUS_NUMERIC_ERROR` (kmat, time indirect). The actual `juno/status.h` enum has 19 codes (`SUCCESS, ERR, NULLPTR_ERROR, MEMALLOC_ERROR, MEMFREE_ERROR, INVALID_TYPE_ERROR, INVALID_SIZE_ERROR, TABLE_FULL_ERROR, DNE_ERROR, FILE_ERROR, READ_ERROR, WRITE_ERROR, CRC_ERROR, INVALID_REF_ERROR, REF_IN_USE_ERROR, INVALID_DATA_ERROR, TIMEOUT_ERROR, OOB_ERROR, CUSTOM_ERROR`). Compiling will fail. **(Co-raised by CE-019; partial overlap with MAE-002.)** | Either (a) add required codes to `juno/status.h` (`OVERFLOW_ERROR`, `INVALID_STATE_ERROR`, `INVALID_ARG_ERROR`, `NUMERIC_ERROR`, `IO_ERROR`; alias `NULL_POINTER` ↔ `NULLPTR_ERROR`) and document in conventions §4.3, or (b) amend every S2 design to use only existing 18 symbols. Recommendation: (a) — add a "Status Code Catalog" subsection to conventions §4 enumerating every legal symbol and semantics; sweep all 27 L2 designs. | OPEN | — | — | OPEN |
| PDR-RID-S2-010 | **Major** | `std::sqrt` / `<cmath>` claimed freestanding-permitted without mechanism — kmat §4.6 | kmat 04_interface.md §4.6 `VecNorm2` table entry states `std::sqrt` is "freestanding-permitted". `<cmath>` is in the **hosted** subset of the C++ standard library, not the freestanding required headers. Under `-nostdlib -ffreestanding` on bare-metal RP2350, `std::sqrt` will fail to link without an explicit math library. Claim is unsubstantiated. | Either (a) add a `juno::math::Sqrt<T>` wrapper using `__builtin_sqrt`/`__builtin_sqrtf` (GCC/Clang builtins, freestanding-available), or (b) document the link-time dependency on platform libm (pico-sdk newlib-nano on Pico2; glibc on POSIX) explicitly in kmat §3 / §11. Option (a) preferred. | OPEN | — | — | OPEN |
| PDR-RID-S2-011 | Minor | `GetUs()` upcast from `TIME_LIB_ROOT_T` to `TIME_LIB_IMPL_T` undocumented | time §4.4 has `GetUs()` dispatch `pfcnTimeProvider(tRoot.pvUserData)` where `pfcnTimeProvider` lives on IMPL, not ROOT. Vtable receives `TIME_LIB_ROOT_T &tRoot`. Impl must upcast via standard-layout guarantee (ROOT is first member of IMPL). Pattern is well-defined but not stated explicitly. | Add to time §4.4: "Inside `TIME_LIB_IMPL_T::GetUs(TIME_LIB_ROOT_T &tRoot)`, the impl upcasts via `static_cast<TIME_LIB_IMPL_T&>(tRoot)`, valid because `JUNO_MODULE_DERIVE` guarantees `TIME_LIB_ROOT_T` is the first member of `TIME_LIB_IMPL_T` and both are standard-layout types per C++11 §9.2." | OPEN | — | — | OPEN |
| PDR-RID-S2-012 | Minor | Variadic function reference `(&LogFmt)(...) noexcept` in vtable — well-formedness not established | log §4.3 vtable declares `JUNO_STATUS_T (&LogFmt)(LOG_LIB_ROOT_T &, JUNO_LOG_LEVEL_T, const char*, const char*, ...) noexcept`. C++ function-reference vtable to variadic function is permitted syntactically, but design's claim that "implementation uses `<cstdarg>` va_list internally" is incorrect — the caller pushes args per C calling convention, not via va_list. Compiling under `-std=c++11 -pedantic -Werror` on GCC arm-none-eabi and x86_64 needs confirmation. | Either (a) remove `LogFmt` from vtable; replace with non-variadic taking pre-formatted `const char *pcMessage` (callers `vsnprintf` themselves before calling `Log`), or (b) keep variadic vtable entry but add explicit note citing C++ function references to variadic-signature compile under `-std=c++11` on GCC/Clang for these target ABIs, with `static_assert(sizeof(&LOG_LIB_IMPL_T::LogFmt) == sizeof(void*))`. Option (a) simpler. | OPEN | — | — | OPEN |
| PDR-RID-S2-013 | Minor | `vsnprintf` availability under `-nostdlib -ffreestanding` unaddressed — log §9 | log §9 rule 6: "`vsnprintf` is the only stdlib formatting routine used". `vsnprintf` is in `<cstdio>` (hosted). coding-standards §3 / constraints.md require lib code to compile with `-nostdlib -ffreestanding`. Design doesn't state how `vsnprintf` is available in freestanding. The library's source files (`log_*.cpp`) may not be required to be freestanding — only headers must be — but design doesn't make this distinction. | Add to log §9 (or §1): "`log_lib` source files are not compiled with `-ffreestanding`; they link against the platform C runtime that provides `vsnprintf`. Only the public header `log_api.hpp` must be freestanding-compatible (no `<cstdio>`)." If `log_api.hpp` does include `<cstdio>`, that is a separate defect. | OPEN | — | — | OPEN |
| PDR-RID-S2-014 | Minor | Ring overflow "drop oldest" policy creates undocumented data hazard for nmea — device §9 | device §9 rule 5: "evicts the oldest bytes" on ring overflow. Consequence: evicted bytes may include partial NMEA sentence, retained newest may start mid-sentence. Design notes "NMEA parsers naturally resync on `$`" — but resync behavior is not a requirement on `nmea_lib`/`gps_lib` in current S2. Undocumented dependency on `nmea_lib`'s contract. | Add to device §4.2.3 (ReadBytes postconditions): "When `JUNO_STATUS_OVERFLOW` is returned, the byte stream in `pcuBuf` may begin at an arbitrary offset within an NMEA sentence; the caller (gps_lib/nmea_lib) must treat the stream as starting at unknown framing and resync on the next `$` delimiter." Tag as cross-module constraint on nmea_lib. | OPEN | — | — | OPEN |
| PDR-RID-S2-015 | Minor | kmat `Invert` LU pivot tiebreak not specified — determinism gap for SW-REQ-KMAT-009 | kmat 04_interface.md §4.2.6 / 05_through_11.md §8 state "LU with partial pivoting" and "deterministic given input bytes." Partial pivoting selects max-magnitude row, but tiebreak when two pivots have exactly equal floating-point magnitude is unspecified. Without a deterministic tiebreak, different compilers/FPUs may select different rows — non-identical factorizations. KMAT-009 requires deterministic results. | Add to kmat §4.2.6 / §9: "Tiebreak rule: when two rows have equal maximum absolute pivot magnitude, select the lower row index (first encountered in top-to-bottom scan)." Total order on input bytes. | OPEN | — | — | OPEN |

### 3.3 CE Findings

| ID | Severity | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|----------|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RID-S2-016 | **Major** | S1-AI-005 corrective action (mlog @ 5 ms) not absorbed into S2 designs | system_design §3.3/§8.2/§11 reflect S1-AI-005 disposition (`kMlogAppPeriodMs = 5`, 637 invocations). time/design.md §8 table line 366 still states `mlog_app | 10 | kMlogAppPeriodMs`. sch/design.md §7.1 line 325 sequence diagram comment: `mlog_app: Execute() [period 10 ms]`. Both contradict the disposed L1 baseline. The Lead's pre-S2 corrective edits did not cascade. | Update time §8 table and sch §7.1 sequence diagram to reflect `kMlogAppPeriodMs = 5`. | OPEN | — | — | OPEN |
| PDR-RID-S2-017 | **Major** | `device_lib<N>` strictly templated but `sys_app` (S8) consumes non-templated form | sys_app §4 line 132 declares `juno::device::DEVICE_LIB_ROOT_T* _ptDev;`; line 167 takes `juno::device::DEVICE_LIB_ROOT_T &tDev`. Neither type exists; `device_api.hpp` only declares `template<size_t N> struct DEVICE_LIB_ROOT_T`. sys_app cannot hold a single non-templated reference because each `<N>` instantiation is unrelated. sys_app POST-probes UART1 device for every sensor that uses it (GPS @ 2048, lora @ smaller); needs strategy for two separate `device_lib` instances at two different `N`. | Either (a) sys_app holds two separate refs `DEVICE_LIB_ROOT_T<2048> &tGpsDev, DEVICE_LIB_ROOT_T<kLoraRx> &tLoraDev`, or (b) extract a non-templated POST-only base interface from device_lib. Cross-S2/S8 decision; carry RID into S8. | OPEN | — | — | OPEN |
| PDR-RID-S2-018 | **Major** | `juno::app::APP_ROOT_T` is fictional — re-raise of MAE-001/SSE-R-007 with cross-section evidence | sch/design.md §4.1 forward-declares; spot-check of imu_app and sys_app L2 designs confirms **no L2 design defines or derives from `juno::app::APP_ROOT_T`**. All apps expose disjoint `<APP>_APP_ROOT_T` aggregates with no upcast path. Breaks entire S1 §8.1 composition root. **(Same root as RID-S2-001 / RID-S2-007.)** | PM-level decision: pick (a) define `juno::app::APP_ROOT_T` in conventions as a thin POD with `Execute`/`Start` function references that every `<APP>_APP_ROOT_T` aliases as first member; or (b) make `sch_lib::Register` take a typed callback triple `(JUNO_STATUS_T(*)(void*), JUNO_STATUS_T(*)(void*), void*, uint32_t)`. Cross-section (S2 + every app section). | OPEN | — | — | OPEN |
| PDR-RID-S2-019 | **Major** | Status-code symbols don't exist — re-raise of SSE-R-009 with project-level framing | Same finding as RID-S2-009. CE adds project-level recommendation: add a "Status Code Catalog" section to conventions §4 enumerating the legal symbols and semantics, then sweep all 27 L2 designs. **(Same root as RID-S2-009.)** | See RID-S2-009 resolution. | OPEN | — | — | OPEN |
| PDR-RID-S2-020 | Minor | `system_design` §8.1 pseudocode `juno::sch::SCH_IMPL_T` vs L2 `SCH_LIB_IMPL_T<N>` — re-raise of MAE-003 with cross-section log | Same finding as RID-S2-003. CE recommends recording in master log Cross-Section Re-Open Log against S1 (per `rid_rfa_log.md` §Cross-Section Re-Open Log). | See RID-S2-003 resolution. Record in master log re-open log when dispositioned. | OPEN | — | — | OPEN |

## 4. RFA List

ID format: `PDR-RFA-S2-NNN`.

### 4.1 MAE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S2-001 | sch_lib has no overrun-detection requirement (FLAG-1) | sch §8.2 FLAG-1 explicitly notes: no SW-REQ-SCH-* requires overrun detection; the diagnostic emission is design-level safety net only. Silent overruns risk masking integration-time issues. | PM action: file SW-REQ-SCH-011 requiring overrun detection with diagnostic emission via failure handler when `time.GetUs() - tExpectedTickUs > kSchTickUs`. Defer to CDR if not in FT1 scope. | OPEN | — | — | OPEN |
| PDR-RFA-S2-002 | log_lib variadic API surface (FLAG-3) | log FLAG-3: LOG-003 mandates printf-style variadic; LibJuno typed function-reference vtable doesn't naturally accommodate variadics. Design exposes one `LogLib_LogFmt(...)` variadic entry; only variadic surface in S2. | Confirm with PM that the resolution (single `LogFmt` formatting into `kLogMaxRecord` stack buffer via `vsnprintf`) is acceptable, or amend LOG-003 to remove variadic mandate in favor of typed `pcMessage`-only API. | OPEN | — | — | OPEN |
| PDR-RFA-S2-003 | POST-bitmap responsibility for foundation-lib `New()` failures is asymmetric | device §9.3 explicitly states caller "records the failure in the POST bitmap (SYS-029/-030)" on Configure failure. time §9, log §9, sch §9 do not mention POST. time/log/sch composition-root failures would be silently absorbed. | Software Lead clarify in conventions §4.3 (or composition-root error contract in system_design) which foundation-lib `New()` failures contribute to the POST bitmap, and ensure each L2 §9 explicitly states its POST contribution (or "none, by design"). | OPEN | — | — | OPEN |
| PDR-RFA-S2-004 | sch_lib POSIX vs Pico2 wait primitive divergence informally described | sch §3 table: POSIX uses `clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, ...)`; Pico2 busy-wait poll on `time.GetUs()`. Deliberate divergence per conventions §6, but only documented in one-line table entry. SCH-008 mandates "equivalent invocation sequences"; busy-wait vs `clock_nanosleep` could yield observably different drift/jitter. | Add brief §6/§11 statement explaining why divergence does not violate SCH-008 (e.g., "both wait until `time.GetUs() >= tNextTickUs`; dispatch sequence identical; only wait-state CPU consumption differs"). Note determinism implication for SITL vs flight test correlation. | OPEN | — | — | OPEN |
| PDR-RFA-S2-005 | kmat `kPivotEpsilon<T>` defaults stated as "tuned in nav L2" without forcing function | kmat 05_through_11.md §9 sets defaults `1e-12f` / `1e-30`; "tuned in nav L2" — but no SW-REQ-NAV-* under review yet binds the threshold. Risk: nav L2 may not realize it inherits a tuning obligation. | Add forward action to S5 (Domain Libraries) review brief to confirm `kPivotEpsilon<T>` values are tuned/accepted by `nav_lib` design, with rationale traceable to SW-REQ-NAV-015 deterministic-nav requirements. | OPEN | — | — | OPEN |
| PDR-RFA-S2-006 | time §4.2 `GetUs` Determinism row references conventions §4.3 incorrectly | time §4.2 contract row "Determinism" cites `conventions.md §4.3` for "no exception unwinding". §4.3 of conventions is "Status semantics", not exception bans (§1.3 / SYS-053). Cross-reference to wrong section; RTM tools relying on cross-references would mis-resolve. | Replace §4.3 reference with `conventions.md §1.3` and/or SYS-053. | OPEN | — | — | OPEN |

### 4.2 SSE-R RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S2-007 | time_lib no health bit — exemption rationale should cross-reference SW-REQ-SYS-058 | time §9 rule 5 states "No health bit" without cross-referencing sensor-health bitmap requirements (SYS-058/-060/-061). Exemption defensible (counter is not a "sensor") but not explicitly stated; RTM gap. | Add to time §9: "Per SYS-058/-060/-061, health bits are assigned to sensors. `time_lib` is not a sensor; failure path returns status to caller. No bit in `JUNO_MSG_SYS_HEALTH_T` for time source." | OPEN | — | — | OPEN |
| PDR-RFA-S2-008 | POST-bitmap `New()` failure attribution all five libs | (Same root as RFA-S2-003.) Clarify which foundation-lib `New()` failures contribute to POST bitmap; each L2 §9 explicitly states its POST contribution. | See RFA-S2-003 resolution. | OPEN | — | — | OPEN |
| PDR-RFA-S2-009 | sch_lib overrun detection diagnostic not promoted to requirement | (Same root as RFA-S2-001.) | See RFA-S2-001 resolution. | OPEN | — | — | OPEN |
| PDR-RFA-S2-010 | kmat bit-identical POSIX/Pico2 claim may be optimistic for `sqrt`/`Invert` | kmat §11 claims "bit-identical numeric output" under matching `-fno-fast-math`. For `+`,`-`,`*` correct under IEEE-754 with matching rounding mode. For `sqrt` and LU pivot division, bit-identity depends on libm implementation — newlib-nano on Pico2 vs glibc on x86_64 paths may differ for subnormal inputs. | Qualify equivalence claim: "Bit-identical output is expected for normal-range FP inputs under `-fno-fast-math`. For subnormal inputs, platform libm implementations may differ; nav_lib shall document its inputs remain in normal range per its own L2 design." | OPEN | — | — | OPEN |
| PDR-RFA-S2-011 | log_lib `eMinLevel` runtime non-mutability relies on convention, not enforcement | log §9 rule 4 states "`tRoot.eMinLevel` is set once at `New()` and never mutated afterwards." No compile-time enforcement; field is plain mutable member. Any caller with non-const reference to `tRoot` could write to it. | Consider making `eMinLevel` `const` in ROOT struct, initialized via aggregate initialization in `New()`'s return value. Confirm zero-init semantics preserved. | OPEN | — | — | OPEN |
| PDR-RFA-S2-012 | sch_lib `static constexpr` namespace-scope constants may trigger -Wunused-variable in some TUs | sch §3/§4.1 declare `static constexpr uint32_t kSchTickMs = 5;` at namespace scope inside header. With `-Wunused-variable -Werror`, may trigger in TUs that include the header without using the constant. C++11 standard form is `constexpr uint32_t kSchTickMs = 5;` (no `static`, internal linkage by default). Project-standard `static constexpr` is consistent with coding-standards §5; flag is informational. | Optional: change to `constexpr` (no `static`) for namespace-scope constants if -Wunused-variable surfaces in CDR. Project-standard `static constexpr` is acceptable; this is informational. | OPEN | — | — | OPEN |

### 4.3 CE RFAs

| ID | Title | Description | Recommended Resolution | Disposition | Owner | Target | Status |
|----|-------|-------------|------------------------|-------------|-------|--------|--------|
| PDR-RFA-S2-013 | kmat header-only deviation should be promoted to project-level documented exception | kmat §3.3 justifies header-only well (no platform code, templates require visible defs, IEEE-754 equivalence stronger than SW-REQ-SYS-043). Sound but lives only inside kmat L2. Risk: future S5/S7 reviewer may re-raise without seeing rationale. | Add a one-line "Documented exception: `kmat_lib` is header-only — see `kmat/index.md` §3.3" entry to conventions §6 (or new "Documented exceptions" appendix). | OPEN | — | — | OPEN |
| PDR-RFA-S2-014 | kmat Invert/QuatNormalize libm linkage should be explicitly called out | (Related to RID-S2-010 framing) On `-ffreestanding` Pico2, `std::sqrt` requires explicit `-lm` (or pico-sdk libm); on POSIX automatic. Freestanding claim in constraints could be misread. | Add to kmat §11 equivalence: "`std::sqrt` is provided by platform libm (Pico2: pico-sdk libm; POSIX: glibc libm); both linked at composition time. Freestanding flag governs C++ runtime, not libm." (Combined execution with RID-S2-010 fix.) | OPEN | — | — | OPEN |
| PDR-RFA-S2-015 | device_lib POSIX `openpty` requires `-lutil` on Linux | device §11 names `openpty` as POSIX backing primitive in tests. `glibc` exposes from `libutil`. If unit-test CMake doesn't link `-lutil`, link fails. Build-system handoff. | Add a one-liner to device §11 build-target table: "POSIX unit tests link `-lutil` for `openpty`". | OPEN | — | — | OPEN |
| PDR-RFA-S2-016 | sch_lib::Register period-multiple precondition could be a `static_assert` in templated overload | sch §4.2 enforces period-multiple at runtime via `JUNO_STATUS_INVALID_ARG_ERROR`. Since `k<App>AppPeriodMs` are `static constexpr`, a templated overload `Register<uint32_t kPeriod>(...)` could promote validation to compile-time. Optional tightening. | CDR-deferred consideration: templated period overload to move period validation to compile time, eliminating one runtime branch. | OPEN | — | — | OPEN |
| PDR-RFA-S2-017 | Pico2 sch `time_us_64()` busy-wait power consumption | sch §3: Pico2 wait is busy-wait poll on `time.GetUs()`. At 5 ms tick × 200 Hz, busy-wait dominates if any app finishes early, burning power. Battery-powered FSW worth quantifying. | CDR-deferred: measure idle CPU time per hyperperiod on Pico2 with `time_us_64()` busy-wait vs WFE/WFI; revisit if power budget tightens at FT2. | OPEN | — | — | OPEN |

## 5. Disposition Decisions

> **Chair statement, 2026-05-03:** "I approve the findings... [Chair clarified that LibJuno's `juno::sch` and `juno::app` are pure interfaces with no implementation; `juno::time` has math/conversion impls but `Now/SleepTo/Sleep` need platform impls; FT1 requirements are still valid — they are fulfilled by LibJuno + FT1 platform impls.] I concur with Option A."
>
> **Effect on findings:** Three reviewer Major findings ("`APP_ROOT_T` undefined") were based on incomplete brief — the type exists in `libjuno/include/juno/app/app_api.hpp`. The actual architectural concern, surfaced by Chair clarification, is that FT1 `time_lib`/`sch_lib` designs created parallel types in the same namespace as LibJuno's interfaces. Captured as new **PDR-RID-S2-021 (Major, Chair-raised)**. Resolution: rewrite FT1 `time_lib` and `sch_lib` designs to **implement LibJuno's interfaces verbatim** (cyclic-executive sch; time provides `Now/SleepTo/Sleep`); align FT1 app lifecycle to LibJuno's `APP_API_T { OnStart, OnProcess, OnExit }`. Reqs stay valid. The cascade is documented in §6 Action Items.

### RIDs

- `[PDR-RID-S2-001]: CLOSE-NO-ACTION` — `APP_ROOT_T` exists in `libjuno/include/juno/app/app_api.hpp`. Superseded by RID-S2-021.
- `[PDR-RID-S2-002]: ACCEPT` — Status code catalog (combined with RID-S2-009/-019). Owner: Software Lead. Target: pre-S3.
- `[PDR-RID-S2-003]: CLOSE-NO-ACTION` — `sch_lib` design rewritten under C-2 to use LibJuno's `SCH_ROOT_T<N1,N2>`; the parallel `SCH_LIB_IMPL_T<N>` name disappears.
- `[PDR-RID-S2-004]: ACCEPT` — Chair updates LOG-007 rationale prose to "stderr". Owner: Chair (PM). Target: batched-S2.
- `[PDR-RID-S2-005]: ACCEPT` — Editorial title fix in (rewritten) sch §11. Owner: Software Lead. Target: pre-S3 (auto-handled by C-2 worker brief).
- `[PDR-RID-S2-006]: ACCEPT` — Editorial: drop redundant `const` from device template parameters. Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-007]: CLOSE-NO-ACTION` — Same root as RID-S2-001.
- `[PDR-RID-S2-008]: CLOSE-NO-ACTION` — `sch_lib` design rewritten; LibJuno's `SCH_ROOT_T` already uses `JUNO_MODULE_ARG` correctly. Auto-resolved by C-2.
- `[PDR-RID-S2-009]: ACCEPT-MOD` — **Option (b) selected (revised from earlier Option (a)):** sweep all 27 L2 designs to use only the 19 codes in `juno/status.h`; for codes that don't fit any existing symbol, use `JUNO_STATUS_CUSTOM_ERROR + N` offsets defined in a new conventions.md §4.7 "Status Code Catalog" subsection. Do NOT modify `juno/status.h` (LibJuno upstream). Owner: Software Lead. Target: pre-S3.
- `[PDR-RID-S2-010]: ACCEPT-MOD` — Option (a) selected: add `juno::math::Sqrt<T>` shim using `__builtin_sqrt`/`__builtin_sqrtf`. Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-011]: CLOSE-NO-ACTION` — `time_lib` design rewritten under C-1 to provide `Now/SleepTo/Sleep` impls for LibJuno's `TIME_ROOT_T`; the upcast question doesn't arise.
- `[PDR-RID-S2-012]: ACCEPT-MOD` — Option (a) selected: remove `LogFmt` from vtable; replace with non-variadic `Log(level, tag, pcMessage)` only. Callers `vsnprintf` themselves. Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-013]: ACCEPT` — Document freestanding-vs-source distinction in log §9. Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-014]: ACCEPT` — Add cross-module constraint to device §4.2.3 (NMEA resync after `JUNO_STATUS_TABLE_FULL_ERROR`). Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-015]: ACCEPT` — Specify pivot tiebreak rule in kmat §4.2.6 / §9. Owner: Software Lead. Target: batched-S2.
- `[PDR-RID-S2-016]: ACCEPT` — Mlog cascade auto-handled by C-1, C-2 rewrites that reference `kMlogAppPeriodMs = 5`. Owner: Software Lead. Target: pre-S3.
- `[PDR-RID-S2-017]: ACCEPT-MOD` — Option (a) selected: `sys_app` holds two distinct templated refs `DEVICE_LIB_ROOT_T<2048>` (GPS) and `DEVICE_LIB_ROOT_T<kLoraRx>` (lora). Re-opens at S8 sys_app review. Owner: Software Lead → S8 board. Target: at-S8.
- `[PDR-RID-S2-018]: CLOSE-NO-ACTION` — Same root as RID-S2-001.
- `[PDR-RID-S2-019]: ACCEPT-MOD` — Same root as RID-S2-009. Option (b) selected.
- `[PDR-RID-S2-020]: CLOSE-NO-ACTION` — Same root as RID-S2-003.
- `[PDR-RID-S2-021]: ACCEPT-MOD` — **Option A selected by Chair:** rewrite FT1 `time_lib` and `sch_lib` to implement LibJuno's published `juno::time::TIME_API_T` and `juno::sch::SCH_API_T<NAppsPerFrame, NFrames>` interfaces verbatim. Map FT1's TDM model onto `SCH_ROOT_T<8, 200>` (8 app slots × 200 minor frames at 5 ms = 1000 ms major frame). Align every per-app L2 to expose `APP_ROOT_T` with `OnStart/OnProcess/OnExit` (replacing the `Start/Execute` model). Revert S1-AI-008's `JUNO_TIME_PROVIDER_T` typedef from conventions.md §4.2 (LibJuno doesn't use a provider callback — Trick injection is via supplying the `Now/SleepTo/Sleep` impl). Owner: Software Lead. Target: pre-S3.

### RFAs

- `[PDR-RFA-S2-001]: CLOSE-NO-ACTION` — LibJuno SCH semantics apply; auto-resolved by C-2.
- `[PDR-RFA-S2-002]: ACCEPT` — Resolved by RID-S2-012 disposition (LogFmt removed from vtable).
- `[PDR-RFA-S2-003]: ACCEPT` — POST-bitmap responsibility for foundation-lib `New()` failures: amend conventions §4.3 with explicit attribution. Owner: Software Lead. Target: batched-S2 (combined with C-3/C-5).
- `[PDR-RFA-S2-004]: CLOSE-NO-ACTION` — Auto-handled by sch rewrite under C-2.
- `[PDR-RFA-S2-005]: ACCEPT` — Carry to S5 nav L2 review brief.
- `[PDR-RFA-S2-006]: CLOSE-NO-ACTION` — `time_lib` rewritten under C-1; the §4.2/§4.3 cross-reference is regenerated.
- `[PDR-RFA-S2-007]: ACCEPT` — Add health-bit exemption rationale to (rewritten) time §9. Auto-handled by C-1.
- `[PDR-RFA-S2-008]: ACCEPT` — Same root as RFA-S2-003.
- `[PDR-RFA-S2-009]: CLOSE-NO-ACTION` — Same root as RFA-S2-001.
- `[PDR-RFA-S2-010]: ACCEPT` — Qualify kmat §11 equivalence claim for subnormal inputs. Owner: Software Lead. Target: batched-S2.
- `[PDR-RFA-S2-011]: ACCEPT` — Make log `eMinLevel` `const` in ROOT struct. Owner: Software Lead. Target: batched-S2.
- `[PDR-RFA-S2-012]: CLOSE-NO-ACTION` — `sch_lib` rewrite handles namespace-scope constants per LibJuno's published pattern.
- `[PDR-RFA-S2-013]: ACCEPT` — Add documented exception note for kmat header-only to conventions §6. Owner: Software Lead. Target: batched-S2 (combined with C-3/C-5).
- `[PDR-RFA-S2-014]: ACCEPT` — Combined with RID-S2-010 fix.
- `[PDR-RFA-S2-015]: ACCEPT` — Add `-lutil` linkage note to device §11. Owner: Software Lead. Target: batched-S2.
- `[PDR-RFA-S2-016]: DEFER` — CDR-deferred consideration; sch rewrite may already use LibJuno's published period validation pattern.
- `[PDR-RFA-S2-017]: DEFER` — CDR-deferred power consumption measurement; carry forward.

## 6. Action Items Created

Targets: **pre-S3** = before S3 convenes; **batched-S2** = batched corrective edits before S10; **at-section** = raised at downstream section's disposition; **CDR** = deferred per Charter §1.2.

| Action ID | Source | Description | Owner | Target | Status |
|-----------|--------|-------------|-------|--------|--------|
| S2-AI-001 | RID-S2-021 (C-1) | Rewrite `time/design.md` to implement LibJuno's `juno::time::TIME_API_T { Now, SleepTo, Sleep }` for POSIX (`clock_gettime`, `clock_nanosleep`) and Pico2 (RP2350 timer, `sleep_us`); use `TIME_ROOT_T` (no parallel types); preserve `JUNO_TIME_US_T` derivation via `Now()` + `TimestampToMicros()`; reflect `kMlogAppPeriodMs = 5` in §8 timing table. | software-systems-engineer | **pre-S3** | DONE 2026-05-03 |
| S2-AI-002 | RID-S2-021 (C-2) | Rewrite `sch/design.md` to implement LibJuno's `juno::sch::SCH_API_T<NAppsPerFrame, NFrames> { Execute, GetMinorFramePeriod, GetMajorFramePeriod }` cyclic-executive scheduler over `SCH_ROOT_T<8, 200>` (8 app slots × 200 minor frames at 5 ms = 1000 ms major); place each app's `APP_ROOT_T*` in its applicable minor frames per its period; provide POSIX impl (`Now`/`SleepTo` via `clock_nanosleep`) and Pico2 impl; reflect `kMlogAppPeriodMs = 5` in §7.1 sequence; align dispatched lifecycle to `OnProcess` (LibJuno). Preserve all 10 SCH-* requirement coverage. | software-systems-engineer | **pre-S3** | DONE 2026-05-03 |
| S2-AI-003 | RID-S2-021 (C-3+C-5) + RFA-S2-003/-008/-013 + RID-S2-009/-019 (C-6) | Edit `conventions.md`: (a) revert S1-AI-008's `JUNO_TIME_PROVIDER_T` typedef from §4.2 (LibJuno doesn't use a provider callback); (b) restate `JUNO_TIME_US_T = uint64_t` as FSW message-field convention derived via `TIME_ROOT_T::Now()` + `TimestampToMicros()`; (c) add §4.7 "Status Code Catalog" enumerating LibJuno's 19 codes plus FSW-specific custom codes derived from `JUNO_STATUS_CUSTOM_ERROR + N` (e.g., `JUNO_FSW_STATUS_NUMERIC_ERROR = JUNO_STATUS_CUSTOM_ERROR + 1`); (d) add §1.4 "App Lifecycle" requiring every app to expose an `APP_API_T` impl with `OnStart`/`OnProcess`/`OnExit` (replacing the `Start/Execute` model the FT1 sch_lib design proposed); (e) add §6 "Documented Exceptions" appendix noting kmat header-only with rationale; (f) add §4.3 "POST Bitmap Attribution" specifying which foundation-lib `New()` failures contribute to the POST bitmap. | software-systems-engineer | **pre-S3** | DONE 2026-05-03 |
| S2-AI-004 | RID-S2-021 (C-4) | Edit `system_design.md` §3.3 module catalog (note that time_lib and sch_lib implement LibJuno interfaces — no parallel types) and §8.1 composition root pseudocode (use `juno::time::TimeInit`, `juno::sch::SCH_ROOT_T<8, 200>` aggregate initialization with the LibJuno-published example pattern). | software-systems-engineer | **pre-S3** | DONE 2026-05-03 |
| S2-AI-005 | RID-S2-009/-019 design sweep | Sweep all 27 L2 designs (already-reviewed plus to-be-reviewed) to replace fabricated status-code names (`JUNO_STATUS_NULL_POINTER`, `JUNO_STATUS_OVERFLOW`, `JUNO_STATUS_OVERFLOW_ERROR`, `JUNO_STATUS_INVALID_STATE_ERROR`, `JUNO_STATUS_INVALID_ARG_ERROR`, `JUNO_STATUS_NUMERIC_ERROR`, `JUNO_STATUS_IO_ERROR`) with the canonical names from conventions §4.7. **Caveat:** the sweep is incremental — designs S3-S9 will be authored against the corrected catalog; already-reviewed designs (S1, S2 log/device/kmat) get their fix in batched-S2. | Software Lead | batched-S2 (rolling) | OPEN |
| S2-AI-006 | RID-S2-010 + RFA-S2-014 | Add `juno::math::Sqrt<T>` shim (using `__builtin_sqrt`/`__builtin_sqrtf`) to a new header (e.g., `libs/kmat_lib/include/kmat_lib/kmat_math.hpp`); document libm linkage policy in (rewritten? — kmat is being *kept*) kmat §11. | Software Lead | batched-S2 | OPEN |
| S2-AI-007 | RID-S2-012 | Remove `LogFmt` from log vtable; replace with non-variadic `Log` accepting pre-formatted `pcMessage`. Update log §4.3, §4.5 of design. | Software Lead | batched-S2 | OPEN |
| S2-AI-008 | RID-S2-013 | Add freestanding-vs-source-file distinction note to log §9. | Software Lead | batched-S2 | OPEN |
| S2-AI-009 | RID-S2-014 | Add NMEA-resync constraint to device §4.2.3 ReadBytes postconditions (cross-module obligation on nmea_lib). | Software Lead | batched-S2 | OPEN |
| S2-AI-010 | RID-S2-015 | Specify pivot tiebreak rule in kmat §4.2.6 / §9 (lower row index wins on equal magnitude). | Software Lead | batched-S2 | OPEN |
| S2-AI-011 | RID-S2-017 | At S8 sys_app review: confirm sys_app holds two distinct `DEVICE_LIB_ROOT_T<N>` references (GPS @ 2048; lora @ kLoraRx). | Software Lead → S8 board | at-S8 | OPEN |
| S2-AI-012 | RID-S2-006 | Drop redundant `const` from device template parameters (`template<size_t N>`). | Software Lead | batched-S2 | OPEN |
| S2-AI-013 | RID-S2-004 | PM action: Chair updates `requirements/log/requirements.json` LOG-007 rationale prose to "stderr". | Chair (PM) | batched-S2 | OPEN |
| S2-AI-014 | RID-S2-005 | Editorial: align sch §11 traceability-table title for SCH-004 to requirements.json title (auto-handled by C-2 worker brief). | software-systems-engineer | pre-S3 (in C-2) | OPEN |
| S2-AI-015 | RFA-S2-005 | Carry forward into S5 (Domain Libraries) review brief: confirm `kPivotEpsilon<T>` values tuned/accepted by `nav_lib`. | Software Lead → S5 board | at-S5 | OPEN |
| S2-AI-016 | RFA-S2-010 | Qualify kmat §11 POSIX/Pico2 bit-identical claim for subnormal inputs. | Software Lead | batched-S2 | OPEN |
| S2-AI-017 | RFA-S2-011 | Make log `eMinLevel` `const` in `LOG_LIB_ROOT_T`. | Software Lead | batched-S2 | OPEN |
| S2-AI-018 | RFA-S2-015 | Add `-lutil` POSIX-test linkage note to device §11 build-target table. | Software Lead | batched-S2 | OPEN |
| S2-AI-019 | post-cascade | Re-run `tools/traceability.py` after C-1..C-5 to confirm 0 errors with rewritten time/sch designs and updated conventions/system_design. | Software Lead | pre-S3 | DONE 2026-05-03 (TRACEABILITY CHECK PASSED — 371 reqs, 370 with test specs) |
| S2-AI-020 | RFA-S2-016 | DEFERRED to CDR — templated period overload optional. | — | CDR | DEFERRED |
| S2-AI-021 | RFA-S2-017 | DEFERRED to CDR — Pico2 busy-wait power consumption measurement. | — | CDR | DEFERRED |

**21 action items.** Pre-S3 critical path: S2-AI-001, -002, -003, -004, -019. Targets summarize:
- **pre-S3** (must clear before S3): 5 items (the cascade C-1..C-4 rewrites + post-cascade traceability check)
- **batched-S2** (executed before S10 closure): 12 items
- **at-section** (raised during downstream sections): 2 items (S5, S8)
- **CDR-deferred:** 2 items

## 7. Section Verdict

- **CHAIR PROCEED** — Section content is acceptable; PDR may proceed to the next section. Pre-S3 cascade actions (C-1..C-4) must close before S3 convenes.

**Verdict Notes**

The Chair approved the findings of the board, then identified that the reviewers' "APP_ROOT_T undefined" Major findings (RID-S2-001/-007/-018) were based on incomplete reviewer briefs (Software Lead error — saved to lessons-learned). The Chair clarified the LibJuno relationship: `juno::sch` and `juno::app` are pure interfaces; `juno::time` provides math/conversion impls but `Now/SleepTo/Sleep` need platform impls. FT1 requirements stay valid; they are fulfilled by LibJuno + FT1 platform impls. The Chair selected **Option A**: rewrite FT1 `time_lib` and `sch_lib` designs to implement LibJuno's published interfaces verbatim and align FT1 app lifecycle to LibJuno's `APP_API_T { OnStart, OnProcess, OnExit }`.

This pivot is captured as **PDR-RID-S2-021** with Software Lead-owned action items C-1..C-4 (S2-AI-001..-004) targeted **pre-S3**. Per Charter §7 exit criterion AC-3, all S2 Major RIDs are either CLOSE-NO-ACTION (superseded by S2-021), or assigned a corrective action with explicit Chair approval and a pre-S3 target.

Five pre-S3 critical action items must close before S3 convenes:
- **S2-AI-001** (rewrite time/design.md to LibJuno-impl model)
- **S2-AI-002** (rewrite sch/design.md to LibJuno-impl model with `SCH_ROOT_T<8, 200>`)
- **S2-AI-003** (conventions.md: revert PROVIDER_T; add §4.7 Status Code Catalog; add §1.4 App Lifecycle; add §6 documented kmat exception; add §4.3 POST bitmap attribution)
- **S2-AI-004** (system_design.md §3.3/§8.1 update for LibJuno-impl model)
- **S2-AI-019** (post-cascade traceability re-verification)

The remaining 12 batched-S2 items, 2 at-section items, and 2 CDR-deferred items are tracked into S10 closure.

**Chair Signature**: Project Manager — 2026-05-03

## 8. Cross-References

### Documents Reviewed

- [docs/design/time/design.md](../../../design/time/design.md)
- [docs/design/log/design.md](../../../design/log/design.md)
- [docs/design/sch/design.md](../../../design/sch/design.md)
- [docs/design/device/design.md](../../../design/device/design.md)
- [docs/design/kmat/index.md](../../../design/kmat/index.md), [04_interface.md](../../../design/kmat/04_interface.md), [05_through_11.md](../../../design/kmat/05_through_11.md)

### Master Log

- [PDR RID/RFA Master Log](../rid_rfa_log.md)

### Related Section Records

- [S1 Architecture](S1_architecture.md) — RID-S2-003 / RID-S2-020 are cross-section re-opens against S1's system_design.md §8.1 pseudocode (record in master log Cross-Section Re-Open Log).
- RID-S2-016 is a cascade verification finding against S1-AI-005 (mlog period not propagated to L2 designs).

## 9. Reviewer Recommendations Summary

| Reviewer | Total RIDs | Major | Minor | Editorial | RFAs | Recommendation |
|----------|-----------|-------|-------|-----------|------|----------------|
| MAE | 6 | 1 | 3 | 2 | 6 | **HOLD** (Major: RID-S2-001 APP_ROOT_T) |
| SSE-R | 9 | 4 | 5 | 0 | 6 | **HOLD** (Majors: -007, -008, -009, -010) |
| CE | 5 | 4 | 1 | 0 | 5 | **HOLD** (Majors: -016, -017, -018, -019) |
| **Section Total** | **20** | **9** | **9** | **2** | **17** | **HOLD** |

## 10. Related-Item Groups (for batched disposition)

| Root Cause | Items | Severity Class |
|-----------|-------|----------------|
| **`APP_ROOT_T` undefined polymorphic root** | RID-S2-001 (MAE, Major), RID-S2-007 (SSE-R, Major), RID-S2-018 (CE, Major) | 3× Major — cross-section project decision |
| **Fabricated status codes** | RID-S2-009 (SSE-R, Major), RID-S2-019 (CE, Major), RID-S2-002 (MAE, Minor — lighter framing) | 2× Major + 1× Minor — sweep across 27 L2s |
| **`SCH_IMPL_T` name drift L1↔L2** | RID-S2-003 (MAE, Minor), RID-S2-020 (CE, Minor) | 2× Minor — cross-section S1 re-open |
| **S1-AI-005 mlog period cascade** | RID-S2-016 (CE, Major) | 1× Major — cascade-verification of disposed S1 action |
| **`std::sqrt` freestanding** | RID-S2-010 (SSE-R, Major), RFA-S2-014 (CE) | 1× Major + 1× RFA |
| **`JUNO_MODULE_ARG` missing on sch template** | RID-S2-008 (SSE-R, Major) | 1× Major — preprocessor defect |
| **`device_lib<N>` cross-section visibility** | RID-S2-017 (CE, Major) | 1× Major — cross-section S2/S8 |
| **POST-bitmap responsibility** | RFA-S2-003 (MAE), RFA-S2-008 (SSE-R) | 2× RFA — same root |
| **sch overrun detection** | RFA-S2-001 (MAE), RFA-S2-009 (SSE-R) | 2× RFA — same root |
| **log variadic / LOG-007 stdout-vs-stderr** | RID-S2-004 (MAE, Minor), RFA-S2-002 (MAE) | 1× Minor + 1× RFA |
