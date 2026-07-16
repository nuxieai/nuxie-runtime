# Renderer R4 Profile Attribution

Date: 2026-07-15; updated 2026-07-16

## Question

The fixed renderer report showed that clockwise-atomic Rust time scaled much
faster than C++ time as draw count increased, but two structural optimizations
made Rust slower. This audit separates resource creation/upload, command
encoding, submission, GPU execution, and completion wait before another code
change.

The paired controls are the existing fenced release runners on the same Apple
M5 Max Metal adapter:

- `gm-bug5099-clockwise-atomic`: one authored draw.
- `gm-bevel180strokes-clockwise-atomic`: twenty authored draws.
- 10 warmup frames and 100 measured frames per runner.
- submit-to-GPU-complete timing with completion after every frame.

CPU attribution uses the Xcode Time Profiler template. GPU attribution uses
the Metal System Trace template. Both launch the exact release runner and
request used by `renderer-perf`; no profiling-only renderer path is present.

## Direct Timing

| runner | draws | median frame | Rust/C++ |
| --- | ---: | ---: | ---: |
| C++ Dawn | 1 | 0.260 ms | 1.00x |
| Rust wgpu | 1 | 3.167 ms | 12.16x |
| C++ Dawn | 20 | 0.942 ms | 1.00x |
| Rust wgpu | 20 | 63.695 ms | 67.60x |

The gap is therefore draw-scaled, not a constant adapter or runner cost.

## CPU Attribution

The Rust twenty-draw profile contains 3,526 samples inside
`RustBackend::render`. Of those, 2,798 samples (79.4%) include
`DeviceExt::create_buffer_init`; measured against `WgpuFrame::finish_internal`
alone, the share is 85.9%. The leaf work is dominated by `__bzero` and
`_platform_memmove`. Render-pass command encoding is about 101 samples and
`Device::poll` is 88 samples.

The one-draw Rust control has 113 `create_buffer_init` samples inside 263
renderer samples. The corresponding C++ profiles contain no comparable
per-draw upload stack. Their active frame samples remain under
`RenderContext::flush` and the single Dawn command-buffer path.

Metal's application submission intervals make the CPU result deterministic:

| runner | draws | pending-write submissions/frame | median pending-write encoder | pending-write encoder/frame |
| --- | ---: | ---: | ---: | ---: |
| C++ Dawn | 1 | 0 | n/a | 0 ms |
| Rust wgpu | 1 | 1.00 | 2.808 ms | 3.736 ms |
| C++ Dawn | 20 | 0 | n/a | 0 ms |
| Rust wgpu | 20 | 19.98 | 2.715 ms | 59.722 ms |

For the Rust twenty-draw case, all other sampled encoder intervals together
are about 2.65 ms/frame: tessellation 0.97 ms, atomic path 0.64 ms, resolve
0.47 ms, transit 0.43 ms, signal 0.10 ms, and frame clear 0.04 ms. C++ encodes
its complete twenty-draw command buffer in about 0.55 ms/frame.

## GPU Attribution

The union of target-process GPU intervals avoids double-counting concurrent
vertex, fragment, and compute channels:

| runner | draws | Metal command buffers/frame | GPU union/frame |
| --- | ---: | ---: | ---: |
| C++ Dawn | 1 | 1 | 0.093 ms |
| Rust wgpu | 1 | 6 | 0.318 ms |
| C++ Dawn | 20 | 1 | 0.646 ms |
| Rust wgpu | 20 | 101 | 6.474 ms |

GPU execution is a real secondary gap, but it cannot explain a 63.695 ms Rust
frame. The 59.722 ms pending-write encoder total nearly saturates the measured
frame by itself. Completion waiting is not the long pole: GPU union is about
6.47 ms/frame, and active `Device::poll` work is a small fraction of the CPU
profile while uploads are serialized ahead of each tessellation pass.

## Architectural Cause

C++ `RenderContext::mapResourceBuffers` maps every flush-wide uniform, path,
paint, contour, tessellation-span, and triangle range before encoding begins;
`unmapResourceBuffers` submits each range once. WebGPU implements those ranges
with persistent triple-buffer rings and `Queue::WriteBuffer`. Its Dawn bridge
creates one command encoder before `RenderContext::flush` and submits one
command buffer afterward. Atomic PLS backing buffers are persistent and only
resized when dimensions change.

