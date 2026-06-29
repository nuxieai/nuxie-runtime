# Save Operation Elision Draw Order Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#10` after
`clipping-proxy-draw-order-runtime-contract.md`. It defines the next
`sortDrawOrder()` slice: modeling C++ `Artboard::clearRedundantOperations()` as
static save-operation facts on the headless sorted drawable order.

## Formal Goal

Expose whether each sorted drawable/proxy needs its own save/restore operation
after C++ clipping proxy interleaving and redundant-operation elision.

The slice is complete when:

- `tools/cpp-probe` exposes `Drawable::m_needsSaveOperation` for each
  `sortedDrawableOrder` node without changing the reference runtime.
- `rive-graph` exposes `SortedDrawableNode::needs_save_operation`.
- Rust applies the same stack algorithm as `Artboard::clearRedundantOperations()`
  over `firstDrawable()->prevDrawable()` order.
- Consecutive clip-start operations and drawables tightly wrapped by clip
  start/end operations match C++ save-elision behavior.
- Focused C++ probe coverage compares the final marker/save stream.

## Scope Lock

This slice does not implement renderer command emission, pending clip-operation
skipping inside `Artboard::drawInternal`, empty-clip counting, render-path
construction, hidden/will-draw filtering, paint allocation, path tessellation,
GPU work, or live draw-order mutation after import.

The next draw-order slice should decide whether to model
`Artboard::drawInternal()` as renderer-independent commands or first expose a
lighter drawability/hidden-filter projection.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it affect `Drawable::m_needsSaveOperation` immediately after
   `clearRedundantOperations()`?
2. Can it be derived from the sorted drawable/proxy order and static clipping
   visibility?
3. Can it be compared as data without replaying renderer calls?

If not, defer it to the draw-command stream slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p rive-graph --test cpp_probe cpp_probe_matches_rust_save_operation_elision_when_available -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
