---
document_type: nav_kalman tutorial — Chapter 03
program: Juno FT1 FSW
sprint: SPRINT-IMPL-NAV-TUTORIAL
chapter: 03
title: State-Space Models
prerequisites: 01 (linear algebra), 02 (probability + Gaussians)
next: 04 (Kalman filter derivation)
---

# Chapter 03 — State-Space Models

> **Reading order.** Assumes Chapter 01 (vectors, matrices, $\mathbf{A}\mathbf{x}$,
> transpose, identity, inverse) and Chapter 02 (random vectors, mean,
> covariance, Gaussians, $\mathcal{N}(\boldsymbol{\mu}, \boldsymbol{\Sigma})$).

The Kalman filter (Chapter 04) is built on top of a *state-space model*. This
chapter is purely about that model — what a "state" is, how it evolves
between samples, and how a sensor relates to it. We end with a 1D worked
example that **Chapter 05 will reuse verbatim** for a numerical Kalman pass.
Lock in the numbers.

---

## 3.1 What is a "state"?

A **state** is the minimum set of variables you need *now* to predict the
future of a system, given any future inputs. Anything you can forget about
the past is not part of the state.

Three examples:

- **Rock falling in 1D.** Altitude $p$ and velocity $v$. Given $(p, v)$
  now and gravity $g$, you can predict $(p, v)$ at any future time. State:
  $\mathbf{x} = [p,\ v]^T \in \mathbb{R}^2$.
- **Pendulum.** Angle $\theta$ and angular velocity $\omega$. State:
  $\mathbf{x} = [\theta,\ \omega]^T \in \mathbb{R}^2$.
- **Rocket.** Position (3) + velocity (3) + attitude quaternion (4) +
  accelerometer bias (3) + gyroscope bias (3) = 16. This is *exactly* the
  FT1 nav state vector — see §3.9.

The key idea: **the state is sufficient**. In practice we have noise — that
is what motivates the Kalman filter in the first place.

Don't confuse "discrete-time" (we sample at fixed intervals) with
"discrete-state" (state takes values from a finite set). Our states are
continuous real-valued; only *time* is discretized.

---

## 3.2 Discrete-time vs continuous-time

The physical world is continuous: gravity tugs smoothly. Continuous-time
dynamics are written as differential equations:

$$
\dot{\mathbf{x}}(t) = \mathbf{f}_c(\mathbf{x}(t), \mathbf{u}(t))
$$

where the dot denotes $d/dt$. Flight software does not solve ODEs at run
time; it samples. Juno FT1's EKF prediction runs **at the IMU sample rate
of 200 Hz** — one prediction per IMU sample, per
`docs/design/nav/algorithm.md` line 105 ("the canonical nominal cadence is
one call per IMU sample at `kImuAppPeriodMs = 5 ms` (200 Hz)"). That fixes
$\Delta t = 5$ ms.

We therefore work in **discrete time** with the recurrence

$$
\mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k, \mathbf{u}_k)
$$

where $t_k = k\,\Delta t$ and $\mathbf{x}_k \approx \mathbf{x}(t_k)$.
§3.5 shows how to convert continuous $\mathbf{f}_c$ into discrete
$\mathbf{f}$.

---

## 3.3 The process model

The **process model** describes how the state evolves from one sample to the
next. In its full generality:

$$
\boxed{\ \mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k, \mathbf{u}_k) + \mathbf{w}_k\ }
$$

The pieces:

- $\mathbf{x}_k \in \mathbb{R}^n$ — the state at sample $k$.
- $\mathbf{u}_k \in \mathbb{R}^m$ — a **control input** or known driving
  signal. Drop it for unforced systems. For Juno FT1, the IMU sample
  (accel + gyro) plays the role of $\mathbf{u}_k$ (`algorithm.md` §3.2
  lines 101–187): we use measured accel and gyro to *drive* the prediction
  rather than treat them as observations of the state ("INS-driven"
  prediction).
- $\mathbf{w}_k \in \mathbb{R}^n$ — **process noise**, zero-mean Gaussian:
  $\mathbf{w}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{Q})$. $\mathbf{Q}$
  is the process noise covariance.
