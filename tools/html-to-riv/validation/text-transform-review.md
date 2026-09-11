# Text transforms: default uppercase/lowercase

A19 now implements none/uppercase/lowercase with inheritance and CSS-wide resets.
Capitalization, locale tailoring and width/kana transformations remain open.

## Semantics and source identities

The [CSS Text transform rules](https://www.w3.org/TR/css-text-3/#text-transform-property)
require full Unicode case mappings after whitespace normalization and before
layout, without rewriting underlying author content. This implementation uses
Unicode 17.0 Rust standard-library whole-string casing, including contextual
Greek final sigma. See [Rust string casing](https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase).
A compile-time assertion prevents unreviewed Unicode table upgrades. Native
Rust 1.97.1 and rustup/WASM Rust 1.94.1 both report Unicode 17.0.0 here.

Presentational casing is applied before glyph validation and run emission.
SourceNode.text_transform retains normalized pre-casing source, rendered text,
and an array mapping each source scalar boundary to its rendered offset. This
keeps expanding mappings such as sharp-s to SS explicit. Break IDs retain their
identity and point to rendered LF offsets. No transform means no new map field.
The original raw authoring HTML/CSS remains caller-owned. Runtime layout still
responds to width changes, and requires no casing-specific host capability.

The initial public test failed with unsupported-property. It now compares
transformed and explicitly cased Rive bytes, verifies expanded scalar offsets,
checks contextual sigma and dotted-I expansion, and exercises inherit/unset/
initial/none. The TypeScript consumer covers the optional mapping structure.

## Qualification corpus

Twenty fixtures cover both cases, expanding mappings, Greek context, dotted-I,
explicit br, pre-line/pre-wrap tabs, nowrap overflow, positive/negative spacing,
combining marks, center/right alignment, inheritance/reset, intrinsic sizing,
hidden text and mixed styles. Each is compared at 240/390/768px using real native
rendering. Native/WASM corpus parity includes all new maps and emitted bytes.
No geometry or pixel limit is widened.

Commands and logs:
- Red contract: /tmp/html-transform-red.log.
- Passing contract: /tmp/html-transform-contract.log.
- Full gate: CARGO_INCREMENTAL=0, review directory transform-initial,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`;
  /tmp/html-transform-gate.log.
- Clippy/boundary: /tmp/html-transform-{clippy,boundary}.log.

## Results and visual review

Native-glyph result: **60/60**, full gate **815/816**, retaining only the previous
A09 em-layout-cascade240 failure. Maximum new geometry error is 0.01171875 CSS px
(limit 0.1). All **747 previous native PNGs are byte-identical**. All 60 new
browser/native pairs were visually inspected in fifteen contact sheets.

Vector result: **59/60**, with all geometry passing. transform-mixed240 exceeds
mean RGBA error (1.569281 > 1); its mismatch ratio 0.008529 remains within 0.01.
Inspected the browser/vector/difference image: words and line positions agree,
and differences follow glyph edges. This stays a failure; no budget changed.
Vector command: NUXIE_NATIVE_GLYPHS=0, review directory transform-vector,
`npm --prefix tools/html-to-riv test -- --grep transform-
--output=test-results-transform-vector`; /tmp/html-transform-vector.log.
An initial anchored grep selected no tests; it was corrected before validation.

All 70 module Rust tests, six CSS runtime regressions, five JS/WASM parity tests,
checked-host test, TypeScript, two gallery tests, Clippy and boundary pass.
All 27 renderer-state controls pass (/tmp/html-transform-state.log).
The compile-time Unicode guard was added after the full gate's build step;
it changes no runtime behavior. Native Clippy compiled it, and the WASM build,
complete corpus parity and typed consumer were rerun successfully afterward
(/tmp/html-transform-wasm-guard.log).

Artifacts under output/playwright/html-to-riv:
- transform-initial/gallery.html and review.json: complete native run, baseline
  comparison and geometry metrics.
- transform-initial/transform-{240,390,768}-{0..4}.png: all inspected pairs.
- transform-vector/gallery.html, review.json and failures-240.png: vector result
  and inspected raster failure.

A19 remains partial: capitalize/titlecase boundaries, language-tag inheritance
and locale-sensitive casing, full-width/full-size-kana, broader script/font
qualification and the vector raster failure remain open. Continue with
capitalization and locale tailoring.
