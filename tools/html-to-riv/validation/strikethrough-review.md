# A21 strikethrough: Chrome reference established

Status: active; compiler support is not yet implemented or qualified. Chrome
153.0.8010.12 is the authoritative browser in these measurements. Firefox is
neither a reference nor a qualification gate.

## Intended semantics

Implement solid `text-decoration-line: line-through`, alone and combined with
`underline`, including the corresponding shorthand. Reuse the accepted
decoration color and thickness grammar. Resolve metrics at the decoration origin
and propagate the decoration through descendants; a child's `none` does not
cancel an ancestor's decoration. Underline offset and skip-ink do not affect
strikethrough. Paint strikethrough after glyphs, preserving underline's existing
before-glyph ordering. Different descendant fonts/sizes, wrapping and multiple
origins still need dedicated controls before these semantics are qualified.

No overline, non-solid style, vertical writing, animation or Grid support is
added by this item. Existing unsupported syntax remains rejected until the
public compiler and versioned runtime contract are implemented and tested.

## Reproducible browser evidence

From `tools/html-to-riv`:

```sh
npm run test:strikethrough-reference
```

`strikethrough-reference.mjs` replays 72 pinned records in
`strikethrough-reference.json`: Inter/OpenSans, sizes 16/20/24/32, thickness
auto/from-font/2px, skip-ink none/auto/all. Each record contains the measured
baseline and red pixel row spans. Transparent glyphs isolate the decoration.
All 72 match the saved Chrome references. All 24 font/size/thickness groups
have identical results across the three skip-ink settings. `--record` is an
explicit reference-writing operation, not part of the validation command.

`strikethrough-paint-reference.mjs` tests 18 font/size/thickness combinations:
Inter/OpenSans, sizes 16/24/32, thickness auto/from-font/2px. Seven captures per
combination produce 126 PNGs. All 18 controls pass these pixel assertions:

- Red strikethrough covers pixels that were opaque black in undecorated glyphs.
- A child with `text-decoration:none` and blue decoration color renders
  identically to the direct red ancestor decoration.
- Skip-ink all and underline offset 20px each leave strikethrough unchanged.
- Combined underline/strikethrough pixels equal the composition of the two
  independent captures.

The paint controls use direct comparisons between Chrome states, not tolerance
changes or recorded native output. They do not check mixed metrics or prove
native equivalence.

Artifacts are under `output/playwright/html-to-riv/strikethrough-reference/`
and `output/playwright/html-to-riv/strikethrough-paint/`. Both directories contain
reports and per-case PNGs. Both metric contact sheets were visually inspected
(24 unique skip-none cases). Both paint contact sheets were visually inspected
(14 representative states at 24px/2px); the other paint captures are checked
programmatically, not claimed as individually inspected. Logs:
`/tmp/html-strikethrough-reference-replay.log` and
`/tmp/html-strikethrough-paint.log`, both terminal exit 0.

## Implementation seam and next gates

The existing runtime draws CSS underline before glyph commands. Strikethrough
requires a separate after-glyph drawing stage; simply emitting another underline
at a negative offset would use the wrong paint order. Native placement must be
checked against the measured Chrome baselines and rows, including thickness
rounding, rather than inferred only from font strikeout metadata.

Remaining: runtime geometry/drawing, a versioned capability and transport,
compiler parsing/origin propagation, public compiler diagnostics, native/WASM
parity, native renderer pixel comparisons, and resizing the same compiled scene
at 240/390/768. Add mixed sizes/fonts, wrapping, multiple origins, translucent
colors and combined-decoration controls. A20's existing residuals remain open;
this reference-only work does not change its qualification.

## Runtime implementation in progress

