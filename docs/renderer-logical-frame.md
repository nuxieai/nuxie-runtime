# Renderer logical frame

`nuxie-renderer` has one production logical-frame seam shared by the WGPU
renderer and a GPU-free null adapter. The seam owns backend-independent draw
state, path admission, fill/stroke/feather preparation, clip replay, logical
resource limits, and the ordered draw/resource plan consumed by each backend.
MSAA intersection-board scheduling and the draw/resource permutation happen
when that shared plan is finalized, before either adapter consumes it.

`WgpuFrame` owns a `LogicalFrame`; draw admission appends each draw and its
resource layout to that object exactly once. Admission retains fill, stroke,
interior, and mode-specific feather tessellations. The production writer emits
the renderer's real typed `PathData`, `PaintData`/`PaintAuxData`,
`ContourData`, `TessVertexSpan`, and `TriangleVertex` records into a retained
buffer ring; there is no parallel count-only geometry serialization.
`WgpuFrame::finish` invokes that writer once, then its encoders consume the exact
typed paths, paints, contours, tessellation spans, triangles, prepared gradients,
and retained geometry before GPU upload and submission.
`NullLogicalRenderer` invokes the identical writer and stops, without a WGPU
context, device, queue, encoder, or submission.

`NullLogicalRenderer::flush` keeps diagnostic hashing out of benchmark timing;
`flush_with_diagnostics` fingerprints the exact typed output records. It also
reports exact shadow bytes, buffer write operations, retained allocation
growth, and per-flush rewinds. Retained capacity is the peak reusable capacity
required by any one logical flush, not the sum of every flush in the frame.
`WgpuFrame::finish_logical_frame_for_differential` crosses the production CPU
boundary, runs the real command encoder and submission path without pixel
readback, and records that the encoder consumed the shared typed output.
Differential tests use that method, not the diagnostic-only planning helper, so
a writer-only shadow implementation cannot falsely pass.

The null adapter deliberately has a narrow begin/draw/flush API. Add an
operation to it only when the logical phase must model that operation for both
backends. Backend-specific encoding, image upload, and readback stay on
`WgpuFrame`; moving those into the logical interface would make the seam wider
without improving CPU measurement fidelity.

## Maintenance contract

- Changes to fill rules, stroke joins/caps, feathering, clipping, resource
  layout, allocation limits, or logical rollover belong in the shared
  logical-frame module. Backend writes must consume that plan rather than
  recomputing it.
- Every new CPU Draw workload shape needs a differential test that feeds the
  same owned path and paint data through WGPU and null logical frames and
  compares their complete reports.
- The null adapter retains typed CPU records that are byte-identical to GPU
  upload inputs, but it must not create or retain GPU resources.
- Logical buffers grow with retained capacity and rewind their written ranges
  after each flush. A later small frame must reuse capacity without retaining
  the previous frame's contents.
- Both adapters lease stroke-preparation scratch from the same bounded retained
  pool lifecycle. The null adapter does not allocate fresh scratch per frame.

This trades a small public diagnostic/null API for one meaningful CPU seam.
The alternative—maintaining a benchmark-only planner—would be easier to call
but could silently stop measuring production work.
