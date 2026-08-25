---
chapter: 02
title: Probability and Gaussians
audience: software engineer rusty on probability; just finished chapter 01 (linear algebra)
prerequisite: chapter 01 — Linear Algebra Refresher
next: chapter 03 — State-Space Systems
---

# Chapter 02 — Probability and Gaussians

This chapter is the probability prerequisite for the rest of the tutorial.
By the end you can read the multivariate-Gaussian PDF, interpret the
covariance matrix geometrically, and recognize the conditional-Gaussian
formula that turns out to be the Kalman update step in disguise.

Prereqs: chapter 01 (vectors, matrices, transpose, inverse, determinant,
symmetric / positive-definite) and high-school calculus. References in
brackets point to the master bibliography in `index.md`. Brown & Hwang [2]
is our primary text for the Bayesian update of §10.

---

## 1. Random variables

A **random variable** (RV) is a name for an uncertain quantity. Two flavors:

- **Discrete** RVs take values from a countable set. Example: one die
  roll, $X \in \{1, \dots, 6\}$, described by a **probability mass function**
  (PMF) $p(x)$ with $\sum_x p(x) = 1$.
- **Continuous** RVs take real values. Example: the noise on one IMU
  accelerometer reading, described by a **probability density function**
  (PDF) $p(x)$ with $\int p(x)\,dx = 1$ and $P(a \le X \le b) = \int_a^b p(x)\,dx$.
  The PDF is a density, not a probability.

**Sensor-noise motivator.** When the IMU reports $2.50 \text{ m/s}^2$, the
truth is something like $2.50 \pm 0.06 \text{ m/s}^2$ — a deterministic
component plus additive noise drawn from a distribution. The rest of this
chapter characterizes that distribution.

---

## 2. Expectation, variance, standard deviation

The **expectation** (or mean) is the long-run average:

$$
E[X] = \sum_x x\,p(x) \quad \text{(discrete)}, \qquad E[X] = \int x\,p(x)\,dx \quad \text{(continuous)}.
$$

**Variance** measures spread; **standard deviation** $\sigma = \sqrt{\text{Var}(X)}$ shares the units of $X$:

$$
\text{Var}(X) = E\!\left[(X - E[X])^2\right], \qquad \sigma = \sqrt{\text{Var}(X)}.
$$

Expectation is linear: $E[aX + bY + c] = a\,E[X] + b\,E[Y] + c$. Variance
is **not**: $\text{Var}(aX) = a^2 \text{Var}(X)$.

### Worked example: 100 die rolls

Fair six-sided die, PMF $p(k) = 1/6$ for $k \in \{1, \dots, 6\}$.

$E[X] = \tfrac{1}{6}(1+2+3+4+5+6) = 3.5$.
$E[X^2] = \tfrac{1}{6}(1+4+9+16+25+36) = \tfrac{91}{6} \approx 15.167$.
Using $\text{Var}(X) = E[X^2] - (E[X])^2$:
$\text{Var}(X) = 15.167 - 12.25 = 2.917$, so $\sigma \approx 1.708$.

Roll the die 100 times: the sample mean lands near $3.5$, sample variance
near $2.92$. Larger sample → tighter to theoretical (law of large numbers).

---

## 3. Covariance (scalar)

For two RVs $X, Y$, the covariance measures co-movement:

$$
\text{Cov}(X, Y) = E\!\left[(X - E[X])(Y - E[Y])\right].
$$

- **Positive**: $X$ above mean ⇒ $Y$ tends above mean (move together).
- **Negative**: $X$ above mean ⇒ $Y$ tends below mean (move oppositely).
- **Zero**: knowing $X$ tells you nothing (linearly) about $Y$ —
  **uncorrelated**.

Note $\text{Cov}(X, X) = \text{Var}(X)$ and $\text{Cov}(X, Y) = \text{Cov}(Y, X)$.
**Correlation** $\rho_{XY} = \text{Cov}(X,Y)/(\sigma_X\sigma_Y) \in [-1, 1]$
is a unit-stripped version of covariance; we use covariance throughout.

### Worked example: two RVs

Joint distribution (each row prob $0.25$): $(X,Y) \in \{(1,2),(1,4),(3,2),(3,4)\}$.

$E[X] = 2$, $E[Y] = 3$, $E[XY] = 0.25(2+4+6+12) = 6$.
$\text{Cov}(X,Y) = E[XY] - E[X]E[Y] = 6 - 6 = 0$ — uncorrelated.

