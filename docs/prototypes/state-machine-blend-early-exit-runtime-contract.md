# State Machine Blend Early-Exit Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after pause-on-exit coverage for
blend transitions. It covers the next transition edge case: interrupting an
active transition whose outgoing source was a supported blend state.

## Formal Goal

Prove C++ parity for early-exit/interruption when a supported blend-state
transition is actively mixing. In C++, a transition with `enableEarlyExit`
allows the current target state to evaluate its own outgoing transitions while
the previous source state is still being mixed. If an interruption transition
fires, the old transition source is dropped, the current target becomes the new
outgoing source, and the new transition follows the normal source/current
mixing rules.

The goal is complete when the runtime slice can:

- Start a non-zero-duration `BlendState1DInput -> AnimationState` transition
  with `EnableEarlyExit`.
- While that transition is active, fire a transition from the current target
  `AnimationState` to another `AnimationState`.
- Compare state-machine advance reports and component transform state against
  C++.
- Preserve transition reset behavior when the active blend-source transition
  is interrupted.
- Avoid admitting view-model/data-binding blend sources or listener-owned
  dispatch.

## Scope Lock

This slice owns only early-exit/interruption coverage for the existing
`BlendState1DInput` runtime subset and simple input-condition target-state
transitions. It does not own `BlendState1DViewModel`,
`BlendAnimationDirect.blendSource=dataBindId`, bindable-property values, data
binding, nested artboards, nested animation remapping, listener-owned dispatch,
hit testing, rendering, callback events, color/non-transform reset, or
non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `BlendStateTransition.exitBlendAnimationId` and transition-source
  storage.
- Existing transition reset behavior.
- Existing blend-state advance/apply behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required to interrupt an active transition that began from a supported
   blend state?
2. Does it reuse existing target-state transition evaluation and transition
   reset behavior?
3. Can it be compared against the C++ probe without adding view-model blends,
   data binding, nested artboards, listeners, rendering, or non-double
   keyframes?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A synthetic C++ probe fixture with `BlendState1DInput`,
  `BlendStateTransition` using `EnableEarlyExit`, a first bool condition, a
  target `AnimationState`, and a second bool-driven transition to another
  `AnimationState`.
- A runtime comparison test that starts the blend transition, fires the target
  interruption while the first transition is active, and compares C++ and Rust
  reports plus transform state.
- Runtime code changes only if the fixture exposes a parity gap.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_blend_state_early_exit_matches_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
