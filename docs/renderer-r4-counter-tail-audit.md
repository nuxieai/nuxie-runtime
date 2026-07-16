# Renderer R4 Counter-Tail Audit

This audit replaces row-at-a-time discovery with a finite cluster queue. It is
based on the authoritative `target/renderer-work-counters.md` report generated
after R4 item 128.

## Baseline And Classification

Item 128 reduced the ranked tail from 35 to 26 rows. It made
`gm-bug339297_as_clip-msaa` exact for bind-group sets, draw calls, draw
instances, tessellation spans, and path patches. The same multi-row logical
flush layout also removed all 17 excess `gm-OverStroke-msaa` spans and 16 of
its 17 excess instances.

The 26 current rows classify as:

| Class | Rows | Meaning |
| --- | ---: | --- |
| Accounting-only Decision | 0 | No current excess is only a counter-definition mismatch. |
| Shared implementation cluster | 26 | Every row belongs to one of four finite mechanisms below. |
| Genuine singleton | 0 | The remaining patch mismatch is mode-paired and therefore shared. |

Upload bytes remain real work: both shims count bytes passed to WebGPU buffer
uploads. Alignment and duplicate payloads must be reduced or retained with a
specific implementation reason; they are not closed by relabeling the counter.

No fresh Dawn pixel capture is needed for any current row. All 13 affected
scene/mode variants already have acceptance-grade references and provenance
pins. Only `OVER-PATCH` needs a new preparation-stage C++ oracle to locate one
record; it does not need a new final-pixel reference.

## Source Map

- `UC`: C++ typed resource uploads in
  `renderer/src/render_context.cpp:2753-2885` and
  `renderer/src/webgpu/render_context_webgpu_impl.cpp:2676-2696`.
- `UM`: Rust MSAA duplicate tessellator/path payloads in
  `crates/nuxie-renderer/src/tessellator.rs:172-236` and
  `crates/nuxie-renderer/src/path_pipeline.rs:817-900`.
- `UA`: Rust atomic duplicate payloads in
  `crates/nuxie-renderer/src/tessellator.rs:172-236` and
  `crates/nuxie-renderer/src/atomic_pipeline.rs:1056-1190`.
- `MC`: C++ mixed midpoint/outer logical-flush allocation and one tessellation
  pass in `renderer/src/render_context.cpp:1128-1160,1516-1533` and
  `renderer/src/webgpu/render_context_webgpu_impl.cpp:4025-4060`.
- `MR`: Rust's split atomic tessellation lifetime in
  `crates/nuxie-renderer/src/lib.rs:2296-2635`.
- `IC`: C++ counts a `renderPassInitialize` draw at
  `renderer/src/render_context.cpp:1847-1865`; Rust clears its backing without
  a draw. After real excess work is removed, Rust may correctly settle one draw
  or instance below the raw C++ count.
- `EC`: C++ wraps logical tessellation locations across texture rows in
  `renderer/src/render_context.cpp:3150-3292`.
- `ER`: Rust item 128's reusable MSAA logical relocation is in
  `crates/nuxie-renderer/src/lib.rs`; atomic packing still has single-row and
  group-row fences.
- `PC`: C++ stroke counting and preparation in
  `renderer/src/draw.cpp:1120-1175,1794-1935`.
- `PR`: Rust stroke preparation in
  `crates/nuxie-renderer/src/draw.rs:280-540`.

## Complete Row Triage

Values are `C++ Dawn / Rust wgpu` from the post-item-128 report. Every row is
class B: shared implementation work.

