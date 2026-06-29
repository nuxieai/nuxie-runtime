# Runtime Triangle Parametric Path Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-star-parametric-path-payload-contract.md`. It adds triangle parametric
path geometry to headless shape-paint commands.

## Formal Goal

Expose C++-matching raw path commands for shape-owned `Triangle` paths.

The slice is complete when:

- `rive-graph::PathGeometryNode` records imported triangle parametric scalar
  fields: width, height, and origin.
- `rive-runtime::RuntimeShapePaintCommand::path_commands` emits matching
  `move`/`line`/`close` commands by feeding triangle vertices through the
  already-ported straight vertex builder.
- C++ probe-backed coverage proves Rust matches C++ for a transformed triangle.

## Scope Lock

This slice covers only:

- static imported `Triangle` scalar fields inherited from `ParametricPath`;
- C++ `Triangle::update()` vertex placement when `ComponentDirt::Path` is set;
- the existing import-independent `Path::buildPath()` straight-vertex behavior
  for triangle vertices.

It does not implement weighted/deformed geometry, parametric dirt propagation,
layout-driven `controlSize`, path effects, fill-rule reversal for holes/negative
transforms beyond existing unsupported behavior, renderer path allocation,
tessellation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of C++ triangle scalar-to-vertex placement or already
   ported `Path::buildPath()` straight-vertex math?
2. Can it be represented as headless raw path commands?
3. Can it be compared through `ShapePaintPath::rawPath()` in the C++ probe?

If not, defer it to a later deformer, path-effect, layout, fill-rule reversal,
or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_triangle_parametric_path_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
