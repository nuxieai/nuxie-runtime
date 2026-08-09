# RasterOrdering logical oracle — 2026-08-09

[UNIV-1727](https://universe.basis.dev/issue/UNIV-1727) adds a production
`RenderMode::RasterOrdering` path to the retained `LogicalFrame` and
`NullLogicalRenderer`. It is a backend-neutral logical mode; `WgpuFactory`
rejects it explicitly because WebGPU does not expose the required raster-order
interlock.

## Pinned C++ oracle

The comparison gate pins rive-runtime commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, its
`tests/bench/draw_pls_path.cpp` workload registry, and the
`tests/common/render_context_null.cpp` capability implementation. At that
revision, `RenderContextNULL` selects RasterOrdering and exercises:

- pixel-local-storage coverage for non-atlased paths and the pinned feather
  atlas selection threshold for large feathers;
- authored draw order with no overlap reordering;
- no explicit interlock barriers;
- three transient PLS planes: clip, scratch color, and coverage;
- one pass for midpoint fills, strokes, and feathers, and two passes for
  interior triangulation;
- retained logical resource allocation, typed buffer writes, rewind, and
  per-frame state teardown.

The Rust mode carries those decisions through the same production
`LogicalFrame` resource plan and typed writer used by MSAA and
ClockwiseAtomic. It adds no benchmark-only execution switch.

## Exact workload coverage

The six constructed workloads (`DrawZeroChopStrokes`, `DrawOneChopStrokes`,
`DrawTwoChopStrokes`, `DrawOneCuspStrokes`, `DrawTwoCuspStrokes`, and
`DrawCustomFeathers`) each execute their exact 1,000 authored draws through the
RasterOrdering Null frame. Tests assert the explicit PLS plan, typed resource
writes, zero fallbacks, and rewind contract.

The pinned `paper.riv` fixture contains 3,861 authored paths. Its authored,
bevel-stroke, round-join-stroke, and feathered preparations cover the other
four registered Draw workloads and assert the same RasterOrdering contract.
The existing Wgpu/Null differential remains unchanged for MSAA and
ClockwiseAtomic.

## Qualification

The deterministic inventory gate recognizes all 20 registered benchmarks as
direct ratios and rejects directional classifications. The committed evidence
run records timing settings, source identities, raw-output hashes, and the
resulting 20-row comparison separately from this semantic oracle.

The candidate head passed:

- six focused RasterOrdering renderer tests;
- the pinned 3,861-path paper differential (one test, 406.05 seconds);
- the feature-gated ten-frame production Null workload test;
- native `nuxie-renderer` all-features compilation;
- default-feature `wasm32-unknown-unknown` compilation; and
- all 27 microbenchmark contract tests plus the pinned upstream provenance
  gate (20 cases, three datasets).
