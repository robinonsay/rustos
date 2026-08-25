---
document_type: Tutorial Chapter — The Linear Kalman Filter, Derived
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent: docs/tutorials/nav_kalman/index.md
prerequisites: Chapters 01 (linear algebra), 02 (probability and Gaussians), 03 (state-space)
target_reader: One software engineer (Robin Onsay) — rusty on linear algebra and probability; no prior nav/controls background
---

# Chapter 04 — The Linear Kalman Filter, Derived

> **Where we are.** Chapter 03 introduced the linear-Gaussian state-space
> model. We now derive, from first principles, the algorithm that computes
> the optimal estimate of $\mathbf{x}_k$ given every measurement up to time
> $k$. The result is five equations — the Kalman filter — first published
> by R. E. Kalman in 1960 [3]. **This chapter is the mathematical core of
> the tutorial.** Read it slowly. Do the algebra by hand once. You will
> return to it many times when implementing FT1's EKF (chapter 06).

---

## 1. Setup — The Linear-Gaussian Model

Restating the model from chapter 03 §3, with no abbreviations:

**Process model** (how the state evolves from time $k-1$ to time $k$):

$$\mathbf{x}_{k} = \mathbf{F}\,\mathbf{x}_{k-1} + \mathbf{w}_{k}, \qquad \mathbf{w}_{k} \sim \mathcal{N}(\mathbf{0},\,\mathbf{Q}).$$

**Measurement model** (what we observe at time $k$):

$$\mathbf{z}_{k} = \mathbf{H}\,\mathbf{x}_{k} + \mathbf{v}_{k}, \qquad \mathbf{v}_{k} \sim \mathcal{N}(\mathbf{0},\,\mathbf{R}).$$

**Initial belief**:

$$\mathbf{x}_{0} \sim \mathcal{N}(\hat{\mathbf{x}}_{0\mid 0},\,\mathbf{P}_{0\mid 0}).$$

**Standing assumptions** (carried from chapter 03):

1. $\mathbf{F}$, $\mathbf{H}$, $\mathbf{Q}$, $\mathbf{R}$ are known. ($\mathbf{Q}$, $\mathbf{R}$ are positive semi-definite; $\mathbf{R}$ is positive definite so $\mathbf{R}^{-1}$ exists.)
2. $\mathbf{w}_k$ and $\mathbf{v}_k$ are zero-mean, Gaussian, mutually independent, and white (uncorrelated across time).
3. The initial state $\mathbf{x}_0$ is independent of all $\mathbf{w}_k$ and $\mathbf{v}_k$.

**The goal.** At each time $k$, given measurements $\mathbf{z}_1, \mathbf{z}_2, \ldots, \mathbf{z}_k$ (abbreviated $\mathbf{z}_{1:k}$), compute the posterior density:

$$p(\mathbf{x}_k \mid \mathbf{z}_{1:k}).$$

**Why we can hope to compute this in closed form.** Under the linear-Gaussian assumptions, this posterior is itself Gaussian for every $k$. The proof is by induction (we will give it constructively below). A Gaussian density is fully described by its mean and covariance — so "compute the posterior" reduces to "compute two finite-size objects: $\hat{\mathbf{x}}_{k\mid k}$ and $\mathbf{P}_{k\mid k}$." This is a small miracle of the linear-Gaussian world; chapter 06 will show how the EKF approximates the same path when $\mathbf{f}$ and $\mathbf{h}$ are nonlinear.

The mean is the *minimum-mean-squared-error* estimate of $\mathbf{x}_k$, and the covariance describes its uncertainty. **No richer summary of our knowledge is possible** — that's what "Gaussian-conjugate posterior" buys us.

---

## 2. The Recursion Structure — Predict and Update

We do **not** recompute the posterior from scratch each step. Instead we maintain a running summary $(\hat{\mathbf{x}}_{k-1\mid k-1},\,\mathbf{P}_{k-1\mid k-1})$ — the posterior at time $k-1$ — and at each new step we do two things:

**(a) Predict.** Push the previous posterior through the process model to obtain the **prior** at time $k$:

$$p(\mathbf{x}_k \mid \mathbf{z}_{1:k-1}) = \mathcal{N}(\hat{\mathbf{x}}_{k\mid k-1},\,\mathbf{P}_{k\mid k-1}).$$

This step uses no new measurement; it only ages the previous posterior by one tick.

**(b) Update.** Condition on the new measurement $\mathbf{z}_k$ to obtain the **posterior** at time $k$:

$$p(\mathbf{x}_k \mid \mathbf{z}_{1:k}) = \mathcal{N}(\hat{\mathbf{x}}_{k\mid k},\,\mathbf{P}_{k\mid k}).$$

The recursion is then:

```
  posterior(k-1) --[predict, uses F, Q]--> prior(k) --[update, uses z_k, H, R]--> posterior(k)
```

