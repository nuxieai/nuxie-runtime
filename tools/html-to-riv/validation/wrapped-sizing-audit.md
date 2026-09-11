# Wrapped item sizing audit

This is a source audit and proposed validation plan, not new runtime evidence or public admission. The fixed-item line-wrapper experiment demonstrates positional alignment, but its sizing transform must not be generalized by copying the fixture adapter.

## Existing seam and exact references

- `src/compiler.rs:397` (`Emitter::layout_box`) encodes native sizing. At line 413, automatic size becomes Fill only on the native parent's cross axis; other automatic sizes use Hug.
- `src/compiler.rs:470` sorts authored siblings by `(order, DOM index)` before emission. Lines 487–502 transfer the authored main dimension and its min/max bounds to a perpendicular wrapper, then clear the inner main dimension to Auto and bounds to zero/absent. This is the useful existing sizing seam.
- `src/compiler.rs:513` maps the authored object, then line 514 recurses using the original authored `Style`. Keep this separation: synthetic wrappers must not become selector ancestors, inheritance parents, source nodes, or percentage-height admission parents.
- `output/line-alignment-wrappers-r1/pairs.json:40` and `:41` show the fixture transform. It adds wrapper widths of 60/60/40px while leaving the authored fixed widths in place. `adapter.py:4` exact-matches fixtures and patches ordinary alignment/wrap fields. Repeating the same fixed size is harmless here but is not a general percentage/bounds transform.
- `src/baseline.rs:28`, `:74`, and `:85` summarize first baseline, used height, and last baseline. Their packed-column assumptions must not silently apply to a new multiline layout. `src/spacing.rs:27` emits synthetic layout participants; their interaction with native line breaking needs separate admission.

## Concrete risks and required treatment

| Input context | Risk in a naive all-item wrapper | Candidate treatment / decisive test |
| --- | --- | --- |
| Percentage main dimension or main min/max | Keeping the value on both layers resolves it twice against different containing blocks; the inner box can become smaller than its line slot. | Transfer dimension and bounds once to the outer participant; clear inner main size/bounds using the existing compiler seam. Test 40% items with min/max bounds across one/two/three-line resize transitions. |
| Auto main dimension with fixed descendants | The outer participant needs the item's intrinsic main contribution before line assignment; the inner Auto/Fill relationship must still expose that contribution. | Preserve native intrinsic measurement, without browser-baked sizes. Test empty auto items, fixed descendant columns/rows, nested wrapping, and competing min/max bounds. |
| Percentage cross dimension or cross min/max | The original containing block is the flex container; the wrapper's cross extent is a line. Retaining the percentage on the inner box can resolve it against a different extent or an indefinite intrinsic measurement. | Do not generalize the existing single-line percentage proof. Investigate an ordinary-file expression retaining the original containing block; otherwise diagnose this context. Test row height:50% and column width:50%, plus cross min/max percentages, with multiple unequal lines and a definite parent cross size. |
| Auto cross size with auto/normal/stretch alignment | Authored cross becomes the perpendicular wrapper's main axis. `layout_box` then emits Hug, even when `stretch=true`; it does not automatically fill the line extent. | Separate stretch-item lowering from positional alignment. Investigate native main Fill/fractional sizing only with intrinsic-contribution proof. Test empty and content-bearing auto-cross items beside a taller/wider fixed sibling. |
| Auto cross size with positional alignment | Intrinsic cross measurement must contribute to line height/width before positioning, without stretching the authored box. | Test nested fixed descendants and cross min/max bounds, including a bound larger than the parent. Compare both authored geometry and the following footer. |
| Main bounds with conflicting min/max or explicit min:auto | Bounds need to constrain line slots once and preserve existing native minimum semantics. | Test min greater than max, empty/nonempty min:auto, and percentage bounds; compare exact line assignment at the boundary. |

A compiler-owned `ItemLayoutPlan` beside the existing wrapper block could derive outer sizes/bounds, inner sizes/bounds, native directions, item alignment, and optional wrapper emission from an already computed authored style plus the parent's wrapping context. Keep the plan scalar. Choose wrapping/paint ordering independently; do not sort synthetic objects as authored children. Preserve the current no-wrap path's bytes.

A single plan must coordinate any paint-preserving nested-artboard composition with the alignment wrapper: the participant that owns main sizing must remain the actual line-breaking slot. Any additional layout box must not introduce a second percentage containing block or duplicate the intrinsic contribution. The current paint composition uses ForegroundLayoutDrawables on existing layout geometry; this audit does not claim a tested nested-artboard arrangement. That combined composition requires its own same-file resize evidence.

## Minimal follow-up matrix

