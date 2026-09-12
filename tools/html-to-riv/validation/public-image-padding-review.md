# Public image point padding

Point padding lowers through ordinary objects stored in the Rive file. The authored outer object owns padding and background; an unpadded inner object owns image content sizing, the intrinsic ratio, fitting and clipping. The runtime, renderer, format, shared dependencies and build configuration remain unchanged. The final R2 checkpoint includes the percentage-sizing repair and complete public/native/visual evidence for its finite matrix. Retained pixel failures and broader image-layout contexts remain explicit; this checkpoint does not qualify arbitrary browser equivalence.

Final review found an admitted context outside the original R1 matrix: with content-box `width:50%; padding:8px 0` or `height:50%; padding:0 10px`, the same-axis padding is zero. R1 applied the percentage twice, producing 16/16 geometry, pixel and presence failures. R2 preserves exact point content dimensions and uses 100% of the already-resolved outer content dimension for authored percentages. The repaired files pass all 16 original/clone resize frames against Chrome, with six directly inspected pairs and ten exact transfers. The original failures remain preserved; [before/after gallery](../output/public-image-padding-percent-r2/visual-r2/gallery.html).

## Admitted contexts and remaining work

The 24-case [padding corpus](public-image-padding-cases.json) has 22 intended scenes and two retained diagnostics. It covers intrinsic auto sizing, explicit dimensions, both box-sizing modes, automatic stretch, definite percentage dimensions in border-box sizing, contain/cover, different source ratios and padding larger than the declared outer size. Padding is in fixed points. Original encoded assets are embedded unchanged; source-map identities continue to identify authored outer objects.

Percentage padding and content-box percentage dimensions with nonzero padding on the same axis remain diagnostic. Content-box percentages with padding only on the other axis are admitted, with the two responsive [percentage controls](public-image-padding-percent-cases.json) qualifying the R2 repair. Own min/max constraints, automatic margins, alignment wrappers/baselines and the existing unresolved flex/intrinsic contexts also remain diagnostic. These are unfinished compiler contexts; the current implementation does not prove them impossible. See the [composition plan](image-padding-plan.md) and [numeric review](image-padding-numeric-review.md). The initial numeric audit missed the percentage bug; its finite scope and subsequent correction remain documented.

The additional [two alpha controls](public-image-padding-alpha-cases.json) exercise the new inner clip and outer background with contain over an opaque background and cover over a translucent background. They use the same original alpha PNG already admitted by the compiler. The existing no-padding JPEG/ratio scenes remain unchanged; this is not another decoder qualification campaign.

## Final build and interface evidence

The final frozen build is `output/public-image-padding-build-r2/frozen`. It passes **311 Rust tests, 56 Node tests, strict TypeScript, native/WASM builds and the immutable source guard**. CLI SHA-256 is `9c93c9cbf702db0ec976e93bfa28b6181ecb752c324bcab5821c37629a7b9b0d`; WASM is `ff655118a9f46be7b7347f32f1a1b7a631c20c6719bb416753772af794b7e270`. The build binds 253 source/artifact files. R1 remains preserved separately and cannot certify a later build without exact output and evidence checks.

| Corpus | Requests | Successful scenes | Exact raw ABI / public JS observations |
| --- | ---: | ---: | ---: |
| Point padding | 24 | 22 | 48 |
| Existing public images, with padding rejection migrated | 40 | 28 | 80 |
| JPEG | 56 | 34 | 112 |
| Additional intrinsic ratios | 12 | 12 | 24 |
| Padding with alpha | 2 | 2 | 4 |
| Content-box percentages with opposite-axis padding | 2 | 2 | 4 |

All **136 requests / 272 raw ABI and public JS comparisons** match CLI results. All **794 historical outputs** retain exact Rive and source-map bytes. The five R1 corpora retain all 134 requests, diagnostic outcomes and 98 successful files/maps. The earlier 27 image, 34 JPEG and 12 ratio scenes also have direct exact request/file/map checks against their pre-padding references. The original `image-padding` request was deliberately migrated from a diagnostic to a successful scene; its HTML/CSS is unchanged, and `priorExpected` preserves the former rejection. See `output/public-image-padding-{layout,regression,jpeg,aspect,alpha,percent}-r2`, the corresponding parity receipts and `output/public-transport-malformed-padding-regression-r2/receipt.json`.

The two new public Rust tests exercise the 26 padding admission controls, repeat compilation for deterministic results, verify unchanged embedded asset bytes and preserve authored parent/image/following-sibling source identities. Actual native/Chrome tests demonstrate the percentage repair; successful compilation or API parity alone would not catch that geometry bug.

## Preserved initial native result

`output/public-image-padding-candidate-r1/native-receipt.json` records **176 geometry, 152 pixel and 148 presence passes from 176 frames**, with no native/browser process errors. Each emitted scene is imported once, cloned once and resized through 240×240 → 390×320 → 768×560 → 240×240 without recompilation. The native tools and mode are the unchanged baseline Rust Metal RasterOrdering implementation; the browser reference is Chrome 153.0.8010.12.

