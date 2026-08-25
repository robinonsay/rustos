# kmat_lib — §4 Interface Definitions

**Parent:** [`index.md`](./index.md)
**Module:** `kmat_lib`
**Namespace:** `juno::kmat`
**Revision:** B

This file holds §4 of the IEEE 1016 design. Sections §1–§3 are in
[`index.md`](./index.md); sections §5–§11 are in
[`05_through_11.md`](./05_through_11.md).

REV B reorients §4 around the kmat-as-kinematics-layer model: primitive
vector and quaternion types and operations are inherited (re-exported)
from `libjuno/include/juno/math/juno_math.hpp` (the LibJuno math module);
kmat itself adds the matrix container `MAT_T<T,R,C>`, matrix algebra
(MatMul, Transpose, Add, Sub, Mult, Invert, MatVecMul), and the
kinematics-specific quaternion operations (`QuatNormalize`, `QuatToMat3`,
`QuatRotate`) that depend on a matrix type.

---

<!-- @{"design": ["SW-REQ-KMAT-001", "SW-REQ-KMAT-002", "SW-REQ-KMAT-003", "SW-REQ-KMAT-004", "SW-REQ-KMAT-005", "SW-REQ-KMAT-006", "SW-REQ-KMAT-007", "SW-REQ-KMAT-013", "SW-REQ-KMAT-015"]} -->
## 4. Interface Definitions

The public surface lives in `libs/kmat_lib/include/kmat_lib/kmat_api.hpp`
within `namespace juno::kmat`. Every kmat-original function is `noexcept`
and templated on element type `T` and dimensions `R`, `C`, `N`. The
header includes `libjuno/include/juno/math/juno_math.hpp` (for
`juno::math::VEC` / `QUAT` and their primitive operations),
`libjuno/include/juno/status.h` (for `JUNO_STATUS_T` and
`JUNO_STATUS_CUSTOM_ERROR`), and `libjuno/include/juno/module.hpp` (for
`juno::RESULT_T<T>`). No other LibJuno headers are required.

### 4.1 Container — `MAT_T<T, R, C>` (kmat-original)

`MAT_T<T, R, C>` is the kmat-original matrix aggregate. Storage member
is `arr[R*C]`, matching the LibJuno math storage convention
(`juno::math::VEC<T, N>::arr[N]`, `juno::math::QUAT<T>::arr[4]`).

```cpp
template <typename T, size_t R, size_t C>
struct MAT_T
{
    static_assert(R > 0, "MAT_T row count must be non-zero");
    static_assert(C > 0, "MAT_T col count must be non-zero");
    static constexpr size_t kRows  = R;
    static constexpr size_t kCols  = C;
    static constexpr size_t kCount = R * C;

    T arr[R * C];   // row-major; element (i, j) at arr[i*C + j]
};

template <typename T> using MAT3_T = MAT_T<T, 3, 3>;
```

- `MAT_T` is a **trivially-constructible aggregate** (no ctors / dtors),
  matching `docs/design/conventions.md` §1.3 and the templated-pattern
  guidance — safe in `.bss` zero-init.
- Storage is **caller-owned `T arr[R*C]`** — no pointer indirection, no
  heap (`SW-REQ-KMAT-001`, `SW-REQ-KMAT-008`).
- Row-major layout chosen (matches C array idiom; row `i`, col `j` lives
  at index `i*C + j`).
- **VEC is no longer aliased to a 1-column `MAT_T`; the two types are
  independent.** Consumers use `juno::math::VEC<T, N>` (re-exported into
  `juno::kmat`; see §4.6.1) for vectors and `juno::kmat::MAT_T<T, R, C>`
  for matrices. The previous `using VEC_T<T, N> = MAT_T<T, N, 1>;` alias
  is **removed in REV B** to align with the LibJuno math module.

### 4.2 Operation contracts — Matrix algebra (kmat-original)

Only matrix operations are documented here. Vector and quaternion
primitive operations are re-exported from `juno::math` and enumerated in
§4.6. `Add(VEC, VEC)`, `Sub(VEC, VEC)`, `Add(QUAT, QUAT)`,
`Sub(QUAT, QUAT)`, `Mult(VEC, T)`, and `Mult(QUAT, T)` are **not**
kmat-original — kmat re-exports them.

#### 4.2.1 `MatMul` — matrix-matrix product (`SW-REQ-KMAT-002`)