Two pictures will help. The predict step **inflates** the covariance ellipsoid (we lose information by waiting; process noise widens our uncertainty). The update step **contracts** the covariance ellipsoid along the directions $\mathbf{H}$ measures (we gain information from the measurement). The Kalman filter's elegance is that both operations are exact closed-form Gaussian updates.

The recursion is what makes Kalman filtering tractable in real time: at each step we touch only the current summary, never the entire history of measurements. This bounded per-step cost is exactly why FT1 can run the filter at 200 Hz on a Pico2 (chapter 11; `algorithm.md` §3.2 line 105).

---

## 3. Predict Step — Derivation

We have the posterior at $k-1$:

$$\mathbf{x}_{k-1} \mid \mathbf{z}_{1:k-1} \sim \mathcal{N}(\hat{\mathbf{x}}_{k-1\mid k-1},\,\mathbf{P}_{k-1\mid k-1}).$$

Apply the process equation:

$$\mathbf{x}_k = \mathbf{F}\,\mathbf{x}_{k-1} + \mathbf{w}_k.$$

We invoke chapter 02 §8 (linear transformation of a Gaussian): if $\mathbf{a} \sim \mathcal{N}(\boldsymbol{\mu}_a, \boldsymbol{\Sigma}_a)$ and $\mathbf{b} \sim \mathcal{N}(\boldsymbol{\mu}_b, \boldsymbol{\Sigma}_b)$ are independent, then $\mathbf{A}\mathbf{a} + \mathbf{b} \sim \mathcal{N}(\mathbf{A}\boldsymbol{\mu}_a + \boldsymbol{\mu}_b,\,\mathbf{A}\boldsymbol{\Sigma}_a\mathbf{A}^T + \boldsymbol{\Sigma}_b)$.

Identify $\mathbf{a} \leftarrow \mathbf{x}_{k-1}\mid\mathbf{z}_{1:k-1}$, $\mathbf{A} \leftarrow \mathbf{F}$, $\mathbf{b} \leftarrow \mathbf{w}_k$. Independence holds because $\mathbf{w}_k$ is independent of past measurements (it's the *future* noise from the perspective of $k-1$).

**Mean** (linearity of expectation):

$$\hat{\mathbf{x}}_{k\mid k-1} = \mathbb{E}[\mathbf{F}\mathbf{x}_{k-1} + \mathbf{w}_k \mid \mathbf{z}_{1:k-1}] = \mathbf{F}\,\mathbb{E}[\mathbf{x}_{k-1}\mid\mathbf{z}_{1:k-1}] + \mathbb{E}[\mathbf{w}_k] = \mathbf{F}\,\hat{\mathbf{x}}_{k-1\mid k-1} + \mathbf{0}.$$

So:

$$\boxed{\;\hat{\mathbf{x}}_{k\mid k-1} = \mathbf{F}\,\hat{\mathbf{x}}_{k-1\mid k-1}.\;}$$

**Covariance**. Let $\tilde{\mathbf{x}}_{k-1} = \mathbf{x}_{k-1} - \hat{\mathbf{x}}_{k-1\mid k-1}$ be the posterior estimation error at $k-1$, so $\mathbb{E}[\tilde{\mathbf{x}}_{k-1}\tilde{\mathbf{x}}_{k-1}^T \mid \mathbf{z}_{1:k-1}] = \mathbf{P}_{k-1\mid k-1}$. The prior error at $k$ is:

$$\tilde{\mathbf{x}}_{k\mid k-1} = \mathbf{x}_k - \hat{\mathbf{x}}_{k\mid k-1} = \mathbf{F}(\mathbf{x}_{k-1} - \hat{\mathbf{x}}_{k-1\mid k-1}) + \mathbf{w}_k = \mathbf{F}\,\tilde{\mathbf{x}}_{k-1} + \mathbf{w}_k.$$

Then:

$$\mathbf{P}_{k\mid k-1} = \mathbb{E}[\tilde{\mathbf{x}}_{k\mid k-1}\tilde{\mathbf{x}}_{k\mid k-1}^T] = \mathbb{E}[(\mathbf{F}\tilde{\mathbf{x}}_{k-1} + \mathbf{w}_k)(\mathbf{F}\tilde{\mathbf{x}}_{k-1} + \mathbf{w}_k)^T].$$

Expanding the outer product and using independence ($\mathbb{E}[\tilde{\mathbf{x}}_{k-1}\mathbf{w}_k^T] = \mathbf{0}$):

$$\mathbf{P}_{k\mid k-1} = \mathbf{F}\,\mathbb{E}[\tilde{\mathbf{x}}_{k-1}\tilde{\mathbf{x}}_{k-1}^T]\,\mathbf{F}^T + \mathbb{E}[\mathbf{w}_k\mathbf{w}_k^T].$$

So:

$$\boxed{\;\mathbf{P}_{k\mid k-1} = \mathbf{F}\,\mathbf{P}_{k-1\mid k-1}\,\mathbf{F}^T + \mathbf{Q}.\;}$$

These are the two **predict equations**. Read them aloud once: "the new mean is $\mathbf{F}$ times the old mean; the new covariance is $\mathbf{F}\mathbf{P}\mathbf{F}^T$ plus $\mathbf{Q}$." Brown & Hwang [2, §5] derive this identically.

---

## 4. Update Step — Two Derivation Paths

We now have the prior $\mathcal{N}(\hat{\mathbf{x}}_{k\mid k-1},\mathbf{P}_{k\mid k-1})$ and a fresh measurement $\mathbf{z}_k = \mathbf{H}\mathbf{x}_k + \mathbf{v}_k$ with $\mathbf{v}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{R})$, independent of $\mathbf{x}_k$. We want the posterior $p(\mathbf{x}_k \mid \mathbf{z}_{1:k})$.

