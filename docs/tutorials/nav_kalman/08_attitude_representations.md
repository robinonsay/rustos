---
document_type: Tutorial Chapter — 08 of 12
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_index: docs/tutorials/nav_kalman/index.md
title: "Attitude Representations: Euler Angles and Quaternions"
prerequisites: Chapter 07 (Coordinate Frames and Transformations)
reading_time: ~3.0 h active
---

# Chapter 08 — Attitude Representations: Euler Angles and Quaternions

> Chapter 07's DCM is correct but carries 9 numbers for 3 DoF. This chapter
> covers the two compressed parameterizations: **Euler angles** (intuitive,
> singular) and **quaternions** (non-singular). FT1 uses quaternions —
> `tAttQuat` (`algorithm.md` line 79) is a Hamilton, scalar-first, body→NED
> unit quaternion. Ends with propagation and renormalization rules driving
> the nav_lib prediction loop (algorithm.md lines 136–140 and 343–351).

---

## 1. What is "attitude"?

**Attitude** is the orientation of one frame relative to another. For
FT1 the relevant pair is **body** relative to **NED**: how is the
airframe rotated, right now, relative to the local-tangent-plane
North-East-Down frame at the launch point? Attitude is a **3-DoF**
quantity — three independent angles fully describe a rigid-body
orientation. Three degrees of freedom can be packaged as:

- **9 numbers** with 6 constraints — the $3\times3$ DCM
  ($\mathbf{R}^T\mathbf{R} = \mathbf{I}$, $\det\mathbf{R} = +1$).
- **3 numbers**, no constraints — Euler angles (roll, pitch, yaw).
- **4 numbers** with 1 constraint — the unit quaternion
  ($q_w^2 + q_x^2 + q_y^2 + q_z^2 = 1$).

All three encode the same physical rotation; they differ in arithmetic
ergonomics and in what fails when. See §11 for the comparison table.

---

## 2. Euler angles (roll, pitch, yaw)

Euler angles describe a rotation as **three sequential rotations about
three axes**. Twelve valid conventions exist (six Tait–Bryan, six
classical Euler) depending on the axis order. The aerospace convention
is **Tait–Bryan ZYX** (yaw-pitch-roll, "3-2-1"):

1. Rotate by yaw $\psi$ about the world $z$-axis.
2. Rotate by pitch $\theta$ about the **new** $y$-axis.
3. Rotate by roll $\phi$ about the **final** $x$-axis.

The composite body→NED rotation matrix is the product of three
elementary rotations (each a textbook 2D rotation embedded in 3D, see
chapter 07 §3):

$$
\mathbf{R}_{B\rightarrow N}(\phi, \theta, \psi)
= \mathbf{R}_z(\psi)\,\mathbf{R}_y(\theta)\,\mathbf{R}_x(\phi).
$$

Multiplying out, with $c_\alpha = \cos\alpha$, $s_\alpha = \sin\alpha$
[1, §2.2.1]:

$$
\mathbf{R}_{B\rightarrow N} =
\begin{pmatrix}
c_\theta c_\psi & s_\phi s_\theta c_\psi - c_\phi s_\psi & c_\phi s_\theta c_\psi + s_\phi s_\psi \\
c_\theta s_\psi & s_\phi s_\theta s_\psi + c_\phi c_\psi & c_\phi s_\theta s_\psi - s_\phi c_\psi \\
-s_\theta       & s_\phi c_\theta                        & c_\phi c_\theta
\end{pmatrix}
$$

FT1 does **not** use Euler angles for the EKF state; we introduce them
only so the reader recognizes the parameterization in textbooks or in
3rd-party guidance code.

---

## 3. Gimbal lock — why FT1 does not store Euler angles

The Tait–Bryan ZYX matrix collapses at pitch $\theta = \pm 90°$:
$c_\theta = 0$, so yaw and roll act about the **same** physical axis,
and only $\phi \pm \psi$ is observable. The parameterization has
**lost a degree of freedom** — this is **gimbal lock** [1, §2.2.1].
An EKF storing $(\phi, \theta, \psi)$ as state has a numerical
singularity: $\partial\mathbf{R}/\partial\psi \equiv 0$, so the filter
cannot update yaw from any measurement.

For FT1 this is **fatal**: a vertically-launched sounding rocket spends
the first second or two of boost at $\theta \approx +90°$. An Euler-angle
filter would either explode (ill-posed Kalman update) or silently freeze
its yaw. So the EKF stores **a quaternion**, which has no such singularity.

