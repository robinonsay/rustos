---
document_type: Tutorial Chapter — The Extended Kalman Filter
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent: docs/tutorials/nav_kalman/index.md
prerequisites: Chapters 01 (linear algebra; matrix calculus), 02 (Gaussians), 03 (state-space), 04 (linear KF derivation), 05 (worked KF example)
target_reader: One software engineer (Robin Onsay) — rusty on linear algebra and probability; no prior nav/controls background
---

# Chapter 06 — The Extended Kalman Filter

> **Where we are.** Chapter 04 derived the five Kalman equations under
> the linear-Gaussian model; chapter 05 walked them through ten cycles
> of a 2-state filter by hand and in NumPy. The world is rarely linear,
> though. The FT1 nav system propagates body-frame accelerations through
> a quaternion attitude — a rotation, fundamentally nonlinear in the
> state. **This chapter extends the linear KF to handle nonlinear
> $\mathbf{f}$ and $\mathbf{h}$ via first-order Taylor linearization at
> the current estimate.** The result is the Extended Kalman Filter
> (EKF). It is the filter the FT1 `nav_lib` IMPL will run at 200 Hz on
> the Pico2 (chapter 11; `docs/design/nav/algorithm.md` §3.2 line 105).

---

## 1. Why the EKF — Where Linearity Fails

The linear KF requires the model to be linear in the state:

$$\mathbf{x}_k = \mathbf{F}\,\mathbf{x}_{k-1} + \mathbf{w}_k,\qquad \mathbf{z}_k = \mathbf{H}\,\mathbf{x}_k + \mathbf{v}_k.$$

Real systems rarely satisfy this. The FT1 nav system has a **nonlinear
process** because attitude propagation rotates body-frame accelerations
into the NED tangent frame, and rotations are intrinsically nonlinear
in the quaternion components (chapter 08 will derive this; `algorithm.md`
§3.2 step 2 line 119 names it). Concretely, `algorithm.md` §3.2 step 2
applies `QuatRotate(q, a_body) → a_ned`, then step 4 integrates `vel +=
a_ned * dt` — the velocity update depends on the attitude state through
a rotation, which is bilinear in $\mathbf{q}$ and $\mathbf{a}$ at best.
There is no $\mathbf{F}$ matrix that captures this exactly.

The FT1 **measurements**, by contrast, **are linear in the state**:

- Baro returns altitude directly (`algorithm.md` §4.1 lines 199–201,
  "$\mathbf{H} = [0\ 0\ 1\ 0\ \ldots\ 0]$" — a single 1.0 in the
  altitude column).