Rust creates one tessellation output texture per draw and recreates those
textures every frame. Each `Tessellator::encode` also creates four initialized
buffers immediately before its render pass. `wgpu::util::DeviceExt` implements
`create_buffer_init` with a mapped-at-creation buffer. In wgpu-core 30,
unmapping that buffer records its staging copy in `Queue::pending_writes`, and
the next `Queue::submit` flushes those copies as the internal `PendingWrites`
command buffer.

The missing causal edge was Rust's submission policy. Generic atomic work
called `submit_and_wait` after every intersection-board group, so each group
forced the accumulated initialized-buffer copies to flush before its first
tessellation pass. Preparing uploads earlier and packing them into four shared
buffers did not change that policy; this is why both item-110 candidates
retained approximately twenty pending-write command buffers despite radically
different buffer counts.

A bounded exact-size tessellation-texture pool then falsified the texture
hypothesis directly. The pool retained at most 256 textures/64 MiB and recycled
only after GPU completion, but the steady-state trace still measured 19.88
pending-write command buffers and 53.996 ms of pending-write encoder time per
frame. It also regressed `bug5099` from 4.010 ms to 4.301 ms and
`bevel180strokes` from 71.605 ms to 78.398 ms. The pool was removed.

The accepted follow-up coalesces independent atomic groups into one command
encoder until their combined authored-draw count reaches the existing 1,024-
draw Metal safety fence. It still submits at logical-flush boundaries and
preserves a single oversized group intact. Across 110 traced frames this
reduced `PendingWrites` from 19.88 to exactly 1.00/frame while preserving all
twenty tessellation and atomic passes. Pending-write encoder time fell to
39.760 ms/frame. The fixed 16-variant old-Rust/current-Rust report improved
from 162.237 ms to 138.841 ms in aggregate (0.8558x); every affected clockwise
scene was flat or faster and MSAA stayed within measurement noise. The full
pixel and V2 regression floors remained green.

The next accepted slice ports C++'s three-buffer upload lifetime while adapting
the transfer path to wgpu. Three guarded slots each retain one union-usage
arena for tessellation spans, uniforms, paths, and contours. Per-draw bindings
use exact aligned slices; overflow pages consolidate on the next submission;
each populated page is uploaded once with `Queue::write_buffer` before submit.

The first two fixed 16-variant alternating reports improved aggregate time to
0.9605x and 0.9826x; the second made every variant faster. A third report
against the exact final binary improved aggregate time to 0.9797x. Its only
minimum outlier, `bug339297_as_clip` clockwise-atomic at 1.176x, reversed in a
targeted A-B-B-A: candidate medians were 3.069 and 3.012 ms versus bracket
baselines of 4.152 and 3.117 ms. The initial standalone Metal comparison
appeared to worsen pending-write encoder time from 59.107 to 74.267 ms/frame,
but the captures were fourteen minutes apart and machine load was not
controlled. A subsequent full-request A-B-B-A campaign brackets both candidate
traces with baselines:

| order | runner | pending-write encoder/frame | median frame |
| ---: | --- | ---: | ---: |
| 1 | baseline | 29.159 ms | 38.404 ms |
| 2 | upload arena | 29.309 ms | 38.258 ms |
| 3 | upload arena | 46.051 ms | 55.362 ms |
| 4 | baseline | 45.718 ms | 56.998 ms |

The paired pending-write result is neutral at approximately 1.006x, while
candidate frame time improves in both brackets. This falsifies the claimed
1.2565x regression and shows the arena's gain is CPU/resource setup outside
the Metal encoder interval. The full renderer corpus, both V2 floors, and the
workspace suite remain green.

## Remaining Pending-Write Attribution

The next audit first rejected an overloaded measurement window rather than
interpreting it. The host snapshot showed `fseventsd` and concurrent Codex and
build workers consuming cores, so only deterministic upload byte counts were
retained from that window. Accepted timing and trace evidence records host load
beside every baseline and candidate capture.

Feature-gated upload telemetry measures the warm three-slot tessellation arena:

| control | upload calls | populated pages | payload | written | page capacity |
| --- | ---: | ---: | ---: | ---: | ---: |
| one draw | 4 | 1 | 784 B | 1,040 B | 5,120 B |
| twenty draws | 80 | 1 | 15,680 B | 20,496 B | 25,480 B |

