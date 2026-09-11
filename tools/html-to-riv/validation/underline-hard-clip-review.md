# A20 hard-clip runtime integration

Text caches a full stripe, geometric fallback, and exclusions per visual line.
During drawing it saves the current state, applies hard difference clips, draws
the full stripe, then restores. If any exclusion is declined, it restores all
partial clip changes before drawing the fallback. Each line has independent
clip state, so tightly spaced adjacent underlines do not clip one another.
Recording and its glyph adapter now preserve these clips through native replay.
No CSS syntax, layout calculations or thresholds changed.

## Results

- Public compiler/module Rust suite: 88 tests pass, including import, resize,
  recorded hard-clip presence, glyph draw ordering and decoration removal.
- Native glyph corpus: 90/90 pass at 240/390/768, compiling once per fixture and
  resizing the same scene. All 90 browser/native/diff pairs visually inspected
  in contact-0.png through contact-14.png. Original CJK Auto/All 240px failures
  are fixed; all 9 CJK comparisons pass the strict red support check.
- Vector corpus: 85/90 pass. Remaining failures are from-font Inter/OpenSans
  at 390px and CJK None at all three widths, as before. This turn completed
  the numeric run; a fresh full vector visual review remains pending.
- Host-state profiles, both isolated and restored-copy: 17/27 pass each.
  Before this change they passed 6/27 and 0/27 respectively. All geometry,
  ordinary pixel and ink-coverage gates pass; the ten residual failures each
  have one or two unmatched red pixels. Remaining cases: opacity, rotation,
  shear at all widths, plus nonuniform scale at 240. New host-state contact
  sheets are retained; fresh visual review of all 54 pairs remains pending.

The native-glyph gallery confirms intact whitespace extents, empty lines,
transparent ink controls, ancestor origins and independent colored underlines.
No visual threshold was widened. The same remaining vector failures and new
host-state residuals are retained; A20 is not fully qualified.

## Artifacts and commands

`output/playwright/html-to-riv/underline-hard-clip-glyph/` contains review.json,
gallery.html and the 15 reviewed contact sheets. Native/vector sources, bytes,
requirements, draw streams and screenshots are in the corresponding
`tools/html-to-riv/test-results-underline-hard-clip-{glyph,vector}` directories.
The vector review is `output/playwright/html-to-riv/underline-hard-clip-vector/`.
Host-state runs are `underline-hard-clip-state-isolated` and
`underline-hard-clip-state` under the same output parent. These are separate from
pre-fix artifacts. `NUXIE_HTML_REVIEW_DIR` now also selects the state-control
output directory.

Focused browser command: `NUXIE_NATIVE_GLYPHS=1` (or 0 for vector) with
`npm --prefix tools/html-to-riv test -- --grep 'underline-' --output=<unique-dir>`
and a unique `NUXIE_HTML_REVIEW_DIR`. Host controls use
`node tools/html-to-riv/validation/glyph-state-control.mjs underline`, optionally
`--no-restore`, and their own unique review directories.

Logs: `/tmp/html-hard-clip-runtime-tests.log`, `/tmp/html-hard-clip-runtime-build.log`,
`/tmp/html-hard-clip-module.log`, `/tmp/html-hard-clip-cjk.log`,
`/tmp/html-hard-clip-glyph.log`, `/tmp/html-hard-clip-vector.log`,
`/tmp/html-hard-clip-state.log`, `/tmp/html-hard-clip-state-restore.log`.
Native/WASM compiler parity and the complete unrelated corpus were not rerun
in this integration step. Additional fallback tests, other backend/wrapper
coverage, general-affine/DPR qualification and clipping performance remain open.

Module Clippy (`--all-targets --no-deps -- -D warnings`) and the runtime dependency
boundary check also pass: `/tmp/html-hard-clip-runtime-clippy.log` and
`/tmp/html-hard-clip-runtime-boundary.log`.

## Fallback and canvas-wrapper follow-up

The public compiler/import test now draws through a stateful backend that declines
hard clips immediately and after accepting one exclusion. It asserts that all
partial hard clips have been restored before subsequent fallback drawing and that
the fallback retains descender gaps. The focused test passes; this exercises the
actual runtime draw path, not a separate model of it.

The Metal implementation is shared through `hard_clip::apply`, with the offscreen
canvas's ExactSourceRendererAdapter and NativeMetalRenderCanvasFrame forwarding
hard clips. Previously the canvas silently selected the explicit fallback because
its wrapper inherited the default unsupported result. Native-metal replay builds
and all five exact native pixel controls still pass after this extraction. A
separate offscreen-canvas pixel test remains pending; compilation and shared-path
tests do not establish its full qualification. Other product backends are not
claimed qualified by this change.

Logs: `/tmp/html-hard-clip-fallback-test.log`, `/tmp/html-hard-clip-shared-build.log`,
`/tmp/html-hard-clip-shared-pixels.log`. The fallback test support is
`tests/support/declining_renderer.rs`.
