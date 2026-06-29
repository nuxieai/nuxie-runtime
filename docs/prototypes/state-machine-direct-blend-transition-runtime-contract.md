# State Machine Direct Blend Transition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after transition animation reset
parity. It defines the next smallest blend-transition surface: C++ probe
coverage for transitions out of `BlendStateDirect` states that use the direct
blend sources already supported by `rive-runtime`.

## Formal Goal

Prove direct-blend transition parity for the currently supported runtime
subset: a `BlendStateDirect` with authored mix-value and root-number-input
blend animations can wait on `BlendStateTransition.exitBlendAnimationId`, then
mix into a simple `AnimationState` with the same state-machine advance reports
and transform state as C++.

The goal is complete when the runtime slice can:

- Build a C++ probe fixture with a `BlendStateDirect` source state and a
  simple `AnimationState` target.
- Include at least two direct blend animations so source-state mixing is
  genuinely exercised.
- Use `exitBlendAnimationId` to choose the backing source animation for
  exit-time gating.
- Mix the outgoing direct blend state against the incoming animation state for
  a non-zero millisecond transition duration.
- Reuse transition animation reset behavior without adding new reset scope.
- Compare state-machine advance reports and component transform state against
  the C++ probe.

## Scope Lock

This slice owns only direct-blend transition coverage for the existing
`BlendAnimationDirect` sources:

- authored `mixValue`
- root state-machine number input

It does not own `BlendAnimationDirect.blendSource=dataBindId`,
`BlendState1DViewModel`, bindable-property blend values, data binding,
view-model inputs, nested artboards, nested animation remapping,
listener-owned dispatch, pointer/keyboard/gamepad input routing, hit testing,
rendering, animation keyframe callback events, color/non-transform reset, or
non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing `BlendStateDirect` advance/apply behavior.
- Existing transition reset behavior from
  `state-machine-transition-animation-reset-runtime-contract.md`.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for a currently supported `BlendStateDirect` to transition
   into a simple `AnimationState` with C++ parity?
2. Does it reuse existing direct-blend source handling and transition mixing?
3. Can it be compared against the C++ probe without admitting data binding,
   view-model blends, nested artboards, listener dispatch, callback events,
   rendering, or non-double keyframes?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A synthetic C++ probe fixture containing a `BlendStateDirect`,
  `BlendStateTransition`, bool condition, and target `AnimationState`.
- A runtime comparison test that sets the number input, advances to the exit
  time, triggers the transition, advances through the mix, and compares C++ and
  Rust reports.
- Runtime code changes only if the fixture exposes a direct-blend transition
  parity gap.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_direct_blend_state_transition_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
