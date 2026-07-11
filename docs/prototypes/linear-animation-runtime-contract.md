# Linear Animation Runtime Contract

Date: 2026-06-28

This document starts roadmap item `#11` after the first artboard-instancing
slice. It defines the smallest runtime animation surface that can prove
`ArtboardInstance` is a real mutable target without pulling state machines,
data binding, drawing, or full keyframe support into scope.

## Formal Goal

Implement direct linear-animation application in `nuxie-runtime`: build the
smallest runtime-owned animation model needed to apply `KeyFrameDouble` values
for transform properties onto an `ArtboardInstance` at a supplied time and mix,
compare the behavior against the C++ probe, and keep full state-machine
execution, nested animations, event firing, callbacks, data binding,
draw/render behavior, and complete keyframe type coverage out of scope.

The goal is complete when the runtime slice can:

- Build copied linear-animation definitions while constructing an
  `ArtboardInstance`.
- Preserve C++ keyed-object/keyed-property pruning for artboard-local targets.
- Apply `KeyFrameDouble` values to the transform properties already admitted by
  the instancing slice: `x`, `y`, `rotation`, `scaleX`, `scaleY`, and `opacity`.
- Accept direct animation time in seconds plus an explicit mix value.
- Match C++ direct linear-animation application for exact keyframes,
  before-first-frame application, between-keyframe hold/linear behavior,
  after-last-frame application, quantized time, and mixed application.
- Mark transform/render-opacity dirt through the same property setter used by
  non-animation mutations.

## Scope Lock

This slice owns direct `LinearAnimation::apply` parity for a narrow numeric
property set. It does not own animation instance playback, looping,
state-machine integration, events, or renderer output.

The external seam for this slice is:

- Input: a constructed `ArtboardInstance`.
- Operation: `ArtboardInstance::apply_linear_animation(index, seconds, mix)`.
- Output: mutated instance transform state plus normal runtime dirt.

The runtime animation model may copy immutable animation facts out of the
imported file during instance construction. It must not mutate imported source
objects.

## Owned By This Slice

This slice owns:

- Runtime-owned linear-animation, keyed-object, keyed-property, and
  `KeyFrameDouble` records.
- Direct keyframe lookup by seconds.
- Hold behavior when the source keyframe has interpolation type `0`.
- Linear interpolation when the source keyframe requests interpolation and no
  custom interpolator is resolved.
- Mix behavior that matches C++ `KeyFrameDouble::apply`.
- C++ probe support for applying a linear animation to a cloned artboard
  instance before dumping runtime state.

## Not Owned Yet

These remain future slices:

- `LinearAnimationInstance::advance`, looping, work-area playback, and
  `advanceAndApply`.
- State machines, blend states, transitions, input evaluation, listeners, and
  events.
- Callback, bool, uint, color, string, id, and other keyframe types.
- Cubic, elastic, scripted, and other custom keyframe interpolators.
- Interpolator-host overrides.
- Nested animation ownership and nested remap behavior.
- Data binding, converters, observers, dirty queues, and contexts.
- Draw commands, renderer paint allocation, clipping-stack mutation, and GPU
  work.

## Admission Rule

Before adding behavior to this slice, answer:

1. Is it required for C++ `LinearAnimation::apply` parity on transform
   `KeyFrameDouble` properties?
2. Does it exercise the existing mutable `ArtboardInstance` property and dirt
   path?
3. Can it be compared against the C++ probe without admitting state-machine,
   rendering, or data-binding behavior?

If the answer is no to all three, do not add it to this goal.

## First Slice

The first implementation slice should add:

- `ArtboardInstance::linear_animations`.
- `ArtboardInstance::linear_animation(index)` and `linear_animations()`.
- `ArtboardInstance::apply_linear_animation(index, seconds, mix)`.
- Runtime-owned `KeyFrameDouble` keyframe lists for keyed transform properties.
- C++ probe option `--runtime-apply-animation animationIndex seconds mix`.
- Tests for exact, interpolated, held, post-last, quantized, mixed, and
  source-separation behavior.

Suggested verification:

```sh
cargo test -p nuxie-runtime
make test
make cpp-compare
```