- GPS returns three position components and three velocity components
  directly (`algorithm.md` §4.2 lines 224–230, "$\mathbf{H}$ ($6 \times \mathrm{kInternalDim}$) has 1.0 entries on the rows for the position
  and velocity components and zeros elsewhere").

This split — **nonlinear process, linear measurements** — is the
central fact about the FT1 EKF. It tells us where we need linearization
and where we do not. The predict step needs work; the update step is,
in form, identical to the linear KF.

A natural question: why not just plug the nonlinear $\mathbf{f}$ into
the covariance equation $\mathbf{P}_{k\mid k-1} = \mathbf{F}\mathbf{P}_{k-1\mid k-1}\mathbf{F}^T + \mathbf{Q}$ and call it done? Because **covariance propagation requires a linear map.** The expression $\mathbf{F}\mathbf{P}\mathbf{F}^T$ is the algebraic shape of "push a Gaussian through a linear transform" (chapter 02 §8); it is meaningless when $\mathbf{f}$ is nonlinear. We need a linear approximation of $\mathbf{f}$ near the current estimate. That linear approximation is the **Jacobian**, and the resulting filter is the **Extended Kalman Filter**.

---

## 2. Linearization via Jacobians

### 2.1 Definition

A continuous, differentiable function $\mathbf{f}: \mathbb{R}^n \to \mathbb{R}^n$ near a point $\mathbf{x}_0$ admits a first-order Taylor expansion:

$$\mathbf{f}(\mathbf{x}) \approx \mathbf{f}(\mathbf{x}_0) + \mathbf{F}\,(\mathbf{x} - \mathbf{x}_0),\qquad \mathbf{F} = \frac{\partial \mathbf{f}}{\partial \mathbf{x}}\bigg|_{\mathbf{x}_0}.$$

The matrix $\mathbf{F}$ is the **Jacobian** of $\mathbf{f}$ at $\mathbf{x}_0$. Each entry is a partial derivative:

$$F_{ij} = \frac{\partial f_i}{\partial x_j}\bigg|_{\mathbf{x}_0}.$$

For an $n$-dimensional input and $n$-dimensional output, $\mathbf{F}$ is square ($n \times n$). The approximation is good in a neighborhood of $\mathbf{x}_0$ whose size depends on the second derivatives of $\mathbf{f}$ — the smaller they are (the "less curved" $\mathbf{f}$ is), the larger the neighborhood within which the linear approximation is accurate.

### 2.2 1-D worked example

Take $f(x) = \sin(x)$. The derivative is $f'(x) = \cos(x)$. Evaluate at $x_0 = 0.5$:

$$F = \cos(0.5) \approx 0.8776.$$

The linear approximation near $x_0 = 0.5$ is:

$$\sin(x) \approx \sin(0.5) + 0.8776\,(x - 0.5) \approx 0.4794 + 0.8776\,(x - 0.5).$$

Spot-check at $x = 0.6$: linear says $0.4794 + 0.8776 \cdot 0.1 = 0.5672$. True value: $\sin(0.6) = 0.5646$. Error: $0.0026$, or about half a percent. The linearization is excellent for this step size; it would degrade noticeably by $|x - x_0| \approx 1$ radian. The Jacobian carries no information about how fast the approximation degrades — that is what the second derivative tells you, and that is exactly the EKF's blind spot (§7).

### 2.3 2-D worked example

Take $\mathbf{f}: \mathbb{R}^2 \to \mathbb{R}^2$ with components:

$$f_1(x_1, x_2) = x_1^2 + x_2,\qquad f_2(x_1, x_2) = \sin(x_1)\,x_2.$$

The Jacobian is the matrix of partials:

$$\mathbf{F} = \begin{bmatrix} \partial f_1/\partial x_1 & \partial f_1/\partial x_2 \\ \partial f_2/\partial x_1 & \partial f_2/\partial x_2 \end{bmatrix} = \begin{bmatrix} 2x_1 & 1 \\ \cos(x_1)\,x_2 & \sin(x_1) \end{bmatrix}.$$

Evaluate at $\mathbf{x}_0 = (1, 2)^T$:

$$\mathbf{F}\big|_{(1,2)} = \begin{bmatrix} 2 & 1 \\ \cos(1)\cdot 2 & \sin(1) \end{bmatrix} \approx \begin{bmatrix} 2.000 & 1.000 \\ 1.081 & 0.841 \end{bmatrix}.$$

Each entry is a number. The matrix is recomputed whenever the operating point $\mathbf{x}_0$ changes — which, in the EKF, is **every tick** (§3.3 below).

For an $n$-dimensional state, the Jacobian has $n^2$ entries. For FT1's 16-state EKF, that is 256 entries to recompute every IMU sample (200 Hz). Most of those entries are zero by inspection (e.g., the bias states evolve as random walks, so their Jacobian rows are identity rows; `algorithm.md` §3.2 step 7 lines 141–144). The closed-form derivations in Groves [1, §14.2] and Trawny & Roumeliotis [4, §3.5] enumerate which blocks are nonzero and what they evaluate to.

---

## 3. The EKF Predict Step

### 3.1 The full nonlinear $\mathbf{f}$ for the state, the Jacobian for the covariance

This is the single most important conceptual point in the chapter. **Read it twice.**

Given the previous posterior $(\hat{\mathbf{x}}_{k-1\mid k-1},\,\mathbf{P}_{k-1\mid k-1})$ and an input $\mathbf{u}_{k-1}$ (for FT1: nothing — see chapter 04 notation table; the IMU sample is treated as part of $\mathbf{f}$, not as $\mathbf{u}$), the EKF predict step is:

**State propagation** uses the **full nonlinear** $\mathbf{f}$:

$$\boxed{\;\hat{\mathbf{x}}_{k\mid k-1} = \mathbf{f}(\hat{\mathbf{x}}_{k-1\mid k-1},\,\mathbf{u}_{k-1}).\;}$$

**Covariance propagation** uses the **Jacobian** $\mathbf{F}_k$ evaluated at the current estimate:

$$\boxed{\;\mathbf{F}_k = \frac{\partial \mathbf{f}}{\partial \mathbf{x}}\bigg|_{\hat{\mathbf{x}}_{k-1\mid k-1}},\qquad \mathbf{P}_{k\mid k-1} = \mathbf{F}_k\,\mathbf{P}_{k-1\mid k-1}\,\mathbf{F}_k^T + \mathbf{Q}.\;}$$

**Do not** propagate the state with $\mathbf{F}_k\,\hat{\mathbf{x}}_{k-1\mid k-1}$. That is the linear KF. It will silently lose the nonlinear part of $\mathbf{f}$ and your filter will diverge. The Jacobian $\mathbf{F}_k$ exists **only to push the covariance through a locally-linear approximation of $\mathbf{f}$**; it never replaces $\mathbf{f}$ itself for the mean.

For FT1, $\mathbf{f}$ is the seven-step strapdown loop in `algorithm.md` §3.2 lines 113–144 (bias correction, body→NED rotation, gravity subtraction, velocity integration, position integration in geodetic coordinates, attitude propagation by small-rotation quaternion, bias evolution). The state propagation is therefore that exact loop, applied once per IMU sample. The covariance propagation is the matrix product $\mathbf{F}_k \mathbf{P} \mathbf{F}_k^T + \mathbf{Q}$ where $\mathbf{F}_k$ comes from the closed-form blocks in Groves [1, §14.2] / Trawny & Roumeliotis [4, §3.5].

### 3.2 Where the Gaussian assumption breaks

Pushing a Gaussian through a nonlinear $\mathbf{f}$ does **not** produce a Gaussian. The EKF nevertheless models the prior at $k$ as Gaussian — it is a deliberate approximation. The mean of that approximate Gaussian is $\mathbf{f}(\hat{\mathbf{x}}_{k-1\mid k-1})$; the covariance is the linearized covariance $\mathbf{F}_k \mathbf{P}_{k-1\mid k-1} \mathbf{F}_k^T + \mathbf{Q}$. When $\mathbf{P}_{k-1\mid k-1}$ is small (we are confident in our estimate; the Gaussian "lives in a small ball around $\hat{\mathbf{x}}_{k-1\mid k-1}$"), this approximation is excellent — the linear region of $\mathbf{f}$ covers the bulk of the probability mass. When $\mathbf{P}_{k-1\mid k-1}$ is large, the approximation degrades. This is the EKF's fundamental failure mode (§7).

### 3.3 $\mathbf{F}_k$ is recomputed every tick

The subscript on $\mathbf{F}_k$ is not decorative. Because the Jacobian depends on the linearization point, and the linearization point moves every tick, **the matrix $\mathbf{F}_k$ is re-evaluated on every prediction step**. This is the deepest practical difference from the linear KF:

- **Linear KF:** $\mathbf{F}$ is constant. The covariance trajectory $\{\mathbf{P}_{k\mid k}\}$ can be precomputed offline because no equation in chapter 04 §3 or §4 references the actual measurements (the "covariance is data-independent" property; chapter 04 §7).
- **EKF:** $\mathbf{F}_k$ depends on $\hat{\mathbf{x}}_{k-1\mid k-1}$, which depends on past measurements. The covariance trajectory is **data-dependent**. No offline precomputation is possible.

Operationally: the FT1 IMPL allocates $\mathbf{F}_k$ as a stack-resident `juno::kmat::MAT_T<double, kInternalDim, kInternalDim>` and rewrites every entry at each `NavLib_PredictImu` call. There is no "$\mathbf{F}$" data member on `NAV_LIB_IMPL_T`; the matrix is transient.

---

## 4. The EKF Update Step

### 4.1 General nonlinear form

Given the prior $(\hat{\mathbf{x}}_{k\mid k-1},\,\mathbf{P}_{k\mid k-1})$ and a measurement $\mathbf{z}_k$ generated by $\mathbf{z}_k = \mathbf{h}(\mathbf{x}_k) + \mathbf{v}_k$, $\mathbf{v}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{R})$:

