# Language casing

A19 adds inherited HTML lang, empty-value reset, and contextual tr/az/lt/el
upper/lower casing. Primary subtags are case-insensitive. Unknown languages
retain default behavior. Capitalize matches pinned Chromium 153.0.8010.12,
including Ijsselmeer for Dutch and Istanbul for Turkish; no locale-titlecase
claim is made. No editor integration or new runtime capability is introduced.

## Evidence so far

- locale-reference.json contains 36 independent browser logical-text references.
  Rust checks them and locale-reference.mjs rechecks installed Chromium in run.sh.
- Public compiler tests cover inherited Turkish, explicit empty/en overrides,
  Lithuanian inserted dots, source boundary offsets and br remapping.
- Exact-offset regressions compare traced text with upstream ICU output and check
  deletions, insertions, expansion, final sigma and empty text.
- The initial lang rejection was established by source inspection; no claim of
  a recorded failing public test before the implementation is made.
- ICU 2.3.0's mapping loop has a narrow source-boundary observer extension under
  vendor/icu_casemap-2.3.0-nuxie-offsets. Casing rules/data are unchanged. The
  additional titlecase helper is not used by this compiler.

## Validation results

Sixteen locale fixtures cover Turkish/Azeri, Lithuanian combining accents,
Greek casing, regional/script tags, inheritance/reset, capitalization, breaks,
spacing/alignment and pre-line. Run at 240/390/768px with same-scene resize.

Full command: CARGO_INCREMENTAL=0 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/locale-initial" bash tools/html-to-riv/validation/run.sh native-glyphs

Full gate: **923/924 passed**; only prior A09 em-layout-cascade at 240px fails
(449 mismatched pixels, ratio 0.005846354166666666). All **48/48 new native-glyph
comparisons pass**, with exact geometry. All **867 older native PNGs are byte
identical** to capitalization's baseline. All 48 new pairs were inspected in
12 contact sheets: casing, combining accents, line breaks, spacing and alignment
agree at all widths.

Vector command: NUXIE_NATIVE_GLYPHS=0 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/locale-vector" npm --prefix tools/html-to-riv test -- --grep locale- --output=test-results-locale-vector

**48/48 vector comparisons pass**, with exact geometry. The combining-accent,
script-tag and Greek groups were additionally visually inspected at each width.
No pixel tolerances were widened. Earlier vector raster gaps remain open.

Seventy-five module tests, six runtime CSS regressions, complete native/WASM
corpus parity (including these fixtures), checked-host tests, TypeScript, both
logical reference scripts, two gallery tests, Clippy, boundary validation and
27 renderer-state controls pass. The ICU observer formatting was subsequently
normalized with rustfmt; no semantic change followed the gate.

Artifacts: output/playwright/html-to-riv/locale-initial and locale-vector hold
standalone galleries, review.json summaries and locale-WIDTH-GROUP.png contacts.
Logs: /tmp/html-locale-{gate,vector,clippy,boundary,state,review,vector-review}.log.

This qualifies the documented inherited language/casing subset using the pinned
Inter font. It does not claim arbitrary language typography or font coverage.
Width/kana remain open; width-kana-research.md records the browser reference gap
and next investigation. Combined transform coverage is now 168/168 native glyph
and 166/168 vector; A19 remains partial overall.
