# Runtime Weighted Points Path Command Contract

Date: 2026-06-29

This document closes the narrow weighted `PointsPath` command-generation slice
from roadmap `#10`.

## Formal Goal

Reproduce C++ import-time/runtime deformation for skinned weighted
`PointsPath` objects when emitting the renderer-independent runtime path command
stream.

The slice is complete when:

- `rive-graph::SkeletalSkinNode` exposes the authored C++ `Skin` matrix in
  `Mat2D` slot order: `[xx, xy, yx, yy, tx, ty]`.
- `rive-graph::SkeletalTendonNode` exposes the C++ `Tendon::inverseBind()`
  matrix in the same slot order, including identity fallback for singular bind
  matrices.
- `rive-runtime` finds the skin whose skinnable is the `PointsPath`, builds a
  bone-transform buffer matching C++ order with identity at slot `0`, then one
  `bone.worldTransform * tendon.inverseBind` entry per tendon.
- Weighted straight and cubic path vertices are deformed before the existing
  `Path::buildPath()` command builder runs.
- Skinned `PointsPath` command emission uses identity `pathTransform()`, matching
  C++ `PointsPath::pathTransform()`.
- A synthetic C++ probe comparison covers a weighted straight vertex, weighted
  cubic vertex handles, skin matrix, bone world transform, and tendon inverse
  bind matrix.

## Scope Lock

This slice covers only command-stream geometry for skinned `PointsPath`
vertices whose authored weights resolve to valid skin bone slots.

It does not implement:

- weighted mesh deformation or tessellation;
- IK solving, constraint solving, or full transform dirt scheduling beyond the
  existing runtime component update pass;
- NSlicer or other `RenderPathDeformer` mutation of completed paths;
- feather renderer/inner-path behavior;
- renderer allocation, GPU work, or draw batching.

## Admission Rule

Before extending this code, ask:

1. Is this needed to turn a skinned weighted `PointsPath` into the same raw path
   command stream C++ emits?
2. Can it be verified by a headless C++ probe comparison?
3. Does it avoid pulling renderer, mesh, constraint, or deformer execution into
   `rive-runtime` prematurely?

If not, it belongs in a later deformer, mesh, renderer, or scheduler slice.

## Verification

```sh
cargo test -p rive-graph graph_projects_skeletal_registration_facts --test cpp_probe
cargo test -p rive-runtime runtime_draw_command_stream_deforms_weighted_points_path_payloads_like_cpp_probe --test cpp_probe
cargo check --workspace
make test
make cpp-compare
```
