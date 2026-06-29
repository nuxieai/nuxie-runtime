# State Machine Early Exit Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after transition timing and
interpolator support. It defines the next smallest transition-scheduler
surface: a simple animation-state transition can be interrupted while it is
still mixing when the active transition has `EnableEarlyExit`.

## Formal Goal

Implement early-exit transition interruption for simple
`AnimationState -> AnimationState` transitions in `rive-runtime`: keep blocking
additional transitions while a timed transition is mixing unless the active
transition enables early exit, allow normal transition search to continue when
early exit is enabled, start the interrupting transition from the current state
with the current mix as `mixFrom`, and compare advance reports and resulting
component state against the C++ probe.

The goal is complete when the runtime slice can:

- Read and admit the `EnableEarlyExit` transition flag for otherwise supported
  simple transitions.
- Continue blocking state changes during timed mixing when early exit is not
  enabled.
- Allow a transition from the current state while a previous transition is
  mixing when that previous transition enables early exit.
- Preserve existing source/current animation mix state when an interruption
  occurs.
- Compare current animation reports and final component state against the C++
  probe.

## Scope Lock

This slice owns only early-exit interruption for simple animation-state
transitions already covered by the input, timing, handoff, and transition
interpolator slices. It does not own random transitions, events, listener
actions, blend states, data binding, nested artboards, scripted conditions,
view-model conditions, or renderer work.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.

## Owned By This Slice

This slice owns:

- Active transition early-exit state on layer instances.
- The scheduler gate that distinguishes interruptible from non-interruptible
  mixes.
- C++ probe comparison coverage for interrupting a timed transition with a
  second input-driven transition.

## Not Owned Yet

These remain future slices:

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

1. Is it required to interrupt a currently mixing simple transition because its
   active transition has `EnableEarlyExit`?
2. Does it reuse the existing simple input/condition, transition mix, timing,
   handoff, and interpolation paths?
3. Can it be compared against the C++ probe without admitting random
   transitions, events, listeners, data binding, blend states, nested artboards,
   or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime transition `enable_early_exit` state.
- Scheduler gating that only blocks during mixing when the active transition is
  not interruptible.
- A C++ probe-backed fixture with two bool inputs and three animation states:
  first input starts an interruptible timed transition, second input interrupts
  it before completion.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_early_exit_transition_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
