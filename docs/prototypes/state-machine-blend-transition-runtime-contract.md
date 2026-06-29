# State Machine Blend Transition Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after direct blend-state runtime
support. It defines the next smallest transition surface: transitions out of
simple blend states, with `BlendStateTransition.exitBlendAnimationId` used for
exit-time gating.

## Formal Goal

Implement blend-state transition runtime support in `rive-runtime`: carry the
imported `BlendStateTransition` exit blend-animation index, use the selected
backing linear animation instance for exit-time checks, and allow a simple
blend state to transition into an animation state while mixing the outgoing
blend state against the incoming state for millisecond transition durations.

The goal is complete when the runtime slice can:

- Import and retain `BlendStateTransition.exitBlendAnimationId` as the
  resolved blend-animation index already exposed by `rive-binary`.
- Evaluate exit time for a `BlendStateTransition` from `BlendState1DInput` or
  `BlendStateDirect` using the selected backing `LinearAnimationInstance`.
- Wait for the selected backing animation instance to reach the authored exit
  time before allowing the transition.
- Use the selected backing animation's duration for percentage exit-time
  conversion.
- Transition from a blend state to a simple `AnimationState`.
- Mix the outgoing blend state against the incoming animation state for
  non-percentage millisecond transition durations.
- Continue advancing the outgoing blend-state source while the transition is
  active unless `pauseOnExit` is set.
- Compare the resulting state-machine advance reports and component transform
  state against the C++ probe.

## Scope Lock

This slice owns only simple transitions out of existing runtime-supported blend
states. It does not own `BlendState1DViewModel`,
`BlendAnimationDirect.blendSource=dataBindId`, bindable-property instances,
data binding, percentage transition durations from blend states, animation
reset factory parity, nested artboards, nested animation remapping,
listener-owned dispatch, hit testing, rendering, animation keyframe callback
events, or non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing linear animation instance advance/apply behavior.
- Existing blend-state advance/apply behavior.

## Owned By This Slice

This slice owns:

- Runtime storage for `BlendStateTransition.exitBlendAnimationId` resolution.
- Exit-time lookup into current `BlendState1DInput` and `BlendStateDirect`
  backing animation instances.
- Transition-source storage for outgoing blend-state instances.
- C++ probe comparison coverage for a blend-state transition that waits on an
  exit blend animation before mixing into an animation state.

## Not Owned Yet

These remain future slices:

- `BlendState1DViewModel` and bindable-property driven blend values.
- `BlendAnimationDirect` with `blendSource=dataBindId`.
- Percentage transition durations from blend states.
- Animation reset factory parity for blend-state transitions.
- Nested artboards and nested animation/event remapping.
- Data binding, bindable-property conditions, view-model conditions, scripted
  conditions, and runtime property overrides.
- Listener-owned actions and pointer, keyboard, gamepad, semantic, or
  reported-event invocation.
- Animation `KeyFrameCallback` event sampling and delayed event reports.
- Non-double keyframes and custom keyframe interpolators.
- Draw-order mutation, clipping-stack mutation, draw commands, renderer paint
  allocation, and GPU work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for a supported blend state to transition into a simple
   animation state with C++ exit blend-animation timing?
2. Does it reuse existing linear animation and blend-state instance paths?
3. Can it be compared against the C++ probe without admitting data binding,
   view-model blends, reset factories, nested artboards, listeners, keyframe
   callback events, non-double keyframes, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime `exit_blend_animation_index` storage on transitions.
- A transition timing helper that can reference either the current animation
  state instance or a selected blend-state backing animation instance.
- Transition-source storage and apply/advance behavior for outgoing
  `BlendState1DInput` and `BlendStateDirect` instances.
- A C++ probe-backed fixture with a `BlendState1DInput`, a
  `BlendStateTransition` with `exitBlendAnimationId`, an input condition, and
  a target `AnimationState`.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_transition_exit_time_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
