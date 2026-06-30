# Linear Animation Color Keyframe Runtime Contract

## Scope

This slice covers `KeyFrameColor` runtime application for imported linear
animations.

The Rust runtime must:

- Import `KeyFrameColor` records alongside the existing `KeyFrameDouble`
  records for keyed properties.
- Apply stepped and linearly interpolated color keyframes using C++
  `KeyFrameColor::apply` / `applyInterpolation` semantics.
- Use C++ `colorLerp` channel rounding for keyframe interpolation and for
  animation `mix` values below `1.0`.
- Store animated color values on the `ArtboardInstance` so later evaluations
  read the current runtime value instead of the static imported value.
- Reflect animated `SolidColor.colorValue` in renderer-independent draw-command
  paint state, including render-color opacity modulation.
- Let existing component color transition comparands read the same runtime color
  value when the referenced component/property has been animated.

## Out Of Scope

This slice does not implement `KeyFrameBool`, `KeyFrameUint`, `KeyFrameId`,
`KeyFrameString`, or `KeyFrameCallback` runtime behavior. It also does not
implement gradient-stop color mutation in gradient payloads, deformer color
updates, renderer paint allocation, animation callback events, or generalized
property storage for every CoreRegistry field kind.

## Completion Checks

- A C++ probe-backed linear-animation test proves interpolated
  `SolidColor.colorValue` draw payloads match C++.
- The same test covers animation `mix` below `1.0` so C++ color blending is
  pinned.
- A C++ probe-backed state-machine test proves component color comparands read
  the animated runtime color value.
- Existing transform keyframe, state-machine, draw-command, and C++ comparison
  suites continue to pass.
