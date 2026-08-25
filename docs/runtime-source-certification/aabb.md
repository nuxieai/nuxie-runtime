# AABB source certification

> **The four findings from the fresh independent review at `19896b470` are
> corrected. Certification is PENDING two new independent reviews of the
> corrected commit. Earlier rejections remain below as process evidence.**

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
**REJECTED BY FIRST FRESH INDEPENDENT REVIEW**. The earlier rejected review
remains recorded above as process evidence. Both independent correction reviews
must restart from the next corrected commit.

## First fresh independent review after renderer consolidation

Verdict: **REJECTED** at candidate commits `51636f6bc`, `64f147bb0`, and
`c6c1ae4fe`.

The direct integer and float bodies in `nuxie-render-api/src/aabb.rs` remain
substantially faithful over their defined source domains. Cross-signed clamp,
intersection, equality, containment, and overlap use a lossless `i128`
comparison domain; unsigned arithmetic wraps as the source unsigned aliases do;
empty intersections normalize to four zeroes; float span and `join` extrema
retain the pinned `std::min`/`std::max` first-operand NaN and signed-zero order;
and all four backend feature configurations compile independently. The
candidate nevertheless cannot be certified because the operative call graph
still contains incorrect or duplicate authority:

1. The corrected Feather caller now has the right source spelling but still
   reaches the wrong bounds algorithm. Pinned `Feather::rebuildInnerPath` calls
   `RawPath::bounds().pad(...)`, and pinned `RawPath::bounds` uses pairwise
   `simd::min`/`simd::max`, which selects the non-NaN operand. Rust
   `RawPath::bounds` instead delegates to `Aabb::from_points`, whose intentionally
   different `std::min`/`std::max` contract preserves a first-point NaN. A
   two-point raw path `[(NaN, NaN), (2, 3)]` therefore bounds to
   `{2, 3, 2, 3}` upstream but to four NaNs in Rust. The existing test
   `raw_path_bounds_use_the_shared_pinned_aabb_extrema_contracts` asserts the
   divergent Rust result, so its green status is negative evidence rather than
   parity evidence. Feather consequently trips `Aabb::inset`'s debug assertion
   while padding (and emits a NaN rectangle when debug assertions are disabled)
   where the pinned call chain emits the finite padded rectangle. This must be
   corrected in the `RawPath::bounds` owner, not patched inside Feather.

2. The receipt classifies renderer `map_bounds` and `map_path_bounds` as harmless
   coordinate conversion, but these helpers are incomplete substitutes for
   pinned `Mat2D::mapBoundingBox`. The source computes extrema before adding
   translation and then collapses any result failing
   `bbox.zw - bbox.xy >= 0` to the all-zero AABB. The renderer helpers transform
   each point with translation already applied, fold extrema, and never perform
   the source nonfinite normalization. For example, a finite unit rectangle
   mapped by an X scale of positive infinity collapses to `{0,0,0,0}` upstream;
   renderer `map_bounds` produces `{+inf,0,+inf,1}`, which `round_out` turns into
   an `i32::MAX`-origin degenerate rectangle. `map_path_bounds` has the same
   defect for all-infinite transformed axes. These are genuine coordinate
   conversion callers, but they still need to reach one exact
   `Mat2D::mapBoundingBox` owner before the shared AABB can certify the result.

3. Typed-AABB ownership is still duplicated in live shared renderer code.
   Pinned `needsScissor` calls `containingBounds.contains(...)`, while
   `render_context_cpp.rs:6787-6790` expands the four comparisons manually.
   Pinned flush closeout calls `renderTargetUpdateBounds.empty()`, while
   `render_context_cpp.rs:7363-7366` expands that policy manually. The same
   source `.empty()` calls remain expanded repeatedly in
   `rive_renderer_cpp.rs` (including clip-path, draw-path, image, and image-mesh
   gates). Their present scalar results match for integer aliases, but they
   preserve independent geometry authority after a correction whose purpose
   was to establish one owner. True backend scissor coordinate/extent assembly
   remains a valid API conversion; these `contains`/`empty` expansions are not.

