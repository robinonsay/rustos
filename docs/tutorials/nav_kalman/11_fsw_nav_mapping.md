---
document_type: Tutorial Chapter 11 — Mapping the Math to FT1's nav_lib
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_index: docs/tutorials/nav_kalman/index.md
covers: SW-REQ-NAV-018, SW-REQ-NAV-019, SW-REQ-NAV-020
prerequisites: Chapters 01-10
---

# Chapter 11 — Mapping the Math to FT1's `nav_lib`

This chapter bridges the math (chapters 01-10) to the C++ you will write.
We walk `docs/design/nav/algorithm.md` end-to-end with line citations,
binding every math symbol to the concrete C++ type or kmat operation. After
this chapter, every line of `algorithm.md` should read like pseudocode.

> **Three windows open:** (1) `docs/design/nav/algorithm.md` (the spec);
> (2) `docs/design/nav/design.md` (vtable + types); (3)
> `libs/kmat_lib/include/kmat/kmat_api.hpp` (the C++ types). "algorithm.md:113-117"
> means lines 113-117 of `algorithm.md`.

---

## 1. The 16-State Vector

The public state vector is pinned at 16 components in algorithm.md §3.1
lines 70-82:

| Block | Component | Dim | Units / Frame | Math symbol |
|-------|-----------|----:|---------------|-------------|
| Position | `tPosLla` (lat, lon, alt) | 3 | deg, deg, m HAE WGS-84 | $\mathbf{p}^{LLA}$ |
| Velocity | `tVelNed` (Vn, Ve, Vd)   | 3 | m/s, NED        | $\mathbf{v}^{NED}$ |
| Attitude | `tAttQuat` (w, x, y, z)  | 4 | unit quaternion, body→NED, Hamilton, scalar-first | $\mathbf{q}_{b}^{n}$ |
| Accel bias | `tAccelBias`           | 3 | m/s², body      | $\mathbf{b}_a$ |
| Gyro bias  | `tGyroBias`            | 3 | rad/s, body     | $\mathbf{b}_g$ |
| **Total**  |                        | **16** | | $\mathbf{x} \in \mathbb{R}^{16}$ |

These 16 numbers ARE the state vector $\mathbf{x}$ from chapter 03 and
chapter 04. They are jointly Gaussian (chapter 02):
$\mathbf{x} \sim \mathcal{N}(\hat{\mathbf{x}},\,\mathbf{P})$. The mean is
what `NavLib_GetState` returns; covariance $\mathbf{P}$ is the 16×16 the
filter maintains internally.

**Triple-stated convention (do not paraphrase):** the quaternion is
**Hamilton convention** (not JPL), stored **scalar-first** (`w, x, y, z`),
and rotates **body→NED**. Chapter 08 walked through both conventions; FT1
is fixed.

**Public 16 vs internal 15.** algorithm.md:84-96 explicitly permits the
IMPL to use either a **full-state EKF** with `kInternalDim = 16` (carrying
the unit quaternion in the state, re-normalized after each predict per
algorithm.md:92), or an **error-state EKF** with `kInternalDim = 15` (a
3-vector small-rotation parameterization injected onto the nominal
quaternion after each step, algorithm.md:94). Either yields the same 16
public outputs (algorithm.md:96-97). **Recommendation for first
implementation: full-state EKF, `kInternalDim = 16`** — avoids extra
nominal-vs-error book-keeping and is simpler to debug against an analytic
reference.

---

## 2. The Covariance Matrix $\mathbf{P}$

algorithm.md lines 175-176 pin the storage:

```cpp
juno::kmat::MAT_T<double, kInternalDim, kInternalDim> tCovariance;
```

A 16×16 (or 15×15) symmetric positive-definite matrix — exactly the
$\mathbf{P}$ from chapter 04, preserved as long as §8 of this chapter is
followed.

**Initial $\mathbf{P}_0$ is diagonal**, with 16 entries supplied by the
caller in `NAV_INIT_T.fInitialCovDiag[16]` per algorithm.md §5.1 line 297.
Indexing matches `NAV_STATE_T` order: `[0..2]` `tPosLla`, `[3..5]`
`tVelNed`, `[6..9]` `tAttQuat`, `[10..12]` `tAccelBias`, `[13..15]`
`tGyroBias`. Small entries (e.g., 1e-6) lock the filter near the seed; large
(e.g., 1e+2) accept the first measurement aggressively. **SW-TC-NAV-023**
verifies this plumbing end-to-end (test_cases.json:421-438). Build
$\mathbf{P}_0 = \mathrm{diag}(\text{fInitialCovDiag})$ in `NavLib_Init` by
zeroing `tCovariance` then writing 16 diagonal entries.

