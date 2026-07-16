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

## Generic-Atomic Flush Lifetime

Feature-gated timers split `AtomicPipeline::encode_batch` into resource and
encoding categories. On the twenty-draw `bevel180strokes` control, dummy
texture creation, sampler creation, and the sampler bind group together cost
about 66 microseconds inside a 5.23 ms frame. Buffer upload and all directly
timed setup total about 460 microseconds. Persisting only the named dummy and
sampler objects therefore cannot explain the dominant command-encoding work.

The Metal traces expose the structural cause. C++ WebGPU emits 2,644 encoder
rows over 110 frames, or 24 encoders per steady frame plus four startup rows.
The item-114 Rust binary emits 11,221 rows, or 102 per frame plus one startup
row. C++ `LogicalFlush::buildDrawList` sorts draws into intersection-board
groups, then `AtomicDrawRenderPass` initializes PLS once, restarts the path
pass at each `plsAtomic` barrier, and resolves once. Rust preserved the same
groups but called the complete initialize/path/resolve lifecycle for every
group.

The accepted port flattens those already validated groups into one preparation
unit per submission. Explicit group starts preserve every barrier, while the
atomic backing clear and final resolve happen once. The existing 1,024-draw
submission fence and every logical-flush boundary remain unchanged. On the
twenty-draw control this changes 20 atomic preparations and 40 atomic render
passes into one preparation, 20 group passes, and one resolve. The full corpus,
including all scenes that failed an unsafe no-barrier experiment, remains
exact=1,409/diverges=0/gated=59.

Two fixed old-Rust/current-Rust reports reproduce the end-to-end gain at
0.797783x and 0.816390x aggregate. Untouched MSAA controls remain within 3.8%
and 1.7%, respectively. A direct twenty-draw A-B-B-A records baseline medians
of 5.881 and 5.156 ms around candidate medians of 2.369 and 2.352 ms, with host
idle between 87.3% and 89.8%.

The independent Metal A-B-B-A is likewise load matched:

| order | runner | host idle | encoder rows | encoder time/frame | median frame |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | item-114 baseline | 86.15% | 11,221 | 4.335 ms | 7.740 ms |
| 2 | flush-wide lifetime | 87.80% | 4,951 | 2.706 ms | 3.939 ms |
| 3 | flush-wide lifetime | 89.28% | 4,951 | 2.050 ms | 3.300 ms |
| 4 | item-114 baseline | 89.31% | 11,221 | 4.502 ms | 7.822 ms |

The current C++/Rust fixed report is now 5.053695x aggregate. Clockwise-atomic
variants range from 2.05x to 4.60x, while MSAA ranges from 5.97x to 11.99x and
owns the worst scene. The candidate trace still has 20 tessellation encoders
and 20 atomic path encoders per frame versus C++'s 24 total encoders, so R4 is
not complete.

## MSAA Path Resource Lifetime

Item 116 reprofiled the exact item-115 binaries before changing another
lifetime. The C++ runner SHA-256 was
`5a550bf5cc4c3d3a8306b7bf68c63f8d220d3ae31b34ea1cfea69fe63359e1b1`;
the Rust runner was
`b16b8400f4f4f76e2f24b74450cd300f381f59bfd96fc0ee75051af6316715a1`.
Paired one- and twenty-draw Time Profiler and Metal System Trace captures in
both modes separate the remaining atomic pass cadence from the larger MSAA
host setup cost.

The twenty-draw MSAA Time Profiler capture contains 119 samples in
`RustBackend::render`. `PathPipeline::prepare` owns 35, including 24 in its
five per-draw `create_buffer_init` calls, six in per-draw null-texture
creation, three in sampler creation, and two in bind-group creation. The
corresponding one-draw profile has only 11 renderer samples. The generic-
atomic twenty-draw control instead spends 76 of 130 samples in render-pass
encoding and only three in `create_buffer_init`, so the MSAA resource site is
mode-specific rather than an extrapolation from encoder counts.

Metal shows the same structural slope. Rust twenty-draw MSAA emits 2,200
tessellation encoders and 2,200 internal texture-clear encoders across 110
frames, plus one pending-write, frame-clear, solid, and composite encoder per
frame. C++ emits one flush-wide tessellation pass and one MSAA draw pass per
steady frame. The internal clear passes belong to the twenty newly created
null textures, not the tessellation attachments. Encoder lifetime is used only
as diagnostic attribution here; the load-bracketed end-to-end runner median
remains the acceptance metric.

C++ `RenderContext::mapResourceBuffers` maps uniform, path, paint, paint-aux,
contour, and tessellation-span rings for the flush. Its WebGPU context also
owns one null texture, all image sampler permutations, and one sampler bind
group. Rust now places the MSAA path resources in exact aligned slices of the
existing guarded frame upload arena, writes each populated page once before
submit, and retains the null texture and sampler bindings on `PathPipeline`.
Tessellation textures and passes remain per draw; item 108's rejected packing
implementation was not restored.

Two fixed 16-variant old-Rust/current-Rust reports improve to 0.908863x and
0.912357x aggregate. Every MSAA variant improves in both reports;
`bevel180strokes-msaa` falls to 0.726788x and 0.724408x. A direct bracket makes
the rotating untouched-atomic outliers explicit: the two baseline pairs for
`bug339297` are 1.039/1.027 and 1.031/1.035 ms, while the candidate pairs are
1.049/1.035 and 0.993/1.027 ms.

