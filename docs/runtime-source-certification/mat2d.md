# Mat2D source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **first complete post-`db1f0bb51` re-review rejected on
two finite live-caller substitutions**

## `src/math/mat2d.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `Mat2D::fromRotation` | `Mat2D::from_rotation` | exact | `upstream_artboard_transform`; `inverse_and_multiply_match_cpp_contraction_order` |
| `Mat2D::scale` | `Mat2D::scale` | exact | `constructors_scale_translate_invert_and_operators_match_cpp_contracts` |
| `Mat2D::translate` | `Mat2D::translate` | exact | `constructors_scale_translate_invert_and_operators_match_cpp_contracts` |
| `Mat2D::multiply` | `Mat2D::multiply`; `Mul`; `MulAssign` | exact | `inverse_and_multiply_match_cpp_contraction_order`; constructor/operator test |
| `Mat2D::mapPoints` | `Mat2D::{map_point,map_points,map_points_in_place}` | exact | `upstream_map_points_complete_sequence`; `bulk_map_matches_scalar_for_distinct_and_in_place_buffers` |
| `Mat2D::mapBoundingBox(Vec2D*, size_t)` | `Mat2D::map_bounding_box` | exact | `upstream_map_bounding_box_complete_sequence` |
| `Mat2D::mapBoundingBox(AABB)` | `Mat2D::map_bounds` | exact | `upstream_map_bounding_box_complete_sequence` |
| `Mat2D::invert` | `Mat2D::invert` | adapted | safe-Rust `Option<Mat2D>` preserves false-with-unchanged-output without an output pointer; constructor/operator test |
| `Mat2D::decompose` | `Mat2D::decompose` | corrected; first independent re-review accepted | `decompose_preserves_pinned_cpp_contraction_bits` |
| `Mat2D::compose` | `Mat2D::compose` | corrected; first independent re-review accepted | `compose_preserves_pinned_cpp_skew_contraction_bits` |
| `Mat2D::scaleByValues` | `Mat2D::scale_by_values` | exact | constructor/operator test; constraint owner suites |

The bounding-box translation preserves the source's non-obvious sequence: it
maps without translation, selects extrema with NaN-ignoring min/max behavior,
rejects any non-finite pre-translation extent using `right - left >= 0` and
`bottom - top >= 0`, and only then adds translation. Empty, all-NaN, and
infinite point boxes therefore collapse to `(0, 0, 0, 0)` exactly as pinned
C++. An infinite matrix translation is applied after that check and remains
infinite, as it does in a release C++ build.

## `include/rive/math/mat2d.hpp`

The v2 denominator contains 33 header authority rows:

| authority unit | Rust owner | disposition |
|---|---|---|
| include-guard macro `_RIVE_MAT2D_HPP_` | none | not-applicable: non-behavioral include guard |
| identity constructor and its lexical initializer row | `Default`; `Mat2D::IDENTITY` | exact |
| six-float constructor and its lexical initializer row | public six-float tuple representation `Mat2D([f32; 6])` | adapted: Rust value construction |
| `values` | `Mat2D::values` | exact |
| mutable and const `operator[]` | `Index`; `IndexMut` | exact |
| `fromScale` | `Mat2D::from_scale` | exact |
| `fromTranslate` | `Mat2D::from_translation` | exact |
| `fromTranslation(Vec2D)` | `Mat2D::from_translation` with two scalar coordinates | adapted: tuple-vector representation |
| `fromScaleAndTranslation` | `Mat2D::from_scale_and_translation` | exact |
| `operator*=` | `MulAssign` | exact |
| span `mapBoundingBox` overload | slice parameter on `Mat2D::map_bounding_box` | adapted: Rust slice boundary |
| `invertOrIdentity` | `Mat2D::invert_or_identity` | exact |
| six scalar getters `xx`, `xy`, `yx`, `yy`, `tx`, `ty` | same-named Rust getters | exact |
| `translation` | `Mat2D::translation` | adapted: tuple-vector representation |
| six overloaded scalar setters | `set_xx`, `set_xy`, `set_yx`, `set_yy`, `set_tx`, `set_ty` | adapted: Rust cannot overload getter/setter names |
| `determinant` | `Mat2D::determinant` | exact |
| matrix-vector `operator*` | `Mul<(f32, f32)>`; `transform_point`; render-API `Mat2D::transform_point` | corrected contraction and final-add operand order; pending fresh independent re-reviews; tuple-vector representation is the approved adaptation |
| matrix-matrix `operator*` | `Mul<Mat2D>` | exact |
| `operator==` and `operator!=` | derived `PartialEq` | exact |

The two lexical initializer rows are conservative parser authority for the
same two constructors; they do not represent additional source behavior. The
tuple/vector and `Option` differences are approved Rust-language adaptations,
not algorithm changes.

## Adversarial findings

The new `mapBoundingBox` owners survive adversarial review. The complete
upstream sequence is green, and two additional bit-exact cases cover the
translation-after-affine grouping and mixed-sign-zero SIMD reduction that the
upstream `Approx` assertions do not distinguish. The empty, partial-NaN,
all-NaN, and infinite-point collapse cases also match. `map_bounds` uses the
exact pinned corner order `(left, top)`, `(right, top)`, `(right, bottom)`,
`(left, bottom)`.

The review instead falsified the pre-existing `decompose` and `compose` rows.
The pinned arm64 C++ source oracle was compiled with the same release
`-ffp-contract=on` policy already cited by the Mat2D inverse/multiply evidence.
For the matrix
`[1.0000001, PI, E, -EPSILON, 5, 6]`, pinned `decompose()` returns skew bits
`0x3e7aefac`; Rust returns `0x3e7aefaa`. Given the exact pinned decomposed
components, pinned `compose()` returns `yy` bits `0xbc8159e8`; Rust returns
`0xbc8159e0`. The upstream expressions are multiply-plus-add contraction sites:
`m0 * m2 + m1 * m3` in `decompose`, and both
`result[linear] * sk + result[skew]` writes in `compose`. Their Rust owners use
ordinary `*` followed by `+`, unlike the already corrected `multiply`,
`determinant`, and `invert` owners. The simple axis-aligned round trip cited by
the implementing receipt cannot observe either discrepancy.

This was a translation failure, not a platform-specific replacement
algorithm. The correction transliterates every contraction candidate in the
two methods with the same explicit Rust `mul_add` strategy used elsewhere in
this owner: the denominator, determinant numerator, and skew numerator in
`decompose`, plus both skew writes in `compose`. The expected-bit tests are now
ordinary green evidence.

Focused evidence run with `CARGO_INCREMENTAL=0`:

- `cargo test -p nuxie-runtime --lib --no-run`: passed;
- `cargo test -p nuxie-runtime math::mat2d`: all ordinary Mat2D and
  `findMaxScale` tests passed;
- `cargo test -p nuxie-runtime --test mat2d_adversarial`: both bit-exact
  bounding-box cases passed;
- the two contraction tests pass at the exact pinned bit assertions above;
- the source-symbol correspondence check passed against pinned
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Result

All 44 v2 authority rows have concrete Rust owners or a justified
not-applicable/adapted disposition. The implementing pass recovered both
missing `mapBoundingBox` overloads, and the first independent review accepted
those owners plus 31 of the remaining 33 header/out-of-line rows. Its two
contraction counterexamples are now corrected and green. This implementing
lane does not self-certify them; the receipt remains pending until a separate
reviewer accepts the correction.

## First independent correction re-review — REJECTED

The `decompose` and `compose` correction in `508aa10fb` is accepted. This
review compiled the actual pinned `src/math/mat2d.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` with arm64 clang `-O3
-ffp-contract=on` and inspected the generated instructions rather than
inferring behavior from the new tests. Pinned `decompose` emits a separate
second product followed by `fmadd` for all three candidate expressions:
`m0*m0 + m1*m1`, `m0*m3 - m2*m1`, and `m0*m2 + m1*m3`. The Rust correction
has exactly that shape: its addend product is evaluated outside `mul_add`, and
the first product is fused with it. Pinned `compose` emits one `fmadd` for each
skew write; the two Rust `mul_add` calls are exact counterparts. The zero-scale
and zero-skew branches, field-write order, translation preservation, and
`TransformComponents` field correspondence are unchanged and exact. All live
draw, constraint, list-constraint, image, and IK consumers reach the corrected
`decompose` owner directly, and every consumer that calls `Mat2D::compose`
reaches the corrected owner directly.

The complete owner re-review nevertheless found an older false-positive row:
the inline matrix-vector `operator*` is not numerically exact. In pinned arm64
C++, the x coordinate

```text
m[0] * x + m[2] * y + m[4]
```

compiles as a separate `m[2] * y`, an `fmadd` of `m[0] * x`, and then a
separate translation add. Runtime `Mat2D::transform_point` and render-API
`Mat2D::transform_point` both use ordinary Rust `*`/`+`, so neither performs
the first contraction. For

```text
m = [1.0000001, 0, 1.0000001, 1, 0, 0]
p = [PI, -2.7182817]
```

the actual pinned C++ operator returns x bits `0x3ed8bc3d`; the current Rust
expression returns `0x3ed8bc40`. The constructor/operator unit test is
circular here: it only asserts that `Mul<(f32, f32)>` equals the same
`transform_point` owner. This path is live across hit testing, input mapping,
semantics, constraints, bones, and text; the render-API duplicate is also live
for mesh and slice-mesh UV mapping. The previously accepted matrix-vector row
therefore cannot remain certified.

The live-path sweep also found a separate downstream source-owner gap in
`constraints/ik_constraint.rs::constrain_ik_rotation`. Pinned
`IKConstraint::constrainRotation` emits contracted `fmla` skew writes, while
the Rust transliteration still uses ordinary `*` followed by `+`. That finding
belongs to the IK-constraint receipt rather than to `Mat2D::compose` because
the pinned IK source also performs the writes inline, but it must not be lost
from the campaign.

The approved safe-Rust boundaries remain sound: a slice replaces `values()`'s
raw pointer; bounds-checked indexing replaces C++ out-of-bounds undefined
behavior; disjoint `map_points` plus the explicit in-place form preserve the
two defined buffer uses without exposing overlapping mutable references; and
`Option<Mat2D>` preserves `invert`'s failure-with-unchanged-output contract.
The identity constructor, equality, multiply, inversion, bulk mapping, and
bounding-box owners examined in the earlier review were not invalidated by the
correction.

Focused evidence for this re-review, all with `CARGO_INCREMENTAL=0` where
applicable:

- `cargo test -p nuxie-runtime math::mat2d --lib`: 9 passed, including both
  corrected bit-exact cases;
- `cargo test -p nuxie-runtime --test mat2d_adversarial`: 2 passed;
- `make --no-print-directory runtime-source-symbol-check`: 7,818 authority
  units across 1,105 owners, passed;
- direct pinned C++ and equivalent Rust probes reproduced the matrix-vector
  counterexample above;
- arm64 assembly inspection confirmed the exact contraction sites in both the
  corrected Mat2D methods and the still-divergent IK call path.

Verdict: **REJECTED for Mat2D certification, while accepting the
`decompose`/`compose` correction itself.** Correct the runtime and render-API
matrix-vector owners from the pinned instruction grouping, add a non-circular
bit-exact counterexample, and obtain a new independent review. Track the IK
skew-write correction under its own source owner.

## Matrix-vector correction after first re-review — PENDING

The rejected matrix-vector row is mechanically corrected in both live owners.
Pinned arm64 C++ emits a separate `m[2] * y` product, contracts
`m[0] * x + product`, and performs the translation add afterward. Runtime
`Mat2D::transform_point` and render-API `Mat2D::transform_point` now spell that
exact grouping as `m0.mul_add(x, m2 * y) + tx` for each coordinate. Runtime
`Mat2D::transform_direction`, the approved tuple-vector owner for pinned
`Vec2D::transformDir`, uses the same contraction without translation. Rust's
explicit `f32::mul_add` makes the proven pinned contraction portable instead
of depending on the Rust compiler to reassociate ordinary arithmetic.

The shared-helper audit distinguishes two upstream algorithms that must not be
collapsed:

- runtime `map_point`, `map_points`, and `map_points_in_place`, render-API
  `map_raw_path_point`, and draw's `map_point_affine` represent pinned
  `Mat2D::mapPoints`; they correctly keep translation inside the skew addend
  before the outer contraction;
- scalar point, direction, `Mul<(f32, f32)>`, mesh UV, slice-mesh UV, hit-test,
  input, semantic, constraint, bone, and text callers all reach one of the
  corrected operator/`Vec2D` owners instead of retaining another ordinary
  multiply-plus-add copy.

The old constructor/operator assertion was circular because it compared
`Mul<(f32, f32)>` only with `transform_point`. It now checks the literal
non-adversarial result `(36, 52)`. New tests use two direct pinned C++ oracles:

- the review witness returns `0x3ed8bc3d` in both coordinates and distinguishes
  the contracted two-product sum from the former Rust expression;
- a second pinned witness returns `0x3fd590f7` in both coordinates, while the
  nested `mapPoints` grouping returns `0x3fd590f6`, proving translation remains
  a separate final add.

Focused correction evidence, with `CARGO_INCREMENTAL=0`:

