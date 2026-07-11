# Runtime SolidColor Paint Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-shape-paint-command-payload-contract.md`. It adds the first concrete
paint-state payload while staying renderer-independent: `SolidColor` mutators on
shape paint commands expose authored and opacity-modulated ARGB colors.

## Formal Goal

Expose C++-matching SolidColor paint state on headless shape-paint draw command
payloads.

The slice is complete when:

- `nuxie-graph::ShapePaintNode` records authored `SolidColor.colorValue` for
  Fill/Stroke paints whose mutator is a `SolidColor`.
- `tools/cpp-probe` emits a `paintState` object for SolidColor-backed
  `shapePaintCommands`, including authored `color` and C++ `renderColor`.
- `nuxie-runtime::RuntimeShapePaintCommand` carries matching SolidColor state and
  computes `render_color` with C++ `colorModulateOpacity(colorValue,
  renderOpacity)`.
- Focused C++ probe coverage proves a non-opaque shape produces the same
  opacity-modulated SolidColor values in Rust and C++.

## Scope Lock

This slice covers only SolidColor paint state:

- authored ARGB `colorValue`;
- alpha modulation by the owning shape's runtime render opacity;
- C++ `std::lround`-style alpha rounding.

It does not implement gradients, gradient-stop sorting, blend modes, stroke
caps/joins, render-paint allocation, path raw geometry, tessellation, effects,
feathers, renderer callbacks, GPU work, or non-shape drawable paint state.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of a paint mutator's renderer-independent state?
2. Can it be represented as stable scalar data in JSON?
3. Does it avoid allocating or mutating renderer-owned `RenderPaint` objects?

If not, defer it to a later gradient, paint-state, effect, path-geometry, or
renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe runtime_draw_command_stream_exposes_shape_paint_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
