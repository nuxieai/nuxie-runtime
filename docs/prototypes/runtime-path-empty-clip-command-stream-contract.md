# Runtime Path Empty-Clip Command Stream Contract

Date: 2026-06-29

This document extends roadmap `#10` after
`runtime-empty-clip-command-stream-contract.md`. It covers the next
renderer-independent part of C++ empty-clip behavior: source shapes whose
registered paths are all hidden or collapsed.

## Formal Goal

Match C++ draw-command filtering for clipping sources that have imported paths
but no currently drawable source path according to the non-geometry parts of
`Shape::isEmpty()`.

The slice is complete when:

- `rive-graph::PathComposerNode` records per-path imported hidden facts from
  `Path.pathFlags`.
- `rive-runtime::ArtboardInstance::draw_commands(&ArtboardGraph)` treats a
  clipping source as empty when every source path is hidden or live-collapsed.
- `tools/cpp-probe` can apply a runtime component collapse mutation before
  `updateComponents()` and draw-command emission.
- Focused C++ probe coverage proves both hidden source paths and collapsed
  source paths suppress enclosed draw commands like C++.

## Scope Lock

This slice admits only facts needed by C++ `Shape::isEmpty()` and
`PathComposer::update()` before geometry:

- imported `Path.pathFlags & 1` hidden state;
- live `Component::isCollapsed()` state for path components;
- existing clipping-shape source shape membership from `rive-graph`.

It does not implement `PathComposer::worldPath()`, path raw geometry,
parametric path building, points-path vertices, path effects, deformers,
fill-rule behavior, clipping render-path construction, renderer clipping stack
operations, or GPU work.

## Admission Rule

Before extending this slice, answer:

1. Is the behavior part of C++ `Shape::isEmpty()` or the empty-clip branch in
   `Artboard::drawInternal()`?
2. Can the needed state be represented as imported path flags or live component
   collapse state?
3. Can the case be compared through `drawCommandStream` JSON without renderer
   callbacks?

If not, defer it to a later path geometry, path composer, clipping render-path,
or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_treats_hidden_clip_paths_as_empty_like_cpp_probe -- --nocapture
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_treats_collapsed_clip_paths_as_empty_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