- `cargo test -p nuxie-runtime math::mat2d --lib`: 10 passed;
- `cargo test -p nuxie-runtime --test mat2d_adversarial`: 2 passed;
- `cargo test -p nuxie-render-api mat2d_point_transform_preserves_pinned_cpp_operator_contraction_bits --lib`: 1 passed;
- `cargo test -p nuxie-render-api --lib`: 27 passed;
- source correspondence: 456 applicable owners, 0 pending absent rows;
- source-symbol correspondence: 7,818 authority units across 1,105 owners,
  with generated authority replayed;
- source-symbol checker unit suite: 33 passed;
- direct arm64 C++ `-O3 -ffp-contract=on` execution and assembly inspection
  produced the expected bits and the `fmul`/`fmla`/final-`fadd` sequence.

The separate inline IK skew-write finding remains outside this correction and
belongs to `IKConstraint::constrainRotation` ownership. This implementing lane
does not self-certify the corrected Mat2D/Vec2D owners. Verdict: **PENDING a
fresh independent re-review.**

## First fresh independent matrix-vector correction re-review — REJECTED

This independent review accepts the finite arithmetic correction in
`1336e9661`. Runtime and render-API `transform_point` now contract the two
linear products, runtime `transform_direction` has the same contraction
without translation, and `Mul<(f32, f32)>` reaches the runtime owner. Direct
arm64 assembly comparison also confirms that the direction owners emit the
same `fmul`/`fmla` sequence. The distinct bulk `Mat2D::mapPoints` translation
continues to be nested inside the skew addend, and its runtime `map_point`,
runtime bulk/in-place forms, render-API raw-path helper, and draw path helper
all emit the expected two-stage `fmadd` grouping. The scripting matrix-vector
operator and renderer point consumers reach the corrected render-API owner;
the audited runtime hit-test, input, semantic, constraint, bone, text, mesh,
and slice-mesh callers reach the appropriate runtime or render-API owner.

The correction is nevertheless not bit-exact for the final translation add
when both the contracted linear result and translation are distinct NaNs. On
this arm64 host, compiling the actual pinned expression with clang 22.1.8 at
`-O3 -ffp-contract=on` emits:

```text
fmul   linear_addend, skew, y
fmla   linear_addend, scale, x
fadd   result, translation, linear_addend
```

The corrected Rust owner emits the same first two instructions but reverses
the final `fadd` operands:

```text
fmul   linear_addend, skew, y
fmla   linear_addend, scale, x
fadd   result, linear_addend, translation
```

That commutation is invisible for finite values, infinities generated from
non-NaN inputs, and signed zero, but arm64 NaN propagation exposes it. For the
x-coordinate witness

```text
m = [qNaN(0x7fc0aaaa), 0, 0, 0, qNaN(0x7fc0bbbb), 0]
p = (1, 1)
```

the pinned C++ operator returns bits `0x7fc0bbbb`, while both corrected Rust
`transform_point` owners return `0x7fc0aaaa`. A two-million-case raw-bit probe
found 112 matrix-point differences of this kind, zero matrix-direction
differences, and zero nested-`mapPoints` differences. A separate three-million
case probe restricted every input to `+0`, `-0`, the smallest positive and
negative subnormal, `+1`, `-1`, the largest finite values, and both infinities
found zero differences in all three algorithms. Thus the blocker is narrowly
the final-add NaN operand precedence, not the accepted contraction or the
translation-grouping distinction.

Spelling the final add as `tx + xx.mul_add(x, xy * y)` (and the corresponding
y expression) emits the same translation-first `fadd` as the pinned arm64
owner while preserving the already accepted non-NaN results. This review does
not edit production; both runtime and render-API owners need that correction
and then a new independent review.

Focused evidence, with `CARGO_INCREMENTAL=0` where applicable:

- `cargo test -p nuxie-runtime math::mat2d --lib`: 10 passed;
- `cargo test -p nuxie-runtime --test mat2d_adversarial`: 2 passed;
- `cargo test -p nuxie-render-api mat2d_point_transform_preserves_pinned_cpp_operator_contraction_bits --lib`: 1 passed;
- `cargo test -p nuxie-scripting mat2d --lib`: 6 passed;
- direct arm64 C++/Rust assembly inspection confirmed the matching contraction
  and the mismatching final-add operand order;
- direct linked C++/Rust randomized and edge-value probes produced the counts
  and qNaN witness above.

The separately identified inline IK skew writes remain an explicit dependency
of the IK-constraint receipt and are outside this verdict, as required.
Verdict: **REJECTED.** Correct final-add qNaN payload precedence in both scalar
point owners and obtain a fresh independent re-review.

## Final-add operand-order correction after first fresh re-review — PENDING

Both scalar point owners now preserve the accepted two-product contraction but
spell the separate final add in pinned ARM64 operand order:
`translation + linear.mul_add(x, skew * y)`. This produces the pinned
`fmul`/`fmla`/translation-first-`fadd` sequence and restores the source's qNaN
payload precedence without changing finite arithmetic.

Literal raw-bit witnesses now cover both runtime and render-API owners. With a
contracted linear qNaN payload of `0x7fc0aaaa` and a translation qNaN payload
of `0x7fc0bbbb`, both coordinates return the pinned ARM64 result
`0x7fc0bbbb`. The accepted `transform_direction` contraction remains
unchanged because it has no translation add. The distinct runtime and
render-API `mapPoints` grouping also remains unchanged.

This correction does not touch the separately tracked IK skew writes and does
not self-certify its production changes. Verdict: **PENDING two fresh
independent re-reviews.**

## First fresh independent final-add correction re-review — REJECTED

This independent review accepts the scalar-point correction in `d85dc97a1`.
For dynamic matrix and point inputs, both Rust scalar-point owners emit the
same ARM64 sequence as the pinned inline C++ operator: a separate skew
`fmul.2s`, a scale `fmla.2s` into that linear addend, and a translation-first
`fadd.2s`. A four-million-case linked raw-bit probe found zero runtime-point
differences and zero render-API-point differences. Its first two million cases
were sampled from positive and negative zero, the smallest and largest
subnormals, the smallest normals, positive and negative one, finite extrema,
both infinities, and positive and negative signaling and quiet NaNs with
distinct payloads; its remaining two million cases used unrestricted raw
32-bit patterns. This includes the rejected review's payload witness:
dynamic point inputs return translation payload `0x7fc0bbbb` in both C++ and
Rust.

The review also compared optimized constant-point caller contexts instead of
mistaking an inlining difference for an owner difference. With `(1, 1)` known
to the optimizer, both pinned C++ and Rust simplify the products and return
linear payload `0x7fc0aaaa`. Thus the workspace release profile may change the
instruction shape at a specialized caller, but it changes the corresponding
C++ and Rust expressions the same way. The source-shaped finite contraction,
translation grouping, signed-zero, subnormal, infinity, signaling-NaN,
quiet-NaN, and payload-precedence checks therefore accept both corrected
scalar owners and `Mul<(f32, f32)>`, which delegates to the runtime owner.

