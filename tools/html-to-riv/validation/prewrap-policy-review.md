# Pre-wrap line-breaking policy

A17 now has an opt-in runtime implementation. The seven previous semantic/paint
failures pass in the native-glyph lane. The focused corpus expands from 14 to 24
fixtures / 72 comparisons. Wrapped tabs remain explicitly rejected and are the
next implementation step; this receipt does not mark A17 fully qualified.

## Behavior and compatibility

Compiler output requests text-css-pre-wrap-v1 and maps css-pre-wrap-v1 to its
actual Text objects using requirements version 2. The checked host installs
Text::set_css_pre_wrap(true) before layout. Missing mappings/capabilities,
conflicting policies for one object, invalid targets, and version-1 claims of
this new capability are rejected. A normal Text sharing the same font keeps
ordinary Rive wrapping. Existing version-1 nowrap scenes remain readable.

The new css_pre_wrap runtime module builds lines from original source characters
and shaped glyph positions. It preserves the full glyph/source ranges, including
spaces, while keeping separate content and trailing-space widths for fit and
alignment. Empty forced lines retain their font metrics. It does not apply
emergency glyph splitting inside an unbreakable word. Overflow alignment is
clamped for the tested LTR profile. The public legacy GlyphLine/Text breakers
are unchanged; the Text occurrence chooses this policy for drawing, measurement,
font fitting and modifier measurement paths.

