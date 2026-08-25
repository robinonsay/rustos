---
document_type: nav_kalman tutorial — Chapter 01
program: Juno FT1 FSW
revision: A
effective_date: 2026-05-08
audience: software engineer rusty on linear algebra; no controls/navigation background assumed
prerequisites: high-school algebra, basic single-variable calculus
chapter: 01 of 12 (Linear Algebra Primer)
---

# Chapter 01 — Linear Algebra Primer

## 0. Purpose and Notation

Self-contained linear-algebra prerequisites for chapters 02–11. Assumes
only high-school algebra and basic calculus. By the end you can (1) read
vector and matrix expressions, (2) multiply matrices and vectors by hand
at 2×2/2×3 scale, (3) recognize symmetric positive-definite matrices and
explain why the Kalman filter requires that property, and (4) map every
math symbol to the FSW types `juno::kmat::MAT_T` and `juno::kmat::VEC_T`.

**Notation used throughout the tutorial.** Scalars are italic lowercase
($x$, $\lambda$, $dt$). Vectors are bold lowercase column vectors
($\mathbf{x}$, $\mathbf{v}$). Matrices are bold uppercase ($\mathbf{A}$,
$\mathbf{P}$, $\mathbf{H}$). Identity is $\mathbf{I}$, zero is
$\mathbf{0}$, transpose is $\mathbf{A}^T$, inverse is $\mathbf{A}^{-1}$.
Math format: LaTeX `$...$` inline and `$$...$$` display.

---

## 1. Vectors

### 1.1 What a vector is

A **vector** is an ordered list of numbers, written as a column:

$$
\mathbf{v} = \begin{bmatrix} 3 \\ 4 \end{bmatrix}, \qquad
\mathbf{w} = \begin{bmatrix} 1 \\ -2 \\ 5 \end{bmatrix}
$$

$\mathbf{v}$ is 2-dimensional, $\mathbf{w}$ is 3-dimensional. Geometrically,
each is an arrow from the origin to the listed point — it has a
**direction** and a **magnitude**. Every vector in this tutorial is a
column; a **row vector** is the transpose,
$\mathbf{v}^T = \begin{bmatrix} 3 & 4 \end{bmatrix}$.

### 1.2 Addition, scalar multiplication, dot product, norm

Addition is componentwise (both vectors same dimension); scalar
multiplication multiplies every component:

$$
\begin{bmatrix} 3 \\ 4 \end{bmatrix} + \begin{bmatrix} 1 \\ 2 \end{bmatrix}
= \begin{bmatrix} 4 \\ 6 \end{bmatrix},
\qquad
2 \cdot \begin{bmatrix} 3 \\ 4 \end{bmatrix} = \begin{bmatrix} 6 \\ 8 \end{bmatrix}
$$

Geometrically, addition is tip-to-tail; scalar multiplication stretches
(or flips, if negative) the arrow.

### 1.3 Dot product and L2 norm

The **dot product** of two vectors of the same dimension is a single
number, $\mathbf{a}\cdot\mathbf{b} = \sum_i a_i b_i$. Worked example with
$\mathbf{a} = (3,4)^T$, $\mathbf{b} = (1,2)^T$:

$$
\mathbf{a}\cdot\mathbf{b} = (3)(1) + (4)(2) = 11
$$

Geometric meaning: $\mathbf{a}\cdot\mathbf{b} = \|\mathbf{a}\|\,\|\mathbf{b}\|\cos\theta$,
where $\theta$ is the angle between the two arrows. Positive dot product
means the vectors mostly agree in direction; zero means perpendicular;
negative means they mostly oppose. We will often write the dot product
as the matrix product $\mathbf{a}^T \mathbf{b}$.

The **L2 norm** (length) is

$$
\|\mathbf{v}\| = \sqrt{\mathbf{v}^T \mathbf{v}} = \sqrt{\sum_i v_i^2}
$$

For $\mathbf{v} = (3,4)^T$: $\|\mathbf{v}\| = \sqrt{9 + 16} = 5$
(Pythagorean theorem). A **unit vector** has norm 1.

```python
import numpy as np
a = np.array([3.0, 4.0]); b = np.array([1.0, 2.0])
print(a @ b, np.linalg.norm(a))   # 11.0 5.0
```

---

## 2. Matrices

### 2.1 Definition and entry notation

