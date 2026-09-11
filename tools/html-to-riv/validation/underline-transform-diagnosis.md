# A20 residual transform diagnosis

The unchanged colored-glyph host suite passes 17/27. This diagnostic adds
`--transparent-glyphs` to `glyph-state-control.mjs underline --no-restore`, making
glyph paint transparent while retaining the same shaped text, underline color,
skip-ink exclusions and geometry. It isolates decoration pixels from glyph
coverage. This is a diagnostic, not a replacement for the colored-glyph gate.

23/27 transparent comparisons pass. All geometry, ordinary image and ink-coverage
checks pass; four red-support failures remain. All 27 pairs were visually
inspected in the three full contact sheets. Six colored-glyph failures disappear
when glyph paint is removed, evidence that overlap contributes to them. It does
not establish a complete root cause or qualify those original failing cases.

`red-decoration-locations.mjs <state-review-dir>` reads existing PNGs and writes
`red-locations.json`, with exact missing/extra coordinates and both RGBA values.
It checks its counts against the shared gate to catch diagnostic drift. It does
not modify images or thresholds.

Remaining transparent cases:

| Case | Pixel | Chrome RGBA | Native RGBA |
| --- | --- | --- | --- |
| nonuniform scale, 240 | 185,61 | 255,25,25,255 | 255,255,255,255 |
| nonuniform scale, 240 | 185,62 | 255,76,76,255 | 255,255,255,255 |
| shear, 240 | 160,94 | 255,202,202,255 | 255,155,155,255 |
| rotation, 390 and 768 | 370,107 | 255,212,212,255 | 255,181,181,255 |

The latter two locations have faint browser ink below the red mask cutoff and
stronger native ink above it; they are not necessarily binary clip errors. Do
not remove them from the existing gate on that basis.

## Translation sweep

`--clip-phase` requires `--no-restore --transparent-glyphs`. It uses the same
compiled scene at 240px with scale (1.4,0.8), y translation 3.5 and seven x
translations. At pixel (185,61):

| x translation | Chrome green channel | Native green channel |
| --- | --- | --- |
| 2.24 | 25 | 26 |
| 2.245 | 25 | 26 |
| 2.249 | 25 | 255 |
| 2.25 | 25 | 255 |
| 2.251 | 25 | 255 |
| 2.255 | 25 | 255 |
| 2.26 | 255 | 255 |

3/7 sweep cases pass; the four middle values preserve the two-pixel failure.
All seven pairs visually reviewed. Native switches between 2.245 and 2.249;
Chrome switches between 2.255 and 2.26. The native stream retains adjacent clip
edges 122.89465 and 123.337524 in text-local coordinates, plus an 8px text
translation. The first edge maps near device x=185.5025 at tx=2.25, close to the
rounding boundary. This is evidence for comparing outline-intercept precision
and clip rasterization independently; it does not justify adding an epsilon or
snapping compiler coordinates. No runtime fix is claimed in this diagnostic step.

Artifacts: `output/playwright/html-to-riv/underline-hard-clip-transparent/` and
`underline-hard-clip-phase/` retain source requests, compiled scene, host states,
streams, PNGs, geometry, results and pixel locations. Commands accept
`NUXIE_HTML_REVIEW_DIR` to preserve separate runs. Logs:
`/tmp/html-hard-clip-transparent.log`, `/tmp/html-hard-clip-transparent-locations.log`,
`/tmp/html-hard-clip-phase.log`, `/tmp/html-hard-clip-phase-locations.log`.
Both diagnostic scripts pass `node --check`; `git diff --check` passes. Compiler,
WASM and runtime code are unchanged, so their suites were not rerun this step.

## Rejected SVG proxy and extraction-scale experiment

`hard-clip-browser-reference.mjs` renders the two recorded exclusion rectangles
and the stripe without text, using nested SVG clip paths in Chrome and the hard
clip operation in Metal. It deliberately retains both `auto` and `crispEdges` SVG
variants. They produce identical antialiased clipping at the sampled location:
green progresses through 126–131 over the translation sweep, instead of the
text case's binary jump. SVG `crispEdges` therefore does not provide the required
hard-clip reference here. This control is evidence against that substitution,
not a failed native renderer gate. Artifacts:
`output/playwright/html-to-riv/hard-clip-browser-reference/`.

