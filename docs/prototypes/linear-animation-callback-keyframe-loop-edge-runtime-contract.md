# Linear Animation Callback Keyframe Loop Edge Runtime Contract

## Scope

This slice extends the callback-keyframe runtime surface with dedicated C++
probe coverage for state-machine animation loop edges.

The Rust runtime must:

- Preserve C++ `LinearAnimationInstance::advance` callback reporting when a
  looping animation wraps forward past its end frame.
- Preserve C++ callback reporting when a ping-pong animation bounces off its
  end frame and reverses direction.
- Keep using the existing `StateMachineInstance` reported-event vector for
  crossed `Event.trigger` callback frames.
- Compare event local ID, core type, name, `secondsDelay`, current animation
  time, and `didLoop` against the C++ probe.

## Out Of Scope

This slice does not add public `LinearAnimationInstance` listener dispatch,
listener-owned routing, audio/open-url side effects, trigger callback targets,
reverse-playback loop edges, work-area loop edges, multi-bounce ping-pong
advances, nested-artboard event propagation, or callback-driven data-binding
behavior.

## Completion Checks

- A C++ probe-backed loop-state-machine fixture crosses callback frames before
  and after a forward loop wrap.
- A C++ probe-backed ping-pong-state-machine fixture crosses callback frames
  before and after the end-frame bounce.
- Focused runtime probes, workspace tests, and `make cpp-compare` pass.
