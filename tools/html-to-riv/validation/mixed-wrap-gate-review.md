# Ordinary geometry-driven visibility gate

A native layout change can select whether an ordinary foreground drawable is visible without modifying DrawRules or using host logic. The **20 px line-separation fixture passes all 8 geometry and pixel comparisons**, with 16 passing clear controls, across same-file original and clone resizing. The deliberately tiny and zero-height fixtures fail to hide the badge after wrapping. This proves a bounded visibility primitive, **not** the proposed general mixed-wrap ordering composition.

## Construction

The two items each have width 60 px in a row/wrap parent of width 50%. Viewports of 320, 200, 120 and 320 px move them between one and two lines and back. Both items use start alignment. A coral badge under the first item is intended to remain visible only while both items share a line. Parent navy fill and all geometry remain ordinary native layout.

The file adds a Node following the first item's cross translation. A child Node follows the second item's world cross translation. A root Node copies that child's local cross coordinate, expressing the difference. An exact DistanceConstraint normalizes a nonzero difference to 65,536 px relative to a fixed origin. An unpainted rectangular Shape follows that gate Node and supplies a ClippingShape attached directly to the badge's authored ForegroundLayoutDrawable. At zero difference the mask covers the viewport; at a 20 px difference the normalized displacement moves it entirely outside the viewport. The mask uses a 32,768 px rectangle covering positive viewport coordinates at its zero position; this experiment does not qualify that size for every future coordinate/transform profile.

The badge Fill is reparented to the foreground drawable. Other authored layout parents and geometry are unchanged. The mask Shape has no paint. There are 12 added ordinary records: one foreground drawable, four Nodes, three TranslationConstraints, one DistanceConstraint, one Shape, one Rectangle and one ClippingShape. The exact appended record count must follow the source; no public resource-bound claim is made here.

Chrome reference CSS uses a media query solely to express the expected visibility at the analytically known 240 px viewport threshold. The adapter removes that rule before compilation. Native output contains no media-query policy, script, binding, conditional host setter, viewport callback or runtime mutation. The native mask responds to actual layout positions, not a compiled viewport threshold. This reference is a primitive test oracle; it does not claim that media queries are publicly supported.

## Preserved boundaries

`DistanceConstraint` returns without normalization below a distance of 0.001. With 0.0005 px item height, the wrapped line coordinate does not produce the large mask displacement; the badge remains visible. At zero height, both lines have the same cross coordinate and this gate cannot distinguish them. Each boundary case passes 4 of 8 pixel checks (the same-line and return states) and fails the other 4. Geometry passes within the unchanged tolerance for all cases. Chrome's own quantization of tiny heights is not treated as native layout identity.

These failures confirm that a general implementation needs a proven positive line-separation bound or another discriminator. Arbitrarily multiplying coordinates is not yet a validated repair. Different item alignments require shared line anchors, likely the perpendicular wrappers established separately. Wrap-reverse sign, column axes, parent transforms, multiple clips, inverted leader gates, nested opacity and arbitrary overlapping painting remain untested by this primitive.

## Evidence

Artifacts are under `tools/html-to-riv/output/mixed-wrap-gate-r1/`; bindings are in [mixed-wrap-gate-receipt.json](mixed-wrap-gate-receipt.json).

- Three scenes: 24/24 geometry checks, 16/24 pixel checks and 48/48 clear controls. Chrome 153.0.8010.12; immutable rust-metal RasterOrdering.
- All 9 distinct scene/viewport pairs were visually inspected, including every failing viewport state. Fifteen return/clone pairs exactly match those reviewed pairs independently in decoded Chrome and native RGBA.
- Three complete adapter reruns reproduce RIV bytes and parsed source maps exactly. The build command, source/dependency/binary hashes, adapter and raw native receipt are preserved and bound.
- Initial driver rejection for an invalid fixture name is preserved in `render.log`; no native result is inferred from that rejected setup. `render-r2` contains the complete native run. Raw driver public scope labels are overridden by the experimental manifest.

The next useful step is an inverted leader gate and a three-item group-selection experiment from [mixed-wrap-order-audit.md](mixed-wrap-order-audit.md), with explicit positive metric bounds. No quadratic scene expansion or general wrapping admission is justified yet.
