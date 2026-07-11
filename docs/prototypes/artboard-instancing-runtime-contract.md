# Artboard Instancing Runtime Contract

Date: 2026-06-28

This document starts roadmap item `#9` after the first dirt/transform runtime
slice. It defines the smallest artboard instancing seam that lets later
animation and state-machine work mutate a real headless artboard without pulling
in full C++ clone behavior too early.

## Formal Goal

Define and implement the first `nuxie-runtime` artboard instancing seam: separate
imported source artboard data from mutable instance state, preserve C++
artboard-local slot identity, expose typed mutable property state for the
transform/component properties needed by animation, keep `nuxie-binary` and
`nuxie-graph` frozen except for true parity bugs, and back the seam with focused
Rust tests plus C++ probe comparisons for initial runtime state.

The goal is complete when the runtime slice can:

- Build an `ArtboardInstance` from a verified `RuntimeFile` and `ArtboardGraph`.
- Preserve every graph artboard-local slot in instance-local order.
- Keep source global IDs and type names distinct from mutable per-instance
  runtime state.
- Map local component IDs to component runtime state without losing slot
  identity.
- Mutate the transform properties needed by linear animation:
  `x`, `y`, `rotation`, `scaleX`, `scaleY`, and `opacity`.
- Mark the mutated component dirty with the same dirt bits later animation
  application needs.
- Recompute local/world transforms and render opacity from mutable instance
  state rather than rereading imported source objects.
- Continue passing the existing C++ runtime-update probe comparison.

## Scope Lock

`nuxie-runtime` owns mutable headless runtime instances. It does not own file
loading or static graph discovery.

The external seam for this slice is:

- Input: `nuxie_binary::RuntimeFile` and `nuxie_graph::ArtboardGraph`.
- Output: a mutable `ArtboardInstance` with stable local slots, component state,
  transform property state, and update reports.
- Errors: construction errors when graph slot/global IDs no longer resolve into
  the imported file model.

`nuxie-binary` remains frozen for this work. `nuxie-graph` remains a static
projection. New file-loading facts or static graph facts must first pass their
own contracts.

## Owned By This Slice

This slice owns:

- `InstanceSlot` metadata for artboard-local source slots.
- Local-slot to component-index lookup.
- Mutable transform property storage for transform components.
- Transform-property setter APIs that add `Transform`, `WorldTransform`, and
  `RenderOpacity` dirt as appropriate.
- Update behavior that reads mutable instance properties.
- Focused tests proving source slots are preserved and source imported objects
  are not mutated by instance property changes.

## Not Owned Yet

These remain future slices:

- Full C++ object cloning and generated concrete object structs.
- Linear animation time advancement and keyframe application.
- State-machine execution, inputs, listeners, transitions, and events.
- Data binding retargeting, data contexts, dirty queues, observers, and
  converter scheduling.
- Nested artboard instance ownership and component-list virtualization.
- Constraint, IK, skeletal, layout, scroll, and text solving.
- Draw-order mutation, active draw-target linked lists, clipping-stack mutation,
  draw commands, renderer paint allocation, and GPU work.
- Scripting and audio.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does this make source artboard data and mutable artboard instance state more
   clearly separated?
2. Does it preserve a C++ artboard-local identity fact that later runtime code
   needs?
3. Is it required for animation/state-machine/data-bind code to mutate a
   headless artboard instance without changing imported source objects?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- `InstanceSlot` and `ArtboardInstance::slots`.
- `ArtboardInstance::slot(local_id)` lookup.
- `ArtboardInstance::set_transform_property`.
- `TransformProperty` enum for the six transform fields animation needs first.
- Tests for local slot preservation, source-vs-instance mutation, dirt marking,
  and update-from-mutated-state.

Suggested verification:

```sh
cargo test -p nuxie-runtime
make test
make cpp-compare
```
