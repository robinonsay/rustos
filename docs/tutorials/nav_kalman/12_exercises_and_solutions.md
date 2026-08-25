# Chapter 12 — Exercises and Worked Solutions

This chapter is your self-check. Nine exercises in difficulty-graded order
(easy → medium → hard → bonus implementation prep), each with a full
step-by-step solution. References to chapters 01–11 and to the literature
are inline. Where a numerical answer is given, it has been cross-checked
by hand and (for non-trivial cases) by NumPy.

References used in this chapter: Groves 2013 [1], Brown & Hwang 2012 [2],
Trawny & Roumeliotis 2005 [4], Solà 2017 [5]. The Trawny technical
report TR-2005-002 is hosted at the University of Minnesota MARS Lab:
`https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf`
(verified reachable 2026-05-08).

---

## Exercise 1 — Easy: Matrix multiplication and transpose

Compute $\mathbf{A}\mathbf{B}$ and $\mathbf{B}^T\mathbf{A}^T$ for
$\mathbf{A} = \begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}$,
$\mathbf{B} = \begin{pmatrix} 5 & 6 \\ 7 & 8 \end{pmatrix}$.
Verify $(\mathbf{A}\mathbf{B})^T = \mathbf{B}^T \mathbf{A}^T$ (this is the
"transpose-product reversal" identity used throughout chapter 04 §3 in the
covariance update $\mathbf{F}\mathbf{P}\mathbf{F}^T$).

**Solution.**

Compute $\mathbf{AB}$ by the row-times-column rule (chapter 01 §3.1):

- $(\mathbf{AB})_{11} = 1{\cdot}5 + 2{\cdot}7 = 5 + 14 = 19$
- $(\mathbf{AB})_{12} = 1{\cdot}6 + 2{\cdot}8 = 6 + 16 = 22$
- $(\mathbf{AB})_{21} = 3{\cdot}5 + 4{\cdot}7 = 15 + 28 = 43$
- $(\mathbf{AB})_{22} = 3{\cdot}6 + 4{\cdot}8 = 24 + 32 = 50$

So $\mathbf{AB} = \begin{pmatrix} 19 & 22 \\ 43 & 50 \end{pmatrix}$, and
$(\mathbf{AB})^T = \begin{pmatrix} 19 & 43 \\ 22 & 50 \end{pmatrix}$.

Now $\mathbf{B}^T = \begin{pmatrix} 5 & 7 \\ 6 & 8 \end{pmatrix}$ and
$\mathbf{A}^T = \begin{pmatrix} 1 & 3 \\ 2 & 4 \end{pmatrix}$:

- $(\mathbf{B}^T\mathbf{A}^T)_{11} = 5{\cdot}1 + 7{\cdot}2 = 19$
- $(\mathbf{B}^T\mathbf{A}^T)_{12} = 5{\cdot}3 + 7{\cdot}4 = 43$
- $(\mathbf{B}^T\mathbf{A}^T)_{21} = 6{\cdot}1 + 8{\cdot}2 = 22$
- $(\mathbf{B}^T\mathbf{A}^T)_{22} = 6{\cdot}3 + 8{\cdot}4 = 50$

So $\mathbf{B}^T\mathbf{A}^T = \begin{pmatrix} 19 & 43 \\ 22 & 50 \end{pmatrix}$,
which equals $(\mathbf{AB})^T$. The identity holds, as it must in
general (see chapter 01 §3.4 for the proof).

---

## Exercise 2 — Easy: Mean and variance from samples

Given the samples $\{2.1, 2.3, 1.9, 2.0, 2.7\}$, compute the sample mean
$\bar{x}$ and the unbiased sample variance $s^2$ using
$\bar{x} = \tfrac{1}{n} \sum x_i$ and
$s^2 = \tfrac{1}{n-1} \sum (x_i - \bar{x})^2$ (chapter 02 §3).

**Solution.**

Sample mean ($n = 5$): $\bar{x} = (2.1 + 2.3 + 1.9 + 2.0 + 2.7)/5 = 11.0/5 = 2.2$.

