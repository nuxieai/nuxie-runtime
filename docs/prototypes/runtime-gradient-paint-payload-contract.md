# Runtime Gradient Paint Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-shape-paint-blend-mode-payload-contract.md`. It extends headless
shape-paint state payloads from solid colors to local-space gradient shader
inputs.

## Formal Goal

Expose C++-matching gradient paint state for shape-owned `Fill` and `Stroke`
commands whose mutator is `LinearGradient` or `RadialGradient`.

The slice is complete when:

- `rive-graph::GradientStopNode` records imported stop color and position.
- `rive-graph::ShapePaintStateNode` records authored linear/radial gradient
  start/end coordinates, opacity, and gradient stops.
- `rive-runtime::RuntimeShapePaintState` exposes linear/radial gradient payloads
  with authored coordinates, authored opacity, inherited render opacity, sorted
  stops, clamped stop positions, and opacity-modulated render colors.
- C++ probe-backed coverage proves Rust matches C++ for a local-space gradient
  with unsorted and out-of-range stops.

## Scope Lock

This slice covers only:

- static imported `LinearGradient` and `RadialGradient` scalar state;
- local-space shader input payloads before renderer allocation;
- stop sorting by authored position;
- stop position clamping to `[0, 1]`;
- stop color modulation by `LinearGradient.opacity * ShapePaintMutator.renderOpacity`.

It does not implement renderer shader allocation, world-space gradient
transforms, deformer-adjusted gradient points, animated gradient dirt
propagation, gradient stop mutation, spread/tile behavior beyond existing C++
defaults, effects, feathers, text/layout/foreground paint propagation, or GPU
work.

## Admission Rule

Before extending this payload, answer:

1. Is the value an input to C++ `LinearGradient::applyTo()` before renderer
   shader allocation?
2. Can it be represented as scalar gradient coordinates and stop values?
3. Can it be compared through the C++ draw-command probe without depending on a
   renderer factory's shader object?

If not, defer it to a later world-gradient, deformer, animation dirt, effect,
feather, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_gradient_paint_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
