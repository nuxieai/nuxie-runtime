# Pinned Chrome layout-length quantization before paint

Read-only source investigation. No product, runtime, renderer or dependency changed. The source analysis below was written as a prediction before capture; a separate dated confirmation section now records the observed boundary result. Neither section grants public qualification. It uses Chromium153.0.8010.12 commit `971a7443b0c9b0a9b2860529b33331b76077ec62`. Existing paint-source snapshots supply LayoutUnit and snapping; missing conversion/layout files were fetched from the same immutable Gitiles commit into `output/wrapped-boundary-quantization-source-r1`. The manifest preserves an initial503 for length_functions.h and a404 for an obsolete flexible_box_algorithm.cc path; the successful header retry and actual flex_layout_algorithm.cc are bound separately.

## Ordered fixed-pixel path

For an ordinary resolved fixed CSS px width/height, padding or margin in this corpus, with zoom1 and no transforms:

1. Numeric CSS length resolution computes a double pixel value. `CSSNumericLiteralValue::ComputeLengthPx` delegates to `CSSLengthResolver::ZoomedComputedPixels` (css_numeric_literal_value.cc131–134); the px case multiplies by zoom (css_length_resolver.cc153–159).
2. `CSSPrimitiveValue::ComputeLength<Length>` stores `Length::Fixed(ClampToCSSLengthRange(...))` (css_primitive_value.cc328–332). The clamp returns float (60–65). `Length` stores that value as float; Fixed and Pixels retain it (length.h147–149,203–216). This does **not** yet quantize to1/64.
3. Ordinary layout resolution calls `MinimumValueForLength`. Its fixed fast path constructs `LayoutUnit(length.Pixels())` (length_functions.h53–63). Inline/block sizes reach it at length_utils.cc67–80 and201–222; margins at1443–1457 and padding at1514–1540.
4. The float LayoutUnit constructor scales the f32 value by64, converts toward zero into its bounded int32 storage, and stores that raw integer (layout_unit.h96–100,125–130). In this corpus's nonsaturating range, the exact operation is `Q(x) = trunc(f32(f32(x) * 64)) / 64`. Positive values resemble floor; negative margins do **not** use floor. Explicit FromFloatFloor and FromFloatRound are distinct APIs (134–151), not the fixed-length path.
5. Flex placement starts with a LayoutUnit padding/initial offset (flex_layout_algorithm.cc1859–1863), adds a LayoutUnit leading margin, stores the item offset, then advances by its LayoutUnit border-box size, trailing margin, distributed space and gap (2008–2016). Ordinary LayoutUnit addition adds raw integers with saturation (layout_unit.h646–650). Thus independently resolved fixed operands are quantized before accumulating the position.
6. Only after layout does the ordinary solid-background painter apply the already documented pixel-edge/thin-box rule. Paint rounding cannot reconstruct the earlier discarded fractional layout information.

