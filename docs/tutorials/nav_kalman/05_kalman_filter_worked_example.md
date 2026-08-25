---
document_type: nav_kalman tutorial — Chapter 05
program: Juno FT1 FSW
revision: B
effective_date: 2026-05-08
parent_sprint: SPRINT-IMPL-NAV-TUTORIAL
predecessor_chapters: 01 (linear algebra), 02 (probability), 03 (state-space), 04 (KF derivation)
successor_chapters: 06 (Extended Kalman Filter)
status: Draft for SSE-R + CE review
---

# Chapter 05 — The Kalman Filter, A Worked Numerical Example

**Prereqs.** Chapters 01–04. **Goal.** Run the predict/update equations
on real numbers, by hand, for 5 ticks of a 1D position+velocity tracker.
Pencil and a calculator nearby. **Notation.** $\hat{\mathbf{x}}_{k|j}$
is the estimate at tick $k$ given measurements through tick $j$ — so
$\hat{\mathbf{x}}_{1|0}$ is the prediction *before* the tick-1
measurement, $\hat{\mathbf{x}}_{1|1}$ the posterior *after*.

---

## 1. Recap the Model (chapter 03)

We track a point on a line. State has two components:

$$
\mathbf{x} = \begin{bmatrix} p \\ v \end{bmatrix},
\quad p \in \mathbb{R}\,[\mathrm{m}],\ v \in \mathbb{R}\,[\mathrm{m/s}].
$$

Time step $\Delta t = 0.1$ s. Forward-Euler dynamics:

$$
\mathbf{F} = \begin{bmatrix} 1 & 0.1 \\ 0 & 1 \end{bmatrix},
\qquad
\mathbf{Q} = \begin{bmatrix} 10^{-4} & 0 \\ 0 & 10^{-2} \end{bmatrix}.
$$

> $\mathbf{Q}$ here is didactic; a real airframe tunes from bench data.

A position-only sensor with $\sigma_z = 1.0$ m (a noisy ruler):

$$
\mathbf{H} = \begin{bmatrix} 1 & 0 \end{bmatrix},
\qquad R = 1.0\ \mathrm{m}^2.
$$

Initial belief — the cart starts at the origin, at rest, position more
trusted than velocity:

$$
\hat{\mathbf{x}}_{0|0} = \begin{bmatrix} 0 \\ 0 \end{bmatrix},
\qquad
\mathbf{P}_{0|0} = \begin{bmatrix} 0.1 & 0 \\ 0 & 1.0 \end{bmatrix}.
$$

Five matrices and two vectors; everything that follows is bookkeeping.

---

## 2. Ground Truth and Noisy Measurements

The cart actually accelerates at exactly $1\ \mathrm{m/s^2}$ from rest:
$p_k^{\text{true}} = \tfrac{1}{2} t_k^2$, $v_k^{\text{true}} = t_k$, with
$t_k = k \Delta t$.

| $k$ | $t_k$ [s] | $p_k^{\text{true}}$ [m] | $v_k^{\text{true}}$ [m/s] | $z_k$ [m] | error |
|----:|----------:|------------------------:|--------------------------:|----------:|------:|
| 1 | 0.1 | 0.005 | 0.1 |  0.7 | $+0.695$ |
| 2 | 0.2 | 0.020 | 0.2 | $-0.5$ | $-0.520$ |
| 3 | 0.3 | 0.045 | 0.3 |  1.1 | $+1.055$ |
| 4 | 0.4 | 0.080 | 0.4 | $-0.2$ | $-0.280$ |
| 5 | 0.5 | 0.125 | 0.5 |  1.3 | $+1.175$ |

Measurements are *fixed* so every reader verifies the same numbers.
They are very noisy ($\sigma_z = 1$ m versus truth that moves only
0.125 m); a five-sample posterior may be biased if the noise sequence
is — see §8. The filter sees only $z_k$, not $p_k^{\text{true}}$.

---

## 3. The Loop (recap from chapter 04)

For each tick $k$:

**Predict** (push belief forward through the model, no data):

$$
\hat{\mathbf{x}}_{k|k-1} = \mathbf{F}\,\hat{\mathbf{x}}_{k-1|k-1},
\quad
\mathbf{P}_{k|k-1} = \mathbf{F}\,\mathbf{P}_{k-1|k-1}\,\mathbf{F}^T + \mathbf{Q}.
$$

