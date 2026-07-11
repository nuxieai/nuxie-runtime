# Linear Animation Instance Runtime Contract

Date: 2026-06-28

This document continues roadmap item `#11` after direct
`LinearAnimation::apply` parity. It defines the smallest mutable animation
instance state needed before state-machine animation states can be modeled.

## Formal Goal

Implement a narrow `LinearAnimationInstance` runtime seam in `nuxie-runtime`: add
a mutable animation-instance state object that wraps the existing direct
linear-animation apply path, supports C++-parity time advancement for transform
`KeyFrameDouble` animations, compares against the C++ probe, and keeps state
machines, callbacks, events, nested animations, data binding, custom
interpolators, rendering, and non-double keyframes out of scope.

The goal is complete when the runtime slice can:

- Create a `LinearAnimationInstance` for an `ArtboardInstance` animation index.
- Initialize instance time from C++ `LinearAnimation::startTime()` semantics.
- Advance instance time using C++ `LinearAnimationInstance::advance` semantics
  for one-shot, loop, and ping-pong animations.
- Respect animation speed and work-area start/end bounds.
- Track `did_loop`, `spilled_time`, `total_time`, `last_total_time`, direction,
  and `keep_going` return values.
- Apply the animation instance through the existing transform-only direct apply
  path.
- Compare instance state and resulting component state against the C++ probe.

## Scope Lock

This slice owns mutable playback state for already-imported linear animations.
It does not own artboard frame advancement, callback reporting, state-machine
execution, nested animation ownership, data-binding advancement, or rendering.

The external seam for this slice is:

- `ArtboardInstance::linear_animation_instance(index)`.
- `ArtboardInstance::linear_animation_instance_with_speed(index, multiplier)`.
- `ArtboardInstance::advance_linear_animation_instance(instance, seconds)`.
- `ArtboardInstance::apply_linear_animation_instance(instance, mix)`.

## Owned By This Slice

This slice owns:

- Linear animation instance time state.
- Loop mode handling for `oneShot`, `loop`, and `pingPong`.
- Work-area start/end bounds.
- Speed and direction effects on advancement.
- Spilled-time and keep-going state.
- C++ probe reporting for animation-instance advancement.

## Not Owned Yet

These remain future slices:

- `advanceAndApply` calling `ArtboardInstance::advance`.
- Keyed callback reporting and event collection.
- State-machine states, layers, transitions, listeners, and inputs.
- Nested animation instance ownership and remapping.
- Data-binding contexts, observers, queues, converters, and advancement.
- Non-double keyframe types.
- Cubic, elastic, scripted, and other custom interpolators.
- Interpolator-host overrides.
- Draw commands, clipping-stack mutation, renderer paint allocation, and GPU
  work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for C++ `LinearAnimationInstance::advance` state parity?
2. Does it reuse the existing direct transform `KeyFrameDouble` apply path?
3. Can it be compared against the C++ probe without admitting callbacks,
   state machines, nested animations, data binding, or rendering?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- `LinearAnimationInstance` state in `nuxie-runtime`.
- Loop/work-area/speed advancement helpers on `RuntimeLinearAnimation`.
- C++ probe option `--runtime-advance-animation animationIndex seconds mix`.
- C++ probe JSON reports for time, direction, loop, spill, totals, and
  keep-going.
- Tests for one-shot clamping, loop wrapping, ping-pong reflection, work-area
  starts, negative speed starts, and post-advance application.

Suggested verification:

```sh
cargo test -p nuxie-runtime
make test
make cpp-compare
```
