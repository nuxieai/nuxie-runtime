# State Machine Cubic Transition Interpolator Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after percentage timing support. It
defines the next smallest transition-mix surface: a simple animation-state
transition can ease its source and target mix values through a resolved
`CubicEaseInterpolator`, matching C++ `StateMachineLayerInstance::apply()`.

## Formal Goal

Implement `CubicEaseInterpolator` support for simple
`AnimationState -> AnimationState` transition mixing in `rive-runtime`: resolve
the already-imported transition interpolator reference, transform `mixFrom` and
`mix` through C++ cubic-bezier solver semantics when applying the outgoing and
incoming animation states, compare advance reports and resulting component
state against the C++ probe, and keep other interpolator families out of scope.

The goal is complete when the runtime slice can:

- Read transition interpolator metadata resolved by `rive-binary`.
- Admit transitions whose interpolator is a `CubicEaseInterpolator`.
- Transform source `mixFrom` and target `mix` with C++ cubic solver semantics.
- Preserve existing linear behavior when no supported interpolator is present.
- Reject unsupported transition interpolator families from this runtime slice
  instead of pretending they are linear.
- Compare current animation reports and final component state against the C++
  probe.

## Scope Lock

This slice owns only `CubicEaseInterpolator::transform()` for transition mix
values. It does not own `ElasticInterpolator`, `CubicValueInterpolator`,
`ScriptedInterpolator`, keyframe interpolation, data-bound interpolators,
transition early exit, random transitions, blend states, events, listeners,
nested artboards, or renderer work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- A runtime transition-interpolator enum for supported transition mix easing.
- C++ cubic-bezier solver parity for `CubicEaseInterpolator`.
- Applying the interpolator to both outgoing and incoming transition mix
  factors.
- C++ probe comparison coverage for a timed transition with a non-linear cubic
  ease curve.

## Not Owned Yet

These remain future slices:

- `ElasticInterpolator`.
- `CubicValueInterpolator` and value-specific interpolation.
- `ScriptedInterpolator` and script state.
- Data-bound transition duration overrides.
- Early exit and transition interruption.
- Random transitions and state randomization.
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

1. Is it required for `CubicEaseInterpolator` to transform a transition mix
   factor?
2. Does it reuse the existing simple input/condition and timed transition mix
   paths?
3. Can it be compared against the C++ probe without admitting elastic/scripted
   interpolators, data binding, early exit, events, listeners, blend states,
   nested artboards, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime cubic transition interpolator metadata.
- A C++-matching cubic-bezier solver.
- Mixed source/current animation application through transformed mix values.
- A C++ probe-backed fixture for a timed transition with a
  `CubicEaseInterpolator` referenced by `StateTransition.interpolatorId`.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_cubic_transition_interpolator_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