The independent twenty-draw MSAA Metal A-B-B-A is load matched:

| order | runner | host idle | encoder rows | pending-write lifetime/frame | median frame |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | item-115 baseline | 83.83% | 4,840 | 2.396 ms | 4.766 ms |
| 2 | shared path lifetime | 80.32% | 2,641 | 1.173 ms | 2.845 ms |
| 3 | shared path lifetime | 84.56% | 2,641 | 1.180 ms | 2.852 ms |
| 4 | item-115 baseline | 83.50% | 4,840 | 2.421 ms | 4.748 ms |

The candidate retains all twenty tessellation passes but removes the twenty
per-frame null-texture clear passes. Its twenty-draw Time Profiler capture
drops from 119 to 55 renderer samples, `PathPipeline::prepare` from 35 to two,
and `create_buffer_init` from 24 to zero. The full C++/Rust report improves
from 5.053695x to 4.598614x aggregate, and the worst scene improves from
11.994x to 8.750x. Renderer exact=1,409/diverges=0/gated=59, V2 floors 584/35,
and the full workspace suite remain green.

## Flush-Wide MSAA Tessellation

Item 117 built immutable runners from item 116 and the current candidate. Their
SHA-256 hashes are `995b2dd726595a440a7f08017541fac10140a9c1f42d42789f19466f455aeca2`
and `ab5d795145aa57f78d0adc4b5de07599f6e6eddb9df912c627dddf0a05930c7b`.
Direct MSAA paths now shelf-pack midpoint spans into one tessellation texture
per logical flush. Path, paint, paint-aux, contour, and uniform resources share
the flush lifetime; only an image draw's texture/sampler group remains per
draw. Existing logical-flush, clip-reset, destination-read, advanced-blend,
and 1,024-draw submission boundaries are unchanged.

The first corpus run located a correctness defect rather than a tolerance
candidate. Authored empty contours leave sparse contour IDs in tessellation
spans, while the compact `ContourData` array contains only drawable contours.
Dense rebasing aliased the next path's first contour. Flush packing now retains
zeroed slots for those authored gaps and rebases the actual IDs. The focused
sparse-ID regression and `gm-emptystroke-msaa` are exact.

The structural oracle is decisive. Independent Metal exports reproduce 2,641
to 551 total encoder rows and 2,200 to 110 tessellation rows over 110 frames.
The 2,090-row reduction is exactly nineteen removed tessellation passes per
frame: the known C++ architecture mismatch moved from twenty passes to one.

Timing is retained as a directional snapshot, not as the proof of this slice:

| report | aggregate candidate/baseline | `bevel180strokes-msaa` |
| --- | ---: | ---: |
| first | 0.923633x | 0.661817x |
| replicate | 0.914020x | 0.651227x |

The freshly built C++ Dawn runner hash is
`cdc8fe44337236f14ae3bda4f212a4985b48c4539ef278d96b4674c844b507eb`.
It selects the same Apple M5 Max adapter and mode as Rust wgpu. The fixed
C++/Rust aggregate improves from 4.598614x to 4.144194x; the worst scene is now
5.764152x. Native C++ Metal is not used as the denominator and cannot execute
the report's MSAA matrix. Renderer exact=1,409/diverges=0/gated=59, V2 floors
584/35, and the full workspace suite remain green.

## Measurement Fence

R4 performance decisions use these controls:

1. Match evidence intensity to uncertainty. A deterministic reduction in
   redundant work is primarily proved by exact structural counters plus
   unchanged output/contracts. One light fixed timing snapshot is enough to
   catch a contradictory gross regression.
2. Use immutable repeated alternating reports and load-matched A-B-B-A when
   the claimed benefit is defined by timing, the effect is disputed, or the
   directional snapshot contradicts the structural oracle.
3. For timing-defined work, treat end-to-end submit-to-GPU-complete frame time
   as primary. Trace intervals are diagnostic and may veto only a reproducible
   material regression, not a one-off absolute-duration change.
4. Record system load for controlled timing and defer captures during known
   build, deletion, or indexing spikes. Absolute values from different load
   windows are not directly comparable.
5. Require the renderer pixel ratchet, relevant deterministic counters, both
   V2 segment floors, and the full workspace suite for every accepted slice.
   Run the normal and scripted V2 make targets serially because both write
   `target/debug/rust-golden-runner` during setup.

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

## Counter Oracle

Item 118 adds feature-gated API-boundary counters to both fenced runners and a
separate `make perf-counter-compare` build lane. Normal timing binaries do not
contain active counter collection. The report validates adapter, mode, scene,
and structural parity, rejects a backend that reports zero live work, and
ranks only candidate excess.

Across the fixed sixteen variants, command encoders and submissions are exact
at 16/16. Rust reports 154 versus 91 render passes, 554 versus 324 bind-group
sets, 278 versus 171 texture bindings, 273,640 versus 156,832 uploaded bytes,
and 220 versus 158 GPU draw calls. The one-frame timing sum is 3.224x and is
directional only.

The top row is `gm-bevel180strokes-msaa`: 63 Rust bind-group sets versus five
in C++ Dawn. Source inspection makes the excess objective. C++ creates the
per-flush group once and carries a `needsNewBindings` flag across draw batches,
invalidating it only after pass restart or image state. Rust currently rebinds
the same three direct-path groups for every draw. Item 119 ports that rule.

The complete source-mapped checklist is
`docs/renderer-r4-mechanism-inventory.md`.
