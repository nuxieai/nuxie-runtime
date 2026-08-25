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

## First post-`42ea58166` fresh independent adversarial review

Verdict: **REJECTED**.

This review started from the combined correction at exact commit
`42ea58166c4e27da0008b3682f5d9fdb61fd18f7` and independently re-read the
pinned AABB, Mat2D, RawPath, PathDraw, RenderContext, RiveRenderer, and backend
call chains. It did not use the earlier review conclusions as authority. The
two bypasses corrected by `42ea58166` are genuinely repaired: the operative
`PathDraw::Make` route now maps the complete raw point span, maps the
stroke/Feather outset through the four-corner overload, no longer rejects a
nonfinite mapped box as `None`, and `LogicalFlush::pushDraws` now calls
`!bounds.empty()`. Two other exact-source omissions prevent acceptance:

1. Pinned `src/math/mat2d.cpp:164-167` bit-casts the final translated lanes to
   `AABB`, then asserts `width() >= 0` and `height() >= 0` before returning.
   Pinned `renderer/src/draw.cpp:452-455` immediately repeats those two checks
   for `PathDraw::Make`. Rust `nuxie-render-api/src/lib.rs:152-159` returns the
   translated AABB without either post-translation assertion, and the ordinary
   live path at `nuxie-renderer/src/draw.rs:479-481` rounds it without restoring
   the PathDraw checks. This differs when finite points have a nonfinite
   translation: a debug C++ witness compiled against the pinned `mat2d.cpp`
   with identity linear terms and positive-infinity X translation terminated
   at line 165 with exit 134, while the corresponding Rust witness did not
   panic and returned positive-infinity left/right with a NaN width. Linear
   nonfinite normalization is correct; the missing checks occur after the
   authored translation step.

2. Pinned `renderer/src/render_context.cpp:707-719` performs the scalar
   Feather-atlas allocation checks, constructs `paddedRegion`, and then
   separately asserts that an atlas-sized `AABBu16` contains the resulting
   region. Rust `render_context_cpp.rs:5950-5961` retains the scalar checks and
   constructs the shared typed AABB, but omits the final
   `AABBu16::contains(*padded_region)` call. The scalar checks make the result
   redundant for a valid rectanizer return, but they do not translate this
   authored typed-AABB operation and cannot certify the complete operative
   call graph.

The remaining requested surface survived the fresh audit. `RawPath::bounds`
retains the odd/even pair-lane seed, correct operand order, numeric-over-NaN
selection, signed-zero extrema, infinity handling, and final XY/ZW reduction;
the odd first-point seed leaves ZW at the source infinities until the final
fold. `Mat2D::map_bounding_box` otherwise retains zero-skew multiplication,
affine fused grouping, pair-lane initialization and reduction, normalization
before translation, and exact four-corner order. Float `Aabb` retains
`#[repr(C)]`, four-float size/alignment, and field offsets. The live inner
Feather path reaches `RawPath::bounds`, and no `transformed_control_bounds` or
renderer-local float-map substitute survives. All renderer integer aliases
resolve to the shared `TypedAabb`; aside from the missing Feather-atlas call
above, the operative scissor, flush, RiveRenderer, WebGL2, WebGPU, Vulkan, and
Metal containment/intersection/empty call sites reach that owner. The manual
comparison in `isOutsideCurrentFrame` remains the separately authored pinned
SIMD-shaped predicate rather than a hidden TAABB expansion.

Evidence from the detached exact-commit worktree:

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api`: all 42 repository tests
  passed; a disposable Rust exceptional witness additionally confirmed the
  missing post-translation panic.
- The compiled pinned C++ exceptional witness terminated with exit 134 at
  `Mat2D::mapBoundingBox`'s width assertion for positive-infinity translation;
  its finite affine, odd/even RawPath, signed-zero, numeric-over-NaN,
  four-corner, and nonfinite-linear probes matched the Rust results.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_aabb`:
  12/12 passed.
- The focused live inner-Feather first-NaN test passed 1/1 after mirroring only
  the repository's ignored fixture assets into the detached worktree.
