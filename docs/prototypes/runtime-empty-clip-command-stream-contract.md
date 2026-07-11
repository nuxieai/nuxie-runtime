# Runtime Empty-Clip Command Stream Contract

Date: 2026-06-29

This document narrows the next roadmap `#10` slice after
`runtime-draw-command-stream-contract.md`. It covers the first
renderer-independent empty-clip case in C++ `Artboard::drawInternal()`.

## Formal Goal

Match C++ draw-command filtering when a visible clipping shape has no modeled
source path and therefore its proxy drawables return non-zero
`emptyClipCount()`.

The slice is complete when:

- `nuxie-runtime::ArtboardInstance::draw_commands(&ArtboardGraph)` applies the
  same `emptyClips` counter shape as C++ for clip start/end proxy drawables.
- A visible `ClippingShape` whose source shape has no imported path
  registrations suppresses logical draw commands between its start and end
  proxies.
- Focused C++ probe coverage proves the command stream for a pathless clipping
  source matches C++.

## Scope Lock

This slice admits only static imported path-presence facts already projected by
`nuxie-graph`:

- `ClippingShapeNode::shape_locals`
- `PathComposerNode::path_locals`
- `ClippingShapeNode::is_visible`

It does not implement `Shape::isEmpty()` in full, path hidden/collapsed runtime
state, `PathComposer::worldPath()`, clipping render-path construction, shape
paint picking, path geometry, path effects, deformers, clipping stack renderer
operations, or GPU work.

In this slice, a clipping shape is considered to have a modeled clip path when
any collected source shape has at least one imported registered path. The
follow-up path hidden/collapse slice is captured in
`runtime-path-empty-clip-command-stream-contract.md`; geometry-backed
`PathComposer::worldPath()` behavior remains later work.

## Admission Rule

Before extending empty-clip behavior further, answer:

1. Is the C++ behavior observable in `Artboard::drawInternal()` before renderer
   callbacks?
2. Is the required path/shape state already represented in `nuxie-graph` or
   `nuxie-runtime`?
3. Can the case be compared through `drawCommandStream` JSON without real
   rendering?

If not, defer it to a later path geometry, clipping render-path, or renderer
slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe runtime_draw_command_stream_suppresses_empty_clips_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
