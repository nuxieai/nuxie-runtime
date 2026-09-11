# Live runtime glyph adapter

The optional glyph API is now connected to actual runtime Text drawing through
`nuxie-renderer::glyph_adapter::GlyphCache`. The compiler still emits semantic
Rive text and embedded fonts. Native layout and shaping run after import and
resize; the renderer receives the settled glyph IDs and baselines directly.
There is no glyph JSON export, subprocess or browser-derived layout in this
path. The original vector renderer remains the default.

## Results and reproduction

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
# After that build, run the unchanged realistic specimens:
cd tools/html-to-riv
NUXIE_NATIVE_GLYPHS=1 NUXIE_HTML_KNOWN_GAPS=1 \
  NUXIE_HTML_REVIEW_DIR=../../output/playwright/html-to-riv/live-glyph-known-gaps \
  npm test -- --output=../../output/playwright/html-to-riv/live-glyph-known-gaps/artifacts browser.spec.mjs
```

The full experimental lane passes **233/233** checks. Its full gate also passes
29 compiler Rust tests, native/WASM byte and source-map parity, five JS/WASM
tests, TypeScript, and two gallery checks. The native resource/adapter suites
now have ten passing tests (the final live-runtime fallback regression was
added and run separately after the full gate). The unchanged known-gap corpus
passes **9/9** comparisons: membership card, typography specimen and Hg at
240/390/768px. Geometry and pixel limits are unchanged.

Cache tests cover reuse across frames and integer translations, subpixel/color
misses, nested save/restore, rejection without destination mutations, eviction
and font ownership. A real imported scene verifies solid text uses glyph masks,
while multiple paints and even-odd fill retain vector drawing. The even-odd
regression failed before adding its eligibility guard. The rasterizer's five
tests cover bounded masks, fractional positioning, alpha/color, invalid input
and empty ink. Default vector streams for the normal cascade, shorthand resets
and colored/translucent fractional text remain byte-for-byte identical to the
pre-integration exports. The pure-runtime boundary and module all-target Clippy pass. The full 233
comparisons were rerun after the even-odd guard and passed again.

Browser/native/difference sheets for the two originally failing fixtures,
membership card and typography specimen were inspected at all three widths.
Wrapping, alignment, baseline placement and colors agree; the remaining edge
differences stay inside the existing tolerances. Full-gallery evidence is in
`output/playwright/html-to-riv/live-glyphs-qualified/gallery.html`; the compact
sheets are `output/playwright/html-to-riv/live-glyph-review-{240,390,768}.png`.
The gallery explicitly labels the renderer profile; a passing glyph-profile
report must not be presented as a passing vector-profile report.

## Installation and compatibility

Enable `nuxie-renderer/native-glyphs-experimental` on macOS. Construct a
`GlyphCache` with the same persistent image factory used by the scene, keep it
across frames, and wrap each fresh renderer at identity before drawing:

```rust,ignore
let mut glyph_cache = GlyphCache::new(factory.clone());
// For each frame, after preparing the backend:
artboard.draw(&mut glyph_cache.wrap(&mut renderer));
```

The cache belongs to one factory/device and cannot be reassigned. It retains at
most 256 entries and 32 MiB of conservatively counted font, key and pixel data.
Individual masks keep the rasterizer's 16 MiB and 8192px limits. It retains the
font Arc while pointer identity is cached, preventing allocation reuse from
aliasing font keys; eviction releases images and font references. Integer world
translation is separated from fractional phase. Rasterization and decoding
finish before destination drawing, so declined requests fall back intact.

The runtime admits one solid fill per style, nonzero/clockwise fill rules,
ordinary visible/clipped text, one static HbFont/size per style and no glyph
modifiers or color-glyph commands. Strokes, gradients, multiple fills, even-odd
fill, feather/path effects, variation axes, mixed font instances, other text
overflow modes and unsupported resources retain vector drawing. Existing
renderers decline the optional interface and keep their prior draw stream.

## Remaining qualification

Update: representative affine/clip/opacity/restoration controls are now covered
by [glyph-state-review.md](glyph-state-review.md), including two fixes. The
remaining broader profile limits below still apply.

The wrapper forwards clip and opacity state and restores transforms around mask
composition, with the dedicated 27-case host-state corpus now passing. Font parsing currently repeats on
cache misses; cache reuse is tested, not yet a performance benchmark. This
implementation is macOS only. Other native targets and browser renderers retain
the vector path and its platform rasterization differences.

A07 therefore remains partial across rendering profiles: syntax and the macOS
glyph lane pass, while the vector lane's three recorded failures remain valid.
Q09's three existing specimens pass on this lane; pricing/onboarding composition
coverage is still pending. No corpus member or tolerance was removed or relaxed.
Next: qualify adapter state handling, then resume A09 and the ordered compiler
backlog using explicitly identified renderer profiles.