| Attribute | Value |
|-----------|-------|
| Signature (RESULT) | `template<typename T, size_t R1, size_t C1, size_t R2, size_t C2> RESULT_T<MAT_T<T,R1,C2>> MatMul(const MAT_T<T,R1,C1> &tA, const MAT_T<T,R2,C2> &tB) noexcept;` |
| Signature (reference output) | `template<typename T, size_t R1, size_t C1, size_t R2, size_t C2> JUNO_STATUS_T MatMul(const MAT_T<T,R1,C1> &tA, const MAT_T<T,R2,C2> &tB, MAT_T<T,R1,C2> &tOut) noexcept;` |
| Compile-time check | `static_assert(C1 == R2, "MatMul: inner dimensions must match")` |
| Preconditions | None at runtime (dimensions enforced at compile time) |
| Postconditions | Returned matrix `tOk.arr[i*C2 + j] = sum_k tA.arr[i*C1 + k] * tB.arr[k*C2 + j]` |
| Error conditions | None — operation always succeeds (`tStatus == JUNO_STATUS_SUCCESS`) |
| Thread safety | Pure function over inputs; trivially thread-safe |

The reference-output form is the primary nav-loop interface (avoids the
`RESULT_T<>` copy when nav holds output storage on its own stack); the
`RESULT_T<>` form exists for ergonomic chaining outside hot loops.

#### 4.2.2 `Transpose` (`SW-REQ-KMAT-003`)

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t R, size_t C> RESULT_T<MAT_T<T,C,R>> Transpose(const MAT_T<T,R,C> &tA) noexcept;` |
| Compile-time check | None beyond `R > 0`, `C > 0` from `MAT_T` |
| Preconditions | None |
| Postconditions | `tOk.arr[j*R + i] = tA.arr[i*C + j]` for all valid `(i, j)` |
| Error conditions | None |
| Thread safety | Pure |

#### 4.2.3 `Add` — MAT_T overload (`SW-REQ-KMAT-004`)

`Add(VEC, VEC)` and `Add(QUAT, QUAT)` come from `juno::math` (re-exported
per §4.6.2). The kmat-original `Add` overload covers `MAT_T` only.

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t R, size_t C> RESULT_T<MAT_T<T,R,C>> Add(const MAT_T<T,R,C> &tA, const MAT_T<T,R,C> &tB) noexcept;` |
| Compile-time check | Operands share `(R, C)` by template deduction |
| Preconditions | None |
| Postconditions | `tOk.arr[k] = tA.arr[k] + tB.arr[k]` for `k in [0, R*C)` |
| Error conditions | None |
| Thread safety | Pure |

#### 4.2.4 `Sub` — MAT_T overload (`SW-REQ-KMAT-013`)

Per the lessons-learned atomicity rule (2026-05-02), Add and Sub are
**separate operations with separate contracts**, not a compound.
`Sub(VEC, VEC)` and `Sub(QUAT, QUAT)` come from `juno::math` (re-exported
per §4.6.2). The kmat-original `Sub` overload covers `MAT_T` only.

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t R, size_t C> RESULT_T<MAT_T<T,R,C>> Sub(const MAT_T<T,R,C> &tA, const MAT_T<T,R,C> &tB) noexcept;` |
| Compile-time check | Operands share `(R, C)` by template deduction |
| Preconditions | None |
| Postconditions | `tOk.arr[k] = tA.arr[k] - tB.arr[k]` for `k in [0, R*C)` |
| Error conditions | None |
| Thread safety | Pure |

#### 4.2.5 `Mult` — MAT_T scalar multiply (`SW-REQ-KMAT-005`)

**REV B aligns the kmat scalar-multiply name with `juno::math::Mult`.**
The function is now `Mult` for both `VEC` (re-exported from `juno::math`,
§4.6.2) and `MAT_T` (kmat-original overload, this contract). REV A's
name `Scale` is removed.

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t R, size_t C> RESULT_T<MAT_T<T,R,C>> Mult(const MAT_T<T,R,C> &tA, T tScalar) noexcept;` |
| Compile-time check | None beyond `R > 0`, `C > 0` from `MAT_T` |
| Preconditions | None |
| Postconditions | `tOk.arr[k] = tA.arr[k] * tScalar` for `k in [0, R*C)` |
| Error conditions | None |
| Thread safety | Pure |

