# L03 align-self: implementation seam investigation

Status: qualified for the native glyph profile. Full native1723/1723 and
new compositions9/9 pass with all image pairs visually accounted for. Vector
geometry108/108 passes;30 text raster failures remain explicit limitations. Historical notes below retain the investigation
sequence; current results are recorded at the end. Investigated while the
L02 broad native run was active; publisher and renderer binaries were not changed.

CSS align-self overrides the parent's cross-axis align-items for one flex item.
Its initial value is auto and it is not inherited; auto uses the parent's
align-items. Cross-axis auto margins take precedence. Flexbox defines auto,
flex-start, flex-end, center, baseline and stretch. Explicit inherit must copy
the parent's computed align-self, not its align-items. Preserve that distinction
through var() and initial/unset. [CSS Flexbox section 8.3](https://drafts.csswg.org/css-flexbox-1/#align-items-property).

Current runtime seam: layout/layout_style_applier.rs::set_align_self already maps
YGAlign to Taffy align_self. LayoutComponent::apply_* sizing near the end of
layout_component.rs overwrites align-self with Stretch for cross-axis Fill and
Auto otherwise. LayoutComponentStyle's alignment_type controls container alignment,
not an individual child's align-self. Merely changing compiler `Style.align` would
align descendants and is therefore incorrect. A correct extension needs a distinct
per-layout occurrence policy (or an evidenced binary field), applied after the
existing sizing-derived auto/stretch assignment. Raw Rive defaults must remain
unchanged. Check layout_participant.rs separately for non-LayoutComponent items;
the compiler's source-mapped boxes currently lower to LayoutComponent.

Validation required before admission/qualification:
- Public compile/import geometry at240/390/768 from one390-wide binary, comparing
  auto to parent align-items, explicit overrides, non-inheritance, explicit inherit,
  initial/unset, and var() valid/invalid substitution.
- Row/column and reverse directions, fixed/auto cross sizes, min/max constraints,
  margins/padding, text intrinsic sizing, images and nested responsive cards.
- Stretch must respect definite cross sizes and min/max; do not replace runtime
  sizing with browser coordinates or equate every auto size with Fill.
- Baseline needs actual text baseline evidence; do not silently map it to start.
  Any deferred values must have explicit diagnostics and tracked residual scope.
- Wrap-line behavior needs browser evidence under current align-content limitations.
- Native/WASM identical bytes/maps/requirements, host capability/target rejection,
  reimport/instance behavior, real native pixels and visual review receipts.

Grid, block formatting contexts, writing-mode, positioning and auto margins are
not admitted by this investigation. Their existing exclusions remain in force.

Browser-only oracle prepared: validation/align-self-oracle.mjs captures216 cases
on Chromium153.0.8010.12 into output/playwright/html-to-riv/align-self-oracle.json.
Four directions × six values × fixed/auto/min-max-bounded sizes × three viewports.
At240 row width, auto follows parent flex-end (fixed30px box at y80; auto20px at
y90); stretch positions at y10, keeps fixed30px, expands auto to100px, and clamps
bounded auto to60px. This is browser reference evidence only: no compiler or native
qualification is implied, and baseline text behavior still needs dedicated cases.
The initial center/fixed request is saved at align-self-initial/scene.json.

Initial compiler request failed as expected with unsupported-property at
css:1:106 (/tmp/align-self-initial.log), using the existing publisher binary.
This is the red starting point for L03; no compiler behavior changed during L02
validation.

Runtime seam increment: public CssAlignSelf enum and per-layout optional override
apply after Rive's sizing-derived alignment. None restores raw behavior; the
override is copied by LayoutComponent clone_core. Setter marks layout style dirty.
The first public regression failed to compile against the missing enum/method
(/tmp/align-self-runtime-red.log). It then passed216 Chromium box references
(/tmp/align-self-runtime-first.log), compiling72 scenes once at390 and resizing
each to240/390/768.

Expanded oracle adds parent align-items:stretch alongside flex-end, producing432
comparisons in144 scenes. Initial expanded run failed: explicit auto returned
20px height instead of Chromium100px, because the compiler encodes parent stretch
through child Fill sizing rather than parent YGAlign::Stretch. CssAlignSelf::Auto
now preserves the resolved sizing-derived default; other values override it.
Expanded432 comparisons pass (/tmp/align-self-runtime-auto.log). Original failed
run /tmp/align-self-runtime-expanded.log and initial216 oracle are retained.
Generator: validation/align-self-oracle.mjs; grouped public-test fixture:
tests/assets/align-self-boxes.json. Expanded full browser reference remains
output/playwright/html-to-riv/align-self-oracle-expanded.json, Chromium153.0.8010.12.

This is box geometry only, not native pixel or text-baseline qualification. No
compiler syntax or host requirement is admitted yet. Next add an explicit per-layout
capability/target contract and style lowering, including cascade, substitution and
reset behavior; retain engine auto/stretch distinction. Additional clone/clear
regression and full public suite are running in /tmp/align-self-runtime-module.log.

Lifecycle correction: first full suite stopped on the new test because changing
alignment after layout left y80 unchanged instead of moving to y45. Updating only
component dirt did not synchronize the retained layout node. The public setter is
now LayoutComponent::set_css_align_self_occurrence(&CoreHandle, Option<CssAlignSelf>)
and returns false for a non-layout target. It releases the owner borrow, synchronizes
style, marks the layout node dirty, then marks layout style dirty.
The432 geometry cases and clone/change/clear/default-restoration test pass2/2
(/tmp/align-self-runtime-lifecycle.log). Earlier failed harness/runtime logs remain.

Oracle regeneration is reproducible: run
`node tools/html-to-riv/validation/align-self-oracle.mjs OUTPUT.json FIXTURE.json`.
A fresh capture into /tmp/align-self-boxes-reproduced.json exactly equals the
checked-in fixture's parsed data, including all432 browser boxes.
The initial216 reference and expanded432 reference remain separately preserved.

No HTML/CSS syntax is newly accepted yet. Baseline enum behavior has only non-text
box geometry coverage, not actual text-baseline or pixel qualification. CSS-wide
keywords, substitution, runtime requirement schema/target validation and published
JS/native parity remain subsequent work. Grid and other exclusions remain intact.

Full module suite207/207 passes (/tmp/align-self-runtime-module-final.log). All
processes in this increment are terminal. L03 remains in progress, with runtime
geometry established and compiler admission deliberately not implemented yet.

Compiler/contract increment (2026-09-09): accepts auto, flex-start, center, flex-end,
stretch and baseline plus existing inherit/initial/unset/var() semantics. Property
is not implicitly inherited. Valid unsupported alignment forms (normal, self-start,
safe/unsafe modifiers, first/last baseline) retain unsupported diagnostics rather
than silently resetting after var(); invalid substituted types reset to unset.
CSS auto emits no occurrence record. Non-auto emits version6,
layout-css-align-self-v1 and unique `{object_id,alignment}` layout_align_self records.
Version6 permits previous payloads; painted descendants use max(version,5), avoiding
a downgrade. Host validates schema, capabilities and all actual target types before
applying overrides. Unknown enum values/fields, duplicates, missing support and
wrong/nonexistent targets are tested. TS version/capability unions updated.

The432 oracle comparisons now compile actual align-self CSS and assert/install its
published target, instead of bypassing unsupported syntax. Public suite209/209,
host5/5, native/WASM builds, parity9/9 and types pass. Logs:
/tmp/align-self-compiler-contract.log, /tmp/align-self-host.log,
/tmp/align-self-publisher-build.log, /tmp/align-self-wasm.log,
/tmp/align-self-parity.log and /tmp/align-self-types.log.

First rendered matrix:72/78 native pass in align-self-first. All72 non-text
alignment cases pass; six mixed-size text baseline cases fail in row and
row-reverse at240/390/768. Directly inspected all six failing image pairs. At390
row: Chromium a.y23,b.y10,c.y20; runtime a.y27.009979,b.y10,c.y22.757484. Labels
are visibly low. Width/height differences stay small; vertical baseline is wrong.
The72 passing box images still await direct review; do not count automation alone
as visual qualification. Original failed files and /tmp/align-self-first.log remain.

Ranked baseline hypotheses: missing first-baseline information in the retained
Taffy bridge; incorrect font/leading metric conversion; direction-specific grouping.
Both directions fail with the same vertical offsets while box baselines pass,
favoring a shared text measurement issue. Source evidence: LayoutComponent calls
TaffyTree::compute_layout_with_measure with a closure returning only Size. The
TaffyView leaf branch calls compute_leaf_layout without a supplied text baseline;
LayoutOutput does have first_baselines, but that information is absent from the
current measure API. Investigate an explicit baseline channel and derive values
from font/line metrics; do not bake Chromium positions or change raw Rive defaults.
No baseline fix or full feature qualification is claimed yet.

All processes from this increment are terminal. Requirement diagnostic wording
now mentions version6 where applicable. The overall goal remains active; next
work is the text-baseline channel and remaining visual qualification.


## Baseline channel and nested sizing follow-up (2026-09-09)

Added an optional Taffy first-baseline callback after measurement, inside the
layout-output cache. Existing Size-returning measure callers remain unchanged.
The runtime enables the callback only when the layout tree contains an explicit
CSS baseline override and dirties cached nodes when that policy changes. Text
provides its first measured alphabetic baseline, adjusted by origin, trim and
authored text offset; internal line-leading padding propagates through Taffy.

Focused bridge test passes (1/1, /tmp/align-baseline-bridge-test.log). Public
alignment runtime tests pass (3/3, /tmp/align-baseline-lifecycle.log), including
432 Chromium box checks, override/clear/clone and text-baseline policy toggles
at all three resize widths. Original six text cases now pass. Expanded native
run passes18/21 (/tmp/align-baseline-expanded.log). Directly inspected all21
image pairs in align-baseline-expanded/sheets: row, row-reverse, wrapped,
padding, leading and image cases match; nested auto-width wrappers collapse.

Ranked hypotheses for nested failure: intrinsic measurement returns zero;
baseline callback changes sizing; nested baseline propagation is wrong. Added
fixed-width and alignment-auto controls without removing the original failure.
Native nested controls pass3/9 (/tmp/align-baseline-nested-controls.log): fixed
width passes all widths, both original and alignment-auto collapse at all widths.
Directly inspected the fixed-width and auto-control sheets (six image pairs).
This isolates intrinsic width, independently of enabling the baseline callback.
Source inspection shows AutoHeight measurement returns base.width() when no exact
constraint is supplied, even for CSS nowrap whose authored fallback is zero.
A CSS-policy-only correction now returns measured advance in that case, retaining
exact stretch widths and the legacy default path. Build/repro verification pending.

Feature remains unqualified:72 box image pairs still need review; final public,
WASM parity, vector and full native regression gates must run after the fix settles.
No tolerances changed and no Chromium geometry is embedded in runtime output.


Intrinsic-width correction verified: native baseline27/27 passes, including the
original nested and alignment-auto controls at240/390/768. Log:
/tmp/align-nested-width-fixed.log. All27 visually reviewed:21 exact source and
image matches to direct reviews, six corrected nested pairs inspected directly.
Receipts: align-nested-width-fixed/{baseline-comparison,visual-inspection}.json.
Full public suite210/210 passes (/tmp/align-baseline-module-final.log). WASM
refresh and full native regression are running; final qualification remains pending.

Host5/5 and TypeScript checks pass (/tmp/align-baseline-host-final.log and
/tmp/align-baseline-types-final.log). WASM release build succeeds
(/tmp/align-baseline-wasm-final.log). Refreshed JS parity is running in
/tmp/align-baseline-parity-final.log; full native run is active in
/tmp/align-baseline-full.log (session46754). Do not restart a live run.


Final parity9/9 passes (/tmp/align-baseline-parity-final.log). All72 box image
pairs from align-self-first have now been directly reviewed using eight contact
sheets (all three widths retained); no mismatch found. Updated that run's
visual-inspection.json. Full native regression continues. Added price-card,
wrapped ordered tiles and reverse-summary compositions for a subsequent focused
run; these were added after the full run loaded its fixture list.

Added a public runtime regression for nested nowrap intrinsic width with no
align-self override at all. It exercises policy false/true/false/true and
resize240/390/768, comparing both wrapper and label widths to Chromium's
independent40.390625px Inter advance. Legacy policy-off remains zero-width.
All four alignment tests pass (/tmp/align-nested-policy-lifecycle-final.log).
Two intermediate test compile errors were corrected; no production code changed.
Full native run (session46754) remains active. Its fixture list was loaded before the added compositions; the three later composition fixtures need a separate9-case native run,
then rerun parity for the enlarged fixture corpus.

Enlarged corpus parity now running in /tmp/align-composition-parity.log
(session72208). Full native run is1723 tests (session46754). Playwright pixel
runs must remain sequential because they share reporter output. Once full native
finishes, inspect failures or compare images to order-direction-full,
align-self-first and align-nested-width-fixed, then run the three compositions
and focused align-self vector profile. Both jobs were polled/started live in
this increment; do not infer completion from this receipt alone.

New composition parity initially failed8/9 because price-card used48px Inter
with56px line-height, below the currently accepted natural ascent+descent floor.
Native compiler independently rejects it with unsupported-line-height. Preserved
input: output/playwright/html-to-riv/align-composition-initial/
align-self-composition-price-card.json. Changed this fixture to64px line-height
to exercise alignment within the declared profile; the existing small-line-height
limitation is not resolved or hidden. Other two compositions compile.

Corrected composition corpus parity9/9 passes
(/tmp/align-composition-parity-corrected.log). Updated parity assertions to show
compiler diagnostics alongside fixture names on future failures. L04 independent
reference preparation is recorded separately; no production binary changed.

Full native1723/1723 passes (/tmp/align-baseline-full.log);1713/1713 exact
source/browser/native image matches to reviewed baselines, no remaining visual
inspection. New compositions9/9 pass (/tmp/align-composition-native.log), and
all9 image pairs were directly reviewed. Both visual-inspection.json receipts
are written. Full run terminal. Focused align-self vector run is active in
/tmp/align-self-vector.log (session27041); initial box72 pass, text cases are
reporting pixel failures. Preserve and inspect these before final classification.


## Final L03 qualification receipt

Accepted six keywords, CSS-wide/var() semantics, per-occurrence version6 host
contract and independent overrides are qualified for the native glyph profile.
Public210 tests pass plus the added lifecycle test in the4/4 focused runtime
suite. Host5/5, refreshed WASM build, enlarged corpus parity9/9 and types pass.
Full native1723/1723 and subsequent compositions9/9 pass; all1713 full-run
image pairs have identical source/browser/native hashes to reviewed baselines,
and all9 new composition pairs are directly reviewed. Every image fixture is
compiled at390 then imported/resized to240/390/768 using those same bytes.

Focused vector78/108 pixel checks pass; geometry108/108 passes with maximum
absolute error0.013671875px. The30 failures are text pixel metrics, not box or
baseline positions. All108 image pairs reviewed:75 exact transfers and33 direct
text comparisons, including the3 passing leading cases. Glyph edge/weight
differences remain; no claim of complete vector profile qualification. Native
text and vector use the same measured layout. See align-self-vector/
visual-inspection.json and /tmp/align-self-vector.log. Original failing reproducers
remain. No tolerance widening, geometry baking or editor integration was used.

All L03 processes are terminal. L04 preparation is in
validation/align-content-investigation.md; its implementation is next. The
overall backlog remains active and vector raster limitations remain open.
