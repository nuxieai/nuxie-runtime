# Linear Animation Instance Callback Report Runtime Contract

## Scope

This slice pins the public Rust `LinearAnimationInstance` callback-report seam
against C++ for plain linear-animation advancement.

Rust already reports crossed `Event.trigger` callback keyframes when
`ArtboardInstance::advance_linear_animation_instance_with_events` is used. This
slice verifies that behavior against C++ `LinearAnimationInstance::advance`
with a `KeyedCallbackReporter`, using the same observable event payload shape
as the state-machine callback tests: event local ID, event core type, event
name, and seconds delay.

## Runtime Rule

For a non-zero plain animation-instance advance, Rust must report every crossed
`Event.trigger` callback keyframe into the provided event vector with C++
`secondsTo - frameSeconds` delay semantics, while preserving the existing
animation timing, loop, work-area, spill, and keep-going behavior.

## Out Of Scope

This slice does not implement `Scene::advanceAndApply` listener notification,
audio playback, open-url side effects, nested-artboard event propagation,
callback targets other than `Event.trigger`, listener-owned routing, hit
testing, pointer/keyboard/gamepad dispatch, or callback-driven data binding.

## Completion Checks

- The C++ probe exposes plain animation-instance reported callback payloads
  through `--runtime-advance-animation`.
- A C++ probe-backed Rust test verifies
  `advance_linear_animation_instance_with_events` reports the same
  `Event.trigger` payload as C++.
- Existing state-machine callback-keyframe probes keep passing.
- Focused runtime probes, workspace tests, and `make cpp-compare` pass.