We give **two** derivations. They produce identical equations. Read both — the reasoning differs, and you will appreciate the equations more after seeing them arrive from two independent directions. Drop subscripts $k$ for readability where unambiguous; "prior" means $\mathcal{N}(\hat{\mathbf{x}}^{-},\mathbf{P}^{-})$ and "posterior" means $\mathcal{N}(\hat{\mathbf{x}}^{+},\mathbf{P}^{+})$.

### 4.1 Path A — Bayes' Rule and Completing the Square

Bayes' rule applied to the prior and the measurement likelihood:

$$p(\mathbf{x} \mid \mathbf{z}) \propto p(\mathbf{z} \mid \mathbf{x})\,p(\mathbf{x}).$$

Both factors are Gaussian:

$$p(\mathbf{x}) \propto \exp\!\Big(-\tfrac{1}{2}(\mathbf{x} - \hat{\mathbf{x}}^{-})^T (\mathbf{P}^{-})^{-1}(\mathbf{x} - \hat{\mathbf{x}}^{-})\Big),$$
$$p(\mathbf{z} \mid \mathbf{x}) \propto \exp\!\Big(-\tfrac{1}{2}(\mathbf{z} - \mathbf{H}\mathbf{x})^T \mathbf{R}^{-1}(\mathbf{z} - \mathbf{H}\mathbf{x})\Big).$$

The product of two Gaussians (in the same variable $\mathbf{x}$) is itself a Gaussian (chapter 02 §8). Multiply, ignore the multiplicative constants (they only adjust normalization), and look at the exponent. Define the "negative log-density" $J(\mathbf{x})$ as twice the negative exponent:

$$J(\mathbf{x}) = (\mathbf{x} - \hat{\mathbf{x}}^{-})^T(\mathbf{P}^{-})^{-1}(\mathbf{x} - \hat{\mathbf{x}}^{-}) + (\mathbf{z} - \mathbf{H}\mathbf{x})^T \mathbf{R}^{-1}(\mathbf{z} - \mathbf{H}\mathbf{x}).$$

Expanding both quadratic forms:

$$J(\mathbf{x}) = \mathbf{x}^T(\mathbf{P}^{-})^{-1}\mathbf{x} - 2\,\mathbf{x}^T(\mathbf{P}^{-})^{-1}\hat{\mathbf{x}}^{-} + \mathbf{x}^T \mathbf{H}^T\mathbf{R}^{-1}\mathbf{H}\mathbf{x} - 2\,\mathbf{x}^T \mathbf{H}^T \mathbf{R}^{-1} \mathbf{z} + \text{const}.$$

Group quadratic and linear terms in $\mathbf{x}$:

$$J(\mathbf{x}) = \mathbf{x}^T\big[(\mathbf{P}^{-})^{-1} + \mathbf{H}^T\mathbf{R}^{-1}\mathbf{H}\big]\mathbf{x} - 2\,\mathbf{x}^T\big[(\mathbf{P}^{-})^{-1}\hat{\mathbf{x}}^{-} + \mathbf{H}^T \mathbf{R}^{-1}\mathbf{z}\big] + \text{const}.$$

This is a quadratic form in $\mathbf{x}$. Pattern-match against $\mathbf{x}^T \mathbf{A} \mathbf{x} - 2\,\mathbf{x}^T \mathbf{b}$, which we know completes to $(\mathbf{x} - \mathbf{A}^{-1}\mathbf{b})^T \mathbf{A} (\mathbf{x} - \mathbf{A}^{-1}\mathbf{b}) - \mathbf{b}^T\mathbf{A}^{-1}\mathbf{b}$. Read off:

$$(\mathbf{P}^{+})^{-1} = (\mathbf{P}^{-})^{-1} + \mathbf{H}^T\mathbf{R}^{-1}\mathbf{H},\qquad \hat{\mathbf{x}}^{+} = \mathbf{P}^{+}\big[(\mathbf{P}^{-})^{-1}\hat{\mathbf{x}}^{-} + \mathbf{H}^T \mathbf{R}^{-1} \mathbf{z}\big].$$

