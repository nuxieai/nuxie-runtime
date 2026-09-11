# A20 device-pixel-ratio controls

Chrome 153.0.8010.12 is authoritative. This control covers native glyph/Metal
rendering of three skip-ink variants (auto/all/none), at DPR 1/2/3, resized to
240/390/768 CSS pixels. Each variant is compiled once at 390px; its identical
Rive bytes are reused for all nine size/DPR combinations. Native and WASM bytes,
source maps and runtime requirements agree for all three inputs.

The probe's diagnostic NUXIE_DEVICE_SCALE sets the physical framebuffer size
and the host drawing transform. Runtime layout and reported geometry remain in
CSS pixels. Chrome uses deviceScaleFactor. Pixel regions are scaled into device
coordinates; pixel tolerances and red-decoration proximity remain unchanged.
This is a validation host control, not additional accepted CSS syntax.

## Results

25/27 cases pass. All 18 DPR 2/3 cases pass. All 27 geometry and ordinary pixel
checks pass. At DPR 1 and width 768, auto and all each fail the stronger red
support gate with two extra native red pixels (zero missing). Auto has 589
reference/590 native red pixels; all has 412/414. These failures are retained,
not waived; their exact coordinates are recorded in extra-pixels.json.

All nine browser/native/diff contact sheets were inspected. Wrapping, CJK
eligibility differences between auto/all, and uninterrupted none lines agree
visually at overview scale. Small glyph edge raster differences remain. Contact
sheets limit images to 768 display pixels; raw screenshots retain device-pixel
resolution. The inspection does not override the two failing sparse-ink checks.

Artifacts, exact requests/fonts, Rive files, manifests, native streams, bounds,
raw PNGs, diffs, contact sheets and results.json are in
output/playwright/html-to-riv/underline-dpr/. Logs:
/tmp/html-underline-dpr-build.log and /tmp/html-underline-dpr.log.

## Reproduction

After the standard WASM and renderer-replay builds:

```sh
cargo build -p nuxie-html-to-riv --features native-glyph-controls --bin html-to-riv --example probe
node tools/html-to-riv/validation/underline-dpr-control.mjs
```

The native-glyph validation lane includes the control. Current expected exit is
nonzero because both DPR 1 failures remain. No complete validation/run.sh pass
is claimed. This run checks the public native compiler and WASM for its three
sources; production compiler code did not change in this step.

## Remaining qualification

Diagnose the two extra-pixel cases using the saved native glyph positions and
streams. Fractional DPR, alternate fonts/thicknesses/offsets, composed host
transforms and other renderer backends remain unqualified. This does not close
A20's existing vector, affine or group-opacity limitations or performance work.


## Reduced half-pixel failure

`--focus` repeats auto/DPR1/768 alone in under a second and retains the same
red-support failure. `--transparent-glyphs` and `--latin-only` remove glyph paint
and the CJK paragraph respectively; both remain red. Removing the last p leaves
`agypqj gap agypqj gap agypqj ga`, still red. The diagnostic greedy reducer
(`python3 tools/html-to-riv/validation/reduce-underline-dpr.py`) tried 31 removals;
no single further character deletion retained this failure. This establishes
single-character deletion minimality, not global minimality.

The original extra pixels are (359,72) and (359,73). The isolated line moves the
same effect up by 40px. Native's final a origin is 351.93603515625. Chrome canvas
full-width-minus-suffix gives 351.9357604980469. The prefix measurement is not a
substitute: ga kerning makes it 352.3677978515625. Native's clip left in the
recorded stream is 351.50003, translated by 8px into the half-pixel boundary.

Three hypotheses were ranked: advance accumulation, hard-clip tie rounding,
and outline-intersection differences. Actual shaped advances summed in f64 give
351.93601989746094, still too large versus Chrome. Combining that f64 sum with
the actual normalized outline intersection yields device left 359.50002348423004;
the f32 accumulated origin gives 359.5000387430191. Therefore changing only
accumulation precision cannot move this native edge below 359.5. No epsilon,
rounding change, or threshold relaxation was applied. Per-glyph advance scaling/
positioning and exact outline differences need further isolation.

