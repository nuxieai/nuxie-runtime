# AABB source certification

> **Rejected findings corrected. Certification is PENDING two fresh,
> independent accepted adversarial reviews from the corrected commit.**

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

## First independent correction review

Verdict: **REJECTED** at correction commit `64c8f852c`.

This review re-read all 49 pinned header units and all nine pinned source units,
including the `math_types.hpp` comparison/cast helpers on which templated
`TAABB` depends. The direct bodies in `crates/nuxie-render-api/src/aabb.rs` are
substantially faithful: the cross-signed `i128` comparison domain covers all
six types instantiated by the pinned tests; `std::min`/`std::max` first-operand
NaN and signed-zero behavior is preserved; `factorFrom` retains the pinned
asymmetric Y grouping; and the `Option` results for failed lossless casts and
invalid corners are reasonable safe-Rust boundaries for source assertions or
`RIVE_UNREACHABLE`. Finite in-range `round` and `roundOut` are also literal;
Rust's defined saturating float-to-integer behavior remains only an adaptation
outside the C++ conversion domain.

The candidate nevertheless fails actual-call-path and single-owner review:

1. `SemanticProvider::rootTransformAABB` is translated with the wrong AABB
   operation. Pinned `src/semantic/semantic_provider.cpp:35-42` constructs
   `AABB::forExpansion()` and calls `AABB::expandTo` four times, in corner
   order. Rust `semantic_provider.rs:185-194` constructs a four-point slice and
   calls `FloatAabb::from_points`, which is the distinct
   `AABB(Span<Vec2D>)` algorithm. The two algorithms differ on real source
   boundary values: `expandTo` ignores a NaN coordinate until a comparable
   coordinate arrives, while the span constructor preserves a first-point NaN
   through source-ordered `std::min`/`std::max`; positive infinity also leaves
   the `forExpansion` minimum at `f32::MAX` instead of initializing both edges
   to infinity. Calling the new shared owner is insufficient when it calls the
   wrong pinned unit.

2. The live inner-feather path still bypasses the shared owner. Pinned
   `src/shapes/paint/feather.cpp:73-88` evaluates
   `path->rawPath()->bounds().pad(strength() * 1.5f)`, reaching the span
   constructor and `AABB::pad/outset/inset`. Rust `draw.rs:21823-21842` instead
   reaches `PathBounds` in `draw.rs:23435-23480`, whose `include` uses
   `f32::min`/`f32::max` and whose `pad` manually edits four fields. Those
   extrema choose different NaN and signed-zero results from the pinned
   first-operand `std::min`/`std::max` contract. This is a production path, not
   dead support code, and directly contradicts the receipt's claim that the
   competing extrema policies were removed.

3. The 20 `TAABB` units are not the workspace's operative integer-bounds
   owner. Production renderer code still declares separate `IAABB` and
   `AABBu16` structs in at least
   `mechanical_port/source/renderer/include/rive/renderer/gpu_hpp.rs`,
   `render_context_hpp.rs`, and `render_target_hpp.rs`, then executes separate
   `intersect_*`, `join_*`, `clamp_bounds_u16`, `boundsToU16`, and `makeWH`
   implementations across the shared, WebGL2, Vulkan, and other renderer
   paths. The new module exports no `AABBi16` or `AABBu16` aliases and its
   templated intersection/cast/join surface is reached only by the focused
   runtime tests. This violates the pinned single `rive/math/aabb.hpp` source
   owner and leaves the supposedly corrected integer authority test-only for
   the consumers that exercise it most heavily.

Evidence remains green but does not discharge these blockers:

- `cargo test -p nuxie-render-api
  raw_path_bounds_use_the_shared_pinned_aabb_extrema_contracts`: 1/1 passed.
- `cargo test -p nuxie-runtime --test upstream_aabb`: 12/12 passed.

Required correction: preserve the exact source operation at each caller
(`forExpansion` plus ordered `expandTo` for semantic root transforms, span
bounds plus `pad` for Feather), and make the shared typed AABB the operative
renderer dependency instead of retaining local type/algorithm substitutes.
Then repeat both independent reviews from the corrected commit.

## Rejected-finding correction

The rejected actual-call-path findings are now corrected rather than waived:

- `SemanticProvider::rootTransformAABB` reaches `for_expansion` and four
  source-ordered `expand_to` calls instead of the span constructor.
- inner Feather reaches the shared `RawPath::bounds().pad(...)` authority
  instead of `PathBounds` extrema and manual padding.
- `IAABB`, `AABBi16`, and `AABBu16` are aliases of the shared `TypedAabb`
  owner throughout the renderer. The duplicate header structs and Vulkan
  extension trait are gone.
- shared renderer, RiveRenderer, WebGL2, WebGPU, Vulkan, and Metal call the
  shared `intersect`, `intersectOrEmpty`, `join`, `clamp_cast`,
  `lossless_numeric_cast`, `MakeWH`, maximal-sentinel, `round_out`, and
  `is_empty_or_nan` operations. This also fixes the WebGL2 substitute that
  returned a nonzero-origin degenerate rectangle where pinned
  `intersectOrEmpty` returns four zeroes.

The remaining rectangle-shaped helpers are separate pinned caller authority,
not duplicate AABB geometry. `map_bounds` and `map_path_bounds` translate
`Mat2D::mapBoundingBox`; backend scissor helpers only convert a source AABB to
the platform API's coordinate/extent representation. They do not choose AABB
extrema, intersections, unions, normalization, or cast policy.

Correction gates:

- `cargo test -p nuxie-render-api`: 40/40 package tests passed (27 unit, three
  canonical-recording, three side-channel, seven upstream RawPath).
- `cargo test -p nuxie-runtime --test upstream_aabb`: 12/12 passed.
- individual `cargo check -p nuxie-renderer --features renderer-vulkan`,
  `renderer-webgpu`, `renderer-webgl2`, and `renderer-metal`: passed. The
  combined four-feature check is not a valid product configuration and fails
  an unrelated mutually-exclusive `makeOreContext` cfg contract; it is not
  used as evidence.

## Provisional result

Implementation status: **CORRECTED CANDIDATE**. Certification status:
**PENDING TWO FRESH INDEPENDENT REVIEWS**. The rejected review remains recorded
above as process evidence, but cannot certify or reject this newer candidate.
