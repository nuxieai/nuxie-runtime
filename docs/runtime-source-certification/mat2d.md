# Mat2D source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **final-add qNaN operand-order correction implemented;
fresh independent re-reviews pending**

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
