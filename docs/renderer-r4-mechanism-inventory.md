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

### Item 120 Update

Attribution found only 3,432 bytes of static frame uploads on the target. The
remaining 51,264 bytes were eighty tessellation-arena writes with 46,080 bytes
of payload. Rust's existing shared midpoint layout excluded every stroke, so
twenty compatible plain strokes each paid a texture-layout and pass lifetime;
C++ lays out the logical flush once.

Plain non-feather strokes now use the shared layout when the run has no image,
triangle, atlas, clip update, forced specialized clockwise batch, or loaded
destination color. A first candidate omitted that last boundary and regressed
seven advanced-blend corpus rows; destination-read runs therefore preserve
their prior per-draw lifetime.

The fixed-matrix Rust totals change as follows:

| counter | item 119 | item 120 | C++ Dawn |
| --- | ---: | ---: | ---: |
| render passes | 154 | 116 | 91 |
| bind groups created | 133 | 67 | 115 |
| bind-group sets | 413 | 332 | 324 |
| texture bindings | 278 | 113 | 171 |
| buffer upload bytes | 273,640 | 172,008 | 156,832 |
| GPU draw calls | 220 | 187 | 158 |

`gm-bevel180strokes-clockwise-atomic` moves from 42 to 23 passes, 54,696 to
13,224 uploaded bytes, and 41 to 22 draws; C++ reports 23, 8,448, and 23. The
four affected Rust directional frames sum to 4.124 ms from 6.937 ms. Their C++
controls remain close at 2.930 ms from 3.029 ms, but this is still a light
cross-window snapshot rather than load-controlled timing evidence.

The refreshed top rows belong to `gm-batchedtriangulations`: MSAA has 14 GPU
draws versus four, while clockwise atomic has 14 passes and 13 draws versus
five and five. Item 121 owns the mode-paired batch-boundary attribution.

### Item 121 Update

C++ sorts the four disjoint interior fills by low-level draw type and merges
contiguous outer-curve and triangle ranges. Rust's atomic path instead built
four tessellation textures and issued four outer plus four interior draws.
Compatible plain interior fills now use one 16-vertex outer-curve layout, one
shared triangle buffer, and one draw per geometry role while retaining the
required outer-to-interior render-pass boundary.

`gm-batchedtriangulations-clockwise-atomic` moves from 14 to five passes and
13 to four draws; C++ reports five and five. The draw difference is C++ Dawn's
explicit initialize draw versus Rust's attachment clear. Path patches are
exact at 56, and spans move 36->30 against C++'s 31. Fixed-matrix Rust totals
move from 116 to 107 passes and 187 to 178 draws; ranked excess rows fall
81->72. The one-frame target snapshot is Rust/C++=1.382x and remains
directional only.

MSAA is now the unambiguous top row: twelve draw-major fill subpasses must
become C++'s three subpass-major batches. Item 122 ports the compact midpoint
layout and merge first; its target is 14->5 total draws. The remaining Rust
composite draw is attributed separately afterward.

### Item 122 Update

Compatible disjoint opaque nonzero fills now use C++'s compact flush-wide
midpoint layout. Rust strips each standalone path's padding envelope, relocates
the geometry into contiguous base-instance ranges, and emits the three fill
subpasses once each over the combined range.

`gm-batchedtriangulations-msaa` moves from 14 to five draws against C++ Dawn's
four. Tessellation spans move 32->23 and path patches remain 81, both exact;
the remaining draw/instance is Rust's separate fallback composite. Fixed-matrix
Rust totals move from 178 to 169 draws, 6,362 to 6,353 instances, 1,677 to
1,668 spans, and 168,936 to 168,424 uploaded bytes. Ranked excess rows fall
72->71. The target's one-frame ratio is 4.904x and the matrix sum is 2.172x;
both are directional only.

The refreshed top family is render passes on every ordinary fixed MSAA row:
Rust reports four versus C++ Dawn's two. C++ clears the transient multisample
attachment and resolves directly into the target. Rust additionally clears
the target, resolves into a transparent fallback, and composites the fallback
back into the target. Item 123 owns direct resolve for clear-owned,
non-advanced runs; preserve-target and destination-read paths stay unchanged.

