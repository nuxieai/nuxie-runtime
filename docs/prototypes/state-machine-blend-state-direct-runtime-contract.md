# State Machine BlendStateDirect Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after the first
`BlendState1DInput` runtime seam. It defines the next smallest blend-state
surface: `BlendStateDirect` states with `BlendAnimationDirect` children whose
mix is driven by either an authored mix value or a root `StateMachineNumber`
input.

## Formal Goal

Implement direct blend-state runtime support in `rive-runtime`: import
`BlendStateDirect` blend animations from `rive-binary`, instantiate
per-blend-animation linear animation instances, advance them with the state
machine, compute C++-matching direct mix values for supported blend sources,
and apply the weighted animations to the artboard.

The goal is complete when the runtime slice can:

- Enter a `BlendStateDirect` from an `EntryState` transition.
- Resolve each `BlendAnimationDirect.animationId` to an existing artboard
  linear animation.
- Apply `blendSource=inputId` by reading a root `StateMachineNumber` input and
  clamping `value / 100` to `[0, 1]`.
- Apply `blendSource=mixValue` by clamping authored `mixValue / 100` to
  `[0, 1]`.
- Advance every backing linear animation instance while it can keep going.
- Apply only non-zero weighted animation instances through the existing linear
  animation application path.
- Keep the state machine advancing while the current state is a direct blend
  state.
- Compare final component transform state against the C++ probe.

## Scope Lock

This slice owns only simple `BlendStateDirect` states whose direct blend
animations use `blendSource=inputId` or `blendSource=mixValue`. It does not own
`blendSource=dataBindId`, bindable-property instances, data binding,
`BlendState1DViewModel`, transition exit blend-animation time, blend-state
transitions, animation reset factories, nested artboards, nested animation
remapping, listener-owned dispatch, hit testing, rendering, animation keyframe
callback events, or non-double keyframes.

The external seam remains:

- Existing `StateMachineInstance` number input mutation APIs.
- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing linear animation instance advance/apply behavior.

## Owned By This Slice

This slice owns:

- Runtime storage for imported `BlendStateDirect` state data.
- Runtime storage for imported `BlendAnimationDirect` blend source data.
- Per-state direct blend animation instances.
- Direct blend mix calculation for authored mix values and root number inputs.
- C++ probe comparison coverage for a simple authored direct blend state.

## Not Owned Yet

These remain future slices:

- `BlendAnimationDirect` with `blendSource=dataBindId`.
- `BlendState1DViewModel` and bindable-property driven blend values.
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

1. Is it required for a simple `BlendStateDirect` to advance and apply authored
   linear animations with input or mix-value blend sources?
2. Does it reuse the existing linear animation instance and transform apply
   paths?
3. Can it be compared against the C++ probe without admitting data binding,
   bindable-property instances, view-model blends, blend-state transitions,
   nested artboards, reset factories, listeners, keyframe callback events,
   non-double keyframes, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- Runtime direct blend-state records for `BlendStateDirect`.
- Runtime blend-animation records for `BlendAnimationDirect` with supported
  blend sources.
- A direct blend-state instance that owns one `LinearAnimationInstance` per
  valid blend animation.
- A C++ probe-backed fixture with one number input, one direct blend state, and
  two authored blend animations driving the same node transform property: one
  via `mixValue`, one via `inputId`.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_direct_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
