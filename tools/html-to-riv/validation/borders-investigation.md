# P01 uniform solid borders investigation

Status: runtime experiment in progress, 2026-09-10. Internal border parsing/cascade, retained layout edges and opt-in ring painting are implemented. Diagnostic solid432/432 and transparency192/192 Chrome/native frames pass; visual coverage is80/432 and8/192 respectively. Public compiler border admission remains disabled. The implementation inventory below describes the original investigation baseline, not the current code. Scope remains the standalone compiler/runtime; Chrome153.0.8010.12 is the reference.

## Sources and present implementation

Local links below are relative to this validation directory.

- [Compiler style model](../src/style.rs) has scalar `radius`, `content_box`, padding, background and overflow; its `emit` writes padding and corner radii but has no border width/style/color model. [Compiler emission](../src/lib.rs) adds a `Fill`/`SolidColor` for background and enables CSS pixel bounds for backgrounds or two-axis clips. A border-only node needs paint and pixel-bound treatment too.
- [Reset](../src/reset.css) sets `box-sizing:border-box` and `border:0`. Preserve this authoring contract when adding CSS border initial values.
- [LayoutComponentStyle](../../../crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs), `apply_style`, already transports four border dimensions to YGStyle. [LayoutStyleApplier](../../../crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs) copies them into `taffy.border`. This is layout machinery, not evidence of CSS border painting.
- [LayoutComponent](../../../crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs), `apply_base_style`, selects Taffy content-box via an occurrence policy. Its solved-output loop currently retains `output.padding` but not `output.border`; `inner_width`/`inner_height` subtract padding only. `CssOverflowClipMargin::bounds` explicitly assumes border and padding origins coincide. `begin_layout_clip` uses the background path for ordinary two-axis clips and outer extents for axis clips. These assumptions must change for bordered CSS occurrences.
- The same runtime file's `paint_geometry` applies CSS pixel snapping; `update_render_path` builds one rounded background path for existing shape paints. A centered stroke on that path would straddle the box edge, so it is not sufficient evidence for the required border ring.
- [CSS clip path](../../../crates/nuxie-runtime/src/mechanical_port/source/css_clip_path.rs) already supports elliptical corner coordinates internally and Chrome's coverage-adjusted outsets, but its public helper accepts circular reference radii. A padding-edge reference derived from borders must be represented explicitly.
- [Deferred content-box border corpus](content-box-border-deferred-cases.json) contains 80 row/column/reversed, sizing/limits/basis controls. [Content-box work record](content-box-contract.md) records why border probes were deferred; reuse the corpus without relabeling old browser captures as public admission.

## Primary CSS semantics