#### 4.2.6 `Invert` (`SW-REQ-KMAT-006`, `SW-REQ-KMAT-007`)

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t N> RESULT_T<MAT_T<T,N,N>> Invert(const MAT_T<T,N,N> &tA) noexcept;` |
| Compile-time check | `static_assert(std::is_floating_point<T>::value, "Invert requires floating-point T")`; square dimension by template deduction |
| Preconditions | `tA` is square (enforced by signature) |
| Postconditions on success | `tOk * tA == Identity<T,N>` within IEEE-754 rounding |
| Error conditions | `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` (FSW extension; see §4.7) returned in `tStatus` when partial-pivot LU detects a pivot-magnitude `< kPivotEpsilon<T>()` — the matrix is non-invertible (or numerically singular). `tOk` contents undefined in that case. |
| Algorithm note | LU decomposition with partial pivoting; pivot threshold from `kPivotEpsilon<T>()` (§4.8). No `throw` (`SW-REQ-KMAT-015`). |
| Thread safety | Pure — scratch storage is function-local |

#### 4.2.7 `MatVecMul` — matrix-vector product (`SW-REQ-KMAT-001`, `SW-REQ-KMAT-002`)

`MatVecMul` is **kmat-original**: `juno::math` has no `MAT` type, so no
matrix-vector product can live there. It is the bridge between the
re-exported `juno::math::VEC<T, N>` and the kmat-original `MAT_T<T, R, C>`.

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T, size_t R, size_t C> juno::math::VEC<T, R> MatVecMul(const MAT_T<T, R, C> &tA, const juno::math::VEC<T, C> &tV) noexcept;` |
| Compile-time check | None beyond `R > 0`, `C > 0` (inner dimensions match by template deduction) |
| Preconditions | None |
| Postconditions | Returned `tOut.arr[i] = sum_k tA.arr[i*C + k] * tV.arr[k]` for `i in [0, R)` |
| Error conditions | None — total over input shapes |
| Thread safety | Pure |

### 4.3 Operator overloads on `MAT_T`

Operator overloads on `juno::math::VEC` and `juno::math::QUAT` already
exist in `juno::math` and are re-exported via `using`-decls into
`juno::kmat` (see §4.6.2). The kmat-original operator overloads cover
**only** `MAT_T<T, R, C>`:

```cpp
template<typename T, size_t R, size_t C>
MAT_T<T,R,C> operator+(const MAT_T<T,R,C> &tA, const MAT_T<T,R,C> &tB) noexcept;

template<typename T, size_t R, size_t C>
MAT_T<T,R,C> operator-(const MAT_T<T,R,C> &tA, const MAT_T<T,R,C> &tB) noexcept;

template<typename T, size_t R1, size_t C1, size_t C2>
MAT_T<T,R1,C2> operator*(const MAT_T<T,R1,C1> &tA, const MAT_T<T,C1,C2> &tB) noexcept;

template<typename T, size_t R, size_t C>
MAT_T<T,R,C> operator*(const MAT_T<T,R,C> &tA, T tScalar) noexcept;
```

Each operator wraps the corresponding named function (`Add`, `Sub`,
`MatMul`, `Mult`). All four are `noexcept` and **infallible** —
operands' shapes are statically checked (matrix-matrix multiply uses
`static_assert(C1 == R2, ...)` inherited from `MatMul`). No
`operator/` is exposed because the underlying op (`Invert`) is fallible
and operators cannot return a status.

### 4.4 Doxygen header block (excerpt)

```cpp
/**
 * @brief Compute the matrix product C = A * B (reference-output form).
 * @tparam T Element type (float or double).
 * @tparam R1, C1 Dimensions of left operand A.
 * @tparam R2, C2 Dimensions of right operand B; static_assert C1==R2.
 * @param tA Left operand (R1 x C1); element (i,k) at arr[i*C1 + k].
 * @param tB Right operand (R2 x C2); element (k,j) at arr[k*C2 + j].
 * @param tOut Output (R1 x C2). Caller-owned; element (i,j) at arr[i*C2 + j].
 * @return JUNO_STATUS_SUCCESS on success.
 */
template <typename T, size_t R1, size_t C1, size_t R2, size_t C2>
JUNO_STATUS_T MatMul(const MAT_T<T,R1,C1> &tA,
                     const MAT_T<T,R2,C2> &tB,
                     MAT_T<T,R1,C2> &tOut) noexcept;
```

### 4.5 Element type policy + LibJuno bridging

