# Language contract: nuxie-html-v1

**Linear gradients: public emission candidate, not yet qualified.** The current
working tree accepts one `linear-gradient()` through `background` or
`background-image`, with default/cardinal/corner/angle directions, existing
named/hex/RGB/HSL colors and currentColor, omitted stops, signed px/em/rem/%
positions and double-position stops (2–256 expanded stops). Stops remain
responsive in version26 `layout-css-linear-gradient-v1` requirements; hosts must
validate and install them alongside the Rive bytes. Relative font lengths are
computed before emission; percentages and corner directions remain live.
Hints, explicit interpolation spaces, repeating-gradient syntax, multiple
background layers and background position/size/repeat controls are intentionally
unsupported pending later work. Default repetition beneath transparent borders
is implemented with two-dimensional tile wrapping. Public native/WASM parity
and 272 original/clone resize frames pass, with audited visual coverage. Broad
regression, backend and performance qualification remain incomplete. See `validation/linear-gradient-implementation-notes.md`.

**Group opacity is implemented; full qualification is pending.** `opacity` accepts
one finite number or percentage, clamps to [0,1], and supports the existing
CSS-wide keywords and custom-property cascade. It is not inherited by default.
Values below 1 create an isolated subtree and stacking context; opacity 1 omits
the group requirement. General opacity math remains intentionally unsupported.
Version 25 `layout-css-group-opacity-v1` requirements must accompany the Rive
bytes and be installed on the imported layout owners. The current supported
execution path records checked group boundaries and composites native canvases;
immediate renderers without group support must reject before drawing. Replay
currently uses viewport-sized intermediate surfaces, a 256 MiB RGBA pixel budget,
at most 64 nested groups, and finite invertible transforms. These are explicit
runtime constraints, not browser-baked layout. Public native/WASM parity and
88-scene / 704-frame focused native lifecycle evidence and 264 reviewed views
are available, including underline/strikethrough clipping. Exact surface-budget
boundary controls pass; host transform combinations, backend constraints and
full regression remain open.
See `validation/group-opacity-implementation-notes.md` for current evidence;
earlier statements that all opacity is rejected are historical.

**Borders and corners have native-profile qualification.** Uniform and physical
side borders accept solid/none/hidden styles, supported colors/currentColor,
and nonnegative px/em/rem or thin/medium/thick widths. Border longhands accept
1–4 values; physical corner radii support independent elliptical axes and live
percentages. Transparent borders retain their used widths. Version22/23 border
and version24 corner occurrence policies must accompany the Rive bytes.
Other border styles, percentage border widths and border images remain excluded.
See `validation/borders-investigation.md` and
`validation/elliptical-radii-review.md` for scoped evidence and limitations.


This is an explicit authoring profile of HTML/CSS. Unsupported input is an
error, including unsupported declarations in selectors that match no elements.
Malformed HTML requiring parser repairs is rejected. Unlike a browser, the
compiler does not silently ignore invalid declarations. Comments are allowed.

## Defaults and formatting

The browser reference must load [`src/reset.css`](src/reset.css) before authored
CSS. Supported containers are flex columns; `span` is also a container, not an
inline run. The implicit artboard/body is a column, with no margin or padding.
All elements use border-box sizing, zero margin/padding/border, zero minimum
sizes and `flex: 0 0 auto`. Headings have no special size or weight. Text defaults
are Inter, 16px, weight 400, line-height 24px, black; text needs an embedded font.
The reset is fixed profile behavior and is not itself input to the compiler.
Authored CSS applies only to descendants of the host body; it cannot restyle the
host html/body canvas. Selectors see ordinary html/body ancestry. For a browser
reference, scope each comma-separated authored selector `S` as
`:where(body) :is(S)` (preserving its specificity). Apply this only to the authored
stylesheet, not the reset. Inline styles already belong to fragment elements.

## Accepted HTML

| Area | Supported |
| --- | --- |
| Elements | `div`, `section`, `main`, `article`, `header`, `footer`, `p`, `h1`–`h6`, `span`, `img` |
| Shared attributes | `id`, `class`, `style`, `data-nuxie-id`, `lang`, inert ASCII `data-*` metadata on supported containers/images |
| Image attribute | `src="asset:<key>"`, resolved from supplied image assets |
| Text | Plain leaf text; `br` in an explicit text-only `display:block` container; entities decoded; normal/nowrap collapse and trim ASCII whitespace per forced line; pre preserves spaces/newlines and default tabs |
| Identity | Unique nonempty authored IDs; otherwise structural paths |

The input is an HTML fragment. It cannot contain `html`, `head`, `body`, `style`
or top-level non-whitespace text. Put CSS in the separate CSS input. Mixed text
and child elements other than `br` is rejected: nested rich text and general
inline formatting are not implemented. Unicode glyphs must exist in the selected font. Bidirectional and
complex-script fidelity are not yet qualified by the browser suite.

## Accepted CSS

