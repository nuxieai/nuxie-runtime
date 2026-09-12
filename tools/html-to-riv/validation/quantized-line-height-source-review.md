# Independent source and evidence review: quantized line height

The private `quantized-line-height` example implements the reviewed Chromium
operation order correctly for this finite Roboto/platform profile. No
blocking arithmetic or measurement defect was found. The main corpus has
**344/344 candidate metric passes**, consisting of **328 nonzero
baseline-and-box passes plus 16 zero-height box passes**. The two old
hypotheses remain failed controls. This is not public text admission or
general paint qualification.

## Arithmetic versus pinned Chromium

The reference is Chromium 153.0.8010.12, commit
`971a7443b0c9b0a9b2860529b33331b76077ec62`, with exact source snapshots and
locations in `output/line-height-quantization-source-r1`. The earlier
`line-height-quantization-source-review.md` supplies the complete call path
and distinguishes platform metrics from font-table metrics.

`SourceHeight::Number(f64)` preserves the number through multiplication by
100 in f64, then converts the percentage to f32. The font size is rounded
to 1/64px before percentage resolution. The percentage product and division
are f32, with no intermediate LayoutUnit quantization; the resulting used
height truncates toward zero to a 1/64px raw integer. The fixed-pixel branch
instead converts to f32 and rounds to a 1/64px raw integer. These match the
pinned `ComputeNumber`, `ConvertLineHeight`, `ComputedLineHeightAsFixed`,
and `MinimumValueForLengthInternal` path. The finite coefficient bounds
keep these operations away from integer saturation.
[Number conversion](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/resolver/style_builder_converter.cc#L2207),
[used-height conversion](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/style/computed_style.cc#L2443),
[percentage arithmetic](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/length_functions.cc#L60)

`source_baseline` first divides the signed raw leading by two using Rust
integer division, then uses `div_euclid(64)` to floor the result. That
matches Chromium's signed raw integer division followed by `Floor`; it
does not replace the operation with a real-number half-leading floor.
Positive hhea ascent/descent are rounded to integer metrics for this
unchanged Roboto profile. That input selection remains font/platform
specific rather than a complete implementation of Chromium's platform
metric selection and overrides.
[Leading split](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/inline/line_utils.cc#L36),
[signed division](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#L618)

An independent f32/f64 emulation reproduced all **45 saved plans**,
including used raw height, ascent/descent, and baseline. The following
controls distinguish important parts of the formula:

| Source | Font fixed for height resolution | Raw used height | Relative first baseline | Independent observation |
| --- | --- | --- | --- | --- |
| Negative odd raw, number and fixed forms | 16 | 1087 | 14 | Leading −129 divides to −64; a real-number floor would incorrectly give baseline 13. Both browser/native baselines are 34 and 50.984375 in world coordinates. |
| 16.01px font, factor 1.333 | 16.015625 | 1366 | 16 | Actual two-line height is 42.6875 and second baseline 57.34375. |
| 24.125px font, factor 1.333 | 24.125 | 2058 | 24 | Actual two-line height is 64.3125; browser second baseline is 76.15625. |

For 16.01px, CSSOM reports `21.3413px`, while the measured line advance is
21.34375px. The native driver correctly compares measured rectangles and
baselines; it does not replace those measurements with the serialized
computed-style string.

## Main corpus provenance and measurements

At the initial review, **2,265 SHA comparisons across 1,674 distinct files**
matched: frozen inputs and their snapshots, example and observer binaries,
source fixtures, pixel helpers, and all 360 Rive/stream/geometry/text/PNG
bindings. Every generated request and saved browser source matches its
fixture. Every Rive and browser data URL contains the unchanged complete
Roboto font; the actual native probe input equals the generated Rive.
The text observer is exactly reused from the reduced-line-height run.

The browser driver uses the fixture's authored CSS, text, and padding
origin. It does not read `plan.json`, emitted used height, native baselines,
or native line count when constructing the reference page. Font readiness
is explicitly checked. For inherited numbers the browser resolves the
parent's number on the child; for inherited fixed lengths it retains the
parent length. The private request carries those values separately; this
is a hand-authored experiment carrier, not a public cascade implementation.

Independent reconstruction of saved observations gave:

| Check | Main candidates | Retained old controls |
| --- | --- | --- |
| Combined available metric gates | 344 / 344 | 0 / 16 |
| Baseline comparison | 328 / 328; 16 zero frames skipped | 0 / 16 |
| Owner rectangle | 344 / 344 | 8 / 16 |
| Original pixel profile | 286 / 344 | 4 / 16 |
| Supplemental ink-region profile | 16 / 344 | 0 / 16 |

All 360 original pixel comparisons, ink-presence measurements, and
supplemental comparisons were recomputed from the existing PNGs and match
their receipts. The old symmetric control preserves a 0.5px baseline
failure despite passing its box comparison; the direct control preserves
both box-height and baseline failures. Zero-height baseline/line-count
comparisons remain skipped.

The largest observed candidate residuals are approximately **0.00001px
for a box dimension** and **0.000005px for a baseline**. These are residuals
of serialized probe observations: native f32 values are printed as decimal
JSON and the driver sums parsed coordinates. They are not claims of exact
binary32 computation or a new numerical proof. The unchanged 0.1px geometry
gate remains the gate.

## Separate precision-order supplement

The main 42 number requests did not distinguish correct double-first
conversion from premature f32 conversion. A separate two-source supplement
closes that specific gap without modifying the main 45 cases, frozen
generator, native driver, text observer, or pixel helper.

`validation/quantized-line-height-precision-cases.json` contains otherwise
identical 16px, two-line sources using these authored number literals:

| Literal | Percentage after correct conversion | Raw used height | Chrome/native owner height | Chrome/native second baseline |
| --- | --- | --- | --- | --- |
| `1.499999939` | 150.0f32 | 1536 | 48 | 61 |
| `1.4999998807907104` | 149.99998474121094f32 | 1535 | 47.96875 | 60.984375 |

The second literal is exactly the first prematurely rounded to f32.
Both Chrome and the frozen generator distinguish them. All **eight
original/clone viewport pairs** preserve an exact height difference of
**0.03125px** and second-baseline difference of **0.015625px**, explicitly
checked beyond the looser 0.1px gate. Both CSSOM/Typed OM serializations
round to `24px`/`1.5`; the measured geometry supplies the discrimination.

The supplement has **16/16 metric passes**, **16/16 original pixel passes**,
and **0/16 supplemental ink-region passes**. All six complete Chrome/native
pairs were directly inspected on three unscaled sheets. Text rows and left
edges align; native stem/curve rasterization differences remain visible.
Ten other original/clone pairs transfer by exact same-source metrics and
complete RGBA identity. Visual inspection does not claim to resolve a
1/64px baseline difference by eye.

Exact deltas, commands, copied source/tools, failures, review coverage, and
a reusable read-only verifier are bound by
`output/quantized-line-height-precision-r1/receipt.json`. The 16 supplement
frames remain separate from the main 360 frames.

## Source chronology and remaining limits

During final packaging, concurrent public request-transport work changed
live `src/lib.rs` and `src/wasm.rs` after the typography generator freeze.
The first live-source equality check and copies of the changed sources are
preserved in the supplement's `verification-live-source-failure.json` and
`live-source-at-verification`. The final verifier checks the **46 frozen
source snapshots** and exact binaries, and records live-source differences.
The typography example, wire encoder, font, schema, native probes, renderer,
and frozen generated files are unchanged. The typography evidence does not
claim to include the later transport change.

The ink supplement localizes RGB/interior means; mismatch ratio and
mean-channel error remain whole-frame, with the unchanged text antialiasing
exclusion. Its failures prevent a blanket paint qualification. Further
font/platform profiles, mixed runs, empty/trailing lines, clipping and
sibling interactions, general whitespace/wrapping, arbitrary fractional
origins, asset admission, and public source grammar remain outside this
finite result. Only the explicitly requested two-source supplement was
newly rendered for this review; the main corpus was not rerendered.

## Reviewed identities

| Artifact | SHA-256 |
| --- | --- |
| `examples/quantized-line-height.rs` | `5f54c504579f83277f62a6fadedec0da8b2df276ff425d5ca4cc98007974249f` |
| `validation/quantized-line-height-build.py` | `d3e728e3ed37a335428403226627d28dc6520eecd6a7da17a1157494f20d5b0c` |
| `validation/quantized-line-height-cases.json` | `2d674285c05bfab8b760372ddb059e830c36fe3ae908018ea26b978b14a3be12` |
| `validation/quantized-line-height-native.mjs` | `2a5aa6cc491941a12172372acc386d9abac443e76bc4293c55606f9899683fcf` |
| `output/quantized-line-height-r1/build-receipt.json` | `9d36e5a3d904bb265036422fa4aa1da9d57e2bff04fce46c65efa65e8f84f097` |
| `output/quantized-line-height-r1/native-receipt.json` | `4aba3b366727c30b63cc0d9c449ce13fa174337a75dbc328e87ee6395b43265e` |
| `output/quantized-line-height-r1/ink-region-receipt.json` | `64e3dd48a21cebf620c225207076ae550c97c849fd5636be87cfc1f34c03eed3` |
| `output/quantized-line-height-r1/generator` | `2d416545be3653d5816addb5f88027176d5f9567213c1f600991d12ce1a277a3` |
| `output/quantized-line-height-r1/text-probe` | `190dd09ce6c3b23fcda10af86859ebc3fb5c4fb4e9803e059248a5d9fd1e6059` |
| `validation/quantized-line-height-precision-cases.json` | `a274ffa9c70446a28127174f53929cb101726576d5085522e44b35db6617df86` |
| `output/quantized-line-height-precision-r1/receipt.json` | `64a1cd7806af82ea2af37fed272dbb9593d008edb50534b64e57b25a079a29db` |