---

## 3. The Process Noise $\mathbf{Q}$

algorithm.md lines 286-296 list the noise sigmas in `NAV_INIT_T`:
`fImuAccelNoiseSigmaMps2[3]`, `fImuGyroNoiseSigmaRps[3]`,
`fImuAccelBiasRandomWalkMps2PerSqrtS[3]`,
`fImuGyroBiasRandomWalkRpsPerSqrtS[3]`. The IMPL stores these by-value at
`NavLib_Init` and squares them as needed (algorithm.md:299-301).

**Recipe (informative; exact form depends on the chosen analytic Jacobian
— see Groves [1, §14.2.4]):**

| Block | Diagonal entries |
|-------|------------------|
| Position   | ≈ 0 (enters position only via velocity integration) |
| Velocity   | $\sigma_\text{accel}^2 \, \Delta t$ |
| Quaternion | $\sigma_\text{gyro}^2 \, \Delta t$ |
| Accel bias | $\sigma_\text{accel-rw}^2 \, \Delta t$ |
| Gyro bias  | $\sigma_\text{gyro-rw}^2 \, \Delta t$ |

Bias blocks are random walks (chapter 03): estimate stays put, covariance
grows linearly in $\Delta t$.

In code, $\mathbf{Q}$ is a `juno::kmat::MAT_T<double, kInternalDim, kInternalDim>`
member (`tProcNoise` per design.md:559), built in `NavLib_Init` from squared
sigmas and the canonical $\Delta t = 5\,\mathrm{ms}$ (algorithm.md:106-107).
If you discretize per-tick from `tSample.tTimestampUs`, rebuild $\mathbf{Q}$
each step.

---

## 4. The Process Model in Code

This is the heart of prediction. algorithm.md §3.2 lines 113-174 gives an
8-step recipe (steps 1-7 at 113-144, step 8 covariance propagation at 145-174); we walk each step.

**Step 1 — Bias correction (algorithm.md:115-117).**
```
a_meas_body = tSample.tAccelBodyMps2 - tAccelBias
omega_body  = tSample.tGyroBodyRps  - tGyroBias
```
Two `juno::kmat::Sub` calls (or three scalar subtractions per axis).

**Step 2 — Rotate accel to NED (algorithm.md:118-120).**
```
a_ned_meas = juno::kmat::QuatRotate(q, a_meas_body);
```
The attitude quaternion `q` rotates a body-frame vector into NED (kmat §4.6).
Chapter 08 derived the operation; chapter 09 explained why we need accel in
the inertial frame to integrate velocity in NED.

**Step 3 — Subtract gravity (algorithm.md:121-124).**
```
a_ned = a_ned_meas - g_ned;     // g_ned = (0, 0, +9.80665) m/s^2
```
Down is positive in NED, so gravity has positive $z$. The constant 9.80665
is the only `constexpr` numeric in the algorithm path
(algorithm.md:333-334).

**Step 4 — Velocity integration (algorithm.md:125-127).**
```
tVelNed_new = tVelNed_old + a_ned * dt;
```
Forward Euler at 200 Hz. Higher-order is an IMPL choice that does not affect
the API.

**Step 5 — Position integration via geodetic update (algorithm.md:128-135).**
Compute `dpos_ned = tVelNed_old * dt + 0.5 * a_ned * dt^2`, convert to
geodetic deltas (lat, lon, alt) using local meridional and prime-vertical
radii of curvature. Closed-form radii in Groves [1, §2.5]. Chapter 07
explained why we cannot just add NED meters to lat/lon degrees directly.

**Step 6 — Quaternion propagation via small-rotation increment
(algorithm.md:136-140).**
```
dq    = quaternion(omega_body * dt);     // axis-angle, half-angle approx
q_new = juno::kmat::HamProd(q_old, dq);  // Hamilton product
q_new = juno::kmat::QuatNormalize(q_new); // see §8
```
Chapter 08 derived the small-angle increment. The body-frame increment is
post-multiplied (right-side) per `juno::kmat::HamProd` semantics.