A **matrix** is a rectangular grid of numbers; shape $m \times n$ means
$m$ rows and $n$ columns. The entry in row $i$, column $j$ is $A_{ij}$.

$$
\mathbf{A} = \begin{bmatrix} 1 & 2 & 3 \\ 4 & 5 & 6 \end{bmatrix}
\quad (2 \times 3),\quad A_{12} = 2,\ A_{23} = 6
$$

A column vector of dimension $n$ is just an $n \times 1$ matrix.

### 2.2 Transpose

The **transpose** $\mathbf{A}^T$ swaps rows and columns; if $\mathbf{A}$
is $m \times n$ then $\mathbf{A}^T$ is $n \times m$:

$$
\mathbf{A} = \begin{bmatrix} 1 & 2 & 3 \\ 4 & 5 & 6 \end{bmatrix}
\Longrightarrow
\mathbf{A}^T = \begin{bmatrix} 1 & 4 \\ 2 & 5 \\ 3 & 6 \end{bmatrix}
$$

Two facts you will use repeatedly: $(\mathbf{A}^T)^T = \mathbf{A}$ and
$(\mathbf{A}\mathbf{B})^T = \mathbf{B}^T \mathbf{A}^T$ (order flips).

### 2.3 Special matrices

- **Identity** $\mathbf{I}$: square, 1s on diagonal, 0s off.
  $\mathbf{I}_3 = \begin{bmatrix} 1 & 0 & 0 \\ 0 & 1 & 0 \\ 0 & 0 & 1 \end{bmatrix}$.
- **Zero** $\mathbf{0}$: every entry 0; any shape.
- **Diagonal**: square, off-diagonal entries 0;
  $\mathbf{D} = \mathrm{diag}(2, 5, 7)$.
- **Symmetric**: square with $\mathbf{A} = \mathbf{A}^T$, equivalently
  $A_{ij} = A_{ji}$. Example:
  $\mathbf{S} = \begin{bmatrix} 2 & 1 & 0 \\ 1 & 3 & 4 \\ 0 & 4 & 5 \end{bmatrix}$.
  Covariance matrices in the Kalman filter are always symmetric — see §9.

---

## 3. Matrix-Vector Multiplication (linear transformation)

A matrix times a vector produces a vector. If $\mathbf{A}$ is $m \times n$
and $\mathbf{x}$ is $n \times 1$, then $\mathbf{A}\mathbf{x}$ is
$m \times 1$, with $(\mathbf{A}\mathbf{x})_i = \sum_j A_{ij} x_j$ — the
$i$-th output is the dot product of the $i$-th row of $\mathbf{A}$ with
$\mathbf{x}$.

### 3.1 Worked example — 2D rotation

The matrix that rotates a 2D vector counter-clockwise by angle $\theta$:

$$
\mathbf{R}(\theta) = \begin{bmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{bmatrix}
$$

For $\theta = 90°$: $\cos = 0$, $\sin = 1$. Apply to $\mathbf{x} = (1, 0)^T$:

$$
\mathbf{R}(90°)\mathbf{x} = \begin{bmatrix} 0 & -1 \\ 1 & 0 \end{bmatrix}
\begin{bmatrix} 1 \\ 0 \end{bmatrix}
= \begin{bmatrix} (0)(1)+(-1)(0) \\ (1)(1)+(0)(0) \end{bmatrix}
= \begin{bmatrix} 0 \\ 1 \end{bmatrix}
$$

The arrow that pointed along the x-axis now points along the y-axis —
exactly a 90° counter-clockwise rotation. **A matrix represents a
linear transformation; multiplying a vector by it applies that
transformation.** In navigation you will see the same idea in 3D, taking
a body-frame vector to the NED frame.

```python
import numpy as np
R = np.array([[0.0, -1.0], [1.0, 0.0]])
print(R @ np.array([1.0, 0.0]))   # [0. 1.]
```

---

## 4. Matrix-Matrix Multiplication

If $\mathbf{A}$ is $m \times k$ and $\mathbf{B}$ is $k \times n$, then
$\mathbf{C} = \mathbf{A}\mathbf{B}$ is $m \times n$, with
$C_{ij} = \sum_\ell A_{i\ell} B_{\ell j}$. In words: $C_{ij}$ is the dot
product of row $i$ of $\mathbf{A}$ with column $j$ of $\mathbf{B}$. The
inner dimension $k$ must match.

### 4.1 Worked 2×2 × 2×2

$$
\mathbf{A} = \begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix},\quad
\mathbf{B} = \begin{bmatrix} 5 & 6 \\ 7 & 8 \end{bmatrix}
$$

