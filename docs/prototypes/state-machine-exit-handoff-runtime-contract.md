# State Machine Exit Handoff Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after absolute exit-time gating. It
defines the next transition-scheduler handoff surface: when a transition is
taken out of an `AnimationState`, C++ can hand spilled source time into the
target state and can hold the source animation while a timed transition mixes.

## Formal Goal

Implement the remaining simple animation-state transition handoff behavior in
`nuxie-runtime`: when a source animation spills past its end before a transition
is taken, advance the target animation by that spilled time; when a transition
uses `PauseOnExit`, keep the source animation fixed while mixing; when
`PauseOnExit` combines with absolute exit time, hold the source at the absolute
exit time before applying the mix. Compare advance reports and resulting
component state against the C++ probe.

The goal is complete when the runtime slice can:

- Preserve source `LinearAnimationInstance::spilled_time` across a state change.
- Advance the target `AnimationState` by the source spilled time immediately
  after the transition is taken.
- Read and admit the `PauseOnExit` transition flag for simple animation-state
  transitions.
- Hold the transition source animation instead of advancing it during timed
  mixing when `PauseOnExit` is set.
- For `EnableExitTime | PauseOnExit`, seek the source animation to the absolute
  millisecond exit time before applying it as the transition source.
- Compare current animation reports and final component state against the C++
  probe.

## Scope Lock

This slice owns only simple `AnimationState -> AnimationState` handoff behavior
around already-supported input conditions, absolute exit time, and millisecond
transition durations. It does not own exit-time-as-percentage,
duration-as-percentage, early exit/interruption, transition interpolators, data
binding duration overrides, events, listener actions, random transitions, blend
states, nested artboards, or renderer work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Source spilled-time handoff into the target animation state.
- Per-transition-source pause/hold state.
- Absolute exit-time seeking for paused source animations.
- C++ probe comparison coverage for spilled-time handoff and pause-on-exit
  timed mixing.

## Not Owned Yet

These remain future slices:

- Exit-time-as-percentage.
- Duration-as-percentage and data-bound transition duration overrides.
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

1. Is it required to hand off source animation spill or hold a paused source
   during a simple transition mix?
2. Does it reuse the existing simple input/condition, absolute exit-time, and
   millisecond duration paths?
3. Can it be compared against the C++ probe without admitting percentage
   timing, early exit, interpolators, events, listeners, data binding, blend
   states, nested artboards, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Target advancement by source spilled time when changing state.
- A per-layer flag for paused transition sources.
- Source-animation seeking for `EnableExitTime | PauseOnExit`.
- C++ probe-backed tests for one-shot spilled-time handoff and pause-on-exit
  timed mixing.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_transition_handoff_matches_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
