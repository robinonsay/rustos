---
document_type: Tutorial Chapter — Sensor Fusion Intuition
program: Juno FT1 FSW
chapter: 10 of 12
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_index: docs/tutorials/nav_kalman/index.md
prerequisites: chapters 01–09
covers_anchor_reqs: SW-REQ-NAV-018, SW-REQ-NAV-019
---

# Chapter 10 — Sensor Fusion: Why and How

> **Bridge chapter.** Chapters 01–06 built the math (linear algebra,
> probability, KF, EKF); chapters 07–09 built the navigation (frames,
> attitude, strapdown). This chapter answers **why** we fuse
> IMU + baro + GPS, and **how** the EKF from chapter 06 automates
> the optimal blend. Chapter 11 maps every symbol to a C++ symbol
> the implementation will instantiate.

Less mathematical than chapters 04 and 06: the math is already done.
What remains is engineering judgment — which sensors are fused, when
their measurements are admitted, and how their relative trust is
encoded in the covariance loaded at `NavLib_Init`.

---

## 10.1 The Fundamental Problem

A single sensor cannot provide a complete, accurate, *and* reliable
navigation solution. Each candidate has a structural gap that no
amount of better hardware in that one box can close: frequency too
low (cannot resolve sub-second boost dynamics), drift over time
(unbounded position error on long flights), dropouts under stress
(loss of fix during the very phase you most need it), per-axis
blindness (one axis well, another poorly), or environmental coupling
(weather or engine plume corrupting the reading).

Sensor fusion combines multiple sensors so that the **strengths of
each cover the weaknesses of the others.** The IMU's high rate covers
the GPS's low rate; the GPS's absolute reference covers the IMU's
drift; the barometer's vertical accuracy covers the GPS's vertical
noise. None of the three sensors alone produces what FT1 needs; the
three together do [6, ch. 1] [1, §1.4].

---

## 10.2 The IMU (`imu_lib`)

**Output.** A 6-axis sample at **200 Hz nominal** per
`docs/design/nav/algorithm.md` line 105 — 3-axis specific force
(m/s², body) and 3-axis angular rate (rad/s, body). The MPU-6050 on
the Pico 2 board produces this over I²C; `imu_lib` exposes it as
`JUNO_MSG_IMU_SAMPLE_T` with `tAccelBodyMps2`, `tGyroBodyRps`, and
`tTimestampUs`.

**Strengths.** Very high sample rate (200 Hz captures sub-5-ms
dynamics — engine vibration, attitude maneuvers, separation); low
short-term noise (MPU-6050 datasheet at `algorithm.md` line 321);
no external dependencies (works inside fairings, under canopy, during
ionospheric scintillation, satellite blackout); no dropouts while
power is up.

**Weaknesses.** Bias drift causes unbounded error growth — chapter 09
derived $\frac{1}{2}b t^2$ position error from a constant accel bias
$b$, so even 0.01 m/s² becomes 18 m over 60 s. **Pure-IMU
dead-reckoning is useless beyond ~1 minute** [1, §6.5] [6, ch. 5].
Gravity subtraction requires attitude — a small attitude error
projects gravity into a horizontal velocity error.

The IMU is the **fastest** and most **available** sensor, but by
itself answers no navigation question lasting longer than ~1 minute.

---

## 10.3 The Barometer (`baro_lib`)

**Output.** A single scalar — altitude in meters, **HAE (height above
the WGS-84 ellipsoid)** per `docs/design/conventions.md` §4 (inherited
verbatim by `algorithm.md` §3.1 line 78). The MPL3115A2 produces this
at ~10 Hz (per the MPL3115A2 datasheet; algorithm.md §5.2 only pins σ values, not sample rates).

> **HAE, not MSL.** The barometer's published altitude is reduced to
> WGS-84 ellipsoidal height by `baro_lib` so it is directly comparable
> to GPS altitude. The EKF innovation
> $\mathbf{y} = z_{\text{baro}} - \hat{x}_{\text{alt}}$ in §10.6 is
> meaningful only if measurement and state share a vertical datum.

