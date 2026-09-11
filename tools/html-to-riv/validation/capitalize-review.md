# Capitalization

A19 adds capitalize to the inherited text-transform operations. It preserves
existing case after each word head and retains normalized-source mappings.
Locale-sensitive transforms and width/kana conversions remain open.

## Browser behavior and implementation

Pinned Chromium 153.0.8010.12 uses simple titlecasing for untagged text. The
[Chromium capitalization implementation](https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/platform/text/capitalize.cc)
explains the separate word-boundary and single-code-unit casing behavior. The
[locale-aware case mapper](https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/renderer/platform/wtf/text/case_map.cc)
uses a different path for declared languages. Source review was corroborated
with direct browser measurements, not used as a substitute for them.

Measured compatibility choices: sharp-s/ligatures do not expand in capitalize;
supplementary-plane letters remain unchanged; titlecase digraphs do change;
letters after a numeric word head remain unchanged. Apostrophes, middots and
underscores preserve word continuity, whereas ASCII/fullwidth period and
ASCII/small/fullwidth colon split words. NBSP is a separator. Combining marks
around punctuation do not hide the following word's head.

The implementation uses pinned icu_casemap/icu_segmenter 2.3.0, compiled data and
dictionary segmentation, then simple-titlecases BMP word heads. See
[ICU4X case mapping](https://docs.rs/icu_casemap/2.3.0/icu_casemap/struct.CaseMapperBorrowed.html)
and [word segmentation](https://docs.rs/icu_segmenter/2.3.0/icu_segmenter/struct.WordSegmenter.html).
Only segmentation input normalizes NBSP and the five punctuation forms; rendered
characters and source indices remain intact. Numbers remain segment heads and
are not skipped. No runtime dependency or casing capability is introduced.

This intentionally matches observed Chromium behavior rather than claiming full
Unicode titlecase semantics. The source/rendered boundary map remains scalar-
identity for capitalize. Untagged upper/lower behavior is unchanged.

## Red tests and reference data

The public contract first failed with unsupported-value. Direct ICU4X segmentation
then failed foo.bar/a:b and their width variants. A locale-aware constructor did
not resolve this, so measured punctuation treatment is explicit. The extended
reference also exposed supplementary-plane casing differences. Those are now
preserved like Chromium. No reference expected value was derived from compiler
output. Logs: /tmp/html-capitalize-{red,reference,locale-probe,expanded-red,tailored,green}.log.

capitalize-reference.json stores 62 browser-measured source/rendered pairs.
The Rust unit test compares the casing function against them. The script
capitalize-reference.mjs independently rechecks installed Chromium's innerText;
run.sh invokes it before visual tests. References include cases whose fonts or
rendering are outside current qualification, so logical casing coverage is not
presented as glyph-rendering coverage.

## Qualification

Twenty fixtures add 60 resized browser/native comparisons: ordinary words,
retained tail case, sharp-s, digraphs, punctuation, apostrophes, numbers, prefix
symbols, Unicode spaces, combining marks, Greek, br, preserved whitespace,
spacing, alignment, inheritance and mixed styles.

Full gate: CARGO_INCREMENTAL=0, review directory capitalize-initial,
`bash tools/html-to-riv/validation/run.sh native-glyphs`;
/tmp/html-capitalize-gate.log. Clippy/boundary:
/tmp/html-capitalize-{clippy,boundary}.log.

## Results and visual inspection

Native-glyph capitalization: **60/60**. Vector: **59/60**. Geometry is exact in
all 60 comparisons. The full glyph gate is **875/876**, retaining only the prior
A09 failure; all **807 previous native PNGs are byte-identical**. Visually
inspected every new pair in fifteen contact sheets.

The vector failure is capitalize-mixed240: mean RGBA error 1.419473 exceeds 1;
mismatch ratio 0.006797 remains within 0.01. Inspected its browser/vector/diff
image: all text and line placement agree, with differences following glyph
edges. No pixel or geometry budget changed. This remains a qualification gap.

All 72 module Rust tests, six runtime regressions, five native/WASM parity tests,
checked-host test, TypeScript, 62 direct Chromium reference checks, two gallery
tests, Clippy and boundary pass. All 27 renderer-state checks pass separately
(/tmp/html-capitalize-state.log).

Vector command: NUXIE_NATIVE_GLYPHS=0, review directory capitalize-vector,
`npm --prefix tools/html-to-riv test -- --grep capitalize-
--output=test-results-capitalize-vector`; /tmp/html-capitalize-vector.log.

Artifacts under output/playwright/html-to-riv:
- capitalize-initial/gallery.html and review.json: full result, unchanged-image
  comparison, geometry and metrics.
- capitalize-initial/capitalize-{240,390,768}-{0..4}.png: all inspected pairs.
- capitalize-vector/gallery.html, review.json and failures-240.png: vector
  results and inspected raster failure.

The combined transform corpus is 120/120 native glyph and 118/120 vector.
A19 remains partial. Next: language attributes/inheritance and locale-sensitive
casing, then full-width/full-size-kana and broader font/script qualification.
