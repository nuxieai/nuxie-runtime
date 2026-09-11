# Flex directions and sibling paint order — investigation

The immutable runtime supports ordinary direction enums and intrinsic/stretch sizing. The current compiler candidate admits all four flex directions, keeps CSS initial/unset as row and the authoring reset as column, and computes inheritance before lowering. Auto main-axis dimensions use Hug; auto cross-axis dimensions use Fill. These are candidates under validation, not full L01 qualification.

## Evidence before paint-order compensation

`output/public-flex-direction-r1/receipt.json`: 14 scenes, 112/112 geometry frames pass, 104/112 pixel frames pass, 224/224 canvas-clear checks pass. Chrome153.0.8010.12 and immutable native rust-metal RasterOrdering; same original/clone scenes across four resize steps. Rust tests pass; all15 Node transport tests pass after WASM rebuild; TypeScript and immutable source guard pass.

The only failing scene is `row-auto-minmax`. An overflowing coral grandchild overlaps the later gold sibling over30×30pixels. Native paints coral on top; Chrome paints gold. Geometry agrees. All8 original/clone frames fail, without changing tolerances. The two full first-frame images and all14 first-frame pairs were directly inspected (review-0.png and review-7.png). This does not certify other viewport images as visually reviewed.

`output/public-flex-regression-r1/receipt.json`: 46 existing scenes,368/368 geometry frames pass and366/368 pixel frames pass. Only the previously recorded minmax-percent-height fractional-edge failure remains (frames2and6). This provides evidence that cross-axis Fill replaces the old100% encoding without new failures in this corpus; it is not blanket proof for indefinite percentage contexts.

The pre-compensation compiler, WASM, compiler source and failed flex receipt are retained with hashes in `output/flex-before-paint-order-r1/manifest.json`. The original run directories retain their detailed artifacts.

## Ordinary-file composition candidate

Baseline draw traversal reverses file sibling order. Candidate: emit sibling subtrees in reverse DOM order, toggle each emitted native flex direction, and use native main-axis flex-end alignment to preserve authored layout positions. Apply the same compensation to the host. Keep computed CSS directions, selectors and original DOM identities unchanged. No runtime changes or host-side logic are involved.

`validation/public-flex-paint-order-cases.json` combines the original14 cases with9 overlap cases: all four directions with solid/transparent ancestor backgrounds and overlapping host siblings. Validate the candidate before admitting its paint-order behavior. Reverse overflow, fractional dimensions, auto sizing, source-map identity and all older layout regressions remain required checks.

At this historical checkpoint L01 remained pending. See the final checkpoint below for the current bounded qualification.

## Compensation checkpoint and stronger counterexample

The first compensated23-scene corpus passes184/184 geometry andpixel frames plus368 clearchecks. All30 independent viewport pairs were directly reviewed; exactdecoded image crop/whiteextension proofs cover the other154 pairs. See public-flex-paint-receipt.json and output/public-flex-paint-checkpoint-r1/manifest.json.62 Rust and16 Node tests pass. This is a bounded passing checkpoint, not general paint-order qualification.

The stronger52-scene regression corpus passes416/416 geometry but398/416 pixel frames. Two failures are the existing fractional-height case. Sixteen new failures are true overlap in reversed-direction parents: paint-overlap-row-reverse and paint-overlap-column-reverse. In these Chrome paints the coral descendant over the gold sibling; the uniformly reversed-emission candidate paints gold over coral. The original reverse fixtures overflowed away from the sibling and did not exercise this overlap. Both corpora are preserved. The row-reverse first-frame pair was directly inspected. General paint-order admission remains unresolved; investigate Chrome's direction-dependent painting before altering lowering.

## Direction-dependent target lowering

The revised composition reverses emission only for ordinary CSS row/column parents. Reverse CSS parents keep DOM emission and native reversed flow with start alignment. The host is ordinary column and receives the ordinary-direction compensation. Computed CSS values remain independent of file-order lowering.

| Computed CSS direction | Native direction | Native main alignment | Sibling file order |
| --- | --- | --- | --- |
| column | column-reverse | end | reverse DOM |
| column-reverse | column-reverse | start | DOM |
| row | row-reverse | end | reverse DOM |
| row-reverse | row-reverse | start | DOM |

This targets observed Chrome153.0.8010.12 fragment painting. The pinned Blink [flex algorithm](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/layout/flex/flex_layout_algorithm.cc#1621) reverses line item indices; [fragment painting](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/paint/box_fragment_painter.cc#1164) traverses the resulting children. The [CSS Flexbox specification](https://www.w3.org/TR/css-flexbox-1/#flex-direction-property) says direction reversal should not alter painting order. Therefore this is explicitly a pinned-browser fidelity decision, not a claim of normative CSS equivalence or agreement across browsers. Future order/wrapping/justification features need fresh combined validation.

Twelve intrinsic-percentage/minimum cases pass96 geometry/pixel frames and192clear checks with the earlier compensated candidate. Twelve direct first-frame pairs were inspected;84 remaining pairs have exactdecoded crop/white-extension proofs in output/public-flex-intrinsic-r1/visual-transfer.json. Frozen artifacts are in output/public-flex-intrinsic-checkpoint-r1/manifest.json. This provides no basis for a blanket rejection of percentage widths in intrinsically sized row items. The final combined rerender also includes these cases.

## Final checkpoint

The final direction-dependent composition passes all328 geometry/pixel frames in41feature scenes, covering original/cloned same-file resizing. The full87scene run passes696/696geometry and694/696pixels with1392clear checks. Only minmax-percent-height frames2and6 fail the unchanged fractional-edge gate; those match the previously recorded limitation. No runtime/renderer/dependency/schema changes were made.

All87 scene/map outputs reproduce exactly from the frozen compiler in output/public-flex-final-checkpoint-r1/manifest.json, which retains the failed render status instead of relabeling the whole run a pass.63Rust tests,17Node tests (including all87 corpus transport comparisons), TypeScript and immutable source guard pass. All41feature scenes have complete visual coverage:20 newly inspected pairs and308 exact prior-review/repeat transfers; see output/public-flex-final-r1/final-visual-coverage.json. The46older regression scenes retain numerical results and exact prior-run image comparisons; this does not expand their existing visual qualification.

L01 is qualified within the documented single-line flex:0 0 auto box profile against the pinned Chrome target. Future wrapping, order, grow/shrink, alignment and additional painting semantics require their own combined qualification. Fractional painting remains a separately recorded limitation. See validation/public-flex-final-receipt.json for current receipt/hashes, and the earlier sections for the preserved failed candidates.