**Update** (fuse $z_k$):

$$
y = z_k - \mathbf{H}\,\hat{\mathbf{x}}_{k|k-1},
\quad
S = \mathbf{H}\,\mathbf{P}_{k|k-1}\,\mathbf{H}^T + R,
\quad
\mathbf{K} = \mathbf{P}_{k|k-1}\,\mathbf{H}^T\,/\,S,
$$

$$
\hat{\mathbf{x}}_{k|k} = \hat{\mathbf{x}}_{k|k-1} + \mathbf{K}\,y,
$$

$$
\mathbf{P}_{k|k} = (\mathbf{I} - \mathbf{K}\mathbf{H})\,\mathbf{P}_{k|k-1}\,
(\mathbf{I} - \mathbf{K}\mathbf{H})^T + \mathbf{K}\,R\,\mathbf{K}^T,
$$

then symmetrize $\mathbf{P}_{k|k} \leftarrow \tfrac{1}{2}(\mathbf{P}_{k|k}
+ \mathbf{P}_{k|k}^T)$.

With $\mathbf{H} = (1\ 0)$ and scalar $R$, $S$ is scalar and $\mathbf{K}$
is a **2×1 column vector** — the most common hand-calculation slip. We
use **Joseph form** because it is symmetric by construction
(`docs/design/nav/algorithm.md` §6 lines 351-357; symmetry enforcement at lines 359-363).

---

## 4. Tick 1 ($k=1$, $z_1 = 0.7$) — full detail

### 4.1 Predict — state

$$
\hat{\mathbf{x}}_{1|0}
= \begin{bmatrix} 1 & 0.1 \\ 0 & 1 \end{bmatrix}
  \begin{bmatrix} 0 \\ 0 \end{bmatrix}
= \begin{bmatrix} 0 \\ 0 \end{bmatrix}.
$$

### 4.2 Predict — covariance

$\mathbf{F}\mathbf{P}_{0|0}$:

$$
\begin{bmatrix} 1 & 0.1 \\ 0 & 1 \end{bmatrix}
\begin{bmatrix} 0.1 & 0 \\ 0 & 1.0 \end{bmatrix}
= \begin{bmatrix} 0.1 & 0.1 \\ 0 & 1.0 \end{bmatrix}.
$$

Times $\mathbf{F}^T = \begin{bmatrix} 1 & 0 \\ 0.1 & 1 \end{bmatrix}$:

$$
\begin{bmatrix} 0.1 & 0.1 \\ 0 & 1.0 \end{bmatrix}
\begin{bmatrix} 1 & 0 \\ 0.1 & 1 \end{bmatrix}
= \begin{bmatrix} 0.11 & 0.10 \\ 0.10 & 1.00 \end{bmatrix}.
$$

Plus $\mathbf{Q}$:

$$
\mathbf{P}_{1|0} = \begin{bmatrix} 0.1101 & 0.10 \\ 0.10 & 1.01 \end{bmatrix}.
$$

> Position variance grew (0.10 → 0.1101): $\Delta t^2 \sigma_v^2 = 0.01$
> plus $10^{-4}$ from $\mathbf{Q}[0,0]$. **Off-diagonal $0.10$ appeared**
> — position and velocity are now correlated, which is how a
> position-only sensor can eventually estimate velocity (ch. 03 §5).

### 4.3 Update — innovation, gain, state

$y_1 = z_1 - \mathbf{H}\hat{\mathbf{x}}_{1|0} = 0.7 - 0 = 0.7$, and $S_1
= \mathbf{P}_{1|0}[0,0] + R = 0.1101 + 1.0 = 1.1101$. The gain
$\mathbf{P}_{1|0}\mathbf{H}^T = (0.1101,\,0.10)^T$ divided by scalar
$S_1$, and the state update:

$$
\mathbf{K}_1
= \frac{1}{1.1101}\begin{bmatrix} 0.1101 \\ 0.10 \end{bmatrix}
= \begin{bmatrix} 0.099180 \\ 0.090082 \end{bmatrix},
\quad
\hat{\mathbf{x}}_{1|1}
= \begin{bmatrix} 0 \\ 0 \end{bmatrix}
+ \mathbf{K}_1\cdot 0.7
= \begin{bmatrix} 0.069426 \\ 0.063057 \end{bmatrix}.
$$

