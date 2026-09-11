# S09/S10 custom properties and variable substitution

Status: public integration in progress; not qualified.

Current semantic reference: [CSS Variables editor’s draft](https://drafts.csswg.org/css-variables-1/),
especially var replacement and Changes Since the 2022 Snapshot. The draft made
var short-circuiting. The older [published snapshot](https://www.w3.org/TR/css-variables-1/)
described dependency edges through unused fallbacks; initial resolver tests
followed that older rule and have been corrected after direct Chrome evidence.

Public implementation now includes case-sensitive/escaped custom names, inert
token values, empty values, importance/inline/source-order cascade, computed
inheritance, initial/inherit/unset, literal-name var, nested and empty fallback,
short-circuit substitution, actual-cycle invalidation, token boundaries and
missing-value unset behavior. Supported shorthand substitution happens before
font expansion and font-relative length resolution. Unsupported rendering
properties/functions still diagnose; revert/revert-layer are explicitly rejected.
Bounds are documented in SUPPORT.md.

Four resolver and six public compiler tests pass. The initial integration exposed
a raw parser bug: an unconsumed final function block was missing from the captured
value. Explicit recursive block consumption fixed it; failing public-test log
/tmp/html-custom-public.log is retained. Existing var-rejection tests were updated
to unsupported env cases now that var is accepted.

Initial full compiler tests and native/WASM builds/corpus parity passed. Six
permanent fixtures produced 15/18 Chrome/native passes at 240/390/768, with three
failures for unused fallback cycles. All six sheets were inspected: the five
passing cases agree in geometry/colors/cards; the failing case changes widths
and wrapping. This was not a raster tolerance problem.

Reproducer: `validation/custom-property-cycle-control.mjs` tests plain browser
CSS without harness scoping. Chrome 153.0.8010.12 returns 20% for unused fallback
cycles and invalid for used/direct cycles. Receipt:
`output/playwright/html-to-riv/custom-properties-initial/direct-chromium-cycles.json`.
Original failing pixels/gallery remain in custom-properties-initial and
`tools/html-to-riv/test-results-custom-properties-initial`.

The resolver now evaluates references lazily and marks actual active cycle
members invalid, including when an inner fallback could otherwise rescue the
cyclic property. No arbitrary 64-property dependency-chain cutoff remains;
there are at most 512 distinct properties per element. Repeat validation completed under custom-properties-short-circuit: full
compiler146/146, native/WASM builds and full accepted-corpus artifact parity pass.
All18/18 Chrome geometry/native pixels pass at240/390/768, resizing scenes
compiled at390. All six sheets were visually inspected: corrected cycle widths
and wrapping agree; other cases also agree with only minor edge rasterization
differences within unchanged thresholds.

Remaining before qualification:

- Complete nonempty computed-value invalidation beyond the conservative token
  classifier now implemented. Unknown keywords/functions and unclassified
  syntax still diagnose; valid CSS outside the rendering profile must not be
  silently reset.
- Audit shorthand invalidation, overwritten declarations, CSS-wide fallback
  values, expansion overflow, long-chain resource bounds and token adjacency
  with public output and browser comparisons.
- Dynamic variable-name substitution is deferred with direct Chrome153 evidence:
  nested-name syntax is rejected by CSS.supports and omitted from CSSOM, while
  literal-name substitution resolves. Receipt: custom-properties-names/direct-chromium-names.json.
- Expand the permanent corpus for these cases, run module/build/parity/pixels
  and inspect sheets before marking S09/S10 qualified.

Grid, editor integration, scripts, interactions, bindings, animation and
@property stay excluded. No pixel tolerances were changed.

Durable final logs are in `output/playwright/html-to-riv/custom-properties-short-circuit/`
(module.log, native-build.log, wasm-build.log, parity.log, pixels.log). Initial
failed pixels and raw parser failure are retained in custom-properties-initial.
These passing cases do not qualify unfinished invalid-value semantics.

Empty-substitution follow-up: a public regression initially failed on an empty
width. The compiler now normalizes substituted token text and applies unset
when the final result is empty. This differs from an empty component inside an
otherwise valid declaration, which remains valid. Public tests cover absent
empty fallback, present empty/whitespace/comment-only values, fallback not taken,
inherited color, padding shorthand overrides and later winning declarations.
Four permanent Chrome fixtures added. Full compiler147/147, native/WASM builds
and full accepted-corpus artifact parity pass. Combined custom-property pixels
30/30 pass at240/390/768, resizing scenes compiled at390. All four new sheets
were visually inspected; six previous sheets are byte-identical to their
reviewed predecessors (prior-sheet-identity.json). No tolerances changed.
Gallery and logs: output/playwright/html-to-riv/custom-properties-empty/.
Nonempty invalid-value handling remains unfinished; S09/S10 stay in progress.

Nonempty invalid-token follow-up: `src/substitution_validity.rs` classifies
unambiguously invalid size/list/color token shapes after substitution. Public
tests cover nonzero unitless and negative sizes, quoted lengths/colors, too many
components, invalid hex/numeric colors, padding and margin shorthand resets,
inherited color, and token-boundary preservation. Unsupported viewport units,
system colors, intrinsic sizing, calc and negative margins still diagnose.
The initial test incorrectly expected negative margins to be supported; it was
corrected to assert their existing profile rejection. No implementation was
changed to enable them. Ten public custom-property tests now pass.
Five new permanent fixtures added; full qualification remains pending.

Invalid-token validation completed: full compiler150/150, native/WASM builds and
full accepted-corpus artifact parity pass. Combined custom-property pixels45/45
pass at240/390/768, resizing scenes compiled at390. Five new sheets visually
inspected; all ten previous sheets are byte-identical to their reviewed versions
(prior-sheet-identity.json). Size resets, inherited/transparent colors, shorthand
resets and token boundaries match Chrome. Thresholds unchanged.
Gallery and durable logs: output/playwright/html-to-riv/custom-properties-invalid/.
This qualifies those cases, not the unfinished S09/S10 feature as a whole.

Resource audit follow-up: 512-property chains in both lexical name orders pass
public tests. Combining 512 properties with 32 inert nested groups per value
caused a native stack-overflow abort, despite both inputs fitting the advertised
limits. The reproducer is now a public regression test. Replaced recursive
property/value traversal with an explicit continuation stack, bounded incremental
concatenation and lazy fallback lookup. The reproducer and prior cycle/public
tests pass; no language limit was reduced.
Additional test checks 513 properties produce input-limit and exponential
expansion permits fallback recovery. Three permanent resource fixtures exercise
these paths through native/WASM publish parity and Chrome/native rendering.
Validation completed: full compiler153/153, native/WASM builds and full
accepted-corpus publish parity pass, including the resource fixtures. Combined
Chrome/native54/54 at240/390/768 pass with the same scene resized from390.
All three new resource sheets inspected; all15 earlier sheets byte-identical to
reviewed prior versions. No tolerance or resource-limit reductions.
Gallery/logs/identity receipt: output/playwright/html-to-riv/custom-properties-resources/.
The original stack-overflow log is retained there as stack-overflow-before.log.
Dynamic-name rejection is independently recorded under custom-properties-names.
Broader invalid keyword/function semantics and remaining audits are unfinished.

Keyword follow-up: added color and length-property keyword classification using
[CSS Color 4](https://www.w3.org/TR/css-color-4/#css-system-colors) (including
deprecated system colors) and [CSS Sizing 4](https://drafts.csswg.org/css-sizing-4/).
Unknown non-vendor color names, invalid length keywords and misplaced CSS-wide
keywords become unset after substitution. Known system/vendor colors and
intrinsic sizing retain profile diagnostics. A public test covers reset and
non-reset paths; four permanent visual fixtures added. Validation is running.

Keyword validation completed: full compiler154/154, native/WASM builds and full
accepted-corpus artifact parity pass. Combined Chrome/native66/66 pass at
240/390/768 with scenes compiled at390. Four new sheets visually inspected;
all18 earlier sheets byte-identical to reviewed versions. No thresholds changed.
Gallery, logs and identity receipt: output/playwright/html-to-riv/custom-properties-keywords/.
Color and length keywords covered here are complete; function grammar and other
property families remain unfinished, so S09/S10 stay in progress.

Color-function follow-up: classify malformed scalar rgb/rgba/hsl/hsla values
after substitution using the existing color parser, only when arguments contain
scalar tokens/separators. Two public tests cover wrong arity, separators, hue
units and mixed legacy channel types, plus valid clamping and preserved
diagnostics for relative colors, none, math and other color spaces. Three
permanent fixtures added; validation is running. Remaining function forms and
other property families are not qualified by these checks.

Scalar color-function validation completed: full compiler156/156, native/WASM
builds and full accepted-corpus artifact parity pass. Chrome/native75/75 pass at
240/390/768 by resizing scenes compiled at390. Three new sheets inspected; all22
earlier sheets byte-identical to reviewed versions. Channel clamping and invalid
function resets match Chrome. Tolerances unchanged. Gallery/logs/identity receipt:
output/playwright/html-to-riv/custom-properties-functions/. Other function forms
and property families remain pending; S09/S10 are still in progress.

Flex/overflow enum follow-up: classify invalid substituted keywords, token types
and arities for flex-direction, flex-wrap, overflow and text-overflow. Preserve
known valid but unsupported values rather than resetting them. Two public tests
pass, covering direction/wrapping defaults, overflow visibility, clipping and
explicit diagnostics. Four permanent fixtures added, including a real Inter
text case where an invalid substituted text-overflow must replace ellipsis with
clipping. Full validation is running.

Flex/overflow validation completed: full compiler158/158, native/WASM builds and
full accepted-corpus artifact parity pass. Chrome/native87/87 at240/390/768 pass,
resizing scenes compiled at390. Four new sheets inspected, including Inter text
clipping with no ellipsis; all25 earlier sheets byte-identical to reviewed
versions. Direction/wrapping defaults and visible overflow also match Chrome.
Tolerances unchanged. Gallery/logs/identity receipt:
output/playwright/html-to-riv/custom-properties-enums/. Other property families
and outstanding custom-property audits remain; S09/S10 stay in progress.


Flex-factor invalidation added for substituted negative, dimension, percentage,
identifier, string and multiple-number values. Public tests verify invalid grow
resets to0 and shrink to1, without restoring earlier declarations; valid2.5
factors preserve output. An initial test exposed the existing equal-grow/shrink
format restriction. Fixtures now set companion factors explicitly, and a public
assertion preserves the unsupported-flex-factors diagnostic when reset factors
are unequal. Full compiler159/159 and native/WASM builds pass; artifact and pixel
validation are underway. Property-family coverage and remaining gaps are recorded
in validation/custom-property-coverage.md.

Flex-factor validation completed: compiler159/159, native/WASM builds and full
accepted-corpus parity pass. Combined Chrome/native93/93 pass at240/390/768.
Both new sheets visually inspected: expected growth and shrink distribution
matches at all widths; narrow edge raster differences stay within unchanged
limits. All29 prior sheets are byte-identical to reviewed versions. Logs, initial
format-limit failure and SHA256 receipt accompany the gallery under
output/playwright/html-to-riv/custom-properties-factors/. S09/S10 remain in
progress; next work is shorthand/cascade and remaining property grammar coverage
listed in custom-property-coverage.md.


Flex shorthand invalidation: a public red test exposed malformed substituted
flex values producing a parse diagnostic instead of initial components. Added
flat-token grammar classification using the ordered factor group and optional
basis from https://drafts.csswg.org/css-flexbox-1/#flex-property. Negative factors,
multiple bases, interleaved basis/factors, excessive components and invalid
keywords/strings now reset all three components; fallback is not used when the
custom property exists but the resulting property value is invalid. Public
coverage includes later longhands, important ordering and retained diagnostics
for valid excluded fractional weights below1, intrinsic bases and viewport units.
Compiler160/160 and native/WASM builds pass; parity/pixels are running. Corrected
SUPPORT wording to distinguish nonnegative CSS factor grammar from the narrower
rendering factor range0 or[1,10000].

Flex shorthand validation completed: compiler160/160, native/WASM artifact parity,
and Chrome/native99/99 pass. Two new sheets inspected at all three widths;
31 prior sheets byte-identical. Evidence: custom-properties-flex-preserved/ under
output/playwright/html-to-riv. The initial96/99 run exposed CSSOM serialization
losing pending shorthand components, rather than a compiler layout mismatch.
The browser harness now scopes selectors by mutating selectorText and adopts the
same stylesheet without serializing its declarations. Direct original-versus-
serialized-versus-preserved Chrome control and initial failures are retained in
custom-properties-flex/. No tolerances changed. Full older corpus revalidation
with the revised harness remains pending, alongside other property families.

Full revised-harness revalidation is complete:1,341/1,342 pass; all1,332
scene comparisons retain identical sources and browser/native images against
prior reviewed versions. Sole failure: unchanged A09 em-layout-cascade240
fractional raster edges, geometry exact. See stylesheet-preservation-review.md
and output/playwright/html-to-riv/stylesheet-preserved-full/. Next independent
work remains other custom-property grammar/cascade families.


Alignment invalidation added: invalid align-items/justify-content substitutions
reset to stretch/flex-start; invalid text-align inherits the parent alignment.
Public red test retained. Classification distinguishes safe/unsafe position
pairs and first/last baseline from malformed combinations, preserving valid
excluded CSS rather than silently resetting it. Sources:
https://drafts.csswg.org/css-align/ and https://drafts.csswg.org/css-text-3/.
Anchor/dialog/vendor/string/function forms outside the rendering profile keep
diagnostics. Three fixtures cover stretch, main-axis start and real Inter
right-aligned wrapping. Compiler162/162 and native/WASM builds pass; parity and
Chrome/native108-case qualification are running.

Alignment validation completed: compiler162/162, native/WASM builds and full
accepted-corpus parity pass. Chrome/native108/108 pass. Three new sheets inspected:
stretched auto-width children, start distribution and right-aligned Inter text
with identical wrapping at each width. All99 earlier scene/width comparisons
retain identical source and browser/native PNGs. Tolerances unchanged. Logs,
public red test, baseline hashes and gallery: custom-properties-alignment/ under
output/playwright/html-to-riv. Remaining grammar families are still in progress.


Radius invalidation added with public red test: substituted negative values,
nonzero unitless numbers, invalid keywords/strings, excessive lists, comma lists
and malformed slash groups reset to0. Valid radius grammar is one to four
nonnegative length/percentage values, optionally slash-separated from another
one-to-four-value group (https://drafts.csswg.org/css-backgrounds/#border-radius).
The classifier preserves valid excluded percentages, per-corner lists, elliptical
radii and unimplemented functions/units as diagnostics. Two fixtures contrast
square reset corners with retained valid rounded corners across widths.
Compiler163/163 passes; publish and visual gates are in progress.

Radius validation completed: compiler163/163, native/WASM builds and accepted
corpus parity pass. Chrome/native114/114 pass; both new sheets visually inspected
at all widths, with square reset corners and retained rounded controls matching
Chrome. All108 previous comparisons have identical source and browser/native
images. No tolerance change. Logs, public red test and image identity receipt:
output/playwright/html-to-riv/custom-properties-radius/. S09/S10 remain in progress.


Font longhand invalidation added after a real-text public red test. Invalid
font-size, font-weight and line-height substitutions inherit rather than restore
earlier declarations. Classification separates CSS ranges (weight1..1000,
nonnegative size/line-height) from the narrower rendering profile. Zero sizes,
weights1/1000/fractional, relative keywords, percentages and math remain explicit
profile diagnostics where unsupported. Sources: https://drafts.csswg.org/css-fonts/
and https://drafts.csswg.org/css-inline/. Public tests compile embedded Inter
text so inheritance impacts actual output. Three visual fixtures cover size,
weight and inherited unitless line-height on a child with a different font size.
Compiler165/165 passes; remaining gates are running.

Font longhand validation completed: compiler165/165, native/WASM builds and
accepted-corpus parity pass. Chrome/native123/123 pass. Three new sheets inspected
at all widths: inherited size/regular weight and unitless line-height preserve
Chrome wrapping and line spacing. All114 earlier comparisons retain identical
source and browser/native images. Evidence: custom-properties-font/ under
output/playwright/html-to-riv. Family/shorthand and remaining grammar audits
continue; no tolerance changes or broader completion claim.


Font-family list invalidation added after a real-text public red test. Empty
comma entries, mixed quoted/unquoted entries, wrong scalar types, punctuation
and reserved default/CSS-wide identifiers in lists inherit. Quoted reserved
words and unknown valid family names retain missing-font/profile diagnostics;
valid fallback lists remain excluded. Source:
https://drafts.csswg.org/css-fonts/#family-name-syntax. Generic/system keyword
combinations and font shorthand grammar still need audit. Compiler166/166 passes;
two Inter fixtures added for invalid lists and quoted-name handling.

Family-list validation completed: compiler166/166, native/WASM builds and
accepted-corpus parity pass. Chrome/native129/129 pass. Both new sheets inspected
at all widths: inherited Inter and quoted Inter retain matching glyphs and
wrapping. All123 prior comparisons source- and image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-family/. Tolerances unchanged;
generic/system keyword combinations and shorthand grammar remain pending.


Family keyword audit found an over-invalidation: Chrome accepts Inter inherit,
Other serif and default Inter as family names; these must retain profile
exclusions, not silently inherit. Leading generic plus another identifier is
invalid (serif Other), as are standalone reserved words in list entries. Fixed
classification and added public regression plus one visual fixture. Direct
control records CSS.supports, specified/computed custom property and computed
family at custom-properties-family-keywords/direct-chromium.json. A further
unresolved case is leading CSS-wide words: inherit Inter passes direct syntax
but computes to parent through var(); keep this in the cross-cutting audit.
Compiler167/167 passed before removal of that misleading excluded-value test
case; no production change followed. Native/WASM builds pass.

Family keyword correction validation: native/WASM parity and Chrome/native132/132
pass. New sheet inspected at all widths;129 prior comparisons are source- and
image-identical. Compiler167/167 and build logs retained along with the public
red test and direct Chrome control. Leading CSS-wide-word substitution remains
an explicit next audit, not qualified by CSS.supports. Evidence:
output/playwright/html-to-riv/custom-properties-family-keywords/.


Leading CSS-wide family-name investigation resolved: direct declarations accept
multi-word names, but after variable or fallback substitution Chrome invalidates
all five leading CSS-wide words with trailing tokens, inheriting the parent.
Direct control records all three paths, including case/comments, plus default
and trailing-keyword controls in custom-properties-wide-prefix/direct-chromium.json.
Classifier now follows that computed behavior for substituted font-family only;
sole CSS-wide values still take their existing path. Public real-text red test
retained; compiler168/168 passes. One visual fixture compares invalid initial
and revert prefixes against inherited Inter. Remaining gates are running.

Leading-family-word validation completed: compiler168/168, native/WASM builds,
accepted-corpus parity and Chrome/native135/135 pass. New Inter sheet inspected
at all widths;132 prior comparisons source- and image-identical. Direct control,
red test and all logs: output/playwright/html-to-riv/custom-properties-wide-prefix/.
This closes the recorded leading-word gap for font-family; shorthand and other
property audits remain active. No tolerances changed.


Font shorthand cascade audit: new real-text public checks pass for missing and
empty substitutions, later font-size/line-height overrides, important shorthand
versus lower-specificity important longhand, later important longhand, and em
line-height using the final font size. No implementation change was needed for
these cases. Compiler169/169 and native/WASM builds pass. Two permanent fixtures
exercise these interacting behaviors at responsive widths. Nonempty malformed
font shorthand grammar remains pending; this audit does not imply it is handled.

Font shorthand cascade validation completed: compiler169/169, native/WASM builds
and accepted-corpus parity pass. Chrome/native141/141 pass. Both new sheets
inspected: font size, line spacing and wrapping match at all three widths;
135 earlier comparisons are source- and image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-font-cascade/. No implementation
change was needed for these scenarios; nonempty shorthand grammar remains the
next font-specific audit. Tolerances unchanged.


Font shorthand grammar stage: missing size/family, malformed line-height,
negative sizes, duplicate weights/excess normal prefixes, and malformed family
lists now invalidate supported-prefix substitutions and inherit all font
components. A later font-size override still applies. Public red test retained;
compiler170/170 passes. Valid CSS outside the rendering profile retains its
diagnostic, including italic/variant/width/system/keyword prefixes, zero size,
fractional weights, percentages and fallback lists. Excluded prefix grammar is
not yet classified exhaustively. Source: https://drafts.csswg.org/css-fonts/#font-prop.
Two permanent real-text fixtures added; native/WASM and visual gates pending.

Font grammar stage validated: compiler170/170, native/WASM builds and accepted
corpus parity pass. Chrome/native147/147 pass. Both new sheets inspected at all
widths: inherited font components and later size override match Chrome wrapping
and line spacing.141 prior comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-font-grammar/. Excluded-prefix
malformed grammar and other property families remain under audit; no tolerance
changes or complete-custom-properties claim.

Font prefix stage completed: unknown prefixes and combined system-font keywords
invalidate after substitution, while recognized unsupported prefixes retain
diagnostics. Public red test reproduced the gap; direct Chrome syntax control
retained. Compiler171/171, native/WASM builds and accepted-corpus parity pass.
Chrome/native150/150 pass. New text sheet inspected at all widths;147 earlier
comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-font-prefix/. Recognized excluded
prefix combinations and other property families remain under audit.


Text-transform invalidation added after a real-text public red test. Unknown
keywords/types, conflicting or duplicate casing transforms, repeated width/kana
modifiers and compound none/math-auto values inherit. Valid combinations follow
https://drafts.csswg.org/css-text-4/#text-transform-property but width/kana/math
rendering remains excluded and diagnostic. Two real-text fixtures cover parent
uppercase/capitalization inheritance and a valid lowercase variable override.
Compiler172/172 passes; remaining gates running.

Text-transform validation completed: compiler172/172, native/WASM builds and
accepted-corpus parity pass. Chrome/native156/156 pass. Both new sheets inspected
at all widths: uppercase, capitalization and lowercase control match glyphs and
wrapping.150 earlier comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-transform/. Tolerances unchanged;
white-space and other remaining grammar families continue under audit.


White-space invalidation added after real-text public red test. Keyword groups
follow https://drafts.csswg.org/css-text-4/#white-space-property: legacy modes
stand alone; collapse and wrap groups are mutually exclusive within each group;
trim none excludes discard modifiers, which cannot repeat. Invalid substitutions
inherit supported parent behavior. Valid compound modes and trimming remain
profile diagnostics; no new rendering mode is enabled. Compiler173/173 passes;
two preserved/collapsed text fixtures added for responsive visual validation.

White-space validation completed: compiler173/173, native/WASM builds and
accepted-corpus parity pass. Chrome/native162/162 pass. Both new sheets inspected
at all widths: preserved/collapsed spaces, newline handling and wrapping match.
156 prior comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-whitespace/. Other grammar and
cascade audits remain active; tolerances unchanged.


Decoration enum invalidation added after a public red test. Invalid substituted
text-decoration-style resets to solid; invalid skip-ink inherits. Public checks
compare both Rive bytes and runtime requirements. Valid non-solid styles remain
explicit exclusions; auto/none/all skip-ink retain existing support. Grammar:
https://drafts.csswg.org/css-text-decor-4/. Two real-text underline fixtures added
for solid reset and inherited none versus valid auto skip-ink. Compiler174/174
passes; remaining gates running.

Decoration enum validation completed: compiler174/174, native/WASM builds and
accepted-corpus parity pass. Chrome/native168/168 pass. Both new sheets inspected
at all widths: solid underlines and none/auto skip-ink behavior match Chrome;
162 earlier comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-decoration-enums/. Remaining
decoration grammar and other audits continue; tolerances unchanged.


Decoration-line invalidation added after public red test. Unknown/types,
duplicate line keywords and compound none/spelling-error/grammar-error invalidate
to none. Local reset preserves propagated ancestor decoration; tests compare
runtime records as well as Rive bytes. Valid overline/blink/error annotations
remain explicit profile exclusions (CSS Text Decoration3/4 grammar). Two new
fixtures cover local removal and ancestor underline preservation. Compiler175/175
passes; publish and visual qualification running.

Decoration-line validation completed: compiler175/175, native/WASM builds and
accepted-corpus parity pass. Chrome/native174/174 pass. Both new sheets inspected
at all widths: local removal preserves ancestor underlines; valid combined
underline/strike control also matches.168 earlier comparisons source/image-
identical. Evidence: output/playwright/html-to-riv/custom-properties-decoration-lines/.
Metrics, position, shorthand and remaining audits continue; tolerances unchanged.

Decoration metric invalidation implemented after a public failing test:
unknown scalar keywords, strings, nonzero unitless numbers and multiple values
reset thickness to auto and inherit underline offset. Offset from-font is invalid;
thickness from-font remains supported. Negative thickness is valid CSS but stays
an explicit renderer-profile diagnostic; negative offsets remain supported.
Control: validation/custom-property-decoration-metrics-control.mjs checks Chrome
syntax plus computed reset/inheritance. Grammar reference:
https://www.w3.org/TR/css-text-decor-4/#text-decoration-thickness-property
and #underline-offset. Public tests include runtime requirements, not only Rive
bytes. Compiler suite passes; native/WASM parity and visual checks are running.

Decoration metrics validation completed: compiler176/176; native/WASM builds and
accepted-corpus parity pass. Chrome/native180/180 pass. Both new sheets inspected
at all three widths; auto thickness reset and inherited/negative offset match.
All174 earlier comparisons source/image-identical. Evidence and direct Chrome
control: output/playwright/html-to-riv/custom-properties-decoration-metrics/.
Position, shorthand and remaining custom-property audits continue. No visual
tolerances changed; negative thickness remains an explicit profile exclusion.

Underline-position invalidation added after a public failing test. Scalar types,
unknown keywords, auto combined with other values and duplicate/conflicting
position groups now inherit the supported parent's auto value. Valid under,
from-font, left/right and combinations remain explicit profile exclusions.
Chrome syntax/computed control is executable at
validation/custom-property-underline-position-control.mjs. Grammar checked against
https://drafts.csswg.org/css-text-decor-4/#text-underline-position-property.
Public tests compare Rive bytes and runtime requirements; compiler suite passes.
Publish and visual qualification pending.

Underline-position validation completed: compiler177/177, native/WASM builds and
accepted-corpus parity pass. Chrome/native183/183 pass. New sheet inspected at all
widths; all180 earlier comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-decoration-position/.
Only auto position is representable, so no claim of non-auto inheritance support.
Shorthand grammar and remaining custom-property audits continue.

Decoration shorthand audit exposed a preflight bug: text-decoration with var()
was rejected as an unknown property before substitution. Validation now expands
its initial components to check the property. Flat substituted grammar detects
unknown scalar tokens, duplicate style/color/thickness and invalid line groups;
malformed values reset all four shorthand components without resetting offset.
Valid excluded line/style/metric forms remain diagnostics; function-containing
malformed grammar is still outside this classifier. Public tests cover valid
substitutions (including RGB), reset versus later longhands, and runtime records.
Two visual cases add valid-var control, reset, later longhands and important reset.
Grammar source: https://drafts.csswg.org/css-text-decor-4/#text-decoration-property
Chrome control: validation/custom-property-decoration-shorthand-control.mjs.
Compiler suite passes; publish/visual qualification pending.

Decoration shorthand validation completed: compiler178/178, native/WASM builds
and accepted-corpus parity pass. Chrome/native189/189 pass; both new sheets
inspected at all widths. All183 earlier comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-decoration-shorthand/.
Original public/preflight failures retained. Function-containing malformed
shorthands and remaining cross-cutting custom-property audits continue.

Decoration RGB/HSL shorthand invalidation added after a public red test. Scalar
RGB/RGBA/HSL/HSLA functions are validated independently, then represented by a
color-slot placeholder only for grammar checking; actual emitted color is never
replaced. Malformed functions and duplicate color/style/metric/line components
reset the entire shorthand. Relative/none/math/wide-gamut and other unclassified
functions retain diagnostics. A mistaken invalid HSL test was corrected using
Chrome evidence: modern hsl(20 30 40) is valid and now a positive public control;
hsl(20 30) is the missing-channel negative case. Original expectation failure
retained. Chrome control: custom-property-decoration-functions-control.mjs.
Compiler suite passes; publish/visual qualification pending.

Decoration color-function validation completed: compiler179/179, native/WASM
builds and accepted-corpus parity pass. Chrome/native195/195 pass. Both new sheets
inspected at all widths; all189 previous comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-decoration-functions/.
Original public failure and corrected-HSL-expectation failure retained. Remaining
unclassified functions, reset combinations and cross-cutting audits continue.

Decoration reset matrix added without production changes. Public tests compare
Rive bytes and runtime requirements for missing variables, present-empty values,
empty fallbacks and initial/inherit/unset fallbacks across all eight decoration
properties. Parent origins, colors, thickness, offset and skip-ink differ from
child values so accidental reset/restore is observable. Separate tests distinguish
custom-property inherit/unset/initial from ordinary-property fallback keywords.
Three new fixtures cover ancestor retention, initial versus inherit fallbacks,
and custom-value inheritance versus invalidation/recovery. Compiler suite passes;
publish/visual qualification pending.

Decoration reset-matrix validation completed: compiler181/181, native/WASM builds
and accepted-corpus parity pass. Chrome/native204/204 pass; all three new sheets
inspected at every width. All195 prior comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-decoration-wide/.
No production changes. Runtime equality covers visually overlapping origins;
remaining property grammar and cross-cutting resource/cascade audits continue.

Simple background substituted-value grammar added after public red test. Unknown
scalar identifiers/strings, malformed hex, duplicate colors or none images,
empty simple layers and colors before a comma now reset to transparent. Known
position/size/repeat/attachment/box/image syntax remains a profile diagnostic;
this is deliberately not full layered-background grammar. Present invalid values
do not use fallback or restore previous green. Public tests retain valid excluded
none+color and multiple-layer forms. Chrome control:
validation/custom-property-background-control.mjs. Two fixtures cover reset and
later background-color restoration. Compiler suite passes; publish/pixels pending.

Simple background validation completed: compiler182/182, native/WASM builds and
accepted-corpus parity pass. Chrome/native210/210 pass. Both new sheets inspected
at all widths; all204 earlier comparisons source/image-identical. Evidence:
output/playwright/html-to-riv/custom-properties-background/.
Complex background functions/position/layers and other audits remain open.

Background RGB/HSL combinations now share the decoration shorthand color-function
validator. Public red test preserved. Duplicate colors, malformed scalar RGB/HSL
inside none+color, and colors before a layer comma invalidate to transparent.
Valid pure functions retain exact output; valid none+color/layers/positions and
unimplemented functions retain diagnostics. Two visual fixtures cover duplicate
color reset, invalid layer position, missing HSL channel and valid HSL/later-color
controls. Chrome control: custom-property-background-functions-control.mjs.
Compiler suite passes; publish/visual qualification pending.

Background color-function validation completed: compiler183/183, native/WASM
builds and accepted-corpus parity pass. Chrome/native216/216 pass; both new sheets
inspected at all widths. All210 previous comparisons source/image-identical.
Evidence: output/playwright/html-to-riv/custom-properties-background-functions/.
Complex background grammar and cross-cutting audits continue; no tolerances changed.

Two realistic light/dark habit-plan cards added. Compositions combine inherited
custom aliases, local shadowing, nested decoration values, font shorthand, HSL,
empty-value fallback suppression, rounded panels and responsive flex actions.
Public test compares each theme to independently authored explicit-value CSS,
including runtime requirements. The initial 24px/1.2 heading hit the documented
natural-line-height bound; failure retained and the supported fixture uses1.4.
No compiler changes. Compiler suite passes; native/WASM and resize pixels pending.

Composition validation: compiler184/184, native/WASM builds and accepted-corpus
parity pass. Chrome/native220/222; two new240px light/dark note underline failures
retained. All216 prior comparisons source/image-identical. Both new sheets inspected
at all widths. Bounds delta<=0.03125px; noteY Chrome184.375/native184.399994.
Light underline Chrome row199 solid; native coverage split rows199/200. This
points to local versus global pixel snapping, not variable resolution. Next inspect
runtime decoration drawing with fractional translation. Evidence and logs:
output/playwright/html-to-riv/custom-properties-compositions/.
Do not change source/tolerances to hide this failure. Goal remains active.

Fractional underline runtime diagnosis: original two240px failures reproduced;
minimal one-node fractional origin fails while integer origin passes (1/6 fails,
2.9s). Hypotheses: scene-translation snapping, skip-ink clipping, baseline rounding.
Drawing path lacked underline paint-space snapping; strikethrough already had it.
Runtime now snaps vertical underline bounds after translation for unit-scale,
axis-aligned CSS placement, before host transform/DPR. Original bounds retained
for other transforms; full and geometric fallback paths rebuilt together.
Original plus minimal/control12/12 pass; fixed sheets inspected. Compiler184/184,
runtime decoration8/8 with pinned font, WASM build pass. Full native regression,
vector12-case controls and accepted-corpus parity are running. No tolerances changed.
Evidence: output/playwright/html-to-riv/underline-fractional-fixed/.

Fractional underline validation complete for native axis-aligned CSS placement:
compiler184/184, runtime8/8, native/WASM parity pass. Focused12/12 native, precise
row6/6, DPR27/27 pass. Full native1,470/1,471 passes; existing A09 only, unchanged.
All1,461 scene pairs match reviewed old-full/custom/fixed baselines. Custom222/222
now pass. Vector3/12 focused controls remain unqualified (sheets inspected).
DPR54/54 images/request sources identical to prior callback baseline. No debug
instrumentation introduced, no tolerance changes. Receipt:
validation/underline-fractional-translation-review.md. Goal remains active.

A09 investigation moved to runtime paint bounds: fractional single rectangle
reproduces, integer control passes. Opt-in LayoutComponent CSS paint/clip bound
snapping plus diagnostic probe flag yields9/9 original/minimal/control and117/117
nearby passes. All changed sheets inspected (94 prior pairs identical,23 changed).
Compiler184/184 passes. Normal publish behavior is NOT yet changed; next add strict
portable runtime capability/target requirements and remove diagnostic-only wiring.
Detailed handoff/evidence: validation/em-edge-paint-review.md. Goal active.

Layout paint contract integrated as v5 with layout targets/capability; diagnostic
enable flag removed. Rust188/188, JS9/9 (all corpus parity), TypeScript pass.
Native117/117 normal loader images match reviewed experimental run. DPR27/27 pass.
Full native run is live; resume its exact handle before any rebuild. Remaining:
full image comparison, clone/raw-Rive validation, fractional layout DPR/vector.
Evidence: validation/em-edge-paint-review.md; output/playwright/html-to-riv/layout-contract/.

## Dedicated layout paint DPR qualification

`validation/layout-pixel-bounds-dpr-control.mjs` passes 27/27 comparisons against
Chromium 153.0.8010.12: fractional-origin rectangles, fractional rounded backgrounds,
and rounded overflow clipping, each at DPR 1/2/3 and widths 240/390/768. Each scene
is compiled once at 390px and resized by the runtime. Native/WASM Rive bytes,
source maps and runtime requirements match. Geometry tolerance remains 0.1 CSS px;
shape pixel limits are unchanged and antialiasing exclusion is disabled.

All nine `review-{fractional,rounded,clipped}-{1,2,3}.png` sheets were visually
inspected. Straight rectangle edges align; rounded and clipped shapes retain
small corner antialiasing differences within the existing limits, without visible
child leakage or shifted straight edges. Artifacts and per-frame metrics:
`output/playwright/html-to-riv/layout-pixel-bounds-dpr/`.

These checks qualify the three translation-only paint cases at integer DPRs;
they do not establish affine transform, fractional DPR, vector text, or complete
regression qualification. Full native 1477-check run remains active; resume session
65444 and `/tmp/html-layout-contract-full.log` before any build or restart. Clone
policy regression coverage remains pending.

Clone-policy validation now passes: `cargo test -p nuxie-runtime --lib
css_pixel_bounds_survive_clone_and_can_be_disabled` ran 1/1 test successfully.
The test checks raw world paint bounds, opt-in snapped bounds, preserved behavior
on a prepared clone, reversible disabling on the clone, unchanged source paint,
and unchanged logical layout. Log: output/playwright/html-to-riv/layout-contract/clone.log.
Full visual session 65444 is still active; full regression and changed-image review
remain pending.

Full layout-contract native regression completed: 1477/1477 checks pass
(1467 scene comparisons plus 10 pixel controls), including the original A09 edge
failure. Review comparison transfers 1226 pairs from explicit reviewed baselines;
241 passing comparisons across 96 families still need visual inspection.
Artifacts: output/playwright/html-to-riv/layout-contract-full/, including pixels.log
and baseline-comparison.json. Receipt: validation/em-edge-paint-review.md.
Full process 65444 is terminal; no restart required.

## Full native visual qualification complete

Final batch: all 35 remaining custom-property sheets / 101 changed comparisons
inspected. Colors, reset/empty regions, inheritance/fallback results, flex placement,
and themed-card geometry agree with Chrome; small rounded-corner antialiasing
differences remain inside unchanged limits. The inspection queue is empty:
241/241 changed comparisons inspected, plus 1226 image/source-identical pairs
whose review transferred from explicit reviewed baselines. All 1467 scene pairs
now have review evidence. Full numeric result remains 1477/1477 checks.

This closes native qualification of the version-5 layout paint contract and
resolves the original A09 fractional shape-edge failure. Rust188, JS9 (corpus
native/WASM parity and host rejection), TypeScript, native/WASM builds, clone1,
layout DPR27 and separate underline DPR27 already pass. Ordinary Rive policy
remains opt-in; no logical dimension rounding or tolerance widening.

A09 remains partial for vector text: earlier em-nested-typography and
em-font-shorthand-final-size 240px failures require fresh accounting. Focused
vector run started in session60933, `/tmp/html-layout-em-vector.log`, output
`layout-contract-em-vector/`. Resume that handle before restarting. Affine and
fractional-DPR qualification is not established. S09/S10 broader substituted-value
grammar audit remains independent work; the overall backlog is not complete.

## Focused vector result

The vector rerun terminated: 31/33 comparisons pass. It covers five em families (em-flex-sizing,
em-font-shorthand-final-size, em-global-values, em-layout-cascade,
em-nested-typography), four rem families, and two background controls.
All 11 browser/vector/diff sheets were visually inspected at all three widths.
Shape edges, spacing and responsive geometry align; text shows the existing
vector coverage differences. Original em-layout-cascade240 now passes.

The two failures are em-nested-typography240 (mean channel error1.2762630208)
and em-font-shorthand-final-size240 (1.0319140625), both above the unchanged1
limit and matching the historical failure metrics. No claim of image identity
with an old vector run is made. All 12 rem and six background-control comparisons
pass. Artifacts: output/playwright/html-to-riv/layout-contract-em-vector/,
including pixels.log, review.json and inspection/*.png. Sessions60933 and63329
are terminal; no work is still running.

A09 native qualification is complete; vector text fidelity remains partial.
Next independent compiler work resumes S09/S10 substituted-value grammar audit.

S09/S10 non-length dimension increment implemented: angle/time/frequency/
resolution/flex units invalidate substituted length slots; valid excluded lengths
retain diagnostics. Chrome182 references and two public Rust regressions pass.
Native/WASM and visual qualification pending. Receipt:
validation/non-length-dimensions-review.md.

## Qualification update

Full module190/190 and JavaScript9/9 pass; the accepted corpus (including new
fixtures) has native/WASM Rive, source-map and requirements parity. Native/WASM
builds and TypeScript pass. Both new fixtures compile once at390 and pass all
six Chrome/native geometry/pixel comparisons at240/390/768. Both comparison
sheets inspected: layout reset widths/padding/margins align; inherited text size,
spacing, wrapping and underline placement agree, with minor existing glyph
raster differences within unchanged limits.

Logs, gallery and sheets: output/playwright/html-to-riv/non-length-dimensions/.
Broader custom-property regression is running in session88854, log
`/tmp/html-var-dimensions-custom.log`, output `non-length-dimensions-custom/`.
Resume before restarting. Compare completed review.json against
layout-contract-full and non-length-dimensions; new source-identical image changes
would need inspection. No renderer changes or new supported length units.

## Increment qualified

Surrounding custom-property regression completed228/228. Every source/browser/
native image pair is identical to the explicitly reviewed layout-contract-full
baseline or the two newly reviewed fixtures:228/228 transferred, no inspection
queue. Evidence: non-length-dimensions-custom/baseline-comparison.json and
pixels.log. Session88854 is terminal; no tasks are running.

This qualifies known angle/time/frequency/resolution/flex units invalidating
substituted length slots in the documented profile. Module190, JS9/parity,
TypeScript, Chrome182 references and native new6 plus surrounding228 all pass.
The previous full native1477-check baseline remains the last full renderer run;
this localized compiler increment used the complete custom-property lane.
Unknown units, function grammar, and remaining complex background/font prefixes
remain S09/S10 work; no full custom-property grammar or vector fidelity claim.

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
