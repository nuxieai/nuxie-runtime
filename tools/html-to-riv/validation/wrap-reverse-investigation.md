# L06 wrap-reverse

Status: independent browser references captured; runtime investigation in progress.
Compiler acceptance, parity and pixel qualification remain pending. L05 full
native regression is still active; its source and fixture matrix are unchanged.

## Intended behavior

Accept `flex-wrap: wrap-reverse`, alongside nowrap and wrap. It creates a
multi-line flex container and swaps cross-start/cross-end, including alignment
within a single line. It does not reverse source identity or main-axis ordering;
combine it independently with row/column reverse and order. CSS-wide values and
custom-property substitution must retain existing cascade semantics.
Reference: [CSS Flexbox 1, flex-wrap](https://drafts.csswg.org/css-flexbox-1/#flex-wrap-property).

This item does not add flex-flow, writing modes, scrolling, Grid, auto margins,
editor integration or scripted interactions. Supported item/line alignment,
dimensions, gaps and paint must continue to compose predictably.

## Browser references and current rejection

`validation/wrap-reverse-oracle.mjs` generated112 scenes/336 viewports using
Chromium153.0.8010.12. Four directions, seven line alignments, four modes:
mixed sizes with flex-start items, overflow with flex-end items, single item
with center alignment, and stretch items with explicit self overrides and an
auto cross-size child. Widths240/390/768. The generator asserts the actual mixed
cross sizes20/35/50 and fixed main sizes, preventing silent fixture regressions.

Durable oracle: tests/assets/wrap-reverse-boxes.json. Original output:
output/playwright/html-to-riv/wrap-reverse-oracle.json.
Log: /tmp/wrap-reverse-oracle.log, session80851 returned exit0.
Four public CLI rejection reproducers are preserved in
output/playwright/html-to-riv/wrap-reverse-initial: all four directions reject
with unsupported-value, Expected wrap or nowrap, exit1.

## Runtime path and validation

LayoutComponentStyle already decodes flexWrapValue2 to YGWrap::WrapReverse.
The compiler currently stores wrapping as bool. A typed wrap mode should keep
the old0/1 encoding and add2, preserve reset/inheritance, and still emit the
independent line-alignment policy for either wrapping mode. An additional host
field may be unnecessary if the existing Rive encoding proves correct.

`tests/wrap_reverse_runtime.rs` temporarily compiles wrap, restores authored
flexWrapValue2 in the imported style before layout, then installs emitted line
and self policies. This is a runtime investigation, not public acceptance:
remove the placeholder and style mutation when compiler support lands. It
resizes the same imported bytes against all336 independent browser references.
First run: /tmp/wrap-reverse-runtime-first.log, session51656.

Remaining gates: public parser/cascade/diagnostics tests; original-CSS geometry
test; native/WASM parity; host compatibility; mixed-size and text/paint/order
compositions; real native pixels, visual review, vector profile evidence and
full regression. No new value is qualified by this reference generation alone.

The first test build failed due to an incorrect direct setter/callback API
(/tmp/wrap-reverse-runtime-first.log). Corrected the test to use
CoreRegistry::set_uint_handle with the generated FLEX_WRAP_VALUE_PROPERTY_KEY.
This exercises the generated property path and requires no runtime source changes.

The corrected runtime test compiled and found a real geometry mismatch:
row/stretch/overflow at240px, child a y70 native versus5 Chromium.
Log /tmp/wrap-reverse-runtime-registry.log, session83560 exit101. The saved
oracle and failing runtime test preserve the reproducer. Investigate reversed
cross-axis fallback for negative space (especially stretch/space-between) before
compiler acceptance. No runtime fix or pixel qualification is claimed.
L05 full regression remains live at session21695; its binaries/fixtures are unchanged.

Expanded the runtime investigation to collect all failures rather than stop at
the first. /tmp/wrap-reverse-runtime-matrix.log (session1362 exit101) shows
96 coordinate mismatches across24 viewports: stretch/space-between overflow,
all four directions at all three widths. Other matrix cases match.

Root cause: generic apply_alignment_fallback changed stretch/space-between to
logical Start under negative space. Chromium retains FlexStart for these line
alignments; around/evenly still use logical Start. Patched only the flex line
call site. Shared alignment/grid code is unchanged.

To validate without replacing binaries in the ongoing L05 full run, added an
isolated Taffy library test using the84 overflow references from the browser
oracle. Initial root-package invocation could not test dev dependencies for a
non-workspace member; direct manifest invocation required an explicit workspace
exclude. Added vendor/taffy-0.12.1-rive-yoga-order to root excludes, consistent
with other vendored crates. No runtime publisher/probe executable was rebuilt.

Standalone regression1/1 passes (/tmp/wrap-reverse-taffy-standalone-corrected.log,
session84906 exit0). Full Taffy library98/98 passes (/tmp/wrap-reverse-taffy-all.log).
The L05 execution inputs were hashed before the patch and verified unchanged
after the isolated tests: distributed-spacing-full-input-hashes.json.

Next after L05 full regression finishes: rebuild/run the complete336-case
runtime investigation with the fix, implement typed compiler wrapping and
replace the temporary mutation with original CSS, then public/parity/pixel gates.
Current isolated success is not full compiler or native renderer qualification.

Compiler implementation prepared while L05 full regression still uses its
existing executables: WrapMode enum preserves Rive0/1 and emits2 for wrap-reverse.
Both wrapping modes emit independent line policy; text-block reset uses NoWrap.
Geometry test now uses original CSS with no style mutation. Added public tests
for bytes/source-map/manifest equivalence through variables, inheritance, resets,
important precedence, invalid syntax and combined version8 distribution.
Removed obsolete wrap-reverse rejection from custom-properties exclusions.

Prepared validation/wrap-reverse-cases.json with116 scenes/348 viewports, including
112 box cases and4 text compositions (ordered cards, reversed column summary,
responsive cards and mixed-font baseline wrapping). This file is not yet in the
main pixel corpus; do not claim its tests have run. Merge it only after L05 full
regression finishes, then build/test and perform the remaining gates.

L05 full regression finished2326/2326 and all2316 image pairs transferred exact
review evidence. Only then rebuilt the public compiler/runtime. Initial public
wrap-reverse tests2/2 and runtime test1/1 pass in /tmp/wrap-reverse-public-first.log
(session16843 exit0): original CSS, emitted wrap value2, all336 Chromium references
on both original and clone through three widths (672 viewport comparisons).
The placeholder compile and imported-style mutation are removed.

Merged the116 prepared fixtures into validation/cases.json after L05 finished.
Added checked host same-byte resize regression at240/390/768. Corrected invalid
syntax tests to use explicit valid document dimensions so rejection cannot come
from document sizing. Full module and WASM builds now running in
/tmp/wrap-reverse-module.log and /tmp/wrap-reverse-wasm.log.
Next: native probe build, host/parity/types and348 focused native pixels, direct
visual review, vector evidence and full regression. No L06 pixel qualification yet.

Full public module223/223 passes (/tmp/wrap-reverse-module.log, session65007 exit0),
including original/clone geometry and updated valid-dimension rejection tests.
Native publisher/probe and WASM builds pass (/tmp/wrap-reverse-build.log and
/tmp/wrap-reverse-wasm.log; WASM session44420 exit0). Host tests8/8 pass
(/tmp/wrap-reverse-host.log, session82771 exit0), including direct emitted-file
wrap-reverse reflow at three widths.

Parity active: /tmp/wrap-reverse-parity.log, session56446.
Native348 pixel run active: /tmp/wrap-reverse-native.log, session44920;
output/playwright/html-to-riv/wrap-reverse-native. Poll these exact handles.
Next: inspect every new image (including baseline composition), preserve any
failures, run vector profile, then full native regression and qualification.

Native/WASM corpus parity9/9 passes (/tmp/wrap-reverse-parity.log, session56446
exit0). TypeScript API checks pass (/tmp/wrap-reverse-types.log). Native348
pixel run remains active at session44920; no pixel qualification claimed yet.

Initial native348/348 pixel comparisons pass (/tmp/wrap-reverse-native.log,
session44920 exit0). Inspected12 composition pairs in4 sheets. Box contact sheets
are generated but not inspected yet:224 unique pairs in38 sheets for336 cases,
wrap-reverse-box-review/manifest.json. Receipt wrap-reverse-native/visual-inspection.json
correctly leaves all336 box cases pending.

Visual review caught a fixture coverage gap: the mixed-font baseline composition
fits a single line at all widths. Retained it as a single-line control and added
wrap-reverse-composition-baseline-multiline with an Extra label. New browser-only
separation checks require a/d vertical distance>=80px at240 and<=30px at390/768.
These checks guard fixture coverage and cannot substitute for native geometry.
Current L06 corpus is351 viewports; the completed348 run predates the new3.

Vector initial348 run remains active at session4513 (/tmp/wrap-reverse-vector.log).
After it finishes run new multiline baseline3 natively and in vector, inspect
them, then complete box and vector reviews. Avoid concurrent pixel runs.
Composition sheet helper now accepts optional case-name prefix; verified by
generating exactly4 original L06 composition sheets without temporary links.

Expanded multiline baseline native3/3 passes and is directly visually inspected
(/tmp/wrap-reverse-baseline-native.log, session89387 exit0). Receipt
wrap-reverse-baseline-native/visual-inspection.json. All15 native composition
viewports are now reviewed;336 box images still await38 contact sheet inspections.
Original vector337/348 passes with11 text composition failures (session4513 exit1).
Additional multiline vector0/3 passes, all3 fail (/tmp/wrap-reverse-baseline-vector.log,
session67338 exit1). Vector geometry audit and image review remain pending.
Expanded parity active: /tmp/wrap-reverse-expanded-parity.log, session69875.
Full native regression started after all focused native pixels passed; visual
review continues while it runs: /tmp/wrap-reverse-full.log. Do not qualify L06
until all reviews, parity and full regression gates pass.

Full native live handle:session47222. At completion compare against
distributed-spacing-full, wrap-reverse-native and wrap-reverse-baseline-native,
then inspect any changed image pairs. Do not restart while live.

Expanded parity9/9 passes (/tmp/wrap-reverse-expanded-parity.log, session69875
exit0). Native box inspection completed indices0..107 in sheets00..17, covering
174/336 source cases including exact within-run duplicates. All row/reversed-row
cases and first column cases inspected without visible mismatch. Sheet01 and
its original single-item screenshot additionally inspected separately.
Remaining indices108..223 in sheets18..37 cover162 box cases. Updated partial
receipts correctly retain these as pending; all15 native compositions are reviewed.

Vector geometry audited independently from pixel status:348/348 original cases
maximum error0.015625px and3/3 added multiline cases maximum0.0126953125px.
No geometry or hidden-state differences outside tolerance. Fourteen pixel
failures remain. geometry-audit.json saved in wrap-reverse-vector and
wrap-reverse-baseline-vector. Vector composition sheets generated with the
new prefix filter but not yet inspected: wrap-reverse-vector-composition-review
(four sheets) and wrap-reverse-baseline-vector-review (one sheet).
Next: finish native box sheets18..37, inspect vector compositions, transfer only
qualified identical box reviews, then finalize after full native session47222 ends.

Native box review completed: sheets18..37 were actually inspected in the next continuation, completing all224 unique pairs across38 sheets and all336 source cases (112 exact within-run duplicates). Overflow, column-reverse, single-item, stretching and distributed-spacing comparisons show no visible mismatch. Native receipts now have no remaining images, including the separately reviewed15 composition pairs. Vector composition review and the full2677-test regression remain pending.

Vector visual review completed: all five composition sheets inspected at240/390/768 (15 pairs). Visible differences concentrate on glyph edges while box geometry, line wrapping and baselines agree. Exact HTML/CSS and both PNG hashes transfer all336 box comparisons from the completed native review (`wrap-reverse-vector/baseline-comparison.json`). Both vector visual receipts have no remaining inspections. Geometry351/351 passes, maximum0.015625px. Vector pixels337/351 pass;14 text failures are preserved and remain unqualified. No tolerance changes. Full native regression is still running under session47222.

L06 final native qualification: full2677/2677 passed (session47222 exit0, `/tmp/wrap-reverse-full.log`). All2667 image pairs match reviewed baselines in source HTML/CSS and both PNG hashes; the full visual receipt has no remaining inspections. Execution binaries, WASM and case corpus verified unchanged through completion before any L07 rebuild. Native wrap-reverse is qualified; vector geometry passes but14 reviewed text pixel failures remain explicit limitations.
