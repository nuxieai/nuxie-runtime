# Coordinate admission for ordinary wrapped-line gates

The current gates need more than a positive CSS size. A useful compiler admission proof must bound line separation, floating-point anchor error, carried extents and the region actually observed through finite masks over the entire claimed resize domain. Sampled passing frames are evidence for a candidate, not those bounds. This audit changes no runtime or public compiler source.

## Source and proof domain

Inspected `output/mixed-wrap-optimized-r1/src/main.rs:9`–`:12`, and `output/wrapped-combined-slots-r1/slot-src/main.rs:19` and `paint-src/main.rs:9`–`:12`. The optimized paint emitter shares signals and emits triangular membership groups, but still normalizes cross-coordinate differences with DistanceConstraint to D=65,536. Combined sizing uses that magnitude as a line-maximum carry reset. Paint masks are rectangles centered at (16,384,16,384), with size32,768: their zero-gate region is [0,32,768] on both axes. Leader inversion subtracts the signed normalized value from D.

`constraints/distance_constraint.rs:71` returns unchanged for distance<0.001; lines75–76 normalize with f32 division/multiplication. `math/vec2d.rs:18`–`:23` computes squared length with f32 multiply-add and square root. TranslationConstraint world/local conversions and slot-edge landmarks introduce additional rounding before this operation.

`SUPPORT.md` restricts compile input viewport axes to (0,16,384]. That check does not enforce a limit on later ordinary runtime resize calls. A public qualification must explicitly identify its resize domain; do not silently infer that the input validator constrains the runtime. The conditions below use every W,H in (0,16,384] as the proposed bounded domain, with ordinary identity-oriented artboard rendering. Broader runtime sizes or observation outside the artboard viewport require recomputed bounds or another composition. No host CSS callback, resize clamp or sidecar is proposed.

## Geometric lower bounds that a compiler can derive

For packed lines with cross extents H_i, common positional line fraction f∈{0,½,1}, and no gap, the common anchor is A_i=S_i+fH_i. Adjacent anchors satisfy:

`|A_(i+1) − A_i| = (1−f) H_i + f H_(i+1)`

Wrap reversal changes the sign, not this magnitude. Container start/center/end offsets are common to the line collection and cancel in exact arithmetic, including overflow. Thus if every possible line has H_i≥L>0, different-line anchors are separated by at least L. Every item's used cross extent≥L is a simple sufficient witness: every nonempty line contains at least one such item. A less restrictive witness can prove a positive item in every possible contiguous line partition, but must account for all main-size resize states and ordering.

Derive cross-size intervals with outward rounding from authored descriptors and parent intervals. A percentage p uses p/100 times its containing-block interval. For the current clamp semantics, used size is `max(minimum, min(base, maximum))`; min overrides a conflicting max. Fixed positive minima can establish L. Percentage-only cross dimensions have infimum0 when the containing block can shrink arbitrarily close to0, so they cannot establish a uniform L merely because all sampled viewports were positive. Explicit min:auto and intrinsic content require an independently proven content bound; do not assume a positive font/box minimum.

An all-zero line can have the same anchor as another line. The existing gate then has no discriminating input, even with infinite amplification. `mixed-wrap-gate-review.md` preserves native/Chrome zero and0.0005px failures. These are failures of this discriminator, not proof that another ordinary composition cannot work.

## Floating-point conditions are separate

Let E_same bound the actual measured same-line scalar difference, and E_gap bound error in a true different-line difference. The unchanged normalizer requires:

- E_same<0.001, so same-line values are not normalized into a false distant line;
- L−E_gap≥0.001, so different lines do normalize;
- finite scalar intermediates and squared distances, with a normalization error bound E_D.

These are insufficient for exact scalar maxima: a same-line residual below0.001 is retained, not changed to zero, so every scan can subtract a small amount from its carry. Over n items, an error allowance must account for accumulation, not only one edge. They are also insufficient for boundary-touching masks if an active gate retains a small translation.

A new standalone f32 source-operation probe is retained under `output/wrapped-coordinate-admission-r1/`. It is arithmetic evidence, not a rendered scene:

- A gap of41 normalizes to65,535.99609375. Leader inversion leaves +0.00390625, not zero.
- The algebraically equal anchors `C+H/2` and `(C+(H−h)/2)+h/2`, with C=16,384, H=50.123 and h=1.002, differ by0.001953125 in f32. Passing that difference to the current normalizer produces65,536. This does not assert that a particular Taffy scene emits exactly this operation sequence; it demonstrates why algebraic equality and positive item minima alone do not prove the implemented anchor calculation safe.

