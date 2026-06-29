# State Machine Blend Percentage Exit-Time Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after pinning percentage-duration
blend transitions. It covers the corresponding behavior where
`BlendStateTransition.exitBlendAnimationId` does matter: percentage exit-time
gating.

## Formal Goal

Prove C++ parity for percentage exit-time gating on supported blend-state
transitions. In C++, `BlendStateTransition::exitTimeAnimationInstance` selects
the authored exit blend animation from the outgoing blend state, and
`StateTransition::exitTimeSeconds` uses that selected animation's duration when
`ExitTimeIsPercentage` is set.

The goal is complete when the runtime slice can:

- Compare a `BlendState1DInput -> AnimationState` transition with
  `EnableExitTime | ExitTimeIsPercentage` against C++.
- Use source blend animations with different durations so the selected
  `exitBlendAnimationId` is observable.
- Wait until the selected backing animation reaches the percentage exit time.
- Mix into the target animation after the exit gate opens.
- Preserve the C++ distinction that percentage mix duration remains
  AnimationState-only.

## Scope Lock

This slice owns only percentage exit-time gating for the existing
`BlendState1DInput` runtime subset. It does not own percentage mix duration
changes, `BlendState1DViewModel`, `BlendAnimationDirect.blendSource=dataBindId`,
bindable-property values, data binding, nested artboards, nested animation
remapping, listener-owned dispatch, hit testing, rendering, callback events,
color/non-transform reset, or non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `BlendStateTransition.exitBlendAnimationId` storage.
- Existing `RuntimeStateTransition::exit_time_seconds` behavior.
- Existing blend-state advance/apply behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required to match C++ percentage exit-time gating from a supported
   blend state?
2. Can the behavior be proved by varying authored blend animation durations?
3. Can it be compared against the C++ probe without admitting view-model
   blends, data binding, nested artboards, listeners, rendering, or non-double
   keyframes?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A synthetic C++ probe fixture with a `BlendState1DInput`,
  `BlendStateTransition`, `exitBlendAnimationId`, `ExitTimeIsPercentage`, a
  bool condition, and a target `AnimationState`.
- Source blend animations with different durations, proving the selected exit
  blend animation drives the gate.
- Runtime code changes only if the fixture exposes a parity gap.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_percentage_exit_time_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