These are the **information-form** Kalman equations. They are correct, but FT1 wants the **covariance form** because we propagate $\mathbf{P}$ rather than its inverse. Convert using the **Woodbury matrix identity** (chapter 01 §6, also called the matrix-inversion lemma):

$$\big[(\mathbf{P}^{-})^{-1} + \mathbf{H}^T\mathbf{R}^{-1}\mathbf{H}\big]^{-1} = \mathbf{P}^{-} - \mathbf{P}^{-}\mathbf{H}^T(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R})^{-1}\mathbf{H}\mathbf{P}^{-}.$$

Define the **innovation covariance** and **Kalman gain**:

$$\mathbf{S} \;\triangleq\; \mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R},\qquad \mathbf{K} \;\triangleq\; \mathbf{P}^{-}\mathbf{H}^T \mathbf{S}^{-1}.$$

Then $\mathbf{P}^{+} = \mathbf{P}^{-} - \mathbf{K}\mathbf{H}\mathbf{P}^{-} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}$. Substituting into the mean expression and using the same Woodbury substitution (algebraic exercise in chapter 12; Brown & Hwang [2, §5] walk through it):

$$\hat{\mathbf{x}}^{+} = \hat{\mathbf{x}}^{-} + \mathbf{K}(\mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-}).$$

Defining the **innovation** $\mathbf{y} \triangleq \mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-}$, the **boxed update equations** are:

$$\boxed{\;\mathbf{y} = \mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-},\quad \mathbf{S} = \mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R},\quad \mathbf{K} = \mathbf{P}^{-}\mathbf{H}^T\mathbf{S}^{-1},\quad \hat{\mathbf{x}}^{+} = \hat{\mathbf{x}}^{-} + \mathbf{K}\mathbf{y},\quad \mathbf{P}^{+} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}.\;}$$

This is the standard **simple-form Kalman update**, derived by Bayes' rule and a complete-the-square argument [2, §5].

### 4.2 Path B — The Joint Gaussian and Conditioning

A different route reaches the same destination using the **conditional Gaussian formula** from chapter 02 §9.

Form the joint distribution of $(\mathbf{x}, \mathbf{z})$ under the prior. Both are Gaussian; their joint is Gaussian with mean and covariance:

$$\begin{bmatrix}\mathbf{x} \\ \mathbf{z}\end{bmatrix} \sim \mathcal{N}\!\left(\begin{bmatrix}\hat{\mathbf{x}}^{-} \\ \mathbf{H}\hat{\mathbf{x}}^{-}\end{bmatrix},\;\begin{bmatrix}\boldsymbol{\Sigma}_{xx} & \boldsymbol{\Sigma}_{xz} \\ \boldsymbol{\Sigma}_{zx} & \boldsymbol{\Sigma}_{zz}\end{bmatrix}\right),$$

where:

- $\boldsymbol{\Sigma}_{xx} = \mathbb{E}[(\mathbf{x} - \hat{\mathbf{x}}^{-})(\mathbf{x} - \hat{\mathbf{x}}^{-})^T] = \mathbf{P}^{-}$.
- $\boldsymbol{\Sigma}_{zz} = \mathbb{E}[(\mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-})(\mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-})^T] = \mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R}$ (same expansion as the predict-covariance derivation; cross-term in $\mathbf{v}$ vanishes by independence).
- $\boldsymbol{\Sigma}_{xz} = \mathbb{E}[(\mathbf{x} - \hat{\mathbf{x}}^{-})(\mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-})^T] = \mathbb{E}[(\mathbf{x} - \hat{\mathbf{x}}^{-})(\mathbf{H}(\mathbf{x} - \hat{\mathbf{x}}^{-}) + \mathbf{v})^T] = \mathbf{P}^{-}\mathbf{H}^T$. (Care: $\boldsymbol{\Sigma}_{zx} = \boldsymbol{\Sigma}_{xz}^T = \mathbf{H}\mathbf{P}^{-}$. The blocks are NOT symmetric individually; **only the joint covariance matrix is symmetric** — $\boldsymbol{\Sigma}_{xz}$ and $\boldsymbol{\Sigma}_{zx}$ are transposes of one another. Brown & Hwang [2, §5] discuss this trap.)

The conditional Gaussian formula (chapter 02 §9):

$$\mathbf{x} \mid \mathbf{z} \sim \mathcal{N}\Big(\boldsymbol{\mu}_x + \boldsymbol{\Sigma}_{xz}\boldsymbol{\Sigma}_{zz}^{-1}(\mathbf{z} - \boldsymbol{\mu}_z),\;\boldsymbol{\Sigma}_{xx} - \boldsymbol{\Sigma}_{xz}\boldsymbol{\Sigma}_{zz}^{-1}\boldsymbol{\Sigma}_{zx}\Big).$$

Substitute:

$$\hat{\mathbf{x}}^{+} = \hat{\mathbf{x}}^{-} + \mathbf{P}^{-}\mathbf{H}^T(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R})^{-1}(\mathbf{z} - \mathbf{H}\hat{\mathbf{x}}^{-}),$$

