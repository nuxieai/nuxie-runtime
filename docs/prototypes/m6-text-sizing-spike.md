# M6 Text Sizing Spike

Date: 2026-07-04

## Why This Is First

M6 currently has 124 parked corpus entries. The largest single diagnostic
bucket is text:

- `rust-runner-unsupported:text`: 59
- `rust-runner-unsupported:images`: 27
- `rust-runner-unsupported:nested-artboard-layout`: 18
- `rust-runner-unsupported:layout-component-paint`: 15
- Smaller buckets: scroll constraints, scripted transition conditions, feather,
  n-slice, focus data.

Text is also the first blocker for the remaining files that previously looked
like M5 data-binding work. This makes text the right opening M6 subsystem.

## C++ Surface

The C++ text implementation is not one feature. It is a stack:

- `src/text` has 31 source files, `include/rive/text` has 30 headers, and the
  combined text surface is about 11k lines.
- Import and lifecycle:
  - `TextValueRun::onAddedClean` registers the run on its owning `Text`.
  - `TextValueRun::onAddedDirty` resolves `styleId` to `TextStylePaint`.
  - `TextStyle::import` registers a font asset referencer.
  - `TextStyle::onAddedClean` finds the owning text and creates
    `TextVariationHelper` only when axes/features exist.
- Styled text assembly:
  - `StyledText` concatenates UTF-8 runs into `Unichar` text plus `TextRun`
    records carrying font, size, line height, letter spacing, and style id.
- Shaping and font access:
  - C++ uses `HBFont` over HarfBuzz and SheenBidi.
  - `Font::shapeText` produces `Paragraph` and `GlyphRun` records.
  - `Font::getPath(glyph)` converts glyph outlines into Rive `RawPath`.
  - Color glyph support is separate and should stay out of the first slice.
- Line layout:
  - `Text::BreakLines`, `GlyphLine::BreakLines`, and
    `GlyphLine::ComputeLineSpacing` turn shaped runs into lines.
  - `OrderedLine` reorders runs into visual order for bidi and ellipsis.
- Draw/build:
  - `Text::buildRenderStyles` computes bounds, optional clipping, line order,
    and style paths.
  - Monochrome text draws as one or more `TextStylePaint` paths.
  - Color glyphs are interleaved through `TextDrawCommand::kColorGlyph`.
- Editing/input:
  - `RawTextInput`, cursor movement, selection paths, keyboard input, and
    scroll interaction are a separate later track.

## Rust State Today

Rust already imports and projects a useful amount of text structure:

- `rive-binary` validates text parentage and tracks text-style asset references.
- `rive-graph` projects `Text`, `TextStylePaint`, `TextValueRun`, text style
  variation helpers, text follow-path dependencies, and shape-paint containers
  for `TextStylePaint`.
- `rive-render-api` already has `RawPath`, render path creation, path builders,
  and recording output compatible with the C++ golden stream.
- `rive-runtime` now has a private text runtime module for the first static
  embedded-font tracer.
- `tools/rust-golden-runner` currently gates any artboard-local `Text` with
  `unsupported: text`, and gates nested child data binds targeting `Text`,
  `TextValueRun`, or `TextStylePaint` as text.
- No text/font dependencies are currently declared in the workspace manifests.

## Corpus Tracer

Use `hello_world.riv` as the first text implementation target.

Reason:

- It is already M6 `text`.
- The direct Rust runner stops at `unsupported: text in Rust golden runner
  (global 4)`.
- The graph projection is small and already complete enough:
  - `FontAsset` global 1.
  - `Text` global 4.
  - `TextStylePaint` global 5.
  - `TextValueRun` global 7.
  - One text style fill with `SolidColor` `0xff00f3ff`.
  - Drawable order contains only the text drawable, plus the artboard
    background fill.
- The C++ golden stream is static:
  - normal artboard/background setup,
  - a transform near `[1,0,0,1,73.4804688,175.785156]`,
  - one large `drawPath` for the text style path,
  - no nested artboard, listener input, text modifiers, data binding, layout
    measurement, or color glyphs.

`align_target.riv` is the first M6 file by manifest order, but it is not the
right first implementation slice: it includes listener align-target and text
modifier/axis objects. Keep it as a follow-up after the static text path is
working.

