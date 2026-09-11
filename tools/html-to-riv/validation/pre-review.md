# White-space pre: preserved text and tab-stop seam

A16 remains partial until tabs are implemented and qualified. The current change
adds a WhiteSpace enum (normal/nowrap/pre), preserves pre source spaces/newlines,
uses native NoWrap plus the existing checked CSS nowrap alignment policy, and
keeps explicit br identities in Unicode-scalar offsets. Normal/initial restore
collapse behavior; inherit/unset copy the mode. No new runtime capability is
introduced for this increment.

The public contract failed first with unsupported-value. It now verifies
preserved offsets, inheritance/reset byte equivalence, difference from nowrap,
HTML CRLF normalization, and an explicit tab diagnostic. A native import test
checks exact preserved text concatenated across word-spacing runs. An attempted
CR numeric-reference case exposed the existing strict HTML parser rejection;
the accepted test uses literal CRLF instead. The parser policy was not relaxed.

The first 12 fixtures passed all 36 browser/native comparisons, including
leading/trailing spaces, empty lines, all-space/all-newline block text,
center/right overflow, spacing composition and explicit br. Four additional
fixtures test anonymous flex whitespace and inherited indentation. Nine of their
12 comparisons failed before the collector discarded whitespace-only anonymous
flex content. The corrected full corpus has 16 fixtures / 48 comparisons.

## Tab semantics and retained reproducer

CSS pre preserves spaces and forced segment breaks. Preserved tabs align to
position-dependent stops rather than a fixed character substitution. See the
primary [CSS Text whitespace rules](https://www.w3.org/TR/css-text-3/#white-space-rules)
and [tab positioning rules](https://www.w3.org/TR/css-text-3/#white-space-phase-2).
The parser normalizes source CR/CRLF; an encoded CR follows separate CSS rules,
but that invalid HTML reference is rejected by this compiler's strict parser.

A direct Chromium probe with pinned Inter, 20px/30px and default tab-size:8
measures the following X origins, relative to the block start:

| Prefix | Tab | Eight literal spaces |
| --- | --- | --- |
| i | 43.90625px | 48.59375px |
| MMMM | 87.8125px | 115.34375px |

The multiline probe resets the stops on the next line. Artifact:
output/playwright/html-to-riv/pre-tabs/browser.json. The compiler currently emits
unsupported-preserved-tab for visible pre text containing a tab; the exact
multiline source is retained in deferred-cases.json as pre-positioned-tabs.
This is an implementation dependency to pursue, not an external blocker.

Next runtime work must retain tab characters/source identity, suppress tab ink,
compute advances from the line position using the font's space/zero metrics,
compose with letter/word spacing and font replacement, and preserve the behavior
through resize. It must distinguish ordinary legacy Rive tab behavior and be
requested by a checked runtime capability. Do not replace tabs with eight spaces
or browser-measured positions.

## Validation

Final results:

- Full native-glyph gate: **529/530**; only the existing A09 em-layout-cascade
  at 240px fails. All **48/48** pre comparisons pass in both renderer profiles.
- All **474** native scene PNGs from nowrap-policy-full are byte-identical.
  The new corpus adds 48 images and changes no earlier scene image.
- All 65 module Rust tests, the runtime policy regression, five JS/WASM tests,
  checked-host requirements, TypeScript, two gallery tests, module Clippy,
  runtime-boundary and 27/27 renderer-state controls pass.
- The deferred tab fixture was added after the full script's Rust phase;
  deferred_corpus_rejections_remain_explicit was rerun and passes with it.
- Visually inspected all 48 browser/native pairs and difference images. Blank
  block lines occupy the expected space; anonymous flex whitespace occupies no
  line box. Preserved spacing and short/overlong line alignment match. A contact
  sheet made the 390px spacing row look compressed, so its original native PNG
  and metrics were inspected directly: spacing is intact, with six mismatched
  pixels and mean-channel error 0.0444. No tolerance changed.

Commands and logs:

- CARGO_INCREMENTAL=0 and NUXIE_HTML_REVIEW_DIR pointing to pre-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`:
  /tmp/html-pre-gate.log. The visual failure stops the script before state
  controls, which were run separately.
- `NUXIE_NATIVE_GLYPHS=0 npm --prefix tools/html-to-riv test -- --grep pre-
  --output=test-results-pre-vector`, review directory pre-vector:
  /tmp/html-pre-vector.log.
- Module Clippy with native-glyph-controls, boundary check and standalone
  glyph-state-control.mjs: /tmp/html-pre-clippy.log, /tmp/html-pre-boundary.log,
  /tmp/html-pre-state.log.
- Public-contract red/green and deferred controls: /tmp/html-pre-red.log,
  /tmp/html-pre-green.log, /tmp/html-pre-deferred.log.

Durable artifacts under output/playwright/html-to-riv: pre-full/gallery.html,
pre-vector/gallery.html, pre-full/baseline-comparison.json, twelve pair contact
sheets and three diff sheets. Initial red/green galleries: pre-probe and
pre-flex-red. The non-tab subset passes, but A16 remains partial. Continue with
real runtime tab stops rather than advancing past the missing semantics.



## Subsequent tab qualification

The tab rejection above describes the earlier checkpoint. Default tabs are now
implemented and qualified through text-css-tabs-v1; the retained deferred
reproducer moved to the accepted tabs-default case. See
[tabs-review.md](tabs-review.md) for the implementation, browser boundary,
33/33 comparisons in both renderers and final A16 qualification scope.