4. The source float `AABB` is a four-float standard-layout value and its pinned
   implementation loads and stores it as four contiguous floats. Rust
   `TypedAabb<T>` explicitly has `#[repr(C)]`, but the public float `Aabb` does
   not. Its current compiler layout happens to be usable through field access,
   but default Rust representation does not certify source field order or ABI.
   Exact layout should be made explicit and covered by size/alignment/offset
   evidence, or the receipt must narrow its claim and prove that no ABI or raw
   layout consumer exists.

Independent evidence run from this candidate remained green and therefore does
not discharge the findings:

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api`: 40/40 package tests
  passed.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_aabb`: 12/12
  passed.
- `CARGO_INCREMENTAL=0 cargo check -p nuxie-renderer --features
  renderer-vulkan`, `renderer-webgpu`, `renderer-webgl2`, and `renderer-metal`:
  all four independent configurations passed.

Required correction: implement pinned SIMD `RawPath::bounds` extrema in its own
owner; route renderer bounding-box transforms through one exact
`Mat2D::mapBoundingBox` owner; replace live source `.contains()`/`.empty()`
expansions with shared typed-AABB calls while leaving true platform rectangle
conversion local; and settle float-AABB representation with explicit evidence.
Then restart both independent AABB reviews from the corrected commit.

## Correction after review `19896b470`

All four blockers were corrected at their source owners instead of being
patched at their witnesses:

1. `RawPath::bounds` now reproduces the pinned odd/even pair-lane setup,
   pairwise `simd::min`/`simd::max` folds, and final XY/ZW reduction. Its SIMD
   scalar primitive selects the numeric operand when exactly one lane is NaN,
   retains the second NaN payload when both are NaN, chooses negative zero for
   `min`, and positive zero for `max`. This remains deliberately distinct from
   the source-ordered `std::min`/`std::max` behavior of
   `AABB(Span<Vec2D>)`. First-NaN, second-NaN, odd/even counts, signed-zero raw
   bits, and infinity raw bits are covered. The live inner-Feather caller now
   turns a first-NaN/two-point raw path into the same finite padded rectangle
   as the source without tripping the debug assertions in `AABB::inset`.

2. The renderer-local `map_bounds` and `map_path_bounds` substitutes are gone.
   Both rectangle and raw-path call sites reach the render API's single exact
   `Mat2D::map_bounding_box`/`map_bounds` owner. That owner preserves the
   pinned pair-lane ordering, affine FMA grouping, extrema-before-translation,
   and inverse non-negative-extent normalization. A positive-infinity X scale
   over a finite unit rectangle now returns four positive-zero bits before any
   renderer rounding, matching pinned `Mat2D::mapBoundingBox`. The separate
   Mat2D certification receipt owns `mapPoints`; exceptional-value findings in
   that method are intentionally not hidden or modified by this AABB
   correction.

3. Live `needsScissor`, flush closeout, tightened-clip assertions, and the
   clip/path/image/image-mesh gates now call shared `TypedAabb::contains` or
   `TypedAabb::empty`. The source-shaped SIMD boundary test in
   `isOutsideCurrentFrame` remains local because it is not a spelling of a
   `TAABB` member call.

4. Float `Aabb` is now `#[repr(C)]`. Compile-time size, alignment, and all four
   field-offset assertions prove its four-contiguous-float layout; a runtime
   raw-float view proves the same ordering with nontrivial values.

Correction gates:

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api`: 42/42 package tests
  passed (29 unit, three canonical-recording, three side-channel, seven
  upstream RawPath).
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_aabb`:
  12/12 passed.
- The focused live inner-Feather first-NaN test passed.
- Independent `CARGO_INCREMENTAL=0 cargo check -p nuxie-renderer --features`
  runs passed for `renderer-vulkan`, `renderer-webgpu`, `renderer-webgl2`, and
  `renderer-metal`.
- Source correspondence reports 456 applicable owners and zero pending rows.
- The generated source-symbol denominator replays 7,818 authority units across
  1,105 owners.

Implementation status: **CORRECTED CANDIDATE**. Certification status:
**PENDING TWO FRESH INDEPENDENT REVIEWS**.
