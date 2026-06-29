# State Machine Scheduled Listener Input Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after scheduled
`ListenerFireEvent` actions. It defines the next smallest listener-action
surface: state/transition-owned `ListenerBoolChange`, `ListenerNumberChange`,
and `ListenerTriggerChange` actions mutate state-machine inputs at their
scheduled start/end occurrence.

## Formal Goal

Implement scheduled listener input-change actions for simple state-machine
advance in `rive-runtime`: carry imported state/transition listener input
actions from `rive-binary`, honor the scheduled occurrence bit in
`ListenerAction.flags`, mutate bool/number/trigger state-machine inputs
synchronously, and compare the resulting immediate transition behavior against
the C++ probe.

The goal is complete when the runtime slice can:

- Read state-attached and transition-attached `ListenerBoolChange`,
  `ListenerNumberChange`, and `ListenerTriggerChange` actions from the existing
  binary import graph.
- Match `ListenerAction::matchesScheduledOccurrence`, where bit `0` of
  `flags` selects `atStart` (`0`) or `atEnd` (`1`).
- Apply `ListenerBoolChange.value` with C++ semantics: `0` sets false, `1`
  sets true, and any other value toggles.
- Apply `ListenerNumberChange.value` to number inputs.
- Apply `ListenerTriggerChange` by firing trigger inputs.
- Perform these actions synchronously so an action on state start can satisfy a
  transition condition in the same public advance.
- Keep nested-input mutation, listener-owned dispatch, and view-model mutation
  out of scope.

## Scope Lock

This slice owns only layer-scheduled listener input-change actions attached to
states and transitions. It does not own nested inputs, listener-owned actions,
hit testing, pointer/keyboard/gamepad/semantic listener invocation,
`ListenerViewModelChange`, scripted listener actions, data binding, nested
artboards, animation keyframe callback events, blend states, or rendering.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing state-machine transition-condition behavior.

## Owned By This Slice

This slice owns:

- Runtime storage for imported scheduled listener input-change actions.
- Scheduled input mutation at state/transition start/end.
- C++ probe comparison coverage for bool, number, and trigger input changes
  causing an immediate transition in the same advance.

## Not Owned Yet

These remain future slices:

- Nested input mutation.
- Listener-owned actions and listener invocation from pointer, keyboard,
  gamepad, semantic, or reported-event matching.
- `ListenerViewModelChange` and scripted listener actions.
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

1. Is it required for a state/transition-owned listener input-change action to
   mutate a state-machine input during simple state-machine advance?
2. Does it reuse the existing input instance and transition-condition paths?
3. Can it be compared against the C++ probe without admitting nested inputs,
   listener-owned dispatch, hit testing, view-model mutation, data binding,
   nested artboards, blend states, scripted actions, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime scheduled listener input-action records with occurrence, target input
  index, and action value.
- State/transition start/end execution next to the existing scheduled listener
  fire-event hooks.
- A C++ probe-backed fixture family with one source state whose start listener
  action mutates a bool, number, or trigger input so the source transition fires
  immediately in the same advance.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_scheduled_listener_input_changes_match_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
