# Line-relative tabs in pre-wrap

A17 now accepts default tabs after soft wrapping. The compiler preserves U+0009
and the original source identities. The original deferred reproducer is now an
accepted fixture, and 15 additional fixtures cover dynamic widths, all physical
alignments, leading/repeated/trailing tabs, forced newlines and br, letter/word
spacing, negative spacing, Open Sans, blank lines, near-stop placement and a
mixed normal/nowrap/pre/pre-wrap scene.

## Runtime contract

The new text-css-wrapped-tabs-v1 capability explicitly requires both
text-css-tabs-v1 and text-css-pre-wrap-v1 in a version-2 envelope. Its purpose is
to make older hosts reject the newly supported combination: recognizing the
older capabilities alone does not establish line-relative tab behavior.
The host still validates each pre-wrap Text target, installs its occurrence
policy, and enables CSS tab shaping on the font assets. Version-1/invalid
combinations fail before rendering. Native/WASM output carries identical metadata.

The font exposes an opt-in experimental_css_tab_advance hook. The HbFont
implementation uses the established eight-space/spacing calculation and pinned
Chromium half-space minimum; default backends return no override. Pre-wrap
layout recalculates advances as it fits tokens. When a token moves to a new
line, only that token is retried against the new origin. Positions remain
cumulative across a paragraph so adjacent glyph-range boundaries stay coherent.
Actual shaped positions/advances are updated for drawing as well as measurement.
Tabs and glyph/source identities are never expanded into fixed spaces.

Existing font validation still requires a positive-width inkless space glyph.
Custom tab-size, CSS Grid, editor integration and broader language/renderer
qualification are not added by this change. The raw Rive defaults and the
non-tab pre-wrap layout path retain their previous behavior.

## Tests and visual evidence

The public contract first failed with unsupported-wrapped-tab. It now compiles
and verifies all three required capabilities. A host providing only the earlier
pre-wrap/tab capabilities is rejected. Invalid capability combinations are also
rejected. The checked native host compiles/imports a wrapped-tab scene and
retains its existing malformed-manifest and target checks.

The new runtime regression checks Chromium's measured X origin after the
original soft break: line index 1, x=131.71875 at text width 224px. It contrasts
the incorrect paragraph-relative result, resizes the same shaped runs through
224/752/224px without losing the correct stop, checks a fresh default occurrence,
verifies glyph/source IDs are unchanged, and confirms tab glyphs have no ink.
The standard gate also runs the existing pre-wrap/nowrap and font replacement
regressions. The browser fixture retains a tight region around X so missing it
cannot hide in the whole-scene pixel average.

Focused native-glyph result: **48/48**, from 16 fixtures at three widths.
Visually inspected all 48 browser/native pairs in twelve contact sheets. The
br/combined-spacing originals at 240px were inspected separately because the
multi-image contact-sheet presentation was unclear; all text is present and
aligned in the original images. Source break identities and same-scene resizing
remain part of the public and browser gates. No tolerance was widened.

## Commands and artifacts

- Red compiler contract: /tmp/html-wrapped-tabs-red.log.
- Initial runtime, public contract and host tests:
  /tmp/html-wrapped-tabs-{runtime,contract,host}.log.
- Focused run: NUXIE_NATIVE_GLYPHS=1, review directory wrapped-tabs-probe,
  `npm --prefix tools/html-to-riv test -- --grep prewrap-tabs-
  --output=test-results-wrapped-tabs-probe`;
  /tmp/html-wrapped-tabs-probe.log.
- Full gate: CARGO_INCREMENTAL=0, review directory wrapped-tabs-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`;
  /tmp/html-wrapped-tabs-gate.log.
- Clippy and boundary: /tmp/html-wrapped-tabs-{clippy,boundary}.log.

Artifacts are under output/playwright/html-to-riv. wrapped-tabs-probe contains
the gallery, twelve prewrap-{240,390,768}-{0..3}.png contact sheets and original
br/combined-spacing browser/native images. The final gallery is
wrapped-tabs-full/gallery.html. Full/vector outcomes are recorded below.


## Long whitespace sequences

Review found that fitting consecutive tabs could repeatedly scan the same
trailing whitespace, even though glyph positioning retried each token at most
twice. Each glyph now caches the last non-hanging prefix boundary, so a candidate
line's content width is retrieved without rescanning that prefix. A 16,384-tab
regression verifies one hanging whitespace line followed by X, without losing
the following text. All five CSS runtime regressions pass; the focused runtime
run completed in 0.32s on this machine (not a portable performance guarantee).
Evidence: /tmp/html-wrapped-tabs-stress.log.

The initial full gate before this optimization passed 682/683, with only the
existing A09 fractional-edge failure. All 627 earlier native PNGs were unchanged;
48 tab comparisons were added. Its gallery is preserved in wrapped-tabs-initial.
The final full gate revalidates the cache optimization rather than relying only
on the focused stress test. Final log: /tmp/html-wrapped-tabs-final-gate.log.


## Final results and remaining work

- Full native-glyph gate: **682/683**, only existing A09 em-layout-cascade fails.
  All **48/48** new tab comparisons pass. All **627** older native PNGs remain
  byte-identical. All **675** browser/native pairs match the initial full run,
  and all 48 match the visually reviewed focused run. See baseline-comparison
  and optimization-comparison JSON in wrapped-tabs-full.
- Complete pre-wrap vector run: **107/120**, including **43/48** new tab cases.
  All geometry is exact (maximum coordinate/size error 0). All **72** earlier
  vector PNGs are byte-identical; their eight existing failures remain.
- Five runtime regressions (including same-shaped-run resizing and the long-tab
  input), **68** module Rust tests, five JS/WASM tests with full corpus parity,
  checked-host rejection, TypeScript, two gallery tests, final module Clippy,
  boundary and **27/27** renderer-state controls pass.
- Inspected all five new vector failures with browser and difference images.
  Text is present and aligned; remaining differences follow glyph edges.

New vector failure inventory:

| Fixture | Widths | Exceeded metric |
| --- | --- | --- |
| prewrap-tabs-after-soft-break | 240, 390, 768 | X-region interior RGB 10.36, limit 6 |
| prewrap-tabs-mixed | 240, 390 | Whole-scene mean RGBA 1.75 / 1.26, limit 1; 240px mismatch ratio also exceeds 1% |

No tolerance, viewport, expected image or text region was widened to pass these
cases. They remain renderer qualification work alongside A07/Q09. A17 now has
120/120 passing glyph comparisons and 107/120 vector comparisons; broader
language qualification remains open as previously documented. The next authoring
feature is A18 pre-line.

Final commands/logs:

- Full gate: /tmp/html-wrapped-tabs-final-gate.log, review directory
  wrapped-tabs-full.
- Vector: NUXIE_NATIVE_GLYPHS=0, review directory wrapped-tabs-vector,
  `npm --prefix tools/html-to-riv test -- --grep prewrap-
  --output=test-results-wrapped-tabs-vector`; /tmp/html-wrapped-tabs-vector.log.
- Final Clippy/state/boundary: /tmp/html-wrapped-tabs-final-{clippy,state,boundary}.log.
- Image comparison and review: /tmp/html-wrapped-tabs-final-review.log;
  wrapped-tabs-vector/failure-summary.json, gallery.html and three failure
  contact sheets. Full/probe/initial artifacts remain available under
  output/playwright/html-to-riv.