A compiler cannot set E_same to zero by assumption. Either prove exactness for a restricted arithmetic profile, or carry a certified rounding-error budget through the actual immutable layout/transform path. Interval or affine arithmetic must retain shared line-offset relationships; unrelated whole-coordinate intervals alone will lose the cancellation information. Derive operation/path bounds from the actual implementation and scene topology; do not label an arbitrary depth/coordinate cap a proof.

A concrete repair candidate using existing primitives is:

1. Project the difference to one scalar axis and take its absolute value.
2. Subtract a constant ε≥E_same and clamp at zero, making same-line values exactly zero.
3. Require L−E_gap−ε to exceed0.001 plus the subtraction-rounding allowance.
4. Normalize positive differences to D, multiply by2 and clamp at D. If the proven normalized lower bound exceeds D/2, this snaps separated values exactly to D despite division rounding.
5. Use the resulting unsigned0/D signal for membership and `D−signal` for leaders; adapt reverse-wrap sign handling accordingly.

This is an implementable ordinary-Node/constraint candidate, not yet a verified replacement. It changes only signal arithmetic, not authored geometry. ε must follow from a proof; it must not merge distinct lines or impose a new minimum on the design. It also does not repair identical zero-line anchors.

## Carry and mask inequalities

For the unchanged carry `max(0, previousMaximum − separation)`, require every possible line maximum≤D−E_D. Otherwise a different line can leak its previous maximum into the next line. With exact snapped0/D gates, the condition simplifies to H_upper≤D, using a conservatively rounded bound. The compiler's authored numeric cap1,000,000 is already larger than D; percentages can resolve larger still. Propagate resolved bounds rather than checking only lexical numbers. Sum-of-item upper extents bounds total packed cross extent and supports ancestor/world-coordinate analysis; it does not replace the per-line carry bound.

Finite masks must cover the observed coordinate region Q, not necessarily every offscreen paint extent. For a scalar observable interval [q_min,q_max], zero-mask interval [m_min,m_max], active translation error E_0, and inactive displacement ±D with error E_D, sufficient conditions are:

`m_min + E_0 ≤ q_min` and `m_max − E_0 ≥ q_max`

`m_min + D − E_D > q_max` and `m_max − D + E_D < q_min`

Apply coverage in the other axis as well; include raster coverage margins when establishing the observer contract. For Q=[0,16,384] and the current mask [0,32,768], inactive displacement has ample geometric separation, but the lower active edge has zero margin for positive E_0. The41px normalization example exposes that missing margin. A conservatively expanded mask or exact snapped gates can address it without changing visible authored layout; both need native pixel proof.

Negative overflow by itself does not invalidate a mask when the only observed region is the positive artboard viewport. However, rendering translated/cropped views of negative artboard coordinates, allowing a larger observation rectangle, or placing gate arithmetic in a differently transformed nested scope changes Q. Current input validation does not establish those alternative observation contracts. Keep geometry-dependent Nodes in a common identity scalar frame, and apply/track the artboard transform consistently; arbitrary local transforms cannot be ignored.

## Implementable admission certificate

A wrapping plan should produce a compiler-internal certificate per scope containing: resize domain; containing-block size intervals; item/line cross lower and upper bounds; conservative anchor/world ranges; actual arithmetic error budget; gate threshold/reset proof; observable-mask coverage proof; and charged graph/paint costs. This is compile-time reasoning, not a required runtime-policy artifact.

Start with definite-size slots, no authored transforms/gaps/baseline sharing, and positional line alignment where the anchor equation applies. Require a positive witness for every possible line and independently bounded slot sizing. Evaluate the inequalities above over the full domain. If a certificate cannot be derived, return a contextual unresolved-semantics diagnostic and preserve the design; do not silently clamp it or certify from sampled tests.

The current optimized and combined emitters do not yet compute these certificates. In particular, positive fixed cross minima can establish geometric separation, but cannot alone establish same-line floating equality or mask margin. Useful next work is a bounded native probe for the dead-zone/snapped gate and an implementation-derived error analysis, followed by joint percent/paint tests at the admission boundaries. Default normal/stretch line distribution and nested independent wrap scopes remain separate prerequisites from `public-wrap-plan.md`.