`ResolvedStrikethrough` accepts a resolved color, positive finite thickness and
finite top-edge offset from each line baseline. The offset is a host-resolved
metric, not yet a claim that compiler font-metric resolution matches Chrome.
It shares full-stripe geometry with underline with skip-ink disabled.
`Text::set_css_strikethroughs` retains this policy through reshaping/resizing,
clears it on empty input and invalidates cached decoration geometry on changes.
Cached decoration paths now carry their paint stage: underline before all glyph
commands, strikethrough after all glyph commands and before restoring the text
clip. Undecorated text retains its existing path.

The compiler transport is still pending. Plan a new explicit strikethrough
capability and requirements version, preserving version 1–3 behavior. Hosts
must validate every Text target before installation and reinstall policies on
fresh imports. This runtime API alone does not enable public CSS syntax.

Runtime import/resize test passed with `native-glyph-controls` enabled:
`cargo test -p nuxie-html-to-riv --features native-glyph-controls --test compiler resolved_underlines_draw_after_import_and_follow_resizing`.
It compiles once and resizes the imported scene to 768/390/240, checks one
strikethrough per resulting line, asserts underline-before/strike-after ordering
for vector paths and native raster glyph images, retains underline's declining
clip fallback checks, and verifies removing both decorations restores the exact
original recording. Log: `/tmp/html-strike-runtime-test.log`, terminal exit 0.
This is renderer-command evidence, not a Chrome/native pixel comparison.

Geometry suite: 7/7 pass with
`NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test css_decoration`
from the repository root. Includes baseline-relative stripe bounds, translated
line bounds and retained width, no skip-ink exclusions, and invalid metric
rejection. The first invocation omitted the required font environment variable
and failed the existing fixture prerequisite (6 passed, 1 prerequisite failure);
the corrected invocation passed all seven. Log:
`/tmp/html-strike-geometry-test.log`, terminal exit 0. No native pixel or WASM
qualification is claimed from these tests.

## Version 4 transport

Added `text-solid-strikethroughs-v1` and version 4 `text_strikethroughs` entries:
unique Text object IDs with ordered `{color, thickness, offset}` lists. Offset
is the top edge relative to each baseline. No skip-ink field is accepted.
Version 4 requires nonempty strikethrough entries and the matching capability;
underline and wrapping policies remain optional and independently validated.
Versions 1–3 reject strikethrough records. Non-finite or out-of-range metrics,
empty lists, duplicate targets and unknown record fields are rejected.
The probe validates all targets before installing any occurrence policies and
installs strikethrough through `Text::set_css_strikethroughs`. JavaScript types
now expose the version 4 branch. CSS emission remains pending.

Validation: four strikethrough contract tests and three existing underline
contract tests pass (`/tmp/html-strike-contract.log`). Full default-feature
module suite: 83 tests pass (`/tmp/html-strike-contract-all.log`); TypeScript
consumer checks pass (`/tmp/html-strike-types.log`). Native-glyph probe build
passes (`/tmp/html-strike-probe-build.log`). All commands exited 0. The previous
feature-enabled runtime import/resize check remains the evidence for native
glyph paint order. New feature native/WASM compiler parity is still pending
because the compiler does not yet emit strikethrough.

`strikethrough-runtime-control.mjs` now supplies explicit version 4 metadata
to one compiled Inter 20px scene and compares resized output at 240/390/768.
It uses the Inter hhea ascent (1984/2048 em) and the candidate top offset
`-ascent/3-thickness/2` for 1/2/3px strokes, with undecorated controls. This
formula is a hypothesis under pixel test, not yet accepted compiler behavior.

First runtime pixel run: 15/24 pass, all 24 geometry comparisons pass. Every
2px strike fails red coverage in both glyph/vector profiles at all three widths;
several also fail shared pixel thresholds. Vector 240px additionally fails
undecorated, 1px and 3px controls. These failures are retained under
`output/playwright/html-to-riv/strikethrough-runtime-controls/`, with report,
stream, manifests and browser/native/difference PNGs. Log:
`/tmp/html-strike-runtime-pixels.log`, terminal exit 1. The glyph 390px sheet
(four cases) was visually inspected: the 2px stroke has visibly different
rasterization; other sheets have not yet been individually inspected. The
placement formula is not qualified. Next investigate baseline/offset and
Chrome stroke snapping with transparent-glyph controls, preserving this run
as the initial reproducer. No thresholds were changed.

