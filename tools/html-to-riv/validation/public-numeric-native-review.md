# Shared numeric token preservation: native and Chrome review

The frozen numeric compiler repairs the tested ordinary numeric fields without changing the runtime. The realistic matrix retains four fractional paint failures; the large-coordinate matrix retains 32 geometry failures. This is bounded evidence, not complete decimal, percentage, color, or content-box equivalence.

The authoritative summary is [public-numeric-native-receipt.json](public-numeric-native-receipt.json). Its bindings identify the exact frozen CLI/WASM, source snapshots, native receipts, Chrome observations, pixel comparisons, complete visual coverage, and retained failures. The compiler is `output/public-numeric-token-r3/frozen/html-to-riv`, SHA-256 `a77a45eac1d958d728a041a4a82f3d7e67e1685afee4ed1c8c06768cdf629e5b`. Runtime baseline remains `6c7ac16617835b5f581784ff08a9e779bb52faf3`; the unchanged baseline probe and Rust Metal renderer are checked by binary hashes. Chrome is pinned to `153.0.8010.12`, native mode is `RasterOrdering`, scale factor is one, and existing 0.1px geometry and pixel gates are unchanged.

| Corpus | Scenes | Frames | Geometry passes | Pixel passes | Clear passes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Visible numeric consumers | 26 | 208 | 208 | 204 | 416 |
| Original/reduced large-coordinate controls | 10 | 80 | 48 | 80 | 160 |
| Supplemental fractional controls | 2 | 16 | 16 | 4 | 32 |

The first two corpora resize each original and clone through 240×160, 390×200, 768×120, then 240×160. The supplemental exact-half scene uses the same sequence; the historical minimal scene preserves its four 100×80 steps. Every repeated viewport has identical native and Chrome PNGs within its source. Cyan and transparent clear checks prove that the ordinary file supplies the white host. All three native commands terminate with status 1 because failures are retained.

The visible sources cover meaningful decimal widths, adjacent f32 values, heights, padding, minimum/maximum widths, percentage sizing, rem/em and percentage font context, variable aliases/inheritance/fallback, escaped units, inline declarations, content-box points/relative padding, and six color boundaries. They use visible nested paint and following siblings where useful. Lexical forms and strict rejection coverage are separately owned by [public-numeric-token-cases.json](public-numeric-token-cases.json), [public-numeric-token-rejections.json](public-numeric-token-rejections.json), and `tests/numeric-tokens.rs`; this scene matrix does not substitute for those tests. Parent validation records 247 passing Rust tests, 41 passing Node tests, native/WASM builds and strict TypeScript. Of 694 prior outputs, 691 reproduce exactly and three corrected numeric outputs have a separate field/native audit in [public-numeric-regression-receipt.json](public-numeric-regression-receipt.json).

## Large-coordinate findings

The original two sources and eight discriminating controls are copied without HTML/CSS changes from [content-box-rounding-repro-cases.json](content-box-rounding-repro-cases.json) and [content-box-rounding-controls.json](content-box-rounding-controls.json). Their source hashes are preserved separately. Supplemental measurements reuse the already rendered ordinary files, restore native dimensions to f32 after reading their short JSON representation, and capture fresh Chrome Typed OM, ResizeObserver local boxes, and world rectangles. All fresh Chrome rectangles match the original render receipt.

| Child / horizontal padding | Native child width | Chrome local border width | Chrome rectangle width |
| --- | ---: | ---: | ---: |
| Percentage / one million on each side (original, reduced, explicit control) | 1000000 | 999999.8125 | 999999.75 |
| Fixed / one million on each side | 999999.875 | 999999.875 | 999999.875 |
| Auto / one million on each side | 1000000 | 999999.875 | 999999.875 |
| Percentage / zero | 999999.8125 | 999999.8125 | 999999.8125 |
| Fixed / zero | 999999.875 | 999999.875 | 999999.875 |
| Percentage / right one million | 999999.8125 | 999999.8125 | 999999.8125 |
| Percentage / left one million | 999999.8125 | 999999.8125 | 999999.75 |
| Percentage / half million on each side | 999999.8125 | 999999.8125 | 999999.75 |

Fixed widths are repaired. Zero/right-padding percentage controls also match both Chrome local and projected widths. The two-million total padding still loses content precision through the rounded three-million outer width: native auto differs by 0.125px and percentage differs locally by 0.1875px. Left/split padding separately shows a 0.0625px difference between Chrome's local box and projected rectangle. The existing 0.1px gate allows that last projected difference. These are observations about tested inputs; this review does not identify a particular Blink instruction or prove a general percentage arithmetic mismatch.

The large scenes' pixel passes are limited to their captured viewport. Most padded children and far edges are offscreen; the reduced unpainted/zero-height scene is entirely white. These images cannot visually establish the offscreen geometry. The 32 geometry failures remain explicit in `output/public-numeric-native-r1/large/render/receipt.json` and the supplemental `large/measure/receipt.json`.

## Fractional boundary controls

