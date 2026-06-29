# Runtime Drawable WillDraw Prerequisite Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-inner-feather-path-payload-contract.md`. It tightens the headless draw
command stream by modeling C++ `willDraw()` prerequisites that are static after
import and do not require a renderer.

## Formal Goal

Match C++ draw-command filtering for `Image` and `NestedArtboard` drawables
whose visibility depends on resolved import-time references.

The slice is complete when:

- `rive-graph::DrawableOrderNode` and `SortedDrawableNode` expose the resolved
  image asset global id for exact `Image` drawables.
- The same nodes expose the resolved referenced artboard global id for
  `NestedArtboard` drawables and its exact drawable subclasses.
- `rive-runtime::draw_commands()` mirrors C++ `Image::willDraw()` by requiring
  non-hidden state, non-zero render opacity, and a resolved `ImageAsset`.
- `rive-runtime::draw_commands()` mirrors C++ `NestedArtboard::willDraw()` by
  requiring non-hidden state and a resolved referenced artboard.
- A C++ probe-backed test covers valid and invalid image asset references plus
  valid and invalid nested-artboard references.

## Scope Lock

This slice covers only draw-stream inclusion/exclusion decisions that can be
made from imported references plus the existing mutable render-opacity state.

It does not implement:

- image rendering, mesh/image scale updates, layout measurement, or asset
  loading;
- nested-artboard cloning, remapping, advancing, or rendering;
- `ArtboardComponentList::willDraw()`, because list item count depends on live
  list instancing and data-context work;
- `ScriptedDrawable::willDraw()`, because it depends on script VM hydration and
  scripted draw flags;
- text shaping/layout or text draw payloads;
- renderer allocation, batching, tessellation, or GPU work.

## Admission Rule

Before extending this slice, ask:

1. Does C++ `willDraw()` make the decision from static imported references plus
   existing mutable render opacity?
2. Can the fact be projected through `rive-graph` without owning live runtime
   state?
3. Can it be compared through `tools/cpp-probe` draw-command stream output?

If not, defer it to a component-list, scripting, text, nested-artboard runtime,
asset-loading, or renderer slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-runtime --test cpp_probe runtime_draw_command_stream_filters_image_and_nested_artboard_will_draw_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
