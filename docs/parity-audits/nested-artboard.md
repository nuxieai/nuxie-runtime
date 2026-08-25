# `NestedArtboard` paired audit

Upstream owner: `src/nested_artboard.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-runtime/src/nested_artboard.rs` owns the mounted occurrence,
  cold clone, animation collection, stateful context, and teardown order.
- `crates/nuxie-runtime/src/artboard.rs` owns construction, replacement,
  retained-component update/collapse, lookup, and child update recursion.
- `crates/nuxie-runtime/src/artboard/nested_artboard.rs` owns context rebinding,
  nested timing, animation-before-child advance, and event collection.
- `crates/nuxie-runtime/src/draw.rs` owns nested preparation, draw recursion,
  mounted transforms, clipping, and geometry hit traversal.
- `crates/nuxie-runtime/src/state_machine/state_machine_instance/state_machine_instance.rs`
  owns `worldToLocal`-equivalent pointer routing, host-ancestor hit propagation,
  and immediate nested-event delivery to registered parent machines.

Verdict: adapted and behaviorally equivalent under Rust ownership.

The paired audit covered constructor/destructor and animation teardown order;
clone/nest/replacement and authored origin behavior; stateful and ordinary data
context binding; focus registration; draw, `willDraw`, host transforms, geometry
hit traversal, and pointer routing; import/registration and nested lookup;
collapsed/paused behavior; local-time quantization and speed; animation owners
before child advancing components; reset; and recursive mounted-child teardown.
Sibling `NestedArtboardLeaf` and `NestedArtboardLayout` rendering remains in the
shared draw dispatcher without changing this base owner's behavior.

The former F13 hit-propagation ceiling was not a runtime discrepancy. The
converted `state_machine_event_test.cpp` case had replaced the first pinned
`Artboard::advance(0)` with `update_components()`, which never advanced the
mounted child, and replaced the second Artboard-only advance with a helper that
also re-advanced the root machine. Restoring the exact Rust semantic seams—
`ArtboardInstance::advance` first, then
`advance_frame_components_with_state_machine` for the owner-safe parent-event
callback—made the upstream fixture green with no runtime behavior change.

Four other ignored `upstream_nested_*` cases require public contour or child
inspection APIs to express their final assertions. They are inspection-surface
denominators, not evidence of a `NestedArtboard` behavior fallback.
