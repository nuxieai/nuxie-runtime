# State Machine Blend Random Transition Runtime Contract

Date: 2026-06-29

This document extends roadmap item `#11` after blend-state early-exit coverage.
It narrows the next scheduler slice to random transition selection from the
already-supported runtime blend states.

## Formal Goal

Cover `LayerStateFlags::Random` transition selection when the source state is a
supported blend state. The runtime should use the same weighted-selection path
for `BlendState1DInput` and `BlendStateDirect` sources that it already uses for
plain animation states, then enter the selected target animation state with the
same transition mix behavior as C++.

The goal is complete when:

- Random selection is probe-covered for `BlendState1DInput` sources.
- Random selection is probe-covered for `BlendStateDirect` sources.
- Zero-weight allowed transitions are skipped when a later allowed transition
  has positive weight.
- Input conditions remain evaluated for every candidate and applied only for
  the selected transition, matching the existing random-transition rule.
- The selected transition can carry a non-zero duration and keep the blend
  source as the transition source while mixing into the selected target.
- The C++ probe comparison covers state-machine advance reports and the final
  component state for both blend source types.

## Scope Lock

This slice owns only random transition selection for imported
`BlendStateTransition` objects attached to supported blend-state sources:
`BlendState1DInput` and `BlendStateDirect`.

It does not own seeded random APIs, distribution tests, authored random
formula converters, view-model blends, nested animation remapping, data binding,
bindable-property overrides, listener-owned dispatch, hit testing,
pointer/keyboard/gamepad inputs, render behavior, animation keyframe callback
events, `StateMachineFireTrigger`, `ListenerViewModelChange`, or non-double
keyframe coverage.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is the source state an already-supported runtime blend state?
2. Does the behavior affect C++ parity for `LayerStateFlags::Random`
   transition selection at import/runtime advance time?
3. Can the behavior be compared with the existing C++ runtime probe without
   admitting view-model, data-binding, listener dispatch, nested artboard, or
   rendering semantics?

If the answer is no to all three, leave it for a later runtime crate/slice.

## Verification

Suggested focused verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_random_transition_matches_cpp_probe -- --nocapture
```

Full verification remains:

```sh
cargo test -p rive-runtime --test cpp_probe -- --nocapture
cargo check --workspace
make test
make cpp-compare
```
