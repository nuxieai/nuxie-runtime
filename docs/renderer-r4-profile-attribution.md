# Renderer R4 Profile Attribution

Date: 2026-07-15; updated 2026-07-17

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
   unchanged output/contracts. Record one light fixed timing snapshot as
   directional context, but do not turn a load-unmatched sample into an
   acceptance gate for counter-defined work.
2. Use immutable repeated alternating reports and load-matched A-B-B-A when
   the claimed benefit is defined by timing, the effect is disputed, or exact
   counters do not locate the claimed benefit. Within each report, alternate
   C++-first and candidate-first pairs, select the minimum C++ control sample,
   and use the candidate from that same sample index. A noisy directional
   sample by itself does not trigger A-B-B-A for an objective work-elimination
   slice.
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
restarting it after a new pass and rebinding for each image draw. Rust
previously rebound the same three direct-path groups for every draw. Item 119
ports that rule.

Item 119 carries those bindings across compatible direct path, fill, and clip
pipelines. Image draws update only group 1; pass restarts and foreign layouts
require all direct-path groups again. Seven fixed MSAA variants remove 141
sets, including `gm-bevel180strokes-msaa` at 63->6. Aggregate Rust work moves
554->413 sets against C++ Dawn's 324. The light one-frame aggregate moves
3.224x->2.033x, a directional smoke check consistent with the structural win.
No repeated alternating timing campaign is required by the measurement fence.

Item 120 attributes the target's 54,696 bytes before changing code. Static
uniform, path, paint, paint-aux, and contour data account for only 3,432 bytes.
The tessellation arena writes 51,264 bytes across eighty packed uploads, with
46,080 bytes of payload. The existing shared midpoint layout admitted simple
fills but excluded compatible plain strokes, so twenty strokes rebuilt the
texture/pass lifetime that C++ lays out once per logical flush.

Plain non-feather strokes now enter that shared layout when the run has no
image, triangle mesh, atlas, clip update, forced specialized clockwise batch,
or loaded destination color. The last condition is evidence-driven: the first
broad candidate failed `overstroke_blendmodes`, `xfermodes2`, and all five
`fix_rectangle` clockwise-atomic frames. Those advanced-blend runs load prior
color and require their old per-draw lifetime. Adding the fence restores the
full renderer corpus to exact=1,409/diverges=0/gated=59.

On `gm-bevel180strokes-clockwise-atomic`, Rust moves from 42 to 23 passes,
54,696 to 13,224 uploaded bytes, and 41 to 22 GPU draws. C++ Dawn reports 23,
8,448, and 23. Across all sixteen variants, Rust moves from 154 to 116 passes,
133 to 67 created bind groups, 413 to 332 bind-group sets, 278 to 113 texture
bindings, 273,640 to 172,008 uploaded bytes, and 220 to 187 GPU draws. The
ranked excess count falls from 92 to 81.

The light directional snapshot is favorable on every affected Rust row; their
sum falls from 6.937 to 4.124 ms while the C++ controls move from 3.029 to
2.930 ms. The full aggregate ratio is intentionally not compared across these
windows because unrelated MSAA controls changed with machine load. Exact work
counters and unchanged output are the acceptance evidence. Both V2 floors
remain 584/35 and the full workspace suite passes.

The refreshed top mode-paired excess is `gm-batchedtriangulations`: MSAA emits
14 GPU draws versus four in C++ Dawn, while clockwise atomic emits 14 passes
and 13 draws versus five and five. Item 121 attributes those pass roles and
C++ batch boundaries before another scheduling change.

## Item 121: Interior Draw-Batch Merge

The four authored fills are non-overlapping and use interior triangulation.
C++ sorts their low-level work by draw group and draw type in
`renderer/src/render_context.cpp:1624-2070`, lays outer-curve and triangle
ranges contiguously, and merges adjacent ranges in `pushDraw` at
`render_context.cpp:3603-3768`. The counter split makes both Rust gaps exact:

- Atomic Rust emitted four tessellation passes, four outer-curve passes, four
  interior passes, and one resolve draw. C++ emits one flush tessellation,
  one outer batch, one interior batch, an explicit initialize draw, and one
  resolve draw.
- MSAA Rust emits one tessellation draw, twelve draw-major fill subpasses, and
  one composite draw. C++ emits one tessellation draw and three subpass-major
  fill batches; it resolves through the render attachment.

The atomic implementation now packs compatible plain outer-curve spans into
one C++-style 16-vertex layout, concatenates their triangle vertices, and
issues one outer and one interior draw per non-overlap group. The
outer-to-interior render-pass boundary is mandatory: a first candidate that
put both roles in one pass made the large-interior repeatability test differ
by three pixels. Restoring the C++ barrier is deterministic and repeatable.

