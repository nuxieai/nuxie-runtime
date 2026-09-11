# CSS normal word wrapping

Status: implemented; focused native validation passes. Full native regression is
running and has exposed preserved Unicode-space regressions. Vector glyph rasterization has nine retained local pixel failures.

Normal whitespace continues to be collapsed by the compiler. Soft wrapping now
uses the opt-in CSS line breaker: an overlong unbreakable word overflows instead
of being split glyph by glyph. Ordinary spaces permit breaks; nonbreaking spaces
remain unbroken. Existing pre/pre-wrap/pre-line/nowrap policies remain separate.
This does not add word-break, overflow-wrap, hyphenation, or writing-mode support.

Nonempty normal text requires `text-css-normal-wrap-v1` and an object-targeted
`css-normal-wrap-v1` policy in requirements version 2 or later (through current
version 5). Hosts must validate capabilities and Text targets before drawing and
install `Text::set_css_normal_wrap(true)` on each mapped occurrence. The policy
survives ordinary layout/reshape operations. The Rive bytes alone do not install
it. Raw Rive text without a policy retains its historical behavior.

The minimal red reproducer and investigation remain in reverse-flex-review.md
and the reverse-flex-minimal output. The new public test first failed because
normal text did not advertise the policy. Tests now cover missing capability,
wrong target, absent policy, duplicated policy and incompatible version. The host
integration compiles the actual scene: policy gives height24; an explicitly raw
legacy manifest on the same Rive bytes gives height48. A host missing the required
capability rejects before writing a render stream.

Validation: module199/199; JavaScript9/9 including native/WASM bytes, source maps
and requirements parity; host3/3; native and WASM builds and TypeScript pass.
Initial failures from stale manifest expectations were retained and corrected,
including an ellipsis test that inadvertently downgraded a version5 manifest.
Logs: /tmp/normal-wrap-{red,module,module-2,build,wasm,js,types,types-2,host,host-2,host-3}.log.

Focused permanent corpus: 20 reverse-flex fixtures plus three word fixtures,
compiled at390 then resized to240/390/768. Native69/69 passes. Vector60/69 passes:
all reverse-flex comparisons pass, all nine new word comparisons pass geometry
but fail local interior RGB6. Overlong-word error8.4832, fitting-word6.6552;
sequence text regions7.2623 and6.4216. No tolerances changed.

Both lanes have 58 source/browser/render-identical pairs transferred from reviewed
baselines and 11 changed/new pairs visually inspected. Whole words, space breaks,
NBSP overflow, backgrounds and reverse placement match Chromium. Vector glyph
edges differ visibly; those failures remain unqualified. Artifacts and durable
visual-inspection.json receipts: output/playwright/html-to-riv/normal-wrap and
normal-wrap-vector. Focused logs: /tmp/normal-wrap-pixels.log and
/tmp/normal-wrap-vector.log.

Full native regression is tracked in /tmp/normal-wrap-full.log and
output/playwright/html-to-riv/normal-wrap-full. Its outcome and changed-image
review are pending; focused success is not a full-regression claim.

## Full-run regression found while the run remains active

At least fifteen comparisons fail in the existing space-break corpus, including
space-break-center/right at240, trailing-center at all widths, mixed at240, and
three overflow families at all widths. This is a real regression, not a new
raster tolerance issue. The trailing-center390 browser/native PNGs were inspected:
normal text is shifted right by half the trailing U+2003 advance (12px at24px
font size). The new normal policy currently shares pre-line's Unicode separator
hanging/content-width rules. Chromium's normal mode retains that separator in
fitting and alignment, as documented in preserved-space-breaks-review.md.

Ranked hypotheses before source/image inspection: inappropriate normal-mode
space trimming; wrong alignment width; different break points. The minimal
single-line trailing-space case rules out break-point selection as the explanation
for that case. Source identifies a policy-specific semantic conflation: all
SpaceSeparator glyphs except Ogham are currently excluded from content_width.
Next correction must distinguish normal collapsed ASCII whitespace from retained
Unicode spaces for both fitting and alignment, while preserving pre-line and
pre-wrap behavior. Existing overflow cases prevent an alignment-only workaround.

No runtime edits were made during the running regression. Preserve the complete
normal-wrap-full failure set before building and testing the correction. L01 and
the normal-wrap policy remain partial.