## Stripe snapping diagnosis

The runtime control now supports `--focus` (2px, glyph profile, 390px),
`--transparent-glyphs`, `NUXIE_STRIKETHROUGH_TEXT` and fractional wrapper
`NUXIE_STRIKETHROUGH_PADDING`. Diagnostic variants require an explicit
`NUXIE_STRIKETHROUGH_RUNTIME_DIR` to preserve the initial reproducer.
`--round-offset` is a diagnostic hypothesis only, not production behavior.
Reports retain per-row red energy and interior RGB samples.

The focused original and transparent variants both fail with 1090 Chrome
versus 1635 native red pixels. Reducing the content to one transparent M retains
the failure: 36 versus 54 red pixels. Chrome paints solid rows 15 and 16;
native paints rows 14–16, with green values approximately 138, 0 and 117
inside the red stripe. This isolates decoration rasterization from glyph ink.

Ranked hypotheses tested: painter Y rounding; mismatched baseline metrics;
different edge coverage rules. Chromium's `DecorationLinePainter` calls
`SnapYAxis` for non-SVG solid rectangles: `floor(y + 0.5)` and thickness
`max(floor(thickness), 1)`. Source snapshots, URLs and SHA-256 hashes are stored
in `strikethrough-single/source-receipt.json` beside the reproduction. These
are current source snapshots; Chrome 153 measurements remain authoritative.
Source: https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/core/paint/decoration_line_painter.cc

Rounding the resolved offset fixes the single M at integral placement, but
fractional wrapper padding disproves that as a complete fix. At 0.25px, Chrome
retains solid rows 15–16 while native covers rows 15–17 (interior green values
64/0/191). At 0.75px Chrome uses solid rows 16–17 while native covers 15–17.
Both native/browser layout Y positions match the specified padding. Therefore
rounding only the compiler metric is insufficient; the runtime needs the
correct paint-coordinate snapping stage. The exact relationship between
paint-origin rounding, line metrics and host transforms remains to be resolved.

The aggregate thresholds missed the fractional one-glyph difference. A new
transparent-control assertion compares the stripe interior column (x=5) with
the existing 6-channel interior limit; it fails the 0.25px variant. No threshold
was widened. The full-scene test remains unchanged for opaque text.

Artifacts under `output/playwright/html-to-riv/`:
`strikethrough-focus`, `strikethrough-transparent`, `strikethrough-single`,
`strikethrough-rounded`, `strikethrough-rounded-quarter-wrapper`,
`strikethrough-rounded-threequarter`, and `strikethrough-rounded-quarter-strict`.
Single and quarter-strict contact sheets were visually inspected. The initial
`strikethrough-rounded-quarter` attempt used body padding, which did not exercise
the intended compiler container and failed geometry; it was replaced with an
explicit div wrapper and is not used to infer snapping behavior.

Reproduce the strict fractional failure from repository root:

```sh
NUXIE_STRIKETHROUGH_RUNTIME_DIR=$PWD/output/playwright/html-to-riv/strikethrough-rounded-quarter-strict NUXIE_STRIKETHROUGH_TEXT=M NUXIE_STRIKETHROUGH_PADDING=0.25 node tools/html-to-riv/validation/strikethrough-runtime-control.mjs --focus --transparent-glyphs --round-offset
```

Last run exited 1 with `stripe interior column differs by more than 6`; log
`/tmp/html-strike-rounded-quarter-strict.log`. Production runtime/contract code
was not changed by these experiments. CSS emission remains pending.

## Line-origin and ascent reference (384 placements)