`transform_direction` also remains a distinct and accepted owner. Its linked
comparison had zero differences across the same four million cases, and its
ARM64 body remains only the pinned `fmul.2s`/`fmla.2s` linear contraction with
no translation add. The separately tracked IK skew-write owner was excluded
from this review as required.

The required control audit did, however, invalidate the older claim that all
Rust `mapPoints` forms were exact. The failure is not the new scalar-point fix;
it is an earlier translation error caused by replacing pinned `float2`/`float4`
SIMD source with superficially equivalent scalar `mul_add` expressions:

- runtime `map_point`, `map_points`, and `map_points_in_place` preserve the
  ordinary-real-number grouping but not the pinned SIMD instruction operand
  placement for multiple NaNs. For matrix bits
  `[ffc01234, ffc01234, 7fc0bbbb, 80000000, 00800000, bf800000]` and point bits
  `[ffffffff, 3f800000]`, pinned `Mat2D::mapPoints` returns
  `[7fc0bbbb, ffffffff]`; all three actual runtime forms return
  `[7fc0bbbb, ffc01234]` in the unoptimized owner probe. The linked dynamic
  comparison found 190,377 runtime-map differences in four million cases,
  all in exceptional NaN precedence rather than finite grouping.
- render-API `map_raw_path_point`, used by `RawPath::add_path` and
  `add_path_backwards`, always executes the affine nested-FMA form. Pinned
  `Mat2D::mapPoints` first tests both skew lanes and uses its scale-plus-
  translation branch when they compare equal to zero, including signed zero.
  With matrix bits
  `[007fffff, 80000000, 80000000, 007fffff, 7fa01234, 7f800000]` and point bits
  `[7f800000, 80000000]`, pinned source returns
  `[7fe01234, 7f800000]`; the actual Rust `RawPath` caller returns
  `[7fe01234, 7fc00000]` because its supposedly zero skew is still multiplied
  by infinity. The linked probe found 7,678 render-map differences.

This is precisely the kind of hidden parity failure the control review is
intended to catch: retaining the high-level algebra was insufficient because
the port dropped both an authored branch and the source's SIMD lane ownership.
The correction must restore `mapPoints` from the pinned implementation as one
shared behavioral owner, including its scale specialization and exceptional
operand precedence, rather than patch either witness independently.

Focused evidence, with `CARGO_INCREMENTAL=0` where applicable:

- `cargo test -p nuxie-runtime math::mat2d --lib`: 10 passed;
- `cargo test -p nuxie-render-api mat2d_point_transform_preserves_pinned_cpp_operator_contraction_bits --lib`: 1 passed;
- a temporary actual-owner test confirmed the three runtime map forms and the
  render-API `RawPath` witness above, then was removed;
- direct pinned C++ and source-shaped Rust ARM64 assembly plus linked raw-bit
  comparison produced the counts above.

Verdict: **REJECTED for Mat2D certification while accepting the
`d85dc97a1` scalar-point correction itself.** Restore the complete pinned
`mapPoints` owner in both runtime and render-API paths, add non-circular
exceptional-value witnesses, and obtain fresh independent re-reviews.

## Full `mapPoints` correction after `a1f4809db` — PENDING

The runtime and render-API ports now restore the complete structural owner
from pinned `Mat2D::mapPoints`: the two skew lanes are tested once, the odd
point is processed first, and all remaining points are loaded and stored in
pairs. Both the zero-skew scale/translation specialization and the two-stage
affine path are present. Runtime `map_point`, `map_points`, and
`map_points_in_place` all reach this owner. Render-API `RawPath::add_path`
maps the entire appended source batch, while `add_path_backwards` first copies
the reversed points and then maps that appended slice in place. The previous
point-at-a-time `map_raw_path_point` substitute has been removed.

The live renderer audit found two additional point-at-a-time substitutes.
`gpu.cpp::find_transformed_area` now sends all four corners through the public,
documentation-hidden render-API batch owner. `rive_renderer.cpp`'s inverse-
clockwise-path owner now maps its four bounds vertices through the in-place
batch form. The remaining pinned renderer `mapPoints` call in
`transform_rect_to_new_space` is represented by its previously corrected
shared `map_bounds` owner rather than individual point transforms. Thus no
live pinned `mapPoints` caller remains translated as repeated scalar
`transform_point` calls.

NaN payload selection is recorded as a Rust/compiler lowering ceiling, not
made into runtime math policy. Pinned clang on the validated macOS AArch64
host and Rust/LLVM release fat LTO may choose different payloads while
preserving NaN classification; neither C++ source nor Rust specifies a
portable signaling/quiet payload order for these expressions. The owners
therefore retain ordinary Rust `mul_add` and test exact finite bits plus
exceptional classification. No ARM-specific payload-selection adapter is
present, and other architectures are not claimed by this receipt to share a
particular payload choice.

The rejected runtime multiple-NaN witness is NaN in both coordinates through
the single-point, distinct-buffer odd/pair, and in-place odd/pair forms. The
rejected zero-skew render witness is stronger: both forward and backward
actual RawPath callers produce NaN x but preserve positive-infinity y; the
removed affine substitute produced NaN y by multiplying signed-zero skew by
infinity. Optimized finite/cancellation witnesses prove fused evaluation, the
renderer transformed-area four-point batch, and the inverse-path in-place
four-point batch. A debug source-order transformed-area probe retains pinned
payload `0x7fc0bbbb`; fat LTO may select `0x7fffffff` while retaining NaN
classification, as covered by the approved ceiling.

The caller audit also exposed a separate finite downstream gap that this
Mat2D correction does not absorb. For matrix bits
`[f905d99f, 34100a4d, 4ad09610, 171199a5, 3c480a80, 85c51df6]` and bounds bits
`[9503e6e0, 0b21de72, 1df23437, b7848489]`, pinned clang/AArch64
`gpu.cpp::find_transformed_area` returns `0x27152238`; the corrected Rust batch
caller returns `0x27152232`. The former point-at-a-time substitute returned
`0x27152277`. This is finite cross-product/contraction behavior after mapping,
not NaN payload freedom and not Mat2D ownership. Pinned `PathDraw::Make`
compares this area with `512 * 512`, while the current Rust live path uses a
separate determinant-area substitute in `draw.rs`; a near-threshold difference
can therefore change the interior-triangulation control decision. This remains
an explicit renderer `gpu.cpp`/`draw.cpp` campaign blocker for a separate
correction lane.