- The two path-bound tests and both transformed Feather/stroke-outset tests in
  `nuxie-renderer` passed 4/4.
- Independent `CARGO_INCREMENTAL=0 cargo check -p nuxie-renderer --features`
  runs passed for `renderer-vulkan`, `renderer-webgpu`, `renderer-webgl2`, and
  `renderer-metal`.

Required correction: restore the post-translation width/height debug checks in
the shared `Mat2D::map_bounding_box` owner (and retain the PathDraw boundary as
needed for literal correspondence), and restore the live atlas-sized
`AABBu16::contains(*padded_region)` assertion. Then restart both post-correction
independent reviews.

Implementation status: **CORRECTED CANDIDATE REJECTED**. Certification status:
**REJECTED BY FIRST POST-`42ea58166` FRESH INDEPENDENT REVIEW**.

## First fresh independent review of correction `ed8692bed`

Verdict: **REJECTED**.

The correction repairs the direct `RawPath`, `Mat2D`, float-layout, Feather,
and named `RiveRenderer` witnesses, but it does not repair the full live
renderer call graph. Two source-authority bypasses remain:

1. Pinned `renderer/src/draw.cpp:452` computes every non-precomputed
   `PathDraw::Make` bound with
   `matrix.mapBoundingBox(path->getRawPath().points())`; stroked and feathered
   paths also compute the device outset with the four-corner overload at line
   481. Rust instead routes all four renderer product roots through
   `draw_cpp.rs::resolve_path_pixel_bounds` and then
   `crates/nuxie-renderer/src/draw.rs:445-504`. That helper still calls
   `transformed_control_bounds`, transforms points one at a time with
   translation already applied, folds scalar `f32::min`/`f32::max`, rejects
   every surviving nonfinite result as `None`, and computes the stroke/feather
   outset from absolute matrix coefficients. It never reaches the new
   pair-lane `Mat2D::map_bounding_box` owner. This retains precisely the
   translation-timing, lane-order, and nonfinite-normalization substitute that
   the previous review required removing, now on the operative path-draw
   route shared by Vulkan, WebGPU, WebGL2, and Metal. A positive-infinity
   scale over finite points is a direct witness: pinned `mapBoundingBox`
   performs the XY/ZW reduction and normalizes an invalid extent to the
   all-zero AABB before rounding, while `transformed_control_bounds` rejects
   the nonfinite extrema and returns `None` before the source-owned AABB
   operation can occur.

2. Pinned `RenderContext::pushDraws` spells its live admission assertion as
   `!draws[i]->pixelBounds().empty()` at `renderer/src/render_context.cpp:509`.
   Rust still expands that exact `TAABB` policy manually at
   `render_context_cpp.rs:5982` as
   `bounds.left < bounds.right && bounds.top < bounds.bottom`. The corrected
   `needsScissor`, flush-closeout, tightened-clip, clip/path/image/image-mesh
   sites now use shared typed-AABB methods, but this surviving production
   assertion means the claimed removal of member-call expansions is not
   complete. The SIMD-shaped `isOutsideCurrentFrame` comparison remains a
   valid separate source spelling and is not this finding.

The corrected owners themselves survived adversarial re-reading:

- `RawPath::bounds` preserves the pinned odd/even initialization, source
  operand order in every pair fold, and final XY/ZW reduction. Optimized
  probes compiled directly against pinned `simd.hpp` with both Apple clang and
  Homebrew clang confirmed numeric-over-NaN selection, negative-zero `min`,
  positive-zero `max`, and second-payload selection for dual quiet NaNs; the
  Rust scalar owner matches those optimized results.
- `Mat2D::map_bounding_box` preserves the zero-skew multiplication branch,
  affine fused multiply-add grouping, translation after extrema and
  normalization, and the four-corner order. For an odd count it initializes
  only XY from the first point and deliberately leaves ZW at positive/negative
  infinity until the final reduction; the implementation does not contain the
  suspected odd-count ZW overwrite.
