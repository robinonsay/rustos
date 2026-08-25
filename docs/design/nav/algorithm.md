---
document_type: nav_lib EKF Algorithm Specification (L2 sister to design.md)
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-03
parent: docs/design/nav/design.md
covers: SW-REQ-NAV-018, SW-REQ-NAV-019
status: Draft (awaiting MAE + SSE-R + CE gate)
---

# nav_lib — EKF Algorithm Specification (L2 sister)

Sister document to [`design.md`](design.md). Covers EKF algorithm internals;
the companion governs public API contract, message catalog, preconditions,
state machine, and traceability for `SW-REQ-NAV-001`–`-017`.

---

## 1. Purpose and Scope

This document specifies the Extended Kalman Filter (EKF) algorithm chosen for
`nav_lib` per `SW-REQ-NAV-018`, the state vector composition consistent with
the 16-state contract pinned in [`design.md`](design.md) §4.1, the process
and measurement models, and the noise/covariance load-time configuration
surface that `NAV_INIT_T` exposes per `SW-REQ-NAV-019`. Reference covariance
values are provided as installation guidance only — every number is
load-time configurable through `NAV_INIT_T`.

Phase-aware sensor fusion (skip `UpdateBaro`/`UpdateGps` during BOOST and a
1-second post-boost settling window) is enforced by `nav_app` — see §8.
From `nav_lib`'s perspective, `PredictImu` is always called and `Update*`
calls arrive when `nav_app` chooses; the boost dead-reckon path is the same
path used when GPS or baro is unavailable (`SW-REQ-NAV-013`).

This file does not redeclare API symbols, message types, or the public
state-machine — those remain in [`design.md`](design.md).

---

## 2. Definitions

Cross-module vocabulary (frames, time base, status semantics, message
naming) is defined in [`../conventions.md`](../conventions.md) §4 and not
redefined here. The 16-state composition is pinned in
[`design.md`](design.md) §2 and §4.1 and inherited verbatim.

Module-local terms used only by this algorithm specification:

| Term | Meaning |
|------|---------|
| State vector | `juno::kmat::VEC_T<double, 16>`-shaped public output per `SW-REQ-NAV-004`; composition pinned in §3.1 |
| Process noise (`Q`) | Caller-supplied covariance terms governing IMU prediction-step uncertainty; loaded via `NAV_INIT_T` (§5.1) |
| Measurement noise (`R_baro`, `R_gps`) | Caller-supplied covariance terms governing baro and GPS update-step uncertainty; loaded via `NAV_INIT_T` (§5.1) |
| Innovation (`y`) | Measurement residual `y = z - h(x)` after subtracting the predicted measurement |
| Kalman gain (`K`) | Innovation weighting matrix `K = P H^T (H P H^T + R)^-1` |
| Internal dimension (`kInternalDim`) | IMPL-private state size used by the EKF math; may equal 16 (full-state EKF) or 15 (error-state EKF using a 3-vector small-rotation parameterization). Public API is always 16 — see §3.1. |
| WGS-84 | World Geodetic System 1984 ellipsoid; the geodetic frame referenced by `tPosLla` per `SW-REQ-SYS-038`/`-039` |

---

## 3. State Vector and Process Model

### 3.1 State vector composition (16 states, public)

The public state vector exposed through `NAV_STATE_T` is the 16-component
vector pinned in [`design.md`](design.md) §4.1:

| Component | Dimension | Units / Frame |
|-----------|-----------|---------------|
| `tPosLla` (lat, lon, alt) | 3 | deg, deg, m HAE (WGS-84) |
| `tVelNed` (Vn, Ve, Vd) | 3 | m/s, NED |
| `tAttQuat` (w, x, y, z) | 4 | unit quaternion, body→NED, Hamilton |
| `tAccelBias` | 3 | m/s², body |
| `tGyroBias` | 3 | rad/s, body |
| **Total** | **16** | |

