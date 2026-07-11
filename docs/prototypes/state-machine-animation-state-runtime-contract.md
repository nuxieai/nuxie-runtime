# State Machine Animation State Runtime Contract

Date: 2026-06-28

This document continues roadmap item `#11` after direct
`LinearAnimation::apply` parity and the first `LinearAnimationInstance`
playback seam. It defines the smallest state-machine runtime surface that can
prove state-machine advancement mutates the same headless artboard instance
without admitting the full state-machine scheduler yet.

## Formal Goal

Implement a narrow first `StateMachineInstance` runtime seam in `nuxie-runtime`:
add runtime-owned state-machine instance/layer/animation-state playback for the
simplest C++ path where a state-machine layer enters an `AnimationState` backed
by a linear animation, reuse the existing `LinearAnimationInstance`
advance/apply path to mutate an `ArtboardInstance`, compare state-machine
advancement and resulting component state against the C++ probe, and keep
inputs, transition conditions, transition mixing, events, listener actions,
blend states, data binding, nested animations, draw/render behavior, and
non-double keyframes out of scope.

The goal is complete when the runtime slice can:

- Build copied state-machine definitions while constructing an
  `ArtboardInstance`.
- Create a state-machine instance for an artboard state-machine index.
- Initialize each layer at its C++ entry state.
- Follow an unconditional zero-duration entry transition into an
  `AnimationState`.
- Advance and apply the backing linear animation through the existing
  `LinearAnimationInstance` path.
- Report current animation time, changed-state count, and advance return value
  for C++ comparison.
- Compare the resulting component state against the C++ probe.

## Scope Lock

This slice owns only enough state-machine playback to bridge from static
state-machine topology to already-supported linear animation playback. It does
not own the general transition scheduler or input/event/data-binding systems.

The external seam for this slice is:

- `ArtboardInstance::state_machine_instance(index)`.
- `ArtboardInstance::advance_state_machine_instance(instance, seconds)`.

## Owned By This Slice

This slice owns:

- Runtime-owned state-machine, layer, and simple state records.
- Initial layer state selection for entry states.
- Unconditional zero-duration transitions from entry/any/current states.
- Animation-state instances backed by existing `LinearAnimationInstance`
  values.
- C++ probe reporting for state-machine advancement.

## Not Owned Yet

These remain future slices:

- State-machine input instances and input mutation.
- Transition conditions, random transitions, exit-time behavior, waiting for
  exit, early exit, and transition mixing.
- Blend states and blend animations.
- Fire events, listener actions, pointer/keyboard/gamepad/semantic listeners,
  focus, and hit testing.
- Data-binding contexts, observers, dirty queues, converters, property
  recorders, and cloned bindable properties.
- Nested artboards and nested animation remapping.
- Non-double keyframes and custom interpolators.
- Draw-order mutation, clipping-stack mutation, draw commands, renderer paint
  allocation, and GPU work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for a C++ state-machine layer to enter and play a simple
   linear-animation `AnimationState`?
2. Does it reuse the existing `LinearAnimationInstance` and transform
   `KeyFrameDouble` application path?
3. Can it be compared against the C++ probe without admitting inputs,
   transition conditions, events, data binding, nested animations, or
   rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime state-machine definitions in `nuxie-runtime`.
- A state-machine instance object with per-layer current state.
- Minimal transition following for unconditional zero-duration transitions into
  animation states.
- C++ probe option `--runtime-advance-state-machine stateMachineIndex seconds`.
- C++ probe JSON reports for advance return value, changed-state count, and
  current animation time.
- Tests for entering an animation state and for a second advance moving the
  backing linear animation clock.

Suggested verification:

```sh
cargo test -p nuxie-runtime
make test
make cpp-compare
```
