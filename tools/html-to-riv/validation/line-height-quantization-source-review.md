# A08: Chromium 153 baseline and leading quantization

Chromium's ordinary inline-text path uses integer font ascent/descent, then
**floors the ascent-side half-leading to a whole CSS pixel** and puts the
remainder below the baseline. For the bounded positive-leading cases, the
baseline relative to the line-box top is
`B = A_integer + floor((L_used - A_integer - D_integer) / 2)`.
That source rule produces **17px, 26px, and 35px** for the existing Roboto
16px/24px, 24px/36px, and 32px/48px cases. It explains the failed symmetric
17.5px hypothesis without selecting an offset from browser measurements.
[CalculateLeadingSpace, lines 36–44](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/inline/line_utils.cc#L36)

## Exact source identity

This audit pins Chromium **153.0.8010.12** to commit
`971a7443b0c9b0a9b2860529b33331b76077ec62`, tree
`f3b8516f22967cf75d95541bf324421ed6c0be4a`. The Gitiles commit response and
the official Chromium GitHub mirror's tag response agree. Successful source
fetches, unsuccessful Gitiles attempts, complete source snapshots, SHA-256
bindings, and a commit-versus-tag byte comparison are preserved under
`output/line-height-quantization-source-r1`. This is a source audit, not a
new native or browser qualification run.
[Pinned Chromium commit](https://chromium.googlesource.com/chromium/src/+/971a7443b0c9b0a9b2860529b33331b76077ec62),
[official mirror tag reference](https://api.github.com/repos/chromium/chromium/git/ref/tags/153.0.8010.12)

## Font metrics and leading are separate calculations

`InlineBoxState::ComputeTextMetrics` selects `GetFontHeight` for ordinary
text without SVG or text-fit paint scaling. It computes the line height,
calls `CalculateLeadingSpace`, and adds the returned ascent/descent leading.
`FontHeight::AddLeading` adds each component to its corresponding metric.
[inline_box_state.cc, lines 121–152](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/layout/inline/inline_box_state.cc#L121),
[font_height.cc, lines 9–13](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/fonts/font_height.cc#L9)

`GetFontHeight` wraps the integer `Ascent` and `Descent` in LayoutUnits.
The setters preserve floating metrics separately and populate the integer
fields using `lroundf`. They are not the subpixel `FixedAscent` and
`FixedDescent` used by `GetFloatFontHeight`.
[font_metrics.h, lines 54–70 and 109–165](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/fonts/font_metrics.h#L54)

Earlier, `SimpleFontData::PlatformInit` obtains Skia metrics and calls
`AscentDescentWithHacks` before the setters. Its ordinary branch already
rounds Skia ascent/descent with `SkScalarRoundToScalar`. The source also has
metric overrides, a tiny-font exception, Linux-family VDMX/subpixel metric
adjustments, and a macOS ascent adjustment for Times, Helvetica, and Courier.
Therefore “round the font's hhea fields” is not a general replacement for
Chromium's platform metrics. The rounding of these inputs is distinct from
the later whole-pixel leading floor.
[simple_font_data.cc, lines 124–136](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/fonts/simple_font_data.cc#L124),
[font_metrics.cc, lines 50–147](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/fonts/font_metrics.cc#L50)

The unchanged fixture font has 2048 units/em, hhea ascent 1900, descent
−500, and line gap zero. Applying the source rule to those scaled metrics
gives the following arithmetic. The font hash and calculations are in
`derived-metrics.json`; these calculations do not read browser output.

| Font size / used height | Scaled hhea A / D | Integer A / D | Leading above / below | Baseline | Old symmetric baseline |
| --- | --- | --- | --- | --- | --- |
| 16 / 24 | 14.84375 / 3.90625 | 15 / 4 | 2 / 3 | 17 | 17.5 |
| 24 / 36 | 22.265625 / 5.859375 | 22 / 6 | 4 / 4 | 26 | 26 |
| 32 / 48 | 29.6875 / 7.8125 | 30 / 8 | 5 / 5 | 35 | 35 |

Equivalence between these font-table metrics and the browser's platform
metrics remains a bounded Roboto/platform assumption checked by the earlier
experiment, not a general font-admission guarantee.

## Unitless used height: preserve the actual operation order

`CSSPrimitiveValue::ComputeNumber` returns **double**. For a unitless number,
`ConvertLineHeight` multiplies that double by `100.0`, then clamps/converts to
float and stores a `Length::Percent`. This internal percentage represents
the inherited factor; an authored CSS percentage follows a different
conversion branch and becomes a fixed length.
[css_primitive_value.h, line 442](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/css_primitive_value.h#L442),
[style_builder_converter.cc, lines 2200–2218](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/css/resolver/style_builder_converter.cc#L2200)

`ComputedLineHeightAsFixed` resolves that percentage against the receiving
font's `ComputedFontSizeAsFixed`, which rounds the computed float font size
to the nearest 1/64px. Its fixed-length branch instead directly calls
`LayoutUnit::FromFloatRound(lh.Pixels())`.
[computed_style.cc, lines 2443–2459](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/style/computed_style.cc#L2443),
[computed_style.h, lines 847–848](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/core/style/computed_style.h#L847)

`Length::Percent()` returns float. `MinimumValueForLengthInternal` evaluates
the LayoutUnit font size times that float, then divides by `100.0f` and
constructs a LayoutUnit from the float result. The `LayoutUnit * float`
overload returns `a.ToFloat() * b`: it does **not** quantize the product to
a LayoutUnit before division.
[length.h, lines 217–219](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/length.h#L217),
[length_functions.cc, lines 60–70](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/length_functions.cc#L60),
[layout_unit.h, lines 564–565](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#L564)

LayoutUnit is `FixedPoint<6, int32_t>`. Its float constructor truncates toward
zero to a 1/64px multiple; `FromFloatRound` uses `roundf`. Thus the final
positive unitless used height is **truncated**, while a fixed pixel length
is **rounded**. The conceptual finite, non-clamping sequence is:

```text
percent_f32 = f32(number_f64 * 100.0_f64)
font_raw = roundf(computed_font_size_f32 * 64.0_f32)
font_fixed_f32 = f32(font_raw) / 64.0_f32
product_f32 = font_fixed_f32 * percent_f32
height_f32 = product_f32 / 100.0_f32
height_raw = trunc_toward_zero(height_f32 * 64.0_f32)
L_used = height_raw / 64
```

This is a derivation of the typed source operations, not an alternate
parser. A request carrier that converts the number to f32 before multiplying
by 100 can lose source precision earlier than Chromium.
[layout_unit.h, lines 125–147 and 473](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#L125)

The precise leading path additionally divides the signed LayoutUnit raw
integer by two using integer division, then calls `Floor`, which shifts
away its fractional bits. For positive leading this equals the opening
formula. For a negative odd raw quantum, preserve truncation toward zero
before the floor; do not silently extend the simplified real-number formula
to every reduced-leading value.
[layout_unit.h, lines 299–305 and 618–622](https://github.com/chromium/chromium/blob/971a7443b0c9b0a9b2860529b33331b76077ec62/third_party/blink/renderer/platform/geometry/layout_unit.h#L299)

## Qualification boundary

The source result is sufficient to define a new, falsifiable
`chromium-leading` hypothesis for homogeneous, positive-leading Roboto text.
It provides no glyph-paint repair. General font metrics, fallback fonts,
mixed runs, vertical/SVG baselines, text-fit scaling, emphasis marks,
text-box trimming, reduced leading, empty/trailing lines, fractional layout
origins, and further line-breaking behavior retain their own evidence
requirements. In particular, the formula is relative to a line-box top;
absolute paint-origin snapping is not established here.

No browser-derived offset, production implementation, test change, renderer
change, runtime mutation, or new native run was made for this audit. The
existing `unitless-line-height-review.md` and its failed 16px hypothesis are
preserved. The new source-backed rule must still be validated on the actual
ordinary emitted file, at original/clone resized viewports, with baseline,
line-box, and pixel checks kept independent.