## First Slice

Goal: promote `hello_world.riv` from `unsupported-feature/text` to exact.

Support only this subset:

- top-level `Text` drawable, not nested child text binding;
- `TextValueRun` children with resolved `TextStylePaint`;
- `TextStylePaint` with visible solid fill;
- `FontAsset` with available font bytes through existing file-asset import
  data;
- static sample-0 draw;
- no `TextInput`;
- no text modifiers;
- no `TextStyleAxis` or `TextStyleFeature`;
- no follow-path text;
- no color glyphs;
- no layout measurement or text as a layout child;
- no text data binding.

Implementation shape:

1. Add a private text runtime module, probably `crates/rive-runtime/src/text.rs`.
   Keep the first API internal to `rive-runtime`; do not design the final
   public text API yet.
2. Add the font/shaping dependency stack from `docs/porting-map-v2.md`
   (`harfrust` over fontations) behind a very small adapter:
   - decode a `FontAsset` payload into a runtime font,
   - shape `StyledText` into paragraph/run/glyph data,
   - expose glyph outlines as `RuntimePathCommand` values.
3. Port the smallest C++ data shapes needed by static text:
   - `RuntimeStyledText`,
   - `RuntimeTextRun`,
   - `RuntimeGlyphRun`,
   - `RuntimeParagraph`,
   - `RuntimeGlyphLine`.
4. Port only the line/bounds code required for `hello_world`:
   - auto/visible text sizing used by the fixture,
   - line spacing,
   - visual line iteration without ellipsis,
   - style path aggregation.
5. Extend `ArtboardInstance::draw_commands` / static draw preparation so a
   supported `Text` drawable emits the same headless path command stream as a
   shape paint draw.
6. Narrow the Rust runner text gate:
   - allow the supported static text subset,
   - keep `unsupported: text` for modifiers, axes/features, `TextInput`,
     follow-path, nested child text binds, color glyphs, and layout-coupled
     text.
7. Promote only `hello_world.riv` after direct C++/Rust stream comparison.

Expected metric movement for that slice:

- `exact`: +1
- `exact-segments`: +1
- `unsupported-feature`: -1
- M6 parked queue: -1

## Result

`hello_world.riv` landed 2026-07-04 through a deliberately narrow static text
path: one top-level `Text`, one `TextValueRun`, one solid-fill
`TextStylePaint`, embedded `FontAsset` bytes, no modifiers/layout/input/data
binding/color glyphs.

`ellipsis.riv` also landed 2026-07-04. It widened the slice to static
`TextStyleAxis` variations and the smallest one-run fixed-height
wrap/ellipsis path needed by the corpus. `hosted_font_file.riv` landed next:
the C++ golden runner imports without a `FileAssetLoader`, so a hosted
`FontAsset` with no in-band contents resolves successfully but has no decoded
font and draws only the text drawable save/restore wrapper.

A follow-up scan showed `new_text.riv` is still too broad for the next slice
(five texts plus multi-run/style, gradient/stroke, clipping, and keyframed
text). Use `animated_clipping.riv` next: it has one text and now isolates
sibling Shape/ClippingShape admission around text draw order.

## Follow-Up Order

After `hosted_font_file.riv`, widen in this order:

1. Sibling Shape/ClippingShape draw-order cases around one supported Text,
   starting with `animated_clipping.riv`.
2. Text sizing/overflow/trim cases that only need the same line breaker.
3. Multi-text or multi-run static cases without modifiers or layout coupling.
4. `TextStyleFeature` and broader variable-font helper behavior.
5. Text modifiers and follow-path text.
6. Text participating in layout measurement.
7. `TextInput`, selection/cursor paths, and focus/input behavior.
8. Color glyphs and image-backed font/color glyph paths.

## Stop Rules

Do not let the first text slice expand into the whole C++ text runtime.

If a candidate requires any of these, leave it parked with `text` or a narrower
M6 diagnostic:

- text editing;
- selection or cursor paths;
- input focus;
- text layout provider behavior;
- scroll constraints;
- variable-font behavior beyond the static `TextStyleAxis` path already proven
  by `ellipsis.riv`;
- modifiers;
- follow-path;
- color glyphs;
- nested child text data binding.
