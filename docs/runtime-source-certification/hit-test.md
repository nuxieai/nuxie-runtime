# HitTester source certification

> **Independent adversarial review: REJECTED.**

## Authority and scope

This receipt reads the complete pinned owners
`src/math/hit_test.cpp` and `include/rive/math/hit_test.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` against
`crates/nuxie-runtime/src/math/hit_test.rs`. The corrected v2 denominator assigns
36 units to the source and three to the header. All 39 were independently
reviewed against their complete bodies, the corrected denominator, focused
evidence, and pinned call sites. Commit `fdfa71e28` restores the missing Rust
surface, but it does not yet establish literal source certification.

| Pinned authority units | Count | Rust ownership / disposition |
| --- | ---: | --- |
| `CULL_BOUNDS`, `MAX_CURVE_SEGMENTS`, `MAX_LOCAL_SEGMENTS` | 3 | Exact constants retained by the unconditional mesh bounds branch and the literal `1 << 8` / `16` segment thresholds. |
| `graphics_roundf`, `graphics_round` | 2 | `graphics_round`; the private float helper is inlined into the Rust expression. |
| Three `Point` constructors, `operator+`, `operator-`, `operator+=`, `operator-=`, and both `operator*` overloads | 9 | Private `Point` construction and field arithmetic in `midpoint`, cubic coefficient construction, clipping, and subdivision. C++ overload syntax is not reproduced. |
| `ave`, `append_line`, `clip_line` | 3 | `Point::midpoint`, `append_line`, and `clip_line`, preserving half-pixel rounding, vertical clipping, winding sign, row order, and x clamping. |
| `compute_cubic_segments` | 1 | `compute_cubic_segments`; the rejected fused addition is corrected to the pinned separately rounded multiply/add sequence, pending fresh review. |
| `CubicCoeff::CubicCoeff`, `CubicCoeff::eval` | 2 | `CubicCoefficient::{new,evaluate}` with the same polynomial grouping. |
| Both `HitTester::reset` overloads | 2 | `clear_windings` preserves the no-argument clear-only behavior; `reset` restores offset, dimensions, zeroed winding storage, and move state. |
| `HitTester::move`, `line`, `quad`, `cubic`, `recurse_cubic`, `close` | 6 | `move_to`, `line_to`, `quad_to`, `cubic_to`, `recurse_cubic`, and `close`. The unusual pinned `quad` behavior is literal: it only assigns the un-offset endpoint. |
| `quickRejectCubic` | 1 | `quick_reject_cubic`, with inclusive top/bottom comparisons. |
| `CubicChop::CubicChop`, `CubicChop::operator[]` | 2 | The seven subdivision points are local values in `recurse_cubic`; checked Rust indexing replaces the source-local wrapper. |
| `HitTester::addRect`, `HitTester::test` | 2 | `add_rect` and `test` preserve the local bodies, but `add_rect` is not connected to the pinned production `Image::hitTest` caller. |
| `cross_lt`, both `HitTester::testMesh` overloads | 3 | `cross_lt`, `test_mesh_point`, and `test_mesh_area`, preserving whole-mesh culling, triangle order, point sign comparison, 1x1 delegation, and the area winding accumulation after every triangle. |
| Header `HitTester()`, `HitTester(const IAABB&)` | 2 | The area constructor is `HitTester::new`. The uninitialized C++ default has no safe callable Rust equivalent; Rust constructs initialized state and exposes the pinned no-argument reset separately. |
| Header include guard | 1 | Not applicable: nonbehavioral C++ preprocessing guard. |

The private `mesh_bounds` uses comparison-shaped `cpp_min`/`cpp_max`, not
Rust's NaN-skipping `f32::min`/`max`, because the pinned `AABB(Span<Vec2D>)`
constructor uses `std::min`/`std::max`. Invalid command order, malformed index
triples, out-of-range indices, and signed overflow are outside defined C++
behavior; the Rust owner reasonably fails closed instead of reproducing
unchecked reads or uninitialized state. Negative, non-overflowing dimensions
are not categorically undefined, however, so they require a separate domain
decision rather than the blanket exclusion in the original receipt.

## Independent falsification

### 1. Finite cubic segmentation differs under the pinned FP profile

