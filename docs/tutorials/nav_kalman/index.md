---
document_type: Tutorial Index — Kalman Filter and Navigation for FT1 nav_lib / nav_app
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_sdp: docs/sdp/index.md (Revision C, §5 master sprint table)
target_reader: One software engineer (Robin Onsay) — rusty on linear algebra and probability; no prior nav/controls background
agent_status: Index file (Chapter 00 of 12); chapters 01–12 authored by sibling workers in parallel
---

# Kalman Filter and Navigation — A Tutorial for FT1's `nav_lib` / `nav_app`

> **Index, reader's guide, canonical notation, and bibliography** for the
> 12-chapter tutorial under `docs/tutorials/nav_kalman/`. This file is
> authored by the agent system as the entry point for the entire tutorial
> per the SDP Revision C amendment (2026-05-08). The chapters themselves
> are written in parallel by sibling workers; this index is the contract
> they all conform to.

---

## 1. Audience and Goals

**Who this tutorial is for.** This tutorial is written for **one specific
reader: a software engineer who is rusty on linear algebra and probability,
and has no prior background in navigation or controls.** Every chapter
assumes that reader and only that reader. We do not assume you remember the
mechanics of matrix multiplication; we do not assume you remember the
definition of variance; we do not assume you have ever seen a rotation
matrix, a quaternion, or a coordinate frame transform. We assume only that
you are a competent C++ engineer comfortable with floating-point arithmetic,
templates, and the FT1 codebase conventions.

**What you will be able to do at the end.** Implement `libs/nav_lib/`
(the EKF compute library specified in
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md)) and
`apps/nav_app/` (the schedule-driven application that drives the filter),
satisfying the `SW-REQ-NAV-001` through `SW-REQ-NAV-019` requirement
baseline at `docs/requirements/nav/requirements.json` and the test cases at
`docs/test_cases/nav/test_cases.json` (`SW-TC-NAV-001` through
`SW-TC-NAV-023`). The acceptance bar is the project's three integration
gates **G1** (POSIX build + ctest), **G2** (`tools/traceability.py` exit 0),
and **G3** (Pico2 cross-compile clean) per `docs/sdp/index.md` §9. This
tutorial is the upstream educational artifact that gates the PM-owned
`USER-NAV-LIB` and `USER-NAV-APP` work per `docs/sdp/index.md` §5 and SDP
risk register entry **SDP-R-08**.

**What this tutorial is *not*.** It is not a textbook. It is not a survey
of estimation theory. It is not a research paper. It is not a sales pitch
for one filter formulation over another. It is the minimum coherent
math-to-code path that lets you implement the FT1 EKF and understand every
line of it. Where a topic exists in the literature but is not needed for
FT1, it is dropped. Where the FT1 specification pins a particular choice
(e.g., Hamilton-convention quaternion, NED tangent frame, analytic Jacobian
per Groves §14.2), the tutorial follows that choice — competing
conventions are mentioned only as orientation for readers who later
encounter them in textbooks.

---

## 2. How to Use This Tutorial

### 2.1 Recommended reading order

Read the chapters **in numerical order** (01 → 12). The order is chosen so
that every chapter's prerequisites are covered by the chapter immediately
before it. Skipping ahead is supported (see §2.3) but the default path is
strictly sequential.

### 2.2 Time investment