> **Aside.** Gimbal lock is a *parameterization* defect, not a physical
> defect. A topological theorem says no smooth global 3-coordinate chart
> on the rotation group $SO(3)$ exists — *any* 3-parameter
> parameterization goes singular somewhere. To avoid it, use 4
> (quaternion) or 9 (DCM) numbers with constraints.

---

## 4. Quaternions — definition and convention

A **quaternion** is a 4-tuple $\mathbf{q} = (q_w,\,q_x,\,q_y,\,q_z)$.
When the 4-tuple has unit norm
$\lvert\mathbf{q}\rvert^2 = q_w^2 + q_x^2 + q_y^2 + q_z^2 = 1$, it
encodes a rotation in 3D space.

**FT1 conventions** — pinned by `docs/design/nav/algorithm.md` line 79
(`tAttQuat (w, x, y, z)`: unit quaternion, body→NED, Hamilton):

- **Scalar-first storage.** $q_w$ at index 0; $(q_x, q_y, q_z)$ at
  indices 1–3. (Some texts and Eigen's `Quaterniond` accessors put the
  scalar last; FT1 is **scalar-first**.)
- **Hamilton convention** (vs. JPL). The two conventions differ in the
  sign of the cross-product term in quaternion multiplication and are
  not interchangeable. FT1 follows Hamilton, consistent with
  `juno::kmat` and Solà [5, §1.2]. Trawny & Roumeliotis [4] uses JPL
  and must be sign-translated — see §6.
- **Body→NED rotation direction.** $\mathbf{q}$ takes a vector
  expressed in body and returns the same physical vector in NED.

The unit-norm constraint is what makes a quaternion encode a *rotation*
rather than a general 4-tuple. Floating-point arithmetic slowly
violates the constraint — see §10.

---

## 5. Geometric interpretation — axis–angle and the half-angle

Every 3D rotation can be written as "rotate by angle $\theta$ about
unit-axis $\hat{\mathbf{n}}$" (Euler's rotation theorem [1, §2.2.2]).
A unit quaternion encodes the same axis–angle pair via

$$
\mathbf{q} = \big(\cos(\theta/2),\;\hat{n}_x \sin(\theta/2),\;\hat{n}_y \sin(\theta/2),\;\hat{n}_z \sin(\theta/2)\big).
$$

The **half-angle** is counterintuitive: $q_w = \cos(\theta/2)$, not
$\cos(\theta)$. So a 360° rotation gives $\mathbf{q} = (-1, 0, 0, 0)$,
not the identity; 720° returns to $(+1, 0, 0, 0)$. The map quaternion
→ rotation is **two-to-one**: $\mathbf{q}$ and $-\mathbf{q}$ encode the
same rotation (the **double cover** of $SO(3)$). Every rotation formula
is invariant under $\mathbf{q} \mapsto -\mathbf{q}$; the sign matters
only in slerp (pick the shorter arc) and logging [5, §1.2].

> **Worked example.** A 90° yaw (rotation about body $z$):
> $\theta = \pi/2$, $\hat{\mathbf{n}} = (0, 0, 1)$, so
> $\mathbf{q} = (\cos(\pi/4), 0, 0, \sin(\pi/4)) \approx (0.7071, 0, 0,
> 0.7071)$. Norm $0.7071^2 + 0.7071^2 = 1.0$ ✓.

---

## 6. Quaternion multiplication (Hamilton product)

Quaternion multiplication, denoted $\otimes$, is the defining operation
on quaternions. If $\mathbf{q}_1$ encodes rotation $R_1$ and
$\mathbf{q}_2$ encodes $R_2$, then $\mathbf{q}_1 \otimes \mathbf{q}_2$
encodes the composition "first apply $R_2$, then $R_1$" — exactly like
matrix multiplication of DCMs. The Hamilton-convention component
formula is [4, eqs. 4–7] (sign-translated from JPL scalar-last to
Hamilton scalar-first; verified against [5, §1.3]):