EKF literature commonly recommends an **error-state** internal
representation (15 internal states with a 3-vector small-rotation
parameterization replacing the 4-component quaternion) for numerical
stability and to avoid the unit-norm constraint on the covariance. Per
[`design.md`](design.md) §2 line 47, the IMPL may use any internal
representation — only the 16-state output is contracted on the API.
Concretely:

- `kInternalDim` may be 16 (full-state EKF on the unit quaternion, with
  explicit re-normalization after each predict — see §6) or 15 (error-state
  EKF with a delta-quaternion update injected onto the nominal quaternion
  after each filter step).
- Either choice yields the same 16-state public output through
  `NavLib_GetState`.
- The choice is documented in the IMPL TU (`libs/nav_lib/src/nav_impl.cpp`)
  and is verified against ground-truth in `SW-TC-NAV-021`.

### 3.2 Process model — IMU-driven prediction

The prediction step is invoked once per IMU sample admitted by `nav_app`
via `NavLib_PredictImu(tRoot, tSample)`. Nominal cadence is 200 Hz
(`kImuAppPeriodMs = 5 ms`, [`contracts.md`](contracts.md) §8); multi-sample drain
may invoke it more often per nav tick. `dt` is computed in the IMPL as the
difference between `tSample.tTimestampUs` and the previously-accepted
sample's timestamp, in seconds.

For one prediction step with input
`tSample = {tAccelBodyMps2, tGyroBodyRps, tTimestampUs}`:

1. **Bias correction.** Subtract the current bias estimates:
   - `a_meas_body = tAccelBodyMps2 - tAccelBias`
   - `omega_meas_body = tGyroBodyRps - tGyroBias`
2. **Rotate accel to NED.** Using the current attitude quaternion `q`, apply
   the body→NED rotation: `a_ned_meas = QuatRotate(q, a_meas_body)`
   (`juno::kmat::QuatRotate`, see kmat §4.6).
3. **Subtract gravity.** Form the kinematic acceleration in NED:
   `a_ned = a_ned_meas - g_ned`, where
   `g_ned = (0, 0, +9.80665) m/s²` (gravity points down in NED). The
   constant `9.80665` is the standard gravity used project-wide.
4. **Velocity integration.** `tVelNed_new = tVelNed_old + a_ned * dt`
   (forward Euler is acceptable at 200 Hz; higher-order integrators are an
   IMPL choice and do not affect the API).
5. **Position integration (geodetic update).** Compute the NED position
   delta `dpos_ned = tVelNed_old * dt + 0.5 * a_ned * dt^2`, then convert
   to geodetic deltas using the local WGS-84 reference at the current
   `tPosLla` (meridional and prime-vertical radii of curvature). Update
   `tPosLla[0..2]` accordingly. Numerical care for the geodetic conversion
   is required when `dpos_ned` magnitude is small relative to the radius of
   curvature; the IMPL is free to use the linearized form as long as it is
   bit-stable across POSIX and Pico2.
6. **Attitude propagation.** Form the small-rotation quaternion
   `dq = q(omega_meas_body * dt)` (e.g., axis-angle-to-quaternion with the
   half-angle approximation when `|omega * dt|` is small), then
   `q_new = QuatMul(q_old, dq)` (Hamilton product, body-frame increment on
   the right per `juno::kmat::QuatMul` semantics). Renormalize per §6.
7. **Bias evolution.** Accel and gyro bias states evolve as zero-mean
   random-walk processes — they have no deterministic dynamics:
   `tAccelBias_new = tAccelBias_old`, `tGyroBias_new = tGyroBias_old`.
   Their uncertainty grows through `Q` only.
