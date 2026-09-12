Ordinary signed padding works for these reduced and zero line-height controls on immutable runtime `6c7ac16617835b5f581784ff08a9e779bb52faf3`. The earlier example's rejection of negative padding was its own restriction. This private experiment changes no runtime, renderer, font, paint settings, public compiler source or support contract.

`LayoutComponentStyle::apply_container_style` copies the padding fields into `YGValue::new`; `layout_style_applier::length` retains finite signed point values in `LengthPercentage`. The pinned vendored Taffy resolver returns point values unchanged. Native import and layout confirm the signs survive. These are synthetic ordinary Rive fields: this does not admit negative authored CSS padding. Selected runtime, vendored layout and lockfile sources are byte-identical to the immutable baseline; their hashes are retained in `output/reduced-line-height-r1/runtime-source-bindings.json`.

The new `examples/reduced-line-height.rs` keeps the previous rounded-font-metric hypothesis unchanged: with native font ascent A, descent D, H=A+D and used line-height L, B=(L+round(A)-round(D))/2. The wrapper emits top=B-A and bottom=L*A/H-B, including negative values. For homogeneous text, native intrinsic height is N*L+A*(1-L/H); adding those pads produces N*L and shifts the first baseline to B without reading N. Emission reads only the source plan and unchanged licensed Roboto head/hhea metrics, with one text Fill. It consumes no browser measurements, line counts, baked positions or host callbacks. The baseline quantization rule remains a finite candidate; parent investigation of that rule is separate.

For inherited 24px line-height at 32px font size, A=29.6875, D=7.8125, B=23, top=-6.6875 and bottom=-4. Native two-line Text retains intrinsic height58.6875, while its ordinary owner measures48 and its baselines are43/67 in world coordinates, matching Chrome. The one-line owner measures24. Responsive wrapping gives6→3→2→6 lines and144→72→48→144 owner heights through240×160→390×200→768×120→240×160. For zero line-height, top=-18.6875 and bottom=-11; native Text retains intrinsic height29.6875, while the owner measures0. Native glyph baselines overlap at worldY31 and the rendered overlap moves into the Chrome position.

Each of seven scenes is imported once, then measured and rendered through four resizes on original and clone:56 frames. Chrome153.0.8010.12 explicitly loads/checks the exact embedded font before collecting computed style, Typed OM, range/canvas metrics and PNGs. Every original source PNG equals its independent used-length CSS control. Zero-height DOM ranges do not distinguish the overlapping line count/baselines, so those checks are skipped; zero metrics below mean owner rectangle only. Inserting baseline markers changes zero-height layout and is not used. Native zero line counts are observations, not browser line-count equivalence.

| Case | Box/baseline metrics | Original pixel gates | Supplemental ink-region gates |
|---|---:|---:|---:|
| Direct inherited24px at32px, two lines | 0/8 | 0/8 | 0/8 |
| Signed wrapper, same two lines | 8/8 | 4/8 | 0/8 |
| Signed wrapper, responsive reduced text | 8/8 | 0/8 | 0/8 |
| Direct zero, two lines | 0/8 | 0/8 | 0/8 |
| Signed zero, two lines | 8/8 | 8/8 | 8/8 |
| Signed zero, responsive text | 8/8 | 6/8 | 2/8 |
| Signed wrapper, reduced single line | 8/8 | 4/8 | 0/8 |
| Total | 40/56 | 22/56 | 10/56 |

The original helper's zero-height DOM region contains no pixels. Its zero passes therefore cover whole-frame error and ink presence, not a local text region. A separate retained receipt applies the same pixel/RGB thresholds to the union of measured native/reference ink bounds, also capturing glyphs outside reduced line boxes. This observer-only region never enters emission. All five signed candidates pass40/40 metric observations; only10/40 pass this additional paint-region check. No tolerance was widened. Direct controls retain height/baseline and paint failures.

All21 distinct full Chrome/native PNG pairs were inspected on six unscaled sheets;35 repeat/clone pairs transfer by exact completeRGBA identity with matching source Rive and native/browser metrics. Direct text is visibly too low; signed text aligns and wraps correctly in these controls, while remaining glyph contour/spacing differences are visible and retained. Zero repeated text shows the expected overlapping paint, including differences that whole-page averages obscure. The coverage receipt binds every inspected/transferred image.

This answers the bounded capability question: this ordinary signed-wrapper construction can subtract leading, including all intrinsic height, on the frozen runtime. No alternative origin/margin construction was needed. It does not establish general typography or public A08 support. Baseline quantization, mixed inline fonts/styles, empty/trailing lines, wider line-breaking rules, source-value numeric bounds, clipping/interactions with siblings, font provisioning and the independent paint gate remain to qualify. These uncommitted candidate files are ready for review; no additional hypothesis or production integration was started.

Evidence: `output/reduced-line-height-r1/receipt.json` (aggregate), `native-receipt.json` (all56 actual observations and original gates), `ink-region-receipt.json` (supplement), `visual/coverage.json` (complete review/transfer proof), and `validation/reduced-line-height-receipt.json` (durable bindings).
