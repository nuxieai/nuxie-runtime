# Bounded Rust/CoreText glyph masks

The opt-in `nuxie-renderer/native-glyphs-experimental` module now rasterizes
already-shaped runs directly through CoreText. It replaces the subprocess and
full-viewport allocation in the earlier diagnostic with a native Rust function
returning a device-aligned mask around the run's ink. It does not alter default
renderers, compiler output, shaping or responsive layout.

The render API also has an optional glyph-run request carrying immutable font
bytes, face/variation identity, glyph IDs and baseline positions, font size,
solid color/alpha and blend mode. Existing renderers decline by default without
changing their recording stream. Runtime invocation and a caching renderer
adapter are still required; this receipt does not qualify A07 or Q09.

## Validation

```sh
CARGO_INCREMENTAL=0 bash tools/html-to-riv/validation/run-glyph-mask-controls.sh rust
CARGO_INCREMENTAL=0 cargo test -p nuxie-render-api --lib
python3 tools/pure-runtime-boundary/check.py
CARGO_INCREMENTAL=0 cargo clippy -p nuxie-html-to-riv --features native-glyph-controls --example native_glyph_composite --test native_glyphs --no-deps -- -D warnings
```

The complete wrapper finished successfully. Rust mask comparisons: **48/48 pass**, at 240, 390 and 768px on
white, dark, lavender and green backgrounds. They use the same runtime-exported
positions and unchanged Chromium references/tolerances as the Swift controls.
The masks are PNG-encoded by the diagnostic example, decoded by the recording
factory and composited through actual Metal replay. No browser coordinates or
pixels enter native rendering. Largest observed allocation: **30,704 bytes**
(240px cascade); maximum observed width 265px and height 38px across the corpus.
Mask metrics are retained alongside each recorded stream. The gate also writes
self-contained `review.html` and `review.png` contact sheets in each width
directory, using the exact images from that run.

Five native tests pass: integer translation invariance while preserving
fractional phase; straight color and alpha exactly once; unsupported/oversized
request rejection; no bitmap allocation for whitespace; and malformed-font
failure with correct null-object ownership. All **36 render API tests pass**,
including unchanged recording behavior when the optional interface declines.
The pure-runtime dependency boundary and targeted example/test Clippy pass.

Browser/native/difference contact sheets for Hg, the cascade, and colored and
translucent fractional text are inspected at all three widths. The matching
wraps, baselines, colors and small residual edge differences agree with the
numerical gate. Artifacts: `output/playwright/html-to-riv/rust-glyph-mask-*` and
`rust-glyph-review-{240,390,768}.png`.

## Limits and next work

The native implementation accepts face zero, no explicit variation coordinates,
solid SrcOver runs and finite nonsingular affine transforms. Limits are 32 MiB
font input, 16,384 glyphs, 4096px font size, 8192px mask sides and 16 MiB mask
storage. It rejects invalid glyph IDs and invalid geometry before drawing; CF
resources are released on all paths. Font bytes outlive all CF objects. Empty
ink returns an empty mask. General affine transforms are implemented but this
visual corpus qualifies translation/fractional positioning only.

There is no cache yet: the installing adapter must bound retained fonts and
images, preserve clip/transform/opacity state, and preflight a complete draw
before declining it. Runtime solid-fill eligibility must exclude unsupported
paint effects, modifiers and color-font semantics. The native code is macOS
only and opt-in; other platforms and the existing vector path remain unchanged.
The standalone compiler still emits semantic Rive text and embedded fonts.

The main compiler suite remains at its last measured **230/233**, with all
three pixel failures retained. Native/WASM compiler qualification has not been
rerun for this renderer-only increment. Next: live runtime invocation through
the optional adapter, bounded caches, compatibility tests and the unchanged
full acceptance/Q09 corpus through the integrated rendering path.
