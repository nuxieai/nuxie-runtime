# Runtime Rectangle Parametric Path Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-feather-paint-payload-contract.md`. It adds the first parametric path
geometry payload to headless shape-paint commands.

## Formal Goal

Expose C++-matching raw path commands for shape-owned `Rectangle` paths.

The slice is complete when:

- `nuxie-graph::PathGeometryNode` records imported rectangle parametric scalar
  fields: width, height, origin, linked corner-radius flag, and corner radii.
- `nuxie-runtime::RuntimeShapePaintCommand::path_commands` emits matching
  `move`/`line`/`cubic`/`close` commands for rectangle paths, including rounded
  corner radii through the existing straight-vertex corner builder.
- C++ probe-backed coverage proves Rust matches C++ for a transformed rectangle
  with linked rounded corners.

## Scope Lock

This slice covers only:

- static imported `Rectangle` scalar fields;
- C++ `Rectangle::update(ComponentDirt::Path)` vertex placement;
- the existing import-independent `Path::buildPath()` straight/rounded
  `StraightVertex` behavior for rectangle vertices;
- local/world shape-paint path selection already modeled by the draw-command
  stream.

It does not implement ellipse, polygon, star, triangle, weighted/deformed
geometry, parametric dirt propagation, layout-driven `controlSize`, path
effects, fill-rule reversal for holes/negative transforms beyond existing
unsupported behavior, renderer path allocation, tessellation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of C++ rectangle scalar-to-vertex placement or already
   ported `Path::buildPath()` straight/rounded vertex math?
2. Can it be represented as headless raw path commands?
3. Can it be compared through `ShapePaintPath::rawPath()` in the C++ probe?

If not, defer it to a later ellipse/polygon/star, deformer, path-effect,
layout, fill-rule reversal, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe runtime_draw_command_stream_exposes_rectangle_parametric_path_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