On `gm-batchedtriangulations-clockwise-atomic`, Rust moves from 14 to five
passes, matching C++, and from 13 to four GPU draws versus C++'s five. The one
missing Rust draw is an advantage with a named cause: wgpu clears the backing
attachment directly while C++ Dawn emits `renderPassInitialize`. Path patches
are exact at 56/56, tessellation spans move 36->30 against C++'s 31, and Rust
uploads 4,600 bytes against C++'s 4,816. Across the fixed matrix, Rust moves
from 116 to 107 passes, 67 to 61 created bind groups, 332 to 302 bind-group
sets, 113 to 98 texture bindings, 172,008 to 168,936 uploaded bytes, and 187
to 178 draws. Ranked excess rows fall from 81 to 72.

The light one-frame target snapshot is Rust/C++=1.382x (0.640/0.463 ms), and
the fixed-matrix sum is 2.365x. These are directional smoke checks only; the
exact work reduction and unchanged outputs are the acceptance evidence.
Renderer exact=1,409/diverges=0/gated=59, both V2 floors remain 584/35, and
the workspace suite passes.

The remaining top row is the MSAA half at 14 draws versus four. Item 122 owns
only the C++-matched compact midpoint layout and subpass-major merge, with a
first structural target of twelve fill draws becoming three and fourteen
total draws becoming five. The separate Rust composite draw/pass is measured
after that merge rather than folded into the same change.

## Item 122: MSAA Subpass-Major Merge

C++ reserves one flush-wide midpoint padding span, writes path patch ranges
contiguously, and adds one final outer-patch alignment envelope in
`renderer/src/render_context.cpp:1094-1160`. Its draw list then sorts the
disjoint fills by low-level draw type and merges adjacent base/count ranges in
`render_context.cpp:1560-2090` and `3603-3768`. Rust previously retained each
standalone path's padding envelope, which separated its base ranges and forced
four calls for each of the three nonzero fill subpasses.

Rust now admits only a narrow compatible family: opaque, un-clipped, nonzero
fills with no image, gradient, feather, or destination read, where every path
fits one tessellation row. It removes the standalone padding spans, relocates
the paths into one contiguous flush range, and emits borrowed-coverage,
forward, and cleanup once each over that range. The synthetic regression pins
the resulting 5 draws, 19 spans, and 12 path patches, then compares the merged
scheduled pixels with a unique-group serialized render. The existing overlap
regression separately preserves source ordering across intersecting paths.

On `gm-batchedtriangulations-msaa`, GPU draws fall 14->5 against C++ Dawn's
four, draw instances fall 114->105 against 104, and tessellation spans fall
32->23, now exact. Path patches stay exact at 81. The one remaining draw and
instance are Rust's separate fallback composite. Across all sixteen fixed
variants, Rust draws move 178->169, instances 6,362->6,353, spans
1,677->1,668, uploaded bytes 168,936->168,424, and ranked excess rows 72->71.

The target's light snapshot is Rust/C++=4.904x (1.389/0.283 ms), while the
fixed-matrix sum is 2.172x. These numbers are directional context only. The
exact draw/range reduction and unchanged output are the acceptance evidence.

The refreshed highest-ranked family is now ordinary MSAA render passes: every
fixed MSAA row reports four in Rust versus two in C++ Dawn. The source cause is
exact. Rust clears the final view, resolves to a fallback texture, and performs
a final premultiplied composite in `crates/nuxie-renderer/src/lib.rs`; C++'s
`MSAADrawRenderPass` clears its transient multisample attachment and resolves
directly to the target in
`renderer/src/webgpu/render_context_webgpu_impl.cpp:3660-3794`. Item 123 owns
the clear-owned, non-advanced direct-resolve path. Preserve-target and
destination-read runs retain fallback composition.

## Item 123: Clear-Owned Ordinary MSAA Direct Resolve

C++'s `MSAADrawRenderPass` clears its multisample color attachment and names
the final target as its resolve attachment. Rust previously resolved ordinary
MSAA into a transparent fallback texture, surrounded that pass with a separate
final-target clear and premultiplied composite, and therefore paid two extra
passes plus one extra draw on every fixed MSAA row.

Rust now resolves directly into the final view only when the ordinary MSAA run
owns the target clear. The multisample attachment receives the frame clear
color, and the fallback texture, standalone clear, and final composite are all
skipped. Later submission chunks and atomic-to-fallback transitions still use
the transparent fallback and composite because they must preserve existing
target contents. Advanced destination-read MSAA remains unchanged. Empty
frames also resolve their clear directly, and logical-flush reloads continue to
seed the multisample attachment from the resolved target.

Across the fixed matrix, Rust render passes fall 107->91, exactly matching C++
Dawn. GPU draws fall 169->161 against C++'s 158, instances 6,353->6,345,
created bind groups 61->53, bind-group sets 302->294, and texture bindings
98->90. Ranked excess rows fall 71->47. On
`gm-batchedtriangulations-msaa`, Rust moves from four to two passes, five to
four draws, and 105 to 104 instances, all exact with C++ Dawn.

