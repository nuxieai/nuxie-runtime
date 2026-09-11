# Cross-percentage sizing with native slots and scalar line maxima

A transparent native sizing slot preserves the original percentage containing block, while an ordinary constrained visible child supplies independent item alignment. The two 24-scene experiments pass **384/384 geometry checks and 376/384 pixel checks**, with **768 clear controls**. Eight fractional-position pixel failures remain preserved. These are fixture compositions, not public compiler admission.

## What changes from the failed percentage wrapper

The earlier perpendicular wrapper resolved a child's cross percentage against a line-sized or indefinite wrapper. Here each transparent slot stays directly under the original wrapped flex container and carries the authored main size, cross percentage and cross min/max bounds. Its native cross size therefore resolves against the original container. The visible authored box is a child with 100% dimensions, and its nested white stripe also sizes normally through native layout.

Only the visible box's cross translation is constrained. Slot geometry is never constrained by its visible child or by the scalar graph. All measurements target slots, avoiding a dependency cycle. This proof relies on fixed main dimensions and cross dimensions/bounds that resolve independently of child content. It does not establish an auto/intrinsic slot whose size depends on its 100% child.

The parent receives the existing combined item/line alignment field for start, center or end. Because the slots all inherit that item alignment, a TransformConstraint sampling each slot at the matching edge or center yields a common anchor for its line. Wrap-reverse reverses the physical cross fraction. Width/height measurement subtracts the slot's two edge landmarks; scalar projection removes the unused coordinate.

## Ordinary scalar graph

Adjacent slot anchors determine line boundaries. Their one-axis difference is normalized by an exact DistanceConstraint to a signed 65,536-pixel displacement when separated; its absolute value is the carry-reset penalty. On the same line it is zero in the tested states.

Two scans compute each line's maximum cross size. The forward scan carries the preceding maximum on the same line and resets it at a line boundary; the backward scan does the same in reverse. The maximum of the two scan values gives every member its full line extent. Ordinary Node parenting, TranslationConstraint copy factors and local/world min clamps provide addition, subtraction, scaling, nonnegative clamping and maximum. No host expressions or runtime policy evaluate these equations.

For a slot of cross extent h, line maximum H, native line/item fraction f and desired item fraction g, the visible cross translation is:

`slotTop + (g - f) * (H - h)`

Fractions are 0, 0.5 or 1, with flex-relative reversal applied for wrap-reverse. Fixed native percentages supply h; the graph adjusts translation only. The visible box remains the source-map target, so geometry checks include its position and its displaced descendant. Synthetic slots are omitted from the read-only join. A production compiler must also preserve authored selectors, inheritance and DOM paths; fixture DOM rewriting does not establish that public behavior.

## Cases and results

Both matrices combine row/column, wrap/wrap-reverse, line start/center/end, and percentage versus percentage-bounded sizing. Main dimensions are 60/60/40px; the parent is 50% on both axes. Item A has cross size 50%, or 35px bounded by minimum 50% and maximum 75%. Items B/C have fixed cross sizes 50/30px. Item alignments are center/end/start. A nested white stripe occupies 100% of A's cross size. A following green footer exposes parent extent and overflow.

| Matrix | Views | Geometry | Pixels | Clear controls |
| --- | --- | --- | --- | --- |
| Initial | 400×400 → 200×200 → 120×120 → return | 192/192 | 184/192 | 384/384 |
| Aspect-changing | Rows: 400×80 → 200×240 → 120×400 → return; columns transpose | 192/192 | 192/192 | 384/384 |

The aspect-changing run closes two gaps in the initial matrix. At the first viewport, A is smaller than the 50px sibling while sharing its line, so its visible box must move independently of the slot and its stripe must follow. The bounded variant actively clamps authored 35px to the 75% maximum of 30px. Later states activate its percentage minimum and change line membership. These behaviors are confirmed in the geometry and direct visual review, not merely inferred from the stylesheet.

The eight failures are initial-matrix center/bounds cases at frames 2/6, both axes and wrap modes. All geometry passes, but fractional colored boundaries disagree with Chrome. The separate [final-translation control](wrapped-percent-slots-control-review.md) omits only the visible-item translations and reproduces all eight native images exactly at those views; the Chrome disagreements persist. This excludes the final positioning constraints at those frames, not every aspect of the new representation or every possible raster cause. Larger-view control failures remain unqualified. No tolerance changed.

## Bounds, cost and remaining work

DistanceConstraint does not normalize separations below 0.001. The carry-reset penalty also assumes measured line extents are below 65,536. Both conditions hold comfortably in these fixtures; they are not generic proofs for arbitrary CSS sizes. Very small/zero line extents, floating-point cancellation and larger coordinates still need explicit treatment.

Independent record parsing counts 35 raw records and 167 augmented records in both fixture families: **132 graph records added**, in addition to the three two-record transparent sizing slots already present in the raw input. The forward/backward construction is linear in item count rather than all-pairs maxima, but this fixed-three-item implementation does not establish generalized resource or performance bounds. Its graphs have not yet been combined with the paint-group graphs and their separate cost.

Auto/intrinsic dimensions, percentage main dimensions, automatic minima, more complex bounds, reverse main directions, negative/zero metrics, group opacity, deeper trees and selector/source-map preservation require separate proof or public lowering. In particular, this does not solve `align-content:normal/stretch` or distributed line alignment. The existing public diagnostic remains appropriate until the relevant computed context has an implemented and qualified lowering.

## Evidence

Artifacts live in `output/wrapped-percent-slots-r1` and `output/wrapped-percent-slots-aspect-r1`. Their experimental manifests override raw driver public labels. Each family's 24 RIV files and parsed source maps reproduce exactly through its complete adapter. The aspect run copies the exact original source and augmenter; the preserved build command refers to that original build, and hashes establish copied identity.

All initial-matrix frames have visual coverage through **38 directly reviewed pairs and 154 exact full-RGBA transfers**; all aspect-matrix frames through **48 direct pairs and 144 exact transfers**. Chrome and native equality are proven independently for transfers, including failed initial frames. Source, dependency, tool, image, reproduction and record-count bindings are in `wrapped-percent-slots-receipt.json`. Chrome is pinned to 153.0.8010.12; native rendering uses the unchanged rust-metal RasterOrdering baseline. No public compiler, runtime or renderer changes were made.
