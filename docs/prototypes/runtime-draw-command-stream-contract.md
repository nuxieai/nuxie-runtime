# Runtime Draw Command Stream Contract

Date: 2026-06-29

This document continues roadmap item `#10` after
`sorted-drawable-hidden-facts-runtime-contract.md`. It defines the first
renderer-independent command-stream slice: logical drawable/proxy commands after
runtime `willDraw()` filtering, without paint or geometry payloads.

## Formal Goal

Expose a headless runtime draw command stream that follows C++
`Artboard::drawInternal()` ordering and filtering for simple drawable cases.

The slice is complete when:

- `tools/cpp-probe` emits `drawCommandStream` by walking
  `firstDrawable()->prevDrawable()` and applying the same pending clip-operation
  and `willDraw()` gate as `Artboard::drawInternal()`.
- `rive-runtime` exposes `ArtboardInstance::draw_commands(&ArtboardGraph)`.
- Commands are logical records only: `Draw`, `ClipStart`, and `ClipEnd`, with
  local IDs, clipping-shape IDs where known, and save-operation flags.
- Runtime filtering uses imported hidden facts plus runtime render opacity for
  the simple shape path already covered by transform/render-opacity updates.
- Focused C++ probe coverage proves hidden and opacity-zero simple shapes are
  filtered from the stream.

## Scope Lock

This slice does not implement render paints, paths, geometry, gradients,
effects, image assets, text shaping, nested artboard drawing, component-list
items, scripted drawable checks, empty-clip counting, clip render-path
availability, renderer callbacks, renderer state mutation, path tessellation,
GPU work, or live draw-order mutation.

The first follow-up empty-clip slice is captured in
`runtime-empty-clip-command-stream-contract.md`. Further draw-command slices
should add one omitted `willDraw()`/empty-clip family at a time, each with a C++
probe fixture.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it affect whether a logical drawable/proxy record appears in
   `Artboard::drawInternal()` for the current runtime instance?
2. Can it be compared as JSON without real renderer calls?
3. Is the needed runtime state already modeled in `rive-runtime`?

If not, defer it to a later renderer, geometry, text, nested-artboard, list, or
script runtime slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_filters_hidden_and_opacity_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