**Step 7 — Bias evolution (algorithm.md:141-144).** Bias states have no
deterministic dynamics — zero-mean random walks. Bias *estimate* does not
change in predict; *uncertainty* grows through $\mathbf{Q}$.

**Step 8 — Covariance propagation (algorithm.md:145-174).**
$$\mathbf{P}_\text{new} = \mathbf{F}\,\mathbf{P}_\text{old}\,\mathbf{F}^T + \mathbf{Q}$$
where $\mathbf{F} = \partial \mathbf{f}/\partial \mathbf{x}$ is the
analytic state-transition Jacobian. **algorithm.md:147-173 mandates
analytic F derived from one of two normative references:**

- **Groves [1, §14.2 / §14.3.1]** — recommended primary
  (algorithm.md:152-158).
- **Trawny & Roumeliotis [4, §3.5, eqs. 147-150]** — Hamilton-convention
  quaternion-error-state Jacobian (algorithm.md:159-165). URL:
  `https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf`.

**Numerical (finite-difference) Jacobians are NOT acceptable**
(algorithm.md:167-170): step-size choice destroys bit-determinism and they
cannot meet the SW-TC-NAV-021 tolerance.

In code: `juno::kmat::MatMul`, `Transpose`, `Add` (algorithm.md:177-178).
You may pre-compute `Transpose(F)` once to save one transpose per step.

---

## 5. The Baro Measurement Model

algorithm.md §4.1 lines 192-217 specifies the baro update.
$h(\mathbf{x}) = x.\text{tPosLla}[2]$ — just the altitude — is **linear in
the state**, so the EKF reduces to the linear KF (no Jacobian; chapter 04
applies directly).

Symbols: $\mathbf{H} = [\,0\;0\;1\;0\;\cdots\;0\,]$, shape
$1 \times \text{kInternalDim}$ (algorithm.md:199-200); $R = \sigma_\text{baro}^2$
from `NAV_INIT_T.fBaroNoiseSigmaM`² (algorithm.md:201-202).

Update (algorithm.md:204-213) — exactly the chapter-04 linear KF:
innovation $y = z_\text{baro} - x.\text{tPosLla}[2]$; innovation covariance
$S = \mathbf{H}\,\mathbf{P}\,\mathbf{H}^T + R$ (scalar — just $P_{2,2} + R$);
gain $\mathbf{K} = \mathbf{P}\,\mathbf{H}^T / S$ (column vector); state
update $\hat{\mathbf{x}}_\text{new} = \hat{\mathbf{x}}_\text{old} + \mathbf{K}\,y$;
Joseph-form posterior
$\mathbf{P}_\text{new} = (\mathbf{I} - \mathbf{K}\mathbf{H})\,\mathbf{P}_\text{old}\,(\mathbf{I} - \mathbf{K}\mathbf{H})^T + \mathbf{K}\,R\,\mathbf{K}^T$
(algorithm.md:211-213; per `SW-REQ-NAV-015`).

In code: `juno::kmat::MatMul`, `Add`, `Sub`, `Mult`, `Transpose` (kmat REV B; algorithm.md still uses legacy `Scale` narratively). No matrix
inversion (S is scalar); guard `S > kPivotEpsilon<double>()` per
algorithm.md:381-384 before dividing.

---

## 6. The GPS Measurement Model

algorithm.md §4.2 lines 219-238 specifies the GPS update. The measurement
is **6-dimensional** (lat, lon, alt, Vn, Ve, Vd). Like baro, $h$ is linear
— $\mathbf{H}$ has 1.0 entries on position and velocity rows, zeros
elsewhere (algorithm.md:226-228); shape $6 \times \text{kInternalDim}$.

$\mathbf{R}_\text{gps}$ is $6 \times 6$ diagonal (algorithm.md:228-233):
[0,0]/[1,1] from `fGpsHorizNoiseSigmaM`² (after meter conversion via local
WGS-84 radii); [2,2] from `fGpsVertNoiseSigmaM`²; [3,3]/[4,4]/[5,5] from
`fGpsVelNoiseSigmaMps`².

Update sequence matches baro. Two differences:

