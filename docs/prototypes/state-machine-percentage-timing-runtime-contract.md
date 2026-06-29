# State Machine Percentage Timing Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after exit-time and transition
handoff support. It defines the next smallest timing surface: simple
animation-state transitions can resolve transition duration and exit time as a
percentage of the source animation duration, matching C++.

## Formal Goal

Implement percentage timing for simple `AnimationState -> AnimationState`
transitions in `rive-runtime`: when `DurationIsPercentage` is set, compute mix
duration from the source animation duration; when `ExitTimeIsPercentage` is
set, compute the exit threshold from the source animation duration, using the
source start time only for pause-on-exit seeking just like C++. Compare advance
reports and resulting component state against the C++ probe.

The goal is complete when the runtime slice can:

- Read and admit `DurationIsPercentage` for simple transitions.
- Resolve percentage transition duration from the outgoing animation duration.
- Read and admit `ExitTimeIsPercentage` for simple transitions.
- Resolve percentage exit time from the outgoing animation duration.
- Preserve existing absolute-duration, absolute-exit, pause-on-exit, and
  spilled-time behavior.
- Compare current animation reports and final component state against the C++
  probe.

## Scope Lock

This slice owns only percentage timing for already-supported simple animation
state transitions. It does not own data-bound duration overrides, early
exit/interruption, transition interpolators, random transitions, events,
listener actions, blend states, nested artboards, or renderer work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Percentage mix duration resolution.
- Percentage exit-time resolution for transition allowance checks.
- Percentage exit-time source seeking when combined with pause-on-exit.
- C++ probe comparison coverage for both percentage duration and percentage
  exit time.

## Not Owned Yet

These remain future slices:

- Data-bound transition duration overrides.
- Early exit and transition interruption.
- Transition interpolators/custom easing.
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

1. Is it required to compute transition duration or exit time as a percentage
   of the outgoing animation duration?
2. Does it reuse the existing simple input/condition, transition mix, exit-time,
   pause-on-exit, and spilled-time paths?
3. Can it be compared against the C++ probe without admitting data binding,
   early exit, interpolators, events, listeners, blend states, nested artboards,
   or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Percentage mix duration resolution for timed transitions.
- Percentage exit-time resolution for delayed transitions.
- C++ probe-backed tests for `DurationIsPercentage` and
  `ExitTimeIsPercentage`.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_percentage_timing_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