$$
\begin{aligned}
(\mathbf{q}_1 \otimes \mathbf{q}_2)_w &= q_{1w} q_{2w} - q_{1x} q_{2x} - q_{1y} q_{2y} - q_{1z} q_{2z} \\
(\mathbf{q}_1 \otimes \mathbf{q}_2)_x &= q_{1w} q_{2x} + q_{1x} q_{2w} + q_{1y} q_{2z} - q_{1z} q_{2y} \\
(\mathbf{q}_1 \otimes \mathbf{q}_2)_y &= q_{1w} q_{2y} - q_{1x} q_{2z} + q_{1y} q_{2w} + q_{1z} q_{2x} \\
(\mathbf{q}_1 \otimes \mathbf{q}_2)_z &= q_{1w} q_{2z} + q_{1x} q_{2y} - q_{1y} q_{2x} + q_{1z} q_{2w}
\end{aligned}
$$

Properties: associative, distributive, **not commutative**
($\mathbf{q}_1 \otimes \mathbf{q}_2 \neq \mathbf{q}_2 \otimes \mathbf{q}_1$
in general). Identity $\mathbf{q}_I = (1, 0, 0, 0)$. Product of two
unit quaternions is itself unit (exactly in real arithmetic; drifts in
floating-point — see §10).

**FT1 implementation.** The Hamilton product is published in
`juno::math` (re-exported into `juno::kmat`) as **`HamProd`** per
`docs/design/kmat/04_interface.md` §4.6.2 line 311:
`QUAT<T> HamProd(const QUAT<T> &q0, const QUAT<T> &q1) noexcept`.
`algorithm.md` line 139 calls the same operation `QuatMul` for
readability. The C++ `operator*` overload on two `QUAT<T>` is also
equivalent (kmat §4.6.2 lines 318–320).

> **Numerical verification.** Compose 90° yaw with itself — the result
> should encode 180° yaw, $\mathbf{q} = (0, 0, 0, 1)$. With
> $\mathbf{q}_1 = \mathbf{q}_2 = (0.7071, 0, 0, 0.7071)$ from §5:
> $w = 0.7071^2 - 0.7071^2 = 0$ ✓; $x = y = 0$ ✓;
> $z = 0.7071^2 + 0.7071^2 = 1$ ✓. So $\mathbf{q}_1 \otimes \mathbf{q}_2
> = (0, 0, 0, 1)$, the 180° yaw quaternion.

---

## 7. Rotating a vector by a quaternion

To rotate a body-frame vector $\mathbf{v}^B$ into NED via a unit
quaternion $\mathbf{q}$, the literature uses the **conjugation
sandwich** [5, §1.4]:

$$
\big[\,0;\,\mathbf{v}^{NED}\,\big]
= \mathbf{q} \otimes \big[\,0;\,\mathbf{v}^{B}\,\big] \otimes \mathbf{q}^*
$$

where $\big[0;\mathbf{v}\big]$ is the **pure quaternion** with zero
scalar part and $\mathbf{v}$ as its vector part, and the **conjugate**
$\mathbf{q}^*$ is

$$
\mathbf{q}^* = (q_w,\,-q_x,\,-q_y,\,-q_z).
$$

The output is again pure (zero scalar; preserved exactly when
$\mathbf{q}$ is unit) and its vector part is $\mathbf{v}^{NED}$.

> **Conjugate vs. inverse.** For unit quaternions, $\mathbf{q}^* =
> \mathbf{q}^{-1}$. For non-unit quaternions, $\mathbf{q}^{-1} =
> \mathbf{q}^* / \lvert\mathbf{q}\rvert^2$. The kmat `Recip` operation
> (§4.6.2 line 314) computes the general inverse and **does not check**
> for non-zero norm. Don't conflate the two on a non-unit quaternion.

**FT1 implementation.** `juno::kmat::QuatRotate` (kmat §4.6.3 line 358;
`VEC<T,3> QuatRotate(const QUAT<T> &q, const VEC<T,3> &v) noexcept`) is
the kmat-original kinematics operation. Per kmat §4.6.3 line 365 the
canonical implementation is `MatVecMul(QuatToMat3(q), v)` — kmat goes
through the DCM rather than the direct sandwich, so the rotation has
exactly one set of floating-point rounding errors.

**Where this is used in nav_lib.** Step 2 of the prediction loop
(`algorithm.md` line 119) rotates body-frame acceleration into NED via
`a_ned_meas = QuatRotate(q, a_meas_body)`. Every IMU sample passes
through `QuatRotate`.

---

## 8. Quaternion → DCM

Sometimes you want the explicit $3\times3$ rotation matrix associated
with a quaternion. Multiplying out the sandwich form symbolically gives
[5, §1.4] [1, §2.2.2]:

