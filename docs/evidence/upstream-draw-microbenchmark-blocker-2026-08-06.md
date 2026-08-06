# Upstream draw microbenchmark equivalence resolution — 2026-08-06

The production `LogicalFrame` and `NullLogicalRenderer` seam resolves the
original missing-workload blocker for the ten `Draw*` cases in
[UNIV-1688](https://universe.basis.dev/issue/UNIV-1688). A capability mismatch
still prevents direct ratios, so the inventory classifies the ten cases as
directional timings and links the follow-up
[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727).

## Shared operation boundary

Pinned C++ `tests/bench/draw_pls_path.cpp` measures ten repetitions of:

1. `RenderContext::beginFrame()` for a 1600x1600 target;
2. `RiveRenderer::drawPath()` for every captured path and paint;
3. `RenderContext::flush()` through `RenderContextNULL`.

The C++ null adapter skips only final GPU submission. Production flush still
performs logical layout, retained allocation growth, shadow-buffer mapping,
typed path/paint/contour/gradient/tessellation/triangle/draw-list writes,
rewind, and per-frame teardown. Its pinned `RenderContextNULL` capability
source enables `supportsRasterOrderingMode`, so the default frame selects
`RasterOrdering`.

Rust Criterion constructs `NullFrameWorkload` once outside the measured
closure. Construction captures the same paths, fill rules, authored paint
style, width, join, cap, feather, color, blend mode, and linear/radial
gradients; applies the pinned forced-stroke or forced-feather mutations; and
prepares immutable `LogicalPathHandle`s. Each measured `run()` then performs
exactly ten repetitions of:

1. `NullLogicalRenderer::begin_frame()` with a 1600x1600
   `ClockwiseAtomic` configuration;
2. `draw_path()` or `draw_path_with_gradient()` for every prepared input;
3. `flush()` through the production logical resource writer.

The same Null renderer is retained across Criterion iterations, matching the
retained C++ context and its allocation-growth behavior. Rust's null adapter
is the terminal consumer of the production typed resource output and performs
no WebGPU device, encoder, pipeline, or submission work. It currently runs
with an explicit `ClockwiseAtomic` configuration. Because that differs from
the upstream `RenderContextNULL` `RasterOrdering` selection, the two raw timings
are useful directionally but their quotient is not a valid speed ratio.

## Production seam evidence

- `crates/nuxie-renderer/src/logical_frame.rs` owns draw admission,
  fill-rule-aware logical planning, flush partitioning, retained resource
  allocation, typed CPU buffer writes, consumption accounting, and teardown.
- `WgpuFrame` delegates its logical work to that same `LogicalFrame` and
  consumes the shared typed output in the production GPU encoder.
- `NullLogicalRenderer` exposes the intentionally small
  `begin_frame`/`draw_path`/`flush` interface and consumes the identical typed
  output into retained shadow buffers without GPU submission.
- `crates/nuxie-renderer/tests/paper_riv_logical_differential.rs` proves all
  four pinned `paper.riv` preparation modes preserve authored paint/gradient
  inputs and produce the same logical reports through Wgpu and Null.
- Renderer unit tests cover all six pinned custom `Draw*` workloads through
  both modes and assert Wgpu/Null logical-report parity.
- The feature-gated microbenchmark test proves one public workload call
  completes ten production Null frames with non-empty typed resource writes.

Direct tessellation helpers remain outside this protocol. They do not include
the upstream frame lifecycle, retained allocation policy, typed resource
writes, or teardown and must not be substituted for the production Null seam.

## Evidence contract

`make microbench-gate` verifies the exact 20-case registry, pinned upstream
benchmark and `RenderContextNULL` capability source hashes/ref, the upstream
RasterOrdering capability assignment, fixture conversions, and local Criterion
registrations.
`make microbench-run` refuses dirty or blocked inventories, builds the C++
benchmark into the sealed run directory from the validated clean pinned
checkout's committed archive, and records the committed Rust source revision, benchmark-content
identity, pinned C++ revision, build command/log and binary hash, tool versions,
settings, and every raw sample hash. `make microbench-compare` accepts only that
sealed run manifest and reports ten direct ratios plus ten directional timings.