$$\mathbf{H}_k = \frac{\partial \mathbf{h}}{\partial \mathbf{x}}\bigg|_{\hat{\mathbf{x}}_{k\mid k-1}}.$$

**Note carefully**: $\mathbf{H}_k$ is evaluated at the **predicted** state $\hat{\mathbf{x}}_{k\mid k-1}$, **not** at the posterior $\hat{\mathbf{x}}_{k\mid k}$. The posterior does not exist yet at the moment we form $\mathbf{H}_k$ — that is what the update is going to compute. This is a common implementer's bug: evaluating the measurement Jacobian at the wrong operating point will silently bias the filter.

The five EKF update equations have the same shape as the linear KF (chapter 04 §4.3 boxed equations) with $\mathbf{H}_k$ replacing the constant $\mathbf{H}$ and the nonlinear $\mathbf{h}$ used inside the innovation:

$$\mathbf{y}_k = \mathbf{z}_k - \mathbf{h}(\hat{\mathbf{x}}_{k\mid k-1}),$$
$$\mathbf{S}_k = \mathbf{H}_k\,\mathbf{P}_{k\mid k-1}\,\mathbf{H}_k^T + \mathbf{R},$$
$$\mathbf{K}_k = \mathbf{P}_{k\mid k-1}\,\mathbf{H}_k^T\,\mathbf{S}_k^{-1},$$
$$\hat{\mathbf{x}}_{k\mid k} = \hat{\mathbf{x}}_{k\mid k-1} + \mathbf{K}_k\,\mathbf{y}_k,$$
$$\mathbf{P}_{k\mid k} = (\mathbf{I} - \mathbf{K}_k\mathbf{H}_k)\,\mathbf{P}_{k\mid k-1}\,(\mathbf{I} - \mathbf{K}_k\mathbf{H}_k)^T + \mathbf{K}_k\,\mathbf{R}\,\mathbf{K}_k^T \quad \text{(Joseph form)}.$$

The innovation uses the nonlinear $\mathbf{h}$ to predict the measurement; everything else uses the Jacobian $\mathbf{H}_k$ to manipulate covariance. Same pattern as the predict step: **nonlinear function for the mean, Jacobian for the covariance.** Brown & Hwang [2, §6] derive the EKF update equations by repeating the chapter 04 §4 conditioning argument with $\mathbf{h}$ linearized — the algebra is identical once $\mathbf{H}_k$ is in hand.

### 4.2 FT1 special case — linear measurements, constant $\mathbf{H}$

The FT1 baro and GPS measurements are both linear in the state. Concretely:

- **Baro** (`algorithm.md` §4.1 lines 199–201): $\mathbf{h}(\mathbf{x}) = x_3$ (the altitude state, the third entry of `tPosLla`). The Jacobian is a row vector $\mathbf{H} = [0\ 0\ 1\ 0\ \ldots\ 0]$ — a single 1.0 in the altitude column, zeros elsewhere. **It does not depend on $\hat{\mathbf{x}}_{k\mid k-1}$.** It is constant.
- **GPS** (`algorithm.md` §4.2 lines 224–230): $\mathbf{h}(\mathbf{x})$ returns the six position-and-velocity states directly. The Jacobian $\mathbf{H}$ is a $6 \times \mathrm{kInternalDim}$ matrix with 1.0 entries on the rows for the position and velocity states and zeros everywhere else. Also constant.

Because both measurement models are linear, $\mathbf{H}_k = \mathbf{H}$ is the same matrix every tick. **The FT1 EKF update path is essentially identical to the linear KF update path** — same five equations, same boxed Joseph form, same constant $\mathbf{H}$. The only EKF-specific work is in the predict step.

This is excellent news for the IMPL: `NavLib_UpdateBaro` and `NavLib_UpdateGps` can hold their respective $\mathbf{H}$ matrices as compile-time-shaped sparse constants. The implementation is no harder than the chapter 05 worked example. The hard part — the Jacobian $\mathbf{F}_k$ — is confined to `NavLib_PredictImu`.

