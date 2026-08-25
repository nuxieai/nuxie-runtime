# Mat2D source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: **correction implemented; independent re-review pending**

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
| `Mat2D::decompose` | `Mat2D::decompose` | corrected; pending re-review | `decompose_preserves_pinned_cpp_contraction_bits` |
| `Mat2D::compose` | `Mat2D::compose` | corrected; pending re-review | `compose_preserves_pinned_cpp_skew_contraction_bits` |
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
| matrix-vector `operator*` | `Mul<(f32, f32)>`; `transform_point` | adapted: tuple-vector representation |
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
