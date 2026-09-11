# L13 aspect-ratio investigation

## Current status (2026-09-09)

**Implemented for investigation; L13 is not qualified.** Chrome remains the
geometry and pixel reference. The sections below preserve the investigation
history; earlier statements about missing implementation or running jobs describe
their recorded stage, not current status.

The public compiler accepts literal numeric ratios, optional `auto` in either
order, and the documented cascade forms. Version14 requirements carry the
independent ratio reference-box policy. Images accept one automatic dimension
with a bare positive authored ratio; natural sizing and combined auto ratios
with an automatic dimension remain excluded pending L14. Numeric math functions
remain unimplemented.

| Evidence | Current result |
| --- | --- |
| Public Chrome geometry oracle | 282 scenes, 2,256 original/clone instance-viewports pass |
| Current shape renderer and visual review | 516/516 geometry/pixel pairs pass; all reviewed or exact-source/image transfers from audited reviews |
| Authored-ratio image renderer | 72/72 pairs pass and directly reviewed |
| Text/compositions | Native84/84 pass and reviewed; vector72/84 pixels pass, twelve reviewed failures remain unwaived |
| Native/WASM parity | 2,297 inputs, nine tests pass |
| Full module | 250 tests across53 targets pass |
| Full native regression | 5,983/5,983 pass on frozen image toolchain; all5,973 scene pairs have verified review transfers |
| Image flex stress | 56 scenes/448 public instance-viewports and168 native comparisons pass;168/168 visual pairs reviewed |

Current shape receipts are under
`output/playwright/html-to-riv/aspect-ratio-{initial,auto,expanded}-native-v3/`.
The original48 constraint failures remain preserved in `aspect-ratio-expanded-native/`.

Remaining work includes image flex stress, remaining text/vector review, numeric
math decisions and full regression review. Earlier content-box full-regression
evidence predates these runtime changes and does not qualify them. No tolerances
were changed.

## Original source investigation

Investigated 2026-09-09 before implementation. The following records the original
contract, source observations, risks and subsequent experiments.

## Standards contract to investigate

