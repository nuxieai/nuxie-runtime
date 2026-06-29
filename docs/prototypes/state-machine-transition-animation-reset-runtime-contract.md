# State Machine Transition Animation Reset Runtime Contract

Date: 2026-06-29

This document continues roadmap item `#11` after blend-state transition exit
timing. It defines the next smallest reset surface: C++-style transition
animation reset for the transform `KeyFrameDouble` subset already supported by
`rive-runtime`.

## Formal Goal

Implement transition animation reset parity for simple state-machine transition
mixing in `rive-runtime`: when a non-zero-duration transition starts, record
the live values of transform properties keyed by the outgoing and incoming
`AnimationState` animations, then reapply those values before every transition
mix apply. This prevents repeated mixed animation application from accumulating
on top of the previous frame's already-mixed values.

The goal is complete when the runtime slice can:

- Build a reset record when a non-zero-duration transition starts.
- Collect the union of transform `KeyFrameDouble` properties keyed by the
  outgoing `AnimationState` and incoming `AnimationState`, matching C++'s
  `AnimationResetFactory::fromStates` shape for the supported property subset.
- Ignore outgoing and incoming blend states while building the reset record,
  matching C++ `fromState`, which only admits `AnimationState`.
- Reapply recorded transform values before each transition apply while the
  transition is actively mixing.
- Clear the reset record when transition source state is cleared.
- Compare a multi-animation blend-state transition fixture against the C++
  probe.

## Scope Lock

This slice owns only transition animation reset for transform properties backed
by `KeyFrameDouble`. It does not own C++ `KeyFrameColor` reset, non-transform
properties, full `AnimationResetFactory` pooling, full artboard reset,
component reset registries, data binding reset, bindable-property reset,
nested artboards, nested animation remapping, listener-owned dispatch, hit
testing, rendering, animation keyframe callback events, or non-double
keyframes.

The external seam remains:

- Existing `ArtboardInstance::advance_state_machine_instance(instance,
  seconds)`.
- Existing linear animation and blend-state apply behavior.
- Existing transform mutation APIs.

## Owned By This Slice

This slice owns:

- Runtime transition reset records for keyed transform properties.
- Applying those records before transition source/current state mixing.
- C++ probe coverage for a blend-state transition whose outgoing blend has
  multiple authored blend animations affecting the same transform property.

## Not Owned Yet

These remain future slices:

- `KeyFrameColor` animation reset.
- Non-transform animated properties.
- Reset behavior for view-model/data-binding/runtime property overrides.
- `BlendState1DViewModel` and bindable-property driven blend values.
- `BlendAnimationDirect` with `blendSource=dataBindId`.
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

1. Is it required to stop repeated transition mixing from accumulating for
   currently supported transform `KeyFrameDouble` animations?
2. Does it reuse the existing linear animation metadata and transform mutation
   APIs?
3. Can it be compared against the C++ probe without admitting color,
   non-transform properties, data binding, view-model blends, nested artboards,
   listeners, keyframe callback events, non-double keyframes, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- A private transition reset record storing `(local_id, transform_property,
  value)` entries.
- Reset construction from the outgoing and incoming animation-state linear
  animations.
- Reset application before transition source/current apply.
- A C++ probe-backed fixture with a multi-animation `BlendState1DInput`
  transition into an `AnimationState`, proving repeated transition application
  matches C++.

Suggested verification:

```sh
cargo test -p rive-runtime --test cpp_probe state_machine_blend_state_transition_reset_matches_cpp_probe -- --nocapture
cargo test -p rive-runtime --test cpp_probe
make test
make cpp-compare
```