- $\mathbf{f}: \mathbb{R}^n \times \mathbb{R}^m \to \mathbb{R}^n$ — the
  state-transition function (linear or nonlinear).

### Linear process model

When $\mathbf{f}$ is linear:

$$
\mathbf{x}_{k+1} = \mathbf{F}\,\mathbf{x}_k + \mathbf{B}\,\mathbf{u}_k + \mathbf{w}_k
$$

$\mathbf{F} \in \mathbb{R}^{n \times n}$ is the **state-transition
matrix**; $\mathbf{B} \in \mathbb{R}^{n \times m}$ the **input matrix**.
This is the form Brown & Hwang [2] §4 builds the linear Kalman derivation
on. Burn it in.

### Worked instance: rock falling in 1D, no input

$\mathbf{x} = [p,\ v]^T$, $\Delta t = 0.1$ s, no forcing, velocity constant
over one sample:

$$
\begin{aligned}
p_{k+1} &= p_k + v_k \,\Delta t \\
v_{k+1} &= v_k
\end{aligned}
$$

Stack into a matrix equation:

$$
\begin{bmatrix} p_{k+1} \\ v_{k+1} \end{bmatrix}
= \underbrace{\begin{bmatrix} 1 & \Delta t \\ 0 & 1 \end{bmatrix}}_{\mathbf{F}}
  \begin{bmatrix} p_k \\ v_k \end{bmatrix}
$$

So $\mathbf{F} = \begin{bmatrix} 1 & \Delta t \\ 0 & 1 \end{bmatrix}$
and there is no $\mathbf{B}\mathbf{u}_k$. Lock this matrix in — §3.6 and
Chapter 05 use it. (To add gravity as input $u_k = -g$, set
$\mathbf{B} = [\tfrac{1}{2}\Delta t^2,\ \Delta t]^T$. Not needed for our
worked example.)

---

## 3.4 The measurement model

The **measurement model** describes how a sensor reading relates to the
state:

$$
\boxed{\ \mathbf{z}_k = \mathbf{h}(\mathbf{x}_k) + \mathbf{v}_k\ }
$$

The pieces:

- $\mathbf{z}_k \in \mathbb{R}^p$ — the sensor reading at sample $k$ ($p$
  = reading dimension).
- $\mathbf{h}: \mathbb{R}^n \to \mathbb{R}^p$ — **measurement function**;
  maps unobservable state to observable reading.
- $\mathbf{v}_k \in \mathbb{R}^p$ — **measurement noise**, zero-mean
  Gaussian: $\mathbf{v}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{R})$.

Chapter 04 leans on: $\mathbf{w}_k$ and $\mathbf{v}_k$ are independent of
each other and across time samples ("white noise").

### Linear measurement model

When $\mathbf{h}$ is linear:

$$
\mathbf{z}_k = \mathbf{H}\,\mathbf{x}_k + \mathbf{v}_k
$$

with $\mathbf{H} \in \mathbb{R}^{p \times n}$ the **observation matrix**.

### Worked instance: a position sensor on the falling rock

Position-only sensor (e.g., altimeter on the rock):

$$
z_k = p_k + v_{\text{noise},k} \qquad
\Rightarrow\qquad
\mathbf{H} = \begin{bmatrix} 1 & 0 \end{bmatrix}
$$

Geometrically, $\mathbf{H}$ is a $1 \times 2$ projector: it picks the
first component out of $[p, v]^T$ — the "shadow" of the state onto the
$p$-axis. A velocity-only sensor would give
$\mathbf{H} = \begin{bmatrix} 0 & 1 \end{bmatrix}$, shadow on $v$-axis.
This shadow intuition matters for observability — §3.7.

---

## 3.5 Discretizing a continuous-time linear system

Given continuous dynamics $\dot{\mathbf{x}}(t) = \mathbf{A}\,\mathbf{x}(t)$,
the exact discrete-time equivalent at sample interval $\Delta t$ is

$$
\mathbf{F} = \exp(\mathbf{A}\,\Delta t)
\qquad\text{where}\qquad
\exp(\mathbf{M}) = \mathbf{I} + \mathbf{M} + \tfrac{1}{2!}\mathbf{M}^2 + \cdots
$$

