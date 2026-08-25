---
document_type: Tutorial Chapter — Coordinate Frames and Transformations
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
sprint: SPRINT-IMPL-NAV-TUTORIAL
parent_index: docs/tutorials/nav_kalman/index.md
chapter_number: 07
covers: SW-REQ-NAV-018
prerequisites: Chapters 01–06 (math primer, KF, EKF)
---

# Chapter 07 — Coordinate Frames and Transformations

> **Goal.** Teach the three coordinate frames the FT1 nav system uses
> (ECEF, NED, body), the rotation matrix (DCM) that transforms
> vectors between them, and the specific conversions in
> [`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md).
> After this chapter the reader is ready for chapter 08, where the
> body→NED DCM is regenerated from a quaternion. Worked examples:
> §4.2 (30°-pitch DCM) and §8B (three vehicle attitudes — A up, B
> flat north, C flat east). Citations use `[N, §X]` against
> [`index.md`](index.md) §6.

---

## 1. What is a Frame?

A **coordinate frame** is a choice of (a) an **origin** in space and
(b) three mutually-orthogonal **basis axes**. Once both are fixed,
every point has a unique triple of numbers and every vector has a
triple of components along the basis axes.

**The same physical vector has different number-tuples in different
frames.** This is the entire reason coordinate frames matter. A pencil
has one physical length and direction; its $(x, y, z)$ depends on
whether you measure relative to the desk, the room, or Earth's centre.

**Why navigation needs more than one frame.** Each FT1 sensor measures
in its **natural frame**: IMU in **body** (chips bolted to vehicle),
GPS in a **world** frame (WGS-84 geodetic), barometer in a **local**
vertical (altitude vs reference pressure). To fuse those in a Kalman
filter (chapters 04, 06) they must all be in **one common frame** —
you cannot add a body-frame acceleration to a world-frame position. So
every measurement is **transformed** into the filter frame. For FT1
that is **NED at the launch site** (§7). Frames sharing an origin but
not axes are related by **pure rotation**; sharing axes but not
origins, **pure translation**; differing in both, a **rigid-body
transform**.

---

## 2. The Three Frames in FT1

FT1 uses exactly three frames, pinned by `docs/design/conventions.md`
§4.6 and
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) §3.1.
$+z$ is a common point of confusion — read carefully.

| Frame | Origin | $x$ axis | $y$ axis | $z$ axis | Used by |
|-------|--------|----------|----------|----------|---------|
| **ECEF** (Earth-Centered Earth-Fixed) | Earth's centre of mass | Equator at $0°$ longitude (prime meridian) | Right-handed (equator at $90°$ E) | Geographic North Pole | GPS receivers; absolute position [1, §2.4] |
| **NED** (North-East-Down) | Reference geodetic point — for FT1, **launch site** | **North** (local meridian) | **East** (local parallel) | **Down** (local gravity) | EKF velocity `tVelNed` (algorithm.md line **78**); body→NED quaternion |
| **Body** | Vehicle design centre | **Forward** (tail-to-nose) | **Right** | **Down** (through bottom) | IMU accel (`tAccelBodyMps2`), gyro (`tGyroBodyRps`) |

ECEF is **Earth-fixed** — rotates with the Earth, one revolution per
sidereal day relative to inertial space. (Sometimes ECEF-r to
distinguish from ECI; FT1 has no ECI.)

**Down is positive in NED.** Not a typo. **In NED, +z points DOWN.**
A rocket climbing 100 m has $\Delta z = -100$. Gravity in NED is
$\mathbf{g}^{NED} = (0,\,0,\,+9.80665)\,\mathrm{m/s^2}$, exactly the
convention in
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) line
**123** ("gravity points down in NED"). Writing `g = -9.80665` would
be **ENU**, a different frame — every sign in the filter would be
wrong. Local Down differs from the geocentric direction by $< 0.2°$
at most latitudes; ignore for FT1 per [1, §2.5]. NED is also called
the **local tangent plane**: $x$-$y$ tangent to WGS-84 at the
reference point; within a few km, locally Cartesian to better than
0.01% (§6.1).

The body convention is standard **aerospace** ($x$-fwd, $y$-right,
$z$-down) pinned by `SW-REQ-SYS-057`. **Not the only convention** —
many robotics texts use $z$-up. Stick with $z$-down here.

```
   NED at launch site         Body (attached to vehicle)
     x North                       x forward
     ^                             ^
     +--> y East          Body --->|--> y right
     v                             v
     z Down                        z down
```

ECEF rotates with the planet; NED is fixed to launch ground; body is
fixed to the vehicle and tumbles as it tumbles.

---

## 3. Geodetic vs Cartesian Coordinates

ECEF is **Cartesian**: a position is $(x, y, z)$ in metres. GPS
receivers, maps, and humans usually use **geodetic** coordinates —
$(\varphi, \lambda, h)$ for latitude, longitude, height-above-ellipsoid
(HAE). These are two parameterizations of the same point related
through the **WGS-84 ellipsoid** (equatorial radius
$a = 6\,378\,137.0$ m, flattening $f = 1/298.257\,223\,563$): geodetic
latitude is the angle between the local ellipsoid normal and the
equatorial plane; longitude is the angle east from the prime meridian;
HAE is the signed perpendicular distance from the ellipsoid [1, §2.4.1].

The FT1 nav state outputs **geodetic** position per algorithm.md §3.1
line 77 (`tPosLla` carries lat-deg, lon-deg, alt-m HAE) using WGS-84
(algorithm.md line 64; `SW-REQ-SYS-038`/`-039`). Downstream consumers
expect degrees, not ECEF. Standard closed-form geodetic→ECEF
[1, §2.4.1]:

$$
\begin{aligned}
N(\varphi) &= \frac{a}{\sqrt{1 - e^2 \sin^2 \varphi}} \\
x &= (N(\varphi) + h)\cos\varphi\cos\lambda \\
y &= (N(\varphi) + h)\cos\varphi\sin\lambda \\
z &= \big(N(\varphi)(1 - e^2) + h\big)\sin\varphi
\end{aligned}
$$

with $e^2 = 2f - f^2 \approx 6.6943799\times 10^{-3}$ the squared
first eccentricity and $N(\varphi)$ the **prime-vertical radius of
curvature**. The inverse (ECEF→geodetic) is iterative [1, §2.4.1].

**Sanity check.** At $\varphi = 0,\,\lambda = 0,\,h = 0$,
$N(\varphi) = a$ and $(x, y, z) = (6\,378\,137,\,0,\,0)$ — the equator
at $0°$ longitude sits on the ECEF $x$ axis at distance $a$. ✓

You will not implement geodetic↔ECEF in `nav_lib` — the filter
operates in NED with the geodetic update done implicitly via local
radii of curvature (algorithm.md §3.2 step 5). The reason to know the
formula exists is that GPS arrives in geodetic and the state stores
geodetic.

---

## 4. Rotation Matrices (DCM)

A **rotation matrix** (also **direction cosine matrix**, DCM) is a
$3\times 3$ matrix $\mathbf{R}_{B}^{N}$ that takes a vector in frame
$B$ and returns the same physical vector in frame $N$:
$\mathbf{v}^N = \mathbf{R}_{B}^{N}\,\mathbf{v}^B$. The notation
$\mathbf{R}_{B}^{N}$ reads "rotation from $B$ to $N$"; this is the
convention used throughout [1, §2.5] and this tutorial. Subscript =
**source** frame, superscript = **destination** frame. Some textbooks
reverse this — pay attention.

### 4.1 Properties

A matrix $\mathbf{R}$ is a (proper) rotation iff:

1. **Orthogonality.** $\mathbf{R}^T\mathbf{R} = \mathbf{I}$. Columns
   (and rows) are mutually orthogonal unit vectors. Makes rotations
   **length-preserving**: $|\mathbf{R}\mathbf{v}| = |\mathbf{v}|$.
2. **Determinant $+1$.** Distinguishes rotation from reflection
   (det $-1$); preserves handedness.
3. **Inverse = transpose.** $\mathbf{R}^{-1} = \mathbf{R}^T$, so
   $\mathbf{R}_{N}^{B} = (\mathbf{R}_{B}^{N})^T$.
4. **Composition.** Inner frames cancel:
   $\mathbf{R}_{C}^{A} = \mathbf{R}_{B}^{A}\,\mathbf{R}_{C}^{B}$.

### 4.2 Worked example — pitch up by 30°

Rocket on the pad with body $x$ forward (horizontal, "North-ish");
pitch the nose up 30°. In NED, "pitch up" rotates body $x$ toward Up
($z = -1$). The pitch rotation about body $y$ by angle $\theta$ takes
a body vector to NED via the standard pitch-only DCM [6, §2.4]:

$$
\mathbf{R}_{B}^{N}(\theta) =
\begin{pmatrix}
\cos\theta & 0 & \sin\theta \\
0          & 1 & 0          \\
-\sin\theta & 0 & \cos\theta
\end{pmatrix}
$$

Sign pattern matches NED: pitch-up *decreases* $z$ because Down is
positive. For $\theta = 30°$, $\cos 30° \approx 0.8660$,
$\sin 30° = 0.5$:

$$
\mathbf{R}_{B}^{N}(30°) =
\begin{pmatrix}
0.8660 & 0 & 0.5 \\
0      & 1 & 0   \\
-0.5   & 0 & 0.8660
\end{pmatrix}
$$

Rotate $\mathbf{v}^B = (1, 0, 0)$ (body forward) into NED:
$\mathbf{v}^N = \mathbf{R}_{B}^{N}(30°)\,\mathbf{v}^B
= (0.8660,\,0,\,-0.5)$.

Read as physics: body forward is $0.8660$ N, $0$ E, $-0.5$ "down"
($0.5$ up) — "nose pitched 30° above horizontal, pointing north"
(horizontal $\cos 30°$, vertical $\sin 30°$ above horizon). ✓ Length
$\sqrt{0.8660^2 + 0.5^2} = 1$ ✓; determinant (expand row 2)
$1\cdot(0.8660^2 - 0.5\cdot(-0.5)) = 0.75 + 0.25 = 1$ ✓.

---

## 5. Free Vectors vs Positions (brief)

A **free vector** (acceleration, angular velocity, force, velocity)
has magnitude + direction but no anchor — only rotation matters:
$\mathbf{v}^N = \mathbf{R}_{B}^{N}\mathbf{v}^B$. A **position** is
anchored to the frame's origin and needs rotation + translation.
For FT1 nav, IMU acceleration and angular rate are free vectors —
so when algorithm.md line 119 rotates accel to NED, **no translation
appears**. Position is sidestepped by carrying it in **geodetic**
coordinates (already global) rather than as a body-relative offset.
See Groves [1, §2.5.4] for the formal vector-vs-point treatment.

---

## 6. Local-Tangent-Plane Geometry and Composition

NED is **local** — its definition depends on a chosen reference
point. For FT1 the reference is the launch site, fixed at
composition time. Given $\varphi_0,\,\lambda_0$, Groves [1, §2.5]
constructs NED as: Down = inward local ellipsoid normal; North =
projection of Earth's spin axis onto the local tangent plane;
East = $\mathrm{North}\times\mathrm{Down}$. The closed-form
ECEF→NED rotation $\mathbf{R}_{ECEF}^{NED}(\varphi_0, \lambda_0)$
is in [1, §2.5.4]; we don't reproduce it here. Within a few km, NED
is locally Cartesian to better than 0.01% [1, §2.5]; at FT1's scale
(< 1 km horizontal) the curvature error is well below sensor noise.

**Composition.** By §4.1, a body vector reaches ECEF in two
rotations:
$\mathbf{v}^{ECEF} = \mathbf{R}_{NED}^{ECEF}\,\mathbf{R}_{B}^{NED}\,\mathbf{v}^{B}$.
$\mathbf{R}_{B}^{NED}$ is the **attitude**, encoded as the quaternion
in `tAttQuat` (chapter 08); $\mathbf{R}_{NED}^{ECEF}$ is the
transpose of the [1, §2.5.4] matrix. **First-applied rotation goes
on the right.**

For an explicit numerical example of a chained body→NED→ECEF
transform, see Groves [1, §2.4]. The §8B example below builds three
direct body→NED DCMs by inspection — the more useful exercise for
FT1 since ECEF never enters the FT1 state.

---

## 7. Why FT1 Uses NED + Body for the EKF

FT1 is an amateur G-motor rocket; trajectory range is **< 1 km
horizontal and < 500 m vertical**. At that scale:

- **NED is locally Cartesian to better than 0.01%** [1, §2.5];
  curvature error is well below the sensor noise floor (GPS horizontal
  $\sigma \approx 2.5$ m per
  [`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) §5.2,
  orders of magnitude larger).
