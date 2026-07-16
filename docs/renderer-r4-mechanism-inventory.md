# R4 C++ Performance Mechanism Inventory

This is the proactive port checklist for Phase R4. It complements the live
`make perf-counter-compare` report: counters identify excess Rust work, and
this inventory points to the C++ mechanism that avoids it. Timing is a light
directional check unless the proposed benefit is itself timing-defined.

The source snapshot is `/Users/levi/dev/oss/rive-runtime` at
`7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`.

## Counter Baseline

The first counter capture covers the fixed eight scenes in both
clockwise-atomic and MSAA modes. Every variant has one logical flush and both
runners report the same structural draw count. API-level totals are:

| counter | C++ Dawn | Rust wgpu | Rust excess |
| --- | ---: | ---: | ---: |
| command encoders | 16 | 16 | 0 |
| render passes | 91 | 154 | 63 |
| bind groups created | 115 | 133 | 18 |
| bind-group sets | 324 | 554 | 230 |
| texture bindings | 171 | 278 | 107 |
| buffer upload calls | 99 | 63 | -36 |
| buffer upload bytes | 156,832 | 273,640 | 116,808 |
| queue submissions | 16 | 16 | 0 |
| GPU draw calls | 158 | 220 | 62 |
| GPU draw instances | 5,872 | 6,371 | 499 |
| tessellation spans | 1,615 | 1,683 | 68 |
| path patches | 4,237 | 4,666 | 429 |

The single-frame timing sum is Rust/C++=3.224x. It is directional only and has
no acceptance threshold. The deterministic work counts are the discovery
oracle.

The highest row is `gm-bevel180strokes-msaa` bind-group sets at 63 versus 5.
C++ `render_context_webgpu_impl.cpp:4271-4358` creates per-flush bindings once
and uses `needsNewBindings` to rebind only after a restarted pass or an image
draw. Rust currently sets the same flush, dummy-image, and sampler groups for
every direct MSAA path. This is item 119.

### Item 119 Update

Rust now carries direct-path binding state until a pass restart or an
incompatible pipeline layout, and image paths update only the image group.
Aggregate Rust bind-group sets fall from 554 to 413; C++ Dawn remains at 324.
`gm-bevel180strokes-msaa` falls from 63 to six, while the other affected fixed
MSAA variants remove another 84 redundant sets. The light one-frame aggregate
moves from 3.224x to 2.033x and remains directional only.

The refreshed highest-ranked row is
`gm-bevel180strokes-clockwise-atomic` uploaded bytes at 54,696 in Rust versus
8,448 in C++ Dawn. Item 120 must map those bytes to concrete buffers and the
C++ lifetime mechanism before reducing them.

## Port Checklist