8. **Covariance propagation.** `P_new = F P_old F^T + Q`, where
   `F = ∂f/∂x` is the state-transition Jacobian evaluated at the current
   estimate. **The IMPL shall use the analytic F derived from one of the
   following normative references** (both peer-reviewed standards in the
   GNSS/INS literature; either may be selected, but the chosen reference
   shall be cited in the IMPL TU and used consistently for all entries
   of F):
   - **Groves, P. D. (2013).** *Principles of GNSS, Inertial, and
     Multisensor Integrated Navigation Systems* (2nd ed.), Artech House,
     **§14.2** ("Inertial Navigation System Error Equations") and
     **§14.3.1** ("INS Error State EKF") — provides closed-form
     transition matrices for both full-state and error-state EKF
     formulations of geodetic-frame INS with quaternion attitude. This
     is the recommended reference for the FT1 implementation.
   - **Trawny, N. and Roumeliotis, S. I. (2005).** "Indirect Kalman
     Filter for 3D Attitude Estimation," University of Minnesota MARS
     Lab **Technical Report TR-2005-002**, available at
     `https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf`
     — provides the quaternion-error-state Jacobian blocks (§3.5, eq.
     147–150) consistent with the Hamilton-convention quaternion used by
     `juno::kmat`.

   Numerical (finite-difference) Jacobians are NOT acceptable for the
   FT1 implementation: they introduce non-determinism (step-size choice
   affects the result bit-for-bit) and they cannot meet the SW-TC-NAV-021
   tolerance against an analytic ground-truth reference. The SW-TC-NAV-021
   reference EKF (see §6, `expected_artifacts` of the test case) shall use
   the same normative reference as the IMPL so both parties derive
   bit-identical F matrices.

`F`, `P`, `Q` are `kInternalDim`-square, implementable via
`juno::kmat::MAT_T<double, kInternalDim, kInternalDim>` (kmat §4.1) with
`MatMul`, `Add`, `Transpose` (§4.2.1–§4.2.3). Quaternion math uses
`QUAT_T<double>`, `QuatMul`, `QuatRotate`, `QuatNormalize` (kmat §4.6).

The IMU-only dead-reckon path during BOOST is the loop above repeated at
IMU cadence — no special boost-mode code. Bias states are estimable only
when measurement updates are available; during long pure-prediction the
bias covariance grows monotonically and bias estimates stay at their
pre-boost values (zero-mean random-walk dynamics).

---

## 4. Measurement Models

### 4.1 Baro update — altitude observation

The barometer reports an altitude in HAE meters. The measurement model is
linear in the state:

- `h(x) = x.tPosLla[2]` (the altitude state, m HAE)
- `H = [0 0 1 0 ... 0]` (1 × kInternalDim, with the 1.0 in the column
  corresponding to altitude in the chosen internal parameterization)
- `R_baro = (fBaroNoiseSigmaM)^2` (1 × 1 scalar variance, populated from
  `NAV_INIT_T.fBaroNoiseSigmaM`, §5.1)

The standard EKF update sequence:

1. Innovation: `y = z_baro - x.tPosLla[2]` (scalar, meters)
2. Innovation covariance: `S = H P H^T + R_baro` (scalar, since `H` is a
   single row)
3. Kalman gain: `K = P H^T / S` (column vector, kInternalDim × 1)
4. State update: `x_new = x_old + K * y`
5. Covariance update — Joseph form recommended for numerical stability
   (`SW-REQ-NAV-015`):
   `P_new = (I - K H) P_old (I - K H)^T + K R_baro K^T`

The simple form `P_new = (I - K H) P_old` is permitted as an IMPL choice
when the divergence-rejection gate (§4.4) and symmetry enforcement (§6)
are both active. The IMPL choice must be documented in the IMPL TU.

### 4.2 GPS update — position + velocity observation

The GPS receiver reports geodetic position and NED velocity. The
measurement is 6-dimensional (lat, lon, alt, Vn, Ve, Vd):

- `h(x)` returns the corresponding 6 states: position (3) and NED velocity
  (3)
- `H` (6 × kInternalDim) has 1.0 entries on the rows for the position and
  velocity components and zeros elsewhere
- `R_gps` is 6 × 6 diagonal with entries populated from
  `NAV_INIT_T.fGpsHorizNoiseSigmaM` (squared, used twice for lat and lon
  after conversion to meters via local WGS-84 radii),
  `NAV_INIT_T.fGpsVertNoiseSigmaM` (squared, for altitude), and
  `NAV_INIT_T.fGpsVelNoiseSigmaMps` (squared, used three times for Vn, Ve,
  Vd) — see §5.1