The fresh one-frame target snapshot is Rust/C++=1.657x and the matrix sum is
2.839x. They are directional context only; no cross-window timing claim is
attached. Exact counter parity and unchanged output are the acceptance
evidence. Renderer exact=1,409/diverges=0/gated=59, V2 floors remain 584/35,
the renderer feature suite passes 267/38, and the workspace suite passes. A
Sol review found no implementation issue and prompted a focused regression
that forces an overlapping translucent SrcOver draw into a later submission
chunk.

The refreshed top excess is mode-paired stroke tessellation on
`gm-bevel180strokes`: Rust emits 120 spans versus C++ Dawn's 63 in both modes.
Rust relocates each standalone stroke into the shared texture but retains its
three local padding spans; C++ emits the three padding spans once per logical
flush. Item 124 owns that compact shared-stroke layout.

## Item 124: Flush-Wide Plain-Stroke Padding

C++ emits the leading midpoint padding, aligned outer-curve transition, and
final sentinel once around all compatible stroke geometry in a logical flush
(`renderer/src/render_context.cpp:1128-1179,1516-1534`). Rust already shared
the texture lifetime, but it copied each standalone path's three zero-contour
padding spans into that texture.

Both Rust paths now strip those local padding spans, relocate only real contour
geometry into contiguous base-instance ranges, and emit one flush-wide padding
envelope. Eligibility is deliberately narrow: multiple plain solid SrcOver
strokes, no clip, image, gradient, advanced blend, destination read, or
multi-row tessellation, with the complete aligned layout fitting one 2,048-wide
row. All other draws keep the prior layout.

On `gm-bevel180strokes`, both modes move from 120 to 63 tessellation spans,
exactly matching C++ Dawn. MSAA instances move 160->103, exact; atomic moves
161->104 against C++'s 105 because C++ counts an explicit initialize draw while
Rust clears the attachment. Path patches remain 40. The same mechanism makes
`gm-CubicStroke` exact at nine spans in both modes.

Across the fixed matrix, Rust spans move 1,668->1,542, instances
6,345->6,219, uploaded bytes 168,424->160,744, and ranked excess rows 47->38.
C++ Dawn reports 1,615 spans, 5,872 instances, and 156,832 uploaded bytes. The
light candidate snapshot reports 1.186x on the atomic target, 3.882x on the
MSAA target, and 1.889x in aggregate. It overlapped the full renderer pixel
sweep, so these are contaminated directional context only and are not compared
with prior timing. Exact work reduction and unchanged output are the acceptance
evidence; no A-B-B-A campaign is warranted by the measurement fence.

The focused mode-paired regression, renderer feature suite, workspace suite,
normal and scripted V2 floors, and full renderer corpus pass. An independent
review found no issue in the eligibility fences, relocation arithmetic,
contour IDs, logical-flush boundaries, or texture-width limits.

The refreshed top excess is direct-stroke draw condensation on `gm-OverStroke`.
C++ emits seven compatible stroke batches plus shared setup/resolve work; Rust
emits one draw per each of twelve strokes. Item 125 owns the narrow contiguous
direct-stroke merge: MSAA targets 13->8 exactly, while atomic targets 14->9
under Rust's zero-draw attachment clear (C++ reports 10 with its initialize
draw).

## Item 125: Compatible Direct-Stroke Batches

C++ sorts the twelve `gm-OverStroke` strokes into seven non-overlap groups and
merges adjacent `msaaStrokes` ranges until a draw-batch, pipeline, resource, or
element-range break. Rust already produced the same seven intersection-board
groups, but encoded every stroke separately. Its shared texture shelf also
left the grouped base-instance ranges noncontiguous when the whole flush could
not fit one row.

Rust now admits only opaque, unclipped, solid SrcOver direct strokes with no
image, gradient, feather, or opacity modulation. Each established draw group
gets one compact texture row when all member layouts and the final padding
sentinel fit the device contract. Atomic and MSAA then merge only identical
pipeline/schedule ranges with contiguous relocated instances. All other draws
retain the old shelf and draw cadence.

On `gm-OverStroke`, atomic draws move 14->9 and MSAA 13->8. MSAA is exact;
C++ Dawn reports ten atomic draws because it counts the initialize operation
that Rust performs with an attachment clear. Path patches remain 498 in Rust
versus 497 in C++, while removing redundant row padding lowers fixed-matrix
Rust spans 1,542->1,514, instances 6,219->6,191, uploads
160,744->159,208 bytes, and draws 161->151. Ranked excess rows fall 38->35.