All 24 pixel failures belong to the small following sibling in three cases: `padding-width-border`, `padding-percent-width-border` and `padding-odd-responsive-percent-width`. Their first-frame tail positions differ by less than 0.011 CSS px, but the local RGB error is above 10. Inspection of the complete first-frame pairs and enlarged crops shows native partial-coverage bands at the top/bottom of the tail where Chrome paints hard edges in these controls. This is evidence of the observed paint difference, not a proof that all fractional layouts or all ordinary-file alternatives fail. The bound crops are in `output/public-image-padding-edge-review-r1/receipt.json`.

The original presence check samples the padded border box with floor/ceil boundaries. In 12 frames it includes differing coverage from the following sibling. Another 16 presence failures have zero reference/native ink because the content box is intentionally empty. Those 28 failures are preserved unchanged. Correcting the observation region and distinguishing blank content from missing image paint must be validated separately; it cannot remove the existing whole-frame, outer-box or sibling pixel gates.

## Native and visual evidence

The [content-aware validation helper](image-content-review.md) observes Chrome's content box using ResizeObserver, records the flat solid background under the image and samples fully contained content pixels. An extra local image-content RGB gate supplements the unchanged whole-frame, outer and sibling gates. Empty content has an explicit blank-paint expectation over a known opaque image background. Ambiguous backgrounds, unsupported observation geometry and missing samples fail the validation control; they are not silently treated as passing compiler behavior. Seventeen finite helper tests cover missing/shifted ink, zero-area content, alpha backgrounds, sibling contamination and preservation of original pixel failures.

The corrected reference capture reuses the exact original native PNGs, with the original tools/reset, recording streams, geometry and raw result identities checked. `output/public-image-padding-candidate-r1/content-preservation-receipt.json` verifies all 176 Chrome images are also unchanged and match the independent Chrome capture. All original pixel metrics/failures and all 28 old presence failures remain recorded. New content coordinates match the independent content-box measurements exactly.

| Native corpus | Frames | Geometry | Pixels | Content presence |
| --- | ---: | ---: | ---: | ---: |
| Point padding | 176 | 176 | 152 | 176 |
| Additional alpha contain/cover | 16 | 16 | 16 | 16 |
| Migrated historical padding request | 8 | 8 | 8 | 8 |
| Repaired content-box percentage dimensions | 16 | 16 | 16 | 16 |

The 24 tail pixel failures are unchanged. No threshold was widened. The default public-image regression combines its eight padding frames with 216 verified historical frames: 224 geometry/presence and 206 pixel passes, preserving all 18 earlier pixel failures. R2 transfers all 416 unchanged main-padding, default-image and alpha frames only after exact file/map/request/tool/reset and raw-result checks; only the two changed percentage files require 16 fresh native frames. The transfer helper resolves nested combined receipts back to actual raw captures; 12 finite controls include eight rejected corrupt provenance cases. See [transfer validation](public-image-transfer-review.md).

Complete visual coverage for padding, alpha, the migrated request and the percentage repair contains **33 directly inspected full pairs and 183 exact RGBA transfers across 216 frames**. The first three reviews were completed for R1; exact source/file/map and actual frame-result equality transfers them to R2 without repeating unchanged inspection:

- [Point-padding gallery](../output/public-image-padding-layout-r1/visual-content-r2/gallery.html): 24 pairs on eight unscaled sheets, 152 exact transfers and all 24 failures visible.
- [Alpha gallery](../output/public-image-padding-alpha-r1/visual-content-r2/gallery.html): two pairs and 14 exact transfers.
- [Migrated regression gallery](../output/public-image-padding-migration-r1/visual-content-r2/gallery.html): one pair and seven exact transfers.
- [Percentage repair gallery](../output/public-image-padding-percent-r2/visual-r2/gallery.html): six pairs, ten exact transfers and two directly reviewed before/after triples. All 16 before/after comparisons preserve identical authored requests, source maps and Chrome measurements/pixels. The 16 old-compiler failures and their separate completed visual review remain unchanged.

Some resized views change recorded ancestor-background rectangle sizes while the image, all other measurements and complete screenshot pixels remain identical. The visual helper permits only those background-chain `borderBox` metadata differences, records their exact source/target values and retains each frame's own gates. It never transfers geometry or gate outcomes. Eighteen projection controls confirm that changed image geometry, presence, colors and other metadata still prevent reuse. The initial redundant preparations remain preserved under `visual`; completed reviews are under `visual-content-r2`. The main gallery's images, local links, search and result filters were checked in pinned Chrome without errors.

Run `python3 validation/public-image-padding-evidence.py` from the module to verify the final R2 source/build, all six corpora, actual native/raw artifacts, exact historical outputs, visual transfers and retained failures. It explicitly checks that the new percentage controls pass all 16 frames and their original failing controls stay failed. The compact [checkpoint receipt](public-image-padding-receipt.json) binds the final verification. The verifier does not compile or render scenes; the previous R1 verifier/output remain preserved separately.

The 99-item counts remain **13 qualified, 23 partial, four investigating and 59 pending**. Image constraints and percentage-padding compositions remain next work. Text paint, broader flex/painting/positioning, responsive expressions and compiler quality remain in the full goal; unresolved is not impossible.