| mechanism | C++ source | Counter or symptom | Rust standing |
| --- | --- | --- | --- |
| Triple-buffer GPU upload rings | `renderer/include/rive/renderer/buffer_ring.hpp:11-79`; `renderer/include/rive/renderer/gpu.hpp:75-77`; `renderer/src/webgpu/render_context_webgpu_impl.cpp:2632-2804` | upload calls/bytes, allocation and pending-write work | Core frame upload arena and guarded completed-frame slots are ported; continue comparing byte volume. |
| Dynamic render-buffer rings | `renderer/src/webgpu/render_context_webgpu_impl.cpp:2253-2334` | per-update buffer allocation and copies | Retained Rust render buffers exist; ring/capacity behavior remains a later counter-led check. |
| Logical-flush container reuse | `renderer/src/render_context.cpp:155-157`, `268-273`, `282-343`, `998-1004` | host allocation without changing GPU counters | Rust retains frame containers in several paths; audit only when profiles identify host churn. |
| Resource-budget flush splitting | `renderer/src/render_context.cpp:497-573`, `663-725` | logical flushes and draws per flush | Rust preserves the 1,024-draw and resource fences; fixed corpus is structurally exact. |
| Frame-wide layout, then upload and encode | `renderer/src/render_context.cpp:740-822`, `953-993` | command encoders, submissions, upload calls | Ported to one encoder and one submission per fixed variant; these counters are exact. |
| Retained allocation high-water marks | `renderer/src/render_context.cpp:837-938`, `2562-2910` | allocation churn and upload capacity | Persistent atomic backing and frame upload arenas are ported. The 125% growth and five-second trim policy are not yet copied wholesale. |
| Gradient content deduplication | `renderer/src/render_context.cpp:575-662` | gradient rows, texture work, draw calls | Functional gradient batching exists; no texture uploads occur in the warm fixed matrix. Revisit with a gradient-heavy counter scene. |
| Skyline feather-atlas packing | `renderer/src/render_context.cpp:663-724`, `2205-2290` | atlas passes, patch instances, texture dimensions | Functional atlas batching is ported; retained atlas allocation policy remains counter-led. |
| Draw-batch merge and explicit barrier breaks | `renderer/src/render_context.cpp:3364-3770` | render passes, GPU draw calls, instances | Atomic flush-wide lifetime is ported. Remaining pass/draw excess is open. |
| Bind only on changed state | `renderer/src/webgpu/render_context_webgpu_impl.cpp:4265-4358` | bind-group sets and created groups | Ported for direct MSAA path-compatible layouts in item 119; fixed Rust sets fall 554->413. |
| Lazy pipeline-layout and render-pipeline caches | `renderer/src/webgpu/render_context_webgpu_impl.cpp:451-791`, `1268-1733`, `4440-4463` | frame-time pipeline creation | Rust pipelines are factory-owned. Counter recording begins after warmup and correctly excludes construction. |
| Factory-owned samplers, null resources, and static geometry | `renderer/src/webgpu/render_context_webgpu_impl.cpp:1845-2037` | bind groups created, texture bindings, initialized buffers | Samplers/null resources/static patch geometry are ported for active paths. |
| Retained transient render-target textures | `renderer/src/webgpu/render_context_webgpu_impl.cpp:2051-2179` | per-frame texture creation and clears | Rust retains core MSAA and atomic backing. Audit destination/gradient/atlas textures by counter scene. |
| Lazily retained atomic PLS buffers | `renderer/src/webgpu/render_context_webgpu_impl.cpp:2867-2909` | initialized-buffer and pending-write work | Ported as guarded persistent slots in item 114. |
| Optional gradient, tessellation, and atlas passes | `renderer/src/webgpu/render_context_webgpu_impl.cpp:3981-4137` | render passes and GPU draws | Flush-wide MSAA tessellation is ported. Gradient/atlas cadence remains report-driven. |
| Barrier-aware render-pass restart | `renderer/src/webgpu/render_context_webgpu_impl.cpp:3280-3800`, `4275-4290` | pass count and mandatory rebinding | Atomic group barriers are preserved. Rust should carry binding invalidation with the same restart boundary. |
| Shared normal-frame command encoder/submission | `renderer/src/webgpu/render_context_webgpu_impl.cpp:3854-4593` | command encoders and submissions | Exact in the fixed report. Mipmap generation remains a distinct image path. |

## Counter Interpretation

- `command_encoders` and `queue_submissions` are closed for the fixed matrix.
  Do not spend R4 work there without a new counter regression.
- `bind_group_sets` no longer owns the top row. Item 119 removed 141 repeated
  direct-MSAA sets; retain the remaining layout-bound sets until another
  source-matched redundancy is identified.
- `render_passes` and `gpu_draw_calls` are next, but their remaining rows mix
  mandatory fill-rule subpasses with avoidable lifecycle boundaries. Split
  them by pipeline/pass class before changing scheduling.
- `buffer_upload_bytes`, `tessellation_spans`, and `path_patches` represent
  real data or geometry output. Reduce them only with a C++-matched data-layout
  or algorithm explanation; never optimize the counter by hiding accounting.
- Lower Rust upload-call count is not automatically a win: Rust coalesces into
  fewer writes while currently uploading more bytes. Byte volume is the open
  signal.

Regenerate the ranked artifact with `make perf-counter-compare`. The JSON and
Markdown outputs live under `target/` and are intentionally not checked in.