- Float `Aabb` has `#[repr(C)]` plus compile-time size, alignment, and field
  offset evidence. The live inner-Feather path now reaches
  `RawPath::bounds().pad(...)` and no longer owns a duplicate path-bounds
  implementation.

Independent evidence was run from a detached worktree at exact commit
`ed8692bed`:

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api`: 42/42 passed (29 unit,
  three canonical-recording, three side-channel, seven upstream RawPath).
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_aabb`:
  12/12 passed.
- Individual `CARGO_INCREMENTAL=0 cargo check -p nuxie-renderer --features`
  runs passed for `renderer-vulkan`, `renderer-webgpu`, `renderer-webgl2`, and
  `renderer-metal`.
- The focused runtime lib-test binary could not be rebuilt in the detached
  worktree because several intentionally untracked `.riv` fixtures are absent
  there; this is an evidence-environment limitation, not the rejection basis.

Required correction: route `PathDraw::Make`'s mapped path bounds and
stroke/feather outset through the shared exact `Mat2D::map_bounding_box` and
four-corner `map_bounds` owners while preserving the already-authored path
preparation phase, and replace the surviving `pushDraws` expansion with
`!bounds.empty()`. Then restart both fresh independent AABB reviews from the
corrected commit.

Implementation status: **CORRECTED CANDIDATE REJECTED**. Certification status:
**REJECTED BY FIRST FRESH INDEPENDENT REVIEW**.

## Correction after first fresh review `4368ad8c4`

Both surviving bypasses were removed at the exact source-shaped callers:

1. `PathDraw::Make`'s operative Rust route no longer calls
   `transformed_control_bounds`. `path_pixel_bounds` now sends the complete raw
   point span through the shared `Mat2D::map_bounding_box` owner and rounds its
   returned AABB. The stroke/Feather route uses that same mapped path box, maps
   `{0, 0, outset, outset}` through the shared four-corner `map_bounds` owner,
   then applies the source width/height plus one-pixel outset before rounding.
   The old point-at-a-time transform, scalar extrema fold, nonfinite `None`,
   and absolute-coefficient outset substitute have been deleted. A focused
   live-call witness proves that positive-infinity scale now reaches pinned
   nonfinite normalization and returns the zero pixel box rather than `None`.

2. `LogicalFlush::pushDraws` now spells the source assertion as
   `!bounds.empty()` through the shared `TypedAabb<i32>` owner. The last known
   manual expansion of this pinned `TAABB` member call is gone.

Focused correction evidence:

- `path_pixel_bounds_uses_pinned_map_bounding_box_nonfinite_normalization`
  passed.
- The ordinary path rounding witness passed.
- Both transformed Feather/stroke outset witnesses passed.

The correction is intentionally limited to the two rejected call-chain
bypasses. The preceding accepted RawPath SIMD, `Mat2D::map_bounding_box`,
typed-AABB, and float-layout owners are unchanged.

Implementation status: **CORRECTED CANDIDATE**. Certification status:
**PENDING TWO FRESH INDEPENDENT REVIEWS**.

## Second fresh independent review of correction `42ea58166`

Verdict: **REJECTED**.

This review independently audited the combined `ed8692bed` and `42ea58166`
candidate against pinned runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, including the operative callers
shared by the Vulkan, WebGPU, WebGL2, and Metal product roots. The rejected
`PathDraw::Make` and `pushDraws` bypasses are corrected, but two exact-source
omissions remain:

1. Pinned `renderer/src/render_context.cpp:707-719` first asserts the scalar
   Feather-atlas coordinates and extents, constructs `paddedRegion`, and then
   separately asserts
   `(AABBu16{0, 0, atlasMaxWidth, atlasMaxHeight}).contains(*paddedRegion)`.
   Rust `render_context_cpp.rs:5950-5961` retains the scalar assertions and
   constructs the same `AABBu16`, but never performs the final shared typed
   `contains` call. The earlier scalar checks make this assertion redundant
   for an ordinary allocator result, but they do not replace the pinned
   typed-AABB owner or satisfy the requirement to translate every operative
   `TAABB::contains`/`empty` call.