- **$\mathbf{S}$ is $6 \times 6$**, inverted via
  `juno::kmat::Invert<double, 6>` (algorithm.md:235-236; kmat §4.2.6). On
  near-zero pivot, `Invert` returns
  `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR`; escalate to
  `juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR` and reject the update
  (algorithm.md:377-381).
- **Gain $\mathbf{K}$ is $\text{kInternalDim} \times 6$**, not a column.

This is the standard tight-coupling GPS-aided INS pattern in Farrell
[6, §10] — GPS directly observes pos+vel; the EKF jointly corrects all 16
states (including biases, via $\mathbf{P}$ off-diagonals).

---

## 7. The GPS Divergence Bound

algorithm.md §4.3 lines 240-254 specifies a **measurement-rejection gate**.
Before applying GPS update, compute horizontal innovation distance
`innov_horiz_m = great_circle_distance((x.tPosLla[0..1]), (z_gps.dLatDeg, z_gps.dLonDeg))`.
If `innov_horiz_m > tInit.fGpsBoundM` (default 200 m, design.md:513):
return `juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR`
(algorithm.md:249-250); transition to `Diverged`; set `bValid = false`.
**Do not apply the update** — neither state nor covariance is altered
(algorithm.md:252-253).

This gate is **FT1-specific** — chapter 04's textbook KF has no measurement
rejection. It is a safety net against GPS spoofing or multipath: a large
innovation more often signals a bad fix than a bad estimate. Trade-off: a
genuinely diverged filter past the bound gets locked out until `NavLib_Init`
is re-called; acceptable because divergence is operator-visible
(`bValid=false`) and recoverable on the ground.

---

## 8. Numerical Stability Requirements

algorithm.md §6 lines 338-392 enumerates four rules; chapter 04 §6/§7 and
chapter 06 explained why each is needed.

1. **Quaternion renormalization (algorithm.md:343-350).**
   `juno::kmat::QuatNormalize(q_new)` after every prediction. Unit-norm
   drifts under Euler propagation. If `|q| < kPivotEpsilon<double>()`,
   `QuatNormalize` returns `JUNO_FSW_STATUS_NUMERIC_ERROR`; escalate to
   `Diverged`.
2. **Joseph form for covariance update (algorithm.md:352-357).** Simple
   form $(\mathbf{I}-\mathbf{K}\mathbf{H})\mathbf{P}$ is faster but loses
   symmetry under round-off; Joseph is symmetric by construction at ~2× cost.
3. **Symmetry enforcement (algorithm.md:359-363).**
   $\mathbf{P}_\text{new} = \frac{1}{2}(\mathbf{P}_\text{new} + \mathbf{P}_\text{new}^T)$
   after each update (one `Transpose`, one `Add`, one `Mult`).
4. **Pivot guarding for matrix inversion (algorithm.md:377-384).**
   `juno::kmat::Invert` for the 6×6 GPS innovation covariance returns
   `JUNO_FSW_STATUS_NUMERIC_ERROR` on near-zero pivot; escalate to
   `Diverged`. The 1×1 baro covariance is scalar; check against
   `kPivotEpsilon<double>()` before dividing.

---

## 9. The State Machine

contracts.md §5 (alignment criteria) and contracts.md §9 (divergence)
define `nav_lib`'s internal state machine. (algorithm.md §8 cross-references `nav_app`'s phase-aware gating but does NOT define `nav_lib`'s state machine — that lives in contracts.md §5; note: §5-§11 of the original design.md were relocated to contracts.md per NAV-A3 split, 2026-05-08):

- **Aligning.** Filter accumulates IMU + baro + GPS; alignment gate fires
  when ≥1 valid GPS fix consumed AND ≥50 IMU samples consumed AND (if
  `bUseBaroAlt`) ≥1 valid baro sample consumed (design.md:355-361).
- **Aligned.** Nominal operation; `bValid = true` (some texts call this "navigating," but `contracts.md §5` uses `Aligned`).
- **Diverged.** GPS bound exceeded, numerical error, or dead-reckoning
  budget exceeded (design.md:504-509). `bValid = false`. Recovery via
  `NavLib_Init` re-call.

`nav_lib` does **no phase-aware gating**. BOOST/COAST/DESCENT/LANDED phase
logic — including skipping `UpdateBaro`/`UpdateGps` during BOOST + a
1-second post-boost settling window — lives in `nav_app` (algorithm.md:32-40,
§8). From `nav_lib`'s perspective, `PredictImu` is always called and
`Update*` calls arrive exactly when `nav_app` issues them.

