---
document_type: Tutorial Chapter 09 — Inertial Navigation and Strapdown Integration
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_index: docs/tutorials/nav_kalman/index.md
prerequisites: Chapters 07 (frames), 08 (attitude / quaternions)
covers: SW-REQ-NAV-018 (process model)
---

# Chapter 09 — Inertial Navigation and Strapdown Integration

> **Reading goal.** Describe what an IMU outputs, why "specific force"
> is not "acceleration," how the seven-step strapdown loop in
> [`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) §3.2
> turns IMU samples into pos/vel/att estimates, why pure INS drifts
> unboundedly, and why FT1's EKF carries bias states and consumes baro
> and GPS aiding. Maps to `algorithm.md` §3.2 (process model) and §3.1
> (16-state vector). Bridges chapter 08 (rotations) and chapter 10
> (fusion).

---

## 1. What an IMU Measures

An **Inertial Measurement Unit (IMU)** is a single sensor module with
two sensing elements. In FT1 the IMU is the InvenSense MPU-6050 MEMS
device:

1. **3-axis accelerometer** — reports **specific force** in body
   frame, units $\mathrm{m/s^2}$.
2. **3-axis gyroscope** — reports **angular rate** in body frame,
   units $\mathrm{rad/s}$.

Per `algorithm.md` line 113, the prediction step receives one IMU
sample with three fields:

```
tSample = { tAccelBodyMps2,   // 3-vector, body, m/s^2 (specific force)
            tGyroBodyRps,     // 3-vector, body, rad/s
            tTimestampUs }    // monotonic timestamp, microseconds
```

The body frame is rigidly attached to the IMU (chapter 07: $+x$
forward, $+y$ right, $+z$ down). The IMU streams at 200 Hz per
`algorithm.md` line 105 ($\Delta t = 5\,\mathrm{ms}$); the strapdown
loop (§3) runs once per sample. **"Strapdown"** = rigidly attached;
gimballed IMUs are obsolete for low-cost vehicles.

---

## 2. Specific Force Is Not Acceleration

The accelerometer measures **specific force** $\mathbf{f}^{body}$ —
non-gravitational force per unit mass on its proof mass:

$$\mathbf{f}^{body} = \mathbf{a}_{\text{true}}^{body} - \mathbf{g}^{body}, \qquad \mathbf{a}_{\text{true}}^{body} = \mathbf{f}^{body} + \mathbf{g}^{body}.$$

**Launch-pad sanity check.** Rocket upright, motionless. Gravity in
NED is $\mathbf{g}^{NED} = (0,\,0,\,+9.80665)\,\mathrm{m/s^2}$ (down
positive). The standard accelerometer convention reports the
*reaction force* through the proof mass — opposite sign to the formula
above. The MPU-6050 returns this convention in `tAccelBodyMps2`: the
at-rest reading along body $+z$ is **+9.80665 m/s²**, not zero (the pad
pushes UP). A free-falling rocket reads $\mathbf{0}$.

To recover kinematic NED acceleration: (a) rotate body reading into
NED using current attitude, then (b) subtract gravity in NED — see
`algorithm.md` §3.2 step 3 lines 121–124.

---

## 3. The Strapdown Integration Loop

The seven-step loop turning one IMU sample into one updated state.
Each step cites `algorithm.md` §3.2 line numbers; chapter 11 maps each
to the corresponding `juno::kmat` symbol. The math is the **process
model** $\mathbf{f}(\mathbf{x},\mathbf{u})$ that chapter 06 wraps with
EKF covariance propagation.

Inputs: previous state $\hat{\mathbf{x}}_{k-1}$ ($\mathbf{p}^{LLA}$,
$\mathbf{v}^{NED}$, $\mathbf{q}$, $\mathbf{b}_a$, $\mathbf{b}_g$); raw
IMU sample
$(\mathbf{f}^{body}_{\text{raw}},\,\boldsymbol{\omega}^{body}_{\text{raw}},\,t_k)$;
$\Delta t = (t_k - t_{k-1}) / 10^6\,\mathrm{s}$.

### Step 1 — Subtract bias estimates (`algorithm.md` lines 115–117)

$$\mathbf{f}^{body} = \mathbf{f}^{body}_{\text{raw}} - \mathbf{b}_{a,k-1}, \qquad \boldsymbol{\omega}^{body} = \boldsymbol{\omega}^{body}_{\text{raw}} - \mathbf{b}_{g,k-1}.$$

In `juno::kmat`, both are vector subtractions (`Sub`).

### Step 2 — Rotate accel to NED (`algorithm.md` line 119)

Using the current attitude quaternion $\mathbf{q}_{k-1}$ (body→NED
per chapter 08), rotate the bias-corrected specific force into NED:

$$\mathbf{f}^{NED} = \mathbf{q}_{k-1} \otimes \mathbf{f}^{body} \otimes \mathbf{q}_{k-1}^{*}.$$

In `juno::kmat`, one call to `QuatRotate(q, v_body)`.

### Step 3 — Subtract gravity (`algorithm.md` lines 122–123)

$$\mathbf{a}^{NED} = \mathbf{f}^{NED} - \mathbf{g}^{NED}, \qquad \mathbf{g}^{NED} = (0,\,0,\,+9.80665)\,\mathrm{m/s^2}.$$

$9.80665$ is the project-wide standard gravity and the **only**
hard-coded numeric in the algorithm path (`algorithm.md` lines 124,
334). The body→NED rotation uses our possibly-wrong attitude
estimate, so attitude error $\delta\boldsymbol{\theta}$ leaks $\sim
g\,\delta\boldsymbol{\theta}$ of horizontal acceleration into
$\mathbf{a}^{NED}$ — the dominant error coupling in pure INS (§5).

### Step 4 — Velocity integration (`algorithm.md` line 125)

$$\mathbf{v}^{NED}_{k} = \mathbf{v}^{NED}_{k-1} + \mathbf{a}^{NED}\,\Delta t.$$

Forward Euler is acceptable at 200 Hz (`algorithm.md` line 127); the
truncation error per step is dwarfed by IMU noise and bias.

### Step 5 — Position integration (`algorithm.md` lines 128–135)

$$\mathbf{p}^{NED}_{k} = \mathbf{p}^{NED}_{k-1} + \mathbf{v}^{NED}_{k-1}\,\Delta t + \tfrac{1}{2}\,\mathbf{a}^{NED}\,\Delta t^2.$$

The $\tfrac{1}{2}\mathbf{a}\Delta t^2$ term matters: at 200 Hz with
200 m/s² peak boost acceleration the per-step correction is $\sim
2.5\,\mathrm{mm}$ — small per step, but $\sim 4$ m of position bias
integrated over 1500 boost samples. The IMPL then converts the NED
position delta to a geodetic delta using the local WGS-84 meridional
and prime-vertical radii of curvature (`algorithm.md` lines 130–135).
Treat the geodetic conversion as a deterministic black box.

### Step 6 — Attitude propagation (`algorithm.md` lines 136–140)

Form the small-rotation increment from the bias-corrected gyro reading
and integrate the attitude:

$$\delta\mathbf{q} = \mathbf{q}\!\left(\boldsymbol{\omega}^{body}\,\Delta t\right), \qquad \mathbf{q}_{k} = \mathbf{q}_{k-1} \otimes \delta\mathbf{q}.$$

Chapter 08 derived $\delta\mathbf{q}$ from the axis-angle form with
the half-angle approximation when $\lvert\boldsymbol{\omega}\,\Delta
t\rvert$ is small. In `juno::kmat`, `QuatMul` then `QuatNormalize` to
enforce $\lvert\mathbf{q}\rvert = 1$ (`algorithm.md` §6 lines
343–351).

### Step 7 — Bias evolution (`algorithm.md` lines 141–144)

Biases have **no deterministic dynamics** — they are zero-mean random
walks. State-propagation lines are trivial:

$$\mathbf{b}_{a,k} = \mathbf{b}_{a,k-1}, \qquad \mathbf{b}_{g,k} = \mathbf{b}_{g,k-1}.$$

Uncertainty grows through process noise $\mathbf{Q}$ only (see §4).
State viewpoint: no-ops. Covariance viewpoint: $\mathbf{Q}$ diagonal
for these states is non-zero.

That completes one iteration. At 200 Hz the loop runs every 5 ms.

---

## 4. Bias States and the Random-Walk Model

Real MEMS IMUs carry a small additive bias on each axis that drifts
slowly with temperature, voltage, vibration, and age. Calibrate-once
does not work; we estimate online.

**Random-walk process model.**

$$\mathbf{b}_{k+1} = \mathbf{b}_{k} + \boldsymbol{\eta}_{k}, \qquad \boldsymbol{\eta}_{k} \sim \mathcal{N}(\mathbf{0},\,\mathbf{Q}_{b}\,\Delta t).$$

$\mathbf{Q}_{b}$ is the **bias random-walk PSD**, units
$(\mathrm{m/s^2})^2/\mathrm{s}$ (accel), $(\mathrm{rad/s})^2/\mathrm{s}$
(gyro). The 1-sigma value in `NAV_INIT_T` is its square root: per
`algorithm.md` line 290 the column header is
$\mathrm{m/s^2}\,/\,\sqrt{\mathrm{s}}$ — **not** $\mathrm{m/s^2}$
(the bias itself). Watching this unit avoids mis-tuning $\mathbf{Q}$
by $\sqrt{\Delta t}$ [1, §14.2.2].

**Why the EKF can estimate biases.** Constant bias is unobservable
with no aiding, but gravity in NED is known — so accel-bias error
produces an inconsistency with gravity-removed velocity once any
aiding arrives. Gyro-bias error produces attitude drift that
conflicts with the position-velocity trajectory. Off-diagonal
$\mathbf{P}$ terms credit these inconsistencies to the bias states
(chapter 10).

**The 16-state vector** (`algorithm.md` §3.1 lines 73–82):

$$\mathbf{x} = \big(\,\underbrace{\mathbf{p}^{LLA}}_{3} ,\, \underbrace{\mathbf{v}^{NED}}_{3} ,\, \underbrace{\mathbf{q}}_{4} ,\, \underbrace{\mathbf{b}_a}_{3} ,\, \underbrace{\mathbf{b}_g}_{3}\,\big), \qquad \dim = 16.$$

---

## 5. Why Pure Dead-Reckoning Fails

Everything above is **dead-reckoning**: integrate IMU from a known
initial state with no external references. It fails.

### 5.1 Accel-bias error grows quadratically in time

Accelerometer bias error $\delta b_a$ (perfect attitude, no other
errors) propagates as:

$$\delta v(t) = \delta b_a \cdot t, \qquad \delta p(t) = \tfrac{1}{2}\,\delta b_a \cdot t^2.$$

For $\delta b_a = 0.05\,\mathrm{m/s^2}$ (typical MPU-6050-class
post-warm-up; cf. `algorithm.md` §5.2 line 321,
$\sigma_a \approx 0.055\,\mathrm{m/s^2}$):

- $t = 60\,\mathrm{s}$: $\delta p \approx 90\,\mathrm{m}$ —
  substantially worse than the FT1 GPS noise floor of 2.5 m.
- $t = 300\,\mathrm{s}$: $\delta p \approx 2.25\,\mathrm{km}$.

Pure INS is useless beyond seconds without aiding [6, §11.4].

### 5.2 Gyro-bias drift couples into position through attitude

Gyro bias $\delta b_g$ accumulates as attitude error $\delta\theta(t)
= \delta b_g\cdot t$, which rotates gravity in the wrong direction in
step 3, leaving residual horizontal acceleration $\sim g\,\delta b_g\,
t$. Integrate to velocity: $\tfrac{1}{2}\,g\,\delta b_g\,t^2$.
Integrate to position: $\tfrac{1}{6}\,g\,\delta b_g\,t^3$. **Cubic in
time** [1, §14.2.4]. For $\delta b_g = 2\!\times\!10^{-3}\,
\mathrm{rad/s}$ at $t = 60\,\mathrm{s}$: $\sim 706\,\mathrm{m}$.

### 5.3 Numerical errors in attitude propagation

Each forward-Euler step leaves $\lvert\mathbf{q}\rvert$ slightly
off-unit. Without renormalization the norm drifts; `QuatRotate`
amplifies the error into rotated-vector magnitude error, feeding back
into steps 3–5 as gravity-subtraction error. The IMPL must call
`juno::kmat::QuatNormalize` after every prediction step
(`algorithm.md` §6 lines 343–351).

### 5.4 Conclusion

Pure INS is **open-loop**: nothing in the strapdown loop bounds
errors. We need closed-loop correction — **aiding sensors**.

---

## 6. Error States, Briefly

Two ways to parameterize an EKF:

- **Full-state (direct).** Estimates the absolute state
  $\hat{\mathbf{x}}$; $\mathbf{P}$ is the covariance of
  $\hat{\mathbf{x}}$.
- **Error-state (indirect).** Estimates the *error* $\delta\mathbf{x}$
  between strapdown state and truth; $\mathbf{P}$ is the covariance of
  $\delta\mathbf{x}$. After each filter step the error estimate is
  injected onto the strapdown state and reset to zero.

Error-state advantages [4, §3] [5, §3]: (a) error dynamics are nearly
linear; (b) error magnitudes are small, improving conditioning of
$\mathbf{P}$; (c) a 3-vector small-rotation parameterization replaces
the 4-component quaternion, eliminating the unit-norm constraint on
$\mathbf{P}$.

`algorithm.md` lines 84–96 leaves the choice to the IMPL — the public
16-state output is identical either way. Chapter 11 walks through
both options at the C++-symbol level. Here, know the choice exists.

> **Trap.** "Error state" $\delta\mathbf{x}$ is the EKF's bookkeeping
> variable; "error in state" is $\hat{\mathbf{x}} -
> \mathbf{x}_{\text{truth}}$. Same words, different objects.
> Conflating them is a common newcomer mistake [5, §3.1].

---

## 7. Why We Need Aiding Sensors

**Aiding sensors** provide absolute references that bound INS error:

- **Barometer (`baro_lib`).** Altitude in m HAE — bounds vertical
  position directly.
- **GPS (`gps_lib`).** Geodetic position and NED velocity — bounds
  horizontal/vertical pos/vel, and indirectly attitude/bias via EKF
  cross-coupling.

The Kalman gain $\mathbf{K} = \mathbf{P}\mathbf{H}^T(\mathbf{H}\mathbf{P}\mathbf{H}^T
+ \mathbf{R})^{-1}$ optimally trades $\mathbf{P}$ vs $\mathbf{R}$ and
credits corrections to the most observable states through off-diagonal
$\mathbf{P}$ — including biases.

Result: with periodic aiding the biases become observable, the EKF
estimates them, and post-update position error stays **bounded**.
Chapter 10 builds the intuition.

---

## 8. Worked Example — 5-Second Dead-Reckon vs. EKF With GPS

Numerical narrative; no code is run.

**Setup.** Vertical thrust, constant $\mathbf{a}^{NED}_{\text{true}}
= (0,\,0,\,-30)\,\mathrm{m/s^2}$ (upward, 30 m/s²). At rest, body $+z$
aligned with NED $+z$; $\mathbf{p}^{NED} = \mathbf{0}$,
$\mathbf{v}^{NED} = \mathbf{0}$. IMU 200 Hz, perfect except $\delta
b_a = +0.05\,\mathrm{m/s^2}$ along body $+z$. Gyro perfect. Run 5 s
(1000 samples).

**Run A — Pure INS.** Per §5.1:

$$\delta v_{z}(5) = 0.05\cdot 5 = 0.25\,\mathrm{m/s}, \qquad \delta p_{z}(5) = \tfrac{1}{2}\cdot 0.05\cdot 25 = \mathbf{0.625\,m}.$$

Altitude is off 0.625 m at $t = 5$ s. Continuing dead-reckon: 90 m at
60 s.

**Run B — INS + GPS updates.** Same trajectory; EKF fuses one GPS
position per second (5 updates) with $\sigma_{\text{GPS}} = 2.5\,
\mathrm{m}$ per `algorithm.md` §5.2 line 326:

1. First update at $t = 1\,\mathrm{s}$: innovation $\sim 0.025\,
   \mathrm{m}$. Kalman weighting between INS $\sigma$ ($\sim 0.05$ m)
   and GPS $\sigma$ (2.5 m) puts almost all weight on INS — small
   position correction.
2. The **off-diagonal** $\mathbf{P}$ between altitude and
   accel-bias-z (from the strapdown $\mathbf{F}$ Jacobian) lets the
   small position innovation flow into a bias correction. The
   estimate converges toward 0.05 m/s² over the first few updates.
3. By $t = 5\,\mathrm{s}$ the residual bias error is reduced by an
   order of magnitude ($\delta b_a \approx 0.005\,\mathrm{m/s^2}$);
   position error sits inside the GPS noise floor — **bounded**.

Exact numbers depend on $\mathbf{Q}$, $\mathbf{R}$, cadence (chapter
05 has a 2-state walk-through), but the behavior is universal: EKF
turns unbounded dead-reckon into a bounded fused estimate
[1, §14.3.1] [2, §11]. INS gives 200 Hz and low short-term noise;
GPS gives bounded long-term position; the EKF inherits the best of
both.

---

## 9. FSW Anchor — Strapdown Loop in `nav_lib` C++ Symbols

Seven steps of `algorithm.md` §3.2 lines 113–144 bound to the
`juno::kmat` operations the IMPL calls. Loop runs once per IMU sample
(5 ms at 200 Hz, `algorithm.md` line 105).

| Step | `algorithm.md` lines | Math | `juno::kmat` operation |
|------|----------------------|------|------------------------|
| 1. Bias correction (accel) | 115–117 | $\mathbf{f}^{body} = \mathbf{f}^{body}_{\text{raw}} - \mathbf{b}_{a}$ | `Sub<double, 3>` |
| 1. Bias correction (gyro)  | 115–117 | $\boldsymbol{\omega}^{body} = \boldsymbol{\omega}^{body}_{\text{raw}} - \mathbf{b}_{g}$ | `Sub<double, 3>` |
| 2. Rotate accel to NED     | 119     | $\mathbf{f}^{NED} = \mathbf{q} \otimes \mathbf{f}^{body} \otimes \mathbf{q}^{*}$ | `QuatRotate(q, v_body)` |
| 3. Subtract gravity        | 122–123 | $\mathbf{a}^{NED} = \mathbf{f}^{NED} - \mathbf{g}^{NED}$ | `Sub<double, 3>` |
| 4. Velocity integration    | 125     | $\mathbf{v}^{NED}_{k} = \mathbf{v}^{NED}_{k-1} + \mathbf{a}^{NED}\,\Delta t$ | scalar mul + `Add<double, 3>` |
| 5. Position integration    | 128–135 | $\mathbf{p}^{NED}_{k} = \mathbf{p}^{NED}_{k-1} + \mathbf{v}^{NED}_{k-1}\,\Delta t + \tfrac{1}{2}\,\mathbf{a}^{NED}\,\Delta t^2$ + WGS-84 | scalar mul + `Add` ×2; geodetic conversion is IMPL-private inline math |
| 6. Attitude propagation    | 136–140 | $\mathbf{q}_{k} = \mathbf{q}_{k-1} \otimes \delta\mathbf{q}(\boldsymbol{\omega}^{body}\Delta t)$ + renorm | `QuatMul(q, dq)` then `QuatNormalize(q)` |
| 7. Bias evolution          | 141–144 | $\mathbf{b}_{a,k} = \mathbf{b}_{a,k-1}$, $\mathbf{b}_{g,k} = \mathbf{b}_{g,k-1}$ | no kmat call (state copy); $\mathbf{Q}$ diagonal set via `algorithm.md` §5.1 noise fields |

Two reminders:

- **Renormalize every step.** Step 6 must end with `QuatNormalize`,
  or unit-norm drifts and step 2's `QuatRotate` cascades magnitude
  error through the loop (`algorithm.md` §6 lines 343–351, normative).
- **Only 9.80665.** Step 3 is the sole hard-coded numeric;
  `algorithm.md` line 334 forbids any other `constexpr` numeric in
  the algorithm path — covariance and noise come from `NAV_INIT_T`.

Chapter 11 expands this into per-line C++ pseudocode against
`NAV_LIB_IMPL_T` field names.

---

## 10. Key Results

- An IMU outputs **specific force** (accelerometer, body, m/s²) and
  **angular rate** (gyro, body, rad/s); not kinematic acceleration.
- The accelerometer at rest on the launch pad reads **+9.80665 m/s²**
  along the body axis aligned with NED-down. A free-falling rocket
  reads zero. Recover kinematic NED acceleration by rotating to NED
  then subtracting gravity.
- The seven-step strapdown loop in `algorithm.md` §3.2 lines 113–144
  is the EKF process model: bias-correct, rotate, subtract gravity,
  integrate velocity, integrate position (geodetic), propagate
  attitude, evolve biases. Runs at IMU cadence (200 Hz).
- Bias states are part of the 16-state vector and evolve as zero-mean
  random walks; their PSD has units
  $\mathrm{m/s^2}\,/\,\sqrt{\mathrm{s}}$ (accel) and
  $\mathrm{rad/s}\,/\,\sqrt{\mathrm{s}}$ (gyro), not
  $\mathrm{m/s^2}$ or $\mathrm{rad/s}$.
- Pure INS dead-reckons: accel-bias position error grows $\propto
  t^2$; gyro-bias position error grows $\propto t^3$ through gravity
  mis-rotation; attitude integration drifts $\lvert\mathbf{q}\rvert$
  off unity if not renormalized.
- 0.05 m/s² accel bias → 0.625 m altitude error after 5 s, 90 m
  after 60 s. This is why aiding sensors are required.
- Aiding sensors (baro, GPS) provide absolute references the EKF
  blends with INS using optimal Kalman weighting; bounded position
  error and observable bias states result.
- Error-state vs full-state is a parameterization choice
  (`algorithm.md` lines 84–96); same 16-state public output either
  way. Don't conflate "error state" (bookkeeping variable) with
  "error in state" (estimation residual).

---

## 11. Exercises

(See chapter 12 for solutions.)

**E09.1 — Bias unit check.** A datasheet lists "bias instability" as
$0.04\,\mathrm{mg}$ and "bias random walk" as
$0.05\,\mathrm{mg}/\sqrt{\mathrm{s}}$. Convert both to SI; decide
which goes in `NAV_INIT_T.fImuAccelBiasRandomWalkMps2PerSqrtS`.

**E09.2 — Quadratic drift.** Verify that with $\delta b_a =
0.02\,\mathrm{m/s^2}$, $T = 90\,\mathrm{s}$, the pure-INS position
error reaches 81 m. What initial bias estimate at $t = 0$ keeps the
$T = 90\,\mathrm{s}$ position error under 1 m without aiding?

**E09.3 — Specific-force sign.** A rocket with body $+z$ pointing up
(opposite the FT1 convention). On the launch pad: what does the
accelerometer read along body $+z$? body $-z$? body $+x$?

**E09.4 — Cubic gyro drift.** Derive the $\tfrac{1}{6}\,g\,\delta
b_g\,t^3$ growth in §5.2 from $\delta\theta(t) = \delta b_g\,t$ and
the small-angle gravity-rotation approximation.

**E09.5 — Renormalization frequency.** Argue from chapter 08 §6 that
renormalizing every 100 prediction steps instead of every step
inflates $\lvert\mathbf{q}\rvert$ deviation by $\sim 100^{1/2}$ before
the renormalize. Why is "every step" the right choice for FT1?

---

## 12. Citations

- IMU specific-force definition and strapdown mechanization:
  [1, §13.2] and [6, §11.2].
- Bias random-walk model and the
  $\mathrm{m/s^2}\,/\,\sqrt{\mathrm{s}}$ unit: [1, §14.2.2] and
  [6, §11.4].
- $t^2$ accel-bias drift and $t^3$ gyro-bias drift: [1, §14.2.4] and
  [6, §11.5].
- Error-state vs full-state EKF: [4, §3] and [5, §3]. Normative for
  FT1 per `algorithm.md` lines 84–96 (IMPL choice) and lines 159–165
  (Trawny quaternion-error Jacobian).
- Aided-INS error budget; GPS aiding bounds INS drift:
  [1, §14.3.1] and [6, §11.6].
- Linear Kalman update / Joseph form (§7 and §8.3): [2, §11].
- FT1-specific anchors: `docs/design/nav/algorithm.md` §3.1 (state
  vector, lines 73–82), §3.2 (process model, lines 113–144), §6
  (numerical stability, lines 343–351).

<!-- @{"design": ["SW-REQ-NAV-018"]} -->
