# Overflow clip margin investigation

Status: public version21 implementation; focused geometry, pixels and visual audits complete. Full regression and regular-suite integration qualification are complete. The dated sections below preserve the investigation history, including the initial nonnegative assumption that Chrome153 evidence later disproved.

The current compiler accepts a content-box, padding-box or border-box origin and an optional finite signed px/em/rem length, in either order; unitless zero is accepted. Negative offsets contract the clip. It applies nondefault policies only to computed two-axis clip overflow. Percentages, math functions, duplicate components and unitless nonzero lengths are intentionally unsupported. Border and padding origins coincide until border support is implemented. CSS-wide keywords and custom-property substitution are covered by the public cascade tests.

Current evidence includes54 signed and60 expanded comparisons per renderer profile,216 composition comparisons per profile,144 initial signed-toolchain native comparisons,192 live original/clone frames and2,424 combined overflow resize updates. Focused visual audits are complete, including72 directly inspected vector text composition views and144 exact-source/image transfers. See `output/playwright/html-to-riv/overflow-l20-receipt.json` for receipts and retained failures. No pixel thresholds were widened.

## Initial hypothesis (superseded for signed lengths)

The [CSS Overflow specification](https://www.w3.org/TR/css-overflow-3/#overflow-clip-margin) defines a nonnegative length and optional visual-box origin, defaulting to padding-box and zero. Percentages and negative lengths are invalid. The property is non-inherited and does not affect hidden overflow. Rounded clip edges use the shadow-spread corner rules; single-axis clip/visible remains unrounded.

## Pinned Chrome evidence

48 scenes / 144 captures from Chrome153.0.8010.12 cover clip, hidden, clip/visible and visible/clip; content/padding origins; margins0/8/24px; radii0/24px. Padding is asymmetric and content extends across all clip boundaries. Source: overflow-clip-margin-cases.json. Outputs: output/playwright/html-to-riv/overflow-clip-margin-initial-oracle.

Full-image hash discriminators show all12 two-axis clip groups change with margin, while all12 hidden and all24 single-axis groups remain identical. Computed-style capture confirms Chrome accepts and retains content-box24px in all four overflow modes. Thus unchanged single-axis captures are not caused by rejected declarations. This browser behavior must be represented explicitly in the compiler profile and tested, rather than assuming axis strips expand.

All48 inputs currently reject with unsupported-property using the frozen proxy-fix compiler. Admission receipt preserves requests and diagnostics in overflow-clip-margin-initial-admission. No feature is admitted by this research.

## Next implementation work

Resolve content/padding origins from live layout dimensions and padding. Keep clip geometry separate from background geometry. Add checked runtime transport and clone/clear lifecycle for rounded two-axis clip bounds. Specify grammar, CSS-wide values and custom-property behavior; add default/omitted/border-box, em/rem and rejection cases. Inspect reference images and compare actual renderer pixels before qualification. Preserve the single-axis Chrome no-effect controls and hidden controls. Do not widen tolerances.


### Clip-margin value parser (2026-09-10)

Added internal ClipMargin parser with content/padding/border origins, optional nonnegative px/em/rem offsets in either order, unitless zero, case-insensitive identifiers and CSS comments. em uses the supplied computed font size; rem uses the documented16px root. Two focused tests pass, covering accepted forms and duplicate components, negative/percentage/unitless-nonzero/unknown-unit/nonfinite rejection. CSS-wide values stay in cascade handling; math functions are not yet supported. This parser is not connected to public CSS admission: runtime clip bounds, transport, cascade and pixel qualification remain to implement. Log: output/playwright/html-to-riv/overflow-clip-margin-parser.log.


### Live clip-margin bounds foundation (2026-09-10)

Runtime `CssOverflowClipMargin` now validates finite nonnegative offsets and resolves local clip bounds from the current LayoutComponent size and used asymmetric padding. Content origin insets each edge before expansion; padding and border origins coincide while compiler borders remain unsupported. The resolver rejects nonfinite resolved edges and does not mutate layout or background paint. It is not yet connected to drawing, transport or public CSS admission. Focused runtime tests cover repeated size/padding changes and invalid/overflowing numeric inputs; log: output/playwright/html-to-riv/overflow-clip-margin-runtime-bounds.log. Both focused runtime tests passed (2/2); this is geometry-unit evidence only, not renderer or browser qualification.

Corner implementation reference: [CSS Backgrounds 3 shadow shape](https://www.w3.org/TR/css-backgrounds-3/#shadow-shape) adjusts small radii with the cubic spread factor independently in each dimension. Existing Path::add_rounded_rect takes scalar circular radii and clamps each to half the smaller extent; it cannot represent content-edge ellipses from asymmetric padding. Add a separate CSS clip path, keeping authored/background paths unchanged, and qualify against the captured Chrome images.


### Clip-margin rounded runtime experiment (2026-09-10)

Separate CSS clip geometry and occurrence policy are implemented internally. Content-edge outsets use asymmetric live padding; circular border corners may produce elliptical clip corners. The background paints before its own margin clip opens, while ordinary and deferred descendants share the same clipping function. Clones retain the policy, and clearing restores imported clipping. Five geometry unit tests and nine integration tests pass, including 64 margin lifecycle frames plus the existing positioned/stacking suite. Logs: overflow-clip-margin-runtime-path.log and overflow-clip-margin-runtime-background.log under output/playwright/html-to-riv.

Pinned Chrome153 source confirms `ShadowContourFollowsBorder` is stable and overflow-clip-margin uses its coverage-adjusted radius formula. This supersedes the earlier assumption that the published snapshot's simple cubic spread rule alone is sufficient. Immutable source receipt: output/playwright/html-to-riv/overflow-clip-margin-chrome-source/receipt.json.

Public CSS admission and checked transport remain unimplemented. `replay-oracle.mjs --clip-margin-experiment` explicitly strips the unsupported declaration from the compiler request and records the diagnostic runtime injection alongside the original browser CSS. These runs are runtime experiments, not public compiler or native/WASM qualification. Numeric/pixel and visual review receipts remain required.


Initial clip-margin runtime experiment completed:144/144 Chrome geometry/pixel comparisons per renderer profile pass, with complete native visual audit and144 exact-input/full-PNG transfers to the explicit vector-v2 run. Runtime injection and original browser CSS are checked before review transfer. Sources and images remain recorded as experiments, not public compiler qualification. The first vector-named run accidentally used nativeGlyphs=1; preserved as a native repeat and excluded from vector evidence. See overflow-l20-receipt.json.clipMarginRuntimeExperiment. Public admission, checked transport, parity and broader live composition remain.


### Public clip-margin candidate: version21 (2026-09-10)

`overflow-clip-margin` now accepts an optional content-box, padding-box or border-box and an optional nonnegative px/em/rem length in either order, including unitless zero and comments. At least one component is required. Default is padding-box0. It is non-inherited; initial/unset reset it, and inherit copies the computed parent value. em uses the final computed font size; rem uses the documented16px root. Cascade, important and custom-property/fallback resolution apply. Negative lengths, percentages, duplicate components and malformed values reject; known invalid values substituted through var() become unset. Math functions and other length units remain unsupported.

The pinned Chrome profile applies the margin only to computed two-axis clip. Hidden and one-axis overflow retain their no-effect behavior. A nondefault effective margin emits version21 with layout-css-overflow-clip-margin-v1 and layout_overflow_clip_margins. The checked host rejects unsupported capabilities, malformed/duplicate/root/non-layout targets, negative/nonfinite offsets and conflicts with axis policies before drawing. Border/padding reference edges currently coincide because compiler border painting remains unsupported.

Full compiler348 tests, host19 tests, typecheck,48 native/WASM parity cases and1544 combined original/clone resize updates pass. Initial48 scenes/144 Chrome geometry/native pixel comparisons pass in each renderer profile. All144 public pairs match the reviewed runtime experiment; promote-clip-margin-review.py audits original browser CSS, the diagnostic-to-manifest policy change, all other compiler inputs/runtime policies, and full PNG identity. No diagnostic injection is used by these public runs. Evidence: output/playwright/html-to-riv/overflow-l20-receipt.json.clipMarginPublic.

L20 remains active for expanded small-radius/cascade controls, realistic text/image/positioning compositions, live resize pixels and full version21 regression. The prior axis proxy snapshot completed6271 checks and6261 exact-input/full-image visual transfers; this does not substitute for a new-runtime regression.


### Chrome153 signed-margin correction (2026-09-10)

The expanded20-scene corpus produced57/60 pixel passes and three preserved failures for a negative custom-property value. Direct image inspection showed Chrome shrinks the clip by8px while the compiler reset it to zero. A computed-style probe confirms Chrome153 accepts both direct negative lengths and negative var() substitutions; negative content-box values also remain negative. Percentages and invalid identifiers through var() reset to0px. Evidence: output/playwright/html-to-riv/overflow-clip-margin-negative-variable-computed.json and overflow-clip-margin-expanded-native/replay.json.

This corrects the earlier published-spec-based nonnegative assumption. The in-progress version21 candidate now admits finite signed px/em/rem offsets; negative margins contract clipping. The parser, substitution validator, runtime constructor and contract validation have been updated; focused tests are running. The original v21 toolchain and failing PNGs remain frozen. An18-scene direct-negative matrix adds square/small/rounded corners, content/padding origins and offsets-8/-24/-80px including empty clips. Signed public native/WASM rebuild, parity and pixel verification remain pending. Earlier348-test/144-pixel candidate receipts do not qualify the signed change.

Signed-margin focused compiler/contract/runtime tests6/6 pass. Native featured build and18-scene Chrome capture are running; signed native/WASM parity and pixels remain pending.


Signed-margin correction validation: native/WASM build and38 expanded/signed parity cases pass; host19 passes. Both renderer profiles pass all54 direct-negative comparisons and60 expanded comparisons, including the three preserved negative-variable reproducers. The public overflow test now covers231 scenes/1848 original-clone resize updates; the margin lifecycle test covers192 positive/negative/empty-clip frames with background and deferred-descendant clip-state assertions. Native visual audit currently covers27/54 signed and21/60 expanded views; remaining review is open. Frozen toolchain: overflow-clip-margin-signed-toolchain. Evidence: overflow-l20-receipt.json.clipMarginSignedCorrection.
