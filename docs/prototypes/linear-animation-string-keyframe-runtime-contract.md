# Linear Animation String Keyframe Runtime Contract

## Scope

This slice covers `KeyFrameString` runtime application for imported linear
animations.

The Rust runtime must:

- Import `KeyFrameString` records alongside the existing double, color, bool,
  and uint keyframe records for keyed properties.
- Apply string keyframes with C++ `KeyFrameString::apply` /
  `applyInterpolation` semantics: the current keyframe value is assigned
  directly, interpolation does not blend, and animation `mix` does not affect
  the string value.
- Store animated string values on `ArtboardInstance` so later evaluations read
  the current runtime value instead of the static imported value.
- Let existing component string transition comparands read the same runtime
  string bytes when the referenced component/property has been animated.
- Cover the narrow observable surface with `SemanticData.label` and component
  string conditions.

## Out Of Scope

This slice does not implement `KeyFrameId` or `KeyFrameCallback` runtime
behavior. It also does not implement callback events, text shaping/layout
rebuilds for animated `TextValueRun.text`, accessibility/semantics propagation
for animated `SemanticData` strings, data-binding propagation, renderer updates,
or generalized string-driven runtime behavior beyond the runtime string overlay
and existing transition comparands.

## Completion Checks

- A C++ probe-backed state-machine test proves component string comparands read
  an animated `SemanticData.label`.
- The same test covers that animation `mix` does not blend string values.
- Existing transform/color/bool/uint keyframe, state-machine, draw-command, and
  C++ comparison suites continue to pass.