Additional captured bounds before correction: space-break-overflow-center at240
has native a.height72/root.height88 versus Chromium a.height108/root.height124.
The trailing-center case has matching box geometry, isolating its glyph alignment
error. Together these require preserved-space advance in both line fitting and
alignment; a post-layout offset cannot fix both. Full-run size is1,582 tests.

Correction staged in source while the original binary continues the full run:
Normal excludes only ASCII space/tab from content width, retaining Unicode
separator advances. PreLine/PreWrap retain their previous rules. Three synthetic
fixed-advance runtime regressions cover trailing-space alignment, a fitting case
that changes line count, and overlong words. Build and tests are pending until
the original full regression terminates; its binary has not been replaced.

## Completed original full run

Original probe run completed:1,550/1,582 pass,32 fail. All15 identified
preserved-space failures remain, plus17 decoration failures: underline-edge-
transparent240; underline-cjk-none/auto/all240; strikethrough-combined-lines,
origin-auto/from-font/10percent/1em and opensans-from-font at240/390; and
strikethrough-fractional-composition240. Final log /tmp/normal-wrap-full.log.
Comparison against layout-contract-full, function-type-custom and normal-wrap
finds1,508/1,572 scene pairs source/image identical and64 needing inspection.
Comparison is saved, but full visual inspection is not complete.

The strikethrough-origin-auto240 browser/native PNGs were inspected: native
decoration extends beyond each soft line's final word by the trailing ASCII-space
advance. Hypothesis: the CSS line breaker excludes that space from content width
but leaves it in GlyphLine's paint range; the raw line breaker previously trimmed
it. Inspect and correct normal-mode line ranges without trimming retained Unicode
spaces. Space fitting and decoration ranges are separate regressions.

The first synthetic runtime test run failed because GlyphRun had no font and
line-spacing code requires one. The fixture now decodes the browser suite's pinned
Inter only for line metrics, retaining synthetic horizontal advances; rerun log
/tmp/normal-wrap-space-unit-2.log is pending. Original failure log retained.

Runtime synthetic rerun2: alignment and overlong-word tests pass; the fitting
fixture's expected pre-line line count was incorrect because both modes overflowed
its first candidate. It now uses `a b\u{2003}ccc` with10px glyph advances and35px
width, distinguishing preserved-space fitting (normal3 lines, pre-line2). Third
run is active, log /tmp/normal-wrap-space-unit-3.log. The original full-run sheets
have finished generating. The probe has not yet been rebuilt with the correction.

Decoration source diagnosis confirms the glyph-range hypothesis: underline
stripes iterate the OrderedLine glyph range and sum advances. Normal lines now
use the cached content endpoint for the emitted GlyphLine, so collapsed trailing
ASCII spaces do not extend paint or decorations, while retained Unicode spaces
remain. The fixed-advance fitting test also asserts that the first line excludes
its trailing ASCII space and the next retains its U+2003. Other whitespace modes
keep their existing glyph ranges. Correction validation remains pending.

Synthetic run3 passed3/3. Because the decoration endpoint edit landed during
that compilation, a final clean-source repeat is running in
/tmp/normal-wrap-space-unit-final.log before qualification. Native CLI/probe
rebuild is queued behind that Cargo lock (/tmp/normal-wrap-corrected-build.log).
Next browser lane must include space-break, normal-wrap, reverse directions,
pre/pre-line/pre-wrap controls, underline, strikethrough and custom-decoration;
then compare changed images and rerun the full regression. No corrected-browser
result is claimed yet.

Final settled-source runtime tests pass3/3 (/tmp/normal-wrap-space-unit-final.log),
including the glyph endpoint assertions. Native build remains active. Original
full-run visual-inspection.json now records1,508 transfers, three directly
inspected changed pairs and61 outstanding pairs. The CJK underline240 image
likewise shows a trailing-space line extension with matching line breaks.

The native rebuild failed with os error28 while writing the runtime rlib:
filesystem had279MiB free. The existing target/debug/incremental cache occupied
13GiB and is being removed as reproducible build data. Test-result, gallery and
failure artifacts are preserved. Retry build only after this cleanup completes;
original failed build log remains /tmp/normal-wrap-corrected-build.log.

