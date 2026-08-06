# Upstream draw microbenchmark equivalence blocker — 2026-08-06

The ten `Draw*` cases in [UNIV-1688](https://universe.basis.dev/issue/UNIV-1688)
do not currently have a direct Rust comparison. The earlier direct-tessellation
prototype did not execute the same operation boundary and its recorded ratios
have been removed.

## Upstream lifecycle

`tests/bench/draw_pls_path.cpp` measures ten repetitions of:

1. `RenderContext::beginFrame()` for a 1600x1600 target.
2. `RiveRenderer::drawPath()` for every captured path and paint.
3. `RenderContext::flush()`.

`RenderContextNULL::flush()` is only the final backend submission no-op. The
production `RenderContext::flush()` still performs all of the following first:

- every `LogicalFlush::layoutResources()` pass;
- frame-wide resource requirement calculation and retained allocation growth;
- mapping the null backend's real shadow-buffer rings;
- every `LogicalFlush::writeResources()` path, paint, contour, gradient,
  tessellation, triangle, and draw-list write;
- buffer unmapping and backend dispatch;
- logical-flush rewind, per-frame allocator teardown, frame descriptor reset,
  and optional container trimming.

The path fill rule is carried through `RiveRenderer::drawPath()` into
`PathDraw::Make()` and affects fill planning and encoded paint data.

The relevant pinned source boundaries are concrete:

- `tests/bench/draw_pls_path.cpp:25-39` owns the ten-frame
  `beginFrame`/`drawPath`/`flush` loop.
- `renderer/src/rive_renderer.cpp:121-187` routes a path and paint through
  `PathDraw::Make`; `renderer/src/draw.cpp:420-615` builds the logical path
  draw, including the fill-rule-dependent path flags.
- `renderer/src/render_context.cpp:760-1032` lays out every logical flush,
  calculates and grows retained resource allocations, maps buffers, calls
  `writeResources`, unmaps, dispatches, rewinds, and tears the frame down.
- `renderer/src/render_context.cpp:1104-1380` and `:1412-2395` implement the
  logical layout and typed resource writes respectively.
- `tests/common/render_context_null.cpp:28-38` maps each `BufferRingNULL` to
  its CPU shadow buffer, while `tests/common/render_context_null.hpp:60` makes
  only the final backend `flush(const FlushDescriptor&)` call a no-op.

## Rust dependency evidence

The Rust renderer has no backend-neutral counterpart to C++ `RenderContext`:

- `WgpuFrame` owns a concrete `Arc<Context>`, and `Context` owns the WebGPU
  device, queue, tessellator, path/atlas/gradient pipelines, and retained GPU
  resources.
- `WgpuFactory::begin_frame()` checks out concrete GPU frame attachments.
- `WgpuFrame::finish_internal()` creates a WebGPU command encoder,
  tessellation upload state, texture leases, and atomic backing before logical
  draw preparation.
- MSAA and atomic logical planning are interleaved with concrete pipeline
  `prepare_*`/`encode` calls, device limits, GPU buffer creation, atlas texture
  creation, submission splitting, and resource lease completion.

These dependencies are visible at the following production symbols:

- `crates/nuxie-renderer/src/lib.rs:300-327` defines `Context` with the
  concrete device, queue, tessellator, pipelines, and retained resources;
  `:2353-2375` stores that `Arc<Context>` directly in `WgpuFrame`.
- `crates/nuxie-renderer/src/lib.rs:1048-1078` has
  `WgpuFactory::begin_frame*` check out a concrete frame attachment lease.
- `crates/nuxie-renderer/src/lib.rs:3221-3304` starts `finish_internal()` by
  cloning GPU attachments, creating the command encoder, and acquiring
  tessellation textures/uploads and atomic backing.
- The same `finish_internal()` then calls concrete pipeline preparation and
  encoding throughout (`path_pipeline.prepare_resources`/`prepare_draw` near
  `:5826-5842`, for example), and its submission closure uses the WebGPU queue
  and device directly near `:3268-3304`.
- `crates/nuxie-renderer/src/logical_flush.rs:71-95` contains only one piece
  of logical accounting. It is not a frame planner and has no buffer-writing
  or teardown interface that a null backend can invoke.

Consequently, a feature-only adapter cannot execute the production logical
flush without also executing WebGPU work. Calling `build_fill_tessellation`,
`build_stroke_tessellation`, or `build_feather_tessellation` directly skips the
resource layout, allocation, buffer-writing, scheduling, and teardown measured
by C++ and is not an equivalent substitute.

## Required production seam

Direct ratios require a production refactor that extracts a backend-neutral
`LogicalFrame` module from `WgpuFrame`. Its small interface must cover
`begin_frame`, `draw_path`, and `flush`; its implementation must own draw
admission, fill-rule-aware logical layout and scheduling, retained resource
allocation policy, typed CPU buffer writes, and frame teardown. WebGPU must use
that same module through a GPU backend adapter, and the benchmark must use it
through a shadow-buffer null adapter.

Until that module exists, the inventory marks all ten `Draw*` cases `blocked`,
the report emits no timings or ratios for them, and `microbench-run` refuses to
create a supposedly complete evidence run. This is intentionally not a
directional comparison.
