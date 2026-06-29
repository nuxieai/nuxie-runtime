# State Machine Timed Transition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the input-driven
state-machine seam. It defines the next smallest state-machine runtime surface:
simple nonzero-duration transitions between animation states can mix from the
previous animation state to the current one across advances.

## Formal Goal

Implement the next narrow `StateMachineInstance` runtime seam in
`rive-runtime`: add timed transition state for simple
`AnimationState -> AnimationState` transitions with nonzero duration, track
source/current states and transition mix across advances, apply both backing
linear animations with C++-matching mix weights, block additional transitions
while mixing unless early-exit support is explicitly added later, compare
advance reports and resulting component state against the C++ probe, and keep
exit time, early exit, random transitions, blend states, events, listener
actions, data binding, nested artboards/animations, custom interpolators,
draw/render behavior, and non-double keyframes out of scope.

The goal is complete when the runtime slice can:

- Start a timed transition after a simple input or unconditional transition is
  allowed.
- Retain the previous animation state as transition source while the current
  animation state advances.
- Advance the transition mix from `0.0` to `1.0` using C++ duration semantics
  for millisecond durations.
- Apply the source animation with the previous mix and the current animation
  with the current mix, matching C++ transform results for simple
  `KeyFrameDouble` animation states.
- Continue reporting current animation time, changed-state count, and advance
  return value through the C++ probe.
- Block further transitions while mixing.
- Compare resulting component state against the C++ probe.

## Scope Lock

This slice owns only the transition state needed to mix simple animation states.
It does not own exit timing, transition interruption, event firing, listener
actions, blend-state evaluation, data binding, nested animations, or renderer
work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Per-layer transition source animation state.
- Per-layer transition mix and previous mix.
- Millisecond transition duration resolution from imported transition records.
- Mixed source/current animation application for simple animation states.
- C++ probe comparison coverage for timed transition advancement.

## Not Owned Yet

These remain future slices:

- Exit time, waiting-for-exit, pause-on-exit holds, and spilled-time handoff.
- Early exit and transition interruption.
- Duration-as-percentage and data-bound transition duration overrides.
- Transition interpolators/custom easing.
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

1. Is it required to mix a simple nonzero-duration transition between two
   `AnimationState` instances?
2. Does it reuse the existing state-machine input/condition and
   `LinearAnimationInstance` animation-state paths?
3. Can it be compared against the C++ probe without admitting exit time, early
   exit, events, listeners, data binding, blend states, nested artboards, or
   rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime transition source state and mix fields on layer instances.
- Millisecond duration handling for nonzero-duration transitions.
- Mixed source/current animation application with mix values matching C++.
- A C++ probe-backed fixture for an input-driven
  `AnimationState -> AnimationState` transition with `duration = 1000`.
- Tests at `0.0`, halfway, and completion advances.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe
cargo test -p rive-runtime
make test
make cpp-compare
```