Cache cleanup completed, leaving9.3GiB available. Native rebuild retry succeeds
(/tmp/normal-wrap-corrected-build-2.log). Checked host3/3 passes with the rebuilt
probe (/tmp/normal-wrap-corrected-host.log). The157-scene/471-comparison corrected
native lane is running in normal-wrap-corrected with log
/tmp/normal-wrap-corrected-pixels.log; module regression is also running.
The corrected space-break-trailing-center390 PNG pair was directly inspected: the
12px alignment discrepancy is gone. No full/focused result claimed until terminal.

Corrected normal-wrap native coverage passes471/471: preserved Unicode spaces
retain fitting/alignment width, and collapsed trailing ASCII spaces no longer
extend decoration glyph ranges. All471 source/browser/native image pairs are
identical to reviewed baselines (layout-contract-full, function-type-custom and
normal-wrap); durable visual transfer is complete. Runtime3/3, module199/199,
host3/3 and JS9/9 native/WASM parity pass. Native rebuild succeeded after clearing
only reproducible incremental build cache. Vector471 coverage is running; full
native rerun remains pending. Earlier32 failures are preserved, not overwritten.
Evidence: validation/normal-wrap-review.md and normal-wrap-corrected artifacts.

Terminal evidence: /tmp/normal-wrap-corrected-pixels.log (471pass),
/tmp/normal-wrap-corrected-module.log (199pass),
/tmp/normal-wrap-corrected-host.log (3pass), /tmp/normal-wrap-corrected-js.log
(9pass). Review transfer: normal-wrap-corrected/baseline-comparison.json and
visual-inspection.json. Vector log: /tmp/normal-wrap-corrected-vector.log.

Corrected vector run completes432/471 with39 retained pixel failures. All471
geometry comparisons pass; maximum absolute visible-box error0.03375244140625
CSS px. Source/image comparison transfers303 pairs from reviewed native/vector
baselines. Eighteen further pairs across six sheets have been inspected (soft
tab breaks, mixed whitespace, Open Sans underline/strike/undecorated controls);
150 remain pending. Wrapping, tabs, decoration widths and clipping agree in those
inspections; glyph rasterization differences remain visible. A legacy preline
review.json lacks the modern cases schema, so it was excluded from automated
transfer instead of silently accepting it. Failure/inspection evidence remains
in normal-wrap-corrected-vector.

Full corrected native run is now active: /tmp/normal-wrap-corrected-full.log,
output/playwright/html-to-riv/normal-wrap-corrected-full and
tools/html-to-riv/test-results-normal-wrap-corrected-full. It has not completed.

Corrected vector visual review complete: all471 pairs accounted for by303
source/browser/render-identical transfers and168 direct inspections (56 sheets
at three widths). Whitespace, tab stops, overlong-word overflow, ligature-word
wrapping, decoration inheritance/reset/color/width and placement agree in the
reviewed scenes. Visible glyph raster differences remain;39 pixel failures are
not waived. Geometry471/471 passes (max0.034px), pixels432/471. Durable evidence:
normal-wrap-corrected-vector/visual-inspection.json and baseline-comparison.json.
Full corrected native regression remains running.

## Final corrected native qualification

Full native regression passes1,582/1,582 (1,572 scene comparisons plus10 controls),
/tmp/normal-wrap-corrected-full.log. All1,572 source/browser/native image pairs
are identical to reviewed baselines: layout-contract-full, function-type-custom
and normal-wrap-corrected. Transfer evidence and completed visual-inspection.json
are in normal-wrap-corrected-full. Both original normal-wrap regressions are
resolved without tolerance changes. All processes for this increment are terminal.

L01 reverse flex directions is qualified for its documented profile:60/60 reverse
comparisons pass in both glyph renderer modes, source identity tests pass,
native/WASM parity passes, and the full native regression is green and reviewed.
The normal-wrap policy has native qualification; its nine focused vector pixel
failures and the broader corrected vector lane's39 pixel failures remain open
renderer limitations. Vector geometry471/471 passes and all471 pairs are reviewed.
Do not interpret L01 qualification as a complete vector-text qualification or as
completion of the compiler backlog. Next independent item L02 remains unsupported;
its source/runtime investigation is in order-investigation.md.
