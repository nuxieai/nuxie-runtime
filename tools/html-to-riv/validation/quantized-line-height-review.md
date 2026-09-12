# A08: source-backed line-height composition on the immutable runtime

A private ordinary-file candidate now matches the measured Chrome line boxes and baselines in all 43 candidate cases across 344 original/clone frames. The previous 16px baseline error is repaired. This is a typography prerequisite, not public text admission: only 16 of the complete 360 frames pass the additional ink-region paint checks. All failed controls and paint differences remain preserved.

## Source semantics and ordinary file construction

The [Chromium source audit](line-height-quantization-source-review.md) pins Chrome153.0.8010.12 to commit `971a7443b0c9b0a9b2860529b33331b76077ec62`. The new `examples/quantized-line-height.rs` preserves a unitless source number as f64 through multiplication by100, converts the internal percentage to f32, rounds the receiving font size to1/64px, performs the f32 percentage multiplication/division, and truncates the used height to1/64px. A fixed source pixel height instead rounds to1/64px. The example uses these source operations, not observed CSSOM line-height, positions or line counts.

For the unchanged licensed Roboto fixture, integer ascent/descent are derived from its scaled head/hhea metrics. The signed remaining leading is represented as a raw1/64px integer, divided by2 with truncation toward zero, then floored to a whole pixel. The baseline is integer ascent plus that top leading. This is a bounded font/platform assumption; it does not implement Chromium's general platform metrics, overrides or fallback selection.

The ordinary layout owner supplies top padding B-A and bottom padding L*A/(A+D)-B. The native homogeneous Text intrinsic height is N*L+A*(1-L/(A+D)); the pads give the owner N*L and shift its first baseline to B without observing N. Signed synthetic padding is permitted by the existing runtime; it does not admit negative authored CSS padding. Text/font assets, one black Fill and ordinary layout objects are stored in the self-contained file. No runtime, renderer, schema, font modification, external policy or browser-baked geometry is involved.

The prior [reduced/zero experiment](reduced-line-height-review.md) and its [independent audit](reduced-line-height-checkpoint-review.md) establish the signed-padding mechanism separately. This candidate changes its baseline/height arithmetic while retaining the same class of ordinary objects.

## Discriminating controls and results

The 45-case corpus contains the32 independent browser-characterization cases, two negative-odd-quantum cases, two fractional font sizes, three responsive paragraphs, inherited factor/fixed-height controls, two zero-height cases and two preserved old encoding controls. Font sizes12–32, multipliers1/1.25/1.5/2 and fractional factors/vertical origins are covered. It remains a finite homogeneous-font corpus.

Each file imports once and clones once. Both scenes resize through240×160,390×200,768×120 and back to240×160 without compilation or source-map-driven mutation. Explicit Chrome font load/check succeeds before every reference capture. Boxes and nonzero-height baselines are measured independently from the emitted plan; zero-height baseline/count checks remain skipped because the DOM marker changes layout.

| Observation | Result |
| --- | --- |
| Candidate box/baseline metric gates | 344/344 frames pass;328 frames include baseline comparisons,16 zero-height frames compare owner boxes only |
| Candidate measured residuals | Largest serialized-probe box residual approximately0.000010px; largest baseline residual approximately0.000005px |
| Old symmetric16px and direct16px controls | 0/16 combined metric passes; original files and complete Chrome/native pixels reproduce exactly |
| Original DOM-box/frame pixel gates | 290/360 pass |
| Supplemental measured-ink region gates | 16/360 pass;344 failures retained |
| Independent prior browser references | All32 full PNGs, boxes and baselines match at390×200; earlier marker controls remain valid |

At16px with factor1.5, the source rule gives baseline17px relative to the owner, replacing the old17.5px hypothesis. At16px with factor1.333, the used height is21.3125px; two lines measure42.625px. The responsive counterpart measures three, two, one, then three lines at63.9375,42.625,21.3125 and63.9375px.