The accepted scalar `transform_point` correction in `d85dc97a1`, the distinct
`mapBoundingBox` owner, and the separately tracked IK skew writes are
unchanged. This implementing lane does not self-certify the correction.
Verdict: **PENDING two fresh independent re-reviews.**

## Renderer transformed-area correction after `166a3105f` — PENDING

The finite downstream blocker discovered during the full `mapPoints` audit is
now corrected at its pinned `gpu.cpp::findTransformedArea` owner. The missing
behavior was the contraction and operand order of `Vec2D::cross`: pinned clang
on the validated macOS AArch64 host separately rounds the negated second
product and contracts the first product into it. The Rust owner now expresses
that exact order as `a.x.mul_add(b.y, -(a.y * b.x))`; it does not add or alter
any NaN-payload policy.

For matrix bits
`[f905d99f, 34100a4d, 4ad09610, 171199a5, 3c480a80, 85c51df6]` and bounds bits
`[9503e6e0, 0b21de72, 1df23437, b7848489]`, direct pinned clang/AArch64,
debug Rust, and release fat-LTO Rust all return `0x27152238`. The removed
uncontracted cross returned `0x27152232`; the earlier point-at-a-time mapping
substitute returned `0x27152277`. The existing exceptional-value witness
continues to assert classification rather than compiler-selected NaN payload.

The complete pinned-consumer audit found one production caller:
`PathDraw::Make`. That path now calls the shared `gpu.cpp` owner with its four
mapped points instead of using the determinant-times-local-bounds shortcut in
`draw.rs`. The distinction is observable at the authored control threshold.
For bounds `[0, 0, 512, 512]` and matrix
`[0x3f800001, 0, 0, 1, 0x4e800000, 0]`, direct pinned clang and both Rust test
profiles return exactly `0x48800000`, so the strict `> 512 * 512` comparison is
false. With zero translation they return `0x48800001`, so it is true. The
removed determinant shortcut ignores mapped-point cancellation and returns
`0x48800001` in both cases, selecting interior triangulation incorrectly for
the translated case. The old helper remains only as non-production legacy
surface; no live `PathDraw::Make` path calls it.

This implementing lane does not self-certify the correction. Verdict:
**PENDING two fresh independent re-reviews.**

## Runtime `mapBoundingBox` correction after independent AABB review — PENDING

The first post-`d2605b4de` AABB reviewer found that the public runtime-only
`Mat2D::map_bounding_box` owner still retained the older scalar point fold even
though the operative renderer owner had been corrected. Internal callers do
not currently reach this runtime owner, so the finding did not invalidate the
renderer AABB review. It did invalidate the broader Mat2D claim that every
public owner preserved the pinned algorithm.

The runtime tuple-based owner now spells the same pinned pair-lane algorithm
as the accepted render-API owner: odd-first initialization; two-point lane
loads; the zero-skew specialization; authored affine FMA grouping; SIMD
min/max NaN and signed-zero selection; cross-lane reduction; nonfinite
normalization before translation; and final post-translation non-negative
width/height debug assertions. It does not redirect through the render-API
type or invent a new shared abstraction, so the runtime source owner remains
visible for direct comparison.

Direct witnesses now preserve negative-zero minima and positive-zero maxima
for both two-point orders, normalize a nonfinite linear result to the zero box
before translation, and panic in debug for positive-infinity translation after
finite linear bounds. The complete existing upstream `mapBoundingBox` sequence
and the full runtime Mat2D suite pass: 12 passed, 0 failed, 0 ignored.

This implementing lane does not self-certify the correction. Verdict:
**PENDING two fresh independent re-reviews.**

## First complete post-`db1f0bb51` fresh independent re-review — REJECTED

This review accepts the corrected core arithmetic owners. Runtime and
render-API `mapPoints` preserve the pinned zero-skew specialization,
odd-first/pair traversal, load-before-store in-place behavior, and authored
affine FMA grouping. Runtime and render-API `mapBoundingBox` preserve the
pair-lane extrema algorithm, SIMD NaN/signed-zero selection, nonfinite
normalization before translation, and final debug assertions. The corrected
`gpu.cpp::find_transformed_area` cross-product contraction returns pinned
finite bits in debug and release fat LTO, and its live `PathDraw::Make`
consumer takes the correct side of the strict `512 * 512` threshold. The
approved ceiling was respected: this review did not require a particular NaN
payload when classification and control behavior remained unchanged.

The complete caller audit nevertheless found two finite source-mapping
failures, so the broader Mat2D certification cannot close.

First, live N-slicing still routes pinned scalar matrix-vector operators
through the distinct bulk `mapPoints` owner. Pinned
`NSlicedNode::updateMapWorldPoint` evaluates both `inverseWorld * worldP` and
`world * slicedP`, and `NSlicedNode::deformLocalPoint` evaluates both
`worldTransform * point` and `inverseWorld * deformedWorldP`. The corresponding
Rust paths in `layout/n_sliced_node.rs` use `Mat2D::map_point`, whose owner
deliberately delegates to `mapPoints`, at all six operative call sites. This
is not merely a naming discrepancy: the already established finite witness

```text
m = [bf185aa5, bf185aa5, 3f5b24a3, 3f5b24a3, 3f20f4c4, 3f20f4c4]
p = [bf33ac98, 3f3a0788]
```

returns `0x3fd590f7` from pinned `Mat2D * Vec2D` but `0x3fd590f6` from pinned
`mapPoints`. A fresh optimized Rust probe reproduced that split. The N-slicer
callers must reach `transform_point`, and their correction needs direct
non-circular evidence rather than comparison with `map_point`.

Second, `RiveRenderer::clipRectImplSource` replaces pinned
`transform_rect_to_new_space` with `to_new.map_bounds(rect)`. Pinned source
maps only the two diagonal points in place through `mapPoints`, then performs
one SIMD min/max between those results. The four-corner `mapBoundingBox`
substitute is observably different for matrices admitted by the source's
epsilon test. With identity destination space, rect `[0, 0, 1, 1]`, and
`currentToNew = [1, 0, -0.00001, 1, 0, 0]`, `maxSkew` remains below pinned
`math::EPSILON`, so the rect path is retained. An actual pinned clang/AArch64
`-O3 -ffp-contract=on` probe returned two-point bounds bits

```text
[00000000, 00000000, 3f7fff58, 3f800000]
```

while the current four-corner substitute returns

```text
[b727c5ac, 00000000, 3f800000, 3f800000]
```

This also shows why the current general `map_bounds` tests cannot certify
this caller: both helpers can be individually correct while substituting one
for the other is not.

Focused evidence, with `CARGO_INCREMENTAL=0` where applicable:

- `cargo test -p nuxie-runtime math::mat2d --lib`: 14 passed;
- `cargo test -p nuxie-runtime --test mat2d_adversarial`: 2 passed;
- `cargo test -p nuxie-render-api --lib`: 30 passed;
- `cargo test -p nuxie-renderer --features renderer-metal transformed_area_ --lib`: 4 passed;
- the same four transformed-area tests passed in release with fat LTO and one
  codegen unit;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- source-symbol checker unit tests: 33 passed;
- direct pinned C++ and optimized Rust probes reproduced both finite
  counterexamples above.

Verdict: **REJECTED while accepting the three named core corrections.** Route
every scalar N-slicer operator call to the scalar operator owner, restore the
literal two-point `transform_rect_to_new_space` algorithm, add direct finite
witnesses at both live callers, and obtain new independent complete reviews.

## Finite live-caller correction after `caca34d63` — PENDING TWO FRESH REVIEWS

The two rejected caller substitutions are mechanically corrected without
changing either shared Mat2D algorithm. All six operative matrix-vector sites
in `layout/n_sliced_node.rs` now call the scalar `transform_point` owner used by
pinned `Mat2D * Vec2D`: the inverse/world pair in
`NSlicedNode::updateMapWorldPoint`, and the world/inverse pair in each of the
local path and local gradient deformation paths. A direct N-slicer context
witness preserves pinned finite result `0x3fd590f7` through the actual
deformation path; routing its first transform through `mapPoints` instead
produces `0x3fd590f6`.

`rive_renderer.cpp::transform_rect_to_new_space` is restored as a visible
source owner. It retains the equal-matrix early return, inversion and
composition order, authored epsilon rejection, a two-element diagonal-point
array mapped in place through `mapPoints`, and source-order SIMD min/max. The
live `clipRectImplSource` consumer calls this owner instead of substituting the
four-corner `map_bounds` algorithm. The admitted tiny-skew witness returns
exact pinned bits `[00000000, 00000000, 3f7fff58, 3f800000]`; the removed
four-corner substitute returns
`[b727c5ac, 00000000, 3f800000, 3f800000]`.

The consumer census found one pinned `transform_rect_to_new_space` call, in
`clipRectImpl`, and six Rust N-slicer transform sites corresponding to the
four upstream matrix-vector expressions (the local path and gradient routes
share `NSlicedNode::deformLocalPoint` semantics). No N-slicer production site
still calls `map_point`, and no clip-rect production site still calls
`map_bounds` for the current-to-existing-clip conversion.

Focused evidence, all with `CARGO_INCREMENTAL=0` where applicable:

- the actual N-slicer and renderer two-point finite witnesses passed in debug;
- both witnesses passed under the workspace release profile's fat LTO and
  single codegen unit;
- renderer Metal, Vulkan, WebGPU, and WebGL2 feature checks passed;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- the source-symbol checker unit suite passed, 33 tests.

This implementing lane does not self-certify either correction. Verdict:
**PENDING TWO FRESH INDEPENDENT REVIEWS.**

## First fresh independent complete review after `60076a4a8` — REJECTED

This review accepts the N-slicer correction. All six operative Rust sites now
reach scalar `transform_point`, preserving pinned `Mat2D * Vec2D` grouping
instead of the distinct `mapPoints` grouping. The actual deformation witness
returns finite bits `0x3fd590f7` in debug and fat-LTO release, while the
deliberately contrasted batch owner returns `0x3fd590f6`. No production
N-slicer site still calls `map_point`.

It also accepts the core batch, bounding-box, and transformed-area owners.
Runtime and render-API `mapPoints` retain the zero-skew branch, odd-first and
pair traversal, load-before-store in-place behavior, and authored FMA
grouping. Both `mapBoundingBox` owners retain pair lanes, source-order SIMD
min/max, signed-zero selection, nonfinite normalization before translation,
and the post-translation debug contracts. `find_transformed_area` retains the
pinned cross contraction, and the live `PathDraw::Make` route takes the
correct sides of the strict `512 * 512` threshold in debug and fat-LTO. The
compiler-only NaN-payload ceiling was not treated as a blocker where
classification and control remained unchanged.

The restored two-diagonal shape of `transform_rect_to_new_space` is correct,
including its equal-matrix return, epsilon gate, in-place two-point
`mapPoints`, and source-order final SIMD min/max. Its inversion and
composition are not correct, however. The function reuses local renderer
helpers `inverse` and `mul`, which are older algebraic substitutes rather than
the pinned Mat2D owners:

- local `inverse` evaluates determinant and translation numerators without
  the pinned contractions and adds a non-source `!det.is_finite()` failure
  branch. With the entirely finite matrix
  `[26cd29b3, 2533fdc2, d01ad4bb, ce87d5a9, 0, 0]`, actual pinned clang/AArch64
  returns determinant bits `0xa7eec560` and `invert` succeeds; the current
  ordinary determinant rounds to positive zero and returns `None`. With
  `[FLT_MAX, 0, 0, FLT_MAX, 0, 0]`, pinned `invert` also succeeds and returns
  `[+0, -0, -0, +0, +0, +0]`, while the extra finite check rejects it. This
  changes both clip-rect fallback control and whether `invertClockwisePath`
  emits its bounds rectangle.
- local `mul` likewise omits all pinned multiply contractions. With current
  matrix bits
  `[9422bf8a, 9788280a, d2ec7e6e, 4d526674, e887c79b, 4bce95e3]`, new-space
  matrix bits
  `[b12b6d28, 2f8cb036, deb18044, 4f302db7, 155fc859, 4858db48]`, and rect bits
  `[daeaf96f, a4eefdb1, 1c3a27c6, 2b866340]`, both algorithms admit the rect.
  Actual pinned clang/AArch64 returns
  `[e8f53a36, 4943d3df, e8f53a36, 4943d3df]`; the current helper returns
  `[e8f53a37, 4943d3df, e8f53a37, 4943d3df]`. This is a finite one-ULP source
  difference, not NaN-payload latitude.

The same helpers expose wider live-call damage. `inverse_matrix` routes
`invertClockwisePath` through the rejected inverse, its later determinant
winding test is independently uncontracted, and `RendererContract::transform`
routes every renderer concatenation through the rejected `mul`. Correcting
only the new clip-rect helper would therefore leave other live pinned Mat2D
callers divergent. The three local uses need one exact renderer Mat2D
multiply/invert owner, plus direct non-circular tests for finite multiply bits,
invert success/failure control, nonfinite determinant behavior, and inverse
path winding/order.

Focused evidence, with `CARGO_INCREMENTAL=0` where applicable:

- runtime Mat2D: 14 passed; adversarial Mat2D: 2 passed;
- render-API library: 30 passed;
- N-slicer caller, transformed-area/threshold (4), two-diagonal clip-rect,
  and inverse-clockwise batch tests passed in debug;
- the same focused caller coverage passed under fat LTO and one codegen unit;
- renderer Metal, Vulkan, WebGPU, and WebGL2 feature checks passed;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- source-symbol checker unit tests: 33 passed;
- actual pinned `mat2d.cpp` compiled with clang/AArch64 `-O3
  -ffp-contract=on` reproduced the finite rect, finite cancellation, and
  infinite-determinant witnesses above; a source-shaped current-helper probe
  reproduced the Rust results.

Verdict: **REJECTED while accepting the N-slicer, batch, bounding-box,
transformed-area, threshold, and two-diagonal portions.** Restore exact local
renderer multiply/invert/determinant ownership at every live caller, add the
direct witnesses above, and obtain two fresh complete reviews.

## Renderer local-owner correction after `0f3f7232b` — PENDING TWO FRESH INDEPENDENT REVIEWS

The renderer-local Mat2D owner in `rive_renderer_cpp.rs` now preserves the
pinned arithmetic and branches instead of using the older algebraic
substitutes. All six matrix-multiply lanes use the source's single contracted
multiply-add followed, for translation, by the separately grouped final
translation add. The shared local determinant uses the authored
`a * d - c * b` contraction. `invert` tests only `det == 0`, accepts infinite
and NaN determinants as the source does, and preserves the source contraction
in both translation numerators.

Every live consumer was audited and corrected through those owners:

- `transform_rect_to_new_space` uses exact local invert and multiply before
  its already-certified epsilon gate and two-point batch map;
- `invertClockwisePath` uses exact local invert and the same exact determinant
  for its winding branch;
- `RendererContract::transform` uses exact local multiply for renderer matrix
  concatenation.

No other call to the removed inverse alias, uncontracted local multiply, or
uncontracted renderer winding determinant remains in this owner. Direct
non-circular witnesses record actual pinned clang/AArch64 results:

- finite cancellation determinant `0xa7eec560`, successful inversion, and all
  six inverse result bits;
- successful `FLT_MAX` diagonal inversion with
  `[+0, -0, -0, +0, +0, +0]`, plus source acceptance of a NaN determinant;
- all six finite renderer-concatenation result bits through the live
  `RendererContract::transform` consumer;
- clip-rect result
  `[e8f53a36, 4943d3df, e8f53a36, 4943d3df]` through the live inverse/multiply
  composition;
- negative contracted winding control through `invertClockwisePath`, where
  the second emitted path point is pinned corner 3 with bits
  `[e8aa8de4, bf61ff57]`.

Focused evidence, all with `CARGO_INCREMENTAL=0` where applicable:

- renderer-metal debug owner tests: 2 passed; inverse-clockwise tests and both
  clip-rect tests passed;
- the workspace release profile's fat LTO and single codegen unit ran all six
  `rive_renderer_cpp` witnesses: 6 passed;
- renderer Metal, Vulkan, WebGPU, and WebGL2 feature checks passed;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- source-symbol checker unit tests: 33 passed;
- pinned `mat2d.cpp` compiled locally on ARM64 with clang `-O3
  -ffp-contract=on` produced the frozen multiply, inverse, determinant, and
  mapped-corner bits used by the witnesses.

This implementing lane does not self-certify the correction. Verdict:
**PENDING TWO FRESH INDEPENDENT REVIEWS.**

## First fresh independent complete review after `f4707d9e6` — REJECTED

This review accepts the correction made in `rive_renderer_cpp.rs` itself. Its
local multiply preserves the six pinned contraction groups and the separate
final translation adds. Its determinant and inverse preserve the pinned
contractions, the sole `det == 0` failure branch, and acceptance of infinite
and NaN determinants. The three live consumers are routed through those
owners: renderer transform concatenation, clip-rect inverse/composition, and
inverse-clockwise inversion and winding. The finite cancellation, finite
composition, infinite-determinant, NaN-determinant, mapped-corner, and winding
witnesses all pass in debug and under fat LTO.

The earlier accepted portions also remain correct in isolation. All six
operative N-slicer matrix-vector sites use scalar `transform_point`, rather
than the distinct bulk grouping. Runtime and render-API `mapPoints` preserve
the zero-skew branch, odd-first and pair traversal, load-before-store in-place
behavior, and authored FMA groups. Both `mapBoundingBox` owners preserve pair
lanes, source-order SIMD min/max, nonfinite normalization before translation,
and debug extent assertions. `find_transformed_area` retains four-point batch
mapping and the pinned cross contraction, and its live `PathDraw::Make`
threshold consumer takes the frozen sides of the strict `512 * 512` branch.
Only compiler-specific NaN payload selection with unchanged classification and
control was treated as latitude.

The complete Mat2D claim still fails because the corrected renderer owner is
not the only operative hand-written Mat2D owner:

- `gpu_cpp.rs` retains ordinary, uncontracted `multiply_mat2d` and
  `inverse_mat2d`. They are live in `ClipRectInverseMatrix::reset` and in every
  image/gradient/clip-matrix branch of `PaintAuxData::set`. With finite matrix
  bits `[26cd29b3, 2533fdc2, d01ad4bb, ce87d5a9, 0, 0]`, pinned clang/AArch64
  produces determinant `0xa7eec560` and admits inversion. The current ordinary
  Rust expression produces `0x00000000` even under fat LTO and rejects it.
  This changes clip coverage and paint-matrix control, not merely a payload.
- The same uncontracted determinant substitute remains live in
  `draw_cpp.rs::contour_directions_for_path`, both triangulator winding fields
  constructed by `make_path_draw_from_source`,
  `draw.rs::build_interior_tessellation`, and the live
  `gr_triangulator.rs::InnerFanTriangulator::new` it constructs. The mapped
  `RiveRenderPath::isClockwiseDominant` owner and the currently test-only
  `draw.rs::clockwise_atomic_negate_coverage` helper repeat the same residue,
  so they cannot serve as exact fallback owners. The finite witness is
  negative in pinned source but positive zero in Rust, reversing clockwise
  contour selection, `reverseTriangles`, and winding negation in the live
  paths; the mapped dominant-winding owner would likewise return the opposite
  result if called.
- The six N-slicer scalar calls are correctly selected, but their captured
  `inverse_world` is built by `draw.rs::runtime_mat2d_invert`, not by the exact
  shared inverse. That helper contracts its determinant but not its two
  translation numerators. For all-finite matrix bits
  `[494a16e6, b6c7c10d, f5c3ef08, 206746c7, f14fd57a, 25756691]`, actual pinned
  clang/AArch64 returns inverse bits
  `[80000000, 89273d80, c8240aad, 9ba9320a, 2e1d3fdc, bb07c630]`; the current
  fat-LTO Rust helper returns the same lanes except translated x
  `0x2e1d3fdd`. This finite one-ULP difference propagates through every
  N-slicer path that captures the inverse.
