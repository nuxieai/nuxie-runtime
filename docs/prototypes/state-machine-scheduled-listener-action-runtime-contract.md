# State Machine Scheduled Listener Action Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after scheduled
`StateMachineFireEvent` collection. It defines the next smallest listener
surface: `ListenerFireEvent` actions attached directly to states and
transitions are performed at their scheduled start/end occurrence and report
events through the existing `StateMachineInstance` event buffer.

## Formal Goal

Implement scheduled `ListenerFireEvent` actions for simple state-machine
advance in `nuxie-runtime`: carry imported state/transition listener actions
from `nuxie-binary`, admit only `ListenerFireEvent` actions with resolved target
events, honor the scheduled occurrence bit in `ListenerAction.flags`, run them
after the matching state/transition fire-event actions like C++, and compare
reported event identity/order against the C++ probe.

The goal is complete when the runtime slice can:

- Read state-attached and transition-attached `ListenerFireEvent` actions from
  the existing binary import graph.
- Resolve each action to its target authored `Event` object when C++ resolves
  it.
- Match `ListenerAction::matchesScheduledOccurrence`, where bit `0` of
  `flags` selects `atStart` (`0`) or `atEnd` (`1`).
- Perform state `atEnd` listener actions after state `StateMachineFireEvent`
  actions.
- Perform state `atStart` listener actions after state `StateMachineFireEvent`
  actions.
- Perform transition `atStart` listener actions after transition
  `StateMachineFireEvent` actions.
- Perform transition `atEnd` listener actions immediately for zero-duration
  transitions and when timed transitions finish mixing.
- Compare reported event count, order, event local ID, core type, name, and
  delay against the C++ probe.

## Scope Lock

This slice owns only layer-scheduled `ListenerFireEvent` actions attached to
states and transitions. It does not own listener-owned actions, hit testing,
pointer/keyboard/gamepad/semantic listener invocation, listener action kinds
that mutate inputs or view models, scripted listener actions, data binding,
nested artboards, animation keyframe callback events, blend states, or
rendering.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing `StateMachineInstance` reported-event query APIs.

## Owned By This Slice

This slice owns:

- Runtime storage for imported scheduled `ListenerFireEvent` actions.
- Scheduled listener action execution at state/transition start/end.
- C++ probe comparison coverage for listener-fired event ordering.

## Not Owned Yet

These remain future slices:

- Listener-owned actions and listener invocation from pointer, keyboard,
  gamepad, semantic, or reported-event matching.
- `ListenerBoolChange`, `ListenerNumberChange`, `ListenerTriggerChange`,
  `ListenerViewModelChange`, and scripted listener actions.
- `StateMachineFireTrigger` and view-model trigger mutation.
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

1. Is it required for a state/transition-owned `ListenerFireEvent` to report an
   event during simple state-machine advance?
2. Does it reuse the existing reported-event buffer and simple transition
   scheduler?
3. Can it be compared against the C++ probe without admitting listener-owned
   actions, hit testing, input/view-model mutation actions, data binding,
   nested artboards, blend states, scripted actions, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime scheduled listener-fire-event records with occurrence and resolved
  event identity.
- State/transition start/end execution next to the existing fire-event hooks.
- A C++ probe-backed fixture with one bool input, two animation states, and
  `ListenerFireEvent` actions proving this order: source state start on first
  entry, then source state end, target state start, transition start, and
  transition end when the bool transition fires.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_scheduled_listener_fire_events_match_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
