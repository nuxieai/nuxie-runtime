# Runtime Weighted Points Path Input Contract

Date: 2026-06-29

This document starts the weighted/deformed path geometry work remaining in
roadmap `#10`. It deliberately captures the imported inputs required for
weighted `PointsPath` deformation before adding skinning math to the runtime
draw-command path.

## Formal Goal

Expose C++ weight payload facts on graph path vertices so a later runtime slice
can reproduce `Skin::deform()` and `Path::buildPath()` for weighted paths.

The slice is complete when:

- `rive-graph::PathVertexNode` records the resolved `Weight`/`CubicWeight`
  object identity already attached during import.
- `PathVertexNode` also records the C++ weight payload words needed by
  `Vertex::deform()` and `CubicVertex::deform()`: `values`, `indices`,
  `inValues`, `inIndices`, `outValues`, and `outIndices`.
- Tests prove those payloads are projected from imported path weights.

## Scope Lock

This slice covers only static imported weight inputs for path vertices.

It does not implement:

- `Weight::deform()` or `CubicVertex::deform()` math in `rive-runtime`;
- `Skin::update()` bone transform buffers;
- `PointsPath::pathTransform()` identity behavior for skinned paths;
- NSlicer or other `RenderPathDeformer` mutation of completed paths;
- weighted mesh rendering, tessellation, renderer allocation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the data a direct imported `Weight` or `CubicWeight` field used by C++
   vertex deformation?
2. Is it static after import, independent of frame update order?
3. Can a later runtime slice consume it without re-reading `rive-binary`
   internals?

If not, defer it to the skin-deformation, NSlicer/deformer, mesh, or renderer
slice that owns the live behavior.

## Next Slice

The next weighted-geometry slice should implement a narrow runtime deformation
context for skinned `PointsPath`:

- find the `SkeletalSkinNode` whose skinnable is the path;
- build C++-ordered skin bone transforms from runtime component world
  transforms and tendon inverse-bind matrices;
- deform weighted straight and cubic path vertices before calling the existing
  `Path::buildPath()` port;
- compare a synthetic weighted `PointsPath` raw command stream against the C++
  probe.

## Verification

```sh
cargo check --workspace
make test
make cpp-compare
```
