# Sorted Drawable Hidden Facts Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#10` after
`save-operation-elision-draw-order-runtime-contract.md`. It defines a narrow
drawability prerequisite: exposing static hidden facts on the headless sorted
drawable order.

## Formal Goal

Expose whether each sorted drawable is hidden according to the imported
`DrawableFlag::Hidden` bit and C++ `Drawable::isHidden()` immediately after
import.

The slice is complete when:

- `tools/cpp-probe` exposes `Drawable::isHidden()` for each
  `sortedDrawableOrder` node.
- `nuxie-graph` exposes `SortedDrawableNode::is_hidden`.
- Imported drawable nodes derive the hidden fact from `Drawable.drawableFlags`.
- Layout proxy nodes inherit the hidden fact from their owning layout drawable.
- Clipping proxy nodes are always visible, matching
  `ClippingShapeProxyDrawable::isHidden()`.
- Focused C++ probe coverage compares sorted local IDs plus hidden flags.

## Scope Lock

This slice does not implement full `Drawable::willDraw()` parity, render-opacity
updates, collapsed dirt, image asset availability, nested artboard reference
checks, component-list item counts, scripted drawable VM/draw checks, empty-clip
counting, pending clip-operation skipping, renderer command emission, paint
allocation, path tessellation, GPU work, or live draw-order mutation.

Full drawability belongs with the renderer-independent command stream because
C++ `willDraw()` combines static hidden flags with runtime-updated render
opacity and type-specific runtime state.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it affect `Drawable::isHidden()` immediately after import?
2. Can it be derived from stored drawable flags or existing proxy ownership?
3. Can it be compared without advancing the artboard or replaying renderer
   calls?

If not, defer it to the draw-command stream slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-graph --test cpp_probe cpp_probe_matches_rust_sorted_drawable_hidden_flags_when_available -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
