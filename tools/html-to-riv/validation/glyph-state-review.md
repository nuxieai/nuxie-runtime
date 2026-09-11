# Glyph renderer state validation

The native-glyph lane now includes `glyph-state-control.mjs`. It compiles one
scene at 390px, resizes that same scene to 240/390/768px, and exercises nine host
states: identity, fractional translation, a clip cutting through glyphs,
modulated opacity, uniform scale, nonuniform scale, rotation, shear and a
combined clip/rotation/opacity case. Every state is followed by an unmodified
copy after restore. Browser references apply equivalent CSS around the source
scene. Native rendering imports the Rive scene and invokes the live adapter;
no browser geometry is passed into the runtime.

These are host renderer controls, not new CSS compiler features. CSS transforms,
clipping and group opacity remain subject to their own backlog items.

## Two defects found and fixed

The first run passed 21/27 comparisons. Fractional translation and nonuniform
scaling failed at all three widths despite geometry agreeing. Pixel row
measurements showed a half-pixel baseline difference. The opt-in rasterizer now
rounds the final device baseline for axis-aligned horizontal glyphs, retaining
horizontal subpixel position and native layout. This follows the orthogonal
coordinate rule in [Skia's glyph position rounding implementation](https://skia.googlesource.com/skia/+/2daf164f6d51/src/core/SkGlyph.cpp).
It reduced the affected glyph region's interior RGB error from 7.654 to 2.266
and from 6.620 to 2.423 (limit 6). A native test verifies rounding happens after
scale, while existing tests retain horizontal phase and translation invariance.

Visual review then caught a second defect even though the existing aggregate
thresholds passed: opacity 0.5 text was visibly too faint. The exact renderer's
image-as-path branch computes final opacity in drawImage and applies modulated
state again in drawPath. The glyph adapter now temporarily normalizes that state
and supplies the intended opacity once to image drawing. Save/restore retains
the surrounding state. Zero opacity consumes no image allocation or draw.
The default exact renderer implementation is unchanged.

A white-background ink-coverage comparison now supplements the shared pixel
metrics for these controls. Per-region native/reference coverage must be within
15%, admitting independent antialiasing while rejecting the roughly halved ink
of the defective opacity path. This is a stricter additional check; no existing
geometry or pixel tolerance changed. It failed six opacity/combined controls
before the correction. After correction, opacity's interior RGB error is 1.129
and ink ratio 1.070; the combined case is 0.490 and 1.076. A native test also
checks nested 0.5 modulation, restoration, explicit image opacity and cache reuse.

Before-fix artifacts are retained in `output/playwright/html-to-riv/`
`glyph-state-controls-before-snap` and
`glyph-state-controls-before-opacity-normalization`.

## Results and reproducibility

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native-glyphs
# State controls alone after the feature-enabled probe/replay build:
node tools/html-to-riv/validation/glyph-state-control.mjs
```

State controls pass **27/27**, including geometry, shared pixels and the added
ink check. Twelve native tests pass (six rasterizer, six adapter). Browser/native/
difference sheets for all nine states were visually reviewed at all three
widths; clipped boundaries, scaled/rotated text, opacity and restored copies
agree. The command generates `results.json`, self-contained per-width review
HTML, full contact sheets and three readable contact-sheet pages per width in
`output/playwright/html-to-riv/glyph-state-controls/`.

The 48 colored-background mask comparisons pass after baseline rounding; the
nine existing realistic specimens also pass. The final full native-glyph gate passes 233/233 original checks plus all 27
state controls, 29 compiler Rust tests and twelve native tests. Native/WASM
parity, five JavaScript tests, TypeScript, two gallery checks, module Clippy and
the pure-runtime boundary also pass. The state corpus is part of that command.

## Limits and next work

The adapter must wrap a fresh renderer with identity transform and unit
modulated opacity. Its scoped normalization covers glyph masks only. P05 must
still examine ordinary image opacity and overlapping descendants when adding
CSS group opacity; these non-overlapping text controls do not qualify group
compositing. Perspective, extreme transforms, variable/color fonts and other
platform backends are not qualified here. Default vector rasterization gaps
remain separately recorded. Next: A09 (`em` lengths), with font-size resolved
against the parent and other lengths against the final element font size.