Estimates assume you read actively (work the examples, redo the algebra by
hand once, and don't just skim). They are deliberately generous — the goal
is to learn, not to race.

| Chapter | Pages-equivalent | Active reading time | Cumulative |
|---------|------------------|---------------------|-----------|
| 01 Linear Algebra Primer | ~12 | 2.0 h | 2.0 h |
| 02 Probability and Gaussians Primer | ~10 | 1.5 h | 3.5 h |
| 03 State-Space Models | ~8 | 1.0 h | 4.5 h |
| 04 The Linear Kalman Filter, Derived | ~14 | 3.0 h | 7.5 h |
| 05 Kalman Filter — A Worked Numerical Example | ~10 | 2.0 h | 9.5 h |
| 06 The Extended Kalman Filter | ~12 | 2.5 h | 12.0 h |
| 07 Coordinate Frames and Transformations | ~10 | 1.5 h | 13.5 h |
| 08 Attitude Representations: Euler Angles and Quaternions | ~14 | 3.0 h | 16.5 h |
| 09 Inertial Navigation and Strapdown Integration | ~12 | 2.5 h | 19.0 h |
| 10 Sensor Fusion: Why and How | ~8 | 1.5 h | 20.5 h |
| 11 Mapping the Math to FT1's `nav_lib` | ~10 | 2.0 h | 22.5 h |
| 12 Exercises and Worked Solutions | ~16 | 3.5 h | 26.0 h |

Plan on **roughly 25–30 hours of focused study** to complete the full
tutorial with the exercises. A reasonable cadence is 60–90 minutes per
session; do not try to absorb chapter 04 (the Kalman derivation) or
chapter 08 (quaternions) in a single sitting.

### 2.3 Pace recommendations and skip rules

The chapters are stratified so a reader who is comfortable with portions of
the prerequisite material can skip selectively:

- **If you remember matrix multiplication, transpose, inverse, eigenvectors,
  and positive-definite matrices solidly** — skim chapter 01, but read its
  notation section and the "matrix calculus" subsection (used heavily in
  chapter 04).
- **If you remember mean, variance, covariance, the multivariate Gaussian
  density, conditioning, and the law of total probability** — skim chapter
  02, but read the "Gaussian conjugacy under linear measurement" subsection
  (the heart of the Kalman update).
- **If you have implemented a Kalman filter before** — read chapter 04 in
  full anyway (the FT1 derivation pins specific notation and the Joseph
  form), then read chapter 06 (EKF) carefully — the Jacobian conventions
  matter.
- **If you have a controls background but no nav background** — chapters
  01–06 will be review; chapters 07–11 are where the FT1-specific content
  starts.
- **Do not skip chapter 11.** It is the bridge between the math and the
  code, and it is where the reading hours pay off. Even a reader who skips
  every other chapter must read 11 to implement `nav_lib`.

### 2.4 What to do alongside reading

- Keep `docs/design/nav/algorithm.md` open in a second window. Every
  formula in the tutorial corresponds to a line in that file.
- Keep `libjuno/include/juno/math/juno_math.hpp` and
  `libs/kmat_lib/include/kmat/kmat_api.hpp` open as a third window — those
  are the C++ types you will instantiate. Chapter 11 cites specific symbols
  from both.
- Work the chapter 12 exercises with pen, paper, and a Python+NumPy REPL.
  The exercises are specifically chosen to expose the bugs that bite
  first-time EKF implementers (sign errors in the Jacobian, body-vs-NED
  frame confusion, quaternion non-normalization).

### 2.5 What to do if you get stuck

Re-read the previous chapter's summary, then the current chapter's
introduction. If a formula is unclear, sanity-check it on a 2D toy problem
(a 2×2 matrix or a 2-state filter) — this is what chapter 05 is for. If a
reference is opaque, consult the corresponding entry in §6 below; Groves
(ref. 1) and Brown & Hwang (ref. 2) have multiple alternative derivations
of every key result.

---

## 3. Chapter Index

The chapter list below is **canonical**: file names, titles, numbering, and
the "covers" anchor reqs are pinned by the brief and shared verbatim by
every sibling worker authoring chapters 01–12.

| #  | File                                          | Title                                                                  | One-sentence description                                                                                                                                                | Prerequisites                                                          |
|----|-----------------------------------------------|------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| 01 | [`01_linear_algebra.md`](01_linear_algebra.md) | Linear Algebra Primer                                                  | Vectors, matrices, transpose, inverse, eigen-decomposition, positive-definite matrices, and the matrix-calculus identities used in the Kalman derivation.                 | Comfort with calculus; no linear-algebra refresher needed.             |
| 02 | [`02_probability_and_gaussians.md`](02_probability_and_gaussians.md) | Probability and Gaussians Primer                                       | Random variables, expectation, variance, covariance, the multivariate Gaussian, conditioning, and Gaussian-linear conjugacy.                                              | Chapter 01 (notation; outer product; quadratic forms).                  |
| 03 | [`03_state_space.md`](03_state_space.md)       | State-Space Models                                                     | Discrete-time dynamical systems written as $\mathbf{x}_{k+1} = \mathbf{F}\mathbf{x}_k + \mathbf{w}_k$ with measurements $\mathbf{z}_k = \mathbf{H}\mathbf{x}_k + \mathbf{v}_k$. | Chapter 01.                                                             |
| 04 | [`04_kalman_filter_derivation.md`](04_kalman_filter_derivation.md) | The Linear Kalman Filter, Derived                                       | The five Kalman equations derived from "minimum-mean-squared-error linear estimator" first principles; predict + update; Joseph form.                                     | Chapters 01–03.                                                         |
| 05 | [`05_kalman_filter_worked_example.md`](05_kalman_filter_worked_example.md) | Kalman Filter — A Worked Numerical Example                              | A 2-state position-velocity filter run by hand and in NumPy across ten cycles, including divergence under bad tuning.                                                     | Chapter 04.                                                             |
| 06 | [`06_extended_kalman_filter.md`](06_extended_kalman_filter.md) | The Extended Kalman Filter                                              | Linearization of nonlinear $\mathbf{f}, \mathbf{h}$ at the current estimate; analytic vs numerical Jacobians; why the FT1 spec mandates analytic.                          | Chapter 04; multivariable calculus refresher in chapter 01.             |
| 07 | [`07_frames_and_transformations.md`](07_frames_and_transformations.md) | Coordinate Frames and Transformations                                   | ECEF, NED tangent frame, body frame; rotation matrices; chains of transforms; the FT1 frame conventions per `docs/design/conventions.md` §4.                              | Chapter 01 (rotation as orthogonal matrix).                             |
| 08 | [`08_attitude_representations.md`](08_attitude_representations.md) | Attitude Representations: Euler Angles and Quaternions                  | Why Euler angles fail near gimbal lock; Hamilton-convention quaternions; quaternion product, conjugate, rotation, normalization; tangent-space derivatives.               | Chapter 07.                                                             |
| 09 | [`09_inertial_navigation.md`](09_inertial_navigation.md) | Inertial Navigation and Strapdown Integration                           | Strapdown mechanization of accel + gyro into pos/vel/att in NED; gravity subtraction; bias states; integration-error growth.                                              | Chapters 07, 08.                                                        |
| 10 | [`10_sensor_fusion_intuition.md`](10_sensor_fusion_intuition.md) | Sensor Fusion: Why and How                                              | Why combine IMU + baro + GPS; complementary error spectra; the EKF as the optimal weighted blender; intuitive arguments for $\mathbf{Q}$ and $\mathbf{R}$ tuning.          | Chapters 06, 09.                                                        |
| 11 | [`11_fsw_nav_mapping.md`](11_fsw_nav_mapping.md) | Mapping the Math to FT1's `nav_lib`                                     | Walk through `docs/design/nav/algorithm.md` §3.2 and §4 line by line, mapping each math symbol to the C++ symbol the IMPL must instantiate.                                | All prior chapters.                                                     |
| 12 | [`12_exercises_and_solutions.md`](12_exercises_and_solutions.md) | Exercises and Worked Solutions                                          | Exercises spanning the chapters above, with full worked solutions; emphasis on the bugs that bite first-time EKF implementers.                                            | All prior chapters.                                                     |

Each chapter ends with: a "Key Results" boxed summary, an "Exercises" set
referenced in chapter 12, and a "Citations" subsection using the bracketed
form `[N, §X]` against the bibliography in §6.

---

## 4. Canonical Notation

Every chapter uses the notation below verbatim. Sibling workers have been
instructed to introduce no alternative symbols. If you encounter notation
in a chapter that conflicts with the table below, treat it as an authoring
defect and report it to the Software Lead.

| Symbol | Meaning |
|--------|---------|
| **Scalars** | Italic lowercase Latin or Greek, e.g. $x$, $\sigma$, $k$. |
| **Vectors** | Bold lowercase, e.g. $\mathbf{x}$, $\mathbf{z}$. All vectors are column vectors unless explicitly noted. |
| **Matrices** | Bold uppercase, e.g. $\mathbf{P}$, $\mathbf{F}$, $\mathbf{H}$, $\mathbf{Q}$, $\mathbf{R}$, $\mathbf{K}$, $\mathbf{I}$. |
| **Time index** | $k$ (discrete, used throughout); $t$ (continuous, used sparingly). |
| **Estimate hat** | $\hat{\mathbf{x}}$ denotes "estimate of $\mathbf{x}$." |
| **A priori (prediction) state** | $\hat{\mathbf{x}}_{k\mid k-1}$: estimate at time $k$ given measurements up to and including $k-1$. |
| **A posteriori (posterior) state** | $\hat{\mathbf{x}}_{k\mid k}$: estimate at time $k$ after fusing the measurement at $k$. |
| **Process model (general)** | $\mathbf{x}_{k+1} = \mathbf{f}(\mathbf{x}_k, \mathbf{u}_k) + \mathbf{w}_k$, with $\mathbf{w}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{Q})$. |
| **Measurement model (general)** | $\mathbf{z}_k = \mathbf{h}(\mathbf{x}_k) + \mathbf{v}_k$, with $\mathbf{v}_k \sim \mathcal{N}(\mathbf{0}, \mathbf{R})$. |
| **Linear case** | $\mathbf{f}(\mathbf{x}_k) = \mathbf{F}\mathbf{x}_k$, $\mathbf{h}(\mathbf{x}_k) = \mathbf{H}\mathbf{x}_k$. The FT1 nav system has **no input vector $\mathbf{u}$** — IMU measurements are treated as observations, not control inputs. |
| **Innovation** | $\mathbf{y}_k = \mathbf{z}_k - \mathbf{h}(\hat{\mathbf{x}}_{k\mid k-1})$. |
| **Innovation covariance** | $\mathbf{S}_k = \mathbf{H}_k \mathbf{P}_{k\mid k-1} \mathbf{H}_k^T + \mathbf{R}_k$. |
| **Kalman gain** | $\mathbf{K}_k = \mathbf{P}_{k\mid k-1} \mathbf{H}_k^T \mathbf{S}_k^{-1}$. |
| **Posterior covariance (Joseph form)** | $\mathbf{P}_{k\mid k} = (\mathbf{I} - \mathbf{K}_k \mathbf{H}_k)\,\mathbf{P}_{k\mid k-1}\,(\mathbf{I} - \mathbf{K}_k \mathbf{H}_k)^T + \mathbf{K}_k \mathbf{R}_k \mathbf{K}_k^T$. |
| **Frames** (per `docs/design/conventions.md` §4) | **ECEF** (Earth-Centered Earth-Fixed, world frame); **NED** (North-East-Down, local-tangent frame at the launch point); **Body** (vehicle body frame; x forward, y right, z down). |
| **Quaternion (Hamilton convention)** | $\mathbf{q} = (q_w,\,q_x,\,q_y,\,q_z)$ with $q_w$ the scalar component. Stores a body→NED rotation. Unit norm $\lvert\mathbf{q}\rvert = 1$ is enforced after every prediction step (see chapter 09 and `docs/design/nav/algorithm.md` §6). |
| **Gravity** | $\mathbf{g}^{NED} = (0,\,0,\,+9.80665)\,\mathrm{m/s^2}$. Down is positive in NED, so the gravity vector has a positive $z$ component. |
| **Multivariate Gaussian** | $\mathcal{N}(\boldsymbol{\mu},\,\boldsymbol{\Sigma})$ with mean $\boldsymbol{\mu}$ and covariance $\boldsymbol{\Sigma}$. |
| **Math format** | LaTeX-style inline `$...$` and display `$$...$$` math in markdown; verify in a standard markdown viewer (the FT1 docs renderer supports KaTeX). |