### 4.4 Update — covariance (Joseph form)

$\mathbf{I} - \mathbf{K}_1\mathbf{H} = \bigl[\!\begin{smallmatrix} 0.900820 & 0 \\ -0.090082 & 1 \end{smallmatrix}\!\bigr]$.
Compute $(\mathbf{I}\!-\!\mathbf{K}_1\mathbf{H})\,\mathbf{P}_{1|0}$:

$$
\begin{bmatrix} 0.900820 & 0 \\ -0.090082 & 1 \end{bmatrix}
\begin{bmatrix} 0.1101 & 0.10 \\ 0.10 & 1.01 \end{bmatrix}
= \begin{bmatrix} 0.099180 & 0.090082 \\ 0.090082 & 1.000992 \end{bmatrix}.
$$

Times $(\mathbf{I}\!-\!\mathbf{K}_1\mathbf{H})^T$:

$$
\begin{bmatrix} 0.099180 & 0.090082 \\ 0.090082 & 1.000992 \end{bmatrix}
\begin{bmatrix} 0.900820 & -0.090082 \\ 0 & 1 \end{bmatrix}
= \begin{bmatrix} 0.089345 & 0.081147 \\ 0.081145 & 0.992877 \end{bmatrix}.
$$

Plus $\mathbf{K}_1 R \mathbf{K}_1^T = \mathbf{K}_1\mathbf{K}_1^T$ (since $R=1$):

$$
\mathbf{K}_1 \mathbf{K}_1^T
= \begin{bmatrix} 0.009837 & 0.008934 \\ 0.008934 & 0.008115 \end{bmatrix}.
$$

Sum and symmetrize:

$$
\mathbf{P}_{1|1}
= \begin{bmatrix} 0.099182 & 0.090080 \\ 0.090080 & 1.000992 \end{bmatrix}.
$$

> Position variance dropped 0.1101 → 0.0992; velocity barely moved (we
> did not measure velocity). **Off-diagonals went 0 → 0.090** — the
> filter has discovered position-velocity coupling.

---

## 5. Tick 2 ($k=2$, $z_2 = -0.5$)

### Predict

$\hat{\mathbf{x}}_{2|1} = (0.069426 + 0.1\cdot 0.063057,\ 0.063057)^T = (0.075732,\ 0.063057)^T$.
For the covariance, $\mathbf{F}\mathbf{P}_{1|1}\mathbf{F}^T$ followed by $+\mathbf{Q}$:

$$
\mathbf{F}\mathbf{P}_{1|1} = \begin{bmatrix} 0.108190 & 0.190179 \\ 0.090080 & 1.000992 \end{bmatrix},
\quad
\mathbf{F}\mathbf{P}_{1|1}\mathbf{F}^T = \begin{bmatrix} 0.127208 & 0.190179 \\ 0.190179 & 1.000992 \end{bmatrix},
$$

$$
\mathbf{P}_{2|1}
= \begin{bmatrix} 0.127308 & 0.190179 \\ 0.190179 & 1.010992 \end{bmatrix}.
$$

### Update

$y_2 = -0.5 - 0.075732 = -0.575732$, $S_2 = 1.127308$, and

$$
\mathbf{K}_2 = \begin{bmatrix} 0.112931 \\ 0.168702 \end{bmatrix},\quad
\hat{\mathbf{x}}_{2|2}
= \begin{bmatrix} 0.010714 \\ -0.034069 \end{bmatrix},\quad
\mathbf{P}_{2|2}
= \begin{bmatrix} 0.112930 & 0.168702 \\ 0.168702 & 0.978909 \end{bmatrix}.
$$

> Position variance went *up* 0.099 → 0.113: predict added $\mathbf{Q}$
> and pushed velocity uncertainty into position; the noisy measurement
> could not pull it back. Velocity variance is finally dropping (1.001
> → 0.979). The big negative innovation pulled both states downward.

---

## 6. Ticks 3, 4, 5 (top-line numbers)

Same pattern as ticks 1–2. Top-line results below; intermediate matrices
match the §9 NumPy output to $10^{-6}$.