2. Pinned `src/math/mat2d.cpp:164-167` bit-casts the translated lanes to an
   `AABB` and then asserts `width() >= 0` and `height() >= 0` before returning.
   Rust `Mat2D::map_bounding_box` returns immediately after adding translation
   and omits both post-translation debug assertions. This is observable for a
   nonfinite translation even though the pre-translation extrema are valid.
   A fresh debug C++ probe compiled directly with the pinned `mat2d.cpp`, using
   identity linear terms and positive-infinity X translation, terminated with
   exit 134 at the pinned width assertion. The corresponding Rust probe did
   not panic and returned positive-infinity left and right coordinates. The
   normalization-before-translation branch itself is correct; the missing
   final assertions are the divergence.

The remainder of the requested surface survived the independent adversarial
sweep:

- `RawPath::bounds` preserves odd/even pair-lane initialization, source
  operand order in the pair folds, and XY/ZW reduction. The Rust `Option`
  empty-path API is normalized by the operative renderer callers to the
  pinned zero AABB.
- `Mat2D::map_bounding_box` preserves the zero-skew branch, odd XY-only seed,
  even pair processing, affine fused grouping, invalid-extent/nonfinite
  normalization, translation timing, and four-corner AABB order, apart from
  the final assertions above. Independent affine, odd-lane, empty, and
  nonfinite probes passed after comparison with hand-derived source results.
- The live path-draw route now sends the complete raw point span through the
  shared `Mat2D::map_bounding_box` owner. Stroke/Feather computes the pinned
  miter/square/Feather radius, maps `{0, 0, radius, radius}` through all four
  corners, and applies its mapped width/height plus one pixel before rounding.
  `transformed_control_bounds` has been removed and has no remaining symbol.
- The other operative typed-AABB member calls correspond: shared render
  context scissor containment, `pushDraws` empty, flush-update empty, and
  tightened-clip containment; `RiveRenderer` clipped/combined empties; and
  Vulkan target/draw containment and empty checks. Local vector/container
  emptiness and platform rectangle conversion were not misclassified as
  `TAABB` calls. `pushDraws` now uses `!bounds.empty()` exactly.
- Float `Aabb` retains its explicit C layout, four contiguous floats, and
  field-order/offset evidence. All four renderer feature roots compile through
  the corrected shared path owner.

Evidence was run from a clean detached worktree at exact commit
`42ea58166c4e27da0008b3682f5d9fdb61fd18f7`:

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api`: 42/42 passed.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_aabb`:
  12/12 passed.
- The focused live nonfinite path-bound witness passed 1/1.
- A temporary independent render-API exceptional probe passed 2/2 for affine
  four-corner mapping, odd RawPath lanes, empty input, and nonfinite linear
  normalization. A separate Rust translation probe passed 1/1 by confirming
  the missing-panic behavior described above; the paired pinned C++ probe
  exited 134 at `Mat2D::mapBoundingBox`'s width assertion.
- Independent `CARGO_INCREMENTAL=0 cargo check -p nuxie-renderer --features`
  runs passed for `renderer-vulkan`, `renderer-webgpu`, `renderer-webgl2`, and
  `renderer-metal` (warnings only).

Required correction: restore the final typed `AABBu16::contains` assertion in
`allocateFeatherAtlasDraw`, and restore the pinned post-translation width and
height debug assertions in `Mat2D::map_bounding_box`. Then restart both fresh
independent AABB reviews from the corrected commit.

Implementation status: **CORRECTED CANDIDATE REJECTED**. Certification status:
**REJECTED BY SECOND FRESH INDEPENDENT REVIEW**.

## Correction after post-`42ea58166` reviews

The two independently reported source-contract omissions are restored without
changing release geometry:

1. The shared `Mat2D::map_bounding_box` owner now retains the pinned final
   `width() >= 0` and `height() >= 0` debug assertions after translation.
   Both ordinary and stroke/Feather `PathDraw::Make` routes repeat those checks
   at their source boundary before rounding or outset, matching the authored
   duplicate assertions. Debug witnesses now panic for positive-infinity
   translation while the separate nonfinite-linear normalization witness still
   returns the pinned zero box.