`T` is restricted at instantiation sites to `float` or `double`. Choice
of `float` vs `double` is owned by `nav_lib` per `SW-REQ-NAV-015`.
`Invert` carries `static_assert(std::is_floating_point<T>::value, ...)`;
element-wise ops are correct for any arithmetic `T` and omit it.
`QuatNormalize` (§4.6.3) is fallible only on floating-point `T`.

`kmat_api.hpp` includes:

| Header | Purpose |
|--------|---------|
| `libjuno/include/juno/math/juno_math.hpp` | `juno::math::VEC<T, N>`, `QUAT<T>`, primitive ops + operators |
| `libjuno/include/juno/status.h` | `JUNO_STATUS_T`, `JUNO_STATUS_SUCCESS`, `JUNO_STATUS_CUSTOM_ERROR` |
| `libjuno/include/juno/module.hpp` | `juno::RESULT_T<T>` |

No other LibJuno headers are required. `kmat_lib` consumes no C-only
LibJuno facility (no `JUNO_POINTER_T`, no `JUNO_MEMORY_BLOCK`).

<!-- @{"design": ["SW-REQ-KMAT-001", "SW-REQ-KMAT-002", "SW-REQ-KMAT-005", "SW-REQ-KMAT-007", "SW-REQ-KMAT-009", "SW-REQ-KMAT-015"]} -->
### 4.6 Vector and quaternion primitives (re-exported from `juno::math`)

All primitive vector and quaternion types and operations are inherited
from `juno::math` via re-export. This subsection enumerates the
inherited symbols and adds the kinematics-specific operations layered
on top. Re-exports use `using`-declarations inside
`namespace juno::kmat { ... }` so callers may write
`juno::kmat::Add(v0, v1)` interchangeably with
`juno::math::Add(v0, v1)`.

#### 4.6.1 Re-exported types and aliases

| Symbol | Source location | Storage layout | Notes |
|--------|-----------------|----------------|-------|
| `VEC<T, N>` | `libjuno/include/juno/math/juno_math.hpp` | `T arr[N]` | `arr[0]=x`, `arr[1]=y`, `arr[2]=z` for N≥3; `static_assert(N > 0)` |
| `Vec2f64` | same | `VEC<double, 2>` | typedef alias |
| `Vec3f64` | same | `VEC<double, 3>` | typedef alias |
| `Vec2f32` | same | `VEC<float, 2>` | typedef alias |
| `Vec3f32` | same | `VEC<float, 3>` | typedef alias |
| `Vec2i32` | same | `VEC<int32_t, 2>` | typedef alias |
| `Vec3i32` | same | `VEC<int32_t, 3>` | typedef alias |
| `QUAT<T>` | same | `T arr[4]`, **scalar-first** | `arr[0]=w` (scalar `s`), `arr[1]=x` (`i`), `arr[2]=y` (`j`), `arr[3]=z` (`k`); cite `SW-REQ-SYS-041` and `docs/design/conventions.md` §4.6 |
| `Quatf64` | same | `QUAT<double>` | typedef alias |
| `Quatf32` | same | `QUAT<float>` | typedef alias |

The `(w, x, y, z)` interpretation maps onto LibJuno's `[s, i, j, k]`
component naming verbatim — they describe the same memory layout. The
Hamilton convention (`i² = j² = k² = ijk = -1`) and the body→NED
rotation interpretation are inherited from `SW-REQ-SYS-041` /
`conventions.md` §4.6 and apply throughout `juno::kmat`.

Re-export mechanism (in `namespace juno::kmat`):

```cpp
namespace juno
{
namespace kmat
{
    using juno::math::VEC;
    using juno::math::QUAT;
    using juno::math::Vec2f64;
    using juno::math::Vec3f64;
    using juno::math::Vec2f32;
    using juno::math::Vec3f32;
    using juno::math::Vec2i32;
    using juno::math::Vec3i32;
    using juno::math::Quatf64;
    using juno::math::Quatf32;
    /* (operations re-exported in §4.6.2 below) */
}
}
```

#### 4.6.2 Re-exported operations

