# AABB source certification

> **Independent adversarial review: REJECTED.**

## Authority and scope

This receipt reads the complete pinned owners `src/math/aabb.cpp` and
`include/rive/math/aabb.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The corrected v2 denominator
assigns nine units to the source and 49 to the header. All 58 were reviewed
against the available split Rust owners:

- `crates/nuxie-runtime/src/math/aabb.rs::{TypedAabb,IntegerAabb}`;
- `nuxie_render_api::Aabb` and its real geometry callers;
- `SemanticBounds` and its semantic-provider callers; and
- the live `joystick_factor_from` translation of `AABB::factorFrom`.

The global corrected denominator remains 1,105 owners / 7,818 authority
units. A passing port of the nine upstream AABB test cases is not sufficient
to certify this 58-unit source authority.

## Complete 58-unit inventory

### `src/math/aabb.cpp` (9)

| Pinned authority unit | Count | Rust ownership / disposition |
| --- | ---: | --- |
| `AABB::AABB(Span<Vec2D>)` | 1 | **Rejected as source correspondence.** `HitTester::mesh_bounds` locally reproduces the ordered `std::min`/`std::max` reduction for both pinned callers, but there is no shared AABB owner for the source symbol itself. |
| `graphics_roundf`, `graphics_round`, `AABB::round`, `AABB::roundOut` | 4 | The private `graphics_round` owner is exact for defined finite/in-range conversion, with `graphics_roundf` inlined. `HitTestArea::around` specializes `AABB(...).round()` for the live symmetric-radius hit-test caller. **Still rejected as a complete row group:** there is no general AABB `round`, and `roundOut` is wholly missing. |
| `AABB::expandTo(AABB&, Vec2D)`, `AABB::expandTo(AABB&, float, float)` | 2 | **Missing as reusable owners.** Raw-path and semantic transform code perform local extrema accumulation, but not with one exact ordered implementation; NaN and signed-zero observations differ in current `f32::min`/`max` paths. |
| `AABB::join` | 1 | **Rejected.** `SemanticBounds::expand` is the live candidate, but adds an empty/NaN early return and uses Rust `f32::min`/`max`, neither of which exists in the pinned body. |
| `AABB::contains` | 1 | Exact body in `nuxie_render_api::Aabb::contains`, including inclusive maximum edges and short-circuit comparison order. |

### `include/rive/math/aabb.hpp`: `TAABB<T>` (20)

| Pinned authority unit | Count | Rust ownership / disposition |
| --- | ---: | --- |
| `width`, `height` | 2 | **Partial/rejected.** Missing from `TypedAabb<T>`. Private `HitTestArea` has i32-only `saturating_sub` variants for HitTester; those agree for defined nonoverflowing source inputs but are not the generic `TAABB<T>` surface. |
| `empty` | 1 | Exact for same-type ordered scalars in `TypedAabb::empty`. |
| `makeMaximal`, `makeMaximallyNegative` | 2 | Exact for the six pinned/tested integer types through `AabbScalarBounds`. |
| `inset`, `outset`, `offset` | 3 | **Missing.** |
| `join` | 1 | Exact for the same-type integer owner. |
| templated `intersect` | 1 | **Partial/rejected.** Same-type intersection exists, but the templated cross-type `math::clamp_cast<T>` contract does not. |
| `intersectOrEmpty` | 1 | **Missing**, including its canonical all-zero empty result. |
| `lossless_numeric_cast`, `clamp_cast` | 2 | **Missing.** Rust has no corresponding per-coordinate checked/sign-aware or saturating cross-integer conversion. |
| same-type `operator==`, `operator!=` | 2 | Exact through derived `PartialEq`/`Eq`. |
| cross-type `operator==`, `operator!=` | 2 | **Missing.** Derived equality only compares one `T`; it does not reproduce the pinned signedness-aware `math::cmp_equal`. |
| templated `contains` | 1 | **Missing**, including signedness-aware cross-type comparisons. |
| templated `overlaps` | 1 | **Partial/rejected.** Same-type overlap is exact, but the public pinned body is cross-type and uses the `math::cmp_*` family. |
| `MakeWH` | 1 | **Missing**, including `lossless_numeric_cast<T>` of the two extents. |

### `include/rive/math/aabb.hpp`: `AABB` and preprocessing (29)

| Pinned authority unit | Count | Rust ownership / disposition |
| --- | ---: | --- |
| include guard | 1 | Not applicable: nonbehavioral C++ preprocessing guard. |
| default constructor | 1 | Exact zero initialization is available as `SemanticBounds::default`, but not on `nuxie_render_api::Aabb`; accepted as a split-owner language adaptation. |
| `AABB(Vec2D min, Vec2D max)` | 1 | **Missing** as an AABB constructor. Callers manually unpack coordinates. |
| `fromLTWH` | 1 | Exact in `SemanticBounds::from_xywh`; other call sites also construct the same four values explicitly. |
| four-float constructor | 1 | Exact in `Aabb::new` and `SemanticBounds::new`. |
| `AABB(IAABB)` | 1 | **Missing** as a constructor; ad hoc field casts do not preserve the source-level owner. |
| `operator==`, `operator!=` | 2 | Exact in derived `PartialEq` for both float split owners, including NaN inequality. |
| `left`, `top`, `right`, `bottom` | 4 | Accepted language adaptation to public `min_x`, `min_y`, `max_x`, `max_y` fields. |
| `min`, `max` | 2 | **Missing** as vector-returning AABB operations. |
| `width`, `height` | 2 | Exact in `nuxie_render_api::Aabb`; the same subtraction is in `SemanticBounds::is_empty_or_nan`. |
| `size`, `center` | 2 | **Missing.** In particular there is no owner for the pinned center grouping `(min + max) * 0.5f`. |
| `isEmptyOrNaN` | 1 | Exact in `SemanticBounds::is_empty_or_nan`, including the inverse comparisons that classify every NaN as empty. |
| `pad`, `inset`, `outset`, `offset` | 4 | **Missing**, including the debug nonnegative-size assertions in `inset`. |
| `forExpansion` | 1 | Exact in `SemanticBounds::for_expansion`. |
| `expand` | 1 | **Rejected** against `SemanticBounds::expand`; see the live counterexamples below. |
| `factorFrom` | 1 | Exact live translation in `joystick_factor_from`, including the pinned asymmetric zero-height grouping that yields NaN while zero width yields `0`. |
| `overlaps` | 1 | Exact in `nuxie_render_api::Aabb::overlaps`, including strict abutting-edge rejection and comparison order. |
| `operator[]` | 1 | **Missing**, including the exact `0 -> min`, `1 -> max` mapping. |
| `RIVE_UNREACHABLE` invocation in `operator[]` | 1 | **Missing with `operator[]`**; Rust checked indexing would be an acceptable boundary only after the two valid mappings exist. |

The tables total exactly 58 units: 9 source + 20 `TAABB` + 29 remaining
header units. Split owners retain a useful subset, but local reimplementations
and test-only call edges are not promoted into a complete AABB owner.

## Adversarial findings

### 1. Live semantic union is not the pinned union

Pinned `AABB::expand` unconditionally invokes `join(*this, *this, other)`.
Current `SemanticBounds::expand` instead returns early when `other` is a point,
line, inverted rectangle, or contains NaN. For example, expanding
`AABB::forExpansion()` by `{0, 0, 0, 0}` produces `{0, 0, 0, 0}` in pinned
C++; Rust leaves the maximally negative sentinel unchanged. This is observable
on `SemanticProvider`'s live child-union path, whose pinned fallback can return
point bounds.

Even for positive-area rectangles, Rust's `f32::min`/`max` is not pinned
`std::min`/`std::max`. Pinned `std::min(+0.0, -0.0)` returns its first operand,
`+0.0`; Rust returns `-0.0`. If the accumulated first operand is NaN, pinned
join preserves that NaN because the ordered comparison is false, whereas Rust
selects the numeric operand. The early return masks some second-operand NaNs
but does not repair first-operand ordering. This is a demonstrated translation
failure, not an approved Rust-language adaptation.

### 2. Point expansion is duplicated with different edge semantics

`HitTester::mesh_bounds` correctly uses comparison-shaped helpers for its two
pinned `AABB(Span<Vec2D>)` call sites. That preserves the behavior needed by
those callers, but a call-site-local reduction is not a Rust owner for the
shared source symbol.

The distinct pinned `expandTo` operation is locally respelled by
`RawPath::precise_bounds` and semantic root-transform accumulation using
`f32::min`/`max` or first-corner initialization. For a path containing only a
NaN point, pinned `forExpansion` plus comparison-based `expandTo` ignores the
NaN and leaves the maximally negative sentinel; the Rust optional bounds
become a NaN-bearing rectangle. For successive `+0.0` then `-0.0` points,
pinned comparisons preserve the first zero bits while Rust extrema choose
signed zeros by numeric-library policy. The absence of one literal `expandTo`
owner has therefore produced inconsistent NaN, signed-zero, and empty-sentinel
policies in real callers.

### 3. The integer owner covers only the upstream-test slice

`TypedAabb` has exact same-type bodies for the operations exercised by
`aabb_test.cpp`, but the header authority is intentionally cross-type. Its
`intersect`, equality, containment, overlap, and casts depend on
signedness-aware comparisons and clamping. Those bodies are absent. Searches
find no production consumers of `TypedAabb` outside its export; the current
calls are the direct-port AABB tests. Passing those tests proves their narrow
same-type inputs, not reachability or completeness of the pinned template API.

### 4. General rounding and basic float geometry remain absent

The live symmetric-radius hit-test caller reaches an exact specialized
`HitTestArea::around`, including the `floor(x + 0.5f)` conversions. There is
still no general `round` operation for an existing AABB, and `roundOut`—floor
minimum axes, ceil maximum axes, then SIMD integer conversion—is wholly
absent. Vector min/max access, center, size, inset/outset/offset/pad, and
valid-index access are likewise absent rather than merely renamed.

## Upstream test correspondence

The direct Rust port does retain all nine upstream `TEST_CASE`s and all 79
logical `CHECK`s:

| Upstream case | Checks | Rust evidence | Result |
| --- | ---: | --- | --- |
| `IAABB_join` | 2 | `iaabb_join_direct_port` | Complete |
| `IAABB_intersect` | 2 | `iaabb_intersect_direct_port` | Complete for the tested same-type overload |
| `IAABB_empty` | 6 | `iaabb_empty_direct_port` | Complete |
| `isEmptyOrNaN` | 14 | `is_empty_or_nan_direct_port` | Complete |
| `AABB contains` | 7 | `aabb_contains_direct_port` | Complete |
| `IAABB overlaps` | 18 | `iaabb_overlaps_direct_port` | Complete for the tested same-type overload |
| `AABB overlaps` | 18 | `aabb_overlaps_direct_port` | Complete; factorization through the integer vector preserves every exactly representable literal |
| `TAABB::makeMaximal` | 6 | `taabb_make_maximal_direct_port` | Complete for all six upstream types |
| `TAABB::makeMaximallyNegative` | 6 | `taabb_make_maximally_negative_direct_port` | Complete for all six upstream types |

The correspondence manifest's note misattributes `isEmptyOrNaN` to
`nuxie_render_api::Aabb`; the executing owner is `SemanticBounds`. More
importantly, none of the nine upstream cases covers `expand`/`join` on float
AABB, `expandTo`, either rounding method, the span constructor, the missing
basic geometry methods, or the cross-type template contracts. The 79 green
checks are valid evidence, but they cover only a small behavioral slice of the
58-unit authority.

## Result

No extra expected-red test is needed to establish the rejection: the missing
public methods are compile-time absences, and the live `SemanticBounds::expand`
counterexamples follow directly from its explicit early return and different
standard-library extrema semantics. Existing focused tests should remain green
and demonstrate that the already-ported slice was not regressed.

Verdict: **REJECTED** pending one exact shared float AABB owner (including
ordered extrema helpers and both rounding contracts), completion of the
cross-type `TAABB` surface, migration of required callers away from divergent
local extrema implementations, focused NaN/signed-zero/cast/rounding evidence,
and a fresh independent re-review.
