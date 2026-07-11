# Clipping Proxy Draw Order Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#10` after
`draw-target-placement-order-runtime-contract.md`. It defines the next
`sortDrawOrder()` slice: interleaving clipping proxy drawables into the
headless sorted drawable order, without renderer command emission.

## Formal Goal

Expose a final drawable-order snapshot that matches the C++ linked-list marker
sequence after `Artboard::sortDrawOrder()` inserts `ClippingShapeProxyDrawable`
start/end operations around clipped drawable runs.

The slice is complete when:

- `nuxie-graph` inserts `ClipStartProxy` and `ClipEndProxy` nodes into
  `ArtboardGraph::sorted_drawable_order`.
- Proxy insertion follows the C++ clipping-stack pass over
  `firstDrawable()->prevDrawable()` order.
- Per-drawable clipping membership is derived from the existing
  `ClippingShapeNode::clipped_drawable_locals` projection, preserving C++
  clipping-shape import order.
- Proxy nodes retain the originating clipping-shape local/global IDs for later
  renderer-command work.
- Focused C++ probe coverage compares the final local/proxy marker sequence.

## Scope Lock

This slice does not implement `needsSaveOperation`, `clearRedundantOperations()`
save/restore elision facts, `Drawable::willDraw()` hidden filtering,
`ClippingShapeStart::emptyClipCount()`, render-path construction, clip command
payloads, renderer paint allocation, path tessellation, GPU work, or live
draw-order mutation after import.

The next draw-order slice should expose save-operation elision or draw-command
facts only after deciding how to compare them against C++, because
`Drawable::m_needsSaveOperation` is protected and not currently visible through
the probe as a plain data field.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it affect the order or kind of proxy markers in the C++ drawable linked
   list?
2. Can it be derived from static clipping-shape membership?
3. Can it be compared without calling a renderer?

If not, defer it to the save-elision or renderer-command slice.

## Verification

Focused verification:

```sh
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-graph --test cpp_probe cpp_probe_matches_rust_clipping_proxy_drawable_order_when_available -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
