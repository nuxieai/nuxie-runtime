# Mixed-reversal wrapping: ordinary-file ordering candidates

The two static foreground chains fail when only one of main direction and wrap direction is reversed. There is no layout-reactive DrawRules target selector in the inspected ordinary constraint/draw paths. A different file composition remains plausible: keep a fixed draw list, duplicate candidate line groups, and select their visible contents with geometry-driven clipping. The complete ordering candidate is **unvalidated**. A subsequent [visibility-gate experiment](mixed-wrap-gate-review.md) proves its first primitive for 20 px line separation and preserves failures at tiny and zero separation; it does not demonstrate the complete solution.

## What the immutable source establishes

Paths below are relative to `crates/nuxie-runtime/src/mechanical_port/source/`. Source hashes and exact equality to the immutable baseline are recorded in [mixed-wrap-order-audit-receipt.json](mixed-wrap-order-audit-receipt.json).

- `generated/draw_rules_base.rs:14` stores one integer `draw_target_id`. `draw_rules.rs:23` resolves it on import; `:42` re-resolves when that property changes and marks draw-order dirt. It does not inspect coordinates, layout siblings or flex lines.
- `artboard.rs:937` resolves inherited rules for authored drawables; `:1039` builds a target dependency graph. `:1122` puts drawables into their active target chains and `:1172` splices those chains before/after fixed target drawables. `:2375` traverses the resulting draw list. Re-sorting the same target graph cannot derive a new line-dependent permutation.
- `constraints/targeted_constraint.rs:96` requires a transform-component target. DrawRules is a ContainerComponent (`generated/draw_rules_base.rs:14`), not a transform component. The existing translation, rotation, scale and transform constraints constrain transforms; none writes a DrawRules target ID.
- `constraints/translation_constraint.rs:65–142` supports source/destination spaces, copy factors, local offsets and min/max clamping. Parent-local coordinates can therefore potentially expose the difference between two measured line anchors. `constraints/scale_constraint.rs:146` preserves position while changing scale; it is not a general position-to-property bridge.
- `constraints/layout_constraint.rs` is a trait for specialized runtime participants, not an independently serialized expression object exposing a flex line index.
- `constraints/distance_constraint.rs:63–82` normalizes the vector from a fixed target to the constrained point to an authored distance in exact mode. It returns without changing the point when distance is below **0.001**. This offers a possible zero/nonzero spatial gate only when the input separation is known to exceed that cutoff; it is not an exact general Boolean comparator.
- `shapes/clipping_shape.rs:232` attaches clipping to drawable occurrences under its parent. `:275` accepts a Node source and gathers ordinary Shape paths below it; moving those shapes can change clipping without changing draw order. The source paths are updated through dependencies (`:291`). `foreground_layout_drawable.rs:78` draws a layout parent's current path, allowing multiple authored paint occurrences to use the same responsive geometry.

These are source findings, not broad absence proofs over every Rive object. Animations, bindings, state machines, scripting and host setters could change properties, but are outside this goal and are not proposed as workarounds.

## Candidate: clipped groups for possible line leaders

For a flat order-sorted sequence of n flex items, line membership is contiguous. Consider one potential paint group for each item i, in order-sorted sequence. Group i represents the flex line that would begin at i. Within every group, include a painted copy of each item j in the required main-direction order.

Activate group i only when i is the first item or its preceding order-sorted neighbor is on a different line. Within an active group, show copy j only when j shares i's line. Groups are drawn in ordinary or reversed line sequence according to wrap direction; members are drawn in ordinary or reversed sequence according to main direction. Each visible item should then paint exactly once, including in mixed-reversal profiles, while the actual draw graph remains static. This is a proposed construction, not a proof that its geometric gates or semantics work.

For the initial fixed-size profile, perpendicular wrappers that stretch to each line can expose a common cross-axis anchor for all members. Comparing authored item positions directly is insufficient because align-self:center/end changes individual cross positions. A strictly positive known lower bound on distinct-line cross separation is also required. Parent transforms, zero-height lines, negative margins, overlapping line boxes and floating-point equality complicate this premise and remain unresolved.

One ordinary gate to investigate:

1. Use a Node following wrapper i, a child Node following wrapper j in world space, and a root-level Node copying the child's local cross translation. The intended result is the line-anchor difference, with its other coordinate zero. This dependency/space composition must be tested rather than assumed.
2. Apply exact DistanceConstraint normalization against the origin. Same-line zero should remain zero; sufficiently separated lines should move to an authored large distance with the known wrap-relative sign.
3. Move a full-viewport clipping rectangle by that signal. An offset and sign can select either same-line or different-line conditions. Two independent clipping shapes intersect the leader and member conditions. Place masks beyond the entire allowed viewport when disabled, rather than through partially visible artwork.
4. Parent each ClippingShape directly to its particular authored foreground occurrence so duplicated copies do not share visibility unintentionally. Keep ForegroundLayoutDrawable parented to its original layout for geometry. Confirm that clipping self/descendant registration and rule inheritance support this arrangement before expanding it.

The mask is a visibility selector; it must not replace artwork with browser captures or baked layout. Fixed viewport coverage bounds may use the compiler's allowed viewport range, but shape placement, orientation and numerical safety still need proof. The 0.001 normalization cutoff cannot be ignored or hidden with an arbitrary large multiplier: tiny line metrics, nearly coincident anchors and cancellation would need an evidenced bound or diagnostic.

## Cost and limits

A flat container requires O(n²) visible copies and membership gates, plus O(n) leader gates. Group ordering can be represented by a fixed draw chain. This is much better than enumerating all permutations but still expensive: 8,192 items imply more than 67 million potential paint occurrences before clipping/gate objects. Any implementation would require explicit composition resource ceilings and tests; it cannot pretend to preserve existing object budgets unchanged. Nested paint subtrees can multiply cost and need a deeper representation.

Because only one copy should contribute, alpha compositing would theoretically remain unchanged, but that depends on exact mutually exclusive clipping. Repainting duplicates with partial masks could change translucent colors or antialiasing. Clip stack restoration, original and clone dependency evaluation, first-frame settling, and resize reversibility are mandatory tests.

## Next decisive experiment

Start with two fixed-size wrapper anchors whose native layout moves between one and two lines. Attach a single visible foreground rectangle to one authored layout and attempt to clip it using the same-line gate above. Include visible and hidden references, return resize and clone frames, independent clear controls, and geometry reads. If that primitive works, add the inverted leader gate, then reconstruct the existing three-item mixed-reversal fixture using candidate line groups. Preserve failures at each stage.

Do not repeat the two static paint chains or announce a general runtime limitation from their failure. The present evidence identifies both a missing direct mechanism in the inspected paths and a concrete ordinary-file composition worth testing. Public admission remains blocked on actual composition evidence, coverage and resource limits, not on an assumed impossibility.
