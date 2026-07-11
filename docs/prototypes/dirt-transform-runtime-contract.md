# Dirt And Transform Runtime Contract

Date: 2026-06-28

This document starts roadmap item `#8` after the static graph seam. It defines
the first mutable headless runtime slice so the Rust port can advance beyond
file import and graph projection without pulling in animation, data binding,
layout, or rendering too early.

## Formal Goal

Define and implement the first mutable headless runtime slice around
`nuxie-runtime`: consume a verified `nuxie_binary::RuntimeFile` plus
`nuxie_graph::ArtboardGraph`, create mutable artboard component state, and
reproduce C++ component dirt scheduling and basic transform update semantics.

The goal is complete when the runtime slice can:

- Represent the C++ `ComponentDirt` bit layout exactly.
- Seed mutable component dirt state with C++ `ComponentDirt::Filthy`.
- Mark components dirty with C++ `Component::addDirt` duplicate suppression.
- Recursively dirty graph dependents through `ComponentNode::dependent_locals`.
- Set artboard-level `Components` dirt through `Artboard::onComponentDirty`.
- Traverse components in C++ `graphOrder` order.
- Implement `Artboard::updateComponents` restart behavior when an update dirties
  an earlier graph-order component.
- Preserve the C++ max-pass guard behavior.
- Skip collapsed components without clearing their pending dirt.
- Update basic `TransformComponent` local transforms, world transforms, and
  render opacity from imported stored values.

## Scope Lock

`nuxie-runtime` owns mutable headless runtime state. It does not own file loading
or static graph discovery.

The external seam for this slice is:

- Input: `nuxie_binary::RuntimeFile` and `nuxie_graph::ArtboardGraph`.
- Output: a mutable `ArtboardInstance` with per-component dirt and transform
  state, plus update reports that expose scheduler behavior for tests.
- Errors: construction errors when graph IDs no longer resolve into the imported
  file model.

`nuxie-binary` remains frozen for this work. `nuxie-graph` remains a static
projection. New live runtime behavior belongs in `nuxie-runtime` or a later
runtime crate.

## Owned By This Slice

This slice owns:

- `ComponentDirt` constants and bit operations.
- Per-component dirt state and collapsed-state checks.
- `add_dirt(local_id, dirt, recurse)` semantics over imported graph components.
- Artboard `Components` dirt and `did_change` state.
- Dirt-depth restart bookkeeping.
- The update pass over C++ `graphOrder` components.
- A max-pass guard surfaced as a report, not as a panic.
- Basic `Mat2D` operations needed by C++ `TransformComponent::updateTransform`
  and `TransformComponent::updateWorldTransform`.
- Basic render opacity propagation through the direct parent
  `WorldTransformComponent`, matching the C++ parent-transform cache rule.

## Not Owned Yet

These remain future slices:

- Animation advancement and keyframe application.
- State-machine execution, inputs, listeners, transitions, and events.
- Data binding, data contexts, dirty queues, source/target mutation, observers,
  and converter scheduling.
- Constraint solving, including transform constraints and IK solving.
- Layout solving, Yoga updates, scroll constraints, and virtualization.
- Artboard cloning and nested-artboard instance ownership.
- Draw-order mutation, active draw-target linked lists, clipping-stack mutation,
  draw commands, renderer paint allocation, and GPU work.
- Text shaping/layout, variable-font mutation, scripting, audio, focus, and
  semantic tree runtime behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does C++ run this behavior inside `Component::addDirt`,
   `Component::collapse`, `Artboard::onComponentDirty`,
   `Artboard::updateComponents`, or `TransformComponent::update`?
2. Can it run over the already imported `RuntimeFile` and static
   `ArtboardGraph` without cloning, animation, data binding, layout, or
   rendering?
3. Is the result needed to make later animation/state-machine/data-bind work
   drive a real mutable artboard?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- `crates/nuxie-runtime`.
- `ComponentDirt` and `Mat2D` primitives.
- `ArtboardInstance::from_graph`.
- `ArtboardInstance::add_dirt`.
- `ArtboardInstance::collapse_component`.
- `ArtboardInstance::update_components`.
- Focused tests for dirt bit layout, recursive dependent dirtying,
  collapsed-component skip behavior, restart-on-earlier-dirt behavior, max-pass
  guard behavior, basic transform propagation, and fixture-backed graph
  construction.

Suggested verification:

```sh
cargo test -p nuxie-runtime
make test
make cpp-compare
```