### Item 123 Update

Clear-owned ordinary MSAA runs now clear the multisample attachment and resolve
directly into the final target. Preserve-target chunks retain the transparent
fallback texture and premultiplied composite, while advanced destination-read
runs keep their existing direct resolve and copy barriers.

Fixed-matrix Rust render passes fall 107->91, exactly matching C++ Dawn. GPU
draws fall 169->161 against C++'s 158, instances 6,353->6,345, created bind
groups 61->53, bind-group sets 302->294, texture bindings 98->90, and ranked
excess rows 71->47. `gm-batchedtriangulations-msaa` is now exact at two passes,
four draws, and 104 instances. The one-frame target and matrix snapshots are
1.657x and 2.839x; both remain directional only.

The top excess is now `gm-bevel180strokes` tessellation spans in both modes:
Rust reports 120 versus C++ Dawn's 63. The 57-span excess is the exact cost of
retaining three standalone padding spans for each of twenty strokes instead of
emitting three padding spans once for the shared logical-flush texture. Item
124 owns that compaction.

### Item 124 Update

Compatible plain strokes now share C++'s flush-wide midpoint padding envelope
in both modes. `gm-bevel180strokes` moves 120->63 spans in atomic and MSAA,
exactly matching C++ Dawn; MSAA instances move 160->103 exactly, while atomic
moves 161->104 against C++'s 105 counted-initialize convention. The fixed Rust
matrix moves from 1,668 to 1,542 spans, 6,345 to 6,219 instances, and 168,424 to
160,744 uploaded bytes. Ranked excess rows fall 47->38.

The candidate's one-frame target ratios are 1.186x atomic and 3.882x MSAA,
with a 1.889x matrix sum. The snapshot overlapped the full pixel sweep and is
therefore contaminated directional context only. Exact counter reduction and
unchanged pixels are the acceptance evidence.

The new highest row is `gm-OverStroke-msaa` GPU draws at 13 versus eight;
atomic reports 14 versus ten. C++ condenses twelve authored strokes into seven
compatible contiguous low-level batches. Item 125 owns the same narrow
direct-stroke range merge in both Rust modes.

### Item 125 Update

Compatible direct strokes now use the existing seven intersection-board groups
as their batch boundaries. When a whole flush cannot fit one texture row, each
group receives a compact row so its base-instance ranges remain contiguous;
the encoders merge only matching pipeline, schedule, resource, and range
contracts. `gm-OverStroke` moves 14->9 atomic draws and 13->8 MSAA draws, with
MSAA exact and atomic one below C++ because Rust has no initialize draw.

Fixed-matrix Rust draws move 161->151, instances 6,219->6,191, spans
1,542->1,514, and uploads 160,744->159,208 bytes. Ranked excess rows fall
38->35. The load-unmatched one-frame matrix snapshot is 2.372x and is recorded
only as directional context; exact counters and unchanged pixels accept the
slice.

### Item 126 Update

The +200 `bug339297` path-patch rows were misclassified as borrowed stroke
coverage. They came from a local single-contour ear triangulator that bypassed
C++'s global interior preparation. Rust now uses `InnerFanTriangulator` for all
eligible contours, prunes authored zero-length fill lines with C++ numeric
equality, and applies the physical counterclockwise-face cull implied by C++
WebGPU's CW-front pipeline convention.

Both target path-patch counts are exact at 423 and 431. A direct C++ oracle
matches contour records, triangle ordering, and every tessellation texel, and
the same-tier final frames are within the standard 2/32 contract at zero
pixels beyond delta 2/max-1. The primary references therefore move from native
Metal to C++ Dawn and the old 1,280-pixel allowances disappear.

Restoring C++'s interior topology also restores legitimate work, so the fixed
matrix is not uniformly smaller: path patches move 4,666->4,266 and instances
6,191->5,987, while passes move 91->94, spans 1,514->1,708, uploads
159,208->179,824 bytes, and ranked positive rows move 35->39. The 2.421x
one-frame snapshot is context only. Exact preparation, exact target counters,
same-tier pixels, the corpus ratchet, and the test floors accept the change;
no A-B-B-A campaign was run. Sol reports no correctness findings.