The corresponding baseline Time Profiler trace contains 4,030 samples inside
the Rust per-frame render path. `create_buffer_init` appears in 3,438 (85.3%),
zero/copy work appears in 2,323 (57.6%), and `Queue::write_buffer` appears in
only two (0.05%). The remaining pending-write cost therefore cannot be
explained by a 20 KiB tessellation upload. Source inspection found the larger
cause: generic atomic recreated zero-initialized color, clip, and coverage
buffers for every frame. At 1,024 square, clip and coverage alone account for
roughly 8 MiB of allocation and zeroing.

C++ WebGPU retains atomic PLS backing and releases it only on resize. The Rust
port now follows that lifetime with three guarded slots, growing each color,
clip, and coverage buffer only when necessary. Every atomic batch records
ordered `clear_buffer` commands for its exact active ranges, preserving the
old zero-initialized semantics and plane capture sizes while avoiding CPU-side
zero vectors and mapped-at-creation staging.

Two independent fixed alternating reports reproduce the gain:

| report | aggregate candidate/baseline | worst untouched MSAA control |
| --- | ---: | ---: |
| first | 0.291290x | 1.035515x |
| replicate | 0.290764x | 1.022341x |

A direct candidate-baseline-baseline-candidate bracket for the twenty-draw
control measured candidate medians of 4.174 and 4.293 ms against baseline
medians of 36.757 and 34.561 ms with 71%-76% host idle. The independent Metal
A-B-B-A sequence was likewise load-matched:

| order | runner | host idle | pending-write encoder/frame |
| ---: | --- | ---: | ---: |
| 1 | baseline | 79.57% | 27.532 ms |
| 2 | persistent backing | 81.34% | 2.899 ms |
| 3 | persistent backing | 81.60% | 2.897 ms |
| 4 | baseline | 75.14% | 28.067 ms |

The candidate Time Profiler trace then falls to 185 per-frame samples:
initialized-buffer creation drops to 27, zero/copy drops to nine, and
`Queue::write_buffer` disappears from sampled stacks. Command encoding is now
the largest CPU category at 100 samples (54.1%). The renderer corpus remains
exact=1,409/diverges=0/gated=59, V2 exact-segment floors remain 584 and 35, and
the full workspace suite passes.

## Measurement Fence

R4 performance decisions use these controls:

1. Compare immutable release binaries with the fixed 16-variant alternating
   report; repeat the report before accepting or rejecting a disputed result.
2. Treat end-to-end submit-to-GPU-complete frame time as the primary metric.
   Trace intervals are diagnostic and may veto only a reproducible material
   regression, not a one-off absolute-duration change.
3. Record Metal comparisons in A-B-B-A order with the full fenced request so
   both candidate captures are bracketed by baselines.
4. Record system load and defer a capture during known build, deletion, or
   indexing spikes. Absolute trace values from different load windows are not
   directly comparable.
5. Require unchanged structural counters, the renderer pixel ratchet, both V2
   segment floors, and the full workspace suite for every accepted slice.

Authoritative source sites:

- C++ flush-wide mapping: `renderer/src/render_context.cpp`,
  `RenderContext::mapResourceBuffers` and `unmapResourceBuffers`.
- C++ WebGPU rings: `renderer/src/webgpu/render_context_webgpu_impl.cpp`,
  `BufferWebGPU` and `make*BufferRing`.
- C++ one-encoder Dawn bridge:
  `crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi_dawn.cpp`,
  `beforeFlush` and `afterFlush`.
- Rust upload creation: `crates/nuxie-renderer/src/tessellator.rs`,
  `Tessellator::encode`.
- Rust submission cadence: `crates/nuxie-renderer/src/lib.rs`,
  `WgpuFrame::finish_internal`.

## Next Measurement

Attribute the newly dominant generic-atomic command-encoding work before
changing another resource lifetime:

1. Separate per-batch dummy image texture/sampler creation, bind-group
   construction, and render-pass encoding in the fixed one- and twenty-draw
   controls.
2. Reuse or batch only the resource class that remains material in a fresh
   profile; preserve image-binding semantics and capture/readback behavior.
3. Compare against the item-114 release binary with the fixed alternating
   report and load-recorded A-B-B-A Metal captures.
4. Keep the 1,024-draw command-buffer fence and logical-flush boundaries until
   a measured change proves they can move.
5. Accept the next implementation only on repeated end-to-end improvement,
   bracketed trace non-regression, and unchanged pixel/V2/workspace floors.
