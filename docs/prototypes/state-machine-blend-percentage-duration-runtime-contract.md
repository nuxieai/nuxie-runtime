# State Machine Blend Percentage Duration Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after direct-blend transition
coverage. It pins a subtle C++ behavior: percentage transition duration from a
blend-state source does not use `BlendStateTransition.exitBlendAnimationId`.

## Formal Goal

Preserve C++ parity for blend-state transitions whose duration is marked as a
percentage. In C++, `StateTransition::mixTime` computes percentage duration
from the source state only when the source is an `AnimationState`; it does not
call `BlendStateTransition::exitTimeAnimation`. Therefore a percentage-duration
transition out of a `BlendState1DInput` or `BlendStateDirect` source currently
collapses to zero mix time.

The goal is complete when the runtime slice can:

- Compare a `BlendState1DInput -> AnimationState` transition with
  `DurationIsPercentage` against C++.
- Show that the transition behaves as zero-duration rather than as a timed
  mix using the selected exit blend animation duration.
- Preserve selected exit blend-animation lookup for exit-time gating only.
- Keep the behavior documented so future work does not accidentally broaden
  percentage-duration semantics away from C++.

## Scope Lock

This slice owns only the C++-observed zero-duration behavior for
percentage-duration transitions out of currently supported blend states. It
does not own changing C++ semantics, data-bound duration overrides,
`BlendState1DViewModel`, bindable-property blend values, data binding, nested
artboards, nested animation remapping, listener-owned dispatch, hit testing,
rendering, callback events, color/non-transform reset, or non-double keyframes.

The external seam remains:

- Existing `StateTransition::transition_duration_seconds` behavior.
- Existing `BlendStateTransition.exitBlendAnimationId` lookup for exit time.
- Existing transition reset behavior.

## Admission Rule

Before adding behavior to this slice, answer:

1. Does it preserve the C++ distinction between percentage mix duration and
   percentage exit time for blend transitions?
2. Can it be compared against the C++ probe without adding new blend sources?
3. Does it avoid changing runtime behavior unless the probe exposes a parity
   gap?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A synthetic C++ probe fixture with `BlendState1DInput`,
  `BlendStateTransition`, `DurationIsPercentage`, a bool condition, and a
  target `AnimationState`.
- A runtime comparison test that proves Rust and C++ both treat the transition
  as zero-duration.
- Runtime code changes only if the fixture exposes a parity gap.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_percentage_duration_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
