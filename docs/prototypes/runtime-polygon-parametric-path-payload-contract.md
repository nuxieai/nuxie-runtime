# Runtime Polygon Parametric Path Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-ellipse-parametric-path-payload-contract.md`. It adds polygon
parametric path geometry to headless shape-paint commands.

## Formal Goal

Expose C++-matching raw path commands for shape-owned `Polygon` paths.

The slice is complete when:

- `rive-graph::PathGeometryNode` records imported polygon parametric scalar
  fields: width, height, origin, point count, and corner radius.
- `rive-runtime::RuntimeShapePaintCommand::path_commands` emits matching
  `move`/`line`/`cubic`/`close` commands by feeding polygon vertices through
  the already-ported straight/rounded vertex builder.
- C++ probe-backed coverage proves Rust matches C++ for a transformed rounded
  polygon.

## Scope Lock

This slice covers only:

- static imported `Polygon` scalar fields;
- C++ `Polygon::buildPolygon()` vertex placement;
- the existing import-independent `Path::buildPath()` straight/rounded
  `StraightVertex` behavior for polygon vertices.

It does not implement star, triangle, weighted/deformed geometry, parametric
dirt propagation, layout-driven `controlSize`, path effects, fill-rule reversal
for holes/negative transforms beyond existing unsupported behavior, renderer
path allocation, tessellation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of C++ polygon scalar-to-vertex placement or already
   ported `Path::buildPath()` straight/rounded vertex math?
2. Can it be represented as headless raw path commands?
3. Can it be compared through `ShapePaintPath::rawPath()` in the C++ probe?

If not, defer it to a later star/triangle, deformer, path-effect, layout,
fill-rule reversal, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_polygon_parametric_path_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