Deviations and squared deviations:

| $x_i$ | $x_i - \bar{x}$ | $(x_i - \bar{x})^2$ |
|-------|-----------------|---------------------|
| 2.1   | $-0.1$          | $0.01$              |
| 2.3   | $+0.1$          | $0.01$              |
| 1.9   | $-0.3$          | $0.09$              |
| 2.0   | $-0.2$          | $0.04$              |
| 2.7   | $+0.5$          | $0.25$              |
| sum   |                 | $0.40$              |

Unbiased variance: $s^2 = 0.40 / (5-1) = 0.40/4 = 0.10$. Standard deviation
$s = \sqrt{0.10} \approx 0.3162$.

NumPy 2-line cross-check:

```python
import numpy as np
x = np.array([2.1, 2.3, 1.9, 2.0, 2.7])
print(x.mean(), x.var(ddof=1))   # -> 2.2  0.10
```

The `ddof=1` flag is critical — `np.var` defaults to the biased $1/n$
estimator. Chapter 02 §3 explains why we use $1/(n-1)$ for the unbiased
estimator.

---

## Exercise 3 — Medium: Conditional Gaussian (2D)

Given the joint Gaussian
$\begin{pmatrix} X \\ Y \end{pmatrix} \sim \mathcal{N}\!\left(\begin{pmatrix} 1 \\ 2 \end{pmatrix}, \begin{pmatrix} 4 & 1 \\ 1 & 9 \end{pmatrix}\right)$,
compute $E[X \mid Y = 5]$ and $\mathrm{Var}(X \mid Y = 5)$ using the
conditional-Gaussian formula from chapter 02 §9.

**Solution.**

The conditional formula (chapter 02 §9, eq. 2.21) is:

$$
\mu_{X\mid Y} = \mu_X + \Sigma_{XY}\,\Sigma_{YY}^{-1}\,(y - \mu_Y), \qquad
\Sigma_{X\mid Y} = \Sigma_{XX} - \Sigma_{XY}\,\Sigma_{YY}^{-1}\,\Sigma_{YX}.
$$

Read off the blocks: $\mu_X = 1$, $\mu_Y = 2$, $\Sigma_{XX} = 4$,
$\Sigma_{YY} = 9$, $\Sigma_{XY} = 1$ (and $\Sigma_{YX} = 1$ by symmetry).
Substitute $y = 5$:

- $E[X \mid Y = 5] = 1 + (1)(1/9)(5 - 2) = 1 + 3/9 = 1 + 1/3 \approx 1.3333$.
- $\mathrm{Var}(X \mid Y = 5) = 4 - (1)(1/9)(1) = 4 - 1/9 = 35/9 \approx 3.8889$.

Sanity check: $\mathrm{cov}(X,Y) = +1 > 0$, so observing a $Y$ above its
mean ($5 > 2$) should pull the $X$ posterior mean above its prior mean of
$1$ — it does ($1.333 > 1$). The conditional variance is smaller than
the prior $\Sigma_{XX} = 4$ — observing $Y$ tells us a little about $X$.
This is the same shrinkage that the Kalman update applies in chapter 04
§6: every measurement reduces (or preserves) covariance, never grows it.

---

## Exercise 4 — Medium: Derive the scalar Kalman gain

For a 1-D state $x$ with process model $x_{k+1} = x_k + w_k$,
$w_k \sim \mathcal{N}(0, q)$, and measurement model $z_k = x_k + v_k$,
$v_k \sim \mathcal{N}(0, r)$, derive the Kalman gain $K_k$ from chapter
04's general formula $\mathbf{K}_k = \mathbf{P}_{k\mid k-1}\mathbf{H}^T(\mathbf{H}\mathbf{P}_{k\mid k-1}\mathbf{H}^T + \mathbf{R})^{-1}$.

**Solution.**

In 1-D, every quantity is a scalar: $\mathbf{P}_{k\mid k-1} \to P_{k\mid k-1}$,
$\mathbf{H} \to H = 1$ (the measurement is $z = x$), $\mathbf{R} \to r$.
Substitute into the general gain expression:

$$
K_k = P_{k\mid k-1}\,(1)\,\big[(1)\,P_{k\mid k-1}\,(1) + r\big]^{-1}
    = \frac{P_{k\mid k-1}}{P_{k\mid k-1} + r}.
$$

This recovers the scalar Kalman gain shown in chapter 04 §6 and Brown &
Hwang [2] §4.2. Two intuitive limits worth noting:

- $r \to 0$ (perfect measurement): $K_k \to 1$ — trust the measurement
  fully; posterior state equals $z_k$.
- $r \to \infty$ (useless measurement): $K_k \to 0$ — keep the prior;
  measurement contributes nothing.

The gain is the relative weight of the prior covariance vs. the
measurement noise. Same shape carries through to the matrix case.

---

## Exercise 5 — Medium: 2-iteration KF by hand

Repeat one tick of the chapter 05 worked example by hand from a different
initial condition. Use chapter 05 §1's matrices:
$\mathbf{F} = \begin{pmatrix} 1 & 0.1 \\ 0 & 1 \end{pmatrix}$,
$\mathbf{Q} = \begin{pmatrix} 0.001 & 0 \\ 0 & 0.01 \end{pmatrix}$,
$\mathbf{H} = \begin{pmatrix} 1 & 0 \end{pmatrix}$, $R = 0.04$. Start from
$\hat{\mathbf{x}}_{0\mid 0} = [1,\,0]^T$,
$\mathbf{P}_{0\mid 0} = \mathbf{I}_2$. Apply one predict + one update with
measurement $z_1 = 0.15$. Compute $\hat{\mathbf{x}}_{1\mid 1}$ and
$\mathbf{P}_{1\mid 1}$.

**Solution.**

**Predict.**

$$
\hat{\mathbf{x}}_{1\mid 0} = \mathbf{F}\hat{\mathbf{x}}_{0\mid 0}
= \begin{pmatrix} 1 & 0.1 \\ 0 & 1 \end{pmatrix}\begin{pmatrix} 1 \\ 0 \end{pmatrix}
= \begin{pmatrix} 1 \\ 0 \end{pmatrix}.
$$

$\mathbf{F}\mathbf{P}_{0\mid 0}\mathbf{F}^T = \mathbf{F}\mathbf{I}\mathbf{F}^T = \mathbf{F}\mathbf{F}^T$:

$$
\mathbf{F}\mathbf{F}^T = \begin{pmatrix} 1 & 0.1 \\ 0 & 1 \end{pmatrix}\begin{pmatrix} 1 & 0 \\ 0.1 & 1 \end{pmatrix}
= \begin{pmatrix} 1.01 & 0.1 \\ 0.1 & 1 \end{pmatrix}.
$$

Add $\mathbf{Q}$: $\mathbf{P}_{1\mid 0} = \begin{pmatrix} 1.011 & 0.1 \\ 0.1 & 1.01 \end{pmatrix}$.

**Update.**

Innovation: $y = z_1 - \mathbf{H}\hat{\mathbf{x}}_{1\mid 0} = 0.15 - 1 = -0.85$.

Innovation covariance: $S = \mathbf{H}\mathbf{P}_{1\mid 0}\mathbf{H}^T + R
= P_{11} + R = 1.011 + 0.04 = 1.051$.

Kalman gain: $\mathbf{K} = \mathbf{P}_{1\mid 0}\mathbf{H}^T / S
= \begin{pmatrix} P_{11} \\ P_{21} \end{pmatrix} / S
= \begin{pmatrix} 1.011 \\ 0.1 \end{pmatrix}/1.051
\approx \begin{pmatrix} 0.96194 \\ 0.09515 \end{pmatrix}$.

State update:
$\hat{\mathbf{x}}_{1\mid 1} = \hat{\mathbf{x}}_{1\mid 0} + \mathbf{K} y
= \begin{pmatrix} 1 \\ 0 \end{pmatrix} + \begin{pmatrix} 0.96194 \\ 0.09515 \end{pmatrix}(-0.85)
\approx \begin{pmatrix} 0.1823 \\ -0.0809 \end{pmatrix}$.