Now change outcome D to $(3, 6)$: $E[Y] = 3.5$, $E[XY] = 7.5$,
$\text{Cov}(X,Y) = 7.5 - 7 = 0.5$. Positive: bigger $X$ tends to bigger $Y$.

---

## 4. The univariate Gaussian (normal) distribution

The bell curve. $X \sim \mathcal{N}(\mu, \sigma^2)$ has PDF

$$
p(x) = \frac{1}{\sqrt{2\pi\sigma^2}} \exp\!\left(-\frac{(x - \mu)^2}{2\sigma^2}\right).
$$

Fully specified by mean $\mu$ and variance $\sigma^2$. Symmetric around
$\mu$, peaked at $\mu$ (height $1/\sqrt{2\pi\sigma^2}$), decaying
exponentially in $(x-\mu)^2$. The classic 68% / 95% / 99.7% rule covers
$\pm 1\sigma$, $\pm 2\sigma$, $\pm 3\sigma$.

**Why Gaussians?**

1. **Central limit theorem.** Sums of many small independent effects
   tend toward a Gaussian. Sensor noise (thermal + quantization +
   electromagnetic) is exactly this pattern.
2. **Closed under linear ops.** $X \sim \mathcal{N}(\mu, \sigma^2)$,
   $Y = aX + b \Rightarrow Y \sim \mathcal{N}(a\mu + b, a^2\sigma^2)$.
   We need the vector version (§8).
3. **Two-parameter sufficient.** Mean and variance fully describe the
   distribution — no higher moments needed. This is what makes a Kalman
   filter tractable: at every step we propagate $(\boldsymbol{\mu},
   \boldsymbol{\Sigma})$ instead of an arbitrary shape.

### Worked example

Accelerometer noise $\mathcal{N}(0, 0.06^2) \text{ m/s}^2$: 68% of readings
within $\pm 0.06$ of truth, 95% within $\pm 0.12$, a $0.20$ excursion is
$>3\sigma$ ($<0.3\%$) — suspect non-noise (bias, bump, fault).

---

## 5. Vectors of random variables

Stack $n$ RVs into a column $\mathbf{x} = (X_1, \dots, X_n)^T$. The
**mean vector** is the elementwise expectation:

$$
\boldsymbol{\mu} = E[\mathbf{x}] = (E[X_1], \dots, E[X_n])^T.
$$

Linearity extends: for constant $\mathbf{A}, \mathbf{b}$,
$E[\mathbf{A}\mathbf{x} + \mathbf{b}] = \mathbf{A}\boldsymbol{\mu} + \mathbf{b}$.

Example: $\mathbf{x} = (X_1, X_2, X_3)^T$ is the noise on three
accelerometer axes; each axis is zero-mean, so $\boldsymbol{\mu} = (0,0,0)^T$.

---

## 6. Covariance matrix

For random vector $\mathbf{x}$ with mean $\boldsymbol{\mu}$:

$$
\boldsymbol{\Sigma} = E\!\left[(\mathbf{x} - \boldsymbol{\mu})(\mathbf{x} - \boldsymbol{\mu})^T\right].
$$

The outer product makes $\boldsymbol{\Sigma}$ an $n \times n$ matrix with
$\Sigma_{ij} = \text{Cov}(X_i, X_j)$. Therefore:

- **Diagonal** = variances ($\Sigma_{ii} = \text{Var}(X_i)$).
- **Off-diagonal** = covariances.
- **Symmetric** ($\Sigma_{ij} = \Sigma_{ji}$).
- **Positive semi-definite** ($\mathbf{a}^T\boldsymbol{\Sigma}\mathbf{a} \ge 0$);
  positive-definite for non-degenerate distributions — connect to ch01 §9.

### Worked 2×2 example

Two IMU axes with variances $4, 9$ and covariance $2$:

$$
\boldsymbol{\Sigma} = \begin{pmatrix} 4 & 2 \\ 2 & 9 \end{pmatrix}.
$$

Symmetric: yes. Positive-definite (ch01 §9 leading-minor test):
$4 > 0$, $\det = 36 - 4 = 32 > 0$. Valid covariance.

---

## 7. The multivariate Gaussian

$\mathbf{x} \sim \mathcal{N}(\boldsymbol{\mu}, \boldsymbol{\Sigma})$ has PDF

$$
p(\mathbf{x}) = \frac{1}{\sqrt{(2\pi)^n |\boldsymbol{\Sigma}|}} \exp\!\left(-\tfrac{1}{2}(\mathbf{x} - \boldsymbol{\mu})^T \boldsymbol{\Sigma}^{-1} (\mathbf{x} - \boldsymbol{\mu})\right),
$$

