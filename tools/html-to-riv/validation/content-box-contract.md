# L12 content-box sizing contract and work record

Status: native-qualified against the frozen v13 toolchain. Chromium remains the oracle. Vector card rendering has24 reviewed, unwaived pixel failures.

## Current qualification evidence

Native qualification is complete for the frozen v13 snapshot. Subsequent L13 engine changes are outside this receipt and require their own regression checks.
All paths below are relative to `output/playwright/html-to-riv/`.

| Gate | Current evidence |
| --- | --- |
| Public compiler | 237 tests across 48 targets; `content-box-public-integration/receipt.json` |
| Runtime lifecycle | Original/clone resize and clear-policy tests; 640 browser geometry comparisons |
| Host admission | 14 tests; version-13 capability and occurrence targets validated |
| Native/WASM | 2,017 inputs, 9 passing tests; `content-box-edge-native-v2/parity-receipt.json` |
| Native geometry/pixels | 510/510 pass: initial240, expanded168, cards24, edges54, images24 |
| Native visual review | All510 reviewed; each corpus has `visual-inspection.json` |
| Expanded vector | 168/168 pass, all reviewed; `content-box-expanded-vector/combined-visual-inspection.json` |
| Vector cards | Geometry24/24; pixels0/24; all24 reviewed and failures retained in `content-box-composition-vector/` |
| Full native regression | 5,983/5,983 pass against frozen v13 toolchain; no failed/skipped checks; `content-box-v13-full/completion-receipt.json` |
| Full visual audit | All5,973 pairs exactly match reviewed HTML/CSS and browser/native PNGs; `content-box-v13-full/visual-inspection.json` |

Native pixel passes do not qualify vector card rendering. Borders and positioning
remain separately deferred; the rejected ellipsis fixture lacked its existing
`display:block` precondition and is retained as failure evidence. Validation uses
Chrome at240/390/768 and resizes the same390-compiled scene, with unchanged gates.

The following work record contains historical intermediate counts; the table
above summarizes the latest verified evidence.

## Intended semantics

Accept explicit `box-sizing: content-box` and `border-box` on supported layout
and text elements. Preserve the existing authoring reset (`border-box`), while
`initial` and non-inherited `unset` resolve to CSS's `content-box` initial value.
Explicit `inherit` copies the parent's computed mode. Cover variables, selector
cascade, inline declarations and important declarations without baking used sizes.

Content-box applies to authored width/height, min/max dimensions and explicit
flex basis. Padding contributes outside those sizes. Auto sizing and intrinsic
text measurement must continue to use the proper content constraints; percentages
must remain responsive when the same compiled artboard is resized. Zero content
sizes still occupy padding. Min constraints win when min exceeds max.

