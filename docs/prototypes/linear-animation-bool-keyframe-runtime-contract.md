# Linear Animation Bool Keyframe Runtime Contract

## Scope

This slice covers `KeyFrameBool` runtime application for imported linear
animations.

The Rust runtime must:

- Import `KeyFrameBool` records alongside the existing double and color
  keyframe records for keyed properties.
- Apply bool keyframes with C++ `KeyFrameBool::apply` /
  `applyInterpolation` semantics: the current keyframe value is assigned
  directly, interpolation does not blend, and animation `mix` does not affect
  the bool value.
- Store animated bool values on `ArtboardInstance` so later evaluations read
  the current runtime value instead of the static imported value.
- Reflect animated `Fill.isVisible` / inherited `ShapePaint.isVisible` in the
  renderer-independent draw-command paint filtering path.
- Let existing component bool transition comparands read the same runtime bool
  value when the referenced component/property has been animated.

## Out Of Scope

This slice does not implement `KeyFrameUint`, `KeyFrameId`, `KeyFrameString`,
or `KeyFrameCallback` runtime behavior. It also does not implement callback
events, generalized property storage for every CoreRegistry field kind, stroke
visibility reveal when static stroke thickness filtering made the paint
invisible, or bool-driven behavior outside draw-command filtering and transition
comparands.

## Completion Checks

- A C++ probe-backed linear-animation test proves an animated fill visibility
  bool filters draw paint payloads like C++.
- The same test covers that animation `mix` does not blend bool values.
- A C++ probe-backed state-machine test proves component bool comparands read
  the animated runtime bool value.
- Existing transform/color keyframe, state-machine, draw-command, and C++
  comparison suites continue to pass.
