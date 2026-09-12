# Image padding: R2 percentage repair and finite numeric review

The current repair fixes an admitted compiler sizing error that the earlier R1 audit missed. R1 copied a content-box percentage into both the outer and inner owners. R2 preserves only point dimensions on the inner owner and uses 100% for a percentage already resolved by the outer owner. No additional reachable correctness hole was found in this bounded source pass. That is not a substitute for the parent's new R2 native/Chrome, API, regression and visual evidence.

This follow-up changed only this review and [image-padding-plan.md](image-padding-plan.md). It ran no compiler builds, tests, new compile campaigns or native/browser captures. The prior R1 hashes and eight compile-only observations remain below, labeled historical.

## Confirmed missed bug

`box_sizing::lower` works per axis. A content-box percentage requires a mixed percentage-plus-points encoding only when that **same axis** has a nonzero computed padding sum. Therefore both of these are admitted:

- `width:50%; padding:8px 0`: horizontal padding is zero, so the outer width resolves 50% against its original containing content width.
- `height:50%; padding:0 10px`: vertical padding is zero, so the outer height resolves 50% against its original containing content height, subject to the existing definite-height checks.

R1's `ContentBox && !auto[axis]` branch copied the authored 50% onto the inner owner as well. It then resolved against the already halved outer content dimension. This was a semantic double application, not a small float discrepancy or an immutable-runtime limitation. The earlier review incorrectly grouped percentage dimensions with point dimensions and described the mixed-size diagnostic too broadly.

The parent preserved actual R1 evidence in [public-image-padding-percent-before-r1/receipt.json](../output/public-image-padding-percent-before-r1/receipt.json): 16 frames, zero geometry/pixel/presence passes and zero capture errors. I read that receipt and the two saved first-frame results; no render was repeated. At 240×240 with the 96×64 asset:

| Source | Correct Chrome outer size | R1 native outer size | Mechanism |
| --- | --- | --- | --- |
| 50% width, vertical 8px padding | 120×96 | 120×56 | Outer width120, incorrect inner width60, ratio height40 plus16 padding |
| 50% height, horizontal 10px padding | 200×120 | 110×120 | Outer height120, incorrect inner height60, ratio width90 plus20 padding |

The two sources are retained in [public-image-padding-percent-cases.json](public-image-padding-percent-cases.json). Their successful-compilation/determinism tests alone would also pass the old wrong compiler; the new geometry, content-pixel/presence and visual checks are the distinguishing regression evidence.

## R2 source identity and implementation check

Current sources were rehashed and byte-compared with `output/public-image-padding-build-r2/frozen/inputs/src`. All five checked files match:

| File | R2 SHA-256 |
| --- | --- |
| `images.rs` | `d90efbc8d011562218bc60b5282dbbfd1b937fa0979510d2d793445c6bcb221c` |
| `compiler.rs` | `2c53f93ce31f34043f782780c17fcbb1477e9f2851124716555b78a6a55d9146` |
| `numeric.rs` | `d093c82e0f59e9978b0d073ef335ef8346893b9ddf7d4a2a257a3052cb510a8e` |
| `box_sizing.rs` | `3ebdbbee9f1821558ccad324cea593f2387857039f7a6ed692ff97cebf82754b` |
| `flex_descriptor.rs` | `65b06dd62b620f4e900949dda76d378cf4632d69c1d5dde6faded0bc3c7ab919` |

The repair relative to frozen R1 changes only the branch in `images::padded_plan`: copy an authored inner dimension only for `BoxSizing::ContentBox` **and** `Size::Pixels(_)`. An authored `Size::Percent(_)` now falls through to `Size::Percent(100.)`. The existing ratio-derived auto axis remains auto, and the intrinsic both-auto/non-stretch case still uses validated asset point dimensions. The repaired CLI/WASM identities and qualification state belong to the frozen build and [restart brief](../GOAL-RESTART.md), not to the older audit hashes below.

The containing percentage is still applied once on the outer owner; the inner consumes that resulting content dimension. In native arithmetic, 100% is another multiply-then-scale expression, not a claim of bit-exact identity. The focused exponent guard models this actual expression. R2 does not reinterpret the inner 100% as an authored source coefficient.

## Reachable combinations checked in source

