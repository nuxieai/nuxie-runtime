# State Machine BlendState1D Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after scheduled listener input
changes. It defines the smallest blend-state runtime surface: a
`BlendState1DInput` state driven by a root `StateMachineNumber` input and
simple authored `BlendAnimation1D` children that point at existing linear
animations.

## Formal Goal

Implement the first runtime seam for input-backed one-dimensional blend states
in `nuxie-runtime`: import authored `BlendState1DInput` blend animations from
`nuxie-binary`, instantiate per-blend-animation linear animation instances,
advance them with the state machine, compute C++-matching blend weights from
the number input, and apply the weighted animations to the artboard.

The goal is complete when the runtime slice can:

- Enter a `BlendState1DInput` from an `EntryState` transition.
- Resolve the blend state's root number input by `inputId`.
- Resolve each `BlendAnimation1D.animationId` to an existing artboard linear
  animation.
- Use authored `BlendAnimation1D.value` entries in import order, matching the
  C++ `animationIndex()` search behavior.
- Advance every backing linear animation instance while it can keep going.
- Set the two neighboring blend-animation mix values using C++ 1D blend math.
- Apply only non-zero weighted animation instances through the existing linear
  animation application path.
- Keep the state machine advancing while the current state is a blend state.
- Compare final component transform state against the C++ probe.

## Scope Lock

This slice owns only simple `BlendState1DInput` states whose input resolves to
a root `StateMachineNumber` and whose blend animations resolve to existing
linear animations. It intentionally does not own `BlendState1DViewModel`,
`BlendStateDirect`, `BlendAnimationDirect`, transition exit blend-animation
time, blend-state transitions, animation reset factories, nested artboards,
nested animation remapping, data binding, bindable-property overrides,
listener-owned dispatch, hit testing, rendering, animation keyframe callback
events, or non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` number input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing linear animation instance advance/apply behavior.

## Owned By This Slice

This slice owns:

- Runtime storage for imported `BlendState1DInput` state data.
- Runtime storage for imported `BlendAnimation1D` value and animation index
  data.
- Per-state blend animation instances.
- Blend weight calculation for root-number-input 1D blends.
- C++ probe comparison coverage for a simple authored blend state.

## Not Owned Yet

These remain future slices:

- `BlendState1DViewModel` and bindable-property driven blend values.
- `BlendStateDirect` and `BlendAnimationDirect`.
- Transition exit blend-animation resolution at runtime.
- Transitions out of blend states, including exit time sourced from blend
  animations.
- Animation reset factory parity for blend states.
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

1. Is it required for a root-number-input `BlendState1DInput` to advance and
   apply simple authored linear animations?
2. Does it reuse the existing linear animation instance and transform apply
   paths?
3. Can it be compared against the C++ probe without admitting direct blends,
   view-model blends, blend-state transitions, nested artboards, data binding,
   reset factories, listeners, keyframe callback events, non-double keyframes,
   or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime blend-state records for `BlendState1DInput`.
- Runtime blend-animation records for `BlendAnimation1D`.
- A blend-state instance that owns one `LinearAnimationInstance` per valid
  blend animation.
- A C++ probe-backed fixture with one number input, one blend state, and two
  authored blend animations driving the same node transform property.

Suggested verification:

```sh
cargo test -p nuxie-runtime --test cpp_probe state_machine_blend_state_1d_input_matches_cpp_probe -- --nocapture
cargo test -p nuxie-runtime --test cpp_probe
make test
make cpp-compare
```
