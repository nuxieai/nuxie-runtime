# A22 text clipping — initial implementation

Accepted: single-keyword `overflow: visible` (default) and `overflow: clip`,
plus explicit inherit/initial/unset. Overflow is not inherited by default.
Clip maps to the existing LayoutComponent clip flag and follows its live layout
bounds and rounded path. Text is shaped and wrapped normally; no substring or
browser-computed bounds are baked into the scene. Sibling drawing resumes after
the runtime restores the clip. Existing radius and padding syntax remains.

Not yet accepted: hidden/auto/scroll, two-value overflow, overflow-x/y,
overflow-clip-margin, ellipsis. These are outstanding work, not removed from
the backlog. `hidden` requires separate scroll-container/layout semantics.
Spec reference: [CSS Overflow 3](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-clip).
A22 is active; clipping nested descendants, dense edge glyphs, decorations,
images, fractional/DPR/host transforms, and vector profile need further checks.

Public tests in `tests/clipping.rs` import actual compiler output, check explicit
inherit/reset plus noninheritance, and resize the same scene to 240/390/768.
Unsupported forms are rejected. Initial two tests pass; the final noninheritance
assertion is also included in the module regression run recorded below.
Native and WASM builds pass (`/tmp/html-clip-build.log`, `/tmp/html-clip-wasm.log`).
The expanded accepted-scene corpus passes exact native/WASM bytes, source maps
and runtime requirements (`/tmp/html-clip-parity.log`, exit 0).

Four fixtures cover nowrap horizontal overflow, fixed-height vertical overflow,
padding, and rounded clipping. Each includes an unclipped sibling to expose
state leakage. Native-glyph geometry and pixels pass 12/12 at 240/390/768;
all four three-width sheets were visually inspected. Shapes and sibling
placement align; ordinary glyph-edge differences remain within unchanged limits.
Command: native Playwright project, `NUXIE_NATIVE_GLYPHS=1`, `--grep text-clip`,
with `NUXIE_HTML_REVIEW_DIR` pointing to the artifact directory below.
Log `/tmp/html-clip-pixels.log`, exit 0.
Artifacts: `output/playwright/html-to-riv/text-clip-initial/` and
`tools/html-to-riv/test-results-text-clip-initial/`. Generate contact sheets with
`validation/composition-review.mjs` using those input/output directories.
This first matrix is evidence for these cases, not full A22 qualification.

Full module regression passes 106 tests, including final noninheritance coverage
(`/tmp/html-clip-module.log`, exit 0). Vector run passes 6/12; retained failures
are horizontal/padded/rounded at 240px and vertical at all widths. All 12 vector
comparisons were visually inspected. Glyph raster differences are visible in
both clipped text and the unclipped sibling; exact root-cause isolation remains.
Artifacts `output/playwright/html-to-riv/text-clip-vector/` and matching module
`test-results-text-clip-vector/`; `/tmp/html-clip-vector.log` exits 1. Reproduce
with the same command and `NUXIE_NATIVE_GLYPHS=0`. No tolerances changed.


## Nested and edge compositions

Six added corpus fixtures cut through glyph bodies, put dense M glyphs against
rounded corners, clip combined underline/strikethrough, nest two rounded clips,
use fractional padding/height/percentage width, and crop an image inside a
rounded media card. Every fixture includes a following green text sibling to
check that the clip does not leak. The image card also places text below its
oversized image, outside the clipped area. Each source is compiled at 390 and
the same scene is resized to 240/390/768 by the established harness.

Native-glyph 18/18 passes; vector 10/18 passes. All 36 comparisons were visually
inspected in the twelve three-width contact sheets. Vector failures are local
RGB limits: rounded-edge at all widths, decorations at 240/390, fractional at
all widths (fractional 240 also fails the sibling region). Geometry passes.
Dense vector glyph differences remain visible; no exact cause has been isolated
for this new matrix, so the failures remain qualification gaps. Clipped glyph
bodies, paths and following sibling placement align in the reviewed scenes.

Run the native Playwright project with
`--grep 'text-clip-(glyph-cut|rounded-edge|decorations|nested|fractional|image-card)'`,
`NUXIE_NATIVE_GLYPHS=1` or `0`, and separate `NUXIE_HTML_REVIEW_DIR` directories.
Artifacts: `output/playwright/html-to-riv/text-clip-compositions/` and
`text-clip-compositions-vector/`, plus matching module `test-results-*` folders.
Logs `/tmp/html-clip-compositions.log` exits 0 and
`/tmp/html-clip-compositions-vector.log` exits 1. Contact sheets were generated
with `validation/composition-review.mjs` from each test-results directory.
Expanded corpus native/WASM scene, source-map and requirement parity passes
(`/tmp/html-clip-compositions-parity.log`, exit 0).

