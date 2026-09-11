# A23 single-line ellipsis — runtime investigation

Current result: CSS-policy probe **24/24 native-glyph comparisons pass**, all
visually inspected. Earlier 18/24 and 6/12 runs used vector fallback despite
requesting native glyphs; see the adapter correction below. CSS acceptance
and full feature qualification remain pending.

Compiler CSS `text-overflow` remains unsupported. This increment investigates
Rive's existing overflow mode before deciding the required CSS runtime policy.
No source text is truncated by the compiler.

`node tools/html-to-riv/validation/ellipsis-runtime-control.mjs` compiles four
baseline sources without text-overflow: long title, fitting title, 8px-wide
box, and fractional width/padding. The same scenes resize to 240/390/768.
Native/WASM baseline scene bytes/maps/requirements must match. The probe-only
environment flag `NUXIE_EXPERIMENTAL_TEXT_ELLIPSIS=1` selects Rive Text overflow
mode 3 after import and before layout; the CSS reference uses text-overflow:
ellipsis. This does not establish native/WASM transport of CSS ellipsis.
Requests use display:block, nowrap, overflow:hidden, 24px Inter and 40px lines.
DPR1, native glyph adapter, actual Metal replay. All 12 comparisons were visually
inspected across four contact sheets.

6/12 pass: long and fitting titles at all widths. Too-narrow and fractional
cases fail local RGB at all widths. The 8px box exposes a semantic mismatch:
Chrome clips the initial A, while Rive draws the first part of its ellipsis.
This requires a CSS-specific runtime behavior before acceptance. Fractional
cases retain text pixel differences; an exact cause is not yet isolated.
Geometry passes throughout. No tolerance changed.

Artifacts: `output/playwright/html-to-riv/ellipsis-runtime/`, including baseline
sources/scenes/manifests, draw streams, bounds, browser/native/diff PNGs and
results.json. `thickness` in results stores fixture names, inherited from the
control runner. `/tmp/html-ellipsis-runtime.log` exits 1 for six retained
failures. Probe build `/tmp/html-ellipsis-probe-build.log` passes.

Inspection points: Text::compute_bounds_info chooses an ellipsis line using
available height; build_render_styles constructs OrderedLine with that decision.
The regular ordered-line rebuild does not request ellipsis. Existing mode 3
therefore needs examination for short heights, alignment, glyph fallback,
clusters and decoration interactions as well as the too-narrow reproducer.
No editor, script, interaction or animation scope was added.

Probe default behavior regression: existing glyph host-state controls pass
27/27 with the experimental flag absent (`/tmp/html-ellipsis-probe-regression.log`,
exit 0). Artifacts: `glyph-state-ellipsis-probe-regression/` in the shared review
root. No runtime production behavior was changed by this diagnostic flag.

## Chrome narrow-box boundary control

`node tools/html-to-riv/validation/ellipsis-narrow-reference.mjs` completes
24 Chrome-only cases: Latin A, ffi ligature, and A + combining acute; widths
0/8/16/20/24/30/40/48px at DPR1, Inter 24px/40px. Each compares actual
text-overflow:ellipsis with plain clipping, isolated first grapheme, first
grapheme + U+2026, isolated U+2026, and three periods. Source HTML, all 144
PNGs, exact decoded-pixel comparisons and browser version are retained in
`output/playwright/html-to-riv/ellipsis-narrow-reference/`. All 24 gallery rows
were visually inspected. `/tmp/html-ellipsis-narrow-reference.log` exits 0.

At 8 and 16px Latin/combining results equal the clipped first grapheme. At
20/24/30/40px they exactly equal first grapheme + U+2026, clipped to the box.
The ffi case at 8px equals an isolated f, but does NOT equal clipping the
original ffi shaping. The initial broad clip-equality assertion failed here;
the final control explicitly asserts this distinction. Thus simply retaining
the original first shaped glyph is insufficient. At wider ffi boundaries,
first-grapheme-plus-marker is visually similar but not pixel-identical; marker
placement/shaping still needs isolation. Width zero comparisons are blank
and carry no semantic evidence.