Reproducers: output/playwright/html-to-riv/underline-dpr-{focus,transparent,latin,trim}/.
The trim directory contains chrome-advances.json and advance-diagnosis.json.
The reducer records every trial and verdict under underline-dpr-reduction/.
The existing outline_probe supplies actual shaped advances/intercepts; its raw
output is /tmp/html-dpr-outline.json. No production rendering code changed in
this diagnosis. The failure is open.


## Advance callback discrepancy isolated

`node tools/html-to-riv/validation/underline-advance-components.mjs --check`
is now a failing scalar/pair/full-string shaping comparison against Chrome.
It exercises the actual HbFont via outline_probe (CSS precision selected), not
an independently reimplemented shaper. The script records 19 measurements.
At 24px, native p/q/j/space each exceed Chrome by exactly 1/65536px; a/g/y agree.
Tested pair deltas are precisely the sum of their constituent scalar deltas,
including gy and ga kerning. Across the reduced text, scalar differences account for 16 fixed-point units;
rounding the corrected total to f32 accounts for the remaining unit and exactly
reproduces Chrome. The measured total-width discrepancy is 17 units:
365.44801330566406 native versus 365.44775390625 Chrome. The JSON decomposition
records this arithmetic independently of the clipping test.

Current Chromium source routes horizontal advances through Skia widths and
converts to signed 16.16 with ClampTo<int>. The local harfrust 0.12.0 built-in
integer metric scaling instead adds 32768 before shifting by 16, rounding to
nearest. These are different advance quantization policies. Sources:
https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/platform/fonts/skia/skia_text_metrics.cc
https://raw.githubusercontent.com/chromium/chromium/main/third_party/blink/renderer/platform/fonts/shaping/harfbuzz_face.cc
These main-branch sources corroborate the mechanism; the measured reference is
still the pinned Chrome 153.0.8010.12, not an assumption that main equals it.

Next implementation seam: CSS-only harfrust FontFuncs advance callback. The
public callback interface requires already-scaled metrics, including any other
metric methods it replaces, so merely overriding width with unscaled default
extents would be incorrect. Preserve default Rive behavior, variation metrics,
GPOS adjustments and the existing overflow bound. Verify the component command,
the reduced clipping command, and the complete DPR matrix after implementation.
No production rounding fix is installed yet; this diagnostic intentionally exits
nonzero until actual native widths match. Results: advance-components.json in
the trim artifact directory. This does not alter existing visual tolerances.


## CSS advance callback implemented

HbFont's CSS precision policy now installs a harfrust FontFuncs callback. It
truncates scaled horizontal advances while leaving GPOS positioning to the
shaper. Other overridden metric methods reproduce the previous built-in scaling
policy; nominal/variation glyph mapping delegates to the built-in font. The
default Rive precision path does not install the callback. The existing scale
bound remains in force.

Validation so far:

- All 19 scalar/pair/full-string comparisons match Chrome after conversion of
  the total to f32, the same representation used by Chrome Canvas. Raw f64 sums
  remain in the diagnostic JSON; the exact comparison uses nativeFloatWidth.
  This corrects the diagnostic's comparison representation, not a tolerance.
- The reduced transparent repro passes 1/1 and the complete DPR matrix passes
  27/27, fixing both prior DPR1/768 failures with unchanged visual gates.
- 91 module tests pass, including the new pinned scalar/pair width regression
  and existing default-Rive/font-replacement coverage.
- Eight wide-advance overflow controls pass through 1,000,000px sizes.
- Native/WASM compiler artifacts match for all three DPR source scenes.
- The repaired auto/all DPR1 contact sheets were inspected (six cases); the
  remaining seven new DPR sheets still need fresh inspection. Seventeen native
  PNGs changed overall, so unchanged prior inspection is insufficient for those.