Cumulative A22 fixture evidence: native-glyph 30/30, vector 16/30; all 60
comparisons visually inspected. This increment changes corpus and receipts
only, with no production changes or altered tolerances. DPR/host transforms,
hidden semantics, axis-specific overflow and clip-margin remain outstanding.


## Device scales and a missing-clip negative control

`node tools/html-to-riv/validation/clipping-dpr-control.mjs` compiles the
fractional, rounded-edge and decorated fixtures once each at 390px, then resizes
the same scenes to 240/390/768 at DPR 1/2/3. Native/WASM bytes, source maps and
requirements must match. CSS geometry uses the unchanged 0.1px threshold;
physical image dimensions must equal CSS dimensions times DPR. Chrome
153.0.8010.12 is the reference; the renderer uses native glyph controls.

23/27 pass all checks. Fractional DPR2 fails at all widths (local text RGB;
240/390 also mean-channel, 240 also sibling RGB); fractional DPR3 at 240 fails
sibling RGB. All geometry and red decoration checks pass. All 27 comparisons
were visually inspected. Glyph-edge differences remain visible, including in
the sibling that is not clipped; this does not yet isolate their exact cause.
No clip or text pixel failure was waived.

A targeted leakage gate checks the white gap below the clip and above the next
sibling, excluding a one-device-pixel edge neighborhood. Every reference gap
must be white, and any native pixel darker than RGB249 fails. All 27 ordinary
cases have zero leaked pixels. This checks bottom leakage; it does not prove
all edges or rounded corner accuracy independently of the ordinary pixel gate.

`--negative-control` deliberately compiles overflow:visible in native/WASM for
the fractional fixture while keeping Chrome clipped. It requires a separate
`NUXIE_HTML_REVIEW_DIR`. All nine cases fail the leakage gate, with 422–11595
leaked pixels; the three DPR2 comparisons were visually inspected and show the
missing clip clearly. Native/WASM parity also holds for the intentionally
unclipped source. This verifies that the new gate detects removed clipping;
it is not a production regression. The negative-control command exits 1.

Artifacts: `output/playwright/html-to-riv/clipping-dpr/` and
`clipping-dpr-negative/`, each retaining sources, scenes, manifests, streams,
bounds, PNGs, contact sheets and results.json. Logs `/tmp/html-clip-dpr.log`
and `/tmp/html-clip-dpr-negative.log`, both exit 1 for the documented failures.
The JSON's `thickness` field contains fixture names (inherited script naming).
No production source or existing tolerance changed in this increment.


## Fractional DPR failure isolation

The DPR runner now exposes separate diagnostic controls, requiring dedicated
`NUXIE_HTML_REVIEW_DIR` paths. Each restricts the matrix to fractional clipping
at three widths and three scales and still asserts native/WASM artifact parity.

* `--unclipped`: overflow:visible in both Chrome and native, preserving source
  dimensions and text. Passes 4/9. All DPR2 widths and DPR3 240/390 fail pixels.
  The white-gap leakage gate is inapplicable and omitted because ink is
  intentionally allowed to overflow. All ordinary pixel and geometry checks
  remain. DPR2 at all widths was visually inspected; text differences persist
  without clipping. Artifacts `clipping-dpr-unclipped/`, log
  `/tmp/html-clip-dpr-unclipped.log` (exit 1).
* `--transparent-clipped-text`: only #clip becomes transparent, retaining the
  green sibling. The first equivalent run used p{color:transparent}, which was
  overridden by #after's existing color, so its actual scope is clipped text
  only. Passes 7/9; DPR2/3 240 fail only sibling local RGB. Its DPR2 sheet was
  visually inspected. Artifacts `clipping-dpr-transparent/`, log
  `/tmp/html-clip-dpr-transparent.log` (exit 1). The named switch now expresses
  this scope explicitly rather than relying on selector specificity.
* `--transparent-glyphs`: explicit #clip,#after color:transparent. Passes 9/9
  geometry, pixels and leakage. DPR2 all-width sheet visually inspected.
  Artifacts `clipping-dpr-all-transparent/`, log
  `/tmp/html-clip-dpr-all-transparent.log` (exit 0).