`aspect-ratio` accepts `auto || <ratio>`, defaults to `auto`, and is not inherited. Bare ratios use the box selected by `box-sizing`. `auto` uses a replaced element’s natural ratio, otherwise none. The September 2026 draft specifies that combined `auto` and ratio values prefer a replaced element’s natural ratio and always calculate ratios in the content box. Degenerate ratios behave as `auto`. A preferred ratio affects automatic dimensions; it does not enforce the ratio when both dimensions are definite. Min/max constraints can break the final ratio. Non-replaced content can impose a content-based minimum. These rules require Chrome probes, particularly the recently clarified combined-value box behavior. [CSS Sizing 4 §§4.1–4.4](https://www.w3.org/TR/2026/WD-css-sizing-4-20260904/#aspect-ratio).

Ratios are two nonnegative unitless numbers separated by `/`; omitting the denominator means one. Decimal/exponent number tokens must use CSS tokenization. Zero or infinite components are degenerate, rather than negative-value parse errors. [CSS Values 4 §5.7](https://www.w3.org/TR/css-values-4/#ratios).

Flex automatic minimums distinguish replaced elements (smaller of content and transferred suggestions) from non-replaced elements (larger), capped by a specified size suggestion and definite main maximum. A definite cross size can supply a transferred suggestion through the ratio. This makes testing images as ordinary containers insufficient. [Flexbox §4.5](https://www.w3.org/TR/css-flexbox-1/#min-size-auto).

The upstream WPT `block-aspect-ratio-050.html` specifically combines `auto 1/1`, border-box, asymmetric padding, and a fixed-height descendant. Adapt its structure to the compiler reset and capture the actual Chrome result before selecting runtime semantics. [WPT source](https://raw.githubusercontent.com/web-platform-tests/wpt/master/css/css-sizing/aspect-ratio/block-aspect-ratio-050.html).

## Proposed admission and exclusion decisions

These are implementation requirements to resolve, not an already accepted subset:

- Preserve a computed representation with an `auto` flag and optional numerator/denominator; reducing every declaration immediately to one float loses the combined-value distinction. Emit a scalar only after semantics have been resolved.
- Cover `auto`, `1`, `16/9`, `1.5 / .5`, `auto 16/9`, `16/9 auto`, capitalization, whitespace/comments, and CSS number exponents. Exercise custom properties, fallback, `!important`, cascade precedence, `inherit`, `initial`, and noninherited `unset`. Reuse the existing compiler cascade contract rather than creating a second parser.
- Valid degenerate `0`, `0/1`, `1/0`, and `0/0` must produce auto behavior without NaN/Infinity reaching layout. Test signed zero and very large/small numeric tokens. A positive finite quotient can overflow/underflow f32 even when its components are valid; define a diagnostic for unrepresentable values instead of silently changing their meaning.
- Reject negative components, dimensions/percentages, incomplete slash pairs, duplicate `auto`, extra ratios, and trailing tokens under the module’s strict diagnostics policy. Record whether each case is invalid CSS or valid CSS outside the supported compiler subset.
- Numeric `calc()`, `min()`, `max()`, and `clamp()` need an explicit decision consistent with existing numeric-expression support. They must not become accidentally accepted by stripping punctuation or partially parsing. Initial literal-number work may retain explicit unsupported diagnostics, with cases and follow-up status.
- Existing exclusions remain: Grid, unsupported formatting contexts, positioning, border painting, scripting, interactions, bindings and animation. Ratio tests must not silently expand these unrelated features. Border-box itself can be tested with padding even while painted CSS borders remain excluded.

## Existing wire and runtime seam

No new binary property is needed merely to encode a positive ratio. `LayoutComponentStyle.aspectRatio` is property **524**, schema double / Rust f32, default zero. Generated deserialization and clone copying already exist. [Schema](../../../crates/nuxie-schema/src/generated/schema.rs#L14992), [generated runtime](../../../crates/nuxie-runtime/src/mechanical_port/source/generated/layout/layout_component_style_base.rs#L227).

`LayoutComponentStyle::apply_item_style` maps values greater than zero to the optional ratio and other values to undefined. `YGStyle::set_aspect_ratio` assigns `taffy.aspect_ratio`. The change callback dirties layout. [Style application](../../../crates/nuxie-runtime/src/mechanical_port/source/layout/layout_component_style.rs#L182), [Taffy adapter](../../../crates/nuxie-runtime/src/mechanical_port/source/layout/layout_style_applier.rs#L509).

The local C++ checkout inspected is `/Users/levi/dev/oss/rive-runtime`, commit `9ed5b5168d95aab07e873db341fb65613d317cfc`. Its corresponding application uses the same positive-value/undefined rule, and its change callback marks the layout node dirty. [Pinned C++ implementation](https://github.com/rive-app/rive-runtime/blob/9ed5b5168d95aab07e873db341fb65613d317cfc/src/layout/layout_component_style.cpp#L76), [pinned generated key](https://github.com/rive-app/rive-runtime/blob/9ed5b5168d95aab07e873db341fb65613d317cfc/include/rive/generated/layout/layout_component_style_base.hpp#L97). No upstream checkout or remote was changed for this research.

Compiler insertion points are `Style` parsing/cascade and `Style::write`, called while emitting each `LayoutComponentStyle`. The current compiler does not write an aspect-ratio property. Preserve authored auto dimensions and flex basis instead of baking a height from the compilation viewport. [Writer](../src/style.rs#L1137), [emission](../src/lib.rs#L707).

Taffy first resolves size/min/max, applies ratio, then adds the content-box padding adjustment. Its generic ratio helper fills exactly one missing dimension; both missing or both known dimensions are unchanged. Flex stretch has a separate cross-size path and explicitly avoids ratio transfer for one maximum-size clamp. These are code observations, not proof of CSS conformance. [Flex items](../../../vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs#L561), [stretch](../../../vendor/taffy-0.12.1-rive-yoga-order/src/compute/flexbox.rs#L1698), [ratio helper](../../../vendor/taffy-0.12.1-rive-yoga-order/src/geometry.rs#L591).

Risk: a bare scalar ratio and global `box_sizing` cannot necessarily represent a combined `auto ratio` with content-box ratio calculations but border-box authored dimensions. Do not solve that by changing the element’s entire box-sizing mode. First probe Chrome, then use a narrowly scoped runtime policy if required. Any new policy needs validated manifest targets, host capability rejection, native/WASM parity, and independent clone/reset behavior; property availability alone does not prove old-host semantic compatibility.

## L14 boundary and image risks

The compiler currently rejects images if either computed dimension is `Auto`, with `unsupported-image-sizing`. It emits an Image child with fit value 7 and a LayoutParticipant beneath an outer layout. [Admission](../src/lib.rs#L566), [image graph](../src/lib.rs#L787). Runtime image measurement returns intrinsic width/height except for exact constraints; it does not itself derive a missing axis from a ratio. The layout component aggregates intrinsically sizeable non-layout children. [Image measurement](../../../crates/nuxie-runtime/src/mechanical_port/source/shapes/image.rs#L351), [layout measurement](../../../crates/nuxie-runtime/src/mechanical_port/source/layout_component.rs#L1785).

Proposed separation: L13 owns authored preferred ratios, including one auto dimension on an image when sizing is determined by an explicit authored ratio. L14 owns natural image dimension discovery, natural-ratio selection for `auto`, both-auto intrinsic image sizes, and their responsive constraints. Combined `auto ratio` on decoded images crosses that boundary and cannot be advertised as equivalent to a bare ratio. Keep an evidenced pending L14 interaction if it cannot yet be correct; do not mark full replaced-element support based only on fixed-size images. Image width/height HTML attributes, missing assets, decode failure, and non-square assets need explicit future contracts.

## Deterministic qualification matrix

Use actual public compiler output, compile once at width 390, retain the RIV hash, instantiate original and clone, and resize each through 240 → 390 → 768 → 240. Record all descendant geometry against Chrome at each viewport, real native Metal pixels, and reviewed browser/native/diff sheets. Keep the existing tolerances unchanged and record native-glyph and vector-glyph outcomes separately.

| Family | Required contrasting cases |
| --- | --- |
| Basic sizing | width fixed/percent with auto height; fixed height with auto width; both definite; both auto; square, wide and tall ratios |
| Box model | border/content box, no padding and asymmetric padding, padding larger than requested dimensions, zero dimensions |
| Constraints | every min/max axis, percentage bounds, min greater than max, determining-axis versus dependent-axis clamp |
| Flex | row/column/reverses, grow/shrink, auto/length/percentage basis, indefinite percentage basis, stretch/start, wrapping/reverse wrapping |
| Contents | empty, fixed child, nested ratio child, percent descendant, multiline text, nowrap/ellipsis, overflow visible/hidden, explicit min zero |
| Images | non-square intrinsic assets; explicit ratio matching/opposing natural ratio; auto and combined syntax; one/both auto dimensions; per-L14 admission status |
| Compositions | responsive media card, avatar beside wrapping text, nested hero/thumbnail, ratio card in wrapping action row |
| Contract | public parsing/cascade, finite wire values, old-host behavior if policy added, native/WASM byte and manifest equality, clone independence |

Capture minimized failures before runtime edits. Add low-level original/clone geometry tests from saved Chrome oracles, then public compiler tests and renderer fixtures. A green scalar-property smoke test cannot qualify the min/max/flex/image interaction matrix. Finally run the existing regression corpus and transfer visual evidence only where source and rendered image hashes match a previously reviewed result.

## Local Chrome discriminator (2026-09-09)

The strengthened probe in `output/playwright/html-to-riv/aspect-ratio-auto-probe-oracle-v2/oracle.json` confirms the distinction in Chrome153.0.8010.12. At390px with padding totals24px horizontal/18px vertical, border-box width120 and bare ratio2 produces120×60, while `auto 2 / 1` and `2 / 1 auto` produce120×66. Border-box height80 produces160×80 for bare ratio and148×80 for combined auto/ratio. Content-box controls agree across all three syntaxes. The same fixture captures240/390/768.

The first probe's24×12 padding accidentally shared ratio2, masking the distinction; its immutable `aspect-ratio-auto-probe-oracle` evidence is historical. Current fixtures use24×18 padding; initial64-case corpus is likewise recaptured under `aspect-ratio-initial-oracle-v2`. These are browser references only; no compiler support or runtime equivalence is claimed. Do not collapse the parsed auto flag into an undifferentiated scalar ratio.

L13 existing-runtime probe: `tests/aspect_ratio_runtime_investigation.rs` compiles a ratio-free input, injects existing property524, installs required content-box/flex policies, clones the artboard and resizes both instances240→390→768→240. Explicit exploratory run fails:40/64 scenes match Chrome throughout;24 fail, with320 coordinate mismatches in auto, max-width and flex-basis variants across all directions and both box modes. Evidence: `output/playwright/html-to-riv/aspect-ratio-runtime-investigation/receipt.json` and `run.log`. The test is explicitly ignored by default while the feature is under investigation; it must be invoked with `--ignored`. This is neither a public compiler test nor a supported feature. Runtime corrections are required before qualification.

L13 used-main correction: flex hypothetical cross sizing now applies an available preferred ratio to the final flexed main size, accounting for content-box padding. Existing Taffy106/106 tests pass. A non-ignored12-scene Chrome original/clone regression passes96 instance/viewports. Full exploratory corpus improves to52/64 scenes matching, with12 failing and256 coordinate mismatches: transferred max constraints and column auto intrinsic sizing remain; column auto additionally gains a width mismatch pending the intrinsic-main correction. Evidence: `output/playwright/html-to-riv/aspect-ratio-used-main/receipt.json`. No public aspect-ratio support or qualification claimed. The ongoing content-box regression uses its immutable pre-L13 v13 toolchain and does not validate this new engine patch.

L13 constraint-transfer correction now retains explicit preferred dimensions when transferring opposite-axis min/max limits through a ratio. Full runtime probe matches60/64 scenes; all8 max-width cases are fixed, and4 column-auto scenes remain (128 coordinate mismatches). Taffy106 tests pass. Receipt: `output/playwright/html-to-riv/aspect-ratio-constraints/receipt.json`. An intrinsic-inline-to-block correction is being tested; neither public CSS support nor qualification is claimed.

L13 runtime foundation now passes all64 scenes across512 original/clone instance-viewports. The column flex basis derives its block size from fit-content inline size and preferred ratio, respecting box sizing. Taffy106 tests pass. The entire probe is now a regular non-ignored regression and passes (no filtered cases); its prior failures remain preserved. Receipt: `output/playwright/html-to-riv/aspect-ratio-intrinsic-column/receipt.json`, regular run `regular-test.log`. This verifies existing-property runtime geometry only: public compiler syntax/transport, auto+ratio distinction, expanded stress cases, native/WASM parity and actual renderer pixels remain outstanding.

L13 parser foundation: `src/aspect_ratio.rs` retains the auto flag and normalized optional ratio;3 internal tests pass for both auto orders, comments, numbers, degenerates and invalid/unrepresentable values. Nonzero numeric underflow is rejected instead of silently becoming an auto/zero ratio. Math functions, units, percentages and negative ratios are excluded from this parser stage. Receipt: `output/playwright/html-to-riv/aspect-ratio-parser/receipt.json`. The parser is not connected to public CSS admission yet; combined auto/ratio runtime policy, transport and renderer gates remain outstanding.

## Ratio reference-box runtime integration

The runtime now stores a cloneable `css_ratio_content_box` occurrence policy separately from `css_content_box`. It selects Taffy's optional `aspect_ratio_box_sizing` while leaving authored sizing unchanged. Flexbox size/constraint resolution and final used-main ratio calculations use this reference box. The property is intended for the current flex layout profile; absolute positioning and other layout algorithms are not qualified by these tests.

The64-case bare-ratio foundation still passes512 instance/viewports. The12-case bare/auto-first/auto-last discriminator passes96 original/clone instance/viewports (36 distinct Chrome captures). Taffy106 tests pass after updating its explicit default-style test initializer for the new field. A separate clear-policy/clone-independence check is running. No public compiler transport or admission has been added yet.

L13 ratio reference-box runtime tests pass: bare64 scenes/512 instance-viewports, combined/control12 scenes/96 instance-viewports, and64 additional clear-original/retained-clone size comparisons. Wrong target rejection and Taffy106 pass. The independent occurrence policy preserves authored box-sizing and clone state. Receipt: `output/playwright/html-to-riv/aspect-ratio-reference-box/receipt.json`. Compiler cascade/transport, host admission and pixel qualification remain outstanding.

L13 public compiler integration: numeric ratios and optional auto flag now pass through cascade (including initial/unset/inherit/variables/important), wire aspectRatio and version14 `layout-css-aspect-ratio-v1` requirements with unique layout targets and explicit ratio reference-box policy. Public76-scene original/clone oracle passes608 instance-viewports;2 contract tests,15 host tests, native/WASM builds and type checking pass. Host rejects old/unknown versions, missing capability, empty/duplicate/invalid targets and malformed policy flags before drawing. Full public suite,2093-input parity, native pixel runs and visual review are pending. Frozen toolchain: `output/playwright/html-to-riv/aspect-ratio-v14-toolchain/`. Qualification is not claimed; image auto sizing, math-function values and additional interactions remain pending.

L13 expanded validation: full public suite245 tests across52 targets passes with0 ignored; native/WASM parity passes9 tests across2093 inputs. Auto/ratio discriminator native36/36 geometry and pixel comparisons pass; visual review remains pending. Receipt: `output/playwright/html-to-riv/aspect-ratio-public-integration/receipt.json`.

L13 visual progress: auto discriminator36/36 reviewed (18 direct and18 exact-image transfers). Initial corpus25/192 accounted for (24 direct plus1 exact-image transfer); remaining167 pending. Expanded96-scene corpus captures288 Chrome viewports for min-width/max-height/conflicting constraints/stretch/wrap/nested ratios across all four directions, both box modes and bare/auto forms. Native replay is running against frozen v14; no expanded pass is claimed yet.

L13 expanded native run completed240/288 passing;48 geometry/pixel failures are all conflicting min/max cases and remain preserved in `aspect-ratio-expanded-native/`. Current corrections normalize a maximum against its minimum before transfer, and derive an automatic dimension from the constrained preferred opposite dimension while retaining raw authored dimensions for flex basis. Expanded public oracle now includes172 scenes; its new align-self policy installation fixes a test-harness omission that falsely reported stretch failures. Corrected runtime/public tests are running, with evidence under `output/playwright/html-to-riv/aspect-ratio-conflict-correction/`. No new renderer pass or qualification is claimed yet.

L13 conflict correction now passes the full172-scene public Chrome oracle across1376 original/clone instance-viewports, plus the auto reference-box lifecycle test and Taffy106 tests. Receipt: `output/playwright/html-to-riv/aspect-ratio-conflict-correction/receipt.json`. The corrected native probe is rebuilding; the48 frozen-v14 renderer failures remain unwaived until a fresh corrected replay and visual review complete.

L13 corrected native replay completes with516/516 geometry and pixel comparisons passing: initial192, auto36, expanded288. Corrected auto36 are visually accounted for by exact source/full-image identity with their reviewed baseline. Initial review remains48/192 on the first snapshot, and expanded corrected review is pending. Original48 conflict failures remain preserved as historical reproducers. Expanded parity2189 inputs is running. Toolchain: `aspect-ratio-v14-corrected-toolchain`; replay directories end in `native-v2`.


### L13 text and composition probe

Added28 scenes in validation/aspect-ratio-text-cases.json: four flex directions,
both box modes, percentage width with wrapping, flex basis with auto ratio,
fixed height with ellipsis, and four responsive media cards. Chrome153 captures
84 viewports; public native replay compiles each scene once at390 then resizes
at240/390/768 with unchanged RIV hashes. Native geometry/pixels84/84 pass;
24 pairs are visually accounted for and60 remain. Vector geometry84/84 passes,
but12 media-card pixel comparisons fail; all12 failure pairs are reviewed and
unwaived (9 direct,3 exact-image transfers). Vector passing72 review remains.
Native/WASM parity passes2217 inputs across9 tests. The first fixture omitted
the required display:block for ellipsis; its rejected run is preserved and the
corrected corpus was recaptured. No runtime edit or tolerance change was needed.
Receipt: output/playwright/html-to-riv/aspect-ratio-text-native-v2/receipt.json.
L13 remains unqualified: finish reviews, text original/clone regression,
image sizing and the full corrected-runtime regression.


### L13 clone regression discovered

The new public text original/clone regression fails reproducibly with96 coordinate
mismatches in the four media cards, only on the clone after its first resize.
Original instances and the clone at its first240px layout pass. A reduced
one-row/two-label footer with no aspect ratio reproduces18 coordinate mismatches
in0.08s at subsequent390/768/240 sizes: View report shrinks from85.84px to12.86px
and wraps to192px high instead of24px. This is evidence of a broader intrinsic
text clone/resize issue; its root cause is not yet established. The regular,
non-ignored tests in tests/aspect_ratio_oracle.rs preserve the failure. Chrome
oracles, repeated logs and test snapshot are recorded in
output/playwright/html-to-riv/aspect-ratio-text-clone/receipt.json. Previous
fresh-import native84/84 pixel results do not establish clone lifecycle fidelity.
Next: finish minimizing the footer, distinguish measurement/policy/cache state,
fix the cause, then rerun the full text oracle and renderer regression.


### Text occurrence policy clone correction

Text clones now retain host-installed CSS wrapping, nowrap alignment, ellipsis,
underline and strikethrough policies. Derived shaping, measurement and drawing
state starts fresh; serialized RIV bytes and fresh-import admission are unchanged.
The prior single-label and footer failures pass without reinstalling policies on
the clone. Public aspect-ratio oracle now passes4 tests covering202 scenes and
1616 original/clone instance-viewports, including all28 text/composition scenes.
The root cause was the generated Text clone path copying only serialized base
properties; it now delegates to Text::clone_core. Historical failures and the
wrap-reinstallation control remain in aspect-ratio-text-clone/receipt.json.
Policy independence unit test passes, covering copied policies, fresh derived state, clone/source independence and default behavior. New frozen-probe pixel/full
regression is still required; old renderer receipts predate this correction.


### Post-clone regression evidence

The full module suite passes248 tests across53 targets with0 ignored. The new
frozen aspect-ratio-v14-clone-toolchain includes the Text clone policy correction.
Text native replay passes84/84 geometry and pixel checks; all84 pairs have exact
HTML/CSS and browser/native PNG identity with the now fully reviewed v2 baseline
(72 direct inspections,12 exact-image transfers). Current receipt:
aspect-ratio-text-native-v3/visual-inspection.json. Vector replay remains72/84
pixels and84/84 geometry; all84 sources/images are identical to its prior run,
with12 failure reviews transferred and72 passing reviews pending. No failure is
waived. Logs and hashes are in aspect-ratio-text-clone/receipt.json. Remaining
L13 work includes shape/vector visual reviews, authored-ratio image sizing and
full corrected-runtime regression; qualification is still withheld.


### Authored-ratio image admission

L13 now admits exactly one auto image dimension with a bare positive numeric
aspect-ratio, retaining the auto dimension and other length/percentage in the
RIV layout. Both-auto natural sizes and combined auto ratios with an automatic
dimension remain rejected pending L14. Existing explicit-both sizing is retained.
24 Chrome image fixtures/72 viewports cover four flex directions, both box modes,
asymmetric padding, percent width, fixed height and min/max constraints. Public
original/clone geometry passes192 instance-viewports; native geometry/pixels72/72
pass. Visual review20/72 is complete so far;52 remain. Native/WASM parity2241
inputs/9 tests passes. Original24 admission failures remain preserved.
Receipt: output/playwright/html-to-riv/aspect-ratio-image-native/receipt.json.
Full module suite is running; full native regression and remaining image review
are pending. L13 is still unqualified; no tolerances or runtime code changed for
this admission increment.


### Image visual review complete; full native regression running

All72 authored-ratio image comparisons are now directly visually reviewed across
four directions, both box modes, three sizing variants and three widths. Image
quadrants, padding, constrained sizes, reversed anchoring and viewport overflow
agree with Chrome; no thresholds changed. Full module suite passes250 tests
across53 targets with0 ignored. Receipt: aspect-ratio-image-native/receipt.json.
The complete existing native regression is running5983 tests against immutable
aspect-ratio-image-toolchain, including the ratio engine and Text clone fixes.
Output: output/playwright/html-to-riv/aspect-ratio-v14-full/. Follow session8434
and run-state.json; no full pass or qualification is claimed before completion
and review. Focused aspect-ratio corpora remain separate from the1979-scene
main corpus and are not implicitly counted in this5983-test run.


### Expanded shape review complete

The corrected `aspect-ratio-expanded-native-v2` snapshot now has all288 Chrome/
native comparisons visually accounted for:174 directly inspected and114
verified identical browser/native PNG pairs. The final review covers forward
and reversed columns, content-box min/max constraints, nested percentage
ratios, and row/reverse-row wrapping at240/390/768. Padding offsets, sibling
order, line breaks and overflowing wide ratio boxes agree with Chrome.
`validation/record-visual-review.py --audit` passes, rechecking image and review
sheet hashes; no visual tolerances changed.

Receipt: `output/playwright/html-to-riv/aspect-ratio-expanded-native-v2/visual-inspection.json`.
This qualifies the review of that frozen snapshot only. Replay with the current
image toolchain, initial-corpus review, image stress and remaining vector review
are still required. The5983-test full regression was confirmed live through
session8434 during this update; its log had reached test1681. It is not yet a
completed full-regression receipt.


### Initial column review and current snapshot replay

Initial-corpus review now covers96/192 pairs: all forward rows and columns,
in both box sizing modes, directly inspected at240/390/768. Automatic dimensions,
both definite dimensions, flex basis, min/max constraints and percentage sizing
match Chrome visually. The receipt passes the image/sheet integrity audit;
reversed rows and columns remain96 explicit unreviewed pairs.

Current-toolchain replays were launched under session19423, sequentially targeting
`aspect-ratio-initial-native-v3`, `aspect-ratio-auto-native-v3` and
`aspect-ratio-expanded-native-v3`, using `aspect-ratio-image-toolchain`.
The runner refuses existing output directories and validates frozen binary hashes.
Their sibling `.log` files preserve exit output. Session19423 was confirmed live;
no complete replay result or cross-snapshot review transfer is claimed yet.


Current snapshot initial192/192 and auto36/36 comparisons have now completed
with zero geometry or pixel failures. All228 pairs are exact HTML/CSS and
rehash-verified browser/native image matches to the original reviewed runs.
Source receipts were audited before transferring review: initial96 pairs carry
forward with96 still unreviewed; auto36/36 are complete. Each output contains
`visual-inspection.json` and the executed `transfer-review.py`. Expanded replay
remains running under session19423. Full regression remains separate.


### Current shape snapshot: all516 comparisons reviewed

Session19423 completed successfully. Initial192, auto36 and expanded288
comparisons all pass geometry and native pixels on `aspect-ratio-image-toolchain`.
The remaining initial reversed rows/columns were directly inspected at all three
widths; the initial receipt is now192 direct reviews and passes the integrity
audit. One min-height contact-sheet display anomaly was checked against both
original240px images, which agree.

All516 current pairs have exact HTML/CSS and rehashed full browser/native PNG
identity with audited reviewed source runs. Current receipts are in
`aspect-ratio-{initial,auto,expanded}-native-v3/visual-inspection.json`; transfer
scripts are retained alongside them. The prior partial initial transfer receipt
is preserved as `visual-inspection-96-reviewed.json`. No tolerance changes.

This completes these shape reviews, not L13 qualification: image stress,
remaining text/vector review and the separate full regression remain open.


### Authored-ratio image flex stress

`validation/aspect-ratio-image-stress-cases.json` adds56 scenes across four flex
directions, both box modes, grow/shrink, auto basis, fractional factors, stretch,
wrapping and both-definite-dimension controls. Chrome153 captured168 references.
The public compiler original/clone resize test passes448 instance-viewports;
the frozen image toolchain passes168/168 geometry and native pixel comparisons.
No runtime or compiler implementation change was necessary for these cases.

Receipts, oracle provenance and test logs are under
`output/playwright/html-to-riv/aspect-ratio-image-stress-native/`. Visual review
currently covers168/168 pairs (direct row grow/shrink/stretch plus verified image
identities);112 remain. Thin image-edge sampling differences are recorded under
unchanged tolerances. The2297-input native/WASM corpus is running in session89073
with log `/tmp/aspect-ratio-image-stress-parity.log`; no parity pass claimed yet.
Natural dimensions and combined auto ratio with an automatic image axis remain
L14 work; this stress corpus does not claim those semantics.


Image-stress parity completed: all nine tests pass, including2297 corpus inputs.
The log is preserved as `aspect-ratio-image-stress-native/parity-pass.log`.
Visual review now accounts for168/168 pairs with a passing integrity audit;
remaining pairs stay explicit in the receipt. Full regression is still separate.


### Image flex stress review complete

All168 Chrome/native pairs are now reviewed, with an integrity audit of every
image and reviewed sheet. The receipt distinguishes direct inspections from
verified exact-image transfers. Four directions, both box modes, all seven
sizing/flex variants and all three viewport widths are covered. Thin sampling
edge differences remain recorded under unchanged tolerances; no missing image
content or layout mismatch was found. Public448 instance-viewports and2297-input
native/WASM parity already pass. L13 remains unqualified pending its remaining
text/vector work, math decisions and full regression review.


### Numeric math reference established

Chrome accepts all16 prospective math inputs; the current frozen compiler rejects
all16 with preserved diagnostics. Seven invalid controls are rejected by Chrome.
The independent oracle has48 geometry/pixel captures, not yet reviewed. Negative
math results clamp per ratio component; NaN, infinity and underflow need explicit
numeric handling. See `validation/aspect-ratio-math-research.md`. Implementation
remains open and is not an external blocker.


### Full frozen regression complete

Session8434 completed with exit0:5983 tests passed, no skips, unexpected failures
or flaky tests. All5973 scene pairs have exact HTML/CSS and rehashed full browser/
native PNG identity with the reviewed `content-box-v13-full` baseline. Its review
receipt, comparison receipt and referenced source receipts were hash-checked.
The1979-case source snapshot, harness/reset and four frozen binaries are unchanged.

Evidence: `aspect-ratio-v14-full/completion-receipt.json`, `visual-inspection.json`,
`baseline-comparison.json`, `playwright-results.json` and `compare-baselines.py`.
This proves the frozen image toolchain's existing-corpus regression, including
ratio runtime fixes and Text clone policy correction. Focused L13 corpora remain
separate. The later substituted-math classifier correction and internal numeric
evaluator are not covered by this frozen compiler and need rebuilt qualification.