| Area | Supported syntax |
| --- | --- |
| Selectors | Type, class, ID, universal, compound, `:first-child`/`:last-child`/`:only-child`, `:nth-child()`/`:nth-last-child()` with optional `of` lists, `:not()`/`:is()`/`:where()` selector lists, descendant, child (`>`), adjacent/general siblings (`+`, `~`), comma lists, attribute presence/`=`/`~=`/`|=`/`^=`/`$=`/`*=` and optional `i` flag; names decode to ASCII, CSS escapes accepted |
| Cascade | Specificity, source order, inline styles, `!important`; automatic text inheritance, `inherit`, and constrained `initial`/`unset` |
| Display | `flex`, `none`, `block` for text-only containers |
| Direction | `flex-direction: row`, `column`, `row-reverse`, `column-reverse`; reverse layout/source identity qualified:60/60 in both profiles and full native1582/1582 (L01) |
| Wrapping | `flex-wrap: nowrap`, `wrap` or `wrap-reverse` (L06 native qualified); supported item, main-axis and line alignment can be combined (L04 native qualified) |
| Flex items | `flex-grow`, `flex-shrink`, `flex-basis`, `flex` shorthand, with the representability restrictions below |
| Order | Signed integer `order`; stable ties and authored source identities; requires declared CSS paint-order capability |
| Alignment | `align-items: flex-start/center/flex-end/stretch`; `align-self: auto/flex-start/center/flex-end/stretch/baseline`; `justify-content: flex-start/center/flex-end/space-between/space-around/space-evenly`; `align-content: flex-start/center/flex-end/stretch/space-between/space-around/space-evenly`. Main around/evenly and line evenly are native-profile qualified (L05); vector text limitations remain. |
| Dimensions | `width`, `height`: nonnegative px, em, rem, %, `auto` |
| Limits | `min-width`, `max-width`, `min-height`, `max-height`: nonnegative px, em, rem or % |
| Box sizing | `border-box`, `content-box`; initial/unset use content-box, inherit copies parent. Requires v13 content-box capability; native-qualified, see [L12 evidence](validation/content-box-contract.md). |
| Spacing | `padding`: nonnegative px, em, rem or %. `margin`: signed px, em, rem or %, plus `auto`; both support 1–4 values and physical side longhands. Percentage spacing has native validation with vector text residuals (L15); negative margins are native-qualified with nine vector text residuals (L16). See [current signed-margin scope](validation/negative-margins-review.md). |
| Gaps | `gap` (1–2 values), `row-gap`, `column-gap`: nonnegative px, em or rem |
| Paint colors | `background-color`, `color`: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `transparent`, `currentColor`, named sRGB colors, numeric/percentage `rgb()`/`rgba()`, `hsl()`/`hsla()` |
| Background gradient | One `linear-gradient()` in `background` or `background-image`; version26 occurrence contract. Public candidate and profile limitations are stated at the top of this document. |
| Group opacity | Finite number or percentage, clamped to [0,1], isolated subtree compositing; version25 occurrence contract and explicit backend/resource constraints above. |
| Borders | Uniform and physical sides; solid/none/hidden, supported colors and widths; 1–4 border longhand values, version22/23 occurrence contracts. |
| Corners | `border-radius`: 1–4 nonnegative px/em/rem/% values per axis, optional slash; physical corner longhands accept 1–2 values. Percentages resolve against live width/height; shared overlap reduction. Requires v24 `layout-css-corner-radii-v1` for per-axis/percentage policy. See validation/elliptical-radii-review.md. |
| Font | `font-family`: one identifier or quoted name; `font-size`: positive px, em or rem; `font-weight`: normal, bold, integer 100–900 |
| Text alignment | `text-align: left`, `center`, `right`; inherited |
| Text clipping | `overflow: visible`, `clip`, static `hidden`; responsive clipping, no scrolling |
| Text ellipsis | `text-overflow: clip`, `ellipsis`; ellipsis requires nonempty text-only `display:block`, `white-space:nowrap`, and hidden/clip overflow; see [current A23 scope](#public-single-line-ellipsis-current-a23-scope) |
| Decorations | Solid `underline`, `line-through`, or both, with documented color/thickness and runtime occurrence requirements below |
| Lines | `line-height`: normal, positive px/em/rem or unitless multiplier; explicit used heights must be at least the selected font's natural ascent plus descent |

Unitless zero is accepted where a px length is accepted. Negative padding, gaps
and dimensions remain rejected. Signed margins are native-qualified,
with qualification limits documented under L16. Auto margins retain native layout units
for runtime distribution; L07 is native-qualified. Percentages remain native layout rules;
they are not baked into rectangles. CSS keywords/property names are insensitive
to ASCII case. Font lookup matches the supplied family and exact weight.
Independent flex line alignment uses the explicit runtime requirement policy
qualified in L04/L05, separate from item alignment.

## Explicit inheritance

`inherit` is accepted, case-insensitively, as the entire value of every supported
property. It copies the parent's computed value. Spacing, gap and flex shorthands
copy their constituent longhands; later declarations and important flags follow
the existing cascade. Quoted font-family names are names, not global keywords.

Inherited percentage dimensions remain percentages relative to the child's
containing block, rather than copying the parent's pixel width. Inherited
`background-color: currentColor` resolves against the child's own color.
Root elements inherit from the fixed host/body profile in reset.css, including
100% width/height. Layout definiteness is recomputed in the child's context;
unsupported flex combinations remain errors after inheritance.

This does not enable unsupported properties: `grid-template-columns:inherit`
is still rejected even in unmatched rules. `initial` and `unset` follow the contract below; they are not aliases for the
profile reset defaults.

## Initial and unset values

`initial` restores CSS initial behavior for representable supported properties:
auto width/height/basis, row direction, nowrap, zero grow, shrink 1, zero spacing
and radius, transparent background, no maximum dimensions, normal flex alignment
(stretch/start), normal weight/line-height, and start text alignment (left in this profile).
The reference environment's medium font size is 16px. `flex:initial` resets all
three longhands to `0 1 auto`; the existing equal-factor restriction still
applies after the cascade, so overriding a factor may be necessary.

`unset` inherits `color`, `font`, `font-family`, `font-size`, `font-weight`, `line-height`
and `text-align`. Other supported properties use their CSS initial behavior.
Shorthands and important declarations participate in the usual cascade.

Initial values outside the current profile produce `unsupported-initial-value`:

| Property | CSS initial behavior requiring future support |
| --- | --- |
| display | Inline formatting |
| min-width / min-height | Automatic minimum sizing |
| font-family | Environment-dependent default font |
| color | Environment-dependent initial/system text color |

These rejections also apply to unmatched declarations, consistently with strict
profile validation. `unset` of inherited properties can still work because it
copies a supported parent value. Unsupported property names remain errors.
Quoted font-family names are not global keywords. This mapping is based on
[CSS Cascade 5](https://www.w3.org/TR/css-cascade-5/#defaulting-keywords) and the
fixed Chromium/LTR reference environment.

## Flex item contract

The profile still defaults to `flex:0 0 auto`; adding flex properties does not
change existing designs. Supported explicit forms include `flex:1`,
`flex:2 2 0%`, `flex:1 1 100px`, `flex:0 0 40px`, `flex:none`, and `flex:auto`, including documented
content-derived basis. Shorthands reset all three longhands;
later declarations and `!important` participate in the ordinary cascade.
Omitted shorthand basis follows Chromium at `0%`, preserving the distinction
from `0px` (the current specification describes a zero-length default).

The semantic reference is the [CSS Flexbox specification](https://www.w3.org/TR/css-flexbox-1/#flex-property),
with pinned Chrome as the executable oracle. Independent grow/shrink factors in [0, 10000],
including sub-unit factors, content-derived auto basis and percentage basis in indefinite
containers have documented native-profile support. Examples include `flex:2`,
`flex:initial`, `flex:0 1 70px` and explicit independent longhands. Corresponding
runtime occurrence policies preserve authored sizing through subsequent layout;
hosts must validate and install every emitted requirement.

This is still an explicit profile, not exhaustive Flexbox equivalence. Consult
`validation/independent-flex-investigation.md`,
`validation/partial-flex-investigation.md`,
`validation/content-auto-investigation.md` and the L11 backlog evidence for
accepted numeric bounds, intrinsic-sizing rules and retained vector-text gaps.
Earlier rejection cases and investigation results below describe their dated
snapshots; they do not override this current contract.

## RGB color contract

Both `color` and `background-color` accept `rgb()` and its `rgba()` alias:

* Legacy commas: `rgb(51, 102, 153)` or `rgba(20%, 40%, 60%, .5)`.
  All three channels must use the same numeric or percentage notation.
* Modern spaces: `rgb(51 40% 153 / 50%)`. Channel units may be mixed;
  slash-separated alpha is optional. Both function names accept either form.
* Finite decimal and scientific notation are accepted. Channels and alpha are
  clamped to their CSS ranges, then rounded to the nearest 8-bit scene channel.
  This quantization can differ slightly from browser compositing precision and
  is measured by the unchanged visual thresholds.
* Comments, case-insensitive function names, inheritance and `!important` use
  the ordinary cascade. Unclosed functions and invalid separators are errors,
  including in unmatched rules. Non-finite numeric tokens are rejected.

The reference is [CSS Color 4 RGB syntax](https://www.w3.org/TR/css-color-4/#rgb-functions),
validated against Chromium. Missing components (`none`), relative colors,
system colors, HWB, wide-gamut
colors and nested calculations remain intentionally unsupported. Variable substitution
is being integrated as documented under S09/S10 below.

## Solid background shorthand

`background` accepts one supported color value, `currentColor`, or `none`.
`background:none`, `initial` and `unset` clear the color to transparent.
`background:inherit` copies the parent's background, retaining currentColor
semantics. Shorthand and `background-color` declarations share the same cascade;
source order, specificity, inline styles and important flags determine the result.

This is the solid-color subset: image layers, comma-separated layers, position,
size, repeat, attachment and box keywords are rejected, including combinations
of these with a color. Background image/position/size state does not yet exist
in the compiler. When those features are added, their shorthand reset semantics
must be implemented together, not treated as an alias for color alone.

## Named and HSL colors

All 148 standard named sRGB colors are accepted case-insensitively, including
aliases such as gray/grey and cyan/aqua. System colors such as CanvasText and
ButtonFace are rejected because they depend on an external browser theme.
The full named palette is compared against Chromium, not compiler-derived
expected color values.

`hsl()` and `hsla()` accept legacy comma-separated and modern space-separated
forms. Hue accepts a number in degrees or `deg`, `grad`, `rad`, `turn`; finite
hues wrap around the color wheel. Legacy saturation/lightness require `%`;
modern syntax also accepts numbers on the 0–100 scale. Saturation/lightness and
alpha clamp to their supported ranges. Alpha accepts a number or percentage,
with a comma in legacy syntax or slash in modern syntax. Colors lower to the
same rounded 8-bit sRGB scene channels as RGB. `none`, relative colors, nested
functions and non-finite numeric values remain rejected.

References: [CSS named colors](https://www.w3.org/TR/css-color-4/#named-colors)
and [HSL syntax](https://www.w3.org/TR/css-color-4/#the-hsl-notation).

## Current color

`currentColor` is accepted (case-insensitively) for `color` and
`background-color`. For `color`, it uses the parent element's computed color
(the profile starts with black). For `background-color`, it resolves to this
element's final computed color, regardless of declaration order. It retains
alpha. Background colors themselves do not inherit. Resolution happens during
compilation; this does not introduce runtime themes or bindings.

The reference is [CSS Color 4 currentcolor](https://www.w3.org/TR/css-color-4/#currentcolor-color).
The browser fixtures cover source order, inline styles, important declarations,
nested text/fill inheritance, default black and transparent colors.

## Unitless line-height

`line-height` accepts a positive finite multiplier up to 10000 as well as px.
Multipliers are inherited as numbers and multiplied by each descendant's final
font size, including explicit `inherit` and `unset`. Pixel values inherit their
computed length unchanged. Declaration order does not change this distinction.

Explicit used heights must remain between the selected font's natural ascent
plus descent and 1000000px. Zero, overlapping explicit line boxes below that
minimum, percentages and non-finite values remain unsupported.
Font shorthand needs the same retained multiplier semantics; it is tracked
separately in the backlog. Existing Q09 typography gaps are not resolved merely
by accepting a different line-height syntax.

## Text alignment

`text-align: left`, `center` and `right` align each rendered line inside the
text content box, after padding. The property inherits and participates in the
ordinary cascade. Alignment remains native as the imported scene is resized;
no per-viewport offsets or browser-measured line breaks are baked into the file.
The default is left. This applies to the profile's plain leaf text, not to
alignment of child flex boxes.

Logical `start`/`end`, `match-parent`, justification, `text-align-last`, direction
controls and bidirectional fidelity remain deferred. Tests qualify the existing
Inter/left-to-right text profile, including wrapped paragraphs, flexible boxes,
inline styles and important overrides.

## Assets and bounds

Fonts: static, upright OpenType/TrueType with a matching declared weight. No
synthetic bold/italic, variable-font axes or fallback fonts. The visual suite
currently qualifies the checked-in Inter regular font, not every accepted font.

Images: static RGB/RGBA 8-bit PNG, at most 4096px per side, no ICC,
gamma/chromaticity, HDR or EXIF metadata (an sRGB marker is allowed). Both
width and height may be explicit; images stretch to that box. Experimental L14
admission now also accepts one or both automatic dimensions. Natural dimensions
come from the validated embedded PNG; automatic axes remain responsive to flex
layout and min/max constraints. A bare positive authored aspect-ratio takes
priority; auto, degenerate ratios and auto+ratio use the loaded natural ratio,
referenced to the content box. Padding and authored box-sizing remain distinct.
The existing version15 ratio capability is required when an automatic image axis
uses these semantics. Initial square/wide/tall public geometry tests pass;
native/WASM parity and128 scenes/384 Chrome/native geometry and pixel comparisons pass. Initial and card visual review covers all384 comparisons. The row-stretch fix passes all 384 flex geometry/pixel comparisons; all 384 now have audited visual coverage, including the 24 corrected views. All 384 stretch-edge comparisons also pass with complete audited visual coverage. Together with initial/card and flex coverage, this is 1,152 focused Chrome/native comparisons. Full native regression with the fix is pending. Vector fallback passes all 1,152 focused geometry checks and 1,071 pixel comparisons; 81 text-card pixel failures remain unwaived. Its 768 flex/edge comparisons have complete audited visual coverage.
Crop/cover/contain, external URL loading, HTML sizing attributes and color-managed
images remain outside this image contract.

Limits: HTML + CSS ≤1 MiB; ≤2048 elements; nesting depth ≤64 below a root
element; ≤1024 selectors; ≤256 declarations per block; ≤128 assets; ≤16 MiB per
asset and ≤32 MiB total asset bytes; ≤64 MiB decoded PNG. Viewport dimensions
must be finite and in (0, 16384]. Px values are capped at 1,000,000 and
layout percentages at 10,000%; underline used-metric limits are described below.
These are resource limits, not promised sensible layouts.

## Stacking support (version19, qualified subset)

`z-index: auto` and signed integers are implemented, including explicit zero,
CSS-wide keywords and custom-property substitution. Fractional values, lengths,
multiple values and math expressions are rejected. Integer contexts are atomic,
including on static flex items; they do not change positioning containing blocks.
Hosts must validate and install `layout-css-stacking-v1` / `layout_stacking`.
Equal levels use the compiler artifact's CSS order-modified object preorder;
hosts must preserve that order and the manifest object identities.

The corrected clip snapshot has288/288 focused Chrome comparisons and audited
visual coverage in each renderer profile. A separate rectangular clipping
lifecycle corpus passes864 original/clone draw comparisons with complete visual
coverage;144 discriminators verify both ancestor clip boundaries affect pixels.
The full regression passes6223 checks; all6213 scene pairs have complete visual
coverage. The additional48 regular overlap checks also pass and are audited.
See validation/stacking-context-investigation.md for snapshot-specific evidence.

## Intentionally unsupported

* CSS Grid; editor integration; scripts, events, bindings, animation and state
  machines. Styled containers have no interactive behavior.
* General block/inline formatting, margin collapsing, tables/lists and nested
  rich text. Leaf text and the documented text-only `br` subset are supported.
* Fixed/sticky positioning, scrolling, transforms, shadows and filters.
  Absolute/relative positioning, static clipping, content-box sizing, borders,
  elliptical radii and group opacity are supported within their stated profiles.
* Radial/conic and repeating gradient functions, interpolation hints or explicit
  interpolation spaces, multiple background layers, image backgrounds and
  background position/size/repeat/origin/clip controls. One linear gradient is
  accepted as the candidate described above; its default tiling is implemented.
  Embedded `img` elements are a separate supported image path.
* CSS functions or units outside the property-specific accepted grammar.
  `var()`, supported RGB/HSL functions and documented property-specific math are
  exceptions; this does not imply general CSS math or viewport-unit support.
  Media/container queries, other unsupported at-rules and CSS nesting remain out.
* Pseudo-elements and pseudo-classes outside the accepted selector table;
  unsupported font fallback, vertical text, logical/justified text alignment,
  custom or multiline ellipsis, and decorations outside the documented subsets.
* Links/form controls, semantic accessibility export, source-format preservation,
  editor mutation/undo and incremental compilation.

Acceptance of syntax is not proof of exhaustive browser equivalence for every
combination. [Validation](VALIDATION.md) describes the measured coverage and the
requirements for extending it.

Known accepted-input visual gaps are also tracked in
[`validation/known-gaps.md`](validation/known-gaps.md). In particular, the
membership card and multi-size typography specimen do not yet meet the pixel
gate. Geometry correctness alone does not qualify those compositions.

## Font shorthand (partial)

`font` accepts `[normal/weight prefixes] <positive-px-em-or-rem-size> [ / <line-height> ]
<single-family>`. Line-height accepts normal, positive px/em/rem or a unitless
multiplier; the existing explicit used-height limits apply. Up to four optional prefix
slots represent style, variant, weight and width: only `normal` is supported for
style/variant/width, and weight accepts `normal`, `bold` or integer 100–900.
The family is one identifier or quoted name, preserving its case and escapes.

Omitted weight resets to 400. All other font subproperties currently have fixed
normal behavior and cannot be authored. `font:inherit` and `font:unset` copy the
parent's supported font fields, retaining a unitless line-height multiplier.
Shorthand and longhand precedence follow source order, specificity and important
flags. Adding font subproperties must extend the shorthand's reset behavior.

Omitted line-height resets to `normal`, retaining the keyword through
inheritance. `line-height:initial` also selects normal. For the pinned browser
profile, normal line spacing is the sum of individually rounded font ascent,
descent and line gap at the element's font size. It is not the profile's 24px
reset. Runtime final-baseline trimming and font-derived trailing space preserve
this line box when fractional natural metrics are slightly larger, while the
runtime still performs wrapping and resizing. Nonpositive or excessive font-
derived line boxes and negative trailing space are diagnosed.

System fonts, `font:initial` (environment-dependent family), non-normal
style/variant/width, fallback lists and units other than px/em/rem remain unsupported.
Normal line-height's geometry matches the browser fixtures. The experimental
macOS glyph lane passes its full pixel gate; the default vector profile retains
three rasterization failures. A07 remains partial across profiles; see
validation/live-glyph-review.md.

Shorthand reset semantics follow [CSS Fonts](https://www.w3.org/TR/css-fonts-4/#font-prop).

Text baseline placement follows the pinned Chromium reference's rounded font
metrics and floored half-leading. The compiler emits a glyph translation while
retaining native line-box sizing and responsive wrapping. This correction does
not promise identical platform font smoothing;
remaining rasterization specimens are documented in validation/known-gaps.md.

The opt-in macOS Rust/CoreText adapter now consumes live runtime glyphs and
passes all 233 full-corpus checks plus nine realistic specimen comparisons.
It does not expand the accepted HTML/CSS language. Default renderers retain
vector drawing. Rectangular clips, representative affine transforms, glyph opacity and
restoration pass 27 host-state controls; see validation/glyph-state-review.md.
CSS group compositing and additional platforms remain unqualified. Installation,
resource bounds and fallback eligibility are in validation/live-glyph-review.md.

## Em lengths

`em` is accepted wherever this profile accepts a length: dimensions, min/max
limits, flex basis (including `flex`), physical padding/margin, gaps, the single
border-radius value, font size and line height. Existing restrictions on auto,
percentages, negative values, flex factors and natural line-height minima remain.

For `font-size`, the basis is the parent's computed font size; repeated size
declarations do not multiply each other. Other em lengths use the element's
final cascaded font size, irrespective of declaration order. Font shorthand is
expanded before resolving those dependencies. An em line height becomes a
computed pixel length that children inherit, unlike a unitless multiplier.
`inherit` copies computed lengths; it does not reevaluate the parent's em token.

The host/body font size is 16px. Units are case-insensitive. Coefficients must be
finite and in [0, 1000000]; resolved values retain the 1000000px limit. Font size
and explicit line height must be positive. Unmatched rules receive syntax and
coefficient checks without inventing a font context; resolved limits are checked
on matching elements. This does not enable ex, calc(),
percentage gaps or other deferred syntax. Relative sizing follows the
[CSS font-relative length rules](https://www.w3.org/TR/css-values-4/#font-relative-lengths).

Em visual qualification is partial: all new geometry checks pass, but the
fractional-edge case exceeds the shape pixel budget at 240px in both render
profiles. The vector profile also fails two new text comparisons. See
[the em validation receipt](validation/em-review.md). Accepted syntax does not
imply pixel equivalence in those cases.

## Root-relative lengths (rem)

The versioned authoring profile fixes the host html font size to 16px explicitly
in reset.css. Authored fragments cannot restyle that host. Every rem is therefore
16 CSS pixels, including font-size and font shorthand. Changing a fragment's top
level or nested font size does not change this basis. Em continues to use the
parent/final element font rules above. Mixed px/em/rem shorthand values are
accepted wherever those lengths are supported. Resizing the artboard does not
change either font-relative unit's basis; percentage layout remains responsive.

Rem coefficients are case-insensitive, finite and nonnegative, at most 1000000;
resolved lengths must also be at most 1000000px. Because the root size is fixed,
that resolved bound is checked even for unmatched declarations. Existing positive
font-size/line-height and natural-height restrictions still apply. Inherited rem
line heights are computed absolute lengths. Configurable host root sizes, full
HTML documents, ex/ch and calc remain excluded. This root contract implements
[CSS root font-relative units](https://www.w3.org/TR/css-values-4/#font-relative-lengths)
within the fragment profile; it does not use the first authored element as root.

The rem subset passes 12 browser/native comparisons in each renderer profile;
see [rem-review.md](validation/rem-review.md). Existing fractional-edge and
vector text limitations still apply when composing other supported values.

## Percentage minimum and maximum dimensions

Min/max width and height accept nonnegative percentages up to 10000%. Values
remain native percentage units in the Rive layout record and are re-resolved
against the parent content box as the scene is resized. Inherit copies the
percentage, not a pixel result. A minimum takes precedence over a smaller maximum.
The min/max syntax does not enable auto/intrinsic keywords or change existing
initial/unset restrictions. Definite and indefinite parent cases, text and flex
interactions pass the 24-case browser/native qualification in both renderer profiles; see
[percentage-limits-review.md](validation/percentage-limits-review.md).

## Letter spacing (partial qualification)

Letter-spacing accepts normal, zero, and signed px/em/rem lengths. Coefficients
and resolved lengths must be finite with absolute value at most 1000000; other
properties retain their existing nonnegative restrictions. Em uses the final
computed element font size; rem uses the fixed 16px host root. Inheritance and
unset copy the computed pixel length. Initial/normal resolve to zero in this
non-justified profile. Font shorthand does not reset letter spacing.

The compiler writes native TextStylePaint letterSpacing and requests the
`text-css-letter-spacing-v1` runtime capability for every nonzero text style.
The checked host applies that policy to font assets and retains it through
replacement. All 39 current spacing comparisons, including optional ligatures, multiple
combining marks and intrinsic trailing spacing, pass in both renderer profiles.
The policy suppresses optional liga/clig/dlig/hlig features at nonzero spacing;
zero/normal spacing preserves them. Required ligatures are not disabled. The
older cluster-only capability remains distinct and does not satisfy the new
requirement. Percentages, other units and calc remain rejected. Font fallback
and broader Unicode-script behavior still need qualification; A12 remains partial.

## Runtime capability contract

Publish Rive bytes, source map and runtime requirements together. Rust output
has `runtime_requirements`; JS/WASM has `runtimeRequirements`; the CLI writes
`.requirements.json`. Version 1 contains a deterministic capability list. Every emitted Text object
requires `text-css-shaping-precision-v1`; text-free scenes may have an empty list.
Hosts accepting this capability must set `ShapingPrecision::CssExperimental`
on imported font assets with `FontAsset::set_shaping_precision_occurrence`
and retain it across replacement. This selects bounded high-precision advances
and offsets; it does not change outline extraction or accept new CSS syntax. Version 2 adds an ordered text_policies list, each
entry containing an artboard-local Text object_id and a versioned policy.
The compiler emits version 2 for nonempty nowrap/pre/pre-wrap/pre-line text. Hosts must validate
the version/capabilities before import, call ensure_text_targets against the
actual imported default artboard, and install every accepted policy. Duplicate
targets (including conflicting policies), missing capability declarations, empty version-2 lists, unknown policies
and missing/non-Text targets are rejected. Version 1 cannot carry occurrence
policies; its older scene-wide behavior remains supported. These checks do not
authenticate a manifest or detect substitution of one valid Text ID for another;
keep the three artifacts from the same compilation together. The checked reference adapter rejects
missing metadata and unsupported requirements before drawing.

Raw Rive import alone does not read this manifest and retains legacy per-glyph
spacing, which differs on multiple-mark clusters. Do not discard the requirements
and claim the same semantics. The manifest is a capability contract, not an
authenticated container. See
[runtime-requirements-review.md](validation/runtime-requirements-review.md).

## Word spacing (partial qualification)

`word-spacing` accepts normal, zero, and signed px/em/rem lengths. Absolute
coefficients and computed pixel values are bounded by 1000000. Initial/normal
are zero; inherit/unset copy the computed length. Font shorthand preserves it.
Percentages, other units, calc and multi-value syntax remain rejected, including
on unmatched selectors.

The current mapping expands U+0020 SPACE and U+00A0 NO-BREAK SPACE after the
whitespace collapse under normal/nowrap. It adds word spacing to the separator's native
advance using a separate style within the same Text object. Ordinary letters
retain their letter spacing, including their ligature policy. It does not add
spacing to fixed-width Unicode spaces or punctuation. Visible-script separators
U+1361/U+10100/U+10101/U+1039F/U+1091F with nonzero word spacing are explicitly
rejected pending qualification. Broader scripts and font fallback remain outside
the qualified text profile.

Run expansion is limited to 4096 per element. Source maps expose every generated
run in logical order through `text_run_ids`; `text_run_id` is null when more than
one run is generated. Recompile source text changes to regenerate the runs.
Changing viewport size does not require recompilation. The emitted nonzero native
spacing styles require the existing CSS spacing capability.

Text with preserved Unicode spaces requests a separate line-break capability;
those spaces retain their own font advances rather than receiving word spacing.
The original wrapping failures and their controls remain in the corpus. See
[preserved-space-breaks-review.md](validation/preserved-space-breaks-review.md)
for the correction and qualification, and the historical
[word-spacing-review.md](validation/word-spacing-review.md).

## Preserved Unicode-space breaks

`text-preserved-space-breaks-v1` handles break opportunities after U+1680,
U+2000–U+2006, U+2008–U+200A, U+205F and U+3000, preserving their advances in word
fitting and line alignment. The compiler requests it only when emitted text
contains one of these characters. U+00A0, U+2007 and U+202F remain nonbreaking.
Hosts install the retained font policy before layout and keep it through asset
replacement. It composes with the separate CSS letter-spacing capability.

This is an explicit space-break policy, not a complete Unicode line-breaking
implementation. General script behavior, fallback and default-ignorable controls
need broader qualification. U+2060 is still rejected when absent from the chosen
font; its browser control is retained in the deferred corpus. Recompile authored
text changes, which may require different fonts, run partitions or capabilities.

The declared preserved-space set passes 48 browser/native comparisons in each
renderer profile. U+1680 has visible ink and must exist in the selected font,
even though it is whitespace. Missing coverage is an explicit missing-glyph
error; the positive Ogham fixture embeds a licensed Noto Sans Ogham face. The
current full glyph result is 397/398, with the unrelated A09 fractional edge
still failing. Broader scripts and font fallback remain separate work.

## Explicit line breaks

`<br>` is supported directly inside a text-only container with `display:block`.
The profile reset still uses flex columns; an unstyled br in a flex container is
rejected as `unsupported-break-context`. Block display supports text/br content
only; child containers and empty block containers are explicitly rejected pending
block-flow support. Flex properties controlling the container's internal layout
(direction, wrapping, gaps, alignment) have no effect on block text, while its
outer flex-item sizing still applies. `display:none` hides all descendant paints.

Breaks lower to native newline characters within one Text object. Leading,
consecutive, trailing and break-only cases retain native line boxes; a trailing
break does not manufacture an extra visible line beyond browser behavior. Text
on each side uses the existing ASCII collapse/trim policy. Normal line height,
wrapping, alignment, letter spacing and word spacing compose with these breaks.
No spacer objects or browser-computed coordinates are emitted.

A br accepts only `id`, `data-nuxie-id` and `lang` (which does not change surrounding text). Authored declarations matching br,
inline styles and other attributes are rejected; typography belongs on the text
container. SourceNode.text_breaks lists break identity, structural path and the
Unicode-scalar offset of its newline in normalized text. Breaks have no separate
layout object; source-map text_run_ids still cover the complete native text.
Structural paths count br siblings. There are at most 4096 breaks per container,
8192 combined element/break identities, and the existing 2048 layout elements.
Recompile authored text changes; runtime resizing uses the same compiled scene.

## White-space nowrap

`white-space:normal` and `white-space:nowrap` are accepted and inherited.
Initial resets to normal; inherit/unset copy the parent's computed mode. Both
modes retain ASCII whitespace collapsing/trimming. Nowrap disables soft wrapping
through native Text.wrapValue but retains explicit br/newline breaks. Font
shorthand does not reset the mode. Pre-line/break-spaces, multi-value
syntax and other values remain rejected, including on unmatched selectors.

Nowrap text requests `text-css-nowrap-alignment-v1`. Before the first layout,
hosts accepting this capability must call `Text::set_css_nowrap_alignment(true)`
on each Text object listed by the version-2 text_policies mapping. Version-1
manifests retain the previous scene-wide installation. The policy clamps negative line
origins only for native NoWrap text. Shorter lines after br still honor center
or right alignment, and ordinary wrapped text retains its existing behavior.
The checked probe installs this policy before initial layout and resizing.

The policy is retained on the text occurrence through resizing, reshaping and
font replacement. It is not serialized as a Rive property: hosts must reinstall
it on fresh imports or instances. Raw Rive import keeps legacy overflow alignment.
The documented subset passes all 36 nowrap comparisons in both renderer
profiles at 240/390/768px, including the four original overflow failure fixtures,
with unchanged limits. Preserved ASCII whitespace is covered below.
See validation/nowrap-review.md for the direct-mapping failures and policy evidence.


## Preserved whitespace: pre

`white-space:pre` inherits, preserves repeated/leading/trailing spaces and source
newlines, and disables soft wrapping. Existing explicit br support remains
limited to text-only block containers. HTML source CRLF/CR is normalized to LF
by the HTML parser. The strict HTML parse-error policy remains in force, including
invalid numeric references to control characters. This adds the CSS value; the
HTML pre element is not added to the accepted element list.

Whitespace-only anonymous flex items are discarded, including indentation around
child elements under inherited pre. Whitespace-only block text retains its line
box. The normal/nowrap collapse behavior and CSS-wide resets are unchanged.
Pre text requests the existing checked text-css-nowrap-alignment-v1 capability.
Its source runs preserve spaces/newlines and br offsets count those characters.

Preserved tabs use default eight-space stops through the checked
text-css-tabs-v1 capability. Hosts must enable
FontAsset::set_experimental_css_tabs_occurrence before first layout; its policy
survives font replacement and option derivation. Tabs retain U+0009 source text
and indices, draw no ink, and use position-dependent advances. Letter and word
spacing affect the stop interval. Fonts require a positive-width inkless space
glyph; otherwise compilation reports unsupported-tab-font.

The minimum advance near a stop follows pinned Chromium's half-space behavior,
which differs from the newer CSS Text draft's half-ch rule. Custom tab-size and
break-spaces remain unsupported. Other ASCII controls
besides TAB/LF/CR remain rejected. See validation/tabs-review.md and
validation/pre-review.md for scope, measurements and qualification evidence.


The current non-tab pre corpus passes all 48 browser/native comparisons in both
renderer profiles, including resizing the same imported scenes at 240/390/768px.
All source text, blank-line and flex-whitespace cases have been visually reviewed.
Default tab qualification is recorded separately in validation/tabs-review.md.


Default tabs pass all 33 comparisons in both renderer profiles, including
alignment/overflow, consecutive and trailing tabs, explicit breaks, spacing,
and two embedded fonts. Together with the 48 non-tab cases, this qualifies the
documented LTR pre subset. Broader bidi/script qualification remains separate.
The half-space near-stop boundary intentionally follows pinned Chromium.


## Pre-wrap (partial)

`white-space:pre-wrap` preserves spaces and source newlines like pre, but enables
soft wrapping. Inherit/unset copy the computed mode; normal/initial restore
collapsing. The existing br restrictions and preserved-control diagnostics apply.
Literal source whitespace and break offsets remain intact in native text runs.

Pre-wrap requests capability text-css-pre-wrap-v1 and occurrence policy
css-pre-wrap-v1 in the version-2 requirements mapping. The checked host validates
the target Text object and calls Text::set_css_pre_wrap(true) before layout.
The policy survives resizing, reshaping and font replacement; reinstall it on a
fresh import. It remains local to the Text occurrence, even when another Text
shares its font. Version-1 manifests cannot claim this capability.

The policy preserves spaces in native glyph ranges, hangs trailing spaces at
soft breaks, and conditionally hangs them before forced breaks/block endings.
Unbreakable words overflow without emergency glyph splitting. Overflow alignment
keeps visible content at the left edge for the tested LTR profile. Source
newlines and whitespace-only lines retain their line metrics. UAX #14 break
opportunities use pinned unicode-linebreak 0.1.5 (Unicode 15.0), converted from
UTF-8 byte indices to Rive scalar indices and restricted to shaped cluster
boundaries. Hyphen, nonbreaking-space, ligature, spacing and mixed-mode fixtures
are covered; dictionary segmentation, newer Unicode tailoring and broader bidi
qualification remain open under the typography backlog.

The non-tab corpus passes all 72 native-glyph comparisons, including the seven
repaired semantic/paint cases and the tight missing-word region. Vector results are 64/72
with all geometry passing; eight glyph-edge/dense-text rasterization failures
remain under unchanged thresholds. See the policy receipt for exact fixtures.

Pre-wrap now accepts literal tabs with default eight-space stops. The compiler
additionally requests text-css-wrapped-tabs-v1, together with text-css-tabs-v1
and text-css-pre-wrap-v1. Hosts supporting only the earlier two capabilities
must reject the new combination. The existing pre-wrap occurrence mapping and
font tab policy remain required; no new global wrapping switch is installed.

The pre-wrap layout policy recalculates tab advances from each soft or forced
line origin, including word/letter spacing, while preserving source characters
and glyph identities. It retains absolute cumulative glyph positions so line
ranges remain coherent. Fonts still require a positive-width inkless space;
other font and control-character diagnostics are unchanged. Default pre tab
stops and ordinary Rive behavior are unchanged. The 16 new fixtures cover the
original deferred reproducer, alignment, consecutive/leading/trailing tabs,
newlines/br, spacing, two fonts, blank lines and mixed whitespace modes.
The wrapped-tab corpus passes 48/48 native-glyph and 43/48 vector comparisons.
Together, pre-wrap passes 120/120 glyph and 107/120 vector checks, all with exact
geometry. Thirteen vector pixel failures and broader language coverage keep A17
partially qualified.

Pre-line, break-spaces and custom tab-size remain unsupported. See
validation/prewrap-review.md for the original reproducers and
validation/prewrap-policy-review.md for the opt-in policy evidence, and
validation/wrapped-tabs-review.md for line-relative tab qualification.


## Pre-line (partial)

`white-space:pre-line` collapses consecutive ASCII spaces and tabs to one space,
removes them around source newlines and explicit br, preserves newlines, and
allows soft wrapping. HTML parsing normalizes literal CR/CRLF to LF. Inherit and
unset copy the mode; normal and initial restore normal whitespace processing.
The existing text-only block restriction on br remains. Break source-map offsets
count Unicode scalars after whitespace normalization.

Empty and space/tab-only pre-line blocks are accepted and emit no Text object;
they have no text line box. Preserved newlines create line boxes. Whitespace-only
anonymous flex text continues to be discarded. Invalid preserved ASCII controls
remain rejected, and this feature does not add the HTML pre element.

Noncollapsible Unicode spaces retain their source identities. Terminal Unicode
space separators hang at both soft and forced line endings, except U+1680
Ogham spaces. These retain their ink and contribute their advances to alignment
to match pinned Chromium; the CSS draft's at-risk
trailing Ogham trimming is intentionally not implemented. Wrapping uses the existing
pinned UAX #14 cluster-boundary policy. Long unbreakable words overflow without
emergency splitting. Broader bidi, dictionary segmentation and newer Unicode
tailoring remain separate typography work.

Nonempty pre-line text requires version-2 text-css-pre-line-v1 capability and
css-pre-line-v1 occurrence mapping. The checked host validates the target Text
object and calls Text::set_css_pre_line(true) before layout. The policy survives
resizing, reshaping and font replacement, and must be installed again on fresh
imports. Tabs collapse before shaping and therefore do not request tab-stop
capabilities. Raw Rive defaults and other text occurrences retain their behavior.

The 24-fixture corpus passes 72/72 native-glyph and 70/72 vector comparisons.
All geometry is within 0.00390625 CSS px of Chromium. The mixed-mode scene at
240/390px exceeds vector rasterization budgets; these remain recorded failures.
Sparse Ogham ink is checked for presence and position as well as pixel error.
See validation/preline-review.md for the documented browser/draft distinction,
visual evidence and remaining qualification limits.


## Text transforms (partial)

`text-transform:none`, `uppercase`, `lowercase` and `capitalize` are accepted and inherited.
Inherit/unset retain the parent operation; initial restores none. Font shorthands
do not reset text-transform. Casing follows default Unicode 17.0 full mappings,
including expanding uppercase mappings and contextual final sigma in lowercase.
The compiler pins the standard-library Unicode table version at build time;
changing it requires requalification. No locale is inferred from the host.

Transformation follows whitespace normalization and precedes font validation,
shaping, spacing and runtime line breaking. Required glyph coverage therefore
applies to rendered text. Responsive layout remains native. No additional runtime
capability is needed for casing already materialized in the emitted text runs.

For nonempty transformed text, SourceNode.text_transform contains source
(normalized text before casing), rendered text, and scalar_offsets. The array
maps every source Unicode scalar boundary, including the end, to its rendered
scalar offset. Expanded characters span multiple rendered scalars; these are
not UTF-16 or byte offsets. SourceBreak.text_offset now explicitly refers to the
rendered newline. Existing untransformed maps omit text_transform. The original
HTML/CSS remains the authoring document; the normalized metadata does not replace
raw source storage or implement a clipboard/accessibility interface.

Language tailoring is described below. Full-width and full-size-kana
remain unsupported and remain A19 work. Other existing HTML/text restrictions
still apply. The 20-fixture default-casing corpus passes 60/60 native-glyph and
59/60 vector comparisons, with maximum geometry error 0.01171875 CSS px. The
mixed-style fixture at 240px retains a vector rasterization failure under the
existing budgets. See validation/text-transform-review.md for evidence and limits.


### Capitalization (partial qualification)

`capitalize` follows pinned Chromium's untagged-text behavior: simple titlecase
at word starts while retaining the remainder's case. This differs from full
Unicode titlecasing: sharp-s and multi-letter ligatures do not expand, and
supplementary-plane characters remain unchanged. These behaviors are explicit
browser compatibility choices, not a claim of full Unicode titlecase semantics.
Digraph titlecase forms such as ǳ to ǲ are supported. Numbers at word starts do
not cause following letters to capitalize; apostrophes and underscores retain
word continuity. NBSP and Chromium's period/colon forms introduce boundaries.

The compiler uses pinned ICU4X 2.3.0 case/word data with dictionary segmentation,
plus measured Chromium punctuation handling. Segmentation normalization changes
neither emitted punctuation nor whitespace. Source/rendered maps remain present;
capitalization retains one rendered scalar per source scalar. Casing stays
compile-time presentation; native responsive layout is unchanged.

A 62-string independent logical-text reference is checked both by Rust tests
and directly against Chromium in the validation workflow. It covers punctuation,
numeric contexts, combining marks, symbols, supplementary characters and mixed
scripts. This is casing coverage; it does not imply fonts or rendering have been
qualified for all those scripts. Language-specific capitalization remains open.
The 20-fixture capitalization corpus passes 60/60 native-glyph and 59/60 vector
comparisons, with exact geometry throughout. The mixed fixture at 240px retains
a vector rasterization failure. Together with upper/lower, current transforms
pass 120/120 glyph and 118/120 vector comparisons. See
validation/capitalize-review.md for scope and evidence.


### Language-specific casing (qualified subset)

The global `lang` attribute inherits through supported elements. An empty value
resets inherited language. Uppercase/lowercase use contextual Turkish (`tr`),
Azeri (`az`), Lithuanian (`lt`) and Greek (`el`) rules, selected by the
case-insensitive primary language subtag. Other languages retain default casing.
No host locale is consulted. This does not enable language-dependent shaping,
hyphenation, direction, or arbitrary inline elements.

Capitalization retains the measured pinned-Chromium behavior for declared
languages too: Dutch `ijsselmeer` becomes `Ijsselmeer`; Turkish `istanbul`
becomes `Istanbul`. Full locale titlecasing is not claimed.

Contextual insertions and deletions are captured by scalar-boundary source maps:
Lithuanian lowercasing can add a dot; Turkish lowercasing can remove a combining
dot. A narrow vendored ICU4X observer records these edits during contextual
mapping, without reconstructing them from a string diff. Thirty-six independent
logical Chromium references and public inheritance/offset tests pass. The new
16-fixture corpus passes 48/48 native-glyph and 48/48 vector comparisons with
exact geometry, including same-scene resizing at all three widths. All new
native-glyph image pairs were visually inspected. This qualifies the documented
casing subset with the pinned Inter font, not arbitrary language typography.
See validation/locale-review.md.


### Solid underline (A20, qualification still active)

The compiler now accepts these declarations and emits version 3 requirements:

| Property | Accepted values |
| --- | --- |
| `text-decoration-line` | `none`, `underline` |
| `text-decoration-color` | Existing color syntax and `currentColor` |
| `text-decoration-style` | `solid` |
| `text-decoration-thickness` | `auto`, `from-font`, nonnegative px/em/rem lengths or percentages, unitless zero |
| `text-underline-offset` | `auto`, signed px/em/rem lengths or percentages, unitless zero |
| `text-decoration-skip-ink` | `auto`, `none`, `all` |
| `text-underline-position` | `auto` |
| `text-decoration` | Any order of optional line, solid style, color and thickness; duplicate components rejected |

`inherit`, `initial` and `unset` apply to these properties. The shorthand resets
line/style/color/thickness; it does not reset offset or skip-ink. Offset and
skip-ink inherit normally. Applied ancestor lines propagate independently, so a
child's `none` retains the ancestor line; a local underline adds another ordered
record with its own color, thickness and offset. This covers the existing flex
box tree and text-only block children, not arbitrary inline formatting.

Em/rem lengths resolve at the declaring element. For propagated lines reaching
a text child, auto/from-font/percentage thickness uses that text's font and size,
matching pinned Chrome. Explicit lengths/percentages round to pixels and used
thickness is at least 1px. Auto thickness is font-size/10; from-font uses a
signed embedded-font underline metric, clamped to at least 1px (including zero
and negative metrics), with auto fallback only when the metric is absent.
Absent-metric fallback remains unqualified against Chrome: the tested font
without a post table is rejected by Chrome OTS before rendering. The compiler
fallback is covered by a public test; see validation/underline-font-metrics-review.md. Auto offset uses
max(1, ceil(thickness/2)); explicit offset is rounded. Resolved values must remain
finite and within the 1,000,000px runtime limits.

Hosts must advertise `text-solid-underlines-v1`, validate the manifest and Text
targets, and install all ordered records before layout. Fresh imports reinstall
them; resizing uses actual runtime text lines. Versions 1/2 retain their meanings.

17 fixtures pass 51/51 native-glyph and 49/51 vector comparisons with exact
Chrome geometry at 240/390/768. The two vector from-font failures are retained.
A20 remains active: broader whitespace, CJK/skip-all, clipping/opacity and missing
font-metric fallback qualification remain. Wavy/dotted/dashed/double decorations,
overline, under/from-font positioning, calc(), vertical writing,
and mixed inline text remain intentionally unsupported. No editor integration
is included. See validation/underline-css-review.md.


Underline edge qualification now covers explicit breaks, preserved and collapsed
whitespace, hidden text and transparent currentColor, including exact invisible
controls. Decoration keywords are ASCII case-insensitive. A new saturated-red
check catches tiny differences that ordinary image metrics miss: the Japanese
edge suite retains Auto/All failures at 240 in the native-glyph profile and
additional vector overlap failures. Hard skip-ink clipping is not yet pixel-
equivalent to Chrome. See validation/underline-edge-review.md; A20 stays active.

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

### A20 hard-clip integration

Runtime underlines now use hard difference clipping where supported, restoring
state before geometric fallback if declined. Native glyph corpus: 90/90 passing
and visually reviewed; vector: 85/90 passing with the five known failures.
Host-state checks improve to 17/27 in each profile, retaining ten sparse red-ink
failures. A20 remains active. See `validation/underline-hard-clip-review.md` for
scope, receipts and outstanding qualification.

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

The remaining underline scale-phase investigation now implicates accumulated
shaping precision against Chrome; see `validation/underline-transform-diagnosis.md`.
This diagnostic finding does not expand supported syntax or qualify the remaining
host-transform cases. Chrome remains the acceptance target.
A higher-precision shaping experiment fixes the preserved underline scale case;
it is not yet an installed runtime policy or a support expansion. Its remaining
host-transform failures and adoption requirements are recorded in the receipt.
The runtime now has an explicit experimental CSS shaping precision option,
retained across font replacement. Compiler output does not yet require or enable
it automatically; broad qualification and capability wiring remain pending.
The native host's alpha modulation is per draw. A font-free Chrome comparison
confirms it does not implement CSS group opacity for overlapping descendants;
P05 retains that work, including underline/glyph overlaps.
Precision validation now includes visual inspection of all 96 changed/added
native-glyph cases and nine changed vector cases. The vector underline profile
retains its five known failures (85/90); full vector regression is in progress.
No new syntax or automatic precision capability has been enabled.
A wide-glyph overflow in experimental CSS shaping precision was found and fixed
before automatic adoption. Its synthetic two-em regression now passes at large
sizes. Full vector validation retains 28 failures that also occur without the
precision option; these remain renderer limitations, not supported exceptions.

Current precision integration: compiled Text objects now require the checked
precision capability, installed automatically by the reference host. The
automatic native-glyph underline lane passes 90/90 with PNGs identical to the
reviewed precision output. See `validation/precision-policy-review.md`; earlier
experiment notes do not describe current capability emission.


### A20 device scale follow-up

Native-glyph/Metal DPR controls pass 25/27 comparisons across skip-ink auto/all/
none and widths 240/390/768. All DPR 2/3 cases pass; DPR 1 auto/all at 768 each
retain two extra red pixels beyond the existing support gate. Geometry passes
throughout; all nine overview contact sheets were inspected. Three source scenes
have native/WASM artifact parity and are reused across all sizes/DPR values.
See validation/underline-dpr-review.md. This does not qualify fractional DPR or
all renderer backends; A20 remains active.

A20 DPR failure reduced to transparent Latin text near a half-pixel clip edge.
Native advance accumulation in f64 alone does not account for the Chrome
difference; no rendering workaround applied. Reproducer and measurements:
`validation/underline-dpr-review.md` (reduced half-pixel failure).

A20 shaping diagnosis now isolates scalar advance quantization: p/q/j/space
each differ from Chrome by 1/65536px, accounting exactly for the reduced-line
width discrepancy. A failing component control and callback implementation seam
are recorded in `validation/underline-dpr-review.md`; the fix remains pending.

CSS shaping now uses a horizontal advance callback matching the observed Chrome
fixed-point conversion. DPR controls improve to 27/27; the reduced failure is
fixed. Module91 and overflow8 checks pass. Broader regression and remaining
DPR visual inspection are pending; see validation/underline-dpr-review.md.

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

Safe pixel conversion is validated by module91, overflow8 and DPR27/27. Font
widths remain170/171: the remaining long-string reference matches intermediate
Chrome accumulation boundaries and is still open. A20 remains partially
qualified; independent authoring work continues with A21. See the DPR receipt.


### Strikethrough work in progress (A21)

`line-through` remains unsupported by the public compiler. Chrome reference
controls now cover solid line metrics, paint order, ancestor propagation,
skip-ink/offset independence and combination with underline. These controls
establish the target behavior; they do not qualify runtime or compiler support.
See `validation/strikethrough-review.md` for scope and remaining gates.

A21 runtime progress: resolved strikethrough drawing now exists behind an
explicit occurrence API. Import/resize testing verifies one stripe per wrapped
line and after-glyph paint order in vector and native glyph rendering. Removing
both underline and strikethrough restores the original recording. Compiler
syntax and transport remain pending; this does not qualify Chrome/native
pixels. See `validation/strikethrough-review.md`.

A21 transport progress: version 4 and `text-solid-strikethroughs-v1` are
implemented with host validation and TypeScript declarations. CSS emission
remains pending. Initial manually supplied runtime controls pass 15/24 pixel
cases and 24/24 geometry comparisons; 2px stroke rasterization differs from
Chrome. Reproducers and review scope: `validation/strikethrough-review.md`.

A21 snapping diagnosis: a single transparent glyph reproduces the stripe
coverage mismatch. Rounding only the resolved offset fails at fractional
container positions. A stricter stripe-interior control catches the difference;
production behavior remains unchanged pending paint-coordinate snapping.
See `validation/strikethrough-review.md` for evidence and reproduction.

A21 phase reference: 384 Chrome stripe placements across two fonts, fractional
container positions and two-line layout now replay successfully. The matching
model rounds line paint origin and resolves ascent before snapping the stripe.
This is reference evidence; runtime snapping and CSS emission remain pending.
Details: `validation/strikethrough-review.md`.

A21 runtime snapping is implemented using a required version 4 `line_baseline`
metric and the original thickness. The runtime recovers each CSS line origin
after layout and snaps at paint time. Native phase pixels pass 32/32 with all
32 inspected; original resized scenes pass glyph 12/12 and vector 8/12, retaining
four vector 240px failures also seen without decoration. Module98, runtime
geometry7 and TypeScript checks pass. Public CSS emission remains pending.
See `validation/strikethrough-review.md` for scope, artifacts and remaining gates.


### Solid strikethrough CSS (A21, implemented; qualification in progress)

The compiler now accepts `text-decoration-line: line-through` and
`underline line-through` in either order. The `text-decoration` shorthand accepts
these lines together with the existing solid style, color and thickness syntax.
Duplicate lines, `none` combined with another line, overline and non-solid
styles remain rejected. Existing thickness grammar is reused: auto/from-font,
nonnegative bounded px/em/rem lengths and percentages, with minimum 1px.
Color/currentColor and CSS-wide inherit/initial/unset follow the existing
decoration cascade. Ancestor decorations propagate; descendant none does not
remove them, and a new descendant decoration adds another ordered origin.

Strikethrough ignores underline offset and skip-ink. Version 4 records carry
color, thickness, baseline-relative top offset and `line_baseline`; hosts require
`text-solid-strikethroughs-v1` and install after-glyph drawing. Thickness resolution
currently follows the existing underline propagation model: computed lengths
remain fixed, while relative auto/from-font/percentage metrics resolve using
the rendered text font. Broader mixed-font propagation pixels remain pending.

Initial native/WASM parity and native-glyph resize pixels pass. Vector 240px
residuals, broader propagation/composition, tiny/extreme fonts, automatic font
metrics, host transforms and DPR remain qualification work. Earlier A21 progress
notes saying CSS is unsupported describe preceding stages. See
`validation/strikethrough-review.md` for current evidence and limits.

A21 composition evidence now covers combined lines, propagated thickness with
mixed font sizes, multiple origins, fractional explicit breaks and a price card.
Native-glyph pixels pass 26/27 decorated cases; the sole OpenSans 240px residual
also occurs without decoration (control 2/3). All 30 comparisons were visually
inspected. Expanded-corpus native/WASM parity and import checks pass. This does
not qualify the new fixtures on the vector profile or resolve host/DPR limits.
See `validation/strikethrough-review.md`.


A21 follow-up: vector compositions pass 18/30 (all 30 visually inspected).
Native-glyph DPR 1/2/3 controls pass 21/27 with visible text, and 27/27 with
transparent glyphs. All geometry and red-stripe checks pass; six visible-text
mean-channel failures remain at DPR 3. DPR 3 sheets are visually inspected;
DPR 1/2 review remains. Chrome is the sole browser reference. Reproduce with
`validation/strikethrough-dpr-control.mjs`, optionally `--transparent-glyphs`
or `--without-decoration`, using distinct `NUXIE_HTML_REVIEW_DIR` directories.
See `validation/strikethrough-review.md` for evidence and qualification limits.

The A21 undecorated DPR control reproduces the same six mean-channel failures
(21/27, or 7/9 unique viewport/scale combinations). This evidences a text
rendering residual independent of decoration; it remains a qualification gap.

A21 metric checks: 20 size/thickness extremes produce host-valid requirements
and identical native/WASM artifacts. Native-glyph 8px and 1px DPR matrices
each pass 27/27 with partial visual review recorded in the receipt. Giant
font rendering and other fonts/line heights remain unqualified. Reproduce
small-text controls with `NUXIE_STRIKETHROUGH_SIZE` and the DPR script.

A21 host-state controls now include strikethrough and native/WASM parity.
The isolated matrix passes 24/27 after visual review exposed darkened stripe/
glyph overlaps and prompted a targeted opacity-interior check. Half-opacity
fails at all three widths; this retains the P05 group-alpha limitation.
See the strikethrough receipt for commands, partial visual-review scope,
and initial aggregate passes that must not be treated as qualification.


### A22 overflow clipping (active)

`overflow: clip` and `visible` now compile to responsive layout clipping;
inherit/initial/unset are supported. Both axes clip together, including the
existing rounded shape. Text still shapes and wraps at runtime. Hidden/auto/
scroll, axis longhands, two-value overflow, clip margin and ellipsis remain
unsupported. Initial glyph pixels pass 12/12 and are visually reviewed;
expanded corpus native/WASM parity passes. Broader clipping qualification is
pending; see `validation/clipping-review.md`.

A22 regression: 106 Rust tests pass; initial vector pixels pass 6/12, with all
12 visually reviewed and six failures preserved. See the clipping receipt.

A22 composition coverage now includes nested rounded clips, glyph-body cuts,
decorations, fractional edges and a cropped image card, with sibling restore
checks. Added glyph 18/18 and vector 10/18; all 36 visually reviewed. Cumulative
clipping fixtures: glyph 30/30, vector 16/30, all inspected. Expanded native/WASM
parity passes. Fourteen vector failures remain; see the clipping receipt.

A22 DPR matrix: 23/27, all visually reviewed. Four fractional-text pixel
failures remain; geometry and bottom-leak checks pass throughout. A deliberate
missing-clip control fails the leakage gate at all nine width/scale combinations.
Run `validation/clipping-dpr-control.mjs`, optionally `--negative-control` with
a separate review directory. Native/WASM parity passes. See the clipping receipt.

A22 fractional DPR isolation reproduces text failures without clipping (4/9).
Hiding clipped text leaves two sibling-only failures (7/9); hiding all text
passes 9/9. This evidences an independent text-rendering residual and preserves
the original 23/27 visible-text gate. Diagnostic controls and review scope are
recorded in `validation/clipping-review.md`.

A22 update: `overflow:hidden` is now accepted for static initial rendering in
the existing flex/text-block profile. It emits responsive clipping like `clip`;
it does not expose programmatic scrolling. Auto/scroll and axis longhands remain
unsupported. Browser differential controls match clip in 27 DPR cases and 18
oversized-flex alignment cases. See the clipping receipt for qualification.

Static hidden validation: Rust107 and expanded native/WASM parity pass.
Actual hidden fixtures pass glyph30/30 and vector16/30, retaining the same
14 vector failures. Every browser/native image is byte-identical to its
already visually reviewed clip counterpart; identity receipts are retained.
`overflow:hidden` is supported for static initial rendering only; programmatic
scrolling remains outside the module. See `validation/clipping-review.md`.

A22 host-state controls now compile mixed clip/hidden text and compare nine
transforms across three widths. Geometry and native/WASM parity pass. Pixels:
15/27 with restored copy, 19/27 isolated; all isolated cases visually reviewed.
Transformed text residuals remain. Existing glyph regression stays 27/27.
Reproduce with `validation/glyph-state-control.mjs clipping`, optionally
`--no-restore`. See the clipping receipt for limits and artifacts.

A23 ellipsis investigation is active; text-overflow CSS is still rejected.
Runtime-only controls pass 6/12 with all comparisons visually inspected. An
8px-wide box exposes a semantic mismatch (Chrome clips the first character;
Rive clips an ellipsis). Fractional cases also fail pixels. See
`validation/ellipsis-review.md`; the probe flag is diagnostic, not a compiler API.

A23 now has 24 Chrome-only narrow-box boundary controls (all visually
inspected), including ffi and a combining mark. They establish first-grapheme
preservation and expose a ligature reshaping requirement; retaining the first
original glyph alone is insufficient. No additional native/compiler support
is claimed. See `validation/ellipsis-review.md` and reproduce with
`node tools/html-to-riv/validation/ellipsis-narrow-reference.mjs`.

A23's expanded Chrome controls isolate the ligature marker mismatch: painting
an isolated prefix while preserving its original text-range width produces
exact reference pixels. The 24-case run passes its diagnostic assertions and
all cases were visually inspected. This is browser-only evidence, not native
qualification; compiler text-overflow acceptance is unchanged. Receipt:
`validation/ellipsis-review.md`, “Ligature marker placement isolated”.

A23 now includes an experimental runtime truncation planner (5/5 focused Rust
unit tests). It preserves original prefix advances and supplied grapheme
boundaries. Rendering integration, real boundary extraction, native/WASM
transport and CSS acceptance remain pending; see the ellipsis receipt.

A23's runtime boundary extractor now handles complete LTR shaped runs with
Unicode grapheme segmentation. Eight focused tests pass including real Inter
shaping against recorded Chrome first-range metrics; WASM compilation passes.
This does not yet enable renderer integration or CSS text-overflow. Details
and reproducible commands are in `validation/ellipsis-review.md`.

A23's CSS ellipsis experiment is now connected to probe rendering for one
unmodified LTR run/line. Native comparisons pass 18/24, all 24 visually
inspected; 8px A is fixed, while 8px ffi and fractional positions retain
local RGB failures. Flag-off glyph regression passes 27/27. Baseline scene
parity passes, but CSS text-overflow transport/acceptance is still pending.
See `validation/ellipsis-review.md` for flags, commands and remaining scope.

A23 correction: earlier ellipsis runs requested native glyphs but used vector
fallback because the adapter excluded ellipsis overflow. The experimental
CSS path now opts into the adapter; the runner verifies glyph-cache use.
All 24 native-glyph comparisons pass and were visually inspected, including
narrow ffi and fractional positions. Earlier vector failures remain retained.
Flag-off glyph regression stays 27/27. CSS acceptance and broader ellipsis
qualification remain pending; see the ellipsis receipt's adapter correction.

A23 alignment controls pass 27/27 with full visual review. Short-height
controls pass 9/12: height:8px grows to ~8.776px natively. The same failure
occurs with ellipsis disabled; all pixels pass, so this is a retained shared
text-layout geometry limitation. Height 20/39/80 controls pass. Both vertical
runs (12 each) were visually inspected; see the ellipsis/clipping receipts.

Short-height text layout is fixed: line-height spacing now belongs to an
internal text container, preserving authored element padding/heights and
automatic line-box sizing. All12 focused vertical comparisons pass and were
visually reviewed. The full native run passes1102/1105 checks; all1095
browser/native image pairs are byte-identical to retained baselines, with
the same three known240px failures. Full107 module tests pass plus the new
public height/resize regression (108 total); native/WASM corpus parity passes.
See `validation/clipping-review.md` for the precise limits and receipts.

A23 DPR controls pass69/72: only fractional-position text at DPR2 fails local
RGB at all three widths. A clip-only control reproduces those failures (6/9),
so they are not ellipsis-specific. All new DPR2/3 and clip-only comparisons
were visually inspected; DPR1 images are identical to the reviewed baseline.
Geometry, native-glyph usage and baseline scene parity pass. Tolerances remain
unchanged; see `validation/ellipsis-review.md` for reproduction and limits.

A23's experimental LTR preparation now handles multiple shaped runs. The
focused suite passes 9/9, including independent marker size/style and partial
second-run source offsets. Word-spacing controls pass 18/18 with full visual
review; original controls remain 24/24 with all 48 browser/native PNGs
byte-identical to the previously reviewed run. Native and WASM builds pass.
This does not qualify mixed-font/style pixels or block-style transport: CSS
text-overflow remains rejected. See `validation/ellipsis-review.md`,
“Multiple shaped runs and word spacing”, for commands and retained limits.

A23 decoration compositions now pass 27/27 with full Chrome/native visual
review. The initial 12/27 run exposed decoration extending through the
ellipsis marker. Runtime decoration coverage now stops at retained source
glyphs while marker painting remains intact. Failing artifacts are retained;
no thresholds changed. This is DPR1 experimental runtime qualification, not
CSS text-overflow acceptance. See the ellipsis receipt, “Decorations stop
before the marker”.

A23's ellipsis decoration matrix also passes 81/81 across DPR1/2/3. All 54
DPR2/3 comparisons were visually inspected; DPR1 images are byte-identical
to the previously reviewed run. Geometry, glyph-cache use and baseline
native/WASM parity pass without threshold changes. Fractional positioning,
other fonts and host transforms remain separate qualification work. See
`validation/ellipsis-review.md`, “Decoration DPR qualification”.

A23 letter-spacing controls pass 27/27 after removing source letter spacing
from the synthetic marker. Initial +2px narrow cases truncated too early;
visual review also caught a -1px case truncating too late despite its broad
pixel gate passing. Changed images were inspected and unchanged images
verified identical. The initial reproducers remain preserved. See the
ellipsis receipt, “Letter spacing excludes the synthetic marker”.

A23 host-state controls pass 27/27 isolated and 27/27 with a subsequent draw
after state restoration, all visually inspected. Tested DPR1 transforms,
clipping and opacity preserve ellipsis; baseline native/WASM parity passes.
An initial flex-box Chrome fixture was invalid and is explicitly retained as
a harness setup error. See `validation/ellipsis-review.md`, “Host transforms
and state restoration”, for reproduction and limits.

A23 now reserves a checked single-line ellipsis occurrence policy in the
portable requirements API. It requires CSS shaping precision and explicit
host support, and validates unique Text targets. The compiler does not emit
it and the native host does not advertise it yet; text-overflow remains
rejected. See `validation/ellipsis-review.md`, “Reserved occurrence contract”.

A23's native host now installs the checked ellipsis occurrence capability.
It validates nowrap, static LTR content and consistent font metrics before
installation. Native host tests pass 2/2; installed default/word-spacing
controls pass 24/24 and 18/18, with images identical to reviewed diagnostic
controls. The compiler still does not parse or emit text-overflow. See the
ellipsis receipt, “Checked runtime installation”, for exact chronology,
preconditions and remaining validation limits.

## Public single-line ellipsis (current A23 scope)

`text-overflow: clip | ellipsis` is accepted. It is non-inherited by default;
explicit `inherit`, `initial`, `unset`, cascade order and `!important` are tested.
For nonempty text, `ellipsis` requires a text-only `display:block` element,
`white-space:nowrap`, and `overflow:hidden` or `overflow:clip`, without explicit
line breaks or preserved control separators. Strings, two-value forms, `fade`,
wrapping and flex text contexts receive explicit diagnostics. Container-only
values have no effect on descendants unless explicitly inherited.

The compiler retains full source text and responsive layout. It emits the
checked `css-single-line-ellipsis-v1` occurrence policy and
`text-css-single-line-ellipsis-v1` capability, together with CSS shaping precision.
Hosts must install that policy; bare Rive bytes do not supply these semantics.
The native installer validates targets before enabling ellipsis. No diagnostic
runtime flags or browser-baked truncation are needed for this public path.

Qualification is partial: Inter LTR native glyph rendering is covered; fractional
DPR2 clipping residuals, vector rendering, broader fonts and mixed-font/bidi or
multiline ellipsis are not qualified. Existing failing reproducers and thresholds
are preserved. Earlier A23 entries below/above are historical stages; statements
that CSS parsing is pending are superseded by this section. See
`validation/ellipsis-review.md`, “Public CSS compiler path”.

Public CSS validation: compiler112/112, complete accepted native/WASM corpus parity,
public-path24/24 (48/48 images identical to reviewed controls), permanent
corpus15/15 at240/390/768 with all five comparison sheets visually inspected.

Open Sans expansion found a specific optional-ligature ellipsis mismatch (18/24;
all inspected). Narrow clip-only controls pass; the clip-only full matrix is23/24
with an independent long-line raster residual. Runtime correction now retains
original shaped glyphs, with11 focused tests passing; visual requalification is
pending. See the ellipsis receipt's Open Sans investigation.

Original-glyph correction validated: Open Sans24/24; all six changed native
images inspected. Inter24/24 and48/48 PNGs unchanged. Open Sans spacing23/27,
all inspected; four long-line residuals remain, including three untruncated
lines. Runtime WASM check passes. This supersedes the pending correction above;
see the ellipsis receipt for exact test scope and preserved failures.

Open Sans permanent corpus now covers two narrow ligature cases and a responsive
media card:9/9 Chrome/native comparisons inspected at240/390/768, compiler112/112
and complete accepted native/WASM corpus parity pass. The four remaining
Open Sans spacing failures all recur without ellipsis (clip-only22/27); this is
a shared rendering residual whose exact cause remains open. See the ellipsis
receipt and retained overlap evidence.

Retained-glyph composition regression: public Inter decoration81/81 across
DPR1/2/3 and word-spacing18/18, with all198 PNGs identical to reviewed baselines.
Open Sans decorations24/27, all inspected; three untruncated768px lines fail RGB
and decoration gates. See the ellipsis receipt; broader font/rendering
qualification remains partial.

The shared Open Sans drift is now fixed: an empty GPOS table previously enabled
legacy kerning fallback for re/rd. Runtime tests3/3 preserve valid GPOS kerning;
Open Sans spacing27/27, decorations27/27 and permanent corpus48/48 pass with
visual review, plus native/WASM parity and runtime WASM check. This supersedes
the Open Sans residual reports above for these tested cases. See
`validation/opensans-kerning-review.md` for scope and preserved failures.

## Attribute selectors (S01)

Presence and the six standard attribute operators are supported in compounds,
descendant/child selectors and comma lists. Values may be identifiers or quoted
strings, including escaped punctuation, commas and Unicode. Attribute names
must decode to ASCII. Attribute selectors contribute class-level specificity;
existing source order, inline declarations and `!important` remain unchanged.
HTML attribute names match case-insensitively; values follow HTML selector
semantics, with `i` explicitly requesting ASCII-only insensitive matching.
It does not equate É and é. Empty equality matches an empty value; empty prefix,
suffix and substring searches do not match. Missing attributes do not match.

Inert `data-*` metadata with nonempty ASCII names is accepted on existing
container/image elements; it adds no runtime behavior. Other unsupported HTML
attributes remain errors, and `br` retains its separately documented restrictions.
Namespaced attributes/selectors and pseudo-classes outside the supported structural child
and selector-list pseudo-classes documented below remain unsupported. Explicit `s` is deferred and rejected: pinned Chrome153 rejects
`CSS.supports('selector([data-x="a" s])')`. Ordinary case-sensitive matching is
available without this flag. This item remains partial until that reference
dependency is resolved; see `validation/attribute-selectors-review.md`.

## Adjacent sibling selectors (S02)

`A + B` matches B when its immediately preceding element sibling matches A.
Whitespace and comments are ignored; hidden elements still count. Elements in
another parent never match. Chains and combinations with attributes, child and
descendant selectors are accepted. The combinator adds no specificity; normal
selector specificity, source order and !important apply. This operates on the
source DOM before layout; it does not mean the nearest visible box. Mixed
non-whitespace text/element content remains outside the HTML profile.
General siblings (`~`) and the three structural pseudo-classes below are also supported.

## General sibling selectors (S03)

`A ~ B` matches B when any earlier element sibling in the same parent matches A.
It never matches A itself, preceding siblings, or descendants in another parent.
Intervening elements, whitespace and comments do not prevent matching; hidden
source elements participate. Chains and combinations with +, child/descendant
combinators and attributes are supported. The combinator adds no specificity.
Matching is compile-time source-DOM behavior; no runtime interaction is implied.

## Structural child selectors (S04)

`:first-child`, `:last-child` and `:only-child` select positions among all element
siblings of a source parent, regardless of tag or display value. Comments and
whitespace text do not count; hidden elements do. Each pseudo-class contributes
one class-specificity unit, including repeated/combined pseudo-classes.
Names are ASCII-case-insensitive and accept CSS escapes. Attribute, child,
descendant, sibling and selector-list combinations are supported. Function
forms of these three names and pseudo-elements remain rejected. Nth forms are
supported as documented below.
This selects source structure at compilation; it adds no runtime interaction.

## Nth child selectors (S05)

`:nth-child(An+B)` and `:nth-last-child(An+B)` accept integer positions,
`odd`, `even`, and CSS An+B formulas with positive, zero or negative coefficients.
Positions start at one; n starts at zero. Reverse forms count from the last
element. All source element siblings count regardless of tag or display value;
comments and whitespace do not. Matching happens once at compilation, while
layout remains responsive at runtime.

An optional `of <selector-list>` filters siblings before counting. The list can
use the currently supported selector grammar, including combinators, attributes
and nested nth forms. Specificity is one class unit plus the highest specificity
in the filter list, even when that branch does not match. Names and odd/even/of
keywords are ASCII-case-insensitive; CSS tokenization handles escapes/comments.

Resource limits: both coefficients must be within -1,000,000..1,000,000, nested
filter lists are limited to 32 levels, and each selector list to 1024 entries.
Out-of-range formulas, empty or malformed filters, unsupported syntax inside
filters, and unclosed functions are rejected even when no element matches.
`:nth-of-type`, `:nth-last-of-type`, other undocumented pseudo-classes and
pseudo-elements remain excluded. No editor or runtime interaction is added.

## Negation selectors (S06)

`:not(<selector-list>)` matches elements that match none of its arguments.
Arguments accept the supported complex selector grammar, including descendant,
child and sibling combinators, attributes, structural pseudo-classes and nested
negations. Relative selectors beginning with a combinator are rejected.
Negation can appear inside nth `of` filters and vice versa.

Specificity equals the highest argument specificity, even for a branch that
does not match; the function adds no specificity of its own. Thus `:not(*)`
has zero specificity, and `:not(.a,#b)` has ID specificity. Normal source order
and importance still apply. Names accept ASCII case variations and CSS escapes.

Lists are strict: an empty, malformed or unsupported branch rejects the rule,
even when other branches are valid or the rule matches no source elements.
Pseudo-elements and undocumented pseudo-classes remain excluded. Nesting shares
the 32-level limit with nth filters; every list is limited to 1024 entries.
This is compile-time source-DOM selection; no interactive states are introduced.

## Matches-any selectors (S07, S08)

`:is(<selector-list>)` and `:where(<selector-list>)` match any argument. They
accept supported complex selectors, attributes, structural child selectors,
negation, nested matches-any functions and nth `of` filters. Names accept ASCII
case variations and CSS escapes. `:is()` uses the maximum argument specificity,
including unmatched valid arguments, without adding its own class unit.
`:where()` contributes zero specificity regardless of its arguments or nesting.

Within the supported token grammar, lists are forgiving: empty entries and
malformed branches (for example `> div` or `#missing > > div`) are discarded
before computing specificity. Empty/all-invalid lists match nothing. This
behavior is confined to is/where; outer lists, not and nth filters remain strict
unless their invalid syntax is contained by an is/where function.

The compiler still diagnoses tokens and features outside its supported profile,
including unknown/interactive pseudo-classes, namespaces, unsupported attribute
flags and pseudo-elements. It does not silently drop those as a browser might
in a forgiving list. Unclosed functions and resource-limit violations remain
errors. Nesting shares the 32-level bound, each list permits at most 1024
branches including discarded branches, and nth coefficient limits still apply.
Matching is compile-time source selection; layout remains responsive.

## Custom properties and var (S09/S10, in progress)

Current implementation accepts case-sensitive custom property names and inert
token values, including empty values, nested groups and strings. Custom values
cascade with normal specificity, source order, inline precedence and !important,
then resolve before inheritance. initial makes a value guaranteed-invalid;
inherit/unset use the parent's computed value. Revert/revert-layer remain errors.

`var(--name[, fallback])` currently requires a literal custom-property name
(dynamic name substitution is deferred: pinned Chrome153 rejects it) and supports nested/empty fallbacks and substitutions
inside supported color functions and shorthands. Missing and cyclic values use
the fallback; without one the consuming declaration uses unset. Only evaluated fallbacks participate in cycle detection, matching current
Chrome and the editor’s draft. Substituted tokens cannot merge with adjacent
tokens. Values remain responsive CSS lengths, not browser-computed positions.

This is not yet qualified support: nonempty invalid substitution is implemented for the
unambiguous token cases below; other parse failures still produce an error.
Empty final substitutions now
compute as unset, without trying a fallback or restoring an earlier value.
An empty component inside an otherwise valid value is removed normally.
Computed-value invalidation for these cases remains unfinished. Existing initial
values outside the compiler profile still produce diagnostics. Variables do not
enable unsupported render features or properties. Inert custom values may store
those tokens, but consuming unsupported functions/layout remains an error.

Bounds: 512 distinct inherited/local custom properties per element, 64 nested
value groups, and 65,536 serialized bytes per expanded value. Expansion overflow
produces a guaranteed-invalid value so a referencing fallback can recover.
See `validation/custom-properties-progress.md` for current evidence and remaining
work before S09/S10 qualification.

Computed-value invalidation currently recognizes invalid scalar/list token
shapes for width/height/min/max/flex-basis, padding, margin, gap, letter/word
spacing and color/background-color/text-decoration-color. Examples include
nonzero unitless lengths, forbidden negative sizes/padding/gaps, too many length
components, quoted lengths/colors, numeric colors and malformed hex colors.
A declaration with these substituted values behaves as unset. Invalid color/length keywords now reset as well. Valid system colors, intrinsic
sizing, CSS-wide keywords and vendor-prefixed identifiers retain their normal
validation, as do functions, units and other unclassified values;
this is intentionally not a blanket conversion of every compiler error to unset.
Background shorthand has additional CSS grammar and is not treated as a color
longhand by this classifier. S09/S10 remain in progress until the remaining
invalid-value and resource cases are resolved.

Custom dependency evaluation uses an explicit work stack: combining nested inert
values with long property chains does not multiply native/WASM call-stack depth.
Dynamic-name reference evidence is recorded in
`output/playwright/html-to-riv/custom-properties-names/direct-chromium-names.json`;
literal-name var resolves in Chrome153 while nested-name forms are rejected.

Keyword classification follows CSS Color 4 named/system/deprecated color
keywords and CSS Sizing 3/4 sizing keywords. An unknown non-vendor color
identifier is invalid; the same applies to identifiers outside the relevant
length property's grammar (for example padding:auto or width:none). These rules
apply only to substituted values. Unsupported system colors and intrinsic sizes
are still diagnosed, not converted to unset. Function grammar and other
property families remain part of the unfinished S09/S10 invalidation work.

Substituted scalar rgb/rgba/hsl/hsla functions now apply unset when their
channel count, separators, units or legacy channel types are invalid. This
includes pure-color background shorthand. Valid clamped channels keep their
normal behavior. Relative colors, none components, nested calculations and
other function families retain profile diagnostics; they are not classified
as invalid merely because the compiler does not render them.

Invalid substituted enum values for flex-direction, flex-wrap, overflow and
text-overflow now use unset. Known reverse-direction/wrapping, scrolling,
two-value overflow, fade and text-overflow string syntax remain profile
diagnostics where unsupported. CSS-wide keywords remain distinct from invalid
keywords. Other property families still require invalidation qualification.

Substituted flex-grow and flex-shrink use CSS nonnegative-number grammar for
invalidation. Rendering accepts 0 or factors in [1,10000], including fractions
such as2.5; positive factors below 1 remain explicitly deferred. Negative numbers, dimensions, percentages, identifiers,
strings and multiple tokens become unset (grow0, shrink1); an earlier declaration
is not restored. CSS-wide keywords retain their existing semantics. Functions
outside the accepted profile retain diagnostics. The existing format restriction
requiring equal resolved grow/shrink factors still applies, including after an
invalid-value reset; independent factors are not newly supported.

Substituted flex shorthand now resets all grow/shrink/basis components for
malformed flat-token grammar: negative factors, multiple bases, interleaved
basis/factors, excess components and invalid keywords/strings. Later longhands
and important declarations retain cascade precedence. A present custom property
whose substitution is invalid does not use its var() fallback. Valid excluded
basis functions/units, intrinsic sizes and factors below 1 retain diagnostics.

Full harness regression completed:1,341/1,342 checks pass. All1,332 scene
comparisons are source- and image-identical to prior reviewed versions. The
existing em-layout-cascade240 fractional-edge failure remains (exact geometry);
A09 is still partial. See validation/stylesheet-preservation-review.md.

Invalid substituted align-items and justify-content values now reset to their
initial flex-layout behavior (stretch and start); invalid text-align values
inherit the parent alignment. Flat keyword grammar includes safe/unsafe position
pairs and first/last baseline recognition so valid excluded forms retain an
explicit diagnostic. Anchor/dialog/vendor/string/function forms outside the
profile are preserved as diagnostics; no new alignment modes are enabled.

Invalid substituted border-radius flat values now reset to square corners:
negative values, nonzero unitless numbers, invalid identifiers/strings, excess
components, commas, empty slash groups or repeated slashes. Existing supported
single radii still render normally. Valid percentage, elliptical and per-corner
radius forms remain excluded with diagnostics, as do unimplemented functions
and units; this change does not enable those rendering modes.

Invalid substituted font-size, font-weight and line-height flat values now
inherit, including negative dimensions/percentages, incorrect scalar types,
invalid keywords and extra components. CSS weight grammar is1..1000, distinct
from the supported integer100..900 rendering range. Valid excluded zero size/
line-height, fractional or outer-range weights, relative size/weight keywords,
percentages and functions keep profile diagnostics rather than inherit.
Font-family and font shorthand invalidation still need further qualification.

Malformed substituted font-family lists now inherit: empty comma entries,
mixed quoted/unquoted names, incorrect scalar types, invalid punctuation and
standalone reserved default/CSS-wide identifiers in list entries. Unknown valid
families and quoted reserved names keep missing-font/profile diagnostics;
valid fallback lists remain excluded. Generic/system keyword combinations
and font shorthand invalidation remain under audit.

Font-family keyword classification follows pinned Chrome evidence: leading
generic keywords cannot be followed by more family-name identifiers; trailing
generic/reserved words inside multi-word names retain profile diagnostics.
Leading CSS-wide words with trailing tokens in substituted font-family values
now inherit, matching Chrome computed behavior. Sole CSS-wide values retain
their normal semantics; quoted names and trailing keywords are not invalidated.

Font shorthand variable substitution is now explicitly validated with later
font-size/line-height overrides and important/specificity ordering. Missing or
empty substitutions inherit font components; later longhands still override
them, and em line-height resolves against the final font size. This validates
existing behavior; malformed nonempty font shorthand grammar remains pending.

For substituted font shorthand with supported normal/weight prefixes, missing
size/family, negative size or line-height, duplicate weights/excess normal slots,
and malformed family lists now inherit font components. Later longhands retain
precedence. Styled/variant/width/system/keyword prefix forms and functions retain
profile diagnostics; their malformed combinations are not exhaustively classified.
Valid zero/percentage/fractional values outside rendering support still diagnose.

Unknown substituted font shorthand prefixes now invalidate and inherit; system
font keywords are only standalone values and invalidate when combined with
extra components. Recognized style/variant/width/size keywords and standalone
system fonts retain profile diagnostics. Their malformed combinations remain
under audit; this does not enable styled or system-font rendering.

Invalid substituted text-transform values now inherit the parent's casing:
wrong scalar types, unknown keywords, duplicate/conflicting case transforms,
repeated width/kana modifiers, and none/math-auto combined with other keywords.
Valid width/kana/math transforms and combinations retain profile diagnostics;
this does not add those rendering modes.

Invalid substituted white-space values now inherit the supported parent mode:
unknown/types, combined legacy modes, duplicate/conflicting collapse or wrap
keywords, and incompatible/repeated trim keywords. Valid compound modes and
trimming retain profile diagnostics. Rendering support remains normal, nowrap,
pre, pre-wrap and pre-line; this grammar handling adds no new rendering mode.

Invalid substituted text-decoration-style values reset to solid; invalid
text-decoration-skip-ink values inherit. Scalar types, unknown keywords and
multiple values are checked. Valid non-solid styles retain diagnostics;
existing auto/none/all skip-ink support is preserved. Decoration line and metrics are qualified below; position and shorthand
grammar still need invalid-substitution qualification.

Invalid substituted text-decoration-line values now reset local decoration to
none: unknown/types, duplicates and none/error-annotation keywords combined with
other values. Ancestor decoration propagation remains intact. Valid overline,
blink and error annotations retain profile diagnostics. Line grammar does not
expand the supported underline/line-through rendering modes.

Invalid substituted decoration thickness resets to auto; invalid underline offset
inherits. This covers unknown scalar keywords, strings, nonzero unitless numbers,
and multiple values. From-font is valid only for thickness. Signed offset values
remain supported. Negative thickness is valid CSS but remains an explicit profile
exclusion, as do unimplemented units/functions; it does not silently reset.

Invalid substituted text-underline-position scalar types/unknown keywords,
combined auto, and duplicate/conflicting under/from-font or left/right groups
now resolve to auto in the supported profile. Valid under/from-font/left/right
and combinations remain explicit rendering exclusions. Only auto parents are
representable, so this does not qualify inheritance from other position modes.

Text-decoration var() declarations now pass property preflight and substitute
before shorthand expansion. Flat malformed shorthand values reset line, style,
color and thickness together; they preserve underline offset/skip-ink and respect
later longhands and importance. Duplicate color/style/thickness, invalid line
groups, wrong scalar types and unknown keywords are classified. Valid excluded
styles/lines/metric units remain diagnostic. Malformed function-containing
shorthands remain under audit; this is not full CSS shorthand qualification.

Decoration shorthand invalid-substitution checking now includes scalar RGB/RGBA/
HSL/HSLA color functions. Malformed functions and duplicate color components
reset the shorthand, as do other conflicting components alongside valid color
functions. Actual color emission remains unchanged. Valid modern numeric HSL is
preserved. Relative colors, none channels, nested math and wide-gamut functions
remain explicit profile exclusions; their full grammar is not classified here.

Decoration missing/empty substitutions and CSS-wide fallbacks are now explicitly
validated across the shorthand and seven longhands. Present-empty custom values
invalidate the ordinary declaration without using its fallback. Initial/inherit/
unset in the ordinary fallback follow the decoration property's rules; those
keywords assigned directly to a custom property instead affect custom-value
inheritance/invalidation. Local reset preserves propagated ancestor underlines.
Offset and skip-ink are not reset by the decoration shorthand. Underline position
remains auto-only; this validation does not expand the rendering profile.

Simple substituted background grammar now invalidates unknown scalar keywords,
strings, malformed hex, duplicate colors/none, empty simple layers and a color
before a layer comma. Invalid values reset to transparent; they do not use the
var fallback or restore prior paint. Valid none+color and multiple-layer forms
remain profile diagnostics. Position, sizing, repeat, attachment, box and image
syntax remains outside this simple classifier; no new background rendering mode
is enabled. Later background-color declarations still override the reset.

Background substitution validation now includes scalar RGB/RGBA/HSL/HSLA in
simple shorthand combinations. Malformed functions, duplicate color components
and a color before a layer comma reset to transparent. Valid pure function colors
retain their exact output. Valid none+color/layers/positions and unimplemented
image/color functions still diagnose; full background grammar remains pending.

Realistic light/dark custom-property compositions are semantically verified
against explicit-value CSS. They combine inherited aliases, local shadowing,
font/decoration shorthand, HSL, empty fallback suppression and responsive flex.
Native visual qualification now passes all three widths after fractional-Y
underline snapping at paint time; original failures remain preserved. Vector
rendering remains unqualified for these compositions. This does not expand support for
line heights below natural font metrics; the original1.2 failure is retained.

Underlines with unit-scale axis-aligned CSS placement snap vertically after scene
translation and before host scaling/DPR. Bounds are retained for other transforms;
scaled/affine world transforms are not newly qualified. Both full-stripe and
geometric fallback drawing paths use the adjusted bounds. This fixes fractional
container placement without changing layout, authored geometry or tolerances.

CSS layout paint-bound snapping uses a serialized version-5 runtime contract:
`layout-css-pixel-bounds-v1` plus unique `layout_pixel_bounds` object IDs. The
compiler opts painted and clipped layouts into this policy. Hosts must validate
capabilities and actual text/layout object targets before installing policies.
The diagnostic enable switch has been removed; the probe uses this normal path.
Plain Rive imports retain their original behavior.

At unit scale with axis-aligned placement, paint and clipping bounds snap after
scene translation and before host DPR scaling. Logical layout and authored
fractional dimensions remain unchanged. Cloning preserves the policy; disabling
it restores the original paint bounds independently of the source object.
Other world transforms retain unsnapped geometry and are not newly qualified.

Targeted native comparisons pass 117/117. Separate underline and layout paint
DPR matrices each pass 27/27. The layout matrix covers fractional-origin
rectangles, rounded backgrounds and rounded overflow clips at widths 240/390/768
and DPR 1/2/3, resizing a single compiled scene. Small corner antialiasing
differences remain within existing limits. Full native regression passes 1477/1477 checks, with all 1467 scene
comparisons visually accounted for (241 changed pairs inspected; 1226 pairs
identical to reviewed baselines). A09 remains partial for vector text fidelity. Fractional DPR, affine transforms and
broader vector/backend fidelity are not established by these checks.
Validation details and preserved original failures: `validation/em-edge-paint-review.md`.

Focused vector em/rem/background checks pass 31/33, all visually inspected.
The original fractional shape-edge case now passes. Two 240px text cases retain
their historical mean-channel errors above the existing limit: nested em
typography and final-font-size shorthand. Native glyph output passes these
cases; full vector text fidelity remains unfinished.

S09/S10 non-length dimension increment implemented: angle/time/frequency/
resolution/flex units invalidate substituted length slots; valid excluded lengths
retain diagnostics. Chrome182 references and two public Rust regressions pass.
Native/WASM corpus parity and six new native visual comparisons now pass; both
sheets inspected. All228 surrounding custom-property comparisons pass and match reviewed images. Receipt:
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
completion of the compiler backlog.

L02 `order` is qualified for the documented native rendering profile. Accepted values are
signed CSS integers (finite oversized tokens clamp to signed32 endpoints), plus
existing CSS-wide resets/inheritance and var() handling. Decimal/exponent syntax,
units, lists and integer-producing math functions are diagnosed as unsupported.
Initial order is0 and it is not implicitly inherited. Siblings use ascending order
with authored order for ties; original IDs, paths and selector matching are kept.
Generated object indices address the particular artifact.

Scenes with sibling layout boxes declare `layout-css-paint-order-v1`, including
scenes without explicit order. Hosts must accept the capability and apply
Artboard::set_css_paint_order(true) to every imported/cloned instance before draw.
This reorders complete paint groups while keeping runtime layout child order.
The capability has no per-object payload and works with requirements versions1–5.
Raw Rive defaults stay off. No grid, general block flow, positioning, navigation,
interaction, binding, script or animation support is introduced.

Evidence: public205-test suite, host4 and native/WASM parity9 pass. Corrected
focused native45/45 and full native1624/1624 pass; all1614 full-run image pairs
match reviewed baselines, with no outstanding visual inspection. Each scene is
compiled at390 and resized to240/390/768. The direction-aware correction resolves
the earlier reverse-overflow regression; original failed artifacts are retained.
Focused vector42/45 passes with geometry45/45; three text-card glyph raster
failures remain explicit limitations of that rendering profile. This qualification
does not claim complete vector-text pixel equivalence. Exact receipts:
validation/order-review.md and order-direction-full/visual-inspection.json.

L03 `align-self` is qualified for the native glyph profile. Accepted values:
auto, flex-start, center, flex-end, stretch and baseline, plus existing CSS-wide
and var() behavior. The property is not implicitly inherited; auto resolves the
parent alignment. Non-auto values require version6 typed layout targets and
`layout-css-align-self-v1` host support. Modern modifiers and other keywords
remain intentionally unsupported.

Public210 tests plus the added lifecycle regression pass; the focused runtime
suite is4/4 including432 Chromium box comparisons. Host5/5, native/WASM
parity9/9 and TypeScript checks pass. Full native1723/1723 and new compositions9/9
pass, with all image pairs visually accounted for. The27 text-baseline cases
include wrapping, leading, padding, images and nested intrinsic-width text.

Vector geometry108/108 passes (maximum error0.013671875px), but pixels pass78/108.
Thirty text raster failures remain explicit limitations of that profile. All108
pairs are reviewed; native glyph pixels pass for the same source scenes. No
tolerances changed. Exact logs, receipts and preserved reproducers are in
validation/align-self-investigation.md.

L04 `align-content` is qualified for the native glyph profile.
Accepted: flex-start, center, flex-end, stretch, space-between and space-around.
Omission uses the authoring reset's flex-start; initial/unset use stretch.
Explicit inherit and var() rules apply; the property is not implicitly inherited.
Other keywords and modifiers remain intentionally unsupported. Version7 and
`layout-css-align-content-v1` carry strict per-layout targets. Every wrapped
container receives an independent line policy, separating line placement from
item alignment and justification.

Public216/216 tests, host6/6, types and final
corpus native/WASM parity9/9 pass. Corrected mixed-size native297/297 comparisons
pass and all images are reviewed, including overflow and realistic compositions.
The oracle asserts intended mixed sizes to prevent a discovered selector
specificity mistake from returning; obsolete equal-size evidence is retained.
Vector geometry297/297 passes (maximum error0.009376526px), while pixels pass
290/297. Seven composition text raster failures remain explicitly open; all
vector pairs are visually accounted for. Full native regression2029/2029 passes;
all2019 image pairs have exact source/browser/native hash matches to reviewed
baselines. See validation/align-content-investigation.md for logs and receipts.

L05 distributed spacing is qualified for the native glyph profile. New values:
justify-content space-around/space-evenly and align-content space-evenly.
Version8 and layout-css-distributed-spacing-v1 gate host policies; original
CSS now passes288 Chromium geometry comparisons plus lifecycle tests. Typed
API checks, public220/220, host7/7 and native/WASM parity9/9 pass. Native297/297
pixel comparisons pass and all images are reviewed. Vector geometry297/297
passes, but8 text composition pixel failures remain open and reviewed. Full
native regression2326/2326 passes; all2316 image pairs match reviewed baselines. Existing exclusions
remain in place. See validation/distributed-spacing-investigation.md.

L06 wrap-reverse is implemented, with pixel qualification pending. It uses
Rive's existing wrap value2 and the existing independent line-alignment policy.
Public parser tests and336 Chromium reference layouts on original and cloned
scenes pass. Reversed stretch/space-between overflow fallback is corrected;
all98 isolated layout-engine tests pass. Public223/223 and host8/8 pass; native
and WASM builds pass. Expanded parity9/9 passes.
Native348+3 pixel comparisons pass, including an asserted multiline baseline.
All15 compositions and336/336 box cases reviewed; full
regression remains pending.
Vector geometry351/351 passes;14 text pixel failures are preserved and visually reviewed. See validation/wrap-reverse-investigation.md.

L06 vector review completed: all15 composition comparisons inspected; all336 box pairs have exact source and browser/native PNG identity with reviewed native cases. Geometry351/351 passes (maximum0.015625px); vector pixels337/351 pass. The14 text pixel failures remain explicit renderer limitations with unchanged thresholds. Full native qualification remains pending.

L07 auto margins: source implementation and tests are prepared but unqualified. Accepted syntax targets one-to-four `auto` or existing nonnegative lengths in margin shorthand, plus physical longhands and existing cascade rules. Padding auto, negative and percentage margins remain excluded. The runtime test corpus includes132 scenes/396 viewport references including root auto margins;136 visual fixtures/408 widths include four realistic text compositions. Builds and test execution await completion of the L06 regression to preserve its executable inputs.

L06 final native qualification: full2677/2677 passed (session47222 exit0, `/tmp/wrap-reverse-full.log`). All2667 image pairs match reviewed baselines in source HTML/CSS and both PNG hashes; the full visual receipt has no remaining inspections. Execution binaries, WASM and case corpus verified unchanged through completion before any L07 rebuild. Native wrap-reverse is qualified; vector geometry passes but14 reviewed text pixel failures remain explicit limitations.

L07 geometry milestone: both auto-margin layout-engine fixes pass396 Chromium reference viewports on original and cloned scenes; all100 standalone Taffy tests pass. The136 visual fixtures are merged. Full public/build/parity/pixel qualification remains pending; auto margins are not yet native-qualified.

L07 public validation:226/226 compiler tests pass, native publisher/probe and WASM builds pass. Focused408 native pixel comparisons, parity and host checks are running; visual qualification remains pending.

L07 focused native408/408 passes, parity9/9 and host9/9 pass. All12 native composition pairs visually reviewed. The396 box cases still need review; vector and full-native evidence remain pending. Tolerances unchanged.

L07 native visual review complete: all408 comparisons reviewed. Vector geometry408/408 passes (maximum0.0107421875px); six composition pixel failures await detailed visual review. Full native regression remains active; qualification pending.

L07 vector visual review complete: all408 pairs accounted for;402 pixel passes and6 preserved text raster failures. Auto-margin geometry and wrapping agree with Chromium. Full native regression remains pending.

L07 final native qualification: full regression15587 exited1 with3084 passes and one ENOSPC screenshot write failure (`/tmp/auto-margin-full.log`). The zero-byte Chromium image and failed run remain intact. Isolated same-input retry47545 passes1/1 (`/tmp/auto-margin-capture-retry.log`). All3074 remaining image pairs have exact source and both PNG identity with reviewed baselines; the one retry pair also matches its reviewed baseline exactly. All five input hashes verified unchanged through retry. Combined coverage3085/3085 is qualified for native; full receipt remaining[]. This is a full run plus one evidenced infrastructure retry, not a clean full-run exit. Six reviewed vector text pixel failures remain.

L08 executed milestone: public232/232 passes (59786, `/tmp/independent-flex-module3.log`), including288 Chromium geometry references on original/clone and same-viewport mutation/clear lifecycle. Native and WASM builds pass, parity9/9 (9452), host10/10 and typecheck pass. Focused native297/297 passes (51095, `/tmp/independent-flex-native.log`). All nine composition comparisons and72 unique box pairs directly inspected across12 sheets;216 exact box duplicates account for all288 box cases. Native visual receipt remaining[]. Tolerances unchanged. Vector297 run72737 active (`/tmp/independent-flex-vector.log`); full regression pending, so L08 remains unqualified overall.

L08 vector run72737 exited1:296/297 pixels pass; narrow panels body interior RGB error8.6835 exceeds unchanged6 threshold. Geometry independently audited297/297, max0.028076171875px; all source IDs and hidden states agree. All nine composition pairs directly reviewed (glyph raster differences), and288 box pairs match fully reviewed native sources and both PNG hashes exactly. Vector visual receipt remaining[]; pixel failure retained. Full native3382 regression20764 is running (`/tmp/independent-flex-full.log`). All five input hashes verified and recorded; do not rebuild or modify the case corpus until terminal.

L09 partial factors remain rejected by the compiler. Isolated layout-engine fix removes duplicate gap subtraction;432 Chromium reference viewports and all101 engine tests pass. This is not compiler or native-renderer qualification. L08 full regression continues with recorded inputs unchanged.

L09 evidence expanded to240 scenes/720 Chromium references; all101 isolated engine tests pass, including min/max constraints and mixed sibling factors/bases. Nine browser-only composition references directly reviewed, including deliberate partial-shrink overflow.244 prospective pixel fixtures and public original/clone test are prepared separately. Compiler admission and native/WASM/rendering execution remain pending L08 full regression.

L09 source candidate accepts finite factors in[0,10000]; sub-unit factors require version10 and layout-css-partial-flex-factors-v1 plus explicit factor payloads. Missing capability, wrong versions and inconsistent payloads reject before drawing. Equal and unequal partial factors are covered. Content-derived auto basis and indefinite percentage basis remain excluded. Source compile and type checks pass; executed public/native/WASM/pixel validation awaits completion of the unchanged L08 regression. This supersedes earlier source-admission notes, without claiming qualification.

Validation correction: L08 full20764 was invalidated and stopped after cargo rustc --test also rebuilt the publisher; post-build hash verification caught the change. The stopped run cannot qualify L08. Frozen executable copying and browser-harness hash verification are now implemented for the restart. L09 cascade/contract tests and720 public original/clone Chromium references pass in directly executed test artifacts. Full public suite9542 and coherent native/WASM builds are running; neither feature is fully qualified yet.

L09 public suite9542 passes236/236 (`/tmp/partial-flex-module.log`). After adding244 pixel fixtures to the main corpus, the two tests that embed the corpus were rerun (55847 exit0, 96 tests; `/tmp/partial-flex-expanded-contract.log`). Native/WASM parity44630 passes9/9 across the expanded corpus. Frozen snapshot smoke84609 passes21/21; all21 source/browser/native image pairs match reviewed baselines exactly and receipt remaining[]. Negative checks confirm hash mismatch aborts before test execution and snapshot overwrite is rejected. Focused native732 run52606 active (`/tmp/partial-flex-native.log`), explicitly using partial-flex-toolchain. Full and vector validation remain pending.

L09 interim native visual review: first90 passed box comparisons captured from completed per-case artifacts while52606 continues. All61 distinct pairs inspected across11 contact sheets;29 exact duplicates covered by PNG hashes. Batch receipt remaining[] applies only to these90 cases, not the complete732 run. See partial-flex-native-review-batch1/visual-inspection.json and partial-flex-box-review-batch1/manifest.json. Remaining images, text compositions, vector and full regression still pending.

L09 interim visual coverage now360/732. Batch2 covers cases91..180:16 new unique pairs inspected,10 within-batch duplicates and64 exact pair matches to reviewed batch1. Batch3 covers all180 reverse-row cases:77 unique pairs inspected across13 sheets,103 exact duplicates. Receipts for each batch have no remaining inspections within their stated scope. Column cases, compositions and final run status remain pending;52606 still active.

L09 focused native52606 passes732/732 (7.5m, `/tmp/partial-flex-native.log`). Column batch4 completed:90 unique pairs inspected across15 sheets,90 exact duplicates, covering180 source cases. All nine text composition comparisons directly reviewed: unused free space, panel sizing and intentional narrow viewport overflow agree with Chromium. Native overall receipt tracks549 reviewed and183 remaining (reverse-column boxes plus original two-child reproducer). Batch5 sheets generated (81 unique pairs/14 sheets), none inspected yet. Vector732 run20863 active (`/tmp/partial-flex-vector.log`), using the same frozen toolchain. Full regression restart remains pending.

L09 partial flex validation update: all732 focused native comparisons pass and are visually reviewed. Vector geometry passes all732, with one preserved narrow-panel text raster failure (731 pixel passes). Full4114 native regression is running against a frozen coherent toolchain; final qualification remains pending. See validation/partial-flex-investigation.md.

L10 investigation started:33 current compiler rejections preserved and99 Chromium content-auto reference viewports captured. Content-derived auto basis remains unsupported; no qualification claimed. See validation/content-auto-investigation.md.

L10 working source is now experimental: intrinsic dimension/factor transport passes48 nested-box original/clone reference viewports. Initial full99 geometry comparisons expose21 row-text failures, preserved in content-auto-experiment-initial. Text measurement experiment and broader qualification remain pending; this is not supported-profile qualification.

L10 implementation remains experimental. All327 current intrinsic-basis geometry references pass, including unbreakable-word overflow; native/WASM and pixel qualification remain outstanding. Working manifests require version11, layout-css-intrinsic-sizing-v1 and a unique layout_intrinsic_sizing target list when output depends on corrected intrinsic sizing. Older hosts must reject this requirement. See validation/content-auto-investigation.md.

Joint L08/L09 full native94020 passes4114/4114 (38.7m). All4104 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed auto-margin, independent-flex and partial-flex baselines; no images remain unreviewed. Frozen toolchain hashes revalidated. Receipt: partial-flex-full/visual-inspection.json and baseline-comparison.json. L08 and L09 are now native-qualified; their documented vector text raster failures remain. The invalidated earlier L08 run is retained and not used as qualification.

L10 intrinsic sizing remains experimental. Working-source alignment fix preserves intrinsic measurement while stretching internal text containers to the resolved content width; six saved-reference alignment pixel comparisons and54 additional alignment stress geometry references pass. Full corrected native/vector pixel validation is still required; no expanded support qualification is claimed.

L11 qualification note: indefinite percentage bases remain intentionally guarded in the compiler. Experimental runtime geometry and thin-box paint comparisons do not expand the accepted syntax. The paint candidate retains a pixel for a collapsed positive extent above four1/64px units, matching Chromium snapping at tested device scale1; it leaves logical geometry unchanged and applies only to the existing translation-only CSS paint-bounds path. Other transform/device-scale behavior is not established by this experiment.

L10 native qualification completed against immutable content-auto-alignment-toolchain:4504/4504 full tests pass; all4494 scene pairs have exact HTML/CSS and browser/native PNG identity with reviewed baselines (4104 from partial-flex-full,390 from corrected native336 plus alignment54). Full visual-inspection.json records all transfers and zero remaining pairs. Prior public228, host12 and native/WASM parity9 passes support the version11 intrinsic-sizing contract. Native content-derived auto basis is qualified;97 focused vector-text pixel failures remain preserved and reviewed. This receipt qualifies the frozen L10 build; subsequent experimental L11 runtime/compiler changes require their own regression validation.

L11 remains unqualified and guarded. A version12 capability contract is proposed in validation/indefinite-basis-contract.md; it is not yet implemented or advertised as accepted syntax. Experimental percentage-basis encoding now preserves authored main dimensions through the independent-factor payload even for equal/zero factors. Public regression and360 geometry comparisons pass; native/WASM compatibility and pixel requalification of this encoding change remain required.

Version12 contract foundation now validates layout-css-indefinite-basis-v1 and preserves intrinsic-target checks when combined with earlier capabilities. Checked-host tests13/13 and TypeScript pass. This is host/manifest infrastructure only: the compiler still guards indefinite percentage bases, and neither source admission nor native/WASM qualification is claimed. Factor native360/360 pixel comparisons pass; direct visual review currently covers9/360 pairs.

L11 source admission candidate now emits version12 and layout-css-indefinite-basis-v1 for previously guarded indefinite percentage sizing. Runtime version never downgrades for intrinsic text descendants. Historical overflow fixture left active rejections but is preserved verbatim in indefinite-basis-cases.json and a positive public regression. Focused admission49/49 tests pass; actual compiled-scene checked host13/13 passes; native/WASM parity9/9 passes across previous plus349 L11/threshold scenes. Original/clone lifecycle70951 passes120 scenes through240/390/768/240 (960 instance/viewport comparisons). Frozen four-artifact indefinite-v12-toolchain created. Main corpus now1835 scenes; full native5551 regression48669 running in indefinite-v12-full, and final public suite48691 running. Keep cases.json unchanged during this full run. Admission is implemented but L11 is not qualified: full regression, complete visual accounting and residual vector failures still need resolution/reporting.

L11 current candidate passes233 public tests and all288 native text pixel comparisons; all288 native text images now have recorded direct/equivalent-image review. The separate36 auto-basis/percentage-main comparisons pass geometry/native pixels and still need visual qualification. Full5551 regression and remaining box/factor/vector review remain outstanding. Version12 vector text replay is being rerun against the final immutable toolchain; earlier88 vector failures remain recorded until revalidated.

L11 visual review update: indefinite-factor-native now accounts for184/360 full image pairs:54 directly inspected (18 scenes at240/390/768) and130 exact browser/native PNG-pair transfers to directly inspected representatives. The176 remaining pairs are explicit in visual-inspection.json. Column/reverse-column percentage overflow and row viewport clipping match Chromium in these inspected scenes; all360 independent geometry/pixel gates already pass. Cross-corpus receipts are separate and their counts must not be added to this receipt. Final version12 vector text replay is terminal:200/288 pixel passes and88 preserved failures, with288/288 geometry passes. Its288 source/browser/native image pairs exactly match the earlier vector replay, but only three have an inspected baseline;285 remain visually unreviewed. Evidence: indefinite-v12-text-vector/baseline-comparison.json and visual-inspection.json. Full5551 native regression remains live; no qualification or tolerance change is claimed.

L11 factor visual review is complete for the focused native replay:360/360 image pairs accounted for, comprising153 directly inspected pairs (51 three-viewport sheets),198 identical-image transfers within the factor corpus and9 transfers to directly reviewed original-basis representatives. Every transfer matches both complete PNG hashes; all720 candidate PNGs,51 contact sheets and18 external representative PNGs were rehashed against receipts. Each of360 candidates independently passes geometry and pixel gates. The receipt is indefinite-factor-native/visual-inspection.json with zero remaining. These are visual-output transfers, not claims of source equivalence. Main frozen version12 full regression is still running; expanded/thin/auto-main and vector visual review remain outstanding. L11 remains unqualified, and no tolerance changed.

L11 implicit percentage-main path: all36 comparisons in indefinite-auto-main-native now directly visually inspected (12 sheets,240/390/768), with all72 source PNGs and12 sheets rehashed. Empty, nested-box and text fixtures cover all four flex directions. Responsive line wrapping, padding, intrinsic extent and sibling placement match Chromium. All36 geometry/native pixel gates pass on the final version12 toolchain. Added the12 fixtures to JavaScript native/WASM artifact parity corpus, alongside the unchanged1835 main scenes; that parity rerun is pending. Main cases.json stays frozen during full5551 regression. This focused evidence does not complete L11 qualification.

L11 follow-up: JavaScript parity rerun94023 passes9/9 tests, including all1847 positive scene inputs (1835 main plus12 implicit percentage-main) with identical native/WASM RIV bytes, source maps and runtime requirements. Log: /tmp/indefinite-auto-main-parity.log. Original-basis native visual coverage is now complete75/75 in indefinite-thin-original-native/cross-corpus-visual-inspection.json:12 directly reviewed reversed-column pairs plus63 complete-image transfers to directly inspected baselines, with hashes checked from files. Expanded native full-frame review remains15/216; threshold native0/108. These counters do not replace prior regional thin-box inspections. Remaining review and full regression still prevent L11 qualification.

L11 thin-pixel threshold visual review complete: native108/108, with15 directly inspected representative pairs and93 exact whole-image transfers. Five sheets include the full frame plus nearest-neighbor8x details of the top-left16x16 region at240/390/768. Invisible zero/subthreshold boxes and one-pixel horizontal/vertical bars at5/64px, including origin0.5 rounding, match Chrome. All216 native PNG files verified against hashes; all108 independently pass geometry/pixels. Vector108/108 has exact HTML/CSS and complete browser/native image identity with this fully reviewed native corpus, verified from files; its visual receipt has zero remaining. Evidence: thin-pixel-native-v2/visual-inspection.json and thin-pixel-vector/visual-inspection.json. Expanded boxes and vector text review plus the ongoing full regression remain outstanding; no tolerance or support scope changed.

L11 full native regression48669 is terminal and passes5551/5551 in25.8minutes: expected5551, skipped0, unexpected0, flaky0, errors0. Saved authoritative Playwright JSON, terminal log, unchanged1835-scene source corpus and SHA256 terminal-summary.json in indefinite-v12-full. This replaces earlier running status. Full visual qualification remains pending. Expanded native review has27/216 pairs accounted for in visual-inspection.json (18 direct plus9 exact-image transfers); the earlier15-pair cross-corpus receipt overlaps and must not be summed. First six row min/max/root-max/nested-percent/zero-pixel scenes directly inspected across240/390/768 and match Chrome. No gates changed; L11 remains unqualified pending remaining visual accounting and explicit vector limitations.

L11 expanded native visual review now accounts for111/216 full-frame pairs: 63 direct and48 exact-image transfers; 105 remain. Reviewed forward/reverse nested percentages, zero-pixel/zero-percent controls, min/max constraints and column wrapping agree with Chrome across240/390/768. Rehashed all222 reviewed candidate PNGs and21 contact sheets. Receipt: indefinite-thin-expanded-native-v2/visual-inspection.json. Cross-corpus and regional receipts overlap and are not additive. Full5551 native tests remain passed; L11 visual qualification remains incomplete.

L11 expanded-box visual review complete216/216:117 directly inspected pairs (39 sheets),89 complete-image transfers within the expanded corpus and10 to directly reviewed original/factor baselines. All432 candidate PNGs,39 contact sheets and20 external representative PNGs rehashed; every candidate independently passes geometry/pixels. Reverse constraints, wrapping, nested percentages, zero-basis distinctions and shrink edges match Chromium in the inspected frames. Vector216/216 has exact HTML/CSS and browser/native PNG identity with the fully reviewed native corpus; its files were also rehashed and its gates pass. Receipts: indefinite-thin-expanded-native-v2/visual-inspection.json and indefinite-thin-expanded-vector/visual-inspection.json, both with zero remaining. Full5551 numeric regression remains passed. Remaining work includes final full-run image accounting and incomplete vector-text review;88 vector text pixel failures remain preserved and unwaived. L11 is not yet qualified.

L11 final native full-run visual accounting complete:5541/5541 scene pairs exactly match reviewed baselines in HTML/CSS and both complete PNG files. Rehashed candidate and baseline images; source receipts are explicitly pinned by SHA256. All5551 tests pass, including10 controls; zero skipped/flaky/unexpected/errors. Evidence: indefinite-v12-full/baseline-comparison.json, visual-inspection.json, terminal-summary.json and compare-baselines.py. There are no remaining native full-run image changes requiring inspection. Focused implicit-percentage-main36 is separately reviewed and included in1847-scene parity. Vector-text review remains incomplete and its88 pixel failures remain unwaived; overall L11 status remains qualification pending until its evidence/limitations are fully recorded.

L11 version12 vector-text visual review now covers104/288 pairs with27 direct inspections and exact complete-image transfers;184 remain. Inspected natural/mixed/nowrap forward and natural reverse rows at240/390/768. Wrapping, badges, card placement and clipping match Chrome, while glyph edge/weight differences are visibly present. The receipt retains per-case pixel failures in both direct and equivalent-image entries; inspection is not a pixel waiver. All288 geometry gates still pass and88 pixel failures remain unchanged. Receipt: indefinite-v12-text-vector/visual-inspection.json. Native full5551 passes and all5541 native scene images are accounted for; L11 remains qualification pending.

L11 vector-text review advances to171/288 pairs (54 direct,114 within-corpus image transfers,3 earlier external transfers);117 remain. Reverse mixed/nowrap rows and natural/mixed columns inspected at240/390/768. Browser/native text wrapping, clipped labels, overflow obscured by later siblings and nested badge placement match visually; glyph edge differences persist. Per-case failures remain in the receipt, and all88 pixel failures remain unwaived. Evidence: indefinite-v12-text-vector/visual-inspection.json. Native regression and visual accounting remain complete; L11 qualification remains pending.

L11 vector-text review now238/288 (81 direct,154 internal exact-image transfers,3 external);50 remaining. Constrained/natural/mixed columns and reverse columns inspected across240/390/768. Text overlap, sibling occlusion, background extents and badges match Chrome, with visible glyph raster differences retained. The88 pixel failures are unchanged and unwaived. Receipt: indefinite-v12-text-vector/visual-inspection.json. Native full regression/visual evidence is complete; vector review remains pending.

L11 final vector-text visual review complete288/288: 123 directly inspected pairs across41 sheets,162 within-corpus whole-image transfers and3 earlier reviewed external transfers. All576 candidate PNGs and contact sheets rehashed; every recorded direct/transfer pixel failure agrees with the unchanged replay. All288 geometry comparisons pass; pixels remain200 passes/88 failures. Wrapping, overlap, clipping and badge placement agree visually, including original-resolution inspection of an ambiguous reduced preview. Glyph edge/weight differences remain visible and unwaived. Evidence: indefinite-v12-text-vector/visual-inspection.json with zero remaining. Native full5551 tests and5541 image pairs remain passed and reviewed. L11 qualification audit is next; this does not qualify the vector text renderer.

L11 audit verified all four frozen toolchain hashes, public233 tests across45 targets, and saved qualification-audit.json in indefinite-v12-full. Closed missing final-toolchain vector lanes: original75/75, equal-factor360/360 and implicit-main36/36 all pass geometry/pixels. Original75 and factor360 have exact source and complete-image identity with reviewed native baselines (files rehashed), and their visual receipts are complete. Implicit-main24/36 match reviewed native images;12 text pairs differ and require direct review despite passing pixel gates. No live replay remains for these runs. Native qualification is still pending consolidation and those12 vector inspections; existing88 vector stress-text failures remain unwaived.

L11 is now native-qualified against frozen indefinite-v12-toolchain. Final audit and exact validation scope are in validation/indefinite-basis-qualification.md. The remaining12 implicit-main vector text images were directly inspected and match wrapping/placement; all36 pass unchanged pixel gates and are reviewed. Vector stress-text88 pixel failures remain explicitly unqualified and unwaived. Next priority is L12 content-box sizing; no new syntax is admitted by this status update.

### L12 investigation (not yet supported)

Content-box sizing now has a [contract and validation plan](validation/content-box-contract.md), paired border-box controls and preserved pre-feature compiler diagnostics. Public support remains border-box only until runtime transport and Chrome/native qualification are complete. Border combinations are preserved for P01.

L12 compiler integration update: content-box is implemented experimentally with version-13 occurrence requirements. Public Chrome geometry, native/WASM parity and host admission tests pass; real renderer pixel and expanded composition qualification remain pending. This supersedes the earlier “not yet supported” implementation status, not the qualification gate.

L12 expanded validation: 56 additional nested/wrapping/text/clip/font-relative scenes pass all 168 Chrome/native geometry and pixel comparisons. Initial visual inspection covers 36/240 pairs; the rest remain pending. Positioning reproducers are preserved for the separately unsupported positioning feature.

L12 composition progress: expanded parity passes across 1983 inputs; eight realistic cards pass 24/24 native comparisons. Visual review remains partial: initial36/240, expanded30/168, cards9/24. Qualification is still pending.

L12 cards: native24/24 pass and all24 visually reviewed; native/WASM parity1991 inputs passes. Vector cards retain24 geometry passes but24 unwaived pixel failures. Remaining initial/expanded native and vector visual reviews prevent qualification.

L12 visual update: vector cards24/24 reviewed with all24 pixel failures retained. Initial native60/240 and expanded48/168 reviewed; native cards24/24 complete. Qualification remains pending.

L12 expanded native visual review: 102/168 pairs accounted for with verified screenshot hashes; 66 remain. No new mismatches found. Initial native review and full regression qualification remain outstanding.

L12 full native regression is running against the frozen v13 toolchain with5983 expected tests, including all144 content-box scenes. Expanded native visual review132/168; remaining reviews and terminal regression results are pending.

L12 expanded native visual review is complete168/168, with hashes verified and no mismatches found. Initial native review and full regression results remain outstanding; qualification is pending.

L12 initial native review now114/240; remaining126 pairs and the active full native regression still prevent qualification. No new discrepancies found.

L12 initial native visual review now covers 174/240 pairs (126 direct, 48 exact image transfers). Reverse-row border-box and column zero/percentage/minimum/basis controls match Chrome. Remaining 66 pairs and full regression completion still prevent qualification. Expanded and card visual receipts remain complete.

L12 initial native visual review is complete240/240 (174 direct, 66 exact full-image transfers); all PNG and sheet hashes verified. Together with expanded168 and cards24, all432 focused native pairs are reviewed and pass. Vector cards24 failures remain unwaived and reviewed. Full native regression and coverage audit remain pending.

L12 coverage audit: padded images (fixed/percentage/max-width/rounded clip, both box modes) pass 24/24 Chrome/native geometry and pixel comparisons, with all 24 directly visually reviewed in `output/playwright/html-to-riv/content-box-image-native/visual-inspection.json`. Edge coverage passes 54/54 comparisons in `content-box-edge-native-v2`; its visual review is pending. The initial edge replay stopped on an ellipsis fixture missing the documented `display:block` precondition; both rejected inputs and the failure log are preserved. Corrected fixtures explicitly set block display. Added both corpora to native/WASM parity; its terminal result and the active full regression remain pending. No tolerance changes.

L12 audit completion: edge native54/54 now directly visually reviewed; image native24/24 already reviewed. Expanded native/WASM parity passes all9 tests across2017 inputs; receipt `output/playwright/html-to-riv/content-box-edge-native-v2/parity-receipt.json` records corpus and matching frozen compiler hashes. Total focused native coverage is510 passing, visually reviewed comparisons. Full regression and additional vector text coverage remain pending.

L12 expanded vector coverage passes168/168 geometry and pixel comparisons. Visual accounting currently105/168:96 exact full-image transfers from reviewed native results plus9 direct text/clip/max-width reviews;63 remain. Receipt: `output/playwright/html-to-riv/content-box-expanded-vector/combined-visual-inspection.json`. Existing vector card failures remain unwaived. Full native regression is still running.

L12 expanded vector visual review is complete168/168 (72 direct,96 exact full-image transfers), with screenshot and sheet hashes verified. All168 geometry and pixel gates pass. Vector cards retain24 reviewed, unwaived pixel failures. Full native regression and final evidence audit still prevent qualification.

L13 preparation: `validation/aspect-ratio-cases.json` defines64 prospective fixtures covering forward/reverse flex axes, both box modes, one/both/auto dimensions, percentage widths, min/max and flex basis. Chrome153 references captured at192 viewports in `output/playwright/html-to-riv/aspect-ratio-initial-oracle/`. All64 are rejected by frozen v13 compiler; rejection receipt is `aspect-ratio-initial-admission/receipt.json`. This is pre-implementation evidence, not support qualification. A separate auto+ratio probe is in progress.

L13 existing-runtime probe: `tests/aspect_ratio_runtime_investigation.rs` compiles a ratio-free input, injects existing property524, installs required content-box/flex policies, clones the artboard and resizes both instances240→390→768→240. Explicit exploratory run fails:40/64 scenes match Chrome throughout;24 fail, with320 coordinate mismatches in auto, max-width and flex-basis variants across all directions and both box modes. Evidence: `output/playwright/html-to-riv/aspect-ratio-runtime-investigation/receipt.json` and `run.log`. The test is explicitly ignored by default while the feature is under investigation; it must be invoked with `--ignored`. This is neither a public compiler test nor a supported feature. Runtime corrections are required before qualification.

L13 used-main correction: flex hypothetical cross sizing now applies an available preferred ratio to the final flexed main size, accounting for content-box padding. Existing Taffy106/106 tests pass. A non-ignored12-scene Chrome original/clone regression passes96 instance/viewports. Full exploratory corpus improves to52/64 scenes matching, with12 failing and256 coordinate mismatches: transferred max constraints and column auto intrinsic sizing remain; column auto additionally gains a width mismatch pending the intrinsic-main correction. Evidence: `output/playwright/html-to-riv/aspect-ratio-used-main/receipt.json`. No public aspect-ratio support or qualification claimed. The ongoing content-box regression uses its immutable pre-L13 v13 toolchain and does not validate this new engine patch.

L13 constraint-transfer correction now retains explicit preferred dimensions when transferring opposite-axis min/max limits through a ratio. Full runtime probe matches60/64 scenes; all8 max-width cases are fixed, and4 column-auto scenes remain (128 coordinate mismatches). Taffy106 tests pass. Receipt: `output/playwright/html-to-riv/aspect-ratio-constraints/receipt.json`. An intrinsic-inline-to-block correction is being tested; neither public CSS support nor qualification is claimed.

L13 runtime foundation now passes all64 scenes across512 original/clone instance-viewports. The column flex basis derives its block size from fit-content inline size and preferred ratio, respecting box sizing. Taffy106 tests pass. The entire probe is now a regular non-ignored regression and passes (no filtered cases); its prior failures remain preserved. Receipt: `output/playwright/html-to-riv/aspect-ratio-intrinsic-column/receipt.json`, regular run `regular-test.log`. This verifies existing-property runtime geometry only: public compiler syntax/transport, auto+ratio distinction, expanded stress cases, native/WASM parity and actual renderer pixels remain outstanding.

L13 parser foundation: `src/aspect_ratio.rs` retains the auto flag and normalized optional ratio;3 internal tests pass for both auto orders, comments, numbers, degenerates and invalid/unrepresentable values. Nonzero numeric underflow is rejected instead of silently becoming an auto/zero ratio. Math functions, units, percentages and negative ratios are excluded from this parser stage. Receipt: `output/playwright/html-to-riv/aspect-ratio-parser/receipt.json`. The parser is not connected to public CSS admission yet; combined auto/ratio runtime policy, transport and renderer gates remain outstanding.

L12 native qualification complete for frozen v13: full5983/5983 tests pass, and all5973 scene pairs exactly match reviewed source and full-image baselines. Focused native510/510 passes/reviewed; parity2017 inputs and host14 pass. Vector expanded168/168 passes/reviewed, while card24 pixel failures remain reviewed and unwaived. See `output/playwright/html-to-riv/content-box-v13-full/completion-receipt.json` and `validation/content-box-contract.md`. Subsequent L13 engine edits are not covered by this frozen snapshot and need separate regression evidence.

L13 ratio reference-box runtime tests pass: bare64 scenes/512 instance-viewports, combined/control12 scenes/96 instance-viewports, and64 additional clear-original/retained-clone size comparisons. Wrong target rejection and Taffy106 pass. The independent occurrence policy preserves authored box-sizing and clone state. Receipt: `output/playwright/html-to-riv/aspect-ratio-reference-box/receipt.json`. Compiler cascade/transport, host admission and pixel qualification remain outstanding.

L13 public compiler integration: numeric ratios and optional auto flag now pass through cascade (including initial/unset/inherit/variables/important), wire aspectRatio and version14 `layout-css-aspect-ratio-v1` requirements with unique layout targets and explicit ratio reference-box policy. Public76-scene original/clone oracle passes608 instance-viewports;2 contract tests,15 host tests, native/WASM builds and type checking pass. Host rejects old/unknown versions, missing capability, empty/duplicate/invalid targets and malformed policy flags before drawing. Full public suite,2093-input parity, native pixel runs and visual review are pending. Frozen toolchain: `output/playwright/html-to-riv/aspect-ratio-v14-toolchain/`. Qualification is not claimed; image auto sizing, math-function values and additional interactions remain pending.

L13 expanded validation: full public suite245 tests across52 targets passes with0 ignored; native/WASM parity passes9 tests across2093 inputs. Auto/ratio discriminator native36/36 geometry and pixel comparisons pass; visual review remains pending. Receipt: `output/playwright/html-to-riv/aspect-ratio-public-integration/receipt.json`.

L13 visual progress: auto discriminator36/36 reviewed (18 direct and18 exact-image transfers). Initial corpus25/192 accounted for (24 direct plus1 exact-image transfer); remaining167 pending. Expanded96-scene corpus captures288 Chrome viewports for min-width/max-height/conflicting constraints/stretch/wrap/nested ratios across all four directions, both box modes and bare/auto forms. Native replay is running against frozen v14; no expanded pass is claimed yet.

L13 expanded native run completed240/288 passing;48 geometry/pixel failures are all conflicting min/max cases and remain preserved in `aspect-ratio-expanded-native/`. Current corrections normalize a maximum against its minimum before transfer, and derive an automatic dimension from the constrained preferred opposite dimension while retaining raw authored dimensions for flex basis. Expanded public oracle now includes172 scenes; its new align-self policy installation fixes a test-harness omission that falsely reported stretch failures. Corrected runtime/public tests are running, with evidence under `output/playwright/html-to-riv/aspect-ratio-conflict-correction/`. No new renderer pass or qualification is claimed yet.

L13 conflict correction now passes the full172-scene public Chrome oracle across1376 original/clone instance-viewports, plus the auto reference-box lifecycle test and Taffy106 tests. Receipt: `output/playwright/html-to-riv/aspect-ratio-conflict-correction/receipt.json`. The corrected native probe is rebuilding; the48 frozen-v14 renderer failures remain unwaived until a fresh corrected replay and visual review complete.

L13 corrected native replay completes with516/516 geometry and pixel comparisons passing: initial192, auto36, expanded288. Corrected auto36 are visually accounted for by exact source/full-image identity with their reviewed baseline. Initial review remains48/192 on the first snapshot, and expanded corrected review is pending. Original48 conflict failures remain preserved as historical reproducers. Expanded parity2189 inputs is running. Toolchain: `aspect-ratio-v14-corrected-toolchain`; replay directories end in `native-v2`.


### L13 corrected conflict visual review

The corrected expanded corpus has48/288 pairs visually accounted for (36 direct,
12 exact full-image transfers), covering every conflicting min/max case across
four flex directions, both box modes, bare/auto ratios and three widths. Chrome
and native agree on panel size, inset child, sibling placement, reverse anchoring
and narrow viewport overflow. All288 expanded numerical comparisons pass; the
original48 failures remain preserved. Native/WASM parity is terminal and passes
2189 inputs across9 tests. L13 remains unqualified: other visual review,
text/composition/image coverage and the corrected-runtime full regression remain.
See validation/aspect-ratio-investigation.md for the current evidence table and
output/playwright/html-to-riv/aspect-ratio-expanded-native-v2/visual-inspection.json
for hashed review records. Chrome is the reference; tolerances are unchanged.


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


Expanded shape visual review is now102/288 (72 direct,30 exact-image transfers),
including row constraints, stretch, wrapping and nested ratios, in addition to
all conflict cases. The checked-in validation/record-visual-review.py rehashes
images and prior sheets before recording or auditing; five integrity tests pass.
Both expanded and image receipts pass its audit. Existing full native regression
remains active under session8434; no completion claim is made.


Expanded ratio review now accounts for180/288 comparisons (108 direct,
72 exact full-image transfers), including reversed-row constraints,
stretch/wrapping, nested ratios and forward-column min/max/stretch. All existing
image/sheet hashes pass the checked-in audit. Remaining108 comparisons are
explicitly listed in aspect-ratio-expanded-native-v2/visual-inspection.json.
The existing full native regression remains running under session8434. These
focused reviews do not establish that the full regression is complete.


### Experimental L13a ratio math (not qualified)

Aspect-ratio accepts scalar `calc()`, `min()`, `max()` and three-argument `clamp()`
in either ratio component, including nested functions/parentheses, arithmetic
precedence and optional auto. Addition/subtraction require CSS whitespace;
comments alone do not supply it. Top-level calculated components clamp negative
results to zero; calculated NaN becomes zero. Infinity and oversized numeric components now convert through the finite layout
representation. Numeric literals saturate to f64 before arithmetic; the infinity
keyword retains IEEE semantics inside expressions. This path is experimental
and still awaiting its native/WASM and pixel gates. Finite
components are rounded through f64 to f32, then values at or below eight f32
epsilons become zero before division, matching Chrome. This includes positive
subnormal/underflow components. Chrome layout approximation and saturation
remain unresolved numeric limits. Dimensions and percentages remain invalid here.
Evaluation is bounded to32 nested levels and256 value terms per component.
Direct declarations, custom-property values and fallbacks use the same parser.
This does not add math support to other properties. Public Chrome geometry tests
pass14 scenes/112 original-clone instance-viewports; rebuilt native/WASM parity,
pixel comparisons and visual review remain pending. The two numeric-limit Chrome
fixtures remain explicitly unqualified, preserved in the oracle asset.

L13a numeric boundary update: Chrome literal and math component clamping is now
measured at exact f32 neighbors; fixed-point layout conversion remains unresolved.
See `validation/aspect-ratio-math-research.md` and the retained
`aspect-ratio-math-precision-v3` probe/source receipt. This is investigation evidence,
not additional qualified syntax. The preprecision full module run passed 258 tests;
postprecision binary parity and visual qualification remain pending.

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


Font-boundary validation limitation: computed font sizes now clamp to10000px
before inheritance and em resolution, matching the recorded Chrome153 probe.
This correction has a public regression but awaits fresh native/WASM and visual
qualification. Very small font-relative ratios can still lose precision because
the current runtime contract stores one f32 ratio: the preserved1/15999 case
misses Chrome height by0.25/0.5px at390/768px. These cases are not qualified; no
geometry tolerance has been widened. See BACKLOG.md's font-boundary investigation.


The preceding font-boundary scalar limitation is corrected in the current
version15 compiler/runtime path: `layout-css-aspect-ratio-pair-v1` requires an
exact positive integer pair on every ratio occurrence, preserved on clones.
All existing Chrome ratio geometry regressions pass. Version14 inputs retain
scalar behavior. Fresh native/WASM parity and native-pixel/visual qualification
of version15 remain pending, so prior frozen snapshot receipts do not qualify it.


Version15 focused qualification now includes32 font-boundary scenes:96 native
geometry/pixel comparisons and256 original/clone instance-viewports pass, with
complete audited visual review. Native/WASM parity passes11 tests. Broader ratio
native regression replay and full L13 qualification remain pending; this focused
result does not extend the intentionally excluded syntax or layout modes.


Version15 broader native regression now passes541 scenes/1623 comparisons with
complete audited visual coverage, including image flex stress. Together with
font-boundary coverage this is573 scenes/1719 comparisons. Small image-edge
raster differences remain within the unchanged profile thresholds. Additional
exact-pair stress and full L13 qualification remain open.


Exact integer-pair stress now adds64 scenes/192 reviewed native comparisons and
512 clone/resize instance-viewports. Combined current ratio evidence is637
scenes/1911 comparisons; native/WASM parity11 tests and host-loading15 tests pass.
This covers the documented numeric, typed absolute-unit and em/rem subset; it
does not admit viewport/font-metric math, percentage gaps, Grid or positioning.

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

### Relative positioning candidate (L17, 2026-09-10)

The compiler accepts `position: static | relative`, `top`, `right`, `bottom`,
`left` and 1–4-value `inset`. Offsets accept auto, signed px/em/rem/percent and
zero, with absolute resolved length <=1000000px and percentage <=10000%. Static
offsets are ignored during emission; computed offsets remain available for
explicit inheritance. CSS-wide values and custom-property substitution apply.
Imported original/clone geometry tests, native/WASM parity and native painting
pass the current initial, composition and interaction corpora. Nine vector-text
composition pixel failures remain; positioning is not fully qualified across
renderer profiles. Absolute/fixed/sticky,
logical insets and general CSS math remain rejected. See
[relative positioning evidence](validation/relative-position-investigation.md).

L17 paint-contract update: relative positioning now requires runtime requirements
version17, capability `layout-css-positioned-paint-v1`, and explicit
`layout_positioned` targets. This includes relative elements with auto/zero offsets.
The host must validate and install the positioned paint policy before drawing.
Missing capability or invalid targets reject. Public version17 initial576, native composition192 and interaction96 pixel
comparisons pass with completed visual audits. Initial576 and interaction96
vector comparisons also pass with audited identical-image review transfers.
Public-host17 tests and native/WASM parity11 pass. The full native regression
passes5983 checks with5973 audited scene comparisons.

L17 initial visual audit complete:576/576 native views reviewed,0 remaining. Final12 reverse-column bottom-offset views match Chrome in translation, narrow gap below green, bottom space, nested inset and host sizing. All576 initial vector views transfer with identical canonical compiler inputs and full Chrome/native PNG bytes; independent transfer audit passes. Composition native192/192 and full native5983/5983 already pass with complete visual evidence. Nine composition vector text pixel failures remain unwaived; remaining interaction qualification audit stays open. Evidence: output/playwright/html-to-riv/positioned-v17-receipt.json.

L17 interaction visual audit complete96/96. Final12 reverse-column/content-box views match Chrome, including growing percentage padding, bottom overflow and purple top-viewport clipping at768. All96 vector views transfer with exact canonical compiler inputs and full Chrome/native PNG identity; transfer audit passes. Public original/clone resize test and expanded native/WASM parity11/11 pass. Nine composition vector text failures remain unwaived; final feature qualification audit remains.

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

L18 native interaction visual review reaches36/144, all direct and independently audited. Forward-column border-box and content-box cases cover auto margins, flex factors, min/max constraints, negative margins, percentage padding and stretch at240/390/768. Visible bounds, overlap order, sibling exposure and responsive growth match Chrome; no colored discrepancy marks observed. Native focused visual coverage now408/516;108 interaction views remain. Vector interaction/composition review and18 unwaived vector text failures remain open. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-interaction-native/visual-inspection.json.

L18 native interaction review reaches72/144, all direct and independently audited. Reverse-column border-box/content-box auto-margin, flex-factor, min/max, negative-margin, percentage-padding and stretch views match Chrome across240/390/768, including changing purple overlap and reversed sibling placement. Native focused visual coverage444/516;72 row-direction interaction views remain. Complete JavaScript suite passes12/12, including expanded native/WASM artifact parity and invalid absolute host contracts (absolute-v18-javascript-full12.log). Vector review and18 unwaived text failures remain open; full native baseline retry remains in progress.

L18 native interaction review reaches108/144, all direct and independently audited. Forward-row border-box/content-box auto margins, flex factors, min/max, negative margins, percentage padding and stretch match Chrome at240/390/768. Checked responsive purple occlusion, thin exposed top/right/bottom strips, growing orange dimensions and teal inset. Native focused visual coverage480/516;36 reverse-row interaction views remain. Full baseline retry session43262 confirmed live; vector review and18 unwaived vector text failures remain open. Receipt: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-interaction-native/visual-inspection.json.

L18 focused native visual qualification completes516/516. Interaction144/144 directly inspected and independently audited; final reverse-row content-box cases match Chrome for overlap, constrained sizing, padding growth and separation at240/390/768. All144 interaction views transfer to vector using exact canonical compiler inputs and full Chrome/native PNG hashes; transfer audit passes. Vector initial/nested/interaction coverage totals324/324. Vector composition192 review remains, including18 unwaived text pixel failures. Full native baseline retry session43262 remains active. Evidence: output/playwright/html-to-riv/absolute-v18-public-receipt.json.

L18 vector composition failure inspection completes18/18 across six text scenes and three widths. Chrome/native/diff sheets show matching two-line wrapping and box placement but visible glyph coverage differences on Quiet/weekend. Independent receipt audit verifies each original PNG and sheet hash plus exact coverage of every failing replay identity. All18 local RGB failures remain unwaived;174 passing composition views still need visual review or exact-input/full-image transfer. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-composition-vector/failure-visual-inspection.json.

L18 focused visual coverage is complete in both profiles:516/516 each. Vector composition192/192 combines24 direct text views with168 transfers requiring identical canonical compiler inputs and full Chrome/native PNG pairs from audited native review. Independent audit verifies unique full coverage, source receipt/replay hashes, target PNG/sheet hashes and failure identity. Numeric result stays174/192 composition passes and18 unwaived text failures (overall vector498/516). Six passing text views also show visible glyph differences; recorded without pixel-identity claims. Full native baseline retry remains active. Evidence: output/playwright/html-to-riv/absolute-v18-auto-inset-margin-composition-vector/composition-visual-coverage.json.

L19 z-index investigation started with48 Chrome reference scenes/144 views (153.0.8010.12). Reference only: compiler/runtime support remains pending; no support or qualification claim. Auto versus integer zero requires distinct context scope, including static flex items. See validation/stacking-context-investigation.md.

L19 pure stacking scheduler implemented;13 targeted runtime tests pass (six new context tests plus seven existing auto-position tests). Public z-index syntax and occurrence metadata are still pending, so this does not qualify support. See validation/stacking-context-investigation.md and output/playwright/html-to-riv/stacking-v19-planner-tests.log.

L19 public compiler implementation now accepts `z-index: auto` and signed integers, including explicit zero, CSS-wide resets/inheritance and custom-property substitution. Integer values create atomic stacking contexts for supported layout boxes, including static flex items; they do not establish positioning containing blocks. Fractional, dimensional, multi-value and math expressions are rejected. Requirements version19 adds `layout-css-stacking-v1` and unique non-root `{object_id, level}` targets in `layout_stacking`; positioned/absolute metadata is optional when only static flex contexts occur. Hosts must validate and install the stacking policy. Native probe installation and JavaScript declarations are connected. Qualification remains pending: five focused compiler/recorded-runtime tests pass, but native/WASM parity, explicit containing-block invariance, stacking clip integration, resize geometry and actual Chrome/native pixel comparisons remain open. This implementation is provisional, not a visually qualified support claim.

L19 first frozen public native replay completes144 views:144 geometry pass,132 pixel pass,12 unwaived paint failures in reverse directions (equal-level CSS order and column-reverse negative-descendant/equal-sibling cases). Row-reverse equal-order three-width sheet directly inspected: Chrome teal covers green overlap, native green covers teal. Other nine failing views remain uninspected. Existing css_ordered_drawables reverses child traversal for reverse flex directions; its interaction with context scheduling needs investigation before changing baseline behavior. Native/WASM toolchain frozen in stacking-v19-toolchain; expanded JavaScript parity running. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 tie-order fix passes6/6 focused tests, including the previously failing reverse-direction regression with original/clone resize. Pure scheduler tests running session26453; rebuild/freeze native probe and replay all144 Chrome views next. No post-fix pixel success claimed yet.

L19 corrected frozen native and vector replays each pass144/144 geometry and pixel comparisons. All12 previously failing views directly re-inspected; green/orange and teal/green overlaps now agree with Chrome. Native review audit covers21/144 views (12 direct plus9 exact image-pair transfers);123 native and vector review remain. Pure scheduler14/14 and public48-scene clone/resize/clear/reinstall geometry test1152/1152 updates pass. Initial failure evidence retained. Expanded text/image/clip compositions, host-negative imported-target checks, full regression and remaining visual qualification remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 host-negative integration passes: missing capability, malformed context lists, style/missing targets, fractional/out-of-range levels reject before rendering; restored valid requirements succeeds. Probe diagnostic capability disable added without changing rendering semantics. Native visual audit42/144 (18 direct,24 exact within-run image-pair transfers) includes row auto versus zero at all widths: teal escape and green overlap match Chrome. Full compiler regression running session3670;102 native views, vector transfer/review and composition expansion remain. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition expansion adds32 scenes/96 Chrome views: four directions, text/image auto versus zero contexts, nested clipping, negative parents and deep auto escape/zero containment. Initial text fixture24px line height rejected by existing Inter natural-metric constraint; initial references and failed replay logs retained. Corrected28px fixture recaptured with pinned Chrome153.0.8010.12. Native/vector replay running sessions26631/65233; expanded13-test JavaScript suite running38651. Initial native visual review48/144 (24 direct,24 exact pair transfers) after ancestor-clip/static-flex sheets inspected at all widths. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native/vector each96/96 geometry/pixel pass. Expanded JavaScript suite13/13 passes including32 composition fixtures native/WASM artifacts. Public resize test now80 scenes/1920 original-clone updates across stacking install/clear/reinstall, all Chrome geometry pass. First12 native composition views (row text/image auto/zero) directly inspected and audited: matching two-line text, quadrant placement and context-dependent green occlusion.84 composition native views and vector review/transfer remain, alongside96 initial native views. Full visual completion and broader regression gate remain open. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review24/96 complete and audited after all forward-row cases inspected. Nested clips hide deep rose, negative parent descendants stay behind host, and deep auto versus zero yields expected rose/green overlap. Added80 stacking fixtures to the regular browser regression gate. Frozen full native suite launched6223 tests (session41911), isolated stacking-v19-full-native outputs; no terminal result claimed. Initial/native composition remaining visual views96/72; vector transfers/review pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition review48/96 completed and audited, covering all forward/reverse row cases at three widths. Reverse text/image right viewport crop, nested clipped rose/teal and hidden negative-parent subtree match Chrome. Reverse auto/zero variants have no green overlap, so their review proves placement/crop rather than context distinction (covered by forward-row overlaps).48 column/reverse-column composition views remain. Full baseline41911 confirmed live this turn; no terminal claim. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition native visual audit72/96 complete, adding all forward-column text/image/clip/negative/deep scenes. Chrome/native match text wrapping, image placement, green/purple overlap, nested clipping and negative-parent hiding. Column auto/zero pairs do not expose sibling overlap, so their visual evidence is limited to placement/visibility. Final24 reverse-column views remain. Full native baseline41911 confirmed live. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 native composition visual review96/96 complete and audited. Final reverse-column views match Chrome; negative-parent rose overflow correctly remains visible as strip below host background. Exact canonical inputs and full browser/native image pairs transfer72/96 vector views, audit passes;24 vector text views differ and need direct inspection (numeric gates already pass). Initial48/144 native review and full baseline still pending. Evidence: output/playwright/html-to-riv/stacking-v19-receipt.json.

L19 composition visual coverage is now complete in both profiles: native96 direct; vector24 direct text views plus72 audited canonical-input/full-image transfers. Independent disjoint-set audit covers all96 vector cases (stacking-v19-composition-line-height-vector/visual-coverage.json). Column text wraps and baselines match Chrome with small visible glyph-edge differences within unchanged thresholds. Auto/zero column pairs establish placement, not overlapping context discrimination. Initial native96 views and initial vector review remain; full6223 native gate confirmed live through session41911. No qualification claim yet.

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

Dynamic stacking clipping now passes all864 real renderer/Chrome geometry and pixel comparisons over12 public compiler scenes, original/clone instances, both ancestor clip toggles and repeated widths. All60 distinct pairs were directly inspected in20 contact sheets;804 additional pairs have audited exact full-image transfers. All720 repeated/clone states are pixel-identical, and144 host/parent clip discriminators show changed pixels in both renderers. New-request native/WASM byte/map/requirements parity passes12/12 (initial CLI-only languageVersion argument error preserved). Evidence: `output/playwright/html-to-riv/stacking-v19-clip-lifecycle-pixels-v2/{replay,visual-inspection,lifecycle-audit}.json` and `stacking-v19-clip-lifecycle-parity-v2/receipt.json`. This qualifies the rectangular shape clip lifecycle interaction only; broader L20 rounded/text/image overflow remains separate. The corrected full6223 gate and its visual audit remain pending; L19 stays active.

L20 initial rounded overflow:27 scenes/81 Chrome comparisons pass both renderer profiles;216 original/clone repeated-size geometry checks and27 native/WASM parity cases pass. Native visual audit covers36/81 views (18 direct plus18 identical hidden controls), including rounded shape/image boundaries and text clipped through glyphs. Remaining45 native views and vector audit remain; no full L20 qualification claimed. The first resize test used an absolute-policy installer for a relative-only contract; corrected to the shipping probe relative installer, preserving the failed log. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L19 qualification completed for the documented subset: corrected full6223/6223 checks and6213/6213 visual transfers pass; regular overlap48/48 checks and visual transfers pass. Focused288/profile and dynamic clip864 frames retain complete visual evidence. See `output/playwright/html-to-riv/stacking-v19-qualification.json`. Broader overflow continues under L20; unrelated vector text limitations remain tracked.

L20 initial visual audit complete in both profiles:81/81 comparisons each. Native54 direct+27 exact hidden-control transfers; vector18 direct+9 within-run transfers+54 audited exact-input/full-PNG cross-run transfers, with disjoint coverage checked. All108 profile-specific visible-versus-clip/hidden discriminators preserve geometry and change pixels. Sparse vector glyph/corner antialiasing differences remain within unchanged gates. Initial27-request parity and216 clone/resize geometry checks pass. L20 remains active for nested/positioned compositions, CSS-wide resets and axis/clip-margin investigation. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested expansion:24 scenes/72 comparisons pass each renderer profile;24 new public native/WASM parity cases and combined51-scene/408 original-clone resize checks pass. Native visual coverage is36/72 after inspecting absolute underlined text clip/initial and relative image unset across all widths. Remaining visual review and CSS-wide discriminators are open. Original br fixture rejected by the documented block-context rule; corrected display:block oracle and failed runs retained. Added reusable `validation/parity-fixtures.mjs`. Evidence: `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 nested audits complete:72 views/profile, native18 direct+54 exact image transfers; vector6 direct+18 within-run exact transfers+48 exact-input/full-PNG transfers. All36 profile-specific reset groups verify identical geometry, inherit=clip, initial=unset and changed pixels when the inner clip is released. Combined focused coverage is153/profile;51 public parity cases and408 clone/resize checks pass. Axis investigation captured18 prospective scenes/54 pinned Chrome views; all18 inputs are explicitly rejected by the current compiler. Runtime currently has only a two-axis rounded clip path; axis policy/transport and clip-margin support remain implementation work, not external blockers. See `validation/overflow-investigation.md` and `output/playwright/html-to-riv/overflow-l20-receipt.json`.

L20 axis foundation implemented: compiler overflow values now retain specified x/y states and compute their coupling, preserving hidden versus clip and distinguishing computed auto. Existing admission is unchanged until runtime support lands. All333 compiler tests across63 result groups pass; native build passes. New native outputs for51 reviewed overflow inputs are byte-identical to previous Rive/map/requirements artifacts. WASM build56207 is in progress. Renderer current device clip bounds provide a bounded exact half-plane intersection path without an arbitrary visible-axis extent; render API/replay/backend/policy/transport work remains.

L20 axis renderer implementation added: optional clip_axis API, precise recording and typed replay command, glyph-adapter forwarding, and Metal device-bound half-plane clipping. Geometry tests3/3 pass. Metal-feature test20200 and stream test40563 remain running; real pixel validation and runtime/compiler policy installation remain pending. No new public axis syntax is admitted.

Axis renderer affine failure fixed: diagnostic output showed initial overallClipPixelBounds uses i32 sentinel extremes. Polygon intersections therefore reached billions of pixels, losing ordinary-edge precision on conversion to f32. Intersecting existing bounds with actual frame dimensions before constructing the strip fixes all36 failures. Fresh frozen renderer passes144/144 Chrome controls, with unchanged thresholds; Metal-feature geometry tests3/3 pass. Nine rotated integer x/y/both views directly inspected: clipped bounds and restored purple agree, sparse slanted-edge antialiasing remains.135 visual views remain unaudited. Original failure corpus/debug output retained. Public axis syntax remains rejected pending runtime policy, compiler/host transport, clone/resize and public parity qualification. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/replay.json and overflow-l20-receipt.json.

Axis renderer controls complete:144/144 Chrome comparisons pass, with audited visual coverage (120 directly inspected views and 24 exact full-image transfers). Final fractional-translation/reflection clips and all unclipped controls agree in extent/cropping/restore; thin fractional edge coverage differences remain within unchanged gates.432 axis-discrimination and288 restored-region checks pass. This qualifies the direct Metal renderer diagnostic matrix only. Runtime/public compiler axis syntax remains pending. Shared integration must cover both LayoutComponent.draw_proxy and begin_css_ancestor_clip, which currently read base.clip and clip a world rounded path. Live dimensions, transform restoration, clone policy and plain-container proxy inclusion must all be preserved. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-finite-frame/{replay,visual-inspection,control-audit}.json.

Transform-aware axis renderer API added: clip_axis_transformed composes an additional local matrix for clipping while restoring the exact prior drawing matrix, avoiding inverse-transform roundoff. Recording/replay accepts optional finite clipAxis matrix; adapter and Metal frame/canvas forwarding included. Expanded render-stream7/7 passes. Fresh local-transform mode completes144/144 Chrome pixel controls; complete authored HTML/CSS and browser/native PNG byte identity transfers all144 reviews from the audited finite-frame run. Control audit432 distinctions/288 restored regions passes. No public compiler axis syntax admitted. Runtime occurrence policy and clone/resize/host integration remain next. Evidence: output/playwright/html-to-riv/overflow-axis-renderer-transformed/{replay,control-audit,visual-identity-transfer}.json.

Live runtime axis policy implemented: LayoutComponent stores optional CssOverflowAxis (horizontal/vertical), clones it, keeps a drawable proxy through toggles, and computes each strip from current layout width/height and world transform. The shared ordinary/deferred clipping helper dispatches clip_axis_transformed; clearing restores the imported two-axis clip flag. An unsupported renderer fails explicitly rather than drawing without the requested clip. Positioned-paint6/6 passes, including32 new original/clone frames with axis switches, repeated240/390/768/240 sizes, clear/reinstall, deferred clips and sibling restore isolation; prior864 dynamic two-axis frames remain covered. This is runtime stream/lifecycle evidence, not public CSS or runtime pixel qualification. Atomic checked installer, versioned compiler/host transport, public admission/parity and actual runtime pixel corpus remain. Evidence: output/playwright/html-to-riv/overflow-axis-live-runtime-tests.log.

Axis overflow contract foundation added: version20, layout-css-axis-overflow-v1 and strict layout_axis_overflow entries {object_id,axis:x|y}. Validation enforces version/capability coupling, unique non-root layout targets and optional stacking coexistence; empty entries are omitted from older outputs. Public Rust exports and TypeScript version union updated. Native probe maps requirements through the checked runtime installer. Focused2/2, full compiler337 tests across64 groups and TypeScript checks pass. Featured probe build25280 is running; initial mistaken example target failed before building and is preserved. Public CSS axis syntax/emission, WASM/parity and runtime pixel qualification remain pending. Evidence: output/playwright/html-to-riv/overflow-axis-contract-{tests,full-tests,types-final}.log.

### Axis overflow: implemented, visual qualification pending

`overflow` accepts one or two `visible`, `clip`, or `hidden` values (X then Y;
a single value sets both). `overflow-x` and `overflow-y` accept one value.
Normal cascade precedence, `inherit`, `initial`, `unset`, and supported `var()`
substitution apply. Inheritance copies the parent's computed axis values.

`clip visible` and `visible clip` emit version20
`layout-css-axis-overflow-v1` requirements. The runtime uses the current layout
width/height, leaves the other axis unrestricted, and does not round a one-axis
strip. Both clipped axes retain the existing rounded clipping path. Mixed
`clip`/`hidden` computes to `hidden` on both axes. Pairing `visible` with `hidden`
computes to `auto` and is rejected with `unsupported-computed-overflow`.
Explicit `auto`, `scroll`, clip margins, and scrolling behavior remain unsupported.

This syntax is implemented and passes focused compiler tests, but its public
native/WASM parity and runtime-emitted Chrome geometry/pixel corpus are still
pending. The completed direct renderer matrix does not qualify this compiler path.

Public axis overflow initial corpus passes: corrected full compiler339 tests/65 groups, fresh native/WASM builds,14 public parity cases,42 Chrome geometry/pixel comparisons per renderer profile, and combined65-scene520 original/clone resize geometry checks. Four prospective visible/hidden scenes remain intentionally rejected for computed auto. Initial replay failed on missing copied oracle PNGs; exact42 reference files copied with hashes and fresh native-images output passes. Twelve one-axis native views directly inspected: responsive clip extents, visible-axis escape, viewport crop and square clip over rounded orange background match Chrome.30 native visual reviews and vector audit remain, followed by composition/host/full regression expansion and clip margins. Evidence: output/playwright/html-to-riv/overflow-axis-public-{native-images,vector,parity} and overflow-l20-receipt.json.

Public initial axis corpus visual review complete:42 native views (24 direct,18 exact within-run image transfers) and42 vector exact compiler-input/full-PNG transfers audited. New48-scene composition matrix covers shape/underlined multiline text/image, relative/absolute children, X/Y clips, painted/plain hosts and outer rounded clip/visible controls with positioned sibling overlap. All144 Chrome comparisons pass per renderer profile,48 new native/WASM parity cases pass, and combined113-scene904 original/clone repeated-size geometry checks pass. Composition visual review/discriminator audits remain pending; no full L20 qualification claimed. Evidence: output/playwright/html-to-riv/overflow-axis-composition-{native,vector,parity,oracle} and overflow-axis-composition-resize.log.


Axis-overflow validation update (2026-09-10): short outer-container controls now have complete native visual coverage (72/72). Vector geometry/pixel gates pass for these controls; 48/72 visual reviews transfer by exact input/image identity and 24 changed text views remain to inspect. This is qualification progress, with no expansion of accepted syntax. L20 remains active.

Short-outer vector follow-up (2026-09-10): the remaining text views are now inspected or transferred by exact image identity. Both profiles have complete 72/72 visual coverage for this corpus, verified by the combined review audit. Accepted syntax is unchanged; broader L20 qualification remains active.

Main axis composition validation (2026-09-10): native144/144 visually audited; vector96/144 exact input/image review transfers and48 changed text views pending. Numeric geometry/pixel comparisons pass in both profiles. This changes qualification evidence only, not accepted syntax.

Main axis composition vector follow-up (2026-09-10): all144 views now have audited visual coverage. The initial42, composition144 and shortouter72 axis comparisons per profile are fully reviewed or transferred with verified identity. This completes focused visual evidence only; L20 remains active and accepted syntax is unchanged.


Axis host regression discovered (2026-09-10): new public host test reproduces missing axis clip for an unpainted, unpositioned container after rebuilding the featured probe. Earlier focused pixel passes do not qualify this configuration. Preserved request, Rive, requirements and stream: `output/playwright/html-to-riv/overflow-axis-plain-host-reproducer/`. Runtime drawable lifecycle investigation is active; do not claim general axis clipping qualified. Logs: overflow-axis-host-initial.log and overflow-axis-host-focused.log.


### Plain axis proxy lifecycle fix (2026-09-10)

The checked installer now creates missing drawable proxies for plain layouts after import. It places nested proxies using complete owner ancestry, preserving subtree boundaries. The first insertion attempt opened clips after child paint; that failing trace is retained in overflow-axis-proxy-nested-debug.log. Corrected runtime tests pass8/8, including32 new original/clone frames with repeated install, clear, reinstall and resizing; existing positioned/stacking lifecycle tests also pass. Rebuilt native host tests pass18/18, including missing axis capability and malformed manifest rejection before stream output. Eight prospective plain/nested/painted x/y fixtures are in validation/overflow-axis-plain-cases.json. Pixel/geometry qualification and parity against a newly frozen toolchain remain pending; prior corpus receipts refer to their original binaries. Logs and source hashes are in overflow-l20-receipt.json.


### Plain-container fix pixel qualification (2026-09-10)

Frozen overflow-axis-proxy-toolchain passes24/24 Chrome geometry/pixel comparisons in each profile and8/8 public native/WASM parity cases. All24 native views were directly inspected; all24 vector views transfer with exact compiler-input/full-PNG identity and an independent audit. Single/nested x/y clips match expected red extents and preserve the blue sibling outside the clipped subtree. Painted x cases retain a thin fractional background-edge difference at390 within unchanged tolerances. Public original/clone resize regression now passes1160 updates across145 scenes. Initial resize run failed only its obsolete1096 count assertion, preserved in overflow-axis-plain-resize.log; corrected count passes in overflow-axis-plain-resize-final.log. Prior focused corpora and full regression still need rerunning with the fixed runtime. L20 remains active.

Fixed-runtime focused regression (2026-09-10): all258 earlier axis comparisons per profile pass with complete exact-input/image review transfers. The24 new plain-container views per profile are also reviewed. Full gallery completion and clip-margin work remain pending; L20 stays active.

Clip-margin investigation (2026-09-10):48 prospective scenes/144 pinned Chrome captures and48 compiler rejections recorded. Margin changes affect two-axis clip controls; hidden and single-axis images stay unchanged despite accepted computed styles. See validation/overflow-clip-margin-investigation.md. Syntax remains unsupported.

Clip-margin parser progress (2026-09-10): the internal authoring-value parser is tested, but overflow-clip-margin remains rejected by the public compiler until runtime, cascade and visual qualification are implemented.


### Live clip-margin bounds foundation (2026-09-10)

Runtime `CssOverflowClipMargin` now validates finite nonnegative offsets and resolves local clip bounds from the current LayoutComponent size and used asymmetric padding. Content origin insets each edge before expansion; padding and border origins coincide while compiler borders remain unsupported. The resolver rejects nonfinite resolved edges and does not mutate layout or background paint. It is not yet connected to drawing, transport or public CSS admission. Focused runtime tests cover repeated size/padding changes and invalid/overflowing numeric inputs; log: output/playwright/html-to-riv/overflow-clip-margin-runtime-bounds.log. Both focused runtime tests passed (2/2); this is geometry-unit evidence only, not renderer or browser qualification.


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

2026-09-10 P01 border baseline:54 prospective uniform-solid-border scenes captured162 Chrome153.0.8010.12 references across width/radius/box-sizing/overflow combinations. The frozen version21 compiler rejects all54 with unsupported-property diagnostics; no border support is admitted. Evidence: output/playwright/html-to-riv/border-solid-initial-oracle and border-solid-initial-admission. Runtime investigation identified used-border retention, content measurement and inner clipping as required work.

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


### Individual border sides: version 23 native qualification

Physical `border-top/right/bottom/left` shorthands and their width/style/color
longhands are implemented, along with one-to-four-value `border-width`,
`border-style` and `border-color` lists. Solid, none and hidden styles use the
documented border profile. Unequal used widths or resolved colors emit
`layout-css-border-sides-v1` requirements with colors in top/right/bottom/left
order; widths remain native responsive layout properties. Uniform used state
retains the version 22 representation. CSS Grid remains excluded.

This profile is qualified for the native renderer. The corrected equal-color
paint grouping has completed the 40-scene edge matrix (320 resize/clone frames)
and 48-scene text/image/clipping composition matrix (384 frames). Both have
audited visual coverage against pinned Chrome 153. The composition audit also
reproduces every RIV file and requirements manifest with its embedded assets.
Public JavaScript/native-WASM parity passes 13 tests.

The corrected initial 24-scene matrix also passes all 192 frames, with exact
reviewed image transfer and public artifact reproduction. The full 7,372-check
regression passes, and all 7,362 image pairs have audited visual coverage.
The complete module suite passes 398 tests and checked-host tests pass 21.
Qualification receipt: `output/playwright/html-to-riv/border-p02-receipt.json`. Earlier
entries above describe historical stages, including pre-version-23 rejection
and the equal-color seam reproducer. Qualification must use the corrected
artifacts, not those earlier candidate results. Current focused evidence:
`output/playwright/html-to-riv/border-sides-grouped-edge-completion.json` and
`output/playwright/html-to-riv/border-sides-grouped-composition-lifecycle/public-composition-audit.json`.

### Per-corner circular radii candidate

Current source accepts one-to-four nonnegative circular lengths for `border-radius` and a single circular length for each physical corner longhand. Supported px/em/rem resolution, CSS-wide resets and custom-property substitution apply. Elliptical pairs/slash syntax and percentages remain explicitly rejected pending P04. Asymmetric CSS paint uses proportional overlap reduction. Initial24 scenes pass192 resize/clone geometry and pixel checks, complete visual review and artifact reproduction; native/WASM13 and module404 tests pass. Broader text/image/clipping compositions and the full7444 regression remain pending. This candidate is not yet qualified.

### P03 composition lifecycle audit complete — 2026-09-10

All108 distinct composition views now have audited visual coverage:54 directly inspected and54 exact full-image transfers. The final six sheets cover expanded container clips, axis image clips and inset image clipping at240/390/768. Image extents, quadrant placement, asymmetric contours and sibling positions agree structurally; sparse curve antialias and right-edge image sampling differences remain under unchanged gates. The public audit reproduces all36 RIV/requirements artifacts with complete assets and verifies288 original/clone resize frames, stream hashes and reviewed PNG pairs. Receipt: `output/playwright/html-to-riv/corner-radii-composition-lifecycle/public-composition-audit.json`. Full regression40992 remains live; broad qualification is pending.

### P03 nested pixels pass; isolated border-box corners reviewed — 2026-09-10

Nested geometry and native pixels pass128/128. Reviewed12/48 distinct views for all four isolated corners in border-box nested flex layouts; selected corner, inner clip, border contour, panel/aside sizing and sibling placement agree with Chrome. Sparse curve antialias differences remain. Expanded parity run passes10 tests including accepted-corpus native/WASM artifact equality; three host checks initially fail ENOENT because NUXIE_NATIVE_PROBE was omitted. Explicit frozen-probe rerun passes all three (no compiler/runtime changes). Original failure log retained alongside `corner-radii-nested-host-r2.log`. Visual audit remains36 views short; full regression40992 still live. Evidence: `output/playwright/html-to-riv/corner-radii-nested-progress.json`.

### P03 nested lifecycle qualification complete — 2026-09-10

All48 nested views directly inspected and hash-audited. Final inheritance and invalid-variable reset sheets match Chrome structurally in both box models; sparse curve antialias residuals retained. Public audit reproduces all16 compiled scenes/requirements and verifies all128 original/clone resize frames against reviewed PNG pairs and stream hashes. Together with initial192 and composition288, focused P03 coverage now totals608 frames across76 scenes. Native/WASM corpus equality and corrected host checks pass as documented. Full regression and broad target gallery audit remain pending; refreshed module suite launched. Receipt: `output/playwright/html-to-riv/corner-radii-nested-lifecycle/public-nested-audit.json`.

### P03 circular corner radii native-qualified — 2026-09-10

Frozen full regression passes7444/7444; combined visual proof accounts for all7434 rendered pairs (7362 prior plus72 new), retaining both component comparisons. Fresh focused audits reproduce76 scenes and verify608 resize/clone frames; all228 distinct focused views have audited coverage. Default module suite396 passes; expanded parity10 plus corrected host3 checks pass, with the original omitted-probe failure preserved. P03 is native-qualified for the documented circular-length LTR profile against the frozen proportional toolchain. Curve antialias/image sampling, direction controls, vector and P04 limitations remain explicit. Receipt: `output/playwright/html-to-riv/corner-radii-p03-receipt.json`. Later P04 source changes are not covered by this qualification.

### P04 first diagnostic pixels pass — 2026-09-10

Experimental ellipse recording and replay each pass128/128 original/clone frames. All16 cases cover percentages, slash lists, mixed pairs, zero axes and overlap in both box models. First6/48 distinct views inspected: border-box percentage ellipse and four-pair corners match Chrome structurally across240/390/768, retaining sparse contour antialias differences under unchanged gates. Evidence: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/replay.json` and `elliptical-radii-geometry-progress.json`. This is runtime-experiment-only; remaining visual coverage, public grammar/host transport/parity and expanded composition qualification remain pending.

### P04 diagnostic visual lifecycle complete — 2026-09-10

All48 initial ellipse views directly inspected. Fixed ellipses, zero-axis square corners and independent shorthand list expansion agree structurally with Chrome in both box models; sparse contour antialias differences retained. The128-frame visual audit verifies source/projection identity, injected radius pairs, actual compiler CSS, complete original/clone resize sequences, stream hashes and reviewed PNG pairs. This is explicitly runtime-experiment-only and does not qualify public compilation/host transport. Receipt: `output/playwright/html-to-riv/elliptical-radii-diagnostic-lifecycle/diagnostic-visual-audit.json`. Public grammar/transport/parity and expanded fractional/composition evidence remain next.

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

Added six prospective opacity stacking scenes and18 pinned Chrome references. At opacity0.5, child z-index -2/0/2 remains isolated below an outside sibling at z-index1. At opacity 1, child z-index2 escapes the non-stacking parent and covers that sibling. All18 exact interior overlap samples confirm this distinction (output/playwright/html-to-riv/group-opacity-stacking-oracle/stacking-samples.json). Canvas begin_frame inspection shows shared render-context beginFrameExecutable; nested use must preserve parent work through an explicit scheduling/recording strategy, not assume independent contexts. Public opacity remains unsupported. P04 full regression remains running.

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

Corrected initial36 and stacking18 views now have complete audited coverage (initial30 direct/6 exact-image transfers; stacking12 direct/6 transfers). Fresh artifact audits verify all18 stripped compiler artifacts and144 original/clone frames against the recorded inputs and reviewed images. Alongside corrected composition12 scenes/96 frames/36 views, the current renderer has30 diagnostic scenes,240 lifecycle frames and90 reviewed views. Stacking discriminators preserve group isolation below opacity 1 and allow child z-index participation at opacity 1. Sparse curved-edge antialias and quantization differences remain explicit; public compiler admission, host transport, native/WASM parity and broader public regression are still pending. Receipt: output/playwright/html-to-riv/group-opacity-corrected-regression-progress.json.

### P05 computed opacity cascade integrated — 2026-09-10

Style now stores group opacity independently of paint colors, defaults to1 without inheritance, supports explicit inherit/initial/unset, and uses the existing normalized number/percentage parser. Added cascade/substitution tests. Initial focused run5/6 exposed math fallback incorrectly invalidating to unset; corrected compatibility classification retains unsupported math diagnostics. Full library48/48 passes. Public admission remains explicitly gated until isolated-compositing transport exists; one regression test covers8 authored values and passes. No new public opacity support claimed. Receipt: output/playwright/html-to-riv/group-opacity-computed-style-progress.json.

### P05 version25 requirement contract tested — 2026-09-10

Added LayoutGroupOpacityRequirement with unique non-root LayoutComponent IDs and finite normalized opacity in [0,1); opaque entries are omitted because opacity 1 does not establish a stacking context. Version 25 and layout-css-group-opacity-v1 must match nonempty entries. The capability explicitly requires isolated subtree compositing, undithered intermediate surfaces and checked renderer capacity. Structural validation rejects missing/extra fields, wrong types, duplicates, root targets, invalid alpha and mismatched versions/capabilities. Corner coexistence and old-version controls pass. Three opacity plus four corner requirement tests pass, and TypeScript version25 declarations/typecheck pass. Checked host installation, public emission and native/WASM parity remain pending; CSS admission stays gated. Receipt: output/playwright/html-to-riv/group-opacity-requirements-progress.json.

### P05 checked host installation verified — 2026-09-10

Probe now advertises and installs version25 group-opacity requirements, supports explicit disabled-capability controls and uses checked instance drawing. Added public try_draw/try_draw_handle entry points preserving artboard frame identity; insufficient group capacity refuses before paint. Existing atomic/clone/clear/resize integration test now exercises the checked instance API and passes. New host transport test passes nested alpha at four widths, missing capability and malformed manifest rejection before stream output. Initial full host run had two feature-configuration failures (probe lacked native-glyph-controls); preserved. Rebuilt with native-glyph-controls and full host regression passes. Public CSS emission remains gated; these are manually supplied manifest controls. Receipt: output/playwright/html-to-riv/group-opacity-host-progress.json.

### P05 public opacity emission candidate — 2026-09-10

Public CSS now emits version25 normalized group requirements for computed opacity below 1, leaving Rive layout and paint bytes unchanged. Explicit inheritance, CSS-wide resets, percentages, clamping, important cascade and custom-property fallback are covered; opaque values omit requirements and preserve prior artifacts. Five requirement/public emission tests pass. General opacity math remains rejected. Full Rust regression initially stopped on a stale opacity-rejection expectation; that test now checks excluded calc syntax, with original failure retained. Full rerun and native/WASM parity are pending; this is an implemented candidate, not a native-qualified feature. Added 30-scene JS/native/WASM parity test using existing Chrome-reference corpora; execution awaits matching builds. Current scope remains static isolated compositing through the checked recording/replay host, not immediate backends without group support.

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


## Archived front-contract snapshots

The following text records earlier implementation stages. It is historical,
not the current accepted/unsupported contract. Receipts and reproducers remain
unchanged. Current scope is stated at the beginning of this document.

### Earlier uniform-border qualification snapshot

**Uniform borders qualified for the native renderer:** the compiler now accepts uniform `border`,
`border-width`, `border-style` and `border-color` and emits version22 runtime
requirements. Widths accept nonnegative px/em/rem, unitless zero and
thin/medium/thick; styles accept solid/none/hidden; colors use the supported
color grammar plus currentColor and transparency. Single values only; individual
sides, multi-value longhands, other styles, percentage widths and border images
remain intentionally unsupported. CSS-wide values and custom-property resolution
follow the existing cascade policy. None/hidden have zero used width; transparent
solid borders still occupy space. The host must support and install
`layout-css-solid-borders-v1`. Focused solid, transparency, plain-border,
text/image, axis clipping, signed clip-margin and deferred-sizing matrices have
completed native visual review. Public original/clone resize pixel audits pass,
including the current combined768-frame and text/image192-frame runs. Native/WASM
parity passes the documented corpora. The full version22 regression passes7180 checks with7170 scene pairs audited
against reviewed compiler inputs and images. Evidence: `output/playwright/html-to-riv/border-p01-receipt.json`.
Earlier gate-closed and partial-review notes below are historical.

### Earlier flex admission and candidate snapshots

The semantic reference is the [CSS Flexbox specification](https://www.w3.org/TR/css-flexbox-1/#flex-property),
with Chromium as the executable oracle. V1 intentionally accepts only a subset:

* The qualified baseline requires equal computed grow and shrink factors. Each factor must be 0 or
  between 1 and 10,000. The `.riv` layout representation exposes one shared
  weight. `flex:2` means unequal factors and is rejected; use an explicitly
  supported declaration only when those equal-factor semantics are intended.
* Basis accepts nonnegative px, %, or auto. Percentages require a definite
  parent main size. The compiler tracks definite dimensions through fixed and
  percentage sizes, stretching and flexible sizing; uncertain cases fail.
* For positive factors, auto basis requires an explicit main-axis dimension
  (`width` in a row, `height` in a column). Content-derived auto basis is deferred.
* Inflexible explicit basis replaces the main dimension. Positive factors use
  native Fill, the shared fractional weight and native basis properties. Main
  dimensions and percentage bases remain responsive; they are not browser-baked.

These restrictions are backed by executable rejection cases in
[`validation/deferred-cases.json`](validation/deferred-cases.json). Browser
comparisons found sub-unit factor/gap differences, incorrect intrinsic text
distribution, and overflow paint ordering with indefinite percentage bases.
Their reproducers remain checked in instead of widening pixel tolerances.
This addition is a verified flex subset, not complete Flexbox support. In
particular, overlapping/overflowing nested content is not visually qualified.

L08 independent factors pass focused native validation and await full regression
qualification. The candidate accepts independently specified grow/shrink values
of 0 or [1, 10000], including `flex:2`, `flex:initial`, `flex:0 1 70px`,
longhands, variables and existing cascade keywords. Unequal factors require
version 9 runtime requirements and `layout-css-flex-factors-v1`; hosts must
validate and install the occurrence payload before layout. Authored dimensions
remain distinct from basis. Sub-unit factors, content-derived auto basis and
indefinite percentage basis remain excluded. The candidate is not yet a
fully qualified extension; see `validation/independent-flex-investigation.md`.


### Earlier unsupported-feature snapshot

#### Earlier intentionally unsupported list

* **CSS Grid**, including all grid properties. The future mapping can use the
  runtime layout engine; v1 never silently translates Grid into flex.
* General block/inline formatting beyond text-only block containers, margin
  collapsing, tables, lists, rich text, justified/logical text alignment, text
  decorations outside the solid underline/line-through subsets below, custom or
  multiline ellipsis, font fallback and vertical text. Explicit br and preserved
  source newlines are supported within their documented subsets.
* Independent grow/shrink factors, factors between 0 and 1, content-derived
  auto basis and percentage basis in indefinite containers. Auto
  margins and alignment values outside the accepted table remain unsupported.
* Absolute/fixed/sticky positioning, transforms, scroll-container overflow or
  scrolling, content-box, border strokes, per-corner radii, opacity, shadows,
  gradients, filters, background images and SVG.
* Functions other than `rgb`/`rgba`/`hsl`/`hsla` and the in-progress `var` support below (including `calc`), system colors, relative/viewport units, at-rules, media/container queries,
  pseudo classes/elements, attribute/sibling selectors and CSS nesting.
* Scripts, links, buttons, form controls, semantic accessibility export, actions,
  events, bindings, animations and state machines. A styled container can look
  like a button, but it has no button behavior.
* HTML/CSS formatting preservation, editor mutation/undo, incremental compilation
  and automatic integration with the existing editor publisher.
