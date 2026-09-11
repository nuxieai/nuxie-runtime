# Negative margins (L16)

Status: native profile qualified. The current compiler accepts the syntax below.
Full native regression and visual accounting pass; nine vector text comparisons
remain unqualified.

Implemented syntax: physical margin shorthand and four longhands accept signed px,
em, rem and percentages; auto and CSS-wide values retain current semantics.
Lengths remain bounded to an absolute1000000px, percentages to10000%.
Negative padding, gaps and dimensions remain invalid. Logical properties,
positioning and general CSS math retain their separate backlog scope.

Margins participate in flex layout and overlap; they are not transforms and
must retain responsive semantics. Flex margins do not collapse. Percentage
margins use the containing inline size. Validate both box models, all four
flex directions, shrink/grow, wrapping, intrinsic sizes, auto margins, order,
clipping, text/image composition and overlap paint order. Large negative
margins and negative outer extents need explicit Chrome controls.

Validation: first establish public compile tests (negative_margins.rs), then
native original+clone compile-once resize240→390→768→240, native/WASM parity,
Chrome geometry and real renderer pixels with direct visual review. Keep the
known vector text residuals open independently; do not waive new failures.

## Current qualification audit

- Syntax: shorthand/physical longhands; signed px/em/rem/percent, auto,
  custom-property substitution, and existing CSS-wide keyword handling.
- Bounds: absolute resolved length at most 1,000,000px and percentage at most
  10,000%; negative padding, gaps and dimensions still reject.
- Focused native geometry/pixels: 432/432 across 144 distinct scenes, covering
  basic spacing, overlap, intrinsic extents, flex growth/basis, ordering,
  clipping, nested percentages, min/max, distribution, text/images, ratio,
  reverse wrapping, stretch and percentage/auto combinations.
- Compile-once lifecycle: all three public oracle groups pass original and
  cloned instances through 240→390→768→240 (1,152 instance updates).
- Visual review: all 432 native views audited. Vector has 423/432 pixel passes;
  all 432 vector views have direct or exact-input/full-image review evidence.
- Native/WASM parity: expanded corpus passes all 11 JavaScript tests.
- Full public suite: 307 tests pass, none ignored.
- Full native regression: 5,983/5,983 checks pass; all 5,973 scene pairs have
  identical HTML/CSS and complete browser/native PNG bytes to the reviewed L15
  baseline. All six shared harness hashes match the frozen run inputs.
- Remaining: unresolved vector text rendering. The nine text failures persist in reduced controls
  without negative margins; integer line-height changes the outcome. This
  does not waive them or prove their precise rendering root cause.
- Exclusions: logical margins, margin collapsing/general block flow, CSS math,
  positioning, Grid and editor integration are outside this increment.

The authoritative aggregate receipt is
`output/playwright/html-to-riv/negative-margin-receipt.json`. Entries below are
chronological history and do not override this current audit.

Initial red command: CARGO_INCREMENTAL=0 cargo test -p nuxie-html-to-riv --test negative_margins
Log: output/playwright/html-to-riv/negative-margins-red.log

L16 signed-margin parser implemented as an unqualified candidate. Public tests went red(2 failures) before the change and now pass2/2:shorthand/longhands, px/em/rem/percent, custom values, explicit magnitude bounds and negative-padding/dimension rejection. Obsolete margin:-1% rejection updated while padding:-1% remains rejected. Chrome/runtime overlap, extents, resize/clone, parity and regression gates remain pending; no visual support claim. Logs:output/playwright/html-to-riv/negative-margins-{red,green}.log.

L16 initial geometry gate passes48 Chrome scenes/144 reference views and384 original-and-clone viewport updates. Covers4 directions×2 box models×pixel/percent/auto/wrap/large-negative-extents/intrinsic cases. Focused public regression passes58 tests; obsolete auto-margin and custom-property rejection assertions removed, accepted behavior covered by new tests. Frozen native/WASM build45752 running in negative-margin-toolchain. Pixel, visual, parity, full regression and expanded composition qualification remain pending. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json.

L16 initial native pixels pass144/144 comparisons at unchanged geometry/pixel gates. Full public9353 remains active. Initial JS parity failed because mutable target/debug/html-to-riv disappeared during Cargo build; frozen-binary retry50563 is running and the failed log is preserved. Native visual review, vector replay and expanded coverage remain pending; no qualification claim.

L16 full public regression passes305 tests,0 ignored. Initial vector replay144/144 geometry/pixel comparisons passes. Frozen JS suite passes10 tests including corpus parity; sole failure was a missing probe symlink, corrected and the host-contract test rerun successfully(1/1). Both failed infrastructure logs retained. Native visual review48/144 complete:all large-negative-extents and percentage cases inspected across directions and box models; overlap, clipping and reverse anchoring align with Chrome. Remaining96 visual views, expanded compositions and full native regression stay open. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json.

L16 expanded composition geometry passes64 scenes/192 Chrome views/512 original-and-clone viewport updates. Includes growth/basis, order, clipping, nested percentage margins, size limits, distribution, text and images across all directions/box models. Native replay8214 and expanded parity2073 are running. Full frozen native regression46959 started in negative-margin-full; preserve shared harness and main corpus while it runs. Basic visual review72/144:all intrinsic cases now inspected, matching parent sizing, overlap and clipping. Qualification remains incomplete.

