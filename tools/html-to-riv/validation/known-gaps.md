# Open composition rendering gaps

Current profile update: all nine comparisons in this corpus pass through the
opt-in live macOS CoreText/Metal adapter. The vector results below remain valid;
this corpus is retained to track both profiles. See [live-glyph-review.md](live-glyph-review.md).

These fixtures remain open under BACKLOG.md Q09. They are accepted syntax with
known rendering differences, not intentionally unsupported compiler inputs.
Run them using the separate known-gap command in VALIDATION.md. The runner uses
the same geometry and pixel assertions, emits real failure artifacts, and must
not mark failures as expected passes or widen thresholds.

## Membership card

The initial comparison exposed auto-width text in a flex row collapsing to zero
width. The compiler emitted every Text participant as Fill, even when its parent
needed the text's intrinsic width. The corrected inflexible auto-basis row case
uses AutoWidth text and a Hug participant. It preserves responsive measurement
in the runtime. A separate `intrinsic-text-row-sizing` fixture qualifies this
mapping at 240, 390 and 768px. This does not remove the existing positive-flex
content-derived auto-basis restriction (L10).

The original card remains intact as a reproducer. After the sizing fix its box
geometry agrees but mixed 12/14/24/28px text exceeds the existing pixel limits.
Differences concentrate on glyphs, not solid fills. At 768px the initial corrected
card had region RGB means of about 8.14 for the amount and 10.15 for the period,
with interior means about 9.18 and 11.81 (limit 6).

## Typography specimen

A second composition isolates 12, 14, 16, 20, 22, 24 and 28px Inter text. It
keeps this problem observable without the card's row layout or color cascade.

## Investigation to continue

Chromium canvas metrics round ascent/descent at these font sizes; inline
baseline markers showed integral first baselines for the sampled integer line
heights. Experimental padding corrections based on rounded metrics and floored
half-leading reduced some errors but did not qualify both compositions. Those
unqualified corrections were removed. The existing line-height lowering remains.

Next: compare actual glyph baseline/outline positions at subpixel precision,
separate baseline placement from rasterization differences, and identify the
compiler/runtime/rendering change needed. Qualify any correction across the full
existing text corpus and the new compositions. The current evidence is not an
external blocker; this is unresolved implementation/validation work.

## Baseline and rasterization isolation (2026-09-08)

`node tools/html-to-riv/validation/text-diagnosis.mjs` renders the original
font-shorthand fixture, equivalent longhands, a sentence, H/Hg, tight leading
and fractional font metrics. It writes measurements and independent browser/
native/difference images under output/playwright/html-to-riv/text-diagnosis.
This is an investigative report, not an acceptance command; inspect its
`failures` arrays. The regular browser spec remains the executable failing gate.

Shorthand and longhands produced identical Chromium pixels and native bytes.
Before baseline correction, a 20px H had native ink centroid y=14.5234 versus
Chromium y=14.0825; native ink area was 65.92 versus 73.36. Tightening its box to
20px made the pixel gate fail independently of surrounding composition. Using
rounded font metrics and floored half-leading for the glyph baseline fixes the
H and original shorthand fixture. Applying the correction as a Text transform
preserves line-box measurement and accepts valid tight leading (22px/27px).

A tight 20px Hg still fails: interior RGB error 6.5944, limit 6. Changing only
the diagnostic browser's -webkit-font-smoothing to antialiased reduces that
error to 2.1122. This identifies a separate font-rasterization contribution;
it is not permission to change the reference stylesheet or loosen thresholds.
The default profile and all acceptance tolerances remain unchanged. The new
`text-rasterization-small-glyphs` reproducer is in known-gaps.json and fails at
all three widths. Existing membership and typography specimens remain intact.
The updated known-gap run has seven failures and two passes across nine checks.

Next: investigate a compatible text-rasterization path and font-dependent normal
line boxes. Do not apply a global renderer color/coverage adjustment to shapes
or change a font's outlines merely to make the browser comparison pass.

Relevant source: Chromium [SimpleFontData](https://chromium.googlesource.com/chromium/src/+/HEAD/third_party/blink/renderer/platform/fonts/simple_font_data.cc)
uses rounded ascent/descent/leading for line spacing. The baseline correction
is qualified against the pinned browser; this does not assert identical font
metrics or smoothing across operating systems.

A subsequent exact-outline control reproduces the Hg and normal-line-height
failures entirely inside Chromium (DOM text versus Canvas paths), while native
versus Canvas paths passes. See text-rasterization-seam.md for the measurements,
existing API limitation and next bounded native-glyph experiment. No generic
renderer adjustment or browser-reference change was made.