2. `allocateFeatherAtlasDraw` now performs the final shared
   `AABBu16::contains(*padded_region)` debug assertion after constructing the
   region, in addition to the source's preceding scalar coordinate/extent
   checks.

Implementation status: **CORRECTED CANDIDATE**. Certification status:
**PENDING TWO FRESH INDEPENDENT REVIEWS**.

## First fresh independent review of correction `d2605b4de`

Verdict: **ACCEPTED** at correction commit
`d2605b4ded833e276d857717e953ef7e6503c582` and reviewed HEAD
`a48e0ac75570168cbdfbd0addb61848bea326f52`.

This review independently re-read the pinned AABB, Mat2D, PathDraw, and
RenderContext authority at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, then followed the operative
Vulkan, WebGPU, WebGL2, and Metal call graph. The two blockers reported by the
post-`42ea58166` reviews are corrected with the pinned ordering and build-mode
semantics:

1. Pinned `Mat2D::mapBoundingBox` normalizes invalid pre-translation extents,
   adds translation only on the valid branch, bit-casts the final lanes to an
   `AABB`, and then asserts both final extents. The shared render-API owner now
   performs its `width() >= 0` and `height() >= 0` `debug_assert!` calls on the
   final translated `Aabb`, after the same normalization branch. The positive-
   infinity translation witness therefore panics in debug builds, while the
   assertions compile away in release builds like pinned `assert` under
   `NDEBUG`.

2. Pinned `PathDraw::Make` repeats those width and height assertions
   immediately after `matrix.mapBoundingBox(...)`, before stroke/Feather
   outset and before `roundOut`. Rust preserves that authored duplication in
   both operative helper routes: `path_pixel_bounds` checks the mapped box
   before rounding, and `feather_pixel_bounds_impl` checks it before computing
   or applying the mapped outset. `resolve_path_pixel_bounds` recomputes these
   bounds even when a precomputed value is supplied in debug builds and takes
   the precomputed early return only in release builds, matching the source's
   `#ifdef NDEBUG` boundary.

3. Pinned `allocateFeatherAtlasDraw` performs its scalar coordinate and extent
   assertions, writes `x` and `y`, constructs `paddedRegion`, applies the
   enclosing `AABBu16::contains` assertion, and only then updates the atlas
   maxima and pending draw list. The Rust executable retains that exact order
   and reaches the shared `TypedAabb<u16>::contains` owner for the final
   assertion. The preceding scalar assertions have not been substituted for
   the typed-AABB call.

The broader live-caller sweep found no surviving alternate bounds owner for
these renderer paths. Ordinary and stroked/Feather PathDraw bounds reach the
shared pair-lane `Mat2D::map_bounding_box`; the mapped outset reaches its
four-corner `map_bounds` overload; clip-rectangle, clip-path, and unit-image
bounds in `RiveRenderer` reach those same owners; and renderer containment,
intersection, and emptiness sites continue to reach shared `TypedAabb`
methods. `isOutsideCurrentFrameExecutable` remains a faithful translation of
the pinned SIMD compound comparison, not an expansion of a source `empty()`
member call. The downstream transformed-area correction at reviewed HEAD does
not alter these AABB contracts.

Focused evidence, all with `CARGO_INCREMENTAL=0` where Cargo was used:

- render-API `map_bounding_box` tests: 2/2 passed, including the required
  post-translation debug panic;
- renderer `path_pixel_bounds_` and `feather_pixel_bounds` filters: passed;
- translated upstream AABB suite: 12/12 passed;
- individual renderer checks passed for `renderer-vulkan`, `renderer-webgpu`,
  `renderer-webgl2`, and `renderer-metal`;
- file correspondence passed with 456 applicable rows and zero pending rows;
- symbol correspondence passed with 7,818 authority units across 1,105
  owners, including generated-authority replay.

Implementation status: **CORRECTED CANDIDATE ACCEPTED BY FIRST FRESH
INDEPENDENT REVIEW**. Certification status: **PENDING SECOND FRESH INDEPENDENT
REVIEW**.