### Item 127 Update

C++ allocates midpoint tessellation vertices once across a `LogicalFlush` in
`renderer/src/render_context.cpp:1128-1160`, then
`renderer/src/render_context.cpp:1516-1533` emits one leading padding span,
optional inter-type alignment, and one final sentinel. Rust already shared a
texture for `gm-batchedconvexpaths`, but ten translucent SrcOver fills retained
path-local padding envelopes because layout compaction was coupled to opaque
fill or stroke draw batching.

Layout sharing is now independent of draw batching. Homogeneous plain nonzero
fills can use one flush-wide midpoint envelope while clip, clip-rect, gradient,
image, feather, advanced-blend, row-width, and flush boundaries remain fenced.
Atomic spans move 101->78 and MSAA 105->78, both exact. Atomic
instances/uploads move 244->221 and 9,752->8,216 bytes; MSAA moves 318->291
instances exactly and 10,400->8,608 bytes. Draws and patches do not change.
Target pixels are byte-identical.

The fixed matrix moves 1,708->1,658 spans, 5,987->5,937 instances,
179,824->176,496 upload bytes, and 39->35 ranked positive rows. The 2.114x
one-frame snapshot is context only. Exact counters and unchanged output accept
the slice without A-B-B-A. The remaining 600 atomic and 992 MSAA target upload
bytes are separately ranked alignment or buffer-layout work.

Item 128 next attributes `gm-bug339297_as_clip-msaa`, where Rust/C++ report
3,704/2,816 upload bytes, 23/18 spans, 854/830 path patches, and 9/8 draws.

### Item 128 Update

The target combined three C++ mechanisms that Rust had modeled separately:

- `renderer/src/draw.cpp:1155-1368` gives every authored line one segment;
  transformed coordinate magnitude does not invoke cubic subdivision.
- `renderer/src/rive_renderer.cpp:578-633` leaves stale stencil resident while
  unclipped content draws and clears it only before an unrelated root clip.
- `renderer/src/render_context.cpp:1128-1160,1516-1533,3150-3292` allocates one
  logical midpoint range across texture rows for clip and content paths.

Rust now follows all three. Its compact midpoint relocation accepts a global
logical base, wraps forward and reflected spans across row boundaries, and
emits one leading/inter-type/final padding envelope for the whole eligible
MSAA logical flush. `gm-bug339297_as_clip-msaa` reaches exact bind sets, draws,
instances, spans, and patches at 5/8/848/18/830. The only target residual is
304 upload bytes. The same mechanism removes the 17-span `OverStroke` MSAA
layout excess, proving this was a shared cause rather than a scene fix.

Resident clip reuse now also mirrors `RiveRenderer::ClipElement`: equivalence
uses matrix, fill rule, and the RawPath mutation snapshot. This distinguishes
separately created paths with equal geometry, as required by the byte-exact
`spotify_kids_demo` Dawn prefix, while preserving same-path reuse in item 128.

The fixed matrix moves 1,658->1,634 spans, 5,937->5,885 instances,
176,496->174,888 upload bytes, and 35->26 ranked positive rows. The 1.999x
one-frame snapshot is directional only. The complete finite tail and its four
cluster queue live in `docs/renderer-r4-counter-tail-audit.md`.

### Item 130 Update

C++ allocates midpoint and outer tessellation in one logical address space,
aligning the outer section only after all midpoint instances and wrapping both
forward and reflected ranges across texture rows. Rust previously prepared the
two `bug339297` atomic families as separate tessellation textures and passes.

Rust now uses the same pre-padding, midpoint section, outer alignment, outer
section, and final-sentinel order. Texture sharing is deliberately independent
of draw batching: unclipped content may use the grouped path, while clip-update
and clipped-content draws retain their semantic pass boundaries and merely
reuse the shared texture and triangle allocation.