- `Mat2D::findMaxScale` and renderer-local `max_scale` still translate the
  pinned `sdot` and discriminant contraction candidates as ordinary
  multiplication and addition. For linear matrix bits
  `[c32d8148, c2d1a0c5, 42d93be7, 4345c7ae]`, actual pinned clang/AArch64
  returns `0x43928724`; both current fat-LTO Rust spellings return
  `0x43928723`. The shared runtime owner is presently reached only by its
  tests, while the renderer-local duplicate is live in feather coverage and
  softening selection.

This census also found two pre-existing non-renderer duplicates that contradict
the established pinned contraction policy: `ListenerAlignTarget` and the Lua
Mat2D binding retain uncontracted inverse owners. They require correction in
their own source-owner lanes; they are not accepted as evidence that the C++
pin is nonfused.

Focused evidence, with `CARGO_INCREMENTAL=0` where applicable:

- debug: runtime Mat2D 13, N-slicer caller 1, adversarial Mat2D 2,
  render-API library 30, and upstream RawPath 7 passed;
- fat LTO and one codegen unit: runtime Mat2D 13, N-slicer caller 1,
  adversarial Mat2D 2, render-API library 29, and upstream RawPath 7 passed;
- renderer debug and fat-LTO groups passed: local owners 2,
  inverse-clockwise plus transformed-area callers 5, clip-rect 2, and
  transformed-area/threshold 4;
- renderer Metal, Vulkan, WebGPU, and WebGL2 feature checks passed;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- source-symbol checker unit tests passed, 33 tests;
- independent source-shaped probes used pinned clang `-O3
  -ffp-contract=on` on ARM64 and the workspace-equivalent Rust fat-LTO/single
  codegen-unit profile for the determinant, N-slicer inverse-translation, and
  max-scale witnesses above.

Verdict: **REJECTED while accepting the corrected `rive_renderer_cpp.rs`
owner, scalar N-slicer routing, batch, bounding-box, transformed-area, and
threshold portions.** Restore exact ownership in the remaining GPU, draw,
triangulator, N-slicer inverse, and max-scale paths, add direct finite/control
witnesses at their live consumers, and obtain two fresh complete reviews.

## Complete concrete-owner correction after `dec84a56d` — PENDING TWO FRESH INDEPENDENT REVIEWS

The complete residue census from the first post-`f4707d9e6` review is now
mechanically corrected. No witness was patched independently: each failure
was traced back to the shared or local source owner that the pinned C++ caller
actually uses.

`gpu_cpp.rs` now preserves pinned `Mat2D::multiply`, determinant, and inverse
arithmetic. All six multiply lanes retain the source contraction, with the
translation add kept outside the contraction. Inverse uses the contracted
`a*d-c*b` determinant and contracted translation numerators, tests only
`det == 0`, and therefore retains the source's infinite/NaN determinant
behavior. The operative `ClipRectInverseMatrix::reset` and every
`PaintAuxData::set` image, gradient, framebuffer-flip, and clip-matrix branch
continue to route through these corrected owners. The finite cancellation
matrix from the rejecting review now produces determinant `0xa7eec560`, and a
centered 2x2 clip reset reaches the owner and emits the frozen six inverse
lanes. A separate six-lane finite multiply witness matches pinned clang.

Every renderer determinant/winding substitute now reaches one contracted
renderer determinant owner. This includes feather-atlas fill direction,
`draw_cpp` contour selection, both `PathDraw::Make` triangulator flags,
interior tessellation and clockwise-atomic winding, the mapped
`RiveRenderPath::isClockwiseDominant` owner, and
`InnerFanTriangulator::new`. The review matrix is negative (`0xa7eec560`) in
pinned source but rounded to positive zero in the removed expressions. Tests
exercise the corrected sign through the live contour, feather, atomic,
render-path, and InnerFan consumers rather than accepting a helper-only bit
assertion.

The N-slicer scalar/operator routing accepted by the previous review remains
unchanged. Its captured inverse now contracts both translation numerators in
`draw::runtime_mat2d_invert`. With finite matrix bits
`[494a16e6, b6c7c10d, f5c3ef08, 206746c7, f14fd57a, 25756691]`, translated x
is the pinned `0x2e1d3fdc` instead of the removed one-ULP substitute
`0x2e1d3fdd`; the actual `RuntimeNSlicedNodeContext::deform_world_point`
consumer observes the same frozen lane.

All three operative `findMaxScale` spellings were included in the census:
the shared runtime Mat2D owner, renderer tessellation/feather
`draw::max_matrix_scale`, and `RiveRenderer`'s feather-softening owner. Each
now contracts the three source `sdot` calls and the discriminant candidate.
The finite review matrix
`[c32d8148, c2d1a0c5, 42d93be7, 4345c7ae]` returns pinned `0x43928724` in all
three owners instead of `0x43928723`; renderer evidence reaches feather atlas
scale and the draw-path softening decision.

The two non-renderer inverse duplicates identified by the review were audited
against their pinned callers rather than silently excluded. Pinned
`ListenerAlignTarget::perform` calls `Mat2D::invert` directly, so its obsolete
nonfused substitute was removed and the listener now reaches the exact shared
runtime owner. Pinned Lua `mat2d_invert` likewise calls the same Rive Mat2D
owner; the approved Lua backend adaptation does not authorize changing its
matrix arithmetic, so the binding now reaches an exact shared render-API
determinant/inverse owner. Both retain finite cancellation witnesses.

Focused evidence with `CARGO_INCREMENTAL=0`:

- debug runtime Mat2D: 15 passed, including the exact `findMaxScale` bits;
- debug N-slicer inverse consumer and ListenerAlignTarget inverse consumer:
  one passed each;
- debug scripting Mat2D inverse: one passed;
- debug renderer finite GPU owner, winding consumers, InnerFan, PathDraw
  contour, mapped render-path, and both feather/max-scale consumers: all
  passed;
- Metal, Vulkan, WebGPU, and WebGL2 renderer feature checks passed;
- source correspondence passed with 456 applicable owners and no pending
  absent rows;
- source-symbol correspondence passed with 7,818 authority units across
  1,105 owners and generated authority replayed;
- source-symbol checker unit tests passed, 33 tests.

The workspace fat-LTO/single-codegen-unit run covers the same finite and
control witnesses. This implementing lane does not self-certify any corrected
owner. Verdict: **PENDING TWO FRESH INDEPENDENT COMPLETE REVIEWS.**