**Strengths.** Direct altitude measurement (no integration, no drift
over time); vertically more accurate than GPS (~1.5 m σ at
`algorithm.md` line 325 vs ~5 m σ GPS vertical at line 327); always
available; low cost, low power.

**Weaknesses.** Single-axis (vertical only); atmospheric variability
(a weather front during flight changes apparent altitude by meters);
vehicle-induced disturbances — slipstream perturbs static pressure
at speed and rocket-engine plume effects perturb the static port
during boost. Plume and slipstream errors are **not Gaussian noise**;
they are systematic transient errors that violate the EKF modeling
assumption, motivating the boost-phase gating in §10.7. ~10 Hz
cannot resolve apogee timing to better than 100 ms.

The barometer is the **vertical specialist** — much better than GPS
for altitude in nominal conditions, much worse for everything else.

---

## 10.4 The GPS (`gps_lib`)

**Output.** A geodetic fix — latitude, longitude, altitude (m HAE) —
**and NED velocity** (Vn, Ve, Vd, m/s) — at **1 Hz** for the
GlobalTop FGPMMOPA6H module specified in
`ai/memory/project-overview.md`. The receiver tracks satellite
pseudoranges *and* Doppler shifts, producing both position and
velocity from the same RF measurement set. **`gps_lib` publishes
both**; consumers that ignore velocity leave half the GPS
information on the table [6, §6.4] [1, §9.3].

**Strengths.** Absolute, globally bounded position (error bounded by
satellite geometry and URA, regardless of flight duration); velocity
independently measured from carrier Doppler — ~0.1 m/s σ at
`algorithm.md` line 328 vs the ~3 m/s implied σ if velocity were
differentiated from 2.5 m σ position fixes at 1 Hz [6, §6.4]. **The
EKF GPS update fuses both** (§10.6, `algorithm.md` §4.2 lines
219–238). No long-term drift.

**Weaknesses.** Low rate (1 Hz cannot drive a 100 Hz estimator on
its own — the IMU must propagate state between fixes). Dropouts
under canopy, ionospheric scintillation, high-G maneuvers (receiver
loses lock), or jamming. Transient error spikes during boost — high
vibration plus fast attitude changes produce multipath and tracking
errors that the receiver cannot detect or flag, so a fix may report
50 m position error with no health bit set [6, §7.5] [1, §9.3.4];
second motivation for the phase-aware gating in §10.7. Noise sigmas
(`algorithm.md` lines 326–328): horizontal ~2.5 m, vertical ~5.0 m,
velocity ~0.1 m/s — **vertical accuracy is half as good as
horizontal**, making the baro's vertical contribution especially
valuable.

GPS is the **absolute-reference sensor** — the only onboard sensor
whose position error is bounded over arbitrary time, and the only
one that gives a true velocity. But it is slow, can drop out, and
emits unflagged transient errors during the phases of greatest
dynamic stress.

---

## 10.5 Complementarity — Why the Three Combine Well

The three sensors have **complementary error spectra.** The errors
that bother one are the very errors the others are best at correcting.

**IMU drift ↔ GPS absolute reference.** IMU bias error grows as
$\sim t^2$ in position; GPS error is bounded for all $t$. Fusing:
the IMU provides the high-rate trajectory between GPS fixes; the GPS
resets the absolute reference once per second; the EKF estimates the
IMU bias states (the `tAccelBias` and `tGyroBias` triples in
`algorithm.md` §3.1 lines 80–82) so dead-reckon is performed with a
bias-corrected IMU. This is the canonical GPS+INS architecture
[6, ch. 7] [1, §14.3].

