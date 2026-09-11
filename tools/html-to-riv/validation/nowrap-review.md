# White-space nowrap: direct mapping and overflow alignment

The compiler accepts normal/nowrap as inherited white-space modes. It emits
native Text.wrapValue=1 for nowrap and leaves normal bytes unchanged. Explicit
br/newline breaks survive; spaces, tabs and authored source newlines retain the
current collapse/trim policy. There is no new font or layout capability in this
initial mapping. Unsupported preserved-space modes remain explicit errors and
will be implemented as separate backlog items.

The public contract test failed before implementation and now covers inherited
nowrap, inherit/unset, normal/initial restoration, byte differences from normal,
and malformed/unsupported unmatched declarations. The full accepted corpus
provides native/WASM byte, source-map and runtime-requirement parity.

The initial 24 focused comparisons passed 22/24: centered/right overflow failed
at 240px. An intrinsic-width/100%-minimum participant experiment did not fix it
and was removed. Additional block-text controls reproduce the same failure,
rejecting the hypothesis that only anonymous flex-item sizing was responsible.
A direct Chromium DOM-range check measured the 302.09375px text range at the
224px box's left edge for both block and flex text with computed text-align:center.
There was no horizontal scroll. Native centers the wider line with a negative
start offset instead. Wider viewports fit and align correctly.

The retained corpus covers left/center/right, explicit br, collapsed whitespace,
letter+word spacing, preserved Unicode spaces, inheritance and block center/right.
The same Rive scene is imported and resized to 240/390/768px. All existing
geometry/pixel tolerances remain unchanged. The direct mapping stays partial
until overflow alignment is represented compatibly at runtime; never hardcode
left alignment for every width or bake the browser's measured line positions.

Next: investigate an explicit retained text overflow-alignment policy that clamps
negative line origins only in the requested CSS nowrap mode, preserves existing
Rive interpretation, composes with centered shorter lines after br, and updates
on resize. It must be requested and checked in compiler output before qualification.
Original experiment artifacts: nowrap-probe and nowrap-hug. The final direct
mapping artifacts are nowrap-full and nowrap-vector.

## Final direct-mapping validation

- Full native-glyph gate: **471/476**. The five failures are the existing
  A09 em-layout-cascade at 240px and nowrap-center, nowrap-right,
  nowrap-block-center and nowrap-block-right at 240px.
- New nowrap corpus: **26/30** in both native-glyph and vector profiles.
  Geometry passes; the four failures are pixel alignment differences.
- All 438 previously recorded native scene PNGs are byte-identical to
  explicit-br-full. Existing tolerances were not changed.
- All 63 Rust tests, five JavaScript/WASM tests, checked-host requirements,
  two gallery checks, TypeScript, module Clippy and runtime-boundary checks pass.
  Renderer state controls pass separately, **27/27**, because the full gate
  stops at visual failures.
- Visually inspected all 30 new browser/native pairs using the six contact
  sheets nowrap-{240,390,768}-{0,1}.png in nowrap-full. Inspection confirms
  clipped beginnings on native overflowing center/right lines, correct
  alignment when the lines fit, retained explicit breaks, and normal wrapping
  restored in the inheritance control.