The [CSS Overflow rule](https://www.w3.org/TR/css-overflow-3/#ellipsis) treats
characters as grapheme clusters, preserves the first one, and specifies
U+2026 with a permitted three-period fallback when unavailable. Chrome is
the authoritative reference for this project. These controls do not prove
compiler support, native rendering, resizing or WASM transport. No runtime
policy changed and CSS text-overflow remains rejected. Next: isolate Chrome's
ligature boundary/marker placement, then implement a CSS-only runtime policy
without changing legacy Rive ellipsis behavior.

## Ligature marker placement isolated

The expanded narrow-reference runner adds independently shaped prefixes and
markers, and a prefix box whose width is measured from a DOM Range over the
original text. This browser-only diagnostic is never a compiler input or a
layout implementation. Final run: Chrome 153.0.8010.12, 24 cases, exit 0,
`/tmp/html-ellipsis-range-isolation.log`; artifacts in
`output/playwright/html-to-riv/ellipsis-range-isolation/`. All 24 cases were
visually inspected in the three per-fixture sheets. Earlier separate-marker
controls remain in `ellipsis-marker-isolation/` (not separately visually
reviewed; superseded by the expanded run).

For ffi at 16/20/24/30px, isolated f plus an independently shaped marker still
fails exact pixel equality. Giving the f its original Range width, 7.65625px,
makes every one of these comparisons exactly equal. At 40px, independently
shaped ff + marker matches; at 48px, ffi + marker matches. The runner asserts
these positive and negative controls. This isolates the earlier wider-ffi
residual: retained layout advance and the painted prefix must be handled
separately. Simply reshaping a prefix and using its new width is insufficient.
Latin/combining first-range widths are 16.359375px and retain their earlier
matching behavior. 48px Latin/combining renders an additional space, so the
first-only candidate is intentionally not expected to match there.

The exact browser version's [LineTruncator source](https://chromium.googlesource.com/chromium/src/+/refs/tags/153.0.8010.12/third_party/blink/renderer/core/layout/inline/line_truncator.cc)
was fetched and retained as `ellipsis-marker-isolation/chrome-line-truncator.cc`.
It shapes the marker independently, chooses a truncation offset from the
existing shape, creates a subrange view and places the marker after the
retained item's width. The pixel control supports separating layout width
from prefix painting; the exact paint implementation has not been traced.

Next runtime work: introduce an opt-in CSS ellipsis policy with retained
source/grapheme boundaries and independent marker shaping, preserving the
original retained advance even when partial clusters require different
painting. Preserve legacy Rive behavior. Public CSS acceptance and runtime
transport remain pending; this increment adds no production feature claim.

## Runtime planning kernel

Added `crates/nuxie-runtime/src/mechanical_port/source/text/css_ellipsis.rs`,
an experimental planner that receives original visual-LTR grapheme-end
advances and independently measured marker width. It returns either unchanged
text or a retained source boundary and original marker position. It keeps the
first grapheme when the prefix/marker cannot fit, leaves fitting text untouched,
handles decreasing advances, and rejects nonfinite/malformed measurements.
No glyph slicing or replacement width is used by the planner.

`cargo test -p nuxie-runtime --lib css_ellipsis` passes 5/5 (59 unrelated tests
filtered), `/tmp/html-css-ellipsis-plan.log`, exit 0. Tests cover narrow-box
preservation, resize recomputation, supplied multi-codepoint boundaries,
decreasing advances, and invalid inputs. Only the first-f 7.65625px value in
the planner test is directly measured from the Chrome control; the other
planner measurements are synthetic unit-test inputs, not additional browser
qualification. This build reports existing repository warnings.

The planner is not yet connected to Text rendering or compiler requirements.
Next work must derive real grapheme boundaries and retained widths from the
original shape, paint truncated clusters appropriately, style/shape the marker
independently, and run native pixel controls before CSS acceptance. The
planner currently describes visual LTR text sequences only; bidi integration
remains outstanding. No end-to-end or WASM parity claim is made by these unit
tests. Existing Rive rendering behavior is unchanged.

## Extracting boundaries from actual shaping

`css_ellipsis::boundaries_for_run` now extracts scalar-indexed grapheme ends
and original advances from a complete LTR GlyphRun. It groups glyphs sharing
a cluster and distributes ligature advance among graphemes. It keeps combining
sequences, ZWJ emoji and regional-indicator pairs intact. Invalid cluster arrays,
nonfinite advances, invalid scalars and RTL runs are rejected. Mixed-run and bidi
assembly are not implemented. Unicode segmentation uses pinned
`unicode-segmentation = 1.13.3` ([library documentation](https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/)).

Tests: 7 synthetic tests pass in `/tmp/html-css-ellipsis-boundaries.log`.
With `NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf`,
`cargo test -p nuxie-runtime --lib css_ellipsis -- --include-ignored` passes
8/8, exit 0 (`/tmp/html-css-ellipsis-boundaries-real.log`). The normally ignored
fixture test uses actual HbFont CSS-precision shaping of ffi, A, and A + acute
followed by a long title; first advances match the previously measured Chrome
Range values within 1/64px DOM quantization. This is a metric check, not a new
pixel tolerance or native rendering pass.

WASM compatibility: `RUSTC=$(rustup which rustc) cargo check -p nuxie-runtime
--locked --target wasm32-unknown-unknown --no-default-features` passes, exit 0,
`/tmp/html-css-ellipsis-boundaries-wasm.log`. This checks compilation only, not
public compiler transport/parity. Repository warnings remain. The planner and
boundary extractor are still not installed in Text rendering. Marker shaping,
partial-cluster painting, mixed runs, renderer validation and CSS acceptance
remain outstanding. No production support or new pixel qualification claimed.

## Probe rendering integration — first native CSS-policy run

`css_ellipsis::prepare` independently shapes the retained prefix and marker
(U+2026 when present, three periods otherwise). It preserves the original
retained advance by adjusting the last prefix advance before sequential
painting. `OrderedLine::from_css_runs` and the opt-in Text diagnostic setter
connect this to the existing draw pipeline. No source text is mutated.
The probe enables this only when both ellipsis experiment flags are present.
It asserts one unmodified run/line and fails on unsupported shaping; mixed
runs, bidi, modifiers and production requirement transport remain pending.
Marker styling currently comes from the one source run, not a separately
transported block style. The helpers/flag are experimental, not a supported
compiler/runtime capability.

Build: `/tmp/html-css-ellipsis-render-build.log`, exit 0. Run:
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-css-runtime node tools/html-to-riv/validation/ellipsis-runtime-control.mjs`.
The runner sets the legacy overflow experiment flag itself. Eight baseline
scenes compile once at 390px and resize to 240/390/768; DPR1, native glyphs,
Metal replay. Baseline native/WASM bytes/maps/requirements match. Input CSS
still omits text-overflow, so this does not prove CSS feature transport.

18/24 comparisons pass; all geometry passes. All 24 were visually inspected
in eight contact sheets. Long/fitting titles, 8px A, 8px combining A+acute,
and ffi at 24/40px pass at all three sizes. Compared with the legacy mode's
original four cases, the 8px A now passes (9/12 instead of 6/12). The new 8px
ffi case and existing fractional-position case fail local RGB at all widths.
They show the expected prefix/marker content, but raster differences remain
unisolated. No tolerance was widened. Failures and all streams/source/scenes/
PNGs are retained under `ellipsis-css-runtime/`; log
`/tmp/html-css-ellipsis-render.log` exits 1 for the six failures.

Flag-off glyph host-state regression: 27/27, exit 0,
`/tmp/html-css-ellipsis-glyph-regression.log`, artifacts
`glyph-state-css-ellipsis-regression/`. This regression checks existing
default glyph behavior; those images were not separately visually reviewed
in this increment. Public CSS text-overflow remains rejected.

Preparation regression tests: the real Inter fixture test now also checks
original advances remain unchanged, the prepared prefix sum retains the
marker position, the marker is a single glyph, and fitting text needs no
replacement. All 8 focused tests pass including the fixture test via
`NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf cargo test -p nuxie-runtime --lib css_ellipsis -- --include-ignored`;
`/tmp/html-css-ellipsis-prepared-tests.log`, exit 0.

## Native glyph adapter correction

A plain-f control (`--plain-f-control`, separate review directory required)
disables both ellipsis flags and compiles only f in an 8px clipped box. It
passes 3/3 with all three visually reviewed. Chrome PNGs exactly equal the
previous ffi-ellipsis reference at each width; native PNGs differ. Identity
hashes: `ellipsis-plain-f-control/identity.json`. Glyph export shows the same
f glyph, position and world matrix. Crucially, the previous ellipsis run has
zero glyph-cache entries, while plain f populates the cache.

Diagnosis: `Text::draw_solid_glyph_style` allowed only visible/clipped overflow,
so ellipsis always fell back to vector paths. Earlier receipts correctly
described native Metal rendering but incorrectly implied use of the native
glyph adapter. The 18/24 CSS-policy and original 6/12 legacy experiments
remain valid vector-rendering evidence; they are not glyph-adapter evidence.

The adapter now accepts ellipsis only when the prepared experimental CSS
policy is enabled. Legacy Rive behavior is unchanged. The control runner
asserts a populated glyph cache for the requested CSS/native profile and
records cache statistics in results.json. This guard would reject the old
run, rather than silently classifying its vector pixels as glyph pixels.

Build `/tmp/html-css-ellipsis-adapter-build.log` passes. Re-run with
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-css-glyph node tools/html-to-riv/validation/ellipsis-runtime-control.mjs`
passes **24/24**, exit 0 (`/tmp/html-css-ellipsis-adapter.log`). All 24
comparisons were visually inspected across eight sheets; geometry and
baseline native/WASM parity pass. Same eight compiled scenes resize to
240/390/768, DPR1. Narrow ffi and fractional-position residuals disappear
in the verified glyph profile. Prior vector reproducers remain retained.
No threshold changed.

Flag-off glyph host regression remains 27/27, exit 0,
`/tmp/html-css-ellipsis-adapter-regression.log`, artifacts
`glyph-state-ellipsis-adapter-regression/` (not separately visually reviewed).
Plain control artifacts: `ellipsis-plain-f-control/`, log
`/tmp/html-css-ellipsis-plain-f.log` exit 0. The flag still supports only one
unmodified LTR run/line; block marker styling, mixed runs, bidi, decorations,
DPR/host transformations and public compiler transport need further work.

## Alignment and short-height controls

The runtime runner now accepts `--alignment-control` and `--vertical-control`,
with the CSS experiment flag and a separate review directory required.
Alignment uses left/center/right with long, fitting and 24px ffi text. It
passes 27/27, all visually inspected, at 240/390/768 with the same baseline
scenes. Artifacts `ellipsis-alignment-initial/`;
`/tmp/html-css-ellipsis-alignment.log` exits 0. Fitting text preserves its
alignment; these overflowing nowrap cases align at the start like Chrome.

Height 8/20/39/80px with a 40px line passes 9/12. All 12 were visually
inspected; pixels pass throughout. The three 8px cases fail geometry: native
clip.height is 8.7758255 rather than 8, and root.height is 32.7758255 rather
than 32. Both renderings are blank there, which is why pixels alone miss it.
Artifacts `ellipsis-vertical-initial/`; `/tmp/html-css-ellipsis-vertical.log`
exits 1. All runs verify native glyph cache usage and baseline scene parity.

`--vertical-control --clip-only` disables both ellipsis flags and removes
Chrome text-overflow. It repeats the exact geometry failure (9/12), with all
pixels passing and all 12 visually inspected. Thus the short-height issue
is independent of ellipsis. Artifacts `ellipsis-vertical-clip-control/`;
`/tmp/html-css-ellipsis-vertical-clip.log` exits 1.

Code inspection identifies compiler-added vertical line-height padding in
`src/lib.rs` (painted_style.padding additions) as the likely source of the
minimum box height. A fix must separate line-box measurement/baseline spacing
from the authored container's CSS padding; merely clamping the correction
would risk wrapped/auto-height semantics. This layout limitation is retained
under A22 as well. No production change or tolerance adjustment this increment.

The shared short-height bug is now fixed by an internal line-box container.
All12 vertical ellipsis controls pass with direct visual inspection; authored
height8 remains8, while public regression tests also preserve automatic
line-height40. The full1095-case corpus retains identical browser/native
pixels and its three known failures. See clipping-review.md “Internal line-box
container (validated)” for build, parity, regression and full-run evidence.

## DPR qualification

The runtime control now accepts `--dpr-control` (requires the CSS experiment
flag and a separate output directory). Eight scenes are each compiled once
at390, then resized240/390/768 at DPR1/2/3. Native layout stays in CSS pixels;
the replay and Chrome images use physical pixels. Glyph-cache use, geometry,
PNG dimensions and baseline native/WASM parity are checked.

`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-css-dpr node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --dpr-control`
passes69/72. All geometry passes. Only fractional width/padding at DPR2 fails
local RGB, at all three viewport sizes. Logs `/tmp/html-css-ellipsis-dpr.log`
exit1; all source/scenes/streams/images/results are retained in
`ellipsis-css-dpr/`. All48 DPR2/3 comparisons were directly visually inspected.
The24 DPR1 comparisons have all48 browser/native PNGs byte-identical to the
previously inspected glyph run, recorded in `dpr1-identity.json`. No tolerance
changed. Long/fitting titles, narrow A/combining/ffi and wider ffi pass across
all three DPR values.

A same-source clipping control uses `--dpr-control --clip-only --fractional-control`
and output `ellipsis-fractional-clip-dpr/`. Both ellipsis flags are disabled
inside the probe and Chrome omits text-overflow. It passes6/9, retaining the
same three DPR2 local-RGB failures; all9 comparisons were visually inspected.
Log `/tmp/html-css-ellipsis-clip-dpr.log` exits1. This establishes that the
fractional-position failure is not specific to ellipsis, but does not yet
isolate the exact raster/positioning cause. It remains a shared text residual.

This is still diagnostic runtime coverage: CSS text-overflow acceptance,
production transport, mixed-run/block-style handling, bidi, decoration
integration and host-transform coverage are not qualified by these results.

## Multiple shaped runs and word spacing

The experimental runtime now prepares a complete LTR line from multiple
shaped runs. Whole retained runs preserve their original glyphs and advances;
only the partially retained run is reshaped. Source indices remain global
Unicode scalar offsets, and the marker retains the original line advance.
The preparation API accepts an independent marker style. The diagnostic Text
caller still supplies its first source style, so block-style transport for
mixed HTML styles is not qualified. Bidi and shared clusters split across
runs remain rejected by this experimental preparation path.

The focused runtime suite passes 9/9, including the real Inter fixture tests:
`NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf cargo test -p nuxie-runtime --lib css_ellipsis -- --include-ignored`.
Log: `/tmp/html-ellipsis-multirun-tests.log`. The new test verifies preserved
whole-run glyphs/advances, a partial second run with a different size/style,
global source offsets and independently sized/styled marker. This is a
shaping-level test, not pixel qualification for arbitrary mixed fonts/styles.

`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-word-spacing node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --word-spacing-control`
passes 18/18: zero, +4px and -2px word spacing, each with long responsive and
72px narrow lines. Each scene is compiled once at 390 and resized to
240/390/768. All 18 comparisons were visually inspected. Geometry, native
glyph-cache use and baseline native/WASM compilation parity pass. Artifacts
are in `output/playwright/html-to-riv/ellipsis-word-spacing/`; log:
`/tmp/html-ellipsis-word-spacing.log`. Thresholds are unchanged.

The original runtime controls pass 24/24 after this change, and all 48
browser/native PNGs are byte-identical to the previously inspected
`ellipsis-css-glyph/` run. The comparison is recorded in
`output/playwright/html-to-riv/ellipsis-multirun-regression/identity.json`;
log: `/tmp/html-ellipsis-multirun-regression.log`. Native feature build and
WASM build pass (`/tmp/html-ellipsis-multirun-build.log` and
`/tmp/html-ellipsis-multirun-wasm.log`). The accepted-scene corpus also passes native/WASM publish-artifact parity
(`node --test --test-name-pattern='accepted scene corpus' tools/html-to-riv/tests/javascript.test.mjs`;
`/tmp/html-ellipsis-multirun-parity.log`, exit 0). Baseline scene parity does
not execute the experimental runtime in WASM; that distinction remains intentional.

CSS `text-overflow` remains rejected. Remaining qualification includes
production transport, block-style markers, mixed-font/style pixels, bidi,
decorations, host transforms and the retained shared DPR2 fractional residual.

## Decorations stop before the marker

The new `--decoration-control` matrix uses underline, line-through and both,
with red 2px strokes, across long responsive, fitting and 40px ligature
lines. Each scene compiles once at 390 and resizes to 240/390/768, DPR1.
The harness checks the existing targeted red-decoration support gate as well
as ordinary pixels, geometry, glyph-cache use and baseline native/WASM parity.

Initial run: 12/27 pass. All 15 truncating cases incorrectly extend decoration
through the synthetic marker; the targeted gate detects every one. Three
combined-decoration narrow cases also fail local RGB. All 27 comparisons
were visually inspected. Reproducers remain in
`output/playwright/html-to-riv/ellipsis-decoration-initial/`, with log
`/tmp/html-ellipsis-decoration-initial.log` (exit 1).

The experimental OrderedLine now records the number of retained source
glyphs. Underline/strikethrough generation stops at that boundary, while the
normal painting iterator continues through the marker. Ordinary OrderedLine
construction leaves decoration coverage unrestricted. This also avoids
using a marker's synthetic source indices for underline skip-ink decisions.

Fixed run: 27/27 pass, all visually inspected, with unchanged thresholds.
Reproduce after the native glyph feature build:
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-decoration-fixed node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --decoration-control`.
Artifacts: `output/playwright/html-to-riv/ellipsis-decoration-fixed/`.
Logs: `/tmp/html-ellipsis-decoration-build.log` and
`/tmp/html-ellipsis-decoration-fixed.log` (both exit 0).

This qualifies the tested single-font DPR1 decoration compositions only.
DPR/host transforms, mixed-style markers and production CSS acceptance remain
pending; the existing fractional-position and vector residuals are unchanged.

Regression evidence for the decoration boundary change:
- Runtime decoration suite: 8/8, including retained-prefix versus marker
  coverage and unchanged ordinary-line coverage. Command:
  `NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf cargo test -p nuxie-runtime --test css_decoration`.
  Log `/tmp/html-ellipsis-decoration-tests-final.log`, exit 0. The initial
  test-only compile error (Paragraph has no Default) was corrected; its log
  remains `/tmp/html-ellipsis-decoration-tests.log`.
- Runtime WASM compile check passes:
  `RUSTC=$(rustup which rustc) cargo check -p nuxie-runtime --locked --target wasm32-unknown-unknown --no-default-features`.
  Log `/tmp/html-ellipsis-decoration-runtime-wasm.log`, exit 0.
- Compiler WASM build passes (`validation/build-wasm.sh`, log
  `/tmp/html-ellipsis-decoration-wasm.log`). The full accepted-scene corpus
  passes native/WASM publish-artifact parity (`node --test
  --test-name-pattern='accepted scene corpus' tools/html-to-riv/tests/javascript.test.mjs`,
  log `/tmp/html-ellipsis-decoration-parity.log`, exit 0).
  These are compilation/artifact checks, not WASM-rendered pixel comparisons.

## Decoration DPR qualification

The same nine decoration fixtures now pass 81/81 comparisons at DPR1/2/3,
resizing each compiled scene to 240/390/768. All 54 DPR2/3 comparisons were
directly visually inspected. The 27 DPR1 cases have all 54 browser/native
PNGs byte-identical to the previously reviewed `ellipsis-decoration-fixed/`
run; `ellipsis-decoration-dpr/dpr1-identity.json` records that check.

Reproduce:
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-decoration-dpr node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --decoration-control --dpr-control`.
Log `/tmp/html-ellipsis-decoration-dpr.log` exits 0. Artifacts, scene bytes,
source maps, requirements, bounds, streams and images are retained under
`output/playwright/html-to-riv/ellipsis-decoration-dpr/`. The ordinary pixel,
targeted red-decoration, geometry, physical-image dimensions, glyph-cache
and baseline compiler native/WASM parity gates all pass. No code or threshold
change was required for the higher-DPR run.

This matrix covers Inter at 24px/40px with 2px strokes at integer container
positions. It does not clear the separately retained fractional-position
DPR2 text residual or qualify arbitrary host transforms and fonts.

## Letter spacing excludes the synthetic marker

`--letter-spacing-control` adds zero, +2px and -1px letter spacing across
long responsive, narrow 40px ffi and narrow 40px combining-mark lines. Each
scene compiles at 390 and resizes to 240/390/768, DPR1.

The initial run passes 24/27 automated comparisons. All three +2px narrow
ffi cases fail local RGB: Chrome retains two f characters while native
retains one. Visual review of all 27 cases additionally catches an extra p
in the -1px long title at 240, despite the ordinary pixel gate passing that
case. This is preserved as evidence that aggregate pixel gates alone cannot
prove truncation semantics. Initial artifacts remain under
`output/playwright/html-to-riv/ellipsis-letter-spacing-initial/`, log
`/tmp/html-ellipsis-letter-spacing-initial.log` (exit 1).

Chrome's pinned LineTruncator::SetupEllipsis shapes the marker directly,
without source letter-spacing application. The experimental preparation now
sets marker letter_spacing to zero; source runs retain their spacing. This
corrects both overly early positive-spacing truncation and overly late
negative-spacing truncation. No tolerance change was made.

The fixed run passes 27/27, including geometry, actual native glyph use and
baseline native/WASM artifact parity. All seven changed native images were
directly visually inspected, alongside two unchanged cases on those sheets.
The remaining native images are byte-identical to the reviewed initial run;
all 27 Chrome references are identical. The 54-image comparison is retained
in `ellipsis-letter-spacing-fixed/initial-identity.json` (47 identical).
Reproduce:
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-letter-spacing-fixed node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --letter-spacing-control`.
Logs `/tmp/html-ellipsis-letter-spacing-build.log` and
`/tmp/html-ellipsis-letter-spacing-fixed.log` exit 0. These are experimental
runtime controls: compiler CSS acceptance remains pending.

The focused runtime suite passes 10/10 with the real Inter fixtures:
`NUXIE_TEXT_TEST_FONT=$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf cargo test -p nuxie-runtime --lib css_ellipsis -- --include-ignored`.
Log `/tmp/html-ellipsis-letter-spacing-tests.log`, exit 0. The added test
asserts retained scalar counts for 40px ffi lines at zero/+2/-1 spacing,
checks source spacing is preserved and verifies identical unspaced marker
advances across all three. This directly guards the discovered truncation
boundary, rather than relying solely on average pixel differences.

The updated runtime also passes the host-free WASM compile check:
`RUSTC=$(rustup which rustc) cargo check -p nuxie-runtime --locked --target wasm32-unknown-unknown --no-default-features`.
Log `/tmp/html-ellipsis-letter-spacing-runtime-wasm.log`, exit 0. This checks
compilation compatibility; it does not claim WASM runtime pixel execution.

## Host transforms and state restoration

`glyph-state-control.mjs ellipsis` now compares one compiled scene containing
a responsive long title, a 40px ffi title with +2px letter spacing and a
fitting sibling. At 240/390/768 it tests identity, fractional translation,
external clipping through glyphs, half opacity, uniform and nonuniform scale,
rotation, shear and combined rotation/clip/opacity. The probe enables both
experimental ellipsis flags, verifies native glyph-cache use, and compares
native/WASM baseline scene bytes, source maps and runtime requirements.
Geometry, ordinary pixels and per-region ink coverage gates are unchanged.

The initial fixture omitted display:block, allowing the shared reset to leave
Chrome paragraphs as flex boxes. Chrome therefore clipped without applying
ellipsis while the diagnostic probe enabled ellipsis. Its 6/27 outcome is
invalid as runtime qualification, not an ellipsis regression. It is retained
in `output/playwright/html-to-riv/ellipsis-host-initial/` and
`/tmp/html-ellipsis-host-initial.log` (exit 1). Only its nine 240px comparisons
were visually inspected. The fixture now explicitly uses text-only blocks;
a new computed-style assertion checks block/nowrap/hidden/ellipsis on every
Chrome reference paragraph (exercised by the restored run below).

Corrected isolated run: 27/27, all visually inspected.
`NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-host-block node tools/html-to-riv/validation/glyph-state-control.mjs ellipsis --no-restore`.
Log `/tmp/html-ellipsis-host-block.log`, exit 0.

Restored-state run: 27/27, all visually inspected. It draws a second copy at
an untransformed offset after restoring host state, checking that transforms,
opacity and clipping do not leak into subsequent drawing.
`NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-host-restored node tools/html-to-riv/validation/glyph-state-control.mjs ellipsis`.
Log `/tmp/html-ellipsis-host-restored.log`, exit 0.

Both directories retain source, scene, requirements, map, host-state inputs,
bounds, glyph data, render streams, browser/native/diff PNGs and review sheets.
No runtime production code or thresholds changed for this increment. These
are Inter, undecorated DPR1 controls; they do not clear the retained
fractional-position DPR2 text failure or qualify decorated group opacity.
CSS text-overflow acceptance and production transport remain pending.

## Reserved occurrence contract

The portable requirements API now recognizes `text-css-single-line-ellipsis-v1`
and its occurrence policy `css-single-line-ellipsis-v1`. The policy names a
Text object in the imported default artboard, using the existing version 2–4
occurrence schema. It requires the CSS shaping precision capability. Hosts
must explicitly advertise support; target validation precedes installation.
Missing precision, missing capability/occurrence pairs, version 1 use and
duplicate targets are rejected. This is a reserved contract: the compiler
still rejects text-overflow and does not emit the new policy; the checked
native host does not advertise it and rejects it before drawing.

Public contract regression:
`cargo test -p nuxie-html-to-riv --features native-glyph-controls --test contract ellipsis_occurrence`.
Passes 1/1 (`/tmp/html-ellipsis-contract-tests.log`, exit 0), checking valid
serialization/validation, unavailable hosts, wrong targets, absent policies,
missing shaping precision and duplicates.

Next integration work is concrete: validate single-line occurrence
preconditions before installing runtime behavior, add compiler cascade and
emission for text-overflow, and route visual fixtures through the public CSS
and manifest path instead of probe environment overrides. Broadening beyond
the proven LTR single-line behavior remains separate work; the policy name
does not claim bidi or arbitrary multiline support.

Contract integration checks pass:
- Full public contract suite: 45/45 (`cargo test -p nuxie-html-to-riv
  --features native-glyph-controls --test contract`;
  `/tmp/html-ellipsis-contract-regression.log`).
- Native host requirements suite: 2/2 (`node --test
  tools/html-to-riv/tests/runtime-requirements.test.mjs`;
  `/tmp/html-ellipsis-contract-host-tests.log`). The new test compiles a real
  nowrap scene, replaces its manifest with the reserved ellipsis occurrence,
  and verifies missing-runtime-capability rejection with no render stream.
- Native glyph feature build and compiler WASM build pass
  (`/tmp/html-ellipsis-contract-build.log`, `/tmp/html-ellipsis-contract-wasm.log`).

No rendering behavior changes or new CSS acceptance are claimed for this
contract increment. Existing diagnostic pixel receipts remain applicable to
the experimental renderer, not yet to manifest-installed ellipsis.

The accepted-scene corpus passes native/WASM publish-artifact parity after
the contract addition (`node --test --test-name-pattern='accepted scene corpus'
tools/html-to-riv/tests/javascript.test.mjs`;
`/tmp/html-ellipsis-contract-parity.log`, exit 0).

## Checked runtime installation

The runtime now exposes `validate_css_single_line_ellipsis` and
`install_css_single_line_ellipsis`. Validation requires nowrap with no CSS
soft-wrap policy, no text modifiers, resolved paint styles with the same
font asset/size/line height, and static LTR content without breaks, tabs,
paragraph/segment separators or bidi controls. Separate spacing styles are
allowed, preserving the compiler's word-spacing representation. The host
must provide CSS shaping precision and an enclosing overflow clip. Clipping
is a host/compiler precondition, not inferred by this Text installer.

The checked native host advertises the capability and validates all ellipsis
targets before installing any occurrence policies. Installation enables CSS
nowrap alignment and CSS ellipsis on each named Text only. Runtime layout
still recomputes truncation when the imported scene is resized. CSS
text-overflow parsing/emission remains pending; this completes the native
installation path, not public compiler authoring support.

The native requirements suite passes 2/2 after the final checks (`node --test
tools/html-to-riv/tests/runtime-requirements.test.mjs`;
`/tmp/html-ellipsis-installer-host-tests-checked.log`, exit 0). It compares
manifest-installed and diagnostic render streams without diagnostic flags
on the installed path, verifies rejection by a host lacking the capability,
and rejects ordinary wrapping, explicit br and pre tabs before any stream
is written. Bidi/modifier/mixed-font rejection is implemented but not covered
by these host fixtures. Native build passes:
`/tmp/html-ellipsis-installer-build-checked.log`.

The visual harness adds `--manifest-control`. It checks native/WASM baseline
compiler artifacts first, then rewrites the occurrence manifest to request
ellipsis and forces both diagnostic probe flags off. This intentionally
does not claim that the compiler emitted the ellipsis requirement itself.

Initial installed controls pass 24/24, with all 48 browser/native PNGs
byte-identical to the previously visually reviewed `ellipsis-css-glyph/`
run. Evidence: `output/playwright/html-to-riv/ellipsis-installed/diagnostic-identity.json`;
log `/tmp/html-ellipsis-installed.log`, exit 0. This run preceded the final
separator/font consistency checks. The final checked installer additionally
passes all 18 word-spacing comparisons; all 36 PNGs are byte-identical to the
reviewed `ellipsis-word-spacing/` controls. Evidence:
`output/playwright/html-to-riv/ellipsis-installed-word-spacing/diagnostic-identity.json`;
log `/tmp/html-ellipsis-installed-word-spacing.log`, exit 0. No fresh manual
pixel review is claimed for these byte-identical images.

Commands (the environment flag selects expanded fixtures; `--manifest-control`
overrides both probe flags to zero):
`NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR=$PWD/output/playwright/html-to-riv/ellipsis-installed node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --manifest-control`;
add `--word-spacing-control` with output directory `ellipsis-installed-word-spacing`
for the final checked multi-run controls. Geometry, native glyph-cache use and
ordinary pixel gates pass without tolerance changes.

The final installer passes the host-free runtime WASM compile check:
`RUSTC=$(rustup which rustc) cargo check -p nuxie-runtime --locked --target wasm32-unknown-unknown --no-default-features`;
`/tmp/html-ellipsis-installer-runtime-wasm.log`, exit 0. No WASM runtime pixel
execution is claimed.

## Public CSS compiler path

Public `text-overflow:clip|ellipsis` parsing and occurrence emission are now
implemented; see SUPPORT.md's current A23 section for the exact subset.
Three public compiler tests exercise emission, unsupported contexts/syntax, and
cascade/reset behavior. Full module tests pass 112/112, including all 45 contract
tests. Native and WASM builds pass; the complete accepted scene corpus passes
native/WASM byte, source-map and requirement parity.

The public-path control run passes 24/24 geometry and native pixel comparisons
at 240/390/768 using each scene compiled once at 390. Diagnostic flags are disabled
inside the probe and the emitted policy is asserted. Its 48 Chrome/native PNGs
are byte-identical to the previously inspected glyph diagnostic run; receipt:
`output/playwright/html-to-riv/ellipsis-public-css/diagnostic-identity.json`.

Five permanent corpus fixtures pass 15/15 at those widths: responsive titles,
40px-font narrow ligatures with letter spacing, word spacing, underline plus
strikethrough, and overflow:clip. All five comparison sheets were inspected,
including the final corrected ligature sheet: retained text and clipped markers
match Chrome, decorations stop before the marker, sibling layout is stable, and
wide titles regain their full text. The first ligature fixture accidentally fit;
increasing its font exposed a fixture-only unsupported line-height diagnostic.
Setting its line height to 64px produced the intended accepted truncation case.
Initial failure logs are retained; they are not claimed as renderer failures.
Gallery: `output/playwright/html-to-riv/ellipsis-public-corpus/gallery.html`.

Reproduce:

```sh
cargo test -p nuxie-html-to-riv --features native-glyph-controls
cargo build -p nuxie-html-to-riv --features native-glyph-controls --bin html-to-riv --example probe
bash tools/html-to-riv/validation/build-wasm.sh
node --test --test-name-pattern='accepted scene corpus' tools/html-to-riv/tests/javascript.test.mjs
NUXIE_EXPERIMENTAL_CSS_ELLIPSIS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/ellipsis-public-css" node tools/html-to-riv/validation/ellipsis-runtime-control.mjs --public-css
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_REVIEW_DIR="$PWD/output/playwright/html-to-riv/ellipsis-public-corpus" npx --prefix tools/html-to-riv playwright test --config tools/html-to-riv/playwright.config.mjs --project=native --grep 'text-ellipsis-' --output tools/html-to-riv/test-results-ellipsis-public
```

Logs: `/tmp/html-ellipsis-public-module-final.log`,
`/tmp/html-ellipsis-public-parity-final.log`, `/tmp/html-ellipsis-public-css.log`,
`/tmp/html-ellipsis-public-corpus-final.log`. No thresholds changed. A23 remains
partial: prior fractional DPR2 and vector failures are retained, and broader
fonts, mixed-font/bidi and multiline ellipsis remain unqualified.

## Open Sans ligature investigation

The runner accepts `NUXIE_ELLIPSIS_FONT` and `NUXIE_ELLIPSIS_FONT_FAMILY` to
serve and embed identical alternate font bytes. OpenSans-Regular.ttf public CSS
controls initially pass18/24. All24 comparison pairs were visually inspected:
ligature-24 and ligature-40 fail at all three viewport widths. Chrome preserves
its original joined ffi glyph; native paints a reshaped f/ff prefix and puts the
marker too early. Geometry and native/WASM artifact parity pass.
Artifacts: `output/playwright/html-to-riv/ellipsis-public-opensans`.

The same font's clip-only control passes23/24; all24 pairs inspected. All narrow
ligature cases pass; only the long line at768 fails local RGB, independently of
ellipsis. Artifacts: `output/playwright/html-to-riv/ellipsis-opensans-clip-control`.
Letter-spacing controls initially pass20/27: zero-spacing narrow cases fail at
all widths; four long-line cases also fail. These spacing images are not yet
visually reviewed. Artifacts: `output/playwright/html-to-riv/ellipsis-opensans-spacing-initial`.

A focused real-font regression confirmed the mismatch: the original glyph605
was replaced by glyph73 (f). Red log `/tmp/html-ellipsis-opensans-red.log`.
Chrome's pinned `LineTruncator::TruncateText` creates a ShapeResultView of the
original shape; it does not reshape a prefix. Runtime preparation now retains
original glyphs whose cluster begins before the source cut and positions the
marker after their full advances. Qualification of this correction is pending;
these initial failing reproducers remain preserved.

### Original-glyph correction validated

Focused runtime tests11/11 pass (`/tmp/html-ellipsis-opensans-green.log`), native
build and host-free runtime WASM check pass. The metadata-only break-index clamp
was applied after the focused test run and included in the native/pixel and WASM
checks. Open Sans public controls now pass24/24. Exactly six native images change
(the previously failing narrow ligatures); all six were visually inspected and
now preserve ffi like Chrome. Other42/48 PNGs equal the initially reviewed run.
Inter regression passes24/24 with48/48 PNGs unchanged from its reviewed public CSS
run. Per-file identity receipts live in each output directory below.

Open Sans letter-spacing matrix improves20/27 to23/27. All27 final pairs were
visually inspected: all narrow ligature and combining cases pass; four long-line
RGB failures remain (spacing0 and+2 at768; spacing-1 at390/768). The three768
failures display the complete untruncated source, so these are not all caused by
ellipsis selection. Keep the390 negative-spacing case for further isolation.
No thresholds were changed and no broad Open Sans raster qualification is claimed.
Native/WASM compile artifacts agree in every control run.

Artifacts:
- `output/playwright/html-to-riv/ellipsis-opensans-retained-glyphs`
- `output/playwright/html-to-riv/ellipsis-inter-retained-glyphs`
- `output/playwright/html-to-riv/ellipsis-opensans-spacing-retained-glyphs`

Reproduce font controls with the prior public-path command, adding
`NUXIE_ELLIPSIS_FONT=OpenSans-Regular.ttf NUXIE_ELLIPSIS_FONT_FAMILY=OpenSans`
and a distinct `NUXIE_HTML_REVIEW_DIR`; add `--letter-spacing-control` for the
spacing matrix. Runtime regression:

```sh
NUXIE_TEXT_TEST_FONT="$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf" NUXIE_ELLIPSIS_LIGATURE_FONT="$PWD/tools/html-to-riv/tests/assets/OpenSans-Regular.ttf" cargo test -p nuxie-runtime --lib css_ellipsis -- --include-ignored
```

Earlier descriptions of reshaping a partial prefix are superseded by the retained
original-glyph implementation. Existing scalar/grapheme selection remains; the
painted glyph and marker advance include the intersected original cluster.

### Shared spacing residual isolated; permanent Open Sans corpus

Open Sans letter-spacing controls with ellipsis disabled pass22/27. All four
remaining ellipsis failures recur in this control, plus zero-spacing390. The
three long-text sheets (nine pairs, including all failures) were visually
inspected; errors accumulate in later text rather than selecting different
ellipsis source characters. This isolates an ellipsis-independent rendering
residual, not its exact cause. The six narrow/combining clip-only sheets were
not re-inspected in this run. Evidence:
`output/playwright/html-to-riv/ellipsis-opensans-spacing-clip-only/results.json`
and `ellipsis-failure-overlap.json` in that directory.

Three permanent `cases.json` entries cover Open Sans ligatures at24px and40px
box widths and a responsive media card with an image, flexing copy column,
title and subtitle ellipsis. They pass9/9 Chrome/native comparisons at240/390/768
from scenes compiled once at390. All nine pairs were visually inspected:
ligatures stay intact, both text blocks shorten on narrow cards, full text
returns when there is room, and the image/container geometry remains stable.
Artifacts: `output/playwright/html-to-riv/ellipsis-opensans-corpus/gallery.html`.
The full compiler suite112/112 and expanded accepted native/WASM corpus parity
pass (`/tmp/html-ellipsis-opensans-module.log`,
`/tmp/html-ellipsis-opensans-corpus-parity.log`).

Reproduce the control with the Open Sans runner environment and
`--clip-only --letter-spacing-control`. Reproduce permanent pixels using the
normal native Playwright command with `--grep 'text-ellipsis-opensans'`.
No acceptance/tolerance changes were made. The shared Open Sans raster issue,
fractional DPR2 issues, and wider renderer/font qualification remain open.

### Retained-glyph composition regression

After original-glyph retention, the public CSS Inter decoration matrix passes
81/81 across DPR1/2/3 and viewport240/390/768; word-spacing passes18/18. All162
and36 Chrome/native PNGs respectively are byte-identical to the earlier visually
reviewed decoration/word-spacing runs. Per-file receipts are `prior-identity.json`
in `output/playwright/html-to-riv/ellipsis-retained-decoration-dpr` and
`ellipsis-retained-word-spacing`. This transfers the existing visual evidence
without claiming a new manual review of identical images. Compiler/WASM artifacts
match and native glyph cache population is checked by each run.

Open Sans decoration expansion passes24/27. All27 pairs were visually inspected.
Narrow ffi ligatures retain their shapes and decorations end before the marker;
short and truncated long lines pass. The three full-width768 lines fail RGB and
red-decoration gates: underline, line-through, and combined. All three show the
full source with no ellipsis, so this is not a truncation-selection failure;
exact shared renderer cause remains unresolved. Failures remain in
`output/playwright/html-to-riv/ellipsis-opensans-decorations`.

Reproduce using the public CSS runner with `--decoration-control --dpr-control`
for Inter, `--word-spacing-control` for its word spacing, and the Open Sans font
environment with `--decoration-control` for the new font expansion. Logs:
`/tmp/html-ellipsis-retained-decoration-dpr.log`,
`/tmp/html-ellipsis-retained-word-spacing.log`,
`/tmp/html-ellipsis-opensans-decorations.log`. No tolerance changes.

### Shared Open Sans residual resolved

The long-line spacing/decoration failures were legacy kerning fallback through
an empty GPOS table. See `opensans-kerning-review.md`: spacing27/27, decorations
27/27 and the full Open Sans permanent corpus48/48 now pass with visual review.
Earlier failed results remain historical reproducers. Native/WASM parity passes.
