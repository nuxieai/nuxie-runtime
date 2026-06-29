# State Machine Input Runtime Contract

Date: 2026-06-28

This document continues roadmap item `#11` after the first
`StateMachineInstance` animation-state seam. It defines the next smallest
state-machine runtime surface: authored inputs can drive simple zero-duration
transitions without admitting the full state-machine scheduler.

## Formal Goal

Implement the next narrow `StateMachineInstance` runtime seam in
`rive-runtime`: add runtime-owned bool, number, and trigger input instances for
imported state machines; expose input lookup and mutation on
`StateMachineInstance`; evaluate simple bool/number/trigger transition
conditions for zero-duration transitions; compare input-driven state changes and
resulting component state against the C++ probe; and keep exit time, transition
duration/mixing, random transitions, blend states, listener actions, events,
data binding, nested animations/artboards, draw/render behavior, and non-double
keyframes out of scope.

The goal is complete when the runtime slice can:

- Build input instances from imported state-machine inputs.
- Preserve input order and expose input count/name/kind/value through the
  state-machine instance.
- Mutate bool and number inputs and fire trigger inputs through public methods.
- Evaluate imported `TransitionBoolCondition`, `TransitionNumberCondition`, and
  `TransitionTriggerCondition` records for zero-duration transitions.
- Consume trigger inputs after an advance with C++ one-use-per-layer behavior.
- Report input-driven state-machine advancement through the C++ probe.
- Compare resulting component state against the C++ probe.

## Scope Lock

This slice makes state machines controllable. It does not own the full
transition scheduler, event system, listener system, data-binding system, blend
state playback, or nested artboard integration.

The external seam for this slice is:

- `StateMachineInstance::input_count()`.
- `StateMachineInstance::input(index)`.
- `StateMachineInstance::input_named(name)`.
- `StateMachineInstance::set_bool(index, value)`.
- `StateMachineInstance::set_number(index, value)`.
- `StateMachineInstance::fire_trigger(index)`.
- Existing `ArtboardInstance::advance_state_machine_instance(instance, seconds)`.

## Owned By This Slice

This slice owns:

- Runtime input instances for imported bool, number, and trigger inputs.
- Input value mutation and C++-style `needs_advance` marking.
- Simple condition evaluation for bool, number, and trigger transition
  conditions.
- Trigger use tracking and post-advance clearing.
- C++ probe commands for ordered input mutation and state-machine advancement.

## Not Owned Yet

These remain future slices:

- Exit time, waiting-for-exit, early exit, and transition duration/mixing.
- Random transitions and state randomization.
- Blend states and blend animations.
- Fire events, listener actions, pointer/keyboard/gamepad/semantic listeners,
  focus, and hit testing.
- Data-binding contexts, observers, dirty queues, converters, property
  recorders, view-model conditions, and bindable-property conditions.
- Nested artboards and nested animation/input remapping.
- Non-double keyframes and custom interpolators.
- Draw-order mutation, clipping-stack mutation, draw commands, renderer paint
  allocation, and GPU work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for bool, number, or trigger inputs to drive a simple
   zero-duration state transition?
2. Does it use imported state-machine input and transition-condition facts that
   already exist in `rive-binary`?
3. Can it be compared against the C++ probe without admitting transition
   mixing, events, listeners, data binding, nested artboards, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime input records and `StateMachineInputInstance` accessors.
- Public input mutation methods for bool, number, and trigger inputs by index,
  with optional name lookup for ergonomic parity with C++.
- Condition records on runtime transitions for bool, number, and trigger
  conditions.
- C++ probe options for ordered state-machine input mutation.
- Tests for bool, number, and trigger input transitions.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe
cargo test -p rive-runtime
make test
make cpp-compare
```