The one-frame snapshot reports 3.507x atomic, 3.455x MSAA, and 2.372x across
the matrix. It is load-unmatched directional context only and neither proves
nor vetoes this counter-defined change. Exact work counters, the production
counter regression, grouped-versus-unbatched MSAA pixels, boundary tests, the
273/38 renderer feature suite, the workspace suite, and the unchanged
1,409/0/59 corpus are the acceptance evidence. Sol found no implementation
defect; its request for end-to-end row and boundary coverage is incorporated.

### Item 126 Update

The apparent borrowed-stroke diagnosis was false. The +200 patches in each
clockwise-atomic `bug339297` row are fill geometry emitted by Rust's bespoke
single-contour ear triangulator. Production C++ sends the same contour through
`GrInnerFanTriangulator` and prunes its two authored zero-length lines before
interior preparation.

Rust now routes every eligible contour through the ported global
`InnerFanTriangulator`, applies C++ numeric point equality during fill
preparation, and uses the equivalent wgpu face culling for C++ WebGPU's CW
front-face convention. Path patches move 623->423 and 631->431, exact with
C++ Dawn. The direct preparation oracle matches contour records, triangle
order, and every `RGBA32Uint` tessellation texel. Full C++ Dawn frames differ
at zero pixels beyond channel delta 2 with maximum delta 1, so the two primary
references move from native Metal to same-tier C++ Dawn and their tolerance
tightens from 2/1,280 to 2/32. Native Metal remains a cross-backend diagnostic:
the old C++ Dawn frame differs at 1,837 pixels/max-33 and Rust at 1,839/max-33.

This is a parity correction, not a blanket work reduction. Removing 400 wrong
outer patches restores the real C++ interior topology: fixed-matrix Rust path
patches move 4,666->4,266, while passes move 91->94, instances
6,191->5,987, spans 1,514->1,708, and uploads 159,208->179,824 bytes. Ranked
excess rows move 35->39 because the report counts every positive row equally,
including small residuals exposed by restored work. The first one-frame matrix
snapshot is 2.421x and is directional context only; no A-B-B-A campaign is
warranted for this source- and oracle-defined correction. Sol reports no
correctness findings.

### Item 127 Update

The `gm-batchedconvexpaths` span excess was redundant padding, not geometry.
C++ allocates tessellation vertices across the full `LogicalFlush`
(`renderer/src/render_context.cpp:1128-1160`) and emits one leading padding
span, optional inter-type alignment, and one final sentinel
(`renderer/src/render_context.cpp:1516-1533`). Rust already relocated the ten
translucent SrcOver fills into one texture, but only strokes and opaque
batchable fills used one shared padding envelope. Each translucent fill kept
its local leading and trailing records.

Rust now shares the midpoint layout for homogeneous plain nonzero fills while
keeping draw batching independent. Clip, clip-rect, gradient, image, feather,
advanced-blend, row-width, and logical-flush boundaries retain the old path.
Atomic spans move 101->78 and MSAA spans 105->78, both exact. Atomic
instances move 244->221 and uploads 9,752->8,216 bytes; MSAA instances move
318->291 exactly and uploads 10,400->8,608 bytes. Draw calls and patch counts
are unchanged. The residual 600 atomic and 992 MSAA upload bytes are a
separate alignment or buffer-layout question.

Across the fixed matrix, Rust spans move 1,708->1,658, instances
5,987->5,937, uploads 179,824->176,496 bytes, and ranked positive rows
39->35. Both target frames are byte-identical before and after. The one-frame
matrix snapshot is 2.114x and remains directional context only; exact work
elimination plus unchanged rendering accepts this source-defined change
without A-B-B-A.

The highest coherent remaining row is `gm-bug339297_as_clip-msaa`: Rust/C++
report 3,704/2,816 upload bytes, 23/18 spans, 854/830 path patches, and 9/8
draws. Item 128 splits clip-update work from content preparation before any
edit.

The complete source-mapped checklist is
`docs/renderer-r4-mechanism-inventory.md`.

### Item 128 Update

`gm-bug339297_as_clip` combines three independent MSAA preparation effects.
C++ counts authored lines as one tessellation segment regardless of transformed
coordinate magnitude (`renderer/src/draw.cpp:1155-1368`), leaves stale stencil
resident while unclipped content draws
(`renderer/src/rive_renderer.cpp:578-633`), and allocates clip plus content
tessellation in one logical coordinate range across texture rows
(`renderer/src/render_context.cpp:1128-1160,1516-1533,3150-3292`). Rust
subdivided the enormous covering rectangle as cubics, eagerly cleared the
stencil, and stopped flush-wide compaction at one texture row.

Rust now keeps lines at one segment, defers a resident stencil clear until an
unrelated clip replaces it, and rebases eligible midpoint spans into one global
multi-row logical-flush envelope. Forward and reflected spans split only when
they cross a texture row, matching C++'s `TessellationWriter` contract.