Commands: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs`
with NUXIE_HTML_REVIEW_DIR pointing to nowrap-full; the focused vector run uses
NUXIE_NATIVE_GLYPHS=0 and `npm --prefix tools/html-to-riv test -- --grep nowrap`
with its review directory pointing to nowrap-vector. Logs for this run are
/tmp/html-nowrap-gate.log, /tmp/html-nowrap-vector.log, /tmp/html-nowrap-state.log,
/tmp/html-nowrap-clippy.log and /tmp/html-nowrap-boundary.log. Durable galleries
are under output/playwright/html-to-riv/nowrap-full and nowrap-vector.

At this checkpoint A15 remained partial. The four failure fixtures became
acceptance tests for the runtime change below, not expected-pass exceptions.


## Opt-in runtime overflow alignment

Independent Chromium controls without the profile reset reproduce the same
behavior for block and flex text. A block containing an overlong line followed
by br and `short` starts the overlong line at x=8px; the short line starts at
97.765625px for center or 187.53125px for right in a 224px-wide box (Arial 20px).
This confirms the clamp belongs to each line, rather than the entire text box.

The runtime now exposes `Text::set_css_nowrap_alignment(bool)`. Default false
preserves ordinary Rive semantics. When true, native NoWrap line origins are
clamped to zero after normal line construction and alignment. Wrapped text is
unchanged. The instance policy is used by drawing, modifier layout, font-fit
measurement and regular layout measurement, survives resize/reshape, and
marks shape/layout dirty when changed. Fresh imports/instances require host
installation; there is no new serialized Rive property or global switch.

Compiler output requests `text-css-nowrap-alignment-v1` for emitted nowrap text.
The checked probe validates availability before import, then installs the policy
on text occurrences before initial layout. JavaScript declarations expose the
new capability. A public contract test failed on the missing requirement before
implementation. The checked-host regression also verifies rejection before
stream output when the capability is unavailable.

A runtime unit regression checks negative legacy origins, per-line clamping,
short-line alignment, repeated narrow/wide/narrow layout, unaffected wrapped
text, policy disable and a fresh default occurrence. The fixture accepts a caller-supplied font path through NUXIE_TEXT_TEST_FONT,
with the existing runtime upstream-font convention as its fallback. The standard
compiler gate supplies its pinned Inter fixture, so CI needs no local upstream
checkout. An initial cross-package include was rejected by the runtime-boundary
check and removed. Two mixed-break browser fixtures add
six comparisons at 240/390/768px. All original failure fixtures remain unchanged.

### Final policy results

- Full native-glyph visual gate: **481/482**. Only em-layout-cascade at 240px
  (A09) fails; nowrap has no expected-failure exceptions.
- Focused nowrap corpus: **36/36** in native-glyph and vector profiles.
- Comparing nowrap-full with nowrap-policy-full: exactly four changed native
  PNGs, the corrected center/right block/flex cases at 240px. The other **464**
  prior images are byte-identical; six mixed-break comparisons are new.
- All 63 compiler-module Rust tests and the runtime policy regression pass.
  The latter also passes using the exact font-injection command now in run.sh.
  All five JS/WASM, checked-host, TypeScript, two gallery, module Clippy,
  runtime-boundary and 27/27 renderer-state controls pass.
- Visually inspected all 36 browser/native pairs and their difference images.
  Corrected overflow keeps the beginning of long lines visible; short lines
  remain aligned. Differences are glyph rasterization edges, with no new
  geometry or line-placement discrepancy. Existing thresholds remain unchanged.

The initial full script stopped at TypeScript because its consumer test had an
explicit capability union missing the new name. After correcting that union,
TypeScript/gallery and the full visual lane were rerun; earlier compiler/WASM
checks had already passed. The final runtime test's configurable fixture path
and its run.sh invocation were separately validated. No failed check is omitted
from qualification merely because a later command ran.

Commands and logs:

- CARGO_INCREMENTAL=0 NUXIE_HTML_REVIEW_DIR pointing to nowrap-policy-full,
  `bash tools/html-to-riv/validation/run.sh native-glyphs`:
  /tmp/html-nowrap-policy-gate.log (through the intermediate TypeScript failure).
- `npm --prefix tools/html-to-riv run typecheck` and `run test:report`; then
  `NUXIE_NATIVE_GLYPHS=1 npm --prefix tools/html-to-riv test` with the same review
  directory: /tmp/html-nowrap-policy-types.log and /tmp/html-nowrap-policy-visual.log.
- `NUXIE_NATIVE_GLYPHS=0 npm --prefix tools/html-to-riv test -- --grep nowrap
  --output=test-results-nowrap-policy-vector` with review directory
  nowrap-policy-vector: /tmp/html-nowrap-policy-vector.log.
- The exact injected-font `cargo test -p nuxie-runtime --locked --lib
  css_nowrap_alignment_is_opt_in_per_line_and_survives_resizing` command in run.sh:
  /tmp/html-nowrap-policy-runtime-ci.log.
- Module Clippy, boundary and standalone glyph-state-control.mjs:
  /tmp/html-nowrap-policy-clippy.log, /tmp/html-nowrap-policy-boundary-final.log,
  /tmp/html-nowrap-policy-state.log.

Durable artifacts: output/playwright/html-to-riv/nowrap-policy-full/gallery.html,
its baseline-comparison.json, nine browser/native contact sheets, three diff
sheets, and nowrap-policy-vector/gallery.html. A15 is qualified for the documented
normal/nowrap subset. A16–A18 retain preserved whitespace work.