with $n$ the dimension and $|\boldsymbol{\Sigma}|$ the **determinant**
(ch01 §6) — not absolute value, not its square root (common trap).
$\boldsymbol{\Sigma}^{-1}$ exists because $\boldsymbol{\Sigma}$ is
positive-definite (ch01 §5).

**Geometric intuition.** Level sets $p(\mathbf{x}) = \text{const}$ are
ellipsoids centered on $\boldsymbol{\mu}$. Principal axes are eigenvectors
of $\boldsymbol{\Sigma}$; axis lengths scale with $\sqrt{\text{eigenvalue}}$.
Diagonal $\boldsymbol{\Sigma}$ → axis-aligned; off-diagonals tilt the ellipsoid.

### Worked example

Using the $2 \times 2$ $\boldsymbol{\Sigma}$ from §6:
$|\boldsymbol{\Sigma}| = 32$;
$\boldsymbol{\Sigma}^{-1} = \tfrac{1}{32}\begin{pmatrix} 9 & -2 \\ -2 & 4 \end{pmatrix}$
(ch01 §5);
normalizer $= 1/\sqrt{(2\pi)^2 \cdot 32} = 1/(2\pi\sqrt{32}) \approx 0.0281$ —
the density at $\mathbf{x} = \boldsymbol{\mu}$.

---

## 8. Linear transformations of Gaussians

The property that powers the Kalman **predict** step.

If $\mathbf{x} \sim \mathcal{N}(\boldsymbol{\mu}, \boldsymbol{\Sigma})$
and $\mathbf{y} = \mathbf{A}\mathbf{x} + \mathbf{b}$ ($\mathbf{A}, \mathbf{b}$ constant),

$$
\boxed{\,\mathbf{y} \sim \mathcal{N}\!\big(\mathbf{A}\boldsymbol{\mu} + \mathbf{b},\; \mathbf{A}\boldsymbol{\Sigma}\mathbf{A}^T\big).\,}
$$

Mean transforms linearly; covariance transforms by the **sandwich**
$\mathbf{A}\boldsymbol{\Sigma}\mathbf{A}^T$. The result is still Gaussian.

**Why this matters.** Kalman predict models the state as
$\mathbf{x}_{k+1} = \mathbf{F}\mathbf{x}_k + \mathbf{w}_k$ with
$\mathbf{w}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{Q})$. Applying the
boxed result: $\boldsymbol{\mu}_{k+1} = \mathbf{F}\boldsymbol{\mu}_k$ and
$\boldsymbol{\Sigma}_{k+1} = \mathbf{F}\boldsymbol{\Sigma}_k\mathbf{F}^T + \mathbf{Q}$.
**That is the Kalman predict equation.** Chapter 04 derives it in this
form. The FSW nav design states it verbatim in
`docs/design/nav/algorithm.md` §3.2 step 8: "$P_{new} = F P_{old} F^T + Q$".

### Worked 2D example

$\mathbf{x} \sim \mathcal{N}(\mathbf{0}, \begin{pmatrix} 4 & 2 \\ 2 & 9 \end{pmatrix})$,
$\mathbf{A} = \begin{pmatrix} 1 & 0 \\ 1 & 1 \end{pmatrix}$ ($Y_1 = X_1$,
$Y_2 = X_1 + X_2$), $\mathbf{b} = \mathbf{0}$:

$$
\mathbf{A}\boldsymbol{\Sigma}\mathbf{A}^T = \begin{pmatrix} 1 & 0 \\ 1 & 1 \end{pmatrix}\begin{pmatrix} 4 & 2 \\ 2 & 9 \end{pmatrix}\begin{pmatrix} 1 & 1 \\ 0 & 1 \end{pmatrix} = \begin{pmatrix} 4 & 6 \\ 6 & 17 \end{pmatrix}.
$$

Sanity: $\text{Var}(Y_2) = \text{Var}(X_1) + \text{Var}(X_2) + 2\text{Cov}(X_1, X_2) = 4 + 9 + 4 = 17$. Matches.

---

## 9. Marginal and conditional Gaussians

Partition $\mathbf{x} = (\mathbf{x}_a; \mathbf{x}_b)$ with joint

$$
\mathcal{N}\!\Big(\begin{pmatrix} \boldsymbol{\mu}_a \\ \boldsymbol{\mu}_b \end{pmatrix},\;\begin{pmatrix} \boldsymbol{\Sigma}_{aa} & \boldsymbol{\Sigma}_{ab} \\ \boldsymbol{\Sigma}_{ba} & \boldsymbol{\Sigma}_{bb} \end{pmatrix}\Big), \qquad \boldsymbol{\Sigma}_{ba} = \boldsymbol{\Sigma}_{ab}^T.
$$