**GPS vertical noise ↔ baro vertical accuracy.** GPS vertical σ is
twice its horizontal (`algorithm.md` lines 326–327: ~5 m vs ~2.5 m);
baro is much better than 5 m σ in nominal conditions (~1.5 m σ,
line 325). Fusing yields a vertical estimate better than either
sensor alone — the EKF weights baro more heavily than GPS in
altitude automatically (§10.6).

**GPS dropouts ↔ IMU continuity.** During boost, attitude maneuvers,
or canopy occlusion the GPS may stop providing fixes; the IMU is
unaffected. The EKF continues to dead-reckon via PredictImu; position
covariance grows; when GPS returns, the next update applies a large
innovation that pulls the state back to GPS-bounded truth. This is
the **degraded-input continuation** behavior pinned by
`SW-REQ-NAV-013`.

The fused state is therefore high-rate (200 Hz), bounded in absolute
error (GPS pulls it back at 1 Hz), vertically accurate (baro beats
GPS alone), continuous through dropouts, and self-calibrating (bias
states estimated from GPS+baro innovations). No single sensor
delivers any one of these; the three together deliver all five.

---

## 10.6 How the EKF Automates the Fusion

This is the central insight. **The EKF math from chapters 04 and 06
*is* the optimal linear-Gaussian fusion algorithm.** The PM does not
write a fusion algorithm; the EKF *is* the fusion algorithm. For each
sensor the EKF requires three things, all already specified in the
FT1 design:

1. **Measurement model $\mathbf{h}(\hat{\mathbf{x}})$** — what the
   sensor would read if the state estimate were truth. Baro:
   $\mathbf{h} = \hat{x}_{\text{alt}}$ (`algorithm.md` §4.1
   line 198). GPS: 6-vector of position (3) and NED velocity (3)
   (`algorithm.md` §4.2 lines 224–226).
2. **Measurement Jacobian $\mathbf{H}$** — linearization of
   $\mathbf{h}$. Baro: $1 \times 16$ row with a single 1 in the
   altitude column (`algorithm.md` §4.1 line 199). GPS: $6 \times 16$
   matrix with 1s in the position and velocity columns
   (`algorithm.md` §4.2 line 227).
3. **Measurement-noise covariance $\mathbf{R}$** — how much we trust
   this sensor. Baro: $\sigma_{\text{baro}}^2$ scalar
   (`algorithm.md` §4.1 line 201). GPS: $6 \times 6$ diagonal from
   `NAV_INIT_T.fGpsHorizNoiseSigmaM`, `fGpsVertNoiseSigmaM`,
   `fGpsVelNoiseSigmaMps` (`algorithm.md` §4.2 lines 229–233 and §5.1
   lines 286–296).

Given those three, the EKF update step (chapter 06) is:

$$
\mathbf{S} = \mathbf{H}\,\mathbf{P}\,\mathbf{H}^T + \mathbf{R}
\qquad
\mathbf{K} = \mathbf{P}\,\mathbf{H}^T\,\mathbf{S}^{-1}
\qquad
\hat{\mathbf{x}}^{+} = \hat{\mathbf{x}}^{-} + \mathbf{K}\,
\bigl(\mathbf{z} - \mathbf{h}(\hat{\mathbf{x}}^{-})\bigr)
$$

with the Joseph-form covariance update from `algorithm.md` §4.1
line 213.

**The Kalman gain $\mathbf{K}$ is the automatic optimal blender.**
$\mathbf{R}$ is in the denominator (via $\mathbf{S}^{-1}$): large
$\mathbf{R}$ (noisy sensor) → small $\mathbf{K}$ → state moves little
per measurement (the filter ignores a sensor it does not trust).
Small $\mathbf{R}$ (precise sensor) → large $\mathbf{K}$ → state
moves a lot per measurement. Likewise $\mathbf{P}$ is in the
numerator: when prior uncertainty is large, the filter is willing to
be moved; when prior confidence is high, a noisy measurement does
not shake it. This is exactly what an engineer would want from a
hand-coded blender — and the EKF math produces it without any
blending rule being specified directly [2, §5.2] [1, §3.4].