Added `strikethrough-phase-reference.mjs` with a pinned JSON replay, also run
by `npm run test:strikethrough-reference`. Inter/OpenSans × 20/24px font sizes
× 30/30.5px line heights × eight padding phases (0 through 0.875px) × 1/2/3px
strokes × two lines yields 384 measured stripe placements in 192 screenshots.
The test records baseline markers and stripe-interior row colors; the second
line exercises fractional line-height accumulation.

The measured target is reproduced in all 384 cases by:

```text
round(lineTop) + round(baseline - lineTop - round(fontAscent)/3 - thickness/2)
round(x) = floor(x + 0.5)
```

Rounding the global stripe top fails 116 placements. Rounding baseline and
offset separately fails 64. Rounding line top then stripe, but retaining raw
font ascent, also fails 64 (OpenSans 20px, odd stroke thicknesses). The final
candidate rounds font ascent before computing the offset and fails zero.
These are Chrome-only predictions; no runtime implementation is implied.

Chromium source confirms separate stages:
- `TextFragmentPainter::ComputePhysicalBoxGeometry` shifts the text paint box
  by `round(line_top) - line_top`.
- `FontMetrics::AscentDescentWithHacks` normally rounds the ascent, with special
  cases including tiny fonts and platform-specific adjustments.
- `DecorationLinePainter::SnapYAxis` rounds the stripe Y and floors thickness.

Sources: https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/core/paint/text_fragment_painter.cc
and https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/platform/fonts/font_metrics.cc
Current source snapshots and SHA-256 receipts are stored with artifacts; they
are mechanism evidence, while Chrome 153.0.8010.12 pixels remain authoritative.

The full replay exits 0 (`/tmp/html-strike-phase-replay.log`). Artifacts:
`output/playwright/html-to-riv/strikethrough-phase-wrapped/`. Its eight-case
contact sheet (16 stripes, 20px/30.5px line-height, 2px thickness, both fonts at
four phases) was visually inspected. Other placements are programmatically
checked. The earlier one-line 192-case diagnostic remains in
`strikethrough-phase/`; the canonical reference now covers two lines.

Next runtime change must retain the layout line top separately from the
baseline and apply snapping at paint time. The public transport's offset should
use the resolved ascent; do not bake viewport-specific coordinates into it.
Native glyph/vector pixels, host transforms/DPR, auto/from-font thickness and
compiler CSS emission/parity remain unqualified.

## Runtime snapping implemented

Strikethrough retains unsnapped geometry, original thickness and the CSS line
origin until painting. `paint_bounds` rounds line origin and stripe position
separately and floors the original thickness. Cached paths are rebuilt when
paint-space placement changes; ordinary underlines retain their existing path.
Snapping occurs before the host renderer's transform. Internal text fit/scaling,
affine host transforms and DPR still require qualification.

Rive's native `GlyphLine.top` is not the CSS line-box top. Using it fixed the
0.25px case but left the 0.75px case one row high. The version 4 record now
requires `line_baseline`, the resolved baseline distance from the CSS line-box
top. `ResolvedStrikethrough::solid` takes this fourth metric, and reconstructs
the responsive line origin as `OrderedLine.y() - line_baseline`. This is a font
and line-height metric, not a browser-measured viewport coordinate. Host
validation rejects non-finite/out-of-range values and missing fields. Types,
probe installation and transport tests were updated. Earlier experimental
version 4 artifacts lack this newly required field and represent the old
prototype contract; versions 1–3 retain their existing interpretation.

The two-line Inter 24px test exposed float cancellation: deriving thickness
by subtracting rounded rectangle edges turned 2px into slightly less than 2px,
then flooring reduced the stripe to 1px. Retaining the original metric fixes it.
The actual failing native case and a numeric regression are retained; do not
reintroduce thickness derivation from rectangle bounds.