| Admitted shape | Outer and inner behavior after repair |
| --- | --- |
| Percentage width, auto height, non-stretch; zero horizontal padding | Outer resolves width; inner width100%; auto content height derives from the ratio; vertical padding remains outside |
| Percentage height, auto width, non-stretch; zero vertical padding | Mirrored calculation, with definite containing height still required |
| Percentage cross axis, automatic main axis, ordinary stretch alignment | Explicit percentage cross size still controls; inner cross fills it and ratio determines the automatic main size |
| Percentage main axis, automatic stretched cross axis | Inner dimensions resolve independently; ratio stays disabled so it cannot undo cross stretch |
| Percentage axis plus fixed point other axis | Percentage becomes inner 100%; the other content-box point stays its exact authored point; ratio stays disabled |
| 0% or 100% controlled axis | They remain explicit percentages, not auto/intrinsic sizes. The new inner 100% does not introduce an intrinsic fallback |
| Inherited or variable-resolved percentage | Computed style retains the original percentage; only the emitted inner plan uses100%. No inner value is copied back into Style |
| Border-box percentage, points, intrinsic dimensions or both-auto stretch | The changed branch does not alter their existing plan selection |

At least one padding axis is nonzero in this padded branch. If both content-box dimensions are percentages, at least one corresponding axis therefore encounters the existing mixed-size rejection (apart from the already documented zero-rounded inset/provenance case). Neither same-axis percentage-plus-point padding nor percentage padding becomes newly supported.

The numeric order remains `child_with_sizing(outer)` → `content_owner(actual inner)` → `image(ratio/fit)`. `Bounds::content_owner` reads the same `owner.sizes` and `owner.packing` used by `layout_box`; the corrected 100% is therefore reflected in both guard and emission. Authored percentage height inside an authored auto-height parent still diagnoses before emission. Unknown containing cross bounds remain unknown through the synthetic100%, then fail the existing image bound requirement. Fixed inner points still recover positive content sizes lost by rounded outer-padding cancellation.

The source-specific guards for own min/max, automatic margins, baseline/alignment wrappers, percentage padding and public nonlegacy flex are unchanged. They are existing admission boundaries, not newly discovered defects and not evidence of impossibility. The descriptor limitation described below remains latent: `img` is void and has no authored child-list Group; synthetic compositions and padded items remain outside arithmetic certificates.

## Limits of this follow-up

No additional production change is requested from this source pass. Percentage factors other than 50%, mixed fixed/percentage axes, reverse parent directions and stretched-auto variants are covered here by branch reasoning, not new native captures. Existing extra100% rounding, ratio reciprocal/division and world-origin precision concerns remain limited to their documented scope below. They do not justify treating every image as unsupported or claiming every admitted extreme is visually proven.

The parent's required R2 evidence must show the two changed files fix all 16 original/clone frames, bind API parity, preserve the existing finite failures, and establish exact output identity before transferring previous native/visual evidence. The historical eight controls below were not rerun by this follow-up. Their hashes must not be presented as new R2 evidence.

## Historical R1 audit and compile-only observations

The earlier R1 review concluded that it had found no new reachable numeric-admission or inherited-style defect. **That review missed the doubled content-box percentage bug described above.** Its eight compile-only controls still produced the recorded five successes and three diagnostics, but none exercised a content-box percentage with only opposite-axis padding. The percentage-chain controls used border-box 100%, which would not distinguish a repeated unit coefficient anyway. They are retained below as finite historical observations, not qualification of R1 or R2 geometry.

Reviewed working tree HEAD `244733b9dc0bdeebe48b6a3856a5ea1d7b5f7cb0`, with the candidate edits frozen in `output/public-image-padding-candidate-r1`. The following current source files were rehashed and matched that candidate's build bindings:

| File | SHA-256 |
| --- | --- |
| `src/images.rs` | `6f0f4244cac11d8557c2339a83272deeb304dc422a0f0d191b752c4e3984f7fd` |
| `src/compiler.rs` | `2c53f93ce31f34043f782780c17fcbb1477e9f2851124716555b78a6a55d9146` |
| `src/numeric.rs` | `d093c82e0f59e9978b0d073ef335ef8346893b9ddf7d4a2a257a3052cb510a8e` |
| `src/box_sizing.rs` | `3ebdbbee9f1821558ccad324cea593f2387857039f7a6ed692ff97cebf82754b` |
| `src/flex_descriptor.rs` | `65b06dd62b620f4e900949dda76d378cf4632d69c1d5dde6faded0bc3c7ab919` |

The frozen CLI SHA-256 is `6259438f62db200888e875145726d0ffb94b032685e2e53c281362c9154951c9`. The runtime remains the target documented in [TARGET.md](../TARGET.md).

### What the source establishes

`images::padded_plan` reads an immutable `Style` reference. It leaves computed widths, bounds, padding, inheritance and numeric provenance intact. The existing zero-padding image path still has its earlier intrinsic/stretch adjustments; the new padded path returns before those mutations. `box_sizing::lower` separately translates fixed content-box dimensions and bounds into outer dimensions, retaining source-number provenance.

