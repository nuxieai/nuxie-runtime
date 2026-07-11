# State Machine Exit-Time Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after simple timed transition
mixing. It defines the next smallest state-machine scheduler surface: simple
transitions can be conditionally allowed but delayed until the current
`AnimationState` reaches an absolute millisecond exit time.

## Formal Goal

Implement absolute exit-time gating for simple
`AnimationState -> AnimationState` transitions in `nuxie-runtime`: evaluate the
same input conditions as the existing transition path, report a waiting state
when conditions pass before the source animation reaches the configured exit
time, keep the state machine advancing while waiting, take the transition once
the source animation crosses the absolute exit time, and compare advance reports
and resulting component state against the C++ probe.

The goal is complete when the runtime slice can:

- Read `StateTransition.exitTime` and the `EnableExitTime` flag from imported
  transition records.
- Treat exit time as an AND condition after existing input conditions.
- Return "waiting for exit" before the current animation state's total time
  reaches the absolute millisecond exit time.
- Keep `StateMachineInstance::advance` returning true while waiting for exit.
- Take the transition once the current animation state's total time reaches or
  crosses the configured exit time.
- Continue to reuse the existing transition duration/mixing implementation once
  a transition becomes allowed.
- Compare current animation reports and final component state against the C++
  probe.

## Scope Lock

This slice owns only absolute millisecond exit-time gating for simple animation
states. It does not own percentage exit times, pause-on-exit holds, spilled-time
handoff, early exit/interruption, transition interpolators, random transition
selection, events, listener actions, blend states, data binding, nested
artboards, or renderer work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Importing exit-time scalar data into runtime transition records.
- Per-layer waiting-for-exit state.
- Absolute millisecond exit-time checks against the current animation
  instance's total time.
- C++ probe comparison coverage for a delayed input-driven transition.

## Not Owned Yet

These remain future slices:

- Exit-time-as-percentage.
- Pause-on-exit and source animation holding at the exit frame.
- Spilled-time handoff from the source animation to the target animation.
- Early exit and transition interruption.
- Duration-as-percentage and data-bound transition duration overrides.
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

1. Is it required to delay a simple condition-allowed transition until an
   `AnimationState` crosses an absolute millisecond exit time?
2. Does it reuse the existing input/condition and transition duration/mixing
   paths?
3. Can it be compared against the C++ probe without admitting percentage exit
   time, pause-on-exit, early exit, events, listeners, data binding, blend
   states, nested artboards, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime transition `exit_time` data.
- Per-layer `waiting_for_exit` tracking.
- A transition allowance result that distinguishes `no`, `waitingForExit`, and
  `yes`.
- Absolute millisecond exit-time checking using `LinearAnimationInstance`
  `last_total_time` and `total_time`, including loop-relative adjustment for
  looping source animations.
- A C++ probe-backed fixture for an input-driven transition with
  `EnableExitTime` and `exitTime = 1000`.
- Tests before exit, at crossing, and after the resulting state change.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_exit_time_transition_matches_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