On `gm-bug339297_as_clip-msaa`, patches move 854->830, draws 9->8, spans
23->18, and instances 878->848, all exact with C++ Dawn; bind-group sets move
6->5 exactly. Upload bytes move 3,704->3,120 against C++'s 2,816, leaving a
separate shared-resource upload row. The reusable multi-row path also removes
all 17 excess `gm-OverStroke-msaa` spans and 16 of its 17 excess instances.

The corpus regression probe also pinned C++ clip equivalence more precisely:
it compares matrix, fill rule, and a globally unique RawPath mutation ID, not
path geometry. `spotify_kids_demo` creates a distinct path with equivalent
geometry and therefore clears and redraws its resident clip. Rust now carries
that mutation snapshot; its seven-draw Spotify prefix hashes exactly to C++
Dawn without giving back the `bug339297_as_clip` counter result.

Across the fixed matrix, Rust spans move 1,658->1,634, instances
5,937->5,885, uploads 176,496->174,888 bytes, and ranked positive rows
35->26. The one-frame matrix snapshot is 1.999x and is directional context
only. Exact counters and unchanged strict Dawn pixels accept this
source-defined correction without A-B-B-A.

The full remaining tail is classified once in
`docs/renderer-r4-counter-tail-audit.md`: all 26 rows belong to four shared
implementation clusters, no final-pixel capture is missing, and no row is
closed as a counter-only accounting difference.

### Item 130 Update

The `BUG-MIX` cluster was one C++ logical-layout mismatch, not ten independent
timing targets. C++ places all midpoint work first, aligns once to the outer
patch span, places all outer work, and emits one final sentinel. Rust now
relocates both sections into that address space and rebuilds any pre-wrapped
forward/reflected spans before upload. Resource sharing no longer implies draw
batching: clip-update barriers retain their original passes.

Both normalized target tuples reach exact C++ values:
`gm-bug339297=(6,5,542,117,423)` and
`gm-bug339297_as_clip=(8,7,555,121,431)` for
`(passes,draws,instances,spans,patches)`. All ten rows disappear and the report
moves 26->16. The final directional one-frame ratios, 1.502x and 0.984x, are
context only; exact work parity and unchanged pixel contracts define
acceptance.

Parallel attribution now feeds the serial tail:

- `OVER-AENV`: replace seven atomic direct-stroke group envelopes with one
  logical-flush envelope while retaining execution groups; deterministic
  deltas are -16 spans, -16 instances, and -1,024 upload bytes.
- `UPLOAD-DUP`: pass tessellator-uploaded uniform/path/contour slices into the
  MSAA pipeline, then atomic. Seven MSAA rows have deterministic typed-payload
  targets; atomic closure follows per-class byte attribution.
- `UPLOAD-LAYOUT`: the 600/992-byte `batchedconvexpaths` residuals are not yet
  proved duplicate payloads and stay separately named.
- `OVER-PATCH`: a twelve-draw per-draw `RIVEATS` oracle must locate the first
  cumulative stroke-preparation mismatch before any source edit.

No fresh final-pixel capture is required. The only new artifact is the focused
preparation oracle for `OVER-PATCH`; the final timing gate remains staged for
the timing-defined R4 decision after deterministic rows are closed.

### Item 131 Update

`OVER-AENV` is closed as one structural cluster. Atomic OverStroke now shares
one logical midpoint envelope across all direct-stroke groups while retaining
the original render barriers. Spans move 506->489, instances 1,005->988, and
uploads 43,496->42,472 bytes. The 1,024-byte reduction exactly matches sixteen
removed 64-byte padding records, and the two positive counter rows disappear;
the report moves 16->14.

The current one-frame ratio is recorded only as directional context because
host load was not controlled. It neither accepts nor rejects this slice. The
deterministic counter delta and unchanged pixel contract are the evidence, and
the remaining atomic OverStroke upload residue stays in `UPLOAD-DUP`.
Final verification passes the renderer corpus at 1,409 exact, zero divergent,
and 59 retained gates; normal/scripted V2 floors at 584/35 exact segments; the
full workspace; formatting; and diff hygiene. Sol's read-only review passes
with no findings.

### Item 132 Update

The upload tail was one shared ownership mismatch. C++ binds its per-flush
typed buffers to both tessellation and drawing; Rust previously copied
uniform/path/contour payloads into tessellation storage and then uploaded them
again in each draw pipeline. One owned `TessellationFlushResources` now carries
the aligned slices through all four consumers.

The mode-wide reports move `14->6` after MSAA and `6->3` after general plus
specialized atomic reuse. Every upload row disappears, including both
`batchedconvexpaths` rows formerly parked as `UPLOAD-LAYOUT`; aggregate fixed
matrix uploads are 148,680 Rust bytes versus 156,832 C++ Dawn bytes. The
one-frame ratios remain directional context only. Exact byte accounting and
the finite row elimination accept this slice without A-B-B-A. Final
verification passes renderer exact=1,409/diverges=0/gated=59,
normal/scripted V2 floors at 584/35 exact segments, the full workspace,
formatting, and diff hygiene. Sol's read-only review passes with no findings.

