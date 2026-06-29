# Runtime Straight PointsPath Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-solid-color-paint-payload-contract.md`. It adds the first raw geometry
payload to headless shape-paint draw commands while avoiding renderer-owned
`RenderPath` allocation.

## Formal Goal

Expose C++-matching raw path commands for the simplest authored shape paths:
straight `PointsPath` objects with zero-radius `StraightVertex` children.

The slice is complete when:

- `rive-graph::PathGeometryNode` records imported `PointsPath` closure/hole
  flags and straight-vertex position/radius facts.
- `tools/cpp-probe` emits picked `ShapePaintPath::rawPath()` command payloads
  for runtime shape-paint commands.
- `rive-runtime::RuntimeShapePaintCommand` carries matching `path_commands`
  for supported straight point paths after the same local/world path selection
  used by C++ `ShapePaint::pickPath()`.
- Focused C++ probe coverage proves a closed straight `PointsPath` produces the
  same `move`, `line`, and `close` command stream in Rust and C++.

## Scope Lock

This slice covers only:

- authored `PointsPath` geometry;
- `StraightVertex` children;
- `StraightVertex.radius == 0`;
- hidden/collapsed path filtering;
- path closure through `PointsCommonPath.isClosed`;
- local and world path transforms through current runtime component transforms.

It does not implement rounded straight corners, cubic vertices, `CubicWeight`
deformation, parametric paths, path reversal for hole/clockwise fill handling,
trim/dash/target/group effects, feathers, NSlicer deformation, renderer
`RenderPath` allocation, tessellation, clipping geometry payloads, or non-shape
drawable geometry.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of `Path::buildPath()` or `PathComposer::update()` before
   `RenderPath` allocation?
2. Can it be represented as a stable command list of verbs and scalar points?
3. Can it be compared against C++ without requiring a renderer factory?

If not, defer it to a later cubic, rounded-corner, deformer, effect, clipping,
or renderer slice.

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
