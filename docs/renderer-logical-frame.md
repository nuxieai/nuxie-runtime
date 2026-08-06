# Renderer logical frame

`nuxie-renderer` has one production logical-frame seam shared by the WGPU
renderer and a GPU-free null adapter. The seam owns backend-independent draw
state, path admission, fill/stroke/feather preparation, clip replay, logical
resource limits, and the ordered draw/resource plan consumed by each backend.
MSAA intersection-board scheduling and the draw/resource permutation happen
when that shared plan is finalized, before either adapter consumes it.

`WgpuFrame` owns a `LogicalFrame`; draw admission appends each draw and its
resource layout to that object exactly once. `WgpuFrame::finish` invokes the
shared production resource writer once, then its encoders consume the prepared
gradient records and plan-owned geometry before performing backend-specific GPU
uploads and submission. `WgpuFrame::prepare_logical_frame` explicitly runs the
same retained-buffer writes with diagnostic fingerprinting. `NullLogicalRenderer`
consumes the production resource writer and stops, without a WGPU context,
device, queue, encoder, or submission. It is intended for CPU benchmarks and
differential tests, not as a second renderer implementation.

`NullLogicalRenderer::flush` keeps diagnostic hashing out of benchmark timing;
`flush_with_diagnostics` fingerprints authored clip rectangles and gradient
kind, geometry, colors, and stops. It also reports exact shadow bytes, buffer
write operations, retained allocation growth, and per-flush rewinds. Retained
capacity is the peak reusable capacity required by any one logical flush, not
the sum of every flush in the frame.

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
- The null adapter may use shadow bytes to model buffer writes, but it must not
  create or retain GPU resources.
- Logical buffers grow with retained capacity and rewind their written ranges
  after each flush. A later small frame must reuse capacity without retaining
  the previous frame's contents.
- Both adapters lease stroke-preparation scratch from the same bounded retained
  pool lifecycle. The null adapter does not allocate fresh scratch per frame.

This trades a small public diagnostic/null API for one meaningful CPU seam.
The alternative—maintaining a benchmark-only planner—would be easier to call
but could silently stop measuring production work.
