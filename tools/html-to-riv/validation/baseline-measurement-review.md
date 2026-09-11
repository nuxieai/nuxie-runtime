# Baseline measurement with ordinary layout objects

Historical checkpoint: current first-baseline admission and remaining restrictions are documented in [first-baseline-review.md](first-baseline-review.md). This supersedes earlier blanket baseline-rejection statements below.

These experiments address the two counterexamples in baseline-landmark-review.md: a changing tallest participant and missing intrinsic parent height. Public baseline support remains rejected pending a general compiler implementation and its qualification.

## Responsive maximum

A zero-width in-flow LayoutComponent is appended under the original parent. Its height is 50% of that parent, with a 60px minimum. The native layout engine therefore computes max(50% of parent height, 60px), matching the largest baseline distance in the authored empty-box fixture. All original participants use the existing bottom-landmark/Y-only constraint composition. The helper does not change main-axis item positions, and the original parent still supplies every percentage basis.

Both row directions pass all 16 geometry/pixel frames and 32 clear controls. The resize sequence makes the percentage-height child grow from 40px to 80px and then shrink to 30px, crossing the fixed 60px participant. Six distinct viewport pairs were visually inspected; ten exact repeat/clone image proofs cover the remaining frames. Both ordinary scenes and diagnostic maps reproduce exactly. This replaces the failed fixed-provider choice with a live ordinary layout expression, without a host maximum calculation or recompilation.

The current experiment recognizes only the authored fixture expression. A general maximum of arbitrary bounded, intrinsic or nested baseline functions is not implemented or implied.

## Intrinsic parent extent

For the fixed nested fixture, the authored baseline distances are 20px, 10px and 40px; the descents are 0px, 50px and 0px. A zero-width helper of height max(ascent) + max(descent) = 90px makes the parent account for the entire baseline group. A landmark at 40/90 of that helper supplies the shared baseline. The nested 60px box anchors at 1/6 of its own height, reflecting its 10px descendant baseline. These values come from authored box dimensions, not browser measurements.

All 64 geometry frames pass across row/reverse and fixed, automatic, minimum-clamped and maximum-clamped parent heights. Automatic parent height is now correctly 90px; minimum and maximum contexts produce the expected 120px and 60px extents. All 128 clear controls pass. Pixel checks pass 56/64: eight frames fail the local RGB gate around the small purple leaf on a fractional horizontal edge. All eight frame-1 pairs were visually inspected; this is not full visual qualification. Tolerances remain unchanged.

## Unconstrained fractional-edge control

A separate fixture uses ordinary transparent layout wrappers and fixed-height spacers to place the same colored rectangles, without baseline helpers, constraints or binary augmentation. All eight geometry frames pass; the same two original/clone pixel frames fail around the leaf. Every corresponding Chrome image and every corresponding native image is decoded-pixel identical to the fixed-parent nested row-reverse experiment: 16 exact image identities.

This establishes that the failure is independent of baseline composition. It does not establish a general renderer cause or justify changing tolerances. The failed native comparisons remain failed. The control, exact output reproduction and comparison proof are preserved in output/baseline-fractional-leaf-control-r1 and bound by baseline-measurement-receipt.json.

## Compiler integration seam

The next implementation should retain compact participant metrics while emitting children sequentially: object IDs, scalar size/bound descriptors, and first/last baseline summaries. It must not retain cloned Style/custom-property environments for every sibling. After recursive emission, a parent can collect the summaries and append ordinary measurement objects, landmarks and constraints without changing authored IDs or selector order.

Recognize evidenced symbolic forms and explicitly reject unresolved combinations. Parent measurement must include both ascent and descent; correct post-layout child positions alone do not establish correct intrinsic layout. General descendant baseline selection, multiple competing responsive expressions, last-baseline combinations and resource bounds remain to implement and validate. The goal still covers the full backlog; these successful compositions do not redefine baseline support around the fixtures.