- **ECEF would be overkill.** Carrying ECEF position would force every
  prediction step to evaluate §3 trig (sin, cos, $N(\varphi)$) at
  200 Hz.
- **ECEF is less interpretable.** "Velocity 3.2 m/s N, 1.1 m/s E,
  0.4 m/s D" vs "$(2.0, -1.7, 1.4)$ m/s ECEF."

So the FT1 filter operates with **NED velocities** (`tVelNed`) and a
**body→NED quaternion** (`tAttQuat`) per algorithm.md §3.1 lines
76–82. State outputs **geodetic** position (`tPosLla`) for downstream
consumers; the geodetic update inside prediction uses local WGS-84
radii of curvature to convert NED position deltas into lat/lon/alt
deltas (algorithm.md §3.2 step 5 lines 128–135). ECEF is **not** in
the FT1 state.

For longer-range vehicles (sounding rockets, orbital, long-range UAV)
the trade-off flips and ECEF wins [1, §14.3]. FT1 is firmly in
"local-tangent-plane is enough" territory.

---

## 8. FSW Anchor — Body→NED Rotation in algorithm.md

Re-read after chapter 08 (quaternions). Excerpt from
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) §3.2
step 2, lines 117–120 (line 119 is the body→NED rotate):

```
1. Bias correction.
   a_meas_body     = tAccelBodyMps2 - tAccelBias
   omega_meas_body = tGyroBodyRps  - tGyroBias
2. Rotate accel to NED. Using the current attitude quaternion q,
   apply the body→NED rotation:
   a_ned_meas = QuatRotate(q, a_meas_body)
   (juno::kmat::QuatRotate, see kmat §4.6).
```

