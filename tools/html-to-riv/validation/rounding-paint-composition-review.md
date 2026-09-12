# Read-only live rounded paint composition review

Scope: the frozen validation bridge and its 28 independently authored recipes. This review establishes record semantics and bounded composition reasoning; it makes no pixel, rendering, stream or public-admission claim. No product or immutable source was changed by this review.

## Live corner measurements

The ordinary LayoutComponent `v` remains unsnapped. Two root Nodes each own one TransformConstraint targeting `v`, with origin fractions (0,0) and (1,1). Immutable `LayoutComponent::constraint_bounds()` returns local [0,width] × [0,height]; `TransformConstraint::target_transform_for` applies the live world matrix to these bounds fractions. Generated default constraint strength is 1; source/destination space defaults are World (wire0). Thus these are cumulative start/end corners after native layout, not authored scalar offsets or browser measurements.

The recipe family has translation-only layout, identity artboard space and no dependency back from `v` to generated helpers. The selected coordinates of both corner Nodes therefore satisfy the scalar primitive's intended source shape. Corners carry both coordinates, but the rounding module copies only its selected axis into root Nodes; its final root outputs have zero orthogonal translation. This matters because each later mask inherits its corresponding scalar Node as parent. Rotation, scale, skew, ComponentOrigin on the measurement nodes, transformed artboards and coupled transforms would invalidate this reasoning and are absent here.

## Thin-box rule and local-space arithmetic

`difference(end,start,axis)` places a child under start, constrains the selected world coordinate to end, then copies its local translation into a root Node. Wire `sourceSpaceValue=1` denotes Local: immutable TranslationConstraint multiplies the target's world matrix by its parent's inverse. In this translation-only family the selected result is end minus start and its orthogonal component is zero.

The helper adds -1/16 to that size, runs the four-stage positive predicate, and obtains 0 or 1. It builds `minimum = roundedStart + flag`. The final end is a child of minimum with a world-target copy from roundedEnd and a local-space zero minimum (`minMaxSpaceValue=1`): inverse-parent conversion, lower clamp, then parent conversion computes max(roundedEnd, minimum). All of these final operands are small exact integers. This gives one pixel only when the measured positive size exceeds1/16 and independent rounded edges would otherwise coincide. At exact1/16 the flag is zero; at5/64 it is one. Nonzero positive rounded extents are already at least one, so the same max does not inflate them.

The formula is the positive-size specialization of the pinned Chrome source rule already recorded in fractional-paint-chrome-rule.md. Negative extents are outside this construction. Native world-corner subtraction is not generally a proof of the original layout size: cancellation at large cumulative coordinates can alter a size near1/16. The supplied family uses small integer/dyadic extents and bounded positions, including dyadic percent extents at the sampled viewport sizes; those cases do not justify arbitrary near-threshold sizes. Native versus Chrome LayoutUnit quantization remains independent of correct native-f32 rounding.

## Masks and coverage

The ordinary drawable is a solid [0,32768]² rectangle at the artboard root. Its alpha/color is 0x80ff6030, matching the authored rgba alpha128/255. A separate white Artboard Fill supplies an ordinary file-owned background. The four mask Shapes contain unpainted ordinary rectangles, each32768 square. Their selected-axis ranges are:

- Leading x: [roundedLeft, roundedLeft+32768], orthogonal [0,32768].
- Trailing x: [finalRight-32768, finalRight], orthogonal [0,32768].
- Leading y: [roundedTop, roundedTop+32768], orthogonal [0,32768].
- Trailing y: [finalBottom-32768, finalBottom], orthogonal [0,32768].

For a positive viewport with width/height at most16384 and scalar edges within [-16384,16384], these extra finite mask boundaries cannot cut the desired rectangle inside the viewport. A thin-box final end can be16385 at the upper edge; its trailing lower bound remains negative and therefore also covers the viewport's desired portion. Negative starts are clipped by the existing Artboard clip. The ordinary source artboard initializes clipping enabled, with origin defaults0. This reasoning requires that enabled artboard clip and the identity axis geometry; renderer clip intersection and actual emitted command matrices still require their own observation. No arbitrary antialiasing margin is assumed.

## Closed recipes and limits

For all28 recipes and viewport axes in [0,16384], selected authored starts range from -0.75 to approximately5324.8 for the32.5% offset profile. That profile ends below5370. Fixed offset plus25% extent ends below4161. Constant orthogonal extents are80. Other fixed/dyadic/cumulative profiles are smaller. These bounds leave the measured corners well inside the scalar domain, even when a small viewport clips most content. Cumulative-half composes0.25px padding plus0.25px spacer before rounding; negative-half/three-quarter use negative margins, not negative dimensions. The recipe set's initial and replay viewports satisfy the intended positive viewport bounds.

The JSON bridge itself does not reject arbitrary nonfinite or out-of-domain dimensions, margins, percentages or initial viewports. Therefore it is a validation constructor for this closed recipe set, not a checked general composition API. Public admission would need domain validation, field ownership checks and an aggregate object budget before emitting the four scalar graphs (2256 rounding records alone). No recipe in this campaign exercises multiple overlapping painted owners or changes painting order; those are not established by this bridge review. Rounded backgrounds, image paints, transforms, nonunit DPR, negative sizes and unclipped/unbounded artboards remain outside the evidence.

## Bound files

| File | SHA-256 |
| --- | --- |
| `tools/html-to-riv/validation/rounding-paint-bridge.rs` | `0d0da8784b22b44f842d8640354dd6266a5439f8e7a8c6a3def8e849ef61bdc5` |
| `tools/html-to-riv/validation/rounding-paint-cases.py` | `a069d86ed15cb3d6a561397dd96ec20d74c6ed3a65647686efbb302a40c20f1c` |
| `tools/html-to-riv/validation/rounding-paint-recipes.json` | `8857a14c56ae25e562f4eddc324377186907d733687ed3ef773073914316113e` |
| `tools/html-to-riv/src/paint_rounding.rs` | `b471e7322d14eb410e1437435fdd2168379b913f5ee4464dfd17c7ecae578955` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/translation_constraint.rs` | `b7d9eb0c64ebb1da20fbf37a296e16c41e8ed42b437d59c9f849e39b8d5eac14` |
| `crates/nuxie-runtime/src/mechanical_port/source/constraints/transform_constraint.rs` | `450f0443e9499db7c5b7a4261d77fe5ede678e40f975ec0b77278c6b5b240f8d` |
| `crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs` | `5154d46f471e2497886b2dbd420d5a0532baf8521f6cfe655a432c60d05122c3` |
| `crates/nuxie-runtime/src/mechanical_port/source/transform_space.rs` | `aaddb365b31ca3192fd7632f4d82c0f50d1d43fe1dea75d11debe877783f773f` |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/constraints/transform_space_constraint_base.rs` | `341a7acae38af1c8e4f448ee905844c0d487fe286f92542f2977a3f671133b60` |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/constraints/transform_component_constraint_base.rs` | `5401349fb2b1d5f0759796e425406555c0e0c0dd89c6de98198c2a7641cf5a08` |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/constraints/constraint_base.rs` | `d0d2a32d7a648a4529079b03d6dcc3b3c48e96be63e31584f811d25775dfbe0b` |
| `crates/nuxie-runtime/src/mechanical_port/source/generated/artboard_base.rs` | `7747238c2ed331800999a69730860311b6b8a1eefce47263a3cfb398f310e9e8` |
| `crates/nuxie-runtime/src/mechanical_port/source/artboard.rs` | `6f218337cbf8daa1567f20244cadd9f0676cb1e6546ae0bc23751d02b404da01` |