> **Disclaimer (don't generalize).** The constant-$\mathbf{H}$ simplification is **specific to FT1's sensor suite** — baro reports a state component directly, GPS reports states directly. In other nav systems (bearing-only target tracker, star-tracker reporting body-frame star vectors), $\mathbf{h}$ is genuinely nonlinear in the state and $\mathbf{H}_k$ varies with the state estimate just as $\mathbf{F}_k$ does. FT1 happens to be easy on the measurement side.

---

## 5. Analytic vs Numerical Jacobians

### 5.1 The two ways to compute a Jacobian

There are two routes to $\mathbf{F}_k$ in code:

- **Analytic.** Differentiate $\mathbf{f}$ by hand (or with a computer-algebra system), arrive at a closed-form expression for each non-trivial entry of $\mathbf{F}_k$, transcribe those expressions into C++. Every entry is exact in real arithmetic and bit-stable in floating-point arithmetic given a fixed evaluation order.
- **Numerical (finite-difference).** Approximate each partial derivative as $\partial f_i/\partial x_j \approx [f_i(\mathbf{x}_0 + \epsilon \mathbf{e}_j) - f_i(\mathbf{x}_0)] / \epsilon$ for some small step size $\epsilon$. No symbolic differentiation required — just $n+1$ evaluations of $\mathbf{f}$.

Numerical Jacobians are tempting because they require zero math. They are **forbidden** for `nav_lib`. Two reasons.

### 5.2 The FT1 contract — analytic only

`docs/design/nav/algorithm.md` §3.2 lines 167–173 reads (verbatim):

> Numerical (finite-difference) Jacobians are NOT acceptable for the FT1
> implementation: they introduce non-determinism (step-size choice
> affects the result bit-for-bit) and they cannot meet the SW-TC-NAV-021
> tolerance against an analytic ground-truth reference. The SW-TC-NAV-021
> reference EKF (see §6, `expected_artifacts` of the test case) shall use
> the same normative reference as the IMPL so both parties derive
> bit-identical F matrices.

So:

1. **Determinism (`SW-REQ-NAV-015`).** The choice of $\epsilon$ in a finite-difference Jacobian affects the bit pattern of $\mathbf{F}_k$. Any compiler change, any platform change, any unrelated build flag that touches floating-point ordering can shift $\epsilon$'s effective working precision and break bit-for-bit reproducibility across POSIX and Pico2. Worse: different `libm` implementations of `sin/cos/sqrt` on POSIX glibc vs Pico2 ARM produce slightly different outputs in the lowest bits, and finite differencing amplifies that disagreement.
2. **Test tolerance (`SW-TC-NAV-021`).** The ground-truth reference EKF used to validate `nav_lib` is itself analytic (`algorithm.md` §6 lines 386–391 — an off-line Python NumPy reference run with `numpy.float64`). A finite-difference IMPL would disagree with the analytic reference in the low-significance bits, blowing the tolerance. The only way to converge IMPL and reference is for both to use the same closed-form Jacobian.

### 5.3 The normative references

`algorithm.md` §3.2 lines 145–166 names two peer-reviewed standards as the normative source for the analytic $\mathbf{F}_k$ blocks. Either may be selected, but the chosen reference must be cited in the IMPL TU and used consistently for all entries of $\mathbf{F}_k$:

- **Groves [1, §14.2 "Inertial Navigation System Error Equations"; §14.3.1 "INS Error State EKF"].** Closed-form transition matrices for full-state and error-state EKF formulations of geodetic-frame INS with quaternion attitude. **Recommended for the FT1 implementation** (`algorithm.md` line 158).
- **Trawny & Roumeliotis [4, §3.5, eqs. 147–150].** Quaternion-error-state Jacobian blocks consistent with the Hamilton-convention quaternion used by `juno::kmat`. The URL is reproduced in `algorithm.md` lines 161–162: `https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf` (verified reachable per `algorithm.md` §3.2; this chapter does not re-issue an HTTP request).

When you sit down to write `nav_lib`, you will derive (or transcribe) the closed-form $\mathbf{F}_k$ from one of these references, store the derivation in a comment block in the IMPL TU, and exercise it against the SW-TC-NAV-021 reference. **There is no third path.** Numerical Jacobians do not pass the gate.

Solà [5] is a recommended supplement (it walks the small-perturbation algebra explicitly), but it is non-normative — the bit-stable derivation must come from [1] or [4].

---

## 6. Worked Nonlinear Example — A Pendulum

To see the full EKF predict step in motion with concrete numbers, take a 2-state simple pendulum. The state is $\mathbf{x} = (\theta,\,\dot\theta)^T$ where $\theta$ is the angle from vertical and $\dot\theta$ is the angular rate. The continuous dynamics are:

$$\ddot\theta = -\frac{g}{L}\sin\theta,$$

with $g = 9.81\,\mathrm{m/s^2}$ and length $L = 1\,\mathrm{m}$, so $g/L = 9.81$. Discretize with forward Euler at $\Delta t = 0.1\,\mathrm{s}$:

$$\mathbf{f}(\mathbf{x}) = \begin{bmatrix}\theta + \dot\theta\,\Delta t \\ \dot\theta - 9.81\,\sin(\theta)\,\Delta t\end{bmatrix} = \begin{bmatrix}\theta + 0.1\,\dot\theta \\ \dot\theta - 0.981\,\sin\theta\end{bmatrix}.$$

The Jacobian:

$$\mathbf{F} = \frac{\partial \mathbf{f}}{\partial \mathbf{x}} = \begin{bmatrix} 1 & 0.1 \\ -0.981\,\cos\theta & 1 \end{bmatrix}.$$

The $-0.981\,\cos\theta$ entry is where the nonlinearity lives — its value depends on the operating point.

**Initial state and covariance:** $\hat{\mathbf{x}}_{0\mid 0} = (0.5,\,0)^T$ rad, $\mathrm{rad/s}$. $\mathbf{P}_{0\mid 0} = \mathrm{diag}(0.01,\,0.01)$ (small initial uncertainty). Process noise $\mathbf{Q} = \mathrm{diag}(0,\,0.001)$ (only the rate state takes a random-walk hit).

**Step 1 — propagate the mean using the full nonlinear $\mathbf{f}$:**

$$\hat{\mathbf{x}}_{1\mid 0} = \mathbf{f}(\hat{\mathbf{x}}_{0\mid 0}) = \begin{bmatrix}0.5 + 0.1\cdot 0 \\ 0 - 0.981\,\sin(0.5)\end{bmatrix} = \begin{bmatrix}0.5 \\ -0.4703\end{bmatrix}.$$

(Using $\sin(0.5) \approx 0.4794$, so $-0.981 \cdot 0.4794 = -0.4703$.) **Notice we did not use $\mathbf{F}$ to do this.**

**Step 2 — evaluate the Jacobian $\mathbf{F}_1$ at the previous estimate $\hat{\mathbf{x}}_{0\mid 0} = (0.5,\,0)^T$:**

$$\mathbf{F}_1 = \begin{bmatrix} 1 & 0.1 \\ -0.981\cdot \cos(0.5) & 1 \end{bmatrix} = \begin{bmatrix} 1 & 0.1 \\ -0.8607 & 1 \end{bmatrix},$$

using $\cos(0.5) \approx 0.8776$, so $-0.981 \cdot 0.8776 = -0.8607$.

**Step 3 — propagate the covariance:** $\mathbf{P}_{1\mid 0} = \mathbf{F}_1\,\mathbf{P}_{0\mid 0}\,\mathbf{F}_1^T + \mathbf{Q}$. Carry out the products:

$$\mathbf{F}_1\,\mathbf{P}_{0\mid 0} = \begin{bmatrix} 1 & 0.1 \\ -0.8607 & 1 \end{bmatrix}\begin{bmatrix} 0.01 & 0 \\ 0 & 0.01 \end{bmatrix} = \begin{bmatrix} 0.01 & 0.001 \\ -0.008607 & 0.01 \end{bmatrix},$$

$$(\mathbf{F}_1\,\mathbf{P}_{0\mid 0})\,\mathbf{F}_1^T = \begin{bmatrix} 0.01 & 0.001 \\ -0.008607 & 0.01 \end{bmatrix}\begin{bmatrix} 1 & -0.8607 \\ 0.1 & 1 \end{bmatrix} = \begin{bmatrix} 0.0101 & -0.007607 \\ -0.007607 & 0.017407 \end{bmatrix},$$

(spot-check: top-left $0.01 \cdot 1 + 0.001 \cdot 0.1 = 0.0101$; top-right $0.01 \cdot (-0.8607) + 0.001 \cdot 1 = -0.007607$; bottom-right $-0.008607 \cdot (-0.8607) + 0.01 \cdot 1 \approx 0.007407 + 0.01 = 0.017407$.)

Add $\mathbf{Q} = \mathrm{diag}(0,\,0.001)$:

$$\mathbf{P}_{1\mid 0} = \begin{bmatrix} 0.0101 & -0.007607 \\ -0.007607 & 0.018407 \end{bmatrix}.$$

**Observations.** The predicted mean is $(0.5,\,-0.4703)$ — the pendulum has not moved (rate started at zero), but it has acquired a negative angular rate consistent with falling toward vertical from the displaced position. The covariance has grown and is now non-diagonal (the off-diagonal entries reflect that $\theta$ and $\dot\theta$ are now correlated — knowing one tells you something about the other after one step of dynamics). All of this came out of two ingredients: the nonlinear $\mathbf{f}$ for the mean, the Jacobian $\mathbf{F}_1$ for the covariance.

This is the EKF predict step, end-to-end. The FT1 16-state version is the same recipe applied to a much bigger state vector — chapter 11 walks through it line by line.

---

## 7. When the EKF Works, When It Fails

### 7.1 The good regime — small $\mathbf{P}$

The first-order Taylor expansion is accurate near $\mathbf{x}_0$. The relevant notion of "near" is set by $\mathbf{P}$: roughly, the Gaussian whose covariance is $\mathbf{P}$ has its probability mass concentrated within a few standard deviations of the mean, so we need $\mathbf{f}$ to be approximately linear over a region of that size. When $\mathbf{P}$ is small (we have a tight estimate), the linearization captures essentially all of $\mathbf{f}$'s behavior over the bulk of the prior, and the EKF approximation is excellent.

This is the regime FT1 operates in once the alignment phase completes: position uncertainty is single meters, attitude uncertainty is sub-degree, biases have been pinned down to within their MEMS noise floor. Small $\mathbf{P}$. The EKF is well-justified.

### 7.2 The bad regime — large $\mathbf{P}$, strong nonlinearity

Two ways the EKF can fail:

1. **Strong nonlinearity over the prior support.** If $\mathbf{f}$ has significant curvature over the region where the prior has appreciable probability mass, the linearized covariance $\mathbf{F}_k\,\mathbf{P}\,\mathbf{F}_k^T$ underestimates (or overestimates) the true posterior covariance. The filter becomes overconfident or underconfident; the next innovation is mis-weighted; errors compound.
2. **Operating point far from truth.** The Jacobian is evaluated at $\hat{\mathbf{x}}_{k-1\mid k-1}$, which may be far from the true state $\mathbf{x}_{k-1}$ if the filter has already drifted. Linearizing about the wrong point produces a useless approximation; the filter cannot recover and **diverges**.

Bar-Shalom et al. [7, §10] surveys these failure modes and discusses the diagnostics — primarily innovation-whiteness checks (chapter 04 §7) and normalized innovation squared (NIS) tests against a chi-squared threshold. When the innovation sequence stops looking white, or NIS exceeds threshold for several consecutive measurements, the filter is in trouble.

### 7.3 Alternatives (out of scope for FT1)

When the EKF approximation is inadequate, two main families of estimators are available:

- **Unscented Kalman Filter (UKF).** Replaces the Jacobian with a small set of "sigma points" deterministically chosen to capture the prior's mean and covariance. The points are pushed through the full nonlinear $\mathbf{f}$, and the posterior mean and covariance are recomputed from the transformed points. Captures second-order effects without needing a Jacobian. Cost: typically $2n + 1$ evaluations of $\mathbf{f}$ per predict step.
- **Particle filter.** Represents the prior as a weighted set of samples (particles); each particle is propagated through $\mathbf{f}$; the weights are updated by the measurement likelihood. Handles arbitrary non-Gaussian distributions. Cost: hundreds to thousands of $\mathbf{f}$ evaluations per step; rarely fits a 200 Hz embedded budget.

Neither is in scope for FT1. The FT1 spec pins the EKF (`algorithm.md` §1, `SW-REQ-NAV-018`). The pendulum and the FT1 nav system are both inside the EKF's good regime — the nonlinearity is mild, the IMU rate is high enough that $\mathbf{P}$ stays small between updates, and the divergence-bound check (`algorithm.md` §4.3 lines 240–254, `SW-REQ-NAV-014`) catches the cases where the filter does drift.

### 7.4 FT1 mitigations

The FT1 spec includes three EKF-failure mitigations worth naming explicitly:

- **Divergence bound.** GPS innovations exceeding `tInit.fGpsBoundM` (default 200 m) are rejected and trip the state machine to `Diverged` (`algorithm.md` §4.3). This is a hard cap on how far the operating point can drift before the filter stops integrating GPS.
- **Joseph form for covariance.** Numerical positive-definiteness preservation under round-off (`algorithm.md` §6 line 354). This was chapter 04 §6.
- **Quaternion renormalization.** The unit-norm constraint on the attitude state drifts under Euler-step propagation; the IMPL must re-normalize after each predict step (`algorithm.md` §6 lines 343–350). Chapter 08 will return to this.

These three are not full UKF-quality robustness, but they are sufficient for the FT1 mission profile.

---

## 8. Pseudocode Summary

The full EKF predict + update cycle, paralleling chapter 04 §8:

```
# Inputs:
#   f, h           : nonlinear process and measurement functions
#   F_jacobian(x)  : analytic state Jacobian, evaluated at point x
#   H_jacobian(x)  : analytic measurement Jacobian, evaluated at point x
#                    (for FT1 baro and GPS, returns a constant matrix)
#   Q, R           : process and measurement noise covariances
#   x_post, P_post : posterior at time k-1

For each time step k:

    # --- Predict step (always run) ----------------------------------
    F_k    = F_jacobian(x_post)         # evaluated at x_{k-1|k-1}  [RECOMPUTED]
    x_pred = f(x_post, u)               # full nonlinear state propagation
    P_pred = F_k * P_post * F_k.T + Q   # covariance via linearization

    # --- Update step (run only when measurement z_k is available) --
    if measurement_available(k):
        H_k    = H_jacobian(x_pred)                     # evaluated at x_{k|k-1}  [RECOMPUTED]
        y      = z - h(x_pred)                          # nonlinear h in innovation
        S      = H_k * P_pred * H_k.T + R               # innovation covariance
        K      = P_pred * H_k.T * inv(S)                # Kalman gain
        x_post = x_pred + K * y                         # state update
        P_post = (I - K*H_k) * P_pred * (I - K*H_k).T \
                 + K * R * K.T                          # Joseph form
        P_post = 0.5 * (P_post + P_post.T)              # symmetry enforcement
    else:
        x_post = x_pred                                 # no measurement → predict-only
        P_post = P_pred

    # FT1-specific post-step (chapter 08, algorithm.md §6 lines 343-350):
    quaternion_renormalize(x_post)

    emit(x_post, P_post)
```

**Highlighted differences from chapter 04 §8 (linear KF):**

- Line `F_k = F_jacobian(x_post)`: the Jacobian is **recomputed every tick** at the previous posterior. The linear KF had a constant `F`.
- Line `x_pred = f(x_post, u)`: the **full nonlinear $\mathbf{f}$** propagates the state. The linear KF used `F * x_post`.
- Line `H_k = H_jacobian(x_pred)`: the measurement Jacobian is **recomputed at the predicted state**, not the posterior. (For FT1's linear measurements, `H_jacobian` returns a constant; in the general nonlinear case it does not.)
- Line `y = z - h(x_pred)`: the innovation uses the **nonlinear $\mathbf{h}$** to predict the measurement. The linear KF used `H * x_pred`.

Everything else is identical in shape to chapter 04: same Joseph form, same symmetry enforcement, same predict-only branch when no measurement is available.

---

## 9. FSW Anchor

In FT1's `nav_lib`, this same algorithm runs at the **IMU sample rate of 200 Hz** (`docs/design/nav/algorithm.md` §3.2 line 105). Concretely:

- The predict step's nonlinear $\mathbf{f}$ is the seven-step strapdown loop in `algorithm.md` §3.2 lines 113–144 (chapter 09 will derive the strapdown mechanization in detail).
- The predict step's Jacobian $\mathbf{F}_k$ is the analytic closed-form matrix from Groves [1, §14.2] or Trawny & Roumeliotis [4, §3.5], per the mandate at `algorithm.md` lines 145–173. **Numerical Jacobians are forbidden** by `SW-REQ-NAV-015` and `SW-TC-NAV-021`.
- The update step's nonlinear $\mathbf{h}$ is trivial — a coordinate selection — for both baro (`algorithm.md` §4.1 lines 199–201) and GPS (`algorithm.md` §4.2 lines 224–230). The Jacobian $\mathbf{H}_k$ is therefore a constant sparse matrix; `NavLib_UpdateBaro` and `NavLib_UpdateGps` in the IMPL look essentially like the chapter 05 worked example.
- The covariance update is mandated to use the **Joseph form** plus post-step **symmetry enforcement** (`algorithm.md` §6 lines 352–363), exactly as derived in chapter 04 §6.
- After every predict step, the attitude quaternion is re-normalized (`algorithm.md` §6 lines 343–350).

The pendulum example of §6 is a 2-state stand-in for the 16-state FT1 EKF. The recipe scales: replace the 2×2 with a 16×16, replace the trivial $\mathbf{f}$ with the strapdown loop, replace the trivial $\mathbf{F}_k$ with the Groves analytic blocks, and you have the FT1 predict step. Chapter 11 walks the full mapping symbol-by-symbol.

---

## 10. Key Results

- The EKF extends the linear KF to nonlinear $\mathbf{f}$ and $\mathbf{h}$ by linearizing them at the current estimate.
- **State propagation uses the full nonlinear $\mathbf{f}$** — never replace it with $\mathbf{F}_k\,\hat{\mathbf{x}}$. **Covariance propagation uses the Jacobian $\mathbf{F}_k$** — that is its sole purpose.
- The Jacobian $\mathbf{F}_k$ is recomputed every tick because it depends on the linearization point.
- The measurement Jacobian $\mathbf{H}_k$ is evaluated at the **predicted** state $\hat{\mathbf{x}}_{k\mid k-1}$, not the posterior.
- For FT1, the measurements are linear, so $\mathbf{H}_k$ is a constant sparse matrix and the EKF update path is shape-identical to the linear KF update path.
- The FT1 spec mandates **analytic Jacobians** per `algorithm.md` §3.2 lines 167–173; numerical/finite-difference Jacobians are forbidden on determinism and test-tolerance grounds.
- Normative references for the analytic $\mathbf{F}_k$: Groves [1, §14.2 / §14.3.1] (recommended) or Trawny & Roumeliotis [4, §3.5] (`algorithm.md` lines 145–166).
- The EKF works well when $\mathbf{P}$ is small and $\mathbf{f}$ is mildly nonlinear; it fails when $\mathbf{P}$ is large or the operating point is far from truth.
- FT1 mitigations: divergence-bound check (`algorithm.md` §4.3), Joseph form, quaternion renormalization.

---

## 11. Exercises (worked solutions in chapter 12)

1. Compute the Jacobian of $f(x) = e^x \sin(x)$ at $x = 1$. Linearize around that point and check the linear approximation against the true value at $x = 1.1$ and $x = 1.5$. At what step size does the linearization error exceed 5%?
2. For the 2-D function in §2.3, evaluate the Jacobian at $\mathbf{x}_0 = (0,\,0)^T$ and at $\mathbf{x}_0 = (\pi/2,\,1)^T$. Which entries change? Which stay the same? What does that tell you about the structure of $\mathbf{f}$?
3. Carry the pendulum example of §6 forward by one more step. Use the predicted $\hat{\mathbf{x}}_{1\mid 0} = (0.5,\,-0.4703)$ as the new operating point, recompute $\mathbf{F}_2$ and $\hat{\mathbf{x}}_{2\mid 1}$, and propagate $\mathbf{P}_{2\mid 1}$. By how much does $\mathbf{F}_2$ differ from $\mathbf{F}_1$?
4. Show that for the linear case $\mathbf{f}(\mathbf{x}) = \mathbf{F}\mathbf{x}$, the Jacobian is the constant $\mathbf{F}$ — and the EKF predict equations collapse exactly to the linear KF predict equations of chapter 04 §3.
5. For the FT1 baro measurement, write out the Jacobian $\mathbf{H}$ as a row vector of length 16 (using the state-component order in `algorithm.md` §3.1). Explain why it is constant.
6. Explain in one paragraph why a finite-difference Jacobian fails the `SW-TC-NAV-021` tolerance test. (Hint: chapter 04's "covariance is data-independent" property combined with the bit-stability requirement of `SW-REQ-NAV-015`.)
7. Implement the §8 pseudocode in Python+NumPy for the §6 pendulum example. Run for 100 steps with synthetic angle measurements (linear $\mathbf{h}(\mathbf{x}) = \theta$). Compare the EKF estimate against ground truth; plot the angle error and the diagonal entries of $\mathbf{P}$ over time.

---

## 12. Citations

- **[1, §14.2; §14.3.1]** — Groves, *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*. Closed-form analytic transition matrices for full-state and error-state INS EKFs; **recommended primary reference for FT1's analytic $\mathbf{F}_k$** per `algorithm.md` §3.2 lines 152–158. Used in §5.3 and §9.
- **[2, §6]** — Brown & Hwang, *Introduction to Random Signals and Applied Kalman Filtering*. EKF derivation by linearization-and-conditioning (the chapter 04 §4 argument repeated with $\mathbf{H}_k$ in place of $\mathbf{H}$). Used in §4.1.
- **[4, §3.5, eqs. 147–150]** — Trawny & Roumeliotis, "Indirect Kalman filter for 3D attitude estimation," MARS Lab Tech. Rep. TR-2005-002 (2005). URL: `https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf` (verified reachable per `algorithm.md` §3.2 lines 161–162). Quaternion-error-state Jacobian blocks consistent with the Hamilton-convention quaternion. Used in §5.3 and §9.
- **[5]** — Solà, "Quaternion kinematics for the error-state Kalman filter," arXiv:1711.02508 (2017). Recommended supplement; non-normative. Used in §5.3.
- **[7, §10]** — Bar-Shalom, Li, Kirubarajan, *Estimation with Applications to Tracking and Navigation*. EKF failure-mode survey and innovation-whiteness diagnostics. Used in §7.2.
- **`docs/design/nav/algorithm.md` §3.2 lines 145–166** — analytic-Jacobian mandate and normative-reference list (Groves [1] / Trawny [4]). Quoted in §5.3.
- **`docs/design/nav/algorithm.md` §3.2 lines 167–173** — numerical-Jacobian prohibition. Quoted verbatim in §5.2.
- **`docs/design/nav/algorithm.md` §4.1 lines 199–201** — baro measurement linearity ($\mathbf{H}$ constant). Cited in §1, §4.2, §9.
- **`docs/design/nav/algorithm.md` §4.2 lines 224–230** — GPS measurement linearity ($\mathbf{H}$ constant). Cited in §1, §4.2, §9.
- **`docs/design/nav/algorithm.md` §6 lines 343–363** — quaternion renormalization, Joseph form, symmetry enforcement. Cited in §7.4, §8, §9.

<!-- @{"design": ["SW-REQ-NAV-018"]} -->