---

## 10. The C++ Types You Will Touch

| C++ symbol | Role | Source |
|------------|------|--------|
| `juno::nav::NAV_LIB_ROOT_T` | Caller-owned root with vtable | design.md:182, 193-197 |
| `juno::nav::NAV_LIB_IMPL_T` | IMPL-private derived; embeds `tStateVec`, `tCovariance`, `tProcNoise`, `tLatestGps` | design.md:556-572 |
| `juno::nav::NAV_LIB_API_T`  | Vtable: `Init`, `PredictImu`, `UpdateBaro`, `UpdateGps`, `GetState` | design.md:184-191 |
| `juno::nav::NAV_INIT_T`     | Caller-supplied init: seed, gps bound, noise sigmas, $\mathbf{P}_0$ diagonal | design.md:122-154 |
| `juno::nav::NAV_STATE_T`    | Public 16-state output | design.md:111-120 |
| `juno::nav::IMU_SAMPLE_T`   | One IMU reading: 3 accel (m/s²), 3 gyro (rad/s), timestamp, valid | design.md:107 |
| `juno::nav::BARO_SAMPLE_T`  | One baro reading: pressure, alt HAE, temp, timestamp, valid | design.md:108 |
| `juno::nav::GPS_FIX_T`      | One GPS fix: lat, lon, alt, NED velocity, timestamp, valid | design.md:109 |

Math storage and operations from `juno::kmat` (algorithm.md:177-180):

- **Storage.** `juno::kmat::MAT_T<double, N, N>`,
  `juno::kmat::VEC<double, N>`, `juno::kmat::QUAT<double>` (kmat REV B; the `_T` aliases `VEC_T`/`QUAT_T` were removed per kmat §4.1 / §4.6.1, though algorithm.md and design.md still use the legacy names narratively). All
  caller-owned, compile-time-fixed, no dynamic allocation
  (`SW-REQ-KMAT-001`/`-008`).
- **Operations.** `juno::kmat::MatMul`, `Add`, `Sub`, `Transpose`,
  `Invert`, `Mult`, `HamProd`, `QuatRotate`, `QuatNormalize` — nine
  operations (kmat REV B; algorithm.md still uses legacy `Scale`/`QuatMul`). All deterministic by `SW-REQ-KMAT-009`/`-010`, which gives
  the EKF its bit-equivalence between POSIX and Pico2
  (`SW-REQ-NAV-015`/`-016`).

**No dynamic allocation. No exceptions. No virtual.** Every API function
is `noexcept` (design.md:199); freestanding-C++11 per
`ai/memory/coding-standards.md` and `ai/memory/constraints.md`. No
constructors/destructors on `NAV_LIB_ROOT_T`/`NAV_LIB_IMPL_T`
(design.md:542). All IMPL data lives inside the embedded `NAV_LIB_IMPL_T`
in caller-owned `.bss`.

---

## 11. Test Cases

`docs/test_cases/nav/test_cases.json` enumerates 23 unit tests
(`SW-TC-NAV-001..023`); the IMPL must pass all before G1 closes. Highlights:

- `SW-TC-NAV-001..003` — accept IMU / GPS / baro, confirm estimate updates
  (test_cases.json:3-54).
- `SW-TC-NAV-004` — 16-state structure inspection (test_cases.json:55-72).
- `SW-TC-NAV-005` — 100 Hz cadence: 100 distinct estimates over one
  simulated second (test_cases.json:73-89).
- `SW-TC-NAV-021` — **EKF verified against an analytic ground-truth
  reference**, generated by an off-line Python NumPy reference EKF with
  `numpy.random.default_rng` seeded from a fixed seed and `numpy.float64`
  precision (algorithm.md:386-391). The tolerance dictates "analytic
  Jacobian, not numerical" (algorithm.md:167-173).
- `SW-TC-NAV-022` — **load-time configurability of noise covariance via
  `NAV_INIT_T`**: filters with different sigmas produce different outputs
  on identical inputs (test_cases.json:403-419).
- `SW-TC-NAV-023` — **initial-covariance plumbing for $\mathbf{P}_0$**:
  high $\mathbf{P}_0$ snaps to GPS; low $\mathbf{P}_0$ stays near seed
  (test_cases.json:421-438).