The normalized `(passes, draws, instances, spans, patches)` tuples are exact at
`(6,5,542,117,423)` for `gm-bug339297` and `(8,7,555,121,431)` for
`gm-bug339297_as_clip`. Both upload totals are below C++ Dawn. The report falls
26->16 rows, removing the complete ten-row `BUG-MIX` cluster in one report.
A Sol review found that a shared clip batch with zero aggregate triangle
vertices could select an empty per-draw buffer vector, and that reflected
row-wrap rebuilding used checked subtraction where C++ deliberately wraps an
unsigned row. Buffer selection now branches on shared-flush ownership, both
reflection decrements use C++-matched wrapping, and focused regressions cover
the empty triangle range plus zero-relocation persistent wrap.

The complete remaining report was attributed concurrently. Atomic OverStroke
still owns sixteen group-local padding spans (`OVER-AENV`); uniform/path/contour
payloads are uploaded once for tessellation and again for draw pipelines
(`UPLOAD-DUP`); the two `batchedconvexpaths` byte rows require per-class layout
telemetry (`UPLOAD-LAYOUT`); and one shared OverStroke preparation patch remains
oracle-first (`OVER-PATCH`).

### Item 131 Update

Atomic direct strokes now use the same one-envelope logical relocation as the
other eligible flush-wide layouts. The tessellation texture is shared across
the flush, but `draw_group_starts` still split render execution wherever the
intersection board requires a barrier. The relocation dedup key also includes
the canonical reflected source location, so equal forward spans with distinct
reflections cannot collapse.

`gm-OverStroke-clockwise-atomic` moves 506->489 spans, 1,005->988 instances,
and 43,496->42,472 uploaded bytes. The exact 1,024-byte delta is sixteen
removed 64-byte envelopes. Both `OVER-AENV` rows disappear and the report
moves 16->14; the remaining 4,328 atomic upload-byte excess stays assigned to
`UPLOAD-DUP`. The one-lower normalized Rust span/instance counts are retained
because C++'s atomic/MSAA raw span counts differ despite equal upload bytes and
C++ counts an initialize operation that Rust performs without a draw.
Sol's independent read-only review passes with no findings across eligibility,
execution barriers, logical relocation, reflection wrapping, texture bounds,
and focused regression coverage.

## Port Checklist

| mechanism | C++ source | Counter or symptom | Rust standing |
| --- | --- | --- | --- |
| Triple-buffer GPU upload rings | `renderer/include/rive/renderer/buffer_ring.hpp:11-79`; `renderer/include/rive/renderer/gpu.hpp:75-77`; `renderer/src/webgpu/render_context_webgpu_impl.cpp:2632-2804` | upload calls/bytes, allocation and pending-write work | Core frame upload arena and guarded completed-frame slots are ported; continue comparing byte volume. |
| Dynamic render-buffer rings | `renderer/src/webgpu/render_context_webgpu_impl.cpp:2253-2334` | per-update buffer allocation and copies | Retained Rust render buffers exist; ring/capacity behavior remains a later counter-led check. |
| Logical-flush container reuse | `renderer/src/render_context.cpp:155-157`, `268-273`, `282-343`, `998-1004` | host allocation without changing GPU counters | Rust retains frame containers in several paths; audit only when profiles identify host churn. |
| Resource-budget flush splitting | `renderer/src/render_context.cpp:497-573`, `663-725` | logical flushes and draws per flush | Rust preserves the 1,024-draw and resource fences; fixed corpus is structurally exact. |
| Frame-wide layout, then upload and encode | `renderer/src/render_context.cpp:740-822`, `953-993` | command encoders, submissions, upload calls | Ported to one encoder and one submission per fixed variant; these counters are exact. |
| Flush-wide tessellation padding | `renderer/src/render_context.cpp:1128-1160`, `1516-1533`, `3150-3292` | tessellation spans, instances, upload bytes | Eligible MSAA clip/content, mixed atomic midpoint/outer, and atomic direct-stroke layouts share one multi-row logical envelope while execution barriers remain separate. |
| Retained allocation high-water marks | `renderer/src/render_context.cpp:837-938`, `2562-2910` | allocation churn and upload capacity | Persistent atomic backing and frame upload arenas are ported. The 125% growth and five-second trim policy are not yet copied wholesale. |
| Gradient content deduplication | `renderer/src/render_context.cpp:575-662` | gradient rows, texture work, draw calls | Functional gradient batching exists; no texture uploads occur in the warm fixed matrix. Revisit with a gradient-heavy counter scene. |
| Skyline feather-atlas packing | `renderer/src/render_context.cpp:663-724`, `2205-2290` | atlas passes, patch instances, texture dimensions | Functional atlas batching is ported; retained atlas allocation policy remains counter-led. |
| Draw-batch merge and explicit barrier breaks | `renderer/src/render_context.cpp:3364-3770` | render passes, GPU draw calls, instances | Compatible fill and direct-stroke ranges now merge across C++-matched group and barrier boundaries. |
| Bind only on changed state | `renderer/src/webgpu/render_context_webgpu_impl.cpp:4265-4358` | bind-group sets and created groups | Ported for direct MSAA path-compatible layouts in item 119; fixed Rust sets fall 554->413. |
| Lazy pipeline-layout and render-pipeline caches | `renderer/src/webgpu/render_context_webgpu_impl.cpp:451-791`, `1268-1733`, `4440-4463` | frame-time pipeline creation | Rust pipelines are factory-owned. Counter recording begins after warmup and correctly excludes construction. |
| Factory-owned samplers, null resources, and static geometry | `renderer/src/webgpu/render_context_webgpu_impl.cpp:1845-2037` | bind groups created, texture bindings, initialized buffers | Samplers/null resources/static patch geometry are ported for active paths. |
| Retained transient render-target textures | `renderer/src/webgpu/render_context_webgpu_impl.cpp:2051-2179`, `3660-3794` | per-frame texture creation, clears, render passes | Rust retains core MSAA and atomic backing. Clear-owned ordinary MSAA now resolves directly into the final target; preserve-target runs retain fallback composition. |
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
- `render_passes` was exact at 91/91 before item 126 restored global interior
  topology. Rust now reports 94 versus 91; the three residual passes are in
  the two `bug339297` rows and remain below the current top-ranked work.