Translate into chapter language:

- `a_meas_body` is a **free vector** (acceleration; §5) in **body**.
- `q` is `tAttQuat`. Per algorithm.md §3.1 line 79 it is a **body→NED**
  rotation in Hamilton-convention form (chapter 08).
- `QuatRotate(q, a_meas_body)` evaluates $\mathbf{v}^{N} =
  \mathbf{R}_{B}^{N}\mathbf{v}^{B}$ — §4–§5's body-to-NED rotation —
  using the quaternion encoding of the DCM. **No translation
  appears**, matching §5's free-vector rule.
- `a_ned_meas` is the same physical acceleration in NED, ready for
  gravity subtraction $\mathbf{g}^{NED} = (0, 0, +9.80665)$
  (algorithm.md line 123) and integration.

`juno::kmat::QuatRotate` is published at
[`docs/design/kmat/04_interface.md`](../../design/kmat/04_interface.md)
§4.6 (specifically §4.6.3):

```cpp
template<typename T>
juno::math::VEC<T, 3>
QuatRotate(const juno::math::QUAT<T> &tQ,
           const juno::math::VEC<T, 3> &tV) noexcept;
```

Postcondition: "Returns the body-frame vector `tV` rotated into NED."
Implementation note (kmat §4.6.3): "canonical implementation is
`MatVecMul(QuatToMat3(tQ), tV)`." The function builds the DCM from the
quaternion (`QuatToMat3`, row-major formula on lines 350–356) and
multiplies. **kmat never stores a separate DCM in the filter** — the
matrix is rebuilt from the quaternion at each call. The quaternion
is the single source of truth for attitude.