The image's source identity, padding and background remain on the outer ordinary LayoutComponent. The unpainted inner LayoutComponent has zero padding and zero minimums; the Image and its ratio/clip attach to that inner object. `compiler.rs:757–787` chooses the image-specific owner before the general content-box owner, so content-box images receive one synthetic content owner rather than nested duplicate wrappers. The outer padding is emitted once. Its background fill is emitted before the inner object/Image, so it covers the padded border box while image fitting uses the inner content box.

The R1 review covered the following point/auto cases. Its earlier general statement that the sizing cases were internally consistent was too broad: it omitted the admitted percentage case now repaired in R2.

- With both axes auto and non-stretch alignment, the inner gets the encoded intrinsic point dimensions. Outer auto measurement can include those dimensions plus padding. No ratio inference is needed.
- With one ratio-derived auto axis, outer packing uses that axis as its main axis. The inner therefore hugs the derived axis rather than accidentally receiving cross-axis Fill/stretch there. Its other axis is the original fixed content-box dimension or 100% of the outer content box for border-box sizing.
- With no ratio-derived axis, packing follows the parent's physical row/column axis. An automatic cross axis remains a stretch case; fixed content-box sizes remain fixed on the inner.
- For both-auto stretch, `adjust_outer` replaces only the emitted outer cross size with 100%. This occurs after normal content-box lowering, so this synthetic border-box percentage does not accidentally enter the authored content-box percentage-plus-padding rejection. Its numeric carrier is an exact constant 100 on `Lowered`, not a changed inherited `Style`.

That last transformation requires a bounded containing cross dimension. It does not manufacture one: `Bounds::child_with_sizing` resolves the outer and subtracts its padding; `Bounds::content_owner` evaluates the actual inner sizes with zero padding, no flexible sizing, and the actual packing direction; only then does `Bounds::image` infer the ratio axis and require finite known bounds. Unknown cross bounds remain unknown through 100%, and the image guard diagnoses them. The two unknown-parent controls below confirm that public candidate behavior.

The order also preserves a positive fixed content dimension when rounded outer addition and padding subtraction erase it. The fixed inner is evaluated from its own point size, independent of the possibly zero outer content estimate. This matches the purpose of the earlier content-owner guard. An intrinsic image can similarly provide finite inner sizes even when the auto outer's preliminary dimensions are unknown.

The immutable Taffy source agrees with the relevant division of work: `vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs:175–211` resolves dimensions, applies aspect ratio and clamps/floors by padding; its flex-basis preparation at lines 699–713 supplies a definite stretched cross size when available. The zero-padding inner avoids applying an image-content ratio to the padded outer border box. Source agreement alone does not establish pixel agreement through all float rounding and measurement phases.

### Provenance and descriptor boundaries

The outer synthetic 100% has correct exact emitted-coefficient provenance. It must continue to be interpreted as a compiler-owned emitted coefficient, not as evidence that the author specified 100%. `capture_item` uses the adjusted `Lowered` metadata, and padded items remain outside the current arithmetic model.

There is a **latent descriptor API limitation**, not a demonstrated public bug: `flex_descriptor::Parent::extract_content_owner` pairs `owner.sizes` with the authored style's numeric widths/heights. That was valid for the original general content owner; an image-specific owner can instead contain intrinsic points or synthetic 100%. Today `img` is an HTML void element and produces no authored child-list Group, so this mismatched extraction is not reached for image owners. The scene index still sees emitted objects; descriptor Groups are not a complete enumeration of every synthetic native group. If image owner groups or replaced-element descendants are ever introduced, this extractor must receive the actual inner numeric carriers, or explicitly unresolved carriers, instead of cloning authored carriers.

The current hard `content_owner` barriers in `flex_sizes`, `flex_world` and `flex_proof`, plus padded participant/local-fact barriers, remain necessary. This review does not authorize removing them or claiming a composition certificate. `Bounds` is a focused exponent guard and does not consume exact scalar provenance or prove browser-level rounding fidelity.

### Eight compile-only controls

Each request used `{width:390,height:320}` and `assets.picture = {kind:"image",bytes:[...exact fixture bytes...]}` from `fixtures/images/ordinary-r1/opaque.png`, SHA-256 `ba8108f30fabb47e6b6220fcc6f5d7cb56a393b71522b02bba3beca85eb9f682`. Commands used the frozen CLI as `html-to-riv request.json result.riv`, with a 10-second per-process timeout. Each was run once in a temporary directory; successful bytes are identified below. Failed cases returned exit 1, one structured diagnostic, empty stdout, and neither output nor map. Successful cases returned exit 0, empty stderr/stdout, and both files. No native rendering was performed.