L16 basic visual review complete:144/144 native views directly inspected and audited; all144 vector comparisons transfer with exact compiler-input and full browser/native PNG identity. Final auto/pixel-margin sheets match Chrome placement, overlap, reverse anchoring and clipping. Expanded native/WASM parity completes11/11. Composition vector replay completes183/192 with9 unwaived pixel failures; native192/192 passes numerically, with visual review pending. Full native regression46959 confirmed live. Receipt:output/playwright/html-to-riv/negative-margin-receipt.json. Qualification remains incomplete.

L16 composition visual review48/192: all clipping and nested-margin scenes inspected at240/390/768 across four directions and both box models. Clip boundaries, overflow, overlap and reverse placement align with Chrome. Nested content-box cases retain thin vertical teal/purple boundary raster differences within unchanged gates, explicitly recorded. Aggregate native visual192/336;144 composition views remain. Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 composition visual review96/192: all grow/basis and min/max-limit scenes inspected at240/390/768 across four directions and both box models. Responsive sizing, sibling shrinkage, overlap and reverse anchoring match Chrome. Aggregate native visual240/336;96 composition views remain (order, distribution, text and images). Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 composition visual review144/192: all order and distribution scenes inspected at240/390/768 across four directions and both box models. Subtree overlap paint order, space-evenly allocation, cross-axis alignment and narrow overflow match Chrome. Aggregate native visual288/336;48 text/image views remain. Full native46959 confirmed live;9 vector pixel failures remain unwaived.

L16 focused native visual review complete336/336 (basic144 plus composition192). Final48 text/image views inspected at240/390/768 across four directions and both box models. Wrapping, auto text-card height, image quadrants, overflow and overlap align with Chrome. Small native glyph coverage differences near the end of weekend remain within unchanged gates and are recorded. Full native46959 confirmed live; composition vector audit and9 unwaived pixel failures plus interaction coverage audit remain.

L16 vector visual audit complete336/336: basic144 exact transfers; composition168 exact compiler-input/full-PNG transfers plus24 directly inspected text views. All9 failures are local RGB errors in row, row-reverse and column border-box text at every width. Wrapping and surrounding geometry agree with Chrome; visible glyph coverage differences retained, including passing controls. Explicit failure receipt:negative-margin-composition-vector/failure-visual-inspection.json. Full native46959 confirmed live; pixel diagnosis and interaction coverage audit remain.

L16 vector minimization: four fresh pinned-Chrome controls/12 views reproduce9 local RGB failures with all geometry passing. Removing every margin or removing siblings retains failure at all widths; changing line-height22.4px to22px passes3/3. Negative margins therefore are not necessary for the residual; fractional line-height sensitivity is consistent with the prior vector baseline investigation, without proving identical root cause. Receipt:negative-margin-text-minimize-vector/diagnosis-receipt.json; direct visual review of these reduced sheets remains pending. Full native46959 remains live (last observed check2242 passing).

L16 coverage audit adds32 scenes/96 pinned Chrome153 references for aspect ratio, reverse wrapping/distribution, stretch and percentage/auto-margin combinations across all directions and box models. Permanent public original/clone resize test and JS parity corpus added. Public20907 and frozen native replay48811 started; qualification pending results and visual review. Reduced vector diagnosis12/12 now directly inspected, including residual glyph differences in the passing22px control. Full native46959 confirmed live; no tolerance changes.

L16 interaction native96/96 and original/clone public test pass. Aspect-ratio subset24/96 views directly inspected and audited; ratio sizing, column shrinkage, overlap and reverse placement align with Chrome. Remaining72 views cover reverse wrapping, stretch and percentage/auto combinations. Expanded parity10449 and vector replay68859 started; full native46959 confirmed live.

L16 interaction visual48/96: reverse-wrap subset24 views inspected across all directions/box models. Narrow content-box row line stacking, distribution and overlap match Chrome. Vector replay96/96 passes at unchanged gates. Remaining48 views cover stretch and percentage/auto combinations. Expanded parity10449 and full native46959 confirmed live.

L16 interaction review complete96/96: stretch and percentage/auto-margin combinations inspected at all widths, matching Chrome sizing, overlap and overflow. All96 vector views transfer with exact compiler inputs and full PNG identity; transfer audit passes. Expanded native/WASM parity11/11 passes. Total focused native432/432 numeric and visually audited; vector423/432 with9 reviewed unwaived text failures. Full native46959 confirmed live; final qualification audit remains.

## Native qualification completion (2026-09-10)

Session46959 exited0. `negative-margin-full/receipt.json` and
`visual-inspection.json` record the completed full gate and verified transfer of
all5,973 scene reviews. `compare-baselines.py` rehashed both PNGs per case, checked
source identity and pinned the source review receipt. No tolerance changes or
newly waived failures. Public307/55groups (none ignored) and expandedJS11/11 are
confirmed from terminal logs. The native profile is qualified for the documented
signed physical margin scope. Vector423/432 retains nine unwaived text failures.
