# Linear Animation Callback Keyframe Remaining Edge Runtime Contract

## Scope

This slice closes the remaining callback-keyframe edge coverage for
state-machine animation advancement.

The Rust runtime must match C++ reported-event behavior for `Event.trigger`
callback keyframes when:

- an animation plays in reverse from its end time;
- a looping animation uses an enabled work area and wraps inside that work
  range;
- a ping-pong animation advances far enough to bounce more than once in a
  single frame.

The comparison must continue to cover event local ID, core type, name,
`secondsDelay`, current animation time, and `didLoop`.

## Out Of Scope

This slice does not add public scene/listener dispatch, audio playback,
open-url side effects, trigger callback targets, nested-artboard event
propagation, data-binding behavior, or callback behavior outside the existing
state-machine reported-event surface.

## Completion Checks

- C++ probe-backed fixtures cover reverse playback, work-area loop wrapping,
  and multi-bounce ping-pong callback reporting.
- The remaining-work audit no longer lists callback edge coverage as open.
- Focused runtime probes, workspace tests, and `make cpp-compare` pass.
