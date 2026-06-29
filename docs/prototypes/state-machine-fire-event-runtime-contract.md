# State Machine Fire Event Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after random transition selection.
It defines the next smallest observable state-machine behavior:
`StateMachineFireEvent` actions attached to authored states and transitions
produce reported events during simple state-machine advance.

## Formal Goal

Implement reported event collection for simple state-machine fire actions in
`rive-runtime`: carry imported `StateMachineFireEvent` actions from
`rive-binary` into runtime states and transitions, clear previously reported
events at the start of each state-machine advance, collect matching
`occursValue` actions when states and transitions start or end, expose pending
reported events through the Rust `StateMachineInstance`, and compare event
identity/order plus existing advance reports against the C++ probe.

The goal is complete when the runtime slice can:

- Read state-attached and transition-attached `StateMachineFireEvent` actions
  from the existing binary import graph.
- Resolve each action to its target authored `Event` object when C++ resolves
  it.
- Fire state `atEnd` actions before leaving the source state.
- Fire state `atStart` actions after entering the target state.
- Fire transition `atStart` actions after a transition is selected.
- Fire transition `atEnd` actions immediately for zero-duration transitions
  and when timed transitions finish mixing.
- Clear previously reported events at the beginning of each public advance,
  matching C++ `StateMachineInstance::advance`.
- Compare event count, order, event local ID, core type, name, and delay
  against the C++ probe.

## Scope Lock

This slice owns only state-machine scheduled `StateMachineFireEvent` collection
for the simple animation-state paths already covered by prior `#11` slices. It
does not own `StateMachineFireTrigger`, listener actions, listener event
matching, event bubbling into data binding, animation keyframe callback events,
nested-artboard event forwarding, audio/open-url behavior beyond reporting the
event object identity, blend states, scripted actions, or rendering.

The external seam becomes:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- New `StateMachineInstance` reported-event query APIs.

## Owned By This Slice

This slice owns:

- Runtime storage for imported state/transition fire-event actions.
- Pending reported-event storage on `StateMachineInstance`.
- State and transition start/end event collection during simple state changes.
- C++ probe extension for reported events emitted after runtime state-machine
  advances.
- C++ probe comparison coverage for state start/end and transition start/end
  event ordering.

## Not Owned Yet

These remain future slices:

- `StateMachineFireTrigger` and view-model trigger mutation.
- Listener actions, listener hit testing, listener event matching, and
  reported-event listener dispatch.
- Animation `KeyFrameCallback` event sampling and delayed event reports.
- Nested artboards and nested animation/event remapping.
- Blend states and blend animations.
- Data binding, bindable-property conditions, view-model conditions, scripted
  conditions, and runtime property overrides.
- Non-double keyframes and custom keyframe interpolators.
- Draw-order mutation, clipping-stack mutation, draw commands, renderer paint
  allocation, and GPU work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for `StateMachineFireEvent` actions attached to simple
   states/transitions to appear in `StateMachineInstance` reported events?
2. Does it reuse the existing simple transition scheduler and imported
   fire-action graph?
3. Can it be compared against the C++ probe without admitting listener
   dispatch, data binding, nested artboards, blend states, keyframe callback
   sampling, scripted actions, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime fire-action records with occurrence and resolved event identity.
- Public reported-event query APIs on `StateMachineInstance`.
- State/transition start/end collection for simple zero-duration and timed
  transitions.
- A C++ probe-backed fixture with one bool input, two animation states, and
  authored events proving this order: source state start on first entry, then
  source state end, target state start, transition start, and transition end
  when the bool transition fires.

Suggested verification:

```sh
tools/cpp-probe/build.sh debug
cargo test -p rive-runtime --test cpp_probe state_machine_fire_events_match_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