The negative-odd-quantum controls have used height16.984375px and raw leading−129. Integer division gives−64, then floor gives−1px: baseline14px. Naively flooring real-valued half-leading would give baseline13px. Both independent number and fixed-length browser cases agree with14px. At font16.01px, the receiving fixed font size is16.015625px and used height21.34375px; the measured two-line browser box is42.6875px. These cases distinguish actual layout arithmetic from CSSOM's serialized computed length.

A separate two-case precision supplement under `output/quantized-line-height-precision-r1` distinguishes source-number precision before percentage conversion. At16px, source number1.499999939 gives raw used height1536; its prematurely f32-rounded literal1.4999998807907104 gives1535. Chrome and native distinguish their two-line heights48 versus47.96875px and second baselines61 versus60.984375px in all eight original/clone frame pairs. Both CSSOM values serialize as24px / TypedOM1.5, so those strings alone cannot validate the rule. All16 supplementary metric/original-pixel frames pass; all16 local ink-region frames fail. This supplement is separate from the main360-frame totals and preserves its own six distinct full-image reviews and exact repeat/clone transfers.

Residuals above describe the serialized observations, not a proof of exact internal floating-point equality. Existing0.1px geometry tolerance was unchanged. Quantization claims additionally use the retained precise residuals and independent source derivation.

## Paint qualification remains open

The same unchanged pixel helper is applied first to the DOM box and then to the union of measured native/reference ink bounds. The union is observer-only; it cannot affect emission. It changes local RGB/interior means. Pixel mismatch ratio and mean channel error remain whole-frame, and the existing text antialiasing exclusion remains active. Empty zero-height boxes and large blank areas therefore do not receive an unsupported local-paint claim from the original pass count.

Only four cases have any supplemental passes:32px/factor2 (4/8), responsive16px/factor1.333 (2/8), zero32px/two identical lines (8/8), and zero32px/responsive (2/8). These include viewport clipping or overlapping paint; they do not establish general glyph fidelity. Native contour/weight differences remain visible after baseline alignment, and longer responsive lines expose horizontal differences. No threshold or paint setting was changed to obtain a pass.

Full visual coverage is recorded in [the visual review](quantized-line-height-visual-review.md) and `output/quantized-line-height-r1/visual/coverage.json`. The parent additionally inspected the16/17px candidate sheet, fractional-origin/negative-leading sheet and the zero/old-control sheet. Their candidate lines align while the retained old controls remain visibly misplaced. The gallery keeps original and supplemental results separate.

## Reproduction, evidence and next work

From the module directory, build/freeze and generate into a fresh path:

```sh
python3 validation/quantized-line-height-build.py output/quantized-line-height-NEW
node validation/quantized-line-height-native.mjs validation/quantized-line-height-cases.json output/quantized-line-height-NEW output/immutable-baseline-toolchain-r2/baseline-probe output/immutable-baseline-toolchain-r2/renderer-replay
node validation/reduced-line-height-ink.mjs output/quantized-line-height-NEW
python3 validation/quantized-line-height-analysis.py output/quantized-line-height-NEW
```

The builder freezes source inputs and executable tools. The renderer is the immutable RustMetal RasterOrdering baseline; the historical CLI mode token remains `clockwise-atomic`. Native observations, failed controls, precise residuals, exact historical/browser joins, local pixel metrics and full-image review are retained under `output/quantized-line-height-r1`. The [independent source/evidence review](quantized-line-height-source-review.md) checks the arithmetic and identities. Durable aggregate bindings are recorded in `validation/quantized-line-height-receipt.json`.

A08 remains investigating. This checkpoint supplies a source-backed homogeneous line-box construction, including fractional/reduced/zero leading in the tested contexts. Public font/text provision, paint fidelity, mixed runs and font metrics, empty/trailing lines, wider line-breaking, sibling/clip interactions, numeric/resource bounds and public transport integration remain open. Continue independent backlog work when no new discriminating paint hypothesis is available; do not rerun these unchanged cases as a substitute for implementation.
