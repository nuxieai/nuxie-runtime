# Runtime Cubic PointsPath Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-straight-points-path-payload-contract.md`. It extends headless raw path
payloads from straight point paths to authored cubic point paths.

## Formal Goal

Expose C++-matching `cubic` raw path commands for authored `PointsPath`
geometry whose vertices are zero-radius `StraightVertex` objects or concrete
cubic vertex objects.

The slice is complete when:

- `nuxie-graph::PathVertexNode` records the cubic scalar fields needed to compute
  concrete cubic in/out points.
- `nuxie-runtime::RuntimeShapePaintCommand::path_commands` emits matching
  `RuntimePathCommand::Cubic` commands for supported `CubicDetachedVertex`,
  `CubicAsymmetricVertex`, and `CubicMirroredVertex` path vertices.
- The C++ probe-backed runtime test covers a mixed straight/cubic closed
  `PointsPath` and compares the exact picked raw path command stream.

## Scope Lock

This slice covers only:

- authored `PointsPath` geometry;
- `StraightVertex.radius == 0`;
- `CubicDetachedVertex`, `CubicAsymmetricVertex`, and `CubicMirroredVertex`;
- concrete cubic in/out point formulas before deformation;
- hidden/collapsed path filtering and local/world path transforms already
  established by the straight-path slice.

It does not implement rounded straight corners, `CubicWeight` deformation,
parametric paths, path reversal for hole/clockwise fill handling, trim/dash/
target/group effects, feathers, NSlicer deformation, renderer `RenderPath`
allocation, tessellation, clipping geometry payloads, or non-shape drawable
geometry.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior a direct authored-vertex branch of `Path::buildPath()`?
2. Can the control points be computed from imported scalar properties without
   skinning/deformer runtime state?
3. Can it be compared against C++ as raw path verbs and scalar points?

If not, defer it to a later rounded-corner, weighted-cubic, deformer, effect,
clipping, or renderer slice.

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