### Item 134 Update

The final three rows were one float-semantics mismatch. A cumulative
twelve-draw probe found exact C++/Rust counts through OverStroke draw 2 and the
first +1 patch at draw 3. A raw `RIVEATS` sub-oracle reduced that difference
to the quadratic-as-cubic's parametric count: C++ emitted four segments while
Rust emitted five; polar and join counts matched.

C++ computes Wang second differences in path-local coordinates and applies
only the matrix's linear `VectorXform`. Rust transformed the control points
first, so cancellation after translation changed the segment ceiling. Shared
stroke, feather, fill, and outer-cubic preparation now use the C++ ordering.
The focused five-span artifact compares bit-for-bit, all twelve prefixes have
exact patch parity in both modes, and MSAA instances are exact at 986. The
fixed report moves `3->0` excess rows. Its current one-frame aggregate 2.618x
and 6.542x worst row are directional context only; exact structure and
unchanged pixels accept the correction without A-B-B-A. The next evidence is
the already-staged timing-defined R4 gate. Sol's final read-only review passes
after the raw segment words, per-prefix span and instance accounting, and a
near-boundary skew transform were added to the regression surface.

### Item 135 Update

The final gate now separates permissive capture from final acceptance: the
pre-tail A artifact may exceed the 2.0x shipping threshold, while only both
post-tail B reports must satisfy it. Comparator tests pin that distinction.
The first real runs did not produce an acceptance result under the then-current
70% host-idle admission fence. An explicitly exploratory run lowered only that
fence to 60%; it completed, but C++ control drift was 1.1893x and A repeat
drift was 1.1928x, so its aggregate result is rejected rather than interpreted
as a candidate verdict.

The invalid trace was still useful for attribution. Every MSAA row carried a
large fixed Rust frame floor even after deterministic command work reached
parity. Source comparison found that Rust created the final color texture, the
four-sample color texture, and the four-sample stencil texture inside every
`finish_internal`, while C++ Dawn's `ensureTarget` and render-target state
retain same-size attachments at context lifetime.

### Item 136 Update

`FrameAttachmentPool` now owns one factory-created attachment set. A frame
checks it out and returns it only after `device.poll(wait_indefinitely())` and
all requested readbacks complete. Overlapping frames allocate independent
attachments, but only one completed set is retained; overflow is dropped.
This matches C++ steady-state lifetime without making concurrent GPU use alias
the same target or retaining an unbounded resource high-water mark.

Two focused tests pin serial identity reuse in both render modes and the
one-entry overflow invariant. Sol's final review passes GPU completion,
readback ordering, early-error behavior, concurrent ownership, cache bounds,
and benchmark fairness. `make perf-counter-compare` remains at zero excess
rows, as expected for a resource-lifetime optimization.

The proportional timing evidence is a light seven-sample directional report,
not a gate. All sixteen old/current rows improve: aggregate 0.5038x,
clockwise atomic 0.8320x, and MSAA 0.2887x. A separate C++ Dawn/current report
is 1.4057x aggregate, 1.3816x atomic, and 1.4497x MSAA. Its worst row,
`gm-bug339297-clockwise-atomic`, is 2.0175x. That last value is close enough to
motivate the final load-admissible gate, but it is not accepted threshold
evidence by itself.

### Item 137 Update

The staged gate no longer divides independently selected C++ and Rust minima.
`rive-renderer-perf-v2` counterbalances execution order across all 112
scene-sample pairs: 56 execute C++ first and 56 execute Rust first. For each
scene it selects the first minimum C++ control and carries the Rust timing from
that exact sample index. The report stores the order vector and selected pair;
the aggregate sums only those pairs and the worst row retains full provenance.
This reduces sensitivity to machine load without pretending that unrelated
sample minima occurred together.

`r4-timing-comparison-v3` validates the exact fixed scene set and order,
adapter identity, timing method, sample order, selected pair, checked sums,
and worst-row provenance before applying the gate. The 2.0x boundary is
inclusive and covered directly. The shell additionally pins the manifest
hash, rejects any path or hash alias among C++/A/B runners, validates host-load
spread before writing a comparison, and always writes `gate-decision.json` so
an environment rejection cannot look like a renderer verdict. Sol's
adversarial review found the baseline-alias and load-decision gaps; both now
have integration regressions alongside low-idle and excessive-spread cases.

Re-evaluating the old item-136 sample vectors with the paired-control selector
gives old/current 0.5414x aggregate, 0.8741x atomic, and 0.3233x MSAA. The
C++/current view becomes 1.4809x aggregate, 1.4744x atomic, and 1.4928x MSAA,
with a 2.0277x worst row. These remain directional because the original runner
used fixed C++-then-Rust order; only fresh v2 artifacts can close item 135.