Use both axes, wrap/wrap-reverse, and reverse main directions where the combined paint candidate permits them. Include positional start/center/end plus auto/stretch; one mixed-order case must combine negative order, equal-order DOM ties, child/sibling selectors, inheritance, and source-map verification. Resize the original and clone through exact-fit, one-pixel-overflow, and return states. Preserve geometry and native/Chrome pixels separately.

Prioritize three discriminating fixtures: (1) percentage main size with a clamp, (2) intrinsic-auto main size with fixed descendants, and (3) auto or percentage cross size beside a fixed cross-size sibling. These test actual sizing contracts missing from fixed-item demonstrations. Cross-percentage and stretch questions remain unresolved compositions, not demonstrated immutable-runtime impossibilities.

## Preserved counterexamples

`output/wrapped-sizing-counterexamples-r1` now exercises eight exact scenes through the existing all-item wrapper adapter and the reconstructed, hash-bound ordinary-file augmenter. The adapter uses the original fixed-main-size transfer unchanged; this run tests cross sizing only. See `wrapped-sizing-counterexamples-receipt.json` and the output experiment manifest, which override the raw driver's public labels.

| Profile | Geometry / pixel results |
| --- | --- |
| Row and column fixed controls | 8/8 each scene |
| Row and column auto-cross stretch | 4/8 each scene |
| Column cross width:50% | 0/8 |
| Column cross min-width:50% | 0/8 |
| Row cross height:50% / min-height:50% | Both reject during transformed compilation; no native frames |

Six scenes rendered 48 original/clone frames: 24 geometry passes, 24 pixel passes, 96 passing clear controls. All six RIV files and parsed source maps reproduce exactly. Each scene uses viewport sequence 400×400, 200×200, 120×120, 400×400, including line reflow. The source, copied reconstructed augmenter, original reconstruction receipt, adapter, cases, diagnostics, and raw result hashes are preserved.

At 400×400, the empty auto-cross stretch item has native cross size 0 instead of Chrome's 30px, so the coral item disappears. These failures recur at frames 0/3/4/7, while narrower layouts placing it on its own zero-height/width line agree. Column percentage cross size gives native width 15 instead of 60px at frame 0 and 0 instead of 60px at frames 1/2. The min-width percentage gives 15px initially and 10px at narrower sizes, instead of 60px; sibling positions also differ. This directly demonstrates changed containing-block/intrinsic behavior for this candidate.

For row height/min-height percentages, the existing compiler rejects the introduced auto-height wrapper context with `unsupported-target-semantics`. This is a compiler admission boundary, not a native failure or runtime impossibility. Separate pinned-Chrome captures show the original authored item at height 60px in its definite 120px-high container. Exact diagnostic logs and these two Chrome-only references are retained outside the render corpus so the native run completes.

All six frame-0 Chrome/native pairs were directly inspected on `visual-0.png` and `visual-3.png`; the two diagnostic Chrome-only images were also inspected. The visual differences agree with the geometry findings. The other 42 native frames have automated evidence only. These counterexamples justify separate cross-percentage and auto-stretch handling before generalizing the successful fixed positional wrapper.

## Empty auto-cross stretch bypass candidate

`output/wrapped-auto-stretch-bypass-r1` preserves a successful follow-up to the auto-stretch counterexample. Only the empty authored item with cross-size `auto` and `align-self:stretch` remains a direct child of the native wrapped container. The fixed center/end siblings retain their perpendicular wrappers. This leaves the existing native cross-Fill behavior available for the stretch item while wrapper main-axis alignment keeps the other items independent of line alignment.

Twelve exact scenes cross row/column, wrap/wrap-reverse, and line start/center/end. All 96 original/clone frames pass geometry and native/Chrome pixel gates; all 192 clear controls pass. All twelve ordinary files and parsed source maps reproduce exactly. The original failed all-item wrapper candidate remains preserved. The new experimental manifest and `wrapped-auto-stretch-bypass-receipt.json` bind sources, reconstructed augmenter, compiler, fixtures and evidence, overriding the driver's public labels.

All 36 distinct Chrome/native pairs (frames 0, 1, and 2 for every scene) were directly inspected on `visual-{0,3,6,9}.png` and `visual-additional-{0,4,8,12,16,20}.png`, with no visible divergence. The coral item fills its shared line's cross extent and collapses consistently when alone on an empty-cross line. All 60 remaining original/clone return frames were proven exact decoded RGBA matches to reviewed frames independently for Chrome and native. `visual-direct.json` and `visual-transfer.json` bind this full 96-frame visual coverage. No crop, extension, or tolerance adjustment was used.

This establishes the explicit empty-auto stretch candidate, not blanket public support. Authored auto/normal alignment equivalents, intrinsic descendants, cross min/max bounds, and integration with the nested subtree paint composition need their own coverage. In particular, a fixed cross-size item with `align-self:stretch` does not perform the same stretch and cannot bypass the wrapper solely because of its alignment keyword. Cross percentages remain unresolved by this change.
