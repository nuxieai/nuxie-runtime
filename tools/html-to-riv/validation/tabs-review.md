# Default preserved tab stops

A16 tab implementation targets the pinned Chromium reference with default
(tab-size:8) stops. Custom tab-size syntax remains unsupported. The initial
native shaper placed X at 17.8125px after i+tab instead of Chromium's 43.90625px;
that failing runtime regression is in /tmp/html-tabs-red.log.

HbFont now has an opt-in experimental CSS tab policy. It shapes a tab with an
inkless space glyph while preserving the original text index and source text.
Before cumulative positions are finalized, each tab's advance is computed from
the current paragraph position and eight space advances including spacing.
Paragraph resets give each new line independent stops. Word-spacing run
partitioning includes tabs so their stop interval receives letter plus word
spacing. No browser positions are baked into the scene.

The policy survives with_options, spacing/space-break policy composition and
FontAsset replacement or clear/reload. Ordinary decoded fonts default to legacy
behavior. The compiler emits text-css-tabs-v1 for preserved tab text, and the
checked host validates this requirement before import and installs the policy
before shaping. A font without a positive-width inkless space glyph is rejected
as unsupported-tab-font. Compiler source runs retain U+0009.

## Reference boundary

The [current CSS Text draft](https://www.w3.org/TR/css-text-3/#white-space-phase-2)
describes a half-ch minimum tab advance. Pinned Chromium 153 instead retains a
tab one space-width before the next stop on that stop. The policy uses half a
space for this boundary, matching the current browser target; this difference
from the draft is intentional and explicitly tested. It is not a claim of
universal browser/spec equivalence.

Pinned Inter 20px browser measurements, relative to the block start:

| Text prefix before X | Letter spacing | X origin |
| --- | --- | --- |
| i + tab | normal | 43.90625px |
| MMMM + tab | normal | 87.8125px |
| seven spaces + tab | normal | 43.90625px |
| eight spaces + tab | normal | 87.8125px |
| i + two tabs | normal | 87.8125px |
| i + tab | 1px | 51.90625px |
| MMMM + tab | 1px | 103.8125px |

The native regression passes these cases, newline resets, inkless tab paths,
unchanged legacy shaping, derived font options, retained asset policy and
composition with the other experimental text policies. Initial real-renderer
probe: 24/24 passing. Full qualification adds overflow, br and Open Sans cases,
for 33 comparisons at 240/390/768px. The former deferred pre-positioned-tabs
reproducer is now in the accepted tabs-default fixture.

## Final validation

- Full native-glyph gate: **562/563**, with only the existing A09 failure.
- Focused tab corpus: **33/33** in native-glyph and vector profiles.
- All **522** native scene PNGs from pre-full are byte-identical; 33 are new.
- All 66 module Rust tests, the standard runtime policy regression, five JS/WASM
  tests, checked-host requirements, TypeScript, two gallery tests, module Clippy,
  runtime-boundary and 27/27 renderer-state controls pass.
- All 33 browser/native pairs and difference images were visually reviewed.
  Stops align, tab glyphs remain invisible, overflowing text retains its start,
  and short lines retain alignment. Observed differences are rasterization edges;
  no visual threshold was changed.

Commands and logs:

- CARGO_INCREMENTAL=0, NUXIE_HTML_REVIEW_DIR pointing to tabs-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`:
  /tmp/html-tabs-gate.log. Its A09 visual failure prevents the final state-control
  step, which was run separately.
- `NUXIE_NATIVE_GLYPHS=0 npm --prefix tools/html-to-riv test -- --grep tabs-
  --output=test-results-tabs-vector`, review directory tabs-vector:
  /tmp/html-tabs-vector.log.
- Module Clippy with native-glyph-controls, runtime-boundary check and standalone
  glyph-state-control.mjs: /tmp/html-tabs-clippy.log, /tmp/html-tabs-boundary.log,
  /tmp/html-tabs-state.log.
- Initial runtime red/green, expanded native regression and initial pixels:
  /tmp/html-tabs-red.log, /tmp/html-tabs-green.log, /tmp/html-tabs-runtime-tests.log,
  /tmp/html-tabs-probe.log.

Durable artifacts under output/playwright/html-to-riv: tabs-full/gallery.html,
tabs-vector/gallery.html, tabs-full/baseline-comparison.json, nine browser/native
contact sheets and three difference sheets. A16 is qualified for the documented
LTR pre subset with default tab stops. Custom tab-size is intentionally
unsupported, and broader bidi/script fidelity remains separate work. Next: A17.