The SW-TC-NAV-021 tolerance separates "EKF works" from "EKF approximately
works." Do not relax it to mask a half-correct Jacobian.

---

## 12. Implementation Roadmap (suggestion, not pinned)

A concrete sequence — none pinned by spec; this minimizes debugging time
for a first-time EKF implementer.

**A — Read.** Read `algorithm.md` and `design.md` end-to-end. Re-read
until every line is clear. If a line is opaque, the gap is in your math —
go back to the relevant chapter.

**B — Derive $\mathbf{F}$ by hand.** Sit with Groves [1, §14.2] OR Trawny &
Roumeliotis [4, §3.5] and write the analytic Jacobian on paper, block by
block. Cross-validate against an off-line Python NumPy reference (small
input, propagate, compute $\mathbf{F}$ via finite differences, compare to
your analytic). The reference EKF for SW-TC-NAV-021 shall use the same
normative reference as your IMPL (algorithm.md:170-173).

**C — Implement `NavLib_Init`.** Build $\mathbf{P}_0$ from
`fInitialCovDiag`, $\mathbf{Q}$ from noise sigmas, $\mathbf{R}_\text{baro}$
and $\mathbf{R}_\text{gps}$ from their sigmas. SW-TC-NAV-022/-023 are the
bar.

**D — Implement `PredictImu` steps 1-7.** No covariance propagation yet.
Test against the Python reference for one IMU sample, then ten samples.

**E — Add covariance propagation (step 8).** Use the analytic $\mathbf{F}$
from B. Test against the Python reference for the covariance trace and
diagonal entries. Apply symmetry enforcement from §8.

**F — Implement `UpdateBaro`.** Test innovation, Kalman gain, posterior —
each individually verifiable against the Python reference.

**G — Implement `UpdateGps` + divergence-bound check.** Test all three
branches: accept, reject (innovation > bound), numerical-error (force
near-singular $\mathbf{S}$ via degenerate $\mathbf{R}$).

**H — Run all `SW-TC-NAV-*` unit tests.** Iterate until all 23 pass.
Hardest are SW-TC-NAV-021 (analytic-Jacobian tolerance) and SW-TC-NAV-005
(100 Hz cadence — easy in test, but reveals timestamp bugs).

**I — Run G2 traceability + G3 Pico2 cross-compile.** `tools/traceability.py`
exit 0; `cmake -DPLATFORM=PICO2 .. && cmake --build .` exit 0. Sprint is
closeable per `docs/sdp/index.md` §9.

---

## Key Results

- **16-state public output** → `juno::nav::NAV_STATE_T`; IMPL may use 16 or
  15 internally; 16 (full-state) recommended for first implementation.
- $\mathbf{P}$ is `juno::kmat::MAT_T<double, kInternalDim, kInternalDim>`;
  $\mathbf{P}_0$ diagonal from `NAV_INIT_T.fInitialCovDiag[16]`.
- $\mathbf{Q}$, $\mathbf{R}_\text{baro}$, $\mathbf{R}_\text{gps}$ all built
  in `NavLib_Init` from `NAV_INIT_T` sigmas.
- 8-step process model maps 1:1 to `juno::kmat` calls; covariance uses
  **analytic $\mathbf{F}$** per Groves [1, §14.2] or Trawny & Roumeliotis
  [4, §3.5].
- Baro = scalar linear measurement; GPS = 6-D linear measurement with
  divergence-bound rejection gate.
- Numerical stability: quaternion renormalize, Joseph form, symmetry
  enforcement, pivot guarding.
- 23 unit tests are the pass bar; SW-TC-NAV-021/-022/-023 are the
  "did the math actually plug in" tests.

## Citations

- [1, §2.5] Groves — meridional/prime-vertical radii for geodetic update.
- [1, §14.2 / §14.3.1] Groves — analytic $\mathbf{F}$ Jacobian (primary).
- [1, §14.2.4] Groves — discrete-time process-noise $\mathbf{Q}$ recipe.
- [4, §3.5, eqs. 147-150] Trawny & Roumeliotis — Hamilton-convention
  quaternion-error-state Jacobian (alternative).
- [6, §10] Farrell — GPS-aided INS / tight-coupling pattern.

---

<!-- @{"design": ["SW-REQ-NAV-018", "SW-REQ-NAV-019", "SW-REQ-NAV-020"]} -->
