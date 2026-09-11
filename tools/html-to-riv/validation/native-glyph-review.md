# Native glyph probe: diagnostic validation receipt

Status: successful bounded prototype, not production renderer integration.
The main acceptance suite remains 230 passing / 3 failing checks; A07 and Q09
remain partial. No compiler semantics, renderer defaults or tolerances changed.

## Reproduction

On macOS, after the standard validation dependencies and native binaries exist:

```sh
CARGO_INCREMENTAL=0 cargo build -p nuxie-html-to-riv --example probe
swiftc tools/html-to-riv/validation/native-glyph-probe.swift -o output/playwright/html-to-riv/native-glyph-probe
node tools/html-to-riv/validation/text-diagnosis.mjs 240
node tools/html-to-riv/validation/native-glyph-control.mjs 240
node tools/html-to-riv/validation/text-diagnosis.mjs 390
node tools/html-to-riv/validation/native-glyph-control.mjs 390
node tools/html-to-riv/validation/text-diagnosis.mjs 768
node tools/html-to-riv/validation/native-glyph-control.mjs 768
```

The runtime probe additionally exports `.glyphs.json` after drawing: actual
shaped glyph IDs, sizes, line positions, paint colors and world transforms.
This uses existing testing accessors and does not modify runtime code. Exact
draw-stream comparisons for normal-line-height-cascade and font-shorthand-
normal-resets at 390px confirm export leaves native drawing unchanged.

The Swift probe uses the same embedded font and those glyph coordinates with
CTFontDrawGlyphs. It does not shape text or measure browser layout. The control
runner deliberately accepts only one font and a white background; each compared
image is rendered natively before the unchanged pixel comparator reads it.

## Result

All **12 smoothed-glyph controls pass**: H, Hg, normal-line-height-cascade and
colored/translucent text at fractional positions, each at 240/390/768px. The
largest local interior RGB error among these controls is 3.18899 (limit 6).
Hg's error is 1.51148 at all widths, versus 6.59439 in the vector-path pipeline.
The unsmoothed native Hg control still fails at 6.02296, isolating a smoothing
contribution with the same native font engine and exported glyph positions.

DOM/native contact sheets for Hg, the normal cascade and colored/alpha text
were visually inspected at all three widths. Baselines, wrapping and colors
agree within the unchanged thresholds. Reference: Chromium 153.0.8010.12,
DPR 1. No browser text or path image was substituted into native output.

These scripts report diagnostic comparisons; successful process exit means the
report was produced, not that every mode passed. Inspect native-glyph-results.json
in the width-specific text-diagnosis output directory.

## Remaining work

This proves a promising native glyph boundary, not a portable mask renderer.
The probe currently paints onto opaque white and directly applies CoreText
smoothing. Next test transparent glyph masks composited by the real renderer
onto different backgrounds, including foreground alpha and subpixel positions.
CoreText mask processing may depend on foreground/background and platform
settings; an opaque probe does not prove atlas compositing correctness.

Only after that control passes should the runtime/renderer preserve optional
glyph-run data through its production and recording interfaces. Font identity,
cache/resource lifetime, clips, transforms, capability/profile selection and
compatibility with existing vector rendering must be explicit. Compiler output
must retain text and responsive layout; this diagnostic rasterizes after native
layout and is not authorization to bake scene images during compilation.

Transparent-mask composition is now tested separately through actual Metal
image drawing: all 48 controls pass. See glyph-mask-review.md. Full-frame
experimental masks still need a production glyph/run-sized integration.