$$\mathbf{P}^{+} = \mathbf{P}^{-} - \mathbf{P}^{-}\mathbf{H}^T(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R})^{-1}\mathbf{H}\mathbf{P}^{-}.$$

Using the same definitions of $\mathbf{S}$ and $\mathbf{K}$:

$$\hat{\mathbf{x}}^{+} = \hat{\mathbf{x}}^{-} + \mathbf{K}\mathbf{y},\qquad \mathbf{P}^{+} = \mathbf{P}^{-} - \mathbf{K}\mathbf{H}\mathbf{P}^{-} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}.$$

Identical to Path A. Bar-Shalom et al. [7, §5] derive the Kalman update by exactly this route; it is the most direct path when the joint structure is already in front of you.

### 4.3 The Five Boxed Equations

Collecting both derivations, the **five Kalman equations** (per measurement) are:

$$\mathbf{y}_k = \mathbf{z}_k - \mathbf{H}\hat{\mathbf{x}}_{k\mid k-1},\qquad \mathbf{S}_k = \mathbf{H}\mathbf{P}_{k\mid k-1}\mathbf{H}^T + \mathbf{R},$$

$$\mathbf{K}_k = \mathbf{P}_{k\mid k-1}\mathbf{H}^T \mathbf{S}_k^{-1},\qquad \hat{\mathbf{x}}_{k\mid k} = \hat{\mathbf{x}}_{k\mid k-1} + \mathbf{K}_k\mathbf{y}_k,$$

$$\mathbf{P}_{k\mid k} = (\mathbf{I} - \mathbf{K}_k\mathbf{H})\mathbf{P}_{k\mid k-1}\quad\text{(simple form)}.$$

Together with the two predict equations from §3, these are the entire linear Kalman filter. Five lines.

---

## 5. The Kalman Gain — What It Means

The gain $\mathbf{K}$ controls how much weight we give the innovation when correcting the prior. Reading $\mathbf{K} = \mathbf{P}^{-}\mathbf{H}^T(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R})^{-1}$:

- **Small $\mathbf{R}$ (low measurement noise).** $\mathbf{K} \to \mathbf{P}^{-}\mathbf{H}^T(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T)^{-1}$. The filter trusts the measurement; the innovation contributes maximally to the state correction.
- **Large $\mathbf{R}$ (high measurement noise).** $\mathbf{R}^{-1}$ shrinks; $\mathbf{K} \to \mathbf{0}$. The filter trusts the prediction; the measurement is largely ignored.

**Scalar intuition.** Take a 1-D state, $\mathbf{H} = 1$, $\mathbf{P}^{-} = p$, $\mathbf{R} = r$. Then:

$$K = \frac{p}{p + r},\qquad p^{+} = (1 - K)\,p = \frac{p\,r}{p + r}.$$

If $r = 0$ (perfect measurement), $K = 1$ and the posterior is $\hat{x}^{+} = z$, $p^{+} = 0$ — we now know the state perfectly. If $r \to \infty$ (useless measurement), $K \to 0$ and the posterior matches the prior. For $r = p$, $K = 1/2$ and we average. **The gain is exactly the precision-weighted blending ratio** between prior and measurement, generalized to vector quantities. This is the deepest single sentence in the chapter — re-read it.

---

## 6. Joseph Form — Numerically Stable Covariance Update

The simple form $\mathbf{P}^{+} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}$ is correct in exact arithmetic but **numerically fragile** in floating point. It is not symmetric on its face: a tiny floating-point asymmetry in $\mathbf{P}^{-}$ propagates and grows. Worse, round-off can drive $\mathbf{P}^{+}$ to lose positive-definiteness, which corrupts the entire filter.

The **Joseph form** is an algebraically equivalent expression that is symmetric by construction:

$$\boxed{\;\mathbf{P}_{k\mid k} = (\mathbf{I} - \mathbf{K}_k\mathbf{H})\,\mathbf{P}_{k\mid k-1}\,(\mathbf{I} - \mathbf{K}_k\mathbf{H})^T + \mathbf{K}_k\,\mathbf{R}\,\mathbf{K}_k^T.\;}$$

**Algebraic equivalence to the simple form.** Expand the first term:

$$(\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}(\mathbf{I} - \mathbf{K}\mathbf{H})^T = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-} - (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T.$$

Adding $\mathbf{K}\mathbf{R}\mathbf{K}^T$:

$$\mathbf{P}^{+}_{\text{Joseph}} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-} - (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T + \mathbf{K}\mathbf{R}\mathbf{K}^T.$$

Examine the second and third terms together:

$$-(\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T + \mathbf{K}\mathbf{R}\mathbf{K}^T = -\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T + \mathbf{K}\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T + \mathbf{K}\mathbf{R}\mathbf{K}^T.$$

Group the last two: $\mathbf{K}(\mathbf{H}\mathbf{P}^{-}\mathbf{H}^T + \mathbf{R})\mathbf{K}^T = \mathbf{K}\mathbf{S}\mathbf{K}^T$. Substitute $\mathbf{K} = \mathbf{P}^{-}\mathbf{H}^T\mathbf{S}^{-1}$, so $\mathbf{K}\mathbf{S} = \mathbf{P}^{-}\mathbf{H}^T$, and therefore $\mathbf{K}\mathbf{S}\mathbf{K}^T = \mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T$. The cross-terms cancel:

$$\mathbf{P}^{+}_{\text{Joseph}} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-} - \mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T + \mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}^{-} = \mathbf{P}^{+}_{\text{simple}}.$$

The two forms agree exactly in real arithmetic. They diverge in floating-point arithmetic: Joseph form preserves symmetry by construction (it is a sum of two manifestly symmetric matrices, $\mathbf{A}\mathbf{P}\mathbf{A}^T$ and $\mathbf{K}\mathbf{R}\mathbf{K}^T$, both of the form "symmetric sandwich"), and is positive-definite-preserving under round-off provided $\mathbf{P}^{-}$ and $\mathbf{R}$ are. The simple form has no such guarantee. Brown & Hwang [2, §5.6] give a worked example where the simple form drifts non-positive-definite within a few hundred steps under realistic round-off; Joseph form does not.

**FT1 mandate.** `docs/design/nav/algorithm.md` §6 (lines 352–357) recommends Joseph form for the FSW EKF: "Joseph form (`P_new = (I - K H) P_old (I - K H)^T + K R K^T`) is recommended over the simple form (`P_new = (I - K H) P_old`) — it is symmetric by construction and roughly 2× the cost. The IMPL chooses; the choice must be documented in the IMPL TU and tested against the ground-truth reference in SW-TC-NAV-021." Bar-Shalom et al. [7, §5.3] gives the canonical exposition of the numerical-stability argument.

---

## 7. Symmetry Enforcement — `P ← ½(P + Pᵀ)`

Even with Joseph form, **round-off accumulates**. After many sequences of `MatMul`, `Transpose`, and `Add` in the covariance update, the off-diagonal entries of $\mathbf{P}$ drift by tens of ULPs from their symmetric counterparts. Eventually a subsequent matrix inverse (in $\mathbf{S}^{-1}$) amplifies the asymmetry into an ill-conditioning problem and the filter diverges. The cheapest and most robust insurance is to **explicitly symmetrize** after each update step:

$$\boxed{\;\mathbf{P} \;\leftarrow\; \tfrac{1}{2}\bigl(\mathbf{P} + \mathbf{P}^T\bigr)\;}$$

Cost: one `Transpose`, one `Add`, one `Scale` — all $O(n^2)$, dwarfed by the $O(n^3)$ matrix multiplies elsewhere in the update. Effect: the result is **bit-exactly symmetric** because $\mathbf{P}^T_{ij} = \mathbf{P}_{ji}$ and the average of those two equal-magnitude entries is identical to its own transpose. This breaks the asymmetry-amplification feedback loop and is essentially free.

Note that symmetry enforcement is **complementary to** Joseph form, not a substitute. Joseph form keeps `P` symmetric and positive-definite **at the algebra level**; symmetry enforcement cleans up **floating-point drift** on top of that. Both are recommended for production filters; FT1 mandates both.

**FT1 mandate.** `docs/design/nav/algorithm.md` §6 (lines 359–363) requires symmetry enforcement after every update: "Symmetry enforcement. After each update, enforce `P_new = 0.5 * (P_new + Transpose(P_new))` to suppress drift away from symmetry caused by floating-point round-off. This is cheap (one `Transpose`, one `Add`, one `Scale` from kmat §4.2) and is recommended regardless of whether Joseph form is used." Brown & Hwang [2, §5.6] discusses the same drift mode in their numerical-stability section.

---

## 8. Properties of the Kalman Filter

**Optimality.** Among all *linear* estimators, the Kalman filter minimizes the mean squared error $\mathbb{E}[\lVert \mathbf{x}_k - \hat{\mathbf{x}}_{k\mid k}\rVert^2]$ on every step. When the noise is genuinely Gaussian, the Kalman filter is also the *optimal nonlinear* estimator — no nonlinear estimator can do better than the conditional mean, and under linear-Gaussian dynamics the conditional mean is exactly what the Kalman filter computes [2, §5; 7, §5].

**Covariance is data-independent.** Inspect the equations for $\mathbf{P}_{k\mid k-1}$ and $\mathbf{P}_{k\mid k}$: they reference $\mathbf{F}$, $\mathbf{H}$, $\mathbf{Q}$, $\mathbf{R}$, and prior covariances — but never the actual measurements $\mathbf{z}_k$. So the entire covariance trajectory $\{\mathbf{P}_{k\mid k}\}$ can be precomputed offline if the model is time-invariant. This property does **not** carry over to the EKF (chapter 06): in the EKF, $\mathbf{F}_k$ and $\mathbf{H}_k$ are evaluated at the current estimate, which depends on past measurements — so EKF covariance is data-dependent.