Don't dwell on the matrix exponential. For the rock:
$\dot{p} = v$, $\dot{v} = 0$ gives
$\mathbf{A} = \begin{bmatrix} 0 & 1 \\ 0 & 0 \end{bmatrix}$.
Because $\mathbf{A}^2 = \mathbf{0}$, the Taylor series **terminates after
two terms**:

$$
\mathbf{F} = \mathbf{I} + \mathbf{A}\,\Delta t
= \begin{bmatrix} 1 & \Delta t \\ 0 & 1 \end{bmatrix}
$$

Same matrix as the §3.3 by-inspection result. The takeaway:

> For $\dot{\mathbf{x}} = \mathbf{A}\mathbf{x}$ sampled at $\Delta t$, the
> discrete update matrix is $\mathbf{F} = \exp(\mathbf{A}\,\Delta t)$.
> When $\mathbf{A}$ is nilpotent the series terminates and you can do it
> by hand.

---

## 3.6 Worked example: 1D position+velocity tracking

The canonical "constant-velocity" tracking model. Chapter 05 runs a full
Kalman filter on **these exact numbers** — do not change them.

- **State.** $\mathbf{x}_k = [p_k,\ v_k]^T \in \mathbb{R}^2$ (m, m/s).
- **Sample interval.** $\Delta t = 0.1$ s (10 Hz).
- **Process** (forward Euler, no input):
  $\mathbf{x}_{k+1} = \mathbf{F}\,\mathbf{x}_k + \mathbf{w}_k$ with
  $\mathbf{F} = \begin{bmatrix} 1 & 0.1 \\ 0 & 1 \end{bmatrix}$.
- **Process noise covariance** — small diagonal; the velocity entry is
  larger because unmodeled accelerations (e.g., wind, thrust drift) act
  on velocity, not directly on position:
  $\mathbf{Q} = \begin{bmatrix} 10^{-4} & 0 \\ 0 & 10^{-2} \end{bmatrix}$
  (units m² and (m/s)²).
- **Measurement** (noisy position only):
  $z_k = \mathbf{H}\,\mathbf{x}_k + v_k$ with
  $\mathbf{H} = \begin{bmatrix} 1 & 0 \end{bmatrix}$.
- **Measurement noise variance** (scalar):
  $\mathbf{R} = \sigma_z^2 = (1.0\ \text{m})^2 = 1.0\ \text{m}^2$ — the
  position sensor is good to ~1 m (1-sigma).
- **Initial state and covariance.** We *guess* the rock starts at $p=0$
  at rest, fairly confident on position (~30 cm) and less confident on
  velocity (~1 m/s):
  $\hat{\mathbf{x}}_{0} = [0,\ 0]^T$,
  $\mathbf{P}_{0} = \begin{bmatrix} 0.1 & 0 \\ 0 & 1.0 \end{bmatrix}$.

> **Lock in those six items** ($\Delta t$, $\mathbf{F}$, $\mathbf{Q}$,
> $\mathbf{H}$, $\mathbf{R}$, $\mathbf{P}_0$). Chapter 05 reuses every
> one. This is the pedagogical model from Brown & Hwang [2] §4 and
> Bar-Shalom et al. [7] Ch. 5.

---

## 3.7 Observability (intuitive)

A state is **observable** if measurements eventually pin it down — given
enough samples, the filter can recover that state component from sensor
readings within the noise level.

Two contrasts:

- **Position-only sensor on $\mathbf{x} = [p,\ v]^T$ (rock falling).**
  Both states observable. $p$ is measured directly. $v$ is also
  observable: across two samples the rate of change of $p$ reveals $v$.
  The Kalman filter implicitly does that finite difference through the
  $p$-$v$ coupling in $\mathbf{F}$ — that is why a position-only sensor
  produces a useful velocity estimate.
- **Position-only sensor on $\mathbf{x} = [v]$ alone.** $v$ is **not
  observable**: without a position state the filter has nothing to anchor
  the position reading against.