All directories are under `output/playwright/html-to-riv/`. The ordinary
clipped-text matrix remains 23/27. These controls show an independent text
rendering residual: clipping is not necessary for the failure, and the sibling
can fail independently. The transparent case tests box rendering, not clipped
glyph coverage; it does not waive the visible-text failures. Remaining scales
of these diagnostic runs have automated evidence only. No production or
threshold changes were made.


## Static overflow:hidden implementation

The authoring reset sets min-width/min-height to zero and defaults containers
to flex; supported block mode contains text only. Scrolling, scripts, focus
interactions, floats and general block formatting remain outside this profile.
`overflow:hidden` now emits the same responsive clip as `clip` for static
initial rendering. This is not a general scroll-container implementation.
The CSS distinction is retained in the documented support contract:
[CSS Overflow 3](https://www.w3.org/TR/css-overflow-3/#valdef-overflow-hidden)
requires programmatic scrolling for hidden in a browser; the compiler has no
scrolling API. Auto/scroll and axis-specific overflow remain rejected.

Before acceptance, `clipping-dpr-control.mjs --reference-hidden` compared native
clip with Chrome hidden across the existing 27 DPR fixtures. All 27 Chrome PNGs
are byte-identical to the earlier clip reference; native PNGs are also identical.
The same four text failures remain (23/27), so earlier direct visual inspection
covers these identical images. Artifacts `clipping-hidden-reference/` under
`output/playwright/html-to-riv/`, log `/tmp/html-clip-hidden-reference.log` exit 1.

`node tools/html-to-riv/validation/overflow-hidden-reference.mjs` also compares
Chrome clip/hidden for oversized flex children with row/column direction,
start/center/end alignment and 240/390/768 widths. All 18 pairs have identical
pixels, geometry and initial scroll offsets. These are browser differential
controls, not native qualification; their images have not been manually
reviewed. Requests and screenshots are retained in `overflow-hidden-alignment/`;
log `/tmp/html-hidden-alignment.log`, exit 0.

Ten hidden counterparts of the clipping corpus are added to exercise the actual
accepted CSS through public native/WASM compilation and runtime resize paths.
A public test checks that static hidden and clip compile to identical scenes.
Build and runtime qualification results for this addition follow below.


Static hidden validation: native and WASM builds pass; module107 passes
(`/tmp/html-hidden-build.log`, `/tmp/html-hidden-wasm.log`,
`/tmp/html-hidden-module.log`, exit 0). Expanded corpus exact native/WASM
artifacts pass (`/tmp/html-hidden-parity.log`, exit 0).
Ten actual hidden fixtures pass glyph30/30 and vector16/30 at all three widths;
logs `/tmp/html-hidden-glyph.log` (exit 0) and `/tmp/html-hidden-vector.log`
(exit 1 for the same 14 vector failures). No new pixel failure appeared.

All 60 browser/native PNGs in each profile are byte-identical to their earlier
clip equivalents, which were all directly visually inspected. Per-image hashes
and identity receipts are stored in `clip-identity.json` in each new review
directory, establishing the visual-review link rather than assuming equivalent
screenshots. Artifacts: `output/playwright/html-to-riv/text-hidden-glyph/` and
`text-hidden-vector/`, plus matching module `test-results-*`. Reproduce using
the native Playwright project, `--grep text-hidden`, and the respective
`NUXIE_NATIVE_GLYPHS` 1/0 profile with separate review/output directories.

A22 remains active for host transforms, axis-specific overflow, clip margin and
pixel residuals. Earlier statements that hidden is rejected describe the prior
implementation; static hidden is now accepted under the scope above.


## Compiled clipping under host renderer state

`node tools/html-to-riv/validation/glyph-state-control.mjs clipping` now covers
a fractional clipped text box, rounded hidden text box and unclipped green
sibling. Nine host states (identity, fractional translation, host clip, half
opacity, uniform/nonuniform scale, rotation, shear and combined clip/rotation/
opacity) run at 240/390/768. The source is compiled once; native/WASM scene,
map and requirements are asserted identical before resizing. DPR is 1.
These are host transforms, not newly accepted CSS transform syntax.

Default save/restore-copy run: 15/27. Fractional translation, uniform scale,
rotation and shear fail pixels at all widths. Isolated `--no-restore` run:
19/27; failures remain for those four states at 240, translation/uniform scale/
rotation at 390, and translation at 768. All geometry checks pass. Every
isolated comparison was visually inspected across nine three-case sheets;
the default restored-copy run has automated evidence only in this increment.
Text raster differences appear within clipped text and the unclipped sibling.
Exact cause isolation remains; aggregate passes here do not supersede the
known P05 group-opacity defect or constitute full transformed-clip qualification.

Artifacts `output/playwright/html-to-riv/clipping-state-controls/` and
`clipping-state-controls-isolated/` retain input, compiled output, per-state
matrices, streams, bounds, images, reviews and results.json. Logs
`/tmp/html-clipping-host.log` and `/tmp/html-clipping-host-isolated.log`, both
exit 1. The existing glyph profile passes 27/27 after script extension:
`/tmp/html-glyph-clipping-regression.log`, exit 0, artifacts
`glyph-state-clipping-regression/` (explicit NUXIE_HTML_REVIEW_DIR override).
No compiler/runtime production source or tolerance changed in this increment.

Short-height regression discovered in A23 controls: a height:8px, line-height:40px
Inter block measures 8.7758255px natively versus 8px in Chrome. Disabling
ellipsis reproduces it; all pixels are blank so geometry is the detecting
gate. Vertical clip controls pass 9/12, all visually inspected, with height
20/39/80 passing. See ellipsis-review.md “Alignment and short-height controls”
and `ellipsis-vertical-clip-control/`. Compiler-added line-height padding is
the suspected cause; no fix yet.

## Internal line-box container (validated)

The compiler now keeps authored CSS padding on the source element and emits
an internal LayoutComponent containing only the vertical line-height spacing
and Text child. This removes the artificial minimum height caused by adding
leading to the authored element's padding while preserving intrinsic line-box
measurement. Baseline transform remains on Text. Internal object IDs change;
the existing nowrap requirements snapshot was updated from Text ID6 to ID8.

Native build and WASM build pass (`/tmp/html-line-box-build.log`,
`/tmp/html-line-box-wasm.log`). The module suite passes 107 tests after updating
that expected structural ID (`/tmp/html-line-box-tests-final.log`); accepted
corpus native/WASM bytes/maps/requirements parity passes
(`/tmp/html-line-box-parity.log`). No runtime requirement version changed.

The prior 8px height reproducer now measures correctly. All 12 ellipsis
vertical controls pass and were visually inspected, including partially
clipped 20px text and 39/80px boxes. Artifacts `line-box-vertical/`,
`/tmp/html-line-box-vertical.log` exit0. The baseline source/scenes have exact
native/WASM parity; ellipsis remains selected by the probe-only flag.

A full 1105-check native run is still running as of this entry:
`/tmp/html-line-box-full.log`, output `tools/html-to-riv/test-results-line-box-full`,
review `output/playwright/html-to-riv/line-box-full/`. Do not claim broad
qualification until its final result and any image changes are reviewed.

The full run is complete: **1102/1105 checks pass**, with 1092/1095 scene
comparisons passing. The three failures are retained baseline failures:
em-layout-cascade, strikethrough-opensans-from-font, and
strikethrough-opensans-baseline, each at 240px. Every one of the 1095 native
PNGs and 1095 Chrome PNGs is byte-identical to its retained baseline.
`line-box-full/baseline-comparison.json` records per-case identity and the
source baseline receipts (full CSS-precision glyph corpus, strike compositions
and baseline, clip initial/compositions, hidden glyph). Thus no existing
corpus raster changed; this does not claim the known failures are fixed.
`/tmp/html-line-box-full.log` exits1 for those three failures. The focused
12 new height comparisons were directly visually inspected this increment;
existing corpus images are compared by identity, not re-inspected individually.

Added public compiler/import/layout regression
`text_leading_does_not_enlarge_authored_height_but_preserves_auto_height`: zero,
8px, 20px and auto-height Inter text, with each compiled scene resized to
240/390/768. It checks both text and parent geometry, including auto line-box
height40. All four clipping tests pass, `/tmp/html-line-box-regression.log`,
exit0. Combined with the prior full107 test pass, the module now has108 tests.
No changes followed to implementation after the full native run.

The authored short-height bug is fixed without changing CSS syntax or
runtime requirements. Internal line-box records add two objects per text
element. CSS ellipsis production transport remains pending, and existing
vector/DPR/host opacity failures remain separate limitations.
