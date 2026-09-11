# A20 solid underline: Chromium reference investigation

Research date: 2026-09-08. Reference browser: Chromium 153.0.8010.12.
This is implementation research, **not visual qualification**. No compiler or
runtime code changed as part of this note. Chrome remains authoritative;
Firefox is not a substitute oracle.

## Primary sources

The Chromium sources below were read at the **exact browser version tag**.
This matters: current main uses a different skip-ink dilation limit.

- [Thickness, decoration origins and underline resolution](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/paint/text_decoration_info.cc)
- [Underline positioning](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/layout/text_decoration_offset.cc)
- [Glyph-intersection clipping](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/paint/text_painter.cc)
- [Underline paint dispatch](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/paint/text_decoration_painter.cc)
- [Propagation boundaries](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/css/resolver/style_adjuster.cc)
- [Accumulated applied decorations](https://chromium.googlesource.com/chromium/src/+/153.0.8010.12/third_party/blink/renderer/core/style/computed_style.cc)
- [CSS Text Decoration Level 3](https://www.w3.org/TR/css-text-decor-3/#line-decoration)
- [CSS Text Decoration Level 4 draft](https://drafts.csswg.org/css-text-decor-4/)

## Thickness and vertical position

`ComputeDecorationThickness` uses `used_font_size / 10` for `auto`.
`from-font` uses `UsedFont::UnderlineThickness`, falling back to that automatic
value if the metric is unavailable. Explicit lengths/percentages are resolved
against used font size and rounded with `roundf`. `ComputeThickness` clamps
ordinary non-SVG text to at least one CSS pixel. Automatic thickness itself is
not rounded here. The decorating box supplies the font when available.
For horizontal alphabetic text, `auto` underline position resolves near the
alphabetic baseline; `from-font` and `under` select distinct paths. These are
findings from the pinned thickness source, not assumptions about all browsers.

`ComputeUnderlineOffsetAuto` returns integer font ascent (scaled), plus a gap,
plus rounded CSS underline offset. With offset `auto`, gap is
`max(1, ceil(thickness / 2))`; with an explicit offset, gap is zero. This offset
is relative to the font-top origin, so the final renderer must account for line
baseline placement rather than adding ascent twice. `from-font` positioning
instead rounds scaled `(float_ascent + font_underline_position)` plus specified
offset; absent metrics fall back to automatic positioning. Percentage offsets
resolve against computed font size and font scaling. `under` uses the bottom of
the em box and a one-pixel gap. These rules come from the pinned positioning
source. `text-decoration-thickness: from-font` does **not** itself select
`text-underline-position: from-font`.

## Propagation is separate from CSS inheritance

The Level 3 model makes decoration-line/style/color non-inherited properties.
An ancestor's applied underline can still reach descendant text. `none` adds
nothing; it does not cancel ancestor decoration. The originating decoration's
color and style remain associated with it. Decorations pass through in-flow
block children and ordinary flex containers/items, but stop at out-of-flow
boxes and atomic inline boundaries. A flex container is not an inline-flex
atomic boundary merely because it uses flex layout.

Pinned `StopPropagateTextDecorations` tests atomic inline display, floating,
out-of-flow positioning, media shadow boundaries, outer SVG, and ruby text.
Its nearby comment mentions relative positioning, but the actual predicate is
`HasOutOfFlowPosition`; relative positioning should be probed, not rejected on
the strength of that comment. Style adjustment obtains ancestor applied data
from the layout parent. Thus compiler propagation should follow the generated
box tree, with explicit absolute-position reset, rather than blindly clone a
single inherited boolean.

Pinned `EnsureAppliedTextDecorationsCache` copies existing applied decorations,
then appends the element's own line/style/color/thickness/offset data.
Multiple origins therefore need separate records: a child adding blue underline
must not replace an already propagated red one. An explicit `inherit` value
still inherits the property through ordinary cascade, potentially introducing
another local decoration. Font/metric selection across block and flex boundaries
needs screenshot probes before choosing which source node supplies metrics.

## Skip ink is required for the default

Level 4 defines inherited `text-decoration-skip-ink`, initially `auto`.
The pinned painter reads skip-ink from the target text style, then clips before
painting each underline. A descendant's skip-ink value can therefore affect
an ancestor-origin underline on that text.

Pinned `TextPainter::ClipDecorationLine` returns immediately for `none`.
Otherwise it reduces decoration bounds vertically by 0.5px on each edge to
ignore shallow intersections, obtains glyph-outline intercepts in that stripe,
and clips those intervals. Default `auto` excludes CJK from interception.
The clip rectangle expands horizontally by `min(thickness, 13px)` and vertically
by 1px. **The pinned limit is 13px; main currently says 5px.** A glyph bounding
box approximation will over-cut letters, and a continuous rectangle will
incorrectly cross descenders. The runtime should obtain actual outline-stripe
intersections or retain an explicit qualification gap; never silently treat
default auto as none.

## Implementation decisions and qualification probes

These are recommendations to test, not claims of implemented support:

1. Start the public syntax with solid `underline`/`none`, explicit color and
   supported thickness/offset forms. Use distinct local-style and propagated
   decoration records. Document which longhands and shorthand forms are accepted.
2. Build underline segments from runtime line shaping after each resize. Do not
   emit browser-measured fixed rectangles. Keep leading/trailing spacing trimming,
   forced breaks, tabs, and alignment associated with their actual line runs.
3. Probe at 8, 10, 13, 16, 20, 24 and 40px font sizes, including fractional sizes;
   compare auto, from-font, 0px, 0.5px, 1.5px, 2px, 0.1em and 10% thickness.
   Use Inter and a second bundled font with different underline metrics.
4. Isolate baseline/offset behavior using uppercase text without descenders,
   then `gypqj` for skip-ink, auto versus none, and large thickness near the
   13px cap. Include tight letter-spacing, ligatures and combining marks.
5. Compare parent underline across block, flex, nested flex, relative and
   absolute descendants. Change child color/font-size, set child none, add a
   second colored underline, and use explicit inherit. Record computed CSS but
   assess actual pixels: computed text-decoration-line alone misses propagation.
6. Exercise short/long wrapped lines, pre/pre-wrap/pre-line, leading/trailing
   whitespace and tabs, `<br>`, empty lines, centered/right alignment, clipping,
   opacity and transforms. Repeat widths 240/390/768 on the same compiled scene.
7. Preserve ordinary text geometry and inspect dedicated underline-region ink
   metrics in addition to whole-image differences. Thin missing lines can be
   hidden by aggregate pixel tolerances. No visual tolerance changes are proposed.

Rendering, propagation, font-origin choices and line-end trimming remain
unqualified until the pinned browser/native fixtures establish them. Source
inspection supplies a starting algorithm, not a replacement for those checks.


## Pinned-browser measurements

The independent `npm run test:underline-reference` captures 48 combinations:
Inter/Open Sans, 16/20/24/32px, auto/from-font/2px thickness, skip-ink none/auto.
Text is transparent with red decoration to isolate underline pixels; a zero-size
inline baseline marker records the browser baseline. The oracle records exact
red-ink intervals per scanline, not only aggregate error, so moving a skip-ink
gap cannot pass merely by keeping the same ink count. The 48 reference images
were visually inspected in six contact sheets.

At Inter 20px, auto paints rows baseline+1 and +2; from-font paints baseline+1.
At 24px, auto paints baseline+2 and +3, while from-font paints baseline+1.
Skip-ink auto cuts real gaps in both fonts. These raster observations validate
the initial source-derived rules but do not yet qualify native underlines.

A separate four-case probe (underline-propagation-reference.json) observes that
both block and flex ancestors propagate red underline through child `none`,
despite the child's computed text-decoration-line remaining none. A child blue
underline paints over red at coincident positions. This does not prove red was
removed: distinct offsets must test multiple simultaneous origins.

## Runtime integration required

Current TextStylePaint/TextStyleBackground wire records have no underline
metrics, offset or skip-ink field. TextStyleBackground only has a corner radius
and builds full glyph selection rectangles; it is not an underline substitute.
The runtime already holds ordered, shaped lines after each layout and exposes
both native-glyph and vector drawing paths. Decoration geometry should derive
from those lines after resize, and be drawn in both paths with the same clip,
transform, opacity and blend state. Adding lines to glyph paths alone would
lose them when the native glyph optimization consumes a text style.

Keep raw Rive rendering unchanged. An explicit occurrence configuration, carried
by a versioned, validated compiler runtime manifest, is the current candidate
for transporting decoration origins and parameters. It must compose with
existing whitespace policies, survive font replacement and be refused by hosts
without support. Resolve the exact interface before accepting CSS syntax.

Next implement runtime solid-line drawing with measured thickness/offset and
outline-stripe skip-ink, then wire compiler propagation and manifest installation.
Required qualification remains public compilation, malformed/unsupported inputs,
native/WASM parity, same-scene resizing, Chrome geometry/pixels and visual review.
No unsupported default should silently render as skip-ink:none.

Artifacts: output/playwright/html-to-riv/underline-reference/report.json,
individual PNGs and contact-0.png through contact-5.png. Reference source:
validation/underline-reference.json. Logs: /tmp/html-underline-reference.log.


## Runtime outline interception implemented

`text/css_decoration.rs::glyph_stripe_intercept` now supplies the opt-in geometry
primitive for skip-ink. It is not connected to ordinary text drawing yet.
Chromium's DEPS pins Skia 9d07e5bad9e3e21da2426946e589daa647218271;
[that revision's SkGlyph.cpp](https://skia.googlesource.com/skia/+/9d07e5bad9e3e21da2426946e589daa647218271/src/core/SkGlyph.cpp)
forms one interval per glyph from band-boundary intersections and control points
strictly inside the band. This is distinct from both full glyph bounds and an
arbitrary contour-by-contour gap list.

The implementation handles lines, implicit closing edges, quadratics and cubics.
Curve roots are found on y-monotonic intervals using derivative roots and bounded
bisection, without flattening curves. Empty/disjoint/nonfinite paths and reversed
bands produce no gap; zero-height bands remain meaningful for thin underlines.
Ordinary Rive rendering has no call to this opt-in function and is unchanged.

Five runtime integration tests pass. The recorded red/green sequence first
exposed missing closed-edge intersections, then missing quadratic/cubic
crossings. Additional tests cover three-crossing cubics, disjoint contours,
invalid inputs and actual Inter descenders. For the last test, the measured
Chrome 20px auto-underline gaps are [10,31), [34,39), [53,63); geometric edges
from native font outlines agree within one pixel of those rasterized edges.
This geometry-to-pixel-boundary check is not a replacement for the existing
0.1px layout or image thresholds, and does not qualify final underline rendering.

Tests: `NUXIE_TEXT_TEST_FONT="$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf" cargo test -p nuxie-runtime --test css_decoration`. This suite is now part
of the normal compiler validation runner. Logs: /tmp/html-underline-intercept-
{red,line,curve-red,curve-green,extended}.log. Next connect line-aware decoration
paths to runtime drawing, implement skip-ink selection/CJK behavior and manifest
transport, and qualify both glyph/vector rendering against Chrome.


The boundary checker caught an initial test-only cross-package include_bytes!
for the Inter fixture. This was removed; the integration test now uses the
existing NUXIE_TEXT_TEST_FONT convention, installed by run.sh. No compiler asset
is embedded into the runtime package. The boundary gate is rerun after the fix.


The final geometry regression also verifies the supplied Inter bytes by SHA-256,
so a different font cannot silently become the reference. Five tests pass after
using the input-font seam. Boundary validation passes. Runtime Clippy is checked
for new-file findings; existing runtime warnings are not treated as newly clean.
The final lint cleanup replaces unchecked slice indexing with pattern matching;
no rendering behavior is enabled by this change.


## Resolved runtime draw integration

`Text::set_css_underlines` now accepts validated resolved solid line records.
Empty is the default, and clearing the records restores the original drawing.
It invalidates shaping so paths are rebuilt for the current visual lines;
clearing render styles also clears underline paths. Each line rectangle loses
merged outline-intersection intervals according to None/Auto/All skip-ink.
Auto uses the pinned Chromium predicate documented in
underline-skip-eligibility-research.md, including the exclusions that are not
expressible as a broad CJK script check. The production table retains the
Chromium BSD attribution. The runtime API is checked against all 481 reference
boundaries, including None/All controls.

Underlines draw after backgrounds and before text glyphs, using the text world
transform, inherited render opacity and blend mode. The path stays separate
from text-style glyph paths, so the native glyph fast path cannot drop it.
The import test first failed because no line was drawn, then passed after
connecting the runtime path. It now resizes the same imported scene at
768/390/240, verifies line counts and descender gaps, verifies actual native
glyph rasterization and decoration-before-glyph order, and clears decoration
to recover the initial recording exactly. This tests installation after import;
it does not pretend the compiler encoded an underline in the Rive bytes.

Commands and logs:

- `cargo test -p nuxie-html-to-riv --features native-glyph-controls`: 78 tests
  pass (`/tmp/html-underline-draw-suite.log`).
- Expanded test: `cargo test -p nuxie-html-to-riv --features native-glyph-controls resolved_underlines_`
  passes (`/tmp/html-underline-draw-expanded.log`).
- Pinned boundary data: `validation/underline-skip-eligibility-reference.json`.

Remaining before accepting CSS: propagated independent decoration origins,
source-font thickness/offset resolution, trailing/hanging whitespace behavior,
validated/versioned transport and host installation/refusal, native/WASM parity,
and final Chrome geometry/pixel comparisons in both renderer profiles. Native
image equality, clipping/opacity/transformed decoration pixels and font-metric
fallback are not established by these recording tests. Keep A20 active.


The six runtime integration tests (including invalid thickness/offset inputs)
pass: `/tmp/html-underline-runtime-suite.log`. The pure-runtime boundary gate
passes: `/tmp/html-underline-draw-boundary.log`. Module Clippy passes with
`--all-targets --no-deps -- -D warnings` after removing an unnecessary struct
update in the import test: `/tmp/html-underline-draw-clippy-final.log`.
Runtime Clippy identified one new collapsible conditional in path construction;
it was simplified without changing the eligibility condition. Existing runtime
lint debt remains separate from compiler lint acceptance.


## Transport and initial pixels follow-up

Version 3 transport, checked host installation/refusal and the first native pixel
controls are now implemented. See underline-runtime-review.md for the exact
schema, tests, 24-case receipt and preserved vector failures. Earlier notes
listing transport/drawing as pending describe previous stages. CSS origin
propagation and metric resolution remain pending; underline syntax is rejected.


## CSS emission follow-up

CSS longhands/shorthand, propagated origins and metric resolution are now wired
through version 3. The 20-case origin reference refines the early decorating-box
notes: auto/percentage/from-font thickness uses the child text font across the
supported block/flex boundaries; computed em lengths stay fixed from the origin.
See underline-css-review.md for the 51/51 native-glyph, 49/51 vector receipt and
remaining edge cases. A20 remains active.
