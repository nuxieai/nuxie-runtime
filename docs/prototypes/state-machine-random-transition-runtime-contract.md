# State Machine Random Transition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after early-exit transition
interruption. It defines the next smallest transition-scheduler surface:
source states flagged with `LayerStateFlags::Random` choose among allowed
outgoing transitions by `StateTransition.randomWeight` instead of taking the
first allowed transition.

## Formal Goal

Implement random transition selection for simple `AnimationState ->
AnimationState` transitions in `rive-runtime`: read the source state's `flags`,
read each transition's `randomWeight`, evaluate all outgoing transitions from a
random state, sum the weights for allowed transitions, choose the weighted
transition with the same default C++ test random value, consume inputs only for
the selected transition, preserve `waitingForExit` for transitions blocked by
exit time, and compare advance reports plus resulting component state against
the C++ probe.

The goal is complete when the runtime slice can:

- Read and admit `LayerState.flags` for runtime state-machine states.
- Recognize `LayerStateFlags::Random`.
- Read `StateTransition.randomWeight`, defaulting omitted values to `1`.
- For random states, evaluate all supported outgoing transitions before
  changing state.
- Ignore allowed transitions with zero random weight when the total positive
  weight selects another candidate.
- Consume trigger/layer inputs only on the transition that is actually selected.
- Preserve existing first-allowed behavior for non-random states.
- Compare the selected target animation and final component state against the
  C++ probe.

## Scope Lock

This slice owns only the random selection rule for already-supported simple
animation-state transitions. It does not own authoring-time transition order
sorting, exposed seeded random APIs, probabilistic distribution tests, random
formula converters, blend states, events, listener actions, data binding,
nested artboards, scripted conditions, or rendering.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Runtime state flags needed to detect random transition states.
- Runtime transition random weights.
- Weighted transition candidate selection for source states flagged random.
- C++ probe comparison coverage for a random source state where the first
  otherwise-allowed transition has zero weight and the second transition has
  positive weight.

## Not Owned Yet

These remain future slices:

- Seeded or injectable random providers for public runtime users.
- Statistical/probability distribution assertions.
- Blend states and blend animations.
- Fire events, listener actions, pointer/keyboard/gamepad/semantic listeners,
  focus, and hit testing.
- Data binding, bindable-property conditions, view-model conditions, scripted
  conditions, and runtime property overrides.
- Nested artboards and nested animation/input remapping.
- Non-double keyframes and custom keyframe interpolators.
- Draw-order mutation, clipping-stack mutation, draw commands, renderer paint
  allocation, and GPU work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required to match C++ transition selection for a source state with
   `LayerStateFlags::Random`?
2. Does it reuse the existing simple input/condition, exit-time, transition
   mix, timing, handoff, early-exit, and interpolation paths?
3. Can it be compared against the C++ probe without admitting blend states,
   events, listeners, data binding, nested artboards, scripted conditions, or
   rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime `LayerState.flags` storage and `LayerStateFlags::Random` detection.
- Runtime `StateTransition.randomWeight` storage with the generated default of
  `1`.
- A random-state transition-selection branch that evaluates all outgoing
  candidates, sums allowed random weights, and selects the weighted candidate
  using the C++ test provider's default random value.
- A C++ probe-backed fixture with one bool input and three animation states:
  the random source has two allowed transitions, the first with random weight
  `0` and the second with random weight `1`, so the selected target is
  deterministic.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_random_transition_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