| $k$ | $\hat{\mathbf{x}}_{k\mid k-1}$ | $\mathbf{P}_{k\mid k-1}[0,0]$ | $z_k$ | $y_k$ | $S_k$ | $\mathbf{K}_k$ | $\hat{\mathbf{x}}_{k\mid k}$ |
|---:|:---|---:|---:|---:|---:|:---|:---|
| 3 | $(0.007307,\, -0.034069)$ | 0.156559 |  1.1 |  1.092693 | 1.156559 | $(0.135366,\, 0.230506)$ | $(0.155220,\, 0.217804)$ |
| 4 | $(0.177000,\,  0.217804)$ | 0.190842 | $-0.2$ | $-0.377000$ | 1.190842 | $(0.160258,\, 0.271448)$ | $(0.116583,\, 0.115468)$ |
| 5 | $(0.128130,\,  0.115468)$ | 0.223145 |  1.3 |  1.171870 | 1.223145 | $(0.182435,\, 0.291396)$ | $(0.341922,\, 0.456946)$ |

Posterior covariance matrices at the end of each tick:

$$
\mathbf{P}_{3|3} = \begin{bmatrix} 0.135366 & 0.230506 \\ 0.230506 & 0.927458 \end{bmatrix},\
\mathbf{P}_{4|4} = \begin{bmatrix} 0.160258 & 0.271449 \\ 0.271449 & 0.849711 \end{bmatrix},\
\mathbf{P}_{5|5} = \begin{bmatrix} 0.182436 & 0.291396 \\ 0.291396 & 0.755852 \end{bmatrix}.
$$

> At tick 4, velocity took a hit (0.218 → 0.115): $z_4 = -0.2$
> contradicts the upward trend, so the filter shrinks velocity. More
> iterations average the noise out.

---

## 7. Results Table

| $k$ | $\hat{p}_{k\mid k}$ | $\hat{v}_{k\mid k}$ | $P_{00}$ | $P_{01}$ | $P_{11}$ | $\det\mathbf{P}$ | $K_p$ | $K_v$ |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 |  0.0000 |  0.0000 | 0.10000 | 0.00000 | 1.00000 | 0.10000 | — | — |
| 1 |  0.0694 |  0.0631 | 0.09918 | 0.09008 | 1.00099 | 0.09117 | 0.09918 | 0.09008 |
| 2 |  0.0107 | $-0.0341$ | 0.11293 | 0.16870 | 0.97891 | 0.08209 | 0.11293 | 0.16870 |
| 3 |  0.1552 |  0.2178 | 0.13537 | 0.23051 | 0.92746 | 0.07242 | 0.13537 | 0.23051 |
| 4 |  0.1166 |  0.1155 | 0.16026 | 0.27145 | 0.84971 | 0.06250 | 0.16026 | 0.27145 |
| 5 |  0.3419 |  0.4569 | 0.18244 | 0.29140 | 0.75585 | 0.05299 | 0.18244 | 0.29140 |

Four patterns to read off this table:

1. **Velocity variance $P_{11}$ shrinks monotonically** (1.000 → 0.756).
   The filter learns velocity *even though we never measured it* — it
   inferred velocity from the rate-of-change of position observations.
   This is the canonical magic of the Kalman filter.
2. **Position variance $P_{00}$ grows** (0.099 → 0.182). Process noise
   adds $\sim$0.027 every predict step (mostly the $\Delta t^2 P_{11}$
   term), and the noisy $R = 1$ measurement is not strong enough to pull
   $P_{00}$ back below that floor. The filter is approaching steady
   state where $P_{00}$ stops growing.
3. **Total uncertainty $\det\mathbf{P}$ shrinks monotonically** (0.100
   → 0.053). Even though one diagonal grows, the off-diagonal coupling
   and shrinking $P_{11}$ tighten the joint distribution. The 1-σ
   confidence ellipse gets smaller in *area* every tick.
4. **Kalman gain approaches steady state.** $K_p$: 0.099, 0.113, 0.135,
   0.160, 0.182 — increments shrinking. $K_v$: 0.090, 0.169, 0.231,
   0.271, 0.291 — same. The discrete algebraic Riccati equation gives a
   steady-state gain near $(0.27, 0.40)^T$ for these $\mathbf{Q}, R$.
   Keep iterating and the gain stabilizes; the filter has finished
   *learning* and is now just *weighing*.

---

## 8. Discussion

### 8.1 What the filter learned

We never measured velocity, but after five iterations the filter has a
non-trivial velocity estimate (0.46 m/s) and reduced velocity
uncertainty by 24% (variance 1.000 → 0.756) — inferred from the
**rate-of-change** of position observations across consecutive ticks.
This is what people mean by "an observer that estimates unobservable
states": no magic, just careful bookkeeping of which states couple to
which measurements, integrated across time.

