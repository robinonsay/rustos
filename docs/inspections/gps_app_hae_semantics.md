---
inspection_id: INS-GPS-APP-010
requirement: SW-REQ-GPS-APP-010
test_case: SW-TC-GPS-APP-012
verification_method: Inspection
inspection_date: 2026-05-12
inspector: Software Lead
verdict: PASS
sprint_context: SPRINT-WAVE5-EXIT Wave 5 carry-forward discharge
---

# Inspection Record — SW-REQ-GPS-APP-010 HAE Altitude in Published GPS Fix

## 1. Requirement Under Inspection

**SW-REQ-GPS-APP-010** — *HAE Altitude in Published GPS Fix*

> The GPS app shall publish GPS altitude referenced to the WGS-84 ellipsoid.

- `verification_method`: **Inspection** (per [docs/requirements/gps_app/requirements.json](../requirements/gps_app/requirements.json)).
- Rationale: parent `SW-REQ-SYS-039` mandates HAE (height above the WGS-84 ellipsoid) as the FSW altitude reference; the GPS app's published fix must use that reference.
- Test case: **SW-TC-GPS-APP-012** (per [docs/test_cases/gps_app/test_cases.json](../test_cases/gps_app/test_cases.json)).

Per the SPRINT-IMPL-05-retro-B RTM-cleanup lesson (2026-05-05), Inspection-method requirements are not formally verified by automated tests. The regression-guard `TEST_F` at `apps/gps_app/tests/gps_app_test.cpp` (TC-012) provides a behavioural pass-through guard only; this record is the formal verification artifact. The GPS-18 closure record §10 carry-forward #4 queued this record for the next sprint, which is now (SPRINT-WAVE5-EXIT).

## 2. Inspection Procedure

Documentation-only inspection. Four atomic steps:

1. Inspect `libs/gps_lib/include/gps_lib/gps_msg.hpp` for the `JUNO_MSG_GPS_FIX_T` struct definition and the `fAltMHae` field's Doxygen comment. Confirm the Doxygen states (a) WGS-84 ellipsoid reference, (b) meters as the SI unit, (c) HAE (height above ellipsoid) semantic label — explicitly and adjacent to the field declaration.
2. Inspect `docs/design/gps_app/design.md` for the published-message schema (§6.3) and the data-flow table (§6.1). Confirm `fAltMHae` is documented as HAE altitude in meters and is bound to `SW-REQ-GPS-APP-010` / `SW-REQ-SYS-039`.
3. Inspect peer documents (`docs/design/conventions.md` §4.6 cross-module frame/unit table; `docs/design/gps/design.md` §2 abbreviation table) for HAE semantics consistency across the project.
4. Confirm the three semantic claims (WGS-84 ellipsoid reference; meters as SI unit; HAE label) appear together in the canonical header and are echoed by the L2 design and the conventions doc, so a downstream consumer reading any of these artifacts cannot misinterpret the field as MSL (mean sea level) or geoid undulation.

## 3. Acceptance Criterion

> The `fAltMHae` field of `JUNO_MSG_GPS_FIX_T` is documented in the canonical header (`libs/gps_lib/include/gps_lib/gps_msg.hpp`) and/or the L2 design (`docs/design/gps_app/design.md` §6.3) with explicit (a) WGS-84 ellipsoid reference, (b) meters as the SI unit, and (c) HAE (height above ellipsoid) semantic label.

## 4. Evidence

### 4.1 Step 1 — Canonical header (`libs/gps_lib/include/gps_lib/gps_msg.hpp`)

**File inspected:** `libs/gps_lib/include/gps_lib/gps_msg.hpp` (220 lines; authored Phase 0 SPRINT-IMPL-18, 2026-05-11).

**Group Doxygen for `JUNO_MSG_GPS_FIX_T`** (lines 94–95):
```
 *  - `fAltMHae`         — altitude in metres above the WGS-84 ellipsoid
 *                         (`SW-REQ-GPS-APP-010`, `SW-REQ-SYS-039`).
```

**Field declaration with adjacent Doxygen** (lines 122–123):
```
    /** @brief HAE altitude in metres above the WGS-84 ellipsoid. */
    float fAltMHae;
```

Semantic-claim coverage:
- **(a) WGS-84 ellipsoid reference** — covered by both quoted blocks: "above the WGS-84 ellipsoid" appears in the field's group Doxygen (line 94) and in the immediate `@brief` Doxygen (line 122).
- **(b) Meters as SI unit** — covered by "in metres" in both quoted blocks (lines 94 and 122).
- **(c) HAE semantic label** — covered by the field name `fAltMHae` itself (line 123) and the explicit "HAE" abbreviation in the field's `@brief` (line 122).