The GPS-vs-baro vertical weighting from §10.5 falls out automatically:
with $\sigma_{\text{baro}} = 1.5$ m and $\sigma_{\text{gps,vert}} = 5.0$ m
the $\mathbf{R}$ diagonal entries differ by $(5/1.5)^2 \approx 11$,
so the baro-update gain on altitude is about 11× the GPS-altitude
gain (other factors equal).

> **The PM does not write a "fusion algorithm" by hand.** The
> measurement model $\mathbf{H}$, the noise $\mathbf{R}$, and the EKF
> update equations together *are* the fusion algorithm. The
> implementation work is to encode $\mathbf{H}$ and $\mathbf{R}$
> correctly and to invoke the standard EKF update — no bespoke
> blending logic.

---

## 10.7 The FT1 Fusion Architecture

The FT1 implementation slots the EKF into a three-call API
(`docs/design/nav/design.md` and `algorithm.md` §3.2 / §4):

- `NavLib_PredictImu` — 200 Hz (every IMU sample). Strapdown
  propagation + covariance forward step (`algorithm.md` §3.2
  lines 102–187).
- `NavLib_UpdateBaro` — up to ~10 Hz (when `nav_app` invokes). Baro
  measurement update, 1-D (`algorithm.md` §4.1 lines 192–217).
- `NavLib_UpdateGps` — up to 1 Hz (when `nav_app` invokes). GPS
  update, 6-D, with divergence-bound rejection (`algorithm.md` §4.2
  lines 219–238 and §4.3 lines 240–254).

A critical design property pinned in `algorithm.md` §4.4 line 258:

> "`NavLib_UpdateBaro` and `NavLib_UpdateGps` accept measurements
> **unconditionally** when invoked (subject only to the precondition
> checks ... and the divergence bound in §4.3 above)."

In other words, **`nav_lib` itself has no flight-phase logic.** It
does not know what BOOST means, does not subscribe to the AFM phase
message, and does not track the flight clock. From `nav_lib`'s
perspective PredictImu is called every IMU sample and Update* calls
happen whenever the caller chooses. `algorithm.md` §8 lines 422–434
restates: during BOOST `nav_app` calls only PredictImu and the EKF
dead-reckons via the strapdown loop (lines 426–428); during the
1-second post-boost settling window predict-only continues
(lines 429–431); during COAST/DESCENT/LANDED `nav_app` resumes
UpdateBaro and UpdateGps (lines 432–434).

**Phase-aware gating lives in `nav_app`, not `nav_lib`.** This
separates the EKF math (generic, reusable) from the FT1-specific
decision about *when* a measurement is trustworthy enough to fuse
(may be re-tuned for FT2). Chapter 11 shows the C++ layout follows
the same separation: `nav_lib` exposes three EKF entry points;
`nav_app` chooses which to call this tick based on the latest
`JUNO_MSG_AFM_PHASE_T` message.

---

## 10.8 Tuning Intuition — How $\mathbf{Q}$ and $\mathbf{R}$ Shape the Filter

The EKF math is fixed. The filter's behavior is shaped by the
covariance values the caller supplies through `NAV_INIT_T` per
`algorithm.md` §5.1 lines 286–297.

**Process noise $\mathbf{Q}$ — how much IMU prediction is trusted.**
Built from `fImuAccelNoiseSigmaMps2[3]`, `fImuGyroNoiseSigmaRps[3]`,
`fImuAccelBiasRandomWalkMps2PerSqrtS[3]`, and
`fImuGyroBiasRandomWalkRpsPerSqrtS[3]`. Larger $\mathbf{Q}$ →
predicted covariance grows faster between updates → larger Kalman
gain when an update arrives → state moves more per update.

**Measurement noise $\mathbf{R}$ — how much each sensor is trusted.**
Built from `fBaroNoiseSigmaM`, `fGpsHorizNoiseSigmaM`,
`fGpsVertNoiseSigmaM`, `fGpsVelNoiseSigmaMps`. Larger σ → smaller
gain → filter discounts that sensor.