$$
\mathbf{R}_{B\rightarrow N}(\mathbf{q}) =
\begin{pmatrix}
q_w^2 + q_x^2 - q_y^2 - q_z^2 & 2(q_x q_y - q_w q_z) & 2(q_x q_z + q_w q_y) \\
2(q_x q_y + q_w q_z) & q_w^2 - q_x^2 + q_y^2 - q_z^2 & 2(q_y q_z - q_w q_x) \\
2(q_x q_z - q_w q_y) & 2(q_y q_z + q_w q_x) & q_w^2 - q_x^2 - q_y^2 + q_z^2
\end{pmatrix}
$$

For a unit quaternion the diagonal entries simplify to
$1 - 2(q_y^2 + q_z^2)$, $1 - 2(q_x^2 + q_z^2)$, $1 - 2(q_x^2 + q_y^2)$
— the form published in `docs/design/kmat/04_interface.md` §4.6.3 lines
352–356 (the `QuatToMat3` matrix-layout block).

**FT1 implementation.** `juno::kmat::QuatToMat3` (kmat §4.6.3 line 339)
returns this matrix as `MAT_T<T, 3, 3>` row-major. The same matrix is
used internally by `QuatRotate`.

> **Worked example — 30° pitch about body $y$-axis.** $\theta = 30°$,
> $\hat{\mathbf{n}} = (0, 1, 0)$, so $\theta/2 = 15°$ and $\mathbf{q} =
> (\cos 15°, 0, \sin 15°, 0) \approx (0.9659, 0, 0.2588, 0)$. Plug in:
> $R_{11} = 0.9659^2 - 0.2588^2 = 0.8660 = \cos 30°$ ✓;
> $R_{13} = 2(0.9659 \cdot 0.2588) = 0.5000 = \sin 30°$ ✓;
> $R_{22} = 1.0$, $R_{31} = -\sin 30°$, $R_{33} = \cos 30°$ ✓. The full
> DCM is exactly the elementary $\mathbf{R}_y(30°)$ from chapter 07 §3
> — quaternion and DCM agree.

---

## 9. Quaternion propagation — integrating the gyro

The IMU's gyroscope measures the body-frame angular velocity vector
$\boldsymbol{\omega}^B$ (rad/s, three components about body $x, y, z$).
To advance the attitude quaternion by one IMU step
$\Delta t \approx 5\,\mathrm{ms}$ (per `algorithm.md` §3.2 line 105,
"kImuAppPeriodMs = 5 ms"), use the **axis–angle-to-quaternion**
small-step recipe [5, §4.6.1].

**Step 1 — build the small-rotation quaternion $\delta\mathbf{q}$.**
Treat the rotation over the interval as "rotate by angle
$\lvert\boldsymbol{\omega}\rvert\,\Delta t$ about unit axis
$\hat{\boldsymbol{\omega}} = \boldsymbol{\omega}/\lvert\boldsymbol{\omega}\rvert$"
(constant angular velocity over the step; acceptable at 200 Hz). Apply
§5's axis–angle formula:

$$
\delta\mathbf{q} = \left(\cos\!\left(\tfrac{\lvert\boldsymbol{\omega}\rvert\Delta t}{2}\right),\;
\frac{\sin(\lvert\boldsymbol{\omega}\rvert\Delta t/2)}{\lvert\boldsymbol{\omega}\rvert}\,\boldsymbol{\omega}\right).
$$

For very small $\lvert\boldsymbol{\omega}\rvert\Delta t$, the small-angle
approximation $\delta\mathbf{q} \approx (1,\,\tfrac12\,\boldsymbol{\omega}\Delta t)$
avoids a 0/0 in the formula; Solà [5, §4.6.1] gives the cross-over
magnitude in double precision as roughly $\lvert\boldsymbol{\omega}\rvert\Delta t < 10^{-8}$ rad.

**Step 2 — compose with the current attitude.** Per `algorithm.md`
line 138–139, "Hamilton product, body-frame increment on the right":

$$
\mathbf{q}_\text{new} = \mathbf{q}_\text{old} \otimes \delta\mathbf{q}.
$$

**Why right-multiply?** $\mathbf{R}_\text{new} = \mathbf{R}_\text{old}\,\mathbf{R}_{\delta}$
when $\mathbf{R}_\delta$ is expressed in the **body** frame; in Hamilton
quaternions this becomes right-multiplication. An NED-expressed
(inertial) increment would left-multiply; FT1's gyro reports body-frame
rates, so right-multiply [5, §4.6].

