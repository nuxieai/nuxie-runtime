# Renderer R4 Profile Attribution

Date: 2026-07-15

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

## Next Port

Port C++ WebGPU's persistent three-buffer upload rings without changing
geometry or GPU pass semantics:

1. Pack each flush's tessellation spans, uniforms, paths, and contours into
   reusable, alignment-correct buffers instead of per-draw
   `create_buffer_init` allocations.
2. Rotate three GPU buffers per resource class and resize only when capacity
   grows, matching `BufferWebGPU` and `gpu::kBufferRingSize`.
3. Upload each used range once with `Queue::write_buffer`; bind per-draw slices
   with exact offsets and sizes while preserving indices, pass order, and
   shader inputs.
4. Keep the 1,024-draw command-buffer fence and logical-flush boundaries until
   a larger measured change proves they can move.
5. Accept only if the twenty-draw trace reduces the remaining 39.760 ms
   pending-write encoder cost, the fixed old-Rust/current-Rust report improves,
   and the 1,468-row corpus remains exact=1,409/diverges=0/gated=59.

Persistent atomic backing-plane reuse remains the next measured resource site
after upload rings.
