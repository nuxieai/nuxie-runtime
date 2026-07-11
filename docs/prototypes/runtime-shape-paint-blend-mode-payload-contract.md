# Runtime Shape Paint Blend Mode Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-rounded-points-path-payload-contract.md`. It adds the first non-color
shape-paint render-state scalar to the headless draw command payload.

## Formal Goal

Expose C++-matching authored and resolved blend mode values for shape-owned
`Fill` and `Stroke` draw commands.

The slice is complete when:

- `nuxie-graph::ShapePaintContainerNode` records the owning container's authored
  drawable `blendModeValue`.
- `nuxie-graph::ShapePaintNode` records the authored shape-paint
  `blendModeValue`, including C++ default `127` for inherit.
- `nuxie-runtime::RuntimeShapePaintCommand` exposes both `blend_mode_value` and
  `render_blend_mode_value`.
- Runtime command emission mirrors C++ `Shape::buildDependencies()` /
  `ShapePaint::blendMode()`: paint value `127` resolves to the owning shape's
  blend mode, otherwise the paint value is used directly.
- C++ probe-backed coverage proves inherited and explicit blend modes match.

## Scope Lock

This slice covers only:

- static imported blend mode values;
- shape-owned `Fill` and `Stroke` paints emitted by the existing shape draw
  command stream;
- C++'s current non-animated `Shape::buildDependencies()` inheritance behavior.

It does not implement renderer blending, GPU blend configuration, animated blend
mode dirt propagation, text/layout/foreground paint propagation, image/mesh blend
payloads, scripted paint data, gradients, effects, feathers, or renderer paint
allocation.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior a scalar fact available before renderer allocation?
2. Does C++ resolve it during import/build-dependency or shape draw setup?
3. Can it be compared through the existing C++ draw-command probe without
   executing a renderer?

If not, defer it to a later renderer, animation dirt, text/layout drawability,
image/mesh drawability, scripted paint, gradient, effect, or feather slice.

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