The Doxygen also binds the field to `SW-REQ-GPS-APP-010` and `SW-REQ-SYS-039` (line 95), making the requirement linkage explicit at the source-of-truth header.

**Verdict: PASS** — all three semantic claims are documented verbatim in the canonical header, on lines adjacent to the field declaration. A downstream consumer including `gps_lib/gps_msg.hpp` and reading IDE-rendered Doxygen tooltips would see "HAE altitude in metres above the WGS-84 ellipsoid" without ambiguity.

### 4.2 Step 2 — L2 design (`docs/design/gps_app/design.md`)

**File inspected:** `docs/design/gps_app/design.md` (500 lines; authored SPRINT-IMPL-PDR-GPS-APP, amended through SPRINT-IMPL-18).

**§6.1 Published bus messages table** (line 265):
```
| `JUNO_MSG_GPS_FIX_T` | every 200 ms tick (heartbeat; `bValid` reflects `GetFix` outcome) | Geodetic position + HAE altitude + NED velocity + fix quality (`SW-REQ-GPS-APP-004`/`-010`) | `nav_app`, `mlog_app`, `telem_app` |
```

**§6.3 Type names (verbatim from `system_design.md` §4)** (line 292):
```
    float          fAltMHae;      // HAE meters (SW-REQ-SYS-039 / SW-REQ-GPS-APP-010)
```

Semantic-claim coverage:
- **(a) WGS-84 ellipsoid reference** — covered by "HAE" label in both quotes, with HAE expanded to "Height Above WGS-84 Ellipsoid" in the peer `gps/design.md` §2 (see §4.3 below) and in `conventions.md` §4.6 (see §4.3 below).
- **(b) Meters as SI unit** — covered by "HAE meters" in §6.3 (line 292).
- **(c) HAE semantic label** — covered by the literal "HAE" in §6.1 (line 265) and §6.3 (line 292) plus the field name `fAltMHae` (§6.3 line 292).

§6.3 additionally cross-binds the field to `SW-REQ-SYS-039` and `SW-REQ-GPS-APP-010`, matching the header's tags exactly. The §11 traceability table at line 495 reiterates: "SW-REQ-GPS-APP-010 | HAE Altitude in Published GPS Fix | §1, §4.3 (`OnProcess`), §6.3, §7.2".

**Verdict: PASS** — L2 design echoes the header's three semantic claims at both the message-catalog level (§6.1) and the type-definition level (§6.3) and binds them to the same requirement IDs.

### 4.3 Step 3 — Peer documents (cross-module consistency)

**File:** `docs/design/conventions.md` §4.6 "Frame and unit conventions" (lines 224–229):
```
| Position | WGS-84 geodetic latitude/longitude (deg), altitude (m) | `SW-REQ-SYS-038` |
| Altitude reference | WGS-84 ellipsoid (HAE) | `SW-REQ-SYS-039` |
| Velocity frame | NED (North-East-Down) | `SW-REQ-SYS-040` |
| Attitude | Unit quaternion (w, x, y, z), body→NED | `SW-REQ-SYS-041` |
| Body axes | X-forward, Y-right, Z-down | `SW-REQ-SYS-057` |
| Units | SI throughout | `SW-REQ-SYS-042` |
```

This is the cross-module canonical source: row 2 binds altitude reference to "WGS-84 ellipsoid (HAE)" under `SW-REQ-SYS-039`; row 1 binds altitude to "(m)" (meters); row 6 binds the whole module-suite to "SI throughout" under `SW-REQ-SYS-042`. Together these rows authoritatively establish all three semantic claims at the project level — any L2 design and any header consuming the convention inherits the semantics.

**File:** `docs/design/gps/design.md` §2 "Definitions and Abbreviations" (line 39):
```
| HAE | Height Above WGS-84 Ellipsoid (`SW-REQ-SYS-039`) |
```

This expands the HAE abbreviation used by both the canonical header's Doxygen and the L2 design.

Semantic-claim coverage:
- **(a) WGS-84 ellipsoid reference** — covered by `conventions.md` line 225 ("WGS-84 ellipsoid (HAE)") and `gps/design.md` line 39 ("Height Above WGS-84 Ellipsoid").
- **(b) Meters as SI unit** — covered by `conventions.md` line 224 ("altitude (m)") and line 229 ("SI throughout").
- **(c) HAE semantic label** — covered by `conventions.md` line 225 and `gps/design.md` line 39.

**Verdict: PASS** — peer documents are consistent with the canonical header and L2 design; no contradictory altitude convention (MSL, geoid, ECEF height) appears anywhere in the project conventions.

### 4.4 Step 4 — Confirmation across all three artifact classes

The three semantic claims appear together as follows:

| Claim | Canonical header (`gps_msg.hpp`) | L2 design (`gps_app/design.md`) | Conventions / peer L2 |
|-------|----------------------------------|---------------------------------|------------------------|
| (a) WGS-84 ellipsoid reference | lines 94, 122 | line 292 ("HAE" — expanded by §4.3) | `conventions.md` line 225; `gps/design.md` line 39 |
| (b) Meters as SI unit | lines 94 ("metres"), 122 ("metres") | line 292 ("meters") | `conventions.md` line 224 ("(m)"), 229 ("SI throughout") |
| (c) HAE semantic label | lines 94, 122 (`@brief` + group Doxygen); field name `fAltMHae` (line 123) | lines 265, 292 ("HAE"); field name `fAltMHae` (line 292) | `conventions.md` line 225 ("HAE"); `gps/design.md` line 39 ("HAE") |

All three claims are documented in three independent canonical artifacts (header + L2 design + conventions), with cross-referencing requirement IDs (`SW-REQ-GPS-APP-010` and `SW-REQ-SYS-039`) anchoring the chain. A downstream consumer cannot reach a non-HAE interpretation of `fAltMHae` from any of these sources.

**Verdict: PASS** — acceptance criterion is satisfied.

## 5. Verdict

**PASS** — the `fAltMHae` field of `JUNO_MSG_GPS_FIX_T` is documented in the canonical header (`libs/gps_lib/include/gps_lib/gps_msg.hpp` lines 94, 122–123) and in the L2 design (`docs/design/gps_app/design.md` §6.1 line 265 and §6.3 line 292) with explicit WGS-84 ellipsoid reference, meters (SI) unit, and HAE semantic label. Peer documents (`conventions.md` §4.6 lines 224–229 and `gps/design.md` §2 line 39) corroborate the convention at the project level. `SW-REQ-GPS-APP-010` is verified by Inspection per its declared method.

## 6. Related Test Artifact (informational)

[`apps/gps_app/tests/gps_app_test.cpp`](../../apps/gps_app/tests/gps_app_test.cpp) contains a `TEST_F` named `OnProcess_HaeAltitudePassThroughVerbatim` (TC-012, declared at line 395) that exercises four representative altitude values (0.0 m, 100.5 m, −20.0 m, 8848.0 m) and asserts byte-for-byte (`memcmp`) preservation of `GPS_FIX_T::fAltMHae` through the gps_lib → gps_app → bus chain. This is a **narrow regression guard** against silent transformation, scaling, or truncation of the altitude field — it verifies pass-through behaviour only, NOT the documentation-based semantic claims that this inspection record covers. The TEST_F does NOT carry a `@verify` tag for `SW-REQ-GPS-APP-010` (intentionally — Test artifacts must not claim verification of Inspection-method requirements per IEEE 829). The intentional omission of `@verify` and the regression-guard comment block citing the SPRINT-IMPL-05-retro-B RTM-cleanup lesson (2026-05-05) were applied during the GPS-18 Phase 3 Lead-direct atomic correction (per `docs/sprints/SPRINT-IMPL-18_gps_app.md` §6 Phase 3 row); see also `gps_app_test.cpp` lines 387–394 for the inline rationale.

## 7. Re-Inspection Triggers

This inspection record must be re-executed (and re-signed) when any of the following changes:

- `libs/gps_lib/include/gps_lib/gps_msg.hpp` — the `fAltMHae` field is renamed, retyped, removed, or its Doxygen comment (lines 94, 122–123) is edited.
- `docs/design/gps_app/design.md` §6.1 (line 265) or §6.3 (line 292) published-message schema is changed (e.g., field renamed, altitude reference text edited, requirement-ID binding altered).
- `docs/design/conventions.md` §4.6 (lines 224–229) altitude-reference convention is changed (e.g., FSW switches from HAE to MSL or introduces an alternate frame).
- `docs/design/gps/design.md` §2 (line 39) HAE abbreviation expansion is changed.
- Any future sprint changes the altitude semantic convention at the system level (e.g., `SW-REQ-SYS-039` reworded; an MSL alternative published).

Re-inspection follows the same 4-step procedure and records a new entry below the Approval section (or supersedes this record with a new `INS-GPS-APP-010-REV-B` document).

## 8. Approval

| Field | Value |
|-------|-------|
| Inspector | Software Lead |
| Date | 2026-05-12 |
| Sprint | SPRINT-WAVE5-EXIT (Wave 5 Exit Gate + GPS-18 carry-forward discharge) |
| Verdict | **PASS** |
| Predecessor inspection | None (first inspection record for SW-REQ-GPS-APP-010) |
| Tooling used | `Read` against working-tree at HEAD post-SPRINT-IMPL-18 closure (2026-05-11); line numbers verified per the 2026-05-08 "line citation off-by-one" lesson |
