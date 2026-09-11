# Pre-wrap direct mapping and runtime requirements

A17 is partial. The compiler accepts pre-wrap as an inherited WhiteSpace mode,
preserves source text like pre, and uses native soft wrapping. Normal/initial
restore collapse; inherit/unset preserve the computed mode. Br identities retain
Unicode-scalar offsets. The public contract failed with unsupported-value before
implementation and now passes alongside the other compiler contracts.

Initial real-renderer corpus: 30/36 comparisons pass. At 240px basic/center/right
text and all-space content create an extra native line. Long words split at
glyph boundaries at 240/390px, while Chromium keeps them on an overflowing line.
These are semantic/geometry failures, not reasons to widen pixel thresholds.
The initial contact sheet is prewrap-probe/failures240.png under
output/playwright/html-to-riv. Two forced-break/space alignment cases bring the
final corpus to 14 fixtures / 42 comparisons. Visual review subsequently found
a seventh failure hidden by aggregate pixel metrics, described below.

The [CSS Text whitespace rules](https://www.w3.org/TR/css-text-3/#white-space-phase-2)
distinguish unconditional hanging at soft wraps from conditional hanging before
forced breaks. Preserved trailing whitespace must not create a new line merely
because its advance overflows. Overlong words must respect allowed break points.
The reference cases must cover both fit and overflow before forced breaks.

## Next runtime seam

The legacy GlyphLine breaker treats the final whitespace interval as another
word and uses emergency glyph splitting when a word exceeds the width. Native
Text.break_lines_for_layout also has no explicit pre-wrap policy or original
text input. Its current opt-in nowrap policy cannot distinguish normal wrapping
from pre-wrap: both compile to native TextWrap::Wrap.

Use an explicit per-text policy rather than changing every font/scene globally.
The host needs a deterministic mapping from authored text to its Text object and
policy, carried alongside the checked capability. Existing source-map entries
identify the layout and text runs but do not explicitly expose the Text object
or whitespace policy. Investigate extending that metadata or an equivalent
requirements mapping, then verify missing/invalid policy mappings are rejected.
Normal and pre-wrap text must coexist in the same scene without one changing
the other's interpretation.

The policy should preserve the original source characters while building line
ranges, retain trailing spaces without creating blank lines, use the correct
hanging width for alignment, and avoid unsolicited glyph-level breaks inside
words. Preserve ordinary raw Rive behavior. Add unit and real-renderer controls
for all changed paths before qualification.

Tabs are retained as open work. Current tab advances are computed during font
shaping relative to the paragraph start; a soft wrap requires recomputing them
relative to the new line before fitting following text. The compiler explicitly
rejects tabs in visible pre-wrap text as unsupported-wrapped-tab. The exact
reproducer is prewrap-tabs-after-soft-break in deferred-cases.json. This is an
internal implementation dependency, not an external blocker, and must not be
silently replaced by fixed spaces or browser-baked positions.

## Visual review exposed a false pass

The first full run reported 598/605, including six pre-wrap failures plus A09.
However, prewrap-forced-hang at 240px dropped the entire first word (`one`) in
native rendering. Its whole-scene mismatch ratio was only 0.00151; element
regional error was 3.28, both below the existing budgets. Inspecting the original
browser/native images confirmed the missing word rather than a contact-sheet
rendering issue. Chromium places it at x=8; native paints none there.

The browser harness now supports optional textRegions fixtures: an element id
plus UTF-16 start/end offsets in its first text node. DOM Range supplies the reference
rectangle, which uses the existing local RGB/interior budgets. Invalid or empty
ranges fail explicitly. This does not modify compiler input or Rive geometry.
The check records its rectangles in metrics for review.

For the forced-hang fixture, the first word's range is x=8, y=11,
width=34.84375, height=24. Native regional error is 47.87 and interior error 55.31,
so the missing text now fails automatically. The same probe passes at 390/768px in the native-glyph profile.
No tolerance was changed. The targeted red result is in
/tmp/html-prewrap-region.log and prewrap-text-region/gallery.html.

Both pixel and geometry failures remain acceptance cases for a runtime fix.
Forced-break overflow alignment is therefore open work in addition to trailing
blank lines and emergency word splitting. A generic negative-origin clamp alone
will not establish correct soft-wrap/hanging semantics; preserve the distinct
forced-break and soft-break tests.

## Validation

Final results with the tight text-region check:

- Full native-glyph visual run: **597/605**. Failures: existing A09 at 240px;
  prewrap-basic/center/right/space-only at 240px; prewrap-long-word at 240/390px;
  and prewrap-forced-hang at 240px.
- Focused corpus: **35/42** native-glyph and **33/42** vector. The two additional
  vector failures are forced-hang at 390/768px: the word is present, but its
  regional/interior RGB error is about 7.0/8.4, exceeding the unchanged interior
  limit of 6. Original browser/vector images were inspected. These are separate
  rasterization evidence alongside A07/Q09, not new whitespace geometry failures.
- All **555** native scene PNGs from tabs-full are byte-identical. All **597**
  browser/native pairs from the initial pre-wrap run are also identical after
  adding text regions; only the verification result changes.
- All 67 module Rust tests, the runtime regression, five JS/WASM tests,
  checked-host requirements, TypeScript, two gallery tests, module Clippy,
  boundary and 27/27 renderer-state controls pass.
- Visually inspected all 42 new pairs and their differences. The multiline
  240px case was inspected as original PNGs because its contact-sheet crop did
  not show its final line. The missing-word and wider vector cases were also
  inspected as originals. Existing tolerances were not widened.

Commands and logs:

- CARGO_INCREMENTAL=0, NUXIE_HTML_REVIEW_DIR pointing to prewrap-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`:
  /tmp/html-prewrap-gate.log (initial full gate, before the text-region check).
- After adding text regions, `npm --prefix tools/html-to-riv run test:report`
  and `NUXIE_NATIVE_GLYPHS=1 npm --prefix tools/html-to-riv test` with that same
  review directory: /tmp/html-prewrap-gallery.log and
  /tmp/html-prewrap-final-visual.log. Compiler/runtime code was unchanged.
- `NUXIE_NATIVE_GLYPHS=0 npm --prefix tools/html-to-riv test -- --grep prewrap-
  --output=test-results-prewrap-vector-final`, review directory prewrap-vector:
  /tmp/html-prewrap-vector-final.log.
- Module Clippy with native-glyph-controls, boundary and standalone
  glyph-state-control.mjs: /tmp/html-prewrap-clippy.log,
  /tmp/html-prewrap-boundary.log and /tmp/html-prewrap-state.log.
- Public red/green and initial focused pixels: /tmp/html-prewrap-red.log,
  /tmp/html-prewrap-contract.log and /tmp/html-prewrap-probe.log.

Durable artifacts under output/playwright/html-to-riv: prewrap-full/gallery.html,
prewrap-vector/gallery.html, prewrap-full/baseline-comparison.json,
prewrap-full/text-region-comparison.json, twelve pair contact sheets, three diff
sheets and original multiline/forced-hang PNGs. The initial failure gallery is
prewrap-probe; the targeted tight-region gallery is prewrap-text-region.

A17 remains partial. The next implementation is the checked per-text layout
policy described above, followed by line-relative wrapped tabs. Keep all seven
semantic failures and two vector-only controls as acceptance evidence.



## Occurrence mapping prerequisite

The checked per-text envelope is now implemented and exercised by the existing
nowrap/pre policy. Version 2 carries actual Text object IDs, separate from the
layout/source map. Version-1 published scenes retain their previous behavior.
See [text-policy-review.md](text-policy-review.md) for the contract and evidence.
No pre-wrap behavior is claimed by this increment: its dedicated policy and
line-break implementation remain next, followed by line-relative tab stops.

## Line-breaking follow-up

The opt-in pre-wrap policy now fixes the seven original semantic/paint failures
in the native-glyph profile. See [prewrap-policy-review.md](prewrap-policy-review.md)
for the expanded corpus, compatibility controls and remaining work. Earlier
counts in this receipt describe the direct mapping before that runtime change.
