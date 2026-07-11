# Runtime Inner Feather Path Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-weighted-points-path-command-contract.md`. It exposes the
renderer-independent path geometry that C++ builds for inner feathers without
implementing renderer clipping, paint allocation, path effects, or GPU work.

## Formal Goal

Expose C++-matching inner-feather derived path commands for shape-owned `Fill`
and `Stroke` draw commands whose attached `Feather` has `inner == true`.

The slice is complete when:

- `tools/cpp-probe` emits `Feather::innerPath()` raw path commands beside the
  existing feather scalar payload.
- `nuxie-runtime::RuntimeFeatherState` exposes the matching
  `inner_path_commands` payload.
- Rust computes inner-feather geometry from the already selected shape-paint
  source path by matching `Feather::rebuildInnerPath()`:
  - bounds of the source raw path padded by `strength * 1.5`;
  - clockwise rectangle from those padded bounds;
  - source raw path appended backwards after applying the feather offset;
  - world-space offsets transformed through the inverse shape world transform.
- A C++ probe-backed test covers non-default strength, offset, and inner state.

## Scope Lock

This slice covers only the derived path payload for inner feathers in the
no-path-effect case already represented by the headless draw-command stream.

It does not implement:

- renderer save/restore, transform, translate, clip, or draw calls;
- feather strength application to `RenderPaint`;
- outer-feather renderer translations;
- path-effect interaction, including rebuilding the inner path from a
  dash/trim/target effect path;
- `DashPath`, `TrimPath`, or `TargetEffect` geometry mutation;
- animated feather dirt propagation or frame scheduling;
- renderer allocation, batching, tessellation, or GPU work.

## Admission Rule

Before extending this payload, ask:

1. Is the value part of the raw `Feather::innerPath()` geometry that C++ builds
   from the selected source `ShapePaintPath`?
2. Can it be compared through the C++ probe without allocating a renderer path
   or paint?
3. Does it avoid requiring path-effect execution or contour measuring?

If not, defer it to a later feather-renderer, path-effect, path-measure, or
renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe runtime_draw_command_stream_exposes_feather_paint_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