**Whiteness of the innovation.** When the filter is well-tuned (the process model and noise covariances match reality), the innovation sequence $\{\mathbf{y}_k\}$ is white — $\mathbb{E}[\mathbf{y}_i\mathbf{y}_j^T] = \mathbf{0}$ for $i \neq j$ [7, §5]. Time-correlation in the innovation is a strong signal that the filter is mis-tuned (often $\mathbf{Q}$ too small, causing the filter to over-trust its own predictions). FSW health monitors exploit this: a normalized innovation squared (NIS) statistic exceeding the chi-squared threshold flags a tuning or sensor problem. FT1's `nav_app` could (post-CDR) add this; for FT1 we rely on the divergence-bound check in `algorithm.md` §4.3.

---

## 9. Pseudocode

The full predict + update loop, one iteration per time step:

```
# Inputs to filter:
#   F, Q          : process model and process-noise covariance
#   H, R          : measurement model and measurement-noise covariance
#   x_post, P_post: posterior at time k-1 (initialized at k=0 from x0, P0)

For each time step k:

    # --- Predict step (always run) --------------------------------
    x_pred = F * x_post                                      # mean propagation
    P_pred = F * P_post * F.T + Q                            # covariance propagation

    # --- Update step (run only when measurement z_k is available) -
    if measurement_available(k):
        y = z - H * x_pred                                   # innovation
        S = H * P_pred * H.T + R                             # innovation covariance
        K = P_pred * H.T * inv(S)                            # Kalman gain
        x_post = x_pred + K * y                              # state update
        P_post = (I - K*H) * P_pred * (I - K*H).T + K*R*K.T  # Joseph form
        P_post = 0.5 * (P_post + P_post.T)                   # symmetry enforcement
    else:
        x_post = x_pred                                      # no measurement → predict-only
        P_post = P_pred

    emit(x_post, P_post)                                     # publish to consumers
```

**Cross-checks against the boxed equations**: line "innovation" matches §4.3 box, line "innovation covariance" matches §4.3 $\mathbf{S}_k$, line "Kalman gain" matches §4.3 $\mathbf{K}_k$, line "state update" matches §4.3 $\hat{\mathbf{x}}_{k\mid k}$, line "Joseph form" matches §6 box (cited from `algorithm.md` §6 lines 352–357), and the symmetry-enforcement line matches §7 box (cited from `algorithm.md` §6 lines 359–363).

**Predict-only branch.** When no measurement is available (e.g., during BOOST in FT1, when GPS and baro are gated by `nav_app`; `algorithm.md` §4.4 lines 257–264), we simply skip the update and publish the prior. This is exactly the dead-reckon path called out in `algorithm.md` §3.2 lines 181–187 — same code path, just no update. **The predict step always runs.**

**Inputs and dimensioning.** For a state of size $n$ and a measurement of size $m$: $\mathbf{F}$ and $\mathbf{P}$ are $n\times n$; $\mathbf{H}$ is $m\times n$; $\mathbf{R}$ and $\mathbf{S}$ are $m\times m$; $\mathbf{K}$ is $n\times m$. Get this dimensioning right before you write any code — it is the single most common bug source. Chapter 11 lists the FT1 dimensions: $n = 16$ (or $15$ in error-state form), $m = 1$ for baro, $m = 6$ for GPS.

---

## 10. FSW Anchor

In FT1's `nav_lib`, this same algorithm runs at the **IMU sample rate of 200 Hz** (`docs/design/nav/algorithm.md` §3.2 line 105) — but with **nonlinear $\mathbf{f}$** (strapdown integration in NED with quaternion attitude propagation; `algorithm.md` §3.2 steps 1–7) and **nonlinear $\mathbf{h}$** (geodetic-frame measurement model; `algorithm.md` §4). Chapter 06 will derive the Extended Kalman Filter, which linearizes $\mathbf{f}$ and $\mathbf{h}$ about the current estimate using analytic Jacobians per Groves [1, §14.2].

The numerical-stability concerns of `algorithm.md` §6 — Joseph form for the covariance update (lines 352–357), post-update symmetry enforcement (lines 359–363), pivot guarding in the matrix inverse — are direct consequences of what you have just learned. The simple-form covariance update is forbidden in flight code on numerical grounds (`algorithm.md` §6 recommendation); it appears in this chapter only because the derivation is more transparent. When you write the FT1 IMPL, use the boxed Joseph form from §6 followed by the boxed symmetry enforcement from §7 — both are mandatory.

The five Kalman equations of §4.3 are not optional in the FSW. Get them wrong by a sign and the filter diverges within seconds.

---

## 11. Key Results

