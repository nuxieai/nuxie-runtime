# Letter spacing (A12, partial)

The compiler maps signed px/em/rem lengths to native TextStylePaint letterSpacing.
Computed lengths inherit without rescaling at a child's new font size; font
shorthand does not reset the property. Normal/initial become zero within the
non-justified profile. Percentages and other units remain rejected. Zero omits
the native property, preserving previous default output.

The public compile regression failed before the mapping and passes after it.
It compares actual text scene bytes, with an embedded font, for em/rem-to-pixel
conversion, signed values, inheritance, unset, initial and normal. Unmatched
unsupported declarations still fail with diagnostics.

Initial seven fixtures pass 21/21 experimental glyph browser comparisons.
The expanded full gate adds multiple combining marks and intrinsic row text
with one/two/three characters. These distinguish per-glyph spacing from spacing
between character clusters and expose trailing-width behavior. The runtime
currently adds spacing to each shaped glyph advance; no shaper changes have
been made for A12 yet.

Full experimental glyph gate: **307/311**. All 49 Rust tests, five JS/WASM
tests, two gallery checks, TypeScript, Clippy and pure-runtime boundary pass.
The 27 renderer-state controls pass separately. New-only vector validation also
passes 24/27; both profiles fail the three multiple-mark comparisons. All 27
new source-box geometry comparisons pass, but that does not verify glyph spacing.

All 27 browser/native/diff triples in the glyph profile were visually inspected.
Positive/negative spacing, inherited lengths, alignment and one/two/three-letter
intrinsic widths agree within the unchanged gates. The multi-mark specimen shows
incorrect mark placement and centered text extent. The other full-suite failure
is the existing A09 fractional shape-edge case.

For the glyph multi-mark case at 240/390/768px, the text interior RGB errors are
23.2437, 13.8892 and 6.9092 against limit 6. The vector equivalents also fail
(22.2181, 13.2774, 6.6058). Wider blank backgrounds do not hide the local error.
Chromium 153.0.8010.12 at DPR 1; actual Rust Metal replay. No thresholds changed.

Source evidence: font_hb.rs adds letter spacing to every shaped glyph advance.
The pinned C++ src/text/font_hb.cpp line 1029 does the same. Thus a blanket shaper
change would change existing Rive behavior. Next: investigate an explicit
browser-compatible cluster-spacing path or another native representation while
preserving the pinned default; do not paper over this by baking browser geometry
or silently claiming all Unicode clusters work. The fixture remains in the main
corpus and A12 remains partial.

Run:

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
```

Semantic reference: [CSS letter spacing](https://www.w3.org/TR/css-text-3/#letter-spacing-property).
No visual thresholds or reference font/reset rules changed.

New-only vector check:

```sh
cd tools/html-to-riv
env -u NUXIE_NATIVE_GLYPHS npm test -- --grep 'letter-spacing-' --output=test-results-letter-vector
```

Review artifacts: output/playwright/html-to-riv/letter-spacing-full/gallery.html
and review-{240,390,768}-{0,1,2}.png. Vector metrics are in
letter-spacing-vector/gallery.html. The checked-in fixtures and compile tests
are the durable reproductions.
