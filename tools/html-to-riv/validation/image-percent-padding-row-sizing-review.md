# Percentage-padding row sizing: private evidence

At the private capture checkpoint, public image percentage padding was rejected. The subsequent public implementation and its narrower row admission are documented in `public-image-percent-padding-review.md`; the findings below preserve the investigation history. The private capture changed no runtime or production compiler source. A subsequent compiler-only outer-overflow guard is committed as `0457965850`; it does not admit percentage padding. All 315 Rust tests pass and 100 existing image files/maps remain exact. See `image-outer-bound-review.md`. The latest completed public checkpoint is `0176899272`.

The 26-source corpus generated24 private ordinary files and retained two unresolved diagnostics. Initial capture at `output/image-percent-padding-candidate-r2/combined-receipt.json` records192 frames:160 geometry,108 pixel and192 presence passes. Four row cases fail all eight outer-width/tail geometry checks despite correct final inner dimensions. The main 72-pair/120-transfer gallery now has complete direct review of all 24 original-resolution sheets. The review confirms visible outer-background and following-tail displacement in the four row contexts; fractional edge/filtering differences remain elsewhere. Complete review does not make these failing contexts qualified.

At240px parent width, row stretch with height120 and padding5% all sides needs outer168x120 and inner144x96. Native produces outer180x120 with the correct inner144x96 at12,12. Row content-box height80 needs outer144x104 and inner120x80; native retains outer120x104 while the inner is correct. This is distinct from the accidental cross-stretch bug.

## Source explanation and discriminating results

Immutable `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs` creates `child_parent_size = Size::from_cross(dir, cross_axis_parent_size)` at675–676. Row intrinsic-main measurement consequently receives no containing width at782. Percentage padding cannot contribute its original-width-dependent amount during this early measure. The parent freezes the legacy item at its hypothetical main size at1214–1225; later layout resolves padding and inner content with the full parent dimensions but retains that outer target width. Column measurement retains width as its cross-axis input. This source explanation is consistent with the retained actual measurements; it does not prove all file compositions impossible.

Three controls at `output/image-percent-padding-row-controls-r1` distinguish this mechanism:

- Content-box height80 with percentage padding only top/bottom: all8 geometry/pixel/presence frames pass. Desired width and early basis both remain120.
- Border-box height120 with vertical5% and horizontal7.5% for a96:64 asset: all8 geometry/presence and6 pixel frames pass.
- Both-auto row stretch with that balanced padding: all8 geometry/presence and6 pixel frames pass.

For the last two, horizontal padding equals natural ratio times vertical padding in real arithmetic, so the missing terms cancel in the desired outer width until the padding floor applies. Their four pixel failures remain. All nine distinct full pairs were inspected at original resolution;15 exact original/clone/restored transfers cover24 frames. See `output/image-percent-padding-row-controls-r1/visual-r1/review-receipt.json`. These special cases are diagnostic controls, not a replacement for the general feature.

A separate deliberate wrong cross-stretch candidate at `output/image-percent-padding-stretch-control-r1` changes synthetic outer Auto to Fill for authored definite cross axes. All16 geometry/pixel checks fail while all16 presence checks pass. All six distinct pairs are inspected, with ten exact transfers. This confirms that image visibility alone cannot validate layout and that synthetic Auto must preserve authored definite cross sizing.

## Remaining work

The independent 208-frame Chrome characterization is verified by `validation/image-percent-padding-chrome-evidence.py`: 672 artifact bindings, 130 exact original/clone/restored repeats, and all 192 corresponding native-campaign Chrome PNGs, outer rectangles and image content dimensions match exactly. See `output/image-percent-padding-chrome-r1/verification.json`. This verifies the browser references, not native equivalence. Preserve the failed initial generator attempt at candidate-r1: it used the wrong global-record offset before accounting for asset records preceding the Artboard; candidate-r2 corrects that generator-only error.

Before public integration, implement a shared emitted outer sizing/stretch decision as specified in `image-percent-padding-plan.md`. The final outward-bounded content-plus-padding check and its numerical overflow/finite controls are now implemented; percentage-padding public admission and its future API diagnostic control remain separate. Do not render enormous dimensions. Investigate an ordinary sizing carrier that preserves original parent width during row intrinsic measurement while contributing only the image extent to parent layout. Merely changing packing direction cannot restore a missing parent-width input. Fixed source parent widths can potentially lower percentages to proven source-derived points, but that is a separate finite context and must never substitute browser-baked viewport geometry for responsive support.

General row percentage-padding and content-box percentages plus nonzero same-axis padding remain unresolved. Continue independent public contexts while recording these limitations; do not declare universal impossibility or promote the private generator to public support.