Stated without proof (derived in Bishop *PRML* §2.3.1–§2.3.2 and Brown &
Hwang [2] Ch. 4):

**Marginal** (just drop the unwanted block):
$\mathbf{x}_a \sim \mathcal{N}(\boldsymbol{\mu}_a, \boldsymbol{\Sigma}_{aa}).$

**Conditional** (given observed $\mathbf{x}_b$):

$$
\mathbf{x}_a \mid \mathbf{x}_b \sim \mathcal{N}\!\big(\,\boldsymbol{\mu}_a + \boldsymbol{\Sigma}_{ab}\boldsymbol{\Sigma}_{bb}^{-1}(\mathbf{x}_b - \boldsymbol{\mu}_b),\;\boldsymbol{\Sigma}_{aa} - \boldsymbol{\Sigma}_{ab}\boldsymbol{\Sigma}_{bb}^{-1}\boldsymbol{\Sigma}_{ba}\,\big).
$$

**This conditional formula IS the Kalman update equation.** Chapter 04
identifies $\mathbf{x}_a$ with the unobserved state, $\mathbf{x}_b$ with
the predicted measurement,
$\boldsymbol{\Sigma}_{ab}\boldsymbol{\Sigma}_{bb}^{-1}$ with the Kalman
gain $\mathbf{K}$, and the bracketed mean/covariance with the posterior.
Compare to the Joseph form in `docs/design/nav/algorithm.md` §4.1 step 5.

### Worked 2D verification

$\boldsymbol{\mu} = (0, 0)^T$,
$\boldsymbol{\Sigma} = \begin{pmatrix} 4 & 2 \\ 2 & 9 \end{pmatrix}$,
condition on $X_2 = 3$ ($\Sigma_{aa}=4$, $\Sigma_{ab}=\Sigma_{ba}=2$, $\Sigma_{bb}=9$):

- Conditional mean: $0 + 2 \cdot \tfrac{1}{9} \cdot 3 = \tfrac{2}{3} \approx 0.667$.
- Conditional variance: $4 - 2 \cdot \tfrac{1}{9} \cdot 2 = \tfrac{32}{9} \approx 3.556$.

So $X_1 \mid X_2 = 3 \sim \mathcal{N}(0.667,\,3.556)$.

**Cross-check via regression:** least-squares predictor
$\hat{X}_1 = \frac{\text{Cov}(X_1,X_2)}{\text{Var}(X_2)} X_2 = \tfrac{2}{9}\cdot 3 = \tfrac{2}{3}$;
residual variance $\text{Var}(X_1) - \frac{\text{Cov}^2}{\text{Var}(X_2)} = 4 - \tfrac{4}{9} = \tfrac{32}{9}$.
Both match — verified.

Conditioning moved the mean from $0$ to $0.667$ (positive covariance
pulls it up) and **shrunk** the variance from $4$ to $3.556$ (we know
more). That uncertainty reduction is exactly what a Kalman update does.

---

## 10. Bayes' rule

The general statement of "update beliefs given new data":

$$
p(\mathbf{x} \mid \mathbf{z}) = \frac{p(\mathbf{z} \mid \mathbf{x})\,p(\mathbf{x})}{p(\mathbf{z})} \;\propto\; p(\mathbf{z} \mid \mathbf{x})\,p(\mathbf{x}),
$$

read as **posterior $\propto$ likelihood $\times$ prior**. The denominator
$p(\mathbf{z})$ is the normalizer making the posterior integrate to 1.
**This is the Kalman update step in its most general form.** With
Gaussian prior and likelihood, the posterior is Gaussian.

### Worked 1D example: Gaussian prior + Gaussian measurement

Following Brown & Hwang [2] Ch. 4 (1D recursive Bayesian update):

- Prior: $X \sim \mathcal{N}(\mu_0, \sigma_0^2)$.
- Measurement: $Z = X + V$ with $V \sim \mathcal{N}(0, \sigma_z^2)$
  independent of $X$, so $Z \mid X = x \sim \mathcal{N}(x, \sigma_z^2)$.

Multiply the two Gaussians and complete the square (algebra in [2] Ch. 4):

$$
\boxed{\;\sigma_+^2 = \frac{1}{\;\tfrac{1}{\sigma_0^2} + \tfrac{1}{\sigma_z^2}\;}, \qquad \mu_+ = \sigma_+^2\!\left(\frac{\mu_0}{\sigma_0^2} + \frac{z}{\sigma_z^2}\right).\;}
$$