| Function | Source signature | Re-exported into `juno::kmat` |
|----------|------------------|-------------------------------|
| `Add(VEC<T,N>, VEC<T,N>)` | `VEC<T,N> Add(VEC<T,N> a, const VEC<T,N> &b) noexcept` (general + 2D / 3D / 4D specializations) | `using juno::math::Add;` |
| `Sub(VEC<T,N>, VEC<T,N>)` | `VEC<T,N> Sub(VEC<T,N> a, const VEC<T,N> &b) noexcept` (general + 2D / 3D / 4D specializations) | `using juno::math::Sub;` |
| `Mult(VEC<T,N>, T)` | `VEC<T,N> Mult(VEC<T,N> a, T scalar) noexcept` (general + 2D / 3D / 4D specializations) | `using juno::math::Mult;` |
| `Dot(VEC<T,N>, VEC<T,N>)` | `T Dot(const VEC<T,N> &a, const VEC<T,N> &b) noexcept` (general + 2D / 3D / 4D specializations) | `using juno::math::Dot;` |
| `Cross(VEC<T,2>, VEC<T,2>)` | `T Cross(const VEC<T,2> &a, const VEC<T,2> &b) noexcept` — 2D pseudoscalar `a.x*b.y − a.y*b.x` | `using juno::math::Cross;` |
| `Cross(VEC<T,3>, VEC<T,3>)` | `VEC<T,3> Cross(const VEC<T,3> &a, const VEC<T,3> &b) noexcept` — right-handed 3D vector cross | (same `using`) |
| `L2Norm2(VEC<T,N>)` | `T L2Norm2(const VEC<T,N> &a) noexcept` (general + 2D / 3D / 4D specializations) | `using juno::math::L2Norm2;` |
| `Add(QUAT<T>, QUAT<T>)` | `QUAT<T> Add(QUAT<T> q0, const QUAT<T> &q1) noexcept` | (same `using` as VEC `Add`) |
| `Sub(QUAT<T>, QUAT<T>)` | `QUAT<T> Sub(QUAT<T> q0, const QUAT<T> &q1) noexcept` | (same `using` as VEC `Sub`) |
| `Mult(QUAT<T>, T)` | `QUAT<T> Mult(QUAT<T> q0, T scalar) noexcept` | (same `using` as VEC `Mult`) |
| `HamProd(QUAT<T>, QUAT<T>)` | `QUAT<T> HamProd(const QUAT<T> &q0, const QUAT<T> &q1) noexcept` — Hamilton product `q0 ⊗ q1` | `using juno::math::HamProd;` |
| `Conj(QUAT<T>)` | `QUAT<T> Conj(QUAT<T> q) noexcept` — negates `(i, j, k)`, preserves `s` | `using juno::math::Conj;` |
| `L2Norm2(QUAT<T>)` | `T L2Norm2(const QUAT<T> &q) noexcept` | (same `using` as VEC `L2Norm2`) |
| `Recip(QUAT<T>)` | `QUAT<T> Recip(const QUAT<T> &q) noexcept` — `Conj(q) / L2Norm2(q)`; **caller must ensure non-zero norm; no check is performed** | `using juno::math::Recip;` |

Operator overloads from `juno::math` are also visible in `juno::kmat`
through the `using namespace juno::math;` already implied for ADL of the
re-exported types: `operator+`, `operator-`, and `operator*` (scalar on
either side) on `VEC<T, N>` and `QUAT<T>`; `operator*` on two `QUAT<T>`
(Hamilton product, equivalent to `HamProd`).

#### 4.6.3 kmat-original kinematics operations (NEW)

These are the kinematics-specific operations layered by kmat on top of
the re-exported primitives. Each operates on `juno::math::VEC<T, 3>`
and / or `juno::math::QUAT<T>` and / or `juno::kmat::MAT_T<T, R, C>`.

##### `QuatNormalize` (`SW-REQ-KMAT-007`, `SW-REQ-KMAT-015`)

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T> RESULT_T<juno::math::QUAT<T>> QuatNormalize(const juno::math::QUAT<T> &tQ) noexcept;` |
| Compile-time check | `static_assert(std::is_floating_point<T>::value, "QuatNormalize requires floating-point T")` |
| Preconditions | None |
| Postconditions on success | `tOk = tQ / sqrt(juno::math::L2Norm2(tQ))`; `juno::math::L2Norm2(tOk) ≈ 1` within IEEE-754 rounding |
| Error conditions | `juno::kmat::JUNO_FSW_STATUS_NUMERIC_ERROR` (§4.7) when `juno::math::L2Norm2(tQ) < kPivotEpsilon<T>() * kPivotEpsilon<T>()` — i.e., the squared magnitude falls below the squared pivot threshold (equivalently: magnitude `< kPivotEpsilon<T>()`). The squared-magnitude check is preferred to avoid an extra `sqrt` on the failure branch. `tOk` contents undefined in that case. |
| Thread safety | Pure |

##### `QuatToMat3` (`SW-REQ-KMAT-001`, `SW-REQ-SYS-041`)

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T> MAT_T<T, 3, 3> QuatToMat3(const juno::math::QUAT<T> &tQ) noexcept;` |
| Compile-time check | None at this op (callers typically restrict `T` to floating-point) |
| Preconditions | `tQ` should be unit-norm for a pure rotation; non-unit input yields a scaled rotation matrix |
| Postconditions | Returned 3×3 body→NED rotation matrix per Hamilton convention with `(w, x, y, z) = (arr[0], arr[1], arr[2], arr[3])` |
| Error conditions | None |
| Thread safety | Pure |