- $C_{11} = (1)(5)+(2)(7) = 19$
- $C_{12} = (1)(6)+(2)(8) = 22$
- $C_{21} = (3)(5)+(4)(7) = 43$
- $C_{22} = (3)(6)+(4)(8) = 50$

$$
\mathbf{C} = \mathbf{A}\mathbf{B} = \begin{bmatrix} 19 & 22 \\ 43 & 50 \end{bmatrix}
$$

### 4.2 Non-commutativity

In general $\mathbf{A}\mathbf{B} \neq \mathbf{B}\mathbf{A}$. For the
matrices above,
$\mathbf{B}\mathbf{A} = \begin{bmatrix} 23 & 34 \\ 31 & 46 \end{bmatrix} \neq \mathbf{C}$.
**Order matters.** When you read $\mathbf{H}\mathbf{P}\mathbf{H}^T$ in
the Kalman update, you cannot reshuffle the factors.

---

## 5. Identity Matrix

For any matrix $\mathbf{A}$ of compatible shape,
$\mathbf{I}\mathbf{A} = \mathbf{A}\mathbf{I} = \mathbf{A}$. Worked
example: $\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}
\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}
= \begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}$. You will see expressions
like $(\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}$ in the Kalman
covariance update; $\mathbf{I}$ there is the identity of the same size
as the product $\mathbf{K}\mathbf{H}$.

---

## 6. Matrix Inverse

### 6.1 Definition and existence

The **inverse** of a square matrix $\mathbf{A}$ is the matrix
$\mathbf{A}^{-1}$ with $\mathbf{A}\mathbf{A}^{-1} = \mathbf{A}^{-1}\mathbf{A} = \mathbf{I}$.
Why it matters: the linear system $\mathbf{A}\mathbf{x} = \mathbf{b}$ has
the unique solution $\mathbf{x} = \mathbf{A}^{-1}\mathbf{b}$ when
$\mathbf{A}^{-1}$ exists. The Kalman gain involves
$\mathbf{S}^{-1}$ where $\mathbf{S} = \mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R}$ —
the central numerical step of every measurement update.

The inverse exists iff $\mathbf{A}$ is (1) square and (2) **non-singular**
($\det \mathbf{A} \neq 0$, equivalently no row is a linear combination of
the others).

### 6.2 Worked 2×2 inverse — cofactor formula

For $\mathbf{A} = \begin{bmatrix} a & b \\ c & d \end{bmatrix}$,
$\det\mathbf{A} = ad - bc$ and

$$
\mathbf{A}^{-1} = \frac{1}{ad-bc}\begin{bmatrix} d & -b \\ -c & a \end{bmatrix}
$$

Take $\mathbf{A} = \begin{bmatrix} 2 & 1 \\ 1 & 3 \end{bmatrix}$:
$\det\mathbf{A} = 6 - 1 = 5 \neq 0$, so

$$
\mathbf{A}^{-1} = \tfrac{1}{5}\begin{bmatrix} 3 & -1 \\ -1 & 2 \end{bmatrix}
= \begin{bmatrix} 0.6 & -0.2 \\ -0.2 & 0.4 \end{bmatrix}
$$

Verify: $\mathbf{A}\mathbf{A}^{-1} =
\begin{bmatrix} (2)(0.6)+(1)(-0.2) & (2)(-0.2)+(1)(0.4) \\
(1)(0.6)+(3)(-0.2) & (1)(-0.2)+(3)(0.4) \end{bmatrix}
= \begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix} = \mathbf{I}$.

```python
import numpy as np
print(np.linalg.inv(np.array([[2.0, 1.0], [1.0, 3.0]])))
# [[ 0.6 -0.2] [-0.2  0.4]]
```

### 6.3 Inverses in software — don't compute them by hand