### Item 135 Admission Policy Update

Per the 2026-07-17 user decision, the gate no longer imposes an absolute
host-idle floor. The sampler still records every boundary and rejects excessive
spread across a bracket. Paired C++ controls and candidate-repeat drift remain
the load-validity checks, so a consistently busy machine can produce evidence
without pretending that a changing machine can.

The first complete no-floor bracket sampled 70.40%-75.10% idle (4.88 points of
spread), with 1.0304x C++ control drift and 1.0076x post-tail repeat drift. The
old A runner's 1.0509x repeat drift misses its 1.05 bound by 0.09%, but does
not obscure the stable post-tail candidate. Its aggregate is 1.6221x C++ Dawn,
and the selected-pair worst row is `gm-OverStroke-clockwise-atomic` at
2.9630x. Repeated medians also leave `gm-bevel180strokes-clockwise-atomic` and
`gm-batchedconvexpaths-clockwise-atomic` near or above 2.0x. The trace is
therefore useful evidence of a clockwise-atomic timing cluster, not a host-idle
rejection and not an R4 pass.

### Item 145 Update

Source comparison found a fixed pass-level mismatch in every generic atomic
frame. C++ Dawn applies the target clear through `colorLoadAction` on its
initial atomic render pass; Rust opened an otherwise empty clear-only render
pass and then loaded the target in the first atomic pass. Rust now carries that
clear into the first eligible generic pass. Specialized clockwise,
destination-read, and advanced paths keep their existing pass ordering.

The counter regression records `bug339297` at five rather than six passes and
`bug339297_as_clip` at seven rather than eight, with draws, instances, spans,
and patches unchanged. The 1,468-row renderer corpus stays at 1,409 exact, zero
divergent, and 59 gated. Adjacent ordinary reports move from 1.8696x to 1.6676x
aggregate (-10.8%). Generic atomic Rust p50 improves 14.3% for
`batchedtriangulations`, 13.6% for `bevel180strokes`, 13.1% for `bug339297`,
and 12.1% for `batchedconvexpaths`. The current selected-pair worst row remains
`gm-OverStroke-clockwise-atomic` at 2.8241x, so this deterministic correction
closes item 145 but does not close the timing-defined item 135.

### Item 146 Update

Generic atomic encoding recreated a one-pixel fallback texture, its view, and
a linear sampler on every frame even though `AtomicPipeline` already retained
descriptor-equivalent null texture and bilinear-clamp sampler resources. C++
Dawn likewise owns its null texture view and image samplers at context
lifetime. Rust now reuses those pipeline resources for absent gradients and
atlases.

Focused diagnostics remove both per-frame creation rows. Generic encode time
moves 134.8->39.5 microseconds for `bevel180strokes` (-70.7%) and
53.7->32.4 microseconds for `OverStroke` (-39.6%). An adjacent seven-sample
fixed-matrix report moves 1.7470x->1.4204x aggregate (-18.7%). `OverStroke`
is 1.8070x by its selected pair and 1.7718x by repeated p50;
`bevel180strokes` is the remaining worst row at 2.0451x selected-pair and
1.9960x p50. Rust already performs fewer passes, bind groups, and uploads than
C++ on both scenes while matching draws, instances, spans, and patches, so no
deterministic excess-work row remains.

The full renderer corpus passes at 1,409 exact, zero divergent, and 59 retained
gates. Replacing the bevel atomic route with MSAA is not pixel-compatible:
the two current outputs differ at 5,297 pixels (normalized RMSE 0.0172). That
experiment is rejected. At that point, item 135 remained open on generic
atomic bevel pass cost rather than resource creation or command volume.

### Item 147 And R4 Close

C++ Dawn creates `m_samplerBindings` once with the render context and reuses it
for every atomic render pass. Rust retained the sampler objects after item 146
but still rebuilt the identical three-binding sampler group for every frame.
`AtomicPipeline` now owns that bind group. Per-frame sampler-group creation
reaches zero, and the bevel row moves from four to three Rust bind groups while
preserving 90 bind-group sets, 23 draws, 105 instances, 63 tessellation spans,
and 40 patches. The 16-variant deterministic comparison remains at zero excess
rows.

The adjacent old/current p50 moves 1.6074->1.5604 ms for bevel (-2.9%); the
summed matrix moves 1.0267x and is directional context only. The final
counterbalanced C++ Dawn/current fixed matrix reports summed p50 ratios of
1.3718x overall, 1.3201x clockwise atomic, and 1.4656x MSAA. Its slowest p50
row is `gm-OverStroke-msaa` at 1.6431x. Every aggregate and row is within the
agreed 2.0x factor, so item 135 and R4 close with deterministic work parity and
same-capability directional timing parity. Final verification passes the
renderer corpus at 1,409 exact, zero divergent, and 59 retained gates; normal
and scripted parity stay at 584 and 35 exact segments; and the full workspace
suite passes.