For the first three and the inheritance control, HTML is `<div id=p><img id=i src=picture></div>`. For both cancellation controls, HTML is `<img id=i src=picture>`. For chain N, HTML is N opening `<div>` tags, then `<img id=i src=picture>`, then N closing `</div>` tags.

| Control | Exact CSS | Result |
| --- | --- | --- |
| unknown-row-stretch | `#p{width:200px;flex-direction:row}#i{padding:8px}` | Bounded-containing-dimensions diagnostic at `/0/0` |
| unknown-column-stretch | `#p{height:200px;align-self:flex-start}#i{padding:8px}` | Same diagnostic at `/0/0` |
| intrinsic-unknown-parent | `#p{flex-direction:row;align-self:flex-start}#i{padding:8px;align-self:flex-start}` | Compile |
| inner-small-point-cancellation | `#i{box-sizing:content-box;width:.03125px;height:.03125px;padding:0 1000000px;align-self:flex-start}` | Compile |
| inner-large-point-cancellation | `#i{box-sizing:content-box;width:999999.875px;height:.03125px;padding:1000000px;align-self:flex-start}` | Compile |
| inherited-width-padding | `#p{box-sizing:content-box;width:100px;height:80px;padding:10px}#i{box-sizing:inherit;width:inherit;height:20px;padding:inherit;align-self:flex-start}` | Compile |
| percentage-chain-8 | `div{width:1000000%;height:100px}#i{width:100%;padding:1px}` | Compile |
| percentage-chain-9 | Same CSS | Percentage-overflow diagnostic at `/0/0/0/0/0/0/0/0/0` |

The diagnostic code for all three failures is `unsupported-target-semantics`. The two unknown-parent messages are `Image sizing requires bounded containing dimensions; intrinsic flex-container cycles need separate qualification`. The chain-nine message is `Resolved percentage size may exceed finite binary32 geometry within the supported viewport domain (0,16384], after native min/max clamping`.

| Successful control | Rive SHA-256 |
| --- | --- |
| intrinsic-unknown-parent | `10bc1372b9bbfa6b9fd2313c3667f85f65df348d369569f23c15b28a7a8458be` |
| inner-small-point-cancellation | `04a86d8dd4a5180f8cd8d6f100754edd4b320bb6d0c8647ee508497016fa4671` |
| inner-large-point-cancellation | `617d173a5089997f5e9651ec273097773412997158f37a256381ce8885f72a5b` |
| inherited-width-padding | `4bf95cb5d91043dc65c58b6f898d3befeeec40815fcb5f5b58aeb16c9f0dc5eb` |
| percentage-chain-8 | `0eccfa4f8f1fa7ca45d2b1cc40819b620a95bde5b7077c2095b6c7b23ddc1413` |

These accepted extreme controls demonstrate guard behavior only. They do not assert renderable giant dimensions, browser/native pixel equivalence, or resolved ancestor world-position error.

### Remaining qualification and narrowly scoped checks

The parent-owned candidate native receipt already contains tail-local RGB failures for width-driven border-box cases. This review does not reinterpret those failures as successes, and no numeric-source argument supersedes their actual pixels. Fractional content heights, following-sibling origins, same-file resize/clone behavior, zero-area image-presence semantics, and asymmetric padding need the parent's existing finite visual/metric workflow.

Additional source concerns are bounded and remain distinct from confirmed defects:

1. Synthetic 100% adds another native multiply-then-scale operation. It is responsive but not an exact identity at every binary32 size. Ordinary fractional width/height and point-cancellation controls should preserve and measure that effect; the guard is not a precision certificate.
2. `Bounds::image` computes its height envelope with multiplication by a rounded reciprocal, whereas Taffy's direct aspect-ratio path divides by the ratio (`geometry.rs:591–596`). This predates the padding change. No missed-overflow public counterexample was established here; a focused bound test against native division near representable boundaries would be needed before calling it a proven error envelope. This is not evidence to reject all padded images or to change the runtime.
3. Percentage padding, content-box percentage dimensions with a nonzero computed padding sum on that same axis, image min/max constraints, margins and alignment wrappers remain specifically rejected. Opposite-axis padding alone does not cause that percentage-dimension rejection; the R1 review failed to cover this admitted distinction. Candidate flex resizing or a future ratio-plus-clamp composition needs its own finite bound view rather than reusing assumptions from the legacy zero-flex inner.
4. Large padding can preserve local inner point dimensions while losing world-origin precision. The two successful cancellation controls above must not be promoted to broad visual qualification from CLI success alone.

No production, runtime, dependency, helper, case or shared progress file was edited by this review.
