# Runtime Rounded PointsPath Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-cubic-points-path-payload-contract.md`. It extends headless raw path
payloads from zero-radius straight/cubic point paths to authored rounded
straight corners.

## Formal Goal

Expose C++-matching `line` plus `cubic` raw path commands for authored
`PointsPath` geometry whose `StraightVertex` children have nonzero `radius`.

The slice is complete when:

- `rive-runtime::RuntimeShapePaintCommand::path_commands` accepts supported
  nonzero-radius `StraightVertex` children instead of dropping the whole path.
- Rust ports the import-independent C++ `Path::buildPath()` rounded-corner math:
  adjacent edge normalization, clamped render radius, ideal control-point
  distance, positive-radius rounding, and negative-radius rotated rounding.
- C++ probe-backed runtime coverage proves positive and negative rounded
  corners produce the same picked raw path command stream.
- Existing straight/cubic path payload coverage continues to pass.

## Scope Lock

This slice covers only:

- authored `PointsPath` geometry;
- `StraightVertex.radius != 0`;
- positive and negative rounded corners using direct neighboring authored
  straight/cubic render points;
- the existing local/world path transforms and hidden/collapsed path filtering.

It does not implement `CubicWeight` deformation, `RenderPathDeformer`
application, NSlicer deformation, parametric path generation, path reversal for
hole/clockwise fill handling, trim/dash/target/group effects, feathers, renderer
`RenderPath` allocation, tessellation, clipping geometry payloads, text/image/
nested/list/scripted drawability, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior a direct scalar branch of `Path::buildPath()` before
   deformer/skinning/effect application?
2. Can it be represented as stable `move`/`line`/`cubic`/`close` commands?
3. Can the behavior be compared against C++ through the existing raw-path probe
   without renderer allocation?

If not, defer it to a later weighted-cubic, deformer, parametric path, effect,
clipping, text/image, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_rounded_point_path_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
