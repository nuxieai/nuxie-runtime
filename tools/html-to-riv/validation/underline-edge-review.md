# A20 whitespace, transparency and CJK edge review

Status: active. Chromium 153.0.8010.12, DPR 1, actual native renderer pixels.
The 13 new fixtures compile at 390 and resize the same scene at 240/390/768.

## Changes and checks

- Fixed ASCII case handling for decoration longhands and shorthand. Decoration
  parsing previously bypassed the existing normalization step, rejecting
  `currentColor` and `UNDERLINE`. A failing public regression test reproduced
  this before the fix. Color/keyword normalization now precedes both paths.
- Added full-frame red-decoration checks to fixtures that deliberately isolate
  saturated red lines from dark glyph ink. Every red pixel must have supporting
  ink within one pixel in the other image, in both directions. Coverage must
  remain within 85–115%. These checks supplement all unchanged layout, image and
  regional tolerances; they do not replace them or accept an absent reference.
- Negative controls prove missing, shifted, overlong and missing-middle-line ink
  fails even when whole-image averages pass. All ten pixel-gate tests pass.
- Corpus font selection now includes the existing renamed Japanese fixture in
  the browser, native import test and JS/native/WASM parity test.

## Results

39 new visual cases per renderer: breaks (including leading/trailing and empty
lines), pre/pre-wrap/pre-line/nowrap spacing, transparent glyphs with visible
underline, skip-auto with wrapping, Japanese None/Auto/All skip-ink, display:none,
transparent currentColor on white and on aliceblue.

| Profile | Passing | Remaining failures |
| --- | ---: | --- |
| Native glyph | 37/39 | CJK Auto and All at 240 |
| Vector | 34/39 | CJK None at all widths; Auto and All at 240 |

All geometry has zero measured error. All 78 browser/native/diff comparisons
were visually inspected in 14 contact sheets. Preserved whitespace, explicit
breaks and invisible output align. Kana skip behavior changes visibly between
Auto and All as expected, while the strict red check exposes tiny residuals.
The earlier 51 native-glyph underline PNGs remain byte-identical after the
keyword fix; the earlier two vector from-font failures are not erased.

The Auto/All 240 failure is a two-pixel faint red sliver near the final Latin
glyph. Pinned source shows Chrome uses non-antialiased difference clipping;
our remaining rectangles are antialiased fills. See underline-clip-research.md
for coordinates, source links and the internal renderer dependency. None's
vector failures instead expose one or two red pixels near overlapping glyph
edges; with no skip-ink active they cannot be attributed to skip clipping.
All these cases pass the ordinary image gate, demonstrating why the additional
sparse-decoration check is necessary. No threshold was widened to remove them.

The initial transparent-currentColor control on aliceblue also triggered exact
PNG inequality. A native control with underline removed produces exactly the
same pixels as the transparent underline. Differences from Chrome were solely
1,131 background pixels shifted from (240,248,255) to (241,249,255) at 240.
The aliceblue case remains in the corpus under the unchanged ordinary pixel
limits. A separate white-background transparent control requires exact equality
and passes, as does hidden content. This isolates transparency from existing
background-color quantization, without waiving the original image thresholds.
The native enabled/disabled control is retained in
`/tmp/html-underline-transparent-baseline.{json,riv,png,stream}` and the initial
artifact under `underline-edge-glyph-final/`.

## Reproduce and other verification

`NUXIE_NATIVE_GLYPHS=1` (or `0`),
`NUXIE_HTML_REVIEW_DIR=<absolute review dir>`, then
`npm --prefix tools/html-to-riv test -- --grep 'underline-edge-|underline-cjk-|red decoration gate' --output=<separate test output dir>`.
The runner includes the negative-control test, so it reports 38/40 or 35/40;
the visual-only counts are 37/39 and 34/39. Commands correctly exit nonzero for
retained CJK failures. Cases stay in the main corpus and will fail the full gate.

Artifacts: `output/playwright/html-to-riv/underline-edge-{glyph,vector}-reviewed/`
and corresponding `test-results-underline-edge-*-reviewed/` directories. Each
contains or links the exact source, Rive, requirements, bounds, PNGs and metrics.

88 Rust module tests pass. All five JS tests pass, including full-corpus
native/WASM byte/source/requirements parity after adding the Japanese font and
new fixtures. WASM build, module Clippy and runtime boundary checks pass.
The full unrelated visual corpus was not rerun in this phase; its prior A09
failure remains in the previous receipt. This phase reran all prior native
underline fixtures plus the new edge cases, and both profiles of new edge cases.

Logs: `/tmp/html-underline-case-red.log`,
`/tmp/html-underline-edge-{suite-final,parity-reviewed,clippy,boundary,glyph-reviewed,vector-reviewed}.log`,
`/tmp/html-underline-pixel-controls.log`.

Next: implement device-correct hard skip-ink clipping and investigate vector
ink overlap, then qualify missing/zero font metric fallback and host transforms,
opacity/clipping. A20 remains active, with no external blocker declared.