The reference values in `algorithm.md` §5.2 lines 319–328 are
**installation guidance only** — every value is load-time
configurable per `SW-REQ-NAV-019`. They are derived from datasheet
specs (MPU-6050, MPL3115A2) and nominal GlobalTop FGPMMOPA6H
performance, not from FT1 flight data; FT2 may re-tune from logged
FT1 data per the explicit note at `algorithm.md` lines 330–334.
**The IMPL must not hardcode any of these σ values** (line 333) —
they live in the caller's `NAV_INIT_T` and the IMPL stores them by
value at `NavLib_Init`.

**Initial covariance $\mathbf{P}_0$.** The `fInitialCovDiag[16]`
field (`algorithm.md` §5.1 line 297) seeds the initial state
covariance. For FT1 the seed is approximate, so the initial
covariance is set generously to allow rapid alignment in the first
few seconds.

---

## 10.9 Why We Don't Trust Any One Sensor Alone

A short reinforcement:

- **IMU alone:** Drifts. Useful for <1 minute. Useless for FT1.
- **Baro alone:** Vertical only. No horizontal information.
- **GPS alone:** 1 Hz. No high-rate dynamics. Drops out under stress.
  Vertical noise is twice the horizontal noise.

Each sensor is **necessary but insufficient**: removing any one
would degrade the navigation solution below FT1's needs (trajectory
reconstruction, live downlinked nav state, AFM phase, recovery
basket). The EKF fuses all three optimally so the nav solution is as
accurate as the sensor-suite physics allow [6, ch. 1] [1, §1.4].

---

## 10.10 Worked Example — A Typical FT1 Trajectory

Pad to landing, phase by phase. **No numbers are computed** —
chapter 05 is the numerical-example chapter. Phase boundaries follow
`algorithm.md` §8.

**T-0, on the pad.** `nav_lib::Init` has just been called; state
machine in `Aligning`; initial covariance `fInitialCovDiag[16]` is
large. IMU reads gravity in body z; baro reads launch-elevation
pressure; GPS gives absolute position. Kalman gains are large (high
$\mathbf{P}$), so each baro and GPS update pulls the state strongly
toward the measured values; attitude stabilizes via gyro integration;
bias states begin to be estimated. After alignment converges the EKF
transitions to `Aligned` (`bValid = true`).

**T+1 s during boost.** `afm_app` has transitioned to
`JUNO_PHASE_BOOST`; `nav_app` applies the phase-aware gating from
`algorithm.md` §8 lines 422–434: **PredictImu every IMU sample
(200 Hz); UpdateBaro and UpdateGps are not called.** The EKF
dead-reckons via the §3.2 strapdown loop. Bias estimates remain at
their pre-boost values (random-walk zero-mean — no observability
without measurement updates). Position and velocity covariances grow
monotonically because $\mathbf{P}$ is propagated by
$\mathbf{F}\mathbf{P}\mathbf{F}^T + \mathbf{Q}$ with no measurement
reduction.

**T+8 s, post-boost cutoff + 1 s settling window.** `afm_app` has
transitioned to `JUNO_PHASE_COAST` ~7 s after boost cutoff.
`nav_app`'s `kNavAppBoostSettlingUs = 1 s` guard
(`nav_app/design.md` §4.4) suppresses Update* calls for one more
second so high-rate dynamics settle. At T+8 s the guard expires; the
first post-boost GPS innovation is **large** (position drifted
during 8 s of pure prediction); the EKF applies a large state
correction; covariance shrinks rapidly; bias states get estimated
against the now-large innovations. Within a few GPS cycles the
filter is again `Running` with `bValid = true`.

