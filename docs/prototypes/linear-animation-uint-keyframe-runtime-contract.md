# Linear Animation Uint Keyframe Runtime Contract

## Scope

This slice covers `KeyFrameUint` runtime application for imported linear
animations.

The Rust runtime must:

- Import `KeyFrameUint` records alongside the existing double, color, and bool
  keyframe records for keyed properties.
- Apply uint keyframes with C++ `KeyFrameUint::apply` /
  `applyInterpolation` semantics: the current keyframe value is assigned
  directly, interpolation does not blend, and animation `mix` does not affect
  the uint value.
- Store animated uint values on `ArtboardInstance` so later evaluations read
  the current runtime value instead of the static imported value.
- Let existing component uint transition comparands read the same runtime uint
  value when the referenced component/property has been animated.
- Cover the narrow observable surface with `CustomPropertyEnum.propertyValue`
  and component enum conditions.

## Out Of Scope

This slice does not implement `KeyFrameId`, `KeyFrameString`, or
`KeyFrameCallback` runtime behavior. It also does not implement callback
events, relationship relinking for animated ID-like uint fields, layout solving
for animated layout-style uint fields, text/style rebuilds for animated text
uint fields, renderer paint allocation, or generalized property mutation beyond
the runtime uint overlay and existing transition comparands.

## Completion Checks

- A C++ probe-backed state-machine test proves component enum comparands read
  an animated `CustomPropertyEnum.propertyValue`.
- The same test covers that animation `mix` does not blend uint values.
- Existing transform/color/bool keyframe, state-machine, draw-command, and C++
  comparison suites continue to pass.
