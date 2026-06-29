# Runtime Shape-Paint Command Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after the empty-clip command-stream
slices. It starts paint/geometry payload work without introducing a renderer:
shape draw commands now expose the ordered shape-paint operations that C++
`Shape::draw()` would attempt.

## Formal Goal

Expose renderer-independent shape-paint payloads for `Shape` draw commands.

The slice is complete when:

- `rive-graph::ShapePaintNode` records the static C++ paint gate and path-family
  choice for imported `Fill` and `Stroke` paints.
- `tools/cpp-probe` emits `shapePaintCommands` for each shape draw command,
  after the same `Shape::draw()` paint visibility and `pickPath()` checks.
- `rive-runtime::RuntimeDrawCommand` carries matching `shape_paints` payloads
  with paint local ID, mutator local ID, paint kind, path kind, and per-paint
  save-operation flag.
- Focused C++ probe coverage proves visible fills/strokes are included while
  invisible paints and zero-thickness strokes are skipped.

## Scope Lock

This slice covers only structural paint operations:

- `Fill` versus `Stroke`;
- `Fill.fillRule == clockwise` choosing `localClockwise`, otherwise `local`;
- `Stroke.transformAffectsStroke` choosing `local` or `world`;
- `ShapePaint.isVisible` and `Stroke.thickness > 0`;
- C++ `Shape::draw()` save-operation flag widening when a shape has multiple
  registered paints.

It does not implement colors, gradients, render-paint allocation, blend modes,
stroke caps/joins, path raw geometry, tessellation, effects, feathers, renderer
callbacks, GPU work, or drawing non-`Shape` drawable families.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior selected inside `Shape::draw()` or `ShapePaint::pickPath()`
   before real renderer calls?
2. Can it be represented as stable IDs/enums/scalars in JSON?
3. Does it avoid constructing renderer-owned `RenderPaint`/`RenderPath`
   objects?

If not, defer it to a later paint-state, path-geometry, effect, or renderer
slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_shape_paint_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
