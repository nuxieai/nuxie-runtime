# Transparent glyph-mask composition: experimental gate

Command, after the standard npm/Chromium setup:

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run-glyph-mask-controls.sh
```

Result: **48/48 comparisons pass**, 16 at each of 240/390/768px. The wrapper was
run successfully after the individual controls; failures produce a nonzero
exit status. This is an experimental rendering regression gate, not a replacement
for the main compiler acceptance suite, which remains 230/233 passing.

The Swift probe now supports mask-smooth and mask-plain modes, clearing its
bitmap to transparent instead of painting onto white. The same embedded font
and runtime-exported glyph IDs, positions, sizes, colors and transforms feed
CoreText. The resulting transparent PNG is decoded through the real recording
factory, drawn as an image, and composited by the native Metal renderer.
No browser image or browser-derived coordinates enter native rendering.

Four samples (H, Hg, normal-line-height-cascade, colored/translucent text at
fractional positions) are compared on four independently rendered backgrounds:
white, #111827, #eeeeff and #369966. Each mask is reused unchanged across those
backgrounds. Browser text renders against the corresponding target surface.
All comparisons use the existing pixel thresholds. The worst interior RGB error
is 3.18899 (limit 6), normal-line-height-cascade at 240px on white.

Browser/native contact sheets covering Hg on white, the normal cascade on
lavender, and colored/alpha text on dark and green backgrounds were inspected
at all three widths. Baselines, wrapping, colors and composition agree within
the unchanged limits. Reference: Chromium 153.0.8010.12, DPR 1. Native composition:
Rust Metal clockwise-atomic through renderer-replay.

## Production boundary still open

The experiment rasterizes full-frame images after native layout, then sends
those images through the renderer. It does not change the compiler output and
is not a proposal to bake scene images at compile time. Shipping support still
needs glyph/run-sized masks, bounded caching, font-resource lifetime, clips,
transforms, scaling and an explicit optional rendering interface that retains
glyph data before it becomes a combined vector path. Existing vector rendering
must remain compatible. The compiler must stay outside the runtime dependency
closure.

Next: implement the optional glyph-rendering adapter and its cache/resource
contract, then run the unchanged full compiler corpus and all Q09 specimens
through that integrated path. These experimental passes do not qualify A07/Q09
or waive the original failures. No external blocker has been identified.