Only `numeric-fractional-height` fails the visible pixel matrix, at frames 0, 3, 4 and 7 (240×160), due to mismatch ratio. Chrome measures `25.484375px` for the source `25.499998092651367px`; its following sibling starts there. Native preserves the f32 source value. This difference remains inside the geometry tolerance.

Replacing only that source number with exact `25.5px` reproduces all eight complete native RGBA images exactly and the same four gate failures. It does **not** reproduce Chrome: every paired Chrome frame differs by 210 pixels. At x=50, y=25, original Chrome is navy `(20,35,63,255)` and exact-half Chrome is green `(102,153,51,255)`; both native sources give partial coverage `(61,94,57,255)`. Chrome measures exactly 25.5 for the replacement and snaps this edge on the opposite side of the threshold. The corresponding sibling edge also differs. The numeric source is therefore related to the known fractional coverage failure, not identical to its browser reference.

The separate historical minimal source `<div id=p></div>` with `#p{height:25.5px;background:#14233f}` reproduces all eight historical Chrome and native images exactly at 100×80. Geometry agrees and all eight pixel comparisons still fail. At x=20, y=25 Chrome is `(20,35,63,255)` and native is `(138,145,159,255)`; at x=50 the native red channel is 137. Full comparisons and coordinate-bound samples are in `fractional/comparisons.json`. The historical [flex-fixed-controls-review.md](flex-fixed-controls-review.md) remains unchanged. This evidence does not prove every possible ordinary-file paint composition impossible.

## Color fields and actual interior pixels

The observer follows the mapped layout node's ordinary Fill and SolidColor ownership to `colorValue`, checking the exact emitted ARGB against the fixture. Prior fields come from the frozen old compiler's admission-only preflight bytes; no old native replay is claimed. For each of all eight original/clone frames, every one of 4,200 pixels in interior rectangle `[10,10,150,40)` is examined. The following histograms are identical across those eight frames; all RGBA values include alpha 255 after painting over the white host.

| Boundary | Emitted ARGB | Chrome interior RGBA (count) | Native interior RGBA (count) |
| --- | --- | --- | --- |
| `rgb(127.49999 0 0)` | `ff7f0000` | `(127,0,0,255)` (4200) | `(127,0,0,255)` (4071); `(128,0,0,255)` (129) |
| `rgb(49.99999% 0% 0%)` | `ff7f0000` | `(127,0,0,255)` (4200) | `(127,0,0,255)` (4071); `(128,0,0,255)` (129) |
| Red alpha `.4999999` | `7fff0000` | `(255,128,128,255)` (4200) | `(255,128,128,255)` (4200) |
| Hue `.11764706deg` | `ffff0100` | `(255,0,0,255)` (4200) | `(255,1,0,255)` (4200) |
| HSL gray `49.99999%` | `ff7f7f7f` | `(127,127,127,255)` (4200) | `(127,127,127,255)` (4071); `(128,128,128,255)` (129) |
| Variable red `127.49999`, alpha `.4999999` | `7f7f0000` | `(191,128,128,255)` (4200) | `(191,128,128,255)` (3411); `(192,128,128,255)` (789) |

CSSOM strings are distinct evidence: Chrome serializes the small hue as green channel 1 and the gray as 128, while their captured pixels are green 0 and gray 127. Alpha serialization is `.498`. The color pixel gates all pass, but the exact one-channel disagreements above remain. Native interior nonuniformity is observed, with no renderer cause asserted or renderer mutation proposed. `color-interiors.json` contains full per-frame histograms and deltas. The initial observer's failed uniform-interior assumption is preserved as `color-interior-initial-failure.json`; recording complete histograms corrects that assumption without changing pixels or gates.

The initial browser-only expectation run retains nine failures across the two alpha cases and gray (three viewports each). Only the expected CSSOM strings were corrected; original `browser/receipt.json` remains, and `browser-r2/receipt.json` records 78 passing observations. This is characterization of serialization, not evidence that a browser string defines rendered color. The old compiler admission preflight remains separately labelled.

## Complete visual review

All 22 complete, unscaled review sheets were directly inspected: visible sheets 0–10 contain 35 distinct source pairs; large sheets 0–9 contain 28; supplemental fractional sheet 0 contains two. Full-frame exact RGBA comparison transfers the review to the remaining 173, 52 and 14 frames respectively. Total coverage is 65 direct pairs plus 239 transfers, accounting for all 304 frames. Transfer is only within the same fixture and bound source/file/map; each complete decoded image must match after explicit white-canvas extension, including every area beyond the smaller viewport. Responsive pictures with changed content therefore require another direct pair. This is not a cropped-region, thumbnail, geometry-only, or perceptual-hash transfer.

Review confirms visible hierarchy, dimensions, responsive inset/width changes, padding, following sibling placement, and color swatches. The fractional horizontal edge differs as described above; subtle native color variation is recorded numerically. Large-scene review establishes only the captured navy/green strip or white region, with the offscreen limitation retained. Each inspected sheet and every source image placement is hash-bound in its `review-sheets.json`; `visual-coverage.json` accounts for every transfer. No new acceptance gate, feature admission, runtime change, or broad equivalence claim is introduced by this validation.