**Takeaway for the PM implementing `nav_lib`:** you do not write the
DCM by hand. Store the quaternion, hand it + the body-frame vector to
`QuatRotate`, and kmat does the arithmetic. The only thing you must
keep in your head is **what frame each vector is in**: `a_meas_body`
body; `a_ned_meas`, `g_ned`, `tVelNed` all NED; `tAttQuat` rotates
body→NED. Get the frames straight and the arithmetic is mechanical.

### 8.5 GPS measurement → state (no rotation)

GPS publishes $(\varphi, \lambda, h)$ and $(V_n, V_e, V_d)$. State
`tPosLla` and `tVelNed` use the **same parameterization**. The EKF
observation model is **direct identity** on those six components:
no rotation needed. Per
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) lines
**222–230**: "The GPS receiver reports geodetic position and NED
velocity. The measurement is 6-dimensional (lat, lon, alt, Vn, Ve,
Vd). `h(x)` returns the corresponding 6 states... `H` (6 ×
kInternalDim) has 1.0 entries on the rows for the position and
velocity components and zeros elsewhere. `R_gps` is 6×6 diagonal..."
A trivial measurement Jacobian. **This is why** the FT1 state holds
geodetic position and NED velocity — the GPS update math disappears.

### 8.6 NED displacement → geodetic position update