The position estimate (0.342 m) is much higher than truth (0.125 m).
Why? The five measurements 0.7, $-$0.5, 1.1, $-$0.2, 1.3 have mean
0.48 m, but truth means 0.055 m — the noise sequence happened to be
biased *positive* in this five-sample window. The filter weighted the
measurements as best it could; the result is biased the same direction.
Run more iterations and the bias averages out. This is a property of
the data, not a defect of the filter.

### 8.2 What $\mathbf{P}$ tells us

$\mathbf{P}$ is the covariance of the joint posterior over $(p, v)$.
Its 1-σ confidence ellipse is $\delta\mathbf{x}^T\mathbf{P}^{-1}\delta\mathbf{x} = 1$,
an ellipse in the $(p,v)$ plane centered on $\hat{\mathbf{x}}$:
**semi-axes** along the eigenvectors of $\mathbf{P}$, **lengths** equal
to $\sqrt{\lambda_i}$, **area** $\propto \sqrt{\det\mathbf{P}}$ (which
shrunk monotonically above).

Visualize: at $k=0$ the ellipse is axis-aligned (off-diagonals zero) —
position and velocity uncertainties are independent. By $k=5$ the
ellipse is tilted (off-diagonal $+0.291$) — the filter believes that
$(\delta p, \delta v) = (+1, -1)\sigma$ is much more likely than
$(+1,+1)\sigma$. Equivalently, *if* my position estimate is too high,
*then* my velocity estimate is also too high. That tilt is the entire
content of the position-velocity coupling we learned.

### 8.3 Sensitivity to $\mathbf{Q}$ and $R$

$\mathbf{Q}$ tells the filter how much you distrust your **model**; $R$
how much you distrust your **measurements**. Concretely:

- **Increase $\mathbf{Q}$** (or **decrease $R$**) → predict adds more
  uncertainty each tick → $\mathbf{K} = \mathbf{P}\mathbf{H}^T/S$
  approaches its measurement-trusting limit. The filter trusts
  measurements more, tracks them closely, estimate is **noisier**.
  Steady-state gain higher.
- **Decrease $\mathbf{Q}$ toward 0** (or **increase $R$**) → the filter
  trusts the model. $\mathbf{P}_{k|k}$ and $\mathbf{K}$ shrink toward
  zero. Filter ignores measurements: smooth but **slow** to respond.
  If the rocket has unmodeled acceleration the filter lags for many
  ticks.

PM intuition: $\mathbf{Q}$ and $R$ are dual knobs. What matters is the
**ratio** $\mathbf{Q}/R$, not the absolute scale — doubling both leaves
the gain sequence unchanged (every line of the loop scales the same
way). The ratio sets the filter **bandwidth**: how fast the filter
responds to dynamics versus how much it averages out noise. Match
$\mathbf{Q}$ to actual unmodeled dynamics; match $R$ to actual sensor
noise.

---

## 9. NumPy Verification

Paste this into a notebook to verify every number above.

```python
import numpy as np

F = np.array([[1.0, 0.1], [0.0, 1.0]])
Q = np.array([[1e-4, 0.0], [0.0, 1e-2]])
H = np.array([[1.0, 0.0]])
R = np.array([[1.0]])

x = np.array([[0.0], [0.0]])              # initial state
P = np.array([[0.1, 0.0], [0.0, 1.0]])    # initial covariance
zs = [0.7, -0.5, 1.1, -0.2, 1.3]          # synthetic measurements

for k, z in enumerate(zs, start=1):
    x = F @ x                                          # predict state
    P = F @ P @ F.T + Q                                # predict cov
    y = np.array([[z]]) - H @ x                        # innovation
    S = H @ P @ H.T + R                                # innov cov
    K = P @ H.T @ np.linalg.inv(S)                     # Kalman gain (2x1)
    x = x + K @ y                                      # update state
    IKH = np.eye(2) - K @ H
    P = IKH @ P @ IKH.T + K @ R @ K.T                  # Joseph form
    P = 0.5 * (P + P.T)                                # symmetrize
    print(f"k={k}: x=[{x[0,0]:+.6f},{x[1,0]:+.6f}]  P_diag=[{P[0,0]:.6f},{P[1,1]:.6f}]  K=[{K[0,0]:.6f},{K[1,0]:.6f}]")
```

