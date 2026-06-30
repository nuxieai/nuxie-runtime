# Linear Animation Callback Keyframe Runtime Contract

## Scope

This slice covers the first observable `KeyFrameCallback` runtime behavior for
state-machine-driven linear animations.

The Rust runtime must:

- Import `KeyFrameCallback` records for keyed callback properties.
- Keep callback keyframes separate from value keyframes because C++ does not
  apply them through `KeyedObject::apply`; it reports crossed callback frames
  through `LinearAnimation::reportKeyedCallbacks`.
- Reproduce the C++ crossing rule for non-zero animation advances: callbacks
  whose keyed frame is crossed between the previous and current animation time
  are reported, with `secondsDelay` equal to `secondsTo - frameSeconds`.
- Thread those reports into the existing `StateMachineInstance` reported-event
  vector when the keyed target is an imported `Event` or `Event` subtype and
  the keyed property is `Event.trigger`.
- Cover the narrow observable surface with an `AnimationState` whose animation
  crosses an `Event.trigger` callback frame and compare event local ID, core
  type, name, and delay against C++.

## Out Of Scope

This slice does not implement plain `LinearAnimationInstance` listener
dispatch, listener-owned event routing, hit testing, pointer/keyboard/gamepad
input dispatch, audio playback, open-url side effects, nested-artboard event
propagation, custom-property trigger callbacks, nested trigger callbacks,
view-model trigger callbacks, dedicated loop and ping-pong callback edge-case
coverage, or callback-driven data-binding behavior.

## Completion Checks

- A C++ probe-backed state-machine test proves an `Event.trigger`
  `KeyFrameCallback` reports the same event and `secondsDelay` as C++ when an
  animation state crosses the callback frame.
- Existing state-machine fire-event and listener fire-event tests keep passing.
- Focused runtime tests, workspace tests, and `make cpp-compare` pass.