Primary semantics: [CSS Sizing Level 3, box-sizing](https://www.w3.org/TR/css-sizing-3/#box-sizing).

## Runtime implementation seam

`Style` retains the computed box-sizing mode. Version-13 requirements transport
content-box object targets, and the runtime occurrence policy selects Taffy
BoxSizing::ContentBox. Host validation, clone preservation and dirty layout
updates are implemented. Unannotated Rive layouts retain the default border-box mode. Do not rewrite percentages as fixed pixels
or add a global switch that changes unannotated Rive files. Verify flex-basis and
text measurement paths as well as the final Taffy style.

## Evidence and required validation

Initial corpus: `content-box-cases.json`, 80 scenes, 240 Chromium viewport
captures. Four flex directions, both sizing modes, fixed/zero/percentage sizes,
minimum/maximum/conflicting limits, auto dimensions, and px/percent/auto flex
basis. Asymmetric padding and an inner child expose both outer geometry and
content origin; a sibling exposes occupied flex space.

Current immutable v12 compiler admission:
`output/playwright/html-to-riv/content-box-padding-admission/receipt.json`:
40 border-box controls accepted; 40 content-box cases rejected. Compiler hash
and exact per-case diagnostics are recorded. These are expected pre-feature
failures, not qualification evidence.

Chromium reference capture uses the existing `content-auto-oracle.mjs` runner
with the new corpus and outputs to
`output/playwright/html-to-riv/content-box-padding-oracle`.

Before admission: extend coverage to nested mixed modes, wrapped/reverse lines,
grow/shrink freezing, stretch, absolute positioning, font-relative dimensions,
text wrapping/ellipsis/clipping, images and cascade keywords. Add public API and
native/WASM parity tests, capability/version validation, and original/clone
resize lifecycle assertions. Replay real native renderer pixels from one
390-wide compilation at 240/390/768, keep current tolerances, inspect every
result or transfer exact reviewed images, then run the full regression suite.

## Exclusions and preserved reproducers

Border declarations are already unsupported and belong to P01. The original
80 border-bearing probes are preserved in `content-box-border-deferred-cases.json`;
Chrome captures live in `output/playwright/html-to-riv/content-box-oracle` and
compiler diagnostics in `output/playwright/html-to-riv/content-box-admission`.
Both modes currently reject there because border-width is unsupported (content-box
fails first where present). Validate combined border/content sizing when P01 lands;
do not claim that interaction is qualified by padding-only cases.

Percentage padding, new intrinsic sizing keywords, grid, scripting, interaction,
bindings, animation and editor integration remain outside this feature. Existing
vector text pixel failures remain unwaived. L12 qualification is still pending.

## Runtime foundation evidence

The per-occurrence `set_css_content_box_occurrence` policy now selects Taffy's
content-box mode, preserves that mode when cloning, and invalidates layout when
changed or cleared. Unannotated scenes retain border-box sizing.

`content_box_runtime` passes fixed/percentage dimensions, min/max/conflicting
limits, repeated resizing, cloning and policy clearing. `content_box_oracle`
passes all 80 Chrome cases on original and clone at 240/390/768/240: 640
instance/viewport comparisons at the existing 0.1px geometry tolerance.
The test temporarily replaces only the unsupported keyword when compiling and
installs the runtime policy explicitly; this proves the runtime foundation,
not public compiler support. Source/oracle hashes and terminal logs are in
`output/playwright/html-to-riv/content-box-runtime-foundation/receipt.json`.
Compiler transport, admission validation, native/WASM parity and real renderer
pixel qualification remain pending.

## Public compiler integration

Explicit content-box, initial/unset, inheritance, variables and important
cascade now emit version 13 with `layout-css-content-box-v1` and unique
`layout_content_box` object targets. Manifest validation rejects missing
capabilities, wrong versions, duplicates and non-layout targets. The probe
installs the policy before rendering and supports a disabled-capability check.
The border-box reset remains unchanged. TypeScript declarations include v13.

The Chrome oracle test now compiles the actual content-box CSS and installs
published targets; all 640 original/clone resize comparisons still pass.
Public contract tests pass; host admission tests pass 14/14. Native/WASM parity
passes 9/9 tests including the 1927-scene combined accepted corpus. Typecheck
passes. Logs: `/tmp/content-box-public-oracle.log`,
`/tmp/content-box-public-contract.log`, `/tmp/content-box-host.log`,
`/tmp/content-box-parity.log`, `/tmp/content-box-types-v2.log`.

The full public suite found historical assertions that rejected the newly
supported property and its CSS initial fallback. These were converted to
positive expectations (invalid padding-box remains rejected); initial failure
logs are preserved as `/tmp/content-box-full-public.log` and
`/tmp/content-box-full-public-v2.log`. The updated full run is v3.
Frozen four-artifact toolchain: `output/playwright/html-to-riv/content-box-v13-toolchain`.
Initial 240 native pixel comparisons are running in
`output/playwright/html-to-riv/content-box-initial-native`; no pixel or visual
qualification is claimed yet.

Terminal results: full public suite 237 tests across 48 targets passes; initial native replay 240/240 passes. Visual inspection and expanded coverage remain pending. Durable logs and hashes: `output/playwright/html-to-riv/content-box-public-integration/receipt.json`.

## Expanded compositions and visual inspection progress

`content-box-expanded-cases.json` adds 56 scenes: nested mixed box modes,
wrapping, stretch, text wrapping, text limits, clipping and em/rem dimensions,
in both box modes and all four flex directions. Chromium v2 captures and native
v2 replay pass 168/168 geometry/pixel comparisons using the frozen v13 toolchain.
No tolerances changed. Native result: `output/playwright/html-to-riv/content-box-expanded-native-v2/replay.json`.

The original exploratory corpus included positioning (unsupported); it is
preserved in `content-box-position-deferred-cases.json`, the first Chrome capture,
and the failed initial replay log `/tmp/content-box-expanded-native.log`.
Positioning qualification remains dependent on that later feature.

Initial visual review now directly covers 36/240 pairs (12 composition sheets):
fixed/percent/basis rows, fixed/auto/basis columns, reverse percentages, zero
content with overflow, and min/max precedence. All inspected pairs agree in
layout and paint. Hash-pinned receipt: `content-box-initial-native/visual-inspection.json`
under the output directory. The remaining 204 initial pairs and all expanded
pairs still require visual review. Expanded corpus parity and additional
realistic compositions remain pending; L12 remains unqualified.

Expanded native/WASM parity now passes 9/9 tests across 1983 scene inputs;
its durable log is `content-box-expanded-native-v2/parity.log`. Expanded visual
review covers 30/168 pairs (27 direct and 3 exact full-image transfers), including
text wrapping/clipping, nested percentages, stretch and wrapping. Initial
review remains 36/240.

`content-box-composition-cases.json` adds eight realistic summary, pricing,
notice and collection cards in both box modes. Their Chrome/native replay passes
24/24 at unchanged gates: `content-box-composition-native/replay.json`. Nine
pairs have been directly reviewed with matching layout, backgrounds, corners,
text wrapping and footer sizing. The remaining 15 pairs and composition parity
are pending. All references are under `output/playwright/html-to-riv/`.

Card native visual inspection is complete: 24/24 directly reviewed with
hash-pinned screenshots and sheets. Native/WASM parity passes 9/9 tests across
1991 input scenes, including all eight cards (`content-box-composition-native/parity.log`).
Expanded native review is now 48/168; initial review remains 36/240.

The vector card profile preserves 24/24 geometry passes but fails all 24 pixel
comparisons, including local text RGB gates. Evidence lives in
`content-box-composition-vector/replay.json`; none are waived. The summary card
has been directly inspected at all three widths: background/layout/wrapping
agree while glyph edges/weight differ visibly. The other 21 vector pairs still
need review. This is a vector text limitation, not native-profile qualification.

Vector card visual review is complete: all24 directly inspected, geometry and
line breaks match while text weight/edge differences remain visible. All24 pixel
failures are unwaived; receipt `content-box-composition-vector/visual-inspection.json`.
Initial native review now covers60/240 (48direct,12exact full-image transfers),
with all referenced PNG hashes verified. Expanded native remains48/168.
Full native regression, remaining initial/expanded review and other coverage
specified above remain outstanding; no L12 qualification claim is made.

Expanded native visual review now covers 102/168 pairs (99 direct, 3 exact full-image transfers). New reviews cover row/column border-box controls, nested percentages, wrapping, stretch, font-relative dimensions, text constraints and clipping. All match Chrome; 66 pairs remain. Receipt: `content-box-expanded-native-v2/visual-inspection.json`. Initial60/240 and both card-profile reviews are unchanged.

The 144 content-box cases are now included in the main corpus (1979 scenes).
Parity keeps the 12 separate indefinite auto-main cases, preserving1991 total
unique inputs. A frozen v13 full native run is active with5983 expected tests:
`output/playwright/html-to-riv/content-box-v13-full`; input snapshot is saved as
`cases-at-run.json`. Do not mutate the main corpus during this run. Terminal
completion and comparison with prior reviewed full baseline are pending.
Expanded native visual review is132/168; remaining36 are recorded explicitly.

Expanded native visual review complete:168/168 (162 directly inspected, 6 exact full-image transfers). All screenshot/sheet hashes and unique coverage verified. Initial native60/240 remains partial; full native regression still running.

Initial native visual review now covers114/240 pairs (87 direct, 27 verified full-image transfers). Additional row controls and reversed content-box fixed/percentage/minimum/auto/flex-basis cases agree with Chrome. Expanded168/168 and card reviews remain complete; full regression is active.

L12 initial native visual review now covers 174/240 pairs (126 direct, 48 exact image transfers). Reverse-row border-box and column zero/percentage/minimum/basis controls match Chrome. Remaining 66 pairs and full regression completion still prevent qualification. Expanded and card visual receipts remain complete.

L12 initial native visual review is complete240/240 (174 direct, 66 exact full-image transfers); all PNG and sheet hashes verified. Together with expanded168 and cards24, all432 focused native pairs are reviewed and pass. Vector cards24 failures remain unwaived and reviewed. Full native regression and coverage audit remain pending.

L12 coverage audit: padded images (fixed/percentage/max-width/rounded clip, both box modes) pass 24/24 Chrome/native geometry and pixel comparisons, with all 24 directly visually reviewed in `output/playwright/html-to-riv/content-box-image-native/visual-inspection.json`. Edge coverage passes 54/54 comparisons in `content-box-edge-native-v2`; its visual review is pending. The initial edge replay stopped on an ellipsis fixture missing the documented `display:block` precondition; both rejected inputs and the failure log are preserved. Corrected fixtures explicitly set block display. Added both corpora to native/WASM parity; its terminal result and the active full regression remain pending. No tolerance changes.

L12 audit completion: edge native54/54 now directly visually reviewed; image native24/24 already reviewed. Expanded native/WASM parity passes all9 tests across2017 inputs; receipt `output/playwright/html-to-riv/content-box-edge-native-v2/parity-receipt.json` records corpus and matching frozen compiler hashes. Total focused native coverage is510 passing, visually reviewed comparisons. Full regression and additional vector text coverage remain pending.

L12 expanded vector coverage passes168/168 geometry and pixel comparisons. Visual accounting currently105/168:96 exact full-image transfers from reviewed native results plus9 direct text/clip/max-width reviews;63 remain. Receipt: `output/playwright/html-to-riv/content-box-expanded-vector/combined-visual-inspection.json`. Existing vector card failures remain unwaived. Full native regression is still running.

L12 expanded vector visual review is complete168/168 (72 direct,96 exact full-image transfers), with screenshot and sheet hashes verified. All168 geometry and pixel gates pass. Vector cards retain24 reviewed, unwaived pixel failures. Full native regression and final evidence audit still prevent qualification.

L12 native qualification complete for frozen v13: full5983/5983 tests pass, and all5973 scene pairs exactly match reviewed source and full-image baselines. Focused native510/510 passes/reviewed; parity2017 inputs and host14 pass. Vector expanded168/168 passes/reviewed, while card24 pixel failures remain reviewed and unwaived. See `output/playwright/html-to-riv/content-box-v13-full/completion-receipt.json` and `validation/content-box-contract.md`. Subsequent L13 engine edits are not covered by this frozen snapshot and need separate regression evidence.