Covariance update (simple form): $\mathbf{P}_{1\mid 1} = (\mathbf{I} - \mathbf{KH})\mathbf{P}_{1\mid 0}$.

$\mathbf{KH} = \begin{pmatrix} 0.96194 & 0 \\ 0.09515 & 0 \end{pmatrix}$, so
$\mathbf{I} - \mathbf{KH} = \begin{pmatrix} 0.03806 & 0 \\ -0.09515 & 1 \end{pmatrix}$.

Multiply by $\mathbf{P}_{1\mid 0}$:

$$
\mathbf{P}_{1\mid 1} \approx \begin{pmatrix} 0.0385 & 0.00381 \\ 0.00381 & 1.00049 \end{pmatrix}.
$$

Note the position variance dropped from $1.011$ to $\approx 0.038$ — the
measurement was strong (low $R$) and shrunk the position uncertainty
sharply. The velocity variance barely moved because we have no direct
velocity measurement; only the cross-covariance with position let the
filter learn anything about velocity. (See chapter 05 §3 for the
side-by-side comparison.)

---

## Exercise 6 — Hard: Compute a Jacobian

For the nonlinear function
$\mathbf{f}(x_1, x_2) = (x_1\cos x_2,\ x_1\sin x_2)^T$, derive the
Jacobian analytically. Evaluate at $(x_1, x_2) = (2, \pi/4)$. Verify
against a finite-difference approximation with $h = 10^{-5}$.

**Solution.**

The Jacobian is the $2\times 2$ matrix of partial derivatives (chapter 06
§2):

$$
\mathbf{J}(\mathbf{x}) = \begin{pmatrix}
\partial f_1/\partial x_1 & \partial f_1/\partial x_2 \\
\partial f_2/\partial x_1 & \partial f_2/\partial x_2
\end{pmatrix}
= \begin{pmatrix} \cos x_2 & -x_1\sin x_2 \\ \sin x_2 & \phantom{-}x_1\cos x_2 \end{pmatrix}.
$$

At $(2, \pi/4)$ with $\cos(\pi/4) = \sin(\pi/4) = \sqrt{2}/2 \approx 0.7071068$:

$$
\mathbf{J}(2, \pi/4) \approx \begin{pmatrix} 0.7071068 & -1.4142136 \\ 0.7071068 & \phantom{-}1.4142136 \end{pmatrix}.
$$

Finite-difference cross-check using the central difference (chapter 06 §6,
eq. 6.4) with $h = 10^{-5}$:

$$
\hat{J}_{ij} = \frac{f_i(\mathbf{x} + h\mathbf{e}_j) - f_i(\mathbf{x} - h\mathbf{e}_j)}{2h}.
$$

Computed in NumPy with `numpy.float64`:

| Entry | Analytic | FD ($h=10^{-5}$) | Agreement |
|-------|----------|------------------|-----------|
| $J_{11}$ | $0.7071068$ | $0.7071068$ | 7 digits |
| $J_{12}$ | $-1.4142136$ | $-1.4142136$ | 7 digits |
| $J_{21}$ | $0.7071068$ | $0.7071068$ | 7 digits |
| $J_{22}$ | $1.4142136$ | $1.4142136$ | 7 digits |

The match is to ~7 digits with $h = 10^{-5}$ — better than the 5-digit
target. Section 6 §6 walks through why central differences scale as
$O(h^2)$ in truncation error and why $h \approx 10^{-5}$ is the sweet
spot for `double`-precision (smaller $h$ amplifies round-off).

---

## Exercise 7 — Hard: Quaternion vector rotation by hand

Rotate the body-frame vector $\mathbf{v}^B = (1, 0, 0)$ (forward in body)
by the unit quaternion $\mathbf{q} = (\cos 30°, 0, \sin 30°, 0) =
(0.8660254, 0, 0.5, 0)$ — a $60°$ pitch rotation about the body $y$-axis.
Compute $\mathbf{v}^N = \mathbf{q} \otimes [0;\,\mathbf{v}^B] \otimes \mathbf{q}^*$
using the Hamilton product (chapter 07 §4).