Validation on the final runtime implementation:
- 98 module tests pass with `native-glyph-controls` enabled, including renderer
  paint order/removal/resize, versioned contracts and three snapping tests.
  The snapping test checks all 384 pinned Chrome placements.
  `/tmp/html-strike-snapping-tests.log`, exit 0.
- Seven runtime decoration tests pass with the pinned Inter fixture supplied:
  `/tmp/html-strike-snapping-geometry.log`, exit 0.
- TypeScript checks pass: `/tmp/html-strike-snapping-types.log`, exit 0.
- 32/32 native glyph phase controls pass and all eight four-case contact sheets
  were visually inspected: both fonts, sizes 20/24, line heights 40/40.5, padding
  phases 0/.25/.5/.75, two transparent M lines, 2px stroke. Includes the strict
  stripe-interior check. `strikethrough-runtime-phases-40/` artifacts and
  `/tmp/html-strike-phases-40.log`, exit 0.
- The original short-line-height matrix is retained in
  `strikethrough-runtime-phases/`: 16/32 passed before the thickness fix; eight
  Inter 24px cases exposed the fixed cancellation bug, and eight OpenSans 24px
  cases were rejected by the compiler's existing minimum line-height rule.
  The Inter 24px two-line reproducer subsequently passes in
  `strikethrough-thickness-fixed/`. Short line heights remain an independent
  compiler limitation, not newly accepted by this change.

Reproduce the phase matrix from repository root:

```sh
NUXIE_STRIKETHROUGH_MATRIX_DIR=$PWD/output/playwright/html-to-riv/strikethrough-runtime-phases-40 NUXIE_STRIKETHROUGH_HEIGHTS=40,40.5 python3 tools/html-to-riv/validation/strikethrough-runtime-phases.py
node tools/html-to-riv/validation/strikethrough-phase-gallery.mjs output/playwright/html-to-riv/strikethrough-runtime-phases-40
```

The gallery crops and magnifies each image's stripe area for inspection; full
original screenshots remain alongside it. No rendering tolerances changed.
CSS parsing/emission and native/WASM feature parity remain pending, along with
broader font/size/thickness, propagation, composition and host-state coverage.

Final original resize replay: 20/24 pass (glyph 12/12, vector 8/12), all
geometry checks pass. The only remaining failures are mean-channel error in
vector 240px for baseline and all three stroke thicknesses. All 24 native PNGs
are byte-identical to the intermediate line-box run (`pixel-identity.json`),
so the independent thickness fix does not alter these 20px cases. The glyph
390px contact sheet was inspected; complete original-matrix sheet inspection
remains pending. Final artifacts: `strikethrough-snapping-final/`; log
`/tmp/html-strike-snapping-final.log`, terminal exit 1 for the four retained
vector failures. Next connect CSS emission to the resolved line-baseline and
ascent contract, then validate public compiler/native-WASM parity and native
pixels across the supported syntax.

## Public CSS emission and native/WASM parity

CSS longhand and shorthand now emit strikethrough alone or combined with
underline, with independent runtime layers and capabilities. Propagated origins
retain both flags. Explicit inheritance copies both; initial resets local flags
without canceling ancestor decoration. Later underline emission takes max(version,
3), so it cannot downgrade a previously emitted version 4 scene. Ascent and
line-baseline resolution use the embedded font and existing compiler line-height
policy; no browser layout data is used by the compiler.

Five new public compiler tests cover shorthand equivalence, combined layers,
origin propagation, inheritance/reset, skip-ink/offset independence, mixed-version
ordering and invalid combinations. An old negative test for combined underline
and line-through initially failed because that syntax is now implemented; it
was replaced by underline plus overline. Invalid-value tests use empty elements
to avoid accidentally passing due to a missing-font diagnostic.

Results:
- Full feature-enabled Rust suite: 103 tests pass, log
  `/tmp/html-strike-css-tests-final.log`. After strengthening the empty-element
  rejection control, all five targeted compiler tests pass again in
  `/tmp/html-strike-css-targeted.log`.