**T+30 s descent.** Phase `JUNO_PHASE_DESCENT`. `nav_app` admits all
three streams: PredictImu at 200 Hz, UpdateBaro at ~10 Hz, UpdateGps
at 1 Hz. The filter is converged; the published
`JUNO_MSG_NAV_STATE_T` carries `bValid = true` every tick.

The full flight follows the §10.7 architecture: PredictImu always;
Update* when `nav_app` chooses; phase-aware gating in `nav_app`,
not `nav_lib`. No "boost mode" or "coast mode" — only a generic EKF
plus phase-aware orchestration in the app layer.

---

## 10.11 Key Results

> 1. **No single sensor satisfies FT1.** IMU drifts; baro is vertical
>    only; GPS is slow and dropout-prone. Each necessary, none
>    sufficient.
> 2. **Complementary error spectra.** The errors of one sensor are
>    bounded by the strengths of the others.
> 3. **The EKF is the fusion algorithm.** The Kalman gain
>    $\mathbf{K} = \mathbf{P}\mathbf{H}^T(\mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R})^{-1}$
>    automatically weights each sensor by inverse noise covariance.
> 4. **`nav_lib` has no phase logic.** Per `algorithm.md` §4.4
>    line 258 and §8 lines 422–434, all three EKF entry points
>    accept inputs unconditionally. Phase-aware gating is a
>    `nav_app` responsibility.
> 5. **Tuning lives in `NAV_INIT_T`.** The σ values per
>    `algorithm.md` §5.1 lines 286–297 control the Kalman gain; the
>    §5.2 lines 319–328 reference values are starting points only.

---

## 10.12 Citations

- `[1, §1.4, §3.4, §6.5, §9.3, §14.3]` Groves — overall GPS+INS
  fusion architecture and sensor noise specifications.
- `[2, §5.2]` Brown & Hwang — Kalman gain as inverse-noise weighting.
- `[6, ch. 1, ch. 5, ch. 7, §6.4, §7.5]` Farrell, *Aided Navigation:
  GPS with High-Rate Sensors* — canonical GPS+INS fusion reference;
  Doppler-velocity precision; multipath transients during boost.
- `docs/design/nav/algorithm.md`: §3.1 line 78 (HAE convention);
  line 105 (IMU 200 Hz); §3.2 lines 102–187 (strapdown);
  §4.1 lines 192–217 (baro update); §4.2 lines 219–238 (GPS update,
  position + velocity); §4.3 lines 240–254 (divergence bound);
  §4.4 line 258 (unconditional admission); §5.1 lines 286–297
  (NAV_INIT_T schema); §5.2 lines 319–328 (reference σ values;
  lines 330–334 caveat); §6 lines 351–357 (Joseph form);
  §8 lines 422–434 (phase-aware gating cross-reference).

---

## 10.13 Exercises (referenced in Chapter 12)
1. **Sensor-substitution.** For each of (a) IMU alone, (b) baro alone,
   (c) GPS alone, predict the FT1 nav solution across pad-to-landing
   and identify the dominant error source.
2. **Kalman-gain inversion.** Given $\mathbf{P}_{\text{alt}} = 1\,\text{m}^2$
   and $\sigma_{\text{baro}} = 1.5\,\text{m}$, compute $\mathbf{K}$ for
   the scalar baro update; repeat with $\sigma_{\text{gps,vert}} = 5.0\,\text{m}$.
   By what factor does the baro gain exceed the GPS-altitude gain?
3. **Why no $\mathbf{u}$ vector?** Explain in two sentences why IMU
   measurements drive $\mathbf{F},\mathbf{Q}$ propagation rather than
   appearing as control inputs $\mathbf{u}$.
4. **Phase-aware gating, code-side.** Read `nav_app/design.md` §4.4
   and write (pseudocode) the predicate deciding whether UpdateBaro
   is called this tick; state why short-circuit ordering matters at
   startup (`_tBoostExitUs = 0`).

<!-- @{"design": ["SW-REQ-NAV-018", "SW-REQ-NAV-019"]} -->