Primary source links: [length_functions.h](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/length_functions.h#53), [CSSPrimitiveValue](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/css_primitive_value.cc#328), [length resolution](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/length_utils.cc#67), [flex placement](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/flex/flex_layout_algorithm.cc#2008), [LayoutUnit](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#125).

## Boundary predictions, explicitly not observations

`predictions.json` records the source-derived arithmetic and f32 operands.

| Case | Predicted Chrome layout/paint | Unnormalized native-f32 implication |
| --- | --- | --- |
| Width nextf32(1/16), bits0x3d800001 | 0.0625000074505806 resolves to1/16; the strict size>1/16 exception is false | The raw authored size is above the threshold. At origin0, corner subtraction preserves it and the current native thin flag is true. At nonzero origins, corner-add/subtract rounding can erase this difference; do not predict a mismatch without the actual placement. |
| Cumulative64.249px plus0.251px | Resolve independently to64.234375 and0.25; sum64.484375; painted start64 | f32 operands64.2490005493164 and0.25099998712539673 sum to64.5; native rounded start65 |
| Margin -0.251px | Quantizes toward zero to-0.25 | A compiler implementation using floor would incorrectly produce-0.265625 |

These predictions assume the stated values are separate layout operands (for example a fixed spacer plus fixed margin/padding), ordinary nonflexing boxes, and no transform/zoom. A single `calc(64.249px + 0.251px)` can resolve through a different computed expression path; this review does not equate it to two layout operands. Likewise an empty-width box can acquire paint through borders or other decoration; the prediction is for the current no-border solid box.

## Responsive lengths and computed positions

The percent route explicitly computes `LayoutUnit(float(maximum_value * percent / 100.0f))` after receiving a LayoutUnit reference size (length_functions.cc60–70). `LayoutUnit * float` converts the reference to float and multiplies (layout_unit.h564–566), then division and the explicit float result precede the1/64 conversion. Therefore the relevant order is already-quantized live basis → f32 percentage multiplication → f32 division by100 → LayoutUnit truncation. Quantizing the authored percentage coefficient to1/64 is a different operation and is not supported by this source.

A compiler-only normalization of resolved fixed layout inputs is semantically warranted for this closed path, provided it preserves the float conversion, sign, bounds, box-sizing and clamp order. It should occur at the particular layout-length conversion boundary, not replace all CSS numbers globally. Fixed dimensions, fixed padding and margins in a source-owned simple layout can be normalized without browser measurements or recompilation on resize; the surrounding layout remains live.

That change would not establish general fidelity. Responsive percentages need quantization after their live reference is known. Flex distribution, center/end offsets, min/max and intrinsic sizing can introduce additional used-value arithmetic; even normalized fixed inputs can be divided or combined later. For example integer division of a LayoutUnit returns a raw-integer quotient (layout_unit.h618–623), which can discard a1/128 half-step. The compiler must investigate a file-level composition or existing native behavior for those operations and reject unsupported semantics honestly. It must not approximate this by rounding only final world corners or normalizing a measured viewport snapshot.

Transforms and some geometry APIs use floating `FloatValueForLength`, which directly returns Pixels for fixed values (length_functions.cc34–41). Applying the LayoutUnit rule indiscriminately to transform coordinates, SVG, images or every paint parameter would therefore be wrong. This review supports a targeted fixed-layout conversion hypothesis, not blanket normalization or an immutable-runtime limitation finding.

## Bound primary files

New-source manifest SHA-256: `3c0a4c1038992ec2853d5cc26612687c79f69a4ff2e7e6e240dac7729e2934f2`. Predictions SHA-256: `faa584f6620800f32e8e7a75d10ed8f41e5699ff4eeab78f58b48c9008458dc6`.

| Source file | SHA-256 |
| --- | --- |
| `third_party/blink/renderer/platform/geometry/length_functions.cc` | `703a8fb9e1a6ad902bbfa32c83de7f8fe52a45096b872fbd3f85d61ca7d6ece1` |
| `third_party/blink/renderer/core/css/css_primitive_value.cc` | `6edde0b4d5cc0399ccc5a040638e43d7ee981850b61f0bda032dad438ac64f44` |
| `third_party/blink/renderer/core/css/css_numeric_literal_value.cc` | `68da790e043059406ab6dd5600bdd37ffbb758cadefa733a7e15148b6fa83231` |
| `third_party/blink/renderer/core/layout/flex/flex_layout_algorithm.cc` | `e99678c91c38568d0d430707ed3ba47efafc352bbdd13be8aa170283041706f1` |
| `third_party/blink/renderer/platform/geometry/length_functions.h` | `080a1c576acfd10fbd8cdc2101c9b1462e9f2f9a0ee8060a834808bf2c579dc7` |
| `third_party/blink/renderer/platform/geometry/length.h` | `fd717adbf0f6010de78df9c1b6c19c33fa369be84b01243a5cfedad9f8351244` |
| `third_party/blink/renderer/core/layout/length_utils.cc` | `91c1414656c23da1b92267ac56e4b7fe1d7f83bb79096858a0d64f6cbd8e0134` |
| `third_party/blink/renderer/core/css/css_length_resolver.cc` | `6a31ffdb9ee8524fdce116adce3a2dbcdddf71db816e32dc3c2be2707e3c0b12` |
| `third_party/blink/renderer/platform/geometry/layout_unit.h` (existing snapshot) | `fdc4e8d9386923943ec92a6868dcf59b34c741d52b19d0ee41c255092ff23624` |

## Observed boundary confirmation

This section was added after the boundary capture completed. The original source predictions and `predictions.json` are unchanged. To avoid interference with the running release benchmark, this update reads only the compact per-case summary; it does not reload or reinterpret the large raw capture receipt. The summary identifies the exact capture and scalar/stream observation receipts by hash.

| Captured case | Pixel-failure frames | Observed clipped paint interval |
| --- | --- | --- |
| boundary-row-thin-next-above | 0–7 | Native[0,1), Chrome empty; measured native size0.0625000074505806, Chrome size0.0625 |
| boundary-column-thin-next-above | 0–7 | Same discrepancy on y |
| boundary-row-accumulated-decimal | 0,1,3,4,5,7 | Representative frame0: native[65,95), Chrome[64,94) |
| boundary-column-accumulated-decimal | 0,1,3,4,5,7 | Same discrepancy on y |

The two source predictions therefore have matching captured counterexamples, including original and clone observations. There are28 pixel-failure frames and28 clipped-edge-difference frames. The92 raw edge-difference frames also include64 saturation-only differences whose clipped intervals agree; these must not be mislabeled Chrome quantization failures. No saturation-intersection failure is reported by the summary. Frames2 and6 of the accumulated-decimal cases are not reported as failures; the source prediction is conditional on the relevant accumulated layout position and does not assert every responsive state disagrees.

Confirmation summary: [case-summary.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-boundary-r1/boundary-observation/case-summary.json), SHA-256 `25c1ba9e82b08a57ad26fe97467cf28f8cad8cf3bb89b400f925c103b385e619`.

- [receipt.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-boundary-r1/receipt.json), recorded source SHA-256 `5efb862d23c0ceddd22ebecfde5f555cf4164535e0d85aa142db68fb42522bb5`.
- [receipt.json](/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-boundary-r1/boundary-observation/receipt.json), recorded source SHA-256 `30f5c1286681cd692e3c3cb52d71048943bda80aca0e8c7aa8ba2e00e072de9f`.

Original prediction artifact remains SHA-256 `faa584f6620800f32e8e7a75d10ed8f41e5699ff4eeab78f58b48c9008458dc6`.

## Concrete next implementation: explicit fixed layout conversion

The next change should model a compiler-owned Length→LayoutUnit conversion, not repair pixels after composition.

1. Add a private fixed-layout conversion carrier, for example `FixedLayoutLength { authored: ScalarProvenance, computed_px_bits: u32, raw_units: i32, emitted: f32 }`. Preserve the original declaration/expression provenance and its pre-quantization native value. Do not replace the original scalar with `exact_constant(quantized)` or claim the authored decimal equals the quantized value. The quantized layout integer is a distinct derived semantic value.
2. Implement a checked constructor for the scoped finite px path: establish the correct computed CSS float input; multiply that f32 by64; truncate toward zero into bounded raw units; divide the integer by64 for the emitted native layout value. Preserve source and result bits plus a typed conversion operation so an independent reader can verify the relation. Explicitly handle representable range and nonfinite rejection under existing compiler limits; do not rely on unchecked language cast behavior or widen accepted dimensions implicitly.
3. Introduce it at resolved **fixed layout length** lowering for preferred/min/max sizes and the supported fixed padding/margin/flex-basis descriptors. Conversion must follow CSS resolution and precede layout combination/clamping at the audited boundary. Percentage coefficients, scalar transforms, SVG coordinates, colors and font properties are not transformed by this operation. A single computed calc expression is normalized after its computation, not token-by-token.
4. Extend the numerical descriptor/binder to compare emitted fields to the derived conversion output while retaining the source relation. `wrapping_sizes::scalar_axis` currently requires record bits equal `ScalarProvenance::native()`; change this through an explicit normalized-layout descriptor rather than overwriting provenance or weakening that comparison. Refresh coordinate/extent/error proofs from the actual emitted values, including positive authored values that legitimately quantize to zero. Keep nonnegativity and supported-zero semantics distinct.
5. First drive the actual private Derived constructor with this descriptor for the complete existing fixed-input boundary corpus. Quantize separately authored64.249 and0.251 layout operands before composition, and quantize the tiny fixed visible width before both layout and thin-paint reasoning. Do not normalize only paint anchors or keep phantom unquantized owner geometry. Preserve previous captures and diagnose whether any remaining discrepancy is layout arithmetic, thin extraction or rendering.
6. Add direct boundary tests for signed nextafter values around n/64, exact/subnormal zero, ±0.251,1/16 and its f32 neighbors, range checks, inheritance, and one-expression versus separate-operand accumulation. Verify original source provenance remains inspectable and wrong conversion/output-bit mutations reject. Public-facing work must exercise Rust/CLI/WASM/JavaScript parity and diagnostic behavior before admission.
7. Rerun the same original/clone resize scenes, strict masks/streams and Chrome pixels with unchanged tolerances. Specifically account for all28 preserved failing frames and retain the64 intentional saturation-only differences. Repeat the existing48-case rounded wrapping matrix and existing public compatibility checks as appropriate to changed paths; do not transfer an old binary qualification to new bytes.

Responsive lengths remain a separate implementation obligation: derive the live percentage used value in the audited float order, quantize **that** result to1/64, then let dependent layout consume it. Normalizing a coefficient or an initial viewport result cannot satisfy this. Later alignment/distribution also performs LayoutUnit arithmetic, so fixed-input normalization alone does not establish arbitrary flex fidelity. Investigate ordinary file compositions and existing runtime operations for those steps; document unsupported paths until proven. No runtime change, browser-baked position or resize recompilation is proposed.