- Native CLI/probe and WASM builds pass in `/tmp/html-strike-css-build.log` and
  `/tmp/html-strike-css-wasm.log`.
- All seven JavaScript tests pass, including corpus parity and six new cases
  (Inter/OpenSans × explicit 2px, combined from-font, percentage thickness)
  with descendant none and an additional child origin. Rive bytes, source maps
  and complete runtime requirements match exactly. Log:
  `/tmp/html-strike-css-js.log`.
- `strikethrough-runtime-control.mjs --compiler-css` now compiles each control's
  CSS once, then resizes that same scene at 240/390/768. It supplies no hand-built
  decoration metadata. Result 20/24, glyph 12/12 and vector 8/12, all geometry
  checks pass. Every native PNG is byte-identical to the prior runtime-controlled
  run. Artifacts: `strikethrough-compiler-css/`, including per-control source,
  compiled scene, metadata, stream, bounds, screenshots and pixel-identity receipt.
  Log `/tmp/html-strike-compiler-css.log`, exit 1 for the same four vector 240px
  mean-channel failures including the undecorated baseline.

Glyph sheets at 240 and 768 were visually inspected. The 390px images are
byte-identical to the previously inspected sheet, covering all twelve glyph
cases. Full vector sheet inspection, additional CSS composition/propagation
pixels, broader font/thickness/host profiles and full corpus rendering remain
pending. This implements CSS emission but does not complete A21 qualification.

## Composition and propagation corpus

Added nine decorated fixtures to `validation/cases.json`: combined underline
and strike, propagated auto/from-font/percentage/computed-em thickness through
mixed font sizes, multiple colored origins, OpenSans from-font, fractional
placement with explicit breaks, and a responsive annual-price card. Added a
tenth undecorated OpenSans control to isolate its glyph-rendering residual.
The fixture named `strikethrough-origin-1em` uses **0.1em**, as specified in its
CSS. These fixtures participate in normal corpus import and WASM parity checks.

The initial run passed 23/27. Three failures were invalid fixture setup: explicit
br requires display:block in the current profile. Correcting the fixture to
declare that mode produced 26/27; the rejected initial inputs and logs remain
in `test-results-strikethrough-compositions`. No compiler acceptance rule was
changed to accommodate the fixture.

Final decorated native-glyph run: 26/27, all geometry checks pass. All nine
three-width browser/native/difference sheets were visually inspected. The sole
failure is OpenSans from-font at 240px, mean-channel error 1.101943359375 versus
limit 1. Its red decoration comparison passes (575 browser versus 574 native
pixels, no missing/extra support). Removing decoration still fails at 240px
with mean-channel error 1.1818359375; 390 and 768 pass. The undecorated control
sheet was also inspected. This evidences an independent glyph-rendering
residual; it is retained and not waived. Combined result: 28/30, not full
qualification.

Commands were the native Playwright project with `NUXIE_NATIVE_GLYPHS=1`,
`--grep strikethrough` (nine decorated fixtures), then
`--grep strikethrough-opensans-baseline` for the added control. Each scene is
compiled once by the established harness and resized at 240/390/768.
Artifacts: `output/playwright/html-to-riv/strikethrough-compositions-valid/`,
`strikethrough-opensans-baseline/` and their matching module `test-results-*`
directories. Logs `/tmp/html-strike-compositions-valid.log` and
`/tmp/html-strike-opensans-baseline.log` exit 1 for retained pixel failures.
`composition-review.mjs` generates inspection sheets from the native harness
attachments; original screenshots are retained.

All seven JS tests pass against the expanded corpus, including byte-for-byte
native/WASM scenes, source maps and runtime requirements:
`/tmp/html-strike-composition-parity.log`, exit 0. The public corpus-import test
also passes: `/tmp/html-strike-composition-import.log`, exit 0. No tolerances or
production source changed during this increment. Vector profile coverage for
these new fixtures, host/DPR, additional metric edge cases and the evidenced
OpenSans glyph discrepancy remain outstanding.


