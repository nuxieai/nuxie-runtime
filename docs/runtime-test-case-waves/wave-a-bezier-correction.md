# Wave A Bézier correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

The independent Wave A review accepted `bezier_utils_test.cpp` ordinals 3 and
5. This correction leaves those rows untouched and replaces the rejected
evidence for ordinals 1, 2, 4, 6, 7, 8, 9, 10, and 11.

## Corrected evidence

- Case 1 now executes the pinned one-buffer input/output overwrite observable,
  the complete odd multi-chop fixture, five randomized iterations, equal-root
  degeneracy, endpoint assertions, and repeated-one behavior. Rust cannot hold
  simultaneous overlapping immutable and mutable borrows, so the same buffer
  is overwritten after its value-copy read ends. The literal test remains
  expected-red at the concrete unequal middle-point assertion.
- Case 2 supplies `Option::None`, the Rust-safe nullable-input action, and
  compares all produced points with the explicit equally spaced production
  multi-chop result for chop counts 1 through 20.
- Case 4 now includes the pinned quadratic-root and inflection reference,
  exhaustive 256-corner flow, bit-authored near-end fixture, rotation and cusp
  accounting, epsilon-boundary fixtures, exact quadratic, and no-fast-math
  regression assertion. The complete flow passes.
- Case 6 constructs the retained production cubic evaluator once per fixture,
  evaluates the pinned two-lane pairs in the same order, and checks both points
  against the polynomial reference for the complete T loop.
- Cases 7, 8, 9, and 11 use the executable live shader harness. It first checks
  that the repo-owned production GLSL is byte-identical to the pinned shader,
  substitutes that repo artifact into the original C++ shader-test include,
  compiles it with `-ffp-contract=off`, and runs exactly one original Catch2
  case. This replaces the rejected local Rust reconstructions and shader-name
  locators with the production GLSL behavior path.
- Case 10 translates the deprecated upstream convex-90 test helper and executes
  production multi-chop behavior across the fixed corpus, all 256 square-corner
  cubics, and 100 values from an exact `std::mt19937_64`/`Rand::f32` port seeded
  with zero. Cusp-section skipping and every rotation bound are executable.

The prior local Rust reconstructions for shader cases remain only as supporting
coverage; no corrected Wave A row cites them. No production behavior changed.

## Focused evidence

Direct Rust cases 2, 6, and 10 passed. Case 4's complete translated flow passed
when explicitly run. Case 1 failed after executing its translated alias and
randomized flow at the expected exact-degeneracy assertion:

```text
left:  Vec2D { x: 0.4917186,  y: 0.767699 }
right: Vec2D { x: 0.49171862, y: 0.767699 }
```

The four production-shader runs passed with these original assertion counts:

```text
find_cubic_coeffs_tangents_glsl:      1,850
clamped_divide_glsl:                     15
find_cubic_max_height_glsl:           74,370
measure_cubic_local_curvature_glsl:   16,539
```

The final scoped renderer module run compiled successfully and reported 10
passing tests plus the three intentional case-1 expected-red entry points. This
receipt does not self-certify broad Wave A.