Velocity integration produces $\Delta\mathbf{p}^{N}$ (metres). State
`tPosLla` is geodetic. Per
[`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md) line
**128**: "Position integration (geodetic update). Compute the NED
position delta `dpos_ned = tVelNed_old · dt + 0.5 · a_ned · dt²`,
then convert to geodetic deltas using the local WGS-84 reference at
the current `tPosLla` (meridional and prime-vertical radii of
curvature). Update `tPosLla[0..2]` accordingly." Roughly: 1 m N
changes latitude by $1/(M+h)$ rad; 1 m E by
$1/((N+h)\cos\varphi)$ rad; 1 m Down changes altitude by $-1$ m.
Closed forms in [1, §2.4]. We don't dwell on formulas — this is how
NED velocity in m/s becomes a lat/lon/alt change.

---

## 8B. Worked Example — Three Attitudes

Take $\mathbf{v}^{B} = (1, 0, 0)$ — 1 m/s body-forward. For three
attitudes, write $\mathbf{R}_{B\to N}$ and verify
$\mathbf{v}^{N} = \mathbf{R}_{B\to N}\mathbf{v}^{B}$ matches
intuition. Body: $x$-fwd $y$-right $z$-down. NED: N E D.

**Useful trick.** The columns of $\mathbf{R}_{B\to N}$ are the body
unit vectors expressed in NED. Stack columns; verify $\det = +1$.

**Attitude A — Vehicle Pointed Straight Up.** Body-x = world up =
NED $-z$. Pick body-y = NED East ($+y$); then body-z = NED North
($+x$).

$$
\mathbf{R}_{B\to N}^{A} = \begin{pmatrix} 0 & 0 & 1 \\ 0 & 1 & 0 \\ -1 & 0 & 0 \end{pmatrix}
$$

$\det$ (row 1) $= 1\cdot(0 - 1\cdot(-1)) = 1$ ✓. Apply to
$(1, 0, 0)$: $\mathbf{v}^{N} = (0, 0, -1)$ — 1 m/s up ($V_d = -1$). ✓

**Attitude B — Vehicle Pointed Flat North.** Body axes line up with
NED axes. Identity rotation:

$$
\mathbf{R}_{B\to N}^{B} = \begin{pmatrix} 1 & 0 & 0 \\ 0 & 1 & 0 \\ 0 & 0 & 1 \end{pmatrix}
$$

$\det = 1$ ✓. Apply: $\mathbf{v}^{N} = (1, 0, 0)$ — 1 m/s N. ✓

**Attitude C — Vehicle Pointed Flat East.** Body-x = NED East =
$(0, 1, 0)$. Body-y = right of East = South = $(-1, 0, 0)$. Body-z
= NED Down = $(0, 0, 1)$.

$$
\mathbf{R}_{B\to N}^{C} = \begin{pmatrix} 0 & -1 & 0 \\ 1 & 0 & 0 \\ 0 & 0 & 1 \end{pmatrix}
$$

$\det$ (row 3) $= 1\cdot(0 - (-1)\cdot 1) = 1$ ✓. Apply:
$\mathbf{v}^{N} = (0, 1, 0)$ — 1 m/s E. ✓

The same physical body vector $(1, 0, 0)$ gave three different NED
descriptions $(0,0,-1)$, $(1,0,0)$, $(0,1,0)$ — the §1 "frames
matter" point made concrete. Chapter 08 will **generate** these DCMs
from a quaternion without hand-placing each column.

---

## 8C. The NED Tangent Point

NED axes **depend on origin**. North at a Texas launch site points
in a different ECEF direction than North at a Sweden launch site.
For **short-range FT1 flights** (a few km horizontal at most) we
freeze the NED frame at the **pad-arm position** and treat its axes
as constants for the entire flight. The error is well below sensor
noise. For **longer flights** (sounding rockets flying tens of km
downrange, sub-orbital trajectories) the NED tangent point would
have to **move with the vehicle** — re-rooting NED periodically — or
the algorithm would have to formulate in ECEF directly. Out of
scope for FT1, flagged so the assumption is visible.

---

## 9. Key Results

- A **frame** = origin + three orthogonal axes; same physical vector,
  different number-tuples in different frames (§1).
- FT1 uses **ECEF** (Earth-fixed), **NED** (local tangent at launch
  site), **body** ($x$-fwd, $y$-right, $z$-down) (§2).
- **Down is positive in NED.** $\mathbf{g}^{NED} = (0, 0, +9.80665)$
  m/s² (§2).
- $\mathbf{R}_{B}^{N}$ takes a vector from $B$ to $N$. Orthogonal,
  $\det = +1$, $\mathbf{R}^{-1} = \mathbf{R}^T$, composes
  $\mathbf{R}_{C}^{A} = \mathbf{R}_{B}^{A}\mathbf{R}_{C}^{B}$ (§4).
- **Free vector** = rotation only; **position** = rotation +
  translation (§5).
- FT1 state: **geodetic position** + **NED velocity** + **body→NED
  quaternion**; ECEF is not in the state (§7).
- Body→NED = `juno::kmat::QuatRotate(q, v_body)`; quaternion is the
  single source of truth, DCM rebuilt on demand inside kmat (§8).

## 10. Exercises (chapter 12)

- E07-1. Verify $\mathbf{R}_{B}^{N}(\theta)$ in §4.2 is orthogonal
  for arbitrary $\theta$ (use $\sin^2\theta + \cos^2\theta = 1$).
- E07-2. Re-do §4.2 for a 60° pitch-up; verify length preserved.
- E07-3. After a 30° pitch-up, what is the body $z$ axis in NED?
  Compute $\mathbf{R}_{B}^{N}(30°)\,(0, 0, 1)$.
- E07-4. (§8B Attitude D, vehicle pointed flat South.) Build
  $\mathbf{R}_{B\to N}^{D}$ by inspection; verify $\det = +1$ and
  apply to $(1, 0, 0)$. Expected: $(-1, 0, 0)$.
- E07-5. IMU measures $\boldsymbol{\omega}^B = (0, 0.1, 0)$ rad/s.
  Express in NED via $\mathbf{R}_{B}^{N}(30°)$.

## 11. Citations

- Frame definitions, geodesy, WGS-84 parameters and radii of
  curvature, ECEF↔geodetic: Groves [1, §2]. Frame composition (DCMs
  between ECI / ECEF / NED / body): Groves [1, §2.4]. Cross-check
  Farrell [6, §2.4]. WGS-84 parameters cross-checkable against NIMA
  TR8350.2 and USGS public documentation.
- Aerospace body convention:
  [`docs/design/conventions.md`](../../design/conventions.md) §4
  citing `SW-REQ-SYS-057`; [6, §2.2].
- FT1 nav anchor lines —
  [`docs/design/nav/algorithm.md`](../../design/nav/algorithm.md):
  line **64** (WGS-84 reference for `tPosLla`); line **78**
  (`tVelNed (Vn, Ve, Vd)`, m/s NED); line **119** (rotate accel to
  NED via `QuatRotate(q, a_meas_body)`); line **123** (gravity points
  down in NED — FT1 sign convention); line **128** (geodetic
  position update via WGS-84 radii of curvature); lines **222–230**
  (GPS measurement model — direct identity in geodetic + NED-velocity).
- `QuatRotate` contract:
  [`docs/design/kmat/04_interface.md`](../../design/kmat/04_interface.md)
  §4.6.3.

<!-- @{"design": ["SW-REQ-NAV-018"]} -->