The border shorthand independently resets omitted components to their initial values: width `medium`, style `none`, color `currentcolor`; `none`/`hidden` have zero computed width. It also resets border-image. Width keywords map to 1/3/5px in the current specification. Negative widths are invalid. Uniform solid painting fills the region between outer and inner edges. Radius normalization prevents adjacent corners overlapping; inner radii subtract border thickness and clamp to zero. Background defaults paint beneath the border. These requirements come from [CSS Backgrounds and Borders 3, borders](https://www.w3.org/TR/css-backgrounds-3/#borders), [corner shaping](https://www.w3.org/TR/css-backgrounds-3/#corner-shaping), [overlap normalization](https://www.w3.org/TR/css-backgrounds-3/#corner-overlap), and [background clipping](https://www.w3.org/TR/css-backgrounds-3/#background-clip).

`currentcolor` remains symbolic until resolved against the element's own color. Explicit inheritance of that keyword still uses the child's color; inheritance of an absolute color remains absolute. Resolve after the winning color declaration, not when the border token is encountered. [CSS Color 4, resolving other colors](https://www.w3.org/TR/css-color-4/#resolving-other-colors).

A specified content-box width excludes padding and borders; border-box width includes them, with a nonnegative content-size floor. The sizing choice also affects min/max dimensions and flex-basis; it cannot be implemented by adding a fixed amount to authored width alone. [CSS Sizing 3, box-sizing](https://www.w3.org/TR/css-sizing-3/#box-sizing).

For example, proposed analytic controls with width80, height40, padding `5px 11px 7px 13px`, border3: content-box outer size110×58, content origin16×8; border-box content size50×22, same content origin. With border-box width0/height0, the inner floor requires a30×18 outer box. These are expected model values, not recorded Chrome results.

Widths require a separate device-pixel snapping decision: positive values below one device pixel snap up, otherwise fractional device pixels round down. Check actual computed styles and layout at the pinned browser's DPR/zoom; do not reuse edge-position rounding as width rounding. [CSS Values 4, border-width snapping](https://www.w3.org/TR/css-values-4/#snap-a-length-as-a-border-width).

Hidden overflow clips at the padding edge. Two-axis clip uses the overflow clip edge; its default origin is padding-box, with content-box and border-box selectable. The published overflow snapshot specifies nonnegative offsets. [CSS Overflow 3](https://www.w3.org/TR/css-overflow-3/#overflow-clip-margin). The project's pinned browser evidence accepts signed offsets and ignores margins for hidden or single-axis clipping; preserve those measured profile differences rather than silently substituting the published grammar. [Existing investigation](overflow-clip-margin-investigation.md).

Chromium source is pinned locally in [the Chrome source receipt](../../../output/playwright/html-to-riv/overflow-clip-margin-chrome-source/receipt.json). It cites [153.0.8010.12 ContouredRect](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/geometry/contoured_rect.cc), [FloatRoundedRect](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/platform/geometry/float_rounded_rect.cc), and [paint property tree construction](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/paint/paint_property_tree_builder.cc). The retained source enables coverage-adjusted corner outsets. Border inner-edge construction and later clip-margin expansion need distinct stages; a shadow-outset formula alone is not a border painter.

## Initial browser/admission baseline

The coordinating task captured54 scenes at three viewport widths,162 captures total, using Chrome153.0.8010.12. Inputs are [border-solid-cases.json](border-solid-cases.json); browser evidence is retained in [border-solid-initial-oracle](../../../output/playwright/html-to-riv/border-solid-initial-oracle). All54 frozen-compiler admission attempts rejected; diagnostics are retained in [border-solid-initial-admission](../../../output/playwright/html-to-riv/border-solid-initial-admission). These are browser references and rejection evidence, not renderer parity or feature qualification. Fractional-width computed-style probes remain outstanding.

## Proposed admission boundary

This is a deliberately bounded proposal, subject to the Chrome matrix and implementation evidence.

| Area | Candidate acceptance | Explicitly deferred/rejected |
|---|---|---|
| Properties | `border`; single-value `border-width`, `border-style`, `border-color` | Side/logical border properties, multi-value edge shorthands, border-image |
| Width | Finite nonnegative px/em/rem, unitless zero, thin/medium/thick | Percentages, negative values, unitless nonzero, other units, math functions, nonfinite values |
| Style | solid plus none/hidden as zero-width reset states | dotted/dashed/double/groove/ridge/inset/outset |
| Color | Existing named sRGB, hex, rgb/rgba, hsl/hsla parser; transparent; currentColor | Unsupported color functions, system colors, color spaces |
| Radius | Existing single uniform scalar length profile | Per-corner, percentage, slash/elliptical authored syntax, multi-value radius |
| Cascade | Existing importance, variables/fallbacks and supported initial/inherit/unset behavior | New cascade/revert behavior outside the current compiler contract |

Width em must use final computed font size, rem the existing root convention. Keep specified width separate from effective width: `border-width:8px; border-style:none; border-style:solid` must recover8. Shorthand parsing must accept component order permutations and comments while rejecting duplicate components. Invalid substituted CSS values should follow the existing invalid-at-computed-value policy; supported versus invalid CSS must remain distinguishable in diagnostics. Do not accept an unsupported side declaration simply because a later declaration happens to make the final border uniform unless a complete side cascade model is intentionally implemented.

Reset discriminators to capture: `border-style:solid` alone retains reset width0; `border:solid` resets width to medium; `border:8px` resets style to none; `border:8px solid; border-color:transparent` reserves layout space; `border:8px solid currentColor; color:red` resolves red. Test CSS-wide values on all four proposed properties, including inherited currentColor versus inherited explicit color.

## Required live runtime design

Retain used border edges alongside used padding in solved layout and clones. Make changes to width, height, border, padding and radius invalidate all dependent layout/paint/clip paths. Audit inner-size consumers, text wrapping and image sizing, auto/intrinsic dimensions, aspect ratios, percentages, flex-basis/min/max, absolute containing blocks, and original/clone clearing. The existing positioned runtime already reads Taffy's border edges when deriving containing blocks; this is a useful control against a new duplicate inset.

Resolve three live boxes: border `[0,0,W,H]`; padding inset by border; content inset by border plus padding. The border ring should have independent outer/inner contours and one alpha application, and its paint must survive the node's own overflow clip. Opaque and transparent backgrounds expose errors that a solid background hides. Uniform radius can stay circular at the padding edge; asymmetric padding produces elliptical content-edge radii. Normalize outer geometry before deriving inner geometry and handle empty inner boxes explicitly.

For ordinary hidden/default clip, use the curved padding edge. For margin clips, first resolve the selected edge and then apply the pinned Chrome corner-outset policy. For one-axis clip, use the appropriate live inner endpoints and preserve its unrounded orthogonal strip; confirm endpoints with browser evidence. Apply equivalent clips to ordinary and deferred positioned/stacking descendants. Preserve imported Rive behavior behind an occurrence policy if CSS semantics differ.

## Concrete validation matrix

Capture exact HTML/CSS/reset, Chrome version, viewport, DPR, zoom, computed border widths/styles/colors, all three box bounds or enough fields to reconstruct them, child rects, screenshots and hashes. Use widths240/390/768 and unchanged geometry/pixel gates. The initial geometry row has browser/admission baseline captures described above. The remaining rows are proposed expansion; no renderer passing count is claimed.

| Group | Inputs | Discriminator |
|---|---|---|
| Initial geometry | widths1/8/24 × radii0/18/90 × content-box/border-box × visible/clip/hidden, asymmetric padding, child crossing each edge | 54 scenes before viewport expansion; outer/inner ring and clipping |
| Existing layout debt | All80 deferred border scenes, original source intact | Both box modes, four flex directions, zero/auto/percent/limits/basis |
| Small dimensions | border8/24, dimensions0/10/40, radius0/4/90 | Nonnegative floor, radius below width, collapsed inner contour |
| Width quantization | widths0/.1/.5/.99/1/1.5/1.99/2.25, origins0/.25/.5/.75, DPR1/2 controls | Separate computed width, layout edges, paint snapping; DPR2 diagnostic unless admitted |
| Cascade | Reset examples above, component permutations, comments, duplicate/invalid tokens, important, variables, inherited explicit/current color | Correct defaults, no eager currentColor, no lost specified width |
| Alpha/paint | Transparent and 50% alpha border, transparent/opaque background, overlapping child, parent opacity | No painted interior, double alpha or missing border-only node |
| Clip origins | border8/24 × content/padding/border origin × margin-8/0/8/24 × radius0/4/18/90 | Previously coincident edges diverge; signed expansion/collapse |
| Overflow controls | hidden and clip/visible/visible-clip against identical margin variants | Margin no-effect remains; axis strip endpoints exclude border |
| Compositions | Text wrapping/ellipsis, image cover/contain, aspect-ratio card, absolute badge, negative/positive z descendants, nested clips | Content dimensions and paint ordering across modules |
| Lifecycle | Repeated240→768→390→240 resize on original and clone; toggle/clear applicable occurrence policies | Recomputed border/content/clip geometry, no stale path/cache |

Compiler unit/cascade and manifest validation should precede public replay. If experimental runtime injection is used, keep it explicitly separate. Then require public compiler output, native/WASM parity, both renderer profiles, full-source visual inspection and unchanged regression checks before qualification. Any retained mismatch is an unresolved result, not justification to widen tolerances.

## Internal width parser foundation

`src/border.rs` now parses finite nonnegative px/em/rem, unitless zero and thin/medium/thick. It preserves fractional and font-relative authored values; no device quantization is applied. Two unit tests pass, including invalid units, negative/overflow lengths and nonuniform lists. CSS-wide values remain cascade work. This is not connected to public admission or runtime painting. Log: `output/playwright/html-to-riv/border-width-parser-tests.log`.

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


### P02 diagnostic visual audit complete — 2026-09-10

All72 distinct three-width Chrome/native/difference views are directly reviewed.
The final six opaque unequal-width sheets cover square,18px and constrained90px
radii in both box models. No missing corner pockets or shifted color boundaries
were observed; sparse curve/miter antialias differences remain under unchanged
gates. Source replay hashes and every projected row were revalidated, and all192
original/clone frames match reviewed full-image pairs. The receipt
`output/playwright/html-to-riv/border-sides-diagnostic-completion.json` binds the
oracle, recordings, replay, reviews and seven-test geometry log. This is runtime
experiment evidence only. Public per-side transport/emission, expanded edges,
parity and full regression qualification are the next work.


### P02 checked runtime installer — 2026-09-10

`Artboard::set_css_border_sides_occurrence` now atomically replaces border
policies using top/right/bottom/left ARGB colors. Uniform installation delegates
to the same validator and replacement path. Both lifecycle tests pass: invalid
roots and target IDs leave prior paint intact; plain-layout proxies survive
resize, clone, clear and repeated reinstallation; switching to uniform paint
clears side overrides without changing the clone. Receipt:
`output/playwright/html-to-riv/border-sides-checked-installer-receipt.json`.
Compiler capability/manifest emission and public pixel/parity qualification
remain outstanding; these tests establish runtime installation behavior only.


### P02 version23 compiler contract candidate — 2026-09-10

Public emission now represents nonuniform used widths/colors with
`layout-css-border-sides-v1` and `layout_border_sides` entries containing physical
top/right/bottom/left ARGB arrays. Uniform used state retains version22 and its
existing bytes. The host probe combines both target lists into one atomic
installation; manifest validation rejects overlap, root IDs, malformed arrays,
missing capabilities and wrong versions. Five border contract tests and
TypeScript checks pass. Public rendering, native/WASM parity and expanded edge
qualification remain pending.

The first full module run exposed a default-stack overflow at the resource-limit
regression. It reproduced in isolation. Moving per-side calculation out of the
recursive emission frame fixes that regression without increasing stack limits.
Logs: `border-sides-v23-stack-repro.log`, `border-sides-v23-stack-fix.log`.
The corrected full run writes `border-sides-v23-module-tests-r2.log`; completion
is not yet established. This remains a candidate, not P02 qualification.


### P02 public focused validation — 2026-09-10

Corrected full Rust suite passes382 tests across70 groups. The added public
individual-side recorder compiles the original24 authored scenes and installs
only their version23 manifest policies; all192 original/clone geometry updates
and192 Chrome/native pixel comparisons pass. The frozen toolchain reproduces
each recorded Rive artifact and requirements. The dedicated audit checks exact
authored HTML/CSS, full PNG pairs, streams and review hashes against the fully
reviewed diagnostic corpus:192/192 public frames have complete visual coverage.
This transfer does not treat diagnostic injection as public compilation.

Expanded native/WASM corpus parity passes all13 JavaScript tests, and TypeScript
passes. Receipt: `output/playwright/html-to-riv/border-sides-v23-focused-receipt.json`.
P02 remains active: broader edge/composition fixtures, malformed-host transport
checks and full native visual regression are outstanding.


### P02 edge corpus and host checks — 2026-09-10

All21 host tests pass, including version23 mixed uniform/side policies and17
malformed-manifest rejection controls. Five negative visual-audit controls reject
changed Rive bytes, authored CSS, stream hashes, image hashes and missing frames.
Added40 Chrome edge scenes: transparent sides, none/hidden, currentColor, two/three
value lists, shorthand ordering, variables/invalid substitution, fractional em
widths and borders exceeding authored dimensions, at square/large radius in both
box models. Public original/clone geometry and pixel replay pass320/320.

Visual review has begun:6/120 distinct views reviewed (overfull radius90 in both
box models). Used-border minimum sizing and colored wedge geometry agree with
Chrome; sparse curve/diagonal antialias differences remain.114 distinct views
remain unreviewed; expanded corpus parity and full regression are outstanding.
Receipt: `output/playwright/html-to-riv/border-sides-v23-edge-progress.json`.


### P02 edge parity and transparent/hidden review — 2026-09-10

The40 edge scenes are now included in the native/WASM corpus; all13 JavaScript
tests pass against the frozen version23 toolchain. Eight additional original-
resolution sheets (transparent top and none/hidden sides, square/large radius,
both box models) are directly reviewed. Space retention/removal, joins, child
clipping and sibling placement match structurally, with sparse antialias
differences under unchanged gates. Visual audit now30/120 distinct views; exact
full-image identity covers80/320 lifecycle frames.90 distinct views remain.
Projection/source hashes and rows were verified before writing
`border-sides-v23-edge-static-review/lifecycle-review-progress.json`.


### P02 equal-color seam found by visual review — 2026-09-10

Direct currentColor square-border review found a light diagonal join absent
from Chrome despite passing aggregate metrics. Failure receipt:
`border-sides-v23-edge-static-review/current-color-seam-review.json`. Started
7372-check full regression session98289 was interrupted (exit130); its frozen
pre-fix artifacts are preserved and cannot qualify the correction.

Runtime now combines all partitions with the same ARGB color into one NonZero
clip before painting the ring. This removes internal antialias seams. New
regression verifies paint grouping through clone, clear and uniform replacement.
Corrected edge recording and replay pass320/320 frames. Directly inspected the
corrected square currentColor390px native image; the light purple diagonal seam
is gone, and browser bytes match the previously viewed reference. Other changed
images still need review; prior edge review is not automatically transferred.
Receipt: `output/playwright/html-to-riv/border-sides-grouped-paint-receipt.json`.


### P02 corrected currentColor review and frozen full run — 2026-09-10

All12 currentColor views are directly inspected in the corrected replay: square
equal-color joins no longer show light seams; rounded edges, translucent right
border, child clips and sibling placement agree structurally. Partial-source
review transfer checks exact authored HTML/CSS, manifest, Rive bytes and both
full PNGs, plus projection/source hashes:18 earlier views remain identical.
Corrected coverage is30/120, with90 still requiring review; changed none/hidden
images were not transferred. Receipts are in
`border-sides-grouped-edge-static-review/{visual-inspection,prior-review-transfer}.json`.

Rebuilt/frozen `border-sides-grouped-toolchain`; all21 host-contract tests pass.
The corrected7372-check full native run is live as session65603; status receipt
`output/playwright/html-to-riv/border-sides-grouped-full-run.json`. Completion and
full visual audit remain pending.


### P02 corrected none/hidden and two-value style review — 2026-09-10

Eight full-resolution three-width sheets inspected: corrected none/hidden
physical edges and two-value none/solid styles, square/large radius in both box
models. Removed edge space, blue side crescents, red/green joins, child clips
and sibling placement match structurally; sparse curved and different-color
join antialias differences remain. Combined corrected coverage is54/120
(36 direct plus18 audited exact-input/artifact/full-image transfers).66 views
remain. Prior-transfer input, artifact and image hashes were revalidated.
Full regression session65603 was polled live and remains in progress.


### P02 corrected three-value list review — 2026-09-10

Twelve three-value width/color views directly inspected: thin red top, repeated
green sides and thick blue bottom match Chrome at all widths in both box models,
with square and large-radius joins. Child clips, background and sibling placement
agree structurally; sparse curve/different-color diagonal antialias differences
remain. Corrected coverage is66/120 (48 direct,18 transferred);54 remain.

`validation/audit-border-edge-review-progress.py` now reproduces the combined
partial audit, checking direct-review receipts, projection rows/source hashes,
public recording hashes, exact authored HTML/CSS/manifests/Rive bytes and full
image pairs. It does not transfer changed or unreviewed source rows.
Full regression remains underway; latest log observation passed check449.


### P02 corrected shorthand and variable review — 2026-09-10

Eight original-resolution sheets inspected for shorthand ordering and variable
width/color lists, at square/large radius in both box models. Later shorthand
reset, final bottom override, physical edge order, unequal widths, child clips
and sibling layout match Chrome structurally. Equal-red joins remain seamless;
sparse different-color/curve antialias differences persist under unchanged gates.
Combined corrected audit passes90/120 views (72 direct,18 exact-input/artifact/
image transfers), leaving30. Full native session65603 remains live.


### P02 corrected edge visual audit complete — 2026-09-10

All120 corrected edge views are visually accounted for:102 directly inspected
and18 transferred only after exact authored input, manifest, Rive artifact and
full-image identity checks. Final inspection covers invalid-variable medium
reset, fractional em widths and square overfull borders in both box models.
No equal-color seams remain in the reviewed cases; sparse different-color join
and curve antialias differences remain under unchanged criteria.

The frozen compiler reproduces all40 public recorded Rive artifacts and
requirements. All320 original/clone frame images match reviewed pairs and their
current hashes. Receipt: `output/playwright/html-to-riv/border-sides-grouped-edge-completion.json`.
Full regression session65603 remains live. Broader text/image/flex compositions,
corrected runtime regression completion and full visual audit remain open.


### P02 text/image and clipping compositions — 2026-09-10

All14 corrected border runtime tests pass. Added48 composition scenes derived
from the P01 text/image and axis/clip-margin cases, with unequal2/8/14/20 widths
and repeated translucent side colors. Pinned Chrome captures144 references.
Public compilation and checked installation pass384 original/clone geometry
updates. Native-glyph recording is complete; pixel replay session31879 and
expanded native/WASM parity session1072 are running. Visual review has not yet
begun for these compositions. Full regression session65603 remains active.
Receipt: `output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`.


### P02 composition replay passes; text/image review — 2026-09-10

Composition replay exits0 with384/384 geometry/pixel frames passing. Expanded
native/WASM parity passes13/13 tests. Eight original-resolution three-width
text/image sheets inspected for square/24px radius and both box models: matching
wrapping, intrinsic heights, image quadrants, translucent border blends, clipping
and sibling position. Equal-color corners remain seamless; sparse glyph/curve
and image filtering differences retained under unchanged gates.

Visual audit covers66/144 distinct views (24 direct,42 exact within-run
image transfers), with78 remaining. Receipt:
`output/playwright/html-to-riv/border-sides-grouped-composition-progress.json`.
Full regression remains pending.
