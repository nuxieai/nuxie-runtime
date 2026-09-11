# Aspect-ratio math reference

Chrome153.0.8010.12 is the target. The compiler currently rejects all16 prospective
math fixtures. The immutable publisher hash and individual diagnostics are in
`output/playwright/html-to-riv/aspect-ratio-math-admission/native-before.json`.
Browser admission/computed values for16 positive and7 negative controls are in
that directory's `probe.json`. Geometry and48 screenshots are captured separately
in `aspect-ratio-math-oracle/`. Captured screenshots are not yet visually reviewed.

The CSS Values specification permits math in numeric positions and clamps
out-of-range top-level results to the target range. Clamping inside intermediate
expressions would change their meaning. See
[CSS Values range checking](https://www.w3.org/TR/css-values-4/#calc-range).

Measured Chrome behavior:

- `calc`, `min`, `max`, `clamp`, nested parentheses/functions and math on either
  side of the ratio slash are accepted. Optional auto works in either position.
- Negative calculation results clamp to zero separately for numerator and
  denominator; negative literal components remain invalid.
- `calc(0 / 0)` computes to zero; `calc(1 / 0)` serializes as infinity. The latter
  requires a runtime representability decision, not blindly emitting infinity.
- `calc(1e-20 * 1e-20)` computes to zero in this pinned Chrome. Do not assume that
  arbitrary double-precision evaluation reproduces browser numeric behavior.
- Dimensions, percentages, missing operands, wrong clamp arity and missing
  required whitespace around addition are rejected by the browser controls.

Implementation work remains: a bounded scalar evaluator with operator precedence,
CSS token/whitespace rules, nested function arity checks, top-level range handling,
resource limits, substitution/cascade validity and runtime representability tests.
There is no shared math evaluator in the module today. The substitution validity
classifier recognizes math for selected properties; aspect-ratio integration must
be audited so valid unsupported expressions are not silently discarded.

These are open implementation tasks, not external dependencies. Natural image
sizing remains separate L14 work. Do not claim math support from this research.


A public regression confirmed that `--ratio:calc(3 / 2); aspect-ratio:var(--ratio)`
previously compiled after treating the substituted value as invalid and applying
unset. Direct math was rejected. The property/function compatibility classifier
now recognizes numeric math as potentially valid for aspect-ratio, so unsupported
math through custom properties and fallback expressions diagnoses instead of
silently changing the scene. Four aspect-ratio public tests pass, including12
direct/variable/fallback cases across calc/min/max/clamp. Red and green logs are
preserved in the admission directory. This fixes rejection semantics only;
evaluation and rebuilt native/WASM diagnostic parity remain pending. The running
full native regression uses its unchanged frozen pre-fix snapshot.

All51 custom-property regression tests also pass; log: `custom-properties-pass.log`.


The internal `src/number_math.rs` evaluator now parses scalar calc/min/max/clamp,
operator precedence, nested functions/parentheses and required sum whitespace.
It propagates NaN rather than using Rust min/max NaN suppression. Range clamping
is intentionally left to the property caller. Limits are32 nesting levels and
256 value terms, exercised by tests. Four unit tests pass, with source hash and
log in `number-math-receipt.json` / `number-math-unit-pass.log`.

This evaluator is not yet connected to CSS admission or AspectRatio::parse.
Tokens currently supply f32 literal values, with f64 intermediate arithmetic;
Chrome numeric-boundary behavior must be qualified before accepting public math.
The direct stylesheet tokenizer and ordinary substituted-value normalization
currently reject nested non-color functions; integration must preserve raw math
whitespace/comments and existing rejection behavior for unrelated properties.


Public integration is now implemented experimentally. Property-specific admission
preserves raw math tokens and leaves unrelated property normalization unchanged.
Fourteen unit tests, five aspect-ratio tests and51 custom-property tests pass.
Fourteen math oracle scenes pass112 original/clone instance-viewports. The two
infinity/subnormal cases remain rejected and are retained as `unqualifiedCases`
in the checked-in oracle. The complete original16-case Chrome capture is intact.
Logs: `integration-red.log`, `integration-green.log`, `integration-suite.log`,
`public-oracle.log` in the admission directory. Rebuilt binary parity and native
pixel qualification remain open; old frozen regression does not validate this code.


Frozen `aspect-ratio-math-toolchain` native/WASM builds succeeded. Fourteen scenes
pass42/42 Chrome geometry/native pixel comparisons with complete visual review
(12 direct,30 exact full-image transfers); receipt integrity audit passes.
Ten JS tests pass, including2311 corpus inputs and six direct/variable/fallback
numeric-limit diagnostic parity cases. Evidence and logs are in
`aspect-ratio-math-native/receipt.json`. Full module regression finished successfully: 258 tests across 53 test-result
groups, with zero failures. The retained log is
`aspect-ratio-math-native/full-module-preprecision.log`; this run predates the
double-literal precision correction and does not validate that later change. Numeric precision,
infinity/subnormal semantics, broader cascade validity and responsive composition
qualification remain open. No full qualification claim or tolerance changes.


### Double-literal precision correction

Chrome probes demonstrated a real evaluator bug: cancellation at16777217 and
100000001, and ratios of1e40 or1e-50 literals, all evaluate to1 in Chrome.
The cached f32 tokenizer value lost those results. Numeric math now parses the
original validated number token as f64 before arithmetic. The new regression
failed before the change and passes afterward; all15 unit and five public
aspect-ratio tests pass, including direct/variable/fallback cancellation cases.
Red/green logs and Chrome measurements are in `aspect-ratio-math-precision/`.
The existing frozen math toolchain and its parity/pixel receipts predate this fix.

A further48-row Chrome probe is preserved in `aspect-ratio-math-precision-v2/`.
Small-positive handling remains a correctness gap: calculated1e-6 remains a ratio
but1e-7 and smaller measured powers compute to zero. At1e-6 Chrome also caps the
resulting layout height. The compiler's current subnormal-only rejection does not
cover this range. Investigate the browser's numeric conversion/range rules and
preserve these cases as failing reproducers before qualifying small ratios.
Do not infer that the previously passing14-scene corpus covers these boundaries.

### Chrome component-clamping boundary: measured, not yet implemented

The 78-row `aspect-ratio-math-precision-v3/probe.json` uses Chrome
153.0.8010.12. Both literal and calculated components at or below
0.00000095367431640625 (eight f32 epsilons) compute to zero. The halfway
rounding neighbor 0.0000009536743732496689 also rounds to the threshold and
computes to zero. The next f32 value, 0.0000009536744300930877, remains
nonzero. Equal tiny numerator/denominator components become 0/0 rather than
being divided to one first; equal components just above the threshold yield
one and a 120px height. A denominator-only tiny value becomes 1/0.

Primary-source explanation: Chromium's
[GetRatioFromList](https://github.com/chromium/chromium/blob/main/third_party/blink/renderer/core/css/resolver/style_builder_converter.cc)
converts both computed components into
[SizeF](https://github.com/chromium/chromium/blob/main/ui/gfx/geometry/size_f.h),
whose constructor clamps each f32 component using `f > 8 * epsilon`.
Downloaded source files and hashes are retained in the v3 directory with
`source-receipt.json`. These main-branch sources explain independently measured
browser behavior; they are not claimed to be the exact installed build sources.

A second boundary remains: the first retained nonzero component divided by one
still produces zero layout height, whereas 1e-6 produces Chrome's capped
33554432px height. `StyleAspectRatio` separately converts the float pair with
`LayoutRatioFromSizeF`, so component clamping alone cannot reproduce layout.
Next inspect that conversion and add browser-backed public regressions covering
both boundaries before changing admission or claiming numeric-limit support.
The compiler still has the recorded small-component mismatch. No tolerances or
qualification status changed.


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


Saturation validation update: full compiler suite267 tests across53 groups
passed. Initial12 and stress72 Chrome/native geometry + Rust Metal pixel
comparisons pass; all84 visually reviewed (36 direct,48 exact-image transfers)
and both integrity audits pass. Receipts in `aspect-ratio-saturation-native-v3`
and `aspect-ratio-saturation-stress-native-v3`. This closes the reproduced
ratio-derived size cap gap for the tested28 scenes/224 original-clone viewports.
The prior2365-input parity job45256 is running; new28 saturation inputs were
added to the parity corpus after that job started and require a subsequent run.
Full L13 replay/cascade qualification remains pending. No tolerance change.


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


### Exact ratio pair follow-up (Chrome font-boundary evidence)

The font-boundary case1-0 computes a layout pair1/15999. Reducing it to the
existing f32 wire ratio loses enough precision to cause0.25/0.5px errors at
390/768px. The preserved arithmetic diagnosis matches Chrome when it keeps the
pair; an f64 calculation on the rounded scalar does not. Both candidate and
restored-runtime test logs are retained under
`output/playwright/html-to-riv/aspect-ratio-font-boundary-precision-investigation/`.

Implementation must preserve computed semantics, not the authored expression:

1. Retain the positive integer layout pair produced by `layout_ratio`, including
   the existing clamping and continued-fraction decisions. Inheritance copies
   this computed pair; custom properties still evaluate in the consumer context.
2. Add an explicitly versioned runtime capability/requirement for the pair.
   Keep the scalar wire property for ordinary Rive consumers, but do not claim
   the new precision semantics without the required capability. Validate both
   components, object targets, capability/version consistency and malformed data.
3. Propagate the pair through occurrence policy, style construction and cloning.
   Apply it to every ratio transfer path: initial dimensions/constraints,
   intrinsic column basis and post-flex cross size. Preserve ordinary Rive
   behavior when the CSS requirement is absent. Match Chrome fixed-point
   transfer rounding and saturation, not just the simplified numerical example.
4. Update the native loader/probe, public oracle installers, JavaScript types and
   parity controls together. Exercise original and clone resize cycles, explicit
   and auto ratios, content/border boxes, both flex axes and large/tiny values.
5. Build fresh frozen native/WASM artifacts; run the32-scene boundary corpus and
   broader existing ratio cohorts, inspect native/browser images, audit review
   receipts, and report remaining limitations. Existing qualification snapshots
   predate these changes and cannot qualify the new contract.
