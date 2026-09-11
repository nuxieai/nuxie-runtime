# Align-self with ordinary Rive compositions

The compiler admits auto, stretch, flex-start, center, flex-end and inherit/initial/unset in its single-line box profile. Initial/unset compute to auto; inheritance copies the parent's final computed align-self. The author's direction, dimensions and bounds remain the values inherited by descendants. Generated wrappers do not appear as authored source identities or alter DOM selectors.

Auto/stretch use existing cross-axis Fill; flex-start uses cross-axis Hug for automatic dimensions. Center/end insert one transparent ordinary LayoutComponent with a perpendicular internal axis. The outer wrapper receives the authored item's original main dimension and main bounds. The authored inner box stretches across that constrained dimension. Its original cross dimension and bounds remain intact and are centered/end-aligned along the wrapper's internal main axis. This also permits negative overflow offsets for oversized items.

No percentage100 substitution, host setter, raster fallback, custom rendering adapter, new wire property or runtime modification is used. At most one extra layout component/style pair is emitted per centered/end-aligned authored box. Existing authored element/depth limits remain in force.

## Preserved failed candidate

The first wrapper kept its internal axis parallel to the authored parent. Transferring main bounds to it did not constrain the actual inner box: an automatic width could remain100px despite max-width60px, or20px despite min-width60px. Nine cases failed all eight original/clone frames:72 geometry/pixel failures out of264 frames. The first row max-width pair was directly inspected. Source, binary and failed receipt remain in output/align-self-parallel-wrapper-checkpoint-r1; full native artifacts are in output/public-align-self-r1.

The perpendicular composition solves those tested failures through the runtime's existing cross stretch behavior. This was a compiler composition correction, not evidence that the underlying runtime needed modification.

## Current validation

The core33 scenes pass264 geometry/pixel frames and528 clear checks. Twelve stronger edge scenes pass96 more frames and192 clear checks. They cover all four directions; fixed, automatic and percentage cross sizes; oversized center/end items; automatic and fixed main sizes constrained by min/max; automatic minima; nested wrappers and percentages; inherited values; and ordered translucent overlaps.

All45 scenes have complete visual coverage:55 directly inspected viewport pairs and305 exact decoded image crop/white-extension transfers. Source maps are used only for read-only geometry inspection. Original and cloned scenes resize without recompilation. The frozen compiler reproduces every emitted Rive and source map exactly.76 Rust tests,20 Node tests, TypeScript and the immutable source guard pass.187 prior fixtures retain byte-identical Rive/source maps against the previous compiler; that does not expand their historical rendering qualification. See validation/public-align-self-receipt.json.

## Remaining L03 work

L03 remains partial. Baseline, normal, logical/self alignment names, and safe/explicit unsafe forms are currently diagnosed as unadmitted, not impossible. The existing percentage-height guard under authored auto-height parents remains unchanged.

The [CSS alignment specification](https://www.w3.org/TR/css-align-3/#align-self-property) suggests normal/stretch and start/end/self-start/self-end aliases within the current horizontal LTR profile. Those need explicit browser/native tests before admission. Computed keyword identity must remain available if writing modes or other contexts are added later.

Safe alignment is a separate candidate: ordinary auto margins inside the perpendicular wrapper could absorb positive free space and fall back to physical start when space is negative. It needs oversized and percentage tests; do not reuse that behavior for ordinary unsafe center/end.

For synthesized empty-box baselines, an intrinsic-height row group with bottom-aligned items may align them at the maximum baseline even when the outer container is taller. Nested boxes, mixed baseline/non-baseline participants and noncontiguous sorted groups require more investigation. The [Flexbox baseline rules](https://www.w3.org/TR/css-flexbox-1/#flex-baselines) and ordinary TransformConstraint target bounds are potential inputs to that work. No general baseline limitation has been established.