The [CSS Text whitespace rules](https://www.w3.org/TR/css-text-3/#white-space-phase-2)
distinguish soft-line trailing spaces from trailing spaces before forced breaks
or block endings. Soft-line spaces hang outside alignment width; at forced
breaks, the fitting portion still contributes to alignment. This is why merely
trimming every line or clamping all negative origins was insufficient.

Break opportunities come from pinned
[unicode-linebreak 0.1.5](https://docs.rs/unicode-linebreak/0.1.5/unicode_linebreak/),
a dependency-free UAX #14 implementation using Unicode 15.0 tables. UTF-8 offsets
are converted to Rive Unicode-scalar indices, and only actual shaped cluster
boundaries can break. General Unicode break opportunities are not a claim of
browser-complete language behavior: dictionary segmentation, newer Unicode
rules, CSS language tailoring, and broad bidi/script qualification remain open.

## Regression evidence

The runtime test first failed on a long unbreakable word: legacy produced three
lines at 224px while the reference requires one overflowing line. It now passes
with trailing/all-space and forced-break controls. It checks repeated resizing,
restoration after disabling the policy, and a fresh default occurrence. A second
runtime regression exercises actual Text layout measurement, control_size,
clearing/reloading the font asset, and a separate Text sharing that asset.
The standard validation script runs both new regressions and the existing
nowrap compatibility regression.

Public compiler contracts check the new capability, occurrence mapping,
version rejection, inheritance/reset behavior and unchanged source-break
identities. The checked-host test adds mixed normal/nowrap/pre/pre-wrap policies.
Complete accepted-corpus JS/WASM/native parity includes requirements as well as
Rive bytes and source maps.

The ten new visual fixtures cover a long word after a short word, overflowing
leading spaces, hyphen opportunities, nonbreaking spaces, Unicode trailing
spaces, Open Sans ligatures, negative spacing, whitespace-only lines and mixed
normal/pre-wrap/nowrap in one scene. Each compiled scene is resized at
240/390/768px. All 72 focused native-glyph comparisons pass without tolerance
changes. The former missing-word fixture retains its tight DOM text-region
check; its first word is now present at the expected origin.

Visually reviewed all 72 browser/native pairs in eighteen contact sheets.
Their crop heights use both actual root bounds so multiline/blank-line results
remain visible. The original overflowing-word and forced-hang fixtures now show
correct line count and visible text. The mixed-mode case preserves each sibling's
wrapping semantics. Final full/vector results and artifact comparisons follow.

## Commands and artifacts

- Public/runtime red: /tmp/html-prewrap-policy-red.log; initial green runtime:
  /tmp/html-prewrap-policy-unit.log.
- Focused native-glyph run: /tmp/html-prewrap-policy-probe.log, with
  NUXIE_NATIVE_GLYPHS=1 and NUXIE_HTML_REVIEW_DIR pointing to prewrap-policy-probe,
  `npm --prefix tools/html-to-riv test -- --grep prewrap-
  --output=test-results-prewrap-policy-probe`.
- Full gate: CARGO_INCREMENTAL=0, review directory prewrap-policy-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`;
  /tmp/html-prewrap-policy-gate.log.
- Module Clippy, boundary and independent state controls:
  /tmp/html-prewrap-policy-{clippy,boundary,state}.log.
- Contract and checked-host checks: /tmp/html-prewrap-policy-{contract,host}.log.

Artifacts live under output/playwright/html-to-riv. The focused gallery and
prewrap-{240,390,768}-{0..5}.png contact sheets are in prewrap-policy-probe.
The final full gallery is prewrap-policy-full/gallery.html.


## Final gate and residual failures

- Full native-glyph gate: **634/635**. Only the existing A09 em-layout-cascade
  fractional-edge case fails. All 72 pre-wrap comparisons pass.
- Exactly the seven repaired old pre-wrap PNGs changed. **590** old native PNGs
  are byte-identical, including all **555** outside pre-wrap. The corpus adds
  30 comparisons. All **72** focused browser/native pairs are identical in the
  probe and final full run. Baseline/probe comparison JSON is in
  prewrap-policy-full.
- **68** module Rust tests, **three** runtime regressions, five JS/WASM tests
  with complete accepted-corpus parity, checked host, TypeScript, two gallery
  checks, module Clippy, boundary and **27/27** state controls pass.
- The checked-host test was strengthened after the full gate: invalid target
  mutations retain the other valid policy entries, so they must fail specifically
  with invalid-text-policy-target rather than an unrelated missing-policy error.
  It also checks conflicting policies and a missing pre-wrap mapping while a
  valid nowrap mapping remains. The final standalone host run passes; no runtime
  behavior or compiler code changed after the full gate.
- Focused vector: **64/72** pixels, **72/72** geometry. All eight pixel failures
  were visually inspected alongside browser images and difference images. Text
  is present, with matching line count/alignment; differences follow glyph edges.

Vector failure inventory (unchanged budgets):

| Fixture | Widths | Exceeded metric |
| --- | --- | --- |
| prewrap-long-word | 240, 390 | Text interior RGB 7.12 / 6.94, limit 6 |
| prewrap-forced-hang | 240, 390, 768 | First-word interior RGB 8.35 / 8.47 / 8.44, limit 6 |
| prewrap-word-after-short | 768 | Text interior RGB 6.17, limit 6 |
| prewrap-policy-mixed | 240, 390 | Whole-scene mean RGBA 1.63 / 1.17, limit 1; 240px mismatch ratio also exceeds 1% |

These are retained renderer qualification gaps alongside A07/Q09. No region,
fixture height, tolerance or expected image was changed to hide them. The new
correct single-line long-word layout concentrates its raster error rather than
retaining the old, incorrect multiline geometry. The mixed fixture exercises
realistic accumulated text density.

The final runtime test helper also retains the existing pinned-font override
and upstream fixture fallback, so the new tests do not introduce a mandatory
new environment variable. The three CSS runtime regressions pass again in
/tmp/html-prewrap-policy-runtime-final.log.

Vector command: NUXIE_NATIVE_GLYPHS=0, review directory prewrap-policy-vector,
`npm --prefix tools/html-to-riv test -- --grep prewrap-
--output=test-results-prewrap-policy-vector`; log /tmp/html-prewrap-policy-vector.log.
The vector gallery, failure-summary.json, three failure contact sheets and
original failed browser/native/diff PNGs live in prewrap-policy-vector.
Final host log: /tmp/html-prewrap-policy-host-final.log. Full/probe image receipt
logs: /tmp/html-prewrap-policy-{final-review,visual-review}.log.

A17 remains partial. Next implement line-relative default tabs after soft breaks,
then reassess this item's remaining renderer and language qualification work.

## Wrapped-tab follow-up

The wrapped-tab implementation is now tracked in
[wrapped-tabs-review.md](wrapped-tabs-review.md). The rejection and next-step
statements above describe the earlier non-tab increment.