This notation is consistent with `docs/design/nav/algorithm.md` §3.2
(state vector composition; gravity vector; quaternion convention) and §4
(measurement-model symbols `H`, `R_baro`, `R_gps`, `S`, `K`, Joseph form).
Where the algorithm.md uses ASCII fall-backs (e.g., `H^T`, `dt`), the
tutorial uses the rendered LaTeX form ($\mathbf{H}^T$, $\Delta t$); the
underlying objects are identical.

---

## 5. Mapping to FT1 FSW

This tutorial is not a survey — it is the educational input to a specific
implementation. Every chapter has a one-to-one or one-to-few mapping into
the FT1 FSW artifacts the PM will produce out-of-band per
`docs/sdp/index.md` §5 (`USER-NAV-LIB` and `USER-NAV-APP`).

**`docs/design/nav/algorithm.md`** is the authoritative algorithm spec.
Chapter 11 walks it section-by-section and is the primary mapping
artifact. Specifically:

- Chapters 04 + 06 derive the EKF equations cited at
  `docs/design/nav/algorithm.md` §3.2 step 8 (covariance propagation
  $\mathbf{P}_{\text{new}} = \mathbf{F}\,\mathbf{P}_{\text{old}}\,\mathbf{F}^T + \mathbf{Q}$,
  algorithm.md lines 145–146) and the analytic-Jacobian mandate at
  algorithm.md lines 147–173.
