# Compiler expansion backlog

This is the persistent work list for the standalone HTML/CSS-to-Rive compiler.
SUPPORT.md is the authoritative accepted language; this file is a plan, not a
promise that pending syntax already compiles. Features retain native responsive
layout wherever representable. The compiler remains independent of the editor.

## Scope and completion

Work in priority order below, choosing independent items when prerequisites are
blocked. Each row is a bounded qualification unit; split it into additional rows
when distinct semantics or dependencies emerge. Do not silently drop rows or
reduce the goal to the easiest subset.

Statuses: **pending**, **active**, **qualified**, **partial**, **blocked**.
Qualified means the explicitly documented subset meets the acceptance gate.
Partial means a useful subset passes but remaining work is still open. Blocked
requires a concrete reproducer, dependency and explanation of why compiler-only
work cannot resolve it. A hard implementation problem is not itself a blocker.
Runtime or format changes may be necessary; investigate and document the seam
before making compatibility-sensitive changes. Do not change baseline runtime
behavior casually to make compiler tests pass.

For each implementation:

1. Specify syntax, semantics, interactions and intentional exclusions.
2. Establish a failing regression at the public compile/native/browser seams.
3. Implement the mapping, including actionable rejection diagnostics.
4. Check Rust behavior and native/WASM byte and source-map parity.
5. Compare Chromium geometry and real native renderer pixels. Compile once and
   resize the imported scene at 240, 390 and 768px where relevant.
6. Inspect browser/native/difference screenshots for the new fixtures.
7. Run the appropriate full gate, update SUPPORT.md and VALIDATION.md, and add a
   validation receipt with commands, results, limits and observed differences.
8. Update this backlog with evidence and the next item.

Do not widen tolerances just to accommodate a feature. Preserve failing
reproducers. Do not substitute browser-baked rectangles for responsive scene
semantics without an explicit documented target/profile decision. Acceptance
of a feature name does not imply every browser combination is supported.

The goal is fulfilled when every row is qualified, or every remaining row has
an evidenced external dependency and no independent implementation work remains.
Report those limitations honestly. A blocked backlog row does not block the
whole goal while useful independent work remains.

Excluded: CSS Grid, editor integration, editor undo/mutation, scripts, event
handlers, interactions, bindings, state machines, animation and arbitrary
web-page importing. Existing strict resource limits remain in force.

## Established baseline

Qualified increments: static HTML/flex profile, embedded fonts and PNGs,
standalone Rust/CLI/WASM/JavaScript interfaces, visual gallery and CI, constrained
flex grow/shrink/basis, RGB/RGBA, currentColor and physical text alignment.
Latest baseline: 143 browser/native checks, 21 Rust tests, five JS/WASM tests,
two gallery checks, TypeScript and module Clippy. See validation/*-review.md.
Existing flex limitations are preserved in validation/deferred-cases.json and
remain work items below; they are not newly qualified by this backlog.

## Priority 1: CSS authoring and typography basics

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| A01 | Standard CSS named colors | qualified | All 148 names; validation/named-hsl-review.md |
| A02 | HSL/HSLA | qualified | Legacy/modern syntax, hue units/wrapping, alpha, clamping; validation/named-hsl-review.md |
| A03 | `inherit` | qualified | All supported properties/shorthands; validation/inherit-review.md |
| A04 | `initial` | qualified | Representable CSS defaults and explicit profile rejections; validation/initial-unset-review.md |
| A05 | `unset` | qualified | Inherited versus non-inherited behavior; validation/initial-unset-review.md |
| A06 | Solid-color `background` shorthand | qualified | Solid colors, none and global keywords; validation/background-review.md |
| A07 | `font` shorthand | partial | Language and macOS glyph lane pass; vector rasterization gaps remain; validation/live-glyph-review.md |
| A08 | Unitless line-height | qualified | Inherited multiplier, font metric minimums; validation/unitless-line-height-review.md |
| A09 | `em` lengths | partial | Native geometry/paint qualified through v5 layout-css-pixel-bounds-v1; original fractional shape-edge failure resolved. Full native1477/1477, all1467 scene pairs reviewed or identical to reviewed baselines; Rust188, JS9/parity, clone1, layout DPR27 and underline DPR27 pass. Focused vector31/33 inspected: original shape failure fixed; em-nested-typography240 and em-font-shorthand-final-size240 retain historical text raster errors. Evidence: validation/em-edge-paint-review.md; original reproducers: validation/em-review.md |
| A10 | `rem` lengths | qualified | Fixed 16px host html root; 12/12 new checks in both renderer profiles. Evidence: validation/rem-review.md |
| A11 | Percentage min/max dimensions | qualified | 24/24 new checks in both profiles; cross-size measurement fix and upstream regression checks. Evidence: validation/percentage-limits-review.md |
| A12 | Letter spacing | partial | 39/39 current checks pass with versioned CSS spacing requirements, including real optional ligatures; fallback/script qualification remains. Evidence: validation/optional-ligatures-review.md |
| A13 | Word spacing | partial | Current word-spacing cases pass; preserved-space capability fixes wrapping/alignment. Broader font/script and format-control qualification remains under T06/T08. Evidence: validation/preserved-space-breaks-review.md |
| A14 | Explicit `<br>` | qualified | Text-only display:block subset; empty/trailing/soft lines and scalar-offset identities. 48/48 new comparisons in both profiles, including strict hidden-paint controls. Evidence: validation/explicit-breaks-review.md |
| A15 | `white-space: nowrap` | qualified | Inherited normal/nowrap, resizing and per-line overflow alignment through checked text-css-nowrap-alignment-v1. All 36 comparisons pass in both renderer profiles; CSS policy remains opt-in. Evidence: validation/nowrap-review.md |
| A16 | `white-space: pre` | qualified | Documented LTR subset, preserved spaces/newlines and default eight-space tabs: 48 non-tab + 33 tab comparisons pass in both profiles. Custom tab-size and broader bidi remain outside this subset. Evidence: validation/pre-review.md, validation/tabs-review.md |
| A17 | `white-space: pre-wrap` | partial | Source-preserving line breaking and line-relative default tabs implemented. 120/120 glyph and 107/120 vector checks pass; all geometry exact. Thirteen vector rasterization cases and broader language qualification remain. Next authoring item: A19. Evidence: validation/prewrap-policy-review.md, validation/wrapped-tabs-review.md |
| A18 | `white-space: pre-line` | partial | Collapsing, preserved newlines, occurrence policy and Chromium Ogham behavior implemented. 72/72 glyph, 70/72 vector; geometry within 0.00390625px. Two mixed-scene vector raster failures remain. Sparse-ink validation now catches missing/misaligned marks. Evidence: validation/preline-review.md |
| A19 | Text transforms | partial | Default/capitalization and inherited tr/az/lt/el casing implemented with source mappings. Combined 168/168 glyph and 166/168 vector; two dense vector raster failures remain. 98 logical references pass. Width/kana deferred: Chrome rejects these values. Next independent item: A20 underline. Evidence: validation/text-transform-review.md, validation/capitalize-review.md, validation/locale-review.md |
| A20 | Underline | partial | Fractional CSS translation snapping fixed: original/reduced/control12/12 native and27/27 DPR pass; full1,477/1,477 with completed native visual review (A09 shape failure now fixed); vector3/12 focused controls remain unqualified. Evidence: validation/underline-fractional-translation-review.md.  CSS and runtime hard-clip drawing integrated; automatic precision capability native glyph 90/90, vector 85/90. Original CJK skip-ink and scale-phase failures fixed. Precision host-state 18/27; affine/group-opacity residuals remain. Nonpositive font metrics qualified in 30/30 Chrome/native cases with native/WASM parity. Absent-post sample rejected by Chrome OTS (reproducer preserved); CSS advance callback fixes DPR controls to 27/27 (original failures retained as reproducers; broader regression qualification pending); backend/performance and broader DPR qualification remain. Evidence: validation/underline-hard-clip-review.md, validation/underline-font-metrics-review.md |
| A21 | Strikethrough | partial | Runtime snapping and version 4 line-baseline contract implemented. Module 98, runtime geometry 7 and 384 reference placements pass. Native phase pixels 32/32 inspected; original resize glyph 12/12, vector 8/12 (four baseline-related vector failures retained). CSS emission implemented; Rust103 and JS7 pass including six native/WASM parity cases. Composition glyph 26/27 plus undecorated control 2/3, all inspected; OpenSans residual occurred without decoration; now fixed by empty-GPOS legacy-kern suppression (OpenSans corpus48/48 inspected; validation/opensans-kerning-review.md). Expanded corpus parity passes. Vector compositions 18/30 inspected. DPR visible text 21/27, stripe-only 27/27; DPR3 inspected, six visible-text failures retained. 20 metric-boundary native/WASM cases pass; 8px/1px DPR matrices each 27/27, partial visual review. Host-state isolated 24/27: opacity overlaps fail targeted visual gate (P05); parity passes. Remaining visual review and broader metric/transform edges remain. Evidence: validation/strikethrough-review.md |
| A22 | Text clipping | partial | overflow: clip/visible/static hidden and CSS-wide resets implemented using responsive LayoutComponent clipping. Initial glyph 12/12 and vector 6/12, all inspected; six vector failures retained. Rust106 and corpus native/WASM parity pass. Nested/edge/decoration/image matrix adds glyph18/18 and vector10/18, all inspected; cumulative glyph30/30 vector16/30. Expanded parity passes. DPR23/27 inspected; four fractional-text failures retained. Bottom-leak gate detects missing clipping in9/9 negative controls. Unclipped DPR control4/9 and sibling-only7/9 isolate independent text residuals; transparent boxes9/9. Static hidden glyph30/30 vector16/30 with all images identical to reviewed clip controls; Rust107 and parity pass. Host-state15/27 restored and19/27 isolated (all isolated inspected); geometry/parity pass. Short-height fixed via internal line box:12/12 inspected, public resize regression passes; full1095 image pairs unchanged, three known failures retained. Axis/clip-margin and pixel residuals remain. Evidence: validation/clipping-review.md |
| A23 | Text ellipsis | partial | Runtime experiment6/12 inspected; 24 expanded Chrome boundary controls inspected; ligature marker placement isolated to retained layout advance versus prefix painting. Too-narrow semantic mismatch and fractional pixels retained. Runtime planner + multi-run LTR grapheme extraction10/10 tests, real Inter metric control and WASM check pass; verified glyph rendering24/24, word-spacing18/18, letter-spacing27/27 (marker spacing corrected; initial24/27 retained), decoration81/81 DPR1/2/3 (initial12/27 marker-decoration reproducers retained) and alignment27/27 inspected; vertical12/12 after shared line-box fix, inspected; DPR69/72 with fractional DPR2 residual also reproduced clip-only6/9; host transforms27/27 isolated and27/27 restored inspected; prior vector18/24 retained, flag-off regression27/27. Baseline native/WASM parity passes; occurrence contract and checked native installer implemented; installed24/24 plus word-spacing18/18 identical to reviewed diagnostics, host tests2/2; public CSS emission implemented, compiler suite passes, public CSS controls24/24 with48/48 images identical to reviewed diagnostics; permanent corpus15/15 inspected; full compiler112/112 and accepted native/WASM corpus parity pass. OpenSans expansion18/24 exposed optional-ligature reshaping; clip-only23/24 confirms narrow cases are ellipsis-specific. Original-glyph retention fix passes11 runtime tests and WASM check; OpenSans24/24 (six changed images inspected), Inter24/24 (all images unchanged). OpenSans spacing23/27 inspected; all four long-line residuals recur clip-only22/27. Permanent OpenSans ligature/media-card corpus9/9 inspected; full module112/112 and expanded corpus parity pass. Retention regression Inter decorations81/81 and word-spacing18/18 (all198 PNGs unchanged); OpenSans legacy-kern fallback fixed: spacing27/27 and decorations27/27, all changed images inspected; full OpenSans corpus48/48 inspected. Evidence also validation/opensans-kerning-review.md. Evidence: validation/ellipsis-review.md |

## Priority 2: selectors, cascade and reusable styles

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| S01 | Attribute selectors | partial | Presence, all six operators, escapes, inert data-* metadata and i implemented; compiler115/115, native/WASM parity, Chrome/native15/15 inspected. Explicit s rejected/deferred: pinned Chrome lacks it. Quoted-comma reference-harness bug fixed; original failures retained. Evidence: validation/attribute-selectors-review.md. Next independent item S02. |
| S02 | Adjacent sibling selector | qualified | Source-DOM element adjacency, comments/whitespace, hidden siblings, chains and specificity; compiler118/118, native/WASM corpus parity, Chrome/native12/12 inspected. Evidence: validation/adjacent-selectors-review.md |
| S03 | General sibling selector | qualified | Later siblings in same source parent, hidden elements, chains and specificity; compiler120/120, native/WASM corpus parity and Chrome/native12/12 inspected. Evidence: validation/general-siblings-review.md |
| S04 | `:first-child` / `:last-child` / `:only-child` | qualified | Source element counting including hidden siblings, escapes and specificity; compiler123/123, native/WASM corpus parity, Chrome/native15/15 inspected. Evidence: validation/structural-selectors-review.md |
| S05 | `:nth-child` / `:nth-last-child` | qualified | An+B formulas, filtered/nested of lists, source counting and max-filter specificity; bounded grammar documented. Compiler127/127, full native/WASM corpus parity, Chrome/native18/18 inspected. Evidence: validation/nth-selectors-review.md |
| S06 | `:not()` | qualified | Strict complex selector lists, nested nth/negation, maximum argument specificity without extra pseudo unit; compiler130/130, native/WASM corpus parity, Chrome/native18/18 inspected including responsive cards. Evidence: validation/negation-selectors-review.md |
| S07 | `:is()` | qualified | Complex/nested lists, maximum surviving specificity and forgiving supported syntax; explicit unsupported-profile errors preserved. Compiler136/136, native/WASM parity, shared Chrome/native24/24 inspected. Evidence: validation/matches-any-selectors-review.md |
| S08 | `:where()` | qualified | Zero argument specificity including nested IDs; shared forgiving-list implementation and explicit profile bounds. Compiler136/136, native/WASM parity, shared Chrome/native24/24 inspected including responsive cards. Evidence: validation/matches-any-selectors-review.md |
| S09 | Custom properties | in progress | Case-sensitive storage, cascade, computed inheritance, short-circuit cycles and bounded expansion integrated. Current module195/195 and JS9/9 native/WASM corpus parity pass; affected Chrome/native258/258 source/image pairs visually accounted for. Last full native baseline1,477/1,477. Broader grammar qualification remains. Evidence: validation/custom-properties-progress.md and validation/function-type-review.md |
| S10 | `var()` and fallbacks | in progress | Literal-name nested/empty fallbacks, missing-value unset and qualified scalar/list/shorthand invalidation implemented, including non-length units, background keyword groups, font prefixes, bare blocks and physical background position/size and function types. Current module195/195, JS9/9 parity and Chrome/native258/258 reviewed. Compatible function argument/logical position grammar, unknown units and documented angle boundary remain. Evidence: validation/custom-properties-progress.md |

## Priority 3: flex, sizing and positioning

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| L01 | Reverse flex directions | qualified | Row/column reverse:60/60 in both renderer profiles, identity/cascade tests and native/WASM parity pass. Corrected full native1582/1582 passes; all1572 scene pairs match reviewed baselines. Normal-wrap policy fixes word splitting; vector-text raster limitations remain separately open. Evidence: validation/normal-wrap-review.md |
| L02 | `order` | qualified (native profile) | Signed integer/cascade/source identity and direction-aware whole-subtree paint policy. Public205, host4, parity9; full native1624/1624 with all1614 images matching reviewed baselines. Focused vector42/45; three text-card raster failures remain explicit renderer limitations. Evidence: validation/order-review.md |
| L03 | `align-self` | qualified (native profile) | Six keywords, version6 occurrence contract and text-baseline channel. Public210 plus lifecycle regression, host5, parity9; full native1723/1723 and compositions9/9 visually accounted for. Vector geometry108/108;30 text pixel failures remain explicit renderer limitations. Evidence: validation/align-self-investigation.md |
| L04 | `align-content` | native qualified | Six keywords, version7 host transport, corrected mixed-size native297/297 visually reviewed. Public216/216, host6, parity9/types pass. Vector geometry297/297;7 text pixel failures documented/reviewed. Full native2029/2029;2019 image pairs match reviewed baselines. Evidence: validation/align-content-investigation.md |
| L05 | `space-around` / `space-evenly` | native qualified | Independent Chromium oracle:96 scenes/288 viewports; three public rejection reproducers preserved. Version8 compiler/host transport implemented. Public220/220 and288 Chromium geometry comparisons plus lifecycle/types pass. Parity9/9, host7/7, native297/297 pixels reviewed. Vector8 text failures open; full native2326/2326 and2316 image pairs match reviewed baselines. Evidence: validation/distributed-spacing-investigation.md |
| L06 | `wrap-reverse` | native qualified | Chromium112 scenes/336 viewports captured; four public rejection reproducers. Reversed overflow fallback fixed; isolated84 Chromium references and98 Taffy tests pass. Public223/223, host8/8 and original/clone336 Chromium references pass. Native351 pixels pass;All15 compositions and336 box cases reviewed. Parity9/9; vector geometry351 passes with14 reviewed text pixel failures; all vector images reviewed. Full native2677/2677; all2667 image pairs source/PNG identical to reviewed baselines. Evidence: validation/wrap-reverse-investigation.md |
| L07 | Auto margins | native-qualified | Existing per-edge Auto unit transport identified; Chromium128 scenes/384 viewports captured across four directions, main/cross auto edges and constrained/wrapped space. Four public rejection reproducers preserved. Typed implementation passes2 public tests; two layout-engine fixes make all396 original/clone Chromium references pass. Taffy100/100 and public226/226 pass; native/WASM builds pass. Focused408 native pixels, parity9/9, host9/9 pass. All408 native images reviewed. Vector geometry408/408 passes;6 text pixel failures reviewed and preserved. All vector images reviewed. Full3084 passes plus1 isolated ENOSPC screenshot retry pass;3075 image pairs match reviewed baselines, inputs unchanged through retry. Native qualified;6 vector text pixel failures remain. Evidence: validation/auto-margin-investigation.md |
| L08 | Independent grow/shrink | native-qualified | Shared Rive weight maps to both factors; optional occurrence policy proposed. Chromium96 scenes/288 viewports captured and four public rejection reproducers preserved. Runtime policy, version9 compiler/manifest/host transport and lifecycle/oracle tests added. Public232/232, parity9/9, host10/10 and types pass; native/WASM builds pass. Focused297/297 native comparisons pass and all reviewed (9 compositions,72 unique box pairs plus216 exact duplicates). Vector geometry297/297 passes; one reviewed text pixel failure retained, all vector images reviewed. Full3382 run invalidated after an unintended publisher rebuild and stopped; joint L08/L09 restart4114/4114 passes;4104 image pairs exactly match reviewed baselines. Evidence: validation/independent-flex-investigation.md |
| L09 | Sub-unit flex factors | native-qualified | Double subtraction of gaps identified in partial-factor distribution. Chromium240 scenes/720 viewports pass isolated engine comparison, including min/max freezing and unequal sibling weights;101 engine tests pass. Nine browser composition references reviewed;244 prospective pixel fixtures and public original/clone oracle prepared. Version10 corrected-distribution capability and compiler/host source added; compile/type checks pass. Public236/236, expanded-corpus tests, parity9/9, host11/11 and types pass. Native/WASM builds pass. Frozen snapshot smoke21/21 passes with reviewed image identity and rejection safeguards checked. Focused732/732 native pixels pass; all732 images reviewed. Vector geometry732/732 passes,731 pixel passes and one reviewed text raster failure retained. Full4114/4114 native passes;4104 image pairs match reviewed baselines. Evidence: validation/partial-flex-investigation.md |
| L10 | Content-derived auto basis | native-qualified | Version11 intrinsic-sizing contract; corrected public228/228, host12/12, native/WASM parity9/9 pass. Corrected focused native336/336 passes; vector239/336 passes with97 preserved pixel failures. Both profiles have all336 pairs visually reviewed. Alignment stress54/profile passes and reviewed. Original full4450 ended4320 pass/130 fail; corrected immutable full4504 passes with zero skips/flaky cases; all4494 scene pairs exactly match reviewed source/images. Evidence: validation/content-auto-investigation.md |
| L11 | Percentage basis in indefinite containers | native-qualified | Version12 capability, public233, host13, native/WASM1847-scene parity and120-scene original/clone960-resize tests pass. Full native5551/5551 passes; all5541 scene pairs visually accounted for. Additional vector original75/factors360/expanded216/threshold108/implicit-main36 pass and are reviewed. Vector text288 geometry passes;200 pixel passes and88 preserved failures, all reviewed. Evidence: validation/indefinite-basis-qualification.md |
| L12 | Content-box sizing | native-qualified | 80-scene Chromium corpus and pre-feature admission receipt: 40 border-box controls accepted / 40 content-box rejects. Runtime occurrence policy now passes 640 Chrome geometry comparisons plus clone/clear lifecycle tests; v13 compiler transport passes public geometry, host14 and native/WASM2017-scene parity. Native initial240/expanded168/cards24 pixel comparisons pass; all432 focused pairs visually reviewed. Audit adds images24 pass/reviewed and edges54 pass/reviewed; expanded vector168 pass/reviewed; frozen full5983/5983 pass with all5973 scene pairs matched to reviewed source/images. Vector cards24 unwaived pixel failures; native profile qualified against frozen v13 snapshot. Border combinations preserved for P01. See validation/content-box-contract.md |
| L13 | Aspect ratio | investigating | Existing wire property524 reaches Taffy;64 prospective shape scenes/192 Chrome captures recorded in aspect-ratio-initial-oracle-v2. Frozen v13 compiler rejects all64 (aspect-ratio-initial-admission/receipt.json). Chrome confirms auto+ratio requires distinct box sizing semantics (12-case/36-capture discriminator); existing-property runtime original/clone resize probe initially40/64; used-main/constraint/intrinsic-column corrections now64/64 (512 original/clone instance-viewports) pass as a regular regression. Taffy106 pass; historical failures preserved. Independent ratio-box policy passes12-case auto discriminator and clear/clone checks; parser3 tests pass. V14 compiler integration passes public608 instance/viewports, host15 and types; public245 tests/52 targets and parity2093 pass; native228/228 geometry/pixels pass; visual review partial; corrected native516/516 pass; historical48 conflict failures preserved. Initial visual192/192, corrected auto36/36, expanded288/288 reviewed (174 direct,114 verified exact-image transfers) including every conflict case; parity2189 passes. Text/composition28 scenes native84/84, vector72/84 pixels (all geometry pass); native84/84 reviewed, all12 vector failures reviewed/unwaived; parity2217 passes. Text clone policy loss fixed: public4 tests/202 scenes/1616 instance-viewports pass; original failures and reinstall control preserved (aspect-ratio-text-clone/receipt.json). Policy independence and full module248 tests pass; rebuilt-probe text native84/84 passes with complete exact-image review transfer, vector same12 failures. Authored-ratio images admitted:24 scenes/public192 instance-viewports/native72 pixels pass;72/72 directly reviewed; parity2241 and full module250 tests pass. Frozen full native5983/5983 passes; all5973 source/image pairs exactly match reviewed baseline (aspect-ratio-v14-full/completion-receipt.json). Current-toolchain initial192/192, auto36/36 and expanded288/288 pass with exact source/image identity and complete audited review transfers (516/516). Session19423 completed successfully. Image flex stress56 scenes/public448 instance-viewports/native168 comparisons pass;168/168 reviewed, parity2297 passes (nine tests). Complete remaining reviews, natural image sizing and full regression review. See validation/aspect-ratio-investigation.md. |
| L13a | Numeric math in aspect ratios | active | Version15 exact integer pair contract, computed-font ceiling and em/rem/typed math implemented. Current ratio evidence637 scenes/1911 native comparisons all pass and reviewed; latest64-scene stress adds512 original/clone instance-viewports. Full module286 plus new stress test, Taffy109, expanded parity11 and host15 pass. Receipts: aspect-ratio-pair-replay-summary.json, aspect-ratio-pair-font-boundary-native/receipt.json, aspect-ratio-pair-stress-native/receipt.json. Full L13/vector limitations remain open; viewport/font-metric math and percentage gaps intentionally unsupported. |
| L14 | Intrinsic image sizing | active | 384 scenes / 1,152 focused Chrome/native comparisons have complete audited visual coverage; 3,072 original/clone instance-viewports pass. Public suite 291/291, expanded parity 11/11, Taffy 110/110. Prior image regressions 240/240 and host/type checks pass. Fixed-runtime full regression and vector replay remain active. See validation/intrinsic-image-investigation.md. |
| L15 | Percentage padding/margins | partial | Native669/669 focused and5983/5983 full checks pass with complete visual audit; public302 and parity/host27 pass. Vector651/669;18 text failures retained. Draw-time diagnostic20/24, not shipping. See validation/vector-text-baseline-investigation.md. |
| L16 | Negative margins | native-qualified | Signed physical px/em/rem/percent margins and auto:432 focused native comparisons reviewed,1152 original/clone updates, public307 and parity11 pass. Full5983 checks pass;5973 scene reviews transferred by exact source/full-PNG identity. Vector423/432;9 text pixel failures remain unwaived. See validation/negative-margins-review.md |
| L17 | Relative positioning | partial | Version17 positioned-paint contract implemented; public compiler317 tests, host17 and native/WASM parity11 pass. Initial native/vector576 each pass and visual audits complete. Native composition192 and discriminator24 pass and reviewed. Full native5983 checks/5973 scene comparisons pass and audited. Composition vector192 geometry/183 pixels; nine text failures directly inspected and unwaived. Interaction96 native/vector comparisons and visual audits pass; final focused evidence audit complete. Nine vector text failures keep this partial. See validation/relative-position-investigation.md and output/playwright/html-to-riv/positioned-v17-receipt.json |
| L18 | Absolute positioning and insets | active | Public v18 candidate:172 scenes/1376 original-clone resize updates;516/516 native geometry/pixel comparisons pass. Compiler325 and JavaScript/parity12 pass. All516 focused views reviewed and audited in both profiles. Vector geometry516/516, pixels498/516 with18 unwaived text failures. Full native baseline retry running after preserved ENOSPC failure. See validation/absolute-position-investigation.md |
| L19 | Stacking order / z-index | qualified | Documented v19 subset:288 focused comparisons/profile and864 live clipping frames pass with complete visual coverage. Corrected full6223 checks/6213 visual pairs and48 regular overlap checks pass and are audited. Compiler331, native/WASM/host checks and2304 stacking resize updates pass. Evidence: output/playwright/html-to-riv/stacking-v19-qualification.json |
| L20 | Overflow clipping | qualified | Documented static version21 subset:303 accepted scenes /909 focused views per profile,2424 original/clone resize updates and192 live margin frames. Full6271 checks/6261 reviewed pairs and regular-suite930 checks/reviewed pairs pass. Signed clip margins, rounded/axis clips, positioned descendants and host lifecycle covered. Scroll semantics excluded; border interactions require P01 extension. Evidence: output/playwright/html-to-riv/overflow-l20-receipt.json |

## Priority 4: painting and assets

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| P01 | Uniform solid borders | qualified | Documented version22 native-renderer scope. Full7180 checks and7170 exact-input/full-image visual transfers pass. Focused static and public original/clone resize matrices complete; native/WASM parity13 and border runtime9 tests pass. Curve antialias differences documented; individual sides and other styles excluded. Evidence: output/playwright/html-to-riv/border-p01-receipt.json. |
| P02 | Individual border sides | native-qualified | Version23 physical sides and 1–4-value lists; solid/none/hidden profile. Corrected paint grouping, 896 focused original/clone frames, full7372 checks and7362 audited visual pairs pass. Module398, host21, JS/native-WASM13 pass. Curve antialias differences retained. Evidence: output/playwright/html-to-riv/border-p02-receipt.json. |
| P03 | Per-corner circular radii | native-qualified | Circular length shorthand/physical longhands; proportional overlap,608 focused resize/clone frames,7444 full checks and7434 audited visual pairs pass. Frozen native/LTR scope; P04/vector/direction exclusions remain. Evidence: output/playwright/html-to-riv/corner-radii-p03-receipt.json. |
| P04 | Elliptical radii | native-qualified | Version24 per-axis px/em/rem/percent shorthand and physical longhands.84 scenes/672 original-clone frames and252 focused views audited; full7696 checks and7686 visual pairs pass. Module410 and expanded native/WASM/host checks pass. Vector36 text failures remain unwaived. Evidence: validation/elliptical-radii-review.md and output/playwright/html-to-riv/elliptical-radii-p04-receipt.json. |
| P05 | Group opacity | active | Public version25 emission, checked host installation and native/WASM parity implemented. Initial/composition, numeric-boundary and overflowing text/image corpora plus decorated text cover88 public scenes/704 original-clone frames/264 reviewed views with artifact audits. Surface-budget boundary controls and16 host affine/modulation controls pass. Frozen corrected full native regression7960/7960; all7950 image pairs covered by audited exact-input/full-image review transfers (7686 baseline+264 opacity). Broader host/image modulation and backend qualification remain. Receipt: output/playwright/html-to-riv/group-opacity-full-r2-validation-receipt.json. See validation/group-opacity-implementation-notes.md. |
| P06 | Linear gradients | qualified | Single-gradient native Metal profile; final 8,052-image replay and focused lifecycle/composition/resource gates pass. Native/WASM parity and 144 retained-renderer performance configurations verified. Explicit extreme-value Chrome precision difference and excluded syntax remain documented. See validation/linear-gradient-qualification.json. |
| P07 | Radial gradients | pending | Public compiler rejects radial syntax. Retained geometry, stop handling, native paint and public integration remain. |
| P08 | Outer box shadows | pending | Blur/spread, alpha, radius and clipping |
| P09 | Inset box shadows | pending | Border/padding edges and clipping |
| P10 | Background images | pending | Position/size/repeat and layer reset semantics |
| P11 | 2D transforms | pending | Composition order, layout footprint and source bounds |
| P12 | Transform origin | pending | Percentage origins and resizing; depends P11 |
| I01 | `object-fit: contain` | pending | Letterboxing and intrinsic ratios |
| I02 | `object-fit: cover` | pending | Cropping and intrinsic ratios |
| I03 | `object-position` | pending | Percent/pixel placement with fit modes |
| I04 | JPEG assets | pending | Decoder safety and consistent embedded rendering |
| I05 | WebP assets | pending | Static images, transparency and decoder limits |
| I06 | Image orientation | pending | Metadata policy and natural dimensions |
| I07 | Color-managed images | pending | Explicit color-space conversion and cross-renderer consistency |
| I08 | SVG assets | pending | Defined static subset, paths/paints and external-reference rejection |
| I09 | Asset deduplication | pending | Stable IDs, deterministic output and shared payload ownership |

## Priority 5: richer text and responsive rules

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| T01 | Mixed text and inline spans | pending | Inline formatting context, source mapping and wrapping |
| T02 | Per-run typography | pending | Color/size/weight baselines, line boxes; depends T01 |
| T03 | Bold/italic semantic tags | pending | Actual font faces and nested resets; depends T01 |
| T04 | Multiple font weights | pending | Expand visual coverage beyond Inter regular |
| T05 | Italic fonts | pending | Face selection, metrics and overhang |
| T06 | Font fallback | pending | Per-glyph/run selection and missing glyph diagnostics |
| T07 | Variable fonts | pending | Axes, instance selection and metrics |
| T08 | Complex-script qualification | pending | Shaping/line breaking with pinned fonts and browser evidence |
| T09 | Bidirectional text | pending | Direction, mixed runs, logical alignment and wrapping |
| R01 | Viewport units | pending | Runtime resize contract versus fixed evaluation |
| R02 | `calc()` | pending | Typed arithmetic, percentages and responsive expression representation |
| R03 | `min()` / `max()` / `clamp()` | pending | Typed expressions and native responsiveness |
| R04 | Media queries | pending | Explicit runtime-responsive versus publish-variant target decision |
| R05 | Container queries | pending | Container dependencies and runtime representation |

## Continuous compiler and validation quality

Run these alongside feature work; they are deliverables, not a replacement for
individual feature qualification.

| ID | Feature | Status | Qualification focus / dependency |
| --- | --- | --- | --- |
| Q01 | Precise source locations | pending | HTML/CSS spans through diagnostics and source maps |
| Q02 | Actionable diagnostics | pending | Unsupported versus malformed versus runtime-unrepresentable input |
| Q03 | Machine-readable capability manifest | pending | Keep implementation, docs and feature statuses consistent |
| Q04 | Determinism expansion | pending | Repeated builds, asset ordering, native/WASM parity |
| Q05 | Malformed-input fuzzing | pending | No panic, bounded work and useful minimized regressions |
| Q06 | Resource-limit boundary coverage | pending | HTML/CSS/assets/nesting limits without excessive test cost |
| Q07 | Performance benchmarks | pending | Small/medium/large representative inputs, time and memory |
| Q08 | Package/version compatibility | pending | API/ABI contracts and independently consumed package smoke test |
| Q09 | Realistic composition corpus | partial | Existing specimens pass macOS glyph lane; vector gaps retained. Pricing/onboarding pending; validation/live-glyph-review.md. |
| Q10 | Deterministic interaction matrix | pending | Multiple supported properties interacting, reproducible failures |

## Progress log

* 2026-09-08: Backlog established after text alignment qualification. Next: A01
  named colors, followed by A02 HSL/HSLA. No new syntax is enabled by this plan.

* 2026-09-08: A01/A02 qualified; accepted gate 158 checks plus Rust/WASM/TS/
  gallery/Clippy. The full named palette and HSL combinations were visually
  inspected at all three widths. Q09 exposed and helped fix intrinsic row text
  sizing; two preserved compositions still fail the pixel gate. See
  validation/named-hsl-review.md and validation/known-gaps.md. Next: A03 inherit.

* 2026-09-08: A03 explicit inheritance qualified. Accepted gate: 176 checks plus
  24 Rust tests and native/WASM, gallery, TypeScript and Clippy checks. Six new
  fixtures visually reviewed at all three widths; validation/inherit-review.md.
  Next: A04 initial, distinguishing CSS initial values from profile reset values.

* 2026-09-08: A04/A05 qualified for the documented profile, including explicit
  unsupported-initial-value diagnostics. Accepted gate 191 checks, 25 Rust tests
  and JS/WASM, gallery, TypeScript/Clippy pass. Fifteen new visual comparisons
  inspected at three widths. Next: A06 solid-color background shorthand.

* 2026-09-08: A06 qualified. Accepted gate 200 checks and 26 Rust tests; native/
  WASM, gallery, TypeScript and Clippy pass. Nine new browser/native comparisons
  visually inspected at three widths. Next: A07 font shorthand and its font/
  line-height reset semantics; A08 is a related prerequisite where needed.

* 2026-09-08: A08 qualified ahead of A07 because font shorthand depends on
  retaining unitless line-height through inheritance. Accepted gate: 209 checks,
  27 Rust tests, native/WASM parity, gallery, TypeScript and Clippy. Nine new
  browser/native pairs inspected at three widths. See
  validation/unitless-line-height-review.md. Next: A07 font shorthand, including
  omitted line-height reset semantics; normal line-height remains unsupported.

* 2026-09-08: A07 explicit-line-height shorthand implemented with reset,
  inheritance and cascade tests. 28 Rust tests and JS/WASM, gallery, TypeScript
  and Clippy pass. Expanded native gate: 216 passed, 2 failed out of 218.
  Mixed-size text exceeds pixel limits at 240/390px; geometry passes. All nine
  new pairs inspected. Failures remain in the default suite, unchanged.
  Equivalent longhands emit byte-identical Rive for the failing scene. Next:
  investigate glyph placement/rasterization alongside Q09, then implement and
  qualify normal line-height and omitted shorthand resets. A07 remains partial.

* 2026-09-08: Corrected text baseline placement using a glyph transform while
  preserving line-box measurement. The two A07 pixel failures now pass, including
  all earlier fixtures: full gate 224 checks, 28 Rust tests, JS/WASM, gallery,
  TypeScript and Clippy pass. Single-glyph and fractional-size regressions added.
  Q09's separate suite has seven failures/two passes after adding a minimized Hg
  rasterization reproducer; browser smoothing is an evidenced separate factor.
  Next: normal line-height and font shorthand defaults; A07 remains partial.

* 2026-09-08: Implemented normal line-height, omitted font-shorthand resets and
  line-height:initial. Runtime baseline trimming plus trailing space preserves
  font-derived line boxes and wrapping without runtime edits. All nine new
  geometry comparisons pass. Expanded gate: 230 pass, 3 pixel failures; 29 Rust
  tests, JS/WASM, gallery, TypeScript and Clippy pass. All new screenshots reviewed.
  Failures remain in the default suite. Browser antialiasing controls pass at
  390px but do not change the reference. Next: resolve text-rasterization fidelity
  alongside Q09; A07 remains partial. See validation/normal-line-height-review.md.

* 2026-09-08: Isolated remaining A07/Q09 text failures using exact native path
  replay in Chromium Canvas. Four Canvas/native comparisons pass; three
  DOM/Canvas comparisons fail. Skia's macOS font-specific smoothing behavior
  and the runtime's unhinted generic-path seam explain the next investigation
  boundary. Added reproducible text-path-control.mjs and a visual review;
  validation/text-rasterization-seam.md records evidence and the native-glyph
  probe to implement next. A07/Q09 remain partial; no external blocker claimed.

* 2026-09-08: Added a native CoreText glyph probe driven by actual runtime glyph
  exports. All 12 smoothed controls pass at 240/390/768px, including colored,
  translucent and fractionally positioned text. Verified existing draw streams
  unchanged and visually reviewed the controls. Main gate remains 230/233;
  A07/Q09 remain partial. Next: transparent glyph-mask composition through the
  real renderer before integrating a compatible glyph-run seam. Evidence:
  validation/native-glyph-review.md.

* 2026-09-08: Transparent CoreText glyph masks now pass 48/48 comparisons when
  composited by real native Metal across three widths and four backgrounds.
  Added a nonzero-on-failure experimental gate, verified it, and inspected
  screenshots. No production renderer or compiler semantics changed. Next:
  integrate an optional glyph-rendering adapter with bounded caches/resources
  and preserve vector compatibility, then rerun A07/Q09. Main gate remains
  230/233; evidence: validation/glyph-mask-review.md.

* 2026-09-08: Added the optional glyph-run API and an opt-in Rust/CoreText
  rasterizer with bounded run masks and explicit resource rejection. All 48
  browser/Metal controls pass; five native tests and 36 render API tests pass,
  as does the pure-runtime boundary. Reviewed all three widths. Largest mask
  in this corpus is 30,704 bytes. Next: caching adapter and live runtime text
  invocation, then full A07/Q09 gates. Main result remains 230/233; no feature
  reclassification. Evidence: validation/rust-glyph-mask-review.md.

* 2026-09-08: Integrated live runtime solid glyph runs with an opt-in factory-owned
  bounded LRU. Full glyph lane 233/233; known-gap specimens 9/9. Compiler Rust,
  native/WASM, JS, TypeScript and gallery gates pass. Ten native tests cover
  raster resources, cache behavior and vector fallback, including an even-odd
  regression. Reviewed all three widths and verified unchanged default vector
  streams. A07/Q09 retain cross-profile limitations. Next: clip/transform/opacity
  qualification, then A09. Evidence: validation/live-glyph-review.md.

* 2026-09-08: Added 27 live host-state comparisons and fixed device baseline
  rounding plus double-applied glyph image opacity. Visual review caught the
  latter despite passing aggregate metrics; added a stricter ink-coverage check
  and a nested-opacity regression. All 27 controls and twelve native tests pass;
  raw mask controls remain 48/48. The state corpus now runs in native-glyphs.
  P05 retains ordinary image/group-opacity work. Next: A09 em lengths. Evidence:
  validation/glyph-state-review.md.

- A09 implements em lengths with parent-relative font size, final computed-font
  length resolution and absolute inherited em line height. Full experimental
  glyph gate: 247/248; new geometry 15/15; host controls 27/27. Fractional shape
  edges fail at 240px despite identical em/px output. Vector new-only: 12/15.
  No tolerances changed. Next: A10 rem contract and implementation; retain A09
  renderer qualification work. Evidence: validation/em-review.md.

- A10 qualifies fixed-root rem lengths, including mixed em/rem shorthands,
  inheritance and authored host scoping. New browser checks 12/12 in each
  renderer profile; full experimental lane 259/260 (preserved A09 edge failure),
  45 Rust, five JS/WASM and 27 host-state controls pass. Next: A11 percentage
  min/max dimensions. Evidence: validation/rem-review.md.

- A11 qualifies native percentage min/max dimensions and fixes flex-basis text
  measurement at clamped cross sizes. New browser checks 24/24 in both profiles;
  full experimental lane 283/284 (A09 edge failure preserved), 48 Rust, five
  JS/WASM, 41 upstream layout, one pinned exact render and 27 host-state checks
  pass. Next: A12 letter spacing. Evidence: validation/percentage-limits-review.md.

- A12 implements signed px/em/rem spacing and computed-value inheritance. Full
  experimental lane 307/311; 24/27 new checks pass in both profiles. Multi-mark
  clusters expose per-glyph versus browser cluster spacing, confirmed in native
  and pinned C++ source. All 49 Rust, five JS/WASM and 27 host-state checks pass.
  Next: A12 cluster-spacing compatibility seam before broader text expansion.
  Evidence: validation/letter-spacing-review.md.

- A12 cluster-spacing experiment: explicit per-font mode fixes all three
  multi-mark failures without changing legacy decode defaults. Experimental full
  gate 310/311, spacing 27/27 in both profiles; 300/303 native scene PNGs unchanged
  and the three corrected images reviewed. All 50 Rust, five JS/WASM, two legacy
  shaping and 27 state controls pass. A12 remains partial: next implement the
  versioned compiler/runtime requirement and retained font-replacement policy,
  then broaden ligature/fallback coverage. Evidence: validation/cluster-spacing-review.md.

- A12 delivery contract now emits versioned runtime requirements in Rust,
  CLI sidecars and JS/WASM. The checked host validates before import and retains
  the selected policy through font replacement, independently per file. Full
  glyph gate 310/311; 52 Rust, five JS/WASM, one checked-host rejection test and
  27 state controls pass. All 303 scene PNGs match the prior opt-in profile.
  Next: true optional-ligature and fallback/cluster qualification for A12.
  Evidence: validation/runtime-requirements-review.md.

- A12 optional ligatures: real Open Sans fixture proves sensitivity (red baseline
  3/12). New CSS spacing capability passes all twelve new comparisons and the
  combined vector subset 39/39; full glyph gate 322/323. All 303 prior native
  PNGs are unchanged; twelve new pairs visually reviewed. All 53 Rust, five
  JS/WASM, checked-host, gallery, TypeScript, Clippy, boundary and 27 state checks
  pass. Older cluster-only semantics remain intact and cannot satisfy the new
  capability. A12 retains fallback/script limitations. Next: investigate A13's
  responsive word-spacing representation. Evidence: validation/optional-ligatures-review.md.

- A13 word spacing maps separator advances through native text runs, preserving
  layout on resize and all source text via plural source-map IDs. Full glyph
  gate 353/356; new geometry 33/33 and pixels 31/33 in both profiles. The two new
  failures expose fixed-width-space wrapping and reproduce with zero word spacing;
  native and pinned C++ share the narrow break classification. All 315 prior
  PNGs unchanged; 33 new pairs reviewed. All 56 Rust, five JS/WASM, checked-host,
  gallery, TypeScript, Clippy, boundary and 27 state controls pass. Next: qualify
  an explicit CSS space-break policy while preserving NBSP and legacy behavior,
  then A14. Evidence: validation/word-spacing-review.md.

- A13 preserved-space break policy fixes the two existing wrap failures and
  qualifies 48 focused comparisons in both renderer profiles. It composes with
  letter spacing, survives font replacement and is requested by versioned output
  metadata. Wider tests found visible U+1680 bypassing glyph admission; missing
  glyphs now reject explicitly and the positive case embeds licensed Noto Sans
  Ogham. Full glyph gate 397/398; only A09 fails. Of 348 prior PNGs, only the two
  corrected cases change. All 42 new pairs reviewed. All 59 Rust, five JS/WASM,
  checked-host, gallery, TypeScript, Clippy, boundary and 27 state controls pass.
  A13 retains T06/T08 and format-control qualification. Next: A14 explicit br.
  Evidence: validation/preserved-space-breaks-review.md.

- A14 qualifies explicit br in text-only block containers, retaining native
  newline reflow and break identity/scalar offsets. Visual review caught hidden
  text that aggregate metrics missed; emitted native visibility flags now hide
  text/images/backgrounds under display:none. Twelve exact blank-image controls
  pass. Full glyph gate 445/446, only A09 fails; new comparisons 48/48 in both
  profiles. All 390 previous PNGs unchanged and all new pairs reviewed. All 62
  Rust, five JS/WASM, checked-host, gallery, TypeScript, Clippy, boundary and 27
  state controls pass. Next: A15 nowrap with explicit breaks preserved.
  Evidence: validation/explicit-breaks-review.md.

- A15 qualified: a checked per-text nowrap alignment policy fixes all four
  narrow center/right failures. All 36 nowrap comparisons pass in glyph/vector
  profiles; full glyph gate 481/482 retains A09 only. All 464 other prior PNGs
  are unchanged. Runtime legacy/resize regression, 63 module Rust tests, five
  JS/WASM tests, checked-host, TypeScript, gallery, Clippy, boundary and 27 state
  controls pass. Next: A16 preserved spaces, newlines and tabs.
  Evidence: validation/nowrap-review.md.

- A16 non-tab subset passes 48/48 glyph and vector comparisons. Inherited pre
  exposed whitespace-only anonymous flex items; those are now discarded while
  block whitespace retains line boxes. Full glyph gate 529/530 retains A09;
  all 474 previous native PNGs are unchanged. All 65 module Rust tests, runtime
  policy regression, JS/WASM, checked-host, TypeScript, gallery, Clippy, boundary
  and 27 state controls pass. A16 stays partial; next is runtime tab stops.
  Evidence: validation/pre-review.md.

- A16 default tabs qualified through checked text-css-tabs-v1, retaining source
  tabs, position-dependent advances, font policy and legacy behavior. All 33 tab
  comparisons pass in both renderers; full glyph gate 562/563 retains A09. All
  522 previous PNGs are unchanged. All 66 module Rust tests, runtime regression,
  JS/WASM, checked-host, TypeScript, gallery, Clippy, boundary and 27 state checks
  pass. Custom tab-size is intentionally unsupported; broader bidi stays with
  T09. Next: A17 pre-wrap with preserved whitespace and soft wrapping.
  Evidence: validation/tabs-review.md.

- A17 direct mapping added with preserved source and explicit wrapped-tab
  rejection. Full glyph run 597/605 retains A09 plus seven pre-wrap failures;
  focused glyph/vector results are 35/42 and 33/42. Visual review caught a
  missing word that aggregate metrics passed; optional browser DOM text regions
  now catch it. Two additional vector-only local rasterization failures remain
  visible alongside A07/Q09. All 555 earlier native PNGs are unchanged. All 67
  module Rust tests, runtime regression, JS/WASM, checked-host, TypeScript,
  gallery, Clippy, boundary and 27 state controls pass. Next: explicit per-text
  runtime configuration and correct hanging/overflow, then wrapped tab stops.
  Evidence: validation/prewrap-review.md.

- A17 prerequisite: version-2 runtime requirements now map occurrence policies
  to actual Text objects, validated before rendering. Nowrap/pre exercise the
  path; version-1 scenes retain their previous interpretation. All 68 module
  tests, parity/host/type/gallery checks and 27 renderer-state controls pass.
  Full glyph comparison remains 597/605 with all 597 image pairs and bounds
  unchanged. A17 remains partial; implement its line-breaking policy next.
  Evidence: validation/text-policy-review.md.

- A17 line-breaking policy implemented with checked text-css-pre-wrap-v1 and
  original source/cluster boundaries. All seven previous semantic failures are
  fixed, including the missing word. Full glyph gate: 634/635, only existing A09.
  Focused results: 72/72 glyph, 64/72 vector, all geometry passing. Eight vector
  rasterization cases remain under unchanged limits. All 555 non-pre-wrap PNGs
  are unchanged; only the seven fixed old cases changed, with 30 new comparisons.
  Three runtime regressions, 68 module tests, parity/host/type/gallery checks,
  Clippy, boundary and 27 state controls pass. Next: wrapped tab stops.
  Evidence: validation/prewrap-policy-review.md.


- A17 wrapped tabs implemented with the explicit text-css-wrapped-tabs-v1
  capability. The original deferred reproducer now compiles. All 48 new glyph
  comparisons pass; vector passes 43/48, retaining five pixel-budget failures.
  Total pre-wrap coverage is 120/120 glyph and 107/120 vector, all geometry exact.
  Full glyph gate: 682/683, only A09. All 627 earlier native PNGs and 72 earlier
  vector PNGs are unchanged. The whitespace-boundary cache avoids repeated
  trailing scans; a 16,384-tab regression passes. Five runtime and 68 module
  tests, corpus parity, host/type/gallery checks, Clippy, boundary and 27 state
  controls pass. A17 remains partial for renderer/language qualification;
  continue authoring expansion with A19 text transforms.
  Evidence: validation/wrapped-tabs-review.md.


### Pre-line qualification

A18 is implemented with 72/72 native-glyph comparisons and 70/72 vector.
Two dense mixed-mode vector cases remain under unchanged budgets. The full
glyph gate is 755/756 with only prior A09; all 675 old native PNGs are identical.
Visual review caught missing Ogham marks that aggregate scores missed. Their
ink and alignment now match pinned Chromium; a sparse-ink presence/position
control preserves the reproducer. See validation/preline-review.md.
Next independent authoring item: A19 text transforms. Keep broader typography
and vector rasterization qualification open.


### Default case transforms

A19 now supports none/uppercase/lowercase and preserves normalized source,
rendered text and scalar-boundary mappings. All 60 glyph comparisons pass;
59/60 vector pass, with one dense mixed-style raster failure. All 747 old native
PNGs are unchanged; full glyph gate 815/816 retains only A09. Unicode 17.0 tables
are pinned and native/WASM parity passes. See validation/text-transform-review.md.
Continue A19 with locale tailoring before moving to A20.


### Capitalization qualification

Capitalize matches the documented untagged Chromium behavior, including simple
BMP titlecase, numeric heads, apostrophes and measured punctuation boundaries.
All 62 independent logical references and 60 native-glyph comparisons pass.
Vector passes 59/60; the dense mixed fixture retains its pixel failure with
exact geometry. All 807 older native PNGs are unchanged; full gate 875/876,
with only A09. See validation/capitalize-review.md. Next: language-tag inheritance
and locale-sensitive casing, followed by width/kana transformations.


### Language casing qualification

A19 now implements inherited/empty-reset lang and contextual tr/az/lt/el upper
and lower casing with exact source scalar offsets. All 36 logical Chromium
references pass; public inheritance/reset/break-offset checks pass. Sixteen new
fixtures pass 48/48 native-glyph and 48/48 vector comparisons with exact geometry;
all new glyph pairs visually reviewed. Full gate 923/924, prior A09 only; all
867 older native images unchanged. See validation/locale-review.md. Width/kana
transformations remain open; validation/width-kana-research.md records Chromium
rejection and initial Firefox support measurements.


### Width/kana reference progress

Established a separate Firefox paint oracle: 10 exact positive comparisons and
2 negative spacing controls, all visually reviewed. A pinned renamed Japanese
font subset is reproducible and public compiler tests enforce reference glyph
coverage. Compiler width/kana operations remain pending; next implement the
226 Unicode width mappings, Appendix G kana mapping, combined grammar and
source offsets, then same-scene native/WASM/browser qualification. See
validation/width-kana-research.md and width-kana-mapping-research.md.


### Chrome remains the qualification target

User clarification: retain Chrome as the compiler comparison target. Width/kana
remain unqualified and explicitly deferred because pinned Chromium rejects the
values. Firefox research and its optional harness are retained as supplemental
evidence only; they are not acceptance gates and do not establish module support.
Do not drop A19 remainder from the backlog or count it completed. Continue A20
underline independently, with Chrome reference pixels and same-scene resize.


### A20 underline reference established

48 pinned-Chrome thickness/skip-ink measurements cover two fonts and four sizes;
scanline intervals and baseline positions are checked by an independent script.
All images inspected. Block/flex propagation probes demonstrate that child none
does not remove ancestor decoration. Runtime line-aware drawing and compiler
transport are still pending; no underline syntax is advertised as supported.
See validation/underline-research.md for the measured rules and integration seam.


### A20 runtime geometry progress

Implemented opt-in glyph/underline-band interception for lines and Bezier curves,
including implicit closing edges. Five runtime tests pass, including real Inter
descender geometry compared with recorded Chrome gap edges. The standard runner
now includes these tests. Runtime drawing, CSS propagation/transport and full
visual qualification remain pending; A20 stays active. Ordinary rendering does
not call the new primitive. See validation/underline-research.md.


### A20 resolved runtime drawing

Resolved solid underlines now rebuild from the actual visual lines and draw
before glyph ink through either vector or native-glyph rendering. A public
compiler/import test resizes the same scene at 768/390/240, checks descender gaps,
checks native glyph drawing retains the decoration, and removes the decoration
to recover the original drawing. Runtime Auto skip-ink follows pinned Chrome
153 eligibility with 481 boundary cases; None and All are checked separately.
This is an opt-in runtime API, not accepted CSS syntax. Propagated decoration
origins, CSS metric resolution, validated manifest transport, trailing-space
semantics and Chrome/native pixel qualification remain required. A20 is active.
See validation/underline-research.md and validation/underline-skip-eligibility-research.md.


### A20 transport and first runtime pixels

Version 3 carries ordered resolved solid underlines per Text object with a new
required capability, bounded finite metrics, unique targets, and host preflight.
The compiler still rejects underline CSS. The runtime control compiles once,
installs explicit test records, resizes at 240/390/768 and compares Chrome against
both real native renderer profiles. All geometry is exact; 12/12 glyph and 8/12
vector controls pass, including undecorated baselines. All four vector failures
are at 240; the undecorated baseline also exceeds mean error. All 24 pairs were
visually reviewed. CSS origin propagation and metric resolution are next.
Evidence: validation/underline-runtime-review.md. A20 remains active.


### A20 CSS emission and origin resolution

Added solid-underline longhands/shorthand, CSS-wide values, propagated origins,
and font/length resolution through the version 3 host seam. Six public compiler
contracts pass. Twenty independent Chrome references distinguish inherited
computed em lengths from child-font auto/percentage/from-font thickness. The
17-fixture corpus passes 51/51 glyph and 49/51 vector; both renderer profiles have
exact layout bounds and all 102 image pairs were inspected. The two vector
from-font failures at 390 remain. See validation/underline-css-review.md.
A20 stays active for broader edge cases and remaining visual failures.


### A20 edge qualification and hard-clip dependency

Added sparse red-pixel checks and 13 edge fixtures. Whitespace, explicit breaks,
visibility and transparency pass. CJK Auto/All expose a two-pixel skip-ink sliver;
pinned Chrome uses hard difference clipping while our segmented fill is
antialiased. This is an internal renderer dependency, not an external blocker.
The edge suite remains 37/39 glyph and 34/39 vector. Source normalization now
accepts mixed-case decoration keywords/currentColor, with a red/green test.
88 Rust tests and full-corpus native/WASM parity pass. See
validation/underline-edge-review.md and validation/underline-clip-research.md.
A20 remains active; do not drop the failing cases or widen thresholds.

### A20 host-transform regression controls

Added the `underline` profile to `validation/glyph-state-control.mjs` and the
native-glyph gate. All 27 comparisons pass geometry and ordinary image/ink limits
but fail the stricter red-decoration check; failures are retained. All pairs
visually reviewed. Existing glyph controls remain 27/27 passing. This extends
validation coverage, not supported CSS syntax or underline qualification.
See `validation/underline-state-review.md`; A20 remains active.

A20 follow-up: isolated host-transform controls (`underline --no-restore`) pass
6/27; the other 21 retain sparse red-ink differences. Runtime geometry now keeps
full stripes and unsnapped clip rectangles for the pending hard-clip backend;
current drawing still uses the geometric fallback. See the updated
`validation/underline-state-review.md`. No new CSS syntax is qualified.

A20 hard-clip transport is implemented and tested: optional renderer operation,
precise recording and text-stream replay, malformed-input rejection, and explicit
unsupported-backend errors. Native coverage and runtime integration remain pending;
no visual qualification or new CSS support is claimed. Evidence:
`validation/underline-clip-research.md`.

A20 native hard-clip step: NativeMetalFrame implements device-space difference
masks. Four mask tests and five exact native pixel controls pass, including
transforms, overlapping exclusions, host clipping and restore. Underline text
still uses its previous fallback; Chrome qualification and runtime integration
remain pending. See `validation/underline-clip-research.md`.

A20 fallback follow-up: actual runtime drawing is tested against immediate and
partial hard-clip refusal, including restoration before fallback. Metal offscreen
canvas forwarding now uses the shared clip implementation; native build and five
exact replay controls pass. Canvas-specific pixels remain unqualified. See
`validation/underline-hard-clip-review.md`.

A20 transform diagnosis: transparent-glyph controls isolate decoration pixels and
pass 23/27, compared with 17/27 with visible glyphs. A seven-case translation sweep
retains a narrow clip-edge transition difference between native and Chrome. These
are supplemental diagnostics, not substitutes for the failing original gates.
Evidence and reproduction commands: `validation/underline-transform-diagnosis.md`.

A20 precision investigation rejected SVG clipping as a hard-clip reference and
reverted a canonical-size-only outline experiment after it regressed the phase
sweep. Existing support is unchanged. Pinned Skia source is now available through
its official GitHub mirror; compare scaler/outline coordinates next. Evidence:
`validation/underline-transform-diagnosis.md`.

### A20 Chrome advance precision diagnosis

Chrome Canvas measurements place q 0.00668px earlier than the runtime in the
preserved scale reproducer. Accounting for this predicts both observed clip
transition intervals. Evidence and raw measurements:
`validation/underline-transform-diagnosis.md` and
`validation/underline-advance-reference.json`. Next test CSS-specific shaping
precision; no production fix or additional passing case is claimed yet.

The A20 precision experiment now passes 7/7 phase cases and 18/27 full host
states. Temporary runtime changes were removed after collecting evidence; next
implement explicit CSS shaping precision with ordinary Rive behavior preserved,
then qualify broad text/resizing regressions. See the diagnosis receipt.

A20 explicit `ShapingPrecision::CssExperimental` runtime API and retained font
asset policy are implemented. Chrome advance / legacy reversal / clone and
replacement regression passes. The probe can opt in for broad qualification;
compiler capability emission remains pending.

P05 now has a font-free native/Chrome overlap reproducer:
`validation/opacity-overlap-reference.mjs`. Native per-draw modulation matches
Chrome per-element alpha within 1 channel value but differs from CSS group
opacity by up to 63. A20 host opacity controls exercise this unresolved group
semantic distinction; retain their failures pending P05. Evidence is in
`validation/underline-transform-diagnosis.md`.

CSS precision full native-glyph regression: 1014/1015 checks, 1004/1005 visual
cases, all 90 underline cases pass. Only the existing A09 em-layout-cascade 240px
failure remains. No regressions versus 966 shared baseline cases; 57 metric
changes and 39 added cases await visual inspection. See the A20 diagnosis receipt.

CSS precision follow-up: all 96 changed/added native-glyph cases are visually
reviewed. Vector underline remains 85/90 with the same five baseline failures
and no regressions. Nine changed vector cases await review; full vector text
qualification and compiler capability wiring remain pending.

Precision qualification found and fixed a large-size wide-glyph overflow before
compiler adoption. The new actual-shaping regression passes and is in run.sh.
The full vector corpus is 987/1015 checks (977/1005 visual cases); all 28 failures
also exist without precision (25 byte-identical, three known CJK None failures).
Post-bound-fix verification and compiler capability wiring remain pending.

Post-overflow-fix verification passes: 89 module tests, scale unit, eight wide
advance samples, 36 font-size samples, 7/7 phase cases with byte-identical PNGs.
Next implement checked compiler precision capability and host installation.

Compiler precision capability is now emitted for Text objects and installed by
the checked host, with legacy manifests preserved and unsupported hosts rejected.
89 module tests, host-contract test, native/WASM corpus parity, TypeScript,
Clippy and automatic 7/7 phase comparisons pass. Full automatic underline lane
is running. Receipt: `validation/precision-policy-review.md`.

A20 DPR receipt: `validation/underline-dpr-review.md`; investigate saved two-extra-pixel auto/all reproducers at DPR1/768 before broadening device-scale coverage.

A20 DPR failure reduced to transparent Latin text near a half-pixel clip edge.
Native advance accumulation in f64 alone does not account for the Chrome
difference; no rendering workaround applied. Reproducer and measurements:
`validation/underline-dpr-review.md` (reduced half-pixel failure).

A20 shaping diagnosis now isolates scalar advance quantization: p/q/j/space
each differ from Chrome by 1/65536px, accounting exactly for the reduced-line
width discrepancy. A failing component control and callback implementation seam
are recorded in `validation/underline-dpr-review.md`; the fix remains pending.

CSS-only advance callback implemented: component widths, reduced repro and DPR27/27 pass; module91 and wide-advance8 pass. Fresh underline/corpus and remaining visual inspections pending; see the DPR receipt.

Post-callback DPR visual review is complete (all nine sheets). Existing underline
90/90 native PNGs are byte-identical to the reviewed baseline. Six JS parity
tests pass; broader corpus and host-state checks are in progress. Receipt:
validation/underline-dpr-review.md.

Post-callback host-state validation remains 18/27 with no new failing cases;
26/27 native images are identical and the one changed passing image was visually
reviewed. Runtime boundary passes. Full native-glyph corpus is still running.

Expanded callback widths pass 165/171: Japanese g at 36px reveals a remaining
scaling-order discrepancy; all Inter/Open Sans checks pass. Focused vector
underline remains 85/90 with the same five failures; six changed cases visually
reviewed. Full native-glyph corpus remains active. See underline-dpr-review.md.

First callback full corpus completed: 1004/1005 visual cases pass, same existing
em-layout-cascade failure; all 1005 native images identical to baseline. Later
pixel-width arithmetic improves font components to 170/171, with a long-string
36px discrepancy still open. Subnormal reciprocal fix is building and awaits
revalidation. Details: validation/underline-dpr-review.md.

A20 safe pixel conversion: module91, wide-advance8 and DPR27/27 pass; font
components170/171 retain a diagnosed accumulation-boundary difference. A20
remains partial with all renderer/DPR/performance gaps retained. Next independent
authoring item: A21 strikethrough. See validation/underline-dpr-review.md.


S09/S10 non-length dimension increment implemented: angle/time/frequency/
resolution/flex units invalidate substituted length slots; valid excluded lengths
retain diagnostics. Chrome182 references and two public Rust regressions pass.
Module190, JS9/native-WASM parity, new6 and surrounding228 native comparisons
pass; all images reviewed or identical to reviewed baselines. Increment qualified. Receipt:
validation/non-length-dimensions-review.md.

S09/S10 background keyword grammar increment implemented: substituted duplicate
repeat/attachment/box groups reset to unset; valid excluded combinations retain
diagnostics. Qualified: Chrome18 references, full module191, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding234 pass. All234
source/image pairs match reviewed baselines. Remaining complex background grammar
is still in progress. Receipt: validation/background-keywords-review.md.

S09/S10 font prefix grammar increment implemented: duplicate style/variant/weight/
width prefixes, excess normal slots and malformed suffixes reset substituted font
values to inherited values. Oblique scalar angles checked; valid excluded styled
fonts retain diagnostics. Qualified: module192, Chrome24 references, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding240 pass. All240
source/image pairs match reviewed baselines. Chrome100grad boundary discrepancy
preserved; broader substitution grammar remains in progress.
Receipt: validation/font-prefix-review.md.

S09/S10 bare-block substitution increment implemented: top-level (), [] and {}
blocks reset ordinary values before normalization; function contents and quoted
strings retain existing behavior. Qualified: module193, Chrome65 references,
JS9/native-WASM corpus parity, TypeScript, new native6 inspected and surrounding246
pass. All246 source/image pairs match reviewed baselines. Broader grammar remains
in progress. Receipt: validation/substitution-block-review.md.

S09/S10 background physical position/size grammar increment implemented: invalid
axis/offset/size/slash groups reset substituted backgrounds; valid excluded forms
retain diagnostics. Qualified: module194, Chrome51 references, JS9/native-WASM
corpus parity, TypeScript, new native6 inspected and surrounding252 pass. All252
source/image pairs match reviewed baselines. Broader grammar remains in progress.
Receipt: validation/background-position-review.md.

S09/S10 function-type increment implemented: known color/image/numeric functions
in incompatible ordinary properties reset after substitution. Compatible and
unknown functions retain existing profile behavior. Qualified: module195, Chrome96,
JS9/native-WASM corpus parity, TypeScript, new native6 inspected and surrounding258
pass. All258 source/image pairs match reviewed baselines. Broader grammar remains
in progress. Receipt: validation/function-type-review.md.

L01 reverse flex directions implemented using existing format/runtime enums.
row-reverse/column-reverse preserve authored source identity and support explicit
inheritance, initial/unset and var(). Module197, JS9/native-WASM parity and
TypeScript pass. Both renderer profiles pass58/60; all sheets inspected or matched
to reviewed images, including corrected visible alignment fixtures. Two narrow
text failures reduce to a pre-existing normal-word-wrap bug (48px word block has
native height48 vs Chrome24); pre-line control passes. L01 remains partial while
that actionable runtime/compiler policy gap is fixed. S09/S10 broader grammar
remains open. Receipt: validation/reverse-flex-review.md.

CSS normal wrapping follow-up: the compiler now requires an explicit
text-css-normal-wrap-v1 capability and css-normal-wrap-v1 Text occurrence policy
(requirements versions2–5). Hosts install Text::set_css_normal_wrap(true) after
validation. Raw Rive behavior is unchanged. Overlong words overflow whole rather
than split by glyph. Module199, JS9/parity, host3 and TypeScript pass. Focused
native69/69 and vector60/69: all reverse-flex60 comparisons now pass in both
profiles; nine new normal-word vector comparisons retain local pixel failures.
All focused images are visually accounted for. Full native regression is running;
L01 remains partial until that result is reviewed. No new word-break/hyphenation
support and no tolerance changes. Receipt: validation/normal-wrap-review.md.

Full normal-wrap regression update: existing preserved Unicode-space fixtures
now expose fitting/alignment regressions (at least fifteen comparisons while the
run is active). The new policy incorrectly shares pre-line space-hanging rules.
Correction and regression rerun are required; focused success does not qualify
the feature. See validation/normal-wrap-review.md for evidence and next action.

Original normal-wrap full run completed1,550/1,582:15 Unicode-space failures
and17 decoration failures. The source space-width correction is staged; its
validation and the decoration glyph-range correction remain pending. Full review
comparison transfers1,508 unchanged pairs, with64 requiring inspection.
Receipt: validation/normal-wrap-review.md.

Corrected normal-wrap native coverage passes471/471: preserved Unicode spaces
retain fitting/alignment width, and collapsed trailing ASCII spaces no longer
extend decoration glyph ranges. All471 source/browser/native image pairs are
identical to reviewed baselines (layout-contract-full, function-type-custom and
normal-wrap); durable visual transfer is complete. Runtime3/3, module199/199,
host3/3 and JS9/9 native/WASM parity pass. Native rebuild succeeded after clearing
only reproducible incremental build cache. Vector471 coverage is running; full
native rerun remains pending. Earlier32 failures are preserved, not overwritten.
Evidence: validation/normal-wrap-review.md and normal-wrap-corrected artifacts.

Corrected vector lane:432/471 pixel checks, all471 geometry checks pass (maximum
0.034px). 303 pairs transfer prior visual review;18 more inspected,150 pending.
All39 pixel failures remain unqualified. Full corrected native regression is
running. Evidence: validation/normal-wrap-review.md.

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

L02 current evidence: module201 and native/WASM builds pass. Native12/18;
six overlap pixel failures despite exact geometry. No-order/painted-parent
controls fail9/9, exposing a broader paint-traversal issue to investigate.
order remains unqualified. Evidence: validation/order-review.md.

L13a numeric boundary update: Chrome literal and math component clamping is now
measured at exact f32 neighbors; fixed-point layout conversion remains unresolved.
See `validation/aspect-ratio-math-research.md` and the retained
`aspect-ratio-math-precision-v3` probe/source receipt. This is investigation evidence,
not additional qualified syntax. The preprecision full module run passed 258 tests;
postprecision binary parity and visual qualification remain pending.


L13a component-clamp correction: literals and math now preserve numeric source
precision and apply Chrome's per-component f32 clamp before division. A new
public regression failed before the fix and exposed both generic serialization
and cached-token rounding errors. All 15 unit and six public aspect-ratio tests
pass. Sixteen responsive row/column fixtures pass Chrome geometry over 128
original/clone instance-viewports (240→390→768→240). Chrome screenshots cover
48 viewports. Receipt: `output/playwright/html-to-riv/aspect-ratio-clamp-oracle/receipt.json`.
Native/WASM rebuild, pixel comparison and visual review are still pending for
this revision; no qualification claim. Chromium's `LayoutRatioFromSizeF` uses a
16-iteration continued-fraction approximation with a 1e-6 error bound and a
truncated fallback. Its small-ratio degeneration and layout saturation require
further work independently of the now-correct component clamp.

Current component-clamp toolchain: native and WASM builds passed; immutable
`aspect-ratio-clamp-toolchain` hashes retained. All48 focused Chrome/native
geometry and Rust Metal pixel comparisons pass without tolerance changes. Visual
review complete (12 direct +36 exact-image transfers), receipt audit passed.
Evidence: `output/playwright/html-to-riv/aspect-ratio-clamp-native/receipt.json`.
Native/WASM parity session53724 is still running; do not count it as passed.
L13a remains active for the separately documented numeric/cascade gaps.


### L13a current approximation stage

The preceding component-clamp snapshot is fully validated for its focused
corpus: 10 JavaScript tests pass, including2327 native/WASM corpus inputs and
six diagnostic cases; all48 native pixels pass with complete visual review.
`aspect-ratio-clamp-native/receipt.json` supersedes earlier running status.

Current source additionally preserves Chrome's lossless 26.6 component-pair
conversion and bounded continued-fraction approximation before scalar emission.
The regression for a nonzero component just above the SizeF clamp failed before
this change and passes afterward. Chrome84-row precision-v4 measurements also
confirm that `1/1048576` retains a ratio while `calc(1/1048576)` degenerates.
15 unit tests, seven public compiler tests and all eight existing aspect-ratio
oracle tests pass after the change. Sixteen additional responsive approximation
fixtures pass128 original/clone instance-viewports. They cover row/column,
pi/e/square-root/golden-ratio literals, the degenerate boundary and lossless tiny
ratios under an explicit220px maximum height. Their48 Chrome screenshots are
retained. Receipt: `output/playwright/html-to-riv/aspect-ratio-approximation-oracle/receipt.json`.

This revision still needs immutable binary parity and native pixel review.
The bounded fixtures do not qualify unbounded layout saturation; Chrome's
33554432px cap remains an open runtime mismatch. Infinity and broader cascade
semantics remain pending. No tolerance changes or full L13a qualification.

Approximation snapshot validation: native/WASM builds passed and immutable
`aspect-ratio-approximation-toolchain` retained. All48 focused Chrome geometry
and native Rust Metal pixel comparisons pass with unchanged tolerances. Visual
review complete:36 direct +12 exact-image transfers; integrity audit passed.
Receipt: `output/playwright/html-to-riv/aspect-ratio-approximation-native/receipt.json`.
Parity76652 and full compiler suite19327 remain confirmed running. Numeric
saturation/infinity and broader cascade qualification remain open.

Approximation terminal results supersede running status: full compiler suite
passed 263 tests across 53 result groups; native/WASM passed all10 JS
tests including2343 corpus inputs and six diagnostic cases. Logs retained in
`aspect-ratio-approximation-native`. All48 focused pixels and visual reviews
remain passed. L13a is still active for unbounded saturation, infinity and
broader cascade semantics; this focused evidence does not qualify those gaps.


L13a nonfinite stage: Chrome25-case probe separates finite literal overflow from
IEEE infinity. Numeric literal1e400 saturates before arithmetic, so
calc(1e400/1e400) yields1; calc(infinity/infinity) yields NaN then zero.
Compiler now preserves that distinction and converts nonnegative oversized
components through the existing finite layout-ratio representation instead of
rejecting them early. New public regression failed before the correction and
passes afterward. All15 unit, eight public compiler and nine existing ratio
oracle tests pass. Twenty-two new bounded row/column fixtures pass176
original/clone instance-viewports; Chrome66 screenshots retained. Evidence:
`output/playwright/html-to-riv/aspect-ratio-nonfinite-oracle/receipt.json`.
Current source needs rebuilt parity and native pixels/visual review; earlier
approximation receipts predate this change. Unbounded layout saturation and
broader cascade semantics remain open. No tolerance changes or full qualification.


Nonfinite snapshot validation complete for its focused bounded corpus: native
and WASM builds passed; full compiler suite 265 tests across 53 groups
passed;10 JS tests include2365 native/WASM corpus inputs and six invalid-math
diagnostic cases. All66 Rust Metal/Chrome geometry and pixel comparisons pass;
visual review30 direct +36 exact-image transfers, integrity audit passed.
Receipt: `output/playwright/html-to-riv/aspect-ratio-nonfinite-native/receipt.json`.

Unbounded saturation remains a reproduced runtime bug. Eight Chrome fixtures
are retained in `aspect-ratio-saturation-oracle`: four explicit oversized-length
controls receive compiler diagnostics; four admitted ratio fixtures fail at all
three widths. For width120 and ratio1e-6, Chrome height33554432 versus native
120000000; with an infinite denominator native reaches4026531840. The mirrored
large-ratio width also exceeds Chrome's cap. `native-geometry.json` preserves
actual frozen-toolchain bounds and diagnostics. These are geometry reproducers,
not pixel-qualified cases. Do not treat bounded green tests as covering them.


L13a saturation correction in progress: the four admitted unbounded fixtures now
match Chrome through32 original/clone instance-viewports. The new public
regression failed before the runtime change; all11 aspect-ratio oracle tests
pass afterward. Taffy now has an optional ratio-derived dimension limit, applied
at initial size transfer, column flex basis and post-flex cross sizing. CSS ratio
occurrences enable Chrome's33554432 limit; ordinary styles default to None.
The occurrence flag survives cloning. This is arithmetic saturation, not an
authored max-size constraint. All107 Taffy unit tests pass including opt-in and
authored-size preservation controls. The added optional field increases Style
by8 bytes; documented size assertions updated after the expected failure.
Evidence: `output/playwright/html-to-riv/aspect-ratio-saturation-oracle/implementation-receipt.json`.
Native runtime toolchain rebuild is underway; fresh parity/pixels, expanded
saturation compositions and broader cascade qualification remain pending.
Previous frozen toolchains retain the failing runtime and are not evidence for
this correction. No tolerance change or full qualification claim.

Saturation stress expansion:24 scenes/72 Chrome captures cover row/column,
both box-sizing modes, padding, auto+ratio, max constraints and flex shrink.
Public lifecycle test68400 is running. First native replay stopped before any
rendering because the rebuilt probe lacked native-glyph-controls; failure retained
in `aspect-ratio-saturation-native`. Correct probe build85538 is running. These
are pending validation gates, not pixel failures or passing evidence.


Saturation stress public test completed:24 scenes/192 original-clone
instance-viewports passed. Initial rebuilt probes/renderers lacked required
native-glyph-controls/native-metal flags; both failed before rendering and are
preserved as build-profile failures. Correct immutable toolchain is now
`aspect-ratio-saturation-toolchain-v3`, with explicit build commands retained.
Native replay of four initial scenes reports12/12 passing; expanded replay94783
is running with53/72 comparisons observed passing. All84 still need visual
review; full regression and parity remain pending. Use v3, not the earlier
incomplete build profiles, for subsequent runtime validation.


L13a scalar cascade correction: Chrome12-case probe confirms malformed scalar
ratios supplied via var() reset to auto instead of reviving an earlier ratio.
A new public regression failed before the correction and now passes. Invalidation
only classifies the bounded scalar grammar; valid unsupported sin(1) and typed
calc(1px/1px) retain diagnostics. Escaped CSS-wide fallback remains supported.
15 unit,10 public compiler and51 custom-property tests pass. The initially
incorrect escaped-keyword test used --ratio:inherit, which inherits the custom
property; corrected to var(--missing, escaped-inherit) to test substitution.
Logs: `output/playwright/html-to-riv/aspect-ratio-cascade-probe/receipt.json`.
Eighteen responsive fixtures are being captured. Their public geometry/parity/
pixels and review remain pending. Typed invalid math such as calc(1px) still
needs typed classification; it is an explicit open cascade gap.
Prior saturation2393-input expanded parity passed before this source change.


Scalar cascade gates completed:18 scenes/144 original-clone instance-viewports,
54 Chrome/native pixel comparisons, full visual review (3 direct +51 exact-image
transfers), audit passed. Native/WASM10 tests include2411 corpus inputs and six
unsupported diagnostic cases; full compiler suite270 tests/53 groups passed.
Frozen `aspect-ratio-cascade-toolchain` and `aspect-ratio-cascade-native/receipt.json`
retain sources, binaries and logs. This does not yet qualify typed arithmetic.

New18-row Chrome probe in `aspect-ratio-typed-math-probe` distinguishes invalid
unit-bearing expressions from valid cancellation. calc(1px), calc(1px+1px) with
proper sum whitespace, min(1px,2px), mixed-type sums and px/s all reset to auto
when substituted. px/px, percent/percent, s/s and px*s/px/s produce numbers;
em/px additionally depends on computed font size. Typed invalidation must track
dimensional exponents rather than reject every expression containing units.
These are retained browser measurements, not implemented/qualified syntax.


Typed ratio arithmetic implementation stage: quantities now carry six dimensional
exponents through products/division; sums and comparisons require matching types,
and final components must be dimensionless. Absolute length, angle, time,
frequency, resolution units and percentages use canonical conversion. Numeric
prefix parsing retains double precision including escaped units. Typed zero is
not dimensionless. Known typed-invalid var() substitutions now reset to unset;
valid cancellation is compiled. Relative/context-dependent units still diagnose.
This only extends aspect-ratio math, not other property grammars. Existing term
and nesting limits remain. Source: https://www.w3.org/TR/css-values-4/#calc-type-checking
and the adjacent absolute-unit definitions; Chrome18-row probe independently
checks invalidation and cancellation.
New unit regression failed before the implementation. All16 unit,11 public
compiler and51 custom-property tests pass. Receipt:
`output/playwright/html-to-riv/aspect-ratio-typed-math-probe/implementation-receipt.json`.
Thirty-four responsive fixtures are being captured. Public oracle/parity/native
pixels/visual review, conversion-edge tests and relative units remain pending;
extreme converted dimension overflow requires investigation. Earlier frozen
cascade receipts predate this evaluator change. No qualification claim.


Typed-math focused validation:34 scenes/272 original-clone instance-viewports,
102/102 Rust Metal pixels + Chrome geometry pass. Visual review complete:9 direct
+93 exact-image transfers; integrity audit passed. Full compiler suite273
tests/53 groups passed. Immutable `aspect-ratio-typed-toolchain`; evidence
`aspect-ratio-typed-native/receipt.json`. Parity20278 remains running over2445 inputs.

Conversion-edge probe20 rows: ordinary absolute-unit conversions and escaped
units match geometry, but four extreme cases do not. Frozen native comparison
in `aspect-ratio-unit-conversion-probe/native-comparison.json` preserves compiler
inputs, RIVs and bounds: huge in/in gives native0 versus Chrome120px; huge in/px
and px/in lose finite ratios96 and1/96; tiny ms/ms gives native120 versus Chrome0.
The first probe-script invocation had a JS escape error before browser execution;
corrected script and successful capture are retained. Investigate Chrome's unit
conversion/evaluation order; do not guess an overflow cutoff or drop these cases.
Relative units and full unit-conversion visual qualification also remain open.


Unit-conversion order correction: exact Chromium153.0.8010.12 source reveals
CSSParserToken clamps numeric tokens to +/-f32::MAX while retaining double
precision inside that range. Earlier docs claiming f64::MAX were incorrect;
equal huge literals did not distinguish the limits. New Chrome probes for
1e40/1e39 and 1e40-1e39 do distinguish them. Numeric token clamping corrected.
Typed division uses multiplication by a reciprocal, unlike eagerly simplified
scalar division. Preserving that order fixes tiny ms/ms underflow. Four recorded
extreme conversion mismatches now pass new unit/public equivalence regressions.
17 unit,11 existing public ratio,14 existing ratio-oracle and51 custom-property
tests pass; the additional public extreme-conversion test passes separately.
Tag-matched source URLs/hashes, red/green logs and24 Chrome probe rows retained in
`aspect-ratio-unit-conversion-probe-v2/implementation-receipt.json`.
Fresh binary parity and geometry/pixel/visual gates remain pending; prior typed
snapshot parity completed10 tests/2445 inputs before this correction. Relative
units and full L13 qualification remain open. No tolerance changes.


### L13a fractional source-size regression (current)

The frozen unit-order snapshot passes all 10 JavaScript parity tests over 2469
inputs, but responsive validation is **not qualified**: 71/72 native comparisons
pass; case 19 at 768px derives 43084.797px height versus Chrome 43084.5px.
The public original/clone oracle and full module run both fail on that geometry.
Logs and the failed comparison are preserved in
`output/playwright/html-to-riv/aspect-ratio-unit-order-native/receipt.json`.

Nine minimal Chrome fixtures distinguish numeric conversion from source-size
quantization: literal `1 / 96` reproduces the same failure as typed math, while
an exact 1/64px source dimension removes it. Chrome truncates both 448.8px and
448.808px to 448.796875px before ratio transfer. The public regression exercises
original and clone instances through 240/390/768/240 resizing. A CSS-only runtime
source-quantization correction is under test; ordinary Rive ratio behavior is
unchanged. See `validation/aspect-ratio-rounding-cases.json` and
`tests/assets/aspect-ratio-rounding-oracle.json`. Native pixels, visual review,
broader fractional constraints, and final current-source qualification remain
pending. Tolerances are unchanged.

The first correction build stopped on disk exhaustion, before test execution.
`cargo clean -p nuxie-html-to-riv` removed 10.8GiB of regenerable build artifacts;
all frozen toolchains and validation outputs were preserved. The corrected
rebuild is tracked separately from this infrastructure failure.

The candidate source-quantization correction now passes all16 public aspect-ratio
oracle tests, including the previously failing unit-order fixture and9 new minimal
rounding scenes with original/clone resizing. Evidence: `aspect-ratio-rounding-oracle/public-green.log`.
Fresh runtime probe build and pixel qualification are pending.

Fractional-source focused replay now passes33 scenes/99 Chrome geometry and Rust Metal
pixel comparisons, with264 original/clone instance-viewports passing public checks.
Visual review:18 directly inspected comparisons +81 exact-image transfers; both
receipt audits pass. Frozen `aspect-ratio-rounding-toolchain` includes the rebuilt
runtime probe; compiler/WASM artifacts are unchanged from the unit-order snapshot.
Receipts: `aspect-ratio-unit-order-native-v2/receipt.json` and
`aspect-ratio-rounding-native/receipt.json`. Full module validation passed277 tests/53 groups; expanded2478-input
parity passed all10 tests. The first expanded parity attempt failed with
ENOENT during concurrent publisher relinking; its log is preserved and the rerun
started after the publisher hash matched the frozen artifact and passed. No tolerance changes.


L13a fractional stress:40 Chrome scenes/120 viewports now cover row/column,
border/content boxes, bare/auto ratios, fractional padding, fixed/grow/shrink,
and min/max constraints. The previous frozen rounding toolchain fails60/120
comparisons (geometry); the public original/clone stress test is also red.
Evidence: `aspect-ratio-rounding-stress-native/receipt.json` and preserved
`public-red.log`. Earlier33-scene receipts remain valid for their frozen scope.

Diagnosis separates individually truncated padding edges from container gap
conversion before flex allocation. For a768px row with100px sibling and8.808px
gap, Chrome uses gap8.796875px and derives659.203125px main size; transferring
1/96 gives63283.5px. Truncating the float-computed659.192px only after allocation
instead gives63282px. This requires correcting upstream layout inputs, not
widening tolerance or adjusting the ratio. A per-edge padding correction is
under test; container gap precision and wider fixed-point semantics remain open.
Full-module277/53 and parity10/2478 passes predate this padding candidate.

Per-edge padding candidate result: all16 previous oracle tests still pass;
the stress oracle now fails16 scenes/48 distinct scene-viewports, all grow/shrink.
Fixed-padding and min/max cases now pass. Current remaining error is1.5px after
ratio amplification of container gap precision. Candidate source remains unqualified
until gap policy and fresh native/WASM/pixel/visual checks complete. Evidence:
`aspect-ratio-rounding-stress-native/padding-candidate.json`. No live jobs remain.

Gap correction: resolved flex gaps now truncate to1/64px before allocation,
including percentage-gap re-resolution. The runtime enables this explicit Taffy
policy throughout a solve tree containing opted-in CSS aspect ratios; ordinary
Rive trees retain float gaps. Original/clone behavior derives from current solve
styles. All17 public aspect-ratio oracle tests pass, including all40 stress scenes;
all108 isolated Taffy library tests pass, including an opt-in gap allocation test.
Logs: `aspect-ratio-rounding-stress-native/gap-public-green.log` and
`gap-taffy-pass.log`. Fresh native pixels/visual review and expanded parity pending.

Gap focused validation complete:40 scenes/120 Chrome geometry + Rust Metal pixels
pass;320 original/clone instance-viewports pass. Visual review33 direct +87 exact
image transfers, audit passed. Full module278 tests/53 groups,
Taffy108 tests, and parity10 tests/2518 inputs all pass. Frozen
`aspect-ratio-gap-toolchain`; receipt `aspect-ratio-gap-native/receipt.json`.
Nested pixel-gap layouts and full L13/relative-unit qualification remain pending.
Percentage gaps remain intentionally unsupported by the compiler; the engine
percentage re-resolution path is not a compiler support claim.
No live jobs remain; tolerances are unchanged.


Nested pixel-gap qualification:16 new scenes/48 Chrome viewports. Frozen gap
snapshot fails21/48 comparisons. Ancestor per-edge pixel-padding correction
now leaves only4 depth-two column wrapper-width failures (12 viewports); child
ratio dimensions match. All17 earlier oracle tests still pass. Percentage gaps
remain intentionally rejected, including custom properties/fallbacks (9 public
admission controls pass). Current ancestor-padding source is not pixel-qualified.

An isolated experiment omitted post-flex known height during intrinsic column
cross measurement. It fixed extreme wrapper widths but ordinary wrappers became
20px versus Chrome23.59375px (16px marker +7.59375px padding). The experiment was
reverted; the complete fix must account for intrinsic content minimum and ratio
sizing together. Logs/reproducers: `aspect-ratio-nested-gap-native/receipt.json`,
`ancestor-padding-candidate.json`, and `intrinsic-height-experiment.log`.
No live jobs remain. Full L13/relative-unit qualification remains open.


Nested-column candidate now passes all18 public ratio oracle tests. CSS intrinsic
inline measurement excludes post-flex allocated height, while ratio-derived
intrinsic contributions retain their min-content width floor. A separate
`css_intrinsic_sizing` Taffy policy is enabled by the runtime for CSS ratio solve
trees, independent of `quantize_gap`; defaults preserve ordinary Rive behavior.
Ancestor pixel padding is truncated per edge before available-size allocation.
All108 engine tests pass. Explicit policy storage adds8 bytes to Style on this
build (String560, Arc528); size assertions updated, no visual tolerance changes.
Evidence: `aspect-ratio-nested-gap-native/intrinsic-policy-pass.log` and
`taffy-pass.log`. Fresh native pixels, visual review, full module, and expanded
2534-input parity remain pending. CSS sizing context:
https://www.w3.org/TR/css-sizing-4/ (Chrome captures remain acceptance evidence).

Nested intrinsic sizing focused validation complete:16 scenes/48 native comparisons
and128 original-clone instance-viewports pass; all48 comparisons directly visually
inspected, audit passed. Full module280 tests/53 groups, engine108,
parity11 tests/2534 inputs (plus9 percentage-gap diagnostic controls) all pass.
Frozen `aspect-ratio-nested-toolchain`; receipt `aspect-ratio-nested-native/receipt.json`.
Full L13 current-toolchain replay, relative-unit math and broader intrinsic cases
remain pending; no tolerance changes or live jobs.


Current nested-toolchain text/image regression:108 scenes/324 Chrome geometry and
native Rust Metal pixel comparisons pass. Review coverage complete:318 exact
compiler-input/full-image transfers used +6 direct comparisons (including both
changed768px reverse-column image pairs). All source and target image hashes,
full compiler inputs including assets, prior direct-review receipts and sheets
were verified. Thin image-edge diffs pass the unchanged image tolerance.
Receipts: `aspect-ratio-current-text-native/receipt.json`,
`aspect-ratio-current-image-native/receipt.json`, and
`aspect-ratio-current-image-stress-native/receipt.json`.

Repeatable cross-run review tool:
`python3 validation/transfer-visual-review.py PREVIOUS_REPLAY CURRENT_REPLAY`
then the same command with `--audit`. The previous run must have complete audited
direct/within-run review. Changed source or PNG pairs remain unreviewed. Fresh
inspection may overlap transferred pairs; coverage counts deduplicate identities.
Five integrity guard checks passed, including changed CSS despite identical pixels,
stale PNG hashes and a tampered receipt; original evidence was not modified.
The existing within-run tool continues to record actual direct inspections.
Full current-snapshot shape/math and vector-fallback qualification remain open.


Current native L13 cohort replay complete:14 shape/math cohorts,393 scenes/1179
comparisons pass; all1179 have audited exact full compiler-input and PNG-pair
review transfers. Combined with current text/images108 scenes/324 comparisons
and nested16 scenes/48 directly reviewed comparisons, the frozen nested toolchain
now passes517 scenes/1551 native comparisons with complete visual coverage.
Summary: `aspect-ratio-current-native-summary.json`; per-cohort receipts in
`aspect-ratio-current-*-native`. Source/asset/image identity is checked before
any review transfer. No live jobs remain and no tolerances changed.

This closes the pending current native shape/math cohort replay. Relative-unit
ratio math, vector fallback qualification and broader intrinsic-sizing semantics
remain open. It is not a rerun of every legacy compiler-gallery scene; the two
clone-only minimal reproducers are covered separately by public runtime tests.


L13a font-relative ratio math implemented, validation in progress: em/rem
expressions inside calc/min/max/clamp now evaluate using the final computed
font-size, or the profile's fixed16px host root respectively. Numeric literals,
typed dimensional cancellation, and existing clamping rules remain unchanged.
The font pass runs before ratio math regardless of declaration order. Inherited
ratios retain their computed value; custom-property expressions use the consumer's
font. Recognized invalid dimensional math resets through substitution as before.
Viewport/container, font-metric and line-height units remain unsupported.

All19 public ratio oracle tests pass, including24 new Chrome scenes/192 original
and clone instance-viewports. Fresh publisher/WASM builds are in progress;
full module, expanded parity, native pixels and visual review remain pending.
Fractional/extreme font contexts need further qualification. Evidence:
`aspect-ratio-font-math-oracle/implementation-receipt.json`. Earlier frozen
snapshots predate this compiler feature; no tolerance changes.

Font-relative ratio focused validation passes24 scenes/72 native comparisons
and192 original-clone instance-viewports. Visual review30 direct +42 exact-image
transfers, audit passed. Full compiler suite283 tests/53 groups and
parity11 tests/2558 inputs pass with fresh native/WASM artifacts. Frozen
`aspect-ratio-font-math-toolchain`; receipt `aspect-ratio-font-math-native/receipt.json`.
Fractional/extreme font contexts remain the next qualification step. No live jobs
or tolerance changes; broader L13 completion remains open.


### Font-boundary ratio precision investigation

The 32-scene font-boundary Chrome corpus remains unqualified. The computed font
size ceiling correction (10000px, applied before inheritance and em resolution)
passes its public regression. After aligning the public oracle with the native
probe's `layout_bounds()` API, the remaining failures are one scene at390/768px
on both original and clone: height errors0.25/0.5px. The 16 public ratio tests and
19 of20 ratio-oracle tests pass; the boundary oracle correctly remains red.

An f64-transfer experiment did not fix these failures and was removed. Evidence
is preserved in `output/playwright/html-to-riv/aspect-ratio-font-boundary-precision-investigation/`:
candidate source, failing test logs, restored-arithmetic log, and numerical
comparison. The computed pair1/15999 becomes a rounded f32 scalar in the current
compiler. Exact pair arithmetic matches all three Chrome heights; using f64
arithmetic on the already-rounded scalar does not. This proves the next
implementation direction, not runtime qualification. Preserve the pair through
an explicit runtime contract and cloned occurrence policy, then repeat public,
native/WASM, native-pixel and visual gates. Existing tolerances stay unchanged.

Full module test run with `--no-fail-fast` completed: 284 passed, 1 failed across 53 result groups. Only `aspect_ratio_font_boundaries_match_chrome_after_clone_and_resize` failed. Full log and counts are preserved in the precision-investigation receipt.


### Exact computed ratio pair implemented (version15, qualification pending)

The compiler retains the computed integer ratio pair, normalized by GCD, in a
version15 requirement with `layout-css-aspect-ratio-pair-v1`. Every ratio target
must have two positive signed-32-bit components; incomplete/invalid pairs and
mismatched versions/capabilities are rejected. Version14 remains readable with
its original scalar semantics. Older hosts cannot accept the new capability.
Occurrence-local runtime style and clones retain the pair. Opted-in flex ratio
transfers use26.6 integer multiply/divide with truncation and signed-32-bit
saturation, following the retained Chrome `LayoutUnit::MulDiv` source.

All20 public Chrome ratio-oracle groups now pass, including the font-boundary
original/clone resize regression. Full compiler286 tests/53 groups, Taffy109,
and JS type checking pass. The first compile borrow error and expected Taffy
style-size assertion failure were corrected; original logs remain preserved.
Evidence: `output/playwright/html-to-riv/aspect-ratio-pair-implementation/receipt.json`.
Fresh native/WASM parity, native pixels, visual review and broader pair-boundary
stress remain pending; no full L13 qualification or tolerance change.


### Version15 focused font-boundary qualification

Frozen `aspect-ratio-pair-toolchain` passes all32 font-boundary scenes:96/96
Chrome geometry and Rust Metal pixel comparisons, plus256 original/clone
instance-viewports in the public oracle. All96 views are reviewed and audited:
21 directly inspected on seven three-width sheets,75 verified exact-image
transfers. Viewport screenshots verify visible content; separate geometry
assertions cover the full heights of extremely tall boxes. No tolerance changes.
Native/WASM parity completes11/11 tests, including the additional32 boundary
inputs. Full module286 tests/53 groups, Taffy109 and type checks remain passed.
Receipt: `aspect-ratio-pair-font-boundary-native/receipt.json`.

Broader qualification is running from `aspect-ratio-pair-replay-plan.json`:
20 cohorts/543 scenes/1629 native comparisons, including the full28-scene text
cohort as a separate group. The older2-scene text repro remains an additional
cohort and does not substitute for full text coverage. Pixel/geometry failures
and changed review images must be resolved or explicitly retained; none of these
in-progress broader runs is claimed qualified. Pair-boundary stress, vector
fallback and full L13 backlog completion remain open.


Replay-plan correction: the historical `aspect-ratio-text-native` directory held
only six completed comparisons from a partially failed old oracle, not a complete
two-scene corpus. Replaying that oracle reaches an intentionally unsupported
ellipsis combination (missing display:block). Frozen pre-pair font-math and
current pair compilers reject the exact same input with identical diagnostics;
`aspect-ratio-pair-text-native/historical-admission-control.json` preserves this
control. The failed run is retained and is not counted as pixel qualification.
The valid broader replay scope is19 cohorts/541 scenes/1623 comparisons, including
the corrected full28-scene text oracle. Full text84/84 passes with audited exact
review transfer from the original v2 direct-review receipt. The initial transfer
attempt against the historical v3 receipt schema and its successful follow-up
are both retained. Remaining replay groups are still running.


Version15 broader native replay completed:19 valid cohorts/541 scenes/1623
comparisons all pass Chrome geometry and Rust Metal pixel thresholds. Audited
exact-source/image review transfers cover1559 comparisons;64 changed image
comparisons (8 image,56 image-stress) require fresh visual inspection. These are
passing numeric comparisons but are not yet visually qualified. All four replay
groups are terminal; no replay process remains running. Summary:
`output/playwright/html-to-riv/aspect-ratio-pair-replay-summary.json`.


Version15 broader visual qualification complete: all1623 native comparisons
(541 scenes across19 cohorts) pass geometry, pixel thresholds and audited review
coverage. Twenty newly inspected three-width image sheets close the64 changed
views; coverage combines direct/within-run review and prior exact-source/image
transfers without double counting. Thin image-edge differences were inspected
and retained within unchanged existing thresholds, not described as pixel exact.
Together with the separately reviewed32-scene/96-view font-boundary corpus,
current version15 evidence covers573 scenes/1719 comparisons. Full module286,
Taffy109, native/WASM11 tests and type checks pass. No processes remain from the
broader replay. Full L13 and vector fallback remain open.

Next:64 exact-pair stress fixtures cross eight numeric ratios with row/column,
border/content box and bare/auto ratio semantics, including fractional padding
and gaps. Chrome capture and public/native/visual qualification are pending;
these fixtures are not included in the completed counts above.


Exact-pair stress qualification completed:64 scenes/192 native geometry/pixel
comparisons and512 original/clone instance-viewports pass. Visual review60 direct
+132 verified identical pairs is complete and audited. Expanded native/WASM
parity11 tests passes. The separate host-loading suite still asserted version14;
its failure was preserved, then expectations and malformed-pair/missing-capability/
legacy-version14 controls were updated:15 host tests pass. Compiler/runtime code
and frozen binaries did not change during this qualification. Receipt:
`aspect-ratio-pair-stress-native/receipt.json`. Current native ratio evidence now
covers637 scenes/1911 comparisons, all reviewed; full L13/vector limitations
remain open. No live validation processes remain.


Full version15 legacy native regression launched with isolated report/config and
frozen binaries:1979 source scenes plus deterministic matrix and gate controls.
Live session17229, `aspect-ratio-pair-full/run.py`; inspect its terminal result
before qualification. Early cases pass; no full-run result is claimed.

L14 investigation started independently while that full run executes. Chrome32
natural-image cases/96 captures complete; frozen version15 rejects all32 with
preserved diagnostics. Runtime image measurement exposes decoded dimensions,
but independently picks axes; replaced-element ratio/constraints need probing.
See validation/intrinsic-image-investigation.md. No language admission change yet.


Existing-runtime intrinsic image probe:32 initial scenes now match Chrome for all
256 original/clone instance-viewports after restoring automatic units AND Hug
scale type on automatic axes, setting intrinsic measurement, and selecting the
natural/content-box or authored ratio with version15 exact-pair policy. Initial
units-only experiment left Fixed scale type and failed240/256; it is preserved
but does not establish a valid automatic-size limitation. Probe output and source
snapshots: `intrinsic-image-runtime-investigation/receipt.json`. Public compiler
admission remains unchanged. Next add non-square asset coverage, then implement
the compiler mapping and complete parity/native-pixel/visual gates. No engine
change was needed for this initial32-scene seam test.


L14 non-square/runtime and public mapping:64 wide/tall cases pass512 runtime
clone/resize comparisons. Compiler natural-image admission now passes all96
square/wide/tall scenes (768 instance-viewports), preserving auto dimensions and
using embedded PNG metadata. Intrinsic measurement uses existing Rive fields;
version15 exact-ratio capability carries natural or authored preferred ratio.
Public pre-implementation rejection and green logs are preserved. Full suite
exposed a stack overflow in the depth/resource-limit test because a PNG decoder
local enlarged the recursive compiler frame. Metadata reading moved to the asset
module; the failing resource-limit test now passes without stack-limit changes.
Full module rerun is live (`/tmp/intrinsic-image-full-public-v2.log`); fresh binary
parity/native-pixel/visual gates remain pending. Evidence:
`intrinsic-image-implementation/receipt.json`. The still-running frozen full
L13 regression predates L14 and cannot qualify these compiler changes.


L14 initial native qualification advances: rebuilt/frozen intrinsic-image
publisher and WASM pass all288 native geometry/pixel comparisons (96 square,
wide and tall scenes). All768 public clone/resize instance-viewports pass; full
module288 tests/53 groups and type check pass. Combined native/WASM11 + host15
suite passes26 tests. The native probe/renderer are unchanged from the exact-pair
snapshot. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-implementation/receipt.json. Visual review remains pending.
32 image/text card compositions were added and Chrome capture started; those
compositions are not included in the passing counts. The full pre-L14 frozen
regression remains independently live in session17229.


L14 card composition stage:32 image/text card scenes pass96 native geometry/pixel
comparisons and256 original/clone instance-viewports. Initial image suite plus
cards now totals128 scenes/384 native comparisons and1024 instance-viewports.
No new compiler/runtime change was needed for cards. Their32 inputs were added
to native/WASM parity, which is running in session44031 (log
`/tmp/intrinsic-image-card-parity.log`). Initial image visual inspection covers
108/288 views after inspecting18 square-asset three-width sheets and auditing
54 direct+54 exact-image transfers;180 initial image views and96 card views still
need review. Thin sampling-edge differences remain within unchanged thresholds.
The full pre-image frozen regression remains live in session17229. No full L14
qualification claim. Receipts: intrinsic-image-native/receipt.json and
intrinsic-image-card-native/receipt.json.


L14 validation follow-up: the expanded native/WASM corpus, including all32 card
inputs, passes all11 JavaScript tests (0 failures); preserved log:
`output/playwright/html-to-riv/intrinsic-image-card-native/native-wasm-parity.log`.
Four tall-image column/content-box sheets were directly inspected at all three
widths: natural ratio, bare authored ratio, constrained height and max-height.
Initial-suite review now covers126/288 views, with162 still pending; card review
remains pending for96 views. Visible overflow, padding and sibling placement
match Chrome; sampling-edge differences retain existing tolerances. This is
partial L14 qualification; broader flex stress and the current compiler full
regression remain required. Chrome is the sole browser acceptance reference;
existing Firefox research is historical supplemental evidence, not further work
or a release gate.


L14 tall-image visual review is complete:12 additional three-width sheets were
inspected (row border/content-box and column border-box); the four column
content-box sheets were recorded in the preceding batch. The audited initial
suite now covers198/288 comparisons through direct review and exact image-pair
transfers, leaving90 wide-image comparisons. Card review remains pending.
Natural-ratio overflow, authored-ratio sizing, constraints, padding and sibling
placement match Chrome visually; no tolerance changes were made.


L14 initial visual gate completed:96 square/wide/tall image scenes pass288
Chrome/native geometry and pixel comparisons, all now covered by audited visual
review (162 direct views and126 exact image-pair transfers). All20 selected
wide-image sheets were inspected at240/390/768, including fixed-height overflow,
authored and natural ratios, both box models, row/column layout and constraints.
Thin sampling edges remain within unchanged tolerances. Card sheets are generated
(28 sheets covering96 views) but not yet inspected. Expanded native/WASM parity
passes. Broader flex stress and full current-compiler regression remain required.
The separate pre-L14 aspect-ratio full run completed with5983 tests passing;
its5973 scene-pair baseline image audit is underway, so this does not yet claim
visual qualification of that full run or full L14 qualification.


L13 full frozen aspect-ratio-pair regression is now qualified:5983 tests pass,
including5973 scene pairs whose exact HTML/CSS and complete browser/native PNG
bytes match the reviewed v14 baseline. Rehashed source receipts, comparison data
and image files; shared harness/reset hashes match and current harness hashes
remain unchanged. There are0 remaining image comparisons. Receipt:
`output/playwright/html-to-riv/aspect-ratio-pair-full/receipt.json`.
This full run predates L14 compiler changes; intrinsic-image full qualification
remains separate and pending.


L14 card visual review advances: all14 selected wide-image card sheets inspected
at240/390/768; audited coverage is48/96 comparisons (42 direct and
6 exact image-pair transfers). Tall-image card review remains pending.
Text wrapping, card extents, note placement, image sizing and viewport overflow
match Chrome; glyph antialiasing and image sampling differences retain existing
tolerances. Full legacy regression using the frozen intrinsic-image compiler is
now running in session11402, with independent output and source snapshots under
`output/playwright/html-to-riv/intrinsic-image-full`; no result claimed yet.


L14 card visual gate completed:32 scenes/96 Chrome-native comparisons all audited
(84 directly viewed comparisons on28 sheets plus12 exact image-pair transfers).
Combined initial and card suites now pass384 native geometry/pixel comparisons
with complete visual coverage, and1024 public original/clone instance-viewports.
Native/WASM parity passes for these inputs. Tall natural/fluid viewport overflow,
fixed-height images, authored ratios, text wrapping and note placement match
Chrome. No tolerance changes. Full intrinsic-image legacy regression remains
running in session11402. Added128 flex stress fixtures spanning wide/tall assets,
both box models, four directions and natural/stretch/grow/shrink/wrap/
wrap-reverse/conflicting min-max/percentage-height combinations; Chrome capture
is running in session38355. These new cases are not yet qualified or counted as
passing.


L14 flex stress exposes an unwaived regression:384 comparisons complete,360
pass and24 fail. All failures are8 row/row-reverse align-items:stretch scenes
(two PNG ratios, two box models, three widths). The public compiler original/
clone resize test also fails and is checked in with the128-scene Chrome oracle.
For wide border-box row stretch, Chrome produces629px image width from the
stretched260px border-box height; native retains104px, moving following siblings
by525px. Taffy flex-basis passes a known cross dimension into ContentSize leaf
measurement, which suppresses aspect_ratio; the native intrinsic leaf returns
natural image width. This is the current diagnostic lead, not a qualified fix.
Preserved receipt and public/native red logs:
`output/playwright/html-to-riv/intrinsic-image-flex-native/receipt.json`.
No failures or tolerances waived. The expanded128 inputs are also included in the
native/WASM parity corpus; that expanded run remains pending. Full frozen legacy
regression continues separately in session11402.


L14 stretch diagnosis: first candidate marked intrinsic CSS leaves as replaced
and retained ratio transfer in leaf ContentSize sizing. The public128-scene
regression still fails;109 existing Taffy tests pass. This is an unqualified
experiment, not a fix. Before-source snapshots and red/engine logs are preserved
in `output/playwright/html-to-riv/intrinsic-image-stretch-investigation`.
Temporary INTRINSIC_TRACE instrumentation is compiling/running in session2445
to determine whether the actual measurement node reaches that path; remove the
instrumentation after diagnosis. Frozen legacy regression11402 remains separate.


L14 stretch fix candidate now passes the128-scene public compiler oracle through
1024 original/clone instance-viewports. The image ratio lives on the layout
wrapper, not a ratio-bearing measured leaf; the ineffective leaf experiment and
trace were removed. CSS intrinsic wrappers are marked replaced, and single-line
row stretch supplies ratio-derived flex basis and automatic minimum width.
All110 Taffy tests pass, including a new wrapper regression (Chrome629px expected
width with24px horizontal/18px vertical padding) and a non-replaced control.
Broader24-test ratio oracle suite is running in session18335. Pixel replay with
rebuilt runtime artifacts remains pending; prior24 native failures stay preserved
and unwaived. The full frozen pre-fix regression11402 cannot qualify this runtime
change. Evidence: intrinsic-image-stretch-investigation/receipt.json.


L14 stretch candidate passes all24 public aspect-ratio oracle tests, including
initial/nonsquare images, cards, flex stress, text, numeric boundaries and cloned
scene resizing. Fresh publisher/probe/Metal renderer/WASM build and freeze is
running in session97527 under intrinsic-image-stretch-toolchain. Added128 stretch
edge cases covering four directions, both assets/box models, explicit flex basis,
zero automatic minimum, min/max width, max-height, authored ratio and auto
cross/main margins; Chrome capture40400 is running. These edge cases have not
yet passed public/native validation. Prior red pixel evidence remains preserved.


L14 stretch edge public gate passes:128 additional scenes/1024 original-clone
instance-viewports match Chrome (flex bases, zero minimum, min/max limits,
authored ratio, automatic margins in all four directions). Native replay is
running for original flex stress46564 and edges10549; expanded native/WASM
parity10336 and full public suite57044 also run. Fresh immutable artifacts are
available in intrinsic-image-stretch-toolchain. Updated the investigation page
with current status, separating historical pending/rejection statements. No
pending native result counted as passing.


L14 stretch candidate full public suite passes291 tests/53 groups. Expanded
native/WASM corpus passes all11 tests including both128-scene flex/edge cohorts.
Corrected flex native replay passes384/384 geometry and pixel comparisons.
Four corrected wide/tall row/reverse-row stretch sheets directly inspected at
all three widths cover24/384 views after audited exact image-pair transfers,
including every previously failing stretch view. The remaining360 flex views
still need review; edge native replay10549 is running. No tolerances changed.
Receipts: intrinsic-image-stretch-investigation/receipt.json and
intrinsic-image-flex-native-v2/receipt.json.


L14 stretch edge native gate passes384/384 comparisons, matching its128-scene
public resize/clone gate. Both stress cohorts now pass768 native comparisons.
Original flex review covers53/384 views after six additional wide-row sheets;
331 flex views and384 edge views remain unreviewed. Edge review sheets are being
generated (74 selected sheets, session69303). Replaying128 previously reviewed
initial/card scenes against the fixed runtime in intrinsic-image-regression-native
(session recorded by the tool) to verify no visual regression; its merged oracle
retains exact original Chrome captures and source inputs. No tolerance changes.


L14 visual review continues: ten additional wide border-box reversed-row/column
sheets inspected at240/390/768. Audited corrected flex coverage is now109/384,
with275 remaining. Wrapping, growth/stretch, constraints, sibling offsets
and viewport overflow agree with Chrome; thresholds unchanged. All74 edge review
sheets are generated but remain unreviewed. Regression replay12429 continues.


L14 fixed-runtime initial/card regression is fully accounted for:384/384 native
geometry/pixel comparisons pass, and all384 full compiler inputs and browser/
native PNG pairs match their independently audited initial/card review receipts.
Transferred288 initial and96 card views, with0 remaining. Receipt:
`output/playwright/html-to-riv/intrinsic-image-regression-native/receipt.json`.
Corrected flex review advances to142/384 after six column/reversed-column sheets;
242 flex views and384 edge views remain. Growth/stretch overflow above the
viewport, reverse wrapping, min/max constraints and percentage heights visually
match Chrome. Existing thresholds unchanged. Full fixed-runtime legacy gate
remains required; pre-fix full regression11402 is independent.


L14 full fixed-runtime legacy regression started in session44428 with frozen
intrinsic-image-stretch-toolchain, isolated output/source hashes and fixture
snapshot under intrinsic-image-stretch-full. The pre-fix run11402 is still live
and preserved separately. Eight additional flex review sheets inspected; audited
coverage now166/384 with218 remaining. Content-box row and reversed
column constraints, wrapping, percentages, sibling positioning and overflow
match Chrome. Edge384-view inspection remains pending.


L14 corrected-flex review advances to202/384 audited comparisons after twelve
wide content-box reversed-row/column sheets. The wide-completion check caught
remaining row grow/shrink views; wide review is not complete. These and tall
views total182 remaining. The inspection receipt itself passed its hash/count
audit. Edge visual review remains pending; full fixed-runtime regression44428
was re-polled and remains live. No tolerance changes.


L14 wide-image flex review is now complete, verified by an empty wide subset of
remaining comparisons. Eight additional grow/shrink and tall-row sheets reviewed
at240/390/768 bring audited corrected-flex coverage to225/384, leaving
159 tall-image views. Tall wrapping overflow, natural sizes and conflicting
constraints match Chrome, with sampling differences under unchanged thresholds.
Edge review and full fixed-runtime regression remain pending.


L14 corrected-flex visual review: ten additional tall-image sheets inspected at
240/390/768 (percentage height, reverse row natural/wrap/reverse wrap/conflicting
limits, and column natural/stretch/grow/wrap). Audited coverage is now 275/384,
with 109 comparisons remaining. Chrome/native placement, sizing and overflow
agree; image sampling seams remain under unchanged thresholds. The authoritative
receipt is `output/playwright/html-to-riv/intrinsic-image-flex-native-v2/visual-inspection.json`.
Both full regression handles (11402 and 44428) were confirmed live this turn;
neither is counted as complete. Stretch-edge visual inspection remains pending.
Chrome is the sole browser reference; Firefox is not a qualification gate.


L14 review update: twelve further tall-image column and content-box wrapping
sheets inspected at all three widths. Corrected-flex audited visual coverage is
326/384; 58 comparisons remain. No new layout discrepancy or tolerance change.
The initial L14 pre-stretch full regression completed with exit 0: 5,983 tests,
5,973 scene comparisons. All 5,973 HTML/CSS and browser/native PNG pairs match
the qualified exact-ratio baseline; source receipt and image hashes were audited,
and all six shared harness/reset hashes match both the prior run and current
files. Evidence: `output/playwright/html-to-riv/intrinsic-image-full/receipt.json`,
`baseline-comparison.json`, and `visual-inspection.json`. This qualifies that
frozen pre-stretch toolchain only. Current stretch full run 44428 is still active;
focused stretch-edge visual review also remains pending.


L14 corrected-flex visual review is complete: 384/384 audited comparisons,
zero remaining. The final twenty tall content-box constraint/wrapping and both
box-model grow/shrink sheets were inspected at 240/390/768. The narrow shrink
case correctly reaches its padding floor in both Chrome and native output;
wide grow/shrink, reverse placement and overflow also agree. Image sampling
seams remain under unchanged thresholds. `intrinsic-image-flex-native-v2/receipt.json`
and `visual-inspection.json` record completion. This completes focused flex
review only; stretch-edge review and current fixed-runtime full regression
remain pending. Full regression session 44428 was confirmed live this turn.


L14 stretch-edge visual review started: twelve wide border-box row/reversed-row
sheets inspected at 240/390/768, covering flex basis, width/height constraints,
authored aspect ratio and cross-axis automatic margin. Audited direct inspection
and identical-image transfers cover 66/384 comparisons, leaving 318. Geometry,
overflow, sibling positions and image appearance agree with Chrome under the
unchanged thresholds. Evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Current full stretch regression session 44428 was confirmed
live; no restart or completion claim. Corrected-flex coverage remains complete.


L14 stretch-edge review advances to 143/384 audited comparisons (241 remain).
Twelve wide-image column/reversed-column and content-box row sheets were viewed
at 240/390/768. Basis, width limits, authored ratio, automatic margins and overflow
match Chrome. Some fractional image boundaries show thin horizontal sampling
seams within the unchanged pixel thresholds; no layout discrepancy was found.
Updated evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Full stretch regression session 44428 was confirmed live.


L14 stretch-edge wide-image visual review is complete, checked by an empty wide
subset in the audited remaining list. Twelve final wide content-box sheets and
four tall row constraint sheets inspected at 240/390/768 bring total coverage to
212/384, leaving 172 tall-image comparisons. Sizing, padding, sibling placement
and overflow agree with Chrome; thin image sampling seams remain within unchanged
thresholds. Evidence is in `intrinsic-image-stretch-edge-native/visual-inspection.json`
and `receipt.json`. Full stretch regression session 44428 remains live as polled.


L14 stretch-edge tall-row review: eight additional authored-ratio, automatic-margin
and reversed-row basis/constraint sheets inspected at 240/390/768. Audited coverage
is 250/384, leaving 134 comparisons. Chrome/native sizing, padding, overflow and
sibling alignment agree; thin image sampling seams remain within unchanged gates.
Evidence: `intrinsic-image-stretch-edge-native/visual-inspection.json` and
`receipt.json`. Full stretch regression handle 44428 was confirmed live this turn.


L14 tall-column stretch-edge visual review advances to 307/384 audited comparisons,
77 remaining. Eight additional sheets inspected at 240/390/768 cover basis,
width limits, authored ratio, automatic margin and reversed-column constraints.
Chrome/native sizing, overflow and sibling placement agree; a thin fractional
ratio boundary sampling seam stays within unchanged gates. Receipts updated in
`intrinsic-image-stretch-edge-native`. Full regression session 44428 is live,
with the log advancing through test 3148; no completion claim.


L14 stretch-edge review advances to 340/384 audited comparisons, with 44 remaining.
Eight tall reverse-column and content-box row sheets inspected at 240/390/768
cover authored ratio, auto margin, basis, minimum width and maximum height.
Layout and overflow match Chrome; the thin fractional column-ratio image boundary
seam remains within unchanged thresholds. Review and aggregate receipts updated
in `intrinsic-image-stretch-edge-native`. Full run 44428 was confirmed live.


L14 stretch-edge visual review complete: 384/384 audited comparisons, none remaining.
The final ten tall content-box and automatic-main-margin sheets were inspected
at 240/390/768. Layout, overflow, image sizing and free-space allocation agree
with Chrome; existing thin sampling seams remain under unchanged thresholds.
The focused initial/card, corrected-flex and edge cohorts now total 1,152 passing
Chrome/native comparisons with complete audited visual coverage (including exact
source/image transfers for initial/cards). SUPPORT.md and the edge receipt were
updated. This does not complete L14 qualification: full fixed-runtime regression
session 44428 is still live, and broader residual validation remains as documented.


L14 fixed-runtime prior-image regression completed: 72 image and 168 image-stress
comparisons pass geometry and native pixels. All 240 complete compiler inputs
and browser/native PNG pairs are identical to the reviewed exact-ratio baseline.
The baseline direct/within-run and cross-run review union was audited before
transfer. Evidence: `intrinsic-image-stretch-prior-images/audit.py`, both prior
image native output directories' `visual-transfer-union.json` and `receipt.json`.
No new inspection was claimed for identical outputs. Host/TypeScript session
56307 and full regression 44428 remain pending as last confirmed live.

L14 host follow-up: session 56307 completed with exit 0 for both the JavaScript
host suite and strict TypeScript check. Frozen WASM hash was verified before the
run. Logs and receipt: `intrinsic-image-stretch-host`. Full native regression
44428 remains the outstanding running validation.


L14 vector fallback validation launched independently under session 69222, using
NUXIE_NATIVE_GLYPHS=0 and the frozen stretch toolchain. It replays all 384 focused
initial/card, flex and edge scenes (1,152 comparisons) against saved Chrome
geometry and screenshots. Source probe confirms that zero disables native glyph
handling. Separate output and launch receipt: `intrinsic-image-stretch-vector`.
No vector pass or visual qualification is claimed yet; text failures will remain
unwaived. Full native session 44428 is still live, with log progress past 4,100.


L15 independent preparation: added 64 percentage-spacing Chrome fixtures covering
all flex directions, both box models, definite/intrinsic hosts and mixed/auto
spacing. Capture session 62216 targets 192 views. Frozen L14 rejection is preserved
in `percentage-spacing-investigation` (exit 1, unsupported-value). Scope and next
validation steps are in `validation/percentage-spacing-investigation.md`.
No percentage-spacing support is claimed; L14 native/vector runs continue.


L14 vector initial/card replay completed: all 384 geometry checks pass; 303 pixel
comparisons pass and 81 text-card pixel comparisons fail, unwaived. The first
failing 390px wide border-box natural card was visually inspected: image and
layout match, with text raster differences. All failures and that inspected
PNG hash triple are preserved in `intrinsic-image-stretch-vector-initial-cards/receipt.json`.
Remaining vector cohorts and full native regression are still running (69222,
44428). L15 Chrome capture completed: 64 scenes / 192 views, Chrome153.0.8010.12;
its receipt records the oracle hash. This is reference evidence, not L15 support.


L15 source investigation found that runtime root margins lose their unit when
there is no layout parent; padding units are preserved. Added eight top-level
spacing fixtures and launched 24 Chrome references (19118). The initial 64 scenes
remain unchanged. Documented compiler retained-value and substitution-validation
seams in `validation/percentage-spacing-investigation.md`. Runtime root-wrapper
behavior still needs testing; no implementation or percentage support claimed.
Native full and vector replay sessions 44428/69222 were confirmed live this turn.


L14 vector replay is terminal: 1,152/1,152 geometry checks pass, 1,071 pixel
comparisons pass, and 81 text-card pixel failures remain unwaived. All 768 flex
and edge comparisons have complete audited exact-input/PNG visual transfers
from the native-glyph cohorts. Initial/card vector visual review remains partial.
Evidence: `intrinsic-image-stretch-vector/receipt.json` and both cohort receipts.
Full native 44428 remains live, past test 5,200. L15 root capture completed with
8 scenes / 24 Chrome views; oracle hash recorded in its investigation receipt.

L14 current qualification update: the frozen stretch-runtime full native regression completed with 5,983/5,983 checks passing and 5,973/5,973 Rust Metal scene comparisons. Exact HTML/CSS and full Chrome/native PNG bytes match the reviewed pre-fix baseline for all 5,973 comparisons; all six shared harness/reset hashes were verified unchanged. Evidence: `output/playwright/html-to-riv/intrinsic-image-stretch-full/{receipt,visual-inspection,baseline-comparison}.json`. Vector fallback remains partial: 1,152 geometry passes, 1,071 pixel passes and 81 unwaived text-card pixel failures. Audited image-only visual coverage is now 1,056 comparisons (288 initial plus 768 flex/edges); two failing text-card views are inspected and 94 card views remain for inspection. Chrome is the sole browser reference; Firefox is not a qualification gate. No tolerance changes.

L15 experimental implementation now retains nonnegative physical padding/margin percentages (finite 0–10000%) through publication and runtime resizing, including mixed lengths and automatic margins. Two public syntax/cascade tests and two Chrome-oracle tests pass: 72 scenes, 576 original-and-clone viewport updates at unchanged 0.1px tolerance. Authored-root cases pass without runtime changes. Negative margins, logical properties and Grid remain excluded. Native/WASM parity, Rust Metal pixel/visual checks, expanded compositions and full regression remain pending; this is not a qualification claim. Evidence: `output/playwright/html-to-riv/percentage-spacing-investigation/implementation-receipt.json`.

L15 rendered validation found eight unwaived pixel failures among 216 initial/root Chrome comparisons; all 216 geometry checks pass. The directly inspected 390px mixed-spacing fixture places the inner rectangle one pixel lower in native output: Chrome y=28.484375 versus native y=28.5, implicating fractional percentage resolution. Failure PNGs and receipts are preserved in `percentage-spacing-validation` and the `percentage-spacing-*-native` directories. Initial native/WASM parity passed ten tests; a missing frozen probe symlink caused the remaining test to fail, and that isolated test passes after the symlink repair. Added 32 Chrome composition fixtures covering nesting, wrapping/reverse wrapping, limits, intrinsic images, text, cascade and intrinsic hosts; captured 96 reference views. Removed the obsolete margin:10% rejection assertion; full public regression is rerunning. No tolerance changes; L15 remains experimental.

L15 composition test is red: 176 coordinate mismatches across the 32-scene original/clone resize corpus. Examples: row wrap tail.y=109.171844 versus Chrome225.20313 at240; intrinsic row card.width=60 versus Chrome72.34375. Full public regression stopped with exit101 at this new test (27 sibling oracle tests passed). Preserve `percentage-spacing-validation/public-full-v2.log`; investigate wrapped-line free-space allocation and cyclic percentage padding/intrinsic sizing separately from the eight pixel-rounding failures. Qualification remains incomplete.

L15 diagnosis correction: wrapped-line mismatches came from the public oracle helper omitting installation of the emitted align-content policy. Added the same installation as the native probe; wrapping errors disappear. The frozen composition native replay confirms 96 comparisons with 12 geometry failures, all intrinsic-container views, and 11 pixel failures. Corrected public test retains 144 intrinsic coordinate mismatches. A separate column padding reference-axis candidate is under test; no runtime fix is qualified yet. Evidence: `percentage-spacing-validation/composition-align-content.log` and `percentage-spacing-composition-native/replay.json`.

The L15 column padding-axis experiment completed with the same 144 intrinsic coordinate mismatches and was reverted exactly to its saved source. No runtime change retained from this experiment. The next target remains intrinsic percentage sizing; the public helper align-content correction is retained.

L15 reduced reproducer: 16 scenes / 48 Chrome views / 128 original-and-clone viewport updates isolate control, padding, margin, combined spacing, fixed parent, disabled shrink, content-box and nested cases in both directions. Baseline reports 184 coordinate mismatches; row padding loss reproduces even with a fixed parent and disabled shrink. `percentage-spacing-validation/minimal-investigation.json` records the evidence. A parent-width-preservation measurement experiment is running against all four percentage-spacing oracle tests; not yet qualified.

L15 parent-width experiment is terminal: initial/root oracles still pass; minimal coordinate mismatches fall from184 to64 and composition mismatches from144 to48. The candidate remains unqualified with failing tests preserved; investigate the remaining column cases and require full regression before qualification. Log: `percentage-spacing-validation/parent-width-experiment.log`.

L15 geometry candidate passes all four Chrome-oracle tests: 120 scenes / 960 original-and-clone viewport updates at unchanged 0.1px tolerance. Preserving parent width for auto-width row measurement and remeasuring content-sized columns after inline width resolves removes the focused geometry failures. This is a broad candidate, not a qualified runtime change: full compiler regression is running (97293), and dedicated runtime checks, fresh parity, native pixel reruns and visual review remain pending. Evidence: `percentage-spacing-validation/geometry-candidate-receipt.json`. Existing frozen pixel failures remain unwaived.

L15 geometry candidate regression: full compiler297 tests across54 groups pass with0 ignored; Taffy111 unit tests pass, including a new direct row/column percentage-padding intrinsic measurement test. The initial workspace-level Taffy command could not run a non-workspace package; the manifest-path invocation completed successfully. Fresh candidate binaries/WASM are building in `percentage-spacing-geometry-toolchain` (38491). The runtime candidate remains unqualified for pixels; previous pixel failures are unwaived. Receipt: `percentage-spacing-validation/geometry-candidate-receipt.json`.

L15 fresh geometry-candidate validation is terminal: native/WASM11 tests pass; native360/360 geometry comparisons pass,347 pixel comparisons pass and13 pixel failures remain unwaived (initial6, root2, composition5, minimal0). Focused rounding references add27 scenes/81 Chrome views; all agree with truncating each percentage edge to1/64px before summation. A dedicated public rounding test uses stricter0.001px tolerance without changing standard gates. Evidence: `percentage-spacing-validation/geometry-native-receipt.json`.

L15 rounding experiment passes all five oracle groups:147 scenes /1176 original-and-clone viewport updates, including27 rounding scenes at stricter0.001px tolerance. Independent edge truncation removes the numerical rounding reproducer. This is still a generic diagnostic implementation: explicit CSS opt-in, requirements contract, measurement scope/performance review, new parity, native pixel/visual checks and full regression remain required. Evidence: `percentage-spacing-validation/rounding-candidate-receipt.json`. The13 frozen-candidate pixel failures are not yet claimed fixed.

L15 explicit opt-in implemented: requirements version16 adds `layout-css-percentage-spacing-v1` and unique `layout_percentage_spacing` occurrence IDs. Rust validation checks capability/version/target consistency; compiler emission, native probe installation, cloned runtime state and JavaScript API types are updated. Taffy defaults remain off; the policy enables measurement and percentage edge precision across the opted-in solve tree. All five focused oracle groups still pass (147 scenes/1176 instance-viewports), and Taffy111 tests pass. Contract tests are running; fresh parity/pixels/full regression and opt-out/performance coverage remain pending. Evidence: `percentage-spacing-validation/opt-in-receipt.json`.

L15 version16 syntax/cascade/contract tests completed:3/3 pass, including missing-capability, downgrade, duplicate/missing occurrence and mixed aspect-ratio contract checks. Fresh parity and pixel qualification remain pending.

L15 opt-out coverage passes: the same Taffy tree toggles false/true/false, restoring float geometry when CSS precision is disabled. All112 Taffy tests pass. Added version16 host tests for unsupported-capability rejection before stream output, invalid/duplicate/missing targets and compile-once viewport replay. Updated TypeScript version/capability assertions; strict typecheck passes. Initial frozen build failed on a missing probe import and is preserved; corrected build runs in `percentage-spacing-v16-r2-toolchain` (57092). Host tests, fresh parity and441 native comparison reruns await the corrected toolchain.

L15 version16 focused gates are green:441/441 Chrome/native geometry and real Rust Metal pixel comparisons pass;27/27 native/WASM and host-contract tests pass. All13 earlier geometry-candidate pixel failures are fixed in this fresh run, with thresholds unchanged and red artifacts retained. Visual inspection has begun:6 directly inspected views plus3 exact within-run image transfers account for9/441 comparisons,432 remaining. Added reusable `validation/make-replay-sheets.py` to prepare unscaled comparison sheets separately from review recording. Full public suite19571 remains running; full native regression, visual completion and nested measurement cost/coverage audits remain outstanding. Receipt: `percentage-spacing-validation/v16-receipt.json`.

L15 full public suite is green:299 tests across54 groups,0 ignored. Full frozen native regression started under48657 in `percentage-spacing-v16-full`. Nested-cost smoke check covers4/8/16/32 levels with3 native probes each: all complete; percentage median13/16/16/26ms versus pixel14/14/16/19ms. These include process/import/layout/stream costs and are not an isolated CPU benchmark or general performance guarantee. Visual coverage is now12/441 audited comparisons; root mixed-spacing repro reviewed at all widths.

L15 rounding visual review complete:81/81 comparisons audited,30 directly inspected views and51 exact full-PNG pair transfers within the run. All ten representative sheets were inspected for padding/margin/combined boundary placement and clipping. Total focused visual coverage93/441, 348 remaining. Full native regression48657 remains active as last confirmed live; no qualification claim yet.

L15 root visual review complete:24/24 directly inspected and audited across row/column flow, percentage padding/margins, mixed values and auto margins. Total visual coverage114/441, 327 remaining. Full native regression48657 was confirmed live during this review; last observed check419 passing. All qualification limitations remain in the version16 receipt.

L15 minimal intrinsic visual cohort complete:48/48 audited comparisons,36 direct views and12 exact-image transfers. Inspected row/column padding, combined margins, controls, fixed parent, margin-only and nested cases at all widths. Total focused coverage162/441, 279 remaining. Full native regression48657 confirmed live; no completion claim.

L15 text/image composition visual review adds24 direct views: row/column × both box modes at three widths. Text wraps and surrounding spacing align; minor glyph raster differences persist within existing limits. Image size/insets align overall; thin image/quadrant boundary differences remain within existing limits and are explicitly recorded, not described as byte-identical. Composition coverage30/96; total186/441, 255 remaining.

L15 wrapping visual review: all eight row/column × border/content-box × wrap/wrap-reverse sheets inspected at 240/390/768; placement, wrapping and percentage spacing align with Chrome. Composition review now54/96; audited focused total210/441,231 remaining. Full native regression48657 confirmed live. Chrome remains the sole browser reference; Firefox is not a qualification gate. L15 remains experimental pending complete visual and regression/coverage audits.

L15 nested/size-limit visual review: eight row/column × border/content-box × nested/limits compositions inspected at all three widths. Insets, constrained card sizes, sibling positions and boundary overflow/clipping align with Chrome. Composition78/96 audited; focused total234/441,207 remaining. Full native regression48657 confirmed live; L15 qualification remains incomplete.

L15 composition visual review complete:96/96 audited, including the final cascade and intrinsic row sheets. Four additional basic row sheets inspected at all widths for percentage padding, margin, auto-margin and intrinsic padding. Focused visual total267/441, 174 remaining, all in the initial matrix. Full native regression48657 confirmed live; no qualification claim yet.

L15 basic matrix review continues: six intrinsic border-box and definite content-box row sheets directly inspected at all widths. Parent sizing, child insets, box expansion and auto-margin distribution align with Chrome. Audited total288/441, 153 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 matrix review: intrinsic content-box padding/mixed/auto-margin plus reversed border-box definite padding/margin/mixed directly inspected at all widths. Parent sizing, overflow, right anchoring and physical insets align with Chrome. Audited total309/441, 132 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 reversed-row review adds six sheets at all widths, covering border-box intrinsic spacing, definite auto-margin and content-box definite padding. Left overflow/clipping, insets, parent widths and box expansion align with Chrome. Audited total330/441, 111 remaining. Regression48657 confirmed live; latest observed check3131 passing. Qualification remains incomplete.

L15 visual review adds remaining reversed content-box row sheets and definite column padding at all widths. Intrinsic sizing, auto spacing, clipping and vertical sibling positions align with Chrome. Audited total348/441, 93 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 column border-box visual review adds definite margin/mixed/auto and intrinsic padding/margin/mixed sheets at all widths. Parent widths, child insets and vertical sibling spacing align with Chrome. Audited total372/441, 69 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 column content-box review adds six sheets at all widths: percentage padding expansion, mixed/auto spacing and intrinsic parent sizing visually align with Chrome, including horizontal overflow. Audited total390/441, 51 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 reverse-column visual review adds six sheets at all widths. Bottom anchoring, physical percentage insets, margin separation and intrinsic sizing align with Chrome. Audited total411/441, 30 remaining. Regression48657 confirmed live; qualification remains incomplete.

L15 focused visual review complete:441/441 comparisons audited across initial192, root24, composition96, minimal48 and rounding81. All ten final reverse-column sheets viewed at every width; bottom anchoring, insets, intrinsic sizing and overflow align with Chrome. Receipts verify direct sheet/PNG hashes and exact within-run image transfers. Existing text/image raster differences remain documented within unchanged thresholds. Full native regression and broader interaction coverage audit remain pending; L15 is not yet qualified.

L15 reverse-composition coverage expansion:32 new scenes/96 Chrome-native geometry and pixel comparisons pass with unchanged thresholds. Public original/clone resize regression passes256 instance-viewport updates. Covers both reversed directions and box models across nesting, wrapping, reverse wrapping, limits, images, text, cascade and intrinsic hosts. Six views directly inspected;90 remain. Expanded native/WASM parity4427 and full regression48657 confirmed live. Evidence: `percentage-spacing-validation/reverse-composition-receipt.json`. This extends the previous441 fully reviewed comparisons; qualification remains incomplete.

L15 full native regression completed:5983/5983 checks pass; all5973 Chrome/Rust Metal scene comparisons have exact HTML/CSS and full browser/native PNG identity with the reviewed intrinsic-image-stretch baseline. Six shared harness/reset hashes match the baseline and current files. Audited receipt: `percentage-spacing-v16-full/receipt.json`. Expanded reverse-composition native/WASM parity11/11 passes;27/96 new views visually audited,69 remaining. L15 remains unqualified pending expanded visual completion and interaction coverage audit.

L15 reverse-composition visual audit advances to51/96: all intrinsic and nested cases in both reversed directions and box modes directly inspected at240/390/768. Intrinsic parent sizing, percentage insets, content-box expansion and offscreen overflow align with Chrome.45 views remain; full regression and parity already pass. Qualification remains incomplete pending visual and interaction audit.

L15 reverse-composition visual review now72/96. All image and text compositions inspected at every width; placement and spacing align with Chrome. Thin image perimeter/quadrant and glyph raster differences are visible within unchanged limits and explicitly recorded. Remaining24 views cover limits and cascade; no qualification claim yet.

L15 reverse-composition visual review complete:96/96 directly inspected and audited, including final limits/cascade sheets across both reversed directions and box modes. Total focused Chrome/native geometry, pixel and visual coverage is537/537. Full native5983 checks/5973 audited scene pairs and expanded parity11/11 pass. Remaining interaction coverage audit is still required before qualification.

L15 interaction audit found a real remaining defect:40 new scenes/120 views yield114 combined passes and6 geometry failures, all column/column-reverse content-box with sub-unit growth and percentage basis. Five of these also fail pixels. Public helper omission for justify-content was corrected; six previous spacing groups pass, new interaction group remains red. Native probe independently reproduces failures. Preserved logs and `percentage-spacing-validation/interaction-receipt.json`; no tolerance changes. Parity79504 remains pending.

L15 basis diagnosis: four reduced Chrome scenes preserve failure after removing all card children and after disabling growth; literal pixel padding passes. This isolates percentage-padding adjustment rather than content measurement or sub-unit distribution. Candidate resolves content-box basis padding against inline width under existing CSS spacing opt-in, leaving basis axis and default behavior unchanged. Public spacing suite70977 running; not yet qualified. Expanded interaction parity11/11 passes. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 basis-axis correction passes all eight public spacing oracle groups:223 scenes/1784 original-and-clone viewport updates. New Taffy regression checks both column directions with policy false→true→false; all113 layout-engine tests pass. This confirms inline-axis padding adjustment while preserving main-axis percentage basis and default behavior. Full public3140 and new frozen native/WASM build14590 are running. Fresh pixels, visual review and full native regression remain required; prior six rendered failures are not yet claimed fixed. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 basis-axis full public regression passes302 tests/54 groups,0 ignored. Frozen v16-r3 native/WASM toolchain built successfully. Started669 focused comparisons across eight cohorts (44178), expanded parity/host checks48956, and full native regression80940 in `percentage-spacing-v16-r3-full`. Shared browser harness and main corpus must remain frozen while80940 runs. Fresh rendered qualification remains pending.

L15 corrected v16-r3 focused replay is terminal:669/669 geometry and native pixel comparisons pass, parity/host27/27 pass. All537 prior comparison pairs transfer visual review with exact compiler-input and full PNG identity. Reduced12/12 directly inspected; both formerly failing column compositions6/6 directly inspected. Total555/669 visual coverage,114 interaction views remaining. Reduced zero-growth240 retains a thin tail-edge raster difference within unchanged limits. Full native80940 confirmed live; vector fallback audit remains separate. Evidence: `percentage-spacing-validation/basis-axis-receipt.json`.

L15 v16-r3 aspect-ratio interaction visual cohort complete:24 direct views across four directions and both box models. Ratio sizing, percentage insets and viewport overflow match Chrome visually. Interaction coverage30/120; aggregate579/669,90 views remain. Full native80940 confirmed live.

L15 row clipping interactions inspected at all widths in both box models and directions: clip boundaries and hidden footer match Chrome. Aggregate visual coverage591/669,78 interaction views remain. Started full669 vector-fallback comparisons with native glyphs disabled (68274); this uses the same frozen v16-r3 toolchain and Chrome references. Full native80940 confirmed live.

L15 column clipping visual inspection complete across both directions/box models:12 more views align with Chrome. Aggregate603/669 reviewed,66 remain. Vector replay68274 remains live; composition cohort reports failures requiring inspection, while interaction/reduced/initial/root/minimal completed successfully. Full native80940 confirmed live. No failures waived.

L15 vector fallback replay completed:669 geometry passes,651 pixel passes,18 unwaived text pixel failures in composition/reverse-composition cohorts. Authoritative cohort results override runner process exit0 because the runner continues after failures. Receipt: `percentage-spacing-validation/vector-v16-r3-receipt.json`. Row order visual review adds12 native views; aggregate615/669,54 remain. Full native regression still pending.

L15 v16-r3 focused native visual review complete:669/669 comparisons audited. Final54 direct views cover column order, remaining grow/basis and all distribution interactions at240/390/768. Insets, reverse placement, free-space allocation, alignment and viewport overflow match Chrome visually. Interaction receipt120/120 passes hash audit. Full native80940 confirmed live;18 vector text pixel failures remain unwaived and require investigation. L15 remains unqualified.

L15 vector failure review:all18 failing text views plus6 passing text controls directly inspected; differences are concentrated in glyph pixels while wrapping and surrounding shape placement align. Another477 vector views transfer from fully audited native sources with exact compiler-input/full-PNG identity;501/669 vector views reviewed,168 non-text composition views remain. Single-scene240px vector replay fails deterministically twice with identical PNGs; native-glyph toggle passes against the same Rive bytes and Chrome image. This isolates a rendering-path difference, not its root cause. Reproducer:percentage-spacing-validation/vector-text-diagnosis.json. No failure waived; scene minimization remains next.

L15 vector text minimization now reproduces on a single display:block text element:Quiet weekend,110px width,Inter16px/1.4. Fresh pinned Chrome153.0.8010.12 controls show the same local text RGB failure after independently removing siblings, zeroing all spacing and removing wrappers; all geometry comparisons pass. This demonstrates percentage spacing is not required for the rendering residual. Single-scene and reduced reproducers remain preserved in percentage-spacing-validation/vector-text-diagnosis.json. Further typography minimization and runtime diagnosis remain; no failure waived.

Vector text diagnosis adds15 pinned-Chrome typography controls. Individual words/glyphs pass but retain nonzero RGB error; the full nowrap phrase still fails. Integer line heights22/23/24 pass, while22.4 fails. Fractional sweep22.125/22.25/22.5/22.75/22.875 is preserved with metrics in percentage-spacing-validation/vector-text-diagnosis.json. Baseline placement, coverage and horizontal placement remain ranked hypotheses; no runtime change or root-cause claim yet.

Vector baseline experiment:temporary opt-in snapping passes both reduced and original single-scene reproducers. Expanded24 text composition views improve from6 passes to16, with8 residual pixel failures; geometry remains checked. Runtime source restored exactly after freezing diagnostic probe; no shipping change retained. Native glyph rasterizer already snaps final world baselines, whereas vector paths keep fractional baselines. This explains part of the error but is not a complete fix; final draw-time transform and remaining coverage need investigation. Evidence:percentage-spacing-vector-snap-experiment/receipt.json. Separately, all669 vector views now have visual audit evidence(645 exact transfers plus24 direct text inspections);18 original failures remain unwaived.

Vector snap audit separates two residuals:16/24 views have exact intended world baselines, while content-box views can miss by0.5625px. All border-box768 failures already have exact baseline snaps, so transform timing is insufficient as a full explanation. In the row-border-box768 text rectangle, summed ink is105740 Chrome,92139 snapped vector and108466 native glyph; this diagnostic shows a remaining coverage difference(about12.9% less vector ink), without changing gates. Audit files:percentage-spacing-vector-snap-experiment/baseline-audit.json and border-768-ink-profile.json. Next:font smoothing/outline coverage and draw-time transform seam; no shipping changes retained.

Native font-smoothing diagnostic now runs at the glyph-producing probe seam. Renderer-only attempt did not exercise rasterization and is preserved as an invalid diagnostic. Fresh probe toggling changes text RGB error from2.0653 to4.1162; both native controls pass. Source restored after freezing diagnostic probe. Ink measurements and unchanged gates are recorded in percentage-spacing-vector-smoothing-experiment/receipt.json. Font smoothing contributes to the coverage difference but is not yet a complete vector explanation or fix.

L15 corrected full native regression completed:5983/5983 checks pass; all5973 Chrome/native scene pairs match exact HTML/CSS and full PNGs from the reviewed prior baseline. Six shared harness hashes match baseline and current files. Receipt:percentage-spacing-v16-r3-full/receipt.json. Draw-time baseline diagnostic improves text24 from16 to20 passes; source restored, four pixel residuals remain. Consolidated investigation:validation/vector-text-baseline-investigation.md. Original vector18 failures remain unwaived; L15 retains vector qualification work.

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

L16 final public suite passes307 tests across55 groups,0 ignored. Current support table now reflects implemented percentage/signed spacing and separates candidate acceptance from qualification. Consolidated scope/coverage audit added to validation/negative-margins-review.md;432 native views audited,423/432 vector pixels pass with9 unwaived text failures. Full native46959 remains live; no final qualification claim.

L17 public v17 initial native replay completes576/576 geometry and pixels. Composition vector completes192/192 geometry,183/192 pixels with9 unwaived text failures. Diagnostic composition direct visual review reaches24/192 after row-border-box both-positioned/order/text/wrap-ratio sheets at all three widths; stacking, wrapping, ratio sizing and overflow match Chrome, with small recorded glyph coverage differences. Full native regression26540 confirmed live by session poll. Larger visual audit and vector qualification remain. Receipt:output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 nested clipping composition visual review is complete across all four directions and both box models at240/390/768. Seven additional sheets/21 views directly inspected: translated child clipping, reversed outer right/bottom clipping and ordinary sibling placement match Chrome. Composition visual total45/192,147 remaining; receipt hash audit passes. Initial vector72993 and full native26540 confirmed live by handle polls. No qualification claim or tolerance change.

L17 initial vector replay72993 terminal0:576/576 geometry and pixel comparisons pass. Composition visual review reaches66/192 after all nested-positioned cases across directions/box models are directly inspected at all widths; descendant escape, sibling overlap, right viewport edge and lower host overflow match Chrome. Review receipt audit passes;126 composition views remain. Full native26540 confirmed live. Nine composition vector text failures remain unwaived; initial visual qualification is still pending.

L17 two-positioned-sibling visual coverage complete across all directions and box models:21 more views directly inspected at240/390/768. Positioned green sibling overlap, subtree coverage, reverse-row ordinary sibling overlap and column-reverse overflow match Chrome. Composition review87/192 with105 remaining, receipt audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain open.

L17 flex-order composition review complete across four directions and both box models:21 additional views directly inspected. Whole-subtree placement, purple sibling overlap, narrow viewport clipping and reverse-column order match Chrome. Composition visual audit108/192,84 remaining; hash audit passes. Full native26540 confirmed live by handle poll. Vector text residuals remain open.

L17 ordinary-sibling overlap composition review complete across directions/box models:21 additional views directly inspected at240/390/768. Positioned subtree coverage, unchanged sibling footprint, reverse-row clipping and column-reverse overflow match Chrome. Composition visual audit129/192,63 remaining for text/images/wrap-ratio; receipt hash audit passes. Full native26540 confirmed live. Nine vector text failures remain open.

L17 image composition visual coverage complete across all directions/box models:21 additional views directly inspected at240/390/768. Image quadrants, percentage offsets, translated parent and sibling placement match Chrome. Thin horizontal image raster differences in border-box and sparse right-edge differences in content-box are explicitly recorded within unchanged gates. Composition review150/192,42 remaining for text/wrap-ratio; receipt audit passes. Full native26540 confirmed live; vector text failures remain open.

L17 native text composition review complete across all directions/box models:21 more views directly inspected. Two-line wrapping, auto card height, nested text offset and sibling placement match Chrome. Small glyph coverage differences near the end of weekend remain documented within unchanged native gates. Composition visual audit171/192,21 wrap-ratio views remaining; hash audit passes. Full native26540 confirmed live. Nine separate vector text failures remain open.

L17 composition native visual audit complete192/192. Final21 wrap-ratio views directly inspected: ratio sizing, narrow wrapping, percentage relative translation, auto host size and viewport clipping agree with Chrome. All192 public v17 composition comparisons receive audited review transfer with exact canonical compiler inputs and full Chrome/native PNG identity. Public discriminator24 also reviewed. Initial576 visual audit, vector text failures/visual evidence, remaining interactions and full native26540 regression still pending;26540 confirmed live. Receipt:output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 composition vector failures directly reviewed:9 failing border-box text views plus3 passing column-reverse controls. Glyph coverage differs while wrapping and box geometry agree; passing controls also retain visible glyph differences, and no root cause is claimed. Another168 non-text views transfer from fully audited diagnostic native source with exact canonical input and full PNG identity. Vector visual total180/192;12 content-box text controls remain. Nine failures stay unwaived. Receipt:positioned-v17-composition-vector/failure-visual-inspection.json. Full native26540 confirmed live.

L17 composition vector visual audit complete192/192:24 directly inspected text views plus168 exact canonical-input/full-PNG transfers. Final12 content-box text controls retain visible glyph coverage differences despite passing unchanged numeric thresholds; wrapping, card heights and offsets match Chrome. Rechecked all192 unique receipt entries and full image/sheet/transfer hashes, with independent source review audit. Nine original pixel failures remain unwaived. Full native26540 confirmed live; initial corpus visual review and remaining interaction audit still pending.

L17 initial native visual audit begins with12 directly inspected row baseline views across definite/intrinsic host heights and both box models. Box dimensions, child insets, sibling gaps and viewport clipping match Chrome. Exact full-image controls bring audited coverage to36/576;540 remain. Full native26540 confirmed live. Nine vector text failures remain open.

L17 initial reverse-row baseline review adds12 direct views across both box models and host-height modes. Right anchoring, left-edge overflow, child inset and host bottom agree with Chrome. Exact full-PNG controls bring initial visual coverage to72/576;504 remain. Receipt audit passes; full native26540 confirmed live.

L17 column baseline review adds12 direct views covering definite/intrinsic host height and both box models. Vertical order/gaps, teal inset, box dimensions and host bottom match Chrome. Exact-image controls bring initial native visual audit to108/576;468 remain. Receipt audit passes; full native26540 confirmed live.

L17 initial baseline controls complete across four directions, both box models and definite/intrinsic heights. Final12 reversed-column views directly inspected: bottom anchoring, leading free space, gaps and intrinsic host size match Chrome. Exact-image baseline/auto/static-offset controls bring audited coverage to144/576;432 offset views remain. Receipt audit passes. Full native26540 confirmed live.

L17 percentage-offset visual review adds24 views across row/reverse-row, both box models and definite/intrinsic container heights. Responsive translation, unchanged sibling flow, overlap and viewport clipping match Chrome at240/390/768. Initial native visual coverage168/576,408 remaining; receipt hash audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain unwaived.

L17 positive percentage-offset visual coverage complete across four directions, both box models and definite/intrinsic heights. Another24 column/reverse-column views directly inspected: overlap, sibling flow, horizontal translation and visible bottom overflow match Chrome at240/390/768. Initial native review192/576,384 remaining; receipt audit passes. Full native26540 confirmed live by handle poll. Nine vector text failures remain unwaived.

L17 frozen full native regression completes5983/5983 checks with exit0. All5973 Rust Metal scene pairs have exact HTML/CSS and full Chrome/native PNG identity with the reviewed negative-margin baseline; comparison audit leaves0 unreviewed. Harness and all four frozen toolchain hashes verified. Completion receipt: output/playwright/html-to-riv/positioned-v17-full/receipt.json. Initial right/bottom percentage row review adds12 direct views across box/height modes; left/top viewport clipping and unchanged sibling flow match Chrome. Initial visual coverage204/576,372 remaining. Nine composition vector text failures and remaining qualification audit stay open.

L17 right/bottom percentage reverse-row audit adds12 directly inspected views across both box models and host-height modes. Increasing overlap over green, top viewport clipping, unchanged sibling flow and host size match Chrome. Initial visual216/576,360 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures remain open.

L17 right/bottom percentage column review adds12 direct views across box models and host heights. Top/left viewport clipping, disappearing teal at768, sibling gaps and host sizing match Chrome. Initial visual228/576,348 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 right/bottom percentage-offset coverage complete across four directions, both box models and height modes. Final12 reversed-column views inspected: upward overlap over green, left clipping, sibling flow and host size match Chrome. Initial visual240/576,336 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset row review adds12 directly inspected views across both box models and height modes. Correct right/down translation without stretching, green overlap, sibling flow and host sizing match Chrome. Initial visual252/576,324 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset reverse-row review adds12 direct views across both box models and height modes. Right/down translation without stretching, right viewport gap, narrow purple left clipping and sibling flow match Chrome. Initial visual264/576,312 remaining; receipt audit passes. Full native5983/5983 remains completed and audited; nine vector text failures stay open.

L17 opposing-inset column review adds12 direct views across box models and height modes. Translation without stretching, thin green overlap, fixed sibling flow and container sizing match Chrome. Initial visual276/576,300 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 opposing-inset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: translated card retains bottom gap and dimensions; sibling flow and leading space match Chrome. Initial visual288/576,288 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem row visual review adds12 direct views across both box models and host-height modes. Constant right/up translation across widths, green overlap, top gap, sibling flow and host sizing match Chrome. Initial visual300/576,276 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem reverse-row review adds12 direct views across box models and host heights. Right-edge placement, top gap, teal inset, sibling flow and narrow purple clipping match Chrome. Initial visual312/576,264 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem column review adds12 direct views across box models and host heights. Constant right/up translation, top gap, increased separation above green, sibling flow and host sizing match Chrome. Initial visual324/576,252 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 em/rem visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: upward green overlap, space below card, sibling flow and host size match Chrome. Initial visual336/576,240 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset row review adds12 direct views across box models and height modes. Translation, narrow green overlap, unchanged dimensions, sibling flow and host size match Chrome. Initial visual348/576,228 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset reverse-row and column review adds24 views across both box models and height modes. Reverse-row right edge stays flush; column translation leaves the expected small gap above green. Card dimensions, nested teal inset, sibling flow, clipping and host bottoms match Chrome. Initial visual372/576,204 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 four-value inset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: translated orange/teal dimensions, gap below card, separation from green, leading space and host bottoms match Chrome. Initial visual384/576,192 remaining; receipt audit passes. Full native regression remains completed and audited; nine vector text failures stay open.

L17 left-offset row visual review adds12 views across both box models and height modes. Rightward orange/teal translation, green overlap, unchanged vertical placement and sibling flow, narrow purple clipping and host sizing match Chrome. Initial visual396/576,180 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset reverse-row visual review adds12 views across both box models and height modes. Narrow right margin, increased separation from green, unchanged vertical placement, reverse-row flow, purple left clipping and host bottoms match Chrome. Initial visual408/576,168 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset column visual review adds12 views across both box models and height modes. Horizontal translation, preserved top padding and vertical gaps, aligned sibling flow, nested teal inset and host bottoms match Chrome. Initial visual420/576,156 remaining; receipt audit passes. Nine vector text failures remain open.

L17 left-offset visual coverage complete across all four directions, both box models and height modes. Final12 reverse-column views inspected: bottom gap, green separation, leading space, sibling alignment and card/host dimensions match Chrome. Initial visual432/576,144 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset row visual review adds12 views across both box models and height modes. Leftward translation, reduced left margin, larger gap before green, unchanged sibling flow and narrow purple clipping match Chrome. Initial visual444/576,132 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset reverse-row visual review adds12 views across both box models and height modes. Leftward orange/teal translation, narrow green overlap, increased right margin, unchanged sibling flow and purple left clipping match Chrome. Initial visual456/576,120 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset column visual review adds12 views across both box models and height modes. Reduced left margin, preserved vertical gaps and top padding, sibling alignment, nested padding and card/host dimensions match Chrome. Initial visual468/576,108 remaining; receipt audit passes. Nine vector text failures remain open.

L17 right-offset visual coverage complete across all four directions, both box models and height modes. Final12 reverse-column views inspected: left margin, bottom padding, separation from green, leading space, nested padding and card/host dimensions match Chrome. Initial visual480/576,96 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset row visual review adds12 views across both box models and height modes. Upward orange/teal translation, reduced top gap, unchanged sibling positions and horizontal gaps, narrow purple clipping and host dimensions match Chrome. Initial visual492/576,84 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset reverse-row visual review adds12 views across both box models and height modes. Upward translation, preserved right anchoring and horizontal gaps, reduced top gap, narrow purple left clipping and host dimensions match Chrome. Initial visual504/576,72 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset column visual review adds12 views across both box models and height modes. Reduced top gap, increased separation above green, preserved left alignment, nested padding and sibling/card/host dimensions match Chrome. Initial visual516/576,60 remaining; receipt audit passes. Nine vector text failures remain open.

L17 top-offset visual coverage complete across four directions, both box models and height modes. Final12 reverse-column views inspected: narrow green overlap, increased space below card, left alignment, nested padding and sibling/card/host dimensions match Chrome. Initial visual528/576,48 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset row visual review adds12 views across both box models and height modes. Upward translation, reduced top space, preserved horizontal gaps, sibling placement, narrow purple clipping and card/host dimensions match Chrome. Initial visual540/576,36 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset reverse-row visual review adds12 views across both box models and height modes. Upward translation, preserved right anchoring and horizontal gaps, reduced top gap, purple left clipping and card/host dimensions match Chrome. Initial visual552/576,24 remaining; receipt audit passes. Nine vector text failures remain open.

L17 bottom-offset column visual review adds12 views across both box models and height modes. Upward translation, reduced top gap, increased separation above green, preserved left alignment and sibling/card/host dimensions match Chrome. Initial visual564/576,12 remaining; receipt audit passes. Nine vector text failures remain open.

L17 initial visual audit complete:576/576 native views reviewed,0 remaining. Final12 reverse-column bottom-offset views match Chrome in translation, narrow gap below green, bottom space, nested inset and host sizing. All576 initial vector views transfer with identical canonical compiler inputs and full Chrome/native PNG bytes; independent transfer audit passes. Composition native192/192 and full native5983/5983 already pass with complete visual evidence. Nine composition vector text pixel failures remain unwaived; remaining interaction qualification audit stays open. Evidence: output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 interaction corpus adds32 scenes across four directions/two box models and auto margins, stretch, negative margins and percentage padding. Pinned Chrome153.0.8010.12 captures96 views. Frozen v17 native replay passes96/96 geometry and pixels (exit0); public original/clone resize regression passes (exit0). Permanent oracle and expanded native/WASM parity corpus added; parity50657 remains running. Visual review96 views and vector replay remain pending. No tolerance changes or qualification claim.

L17 interaction vector replay passes96/96 geometry and pixels (exit0). First12 native views directly inspected: row/border-box auto margins, stretch, negative margins and percentage padding match Chrome at all widths, including bottom overflow and sibling overlap. Visual audit12/96,84 remaining. Expanded parity50657 completed with exit0; nine prior composition vector text failures stay open.

L17 row/content-box interaction review adds12 direct views at240/390/768: auto-margin free-space placement, stretched height, bottom overflow, negative-margin overlap and growing percentage padding match Chrome. Visual audit24/96,72 remaining. Native/vector96/96 and expanded parity11/11 pass; nine earlier composition vector text failures remain unwaived.

L17 reverse-row/border-box interaction review adds12 direct views: right viewport truncation, auto-margin placement, stretched height/bottom overflow, negative-margin sibling gaps and percentage-padding inset match Chrome at all widths. Visual audit36/96,60 remaining. Native/vector96/96 and parity11/11 pass; nine prior composition vector text failures remain open.

L17 reverse-row/content-box interaction review adds12 views at all widths: outer card sizing, teal inset, right truncation, auto-margin placement, stretched height/bottom overflow, negative-margin gaps and percentage padding match Chrome. Visual audit48/96,48 column/reverse-column views remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 column/border-box interaction review adds12 views: auto-margin right placement, viewport truncation, horizontal stretch, negative-margin overlap, percentage-padding inset and sibling vertical flow match Chrome at all widths. Visual audit60/96,36 remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 column/content-box interaction review adds12 views: outer dimensions, auto margins, horizontal stretch, overlap, negative margins and percentage padding match Chrome. The768 percentage case preserves purple overflow below the host. Visual audit72/96,24 reverse-column views remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 reverse-column/border-box interaction review adds12 views: auto margins, right truncation, horizontal stretch, bottom overflow, negative-margin gaps and percentage-padding bottom placement match Chrome. Visual audit84/96,12 remaining. Native/vector96/96 and parity11/11 pass; nine earlier composition vector text failures remain open.

L17 interaction visual audit complete96/96. Final12 reverse-column/content-box views match Chrome, including growing percentage padding, bottom overflow and purple top-viewport clipping at768. All96 vector views transfer with exact canonical compiler inputs and full Chrome/native PNG identity; transfer audit passes. Public original/clone resize test and expanded native/WASM parity11/11 pass. Nine composition vector text failures remain unwaived; final feature qualification audit remains.

L17 final focused evidence audit verifies19 aggregate file hashes,888 passing native comparisons with complete image audits,672 initial/interaction vector comparisons with complete audits, and192 composition vector geometry comparisons with183 pixel passes/nine unwaived failures. Existing full native5983-check receipt remains separate from focused coverage. L17 stays partial for vector text; no external blocker is claimed. Next independent implementation item is L18 absolute positioning.

L18 initial runtime diagnostic:28 scenes/84 pinned Chrome views captured. Existing absolute wire passes geometry for24 direct-child scenes (auto/start/end/percent/opposing/auto-margin across four directions), but four static-ancestor scenes produce128 coordinate mismatches across original/clone and240/390/768/240 resize updates. At390 the card x is32 instead of39; at768 it remains32 instead of76.796875. This isolates use of the immediate static wrapper as containing block rather than the positioned host. Public syntax remains rejected; the failing regression is retained in tests/absolute_position_runtime.rs. Next: investigate runtime containing-block representation without flattening paint ancestry or baking viewport coordinates. Evidence: output/playwright/html-to-riv/absolute-position-initial-receipt.json. No pixel or visual qualification is claimed.

L18 containing-block expansion adds16 scenes/48 Chrome views. Combined44-scene original/clone diagnostic fails256 coordinates across12 scenes. All-auto nested cases and positioned-wrapper controls pass; nested left-only cases fail only x, nested top-only only y, confirming each auto axis must retain its immediate-parent static position while explicit insets use the nearest positioned ancestor. Existing flexbox absolute solver iterates immediate children and uses the same AlgoConstants for both roles. Merely reparenting solve nodes would lose the now-proven correct auto-axis behavior. Next implementation needs a separate containing-block size/origin input plus retained static-position context, with coordinates converted back to the original parent. Paint/clipping ancestry must stay intact. Evidence: output/playwright/html-to-riv/absolute-position-containing-block-receipt.json. No syntax admission or pixel qualification yet.

L18 engine seam implemented: opt-in css_absolute_containing_block carries padding-box size/origin relative to the immediate parent. Explicit insets and size resolution use that context; auto-axis alignment retains original parent constants; explicit-axis coordinates convert back to the original parent. Default None preserves existing behavior. New percentage-x/auto-y regression fails before implementation and passes after, including context resize and clearing; all115 engine tests pass. Style memory-size assertions reflect the added field. Runtime population is not yet installed, so the44-scene public-import diagnostic remains unresolved and no CSS admission or pixel qualification is claimed. Logs: output/playwright/html-to-riv/absolute-containing-block-engine-red.log and absolute-containing-block-engine-green.log. Next: live containing-block ancestry/measurement installation and expanded end-inset, size and auto-axis regressions.

L18 engine end-inset/percentage-size/opposing-inset tests now cover all four flex directions. A new nonzero-inset auto-margin regression first fails x140 vs125 in parent-local coordinates. Opt-in containing-block free space now subtracts opposing insets before distributing auto margins; all116 engine tests pass. Added four Chrome nested nonzero-inset auto-margin scenes (containing corpus20 scenes/60 views, total48 scenes). At390 all four directions produce card{x:145,y:93,width:90,height:70}, confirming the solver expectation after subtracting its test parent origin. Permanent diagnostic oracle expanded. Runtime context installation and native pixel qualification remain pending. Logs: output/playwright/html-to-riv/absolute-containing-block-end-insets-{red,green}.log; browser evidence:absolute-position-containing-block-expanded-oracle/oracle.json.

L18 runtime containing-block integration passes all48 diagnostic scenes across384 original/clone resize updates (240/390/768/240, exit0). Opt-in authored static/positioned markers are cloned and locate the nearest containing ancestor; bounded layout passes update live padding-box sizes/origins without changing parentage. Initial integration retained72 resize-coordinate failures because unchanged static parents skipped updated descendants. Comparing new solved geometry and propagating descendant changes through those parents resolves the remaining failures. Original failing log preserved. Public absolute admission, capability/installer validation, lifecycle clearing tests, broader nested/auto-size coverage and native pixels remain pending. Evidence: output/playwright/html-to-riv/absolute-position-runtime-context-receipt.json.

L18 lifecycle checks pass: policy clearing/re-enable at three widths, retained-clone independence, invalid style target rejection, and changing the nearest positioned ancestor. Three relative-position original/clone regressions pass. Added12 nested absolute scenes with percentage size/insets, end insets and opposing-inset auto sizing across four directions. Combined60 scenes/180 Chrome references pass480 original/clone resize updates plus lifecycle checks. No tolerance changes. Public CSS admission, checked host contract and native pixel evidence remain pending. Evidence: output/playwright/html-to-riv/absolute-position-runtime-nested-receipt.json.

L18 draft version18 transport added with layout-css-absolute-position-v1 and layout_absolute targets. Validation requires nonempty unique absolute targets contained within valid positioned targets, the matching capability and version18; old hosts reject missing capability. Two roundtrip/malformed contract tests and all five relative-position compiler tests pass. TypeScript capability/version union and type checks pass. Computed positioning now uses an explicit Static/Relative mode instead of a boolean; absolute admission remains intentionally gated until the host installer is connected. The compiler does not emit version18 yet. Runtime wire/mode verification, JavaScript host integration, rebuilt WASM parity and pixel gates remain. Evidence: output/playwright/html-to-riv/absolute-position-contract.log and absolute-position-contract-types.log.

L18 checked runtime installer implemented as Artboard::set_css_absolute_position_policy_occurrence. It validates unique non-root layout targets, absolute subset and exact agreement with imported absolute position wires before changing containing-block or paint policies. Initial mutable-artboard version exposed a RefCell borrow conflict; occurrence-based validation releases the artboard borrow before callbacks and resolves it. All60-scene/480-update geometry checks and existing lifecycle checks pass with installer use; six malformed-list cases per scene reject without disturbing the valid installed policy. Probe and public oracle host now install the v18 policy before layout/draw; probe compile check pending. Compiler emission and pixel qualification remain pending. Evidence: output/playwright/html-to-riv/absolute-position-installer-occurrence.log (exit0); initial failure retained in absolute-position-installer.log.

L18 candidate public syntax now accepts position:absolute with existing physical insets (auto, signed px/em/rem/percent, zero, shorthand, CSS-wide values and variables). Version18 and layout_absolute are emitted with positioned-paint requirements after descendant compilation. Public output passes all60 Chrome scenes across480 original/clone resize updates. Import-time validation initially rejected cached solver styles; checking the imported position wire fixes it (initial failure preserved). Three absolute contract/cascade tests and five relative tests pass. Public host installation is connected. Fixed/sticky, logical insets and general inset math remain rejected. Rebuilt WASM/native tools and expanded parity corpus are in progress; pixel/visual qualification and broader compositions remain pending.

L18 public validation update: corrected native-glyph-controls build completes; the earlier misconfigured probe and failed replay logs remain preserved. Frozen absolute-v18-native-toolchain passes all180 focused native geometry/pixel comparisons (84 initial,96 nested) against pinned Chrome. Expanded native/WASM parity passes11/11. Checked-installer whole-policy clear/reinstall now passes repeated390/768/240 resizing with retained-clone independence. Direct review currently covers9 nested views; remaining visual review, vector qualification, broader compositions/interactions and full regression remain pending. No tolerances changed. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 vector replay also passes180/180 geometry/pixel comparisons using the same frozen toolchain with nativeGlyphs=0. This is numeric evidence; vector visual qualification is still pending. Receipt: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 composition expansion:64 scenes/192 pinned Chrome views now cover overlap, positioned siblings/descendants, clipping, text, images and wrapping/aspect-ratio combinations across four directions and two box-sizing modes. New public original/clone test retains64 failing coordinates in eight text scenes: auto-width absolute text measures90 vs87 (border-box) or106 vs103 (content-box), exactly the3px left inset. Frozen pre-fix native replay has24 geometry failures and0 pixel failures, illustrating why geometry is an independent gate. Existing full compiler regression passes323 tests across60 groups before this new failing test. Nested direct visual review advances to33/96. Solver candidate now subtracts horizontal insets/margins from available auto-width measurement under the CSS policy; verification and rebuilt pixel replay pending. Failure logs preserved in absolute-v18-composition-{geometry,native}.log; aggregate receipt absolute-v18-public-receipt.json.

L18 inset-width fix verified: both public absolute oracle tests pass (124 scenes,992 original/clone resize updates), and all116 engine tests pass. The64 composition coordinate failures are resolved. Frozen pixel evidence remains pre-fix; rebuild/parity/pixel reruns and full visual qualification remain required. Logs: absolute-v18-composition-inset-width.log and absolute-v18-inset-width-engine.log.

L18 rebuilt inset-width toolchain passes372/372 native geometry/pixel comparisons. Vector geometry passes372/372, pixels354/372:18 text comparisons fail local RGB error across six composition scenes; no waiver. Direct inspection of row/border-box vector text at all3 widths confirms glyph coverage differences with matching wrapping and box edges. Corresponding native text sheets pass and are reviewed. Expanded native/WASM parity including64 composition scenes passes11/11. Pre-fix nested native review45/96; post-fix composition native6/192. Full visual audits and cross-run transfer remain pending. Frozen manifests, initial failure evidence and new results are recorded in output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 post-fix full compiler regression passes324 tests across60 groups with0 failures/ignored tests. Log: output/playwright/html-to-riv/absolute-v18-inset-width-full-regression.log. Visual qualification and18 vector text pixel failures remain open.

L18 deterministic interaction expansion adds48 scenes/144 Chrome views: auto margins, stretch, negative margins, percentage padding, min/max opposing-inset sizing and flex factors, four directions and two box-sizing modes. Initial public geometry has256 coordinate failures in eight auto-margin scenes; pre-fix pixel replay preserves24 failing views. With either opposing inset auto, CSS auto margins must resolve to zero rather than consume free space. Opt-in runtime correction now passes all172 scenes/1376 original-clone resize updates; extended engine start/end/mixed-axis margin checks and all116 engine tests pass. Rebuilt pixels, expanded parity and post-fix regression pending. Native nested visual review57/96. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 auto-inset margin toolchain is frozen after successful featured build. Native/vector144-view interaction replays running. Initial expanded parity run failed because Cargo temporarily replaced the publisher during rebuild (ENOENT), not artifact disagreement. Tests now accept NUXIE_NATIVE_COMPILER to pin a frozen publisher; frozen retry running. Initial failure remains in absolute-v18-interaction-parity.log.

L18 corrected auto-inset margin interaction replays pass144/144 native and144/144 vector geometry/pixel comparisons. Direct visual review, frozen parity result and regression of prior corpora against the latest toolchain remain pending.

L18 latest runtime regression passes325 tests across60 groups with0 failures/ignored. All516 native focused comparisons (initial84,nested96,composition192,interaction144) pass with the auto-inset-margin toolchain. Native nested direct review reaches72/96, with24 views remaining. Frozen publisher parity exposed a second harness assumption: probe path was derived as examples/probe; NUXIE_NATIVE_PROBE now pins that executable explicitly. Both-tool frozen retry and latest vector replays running; prior failures preserved. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 nested visual group complete96/96 with independent audit. Exact canonical compiler inputs and full Chrome/native PNG pairs transfer all96 views to latest native and vector replays; both transfer audits pass. Latest full focused vector geometry passes516/516;18 composition text pixel failures remain unwaived. Frozen publisher+probe parity now passes11/11. Initial/composition/interaction visual review and final qualification remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 public host-negative test passes: two missing capabilities, four malformed absolute target lists, and two schema-valid lists inconsistent with imported absolute wires reject; restoring valid requirements succeeds. Probe adds diagnostic NUXIE_DISABLE_CSS_ABSOLUTE_POSITION. Initial native direct visual review9/84. Full baseline native Playwright suite launched with frozen auto-inset-margin toolchain and isolated absolute-v18-full-native outputs; no result claimed yet. Host log: output/playwright/html-to-riv/absolute-v18-host-negative.log.

L18 initial visual audit progresses to42/84 views (33 direct,9 exact image-pair transfers),42 remaining. Newly reviewed row percentage/opposing/auto-margin/static-ancestor and reverse-row auto/start/end/percentage cases match Chrome across all widths. Receipt audit passes. Full native baseline remains live under session9759; partial progress is not a completion claim. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 initial visual qualification completes84/84 (75 direct views plus9 exact image-pair transfers); independent audit passes. All84 views transfer to the latest vector replay with matching canonical compiler input and full Chrome/native PNG hashes; transfer audit passes. Together with the completed nested96 views, initial/nested visual coverage is180/180 in both profiles. Composition and interaction visual review,18 unwaived vector text pixel failures, and the live full native baseline remain pending. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition visual review reaches28/192 (24 direct plus4 exact image-pair transfers), audit passed. Eight column/border-box scenes cover positioned siblings, nested overflow, clipping, image placement, paint order, overlap, two-line text and responsive aspect-ratio growth at three widths. No geometry/paint-order mismatch observed; sparse image/text edge differences remain within existing gates. Remaining composition/interaction review and18 unwaived vector text failures stay open. Receipt: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition visual audit reaches56/192 (48 direct,8 exact image-pair transfers). All eight column/content-box scenes inspected across240/390/768: sizing, clipping, overflow, sibling paint order, text wrapping and responsive ratio agree with Chrome. Image case has visible internal-edge diff outlines despite passing existing gates (240px:256 mismatched pixels, max geometry error0.006251px); retained as an asset-rendering observation, not pixel identity. Full baseline session9759 confirmed live; no completion claim.18 vector text failures remain unwaived. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches68/192 (60 direct,8 exact image-pair transfers), audit passed. Reverse-column border-box overlap, both-positioned siblings, nested positioned descendants and clipping match Chrome at all three widths.124 composition views remain, alongside interaction review and18 unwaived vector text pixel failures. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches79/192 (72 direct,7 exact image-pair transfers), audit passed. Reverse-column border-box order/text/image/wrap-ratio sheets inspected at all widths. Text baselines and wrapping, image crop, growing ratio coverage and bottom overflow agree with Chrome; glyph/image edge differences are retained, not claimed pixel-identical. Remaining composition views:113; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches91/192 (84 direct,7 exact image-pair transfers), audit passed. Four reverse-column content-box overlap/positioned/nested/clip cases inspected at all widths: enlarged box dimensions, static sibling locations, ancestor-relative teal placement and clipping edges agree with Chrome. Remaining composition views:101; interaction and18 vector text failures remain open. Full native session9759 confirmed live with progress through test2965. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches102/192 (96 direct,6 exact image-pair transfers), audit passed. All forward/reverse column compositions now directly inspected across both box-sizing modes and three widths. Final reverse-column content-box text/image/order/ratio cases preserve matching layout, wrapping, crop and overflow. Image internal-edge and glyph-edge differences remain recorded; no pixel identity claimed.90 row/reverse-row views remain. Full native session9759 confirmed live; interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches114/192 (108 direct,6 exact image-pair transfers), audit passed. Row border-box overlap/nested/clip/order scenes inspected at all widths. Orange sibling overlap, teal overflow/clip boundaries and purple viewport cropping agree with Chrome. Remaining composition views:78; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches123/192 (117 direct,6 exact image-pair transfers), audit passed. Row border-box text/image/ratio sheets reviewed at all three widths, completing that group. Two-line wrapping and top clipping, image quadrant placement, responsive sibling coverage and bottom overflow agree with Chrome; sparse glyph/image differences remain recorded. Remaining composition views:69; interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches135/192 (129 direct,6 exact image-pair transfers), audit passed. Row content-box overlap/nested/clip/order inspected at all widths; larger orange bounds, exposed purple strips, teal overflow/clip and narrow viewport cropping agree with Chrome. Remaining composition views:57; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches144/192 (138 direct,6 exact image-pair transfers), audit passed. Forward-row content-box text/image/ratio reviewed at all widths, completing forward-row coverage. Wrapping, crop, image geometry, responsive sibling coverage and overflow agree with Chrome; visible image/glyph edge differences retained.48 reverse-row views remain; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 latest native composition review reaches156/192 (150 direct,6 exact image-pair transfers), audit passed. Reverse-row border-box overlap/both-positioned/nested/clip inspected across all widths: right-tracking static siblings, host-relative teal placement, changing overlap and clip edges agree with Chrome. Remaining composition views:36; interaction review and18 vector text failures remain open. Full native session9759 confirmed live. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition review reaches168/192 and audit passes, completing reverse-row border-box coverage.24 reverse-row content-box views remain. Saving review initially failed ENOSPC; full native session9759 then exited1 with ENOSPC creating Playwright worker artifacts. This is an incomplete infrastructure-failed run, not a qualification pass. Removed13 older rebuildable runtime rlib archives (6.82GB), preserving latest two and all rendered evidence; review save/audit now pass. Full baseline needs fresh replay. Interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition review reaches180/192 and audit passes. Reverse-row content-box overlap/both-positioned/nested/clip agree with Chrome at all widths;12 final views remain. Disk recheck shows53GiB available. Fresh full native baseline launched in absolute-v18-full-native-retry (session43262) with frozen toolchain; failed ENOSPC run preserved separately. Interaction review and18 vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 native composition visual review completes192/192 (186 direct,6 exact image-pair transfers); independent audit passes. Final reverse-row content-box order/text/image/ratio cases agree with Chrome for wrapping, clipping, changing overlap and responsive overflow at240/390/768. Image/glyph edge differences remain explicitly recorded, not pixel identity. Initial+nested+composition native visual coverage now372/516; interaction144 remains. Full baseline retry session43262 confirmed live. Composition vector review and18 unwaived text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 vector composition failure inspection completes18/18 across six text scenes and three widths. Chrome/native/diff sheets show matching two-line wrapping and box placement but visible glyph coverage differences on Quiet/weekend. Independent receipt audit verifies each original PNG and sheet hash plus exact coverage of every failing replay identity. All18 local RGB failures remain unwaived;174 passing composition views still need visual review or exact-input/full-image transfer. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-composition-vector/failure-visual-inspection.json.

L19 occurrence stacking metadata connected to runtime paint trees and clone lifecycle. Host integration passes3/3 tests: auto/zero containment, static flex contexts, invalid-target atomicity, clone resize, independent clear/reinstall. Public CSS/versioned contract and pixel qualification remain pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json. L18 frozen full native retry completed5983/5983 checks; full visual audit remains pending.

L19 public z-index parser/emission and version19 host contract implemented;5/5 focused tests pass (two compiler/contract and three recorded-runtime). Native probe and JS types connected. Featured toolchain build running; native/WASM parity and Chrome replay pending. Evidence: output/playwright/html-to-riv/stacking-v19-public-tests.log.

L19 first frozen public native replay completes144 views:144 geometry pass,132 pixel pass,12 unwaived paint failures in reverse directions (equal-level CSS order and column-reverse negative-descendant/equal-sibling cases). Row-reverse equal-order three-width sheet directly inspected: Chrome teal covers green overlap, native green covers teal. Other nine failing views remain uninspected. Existing css_ordered_drawables reverses child traversal for reverse flex directions; its interaction with context scheduling needs investigation before changing baseline behavior. Native/WASM toolchain frozen in stacking-v19-toolchain; expanded JavaScript parity running. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 expanded JavaScript suite completed12/12 pass, including48 added stacking scenes with identical native/WASM bytes, source maps and requirements. Paint failures remain open. Log: output/playwright/html-to-riv/stacking-v19-javascript.log.

L19 all12 initial failed views now directly inspected. Column-reverse remaining9 views show orange covering green in native where Chrome green covers orange. Added public recorded-runtime regression across four directions, two CSS orders, originals/clones and repeated resize; pre-fix fails row-reverse240 as expected. Fix retains compiler occurrence preorder for stacking ties (including auto positioned entries), independently of ordinary reverse-flex paint order. Added pure scheduler ordinary-order preservation regression. Post-fix focused tests running session68598; fresh pixels and broader checks pending. Initial frozen failure evidence preserved.

L19 tie-order fix passes6/6 focused tests, including the previously failing reverse-direction regression with original/clone resize. Pure scheduler tests running session26453; rebuild/freeze native probe and replay all144 Chrome views next. No post-fix pixel success claimed yet.

L19 corrected frozen native and vector replays each pass144/144 geometry and pixel comparisons. All12 previously failing views directly re-inspected; green/orange and teal/green overlaps now agree with Chrome. Native review audit covers21/144 views (12 direct plus9 exact image-pair transfers);123 native and vector review remain. Pure scheduler14/14 and public48-scene clone/resize/clear/reinstall geometry test1152/1152 updates pass. Initial failure evidence retained. Expanded text/image/clip compositions, host-negative imported-target checks, full regression and remaining visual qualification remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 host-negative integration passes: missing capability, malformed context lists, style/missing targets, fractional/out-of-range levels reject before rendering; restored valid requirements succeeds. Probe diagnostic capability disable added without changing rendering semantics. Native visual audit42/144 (18 direct,24 exact within-run image-pair transfers) includes row auto versus zero at all widths: teal escape and green overlap match Chrome. Full compiler regression running session3670;102 native views, vector transfer/review and composition expansion remain. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 full compiler regression completed: 330 passed across 62 groups, 0 failed, 0 ignored. Log: output/playwright/html-to-riv/stacking-v19-full-compiler-tests.log.

L19 composition expansion adds32 scenes/96 Chrome views: four directions, text/image auto versus zero contexts, nested clipping, negative parents and deep auto escape/zero containment. Initial text fixture24px line height rejected by existing Inter natural-metric constraint; initial references and failed replay logs retained. Corrected28px fixture recaptured with pinned Chrome153.0.8010.12. Native/vector replay running sessions26631/65233; expanded13-test JavaScript suite running38651. Initial native visual review48/144 (24 direct,24 exact pair transfers) after ancestor-clip/static-flex sheets inspected at all widths. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native/vector each96/96 geometry/pixel pass. Expanded JavaScript suite13/13 passes including32 composition fixtures native/WASM artifacts. Public resize test now80 scenes/1920 original-clone updates across stacking install/clear/reinstall, all Chrome geometry pass. First12 native composition views (row text/image auto/zero) directly inspected and audited: matching two-line text, quadrant placement and context-dependent green occlusion.84 composition native views and vector review/transfer remain, alongside96 initial native views. Full visual completion and broader regression gate remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review24/96 complete and audited after all forward-row cases inspected. Nested clips hide deep rose, negative parent descendants stay behind host, and deep auto versus zero yields expected rose/green overlap. Added80 stacking fixtures to the regular browser regression gate. Frozen full native suite launched6223 tests (session41911), isolated stacking-v19-full-native outputs; no terminal result claimed. Initial/native composition remaining visual views96/72; vector transfers/review pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review48/96 completed and audited, covering all forward/reverse row cases at three widths. Reverse text/image right viewport crop, nested clipped rose/teal and hidden negative-parent subtree match Chrome. Reverse auto/zero variants have no green overlap, so their review proves placement/crop rather than context distinction (covered by forward-row overlaps).48 column/reverse-column composition views remain. Full baseline41911 confirmed live this turn; no terminal claim. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native visual audit72/96 complete, adding all forward-column text/image/clip/negative/deep scenes. Chrome/native match text wrapping, image placement, green/purple overlap, nested clipping and negative-parent hiding. Column auto/zero pairs do not expose sibling overlap, so their visual evidence is limited to placement/visibility. Final24 reverse-column views remain. Full native baseline41911 confirmed live. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition visual review96/96 complete and audited. Final reverse-column views match Chrome; negative-parent rose overflow correctly remains visible as strip below host background. Exact canonical inputs and full browser/native image pairs transfer72/96 vector views, audit passes;24 vector text views differ and need direct inspection (numeric gates already pass). Initial48/144 native review and full baseline still pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 initial native visual audit now covers78/144 views (45 direct,33 exact pair transfers);66 remain. New forward-row reviews confirm negative descendant hiding, CSS order placement and absolute sibling levels. Reverse-row negative auto hides teal while zero exposes teal above parent; positive auto/zero have no sibling overlap. All inspected crops and overlaps match Chrome. Initial vector audit remains pending; composition96/profile coverage is complete. Full baseline41911 confirmed live this turn.

L19 initial native review now93/144 (60 direct,33 exact image transfers), audit passed. All forward/reverse row views covered; remaining51 column views. Reverse-row static flex context preserves root-relative absolute placement; ancestor clipping crops teal at parent boundaries; absolute sibling overlaps and negative sibling phases match Chrome at all three widths. Initial vector review, lifecycle expansion and full baseline completion remain. Session41911 confirmed live.

L19 initial native audit covers111/144 views (72 direct,39 exact transfers);33 remain. Forward-column negative-auto hides teal while negative-zero paints it above orange; positive auto/zero lack sibling overlap and prove placement only. All widths visually agree with Chrome. Initial vector review and lifecycle expansion remain; full6223 baseline session41911 confirmed live.

L19 initial native visual audit now126/144 (87 direct,39 exact transfers). All forward-column views covered;18 reverse-column views remain. CSS-order and negative sibling overlap, static context placement and ancestor clipping match Chrome at all widths. Initial vector transfer must await completed native review. Full baseline41911 confirmed live; lifecycle expansion remains.

L19 focused visual coverage complete: initial native144/144 (105 direct,39 exact within-run image transfers); vector144/144 canonical-input and full browser/runtime PNG transfers audited. Composition96/profile already complete, for240/profile total. Final reverse-column inspection matches Chrome including orange/green overlap, root-relative static-context child placement, ancestor clipping and teal overflow beyond host bottom. Full6223 baseline remains live session41911; dynamic clip lifecycle, expanded overlap interactions and full regression audit remain before qualification.

L19 dynamic clip lifecycle regression added in tests/positioned_paint_runtime.rs. Four existing tests pass; new regression fails clip-depth assertion after toggling clipping, even after correcting initial expected outer artboard clip. Reproducer: stacking-v19-live-clips-reproducer.log. Matrix includes original/clone, four directions and auto/zero/negative parent contexts but stops on first failure; do not claim full matrix execution. Diagnose stale runtime clip plan versus test assumptions next. Focused static visual coverage remains240/profile complete; full baseline still pending.

L19 dynamic clipping diagnosis: cached CSS plan omitted ordinary proxy for initially unpainted/unclipped host. Deferred groups reopened its live clip, but ordinary child background missed it. runtime_tree now supplies proxy for plain layout containers in CSS plans. Focused five-test run active8481 (stacking-v19-live-clips-proxy-fix.log); no passing claim yet. Frozen full baseline41911 remains pre-fix and live; its result cannot qualify this new runtime change. Fresh renderer qualification and regressions required after focused tests.

L19 clip-proxy fix focused regression completed:5/5 pass, including864 dynamic clip/resize draws across four directions, three parent context policies, originals/clones and repeated host/parent clipping transitions. Paint order and clip save/restore balance pass. Original failure retained. Fresh frozen renderer comparisons and broader regression still required; existing full41911 uses pre-fix snapshot.

L19 clip fix qualification: vector composition96/96 passes with clip-toolchain, visual audit pending. Native replay failed preflight because concurrently running cargo test rebuilt the probe without native-glyph-controls before snapshot copy. Keep failed snapshot/log; wait compiler suite21908 terminal, then rebuild featured probe and freeze a new snapshot sequentially. Do not treat native preflight failure as a pixel result. Earlier full baseline41911 remains independent/pre-fix.

L19 clip fix full compiler suite terminal:331 passed,0 failed across62 result groups (stacking-v19-clip-compiler-tests.log). Vector composition72/96 exact-input/full-image transfers audited;24 text views remain. Featured probe rebuild now runs sequentially after suite completion (session48219, stacking-v19-clip-probe-sequential-build.log); freeze new snapshot only after terminal success. Native replays and broader visual regression remain pending.

L19 clipping fix featured snapshot passes all240 focused geometry/pixel comparisons in each profile. Audited exact source/full-image transfers cover native240 and initial vector144. Composition vector72 transfer from native plus24 matching earlier directly reviewed vector text images independently cover96/96 (stacking-v19-clip-featured-composition-vector/visual-coverage.json). Full compiler331 and live lifecycle864 draws pass. Fresh full regression, expanded overlap interactions and dynamic clip pixel comparisons remain; pre-fix full41911 cannot qualify this fix.

L19 overlapping text/image corpus adds16 scenes/48 Chrome153.0.8010.12 views. All48 have visible inner/sibling intersection;24 auto/zero pairs have identical geometry and distinct Chrome pixels (stacking-v19-overlap-oracle/discriminator.json). Public geometry now96 scenes/2304 original-clone lifecycle updates passes. Native/vector replay52624/46455 and expanded parity running; visual review pending. Corrected frozen full6223 native gate launched96889 in stacking-v19-clip-full-native, independently of pre-fix41911.

L19 overlap native visual audit12/48 complete: forward-row text/image auto/zero pairs at all3 widths. Auto exposes full text/quadrants, zero green occludes most content; Chrome/native edges and text wrapping match. Remaining36 native and48 vector views; parity82349 confirmed live. Corrected full96889 and pre-fix41911 confirmed live. Numeric overlap48/profile and2304 public lifecycle updates already pass.

Correction to preceding live note: pre-fix baseline41911 returned terminal success in this turn:6223/6223 passed in28.5m. Full visual audit remains pending, and this tie snapshot does not include the clip lifecycle fix. Corrected full96889 remains running.

L19 overlap native visual audit24/48 complete: all forward/reverse row pairs reviewed. Reverse zero context hides visible text/image except top strip, auto reveals viewport-cropped content; Chrome/native agree.24 column native views and48 vector reviews remain. Expanded accepted native/WASM corpus passes; initial JS run10pass/3host-path ENOENT, targeted retry with explicit frozen NUXIE_NATIVE_PROBE passes3/3. Both logs preserved; all13 tests accounted for without rerunning passing corpus. Corrected full regression remains pending.

L19 overlap native visual review36/48 complete and audited. Forward-column auto shows full text/image above green, zero occludes same regions as Chrome, including thin exposed strips. Final12 reverse-column native views and48 vector reviews remain. Corrected full96889 confirmed live this turn; dynamic clip pixel validation remains open.

L19 overlap native visual review48/48 complete and audited. Reverse-column auto/zero text and image occlusion matches Chrome at all widths. Vector27/48 exact canonical-input/full-image transfers audited;21 text views still need review. Full96889 confirmed live. Dynamic clip pixels and full regression audit remain before qualification.

L19 overlap vector review now36/48 covered:27 audited exact-input/full-image transfers plus9 directly inspected row text views. Wrapping, baseline, viewport crop and green occlusion match Chrome; sparse colored glyph-edge differences remain visible within unchanged thresholds.12 column text views remain. Full corrected regression and dynamic clip pixel gates remain pending.

Stacking overlap audit completed: all 48 native views directly reviewed; vector coverage is 21 directly reviewed views plus 27 audited exact compiler-input/full-PNG transfers, with disjoint complete coverage recorded in `output/playwright/html-to-riv/stacking-v19-overlap-vector/visual-coverage.json`. This brings focused stacking comparisons to 288 per renderer profile, all passing with audited visual coverage. Sparse vector glyph-edge differences remain visible within unchanged gates. Dynamic clip pixel qualification and the corrected full regression remain pending; L19 stays active.

Dynamic stacking clip pixels: added `examples/stacking_clip_probe.rs` and `validation/stacking-clip-lifecycle.mjs`. Twelve public compiler scenes now produce 864 sequential original/clone draw frames, cycling both ancestor clips and widths, including initially unpainted/unclipped hosts. Recording v3 adds required explicit frame boundaries; earlier borrow/recording failures are retained. Chrome/real renderer run `stacking-v19-clip-lifecycle-pixels-v2` is active (session 67980). This is new validation infrastructure, not qualification: terminal comparisons, discriminators, new-request parity and visual inspection remain. The corrected full6223 regression remains independently active (session96889).

Dynamic stacking clipping now passes all864 real renderer/Chrome geometry and pixel comparisons over12 public compiler scenes, original/clone instances, both ancestor clip toggles and repeated widths. All60 distinct pairs were directly inspected in20 contact sheets;804 additional pairs have audited exact full-image transfers. All720 repeated/clone states are pixel-identical, and144 host/parent clip discriminators show changed pixels in both renderers. New-request native/WASM byte/map/requirements parity passes12/12 (initial CLI-only languageVersion argument error preserved). Evidence: `output/playwright/html-to-riv/stacking-v19-clip-lifecycle-pixels-v2/{replay,visual-inspection,lifecycle-audit}.json` and `stacking-v19-clip-lifecycle-parity-v2/receipt.json`. This qualifies the rectangular shape clip lifecycle interaction only; broader L20 rounded/text/image overflow remains separate. The corrected full6223 gate and its visual audit remain pending; L19 stays active.

The completed pre-clip stacking full baseline now has6213/6213 scene pairs audited against complete prior reviews with exact complete compiler-input and full browser/native PNG identity; all6223 numeric checks passed. Reusable audit: `validation/audit-gallery-transfer.py`; evidence: `output/playwright/html-to-riv/stacking-v19-full-native/baseline-comparison.json`. The corrected clip snapshot is still running and must pass its own audit. Separately prepared27 prospective L20 rounded overflow fixtures; no rounded-overflow qualification is claimed.

L20 initial rounded overflow:27 scenes/81 Chrome comparisons pass both renderer profiles;216 original/clone repeated-size geometry checks and27 native/WASM parity cases pass. Native visual audit covers36/81 views (18 direct plus18 identical hidden controls), including rounded shape/image boundaries and text clipped through glyphs. Remaining45 native views and vector audit remain; no full L20 qualification claimed. The first resize test used an absolute-policy installer for a relative-only contract; corrected to the shipping probe relative installer, preserving the failed log. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L19 qualification completed for the documented subset: corrected full6223/6223 checks and6213/6213 visual transfers pass; regular overlap48/48 checks and visual transfers pass. Focused288/profile and dynamic clip864 frames retain complete visual evidence. See `output/playwright/html-to-riv/stacking-v19-qualification.json`. Broader overflow continues under L20; unrelated vector text limitations remain tracked.

L20 initial visual audit complete in both profiles:81/81 comparisons each. Native54 direct+27 exact hidden-control transfers; vector18 direct+9 within-run transfers+54 audited exact-input/full-PNG cross-run transfers, with disjoint coverage checked. All108 profile-specific visible-versus-clip/hidden discriminators preserve geometry and change pixels. Sparse vector glyph/corner antialiasing differences remain within unchanged gates. Initial27-request parity and216 clone/resize geometry checks pass. L20 remains active for nested/positioned compositions, CSS-wide resets and axis/clip-margin investigation. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested expansion:24 scenes/72 comparisons pass each renderer profile;24 new public native/WASM parity cases and combined51-scene/408 original-clone resize checks pass. Native visual coverage is36/72 after inspecting absolute underlined text clip/initial and relative image unset across all widths. Remaining visual review and CSS-wide discriminators are open. Original br fixture rejected by the documented block-context rule; corrected display:block oracle and failed runs retained. Added reusable `validation/parity-fixtures.mjs`. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested audits complete:72 views/profile, native18 direct+54 exact image transfers; vector6 direct+18 within-run exact transfers+48 exact-input/full-PNG transfers. All36 profile-specific reset groups verify identical geometry, inherit=clip, initial=unset and changed pixels when the inner clip is released. Combined focused coverage is153/profile;51 public parity cases and408 clone/resize checks pass. Axis investigation captured18 prospective scenes/54 pinned Chrome views; all18 inputs are explicitly rejected by the current compiler. Runtime currently has only a two-axis rounded clip path; axis policy/transport and clip-margin support remain implementation work, not external blockers. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 axis foundation implemented: compiler overflow values now retain specified x/y states and compute their coupling, preserving hidden versus clip and distinguishing computed auto. Existing admission is unchanged until runtime support lands. All333 compiler tests across63 result groups pass; native build passes. New native outputs for51 reviewed overflow inputs are byte-identical to previous Rive/map/requirements artifacts. WASM build56207 is in progress. Renderer current device clip bounds provide a bounded exact half-plane intersection path without an arbitrary visible-axis extent; render API/replay/backend/policy/transport work remains.

Axis value-model verification completed: WASM build passes and all51 overflow inputs pass native/WASM byte/map/requirements parity on the new compiler snapshot (`overflow-axis-model-compiler`). No public axis syntax has been admitted yet.

L20 axis renderer implementation added: optional clip_axis API, precise recording and typed replay command, glyph-adapter forwarding, and Metal device-bound half-plane clipping. Geometry tests3/3 pass. Metal-feature test20200 and stream test40563 remain running; real pixel validation and runtime/compiler policy installation remain pending. No new public axis syntax is admitted.

Axis renderer verification: Metal-feature build and3 geometry tests pass; full render-stream suite7 tests passes, including precise axis round-trip, invalid-input rejection and unsupported-backend error. Actual pixel controls remain next.

Axis renderer frame wiring and pixel controls: the first stream run failed explicitly with UnsupportedOperation("clipAxis") because NativeMetalFrame lacked forwarding. Added forwarding in NativeMetalFrame and its canvas wrapper; the rebuilt frozen renderer completes144 Chrome153.0.8010.12 controls.108 pass;36 rotated/sheared x/y/both clips fail unchanged thresholds. Translation, fractional translation, scale, reflection and all unclipped controls pass. Rotated x/integer and unclipped rotation sheets directly inspected at all3 widths: native clip exposes excess teal while uncut transform and restored purple agree. This is renderer-only diagnostic evidence, not public axis syntax admission or responsive compiler qualification. Preserve both failed runs. Next: diagnose affine clip polygon/backend consumption before runtime/compiler policy integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-frame/failure-audit.json.

Axis renderer affine failure fixed: diagnostic output showed initial overallClipPixelBounds uses i32 sentinel extremes. Polygon intersections therefore reached billions of pixels, losing ordinary-edge precision on conversion to f32. Intersecting existing bounds with actual frame dimensions before constructing the strip fixes all36 failures. Fresh frozen renderer passes144/144 Chrome controls, with unchanged thresholds; Metal-feature geometry tests3/3 pass. Nine rotated integer x/y/both views directly inspected: clipped bounds and restored purple agree, sparse slanted-edge antialiasing remains.135 visual views remain unaudited. Original failure corpus/debug output retained. Public axis syntax remains rejected pending runtime policy, compiler/host transport, clone/resize and public parity qualification. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/replay.json and overflow-l20-receipt.json.

Axis renderer visual coverage now36/144: all rotated/sheared x/y/both integer/fractional views directly inspected and audited at three widths. Bounds, viewport crops and restored purple match Chrome; thin slanted-edge antialiasing differences remain within unchanged gates. New validation/audit-axis-renderer.py verifies432 distinct-axis pair comparisons across36 groups and288 restored-region controls against current PNG hashes. These controls prove axis differences and save/restore isolation; they do not replace remaining108 visual reviews or public compiler integration. Evidence: overflow-axis-renderer-finite-frame/{visual-inspection,control-audit}.json under output/playwright/html-to-riv.

Axis renderer visual audit now78/144 after directly inspecting translation and scale X/Y/both integer/fractional sheets at all widths. Placement, visible extents and restored purple agree with Chrome; fractional right/bottom edges retain thin coverage differences under unchanged thresholds. Coverage includes exact within-run full-image transfers recorded by the review tool, not additional direct inspection claims.66 views remain. Numeric144/144 and control audits432 axis distinctions/288 restored regions remain passing. Public compiler axis admission is still pending runtime integration. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/visual-inspection.json.

Axis renderer controls complete:144/144 Chrome comparisons pass, with audited visual coverage (120 directly inspected views and 24 exact full-image transfers). Final fractional-translation/reflection clips and all unclipped controls agree in extent/cropping/restore; thin fractional edge coverage differences remain within unchanged gates.432 axis-discrimination and288 restored-region checks pass. This qualifies the direct Metal renderer diagnostic matrix only. Runtime/public compiler axis syntax remains pending. Shared integration must cover both LayoutComponent.draw_proxy and begin_css_ancestor_clip, which currently read base.clip and clip a world rounded path. Live dimensions, transform restoration, clone policy and plain-container proxy inclusion must all be preserved. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/{replay,visual-inspection,control-audit}.json.

Axis runtime integration foundation: ordinary draw_proxy and deferred begin_css_ancestor_clip now share begin_layout_clip, preserving caller-owned save/restore and ClipSaved bookkeeping. Positioned-paint regression5/5 passes, including864 original/clone dynamic clip/resize draw frames. Initial cargo package-name typo failed before tests and is preserved; corrected nuxie-html-to-riv run passed. No new overflow syntax admitted. Next: occurrence policy and transform-aware strip dispatch, with proxy/path invalidation, clone handling and checked compiler/host transport. Existing direct-renderer144-view qualification remains separate from this runtime integration. Evidence: output/playwright/html-to-riv/overflow-shared-clip-runtime-tests-corrected.log.

Transform-aware axis renderer API added: clip_axis_transformed composes an additional local matrix for clipping while restoring the exact prior drawing matrix, avoiding inverse-transform roundoff. Recording/replay accepts optional finite clipAxis matrix; adapter and Metal frame/canvas forwarding included. Expanded render-stream7/7 passes. Fresh local-transform mode completes144/144 Chrome pixel controls; complete authored HTML/CSS and browser/native PNG byte identity transfers all144 reviews from the audited finite-frame run. Control audit432 distinctions/288 restored regions passes. No public compiler axis syntax admitted. Runtime occurrence policy and clone/resize/host integration remain next. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-transformed/{replay,control-audit,visual-identity-transfer}.json.

Live runtime axis policy implemented: LayoutComponent stores optional CssOverflowAxis (horizontal/vertical), clones it, keeps a drawable proxy through toggles, and computes each strip from current layout width/height and world transform. The shared ordinary/deferred clipping helper dispatches clip_axis_transformed; clearing restores the imported two-axis clip flag. An unsupported renderer fails explicitly rather than drawing without the requested clip. Positioned-paint6/6 passes, including32 new original/clone frames with axis switches, repeated240/390/768/240 sizes, clear/reinstall, deferred clips and sibling restore isolation; prior864 dynamic two-axis frames remain covered. This is runtime stream/lifecycle evidence, not public CSS or runtime pixel qualification. Atomic checked installer, versioned compiler/host transport, public admission/parity and actual runtime pixel corpus remain. Evidence: output/playwright/html-to-riv/overflow-axis-live-runtime-tests.log.

Axis overflow checked scene installer added: Artboard.set_css_overflow_axes_occurrence validates the root, all object IDs/types and duplicates before mutations, clears omitted overrides back to imported clipping, releases the artboard borrow during per-layout invalidation, and rebuilds draw order. Positioned-paint7/7 passes. New regression proves invalid lists retain prior policy, clearing restores the imported clip flag and clipPath drawing, clones retain their own axis, and clone replacement leaves original cleared. Existing32 axis lifecycle and864 ordinary clip frames remain covered. Versioned compiler/host transport, public syntax and runtime pixel qualification remain open. Evidence: output/playwright/html-to-riv/overflow-axis-installer-runtime-tests.log.

Axis overflow contract foundation added: version20, layout-css-axis-overflow-v1 and strict layout_axis_overflow entries {object_id,axis:x|y}. Validation enforces version/capability coupling, unique non-root layout targets and optional stacking coexistence; empty entries are omitted from older outputs. Public Rust exports and TypeScript version union updated. Native probe maps requirements through the checked runtime installer. Focused2/2, full compiler337 tests across64 groups and TypeScript checks pass. Featured probe build25280 is running; initial mistaken example target failed before building and is preserved. Public CSS axis syntax/emission, WASM/parity and runtime pixel qualification remain pending. Evidence: output/playwright/html-to-riv/overflow-axis-contract-{tests,full-tests,types-final}.log.

Axis version20 featured probe build25280 completed successfully. Public CSS admission and runtime scene qualification remain pending.

Public axis CSS parsing/emission implemented. overflow accepts1/2 values and overflow-x/y one; visible/clip/hidden, normal precedence, CSS-wide keywords and supported custom substitutions covered. Inherit copies computed values. Final visible/hidden coupling rejects computed auto; clip/visible emits version20 axis requirements, both-axis pairs use existing clipping. Focused6 tests pass including prior clipping regressions; old axis rejection cases replaced with unsupported scrolling pairs. Initial new test diagnostic-vector access compile error preserved. Full compiler84510 is running. New support section explicitly marks public parity/runtime scene geometry/pixels pending, separate from direct renderer qualification. Evidence: output/playwright/html-to-riv/overflow-axis-public-tests-corrected.log.

Initial public full suite84510 failed on the intentionally obsolete hidden-clip custom-property rejection. Updated it to reject hidden-visible and assert hidden-clip equals hidden; full rerun40314 active, no terminal passing claim.

Public axis overflow initial corpus passes: corrected full compiler339 tests/65 groups, fresh native/WASM builds,14 public parity cases,42 Chrome geometry/pixel comparisons per renderer profile, and combined65-scene520 original/clone resize geometry checks. Four prospective visible/hidden scenes remain intentionally rejected for computed auto. Initial replay failed on missing copied oracle PNGs; exact42 reference files copied with hashes and fresh native-images output passes. Twelve one-axis native views directly inspected: responsive clip extents, visible-axis escape, viewport crop and square clip over rounded orange background match Chrome.30 native visual reviews and vector audit remain, followed by composition/host/full regression expansion and clip margins. Evidence: output/playwright/html-to-riv/overflow-axis-public-{native-images,vector,parity} and overflow-l20-receipt.json.

Public initial axis corpus visual review complete:42 native views (24 direct,18 exact within-run image transfers) and42 vector exact compiler-input/full-PNG transfers audited. New48-scene composition matrix covers shape/underlined multiline text/image, relative/absolute children, X/Y clips, painted/plain hosts and outer rounded clip/visible controls with positioned sibling overlap. All144 Chrome comparisons pass per renderer profile,48 new native/WASM parity cases pass, and combined113-scene904 original/clone repeated-size geometry checks pass. Composition visual review/discriminator audits remain pending; no full L20 qualification claimed. Evidence: output/playwright/html-to-riv/overflow-axis-composition-{native,vector,parity,oracle} and overflow-axis-composition-resize.log.

Axis composition review now42/144 native views (12 direct plus exact full-image transfers) after unpainted text/image X/Y sheets inspected. Text line/underline clipping, quadrant cropping, outer rounded bounds and green sibling overlap agree with Chrome. Outer-clip audit found original X cases mostly fit outer height, so they prove placement rather than nested clipping. Preserved them and added24 short-outer X variants. New72 comparisons/profile and24 public parity cases pass;72 paired scene groups verify144 browser/runtime image changes across the outer clip toggle. Combined137-scene1096 original/clone size checks pass. Short-outer visual review and remaining composition native/vector review still pending. Evidence: output/playwright/html-to-riv/overflow-axis-{composition,shortouter}-discriminators.json and overflow-axis-shortouter-{native,vector,parity}.

* 2026-09-10: L20 clip-margin research captured48 scenes/144 Chrome references and preserved48 current admission rejections. Two-axis versus single-axis/hidden discriminators established; implementation pending. See validation/overflow-clip-margin-investigation.md. Full proxy regression gallery continues in session18728.

* 2026-09-10: L20 internal clip-margin value parser added;2 focused tests pass. Public admission remains disabled pending live clip bounds, transport and qualification. Frozen full gallery continues independently in session18728.

* 2026-09-10: L20 live clip-margin bounds resolver added with finite offset/edge checks and current asymmetric padding. Runtime bounds tests2/2 pass; draw-path, rounded geometry and checked compiler transport remain. Public admission unchanged. See validation/overflow-clip-margin-investigation.md.

* 2026-09-10: L20 separate elliptical clip path, clone/clear policy and background-before-clip ordering implemented internally; geometry5/5 and integration9/9 pass. Pinned Chrome coverage-factor rule verified. Runtime pixel experiment preparation active; public CSS admission/transport still pending.


Initial clip-margin runtime experiment completed:144/144 Chrome geometry/pixel comparisons per renderer profile pass, with complete native visual audit and144 exact-input/full-PNG transfers to the explicit vector-v2 run. Runtime injection and original browser CSS are checked before review transfer. Sources and images remain recorded as experiments, not public compiler qualification. The first vector-named run accidentally used nativeGlyphs=1; preserved as a native repeat and excluded from vector evidence. See overflow-l20-receipt.json.clipMarginRuntimeExperiment. Public admission, checked transport, parity and broader live composition remain.

* 2026-09-10: Public clip-margin v21 candidate passes compiler348/host19/types/parity48 and1544 overflow clone/resize updates;144 comparisons per profile and promotion visual audits complete. Expanded20-scene cascade/small-radius Chrome oracle running in session79781. Prior proxy full6271/visual6261 completed. L20 remains active for expanded/live composition and v21 regression.

* 2026-09-10: Expanded clip-margin corpus57/60 pixels reveals Chrome153 accepts signed margins; initial nonnegative assumption corrected after direct/computed-style probes. Source now admits finite signed lengths; tests running, signed rebuild/parity/pixels pending. Failing three views preserved.18 additional contraction/empty-clip fixtures prepared.

Signed-margin focused compiler/contract/runtime tests6/6 pass. Native featured build and18-scene Chrome capture are running; signed native/WASM parity and pixels remain pending.


Signed-margin correction validation: native/WASM build and38 expanded/signed parity cases pass; host19 passes. Both renderer profiles pass all54 direct-negative comparisons and60 expanded comparisons, including the three preserved negative-variable reproducers. The public overflow test now covers231 scenes/1848 original-clone resize updates; the margin lifecycle test covers192 positive/negative/empty-clip frames with background and deferred-descendant clip-state assertions. Native visual audit currently covers27/54 signed and21/60 expanded views; remaining review is open. Frozen toolchain: overflow-clip-margin-signed-toolchain. Evidence: overflow-l20-receipt.json.clipMarginSignedCorrection.


2026-09-10 continuation: Signed-margin visual review is complete for54/54 native views, with54 audited exact-input/full-PNG transfers to the vector profile. The current signed toolchain also passes144/144 initial-corpus regression comparisons. New72-scene shape/text/image compositions cover relative/absolute descendants, clipped/visible outer ancestors, content/padding origins and offsets-8/0/24; native/WASM parity72 and216/216 geometry/pixel comparisons per profile pass. Composition visual review remains pending. The public original/clone resize test now includes303 scenes and2424 updates and passes; the earlier stale expected-count assertion failure is preserved separately. Expanded visual review, live same-scene pixel qualification and fullv21 regression remain open; L20 is still active. Evidence: output/playwright/html-to-riv/overflow-l20-receipt.json.clipMarginSignedCorrection.


2026-09-10 lifecycle follow-up: The clip-margin lifecycle test now compiles the public CSS property and checks its emitted manifest before applying the checked runtime installer. Both lifecycle/atomicity tests pass. Optional NUXIE_CLIP_MARGIN_RECORDING emits192 sequential original/clone frames with repeated240/390/768/240 resizing and clear/reinstall controls; validation/clip-margin-lifecycle.mjs compares these streams with the equivalent live Chrome DOM. Pixel comparison is running (session82244), not yet qualified. The full frozenv21 native regression is running separately (session52141). First composition review covers3/216 views; the remaining213 are open. Runtime policy transitions are validation controls, not support for author scripting or interactions.

Lifecycle pixel run completed:192/192 sequential original/clone geometry and native pixel comparisons pass against pinned Chrome153. Visual review remains pending; fullv21 regression continues in session52141.

2026-09-10 expanded clip-margin audit: all60 native views now have complete direct/exact-image visual coverage; all60 vector views have audited exact-source/full-PNG transfers. Inspected cases include border-box default/expansion, declaration comments/order, final-font em computation, inherit, content-box origin and intermediate radii. Sparse corner antialias differences are preserved under existing thresholds. Signed54/profile and expanded60/profile visual audits are complete; composition and lifecycle reviews plus initial signed review transfer and fullv21 regression remain pending.

2026-09-10 visual continuation: initial signed-toolchain regression144/144 now has an audited promotion from the directly reviewed runtime corpus, checking unchanged source CSS, matching public policy and exact full PNG pairs. Composition review covers21/216 views, including underlined text and images crossing expanded/contracted and ancestor clips. Lifecycle review covers96/192 frames after direct inspection of12 unique square-host image pairs and exact-image transfers across repeated original/clone frames. A thin host right-edge rasterization difference at390px also appears in clear controls and remains under unchanged thresholds. Rounded lifecycle96 and composition195 views remain open; fullv21 regression remains running in session52141.

2026-09-10 lifecycle visual audit complete:192/192 original/clone frames now covered by28 directly inspected unique Chrome/native pairs and164 exact-image transfers. Rounded and square hosts preserve background/siblings through clear/reinstall and empty clips. Native PNG hashes are identical for every repeated logical state across original/clone and resize cycles. The lifecycle runner now asserts that invariant and writes repeat-stability.json; validation rerun session14866 is in progress. Fullv21 regression session52141 remains active; composition195 views remain unreviewed.

Repeat-stability runner validation passed:192/192 Chrome geometry/pixel comparisons;36 distinct logical states and156 repeated frames have identical native PNG hashes. Receipt: overflow-clip-margin-lifecycle-stability/repeat-stability.json. The earlier directly reviewed lifecycle run remains the visual source.

2026-09-10 composition audit: all12 absolute-image cases now visually covered at all three widths, including content/padding origins, contracted/default/expanded margins and clipped/visible ancestors. Overall composition coverage is51/216 with165 remaining. Matching image bounds, quadrant boundaries, sibling overlap and ancestor clipping were inspected; sparse corner rasterization differences remain visible within unchanged thresholds. Fullv21 regression session52141 continues.

2026-09-10 relative-image review complete: all24 image composition scenes (72 views) now visually covered, including relative/absolute placement, content/padding margins and clipped/visible outer ancestors. Overall composition review is84/216 with132 remaining, now text and solid-shape cases. Fullv21 regression session52141 remains active. No tolerance changes.

2026-09-10 absolute-text audit: all12 absolute underlined-text composition scenes now reviewed at all three widths. Glyph/underline clipping, expanded lower lines and ancestor clipping match Chrome within unchanged thresholds. Overall composition review is114/216, with102 remaining. Relative text and solid-shape cases remain; fullv21 regression session52141 is still running.

2026-09-10 relative-text audit complete: all24 text composition scenes (72 views) now visually covered. Relative/absolute glyph fragments and underlines match Chrome across content/padding margins and outer clipping. Overall composition review is147/216 with69 solid-shape views remaining. Fullv21 regression session52141 remains active. No tolerance changes.

2026-09-10 solid-shape audit complete: all216 native composition views now have audited visual coverage. Contracted/expanded contours, relative and absolute descendants, ancestor clipping, host paint and siblings match Chrome within unchanged thresholds. Vector composition216/216 geometry/pixel checks pass;144 shape/image views have audited exact-source/full-PNG review transfers. The72 vector text views need separate visual inspection. Fullv21 regression session52141 remains live (latest observed5348 checks).

2026-09-10 vector composition continuation:18 absolute-text/outer-clip views directly inspected. Glyph and underline cutoffs match Chrome; vector glyph-edge raster differences remain visible under unchanged passing gates. Audited combined coverage162/216 (18 direct,144 exact-source/image transfers,zero overlap);54 text views remain. Native composition audit is complete216/216.

2026-09-10 composition visual qualification complete:216/216 views per profile pass geometry/pixel checks and now have complete audited visual coverage. Vector review combines72 directly inspected text views and144 exact-source/full-PNG shape/image transfers with zero overlap. Relative/absolute text, underlines, signed content/padding margins, ancestor clipping and visible-overflow sibling composition were inspected at240/390/768px. Glyph-edge raster differences remain visible within unchanged thresholds. Fullv21 regression session52141 remains live; full regression audit is pending.

2026-09-10 signed version21 full tests complete: compiler348 tests across68 result groups pass (session76332 exit0), and full Chrome/native6271 checks pass in24.3min (session52141 exit0). Full regression exact-input/full-PNG visual audit is running in session17462 against overflow-axis-proxy-full-native; do not claim full visual qualification until that audit completes. Focused composition216/profile and lifecycle192 visual audits remain complete.

2026-09-10 full signed-version21 visual audit complete:6271 checks passed, and all6261 scene pairs have exact complete-input and full Chrome/native PNG identity with the audited axis-proxy baseline. Audit session17462 exited0. All303 accepted overflow scenes are now included in the normal browser regression gate (909 additional views); four prospective axis pairs computing to auto remain intentionally rejected and covered by compiler tests. The regular-suite integration run is active in session14833, with its own image audit pending.

2026-09-10 L20 qualified for the documented static-overflow subset: regular-suite930/930 checks pass and all930 full-input/full-PNG review transfers audit successfully (session61601 exit0). Together with6271 full regression checks/6261 reviewed pairs, focused renderer audits, public parity,2424 resize updates and192 live clip-margin frames, this closes L20. Scrolling remains excluded and nonzero borders must extend/requalify clipping in P01.

2026-09-10 border width resolution:42 computed-style observations captured with pinned Chrome153 at page zoom1 and DPR1/2. Both DPRs compute positive subpixel widths as1 CSS px, floor larger fractional widths, and map thin/medium/thick to1/3/5px. The internal resolver now matches all30 width observations using final-font em and fixed16px-root rem; three unit tests pass, including invalid/overflow lengths. Reset controls confirm border-style:solid alone leaves0px, border:solid resets to3px, and border:8px without style computes0px. This does not qualify border painting or admit public border CSS. Evidence: output/playwright/html-to-riv/border-width-resolution-receipt.json.

2026-09-10 border shorthand foundation: internal Border/BorderStyle/BorderColor types now parse a uniform width, none/hidden/solid style and supported sRGB color in any component order. Omitted shorthand components reset to medium/none/currentColor; the authoring reset border:0 remains distinct. currentColor stays symbolic, and none/hidden use zero width without discarding specified width. Six focused tests pass covering permutations, comments/function colors, duplicates, malformed/unclosed/nested functions and existing Chrome width observations. CSS-wide cascade, public admission, runtime geometry and border painting remain outstanding. Evidence: output/playwright/html-to-riv/border-shorthand-receipt.json.

2026-09-10 border internal cascade: Style now carries Border values and applies shorthand/longhands with CSS-wide inherit/initial/unset. Width resolves after the final font pass; inheritance copies computed pixels (including zero under none/hidden), while currentColor remains symbolic. The reset remains border:0 and borders do not inherit by default. Nine focused tests pass. Public validation explicitly rejects all four border properties, including initial/variable forms, pending rendering. Full compiler regression is running (border-cascade-full-tests.log). Invalid-variable classification, runtime border edges and painting remain pending.

2026-09-10 border variable invalidation: internal cascade now resets definitely invalid border substitutions (negative/nonlength widths, duplicate components and invalid colors) while retaining diagnostics for valid unimplemented CSS such as dashed style, side lists and math. Supported RGB/HSL variable values retain their color component. Ten focused tests pass. Prior full cascade suite357 tests/68 groups passed; custom-property regression is running after this validator extension. Public border admission remains gated. Logs: border-variable-tests.log and border-variable-regression.log in output/playwright/html-to-riv.

Border variable regression completed:51/51 custom-property tests pass. Receipt: output/playwright/html-to-riv/border-variable-receipt.json.

2026-09-10 runtime border-edge foundation: layout solver output now retains physical border edges alongside padding, detects edge changes as new layout, publishes both on ordinary/occurrence updates, and clears solved edges with layout resets. Opt-in clip-margin bounds now distinguish border, padding and content origins with signed offsets. A focused test covers asymmetric borders, padding and repeated sizes; runtime test session66021 is running. Ordinary content measurement, hidden/axis clipping, border ring painting and public transport/admission remain pending. Evidence: output/playwright/html-to-riv/border-runtime-edge-receipt.json.

Runtime border-edge tests completed:7/7 pass. Existing overflow resize and clip-margin lifecycle regression now running in session64249.

2026-09-10 border runtime policy: opt-in border geometry now subtracts used borders from inner dimensions with a zero floor; one-axis clips use padding-box intervals; default two-axis clips use inset rounded paths. Clip-margin bounds and draw paths share the same border/padding/content inset calculation. Box paint precedes its inner descendant clip. Policy cloning/clearing and collapsed dimensions have a focused test; session24081 is running. Earlier edge-retention overflow regression passed3 tests (2424 resize updates plus margin lifecycle assertions). Public host transport, compiler emission, measured content origins and border painting remain pending.

2026-09-10 runtime policy tests pass8/8. Added a54-scene diagnostic layout integration test using schema border-width injection into borderless compiled scenes; it compares every source box with independent Chrome references across original/clone and240/390/768/240 resize cycles (432 updates). Session97250 is running. This isolates runtime layout support and cannot qualify public compiler emission or pixels. Text origins need no separate padding adjustment for layout participants: their solved locations already come from the layout engine; richer text/image cases remain to verify.

Diagnostic border layout result:54/54 scenes and432/432 original/clone resize updates match Chrome source geometry. Session97250 exited0. Public border emission and rendered border pixels remain unqualified.

2026-09-10 border ring geometry: added an even-odd outer/inner contour builder using live border widths. Outer radius normalization precedes inset subtraction; zero widths yield an empty ring, collapsed inner boxes retain the outer fill, and nonfinite/negative widths reject. All5 css_clip_path tests pass, including2 ring-specific checks. Drawing, alpha/overlap validation and Chrome pixels remain pending; this is geometry evidence only. Receipt: output/playwright/html-to-riv/border-ring-geometry-receipt.json.

2026-09-10 border runtime drawing connected: optional border color paints the live rounded ring with even-odd fill, inherited render opacity and blend mode, after background and before descendant clipping. Color policy survives cloning; render paths/paint are allocated lazily. The diagnostic54-scene test draws all432 original/clone resize frames and confirms border draw presence while retaining exact Chrome geometry comparisons. Test passes. Rendered pixel/alpha/clipping validation, border-only proxy installation and checked public transport/emission remain pending. Evidence: output/playwright/html-to-riv/border-runtime-draw-receipt.json.

2026-09-10 border sequential pixel validation started: diagnostic test recorded432 original/clone resize frames successfully. The lifecycle runner now supports an explicitly labelled border-diagnostic recording, preserving original browser CSS and injected width/color separately from compiler request CSS. Native Metal replay/Chrome geometry and pixel comparisons plus repeated-state hash checks run in session17183. Initial89 frames pass numeric gates; no complete passing or visual-review claim yet. First1px/18px-radius content-box clip browser/native pair inspected with matching visible border and child clipping. Evidence: output/playwright/html-to-riv/border-initial-pixel-receipt.json.

2026-09-10 border initial native replay complete:432/432 diagnostic geometry/pixel comparisons pass, with162 distinct states and270 repeated original/clone states rendering byte-identically. Direct inspection of12 thick-border views (radii18/90, content-box/border-box, all widths) plus exact-image transfers gives audited64/432 visual coverage;368 remain. Square inner corners when thickness exceeds radius and normalized capsule contours match Chrome; sparse outer-edge raster differences remain visible under unchanged thresholds. Public emission, broader alpha/border-only compositions and full visual qualification remain pending. Receipt: output/playwright/html-to-riv/border-initial-pixel-receipt.json.

2026-09-10 border visible-overflow review: six thick rounded overlap views inspected across both sizing modes and three widths. Child paints over border/sibling as Chrome does; audited review coverage80/432,352 remaining. Added24 prospective paint cases covering zero/half alpha, transparent/opaque background, absent/present child and radii0/18/90. Chrome capture session2123 is running; runtime replay for this expansion remains pending.

Paint expansion Chrome capture completed:24 scenes/72 references on Chrome153.0.8010.12. Runtime admission, draw and pixel checks remain pending.


### Border transparency runtime evidence — 2026-09-10

The24-scene alpha/background/child matrix now passes192/192 Chrome/native geometry and pixel comparisons across original/clone resizing at240/390/768/240. Four full-resolution sheets were inspected directly; exact paired-image identity extends coverage to8/192 frames, leaving184 unreviewed. Inspection confirms half-alpha ring blending, empty transparent interiors, background beneath transparent borders, and rounded inset child clipping. Thresholds are unchanged. These are injected runtime experiments, not public compiler support; plain unclipped border-only proxy installation and public transport/parity remain outstanding. Receipt: `output/playwright/html-to-riv/border-paint-pixel-receipt.json`.


### Checked border occurrence installation — 2026-09-10

Added atomic `Artboard::set_css_borders_occurrence`, sharing late drawable-proxy installation with axis overflow. Plain nested unclipped border-only boxes now receive proxies; repeated installation avoids duplicate draws, invalid IDs leave policies unchanged, and clearing an original leaves its clone independent. All3 border runtime tests (including both Chrome geometry corpora through this installer) and8 positioned/axis/stacking runtime regressions pass. This does not establish pixel qualification for the new installer or public compiler support. The previous injected pixel recordings remain historical evidence. Receipt: `output/playwright/html-to-riv/border-installer-receipt.json`.


### Checked installer pixel coverage — 2026-09-10

The checked border installer passes192/192 transparency frames; exact HTML/CSS, injected values and both full PNG hashes match the prior injection recording for every frame (`border-installed-paint-native/installer-comparison.json`). This identity check does not claim completed visual review. A new18-scene plain border-only matrix has no background or overflow clip, covering widths1/8/24, radii0/18/90 and alpha128/255. All144 original/clone resize frames pass Chrome153 geometry and native pixel gates. Four original-resolution sheets were inspected; audited direct/exact-image coverage is10/144, with134 remaining. Thin/thick rings, normalized corners, empty interiors and sibling placement agree, with retained curve antialias differences under unchanged gates. Receipt: `output/playwright/html-to-riv/border-plain-pixel-receipt.json`. Public compiler emission, checked format/capability transport, Rust/WASM/JS parity, broader compositions and complete visual qualification remain open.


### Border requirement transport — 2026-09-10

Added runtime requirements version22, capability `layout-css-solid-borders-v1`, and strict `layout_borders` entries containing non-root object IDs and packed ARGB u32 colors. Widths remain live Rive layout-style data. Validation requires unique targets and matching capability/version, rejects malformed colors/fields and checks imported target types; borders can coexist with version21 clip-margin policies without relaxing legacy validation. Seven focused Rust border/clip-margin/axis contract tests and the TypeScript API check pass. The TypeScript union now represents version22 and optional earlier policies. This contract is not yet emitted by the compiler or installed from a public manifest; CSS border admission stays gated and native/WASM/public pixel qualification remains pending. Receipt: `output/playwright/html-to-riv/border-contract-receipt.json`.


### Public border emission — 2026-09-10

Compiler admission now emits uniform used physical border widths into Rive layout styles and version22 ARGB border requirements, including transparent solid borders. Border paint targets also request CSS pixel bounds. The validation probe checks the border capability and imported target types, then invokes the checked installer; `NUXIE_DISABLE_CSS_BORDERS=1` removes host support. Public compilation of all96 initial solid/transparency/plain scenes passes768 original/clone resize geometry updates. The full module suite passes366 tests across70 result groups; the obsolete gate-closed unit test was replaced with declaration acceptance coverage, with the original failed run retained. Public pixel qualification, native/WASM parity, host rejection coverage and richer compositions remain pending. Receipt: `output/playwright/html-to-riv/border-public-emission-receipt.json`.


### Version22 native/WASM and public pixels — 2026-09-10

Fresh native and WASM builds are frozen in `output/playwright/html-to-riv/border-v22-toolchain`. All13 JavaScript tests pass; the accepted-corpus parity test now includes all96 border fixtures and checks exact Rive bytes, source maps and runtime requirements. The version22 host contract test passes valid paint and rejects disabled capability, malformed color/version/fields and invalid/duplicate/root targets before writing draw streams. Public compiler/probe/renderer replay passes54/54 plain and162/162 solid-border Chrome geometry/pixel comparisons. Three plain public comparisons have audited direct visual review;51 plain and all162 solid public comparisons remain visually unreviewed. Existing diagnostic lifecycle evidence is retained separately and does not establish public lifecycle pixel qualification. Public alpha pixels, complete review, lifecycle pixels, richer composition and broad regression remain pending. Receipt: `output/playwright/html-to-riv/border-v22-parity-pixel-receipt.json`.


### Public border lifecycle and clip-origin compositions — 2026-09-10

Public transparency passes72/72 comparisons, with3 directly reviewed and69 remaining. The lifecycle recorder now stores actual public scene bytes and manifests with explicit `border-public` metadata, installs CSS pixel bounds, and uses monotonically increasing sample indices even without recording. All96 recorded artifacts and manifests match the frozen compiler exactly. The original/clone240/390/768/240 replay passes768/768 geometry/pixel checks;480 repeated frames across288 logical states are byte-identical. No public lifecycle visual coverage is claimed yet. Added18 combinations of rounded half-alpha borders, both box-sizing modes and signed clip margins(-8/0/12px) from border/padding/content origins; public replay passes54/54, with visual review and expanded parity/lifecycle pending. Text/image/axis compositions and broad regressions remain open. Receipt: `output/playwright/html-to-riv/border-v22-lifecycle-receipt.json`.


### Border text/image composition: visual defect preserved — 2026-09-10

Added24 text/image cases covering both box-sizing modes, visible/clip/hidden overflow and radii0/24. All72 aggregate geometry/pixel comparisons pass, but visual inspection detects squared native image-content corners where Chrome rounds the content inside border and padding. The focused `validation/check-border-image-corner.py` fails at all3 widths: pixel(40,36) is Chrome orange background(234,164,61) versus native red image(230,100,40). This is a real qualification failure despite aggregate passes; no tolerance changed. Text border-box clip review covers3 direct views and6 identical image-pair transfers(9/72); image corners are explicitly unqualified. Failure receipt: `output/playwright/html-to-riv/border-v22-content-native/image-corner-failure.json`. Next: minimize/diagnose replaced-content clipping, then fix and rerun both focused and aggregate checks.


### Image corner diagnosis — 2026-09-10

Reduced to a single padded image, with square, visible, no-border and no-padding controls. Rounded hidden/clip fail; square/visible/no-padding controls agree at sampled corners, and removing the border retains the failure. Chrome computed styles expose `overflow-clip-margin:content-box` on the image. Native no-border stream clips the outer124x106 rounded box, not the100x90 content region. Evidence supports missing replaced-image clip-origin semantics, rather than border paint or sampling. CSS Overflow4 also distinguishes the default replaced-element content-box origin: https://www.w3.org/TR/css-overflow-4/. This is an existing borderless image clipping gap exposed by the new matrix; it remains unqualified. Diagnosis: `output/playwright/html-to-riv/border-image-corner-minimal-native/diagnosis.json`. Next: preserve authored clip-margin/global-keyword semantics, implement image-specific default/hidden handling without changing container clips, and rerun focused corners plus the original corpus.


### Image content-corner correction — 2026-09-10

Compiler now seeds the image UA `overflow-clip-margin:content-box` before authored declarations and emits clip-margin policy for two-axis hidden images as well as clip. Explicit initial/unset remain padding-box zero; inheritance and variable values retain cascade semantics. A public compiler regression failed on the missing content-box requirement before the fix and passes afterward, alongside two existing margin contract tests. Frozen corrected toolchain passes the original72 comparisons and all3 formerly failing focused corners; all15 minimal comparisons and15 corner/control samples also pass. Corrected image corners were inspected at original resolution, with6/72 audited direct/identical-pair visual coverage. All13 JavaScript/parity tests pass with47 added content/margin/minimal scenes. Full module/broad renderer regression, full review and richer replaced-image axis/authored-margin lifecycle coverage remain pending. Receipt: `output/playwright/html-to-riv/image-clip-origin-fix-receipt.json`. Historical failing recordings remain intact.


### Image correction regression and border axis matrix — 2026-09-10

Full compiler suite after image-clip correction passes367 tests across71 result groups. New24-scene image/container matrix combines clip-visible, visible-clip, hidden and clip with content-box -4px, padding-box8px and border-box0px margins. All72 geometry/pixel comparisons pass;6 direct inspections plus9 exact-image transfers cover15/72, leaving57 unreviewed. Full native/WASM artifact corpus passes with24 new scenes. Three host checks initially failed to launch because the frozen compiler override lacked its separate probe override; targeted rerun with matching frozen probe passes3/3. Logs preserve both runs; no compiler mismatch was found. Expanded lifecycle, complete review and broad renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-axis-margin-receipt.json`.


### Border composition visual review — 2026-09-10

The border/axis/margin matrix now has complete audited72/72 visual coverage:21 direct original-resolution comparisons and51 exact browser/native image-pair transfers. Inspection covers border-box versus content-box clip origins, signed margins, horizontal/vertical strips, hidden containers and replaced images. Corrected text/image matrix review advanced to42/72:15 direct comparisons and27 exact-image transfers, including all36 text views across both box-sizing modes and square/rounded corners. Thirty image comparisons remain unreviewed. No thresholds changed and no aggregate-only pass was counted as inspection. Evidence: `output/playwright/html-to-riv/border-composition-visual-progress.json` and both replay visual-inspection receipts. Feature qualification still requires remaining visual/lifecycle and broad renderer regression coverage.


### Corrected content matrix visual completion — 2026-09-10

The corrected text/image matrix now has complete audited72/72 visual coverage:30 directly inspected original-resolution comparisons plus42 exact browser/native image-pair transfers. The final five image sheets verify both sizing modes, square/rounded corners and visible versus clipped image content at all three widths. Sparse image sampling/antialias differences remain within unchanged thresholds; the former content-corner defect stays fixed. Together with the completed axis/margin matrix, these two corpora have144/144 reviewed comparisons. Other border corpora, expanded same-scene lifecycle and broad renderer regressions still require qualification. Receipt: `output/playwright/html-to-riv/border-composition-visual-completion.json`.


### Border axis/image public lifecycle — 2026-09-10

Extended public lifecycle recording to image assets and emitted axis/clip-margin policies. All24 image/container axis-margin scenes pass192 original/clone240/390/768/240 geometry and native pixel frames, with120 repeated frames identical across72 logical states. Every recorded Rive artifact and manifest exactly matches the frozen public compiler. A dedicated repeatable audit validates source review completeness, source HTML/CSS/assets, identical Rive bytes/manifests, stream hashes and exact full browser/native PNG pairs before transferring reviewed coverage;192/192 lifecycle frames now have audited visual coverage from the completed static matrix. The replay helper embeds the actual quadrant asset and waits for decode. Other border review/lifecycle corpora and broad renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-axis-image-lifecycle-receipt.json`; audit: `validation/audit-border-lifecycle-review.py`.


### Deferred content-box border corpus admitted — 2026-09-10

The80 scenes deferred during box-sizing work now compile publicly. Fresh Chrome references compare against the corrected frozen compiler/host:240/240 native and240/240 vector comparisons pass. Exact paired browser/native PNG identity across profiles holds for240/240; no completed visual transfer is claimed. Zero-size and reversed-row controls were inspected at original resolution; native visual coverage is12/240, with228 remaining. All13 JavaScript/native-WASM tests pass with the80 scenes added to the accepted artifact parity corpus. The corpus retains its historical deferred filename for traceability; current status is implemented with qualification in progress. Live clone/resize coverage, complete review and broad full renderer regression remain pending. Receipt: `output/playwright/html-to-riv/border-deferred-receipt.json`.


### Deferred border lifecycle: clone paint failure — 2026-09-10

Extended recorder to80 deferred border sizing scenes. Initial geometry run exposed a recorder omission of emitted flex-factor policies; added their installation and CSS paint-order policy, and replaced self-approval of arbitrary capabilities with an explicit implemented whitelist. Seven recorder tests pass, and all640 original/clone geometry updates agree with Chrome. Recorded artifacts/manifests match the frozen compiler. Pixel replay passes632/640;8 failures affect only cloned zero-sized normal-row content-box/border-box cases. Visual inspection shows the overflowing brown child painting over the adjacent yellow sibling, unlike Chrome/original; geometry is unchanged. Both failed recording and failing pixel artifacts are preserved. Static review progressed to21/240; lifecycle qualification remains failed. Next: minimize clone overlap ordering and fix with a focused render regression. Failure receipt: `output/playwright/html-to-riv/border-deferred-lifecycle-failure.json`.


### CSS paint-order clone correction — 2026-09-10

A reduced borderless overlap test reproduced clone draw-order reversal. `clone_instance_definition` copied positioned/stacking policies but omitted `css_paint_order`; it now copies that policy too. An older test explicitly expected policy loss; updated it to assert inherited initial policy plus independent toggles afterward. Focused border/positioning tests pass16/16, and the full module suite passes370 tests across71 result groups. Corrected deferred lifecycle passes640/640 pixels, fixing all8 former clone failures; all640 full image pairs match static references. A formerly failing clone sheet was visually inspected, with4/640 direct/identical-pair coverage; full review remains pending. Old failure artifacts are retained. The frozen image-clip-origin probe predates this runtime correction and must be refreshed for subsequent broad runtime qualification. Receipt: `output/playwright/html-to-riv/clone-css-paint-order-fix-receipt.json`.


### Border full regression and sizing review — 2026-09-10

Frozen the rebuilt compiler/probe after the CSS paint-order clone correction in `border-clone-fixed-toolchain`. Full Chrome/native regression started with7180 checks, session80193, isolated `border-v22-full-native` review and `border-v22-full-artifacts` outputs; terminal result and full visual audit remain pending. Deferred static border sizing review now39/240 views (24 direct plus15 exact full-image transfers), audited after inspecting column border-box auto/basis-auto/basis-percent/basis-px at all three widths. No visible discrepancies in those sheets. Lifecycle640/640 pixels remain passing with incomplete visual coverage. Run handle/manifest receipt: `output/playwright/html-to-riv/border-v22-full-run.json`.


### Border column sizing visual review — 2026-09-10

Deferred border sizing visual review now93/240 views:60 direct and33 audited exact full-image transfers. Newly inspected forward columns cover fixed/maximum/percentage/zero dimensions and content-box flex-basis; reversed columns cover auto/basis-auto/basis-px. All three widths agree visually with Chrome, including zero-size child overflow and reversed bottom anchoring. `record-visual-review.py --audit` passes;147 views remain. Full7180-check regression session80193 was polled live; completion remains pending.


### Border reversed-column and row review — 2026-09-10

Deferred border sizing review now147/240 views:96 directly inspected and51 audited exact full-image transfers. Reversed-column border-box fixed/maximum/percentage/zero and content-box basis/fixed/maximum/percentage cases match Chrome, including bottom-edge child overflow. Initial horizontal auto/basis-auto/basis-percent cases also match at240/390/768 widths. Review audit passes;93 views remain. Full regression session80193 remains pending after live polling.


### Border sizing matrix visual completion — 2026-09-10

Deferred border sizing matrix now has complete240/240 static native visual coverage (162 direct, 78 exact paired-image transfers); review audit passes. All240 vector views transfer with exact compiler-input and full browser/native PNG identity. Dedicated public lifecycle audit verifies source HTML/CSS, identical Rive bytes/manifests, stream hashes and full PNG pairs before transferring all640 original/clone resize frames from the completed static review. No new direct inspection is claimed for these transfers. Historical clone failures remain preserved. Full7180-check regression session80193 is still live, and other border qualification work remains. Receipt: `output/playwright/html-to-riv/border-deferred-visual-completion.json`.


### Border signed clipping review — 2026-09-10

Public signed border clip-margin matrix visual coverage reaches33/54 views (27 direct, 6 exact full-image transfers). Inspected border-origin negative/zero/positive margins under both box-sizing modes and content-origin margins under border-box sizing. Border exposure, orange padding, teal child clipping, responsive right remainder and green sibling overlap agree with Chrome; sparse curve antialias differences are retained. Review audit passes. Full regression session80193 remains pending after live polling; remaining matrix reviews and border lifecycle compositions remain open. Evidence: `output/playwright/html-to-riv/border-v22-clip-margin-native/visual-inspection.json`.


### Border clip-margin review complete and lifecycle regression — 2026-09-10

Signed border clip-margin static matrix completes54/54 audited visual coverage (48 direct, 6 exact paired-image transfers). Final content/padding-origin cases match Chrome structural geometry and clipping with retained curved-edge antialias differences. Added permanent public_border_signed_clip_margin_lifecycle_matches_chrome using the18-scene pinned Chrome oracle; targeted test passes and records144 original/clone240/390/768/240 updates. Pixel replay launched in `border-signed-margin-lifecycle`; terminal result and lifecycle review transfer remain pending. Full regression session80193 remains pending. Static review: `output/playwright/html-to-riv/border-v22-clip-margin-native/visual-inspection.json`; targeted log: `output/playwright/html-to-riv/border-signed-margin-recording.log`.


### Border signed lifecycle complete and transparency review — 2026-09-10

Signed border clip-margin lifecycle completes144/144 geometry/pixel frames with90 identical repeats across54 states. Public artifact/source/full-PNG audit transfers complete visual coverage from the reviewed54 static views; no direct lifecycle inspection is implied. Receipt: `output/playwright/html-to-riv/border-signed-margin-completion.json`. Public border transparency review now33/72 views; transparent and half-alpha square rings inspected with all background/child combinations, matching Chrome colors, border geometry and clipping. Review audit passes. Remaining transparency/plain/solid reviews, text/image lifecycle and full regression qualification remain open.


### Border transparency visual completion — 2026-09-10

Public border transparency matrix completes72/72 visual coverage (66 direct,6 exact-image transfers), audit passes. Transparent/half-alpha borders at radii0/18/90 with background and child combinations match Chrome clipping, color blending, normalized corners and responsive paint. Sparse curve antialias differences remain recorded. Opaque border coverage is in the separate solid/plain corpora, still pending full review. Lifecycle paint frames remain part of the combined96-scene public recording and require audited review transfer once its other source corpora are complete. Full7180-check regression session80193 remains pending. Receipt: `output/playwright/html-to-riv/border-paint-visual-completion.json`.


### Plain border width and corner review — 2026-09-10

Plain public border visual coverage reaches30/54 views, audit passes. Inspected one-pixel opaque/half-alpha rings at radii0/18/90 and24px opaque borders. Border continuity, normalized contours and inner corner collapse when border exceeds radius match Chrome; curved-edge antialias intensity differences are retained. Full regression session80193 was polled live, with log progress past2765 checks; no full result claimed. Remaining plain/solid review and text/image lifecycle qualification stay open. Evidence: `output/playwright/html-to-riv/border-v22-plain-native/visual-inspection.json`.


### Plain border visual completion — 2026-09-10

Plain public border matrix completes54/54 directly inspected views, review audit passes. Widths1/8/24, radii0/18/90 and opaque/half-alpha cover continuous thin strokes, uniform translucent rings, normalized curves and square inner corners when border exceeds radius. Curved-edge antialias intensity differences remain documented; no tolerances changed. Combined solid-border source review remains outstanding before complete96-scene lifecycle review transfer. Text/image lifecycle and full renderer regression qualification remain open. Receipt: `output/playwright/html-to-riv/border-plain-visual-completion.json`.


### Solid border composition review — 2026-09-10

Solid public border composition review reaches40/162 views (18 direct,22 exact paired-image transfers), audit passes. Initial thin-border square and rounded clipped/visible controls match Chrome child overflow, sibling occlusion, content-box extents and responsive orange remainder. Curve antialias differences remain retained. Other composition views, combined lifecycle visual transfer, text/image lifecycle and full renderer regression remain open; session80193 polled live. Evidence: `output/playwright/html-to-riv/border-v22-solid-native/visual-inspection.json`.


### Thick border composition review — 2026-09-10

Solid public border composition review reaches70/162 views (36 direct,34 exact paired-image transfers); audit passes. Added large-radius thin content-box and thick square/rounded border cases in clipped/visible modes. Inner clipping, purple border coverage, child overflow and responsive orange remainder match Chrome; sparse curve antialias differences remain retained. Full regression80193 polled live; remaining solid views, combined lifecycle review and text/image lifecycle remain open. Evidence: `output/playwright/html-to-riv/border-v22-solid-native/visual-inspection.json`.

### Thick rounded border composition review — 2026-09-10

Solid border visual coverage is now 97/162 views, with 65 remaining; `border-v22-solid-native/visual-inspection.json` passes its image/sheet hash audit. Six additional original-resolution sheets cover 24px borders with radius18 (border-box visible; content-box clip/visible) and radius90 (border-box clip/visible; content-box clip), each at240/390/768. Outer/inner contour normalization, square inner corners when width exceeds radius, child clipping/visible overlap and sibling placement agree with Chrome. Sparse curve antialias differences remain under unchanged criteria. Exact-image transfers are recorded separately from direct inspection. Full7180 native regression remains live (session80193 confirmed by polling), with progress beyond4650; terminal completion and broad visual audit remain outstanding.

### Medium border composition review — 2026-09-10

Solid border visual coverage is now 136/162 views (26 remaining), with the review receipt passing its image/sheet hash audit. Nine additional original-resolution sheets cover 24px/radius90/content-box visible overflow, all four 8px/radius0 box-sizing/overflow combinations, and all four 8px/radius18 combinations; each includes240/390/768 widths. Border thickness, inner clips, rounded contours, orange remainder and child/sibling occlusion match Chrome structurally. Sparse curve antialias differences remain under unchanged criteria. Direct reviews and exact-image transfers remain separately recorded in `border-v22-solid-native/visual-inspection.json`. Full7180 regression was confirmed live by session80193 polling and has advanced beyond4950 checks; no terminal pass or broad visual qualification claimed.

### Solid border static visual review complete — 2026-09-10

All54 solid-border composition scenes /162 views now have audited visual coverage: 108 directly inspected and 54 exact-image transfers. Final sheets cover large-radius8px borders in both box-sizing modes with clip/visible overflow, plus remaining1px visible-overflow cases. Outer/inner contours, background extents, child clipping and sibling occlusion match Chrome structurally. Thin and large-radius contours retain documented antialias intensity differences under unchanged criteria. Receipt: `output/playwright/html-to-riv/border-solid-visual-completion.json`. This completes the static matrix only; combined lifecycle audit, text/image lifecycle coverage and the full regression audit remain outstanding. P01 remains in progress.

### Combined border lifecycle review and current replay — 2026-09-10

The public lifecycle audit now accepts multiple independently complete static references while rejecting ambiguous case identities. The existing96-scene combined border recording has768/768 frames audited against the solid/paint/plain references with exact HTML/CSS, RIV, requirements, stream hashes and both PNGs; receipt is `border-v22-public-lifecycle/public-visual-transfer.json`. Single-source signed-margin audit still passes144 frames. Negative checks reject changed HTML, stream/image hashes, missing frames and duplicate sources (`border-multisource-audit-negative-tests.json`). Current runtime recording was regenerated by the public combined geometry regression, which passes; fresh768-frame pixel replay is running as session88803 in `border-combined-clone-fixed-lifecycle`, using the frozen clone-fixed renderer. Historical success does not imply this fresh run has completed. Text/image lifecycle and broad regression qualification remain outstanding.

### Current combined lifecycle complete; text/image recording added — 2026-09-10

The current clone-fixed combined border run passes768/768 pixel comparisons and all768 frames pass the multi-source public artifact/visual audit. Receipt: `border-combined-clone-fixed-lifecycle/public-visual-transfer.json`. Added24-scene text/image original-and-clone regression using pinned Inter, CSS shaping precision/normal-wrap policies and native glyph recording under the native-glyph-controls feature. All192 geometry updates pass. Browser lifecycle replay now loads the same pinned font and waits for fonts; public audit checks the complete font/image asset map. The192-frame text/image pixel replay is running as session90133 in `border-content-clone-fixed-lifecycle`; no visual completion claimed yet. Text/image oracle preserves the original Chrome measurements with explicit border metadata. Full regression qualification remains outstanding.

### Text/image border lifecycle complete — 2026-09-10

All24 text/image scenes pass192 original-and-clone resize pixel comparisons and all192 frames pass the exact source, complete assets, RIV, requirements, stream and full-PNG visual audit. The native-glyph-controls border runtime test file passes9/9 tests. Receipt: `border-content-lifecycle-completion.json`. The support summary and P01 status now reflect completed focused static/lifecycle review; broad qualification still awaits the frozen7180-check full regression and visual audit.

### Version22 full regression passes — 2026-09-10

Frozen version22 full native regression exited0 with7180/7180 checks passing in31.1 minutes. Verified terminal handle80193, completed review status/check count and all four frozen toolchain file hashes. `border-v22-full-run.json` records terminal success and the review hash; qualificationComplete remains false. Exact compiler-input/full-PNG visual transfer audit is running as session96312 against the completed v21 full and regular-suite references; output `border-v22-full-visual-audit.log`. Separately, text/image audit negative checks reject modified font bytes, font weight and unexpected assets (`border-content-asset-audit-negative-tests.json`). Broad visual qualification is not inferred from numerical pass.

### P01 qualified; P02 browser baseline established — 2026-09-10

P01 is qualified for the documented version22 native-renderer scope: full7180 checks and7170/7170 exact compiler-input/full-PNG visual transfers pass, alongside completed focused static/lifecycle reviews and public parity. Receipt: `border-p01-receipt.json`; curved-edge antialias differences remain documented under unchanged criteria. P02 begins with24 individual-side scenes /72 Chrome153 views covering unequal widths, zero-width sides, distinct opaque/translucent colors, rounded joins and both box-sizing modes. All24 current compiler rejections are preserved in `border-sides-admission/admission.json`; individual sides are not yet supported. Runtime currently paints one ring color although solved border widths are per-side; independent colors need a checked transport and non-overlapping corner paint construction before qualification.

### P02 physical-value parser foundation — 2026-09-10

Added tested one-to-four-value expansion in CSS top/right/bottom/left order for widths, styles and colors. Functional colors remain single components; malformed lists, excess values, delimiters and unsupported widths reject. Shared single-color parsing now serves the existing uniform cascade. All12 border unit tests pass; initial missing-parser failure is preserved. Evidence: `border-sides-parser-progress.json`, red/green logs. This helper is not public side admission: cascade per-side state, versioned host transport, independent corner-color rendering, resize/clone pixels and full parity/qualification remain to implement. Existing P01 evidence refers to its frozen qualified toolchain.

### P02 physical border state — 2026-09-10

Replaced the uniform computed border field with four physical values in CSS top/right/bottom/left order. Final-font width resolution, CSS-wide component inheritance, uniform shorthand/reset assignment and schema edge emission now operate per edge. Version22 emission explicitly rejects nonuniform state until the per-side transport is implemented; public syntax admission remains unchanged. Initial library32 tests pass (`border-side-state-tests.log`). Added an unequal-edge inheritance/reset regression; the full module run is active as session38282 (`border-side-state-module-tests.log`) and requires terminal verification. Artifact equivalence to the frozen P01 compiler remains to check before claiming this refactor preserves qualified output.

### P02 state refactor verified — 2026-09-10

The complete module test run exited0: 374 tests across71 result groups pass. The additional unequal-edge inheritance/reset regression passes in a separate targeted run. Recompiled all242 focused P01 scenes with the current compiler; RIV, source-map and runtime-requirements bytes match the frozen reviewed artifacts exactly. Evidence: `border-side-state-completion.json` and `border-side-state-equivalence/receipt.json`. This proves existing focused output preservation through the physical-edge state refactor; it does not qualify P02 rendering or native/WASM parity for forthcoming side syntax. Next implement per-side cascade declarations and versioned runtime colors.

### P02 side cascade implementation — 2026-09-10

Added physical side shorthand/component application, CSS-wide side inheritance/reset and one-to-four-value uniform longhand expansion into per-edge state. Selected-edge overrides preserve other edges; uniform shorthand resets every edge. Substitution grammar dispatch recognizes side properties through their corresponding border component. Library35 tests pass, including selected-edge overrides, shorthand reset, inheritance and invalid negative-width variable reset (`border-side-cascade-tests.log`). Nonuniform computed borders still reject at version22 emission until runtime side-color transport exists. Additional substitution/cardinality tests, public diagnostics, parity and runtime/pixel coverage remain; this is not P02 qualification. Repeated uniform-value syntax may now resolve to the existing uniform transport, but expanded public syntax has not yet received the complete qualification gate.

### P02 invalid substituted component lists fixed — 2026-09-10

A new regression exposed incorrect errors for variables containing multiple values in a single physical side component. The top-level component-count check now runs before normalization as well as in substitution validity, so invalid width/style/color lists reset only that component. Function arguments remain one component; valid unsupported single calc(), dashed and oklch() values retain diagnostics. Preserved both failing stages: initial scalar-width error and normalization error for functional color plus extra value. Library36 tests now pass (`border-side-cardinality-green-r2.log`); receipt `border-side-cardinality-receipt.json`. Runtime transport and corner paint remain outstanding.

### P02 candidate side partition geometry — 2026-09-10

Added an experimental runtime helper constructing four side clip quadrilaterals for the shared rounded border ring. It uses solved unequal widths, validates finite/nonnegative inputs and collapses overfull inner dimensions proportionally to avoid crossed polygons. The geometry test checks positive orientation and total square-ring area for unequal/zero/overfull edges, plus invalid inputs; it passes. Existing css_clip_path tests also pass (`border-side-partition-clip-tests.log`). Inspected the Chrome390px translucent large-radius content-box reference: curved color transitions remain a pixel-comparison requirement. The helper is deliberately not wired into paint yet; area checks do not establish seam-free antialiasing or Chrome-equivalent rounded/overfull joins. Next wire diagnostic per-side paint and compare the full72-view reference matrix before public capability admission.

### P02 diagnostic per-side paint path — 2026-09-10

Added experimental per-side runtime colors in top/right/bottom/left order, cloned with the layout occurrence. The ordinary checked uniform installer clears the experimental override. Distinct colors draw the shared even-odd border ring through side clip paths with saved/restored renderer state; equal colors retain a single ring draw. This path has no public capability admission yet. Initial compile error from an incomplete edit is preserved in `border-side-diagnostic-render-build.log`; corrected runtime geometry build is running as session72616 (`border-side-diagnostic-render-build-r2.log`). Next extend the diagnostic recorder with side widths/colors and compare Chrome72 views. Neither seam-free rasterization nor side-paint lifecycle has been verified.

### P02 diagnostic recording pipeline — 2026-09-10

Corrected runtime side-paint build passed6 clipping geometry tests (`border-side-diagnostic-render-build-r2.log`). Extended the existing lifecycle recorder with explicit diagnostic compiler CSS, top/right/bottom/left widths and colors, per-edge schema injection and experimental paint installation. Prepared24-scene pinned Chrome oracle metadata from the untouched measurements. Diagnostic original/clone240/390/768/240 geometry recording is running as session6278 into `border-sides-diagnostic-recording`; log `border-sides-diagnostic-recording.log`. Replay provenance now preserves injected side widths/colors and remains runtime-experiment-only. No public P02 admission or pixel pass is claimed. On terminal geometry success, run clip-margin-lifecycle.mjs with this recording, the frozen border-clone-fixed renderer and a fresh border-sides-diagnostic-lifecycle output.

P02 diagnostic recording session6278 subsequently exited0: all192 geometry updates pass. The fresh192-frame Chrome/native pixel replay has started in `border-sides-diagnostic-lifecycle`; pixels and visual review remain outstanding.

### P02 first pixel failure: rounded inner-corner pockets — 2026-09-10

Initial side-paint replay finished with60/192 pixel failures (geometry had passed192/192). Direct inspection of the translucent radius90 content-box390px pair shows unpainted orange corner regions absent in Chrome: quadrilateral side clips stop at inner rectangular corners and omit portions of the rounded ring. Failure images and `border-sides-diagnostic-lifecycle/failure-review.json` are preserved. Candidate correction extends each partition via the inner center to cover the outer box; the shared ring removes the interior. Updated area invariant checks total outer-box coverage. New192-frame recording build is running as session29555 in `border-sides-pocket-fixed-recording`; log `border-sides-pocket-fixed-recording.log`. Geometry and pixel results for the correction remain unverified; no tolerance changes or public admission.

### P02 miter direction correction — 2026-09-10

The center-extension replay finished with12/192 pixel failures, down from60; the original and corrected failures remain preserved. Directly inspected the prior translucent radius90/content-box390 case: missing paint is fixed but corner color boundaries shift because clip edges turn toward the inner center. Chromium border painter implementation provides the relevant geometric approach: extend outer-to-inner miter rays to the chord between inner arc endpoints (reference: https://chromium.googlesource.com/chromium/blink/+/refs/heads/main/Source/core/paint/BoxBorderPainter.cpp). This is implementation guidance, not proof of pinned Chrome153 equivalence. Candidate runtime correction now normalizes radii, computes inner radii and preserves each miter ray; fresh geometry recording build session42071 targets `border-sides-miter-ray-recording`. Pixel replay and targeted seam review remain required. No public admission or tolerance changes.

### P02 miter-ray replay passes — 2026-09-10

Miter-ray recording geometry passes192 updates and pixel replay session73237 exits0:192/192 frames pass with unchanged criteria. Four original-resolution Chrome/native pairs inspected, including the previously defective translucent large-radius case, zero-top-width large radius, and opaque/translucent square joins. Missing paint and shifted joins are corrected in those inspected cases. Partial receipt: `border-sides-miter-ray-lifecycle/selected-image-review.json`; complete visual coverage is still pending. Added a focused runtime regression checking ray collinearity and the inner-arc chord endpoint; geometry suite currently building as session34717 (`border-side-miter-geometry-tests.log`). Public capability transport, broader edges and full P02 qualification remain outstanding.

### P02 structured visual review begins — 2026-09-10

All7 runtime clipping/miter tests pass. Created a72-view projection of original frames0/1/2 from the192-frame diagnostic replay, preserving original frame names and source replay hash; every projected row is verified equal to its source. Three full-resolution three-width sheets inspected: translucent unequal-width radius90 content-box, opaque zero-top-width radius90 border-box, and translucent square content-box. Visual receipt audited9/72 distinct views; exact full-image matches cover24/192 lifecycle frames. Evidence: `border-sides-miter-ray-static-review/visual-inspection.json` and `lifecycle-review-progress.json`. All remain runtime-experiment-only; remaining63 distinct views need review, and public capability integration remains open.

### P02 translucent zero-edge visual review — 2026-09-10

Six further full-resolution three-width sheets reviewed: zero-top-width translucent borders at0/18/90 radius in both box-sizing modes. Visual receipt now27/72 distinct views, with72/192 lifecycle frames matched by audited full-image identity;45 distinct views remain. Exposed top background, unequal side thickness, thin bottom, rounded color transitions, child clip and sibling placement match structurally. Sparse curve antialias differences retained under unchanged criteria. Projection row identity and source hash verified before updating `border-sides-miter-ray-static-review/lifecycle-review-progress.json`. This remains diagnostic evidence, not public compiler qualification.

### P02 opaque zero-edge visual review — 2026-09-10

Reviewed three further original-resolution three-width sheets: opaque zero-top-width square borders in both box-sizing modes and radius18 border-box. Brown/green/blue side colors, exposed top background, child clipping and sibling placement match structurally; small diagonal/curve antialias differences remain. Visual audit now36/72 distinct views and96/192 exact-image lifecycle frames;36 distinct views remain. Projection rows and source hash were revalidated for `border-sides-miter-ray-static-review/lifecycle-review-progress.json`. Public compiler capability integration remains outstanding.

### P02 unequal translucent side review — 2026-09-10

Six additional three-width sheets reviewed: opaque zero-top-width radius18/90 content-box and unequal four-side translucent square/radius18/radius90 combinations. Thin top and asymmetric side colors, flattened inner corners, child clips, background remainder and sibling position match structurally; sparse antialias differences retained under unchanged gates. Audit now54/72 distinct views and144/192 exact-image lifecycle frames;18 distinct views remain. Source projection identity and hashes revalidated. Evidence remains diagnostic-only; public capability transport and qualification are outstanding.

### Individual-side border composition review — 2026-09-10

Audited visual coverage reaches102/144 distinct composition views (48 direct,54 exact full-image transfers), with42 remaining. Eight additional original-resolution sheets cover visible rounded images in both box models and container axis/hidden/clip behavior with signed content/padding clip margins. Image extents, selected-axis clipping, perpendicular overflow, rounded contours and sibling occlusion agree with Chrome; sparse image-filtering/curve antialias differences remain under unchanged gates, with no repeated-color seam observed. Geometry and pixels remain384/384; public native/WASM parity13/13. Full regression session65603 was polled live and its log has reached4401/7372; terminal completion and broad visual audit remain outstanding. Evidence: `output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`. P02 remains active.

### Individual-side border composition static review complete — 2026-09-10

All144 composition views now have audited visual coverage. Final six original-resolution sheets cover expanded container clip margins, image axis clipping, and inset clip/hidden image cases at240/390/768. Chrome/native image extents, border contours, clip boundaries and sibling occlusion agree structurally; curve antialias and sparse image sampling differences remain under unchanged criteria. The full384-frame resize/clone visual and asset-reproduction audit remains pending, as does the live full regression. Evidence: `output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`. P02 is not yet qualified.

### Individual-side composition lifecycle audit — 2026-09-10

The focused48-scene composition corpus now has complete384-frame visual coverage across original/clone240/390/768/240 resizing. `validation/audit-border-composition-review.py` verifies projection provenance, complete frame sequences, stream and PNG hashes, exact reviewed image pairs, and fresh frozen-compiler reproduction of all48 RIV files and runtime requirements with Inter/quadrant assets. Both corrupted-RIV and corrupted-requirements negative controls reject at the intended artifact comparison. Static coverage is66 directly inspected views plus78 exact-image transfers. Geometry/pixels384/384 and native/WASM parity13/13 pass. Receipt: `output/playwright/html-to-riv/border-sides-grouped-composition-lifecycle/public-composition-audit.json`; negative controls: `output/playwright/html-to-riv/border-sides-composition-audit-negative-tests.json`. Full7372 regression and its broad visual audit remain pending; P02 remains unqualified.

### Corrected initial border-side lifecycle verified — 2026-09-10

Re-recorded all24 initial scenes with the corrected runtime. Geometry and fresh pinned-Chrome/native replay pass192/192 original/clone frames. The public audit verifies full-PNG identity to completed direct reviews and reproduces all24 scene/requirement artifacts with the frozen corrected toolchain. Receipt: `output/playwright/html-to-riv/border-sides-grouped-initial-completion.json`. Full regression65603 and current full module tests17367 remain running; P02 remains active. SUPPORT.md now consolidates the current version23 candidate scope and outstanding qualification gates.

### Corrected compiler module suite complete — 2026-09-10

The current module suite with native glyph controls exits0:398 tests pass across71 result groups, including the final border compositions. Receipt: `output/playwright/html-to-riv/border-sides-grouped-module-tests-receipt.json`. The7372-check frozen native pixel run remains live; broad visual audit is still pending.

### Border gallery reference audit prepared — 2026-09-10

Added `validation/border-gallery-reference.py`; source-only revalidation passes192 views from completed corrected initial/edge lifecycle evidence. Full-gallery comparison checks exact compiler inputs, RIV/requirements and both full PNGs; target execution remains pending the full suite. Full session65603 was verified live and log progress reached6603/7372. Evidence: `output/playwright/html-to-riv/border-sides-gallery-source-audit.log`. No full visual qualification claimed.

### Per-corner radius reference baseline prepared — 2026-09-10

Prepared24 scenes/72 pinned-Chrome153 views for two/three/four radius values, oversized overlapping corners, isolated longhand and cascade override, both box-sizing modes and plain/unequal-border paint. Current frozen compiler rejects all24; admission diagnostics are preserved. Inspected one four-corner content-box border reference only; no native support or qualification claimed. Fixtures: `validation/corner-radii-initial-cases.json`; receipt: `output/playwright/html-to-riv/corner-radii-initial-baseline.json`. P02 full run remains active beyond6925/7372.

### Version23 full native regression passes — 2026-09-10

Full frozen corrected native suite exits0:7372/7372 checks pass with7362 rendered scene pairs. Border gallery audit verifies all192 new views against completed lifecycle evidence, including exact authored inputs, RIV/requirements and Chrome/native PNGs. Baseline comparison against the prior7170-view gallery is still running as session28882; full visual qualification remains open. State: `output/playwright/html-to-riv/border-sides-grouped-full-run.json`; border proof: `output/playwright/html-to-riv/border-sides-grouped-full-native/border-lifecycle-comparison.json`.

### P02 native qualification complete — 2026-09-10

Full regression7372/7372 and visual coverage7362/7362 pass. The baseline comparison covers7170 unchanged pairs; the independently audited border lifecycle comparison covers all192 remaining pairs with exact public inputs, artifacts and both PNGs. Combined receipt preserves both component proofs. Focused matrices cover896 original/clone frames; module398, host21 and native/WASM13 pass. P02 is native-qualified for the documented physical solid/none/hidden border profile. Per-corner radii P03 is next; other renderer/style limitations and realistic composition expansion remain explicit. Receipt: `output/playwright/html-to-riv/border-p02-receipt.json`.

### P03 circular-radius shorthand state — 2026-09-10

Compiler style state now stores TL/TR/BR/BL radii, expands one-to-four-value border-radius shorthands, and emits distinct native corner fields with linkCornerRadius=false only for unequal values. Uniform emission retains its original field order and linked default. Two focused unit tests pass for shorthand ordering, inherit/initial/unset, em/rem resolution and rejection of negative/excess/slash/percentage values. Log: `output/playwright/html-to-riv/corner-radii-shorthand-tests.log`. Physical longhands, substitution integration, public artifact equivalence, native/WASM and Chrome/native lifecycle pixels remain outstanding; P03 is not qualified.

### P03 physical corner longhands and substitution — 2026-09-10

Added all four physical corner longhands, selected-corner CSS-wide inheritance/reset and em/rem resolution. Substitution validity distinguishes malformed values (selected-corner unset) from valid excluded elliptical/percentage/function syntax (explicit rejection). Three public compiler tests pass for shorthand/longhand RIV and requirements equivalence, order/reset behavior, invalid variables and explicit excluded-value errors;38 library tests pass. Initial test invocation omitted required viewport dimensions; corrected tests use390x320 and the original setup failure is preserved. Logs: `output/playwright/html-to-riv/corner-radii-longhand-tests-r2.log` and `corner-radii-library-tests.log`. Native/WASM and public lifecycle pixel qualification remain outstanding.

### P03 public lifecycle geometry passes — 2026-09-10

All24 initial corner-radius scenes pass192 original/clone geometry updates. Extended the border lifecycle harness for explicitly borderless fixtures, retaining empty-border manifest and absent-paint assertions; original harness assumption failure is preserved. Fresh recording: `output/playwright/html-to-riv/corner-radii-initial-recording-r2`. Pixel replay session42184 compares fresh streams with pinned Chrome using the unchanged frozen renderer; native compiler/probe build63280 runs separately. Visual review, new artifact reproduction and native/WASM parity remain pending. State: `output/playwright/html-to-riv/corner-radii-initial-progress.json`.

### P03 oversized-corner pixel failure — 2026-09-10

Initial192-frame pixel replay exits1 with32 failures, all four oversized-corner variants across both box-sizing/paint modes and original/clone sizes. Directly inspected Chrome/native240px borderless example: native contour differs structurally despite matching bounds. Background path used independent native radius clamping, while CSS border/clip helpers already apply proportional overlap reduction. Candidate correction routes unequal corners under CSS pixel-bounds policy through the shared CSS rounded geometry; native behavior without CSS opt-in and uniform paths remain unchanged. Corrected recording build runs as8275. Failure receipt: `output/playwright/html-to-riv/corner-radii-overlap-failure.json`. No pixel fix or qualification claimed yet.

### P03 proportional corner correction passes focused pixels — 2026-09-10

Corrected geometry and pixel replay pass192/192 frames, resolving all32 original oversized-corner failures under unchanged gates. Two original-resolution three-width sheets inspected for border-box plain/unequal-border oversized corners; proportional contours, inner clipping, padding and siblings agree structurally with sparse curve antialias differences. Review audit6/72 complete. Native and WASM builds pass; frozen `corner-radii-proportional-toolchain` created. Expanded JavaScript parity runs as97374, now including initial radius fixtures. Full review, artifact audit and broader regression remain pending. State: `output/playwright/html-to-riv/corner-radii-initial-progress.json`.

### P03 native/WASM parity and oversized-corner review — 2026-09-10

Expanded public JavaScript/native-WASM suite exits0 with13/13 tests. Visual audit reaches18/72 views, including every oversized-corner case and four-value border-box cases. Physical corner ordering, proportional overlap, unequal-border inner contours and child clipping agree with Chrome structurally, with sparse curve antialias differences retained. Current full compiler module suite has started (`corner-radii-module-tests.log`). Artifact reproduction, remaining visual reviews and full pixel regression remain pending.

### P03 diagonal-corner review and stale exclusion regression — 2026-09-10

Visual audit reaches30/72 views after reviewing four-value content-box and two-value border-box sheets at all widths. Contours, clipping and border joins match structurally. Full module run stopped at the historical custom-property test expecting two-value radius rejection. Updated it to verify public output equivalence for two/three/four-value substitution; elliptical/percentage/viewport/calc exclusions remain asserted. Original failure log is preserved (`corner-radii-module-tests.log`); corrected full run is in `corner-radii-module-tests-r2.log`. P03 qualification remains open.

### P03 module suite passes and broad regression starts — 2026-09-10

Current full compiler module suite exits0 with404 tests across72 result groups. Visual audit42/72 covers the additional two-value content-box and three-value border-box cases; no structural mismatch observed. Added initial corner-radius fixtures to the permanent browser regression and started the frozen native full run (`corner-radii-full-native.log`), expected7444 checks. Remaining30 focused views, artifact/lifecycle audit and broader compositions remain pending.

### P03 initial static visual review complete — 2026-09-10

All72 distinct initial views now directly inspected and hash-audited. Final sheets cover three-value content-box, isolated top-left and bottom-left cascade overrides in both box-sizing/border modes. Chrome/native geometry, contour placement, inner clips and siblings agree structurally; sparse curve antialias differences retained. Public192-frame lifecycle artifact/visual audit remains pending. Full7444 regression session40992 confirmed live. Evidence: `output/playwright/html-to-riv/corner-radii-proportional-static-review/visual-inspection.json`. P03 remains unqualified.

### P03 initial public audit complete; compositions expand — 2026-09-10

All24 initial scenes reproduce exactly from the frozen compiler; all192 original/clone frames have audited exact reviewed PNG coverage, stream provenance and complete resize sequences. Generalized the existing composition auditor with an explicit expected scene count, retaining all frame/artifact checks. Receipt: `output/playwright/html-to-riv/corner-radii-proportional-lifecycle/public-corner-audit.json`. Added36 asymmetric-radius text/image/axis-margin compositions; independent Chrome references are capturing as58411. Full7444 regression40992 remains live beyond596 checks. Broader composition and full visual qualification remain open.

### P03 text/image/clipping composition geometry passes — 2026-09-10

All36 new asymmetric-radius compositions pass288 public original/clone geometry updates against pinned Chrome. Pixel replay41609 and expanded native/WASM parity93443 run against the frozen corrected toolchain. Full regression40992 remains live. State: `output/playwright/html-to-riv/corner-radii-composition-progress.json`. No composition pixel/visual qualification claimed yet.

### P03 composition pixels and parity pass — 2026-09-10

All288 composition pixel/geometry frames pass and expanded public native/WASM parity13/13 passes. First four original-resolution sheets cover clipped text and images in both box models; text wraps/heights, inner corners, border joins and siblings agree with Chrome structurally. Visual audit30/108 views (12 direct,18 exact-image transfers). Remaining78 views and complete lifecycle/artifact audit remain pending, alongside full regression. State: `output/playwright/html-to-riv/corner-radii-composition-progress.json`.

### P03 visible-image and axis review — 2026-09-10

Added direct review of visible images in both box models and negative-content-margin container clips on each individual axis, at all three widths. Image extents, perpendicular overflow, exposed asymmetric corners and sibling occlusion match Chrome structurally; sparse image filtering/curve antialias differences retained. Review audit updated in `output/playwright/html-to-riv/corner-radii-composition-progress.json`. Full regression40992 confirmed live; composition artifact audit and remaining visual views stay open.

### P03 signed clip-margin review advances — 2026-09-10

Composition visual coverage reaches66/108 views (36 direct,30 exact-image transfers), with42 remaining. Four original-resolution sheets cover both axis combinations with expanded padding-box margins and hidden/clip with inset content-box margins at240/390/768. Selected-axis clipping, perpendicular overflow, asymmetric contours, exposed backgrounds and sibling occlusion match structurally; sparse curve antialias differences remain under unchanged criteria. Review receipt re-audit passes. Full regression session40992 was confirmed running; terminal result and broad audit remain pending. Evidence: `output/playwright/html-to-riv/corner-radii-composition-progress.json`.

### P03 composition lifecycle audit complete — 2026-09-10

All108 distinct composition views now have audited visual coverage:54 directly inspected and54 exact full-image transfers. The final six sheets cover expanded container clips, axis image clips and inset image clipping at240/390/768. Image extents, quadrant placement, asymmetric contours and sibling positions agree structurally; sparse curve antialias and right-edge image sampling differences remain under unchanged gates. The public audit reproduces all36 RIV/requirements artifacts with complete assets and verifies288 original/clone resize frames, stream hashes and reviewed PNG pairs. Receipt: `output/playwright/html-to-riv/corner-radii-composition-lifecycle/public-composition-audit.json`. Full regression40992 remains live; broad qualification is pending.

### P03 full-gallery source evidence revalidated — 2026-09-10

The gallery transfer auditor now accepts combined comparisons only after checking their component hashes, original successful rows, unique/full scene coverage and retained source receipt hashes. Revalidated all7362 P02 source pairs against complete compiler inputs and full browser/native PNG bytes. Negative controls reject a changed component hash, changed union image hash and omitted row. Evidence: `output/playwright/html-to-riv/corner-radii-baseline-source-audit.json`. This prepares prior-scene transfer for the P03 full regression; it does not claim the still-running target passed. Full session40992 remains live, last logged3545/7444.

### P03 corner gallery references prepared — 2026-09-10

Added `validation/corner-gallery-reference.py`: source-only mode re-runs the public lifecycle audit, reproduces24 RIV/requirements artifacts and verifies192 reviewed frames before exposing72 initial-view references. Gallery mode reuses the exact public-input/artifact/full-PNG comparator. Source verification passes. A synthetic gallery control transfers one unchanged pair; changed RIV and requirements reject, while a changed native PNG remains explicitly unreviewed. Evidence: `output/playwright/html-to-riv/corner-radii-gallery-source-audit.log` and `corner-radii-gallery-audit-controls.json`. These are audit-tool checks, not full-target qualification. Full regression40992 remains confirmed live.

### P03 all-corner cascade tests and nested Chrome references — 2026-09-10

Public corner test suite passes5/5, including all four selected-corner initial/unset/missing/invalid-variable resets, em/rem/fractional resolution and matching-parent-corner inheritance. Log: `output/playwright/html-to-riv/corner-radii-all-corner-tests.log`. Added16 nested-flex scenes covering every isolated corner, fractional/oversized radii, inheritance and invalid-variable reset in both box-sizing modes. Pinned Chrome153 captures48 reference views successfully; runtime/pixel/parity/visual qualification is pending. Fixtures: `validation/corner-radii-nested-cases.json`; progress: `output/playwright/html-to-riv/corner-radii-nested-progress.json`. Updated stale implementation notes to distinguish completed focused evidence from remaining qualification.

### P03 nested public lifecycle geometry passes — 2026-09-10

Added the16-scene nested oracle to the permanent public runtime test and native/WASM parity corpus. Initial recording failed because missing borderColor metadata selected the harness historical default instead of the authored #76539a; corrected only the expected metadata and preserved the original log. Corrected public recording passes128 original/clone resize geometry comparisons. Pixel replay11084 and expanded parity16589 are running; full regression40992 remains confirmed live. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json` and `corner-radii-nested-recording-r2.log`. No nested pixel or visual qualification claimed yet.

### P03 nested pixels pass; isolated border-box corners reviewed — 2026-09-10

Nested geometry and native pixels pass128/128. Reviewed12/48 distinct views for all four isolated corners in border-box nested flex layouts; selected corner, inner clip, border contour, panel/aside sizing and sibling placement agree with Chrome. Sparse curve antialias differences remain. Expanded parity run passes10 tests including accepted-corpus native/WASM artifact equality; three host checks initially fail ENOENT because NUXIE_NATIVE_PROBE was omitted. Explicit frozen-probe rerun passes all three (no compiler/runtime changes). Original failure log retained alongside `corner-radii-nested-host-r2.log`. Visual audit remains36 views short; full regression40992 still live. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json`.

### P03 nested content-box and overlap review — 2026-09-10

Nested visual coverage reaches36/48 directly inspected views. All isolated content-box corners and fractional/oversized radii in both box models agree structurally with Chrome across240/390/768. Proportional overlap reduction follows changing nested flex width; inner clipping, background exposure, parent heights and sibling/aside placement match. Sparse contour antialias residuals retained under unchanged criteria. Twelve inheritance/variable-reset views remain, followed by the128-frame public artifact audit. Full regression40992 confirmed live, last logged5369/7444. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json`.

### P03 nested lifecycle qualification complete — 2026-09-10

All48 nested views directly inspected and hash-audited. Final inheritance and invalid-variable reset sheets match Chrome structurally in both box models; sparse curve antialias residuals retained. Public audit reproduces all16 compiled scenes/requirements and verifies all128 original/clone resize frames against reviewed PNG pairs and stream hashes. Together with initial192 and composition288, focused P03 coverage now totals608 frames across76 scenes. Native/WASM corpus equality and corrected host checks pass as documented. Full regression and broad target gallery audit remain pending; refreshed module suite launched. Receipt: `output/playwright/html-to-riv/corner-radii-nested-lifecycle/public-nested-audit.json`.

### P03 module suite complete; P04 references prepared — 2026-09-10

Refreshed module suite exits0: 396 tests across72 result groups pass (`corner-radii-final-module-tests.log`). P03 full visual regression remains pending. Prepared16 P04 elliptical/percentage scenes and48 pinned Chrome153 views; all16 frozen current-compiler rejections are preserved in `elliptical-radii-initial-admission.json`. Source inspection finds existing internal elliptical path construction but circular public geometry inputs; proposed live per-axis length/percentage occurrence contract and qualification sequence are documented in `validation/elliptical-radii-implementation-notes.md`. This is preparation only; no P04 syntax admitted.

### P04 per-axis path foundation — 2026-09-10

Extracted elliptical_outset_path with four horizontal/vertical pairs; existing circular helper forwards equal pairs. Shared overlap reduction and contour construction now have an explicit per-axis entry point, without public syntax admission or responsive value storage. Geometry test build77464 is running (`elliptical-radii-geometry-tests.log`); no test pass claimed yet. Frozen P03 full regression40992 remains live and has reached6699/7444. Evidence: `output/playwright/html-to-riv/elliptical-radii-geometry-progress.json`.

### P04 elliptical border geometry foundation — 2026-09-10

Initial per-axis helper tests pass9/9. Added per-axis border ring and side partition entry points while retaining circular wrappers. New tests assert axis-specific inner contour offsets and miter-ray/chord intersections; replaced redundant wrapper equivalence checks with concrete expected geometry. Expanded geometry build53225 remains running (`elliptical-radii-border-geometry-tests.log`). No public P04 admission or pixel qualification yet. Frozen P03 full regression40992 remains confirmed live and reached7180/7444.

### P03 full regression passes; P04 responsive values foundation — 2026-09-10

Frozen P03 full regression40992 exits0:7444 checks pass,7434 rendered pairs. Baseline and new-corner visual audits have started; qualification remains pending those results. Receipt: `output/playwright/html-to-riv/corner-radii-full-run.json`. P04 elliptical border geometry tests pass10/10. Added checked unresolved pixel/percentage radius values with per-axis live dimension resolution and overflow rejection; tests16335 running. This value type is not yet installed on runtime occurrences or emitted by the compiler.

### P03 new gallery views audited; P04 value tests pass — 2026-09-10

All72 new corner gallery views pass exact public-input/RIV/requirements/full-image comparison to reviewed lifecycle evidence. Baseline comparison96393 remains confirmed running. Added reusable combined-gallery proof construction with completed-gallery identity, component row counts, conflict/missing-coverage rejection and retained hashes; positive union and missing-row controls pass. P04 responsive value tests pass2/2, covering independent width/height percentage resolution after resize/copy and invalid/overflow rejection. P04 remains unintegrated with runtime occurrences or compiler output.

### P03 circular corner radii native-qualified — 2026-09-10

Frozen full regression passes7444/7444; combined visual proof accounts for all7434 rendered pairs (7362 prior plus72 new), retaining both component comparisons. Fresh focused audits reproduce76 scenes and verify608 resize/clone frames; all228 distinct focused views have audited coverage. Default module suite396 passes; expanded parity10 plus corrected host3 checks pass, with the original omitted-probe failure preserved. P03 is native-qualified for the documented circular-length LTR profile against the frozen proportional toolchain. Curve antialias/image sampling, direction controls, vector and P04 limitations remain explicit. Receipt: `output/playwright/html-to-riv/corner-radii-p03-receipt.json`. Later P04 source changes are not covered by this qualification.

### P04 experimental occurrence integration — 2026-09-10

LayoutComponent now retains optional checked per-axis radius values, copies them on clone, and exposes experimental install/clear. Paint geometry resolves percentage axes from current layout width/height; background, border ring, side partitions and clip-margin paths consume common resolved pairs. Legacy circular callers retain equal-axis geometry. Added a resize/clone/clear regression asserting live path endpoints; test build26548 is running (`elliptical-radii-occurrence-tests.log`). This remains runtime experimentation: host contract, compiler admission, fractional browser discrimination and full pixel qualification are pending. P03 qualification continues to refer to its immutable frozen toolchain.

### P04 diagnostic browser lifecycle recorder — 2026-09-10

Occurrence/value tests pass3/3, including path endpoints after resize, clone and clear. Prepared16 diagnostic oracle cases with explicit pixel/percentage pairs and compiler CSS that omits unsupported radius declarations. Recorder installs experimental radius occurrence values after public base-scene compilation; evidence uses ellipse-diagnostic kind. Replay preserves injected pairs and actual compiler CSS and marks runtime-experiment-only, preventing confusion with public compiler qualification. Recording build16371 is running (`elliptical-radii-diagnostic-recording.log`). Public grammar/transport remain unsupported; geometry/pixel results pending.

### P04 first diagnostic pixels pass — 2026-09-10

Experimental ellipse recording and replay each pass128/128 original/clone frames. All16 cases cover percentages, slash lists, mixed pairs, zero axes and overlap in both box models. First6/48 distinct views inspected: border-box percentage ellipse and four-pair corners match Chrome structurally across240/390/768, retaining sparse contour antialias differences under unchanged gates. Evidence: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/replay.json` and `elliptical-radii-geometry-progress.json`. This is runtime-experiment-only; remaining visual coverage, public grammar/host transport/parity and expanded composition qualification remain pending.

### P04 content-box and overlapping percentage review — 2026-09-10

Diagnostic visual review reaches18/48 views. Content-box percentage and independent-pair ellipses, plus oversized percentage radii in both box models, match Chrome structurally at240/390/768. Border-box expansion, responsive overlap reduction, child clips and sibling positions agree; sparse contour antialias residuals remain under unchanged criteria. Runtime-experiment qualification only; public compiler/host work remains pending. Evidence: `output/playwright/html-to-riv/elliptical-radii-diagnostic-static-review/visual-inspection.json`.

### P04 mixed-unit and physical longhand review — 2026-09-10

Diagnostic visual coverage reaches30/48 directly inspected views. Mixed fixed/percentage axes and isolated top-left pairs match Chrome structurally in both box models across240/390/768. Responsive contours, inner clipping, exposed background and sibling placement agree; sparse contour antialias residuals retained under unchanged gates. Eighteen remaining views cover fixed ellipses, zero axes and unequal shorthand list lengths. Public compiler/host transport and expanded qualification remain pending. Evidence: `output/playwright/html-to-riv/elliptical-radii-geometry-progress.json`.

### P04 diagnostic visual lifecycle complete — 2026-09-10

All48 initial ellipse views directly inspected. Fixed ellipses, zero-axis square corners and independent shorthand list expansion agree structurally with Chrome in both box models; sparse contour antialias differences retained. The128-frame visual audit verifies source/projection identity, injected radius pairs, actual compiler CSS, complete original/clone resize sequences, stream hashes and reviewed PNG pairs. This is explicitly runtime-experiment-only and does not qualify public compilation/host transport. Receipt: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/diagnostic-visual-audit.json`. Public grammar/transport/parity and expanded fractional/composition evidence remain next.

### P04 specified-axis compiler parser — 2026-09-10

Added isolated corner-radii component parser preserving Pixels/Em/Rem/Percent values. Independent one-to-four slash lists expand in TL/TR/BR/BL order; physical longhands accept one or two axis values. Tests3/3 pass for distinct units, comments, zero, shorthand expansion and malformed/negative/nonfinite/unsupported rejection. CSS-wide cascade keywords remain handled outside this parser. Public style application and runtime transport are not yet integrated, so compiler admission remains unchanged. Evidence: `output/playwright/html-to-riv/elliptical-radii-parser-receipt.json`; source: `src/corner_radii.rs`.

### P04 axis pairs integrated with computed style — 2026-09-10

Computed style now stores four horizontal/vertical radius pairs and uses the new shorthand/longhand parser. Selected-corner inheritance copies both axes; final-font resolution converts em/rem while retaining percentage values. Existing circular values emit the established linked/unlinked scalar fields. Noncircular/percentage output rejects explicitly until checked transport exists. Library42 and public corner/custom-property56 tests pass. Evidence: `output/playwright/html-to-riv/elliptical-radii-style-receipt.json`. Equal-axis syntax may resolve to existing circular output; expanded public syntax qualification still awaits transport/parity/browser gates.

### P04 checked occurrence installer — 2026-09-10

Added Artboard::set_css_corner_radii_occurrence with all-target validation before mutation and replacement semantics that restore imported radii for omitted layouts. Regression passes for non-artboard roots, zero/missing/non-layout/duplicate IDs, no partial mutation, clone independence, clear and reinstall across240/390/768/240. Existing16-scene experimental ellipse Chrome geometry lifecycle also passes through this checked installer. Two focused tests pass; no new pixel replay or public compiler qualification claimed. Compiler transport, extreme percentage overflow behavior and expanded fractional/composition validation remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-installer-receipt.json`.

### P04 overflow-safe live corner resolution — 2026-09-10

Found a finite-value overflow path: percentage resolution could exceed f32 and hit the paint expect; large finite pixel pairs could overflow edge sums. Added f64 fallback resolution with one shared CSS overlap factor before narrowing, preserving ordinary representable inputs. Five runtime tests pass, including actual path endpoints after clone/resize and huge pixel/percentage controls; the existing16-scene checked-installer Chrome geometry lifecycle test also passes. Extreme inputs have analytic/runtime evidence only, with no new Chrome pixel claim. Public compiler transport and expanded visual qualification remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-overflow-receipt.json`.

### P04 public per-axis transport and initial pixels — 2026-09-10

Public compilation now emits version24 `layout-css-corner-radii-v1` with `layout_corner_radii` entries: unique layout object IDs and four TL/TR/BR/BL pairs, horizontal then vertical, each exactly `{pixels:number}` or `{percent:number}`. Finite nonnegative computed px/em/rem and percentage axes, slash-list expansion, physical longhands and existing cascade/reset rules are implemented. Circular pixel pairs retain legacy artifacts. Math and viewport units remain excluded. Checked native probe installation and TypeScript declarations are added; unsupported hosts reject before drawing. This is implemented, not yet fully qualified.

Focused border/corner public contracts14 tests pass; initial public recording and Chrome/native replay128/128 frames pass. Direct review9/48 views covers border-box fixed ellipses, percentage ellipses and four distinct corners. Full-resolution sheets are in `output/playwright/html-to-riv/elliptical-radii-public-static-review`. Native/WASM build/parity and remaining visual/public-artifact audit are pending.

Preserved failures: first recording compared JSON integer/float variants (fixed with typed comparison); r2 exposed30% becoming30.000002% (fixed by reading authored percentage points directly, with decimal/exponent/comment controls); module r1 exposed public border tests consuming diagnostic compilerCss (public tests now always compile authored CSS); r2/r3 exposed default-stack depth overflow (boxed retained computed styles fixes original resource-limit regression; added independent depth controls); r4 reached obsolete custom-property ellipse rejection (now checks public equivalence for newly accepted syntax). Initial TypeScript checks required updating the explicit version/capability assertions. Current full module/build sequence is running, so no full-suite or parity pass is claimed yet.

### P04 public transport tests complete — 2026-09-10

Default module409 tests across74 groups pass; native CLI/probe and WASM builds succeed. Expanded JavaScript/native-WASM suite13/13 passes including the initial16 ellipse corpus; targeted version23/24 host tests2/2 and TypeScript pass. Frozen compiler reproduces every recorded RIV/requirement from authored HTML/CSS (16/16). Public Chrome/native128-frame replay passes, with12/48 distinct views directly inspected and36 remaining. All associated processes are terminal. Receipt: `output/playwright/html-to-riv/elliptical-radii-public-transport-receipt.json`; immutable binaries: `elliptical-radii-public-toolchain`. P04 remains active pending full visual audit, fractional/composition expansion and full native regression.

### P04 initial public lifecycle audit complete — 2026-09-10

All48 initial public ellipse views are directly inspected and hash-audited. Content-box and border-box fixed/mixed/percentage pairs, independent shorthand expansion, physical longhands, zero-axis square corners and shared overlap reduction agree structurally with Chrome153 across240/390/768. Sparse curve antialias differences remain under unchanged gates. Fresh public audit reproduces16 compiled RIV/requirement pairs and verifies all128 original/clone resize frames against reviewed images and stream hashes. Receipt: `output/playwright/html-to-riv/elliptical-radii-public-lifecycle/public-ellipse-audit.json`. This completes the initial corpus only; fractional bases, zero-axis positive clip margins, nested/text/image/unequal-border compositions and full native regression remain open. P04 stays active.

### P04 fractional and clip-margin edge corpus — 2026-09-10

Added16 permanent public edge scenes and pinned Chrome153 references: fractional box dimensions/offsets, percentage and mixed axes, zero-axis square corners, overlap, and positive clip margins using border/padding/content origins in both box models. Public same-scene original/clone geometry128/128 and native pixel replay128/128 pass. Direct review9/48 views confirms fractional border-box percentage/mixed contours and square zero-axis expanded clips;39 views remain. No runtime change was needed for this initial edge run. Expanded corpus parity and full public artifact/visual audit remain pending. Receipt: `output/playwright/html-to-riv/elliptical-radii-edge-progress.json`.

### P04 visual review exposes fractional clip-edge residual — 2026-09-10

Edge visual review reaches21/48 views. Fractional content-origin clip margins show thin continuous edge differences despite passing existing aggregate pixel gates. Preserved a six-scene control corpus contrasting fractional square corners with integer ellipse/square geometry in both box models. Frozen-toolchain replay14/18 passes; four fractional-square240/390 comparisons fail mismatch ratio, all geometry passes. Directly inspected nine border-box control views: integer square edges match and integer ellipses retain sparse curve residuals, while fractional square clips reproduce continuous edge mismatch. This isolates a shared clip/rounding issue, not ellipse-specific behavior. No tolerance changes or control qualification. Expanded native/WASM parity13/13 passes. Evidence: `output/playwright/html-to-riv/elliptical-radii-clip-control-replay/failure-inspection.json`; progress: `elliptical-radii-edge-progress.json`. Next: isolate fractional translation versus content inset and correct shared clip geometry. All processes in this increment are terminal.

### P04 fractional clip rounding isolated; first fix incomplete — 2026-09-10

Ten one-variable controls isolate failures to fractional padding (26/30 comparisons pass; translation/offset/width/height controls pass). Browser pixel samples show whole-pixel clip edges versus native partial coverage. Added failing runtime regression, then snapping final clip bounds makes the test and30/30 isolation comparisons pass; three padding-only views directly inspected show the continuous mismatch gone. Combined original controls still15/18, exposing double rounding from deriving clip edges from an already-snapped outer box. Preserved first-fix toolchain/results. Updated implementation now derives clip bounds from unrounded layout/insets and snaps once; added combined translation and affine opt-out checks. Session34940 is testing/building; no second-fix pass claimed. Receipt: `output/playwright/html-to-riv/elliptical-radii-clip-rounding-progress.json`.

### P04 clip rounding controls now pass — 2026-09-10

Revised clip geometry derives unrounded live layout/inset edges and rounds once in world pixel coordinates under the existing CSS pixel-bounds policy. Runtime regression passes including fractional translation+padding and affine opt-out. Frozen updated probe passes30/30 one-variable and18/18 combined Chrome comparisons; original and first-fix failures remain preserved. Directly inspected three combined fractional square border-box views: continuous edge mismatch and one-pixel shifts are gone. This is focused control evidence, not full runtime qualification. Updated public ellipse lifecycle, remaining visual/public audits and broad regression are next. Receipt: `output/playwright/html-to-riv/elliptical-radii-clip-rounding-progress.json`. All processes are terminal.

### P04 separate rounded and snapped clips resolve edge discrepancy — 2026-09-10

Rounding one combined path fixed square clips but changed rounded fractional-edge coverage. Pinned Chrome153 source confirms two clip nodes for ordinary rounded boxes: rounded border clip plus snapped overflow rectangle. Runtime now preserves the fractional rounded contour and intersects it with a separate snapped rectangle when needed. Source evidence: `output/playwright/html-to-riv/elliptical-clip-chromium-source/source.json`.

Added targeted fractional-edge sampling (`validation/clip-edge-samples.py`) which fails on the prior implementation despite passing area metrics; all8 updated lifecycle frames pass. Updated public edge replay128/128, isolation30/30, combined controls18/18 and default module410 tests pass. Directly inspected six content-origin ellipse views across both box models; continuous edge discrepancies are gone, sparse curve antialias differences retained.42 updated views and broader public/full regression qualification remain. Receipt: `output/playwright/html-to-riv/elliptical-radii-dual-clip-receipt.json`. All processes terminal.

### P04 refreshed initial lifecycle and fractional review — 2026-09-10

Current dual-clip runtime passes all128 original/clone frames in the refreshed initial public ellipse lifecycle (elliptical-radii-initial-dual-lifecycle); process80431 exits0. Its visual/public artifact audit is still pending. Updated edge review reaches18/48 hash-audited views: fractional percentage and mixed-unit corners in both box models agree structurally at240/390/768, with sparse curve antialias residuals retained.30 updated edge views remain, followed by composition expansion and full native regression. Summary row now reflects implemented v24 transport rather than historical preparation. Evidence: output/playwright/html-to-riv/elliptical-radii-dual-clip-receipt.json.

### P04 updated edge corpus audit complete — 2026-09-10

All48 updated edge views directly inspected and hash-audited. Zero-axis square corners, single square corner mixed with ellipses, border/padding/content-origin positive clip margins and oversized proportional overlap agree structurally with pinned Chrome153 across240/390/768 in both box models. Sparse curve antialias residuals remain; tolerances unchanged. Fresh public artifact audit reproduces16 RIV/requirement pairs from authored input and verifies all128 original/clone resize frames, source identities, stream hashes and reviewed images. Receipt: output/playwright/html-to-riv/elliptical-radii-edge-dual-lifecycle/public-ellipse-audit.json. Initial refreshed lifecycle visual audit, nested/text/image/unequal-border compositions and full native regression remain; P04 stays active.

### P04 refreshed initial corpus audit complete — 2026-09-10

All128 refreshed initial lifecycle image pairs are exactly identical to previously reviewed images, with unchanged authored HTML/CSS and runtime requirements. Revalidated the original48-view visual audit and fresh16-scene public artifact reproduction; matched refreshed RIV/requirement artifacts, checked all original/clone240/390/768/240 sequences, current stream hashes and full PNG hashes against the reviewed source. No new direct inspection is claimed. Reproducible verifier and receipt: output/playwright/html-to-riv/elliptical-radii-initial-dual-lifecycle/audit-refresh.py and public-ellipse-refresh-audit.json. Both initial and edge corpora are now audited under the corrected clip runtime. Nested/text/image/unequal-border composition expansion and full native regression remain; P04 is active.

### P04 composition expansion geometry passes — 2026-09-10

Added52 public ellipse scenes and156 pinned Chrome153 references: text/images with unequal translucent borders, visible/clip/hidden and axis clips, signed clip margins, nested flex, independent physical corners and inheritance. Public original/clone resize geometry416/416 passes. First run failed missing expected borderColor metadata in nested fixtures; corrected to authored #76539a, preserving original output. No compiler/runtime fix was needed for geometry. Expanded JS parity corpus registered but not yet run. Native pixel replay is running; no pixel or visual qualification claimed. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 composition renderer profiles separated — 2026-09-10

Default-feature recording completed as vector text:416 geometry frames pass,380 pixel frames pass and36 fail across six text scenes. Direct inspection of three visible border-box text views finds text raster residuals while wrapping/borders/layout agree structurally; all failures retained, no waiver. Native qualification requires native-glyph-controls; rebuilt recorder with that feature and all416 geometry frames pass. Native-glyph pixel replay and expanded native/WASM parity are running. Source, vector failure inspection and current paths are recorded in output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 native composition pixels pass — 2026-09-10

Native-glyph composition replay416/416 geometry/pixel frames passes with repeat-state stability. Prepared156 distinct full-resolution views; first three visible border-box text views directly inspected, with glyphs/wrapping/unequal translucent borders and elliptical corners matching structurally. Expanded accepted native/WASM corpus passes; initial suite10/13 with three ENOENT host setup failures from omitted frozen probe override. Explicit NUXIE_NATIVE_PROBE retries pass all three affected host tests (2+1); original failure log retained. Vector36 text pixel failures remain unwaived. All processes terminal. Three direct views plus six exact-image within-run transfers cover9/156; remaining147 views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-progress.json.

### P04 content text/image visual review — 2026-09-10

Composition visual coverage reaches42/156 (21 directly inspected plus21 audited exact-image transfers). Content-box text and both box-model image visible/clip cases agree structurally with Chrome across240/390/768; image clips follow elliptical inner contours, and negative content-origin clip margins contract coverage correctly. Sparse curved-border antialias and thin image-right-edge sampling residuals at768 remain explicitly recorded, including their presence in visible controls. No tolerance changes or pixel-perfect claim.114 views plus public artifact audit/full regression remain. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 axis and container clip-margin review — 2026-09-10

Composition coverage reaches78/156 views (39 direct plus audited exact-image transfers). Inspected horizontal-only/vertical-only overflow, negative content-origin and positive padding-origin clips, border-origin clips and static hidden behavior at240/390/768. Child bounds, exposed borders and sibling placement agree structurally with Chrome. Short shallow lower-left contour residual in border-origin clip and other curve antialias differences retained explicitly under unchanged gates; no pixel-perfect claim. Remaining78 views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 image matrix and nested upper corners reviewed — 2026-09-10

Composition coverage reaches120/156 (54 direct views plus audited exact-image transfers). Remaining image combinations are covered by exact within-run image identity after inspecting the one-axis image control; thin image-edge sampling residual remains documented. Nested isolated top-left/top-right ellipses in both box models agree structurally at240/390/768, including percentage flex sizing and independently rounded panel/aside. Sparse contour antialias residuals retained. Remaining36 nested views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 nested lower corners and fractional radii reviewed — 2026-09-10

Composition review reaches138/156 views (72 direct plus audited exact-image transfers). Lower-right/lower-left isolated ellipses and fractional independent axes in both box models agree structurally with Chrome across240/390/768: child clips, exposed backgrounds and nested flex positions match. Sparse contour antialias residuals retained under unchanged gates. Remaining18 overlap/inheritance/invalid-variable views, public artifact audit and full native regression. Evidence: output/playwright/html-to-riv/elliptical-radii-composition-native-static-review/visual-inspection.json.

### P04 composition visual and artifact audit complete — 2026-09-10

All156 composition views covered (90 directly inspected plus audited exact-image transfers). Final overlap, inherited-axis and invalid-variable reset cases agree structurally with Chrome in both box models across240/390/768; curve/image sampling residuals remain explicit. Fresh public artifact audit reproduces52 RIV/requirement pairs and verifies416 original/clone resize frames, authored inputs, stream hashes and reviewed image pairs. Native/WASM accepted corpus and corrected host checks pass; vector36 text failures remain unwaived. Receipt: output/playwright/html-to-riv/elliptical-radii-composition-native-lifecycle/public-ellipse-audit.json. P04 remains active pending full native regression and its visual audit.

### P04 full native regression launched — 2026-09-10

Registered84 audited ellipse scenes (initial16, edge16, composition52) in the regular browser regression, adding252 comparisons. Built current probe with native-glyph-controls; frozen prior dual-clip probe intentionally lacked that profile. Preserved prior toolchains and froze elliptical-radii-native-full-toolchain with new probe hash. Full native session7608 is confirmed running7696 checks with isolated output/results/gallery. No full-pass claim yet. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json. Focused native composition audit remains complete; full regression and visual audit are pending.

### P04 full-gallery reference audit prepared — 2026-09-10

Added ellipse-gallery-reference.py with fresh lifecycle/public-artifact audit for all84 scenes and252 reference views. Gallery comparison requires exact authored input including embedded Inter/quadrant assets, RIV, requirements and full browser/native PNGs; changed pairs remain unreviewed. Source-only verification passes252/252; completed-gallery comparison remains pending. Full regression7608 confirmed live, observed365/7696 checks. No full-pass claim. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json.

### P05 group opacity references prepared during P04 regression — 2026-09-10

Prepared12 prospective group-opacity scenes and36 pinned Chrome153 references: opacity0/0.5/1 with overlapping children, nested opacity, rounded clipping and translucent borders. Frozen compiler rejects all12; exact diagnostics preserved in output/playwright/html-to-riv/group-opacity-initial-admission/receipt.json. Renderer API inspection finds per-draw modulate_opacity but no group layer operation in the inspected Renderer trait; implementing child alpha alone cannot satisfy overlap semantics. No opacity syntax admitted or native qualification claimed. P04 full regression7608 remains confirmed live.

### P05 interior opacity discriminator established — 2026-09-10

Added group-opacity-samples.py for solid interior pixels in overlap/nested controls at opacity0/0.5/1 across240/390/768. All18 Chrome reference frames pass; sibling alpha remains unchanged. The analytic per-draw-alpha overlap control is rejected. Expected channels use integer quantization with one-channel-unit allowance; initial nearest-rounding expectation disagreed with Chrome half-alpha output and is preserved in group-opacity-initial-samples-rounding-failure.json. This is reference validation only, not native opacity support. Borders/clipping still require full visual validation. P04 full native session7608 remains running.

### P05 renderer boundary investigation — 2026-09-10

Documented existing texture-backed RenderCanvas as a candidate isolation mechanism, plus missing Renderer/recording/replay group contract and duplicate inherited-alpha risk. Planned checked group semantics, stacking-context isolation, clip/overflow bounds, nested resource ownership and lifecycle validation in validation/group-opacity-implementation-notes.md. No group opacity implementation/admission claimed. P04 frozen full regression remains active.

### P05 stacking-context discriminator — 2026-09-10

Added six prospective opacity stacking scenes and18 pinned Chrome references. At opacity0.5, child z-index -2/0/2 remains isolated below an outside sibling at z-index1. At opacity1, child z-index2 escapes the non-stacking parent and covers that sibling. All18 exact interior overlap samples confirm this distinction (output/playwright/html-to-riv/group-opacity-stacking-oracle/stacking-samples.json). Canvas begin_frame inspection shows shared render-context beginFrameExecutable; nested use must preserve parent work through an explicit scheduling/recording strategy, not assume independent contexts. Public opacity remains unsupported. P04 full regression remains running.

### P05 isolated opacity parser implemented — 2026-09-10

Added src/opacity.rs: single finite number/percentage, comments, exponent syntax, clamp before f32 narrowing and canonical positive zero. Three focused tests pass, covering large finite clamping and malformed/nonfinite/unit/math rejection. CSS-wide keywords/substitution remain cascade responsibilities. Parser is intentionally unconnected to public style admission until group rendering/recording/host support exists. Receipt: output/playwright/html-to-riv/group-opacity-parser-receipt.json. P04 frozen regression remains live, observed1866/7696 checks; new isolated parser does not change its immutable compiler.

### P05 real offscreen compositing test added — 2026-09-10

Added renderer test offscreen_canvas_composites_overlapping_shapes_with_one_opacity: draw overlapping red/green shapes on blue canvas, finish offscreen frame, composite image over white at0/0.5/1 and assert red-only/overlap/background interior pixels and opaque output alpha. Test build61153 remains confirmed running; no pass claimed. This verifies candidate sequential canvas mechanism only, not nested active-frame support or public CSS group opacity. P04 frozen regression7608 remains active, last observed2329/7696. Evidence: output/playwright/html-to-riv/group-opacity-offscreen-progress.json.

### P05 offscreen and nested alpha pixels pass — 2026-09-10

Corrected renderer test build to product feature renderer-metal after preserved internal-feature missing-adapter build failure. Real overlapping-shape canvas compositing passes at0/0.5/1. Added nested transparent canvas test: two half-opacity composites produce quarter-opacity red over white while untouched pixels remain white. Both opacity tests pass; filtered run5/5 including existing offscreen tests. This proves sequential canvas compositing only; parent-frame suspension/recording, runtime group boundaries, stacking policy and public transport remain pending. Evidence: output/playwright/html-to-riv/group-opacity-offscreen-progress.json. P04 frozen full regression remains running.

### P05 parent clip and state restoration pixels pass — 2026-09-10

Added real renderer regression for translated half-opacity canvas under a parent clip, followed by an opaque sibling after restore. Interior samples confirm clipped group coverage, excluded pixels and restored sibling transform/clip/alpha. All three opacity canvas tests pass; filtered offscreen run6/6. Candidate sequential composition now has overlap, nested transparency and parent-state evidence. Runtime scheduling/recording and public group admission remain pending. Receipt: output/playwright/html-to-riv/group-opacity-offscreen-progress.json. P04 full native regression remains active.

### P05 browser value parity and replay scheduling constraint — 2026-09-10

Pinned Chrome value probe passes21 controls for candidate opacity number/percentage parsing, exponent syntax, comments, finite large clamping and malformed token rejection. This supports the isolated parser contract; public opacity remains gated. Recorded replay_frame's already-open renderer constraint: nested textures require preparation before parent-frame creation or explicit suspension, with balanced-group/resource validation and retained texture ownership. Evidence: output/playwright/html-to-riv/group-opacity-values.json and validation/group-opacity-implementation-notes.md. P04 full native regression remains confirmed live, observed3762/7696.

### P04 gallery audit negative controls pass — 2026-09-10

Added executable verifier controls using freshly audited252 ellipse reference views. Synthetic complete report transfers252 exact pairs; changed embedded asset rejects, substituted image leaves one pair unreviewed, and missing check rejects incomplete gallery. These are verifier tests only, not full-regression or new visual qualification. Receipt: output/playwright/html-to-riv/ellipse-gallery-audit-controls.json. Full native session7608 remains active, observed4177/7696 checks.

### P05 portable group structure and prepaint rejection — 2026-09-10

Added typed/text beginOpacity/endOpacity commands and a child-first structural plan. Preflight checks finite normalized alpha, maximum64 nested groups, balanced boundaries and save/restore isolation; malformed groups cannot paint or allocate replay resources. Existing already-open-frame replay explicitly rejects valid groups until preparation exists. Stream suite11/11 passes, including4 new structural/negative tests. This is infrastructure only: group execution, renderer recording, runtime policy and public CSS opacity remain pending. Receipt: output/playwright/html-to-riv/group-opacity-stream-preflight-receipt.json. Full P04 session7608 confirmed live this turn; latest observed5668/7696 checks.

### P05 inherited group state planning passes — 2026-09-10

Group preparation now retains ordered inherited transform, clip and per-draw modulation state using persistent command chains. Group boundaries implicitly restore state; explicit save/restore and nested groups cannot leak state to following siblings. Two new tests cover nested state ordering and10000 sibling groups sharing10000 state nodes without copying their histories. Stream suite13/13 passes. This is preparation infrastructure, not texture execution or public opacity support. External clips must be applied at composition rather than baked and applied again. Receipt: output/playwright/html-to-riv/group-opacity-state-plan-receipt.json. P04 full native session7608 confirmed live this turn, latest observed6069/7696 checks.

### P05 sequential portable stream canvas execution passes — 2026-09-10

Implemented render_frame_to_canvas with shared loaded resources, child-first offscreen rendering and final owned output canvas. Inherited transforms place group contents; ancestor clips/modulation apply at composition, with inverse transforms returning textures to device coordinates. Frames finish even when execution reports an error. Real Metal nested overlap/translation/parent-clip/sibling pixel test passes; stream suite15/15 including allocation/transform/unsupported-backend controls passes. Candidate uses viewport textures,256MiB aggregate pixel budget and finite invertible transforms; caller must have no active shared-context frame. Tight bounds, singular transforms, runtime/recording/host/compiler integration and Chrome visual qualification remain. Public CSS opacity remains rejected. Receipt: output/playwright/html-to-riv/group-opacity-canvas-execution-receipt.json.

### P05 recording interface and repeated alpha controls pass — 2026-09-10

Renderer now exposes opt-in begin_opacity_group/end_opacity_group; unsupported backends decline without changing output. RecordingRenderer emits typed-stream syntax, checks finite normalized alpha/depth64 and unmatched ends. Stream16/16 tests pass. Expanded real Metal canvas test covers seven outer/inner alpha configurations including zero and one; returning to half/half produces identical full pixel buffers on the same factory. Runtime policy and compiler/host admission remain pending. Receipt: output/playwright/html-to-riv/group-opacity-recording-receipt.json.

### P04 full regression passes; ellipse references all transfer — 2026-09-10

Frozen full native session7608 terminates successfully:7696/7696 checks pass. Fresh focused-source/public-artifact audit transfers all252 ellipse gallery pairs by exact inputs/artifacts/full PNGs, with zero remaining ellipse pairs. Baseline audit13921 is still running; full visual qualification is not yet claimed. P05 render-api suite36/36 also passes after opt-in group recording methods. Evidence: output/playwright/html-to-riv/elliptical-radii-full-progress.json and group-opacity-recording-receipt.json.

### P04 native qualification complete — 2026-09-10

Full7696/7696 checks and7686/7686 visual pairs are audited:7434 exact baseline transfers plus252 exact focused ellipse transfers. Combined proof, completed checks and frozen binary hashes verified. P04 now native-qualified for the documented v24/LTR/native-glyph scope;84 scenes/672 original-clone frames, module410 and expanded native/WASM/host evidence retained. Vector36 text failures and curve/image sampling differences remain explicit. P05 remains active and unqualified. Receipt: output/playwright/html-to-riv/elliptical-radii-p04-receipt.json; contract: validation/elliptical-radii-review.md.

### P05 runtime stacking/group plan passes — 2026-09-10

Runtime paint planner now represents opacity and emits begin/end boundary events around complete isolated contexts. Alpha0 and0.5 isolate negative/positive positioned descendants; alpha1 retains ordinary stacking. Nested boundary ordering and deferred ancestor clips pass alongside existing stacking regressions:16/16 tests. Production runtime trees still default to1; occurrence policy, checked renderer admission and boundary execution remain pending. No public CSS opacity claim. Receipt: output/playwright/html-to-riv/group-opacity-runtime-plan-receipt.json.

### P05 runtime installation and recorded lifecycle pass — 2026-09-10

Artboard now atomically installs validated layout opacity targets, copies policy to clones, rebuilds stacking plans and emits renderer group boundaries. Renderer capacity preflight enables try_draw_internal_handle to return false before painting; legacy infallible drawing asserts on unsupported group rendering. Lifecycle integration test passes invalid-target/alpha atomicity, original/clone resize240/390/768/240, clear/alpha1 restoration and nested recording boundaries. Exhausted recording capacity rejects with an unchanged stream. Public compiler/host transport and runtime-emitted browser/native pixel qualification remain pending. Receipt: output/playwright/html-to-riv/group-opacity-runtime-install-receipt.json.

### P05 native replay CLI executes opacity groups — 2026-09-10

Native Metal replay detects group commands and renders their canvases before opening the output frame; flat streams retain the existing replay path. Both standard and atomic modes pass nested-alpha interior pixels, two-run PNG identity and explicit white-clear override. Translucent-clear controls confirm alpha is not applied twice; truncated group streams fail without producing a PNG. Four repeated opaque images plus two translucent images are exercised. This uses a synthetic stream, not runtime CSS/browser qualification. Receipt: output/playwright/html-to-riv/group-opacity-replay-cli-r2/receipt.json. Frozen P04 binaries remain unchanged.

### P05 runtime opacity Chrome/native corpus passes — 2026-09-10

Twelve initial opacity fixtures compile once with opacity stripped only in the diagnostic compiler request; checked runtime policy installs authored alpha.96 original/clone resize frames pass geometry and native pixel comparison against pinned Chrome153, with repeated native state identity. Targeted18 overlap/nested interior sample frames pass.12/36 distinct views directly inspected: half-alpha overlap/nested, elliptical clipping and translucent borders agree structurally; sparse contour antialias and uniform color quantization differences retained under unchanged gates. Remaining24 views, full diagnostic evidence audit, stacking/richer compositions and public compiler/host admission. Receipt: output/playwright/html-to-riv/group-opacity-initial-runtime-progress.json.

### P05 initial runtime visual and artifact audit complete — 2026-09-10

All36 initial views covered by30 direct reviews and6 exact-image transfers. Final zero/one controls preserve layout, child occlusion, nested alpha, elliptical clips and translucent borders; sparse contour/quantization differences remain explicit. Fresh diagnostic audit reproduces12 opacity-stripped RIV/requirement pairs, verifies authored CSS and injected alpha, group balance/value counts,96 original/clone frames, source/projection/stream hashes and reviewed full PNGs. Frozen replay renderer hash also verified. This qualifies runtime diagnostic evidence only; public CSS opacity remains rejected. Receipt: output/playwright/html-to-riv/group-opacity-initial-runtime-lifecycle/diagnostic-opacity-audit.json. Stacking/richer compositions and compiler/host transport remain.

### P05 opacity stacking runtime corpus audited — 2026-09-10

Six stacking fixtures pass48 original/clone resize geometry/pixel frames and exact overlap samples.18 distinct views covered by12 direct inspections and6 exact-image transfers. Half-opacity seals negative/zero/positive descendants under outside z1 sibling; full-opacity z2 escapes and covers it, z0 stays under and negative stays behind parent background. Fresh diagnostic audit reproduces6 stripped compiler artifacts and verifies injected alpha, streams/lifecycle and reviewed PNGs. Combined initial+stacking evidence now18 scenes/144 frames/54 reviewed views; public compiler/host transport and richer text/image/composition coverage remain pending. Receipt: output/playwright/html-to-riv/group-opacity-stacking-runtime-receipt.json.

### P05 glyph adapter and composition references prepared — 2026-09-10

GlyphRenderer now forwards opacity group capacity/boundaries and restores cached transform/modulation state. Added nested-state and unsupported-backend tests. Initial command omitted native-glyphs-experimental and selected zero tests; preserved. Corrected build36556 confirmed live, no pass claimed. Twelve text/image/mixed fixtures produce36 pinned Chrome references; removed unused image-only text rule with exact36-image recapture identity. Runtime composition recording/replay remains pending. Evidence: output/playwright/html-to-riv/group-opacity-glyph-adapter-progress.json and group-opacity-composition-reference-receipt.json.

### P05 text/image automated pass exposes visual discrepancy — 2026-09-10

Corrected glyph-adapter tests2/2 pass. Asset-aware runtime recorder and glyph wrapping produce96/96 composition geometry/pixel frames with repeat stability across12 scenes. Direct mixed-visible half-opacity inspection (three widths) catches nonuniform native image interiors despite aggregate passes: Chrome quadrant is constant, native varies by2–3 channel units. Opaque image-only control varies0–1; mixed opaque output (with nested text group) varies0–2. No composition visual qualification claimed and no tolerances changed. Frozen recordings/replay and diagnostic samples retained. Evidence: output/playwright/html-to-riv/group-opacity-composition-runtime-progress.json and group-opacity-composition-image-interior-diagnostic.json. Next investigate added offscreen/image-composition stages before public opacity admission.

### P05 intermediate dither accumulation fixed — 2026-09-10

Minimal flat rectangle reproduces growing interior variation:1 channel unit without groups,3–5 through1–3 groups. Nearest sampling does not improve it and was reverted. Same-binary dithering control removes growth;64 original image frames stay within1 unit. Added explicit RenderCanvas::begin_compositing_frame, implemented by native canvas with dithering disabled only for intermediate frames. Ordinary canvas and final frame semantics remain unchanged; temporary environment switch removed. Eight flat-color controls, CLI group/clear/rejection controls, render-api36 and stream16 tests pass. Fixed original composition96/96 replay passes and all native PNGs exactly match the positive diagnostic control. Three corrected mixed-visible widths directly reviewed (plus exact-image transfers); remaining composition review and fresh artifact audit pending. Receipt: output/playwright/html-to-riv/group-opacity-dither-diagnosis.json.

### P05 corrected composition visual and artifact audit complete — 2026-09-10

All36 corrected text/image/mixed Chrome/native views now have audited visual coverage (18 directly inspected and18 exact full-image transfers). Text wrapping, nested alpha, image quadrants, responsive borders and outside siblings agree visually; small quantization and sparse curved-edge antialias residuals remain documented without tolerance changes. Fresh asset-aware artifact audit reproduces12 opacity-stripped compiler artifacts and verifies injected alpha, recorded group balance, all96 original/clone resize frames and reviewed image hashes. Public opacity CSS remains unqualified. Next revalidate initial/stacking corpora with the corrected renderer, then complete compiler/host transport and parity. Evidence: output/playwright/html-to-riv/group-opacity-composition-compositing-frame-lifecycle/diagnostic-opacity-audit.json and group-opacity-composition-runtime-progress.json.

### P05 corrected initial and stacking replay passes — 2026-09-10

The frozen undithered-compositing renderer passes all96 initial and48 stacking frames against pinned Chrome, including original/clone repeat stability. Initial corrected review has6/36 directly inspected views (nested half-opacity and translucent border at three widths); stacking18 views await review. Only9 initial views are byte-identical to the old output; no wholesale review transfer claimed. Fresh artifact audits remain pending until visual coverage completes. Evidence: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json. Public opacity admission remains pending.

### P05 corrected runtime corpora fully audited — 2026-09-10

Corrected initial36 and stacking18 views now have complete audited coverage (initial30 direct/6 exact-image transfers; stacking12 direct/6 transfers). Fresh artifact audits verify all18 stripped compiler artifacts and144 original/clone frames against the recorded inputs and reviewed images. Alongside corrected composition12 scenes/96 frames/36 views, the current renderer has30 diagnostic scenes,240 lifecycle frames and90 reviewed views. Stacking discriminators preserve group isolation below opacity1 and allow child z-index participation at opacity1. Sparse curved-edge antialias and quantization differences remain explicit; public compiler admission, host transport, native/WASM parity and broader public regression are still pending. Receipt: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json.

### P05 computed opacity cascade integrated — 2026-09-10

Style now stores group opacity independently of paint colors, defaults to1 without inheritance, supports explicit inherit/initial/unset, and uses the existing normalized number/percentage parser. Added cascade/substitution tests. Initial focused run5/6 exposed math fallback incorrectly invalidating to unset; corrected compatibility classification retains unsupported math diagnostics. Full library48/48 passes. Public admission remains explicitly gated until isolated-compositing transport exists; one regression test covers8 authored values and passes. No new public opacity support claimed. Receipt: output/playwright/html-to-riv/group-opacity-computed-style-progress.json.

### P05 version25 requirement contract tested — 2026-09-10

Added LayoutGroupOpacityRequirement with unique non-root LayoutComponent IDs and finite normalized opacity in [0,1); opaque entries are omitted because opacity1 does not establish a stacking context. Version25 and layout-css-group-opacity-v1 must match nonempty entries. The capability explicitly requires isolated subtree compositing, undithered intermediate surfaces and checked renderer capacity. Structural validation rejects missing/extra fields, wrong types, duplicates, root targets, invalid alpha and mismatched versions/capabilities. Corner coexistence and old-version controls pass. Three opacity plus four corner requirement tests pass, and TypeScript version25 declarations/typecheck pass. Checked host installation, public emission and native/WASM parity remain pending; CSS admission stays gated. Receipt: output/playwright/html-to-riv/group-opacity-requirements-progress.json.

### P05 checked host installation verified — 2026-09-10

Probe now advertises and installs version25 group-opacity requirements, supports explicit disabled-capability controls and uses checked instance drawing. Added public try_draw/try_draw_handle entry points preserving artboard frame identity; insufficient group capacity refuses before paint. Existing atomic/clone/clear/resize integration test now exercises the checked instance API and passes. New host transport test passes nested alpha at four widths, missing capability and malformed manifest rejection before stream output. Initial full host run had two feature-configuration failures (probe lacked native-glyph-controls); preserved. Rebuilt with native-glyph-controls and full host regression passes. Public CSS emission remains gated; these are manually supplied manifest controls. Receipt: output/playwright/html-to-riv/group-opacity-host-progress.json.

### P05 public opacity emission candidate — 2026-09-10

Public CSS now emits version25 normalized group requirements for computed opacity below1, leaving Rive layout and paint bytes unchanged. Explicit inheritance, CSS-wide resets, percentages, clamping, important cascade and custom-property fallback are covered; opaque values omit requirements and preserve prior artifacts. Five requirement/public emission tests pass. General opacity math remains rejected. Full Rust regression initially stopped on a stale opacity-rejection expectation; that test now checks excluded calc syntax, with original failure retained. Full rerun and native/WASM parity are pending; this is an implemented candidate, not a native-qualified feature. Added 30-scene JS/native/WASM parity test using existing Chrome-reference corpora; execution awaits matching builds. Current scope remains static isolated compositing through the checked recording/replay host, not immediate backends without group support.

### P05 public emission module and parity pass — 2026-09-10

Full corrected Rust module suite426/426 passes. Matching native and WASM compilers build successfully; new30-scene public opacity parity test passes across initial, stacking and text/image composition Chrome-reference inputs, comparing complete Rive bytes, source maps and runtime requirements. Public Chrome/native pixels and compile-once clone/resize qualification remain pending; runtime-only diagnostic receipts are not promoted to public evidence. Receipt: output/playwright/html-to-riv/group-opacity-public-emission-progress.json.

### P05 public lifecycle pixels and artifact audit pass — 2026-09-10

Thirty authored opacity scenes now compile directly, install only emitted group requirements, clone once and resize both instances across240 frames. Geometry and native/Chrome pixel gates pass with repeated-state byte stability. All90 public views exactly match authored HTML/CSS and complete browser/native PNGs from the corrected, independently audited diagnostic reviews; specialized visual transfer records this without claiming new inspection or transferring compiler qualification. A separate fresh public artifact audit reproduces all30 Rive artifacts/requirements, checks source-map alpha targets, all240 stream/frame identities and group balance, and verifies reviewed images. Broader edges, audit negative controls and full regression remain before P05 qualification. Receipt: output/playwright/html-to-riv/group-opacity-public-lifecycle-progress.json.

### P05 audit negative controls and public precision correction — 2026-09-10

Eighteen negative controls reject changed HTML/CSS, hash claims, actual PNG/Rive bytes, qualification, missing/duplicate views, requirements, injected policy, frame identity and missing frames; two positive controls pass. Controls operate in temporary copies and leave originals read-only. Public boundary tests exposed generic CSS f32 token normalization rejecting finite1e100 opacity before clamping. Numeric opacity text is now preserved until computed-value invalidation and the f64 opacity parser; this also retains near100% precision. First fix prematurely validated multi-token var fallbacks; retained failure and corrected ordering. Library48 plus opacity requirement/public6 tests pass. Existing public visual receipts describe the preceding frozen outputs; refreshed parity, Chrome boundary evidence and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-audit-precision-progress.json.

### P05 Chrome opacity boundary semantics and parity pass — 2026-09-10

Pinned Chrome28 controls (14 values authored directly and through var fallback) agree with public requirement presence using an observable z-index/elementFromPoint discriminator. Chrome serializes computed opacity as1 for some values that still isolate stacking, so computed-style text alone is insufficient:99.999997% retains a group while99.999999% does not. Large finite values clamp correctly. Refreshed WASM and native builds pass opacity parity across30 composition scenes plus28 boundary inputs, comparing Rive bytes, source maps and requirements. Browser screenshots are captured but not claimed as new native visual qualification. Native boundary lifecycle pixels, broader combinations and full regression remain. Receipt: output/playwright/html-to-riv/group-opacity-precision-progress.json.

### P05 native boundary pixels visually audited — 2026-09-10

The28 direct/var boundary inputs now compile publicly and replay through the native renderer at240x320. All geometry/pixel gates and exact browser/native100,100 overlap-color samples pass. Four distinct full-resolution image pairs directly inspected: invisible, half-alpha, fully opaque (red over green), near-opaque isolated (green over red). Remaining24 views have verified exact full-PNG transfers; review audit complete. Small quantization differences remain explicit, tolerances unchanged. These static controls do not establish clone/resize or broader composition coverage. Receipt: output/playwright/html-to-riv/group-opacity-native-boundary-progress.json.

### P05 boundary clone/resize pixels reviewed; metadata correction — 2026-09-10

Twenty-eight boundary scenes now compile once and resize original/clone across224 frames. Geometry/pixels/repeat stability pass, with84 views reviewed (12 direct and72 full-image transfers). Fresh artifact audit exposed recorder serde_json::Value widening f32 alpha versus compiler shortest-roundtrip serialization. Recorder now serializes requirements through the public JSON serializer; corrected recording test passes and all224 stream files are byte-identical to the prior recording. Alpha comparison in the audit uses exact f32 representation for known f32 fields, with no tolerance. Corrected metadata/replay binding and final artifact audit remain pending, so no completed boundary qualification claimed. Receipt: output/playwright/html-to-riv/group-opacity-boundary-lifecycle-progress.json.

### P05 corrected boundary artifact audit complete — 2026-09-10

Corrected metadata recording replays224/224 frames successfully;84 views carry forward prior inspection only after authored source, complete PNG, stream and freshly generated sheet identity checks. Fresh public artifact audit now passes with exact f32 semantic alpha comparison and exact serialized requirement reproduction. Boundary controls are complete within this frozen native-renderer scope. Independently removed the per-group linear plan scan during canvas composition by retaining the group node alongside its indexed prepared canvas; stream tests pass, native verification pending. Receipts: output/playwright/html-to-riv/group-opacity-boundary-completion-progress.json and group-opacity-boundary-public-lifecycle-r2/public-opacity-audit.json. Broader composition/resource controls and full regression remain before P05 qualification.

### P05 indexed canvas lookup preserves all focused native frames — 2026-09-10

The indexed prepared-canvas lookup passes16 stream tests, native CLI alpha/clear/rejection controls and8 flat-color controls. All464 audited public opacity frames rerender with complete native PNG identity to their reviewed baselines; no new visual inspection claimed or needed for unchanged full images. Updated the current P05 backlog row and leading support contract to describe implemented version25 behavior, remaining qualification and explicit replay constraints. Broader opacity compositions, backend/resource limits and full visual regression remain. Receipt: output/playwright/html-to-riv/group-opacity-indexed-native-progress.json.

### P05 resource boundary and overflowing composition coverage — 2026-09-10

Added exact256MiB surface-budget boundary controls: root plus63 1024² groups reaches the allocation interface; an extra group or column rejects before paint, for nested and sibling groups. Stream17 tests pass. Added18 public scenes combining opacity0/.5/1, border/content-box sizing, visible/rounded/axis clipping and genuinely overflowing text/images. All144 original/clone geometry/pixel frames and repeat checks pass; expanded native/WASM opacity parity covers76 scenes. First three half-opacity rounded-clip views directly reviewed; remaining51 views and artifact audit pending. Recorder now installs content-box and axis policies for this corpus. Receipt: output/playwright/html-to-riv/group-opacity-overflow-composition-progress.json.

### P05 overflowing compositions audited and registered for regression — 2026-09-10

All18 overflowing text/image scenes now have complete54-view coverage and a fresh public artifact audit covering144 original/clone frames. Rounded and x-only clipping, both box-sizing modes, nested text opacity and outside-sibling stacking agree with pinned Chrome. Sparse edge antialias and image quantization differences remain within existing gates; tolerances unchanged. Across public focused corpora the total is76 scenes/608 frames/228 views. Registered all five opacity corpora in the regular browser regression, adding228 comparisons; the full gate has not yet passed. Evidence: output/playwright/html-to-riv/group-opacity-overflow-composition-lifecycle/public-opacity-audit.json and group-opacity-overflow-composition-progress.json.

### P05 full native regression launched — 2026-09-10

Frozen current compiler, native-glyph-controls probe, indexed native renderer and WASM in group-opacity-native-full-toolchain with SHA-256 manifest. Verified installed Chrome153.0.8010.12. Full native regression session82980 reports7924 checks (prior7696 plus228 opacity comparisons); module regression session80405 is separately running. Outputs and gallery are isolated from earlier receipts. No full-pass or P05 qualification claim yet. Evidence: output/playwright/html-to-riv/group-opacity-full-progress.json.

### P05 full module pass and gallery reference verification — 2026-09-10

Full native-glyph-controls Rust module regression passes442 tests across76 result blocks, exit0. Added opacity-gallery-reference.py, which freshly revalidates public artifacts/lifecycle/review evidence for all228 opacity views before comparing a completed full gallery by exact authored inputs, assets, Rive bytes, requirements and both full PNGs. Source verification passes228/228. Verifier controls pass exact pairs, reject changed embedded assets/incomplete galleries and leave changed native images unreviewed. Initial control failed because the synthetic fixture assumed separate requirements files; corrected to use the already-audited recording metadata, preserving the failure log. Full native session82980 remains live; full JS parity session28814 now running. No completed full visual regression claimed. Evidence: output/playwright/html-to-riv/group-opacity-full-progress.json.

### P05 decorated opacity compositions pass automated gates — 2026-09-10

Full JavaScript regression15/15 passes (before decoration expansion). Added12 public underline/strikethrough scenes crossing zero/half/opaque parent alpha, nested half-opacity text and visible/clipped overflow. Pinned Chrome36 references captured; helper now installs emitted underline/strikethrough policies. All96 original/clone geometry/native-pixel frames and repeat checks pass. Expanded focused public/native/WASM parity passes88 inputs. Three half-opacity visible underline views directly inspected;33 views and fresh artifact audit remain. These new fixtures are not part of the already-running7924-check frozen full regression. Evidence: output/playwright/html-to-riv/group-opacity-decoration-progress.json.

### P05 decorated opacity visual and artifact audit complete — 2026-09-10

All36 decorated-text views now have complete coverage (27 direct,9 exact full-image transfers for invisible controls). A fresh audit reproduces12 public Rive artifacts/requirements and verifies96 original/clone resize frames. Underline clipping and visible overflow, strikethrough positioning, nested text alpha and outside-sibling placement agree with Chrome; existing sparse antialias/quantization residuals retained without tolerance changes. Focused public totals now88 scenes/704 frames/264 views. The decoration corpus remains separate from the running7924-check full regression and will be registered afterward. Host transform/backend constraints and full regression review remain. Receipt: output/playwright/html-to-riv/group-opacity-decoration-progress.json.

### P05 host modulation reproducer exposes image-path double alpha — 2026-09-10

Added analytic native controls for ancestor translation/scale/rotation/shear and host modulation, with overlapping group shapes on both native modes. Initial stream writer emitted spaces unsupported by the parser; preserved and corrected. Native translation+host0.5+group0.5 then fails at interior(6,6):RGBA255,223,223,255 versus expected255,191,191,255. Image-as-path drawImage multiplies host modulation into image paint, then drawPath multiplies it again. Changed that branch to pass unmodulated image alpha; direct image branch unchanged. Renderer build33758 running, no fix-pass claim yet. The separate7924-check full regression uses the preceding frozen renderer and cannot qualify this correction. Receipt: output/playwright/html-to-riv/group-opacity-host-state-progress.json.

### P05 host modulation correction passes analytic controls — 2026-09-10

Renderer build exits0; all16 translation/scale/rotation/shear × host1/0.5 × two-native-mode controls pass both single-shape and overlap samples. Corrected image-path alpha is applied once. Frozen renderer in group-opacity-host-state-fixed-toolchain; existing public frame identity and broader image regression remain pending. Synthetic controls do not qualify CSS transforms. Receipt: output/playwright/html-to-riv/group-opacity-host-state-fixed/receipt.json.

### P05 corrected renderer preserves704 frames; full gate restarted after ENOSPC — 2026-09-10

All704 audited public opacity original/clone frames rerender with exact full native PNG identity after the host-modulation correction. Extended the identity tool to include overflow and decoration corpora. Original full session82980 terminated exit1 after1877 passes when disk exhaustion prevented artifact writes; failure evidence preserved, no full-pass claim. Removed only regenerable Cargo dependency/test executables (16.5GiB), retaining sources and validation artifacts. Froze corrected renderer plus existing compiler/probe/WASM in group-opacity-native-full-toolchain-r2. Registered12 decoration scenes; new full session15010 reports7960 checks. Expanded gallery source verification and controls pass264 exact references, changed assets rejected, changed images left unreviewed, incomplete galleries rejected. Full output review remains pending. Receipt: output/playwright/html-to-riv/group-opacity-full-r2-progress.json.

### P06 linear-gradient reference and rejection baseline — 2026-09-10

While corrected P05 full regression runs, prepared16 prospective linear-gradient scenes and48 pinned Chrome153.0.8010.12 references: cardinal/corner/angle directions, three stops, explicit/decreasing/coincident/out-of-range stops, alpha, and mixed pixel/percentage positions. Frozen public compiler rejects all16, preserving exact diagnostics. Runtime inspection finds explicit numeric gradient endpoints rather than layout-relative CSS geometry; responsive updates need a checked runtime seam, not compile-viewport coordinates. Documented implementation/validation order and remaining semantics in validation/linear-gradient-implementation-notes.md. No gradient syntax admitted or native pixels claimed. Receipt: output/playwright/html-to-riv/linear-gradient-initial-progress.json.

### P06 responsive gradient geometry and stop fixup implemented — 2026-09-10

Added runtime css_linear_gradient module resolving cardinal/angle/corner endpoints from current box dimensions, plus source-preserving omitted/decreasing/pixel/percentage stop fixup. Four standalone Rust tests pass, covering diagonal endpoints, corner midpoint constraints across aspect ratios, resize-dependent hard stops, out-of-range positions and invalid/degenerate inputs. Runtime cargo check passes. A separate harness compiles the runtime helper and compares its predicted red/blue ramp against120 pinned Chrome samples across eight directions and three widths; all pass the declared two-channel-unit quantization bound. This is geometry evidence only, not native rendering or public gradient admission. Parser, checked runtime paint update/clone seam, public transport/parity and pixel qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-geometry-progress.json.

### P06 specified gradient component parser tested — 2026-09-10

Added an unconnected gradient parser retaining direction, currentColor, px/em/rem/percentage/omitted positions and expanded double-position color stops. Supports existing named/hex/RGB/HSL colors, angles/cardinal/corner directions, comments and256 expanded stops. Three tests pass including all16 prospective Chrome inputs and malformed/unsupported/limit controls. Initial test caught nonfinite numeric token serialization producing a finite angle; validation now rejects before serialization and the original failure is retained. Independent pinned Chrome controls confirm single-color double-position gradients are valid, as are hints and explicit color spaces; the latter remain explicit parser exclusions pending implementation, not claimed invalid CSS. Public admission remains gated by missing runtime paint/host transport and qualification. Receipt: output/playwright/html-to-riv/linear-gradient-parser-progress.json.

### P06 computed gradient cascade integrated behind emission gate — 2026-09-10

Style retains a gradient separately from background color. Background shorthand resets both; background-color preserves the image; background-image none/initial/unset clears it. Relative stops resolve against the final font size (rem uses the profile16px root), so explicit inheritance copies computed lengths. CurrentColor stays symbolic for final color resolution. Gradient function syntax is preserved past the ordinary color-only serializer; variable fallbacks retain valid gradients and invalidate plain red/length/duplicate-none image values to unset. Public compilation explicitly rejects remaining gradients until responsive paint transport exists. Initial failures (test diagnostic-vector access, nested-function serialization) are preserved; corrected focused7/7 and complete library55/55 pass. P05 frozen full run remains independent and live. Receipt: output/playwright/html-to-riv/linear-gradient-cascade-progress.json.

### P06 checked gradient paint description and extended stop domain — 2026-09-10

Runtime inspection confirms ordinary Rive LinearGradient clamps stop positions to0..1. Added checked CssLinearGradient retaining authored direction/colors/positions with matching count and2..256 stop limits. Resolution first performs CSS stop fixup, then expands endpoints to contain out-of-range stops and normalizes them for the shader, preserving their colors inside the box. Seven standalone tests pass, including the prior four geometry controls, out-of-range color coordinates, clone/repeated resize and malformed/empty inputs. This does not yet install or draw CSS gradients. Layout drawing, positioning-area/border semantics, atomic host transport and native interpolation qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-paint-policy-progress.json.

### P06 experimental layout gradient drawing and atomic installation — 2026-09-10

LayoutComponent now retains optional checked gradient paint, clones authored values, and paints it after solid fills/before borders using current dimensions and rounded background geometry. The Artboard occurrence installer validates all non-root layout targets and duplicates before replacing the policy map; clearing one occurrence leaves its clone intact. Integration test passes eight original/clone resize draws with expected recorded endpoints, invalid-target atomicity, clear and clone independence. Early adapter type build failures and test-held factory borrow failure are preserved; corrected run passes. This is recorded-runtime evidence only: native interpolation, positioning-area/border/repeat behavior and public transport remain unqualified, and public gradient emission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-runtime-installation-progress.json.

### P06 native replay exposes missing gradient-only drawable proxy — 2026-09-10

Added16 explicitly injected gradient descriptions to the diagnostic oracle and recorded128 compile-once original/clone frames; geometry passes. First real native replay fails128/128 with blank gradients. Inspected native image and recording: no gradient commands were emitted because owners with no ordinary fill did not request a drawable proxy. The earlier white-background installation control masked this. Updated needs_drawable_proxy to include CSS gradients and removed that solid fill from the integration regression. Corrected full gradient-runtime recording session6233 building; no fix-pass claim yet. Expanded lifecycle harness labels gradient injection runtime-experiment-only. Public gradients remain gated. Receipt: output/playwright/html-to-riv/linear-gradient-initial-runtime-progress.json.

### P06 corrected proxy renders gradients;24 native residuals retained — 2026-09-10

Both gradient runtime integration tests pass after proxy admission correction. Corrected128-frame replay now renders gradients:104 frames pass existing native/Chrome gates and24 fail, confined to decreasing-stop, transparent-stop and mixed-alpha scenes (eight frames each). Geometry remains correct. First default native image inspected to confirm the blank-paint defect is removed; no complete browser/native visual review claimed. Residual interpolation/hard-stop diagnosis is next, with original and corrected outputs preserved. Receipt: output/playwright/html-to-riv/linear-gradient-initial-runtime-progress.json.

### P06 exterior stops fixed; alpha interpolation mismatch isolated — 2026-09-10

Direct browser/native inspection isolates decreasing-stop failure: native lost the first red color before coincident leading stops. CSS resolution now adds equivalent constant-color endpoint stops before native normalization, retaining hard transitions without approximation. Eight pure tests and two runtime recording tests pass. Native128-frame rerun improves to112 passing; only16 transparent/mixed-alpha frames fail. Eight preserved interior samples confirm Chrome premultiplied interpolation versus native straight-color interpolation (each matches its respective analytic formula within2 channel units). Shader source also explicitly premultiplies after interpolating unmultiplied colors. Next requires a checked CSS interpolation path while preserving ordinary Rive gradient behavior; public admission remains gated. Receipt: output/playwright/html-to-riv/linear-gradient-interpolation-progress.json.

### P06 explicit premultiplied gradient transport tested — 2026-09-10

Added opt-in Factory::make_premultiplied_linear_gradient returningNone for unsupported/invalid input, forwarding through persistent factory wrappers. Recording validates finite coordinates, matching stop counts and normalized monotonic positions, and emits makePremultipliedLinearGradient. Stream parser retains a distinct resource and replay refuses unsupported factories; ordinary gradient commands remain unchanged. API regression passes; stream19/19 includes round-trip through persistent recording, unsupported rejection, no output for malformed inputs and ordinary-mode controls. Initial new tests omitted the empty frame marker; preserved failure and corrected fixtures. Native mode implementation and CSS draw selection remain pending, so transparency residuals are not claimed fixed. Receipt: output/playwright/html-to-riv/linear-gradient-premultiplied-transport-progress.json.

### Exact-stop gradient visual audit complete — 2026-09-10

Corrected runtime experiment r6 passes128/128 Chrome/native original-clone frames and16/16 targeted hard-stop frames. Full-resolution review now covers every frame:39 direct views,79 within-run exact-image transfers and10 cross-run exact-input/full-image transfers; combined audit reports zero remaining. Five review-identity tests include eight semantic/RIV mutations and stale/wrong-path/failed audit rejection. Stop-table capacity controls pass14336 samples; full renderer suite passes474 tests with6 ignored. Public gradient CSS remains gated: transport/emission, native/WASM parity, performance and broader composition qualification remain. Evidence: `output/playwright/html-to-riv/linear-gradient-exact-table-progress.json`, `linear-gradient-initial-lifecycle-r6/visual-coverage.json`, and `linear-gradient-review-identity-controls.json`.

### Gradient portable contract foundation — 2026-09-10

Added Rust version26 capability `layout-css-linear-gradient-v1` and occurrence payloads retaining corner/degree directions, unpremultiplied ARGB colors, omitted stops and signed pixel/percentage positions. Validation enforces2–256 stops, finite numeric values, unique non-root LayoutComponent targets and capability/version agreement. Four gradient contract tests and six opacity regression tests pass; the full library suite also passes. Version26 can coexist with opacity without weakening prior version checks. Public CSS emission remains gated while host installation, JS types and native/WASM parity are connected. Receipt: `output/playwright/html-to-riv/linear-gradient-contract-progress.json`.

### Gradient checked recording host and JS types — 2026-09-10

The probe validates version26 before drawing, converts retained gradient values into checked runtime paints, and installs occurrence targets atomically. Host tests pass four responsive widths,13 malformed-manifest controls and explicit missing-capability rejection; runtime tests pass original/clone resize and replacement/clear controls. JavaScript version26 declarations expose exclusive direction/position variants and pass typechecking. This qualifies the recording-host adapter only; public CSS emission, native/WASM parity, public native lifecycle pixels and unsupported-backend preflight remain. Receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

Gradient host regression completes24/24 after rebuilding the probe with required `native-glyph-controls`. Initial21/23 attempt is retained: both failures explicitly rejected the missing native-glyph build feature. Updated receipt: `output/playwright/html-to-riv/linear-gradient-host-progress.json`.

### Public gradient initial validation passes — 2026-09-11

Fresh native/WASM parity passes6 tests; public128 original-clone resize frames pass pinned Chrome geometry/native pixels. Five depth/element/hidden/source-target boundary tests pass. Fresh public artifact audit passes16scenes/128frames and23 negative controls reject mutations. All128 authored-source/full-image pairs match reviewed runtime diagnostics; specialized visual transfer audit remains pending. New composition corpus has54 Chrome reference images across18 accepted cases, including default image repetition beneath transparent borders. Broader semantics/performance/full regression remain; P06 is active, not qualified. Evidence: output/playwright/html-to-riv/linear-gradient-public-integration-progress.json.

### Gradient composition defect confirmed — 2026-09-11

Initial public visual transfer audit completes128/128 with zero new inspections, based on exact authored HTML/CSS and full browser/native PNG identity plus independent public provenance. Broader composition native r1 compares54 views:30 pass,24 fail, all geometry passes. Eight cases fail at each width, including transparent/translucent/corner/wide/rounded borders, content clipping and opacity overlap. Inspected horizontal transparent-border pair confirms Chrome repeats endpoint colors beneath borders while current native paint clamps. Correct fix needs2D tile coordinates wrapped before scalar gradient projection; simply repeating scalar t cannot represent diagonal/corner tiles. Evidence: linear-gradient-public-lifecycle-r1/visual-public-gradient-transfer.json and linear-gradient-composition-native-r1/receipt.json under output/playwright/html-to-riv. P06 remains active.

### Repetition implementation candidate — 2026-09-11

Public compiler regression before tile changes completes453 tests across79 suites. New checked tiled-premultiplied paint transport preserves existing untiled commands; API36 and stream22 tests pass. Candidate native gradient retains immutable tile metadata through modulation; runtime requests tiling for borders, shader auxiliary data preserves2D UV and wraps before projection. Native build/pixel verification pending; original24/54 composition failures remain evidence. Initial performance capture completes80 measured runs plus16warmups, recording end-to-end replay only (not GPU/frame timing). Evidence: linear-gradient-public-module-receipt.json, linear-gradient-tile-transport-tests.log and linear-gradient-performance-initial/receipt.json under output/playwright/html-to-riv.

### Tiled composition pixels corrected — 2026-09-11

Frozen tiled toolchain builds and solid smoke pass. Composition r3 now54/54 automatic comparisons pass, versus30/54 before; targeted border gate36/36 samples passes and detects the earlier3 fractional-border visual failures. Transport58 and coordinate3 tests pass. Corrected full-image review/artifact audit, tiled original-clone composition lifecycle, backend/modulation/performance and full regression remain. Initial128-frame replay is running. Evidence: output/playwright/html-to-riv/linear-gradient-tile-progress.json.

### Tiled lifecycle and renderer regression pass — 2026-09-11

All18 public composition scenes compile once and pass144 original/clone Chrome/native frames plus fresh lifecycle artifact audit. Every authored-source/full-image pair equals a reviewed static composition; final transfer audit underway. Corrected static audit passes54/54,27direct/27exact transfers. Full native renderer suite479pass0fail6ignored; host synthetic16images/1328analytic samples pass across bothMetalpaths/affine/modulation/sharedstops. Current compiler regression and broad integrated native/performance qualification remain. Receipt: output/playwright/html-to-riv/linear-gradient-tile-progress.json.