Artifacts: output/playwright/html-to-riv/underline-dpr-callback/ and
underline-dpr-trim-fixed/. Original failures remain in their original folders.
Logs: /tmp/html-css-callback-{build,tests,wide,dpr,trim}.log.
The fresh native underline run passes 90/90 in 33.3s, logged in
/tmp/html-css-callback-underline.log. Gallery: underline-callback-glyph/.
Changed-case comparison and fresh visual review for this run remain pending.
Broader font/variation/fallback-mark, vector, host-transform and corpus checks
remain necessary because this changes all CSS-shaped text. A20 remains active.


## Follow-up visual and parity verification

All nine post-callback DPR contact sheets have now been inspected, including
the remaining seven sheets at DPR2/3 and skip-ink:none. Wrapping, line placement,
skip-ink policy differences and restored red support match the Chrome reference
at the recorded overview scale. The raw device-resolution images remain saved.

The 90-case native underline run has zero added/removed/source-changed cases,
zero changed metrics and all 90 native PNGs byte-identical to the previously
reviewed automatic-precision baseline. See underline-callback-glyph/
baseline-comparison.json and pixel-identity.json. No new visual inspection is
claimed for these identical images; their prior reviewed pixels are unchanged.

The six JavaScript tests pass, including full accepted-corpus native/WASM
artifact parity; log /tmp/html-css-callback-js.log. The broader native-glyph
corpus is running with separate test-results-css-callback-full-glyph output and
css-callback-full-glyph gallery directory; log /tmp/html-css-callback-full-glyph.log.
Host-state controls are also running in underline-callback-state/ with log
/tmp/html-css-callback-state.log. Neither unfinished run is counted as passing.

Host-state follow-up completed at 18/27, with the same nine failing cases as
the previous precision run and no newly failing cases. Failure identities are
compared in underline-callback-state/baseline-comparison.json; these limitations
remain open and are not waived by the passing DPR matrix.

The host-state PNG comparison found 26/27 byte-identical native images. The one
changed image, nonuniform-scale-768, was inspected against Chrome at native
resolution: transformed and restored-copy text, underline placement and ink
exclusions agree within the unchanged gates. Existing failing images are
byte-identical to their reviewed baseline. Pixel evidence is recorded in
underline-callback-state/pixel-identity.json.

The runtime boundary passes (28 protected packages, 57 dependency tables),
logged in /tmp/html-css-callback-boundary.log. The full corpus process remains
live; no final full-corpus outcome is claimed here.


## Expanded font-size and vector checks

The vector underline run remains 85/90 with precisely the same five failures
(two from-font at 390 and CJK-none at all three widths). No added/removed/source
changes or new failures. Six CJK metrics changed; their browser/native/diff
contact sheet was inspected. The five failures remain limitations. Evidence:
underline-callback-vector/baseline-comparison.json and changed-0.png; log
/tmp/html-css-callback-vector.log. This is focused vector coverage, not a full
post-callback vector corpus pass.

`advance-font-matrix.py` expands exact Chrome/native width comparison to three
fonts and 16/24/36px: 165/171 measurements pass. All 57 Inter and 57 Open Sans
measurements pass. Japanese 16/24px pass, but Japanese 36px has six failing
measurements, all involving g. At 36px, g measures 20.304000854492188 in Chrome
versus 20.303985595703125 native (one fixed-point unit). This is an open callback
scaling issue, not a pixel qualification result. The current callback scales
before multiplying the font-unit advance; f32 order matters. Diagnostic arithmetic
for g (564 font units, UPM 1000) yields 20.303985595703125 for scale-first,
but 20.304000854492188 when the pixel width is computed before fixed-point
conversion. Multiplication-first and normalized-first both match this one sample;
more evidence is required to choose the implementation arithmetic.

Artifacts: output/playwright/html-to-riv/advance-callback-fonts/summary.json and
all nine detailed component JSON files. Reproduce with:
`python3 tools/html-to-riv/validation/advance-font-matrix.py` (currently nonzero).
The ongoing full-corpus run uses the unchanged callback binary so its evidence
will remain attributable; do not rebuild its probe until that run is terminal.