## Exact-Parity Reopening

The 2.0x close above is retained as historical evidence, but it is no longer
the active performance definition. On 2026-07-17 the target was raised to
1.0x. The frozen comparison, exact gate, investigation loop, current native
Metal attribution, accepted/rejected hypotheses, and next queue now live in
`docs/renderer-parity-workflow.md`.

## Exact-Parity Architecture Attribution

The earlier 46-command-buffer trace in this document is historical, not the
current topology. The accepted vendored wgpu 30 core/HAL path preserves an
open Metal command buffer only when exhaustive render-command classification
proves that the next pass has no query reset, indirect validation, hidden
render-bundle work, store-discard repair, or other ordered pre-pass work. A
second opt-in drops a transition-only native buffer only when neither buffer
nor texture initialization encoded a clear. Metal is the only backend that
opts in, and strict-event sync disables both capabilities.

The current steady trace now matches C++ Dawn exactly at one physical
`MTLCommandBuffer` and three encoders: blit, tessellation, and solid. Canonical
source patch SHA-256 values are
`9751a43416597ec05ba9608f924cd4ada7eeb123643f0b45eec671c3c0245411`
for `wgpu-core` and
`9e55f5a57cbe17cfe0d61d22ab5c691e88e2dfba510496bd4a039fbc85893e69`
for `wgpu-hal`. The core patch also keeps attachment-overlap validation in an
inline `ArrayVec` for normal attachment counts and promotes without changing
membership or error order when that capacity is exceeded.

With physical submission topology equal, the focused CPU profile attributes
the largest remaining native backend category to render-pass begin/encoder
setup. A descriptor-caching probe was not retained because its measured effect
was not material. This identified the next cost center during investigation;
the final timing verdict is recorded below.

Renderer-side accepted changes in the same exact-parity pass are also
source-defined rather than timing claims:

- MSAA draw ordering is one flat schedule built by one retained board walk,
  with explicit authored ties and an in-place move permutation;
- solid-only frames leave gradient definitions and per-draw gradient tables
  empty;
- clockwise-atomic paints, auxiliaries, and triangle vertices share a grouped
  retained upload;
- clockwise coverage uses the C++ nonzero generation prefix and clears the
  full retained allocation only on allocation, growth, or generation wrap;
- three completion-guarded clockwise slots retain both coverage buffers and
  clip textures, preventing in-flight reuse;
- contour midpoint preparation reads endpoints in place, and direct MSAA
  stroke batches are derived on demand without full-frame end/continuation
  side arrays;
- exact construction-order multi-contour strokes bypass the generic contour
  remap walk and sort, eliminating 520 visits and 12 sorts per `OverStroke`
  frame;
- the three upload slots each retain six MSAA packing vectors, release excess
  capacity above 1 MiB per slot, and cleanly replace only an active staging
  belt when a frame is abandoned;
- a context-owned one-slot stroke-preparation scratch pool mirrors C++'s
  resettable midpoint-fan allocator, recycles through abandoned frames, uses
  uncached overflow for concurrent frames, and drops retention above 1 MiB;
- physical work metrics count every aligned staging copy. The final 16-row
  report has zero Rust excesses; CWA uses five Rust buffer copies versus six or
  seven C++ copies, and MSAA uses one versus six.

The three-slot design may retain roughly three times one frame's peak
coverage-and-clip allocation. That bounded memory increase is the explicit
price of safe concurrent frame ownership. Packing scratch has a separate bound
of 1 MiB per slot, roughly 3 MiB total.

## Exact-Parity Result

R4.1 closed on 2026-07-18. The source-controlled gate consumed the first five
fresh, seven-sample, counterbalanced reports after that final source change and
passed at these median Rust/C++ ratios:

| scope | ratio | reports at or below 1.0 |
| --- | ---: | ---: |
| overall | 0.966058 | 5/5 |
| clockwise atomic | 0.941193 | 5/5 |
| MSAA | 0.996544 | 3/5 |

Thus production Rust is about 3.4% faster overall on this fixed Apple M5 Max
matrix. The estimator equal-weights the two runner launch orders inside each
report and then takes the median of five. The older minimum-selected paired
diagnostic remains above 1.0 and is non-gating because selecting the minimum
C++ control biases that comparison.

The final runner SHA-256 values are
`c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`
for C++ Dawn and
`d876c3c4b35ec8d116af3e1f059f51830058b9baabe0090c9e5476212055d4b9`
for Rust. The complete provenance, source identity, five report hashes, gate
artifact, and reproduction workflow are in
`docs/renderer-parity-workflow.md`.