Matrix layout (row-major, `(w, x, y, z) = (arr[0], arr[1], arr[2], arr[3])`):

```
[ 1 - 2*(y*y + z*z),   2*(x*y - w*z),       2*(x*z + w*y)      ]
[ 2*(x*y + w*z),       1 - 2*(x*x + z*z),   2*(y*z - w*x)      ]
[ 2*(x*z - w*y),       2*(y*z + w*x),       1 - 2*(x*x + y*y)  ]
```

##### `QuatRotate` (`SW-REQ-KMAT-001`, `SW-REQ-SYS-041`)

| Attribute | Value |
|-----------|-------|
| Signature | `template<typename T> juno::math::VEC<T, 3> QuatRotate(const juno::math::QUAT<T> &tQ, const juno::math::VEC<T, 3> &tV) noexcept;` |
| Compile-time check | None |
| Preconditions | `tQ` should be unit-norm to produce a length-preserving rotation |
| Postconditions | Returns the body-frame vector `tV` rotated into NED; canonical implementation is `MatVecMul(QuatToMat3(tQ), tV)` (§4.2.7) — single code path, single set of rounding errors |
| Error conditions | None |
| Thread safety | Pure |

The implementation deliberately uses the matrix-vector path rather than
the alternative `q ⊗ v ⊗ q*` quaternion sandwich to keep numerical
behavior consistent with the explicit rotation matrix exposed by
`QuatToMat3` (a kinematics-layer cohesion principle: one rotation,
one set of floating-point operations).

### 4.7 FSW-extension status code (`JUNO_FSW_STATUS_NUMERIC_ERROR`)

`juno/status.h` does not define a numeric-error / singular-matrix code,
so `kmat_lib` declares one as the canonical FSW extension per
`docs/design/conventions.md` §4.8. The offset is `+1` (first FSW
extension); the symbol is consumed by `Invert` (§4.2.6) and
`QuatNormalize` (§4.6.3) and is the only fallible-mode code returned by
this library. The declaration lives in the public API header alongside
the operation prototypes:

```cpp
namespace juno
{
namespace kmat
{
    // Per docs/design/conventions.md §4.8:
    // Offset +1 from JUNO_STATUS_CUSTOM_ERROR.
    // Returned by Invert() on a zero/underflow-bounded pivot and by
    // QuatNormalize() on a near-zero magnitude quaternion.
    static constexpr JUNO_STATUS_T JUNO_FSW_STATUS_NUMERIC_ERROR =
        JUNO_STATUS_CUSTOM_ERROR + 1;
}
}
```

The constant is `static constexpr`, has internal linkage, and is
read-only after translation; it carries no global mutable state
(`conventions.md` §5).

### 4.8 `kPivotEpsilon<T>()` function template

C++11 lacks variable templates (a C++14 feature); the **function-template
form** is the canonical C++11-compatible expression of a per-type
compile-time constant. It is consumed by `Invert` (§4.2.6) and
`QuatNormalize` (§4.6.3) and lives in `juno::kmat` alongside them:

```cpp
namespace juno
{
namespace kmat
{
    template <typename T> static constexpr T kPivotEpsilon() noexcept;
    template <> constexpr float  kPivotEpsilon<float>()  noexcept { return 1e-12f; }
    template <> constexpr double kPivotEpsilon<double>() noexcept { return 1e-30; }
}
}
```

Rationale: the function-template form preserves the per-type
compile-time-constant semantics under `-std=c++11 -pedantic`. Numeric
values are tunable in `nav_lib`'s L2 design once the FT1 covariance
scaling is finalised.

---

Continue to [`05_through_11.md`](./05_through_11.md) for §5 State
Machines through §11 Traceability.
