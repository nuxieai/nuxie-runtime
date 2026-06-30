# Linear Animation Id Keyframe Runtime Contract

## Scope

This slice covers `KeyFrameId` runtime application for imported linear
animations.

The Rust runtime must:

- Import `KeyFrameId` records alongside the existing double, color, bool, uint,
  and string keyframe records for keyed properties.
- Apply ID keyframes with C++ `KeyFrameId::apply` / `applyInterpolation`
  semantics: the current keyframe value is assigned directly through the uint
  setter path, interpolation does not blend, and animation `mix` does not
  affect the ID value.
- Store animated ID values in the existing runtime uint overlay on
  `ArtboardInstance` so later evaluations read the current runtime value
  instead of the static imported value.
- Let existing component uint transition comparands read the same runtime ID
  value when the referenced component/property has been animated.
- Cover the narrow observable surface with the ID-typed
  `CustomPropertyEnum.propertyValue` field and component enum conditions.

## Out Of Scope

This slice does not implement `KeyFrameCallback` runtime behavior. It also does
not implement callback events, relationship relinking for animated parent/asset
or text-style ID fields, text/style rebuilds, data-binding propagation,
renderer updates, or generalized ID-driven runtime behavior beyond the runtime
uint overlay and existing transition comparands.

## Completion Checks

- A C++ probe-backed state-machine test proves component enum comparands read
  an animated `CustomPropertyEnum.propertyValue` driven by `KeyFrameId`.
- The same test covers that animation `mix` does not blend ID values.
- Existing transform/color/bool/uint/string keyframe, state-machine,
  draw-command, and C++ comparison suites continue to pass.
