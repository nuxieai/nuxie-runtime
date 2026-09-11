# L05 distributed spacing

Status: browser references captured; compiler/runtime implementation and qualification pending.
L04 full regression remains active; these preparations do not change its source or fixtures.

## Scope and semantics

Add `justify-content: space-around | space-evenly` and
`align-content: space-evenly` to the standalone flex profile. Retain
`space-between` and the already implemented line `space-around` as controls.
Cover all four supported flex directions, multiple lines, single items,
overflow, mixed item sizes, padding and gaps. Align-content has no effect on
a nowrap container, even when its children overflow. A wrap container with
one line remains eligible for line distribution.

Space-around allocates half as much extra space at the outer edges as between
subjects; space-evenly allocates equal extra space at both edges and between
subjects. Both use safe-center fallback; space-between uses safe-flex-start.
These rules concern remaining space in addition to authored gaps.
Primary reference: [CSS Box Alignment 3, distributed alignment](https://drafts.csswg.org/css-align-3/#distribution-values).

This item does not add Grid, wrap-reverse, writing modes, arbitrary alignment
grammar, place-content, editor integration or interactions. CSS-wide keywords
and custom-property substitution must follow the module's existing contract.

## Independent browser evidence

`validation/distributed-spacing-oracle.mjs` generated 96 scenes and 288 viewport
references using Chromium 153.0.8010.12. Matrix: four directions, two properties,
three distribution values, four modes (nowrap, wrap, single-item, overflow),
three viewport widths (240, 390, 768). Height is 320.

The generator asserts actual cross sizes 20/35/50 and main sizes 60/80/40 for
multi-item cases. Specific child selectors prevent the specificity error found
during L04 visual review. Durable reference:
`tests/assets/distributed-spacing-boxes.json`. Generated original:
`output/playwright/html-to-riv/distributed-spacing-oracle.json`;
log `/tmp/distributed-spacing-oracle.log`.

Three public CLI inputs and their unsupported-value diagnostics remain in
`output/playwright/html-to-riv/distributed-spacing-initial/`: main space-around,
main space-evenly and line space-evenly. All exited 1 without producing Rive.
Browser reference generation does not establish compiler or renderer support.

## Runtime investigation and next implementation

`layout/layout_style_applier.rs` already maps YGJustify SpaceAround/SpaceEvenly
to Taffy. YGAlign has SpaceAround but lacks SpaceEvenly. Rive's authored
LayoutAlignmentType cannot encode these main-axis values directly; extending
the compiler's numeric justify field would overlap its position encoding.

Use an explicit per-container runtime policy applied after ordinary Rive style
translation, with clone preservation, dirty propagation, change/clear behavior
and no override on ordinary Rive files. Extend line alignment without changing
existing numeric YGAlign mappings. Add a versioned host capability/manifest
contract so an older host rejects unsupported policies before installation.
The exact new schema remains to be implemented and tested.

## Required qualification

- Public compiler parsing, CSS-wide values, variables, diagnostics and source maps.
- Strict schema/capability/target validation, including missing-capability hosts.
- Compile once at 390 and resize the same imported bytes through all three widths
  against the 288 independent browser references; clone and clear policy tests.
- Native/WASM byte, source map and runtime requirements parity; TypeScript types.
- Chromium/native pixel matrix plus realistic text compositions and interactions
  between supported layout features, without scripted UI interactions.
- Inspect every new visual pair, preserve failures, run full regression, and
  update SUPPORT.md, VALIDATION.md and BACKLOG.md with profile-specific evidence.

No tolerances are changed. Neither browser references nor passing box geometry
may qualify text pixels or the full compiler feature.

## Runtime implementation and first failure

Added CssJustifyContent occurrence policy and CssAlignContent::SpaceEvenly.
Policies apply after Rive style translation; main policy is cloned, optional by
default, and setter releases the owner borrow before sync/dirty propagation.
YGAlign::SpaceEvenly was appended, leaving existing repr values and numeric
From mappings unchanged. Public compiler transport is not implemented yet.

`tests/distributed_spacing_runtime.rs` is explicitly a runtime-only investigation:
it compiles a supported placeholder, installs the authored runtime policy, then
resizes one imported file through all288 independent Chromium comparisons.
Replace the placeholder with original CSS once compiler transport lands; this
test must not be cited as public compiler qualification in its current form.

First run failed: /tmp/distributed-spacing-runtime-first.log, session20275 exit101.
Row-reverse, justify-content:space-between, nowrap,240px: first child x230 native
versus170 Chromium. The generic Taffy fallback changed distributed overflow to
logical Start. Chromium treats space-between differently from around/evenly:
for the same reversed fixture, around/evenly put a at230 and d at10, while
space-between puts a at170 and d at-50. At80px space-between puts a at10,
while around/evenly still put a at230. These values are in the saved oracle.

Added a flex main-axis space-between overflow fallback to FlexStart in vendored
Taffy; shared line/grid alignment math is unchanged. Verification is running in
/tmp/distributed-spacing-runtime-fallback.log, session4754. This is not yet a
qualified runtime fix; lifecycle, public contract, parity and pixel gates remain.

Fallback verification completed: runtime oracle test1/1 passes all288 viewport
references, session4754 exit0, /tmp/distributed-spacing-runtime-fallback.log.
Added a separate lifecycle regression covering main-policy change across all six
values, resize, clone preservation, independent clone clear and original clear.
Running both tests in /tmp/distributed-spacing-runtime-lifecycle.log.

Initial lifecycle test failed because its copied fixture declared row while
asserting main-axis positions on y (/tmp/distributed-spacing-runtime-lifecycle.log).
Corrected fixture to explicit column; expectations remain10/70/130 based on
160px height,10px padding and20px child. No runtime change for this test bug.
Corrected run: /tmp/distributed-spacing-runtime-lifecycle-corrected.log.

Corrected runtime investigation2/2 passes:288 Chromium geometry references
and six-policy lifecycle/change/resize/clone/clear regression. Session27139
returned exit0. Next: implement public compiler values and strict versioned
host transport, replace placeholder lowering in the geometry test, then parity,
pixels, real compositions, review and full regression.

## Public compiler transport implementation

Accepted new values now lower through version8 with
`layout-css-distributed-spacing-v1`. Main-axis around/evenly use strict
`layout_justify_content` entries (object_id, alignment); line evenly extends
layout_align_content and requires both the line and new distribution capabilities.
Version8 iff a new distribution policy exists; older schemas cannot carry it.
Unique targets, payload/capability consistency and actual imported target types
are validated before host installation. New justification values use a safe
Rive encoding placeholder, then the mandatory runtime policy; no coordinate baking.
Later descendant policies retain max version rather than downgrading8 to7.

The geometry test now compiles original authored CSS and installs the emitted
manifest; the temporary placeholder substitution was removed. Public runtime
geometry/lifecycle2/2 and existing line contract3/3 pass in
/tmp/distributed-spacing-public-runtime.log (session60016 exit0).
TypeScript passes /tmp/distributed-spacing-types-corrected.log after adding
version8 to the test's explicit version union.

Added public cascade/variables/reset and strict manifest regressions, a real
host rejection test,96 colored matrix scenes and3 responsive text compositions
(297 pixel comparisons). Full module first run exposed an obsolete
custom-properties exclusion of justify-content:space-evenly; removed that entry,
with new public acceptance coverage. Corrected full run is active in
/tmp/distributed-spacing-module-corrected.log, session79238.
Native/WASM builds, parity, host verification and all pixel reviews remain pending.

Full public suite220/220 passes (/tmp/distributed-spacing-module-corrected.log),
session79238 exit0. Native publisher/probe build and WASM build pass in
/tmp/distributed-spacing-build.log and /tmp/distributed-spacing-wasm.log
(WASM session11002 exit0). Host tests7/7 pass, /tmp/distributed-spacing-host.log,
session62971 exit0. New host test covers combined main/line evenly, successful
centered output, missing capability, old version, duplicate target, style target
and absent target, with rejection before render stream creation.

Focused native297 pixel comparisons are active: /tmp/distributed-spacing-native.log,
session35060; output/playwright/html-to-riv/distributed-spacing-native.
Corpus parity is active: /tmp/distributed-spacing-parity.log. Do not restart these
merely because a poll yields; observe live handles. After native completion,
review all new images (box contact sheets and full composition sheets), then
vector comparison and full native regression. No L05 pixel qualification yet.

Focused native297/297 passes; all297 images reviewed. Box contact manifest has
152 unique PNG pairs, all inspected in26 sheets;136 byte-identical within-run
duplicates retain their source identities and hashes. Nine compositions reviewed
in3 composition sheets. Receipts: distributed-spacing-native/visual-inspection.json
and distributed-spacing-box-review/{manifest,visual-inspection}.json.
Corpus parity9/9 passes (/tmp/distributed-spacing-parity.log; session43470 exit0).

Vector289/297 pixels pass, with8 composition text failures preserved in
/tmp/distributed-spacing-vector.log (session99222 exit1). All297 geometry cases
pass with maximum absolute error0.015625px. All297 visual pairs accounted for:
288 exact source/browser/native matches to native-reviewed box cases and9
directly inspected compositions. Receipt distributed-spacing-vector/visual-inspection.json.
Vector text raster/weight differences remain open; no threshold changes.

Full native regression is running in /tmp/distributed-spacing-full.log, session21695.
After completion compare images against align-content-corrected-full and
distributed-spacing-native, inspect changed pairs and finalize native qualification
only if full regression and review pass. Do not restart a live run.

Final L05 native qualification: full2326/2326 passes in12.8minutes, session21695
exit0, /tmp/distributed-spacing-full.log. All2316 source/browser/native PNG
pairs exactly match reviewed baselines align-content-corrected-full and
distributed-spacing-native. Receipts distributed-spacing-full/{baseline-comparison,
visual-inspection}.json. Execution-input hashes were captured before L06 source
edits and verified unchanged through full run completion.
L05 native profile qualified; vector8 composition pixel failures remain open.