We will not formalize this with the observability matrix
$[\mathbf{H}^T,\ (\mathbf{H}\mathbf{F})^T,\ \cdots]^T$ in this tutorial.
Brown & Hwang [2] §4.6 covers the formal version. Takeaway: **the process
model couples states to each other, and that coupling is what makes
unmeasured states observable through measured ones.**

---

## 3.8 Nonlinear state-space

Same framework, general functions instead of matrices:

$$
\mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k, \mathbf{u}_k) + \mathbf{w}_k
\qquad
\mathbf{z}_k = \mathbf{h}(\mathbf{x}_k) + \mathbf{v}_k
$$

**One-line nonlinear example: pendulum.** $\mathbf{x} = [\theta, \omega]^T$,
with gravity $g$ and length $L$:

$$
\mathbf{f}(\mathbf{x}) = \begin{bmatrix} \theta + \omega\,\Delta t \\
\omega - (g/L)\sin\theta\,\Delta t \end{bmatrix}
$$

The $\sin\theta$ is the nonlinearity — no constant $\mathbf{F}$ captures
this for all $\theta$. The Extended Kalman Filter (Chapter 06) handles
nonlinear $\mathbf{f}, \mathbf{h}$ by **linearizing** around the current
estimate — locally approximating $\mathbf{f}$ by a Jacobian
$\mathbf{F} = \partial \mathbf{f}/\partial \mathbf{x}$ at $\hat{\mathbf{x}}_k$,
then running linear-Kalman update with that local $\mathbf{F}$.

---

## 3.9 FSW Anchor — The FT1 nav System as State-Space

Map this whole chapter onto the FT1 navigation EKF. Open
`docs/design/nav/algorithm.md` alongside.

### State vector

Per `algorithm.md` §3.1 lines 70–82, the FT1 nav state is 16-dimensional:

| Component | Dim | Units / Frame |
|-----------|-----|---------------|
| `tPosLla` (lat, lon, alt) | 3 | deg, deg, m HAE (WGS-84) |
| `tVelNed` (Vn, Ve, Vd) | 3 | m/s, NED |
| `tAttQuat` (w, x, y, z) | 4 | unit quaternion, body→NED, Hamilton |
| `tAccelBias` | 3 | m/s², body |
| `tGyroBias` | 3 | rad/s, body |
| **Total** | **16** | |

So $\mathbf{x}_k \in \mathbb{R}^{16}$. The bias states matter: IMU
accelerometer and gyroscope have small offsets that drift over hours;
carrying them as state lets the filter learn and subtract them. Without
them, IMU-only dead-reckoning during BOOST would diverge.

### Process model

The FT1 process model is the IMU-driven prediction in `algorithm.md`
§3.2 lines 101–187:

$$
\mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k,\ \text{IMU sample}_k) + \mathbf{w}_k
$$

Key points:

- The **IMU sample** (`tAccelBodyMps2`, `tGyroBodyRps`) plays the role of
  $\mathbf{u}_k$ — a known driving signal, not an observation. This
  input-driven form is convention in the GNSS/INS literature (Groves [1]
  §14).
- The process is **nonlinear**: step 2 of §3.2 applies a body→NED
  quaternion rotation; step 6 composes a small-rotation quaternion via
  the Hamilton product. Both operations are nonlinear in $\mathbf{x}$.
- Because the process is nonlinear, we need an **EKF** (Chapter 06), not
  the linear KF of Chapter 04 — the Jacobian-linearization step previewed
  in §3.8.

### Measurement models

Both FT1 measurement models are **linear in the state** — the nav system
is therefore "nonlinear in process, linear in measurements," a classic
EKF shape.

- **Baro** (`algorithm.md` §4.1 lines 198–199):
  $\mathbf{H}_{\text{baro}} = \begin{bmatrix} 0 & 0 & 1 & 0 & \cdots & 0 \end{bmatrix}$
  picks out the altitude state. $\mathbf{R}_{\text{baro}} = (\text{fBaroNoiseSigmaM})^2$
  is a $1 \times 1$ scalar.