The update sequence is the same shape as §4.1, with `S = H P H^T + R_gps`
inverted using `juno::kmat::Invert<double, 6>` (kmat §4.2.6). Joseph form
for the covariance update is recommended.

### 4.3 GPS divergence-bound check (`SW-REQ-NAV-014`)

Before applying the GPS update, the IMPL must check the horizontal-position
innovation against `tInit.fGpsBoundM` (default `kNavGpsBoundM_default = 200.0`
m, [`contracts.md`](contracts.md) §9):

- `innov_horiz_m = great_circle_distance((x.tPosLla[0], x.tPosLla[1]),
   (z_gps.dLatDeg, z_gps.dLonDeg))` (or local-tangent-plane approximation
  using meridional and prime-vertical radii of curvature; the IMPL choice
  must be deterministic across POSIX/Pico2 per `SW-REQ-NAV-016`)
- If `innov_horiz_m > tInit.fGpsBoundM`: return
  `juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR`
  ([`design.md`](design.md) §4.5), transition state machine to `Diverged`,
  set internal `bValid = false`. The update is **not** applied — neither
  state nor covariance is altered by a rejected GPS measurement.
- Else: apply the §4.2 update normally.

### 4.4 Update gating note (algorithm-level)

`NavLib_UpdateBaro` and `NavLib_UpdateGps` accept measurements
**unconditionally** when invoked (subject only to the precondition checks
in [`design.md`](design.md) §4.3 and the divergence bound in §4.3 above).
Phase-aware gating — skipping these calls during BOOST and during a 1-second
settling window after BOOST exit — is the responsibility of `nav_app`.
`nav_lib` does not subscribe to `JUNO_MSG_AFM_PHASE_T`. See
[`../nav_app/design.md`](../nav_app/design.md) §4.4 for the gating logic.

A measurement-acceptance gate (e.g., reject baro if
`|y| > N * sqrt(S)` for some configurable `N`) is an IMPL choice. If
implemented, it must be deterministic and the threshold must be either
hardcoded or carried on `NAV_INIT_T`; this design does not pin a specific
value. SW-TC-NAV-022 verifies the load-time configurability of the noise
fields, not the optional gate.

---

## 5. Noise / Covariance Configuration

### 5.1 NAV_INIT_T schema (`SW-REQ-NAV-019`, `SW-REQ-NAV-020`)

`NAV_INIT_T` is declared in [`design.md`](design.md) §4.1. Per
`SW-REQ-NAV-019`, it must carry the following caller-supplied noise and
covariance fields. These are appended to the existing `NAV_INIT_T`
(`tInitialState`, `fGpsBoundM`, `bUseBaroAlt`); the field types and
naming follow `docs/design/conventions.md` §4 (units suffix, frame
suffix, Hungarian prefix):

| Field | Type | Units | Use |
|-------|------|-------|-----|
| `fImuAccelNoiseSigmaMps2[3]` | `double[3]` | m/s² (1-sigma) | Per-axis IMU accel measurement noise; populates `Q` diagonal entries for accel-derived states |
| `fImuGyroNoiseSigmaRps[3]` | `double[3]` | rad/s (1-sigma) | Per-axis IMU gyro measurement noise; populates `Q` diagonal entries for gyro-derived states |
| `fImuAccelBiasRandomWalkMps2PerSqrtS[3]` | `double[3]` | m/s² / √s | Accel bias random-walk rate; populates `Q` diagonal for accel-bias states |
| `fImuGyroBiasRandomWalkRpsPerSqrtS[3]` | `double[3]` | rad/s / √s | Gyro bias random-walk rate; populates `Q` diagonal for gyro-bias states |
| `fBaroNoiseSigmaM` | `double` | m (1-sigma) | Baro altitude measurement noise; populates `R_baro` |
| `fGpsHorizNoiseSigmaM` | `double` | m (1-sigma) | GPS horizontal-position noise; populates `R_gps` lat/lon entries (after meter conversion) |
| `fGpsVertNoiseSigmaM` | `double` | m (1-sigma) | GPS vertical-position noise; populates `R_gps` altitude entry |
| `fGpsVelNoiseSigmaMps` | `double` | m/s (1-sigma) | GPS velocity noise; populates `R_gps` velocity entries |
| `fGpsBoundM` | `double` | m | GPS divergence bound (already pinned by `SW-REQ-NAV-014`; included here for completeness) |
| `fInitialCovDiag[16]` | `double[16]` | per-state variance | **Initial state covariance P_0 diagonal per `SW-REQ-NAV-020`** (added 2026-05-03 to close implementation-readiness gap G2). Indexing matches `NAV_STATE_T` component order: `[0..2]` `tPosLla` variance; `[3..5]` `tVelNed` variance (m²/s²); `[6..9]` `tAttQuat` variance; `[10..12]` `tAccelBias` variance (m²/s⁴); `[13..15]` `tGyroBias` variance (rad²/s²). Caller chooses values reflecting seed confidence. The IMPL constructs `P_0 = diag(fInitialCovDiag)` at `NavLib_Init` and uses this as the EKF's initial covariance for the first `PredictImu`/`UpdateBaro`/`UpdateGps` cycle. |

