# Named/HSL colors and intrinsic row text: validation receipt

Command: `CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run.sh native`.
The final accepted-profile run passed 158 browser/native checks (150 scenes and
eight sensitivity checks), 23 Rust tests, five JavaScript/WASM tests, two gallery
checks and TypeScript. Module Clippy passed with warnings denied. Native/WASM
bytes and source maps agree across the accepted corpus.

Visually inspected all five new fixture pairs at 240, 390 and 768px:

* named-colors-complete-palette (all 148 standard named sRGB colors)
* named-colors-text-and-currentcolor
* hsl-hues-units-clamping
* hsl-text-alpha-cascade
* intrinsic-text-row-sizing

Chromium 153.0.8010.12, DPR 1; native Rust Metal. Palette colors, hue conversion,
alpha, inherited text and natural row widths agree within the unchanged
thresholds. Tests compile once and resize the imported scene. Named color
values are supplied by the existing cssparser dependency; browser screenshots
provide an independent oracle for the complete table. Known-value compiler
checks cover aliases, case handling, hue units and rotations, alpha, clamping,
malformed syntax and deferred functions/system colors.

A01 and A02 are qualified for the explicit SUPPORT.md contract. Missing color
components, relative colors and nested functions remain intentionally excluded.

The realistic card exposed a zero-width intrinsic row-text bug, now fixed in
the compiler using native AutoWidth/Hug sizing for inflexible auto-basis text.
No runtime code or wire-format changes were needed. This is distinct from the
existing unsupported positive-flex intrinsic basis case.

The original card and an added typography specimen still fail visual limits.
Their separate known-gap run reproduced five failures and one pass across six
comparisons. They remain intact in known-gaps.json and are documented in
known-gaps.md; Q09 remains partial. This receipt does not count those failures
as qualified output. No tolerance was widened and no font-metric experiment
that failed qualification was retained.