- **GPS** (`algorithm.md` §4.2 lines 219–233): 6-D measurement (lat,
  lon, alt, Vn, Ve, Vd). $\mathbf{H}_{\text{gps}}$ is $6 \times 16$ with
  `1.0` entries on position/velocity rows. $\mathbf{R}_{\text{gps}}$ is
  $6 \times 6$ diagonal from `fGpsHorizNoiseSigmaM` (squared, lat and
  lon), `fGpsVertNoiseSigmaM` (squared, altitude), and
  `fGpsVelNoiseSigmaMps` (squared, three velocity components).

### Where the noise numbers come from

Per `algorithm.md` §5.1 lines 286–296, the load-time `NAV_INIT_T` schema
carries the IMU sigmas (`fImuAccelNoiseSigmaMps2[3]`,
`fImuGyroNoiseSigmaRps[3]`, `fImuAccelBiasRandomWalkMps2PerSqrtS[3]`,
`fImuGyroBiasRandomWalkRpsPerSqrtS[3]`) that populate $\mathbf{Q}$, plus
the measurement sigmas (`fBaroNoiseSigmaM`, `fGpsHorizNoiseSigmaM`,
`fGpsVertNoiseSigmaM`, `fGpsVelNoiseSigmaMps`) that populate
$\mathbf{R}_{\text{baro}}$ and $\mathbf{R}_{\text{gps}}$ (6×6 diagonal,
see `algorithm.md` §4.2 lines 226–233). Every covariance number is
**load-time configurable** — $\mathbf{Q}$ and $\mathbf{R}$ are the
filter's tuning knobs.

---

## 3.10 Recap

You now have the four building blocks every Kalman filter needs:

1. **State vector** $\mathbf{x}_k$ — sufficient summary for prediction.
2. **Process model** $\mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k,
   \mathbf{u}_k) + \mathbf{w}_k$ with covariance $\mathbf{Q}$.
3. **Measurement model** $\mathbf{z}_k = \mathbf{h}(\mathbf{x}_k) +
   \mathbf{v}_k$ with covariance $\mathbf{R}$.
4. **Initial estimate** $\hat{\mathbf{x}}_0$ and covariance $\mathbf{P}_0$.

For linear systems, replace $\mathbf{f}, \mathbf{h}$ with constant matrices
$\mathbf{F}, \mathbf{H}$. Chapter 04 derives the linear Kalman filter on
this; Chapter 05 runs the §3.6 worked example numerically; Chapter 06
extends to nonlinear $\mathbf{f}$ (the EKF) to handle the FT1 nav system
from §3.9.

---

## References

[1] Groves, P. D. (2013). *Principles of GNSS, Inertial, and Multisensor
Integrated Navigation Systems* (2nd ed.), Artech House. §14 (INS error
state EKF).

[2] Brown, R. G., and Hwang, P. Y. C. (2012). *Introduction to Random
Signals and Applied Kalman Filtering* (4th ed.), Wiley. **§4
(state-space framework, linear discrete-time models).** This chapter's
shape and notation follow Brown & Hwang [2] §4 throughout.

[3] Kalman, R. E. (1960). "A New Approach to Linear Filtering and
Prediction Problems." *Trans. ASME — J. Basic Eng.*, 82, 35–45.

[7] Bar-Shalom, Y., Li, X. R., and Kirubarajan, T. (2001). *Estimation
with Applications to Tracking and Navigation*, Wiley. Ch. 5 (linear
state-space and the Kalman filter), Ch. 6 (extensions).

(References [4] Trawny & Roumeliotis 2005, [5] Solà 2017, [6] Farrell
2008 are not cited in this chapter; they appear in Chapter 06 EKF.)

---

## Cross-references

- `docs/design/nav/algorithm.md` §3.1 lines 70–82 (state vector); §3.2
  lines 101–187 (IMU-driven process); §4.1 lines 198–199 (baro
  $\mathbf{H}$); §4.2 lines 219–233 (GPS $\mathbf{H}$,
  $\mathbf{R}_{\text{gps}}$); §5.1 lines 286–296 (`NAV_INIT_T` noise schema)
- Chapter 02 — Gaussians, mean, covariance, independence
- Chapter 04 (next) — derives linear Kalman filter on §3.3 + §3.4
- Chapter 05 — numerical worked example reusing §3.6 verbatim
- Chapter 06 — Extended Kalman filter; revisits §3.8 + §3.9