**Solution.**

Half-angle convention: a $60°$ rotation gives a quaternion with $\cos(60°/2) =
\cos 30°$ in the scalar slot. Pure-vector quaternion form of $\mathbf{v}^B$:
$\tilde{\mathbf{v}}^B = (0, 1, 0, 0)$. Conjugate: $\mathbf{q}^* = (0.8660254, 0, -0.5, 0)$.

**Step 1: $\mathbf{q} \otimes \tilde{\mathbf{v}}^B$.** Using the Hamilton product
(chapter 07 eq. 7.3) with $\mathbf{q} = (w_1,x_1,y_1,z_1) = (0.866, 0, 0.5, 0)$
and $\tilde{\mathbf{v}}^B = (w_2,x_2,y_2,z_2) = (0, 1, 0, 0)$:

- $w = w_1w_2 - x_1x_2 - y_1y_2 - z_1z_2 = 0 - 0 - 0 - 0 = 0$
- $x = w_1x_2 + x_1w_2 + y_1z_2 - z_1y_2 = 0.866 + 0 + 0 - 0 = 0.866$
- $y = w_1y_2 - x_1z_2 + y_1w_2 + z_1x_2 = 0 - 0 + 0 + 0 = 0$
- $z = w_1z_2 + x_1y_2 - y_1x_2 + z_1w_2 = 0 + 0 - 0.5 + 0 = -0.5$

Intermediate: $\mathbf{q}\otimes\tilde{\mathbf{v}}^B = (0, 0.866, 0, -0.5)$.

**Step 2: that result $\otimes \mathbf{q}^*$.** Now
$(w_1,x_1,y_1,z_1) = (0, 0.866, 0, -0.5)$ and
$(w_2,x_2,y_2,z_2) = (0.866, 0, -0.5, 0)$:

- $w = 0\cdot 0.866 - 0.866\cdot 0 - 0\cdot(-0.5) - (-0.5)\cdot 0 = 0$
- $x = 0\cdot 0 + 0.866\cdot 0.866 + 0\cdot 0 - (-0.5)(-0.5) = 0.75 - 0.25 = 0.5$
- $y = 0\cdot(-0.5) - 0.866\cdot 0 + 0\cdot 0.866 + (-0.5)\cdot 0 = 0$
- $z = 0\cdot 0 + 0.866\cdot(-0.5) - 0\cdot 0 + (-0.5)\cdot 0.866 = -0.433 - 0.433 = -0.866$

Result: $\mathbf{v}^N = (0.5,\ 0,\ -0.866)$.

**Sanity check vs. DCM (chapter 08 §3).** A right-hand rotation by $\theta = 60°$
about the body $y$-axis yields the rotation matrix
$\mathbf{R}_y(\theta) = \begin{pmatrix} \cos\theta & 0 & \sin\theta \\ 0 & 1 & 0 \\ -\sin\theta & 0 & \cos\theta \end{pmatrix}$.
Applied to $(1, 0, 0)^T$ gives $(\cos 60°,\,0,\,-\sin 60°) = (0.5,\,0,\,-0.866)$ — same answer.

In NED ($z$ is **down**), the third component $-0.866$ corresponds to motion
in the $-z$ direction, i.e., **upward**. So a $+60°$ pitch about the body
$y$-axis tilts the body's nose vector ($\mathbf{v}^B = $ forward) upward —
consistent with the right-hand-rule convention used in chapter 08 §2.
(Note: the brief described this as "forward + down"; given NED's
$z$-down convention, $-0.866$ along $z$ is forward + up. The math is
right; the textual description in NED is "forward + up.")

---

## Exercise 8 — Hard: Baro innovation covariance for FT1

