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
buffers immediately before its render pass. The first profile proved that a
`PendingWrites` command buffer precedes almost every tessellation pass, but it
did not by itself distinguish buffer upload from first-use texture work.

A controlled follow-up made that distinction. Merely preparing all buffers
before encoding left 19.99 pending-write submissions/frame. Packing every span
into one buffer and sharing one uniform/path/contour bind group still left
19.85/frame, with each event immediately preceding the next distinct
tessellation texture's first pass. The packed-buffer candidate also changed
the 20-draw median from 69.644 ms to 70.463 ms. The only per-draw resource left
unchanged by both candidates was the newly allocated output texture, so the
trace supports first-use texture initialization as the submission boundary.
Both candidates were removed.

Authoritative source sites:

- C++ flush-wide mapping: `renderer/src/render_context.cpp`,
  `RenderContext::mapResourceBuffers` and `unmapResourceBuffers`.
- C++ WebGPU rings: `renderer/src/webgpu/render_context_webgpu_impl.cpp`,
  `BufferWebGPU` and `make*BufferRing`.
- C++ one-encoder Dawn bridge:
  `crates/nuxie-renderer-ffi/cpp/rive_renderer_ffi_dawn.cpp`,
  `beforeFlush` and `afterFlush`.
- Rust interleaving: `crates/nuxie-renderer/src/tessellator.rs`,
  `Tessellator::encode`.

## Next Port

First adapt C++'s persistent resource lifetime without changing geometry or
GPU pass semantics:

1. Check out size-compatible tessellation textures from a bounded pool.
2. Hold checked-out textures until the frame's GPU-completion fence passes,
   then return them for a later frame.
3. Preserve one tessellation texture per draw, existing indices, pass order,
   and every shader input. Do not revive vertical packing or pass merging.
4. Cover concurrent/in-flight frames and bounded retention explicitly.
5. Accept only if the steady-state twenty-draw trace collapses pending-write
   submissions from approximately twenty per frame to a small constant, the
   fixed old-Rust/current-Rust A/B improves, and the 1,468-row corpus remains
   exact=1,409/diverges=0/gated=59.

Persistent buffer rings and atomic backing-plane reuse remain subsequent
measured work if texture reuse does not close enough of the gap.