| # | Scene and counter | Value | Cluster | Target and next action |
| ---: | --- | ---: | --- | --- |
| 1 | `bug339297_as_clip` atomic render passes | 8 / 10 | `BUG-MIX` | One mixed logical-flush tessellation texture/pass; target 8. |
| 2 | `bevel180strokes` MSAA upload bytes | 8,448 / 10,304 | `UPLOAD-DUP` | Share tessellator resources with the path pipeline; target 8,448. |
| 3 | `CubicStroke` MSAA upload bytes | 2,176 / 2,608 | `UPLOAD-DUP` | Same shared-slice port; target 2,176. |
| 4 | `bug5099` MSAA upload bytes | 1,984 / 2,320 | `UPLOAD-DUP` | Same shared-slice port; target 1,984. |
| 5 | `bug339297` atomic render passes | 6 / 7 | `BUG-MIX` | Combine midpoint and outer/interior tessellation; target 6. |
| 6 | `batchedtriangulations` MSAA upload bytes | 3,584 / 4,160 | `UPLOAD-DUP` | Reuse uniform/path/contour slices; target 3,584. |
| 7 | `bug339297_as_clip` atomic texture bindings | 13 / 15 | `BUG-MIX` | One tessellation texture removes two bindings; target 13. |
| 8 | `bug339297_as_clip` atomic upload calls | 7 / 8 | `BUG-MIX` | Consolidate the mixed run, then reattribute any residue; target 7. |
| 9 | `bevel180strokes` atomic upload bytes | 8,448 / 9,640 | `UPLOAD-DUP` | Reuse tessellator resources in `AtomicPipeline`; target 8,448. |
| 10 | `OverStroke` atomic upload bytes | 38,144 / 43,496 | `OVER-AENV`, then `UPLOAD-DUP` | Port the multi-row envelope first, then remove duplicate payloads; target 38,144. |
| 11 | `batchedconvexpaths` MSAA upload bytes | 7,616 / 8,608 | `UPLOAD-DUP` | Shared-slice port; target 7,616. |
| 12 | `OverStroke` MSAA upload bytes | 38,144 / 43,008 | `UPLOAD-DUP` | Geometry spans are exact; deduplicate payloads; target 38,144. |
| 13 | `bug339297_as_clip` atomic draw calls | 8 / 9 | `BUG-MIX`, `IC` | Remove two tessellation draws; normalized Rust target is 7 because C++ counts initialize. |
| 14 | `bug339297` MSAA upload bytes | 2,560 / 2,848 | `UPLOAD-DUP` | Geometry is exact; share duplicate slices; target 2,560. |
| 15 | `bug339297_as_clip` MSAA upload bytes | 2,816 / 3,120 | `UPLOAD-DUP` | Item 128 closed structure; remove the remaining 304 bytes. |
| 16 | `bug339297_as_clip` atomic upload bytes | 13,324 / 14,732 | `BUG-MIX`, then `UPLOAD-DUP` | Consolidate batches, then classify any residual; target 13,324. |
| 17 | `batchedconvexpaths` atomic upload bytes | 7,616 / 8,216 | `UPLOAD-DUP` | Reuse tessellator resources in `AtomicPipeline`; target 7,616. |
| 18 | `bug339297` atomic upload bytes | 12,996 / 13,468 | `BUG-MIX`, then `UPLOAD-DUP` | Remove repeated mixed-batch payload/padding; target 12,996. |
| 19 | `OverStroke` atomic tessellation spans | 490 / 506 | `OVER-AENV` | Reuse item 128's logical multi-row relocation; target 490. |
| 20 | `bug339297_as_clip` atomic tessellation spans | 121 / 124 | `BUG-MIX` | One mixed flush envelope; target 121. |
| 21 | `OverStroke` atomic draw instances | 989 / 1,005 | `OVER-AENV`, `OVER-PATCH`, `IC` | Remove 16 spans and one patch; normalized Rust target 988. |
| 22 | `bug339297` atomic tessellation spans | 117 / 118 | `BUG-MIX` | Remove the second local envelope; target 117. |
| 23 | `bug339297_as_clip` atomic draw instances | 556 / 558 | `BUG-MIX`, `IC` | Remove three span instances; normalized Rust target 555. |
| 24 | `OverStroke` atomic path patches | 497 / 498 | `OVER-PATCH` | Preparation oracle locates the extra contour/close/join record; target 497. |
| 25 | `OverStroke` MSAA path patches | 497 / 498 | `OVER-PATCH` | Same shared source correction; target 497. |
| 26 | `OverStroke` MSAA draw instances | 986 / 987 | `OVER-PATCH` | Falls with the extra path patch; target 986. |

## Ordered Cluster Queue

1. `BUG-MIX` owns 10 rows. Port C++'s single mixed midpoint/outer
   logical-flush tessellation allocation and pass for the two `bug339297`
   atomic variants.
2. `OVER-AENV` owns two primary rows and part of one upload row. Reuse item
   128's multi-row logical relocation in atomic packing. The deterministic
   first-order upload reduction is 16 spans times 64 bytes, or 1,024 bytes.
3. `UPLOAD-DUP` owns 11 primary rows plus residual bytes from the first two
   clusters. Make tessellator-created uniform/path/contour resources reusable
   by both path pipelines while preserving true written-byte accounting.
4. `OVER-PATCH` owns three rows. Add one focused C++/Rust per-draw preparation
   oracle, find the 498th Rust patch, and correct the shared stroke source.

## Execution And Stop Rules

- Attribute clusters in parallel, but keep renderer implementation and
  acceptance serial.
- Regenerate the full counter report after each cluster, not after every row.
- Deterministic counter and pixel changes need one directional timing snapshot,
  not an A-B-B-A campaign. A-B-B-A is reserved for timing-defined acceptance.
- Use existing final-pixel references for all four clusters. Capture a new
  artifact only for the `OVER-PATCH` preparation oracle.
- A cluster closes when its named rows disappear or a narrower residual is
  reclassified here with source evidence. R4 counter-tail work ends when this
  finite table is empty; unrelated optimizations do not enter this audit.
