# Runtime Ellipse Parametric Path Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-rectangle-parametric-path-payload-contract.md`. It adds ellipse
parametric path geometry to headless shape-paint commands.

## Formal Goal

Expose C++-matching raw path commands for shape-owned `Ellipse` paths.

The slice is complete when:

- `rive-graph::PathGeometryNode` records imported ellipse parametric scalar
  fields: width, height, and origin.
- `rive-runtime::RuntimeShapePaintCommand::path_commands` emits matching
  `move`/`cubic`/`close` commands using C++'s `circleConstant`.
- C++ probe-backed coverage proves Rust matches C++ for a transformed ellipse.

## Scope Lock

This slice covers only:

- static imported `Ellipse` scalar fields inherited from `ParametricPath`;
- C++ `Ellipse::update(ComponentDirt::Path)` vertex and control-point
  placement;
- local/world shape-paint path selection already modeled by the draw-command
  stream.

It does not implement polygon, star, triangle, weighted/deformed geometry,
parametric dirt propagation, layout-driven `controlSize`, path effects,
fill-rule reversal for holes/negative transforms beyond existing unsupported
behavior, renderer path allocation, tessellation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the behavior part of C++ ellipse scalar-to-cubic placement?
2. Can it be represented as headless raw path commands?
3. Can it be compared through `ShapePaintPath::rawPath()` in the C++ probe?

If not, defer it to a later polygon/star/triangle, deformer, path-effect,
layout, fill-rule reversal, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_ellipse_parametric_path_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