The FT1 baro measurement model is $z = x.\text{altitude}$ with
$\mathbf{H} = [\,0\ \,0\ \,1\ \,0\ \cdots\ 0\,]$ (a $1\times \texttt{kInternalDim}$
row vector with a single $1$ in the altitude column; see
[`algorithm.md`](../../design/nav/algorithm.md) §4.1, lines 198–213).
Given the post-predict covariance has $P_{33} = 9~\text{m}^2$ (altitude
variance) and $\mathbf{R} = \sigma_{\text{baro}}^2 = 1.5^2 = 2.25~\text{m}^2$,
compute the innovation covariance $\mathbf{S}$ and the altitude row of
the Kalman gain.

**Solution.**

Innovation covariance: $\mathbf{S} = \mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R}$.
Since $\mathbf{H}$ has a single $1$ in position 3 and zeros elsewhere,
$\mathbf{H}\mathbf{P}\mathbf{H}^T$ extracts $P_{33}$ — it is a scalar:

$$
S = P_{33} + R = 9 + 2.25 = 11.25~\text{m}^2.
$$

Kalman gain: $\mathbf{K} = \mathbf{P}\mathbf{H}^T / S$. The vector
$\mathbf{P}\mathbf{H}^T$ extracts the third **column** of $\mathbf{P}$
(again because of the single non-zero in $\mathbf{H}$). The diagonal
entry $K_3$ — the gain applied to the altitude state — is:

$$
K_3 = P_{33} / S = 9 / 11.25 = 0.8.
$$

So 80% of the baro innovation is applied to the altitude state. The
other entries of $\mathbf{K}$ are non-zero through the off-diagonal
entries of column 3 of $\mathbf{P}$ — they correlate altitude with
vertical velocity, vertical accel bias, etc., and let the baro update
adjust those states too (the same cross-covariance mechanism that let
position-only measurements update velocity in exercise 5).

This scalar $S$ form is why the IMPL (per `algorithm.md` §4.1 step 2)
treats the baro update as a single division rather than a $1\times 1$
matrix inverse — it's the same operation, expressed scalarly for clarity.

---

## Exercise 9 — Bonus / Implementation Prep: Filling in the F Jacobian for FT1

