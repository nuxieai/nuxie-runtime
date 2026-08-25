# AABB source certification

> **Correction candidate complete; two fresh independent adversarial reviews pending.**

## Authority and scope

This receipt covers every authority unit in pinned
`include/rive/math/aabb.hpp` and `src/math/aabb.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`: 49 header units and nine source
units. The direct shared Rust owner is now
`crates/nuxie-render-api/src/aabb.rs`. Runtime reexports it instead of owning a
second implementation, and live raw-path, hit-test, drawing, semantic, and
layout callers use that shared owner.

The previous review at `ff2ccd9b9` rejected the port despite all nine upstream
tests being green. It found missing public operations, divergent float extrema,
and duplicate call-site implementations. This correction addresses those
translation failures rather than preserving the narrow test-shaped port.

## Complete 58-unit inventory

### `include/rive/math/aabb.hpp`: `TAABB<T>` (20)

| Pinned authority | Units | Correction candidate |
| --- | ---: | --- |
| `width`, `height`, `empty` | 3 | Direct field arithmetic and ordered emptiness in `TypedAabb`. Rust wrapping arithmetic only defines the C++ overflow/implementation-defined boundary; defined source inputs are literal. |
| `makeMaximal`, `makeMaximallyNegative` | 2 | Direct six-type scalar-bound constructors. |
| `inset`, `outset`, `offset`, `join` | 4 | Direct field-ordered translations. |
| templated `intersect`, `intersectOrEmpty` | 2 | Cross-signedness clamping precedes the same max/min grouping; empty intersections canonicalize to four zeroes. |
| `lossless_numeric_cast`, `clamp_cast` | 2 | Per-coordinate signedness-aware conversions. The Rust lossless boundary returns `Option` instead of relying on a C++ assertion; successful values are identical. |
| same-type `==`, `!=` | 2 | Derived structural equality covers both operators. |
| cross-type `==`, `!=` | 2 | `equals` compares in a signedness-independent `i128` domain; Rust uses a named method because cross-type `PartialEq` would overlap its same-type implementation. |
| templated `contains`, `overlaps` | 2 | Direct comparison order in the same signedness-independent domain. |
| `MakeWH` | 1 | Lossless extent conversion with canonical zero origin; failed C++ assertion becomes `None`. |

### `include/rive/math/aabb.hpp`: float `AABB` and preprocessing (29)

| Pinned authority | Units | Correction candidate |
| --- | ---: | --- |
| include guard | 1 | Nonbehavioral C++ preprocessing; not applicable in a Rust module. |
| default, vector-pair, `fromLTWH`, four-float, and `IAABB` constructors | 5 | `Default`, `from_min_max`, `from_ltwh`, `new`, and `from_integer`. The integer constructor is deliberately limited to `IntegerAabb`, matching `IAABB`. |
| `==`, `!=` | 2 | Derived float structural equality retains NaN inequality. |
| `left`, `top`, `right`, `bottom` | 4 | Public Rust fields are the approved language-shape adaptation. |
| `min`, `max`, `width`, `height`, `size`, `center` | 6 | Direct translations; vector-valued operations return `Vec2D`, including the pinned center grouping. |
| `isEmptyOrNaN` | 1 | Exact inverse-comparison body. |
| `pad`, `inset`, `outset`, `offset` | 4 | Direct grouping; `debug_assert!` preserves debug-only size assertions. |
| `forExpansion`, `expand`, `factorFrom`, `overlaps` | 4 | Direct sentinel, ordered join, asymmetric factor grouping, and strict-edge overlap. |
| `operator[]` and `RIVE_UNREACHABLE` | 2 | `corner(0/1)` preserves the two valid mappings; invalid input returns `None`, the safe-Rust boundary for unreachable input. |

### `src/math/aabb.cpp` (9)

| Pinned authority | Units | Correction candidate |
| --- | ---: | --- |
| `AABB(Span<Vec2D>)` | 1 | `from_points(&[Vec2D])` uses the first point and source-ordered `std::min`/`std::max` equivalents; empty spans produce zero bounds. |
| `graphics_roundf`, `graphics_round`, `round`, `roundOut` | 4 | Direct `floor(x + .5)`, per-edge round, and floor-min/ceil-max round-out for finite in-range source inputs. Rust float-to-int casts provide a defined safe boundary outside the C++ conversion domain. |
| two `expandTo` overloads | 2 | `expand_to` delegates to the exact comparison-shaped `expand_to_xy`; NaNs are ignored and first signed-zero bits are retained. |
| `join` | 1 | One shared ordered-extrema owner preserves first-operand NaN and signed-zero behavior. |
| `contains` | 1 | Direct inclusive-edge comparison order. |

The table totals remain 20 + 29 + 9 = 58.

## Live-call correction

The shared owner is not test-only:

- `RawPath::bounds` uses the span constructor, and `RawPath::precise_bounds`
  uses `for_expansion` plus `expand_to`.
- `HitTester` uses shared span bounds and shared rounding.
- draw geometry, semantic root transforms, and layout world bounds use shared
  point accumulation.
- `SemanticBounds::expand` delegates to shared ordered `join` without the old
  point/line/NaN early return.

This removes the earlier competing `f32::min`/`max` policies. Focused live
tests prove first-operand NaN, signed zero, NaN point expansion, point/line
semantic unions, cross-type casts, general rounding, and valid corner access.

## Upstream and correction evidence

- All nine upstream `aabb_test.cpp` cases and all 79 translated checks remain
  present in `crates/nuxie-runtime/tests/upstream_aabb.rs`.
- The correction suite adds three focused tests; current result is 12/12.
- The complete `nuxie-render-api` suite passes, including the live RawPath
  extrema test.
- Focused runtime HitTester tests and `cargo check -p nuxie-runtime` are part of
  the correction gate.

## Provisional result

Implementation status: **CORRECTION CANDIDATE COMPLETE**. Certification status:
**PENDING TWO INDEPENDENT ADVERSARIAL REVIEWS**. Neither green tests nor this
implementer-authored receipt may promote the family to accepted parity.