### Vector compositions and host device scale

Chrome 153.0.8010.12 remains the reference; Firefox is not a gate or target.
The vector composition run passes 18/30 comparisons. All ten three-width
contact sheets were visually inspected. Failures remain at 240px for origin
auto/from-font/10percent, and all three widths for OpenSans from-font, the
price card, and the undecorated OpenSans baseline. All red decoration checks
pass. Preserve these failures rather than qualifying the vector profile.
Artifacts: `output/playwright/html-to-riv/strikethrough-compositions-vector/`
and `tools/html-to-riv/test-results-strikethrough-compositions-vector/`.
Log: `/tmp/html-strike-compositions-vector.log` (exit 1).

`node tools/html-to-riv/validation/strikethrough-dpr-control.mjs` compares
three thickness modes (auto/from-font/2px), three runtime widths (240/390/768),
and host device scales 1/2/3 using the native-glyph renderer. Each source is
compiled once, then its same scene is resized. Native/WASM bytes, source maps
and requirements are asserted equal. Layout stays in CSS pixels; actual image
dimensions must equal viewport dimensions times device scale. The fixture
uses the embedded Japanese font, CJK and Latin, and fractional top padding.

Opaque text passes 21/27. All geometry and red decoration checks pass. The six
failures are mean-channel error at DPR 3, widths 240 and 390, for all three
thickness modes. With `--transparent-glyphs`, all 27 comparisons pass. These
controls support stripe correctness in this matrix, but do not waive visible
text failures or qualify other fonts and transforms. DPR 3 contact sheets
for all thickness modes were visually inspected in both runs (nine cases
each); DPR 1/2 sheets have not yet been visually inspected.

Artifacts are `output/playwright/html-to-riv/strikethrough-dpr/` and
`strikethrough-dpr-transparent/`, each with source, compiled scenes, manifests,
streams, browser/native/diff PNGs, contact sheets and `results.json`.
Logs `/tmp/html-strike-dpr.log` (exit 1) and
`/tmp/html-strike-dpr-transparent.log` (exit 0). Override
`NUXIE_HTML_REVIEW_DIR` for each experiment. No visual tolerances changed.
The optional `--without-decoration` control removes decorations from the
same source to investigate glyph residuals independently.


The undecorated DPR control also passes 21/27, failing mean-channel error at
DPR 3 and widths 240/390 in every repeated thickness group. Geometry passes.
The three thickness groups intentionally repeat the same undecorated source;
this represents nine unique viewport/scale combinations, seven passing.
The auto/DPR3 contact sheet (all three widths) was visually inspected. Glyph
edge/position differences are visible without any decoration. Together with
the transparent controls, this demonstrates an independent text-rendering
residual in this fixture; the decorated failures remain failing gates.

Initial baseline run incorrectly applied the nonempty-red-stripe gate to an
undecorated fixture. Its artifacts remain at `strikethrough-dpr-baseline/`.
The corrected control asserts zero red pixels in both images instead, while
retaining all ordinary pixel and geometry thresholds. Rerun artifacts:
`output/playwright/html-to-riv/strikethrough-dpr-baseline-valid/`;
log `/tmp/html-strike-dpr-baseline-valid.log` (exit 1 for six pixel failures).
All 27 native images are byte-identical to the original baseline run, so its
visual inspection remains applicable. Native/WASM parity assertions pass.


### Small text and transport bounds

Added public compiler coverage for font sizes 0.1/1/8/128/800000px crossed
with auto/from-font/0px/1000000px thickness (20 combinations), using explicit
1000000px line height. Each output passes host requirement validation and a
JSON round trip. All six strikethrough Rust tests pass:
`cargo test -p nuxie-html-to-riv --features native-glyph-controls --test strikethrough`
(`/tmp/html-strike-metric-extremes.log`, exit 0). Matching 20 native/WASM
byte/source-map/requirement comparisons pass in the new JS test:
`node --test --test-name-pattern='strikethrough metric extremes' tools/html-to-riv/tests/javascript.test.mjs`
(`/tmp/html-strike-metric-parity.log`, exit 0). These are transport checks,
not giant-font rendering qualification; no invalid manifest was reproduced.