- **The linear-Gaussian posterior is Gaussian** at every time step. The Kalman filter recursively maintains its mean and covariance.
- **Predict** propagates the posterior through $\mathbf{F}$ and adds $\mathbf{Q}$: $\hat{\mathbf{x}}_{k\mid k-1} = \mathbf{F}\hat{\mathbf{x}}_{k-1\mid k-1}$, $\mathbf{P}_{k\mid k-1} = \mathbf{F}\mathbf{P}_{k-1\mid k-1}\mathbf{F}^T + \mathbf{Q}$.
- **Update** applies the measurement: $\mathbf{y}_k = \mathbf{z}_k - \mathbf{H}\hat{\mathbf{x}}_{k\mid k-1}$, $\mathbf{S}_k = \mathbf{H}\mathbf{P}_{k\mid k-1}\mathbf{H}^T + \mathbf{R}$, $\mathbf{K}_k = \mathbf{P}_{k\mid k-1}\mathbf{H}^T\mathbf{S}_k^{-1}$, $\hat{\mathbf{x}}_{k\mid k} = \hat{\mathbf{x}}_{k\mid k-1} + \mathbf{K}_k\mathbf{y}_k$, $\mathbf{P}_{k\mid k} = (\mathbf{I} - \mathbf{K}_k\mathbf{H})\mathbf{P}_{k\mid k-1}$ (or Joseph form).
- **The Kalman gain is the precision-weighted blend** between prior and measurement.
- **Joseph form** is algebraically equivalent to the simple form and numerically stable; FT1 mandates it (`algorithm.md` §6 lines 352–357).
- **Symmetry enforcement** $\mathbf{P} \leftarrow \tfrac{1}{2}(\mathbf{P}+\mathbf{P}^T)$ after every update is cheap insurance against round-off drift; FT1 mandates it (`algorithm.md` §6 lines 359–363).
- **The covariance trajectory is data-independent** for the linear KF; this property fails for the EKF.
- **The innovation is white** when the filter is well-tuned — useful for filter-health monitoring.

---

## 12. Exercises (worked solutions in chapter 12)

1. Verify the predict-step covariance derivation by writing $\tilde{\mathbf{x}}_{k\mid k-1} = \mathbf{F}\tilde{\mathbf{x}}_{k-1} + \mathbf{w}_k$ and expanding $\mathbb{E}[\tilde{\mathbf{x}}_{k\mid k-1}\tilde{\mathbf{x}}_{k\mid k-1}^T]$ term by term.
2. Carry out the Woodbury identity step in §4.1 explicitly to convert the information form to the covariance form. (Brown & Hwang [2, §5] is a hint.)
3. Derive the joint covariance $\boldsymbol{\Sigma}_{xz} = \mathbf{P}^{-}\mathbf{H}^T$ by expanding the outer product, taking care to distinguish $\boldsymbol{\Sigma}_{xz}$ from $\boldsymbol{\Sigma}_{zx}$.
4. Verify the Joseph-form algebraic equivalence by repeating §6's expansion on paper. Find the step at which $\mathbf{K}\mathbf{S}\mathbf{K}^T$ becomes $\mathbf{P}^{-}\mathbf{H}^T\mathbf{K}^T$.
5. For the scalar problem $H = 1$, $P^{-} = p$, $R = r$, plot $K$ as a function of $r/p$ from $0.01$ to $100$. Identify where $K = 0.5$, where $K \to 1$, where $K \to 0$.
6. Implement the §9 pseudocode in Python+NumPy for a 2-state position-velocity system. Run for 100 steps with synthetic measurements; compare simple-form and Joseph-form covariance traces under tight $\mathbf{R}$ (chapter 05 walks this through).
7. Show that the innovation $\mathbf{y}_k$ has covariance $\mathbf{S}_k$ when the filter is consistent. Use the prior-error and measurement-noise definitions.

---

## 13. Citations

- **[2, §5; §5.6]** — Brown & Hwang, *Introduction to Random Signals and Applied Kalman Filtering*. Bayes-rule + complete-the-square derivation (§5); Joseph form and numerical stability (§5.6). Used in §4.1, §6.
- **[3]** — R. E. Kalman, "A new approach to linear filtering and prediction problems," 1960. Original publication; cited in §1 and §4.3 for historical primacy.
- **[7, §5]** — Bar-Shalom, Li, Kirubarajan, *Estimation with Applications to Tracking and Navigation*. Joint-Gaussian + conditional-Gaussian derivation; whiteness of innovation. Used in §4.2, §7.
- **`docs/design/nav/algorithm.md` §3.2 line 105** — IMU sample rate (200 Hz), cited in §9.
- **`docs/design/nav/algorithm.md` §6 lines 352–357** — Joseph-form recommendation, cited in §6 and §9 and §10.
- **`docs/design/nav/algorithm.md` §6 lines 359–363** — Symmetry-enforcement requirement, cited in §7 and §9 and §10.

<!-- @{"design": ["SW-REQ-NAV-018"]} -->