The exact Skia revision from pinned Chrome DEPS is now readable through the
official GitHub mirror, despite the earlier Gitiles 404 responses:
[SkTextBlob.cpp](https://github.com/google/skia/blob/9d07e5bad9e3e21da2426946e589daa647218271/src/core/SkTextBlob.cpp)
and [SkFontPriv.h](https://github.com/google/skia/blob/9d07e5bad9e3e21da2426946e589daa647218271/src/core/SkFontPriv.h).
The intercept implementation requests unhinted paths at canonical size 64, then
scales intervals to the text size. Our ordinary normalized font path is extracted
at size 2048. An experiment introduced a CSS-only extraction path at 64, leaving
ordinary glyph drawing unchanged. It did not fix the residual: six native sweep
PNGs remained byte-identical; tx=2.245 regressed. The sweep went from 3/7 to 2/7
passing. The experiment was reverted; its outputs remain in
`output/playwright/html-to-riv/underline-intercept-64-phase/` and logs
`/tmp/html-intercept-64-build.log`, `/tmp/html-intercept-64-phase.log`.

The size difference alone is not the explanation. Pinned Skia has distinct
[CoreText path generation](https://github.com/google/skia/blob/9d07e5bad9e3e21da2426946e589daa647218271/src/ports/SkScalerContext_mac_ct.cpp)
and [Fontations path generation](https://github.com/google/skia/blob/9d07e5bad9e3e21da2426946e589daa647218271/src/ports/SkTypeface_fontations.cpp).
Next compare actual outline coordinates/scaler behavior for the relevant glyph,
then interception arithmetic. Do not infer which path Chrome selected merely
from the operating system, or adopt the regressing scale change for source
similarity. Source snapshots are `/tmp/html-pinned-Sk*.{cpp,h}`; no CoreText
production dependency was introduced into the compiler or runtime.

After reverting, the native probe builds and all seven sweep PNGs are
byte-identical to the pre-experiment baseline (3/7 passing, four preserved
failures). Reverted artifacts: `underline-intercept-64-reverted/`; logs:
`/tmp/html-intercept-64-reverted-build.log` and
`/tmp/html-intercept-64-reverted-phase.log`. No production change from this
experiment remains.

## Shaped advance precision explains the translation interval

`outline_probe.rs` now records accumulated origins and accepts a diagnostic text
argument. `coretext-outline-reference.swift` reads the exact supplied face and
exports normalized paths plus unshaped advances; spaces correctly have no path.
For q/j, CoreText and runtime stripe intercept differences are below 0.00006px
at 24px. This is too small to account for the transition alone. CoreText is a
diagnostic comparison, not the acceptance target or proof of Chrome's backend.

The new `underline-advance-reference.mjs` measures the actual Chrome font at
24px using Canvas prefix widths, total-minus-suffix widths, and DOM ranges.
Prefix/suffix disagreement at `y` exposes contextual `gy` kerning: unshaped
CoreText advance sums must not be used as browser glyph origins. At `q`, both
Canvas measurements agree at 108.21597290039062px. Runtime shaping reports
108.22265625px, a 0.006683349609375px difference. DOM Range edges are rounded
and are not substituted for these measurements.

Using that difference with the recorded native q outline and clip padding
predicts the device half-pixel transition at tx=2.2568420410156307 for Chrome,
versus tx=2.247485351562517 for native. Both fall inside their independently
measured phase intervals: Chrome (2.255, 2.26], native (2.245, 2.249]. This makes
accumulated shaping precision a supported explanation for this specific scale
failure. It does not yet explain rotation/shear or glyph-overlap residuals.
Runtime uses integer advances at scale 2048 before conversion to CSS size.

Compact raw measurements and predictions are retained in
`underline-advance-reference.json` (Chrome 153.0.8010.12). Commands:
`node validation/underline-advance-reference.mjs`,
`cargo run -p nuxie-html-to-riv --example outline_probe -- tests/assets/NuxieJapaneseFixture-Regular.otf '／＿ agypqj'`
(the Rust command's asset path is relative to the module), and the Swift
reference with that same font/text. Logs/data are `/tmp/html-{runtime,coretext}-gap-outlines.*`
and `/tmp/html-chrome-gap-advances.*`. JS syntax check passes. No production code
or tolerances changed. Next: test higher-precision shaping behind explicit CSS
semantics, preserving ordinary Rive shaping, then rerun the phase/host controls
and broad text regression corpus before adopting it. A20 remains active.

## Higher-precision shaping experiment

A temporary diagnostic used integer shaping scale `round(font_size * 65536)`
instead of 2048, with the corresponding advance/offset conversion. It changed
no path extraction or clip rasterization. It is retained as
`underline-precision-experiment.patch` for reproduction, but removed from the
runtime source: a process environment variable is not the production policy
interface, and ordinary Rive shaping must remain stable.

With `NUXIE_DIAGNOSTIC_SHAPING_PRECISION=1` in the experiment binary:

- The exact seven-case transparent clip-phase sweep improves from 3/7 to **7/7**.
  All seven browser/native/diff pairs were visually inspected in three contact
  sheets. No missing stripe segments were observed. Artifact directory:
  `output/playwright/html-to-riv/underline-precision-phase/`.
- The full visible-glyph host suite with restored second copy improves from
  17/27 to **18/27**. The nonuniform-scale 240px failure is fixed; opacity,
  rotation and shear still fail red support at all three widths. Geometry,
  ordinary pixel comparison and ink coverage pass all 27. Its visual review is
  still pending; these numerical results are not full qualification. Artifacts:
  `output/playwright/html-to-riv/underline-precision-state/`.

Commands use `glyph-state-control.mjs underline --no-restore
--transparent-glyphs --clip-phase` and `glyph-state-control.mjs underline`, with
separate `NUXIE_HTML_REVIEW_DIR` values. Logs: `/tmp/html-precision-build.log`,
`/tmp/html-precision-phase.log`, `/tmp/html-precision-state.log`. The experiment
strengthens the shaping-precision diagnosis and supports implementing an explicit
CSS font policy. It does not prove that this scale reproduces Chrome for every
font, size, feature, bidi run or fallback. Before adoption, preserve policy across
font variations/features and asset replacement, check finite/extreme sizes, add
public compile/import regressions, and rerun the broad text/resizing corpus.

The restored probe was rebuilt and rechecked: all seven native phase PNGs are
byte-identical to the original baseline (3/7 passing). Artifacts:
`underline-precision-restored/`; logs `/tmp/html-precision-restored-build.log`
and `/tmp/html-precision-restored-phase.log`. No diagnostic environment switch
remains in runtime source.

## Explicit runtime policy implementation

`HbFont::with_shaping_precision(ShapingPrecision::CssExperimental)` now selects
high-precision advances/offsets without changing outline extraction or default
Rive decoding. Font option, spacing, tab and preserved-space clones retain it.
`FontAsset::set_shaping_precision_occurrence` retains the selection across font
replacement/decode/restore; selecting `Rive` restores legacy advances. Scale
conversion uses f64 bounds before conversion to i32, with nonpositive/nonfinite
sizes retaining the legacy scale. Extreme font-size shaping is not qualified by
that numeric guard alone.

The regression `css_shaping_precision_matches_chrome_and_survives_font_replacement`
passes, checking the independent Chrome q-origin measurement, legacy exact
advance, clone chains, replacement after clearing the font, and reversal. Log:
`/tmp/html-css-precision-test.log`. The initial test incorrectly assumed one
shaping run; it now sums across the actual script runs.

The validation probe exposes `NUXIE_CSS_SHAPING_PRECISION=1` to select this API
for imported font assets. This is a host experiment control, not an environment
lookup in runtime code. Compiler capability emission and general adoption are
still pending broad qualification; ordinary probe runs remain unchanged.

The explicit policy passes **7/7** phase comparisons. All seven native PNGs are
byte-identical to the previously visually inspected precision experiment.
Artifacts: `output/playwright/html-to-riv/underline-css-precision-phase/`; log:
`/tmp/html-css-precision-phase.log`. The first attempt was invalidated because a
concurrent default module test build replaced the shared probe without native
glyph support. After the tests finished, the probe was rebuilt with
`native-glyph-controls` and the complete phase run passed. Do not overlap module
Cargo tests/builds with visual runs sharing this executable.

Default module tests pass **77 tests** in `/tmp/html-css-precision-module.log`;
feature-gated glyph tests, native/WASM parity, and broad visual corpus remain
pending. The pure runtime boundary passes (28 packages / 57 tables), log
`/tmp/html-css-precision-boundary.log`; `git diff --check` passes.

The scale-boundary unit test completed successfully (one test), log
`/tmp/html-css-precision-scale.log`. A full native-glyph corpus run is now
underway with the explicit CSS precision option in
`output/playwright/html-to-riv/css-precision-full-glyph/`; no qualification claim
is made until its results and visuals have been reviewed. Future gallery reports
record shaping precision explicitly; older reports display “not recorded”.

Visual follow-up: inspected `underline-precision-state/review-240-3.png`
(rotation, shear and combined controls, including restored copies). Text,
underline extents and clipping agree at composition scale; the known sparse red
support failures remain and are not waived by inspection. The other eight
contact sheets still require review.

Completed the remaining eight contact-sheet reviews for
`underline-precision-state/review-{240,390,768}-{1,2,3}.png`: all 27 host cases
have now been inspected, including transformed and restored copies. Line breaks,
text extents, clipping, scale, opacity and restored state agree at composition
scale. Sparse red differences remain visible in the difference panels and the
nine numeric failures remain open. This completes the experiment's visual
inspection, not qualification of all host transforms or the full corpus.

## Opacity operation mismatch (P05)

`opacity-overlap-reference.mjs` compares two overlapping red/black rectangles.
The native probe's operation is `modulate_opacity`; the underline host control
uses CSS opacity on the whole transformed parent. Runtime `modulateOpacity`
only multiplies `current_state().modulatedOpacity` in `rive_renderer_cpp.rs`;
it does not isolate the group. Existing non-overlapping glyph controls could
not expose this semantic distinction.

Chrome 153 group opacity produces overlap RGBA [126,126,126,255], while native
per-draw modulation produces [127,64,64,255]. Chrome per-element opacity gives
[127,63,63,255]. Across all pixels native matches per-element opacity within one
channel value, but differs from group opacity by up to 63. All three small images
were visually inspected; the overlap distinction is clear. Results and PNGs:
`output/playwright/html-to-riv/opacity-overlap-reference/`; log
`/tmp/html-opacity-overlap.log`. This is diagnostic, not a relaxed acceptance gate.

The underline opacity residuals must therefore be considered alongside P05 group
compositing. This control establishes the operation mismatch; it does not by
itself attribute every failed glyph-edge pixel. Preserve the existing host
failures until true group semantics are implemented/qualified. No compiler
opacity syntax is enabled and no existing reference was changed.

## Full native-glyph corpus with explicit CSS precision

The full run finished in 8.1 minutes: **1014/1015 runner checks pass**, including
**1004/1005 browser/native visual cases** and all **90/90 underline cases**.
The only failure is the existing `em-layout-cascade` at 240px (449 mismatched
pixels, ratio 0.00584635, geometry within 0.1px). No thresholds changed.
`compare-reviews.mjs` compares this report with `underline-full-gate`: 966 shared
cases, zero removed, zero source changes, zero regressions, one unchanged
failure, and 39 added cases. Metrics differ in 57 shared cases. Note that the
older baseline predates hard-clip integration too: this is a combined runtime
regression comparison, not isolation of shaping precision alone.

Artifacts: `output/playwright/html-to-riv/css-precision-full-glyph/`, with
`review.json`, `gallery.html`, `baseline-comparison.json`, and
`run-configuration.json`. The sidecar records the actual precision environment
and probe hash because this run loaded the older gallery reporter before the
new metadata field was added. Source/scene/PNG artifacts are retained under
`tools/html-to-riv/test-results-css-precision-full-glyph/`. Log:
`/tmp/html-css-precision-full-glyph.log`.

`review-changes.mjs` generates full-resolution browser/native/diff sheets for
all 57 changed-metric and 39 added cases. Their visual inspection is pending;
passing numeric results alone do not complete qualification. Compiler capability
wiring, vector qualification, and the documented host-transform/P05 limitations
remain open.

Contact-sheet progress: `changed-0.png` inspected (space-break-widths-0 and
preline-ogham at all three widths). Wrapping, centered spacing, text baselines
and backgrounds agree. Sheets 1–15 remain pending; this is 6/96 changed/added
cases inspected, not a claim that the full corpus was visually reviewed.

Native/WASM publish parity also passes: all five JavaScript tests, including the
entire accepted scene corpus, in `/tmp/html-css-precision-js-parity.log`. The
compiler/schema is unchanged by the opt-in runtime policy; parity verifies the
current native and existing WASM artifacts still agree, not that the pending
precision capability has been emitted.

## Changed-image review and vector follow-up

All **96/96 changed/added native-glyph cases** have now been visually inspected
in `css-precision-full-glyph/changed-0.png` through `changed-15.png`. This includes
all underline fixtures, Ogham preserved spaces and space-break widths. Wrapping,
alignment, ancestor origins, independent decoration colors, whitespace extents,
CJK skip policies, hidden content and transparent controls agree with Chrome.
Glyph/underline raster differences remain visible (notably OpenSans from-font);
no new structural defect was found and no existing numeric failure is waived.
This is review of the changed/added set, not a claim that all 1005 cases were
freshly inspected in this run.

The explicit-precision vector underline run passes **85/90**, with exactly the
same five failures as the hard-clip baseline: Inter/OpenSans from-font at 390px
and CJK None at 240/390/768. No added/removed/source-changed cases or pass/fail
regressions. Nine cases have changed metrics. Results:
`output/playwright/html-to-riv/css-precision-underline-vector/`; log:
`/tmp/html-css-precision-underline-vector.log`. Vector changed-image review is
next; this focused run does not qualify the entire unrelated vector text corpus.

All nine changed vector cases (CJK None/Auto/All at all widths) were visually
inspected in `css-precision-underline-vector/changed-{0,1}.png`. Their layout and
skip-policy differences agree with Chrome; the denser vector glyph raster
mismatch remains visible and the three None failures remain open. The other two
from-font failures have unchanged metrics, and this step does not claim a fresh
review of all 90 vector pairs. A full vector corpus run is now in progress under
`css-precision-full-vector/`, log `/tmp/html-css-precision-full-vector.log`.

## Actual shaping at extreme sizes

`outline_probe` now accepts optional size and `css` arguments.
`precision-size-reference.py` runs the real shaping backend over nine sizes
(0.01, 0.1, 1, 24, 100, 1000, 16384, 32768, 1000000) for Inter, Open Sans,
the Japanese fixture and Ogham. All 36 samples finish with no missing glyphs
and no negative advances. Normalized total-advance variation against each
font's 24px result is below 0.05% in these samples (largest at tiny sizes).
This tests actual advances across the scale clamp, beyond the scale-conversion
unit test. It is not a browser equivalence result or universal font/feature
bound. Raw results: `validation/precision-size-reference.json`; log
`/tmp/html-precision-sizes.log`; build log `/tmp/html-precision-size-probe-build.log`.
Only the separate outline_probe executable was built while the vector corpus
was running; the active visual probe was not replaced.

The full vector run is still active. Initial failures include normal-line-height
and font-shorthand cases plus the known three em cases. Earlier `em-vector`
metadata confirms the latter were already failing. No suitably labeled prior
vector metadata was found for the first three cases, so they must be rerun
without CSS precision after the full run completes before classifying them as
existing failures or regressions. Do not infer that classification from the
native-glyph results.

## Isolating full-vector failures from precision

`compare-precision-replay.mjs` replays the exact compiled scene and viewport
with the CSS-precision environment removed, writing separate default-precision
artifacts. It compares native PNG bytes and bounds bytes; it does not replace
the Chrome screenshot or recompile authoring data.

The first 12 failing full-vector cases have **byte-identical native PNGs and
bounds** at default precision: font-shorthand-normal-resets 390, normal-line-height-
cascade 240/390, em-layout-cascade 240, em-nested-typography 240,
em-font-shorthand-final-size 240, prewrap-long-word 240/390,
prewrap-forced-hang 240/390/768, and prewrap-word-after-short 768. These failures
therefore exist independently of the precision option, even where an older
labeled baseline report was unavailable. They remain real vector limitations.

Artifacts: `css-precision-vector-baseline/` and
`css-precision-vector-baseline-extra/` under the output directory. Logs:
`/tmp/html-vector-precision-baseline.log` and
`/tmp/html-vector-precision-baseline-extra.log`. The full run remains active;
additional failures must be checked before a complete regression conclusion.

## Wide advance overflow found before capability adoption

The four-font size sweep did not cover glyphs wider than one em. A checksum-valid
synthetic derivative of the Japanese fixture changes one hmtx advance to two em.
With the old i32-max scale clamp it shapes correctly at 24px but incorrectly
returns -32768px at 16384px, then near-zero negative advances at larger sizes.
Before-fix measurements are retained in `precision-wide-advance-before.json`.

The scale bound now uses font units-per-em times 16384, bounding the u16 hmtx
advance before multiplication and reserving signed headroom for positioning.
Ordinary Rive precision remains 2048. `precision-wide-advance.py` regenerates
the temporary font, recomputes SFNT checksums and asserts actual positive,
correct two-em advances at 24, 16384, 32768 and 1000000px. The earlier full
native/vector runs used the pre-guard executable; targeted post-fix validation
is required before adopting the policy. The active vector run's executable
has not been replaced (only the separate outline_probe is being rebuilt).

The wide-advance regression passes after the bound correction: advances are
exactly 48, 32768, 65536 and 2000000px for the four tested sizes. Before/fixed
measurements are in `precision-wide-advance-{before,fixed}.json`. The main
`validation/run.sh` now builds outline_probe and runs this actual-shaping
regression. Its shell syntax and `git diff --check` pass. Post-fix main-probe
phase checks and updated scale-unit tests remain to run.

## Full vector result and default-precision classification

The pre-bound-correction full vector run finished: **987/1015 checks pass**,
**977/1005 visual cases pass**, 28 failures. Default-precision replays of all
28 exact scenes show byte-identical native PNGs for 25 and identical bounds
for all 28. The three nonidentical PNGs are CJK None at each width; all three
also fail in the retained default-precision 90-case vector baseline. Thus no
new failing case has been attributed to the CSS precision option. This does not
qualify the 28 failures or establish improvement of every changed vector pixel.

Consolidated replay evidence:
`css-precision-full-vector/default-precision-comparison.json`; full artifacts
and gallery in the same directory. The final nine replays are in
`css-precision-vector-baseline-final/`, log
`/tmp/html-vector-precision-baseline-final.log`. The run used the old scale bound;
post-fix validation is tracked separately. Full vector visual review beyond
the already reviewed nine CJK changes remains incomplete.

## Post-bound-fix verification

- Scale-conversion unit test passes (`/tmp/html-precision-bound-unit.log`).
- Feature-enabled compiler/module suite: **89 tests pass**, including Chrome
  advance agreement, font policy retention/reversal and native glyph adapters
  (`/tmp/html-precision-bound-module.log`).
- Wide-advance regression now covers both two-em and maximum-u16 hmtx widths at
  all four sizes: **8 samples pass**, with valid SFNT checksums and no negative
  advance. `precision-wide-advance-fixed.json` contains the expanded results.
- Four-font size diagnostic remains **36 samples**, zero negative advances or
  missing glyphs (`/tmp/html-precision-sizes-fixed.log`).
- Rebuilt native-glyph probe passes **7/7** phase cases; all seven native PNGs are
  byte-identical to the previously visually reviewed explicit-policy output.
  Artifacts: `underline-precision-bound-phase/`; logs:
  `/tmp/html-precision-bound-probe-build.log`, `/tmp/html-precision-bound-phase.log`.

The full native corpus's recorded glyph sizes top out at 24px. The bundled
Japanese/Ogham fonts have UPM 1000 and Inter/OpenSans UPM 2048, so the new bound
starts at 250px or 512px respectively. The bound therefore does not change
shaping scale for those recorded corpus glyphs. This numeric audit plus targeted
post-fix tests avoids claiming a redundant full rerun. Compiler capability
emission/host installation remains the next implementation step; A20 and
existing vector/group-opacity limitations remain open.
