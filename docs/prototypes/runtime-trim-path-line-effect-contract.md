# Runtime Trim Path Line Effect Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-drawable-will-draw-prereq-contract.md`. It starts effect path mutation
with the smallest useful C++-comparable slice: line-only `TrimPath` effect
payloads.

## Formal Goal

Expose C++-matching `effect_path_commands` for shape-paint commands whose
registered stroke effects contain a supported `TrimPath`.

The slice is complete when:

- `rive-graph::StrokeEffectNode` carries the imported `TrimPath` scalar fields
  needed by C++ `TrimPath::trimPath`: `start`, `end`, `offset`, and
  `modeValue`.
- `tools/cpp-probe` emits `ShapePaint::lastEffectPath(shapePaint)` as
  `effectPathCommands` beside the existing picked source `pathCommands`.
- `rive-runtime` computes the same effect path for single-contour line-only
  source paths in sequential and synchronized trim modes.
- A C++ probe-backed test covers a visible stroke, an open straight
  `PointsPath`, and a nontrivial trim range.

## Scope Lock

This slice covers only:

- effect-path payload reporting, not replacement of the source path payload;
- `TrimPath` effects registered directly on a shape paint;
- source paths made of `move`, `line`, and optional `close` commands;
- one contour whose trim range does not require multi-contour reordering.

It does not implement:

- cubic, quad, rounded, or measured curve segmentation;
- `DashPath`, `TargetEffect`, `GroupEffect`, or `ScriptedPathEffect`;
- multi-contour trim reordering and wrapped trim ranges;
- effect invalidation caches or dirt scheduling;
- effect-path-aware inner-feather rebuilding;
- renderer draw-path selection, paint allocation, tessellation, or GPU work.

## Admission Rule

Before extending this slice, ask:

1. Can the effect be computed from the already emitted headless raw path command
   stream without a renderer or C++ `PathMeasure` equivalent?
2. Can the exact result be compared through `tools/cpp-probe`
   `effectPathCommands`?
3. Does it preserve `path_commands` as the picked source path while exposing
   effect output separately?

If not, defer it to a broader path-measure, effect-chain, feather-effect, or
renderer slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_exposes_line_trim_path_effect_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
