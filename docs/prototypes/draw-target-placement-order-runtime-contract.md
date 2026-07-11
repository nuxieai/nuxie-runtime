# Draw Target Placement Order Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#10` after static `drawable_order` and
`draw_target_order` graph projection. It defines the first `sortDrawOrder()`
slice: active draw-target grouping and before/after placement splicing, without
clipping proxy interleaving or renderer commands.

## Formal Goal

Expose a headless sorted drawable order that matches the C++ linked list after
`Artboard::sortDrawOrder()` has grouped drawables by active `DrawTarget` and
spliced each target group before or after its target drawable.

The slice is complete when:

- `tools/cpp-probe` exposes the post-`sortDrawOrder()` drawable linked list by
  walking `Artboard::firstDrawable()` through `Drawable::prevDrawable()`.
- `nuxie-graph` exposes `ArtboardGraph::sorted_drawable_order`.
- Rust uses existing `drawable_order`, `draw_rules`, `draw_targets`, and
  `draw_target_order` facts to mirror C++ active target grouping.
- `DrawTarget.placementValue == 0` splices grouped drawables before the target
  drawable, and `placementValue == 1` splices them after it.
- Focused C++ probe coverage compares the sorted local drawable order for both
  before and after placement.

## Scope Lock

This slice does not implement clipping stack proxy insertion,
`clearRedundantOperations()`, `needsSaveOperation`, hidden/will-draw filtering,
layout proxy identity beyond preserving existing `drawable_order` entries,
draw commands, renderer paint allocation, path tessellation, GPU work, or live
draw-order mutation after import.

Clipping proxies are the next draw-order slice because they require modeling
`ClippingShape::createProxyDrawable()`, clip-start/clip-end operations, and save
operation elision.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it directly affect the post-`sortDrawOrder()` order of authored
   drawables captured by `firstDrawable()/prevDrawable()`?
2. Can it be derived from the existing static draw-target and draw-rule graph?
3. Can it be compared without renderer calls or clipping proxy commands?

If not, defer it to the clipping-command or renderer-command slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-graph --test cpp_probe cpp_probe_matches_rust_sorted_drawable_order_when_available -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
