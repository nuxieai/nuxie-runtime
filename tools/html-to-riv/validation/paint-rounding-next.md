# Completed experiment: live paint-only edge rounding

The scalar and standalone paint experiments described below are now implemented. See rounding-scalar-review.md and rounding-paint-review.md: 1696 scalar frames and 224 paint frames pass. The sections below preserve the original design proposal, not current implementation instructions. The implemented negative search uses the separate negative-source pass described in the scalar review.

Next integrate the file composition through compiler-owned construction, checked coordinate domains, independent emitted-field validation and aggregate resource bounds, then test it on the actual wrapped Derived candidate. Preserve its 48 prior pixel failures until a fresh campaign establishes the new result. GOAL-RESTART.md is the authoritative restart order.

## First build a scalar rounding graph

Use only existing Nodes and TranslationConstraints. Work initially over a declared finite coordinate range such as [-16384,16384], with exact source-bit recipes and native original/clone resizing. This range is an experimental domain, not a replacement for all99items or a public admission limit.

Let P(x) be the observed four-stage gain/clamp predicate, one for positive finite x and zero for nonpositive x. Do not use a large constant shift of x before rounding: adding16384 can erase an adjacent-float difference just below0.5.

For nonnegative x, build a binary search for the greatest integer n satisfying n-0.5 <= x. Start accumulator a=0. For powers2^k from16384 down to1, candidate n=a+2^k; bit is 1-P((n-0.5)-x); next accumulator is a+bit*2^k. Integer accumulators and half-integer thresholds are exactly representable in the proposed range. Independently verify subtraction, native parent/local transforms and dependency order; do not assume algebra alone proves the emitted graph.

For negative x, use magnitude max(-x,0) and the strict test P(magnitude-(n-0.5)), then negate the resulting accumulator. This makes negative half ties round toward positive infinity. Evaluate positive and negative parts separately and subtract their nonnegative integer results. Test both signs, signed zero, ±0.5 and adjacent f32 values, quarter fractions, range boundaries, subnormals and dynamic threshold crossings. Record every bit/accumulator and record count before assuming this is a reusable compiler primitive. No DistanceConstraint, binding, animation or host setter is needed by this proposal.

## Then test the resulting paint, not just scalars

A dynamically rounded start and end do not directly set an ordinary Rectangle's dimensions. One plausible file-only paint composition is an ordinary large solid rectangle, clipped by four large ordinary rectangular masks whose translated boundaries are the rounded left/right/top/bottom. Existing TranslationConstraints can move those mask Shapes; the intersection supplies a responsive paint rectangle without changing the layout owner's size or requiring data bindings. Preserve the artboard clip, draw order and original color/alpha. Prove/check the finite mask coverage and actual streamed AABBs; do not assume arbitrary clipping transforms are safe.

This is a vector/clip composition, not rasterization or browser-baked geometry. It is still a candidate. Test first on the20standalone paint controls, adding thin-box and negative/cumulative-origin controls. Chrome's thin-box exception requires a minimum painted extent when the unrounded size exceeds1/16px but rounded extent is zero; blindly rounding the two edges misses that case. Transforms, rounded backgrounds, images and other paints need separate treatment.

Read live start/end anchors from ordinary layout geometry (existing TransformConstraint origin fractions can expose corners). Do not feed getBoundingClientRect into construction or recompile on resize. Preserve unsnapped owner geometry, compare actual paint paths/clips and real native pixels to pinned Chrome, test original/clone sequences, and visually inspect changed outputs.

A successful native-f32 rounding graph does not prove equality to Chrome's already-quantized LayoutUnit layout values. Quantization differences near half boundaries remain a separate admission obligation. Preserve counterexamples instead of hiding them in tolerance or excluding responsive cases from the overall goal.

## Resume and completion boundary

The previous private wrapping matrix still has48pixel failures and standalone paint has24. Public wrapping remains unadmitted. If the proposed graph fails, preserve a minimal native counterexample and continue a discriminating alternative or independent backlog work. If it works, carry it through a compiler-owned bounded primitive, field/domain/resource validation, real wrapping paints and public interface tests before claiming support. Runtime, renderer, schema, dependencies and root build configuration remain immutable throughout.
