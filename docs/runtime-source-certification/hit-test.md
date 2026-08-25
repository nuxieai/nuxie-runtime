# HitTester source certification

> **Correction implemented; independent adversarial review pending.**

## Authority and scope

This receipt reads the complete pinned owners
`src/math/hit_test.cpp` and `include/rive/math/hit_test.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` against
`crates/nuxie-runtime/src/math/hit_test.rs`. The corrected v2 denominator assigns
36 units to the source and three to the header. All 39 are inventoried below;
none is accepted until a separate reviewer has tried to falsify the mapping and
the focused evidence.

| Pinned authority units | Count | Rust ownership / disposition |
| --- | ---: | --- |
| `CULL_BOUNDS`, `MAX_CURVE_SEGMENTS`, `MAX_LOCAL_SEGMENTS` | 3 | Exact constants retained by the unconditional mesh bounds branch and the literal `1 << 8` / `16` segment thresholds. |
| `graphics_roundf`, `graphics_round` | 2 | `graphics_round`; the private float helper is inlined into the Rust expression. |
| Three `Point` constructors, `operator+`, `operator-`, `operator+=`, `operator-=`, and both `operator*` overloads | 9 | Private `Point` construction and field arithmetic in `midpoint`, cubic coefficient construction, clipping, and subdivision. C++ overload syntax is not reproduced. |
| `ave`, `append_line`, `clip_line` | 3 | `Point::midpoint`, `append_line`, and `clip_line`, preserving half-pixel rounding, vertical clipping, winding sign, row order, and x clamping. |
| `compute_cubic_segments` | 1 | `compute_cubic_segments`, retaining the pinned distance formula and 1..256 bound. |
| `CubicCoeff::CubicCoeff`, `CubicCoeff::eval` | 2 | `CubicCoefficient::{new,evaluate}` with the same polynomial grouping. |
| Both `HitTester::reset` overloads | 2 | `clear_windings` preserves the no-argument clear-only behavior; `reset` restores offset, dimensions, zeroed winding storage, and move state. |
| `HitTester::move`, `line`, `quad`, `cubic`, `recurse_cubic`, `close` | 6 | `move_to`, `line_to`, `quad_to`, `cubic_to`, `recurse_cubic`, and `close`. The unusual pinned `quad` behavior is literal: it only assigns the un-offset endpoint. |
| `quickRejectCubic` | 1 | `quick_reject_cubic`, with inclusive top/bottom comparisons. |
| `CubicChop::CubicChop`, `CubicChop::operator[]` | 2 | The seven subdivision points are local values in `recurse_cubic`; checked Rust indexing replaces the source-local wrapper. |
| `HitTester::addRect`, `HitTester::test` | 2 | `add_rect` and `test`, preserving transformed corner order, authored direction, implicit close, and nonzero/even-odd masks. |
| `cross_lt`, both `HitTester::testMesh` overloads | 3 | `cross_lt`, `test_mesh_point`, and `test_mesh_area`, preserving whole-mesh culling, triangle order, point sign comparison, 1x1 delegation, and the area winding accumulation after every triangle. |
| Header `HitTester()`, `HitTester(const IAABB&)` | 2 | The area constructor is `HitTester::new`. The uninitialized C++ default has no safe callable Rust equivalent; Rust constructs initialized state and exposes the pinned no-argument reset separately. |
| Header include guard | 1 | Not applicable: nonbehavioral C++ preprocessing guard. |

The private `mesh_bounds` uses comparison-shaped `cpp_min`/`cpp_max`, not
Rust's NaN-skipping `f32::min`/`max`, because the pinned `AABB(Span<Vec2D>)`
constructor uses `std::min`/`std::max`. Invalid command order, malformed index
triples, out-of-range indices, negative dimensions, and allocation overflow are
outside defined C++ behavior; the Rust owner fails closed instead of
reproducing unchecked reads, uninitialized state, or signed overflow.

## Demonstrated source-port omission and correction

The prior `faithful` file row was incomplete. Rust had no owners for the
no-argument reset, `quad`, `addRect`, `cross_lt`, or either `testMesh` overload.
The correction translates those exact bodies without introducing a new curve
algorithm or mesh hit-test policy.

Focused evidence in `math::hit_test::tests` now observes:

- transformed clockwise and counterclockwise rectangle accumulation;
- the pinned quadratic endpoint-only behavior, including its lack of offset;
- both mesh overloads, both triangle windings, whole-bounds rejection, and the
  1x1/area paths;
- safe rejection of topology for which C++ has undefined indexed access; and
- the clear-only no-argument reset state transition.

`CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --lib
math::hit_test::tests -- --nocapture` passes all five focused tests. This proves
the new Rust observations, not independent source equivalence. A separate
reviewer must still inspect all 39 rows, adversarial float/NaN/edge cases, and
production call-site reachability before changing this receipt to accepted.