- Item 124 keeps `gm-bevel180strokes` tessellation spans exact at 63 in both
  modes. Item 127 moves `gm-batchedconvexpaths` spans to 78 exactly in both
  modes by matching C++'s flush-wide padding envelope.
- `gpu_draw_calls` no longer owns the top row. Item 125 moves
  `gm-OverStroke` to eight MSAA draws exactly and nine atomic draws versus ten;
  C++'s counted initialize operation explains the lower Rust value.
- `path_patches` no longer owns the top row. Item 126 moves both
  clockwise-atomic `bug339297` variants to exact C++ counts at 423 and 431 by
  matching global interior preparation.
- `gm-batchedconvexpaths` retains 600 atomic and 992 MSAA excess upload bytes
  after item 127 makes its spans exact. Treat those as separate alignment or
  buffer-layout work; they are no longer the highest coherent row.
- Item 128 makes `gm-bug339297_as_clip-msaa` exact for bind sets, draws,
  instances, spans, and patches. Its remaining 3,120/2,816 upload-byte row is
  part of the shared `UPLOAD-DUP` cluster.
- Item 130 closes all ten `BUG-MIX` rows and moves the report 26->16. The
  remaining rows are assigned to `OVER-AENV`, `UPLOAD-DUP`,
  `UPLOAD-LAYOUT`, or `OVER-PATCH` in
  `docs/renderer-r4-counter-tail-audit.md`. Work that does not close or narrow
  one of those finite rows is outside the counter-tail queue.
- Item 131 closes both `OVER-AENV` rows and moves the report 16->14. The
  remaining tail is eleven upload rows plus the three shared `OVER-PATCH`
  rows; atomic OverStroke's residual 4,328 upload bytes belong to
  `UPLOAD-DUP`.
- `buffer_upload_bytes`, `tessellation_spans`, and `path_patches` represent
  real data or geometry output. Reduce them only with a C++-matched data-layout
  or algorithm explanation; never optimize the counter by hiding accounting.
- Lower Rust upload-call count is not automatically a win: Rust coalesces into
  fewer writes. Item 120 reduced the byte excess from 116,808 to 15,176 with a
  source-matched shared layout; the residual is no longer the highest row.

Regenerate the ranked artifact with `make perf-counter-compare`. The JSON and
Markdown outputs live under `target/` and are intentionally not checked in.
