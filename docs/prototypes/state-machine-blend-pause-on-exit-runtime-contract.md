# State Machine Blend Pause-On-Exit Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after percentage exit-time coverage
for blend transitions. It covers the next smallest transition behavior:
`pauseOnExit` source-hold semantics for supported blend-state transitions.

## Formal Goal

Prove C++ parity for `pauseOnExit` on transitions out of supported blend
states. In C++, `pauseOnExit` sets `m_holdAnimationFrom` so the outgoing state
does not advance while it is being mixed into the target. The separate
exit-time snap performed by `StateTransition::applyExitCondition` is only valid
for outgoing `AnimationState` instances; blend states are held at their current
advanced time rather than snapped to an exit-time sample.

The goal is complete when the runtime slice can:

- Compare a `BlendState1DInput -> AnimationState` transition with
  `EnableExitTime | PauseOnExit` against C++.
- Wait for the selected `exitBlendAnimationId` backing animation to reach the
  exit time.
- Hold the outgoing blend-state source during the transition mix instead of
  advancing it further.
- Preserve existing transition animation reset behavior.
- Avoid adding AnimationState-only exit-time snapping to blend sources.

## Scope Lock

This slice owns only `pauseOnExit` source-hold behavior for the existing
`BlendState1DInput` runtime subset. It does not own view-model blends,
`BlendAnimationDirect.blendSource=dataBindId`, bindable-property blend values,
data binding, nested artboards, nested animation remapping, listener-owned
dispatch, hit testing, rendering, callback events, color/non-transform reset,
or non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `BlendStateTransition.exitBlendAnimationId` exit-time lookup.
- Existing transition reset behavior.
- Existing blend-state advance/apply behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required to hold an outgoing supported blend state during transition
   mixing when `pauseOnExit` is set?
2. Does it preserve C++'s AnimationState-only exit-time snap behavior?
3. Can it be compared against the C++ probe without adding view-model blends,
   data binding, nested artboards, listeners, rendering, or non-double
   keyframes?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A synthetic C++ probe fixture with a `BlendState1DInput`,
  `BlendStateTransition`, `exitBlendAnimationId`, `EnableExitTime |
  PauseOnExit`, a bool condition, and a target `AnimationState`.
- A runtime comparison test that advances the outgoing blend state, triggers
  the transition, advances through the mix, and compares C++ and Rust state
  machine reports plus transform state.
- Runtime code changes only if the fixture exposes a parity gap.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_blend_state_pause_on_exit_matches_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