Two readings:

1. **Precisions add.** Precision $\tau = 1/\sigma^2$, so $\tau_+ = \tau_0 + \tau_z$.
2. **Mean is precision-weighted.** $\mu_+$ is a weighted average of $\mu_0$
   and $z$ with the more confident source pulling harder.

### Numerical example

Prior $\mu_0 = 100\,\text{m}$, $\sigma_0 = 10\,\text{m}$.
Measurement $z = 105\,\text{m}$, $\sigma_z = 5\,\text{m}$.

- Precisions: $\tau_0 = 0.01$, $\tau_z = 0.04$, sum $= 0.05$.
- $\sigma_+^2 = 1/0.05 = 20\,\text{m}^2$, $\sigma_+ \approx 4.47\,\text{m}$.
- $\mu_+ = 20\,(100/100 + 105/25) = 20 \cdot 5.2 = 104\,\text{m}$.

Posterior $\mathcal{N}(104, 20)$. Mean lies between $100$ and $105$,
closer to the measurement (smaller $\sigma$, larger precision); posterior
$\sigma$ is smaller than either input — fusion reduces uncertainty.

Recursive Bayesian update — fold in one measurement at a time, carry the
posterior as the next prior — **is** the 1D linear-Gaussian Kalman filter.
Chapter 04 generalizes to vectors by chaining §8 (predict) with §9 (update).

---

## FSW anchor

In the Juno FT1 EKF, the navigation state is a 16-vector
$\mathbf{x} = (\text{tPosLla}, \text{tVelNed}, \text{tAttQuat},
\text{tAccelBias}, \text{tGyroBias})$ — see
`docs/design/nav/algorithm.md` §3.1, lines 71–82. Its uncertainty is
encoded by the $16 \times 16$ symmetric positive-definite covariance
matrix $\mathbf{P}$ — exactly the matrix-valued $\boldsymbol{\Sigma}$ of
§6, just specialized to dimension 16. The IMU, baro, and GPS noise are
modeled as zero-mean Gaussian random vectors with caller-supplied
diagonal covariances `fImuAccelNoiseSigmaMps2[3]`,
`fImuGyroNoiseSigmaRps[3]`, `fBaroNoiseSigmaM`,
`fGpsHorizNoiseSigmaM`, `fGpsVertNoiseSigmaM`,
`fGpsVelNoiseSigmaMps` — see `docs/design/nav/algorithm.md` §5.1, lines
286–296. Every concept in this chapter — mean vector, covariance matrix,
linear transformation of a Gaussian, conditional Gaussian, Bayes' rule —
shows up directly in the FSW EKF code path. Chapters 03 onward will pull
these threads through.

---

## Recap

- RV → PDF/PMF; mean $E[X]$, variance $\text{Var}(X)$, std-dev $\sigma$.
- Covariance measures co-movement. Stack RVs into a vector with mean
  $\boldsymbol{\mu}$ and symmetric positive-definite covariance
  $\boldsymbol{\Sigma}$ (variances on diagonal, covariances off).
- Univariate Gaussian $\mathcal{N}(\mu, \sigma^2)$ — two-parameter,
  closed under linear ops, justified by CLT.
- Multivariate Gaussian $\mathcal{N}(\boldsymbol{\mu}, \boldsymbol{\Sigma})$
  has ellipsoidal level sets; normalizer uses $|\boldsymbol{\Sigma}|$.
- **Predict** ($\S 8$): $\mathbf{y} = \mathbf{A}\mathbf{x} + \mathbf{b}
  \Rightarrow \mathcal{N}(\mathbf{A}\boldsymbol{\mu} + \mathbf{b},\,\mathbf{A}\boldsymbol{\Sigma}\mathbf{A}^T)$.
- **Update** ($\S 9$): conditional-Gaussian formula = Kalman update.
- **Bayes** ($\S 10$): posterior $\propto$ likelihood $\times$ prior;
  Gaussian-Gaussian → precision-sum, precision-weighted mean.

Next: chapter 03 — state-space systems.

---

## Reference

- [2] Brown, R. G. and Hwang, P. Y. C. (2012). *Introduction to Random
  Signals and Applied Kalman Filtering* (4th ed.), Wiley — Ch. 4 covers
  the recursive 1D Bayesian update of §10 and the conditional-Gaussian
  result of §9. Bishop *PRML* §2.3.1 was used as a cross-check for §9.