Module Clippy completes with dependency warnings and one unnecessary unwrap in
the earlier underline test. The test was rewritten using a match; this small
cleanup has not yet been rerun. Log: /tmp/html-css-callback-clippy.log.


## Full-corpus result and pixel-width arithmetic follow-up

The first callback implementation's full native-glyph run completed in 8.2m:
1014/1015 runner checks and 1004/1005 visual cases pass. The only failure remains
em-layout-cascade at 240px. Compared with css-precision-full-glyph there are zero
added, removed, source-changed, metric-changed or newly failing cases. All 1005
native PNGs are byte-identical to that saved baseline (pixel-identity.json).
This run is terminal; it does not validate later pixel-width arithmetic edits.

Expanded diagnostic arithmetic covers 252 single-glyph Chrome measurements
across three fonts and twelve sizes (including decimal sizes). Scale-first has
three failures; computing the f32 pixel width before fixed-point conversion
matches all measurements with either multiply-first or double-intermediate
arithmetic. Normalizing first has one failure. Details are in arithmetic.json
under advance-callback-fonts/, advance-callback-fractions/ and
advance-callback-decimals/. These are arithmetic diagnostics, not substitute
native-rendering tests.

The runtime now computes pixel width with a double intermediate then rounds it
to f32 before fixed-point conversion. The native font matrix improves 165/171
to 170/171. The remaining failure is the long string at Japanese 36px:
Chrome 548.1718139648438, native float total 548.171875. All scalar and pair
checks pass; total accumulation/representation needs further diagnosis.
Artifacts: advance-pixel-width-fixed/. The DPR matrix remains 27/27 after this
edit, in underline-pixel-width-dpr/; its new images are not yet reviewed.

A further robustness probe at font-size 1e-40 found an overflowing f32 reciprocal
producing advance 2.14747207330253e-31. Conversion now performs the ratio in f64
and retains the prior fallback for nonpositive/nonfinite sizes. A native
subnormal-size regression was added alongside the Chrome 36px g regression.
The safe conversion build is running (/tmp/html-pixel-width-safe-build.log);
module, tiny-size, matrix and pixel revalidation remain pending for that final
source revision. No complete qualification is claimed.


## Safe pixel conversion validated; accumulation remains open

The final safe conversion build completed. All 91 module tests pass, including
36px g and the native subnormal-size regression. The 1e-40 probe now emits zero
advance instead of the overflowing result. Eight wide-advance controls pass.
Logs: /tmp/html-pixel-safe-tests.log, /tmp/html-pixel-safe-wide.log; the direct
tiny probe is /tmp/html-pixel-safe-tiny.json.

The three-font matrix remains 170/171. DPR comparisons pass 27/27 with native/
WASM artifact parity for all three sources. Artifacts: advance-pixel-safe/ and
underline-pixel-safe-dpr/. Native image identity versus the reviewed callback
DPR output is recorded in underline-pixel-safe-dpr/pixel-identity.json.

The long-string failure is further isolated by grouped-widths.json and
grouped-arithmetic.json in advance-pixel-width-fixed/. At Japanese 36px, Chrome
measures the prefix ending in the third agypqj as 500.183837890625. Adding the
measured space width (8.063995361328125) in f32 produces 508.2478332519531,
exactly Chrome's prefix-with-space width. Adding ga (39.92399597167969) then
rounding to f32 yields 548.1718139648438, exactly Chrome's full-string width.
The native sum-once result is 548.171875. These observations support an
intermediate accumulation boundary; they do not establish a general word-based
shaping policy. The exact diagnostic failure remains retained. No tolerance
change or ad hoc space-rounding rule was introduced.

A20 is partial, not fully qualified. Remaining scope includes this accumulation
case, fractional/general DPR, backend and performance qualification, the five
focused vector failures, and the nine existing host-state failures. The native
1005-case corpus receipt above predates pixel-width arithmetic and is not a
claim of a final-source rerun. Continue independent A21 strikethrough work while
retaining these shared text/runtime follow-ups; none is marked completed.