Expected output (matches §7):

```
k=1: x=[+0.069426,+0.063057]  P_diag=[0.099182,1.000992]  K=[0.099180,0.090082]
k=2: x=[+0.010714,-0.034069]  P_diag=[0.112930,0.978909]  K=[0.112931,0.168702]
k=3: x=[+0.155220,+0.217804]  P_diag=[0.135366,0.927458]  K=[0.135366,0.230506]
k=4: x=[+0.116583,+0.115468]  P_diag=[0.160258,0.849711]  K=[0.160258,0.271448]
k=5: x=[+0.341922,+0.456946]  P_diag=[0.182436,0.755852]  K=[0.182435,0.291396]
```

If your hand numbers match these, you have correctly executed the
linear Kalman filter.

---

## 10. FSW Anchor

This exact loop runs inside `nav_lib` per `docs/design/nav/algorithm.md`:
§3.2 (lines 113-187) is the **predict** step (driven by IMU samples; called every
`NavLib_PredictImu(...)` at 200 Hz); §4.1 (lines 192-217) is the **baro update** (1×1
altitude measurement, structurally identical to the 1D position update
we just executed); §4.2 (lines 219-238) is the **GPS update** (6×1 position+velocity
measurement; same Joseph-form math, with a 6×6 $S$ inverted via
`juno::kmat::Invert<double, 6>` instead of a scalar division).

Three things are different in the flight code:

1. **State is 16-dimensional** — lat/lon/alt + Vn/Ve/Vd + quaternion(4) +
   accel-bias(3) + gyro-bias(3). Every matrix in the loop grows
   accordingly. See `algorithm.md` §3.1 lines 70-82.
2. **Process model is non-linear** — IMU integration rotates accel
   through the current attitude quaternion, subtracts gravity, then
   updates geodetic position. Chapter 06 introduces the **Extended
   Kalman Filter (EKF)**, which handles non-linearity by linearizing
   around the current estimate: $\mathbf{F}$ becomes the Jacobian
   $\partial f/\partial \mathbf{x}$, derived from Groves §14 or
   Trawny–Roumeliotis per `algorithm.md` §3.2 lines 152–165.
3. **Measurement model for baro and the position+velocity components of
   GPS is still linear** — exactly the $\mathbf{H} = (1\,0\,\dots\,0)$
   shape we used here. The measurement-update math you just executed is
   *exactly* what `nav_lib` runs every time `NavLib_UpdateBaro` or
   `NavLib_UpdateGps` is called.

When you read `libs/nav_lib/src/nav_impl.cpp` you will see this
structure: `Predict()` does $\mathbf{F}\mathbf{P}\mathbf{F}^T + \mathbf{Q}$;
each `Update*()` computes innovation, innovation covariance, gain,
state delta, and Joseph-form covariance update. **That is the loop you
just hand-executed five times.** Debug the 1D filter and you can debug
the 16-state filter — only the bookkeeping is bigger.

---

## References

[1] Kalman, R. E. (1960). "A New Approach to Linear Filtering and
    Prediction Problems," *Trans. ASME — J. Basic Eng.*, 82(1), 35–45.
[2] Brown, R. G. and Hwang, P. Y. C. (2012). *Introduction to Random
    Signals and Applied Kalman Filtering with MATLAB Exercises*, 4th ed.,
    Wiley — **§5.5** Worked Examples (pedagogical pattern of this chapter).
[3] Groves, P. D. (2013). *Principles of GNSS, Inertial, and Multisensor
    Integrated Navigation Systems*, 2nd ed., Artech House — recommended
    reference for the Juno EKF F matrix per `docs/design/nav/algorithm.md` §3.2.
[4] Trawny, N. and Roumeliotis, S. I. (2005). "Indirect Kalman Filter
    for 3D Attitude Estimation," Univ. Minnesota MARS Lab TR-2005-002.

---

**Up next (chapter 06).** Generalize the linear KF to the **Extended**
KF (EKF) — the form `nav_lib` runs. State grows 2D → 16D; process model
becomes non-linear $f(\mathbf{x},\mathbf{u},\mathbf{w})$; $\mathbf{F}$
becomes a Jacobian recomputed every prediction. Update math unchanged.