Goal: derive the analytic state-transition Jacobian
$\mathbf{F} = \partial \mathbf{f}/\partial \mathbf{x}$ for FT1's 16-state
EKF (see [`algorithm.md`](../../design/nav/algorithm.md) §3.2 step 8) using
Groves 2013 [1] §14.2 as the normative reference. Treat this as a
worksheet: fill in the cells, cite the literature equation for each
non-zero block, and **verify** the result against an autodiff reference
(exercise 6's finite-difference technique scaled up).

### Worksheet (16×16 sparsity grid)

Columns and rows are grouped by state-vector blocks (chapter 11 §3 shows
the same composition):

| | $\mathbf{p}$ (3) | $\mathbf{v}$ (3) | $\mathbf{q}$ (4) | $\mathbf{b}_a$ (3) | $\mathbf{b}_g$ (3) |
|---|---|---|---|---|---|
| $\dot{\mathbf{p}}$ (3) | 0 | $\mathbf{I}_3$ ① | 0 | 0 | 0 |
| $\dot{\mathbf{v}}$ (3) | 0 | 0 | $A_{vq}$ ② | $-\mathbf{R}_{B\to N}$ ③ | 0 |
| $\dot{\mathbf{q}}$ (4) | 0 | 0 | $A_{qq}(\boldsymbol{\omega})$ ④ | 0 | $A_{qb}(\mathbf{q})$ ⑤ |
| $\dot{\mathbf{b}}_a$ (3) | 0 | 0 | 0 | 0 ⑥ | 0 |
| $\dot{\mathbf{b}}_g$ (3) | 0 | 0 | 0 | 0 | 0 ⑦ |

Seven non-zero (or pinned-zero) blocks, numbered ① through ⑦.

### Block-by-block notes

- **① $\partial\dot{\mathbf{p}}/\partial \mathbf{v} = \mathbf{I}_3$.**
  Position rate-of-change is velocity (algorithm.md §3.2 step 4, $\dot{\mathbf{p}} = \mathbf{v}$).
  No literature lookup needed.
- **② $\partial\dot{\mathbf{v}}/\partial \mathbf{q}$.** Body acceleration
  enters NED via the quaternion rotation, so this block is
  $\partial(\mathbf{R}_{B\to N}(\mathbf{q})\,\mathbf{a}_{\text{body}})/\partial\mathbf{q}$.
  This is the most complex block. Closed form: Groves [1] eq. 14.45 (rate
  matrix), or equivalently Trawny [4] eq. 147 in error-state form. If you
  use error-state EKF (kInternalDim = 15, see algorithm.md §3.1), this
  block becomes $-[\mathbf{R}_{B\to N}\mathbf{a}_{\text{body}}\times]$ (a
  3×3 skew-symmetric matrix) — substantially simpler than the 3×4
  full-state form.
- **③ $\partial\dot{\mathbf{v}}/\partial\mathbf{b}_a = -\mathbf{R}_{B\to N}$.**
  Bias is subtracted in body frame, then rotated into NED. Sign is negative
  because bias is **subtracted** in the bias-correction step (algorithm.md §3.2 step 1).
  Cite Groves [1] eq. 14.46.
- **④ $\partial\dot{\mathbf{q}}/\partial\mathbf{q} = \tfrac{1}{2}\Omega(\boldsymbol{\omega})$.**
  Where $\Omega(\boldsymbol{\omega})$ is the $4\times 4$ quaternion rate
  matrix from $\boldsymbol{\omega} = \tilde{\boldsymbol{\omega}}_{\text{meas}} - \mathbf{b}_g$.
  Trawny [4] eq. 148.
- **⑤ $\partial\dot{\mathbf{q}}/\partial\mathbf{b}_g$.** A function of the
  current quaternion (not of $\mathbf{q}$ alone). Trawny [4] eq. 149.
- **⑥, ⑦ Bias dynamics.** Both bias states are random walks with no
  deterministic dynamics (algorithm.md §3.2 step 7), so all rows of these
  blocks are zero. Their uncertainty grows through $\mathbf{Q}$ only, not
  through $\mathbf{F}\mathbf{P}\mathbf{F}^T$. This is **NOT a derivation
  shortcut** — it is a modeling decision pinned in the algorithm spec.

### Verification path

Per `algorithm.md` lines 386–392, SW-TC-NAV-021 specifies a NumPy
ground-truth EKF with `numpy.float64` and a fixed seed. To verify your
hand-derived $\mathbf{F}$:

1. Implement $\mathbf{F}$ in C++ using `juno::kmat::MAT_T<double, 16, 16>`
   (or 15×15 for error-state).
2. Implement the same $\mathbf{F}$ in Python via SymPy (symbolic
   differentiation of $\mathbf{f}$) or via JAX/autograd autodiff on the
   numerical $\mathbf{f}$.
3. At several test states (nominal hover, $30°$ pitch, near-singular
   quaternions $|\mathbf{q}-\hat{\mathbf{q}}|<10^{-6}$, and large bias
   $|\mathbf{b}|=0.1$), compare entry-by-entry. Target agreement: $\sim 10^{-9}$
   relative or absolute on every entry.
4. Cross-check against finite differences (exercise 6 technique) with
   $h = 10^{-5}$ — agreement to $\sim 5$–$7$ digits. Larger
   disagreement points to a missing block or a sign error.

### What this exercise deliberately does not do

It does not derive each closed-form block — Trawny [4] §3.5 runs ~6 pages
and Groves [1] §14.2.4 runs longer. The point is to confirm the
**sparsity pattern**, memorize the **seven non-zero blocks** with their
equation numbers, and plan the **verification path** before writing
code. This is the implementation-prep checklist for USER-NAV-LIB.

---

## Where next?

You're done with the tutorial. Read `docs/design/nav/design.md` and
`algorithm.md` end-to-end, set up your SymPy reference EKF, then code
`nav_lib` against SW-TC-NAV-001..023 using the chapter 11 §6 checklist.