**FT1 implementation.** Step 6 of the prediction loop (`algorithm.md`
lines 136–140) forms $\delta\mathbf{q}$ from $\boldsymbol{\omega}^B\Delta t$,
computes `q_new = QuatMul(q_old, dq)` ("Hamilton product, body-frame
increment on the right"), then renormalizes per §10. kmat symbol is
`juno::kmat::HamProd` (kmat §4.6.2 line 311); `algorithm.md` calls it
`QuatMul`.

---

## 10. Renormalization — keeping $\lvert\mathbf{q}\rvert = 1$

Floating-point arithmetic does not preserve the unit-norm constraint.
Each `HamProd` introduces relative error of order $10^{-16}$; over
thousands of IMU steps the quaternion drifts off the unit 3-sphere. A
quaternion with $\lvert\mathbf{q}\rvert \neq 1$ no longer encodes a pure
rotation: `QuatToMat3` returns a *scaled* matrix (kmat §4.6.3 line 345),
and `QuatRotate` produces a length-non-preserving rotation that corrupts
velocity and position.

**The fix.** After every prediction step, project back onto the unit
3-sphere:

$$
\mathbf{q} \leftarrow \frac{\mathbf{q}}{\lvert\mathbf{q}\rvert} = \frac{\mathbf{q}}{\sqrt{q_w^2 + q_x^2 + q_y^2 + q_z^2}}.
$$

**FT1 implementation.** `juno::kmat::QuatNormalize` (kmat §4.6.3 line
328); `RESULT_T<QUAT<T>> QuatNormalize(const QUAT<T> &q) noexcept`.
Returns $\mathbf{q} / \sqrt{\lvert\mathbf{q}\rvert^2}$ on success.

**Failure mode.** If $\lvert\mathbf{q}\rvert^2 < \mathtt{kPivotEpsilon}^2$
(`kPivotEpsilon<double>() = 10^{-30}`, kmat §4.8 line 418), it returns
`juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR`. Per `algorithm.md` §6 lines
343–351, nav_lib must then transition to `Diverged` and return
`juno::nav::JUNO_FSW_STATUS_DIVERGED_ERROR` (design.md §4.5).

"After every prediction step" is mandatory — even one or two un-normalized
propagations distort rotated acceleration and corrupt velocity integration.

---

## 11. Why FT1 uses quaternions

| Property | Euler (3) | DCM ($3\times3$, 9) | Quaternion (4) |
|----------|-----------|---------------------|----------------|
| Constraints | none | 6 (orthogonality) | 1 (unit norm) |
| Singularities | gimbal lock | none | none |
| Composition cost | trig | 27 mul + 18 add | 16 mul + 12 add |
| Smooth interpolation | painful | painful | natural (slerp) |
| EKF suitability | poor | poor (over-parameterized) | excellent |

For a vertically-launched rocket passing through $\theta = +90°$ pitch
during boost, the gimbal-lock entry alone forces the choice [5, §1.1].
FT1 stores the quaternion as the EKF attitude state (`tAttQuat`,
`algorithm.md` line 79); uses `HamProd` / `QuatRotate` / `QuatToMat3` in
the prediction step (`algorithm.md` lines 119, 139); and `QuatNormalize`
immediately after (`algorithm.md` lines 343–351). Chapter 09 folds these
into the full strapdown mechanization.

---

## 12. Key Results

- **3 DoF.** Euler: 3 numbers / 0 constraints / singular at $\pm 90°$
  pitch. Quaternion: 4 numbers / 1 unit-norm constraint / no
  singularities.
- **Gimbal lock.** Tait–Bryan ZYX has a $\phi - \psi$ Jacobian rank
  deficit at $\theta = \pm 90°$, so yaw becomes unobservable; therefore
  FT1 does not store Euler angles in the EKF state.
- **FT1 quaternion conventions.** Hamilton, scalar-first, body→NED,
  unit norm — pinned by `algorithm.md` line 79.
- **Hamilton product.** §6 component formula (verified: 90° + 90° yaw
  = 180° yaw); `juno::kmat::HamProd` (kmat §4.6.2 line 311), called
  `QuatMul` by `algorithm.md` line 139.
- **Vector rotation.** $\mathbf{v}^{NED}=$ vector part of
  $\mathbf{q}\otimes[0;\mathbf{v}^B]\otimes\mathbf{q}^*$ with conjugate
  $\mathbf{q}^* = (q_w,-q_x,-q_y,-q_z)$; `juno::kmat::QuatRotate`
  (kmat §4.6.3 line 358); used at `algorithm.md` line 119.
- **Quaternion → DCM.** Matrix in §8; 30°-pitch example agrees with
  chapter 07's $\mathbf{R}_y(30°)$.
- **Propagation.** $\delta\mathbf{q}$ from axis–angle, then
  $\mathbf{q}_\text{new}=\mathbf{q}_\text{old}\otimes\delta\mathbf{q}$
  (right-multiply, body-frame increment) — `algorithm.md` lines 136–140.
- **Renormalize after every prediction step.** Mandated by
  `algorithm.md` §6 lines 343–351; `juno::kmat::QuatNormalize` returns
  `JUNO_FSW_STATUS_NUMERIC_ERROR` if $\lvert\mathbf{q}\rvert <
  \mathtt{kPivotEpsilon}$, in which case nav_lib transitions to
  `Diverged`.
- **Conjugate vs. inverse.** Equal for unit quaternions; differ by
  $\lvert\mathbf{q}\rvert^2$ otherwise.

---

## 13. Exercises

(Worked solutions in chapter 12.)

1. Verify by hand that $\mathbf{q} = (0, 1, 0, 0)$ encodes a 180°
   rotation about body $x$ (roll-over). Compute its DCM via §8 and
   show it agrees with $\mathbf{R}_x(180°)$ from chapter 07.
2. The 90°-yaw composition in §6 is commutative because both rotations
   share an axis. Construct an explicit pair of quaternions whose
   Hamilton products in the two orders differ.
3. Show that `QuatNormalize` is idempotent on unit quaternions: applying
   it twice gives the same result as once (up to floating-point).
4. Implement `HamProd` in 12 lines of pseudocode from §6; test against
   a NumPy reference on 100 random unit quaternions.
5. Double cover: pick a quaternion $\mathbf{q}$ and verify that
   $\mathbf{q}$ and $-\mathbf{q}$ produce the same DCM via §8 (every
   entry depends on products of two quaternion components, so the sign
   cancels).
6. Gimbal-lock numerical demo: at $\theta = 89.9°$, compute the
   condition number of the Jacobian
   $\partial\mathbf{R}/\partial(\phi, \theta, \psi)$ for the ZYX Euler
   matrix and observe that it grows like $1/\cos\theta$ as
   $\theta \to 90°$. Quaternions have no analogous degenerate Jacobian.

---

## 14. Citations

- [1, §2.2.1] Tait–Bryan ZYX matrix; gimbal-lock characterization.
- [1, §2.2.2] Euler's rotation theorem; quaternion → DCM formula.
- [4, eqs. 4–7] Hamilton-product component formula (sign-translated
  to scalar-first Hamilton from Trawny–Roumeliotis's scalar-last JPL
  presentation; consistent with `juno::kmat`'s Hamilton convention).
- [5, §1.1] Comparative table of attitude representations.
  [5, §1.2] Hamilton vs. JPL; double cover.
  [5, §1.3] Hamilton product.
  [5, §1.4] Conjugation sandwich; quaternion → DCM.
  [5, §4.6, §4.6.1] Quaternion propagation; axis–angle small-step.
- `docs/design/nav/algorithm.md` **line 79** (tAttQuat unit, body→NED,
  Hamilton, scalar-first); **line 119** (`QuatRotate` rotates body
  accel into NED, step 2); **lines 136–140** (right-multiplied
  body-frame quaternion propagation, step 6); **lines 343–351**
  (mandatory renormalization after every prediction step; `Diverged`
  transition on failure).
- `docs/design/kmat/04_interface.md` **§4.6.2 line 311** (`HamProd`,
  the Hamilton product algorithm.md calls `QuatMul`);
  **§4.6.3 line 339** (`QuatToMat3`); **§4.6.3 line 358**
  (`QuatRotate`); **§4.6.3 line 328** (`QuatNormalize` signature,
  postconditions, `JUNO_FSW_STATUS_NUMERIC_ERROR` failure).

> **Onward.** Chapter 09 stitches the quaternion propagation in §9
> with the velocity and position integration from chapter 07 to derive
> the full strapdown mechanization (`algorithm.md` §3.2 steps 1–7).

<!-- @{"design": ["SW-REQ-NAV-001", "SW-REQ-NAV-002"]} -->