The IMPL stores these values into `NAV_LIB_IMPL_T` at `NavLib_Init` entry
(by value copy from the caller's `NAV_INIT_T`), squares them as needed to
produce variances, and uses them throughout the lifetime of the filter.

There is **no API to update covariance mid-flight** — load-time only.
Re-tuning requires `nav_app` to call `NavLib_Init` again with a fresh
`NAV_INIT_T`, transitioning the state machine to `Aligning` per
[`contracts.md`](contracts.md) §5. This keeps the configuration surface
auditable and preserves determinism (`SW-REQ-NAV-015`).

### 5.2 Reference covariance values (installation guidance only)

The values below are **installation guidance** derived from the MPU-6050
datasheet and nominal GPS performance. They are **not** pinned defaults —
every value is load-time configurable per `SW-REQ-NAV-019` and the
`nav_app` composition root supplies them through `NAV_INIT_T`. Treat this
table as a starting point for FT1; FT2 may re-tune.

| Reference value | Suggested number | Source |
|-----------------|------------------|--------|
| Accel noise σ (per axis) | ~400 µg/√Hz × √200 Hz ≈ 5.6 mg ≈ 0.055 m/s² | MPU-6050 datasheet (Total RMS Noise) |
| Gyro noise σ (per axis) | ~0.005 °/s/√Hz × √200 Hz ≈ 0.07 °/s ≈ 0.0012 rad/s | MPU-6050 datasheet |
| Accel bias random walk | ~5e-4 m/s²/√s | Typical MEMS accelerometer aging |
| Gyro bias random walk | ~2e-5 rad/s/√s | Typical MEMS gyroscope aging |
| Baro altitude noise σ | ~1.5 m | MPL3115A2 datasheet altitude resolution + atmospheric variability |
| GPS horizontal position σ | ~2.5 m | Nominal HDOP=1.5 × 1.5 m URA |
| GPS vertical position σ | ~5.0 m | Typical 2× horizontal degradation |
| GPS velocity σ | ~0.1 m/s | Typical receiver velocity noise |

> **These values are provided as installation guidance only. The
> `NAV_INIT_T` fields in §5.1 are the authoritative source at run time
> per `SW-REQ-NAV-019`. The IMPL must not hardcode any of these numbers
> as algorithm constants — the only `constexpr` numeric in the algorithm
> path is the standard gravity `9.80665` (§3.2 step 3).**

---

## 6. Numerical Stability and Determinism

The EKF math is sensitive to several numerical hazards. The IMPL must
address each:

- **Quaternion renormalization.** After each prediction step,
  `q_new = QuatNormalize(q_new)` (`juno::kmat::QuatNormalize`, kmat §4.6)
  — the unit-norm constraint drifts over time under the Euler-step
  propagation in §3.2. `QuatNormalize` returns
  `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` if `|q| < kPivotEpsilon<T>`;
  in that case the IMPL must transition to `Diverged` and return
  `juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR` to the caller per
  [`design.md`](design.md) §4.5.

- **Covariance update form.** Joseph form
  (`P_new = (I - K H) P_old (I - K H)^T + K R K^T`) is recommended over
  the simple form (`P_new = (I - K H) P_old`) — it is symmetric by
  construction and roughly 2× the cost. The IMPL chooses; the choice
  must be documented in the IMPL TU and tested against the ground-truth
  reference in SW-TC-NAV-021.

- **Symmetry enforcement.** After each update, enforce
  `P_new = 0.5 * (P_new + Transpose(P_new))` to suppress drift away from
  symmetry caused by floating-point round-off. This is cheap (one
  `Transpose`, one `Add`, one `Scale` from kmat §4.2) and is recommended
  regardless of whether Joseph form is used.

- **Determinism (`SW-REQ-NAV-015`).** Bit-identical outputs on POSIX and
  Pico2 require: (a) all `juno::kmat` operations are deterministic
  (`SW-REQ-KMAT-009`/`-010`) — `nav_lib` inherits this property by
  construction since the only floating-point math it does outside of
  kmat is the gravity subtraction and dt scaling; (b) no time-dependent
  ordering in matrix operations — straight-line code only, no thread
  fanout; (c) no compile flags that vary across builds (CMake supplies
  consistent `-O2 -fno-fast-math -fno-exceptions -fno-rtti` to both
  POSIX and Pico2 targets). The IMPL must not introduce
  `-ffast-math` or any non-IEEE-754 floating-point behavior.

- **Pivot guarding in measurement update.** `juno::kmat::Invert` for the
  6×6 GPS innovation covariance returns
  `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` when partial-pivot LU
  detects a near-zero pivot (kmat §4.2.6). The IMPL must escalate this
  to `juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR` and reject the update.
  The 1×1 baro innovation covariance is a scalar; the IMPL must check
  `S > kPivotEpsilon<double>` before dividing, using the same
  threshold the kmat layer uses (cross-reference kmat §9 in the
  IMPL TU).

- **Reference EKF (off-line).** The ground-truth values used by
  `SW-TC-NAV-021` to validate the algorithm are produced by an
  off-line Python NumPy reference EKF run with `numpy.random.default_rng`
  seeded from a fixed seed and with `numpy.float64` precision throughout.
  The reference is committed to the repo at the path identified in the
  test case JSON; review by SSE-R confirms the reference matches the
  algorithm pinned in §3 and §4.

---

## 7. Implementation Notes (informative)

Informative — none of this section alters the contract. The IMPL declares
internal state/covariance via `VEC_T<double, kInternalDim>` and
`MAT_T<double, kInternalDim, kInternalDim>` (see [`design.md`](design.md)
§10.1). This spec does not prescribe a specific kmat call sequence; the
IMPL author selects ordering using only published kmat types from
[`../kmat/04_interface.md`](../kmat/04_interface.md) §4.1, §4.2, §4.6.

Freestanding-compliance (`SW-REQ-SYS-050`/`-053`): no STL containers, no
exceptions, no virtual dispatch, no `new`/`delete`. Implementable in C++11
freestanding with `<cstddef>`, `<cstdint>`, `<cmath>` (`std::sqrt`,
`std::sin`, `std::cos`), and LibJuno kmat/status/result headers.

---

## 8. Phase-Aware Behavior (cross-reference)

`nav_lib` has **no phase awareness**. It does not subscribe to
`JUNO_MSG_AFM_PHASE_T`, does not track flight phase internally, and has
no per-phase code paths. From the algorithm's perspective:

- During BOOST: `nav_app` calls only `PredictImu`. The EKF dead-reckons
  using the §3.2 prediction loop; bias states stay at their pre-boost
  values (random-walk zero-mean) and their covariance grows.
- During the 1-second post-boost settling window: same behavior as BOOST
  (predict-only), allowing high-rate IMU dynamics to settle before
  re-introducing baro and GPS measurement updates.
- During COAST / DESCENT / LANDED: `nav_app` resumes calling
  `UpdateBaro` and `UpdateGps` at their respective cadences; the EKF
  measurement-update path (§4) is exercised normally.

`nav_app` performs all phase gating per
[`../nav_app/design.md`](../nav_app/design.md) §4.4 (added in this
sprint). The IMU-only dead-reckon during boost is the same algorithm path
as the degraded-input continuation contract `SW-REQ-NAV-013` /
[`design.md`](design.md) §3.4 — there is no special-case code.

---

## 9. Cross-References

- [`design.md`](design.md) — public API contract, message catalog, vtable
  shape, per-function preconditions, FSW-extension status codes,
  state machine, and full per-requirement traceability for
  `SW-REQ-NAV-001`–`-017`.
- [`index.md`](index.md) — TOC for the `nav_lib` L2 design (Lead-direct,
  Phase 4 of this sprint).
- [`../conventions.md`](../conventions.md) — cross-module vocabulary,
  time base, frames, status-code policy, FSW-extension namespace policy.
- [`../kmat/04_interface.md`](../kmat/04_interface.md) — kmat published
  types (`MAT_T`, `VEC_T`, `QUAT_T`) and operations (`MatMul`,
  `Transpose`, `Add`, `Sub`, `Scale`, `Invert`, `QuatMul`,
  `QuatNormalize`, `QuatRotate`, `MatVecMul`).
- [`../kmat/05_through_11.md`](../kmat/05_through_11.md) — kmat numeric
  policy (`kPivotEpsilon<T>`), determinism contract.
- [`../nav_app/design.md`](../nav_app/design.md) — subscriber-side
  phase gating logic; `JUNO_MSG_AFM_PHASE_T` subscription;
  predict-only dead-reckon during BOOST + 1 s settling.
- `libjuno/include/juno/result.hpp` — `RESULT_T<T>` returned by every
  `nav_lib` API call.
- `libjuno/include/juno/status.h` — `JUNO_STATUS_T`,
  `JUNO_STATUS_CUSTOM_ERROR` base for FSW extensions.

---

## 10. Memory Ownership (delta from contracts.md §10)

Adds **no new caller-owned storage** beyond [`contracts.md`](contracts.md) §10.
The §5.1 `NAV_INIT_T` noise/covariance fields are caller-supplied by value
at `NavLib_Init` entry (caller constructs on stack or `.bss`, passes by
`const&`). The IMPL copies values into existing `NAV_LIB_IMPL_T` storage
(`tProcNoise` plus measurement-noise members `tBaroVar`, `tGpsVar` of
identical ownership and lifetime). No heap, no new buffers, no new
lifetime concerns. [`contracts.md`](contracts.md) §10 remains authoritative.

---

## 11. Traceability

The `<!-- @{"design": [...]} -->` tag below is authoritative; this table
is a delta covering only EKF-pin amendment requirements. Full nav_lib
traceability for `SW-REQ-NAV-001`–`-017` remains in
[`contracts.md`](contracts.md) §11.

| Req ID | Title | Sections |
|--------|-------|----------|
| `SW-REQ-NAV-018` | EKF Algorithm | §1, §3 (state vector + process model), §4 (measurement models), §6 (numerical stability) |
| `SW-REQ-NAV-019` | Configurable Noise Covariance | §5.1 (`NAV_INIT_T` schema), §5.2 (reference values), §10 (memory delta) |

Test cases verifying these requirements (added in Phase 1 of this
sprint, see `docs/test_cases/nav/test_cases.json`):

| Test ID | Verifies | Type |
|---------|----------|------|
| `SW-TC-NAV-021` | `SW-REQ-NAV-018` (algorithm pinned to EKF; ground-truth comparison) | Unit |
| `SW-TC-NAV-022` | `SW-REQ-NAV-019` (load-time configurability of noise/covariance via `NAV_INIT_T`) | Unit |

<!-- @{"design": ["SW-REQ-NAV-018", "SW-REQ-NAV-019"]} -->