The DPR script now accepts finite positive `NUXIE_STRIKETHROUGH_SIZE`.
At 8px and 1px, each 27-case matrix passes geometry, ordinary pixels and red
support, including exact native/WASM artifacts for all three thickness modes.
Commands use that environment variable with values 8 and 1 respectively,
plus separate `NUXIE_HTML_REVIEW_DIR` paths, running
`node tools/html-to-riv/validation/strikethrough-dpr-control.mjs`.
Outputs: `output/playwright/html-to-riv/strikethrough-dpr-small/` and
`strikethrough-dpr-tiny/`; logs `/tmp/html-strike-dpr-small.log` and
`/tmp/html-strike-dpr-tiny.log`, both exit 0. Chrome 153.0.8010.12.
The 8px DPR3 sheets for all thicknesses and widths were visually inspected
(nine comparisons). For 1px, browser/native DPR3 240px images for all three
thicknesses were inspected: the stripe dominates the tiny glyph ink, as in
Chrome. Remaining small/tiny widths/scales are automated evidence only.
These cases use one embedded font and a 40px line box; they do not qualify
all tiny-font metrics, tight line heights, vector output or host transforms.
No production behavior or tolerance changed.


### Host transforms and opacity: visual review strengthened the gate

`validation/glyph-state-control.mjs strikethrough` now compiles a Japanese/Latin
2px red strikethrough source once and resizes the same scene to 240/390/768.
Native/WASM scene bytes, source maps and requirements are asserted identical.
Nine host states cover identity, fractional translation, clipping, opacity,
uniform/nonuniform scale, rotation, shear, and combined rotation/clip/opacity.
These are host renderer operations, not additional accepted CSS syntax.
DPR is 1. The default draws a second copy after save/restore; `--no-restore`
isolates the transformed copy. The ordinary glyph profile still passes 27/27
(`/tmp/html-glyph-state-strike-regression.log`, exit 0).

Initial broad gates passed 27/27 both with and without the restored copy:
`strikethrough-state-controls/` and `strikethrough-state-isolated/` under
`output/playwright/html-to-riv/`. Logs `/tmp/html-strike-state.log` and
`/tmp/html-strike-state-isolated.log` exited 0. All nine isolated 240px states
were visually inspected; restored-copy rotation/shear/combined at 240px were
also inspected. Other widths have automated evidence only so far.

Visual review found dark red overlaps in the opacity case despite those
aggregate passes. At (18,28), Chrome is RGB(255,126,126), native (191,63,63).
There are 202/199/199 strongly darkened pink reference pixels at the three
widths. This is consistent with the existing P05 per-draw versus group-alpha
limitation; these initial passes must not be taken as opacity qualification.

A new opacity stripe-interior check compares solid pink Chrome pixels using
mean RGB error 6 (the existing interior threshold). The isolated rerun now
passes 24/27, with opacity failing at every width. Geometry, ordinary pixel,
red support and ink coverage still pass. Artifacts:
`output/playwright/html-to-riv/strikethrough-state-opacity-gate/`;
log `/tmp/html-strike-state-opacity-gate.log`, exit 1. All 27 native PNGs are
byte-identical to the first isolated run, preserving the visual-review evidence.
Reproduce with `NUXIE_HTML_REVIEW_DIR` set to a separate output directory and
`node tools/html-to-riv/validation/glyph-state-control.mjs strikethrough --no-restore`.
The new check is specific to identity half-opacity; transformed alpha cases
remain broader qualification work. No production change or relaxed tolerance.