- Chapters 08 + 09 derive the strapdown equations corresponding to
  algorithm.md §3.2 steps 1–7 (lines 113–144): bias correction, accel
  rotation to NED, gravity subtraction, velocity and position integration,
  attitude propagation by small-rotation quaternion, and bias evolution as
  zero-mean random walk.
- Chapter 10 derives the measurement-update equations at algorithm.md §4.1
  (baro update, lines 192–217) and §4.2 (GPS update, lines 219–238),
  including the divergence-bound check at §4.3 (lines 240–254) and the
  Joseph form recommended at §6 (lines 351–357).
- Chapter 11 binds every state in algorithm.md §3.1 (lines 73–82, the
  16-state composition `tPosLla` + `tVelNed` + `tAttQuat` + `tAccelBias` +
  `tGyroBias`) to the C++ field on `juno::nav::NAV_STATE_T` and every
  caller-supplied noise field in algorithm.md §5.1 (lines 286–297) to the
  corresponding member of `NAV_INIT_T`.

**`docs/design/nav/design.md`** is the public API contract (vtable shape,
state machine, message catalog, per-call preconditions). Chapter 11
references it for symbol names; the API is intentionally
**algorithm-stable** per design.md §3.2, so the math you learn from this
tutorial maps to a stable C++ surface.