For $n > 3$, the cofactor formula costs $O(n!)$. Production code uses
**LU decomposition with partial pivoting**: $O(n^3)$ and numerically
stable. In FT1 FSW that's `juno::kmat::Invert` in
`libjuno/include/juno/math/juno_math.hpp`; the contract is in
`docs/design/kmat/04_interface.md` §4.2.6. The EKF GPS update calls
`juno::kmat::Invert<double, 6>` for the 6×6 innovation covariance — see
`docs/design/nav/algorithm.md` §4.2 and §6 ("Pivot guarding"). Write the
math in this notation; kmat computes it.

---

## 7. Determinant (briefly)

The **determinant** $\det(\mathbf{A})$ is a single number summarizing a
square matrix. Geometrically, $|\det(\mathbf{A})|$ is the factor by
which the linear transformation $\mathbf{A}$ scales volume (area in 2D,
volume in 3D). For 2×2: $\det \begin{bmatrix} a & b \\ c & d \end{bmatrix} = ad-bc$.

Crucial fact: $\mathbf{A}$ is non-singular (invertible) iff $\det(\mathbf{A}) \neq 0$.
A matrix with determinant 0 collapses some direction to zero; the
transformation cannot be undone, so $\mathbf{A}^{-1}$ does not exist.

---

## 8. Eigenvalues and Eigenvectors (briefly)

For a square matrix $\mathbf{A}$, a non-zero vector $\mathbf{v}$ is an
**eigenvector** with **eigenvalue** $\lambda$ if

$$
\mathbf{A}\mathbf{v} = \lambda \mathbf{v}
$$

Geometrically, an eigenvector is a direction that $\mathbf{A}$ does not
rotate — only stretches (or flips) by $\lambda$. Why we care: every
symmetric real matrix has $n$ real eigenvalues and a full set of
mutually-perpendicular eigenvectors. We use this fact in §9 to define
positive-definiteness; we will not compute eigenvalues by hand.

---

## 9. Symmetric and Positive-Definite Matrices (the KF money property)

### 9.1 Symmetric and positive-definite

$\mathbf{A}$ is **symmetric** iff $\mathbf{A} = \mathbf{A}^T$.
A symmetric matrix is **positive-definite** (PD) if **all its
eigenvalues are strictly greater than zero**. Equivalent characterization:

$$
\mathbf{x}^T \mathbf{A} \mathbf{x} > 0 \quad \text{for every non-zero } \mathbf{x}
$$

If we relax to $\geq$, we get **positive-semidefinite** (PSD).

### 9.2 Why this matters for the Kalman filter

The Kalman filter's state-uncertainty representation is the **covariance
matrix** $\mathbf{P}$. By construction (chapter 02 derives this from
probability), $\mathbf{P}$ is always **symmetric positive-definite**:
diagonal entries are variances (positive); off-diagonals are covariances
(bounded by Cauchy-Schwarz). Brown & Hwang [2] §4.2 and Groves [1] §3.2.4
both lead with this property because **every step of the filter assumes
it**. If $\mathbf{P}$ ever drifts to non-SPD — e.g., grows a tiny
negative eigenvalue from round-off — the next inverse
$(\mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R})^{-1}$ can blow up and
the filter diverges.

### 9.3 Worked example — round-off can break PD

Start with a barely-PD matrix:

$$
\mathbf{P} = \begin{bmatrix} 1 & 0.99 \\ 0.99 & 1 \end{bmatrix}
$$

$\det\mathbf{P} = 1 - 0.9801 = 0.0199 > 0$. Eigenvalues are
$1 \pm 0.99 = \{1.99, 0.01\}$ — both positive, so PD, but barely.

Suppose a Kalman update *should* produce the same $\mathbf{P}$ but, due
to round-off in the floating-point operations, lands at:

$$
\mathbf{P}' = \begin{bmatrix} 1 & 0.99 \\ 0.99000001 & 1 \end{bmatrix}
$$

Two things broke: (1) $\mathbf{P}'$ is no longer symmetric — off-diagonals
disagree; (2) even after symmetrizing, the smallest eigenvalue can fall
slightly negative, breaking PD. Standard fix, used in `nav_lib`: after
every covariance update, force symmetry by averaging with the transpose:

$$
\mathbf{P}_{\text{sym}} = \tfrac{1}{2}\bigl(\mathbf{P}' + (\mathbf{P}')^T\bigr)
$$