Pinned `compute_cubic_segments` evaluates `dx * dx + dy * dy`. The upstream
unit-test build selects `--no_ffp_contract` in `tests/unit_tests/test.sh`, so
the two products and addition are separately rounded. Rust instead evaluates
`dx.mul_add(dx, dy * dy)`, requiring a fused multiply-add. The correction now
uses `dx * dx + dy * dy`, and a focused bit witness requires the pinned
`0x48364002` squared distance and 36 segments.

A finite witness is `a = (f32::from_bits(0x43d7ffe2),
f32::from_bits(0x3f69e89d))` with `b = c = d = (0, 0)`. Compiling the complete
pinned owner with Clang `-O2 -ffp-contract=off` produces squared-distance bits
`0x48364002`, an exact raw count of `36.0` (`0x42100000`), and segment count
36. The Rust expression produces squared-distance bits `0x48364003`, raw count
`36.000004` (`0x42100001`), and segment count 37. Both inputs and all
intermediate conversions are finite and defined. Because both counts exceed
`MAX_LOCAL_SEGMENTS`, the discrepancy also changes recursive subdivision and
sampling density. This falsifies the claim that the distance formula is
literal.

### 2. The required production `addRect` call edge is still absent

Pinned `src/shapes/image.cpp::Image::hitTest` constructs `HitTester` with the
query area, calls `addRect` using the image bounds and composed transform, then
calls `test`. Rust `HitTester::add_rect` is referenced only by its focused unit
test. The production listener path in
`StateMachineInstance::hit_expandable` sends every non-`Shape` owner,
including `Image`, through `component_hit_test_point`; it never invokes the
restored rectangle raster. Thus the method body exists but the pinned image
hit-test behavior remains unreachable. The mesh overloads have only pinned GM
callers, and `quad`/clear-only reset likewise have no required runtime caller;
their lack of production calls is not independently a mismatch.

### 3. The invalid-area exclusion was too broad

For an `IAABB` whose modest dimensions are negative, `width()` and `height()`
are defined signed subtraction. With one negative axis, the pinned area-mesh
overload converts the negative product to `size_t` for `std::vector`, producing
an allocation failure/exception rather than a boolean miss. Rust converts the
negative axis to no allocation and returns `false`. This may be an intentional
Rust safety adaptation, but it is not undefined C++ behavior and must be
explicitly adjudicated instead of silently certified as literal parity.

The remaining authority rows survived review: half-pixel rounding and
in-range float-to-int conversion, midpoint and polynomial grouping, clipping
and winding order, quick rejection, transform/corner direction, fill masks,
point-mesh sign tests, NaN-preserving `AABB` min/max ordering, persistent area
windings, 1x1 delegation to `(left, top)`, reset state, and the safe initialized
replacement for the otherwise unusable uninitialized default constructor all
match their defined pinned observations.

## Demonstrated source-port omission and correction

The prior `faithful` file row was incomplete. Rust had no owners for the
no-argument reset, `quad`, `addRect`, `cross_lt`, or either `testMesh` overload.
The correction restores those bodies without introducing a new curve algorithm
or mesh hit-test policy. The independent review nevertheless rejects complete
certification because cubic segmentation is numerically different and the
required image caller does not reach `add_rect`.

Focused evidence in `math::hit_test::tests` now observes:

- transformed clockwise and counterclockwise rectangle accumulation;
- the pinned quadratic endpoint-only behavior, including its lack of offset;
- both mesh overloads, both triangle windings, whole-bounds rejection, and the
  1x1/area paths;
- safe rejection of topology for which C++ has undefined indexed access; and
- the clear-only no-argument reset state transition.

`CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --lib
math::hit_test::tests -- --nocapture` passes all five focused tests, and
`make --no-print-directory runtime-source-symbol-check` confirms the corrected
7,818-unit/1,105-owner denominator. Those tests prove the restored surface but
do not cover the finite FP witness or the absent image call edge. Verdict:
**REJECTED** pending production Image/Artboard geometry-hit integration and two
fresh independent re-reviews. The segmentation correction is implemented but
not self-certified. Negative, non-overflowing IAABB extents are explicitly a
safe-ownership adaptation: pinned C++ converts the resulting negative element
count to an impractically large `size_t` allocation, while Rust fails closed
without attempting that allocation. This is not presented as a literal return
value equivalence.

The production correction must be made at the actual pinned call edge:
`Artboard::hitTest -> Image::hitTest -> HitTester::addRect`. It must not be
worked around in state-machine `HitExpandable`, because pinned Image does not
override `hitTestPoint`; listener hit testing and drawable geometry hit testing
are separate surfaces.
