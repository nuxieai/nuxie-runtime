# Text rasterization: isolated failure and next implementation seam

Status: investigation evidence, not feature qualification. A07 remains partial;
Q09 remains open. Main acceptance is still 230 passing / 3 failing checks.
No compiler, runtime, renderer or acceptance-policy change was made in this
investigation.

## Reproduce the control

Run from the repository root after building the compiler and native probes:

```sh
node tools/html-to-riv/validation/text-diagnosis.mjs
node tools/html-to-riv/validation/text-path-control.mjs
```

The second command parses the recorded native draw stream for four bounded
scenes and replays the exact verbs, control points, transforms, clips and solid
fills in Chromium Canvas. It rejects unsupported commands and paint effects.
This is not a general stream player or a replacement browser reference. Both
scripts emit reports under output/playwright/html-to-riv/text-diagnosis; their
successful execution means measurements were produced, not that every measured
comparison passes. Inspect the failure arrays.

All four Canvas-path versus native comparisons pass. DOM text versus Canvas
paths fails on Hg and both normal-line-height scenes, entirely within Chromium.
Measured interior RGB errors at 390px (unchanged limit 6):

| Scene / region | Canvas paths vs native | DOM text vs Canvas paths |
| --- | ---: | ---: |
| H / text | 0.88690 | 4.31944 |
| Hg / text | 2.79847 | 6.77296 |
| font-shorthand-normal-resets / a | 3.42806 | 6.27437 |
| normal-line-height-cascade / a | 3.49252 | 6.29365 |

The DOM/Canvas/native contact sheet was visually inspected. The main acceptance
fixtures, reference stylesheet and tolerance values remain unchanged. This
control narrows the issue toward font-specific rendering rather than a generic
native shape coverage failure; it does not identify every remaining source of
pixel error or prove a replacement renderer.

## Existing runtime seam

* `font_hb.rs::glyph_path` draws an unhinted outline at STANDARD_SCALE.
* `text.rs::build_render_styles` transforms those outlines per glyph and combines
  them in TextStylePaint paths.
* `text_style_paint.rs::draw` uses the ordinary shape paint path, including its
  opacity paint pool.
* `nuxie-render-api::RenderPaint` has fill/stroke/color/feather/blend/shader
  controls, but no glyph rasterization mode. `Renderer` receives paths/images;
  it does not receive font instances, glyph IDs or font sizes for these draws.
* The current schema exposes no font smoothing/hinting/rasterization property.

Consequently, another line-height formula or a general shape-color adjustment
cannot express a font rasterization policy through the existing compiler
mapping. The compiler should retain semantic text and font assets. Modifying
font outlines, baking browser rectangles or rasterizing whole layouts during
compilation would evade the intended runtime-responsive compiler contract.

Skia's [macOS font implementation](https://raw.githubusercontent.com/google/skia/main/src/ports/SkTypeface_mac_ct.cpp)
distinguishes un-dilated antialiased outlines from CoreGraphics-smoothed font
masks and handles their gamma separately. It also notes platform smoothing
settings affect dilation. This supports a font-specific investigation; it does
not supply a universal scalar adjustment for ordinary vector paths.

## Next bounded experiment

Build a diagnostic native glyph probe with the same embedded font, glyph IDs,
positions and font sizes. Compare native font masks against the pinned DOM
reference at the default smoothing setting. First validate H/Hg and both normal
fixtures; then vary foreground/background color, opacity and subpixel position.
The pass/fail signal must remain the existing geometry and pixel comparator.
Keep this probe separate from shipping renderer code until it demonstrates a
viable mapping. Do not change global renderer coverage or the browser reset.

If the probe passes, design an explicit optional glyph-run rendering seam that
preserves font identity, glyph IDs/positions, size, transforms, paint and clips
through recording/replay. It must retain the current vector path behavior as
the default for existing runtime clients, define how compiler output requests
its rendering profile, and work without a compiler dependency in the runtime.
Font-resource lifetime, cache keys (including scale and subpixel position),
backend capability reporting, file/stream compatibility and replay tests are
required parts of that change. This is proposed follow-up work, not an approved
new serialization contract or implemented capability.

This remains internal implementation work, not an evidenced external blocker.

The bounded native glyph experiment is now implemented and passes 12 smoothed
controls at three widths using exported runtime glyph IDs/positions. See
native-glyph-review.md. Transparent mask composition and production integration
remain unproven; the original acceptance failures have not been waived.

Transparent mask composition now passes 48 controls across three widths and
four backgrounds; glyph-mask-review.md records the executable gate. The next
step is the optional production glyph-rendering adapter, with bounded resources
and compatibility-preserving fallback, rather than further full-frame probes.