**`docs/requirements/nav/requirements.json`** is the requirement baseline
(`SW-REQ-NAV-001` through `SW-REQ-NAV-019`). The tutorial does not enumerate
each requirement — that is the design documents' job — but the chapter
prerequisites and exercises are sized to ensure that a reader who completes
chapter 12 can read each requirement and identify the chapter that
explains its rationale.

**`libs/nav_lib/`** is where the implementation will live (PM-owned per
SDP risk register **SDP-R-08**). The `nav_lib` IMPL will instantiate
`juno::kmat::MAT_T<double, kInternalDim, kInternalDim>` for the covariance
$\mathbf{P}$, `juno::kmat::VEC_T<double, kInternalDim>` for the state, and
`juno::kmat::QUAT_T<double>` for the attitude (chapter 11 lists every
kmat symbol used). The 23 unit test cases at
`docs/test_cases/nav/test_cases.json` (`SW-TC-NAV-001` through
`SW-TC-NAV-023`) are the pass/fail bar.

**`apps/nav_app/`** is the schedule-driven application that hosts
`nav_lib` (also PM-owned per SDP-R-08). It performs all bus
subscriptions and publish operations, the phase-aware gating during BOOST
and the 1-second post-boost settling window
(`docs/design/nav/algorithm.md` §4.4, lines 257–264), and drives the
filter at the cadences pinned in design.md §4. The 18 unit test cases at
`docs/test_cases/nav_app/test_cases.json` (`SW-TC-NAV-APP-001` through
`SW-TC-NAV-APP-018`) are the pass/fail bar.

**Same gates as any agent-produced library.** Per `docs/sdp/index.md` §10
exit criteria 3 and 4, the PM-owned implementations pass the same G1
(POSIX build + ctest) + G2 (`tools/traceability.py`) + G3 (Pico2
cross-compile) gates that every agent sprint passes — this tutorial is not
a substitute for those gates; it is the educational on-ramp to them.

---

## 6. References

The list below is the **complete authoritative bibliography** for the
tutorial. Every non-trivial mathematical claim made in any chapter must
cite one of the seven entries below using the `[N, §X]` form (e.g.,
`[1, §14.2]` for the Groves analytic-Jacobian derivation). Entries are
numbered in IEEE order; URLs and DOIs have been verified by sibling
workers and are reproduced verbatim from `docs/design/nav/algorithm.md`
§3.2 references where they overlap.