This is the "symmetry enforcement" bullet in
`docs/design/nav/algorithm.md` §6. Cost: one transpose, one add, one
scale. Brown & Hwang [2] §5.7 ("Stability") and Groves [1] §3.2.4 both
prescribe it.

```python
import numpy as np
Pp = np.array([[1.0, 0.99], [0.99000001, 1.0]])
Psym = 0.5 * (Pp + Pp.T)
print(np.allclose(Psym, Psym.T), np.linalg.eigvalsh(Psym))
# True [0.01 1.99]
```

---

## 10. Block Matrices (briefly)

A **block matrix** has matrix-valued entries. With $\mathbf{A}$ being
$2\times2$, $\mathbf{B}$ being $2\times3$, $\mathbf{C}$ being $1\times2$,
$\mathbf{D}$ being $1\times3$:

$$
\mathbf{M} = \begin{bmatrix} \mathbf{A} & \mathbf{B} \\ \mathbf{C} & \mathbf{D} \end{bmatrix}
$$

is a $3 \times 5$ matrix in block form. Block sizes must be compatible
(rows in the same block-row share height; columns in the same
block-column share width).

A **block-diagonal** matrix has only diagonal blocks:

$$
\mathbf{R}_{\text{gps}} = \begin{bmatrix} \mathbf{R}_{\text{pos}} & \mathbf{0} \\ \mathbf{0} & \mathbf{R}_{\text{vel}} \end{bmatrix}
$$

Chapter 11 will use exactly this for the GPS 6-dimensional measurement
model (lat, lon, alt, Vn, Ve, Vd) — see `docs/design/nav/algorithm.md`
§4.2: "$\mathbf{R}_{\text{gps}}$ is 6 × 6 diagonal …". Block-diagonal
$\mathbf{R}$ encodes the assumption that horizontal-position,
vertical-position, and velocity noise are uncorrelated — the standard
simplification for a GPS receiver.

---

## 11. How this maps to FT1's `nav_lib`

Every vector and matrix you've learned about has a direct C++ type. Per
`docs/design/kmat/04_interface.md` §4.1: a length-$n$ vector $\mathbf{v}$
is a `juno::kmat::VEC_T<T, N>`; an $m \times n$ matrix $\mathbf{A}$ is a
`juno::kmat::MAT_T<T, M, N>`.

The 16-state EKF in `docs/design/nav/algorithm.md` lines 175–180 declares
its covariance as `juno::kmat::MAT_T<double, 16, 16>` and its state as
`juno::kmat::VEC_T<double, 16>`, with $\mathbf{F}$, $\mathbf{P}$, and
$\mathbf{Q}$ all square of side `kInternalDim` (16 for full-state EKF, 15
for the error-state variant). Quaternion math (`QuatMul`, `QuatRotate`,
`QuatNormalize`) and matrix inversion (`Invert`) are in the same kmat
header. When you read $\mathbf{A}\mathbf{B}$ in this tutorial, it
corresponds to `juno::kmat::MatMul(...)`; transpose maps to
`juno::kmat::Transpose`; inverse maps to `juno::kmat::Invert`. The math
notation is the contract; kmat is the implementation.

---

## References

[1] P. D. Groves, *Principles of GNSS, Inertial, and Multisensor
Integrated Navigation Systems*, 2nd ed., Artech House, 2013 — §3.2.4
covers symmetric-positive-definite covariance properties.
[2] R. G. Brown and P. Y. C. Hwang, *Introduction to Random Signals and
Applied Kalman Filtering with MATLAB Exercises*, 4th ed., Wiley, 2012 —
§4.2 introduces the covariance matrix; §5.7 describes the
symmetry-enforcement trick used in §9.
[3] R. E. Kalman, "A new approach to linear filtering and prediction
problems," *J. Basic Engineering*, 82(1):35–45, 1960.
[4] N. Trawny and S. I. Roumeliotis, "Indirect Kalman filter for 3D
attitude estimation," Univ. of Minnesota MARS Lab TR-2005-002, 2005.
[5] J. Solà, "Quaternion kinematics for the error-state Kalman filter,"
arXiv:1711.02508, 2017.
[6] J. A. Farrell, *Aided Navigation: GPS with High Rate Sensors*,
McGraw-Hill, 2008.
[7] Y. Bar-Shalom, X. R. Li, and T. Kirubarajan, *Estimation with
Applications to Tracking and Navigation*, Wiley, 2001.
