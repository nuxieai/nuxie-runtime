# `lua_gpu.cpp` paired audit

Upstream owner: `src/lua/renderer/lua_gpu.cpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

Rust owners:

- `crates/nuxie-scripting/src/gpu_canvas.rs` owns the GPU-prefixed userdata,
  resource identities, retained submission snapshots, and explicit pass
  lifecycle under D18.
- `crates/nuxie-scripting/src/vm/lua_canvas.rs` owns the mixed-file 2D Canvas
  userdata and frame lifecycle.
- `crates/nuxie-scripting/src/vm/lua_image.rs` owns the mixed-file
  `Image:view` cache and renderer validation request.
- `crates/nuxie-render-api/src/lib.rs` owns the Rust renderer adaptation seams
  for RenderCanvas frames and renderer-backed sampled-image occurrences.
- `crates/nuxie-renderer/src/exact_source_adapter.rs` and the native backend
  factories own the exact translated RenderContext/RenderCanvas execution.
- `crates/nuxie/src/ore_metal_gpu_canvas.rs` owns the authenticated native
  product execution currently covered by the D18 GPU-prefixed ceiling.

Verdict: ported and structurally adapted under D18/X3, with behavior pending
the campaign-wide verification phase.

row_id: "B6-0280"; upstream: "src/lua/renderer/lua_gpu.cpp"; verdict: ADAPTED;

The earlier audit examined only the GPU-prefixed class block. That was a
source-correspondence failure: the same C++ file also owns the 2D
`ScriptedCanvas` implementation and `Image:view`, but the manifest marked the
whole file partial and left both sections behind. This pass audited the entire
pinned owner and made those mixed responsibilities explicit instead of
treating the filename as one subsystem.

Canvas 2D now preserves the pinned behavior: optional or zero dimensions are
deferred; resize-to-zero drops both canvas and image; nonzero replacement is
allocated before the previous backing is released; the image userdata remains
stable until a successful replacement; width and height reflect the backing;
beginFrame is drawing-phase-only, rejects nested frames and deferred canvases,
accepts the optional clear color, and returns a retained non-owning Renderer;
endFrame invalidates that Renderer before it consumes the frame; and dropping
an un-ended Canvas invalidates the Renderer without submitting it. The exact
renderer path begins against the Canvas dimensions, creates the source command
buffer, flushes to the Canvas render target with that external command buffer,
and commits in pinned order.

`Image:view` now requires an active renderer context, rejects a non-Rive or
foreign-domain image, verifies that the exact source texture exists, caches
one renderer-backed GPU texture occurrence, and returns texture-view userdata
that retains the original RenderImage. The GPU-canvas plan carries that owner
directly; there is no CPU pixel decode or substitute texture. The exact WebGL
backend contains the pinned canvas-import mirror implementation; the later
campaign-wide behavioral phase must prove the adapted binding reaches it
before this row can be behaviorally promoted.

Evidence:

- The Canvas owner test proves deferred resize, allocate-before-replace,
  stable image identity, drawing-phase and frame guards, clear color,
  save-stack cleanup, renderer invalidation, submission order, the distinct
  constructor diagnostic, and the pinned non-table beginFrame descriptor
  behavior.
- The Image owner test proves renderer-context validation, cached resource
  identity, dimensions and format, and retained exact-image ownership in the
  sampled binding.
- Metal, Vulkan, and WebGPU feature builds compile the restored RenderCanvas
  seam. WebGL2 is checked only with `wasm32-unknown-unknown`; local validation
  remains unavailable when that Rust target is not installed and is not
  replaced with Emscripten.
- Port-manifest and source-correspondence gates bind this audit to the pinned
  complete C++ owner. Behavioral promotion remains reserved for the later
  campaign-wide verification phase.