[1] **P. D. Groves**, *Principles of GNSS, Inertial, and Multisensor
Integrated Navigation Systems*, 2nd ed. Boston, MA, USA: Artech House,
2013. ISBN: 978-1-60807-005-3.
**Recommended primary text.** Chapters 14.2 ("Inertial Navigation System
Error Equations") and 14.3.1 ("INS Error State EKF") are normative for
FT1's analytic $\mathbf{F}$ Jacobian per
`docs/design/nav/algorithm.md` §3.2 (algorithm.md lines 152–158).

[2] **R. G. Brown and P. Y. C. Hwang**, *Introduction to Random Signals
and Applied Kalman Filtering*, 4th ed. Hoboken, NJ, USA: Wiley, 2012.
ISBN: 978-0-470-60969-9.
Standard introductory text for the linear Kalman filter and the EKF;
referenced primarily in chapters 02, 04, and 06 for accessible
derivations of probabilistic conjugacy results and the Joseph form.

[3] **R. E. Kalman**, "A new approach to linear filtering and prediction
problems," *Journal of Basic Engineering*, vol. 82, no. 1, pp. 35–45,
Mar. 1960. DOI: [10.1115/1.3662552](https://doi.org/10.1115/1.3662552).
The original Kalman filter paper. Cited in chapter 04 for historical
context; not a primary derivation reference (Kalman's original notation
differs from modern usage).

[4] **N. Trawny and S. I. Roumeliotis**, "Indirect Kalman filter for 3D
attitude estimation," University of Minnesota, Minneapolis, MN, USA, MARS
Lab Tech. Rep. TR-2005-002, Mar. 2005. [Online]. Available:
https://www-users.cs.umn.edu/~trawny/Publications/Quaternions_3D.pdf
Quaternion-error-state Jacobian blocks (§3.5, eqs. 147–150) consistent
with the Hamilton-convention quaternion used by `juno::kmat`. Cited in
chapter 08 (quaternion error-state) and chapter 06 (EKF Jacobian for
attitude states); also the secondary normative reference per
`docs/design/nav/algorithm.md` §3.2 (algorithm.md lines 159–165).

[5] **J. Solà**, "Quaternion kinematics for the error-state Kalman filter,"
*arXiv preprint*, arXiv:1711.02508 [cs.RO], Nov. 2017. [Online]. Available:
https://arxiv.org/abs/1711.02508
Accessible quaternion-EKF tutorial with worked-out small-perturbation
algebra. Useful supplement for chapters 06 and 08; non-normative.

[6] **J. A. Farrell**, *Aided Navigation: GPS with High-Rate Sensors*. New
York, NY, USA: McGraw-Hill, 2008. ISBN: 978-0-07-149329-1.
GPS+INS integration reference. Cited in chapter 09 (strapdown
mechanization) and chapter 10 (GPS-aided INS error budget).

[7] **Y. Bar-Shalom, X.-R. Li, and T. Kirubarajan**, *Estimation with
Applications to Tracking and Navigation: Theory, Algorithms and Software*.
New York, NY, USA: Wiley, 2001. ISBN: 978-0-471-41655-5.
Comprehensive estimation theory text. Cited in chapter 04 (alternative
derivation) and chapter 12 (advanced exercises only).

> **Citation discipline.** The phrase "it is well known that..." is
> banned in every chapter. Every theorem, derivation, named result, or
> non-trivial formula must carry a `[N, §X]` citation against the list
> above. Statements that are genuinely standard (e.g., the definition of
> matrix transpose) need no citation. When in doubt, cite. References
> outside this list are **not** permitted in chapters 01–12 — extending
> the list requires PM sign-off and an index revision.

---

## 7. Document Self-Containment Note

This index file is intentionally **self-contained**: a reader can answer
"what is this tutorial, who is it for, what order do I read it in, what
notation does it use, and what books does it reference" without opening
any of the 12 chapters. The chapters reference back to this file (notation
table §4, references §6) but never duplicate its content.

---

## 8. Revision History

| Revision | Date       | Author                                | Notes                                                                                                                                                          |
|----------|------------|---------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| A        | 2026-05-08 | Software Systems Engineer (worker)    | Initial issue — index, audience, reader's guide, chapter index, canonical notation, references, FT1 mapping. Chapters 01–12 authored in parallel by siblings. |

<!-- @{"design": ["SW-REQ-NAV-018", "SW-REQ-NAV-019"]} -->
