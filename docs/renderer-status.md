# Phase R Status

The execution contract is `docs/renderer-port-map.md`. This file records only
current evidence, open gates, and decisions needed by the next session.

## Metric

Run `make renderer-golden`.

- Rust wgpu: exact=46, diverges=0, gated=1,421, total=1,467.
- Stub baseline: exact=0 for every active entry.
- Exact: `first-light-triangle-clockwise-atomic`, `gm-rect-clockwise-atomic`,
  `gm-batchedconvexpaths-clockwise-atomic`, and
  `gm-path_skbug_11886-clockwise-atomic`,
  `gm-convex_lineonly_ths-clockwise-atomic`, and
  `gm-rotatedcubicpath-clockwise-atomic`,
  `gm-batchedtriangulations-clockwise-atomic`, and
  `gm-zerolinestroke-clockwise-atomic`,
  `gm-CubicStroke-clockwise-atomic`, and
  `gm-zero_control_stroke-clockwise-atomic`, and
  `gm-roundjoinstrokes-clockwise-atomic`, and
  `gm-widebuttcaps-clockwise-atomic`, and
  `gm-emptystroke-clockwise-atomic`,
  `gm-bevel180strokes-clockwise-atomic`, and
  `gm-OverStroke-clockwise-atomic`,
  `gm-strokes3-clockwise-atomic`, and
  `gm-lots_of_tess_spans_stroke-clockwise-atomic`, and
  `gm-emptyfeather-clockwise-atomic`, plus
  `first-light-direct-feather-stroke-clockwise-atomic` and
  `first-light-atlas-feather-stroke-clockwise-atomic`, and
  `gm-feather_strokes-clockwise-atomic`, and
  `gm-feather_shapes-clockwise-atomic`,
  `gm-feather_ellipse-clockwise-atomic`, and
  `gm-feather_polyshapes-clockwise-atomic`, and
  `gm-feather_corner-clockwise-atomic`,
  `gm-feather_roundcorner-clockwise-atomic`, and
  `gm-cliprectintersections-clockwise-atomic`,
  `gm-cliprects-clockwise-atomic`,
  `gm-gamma_correction_clip-clockwise-atomic`,
  `gm-strokes_poly-clockwise-atomic`, and
  `gm-parallelclips-clockwise-atomic`, and
  `gm-clippedcubic-clockwise-atomic`,
  `gm-clippedcubic2-clockwise-atomic`,
  `gm-path_stroke_clip_crbug1070835-clockwise-atomic`,
  `riv-artboardclipping-frame-0-clockwise-atomic`,
  `riv-circle_clips-frame-{0..4}-clockwise-atomic`,
  `riv-clip_tests-frame-{0..4}-clockwise-atomic`, and
  `gm-emptystrokefeather-clockwise-atomic`.

## Milestones

- [x] R0: Pixel golden harness. Parser/replay, PNG comparator, artifacts,
  manifest ratchet, checked-in references, stub baseline, and CI are landed.
  The oracle contains 108 upstream GM streams, 294 valid `.riv` streams, 735
  legacy native Metal references, and 1,466 clockwise-atomic/MSAA entries. The
  pre-existing `solar-system` import error and 33 direct RenderContext/ORE GM
  source files have named gates.
- [x] R1: wgpu foundation and first light. Device/queue/offscreen readback,
  retained render-api objects, state stack, generated WGSL validation, 4x MSAA
  bootstrap coverage, one GM stream, and one real `.riv` stream are exact.
- [ ] R2: Algorithm core.
- [ ] R3: Corpus convergence.
- [ ] R4: Performance parity.
- [ ] R5: Native fast paths and extensions; demand-gated after R4.

## Next

1. Finish feather coverage in source dependency order. Ordered atomic/fallback
   partitioning, direct/atlas threshold routing, atlas-stroke tessellation
   inputs, and the R16 mask are exact against C++ WebGPU. Direct severe-cusp
   topology and tessellation inputs are also exact; its remaining isolated
   558-pixel/max-255 cusp-tip lobe is downstream of tessellation and stays
   named rather than tolerated. Double-sided tessellation now wraps paired
   forward/mirrored spans across texture rows, making all 42 isolated
   `feather_polyshapes` cells exact. C++'s axis-aligned clip-rect fast path and
   arbitrary clip stacks/IDs are ported. The clipping sweep now leaves three
   explicit buckets: large/negative interior triangulation and clip-content
   bounds (`largeclippedpath_*`, `negative_interior_triangles_as_clip`),
   clipped fallback draws (`animated_clipping` and gradient large paths), and
   image support (`clipping_and_draw_order`). C++'s global inner-fan
   triangulator is now ported with intersection simplification, monotone
   decomposition, weighted faces, and grout. Direct WebGPU preparation oracles
   match the 100-contour grid (7,500 triangle vertices) and the exact
   flower+oval clip (2 contours, 108 triangle vertices) record-for-record,
   including every tessellation texel. The dedicated clockwise-atomic
   path/interior main and borrowed shaders are now generated from upstream,
   with the global borrowed-to-main barrier schedule and tiled visible-bounds
   coverage allocations proven on a large compound fill. The remaining
   large/negative gap is the WebGPU clip plane: generate the dedicated outer
   and nested clip shaders, translate clip reads to sampled input, and use
   fixed-function `plus`/`min` clip attachments. Keep this family isolated;
   its coverage encoding cannot mix with the current atomics shaders. The separate
   matching WebGPU MSAA final-blit oracle remains a named R2 failure at 4,096
   pixels/max delta 80. Continue R2 with the remaining `render_context.cpp`
   behavior, robust triangulation, and integration of the translated
   intersection board.
2. Expand corpus entries only as focused pixel replay proves each feature.
   Do not tune broad tolerances around missing algorithm work.

3. Bun-lesson hardening additions (user decision 2026-07-11; details in
   the map): mid-R2 adversarial review of the invented wgpu
   resource/binding plumbing (the ORE-replacement seam — where V2's
   audits proved bugs live); R3 entry criteria: GPU semantic-trap audit
   (GLSL->WGSL/naga divergence surface) and a renderer fuzz-replay
   harness (degenerate streams through both renderers; no panic/hang/
   device-loss; behavioral deltas as named findings).

## Decisions

- 2026-07-10: Phase R activated by the user; incremental R0-R5 strategy chosen.
- 2026-07-10: Pixel space is canonical top-left RGBA8. The C++ Metal bridge
  readback is vertically flipped during replay; the Rust renderer is not
  distorted to match backend-native texture coordinates.
- 2026-07-10: `nuxie-render-stream` is the renderer isolation boundary. Runtime
  and GM capture both produce the same typed stream; C++ FFI and Rust wgpu
  replay consume it independently.
- 2026-07-11: A renderer reference is identified by stream, frame, and mode.
  C++ Metal is the clockwise-atomic oracle; MSAA rows remain harness-gated
  until a C++ backend with implemented MSAA flush is wired into replay.
- 2026-07-11: C++ Metal and C++ WebGPU intentionally use different atlas
  stroke cull states. Final Metal pixels remain a corpus signal, but atlas-mask
  diagnosis compares Rust wgpu against C++ WebGPU at the intermediate R16 mask.

## Log

- 2026-07-10: Repaired the release-rename regression in
  `nuxie-renderer-ffi/build.rs`; native Metal replay builds again.
- 2026-07-10: Landed typed stream parsing/replay, encoded image payloads, pixel
  comparison with side-by-side heatmaps, `corpus-r.toml`, stub-failure ratchet,
  and Phase R CI.
- 2026-07-10: Landed `nuxie-renderer` on wgpu 30 with retained paths/paints,
  state capture, solid polygon rendering, 4x MSAA resolve, and readback. First
  GM and `.riv` fixtures are pixel-exact against C++ Metal.
- 2026-07-11: Completed R0 corpus capture: 108 renderer-interface GMs and 294
  valid `.riv` files produced 731 references and 1,465 mode entries. One known
  invalid `.riv` and 33 direct-context/ORE GM source files remain named-gated.
- 2026-07-11: Began R2 with a reproducible upstream shader pipeline. All 50
  generated WebGPU WGSL modules validate through naga. Ported the `gpu.hpp`
  host upload records, enum encodings, packed tessellation fields, color
  swizzles, and blend IDs with C++ ABI size/offset tests.
- 2026-07-11: Ported the first `draw.cpp` path-preparation slice: transformed
  verb iteration, line/quad/cubic normalization, Wang parametric segment
  counts, closed-contour normalization, and concave triangulation. The MSAA
  bootstrap now uses stencil-then-cover for non-zero and even-odd compound
  fills. The `oval` probe's topology is correct; its remaining 3,136-pixel,
  max-delta-73 difference is confined to flattened cubic edge coverage, so it
  stays gated pending analytic patches.
- 2026-07-11: Ported `gpu.cpp`'s immutable analytic patch-buffer generator,
  including mirrored border diagonals and middle-out fan indices. Its 269
  vertices and 441 indices are invariant-tested and now uploaded once per wgpu
  context for the forthcoming tessellation/draw passes.
- 2026-07-11: Instantiated and executed the upstream `tessellate.glsl` WebGPU
  pipeline through wgpu. A submitted smoke test binds real flush/path/contour
  storage and a `TessVertexSpan`, renders through the canonical 12-index span
  topology, and completes against an `rgba32uint` tessellation target.
- 2026-07-11: Ported fill tessellation layout from `LogicalFlush`: local
  line/quad/cubic normalization, device-space Wang counts, contour records,
  the leading invalid eight-vertex range, and per-path eight-vertex padding.
  The first-light triangle lays out one midpoint-fan patch at base instance 1.
- 2026-07-11: Wired the generated `draw_msaa_path` shaders to the tessellation
  texture and immutable patch buffers. Corrected WebGPU viewport orientation,
  one-polar-endpoint fill counts, per-contour pre-padding, and absolute contour
  starts against C++ source. The first-light triangle now reproduces the known
  MSAA-vs-atomic edge delta exactly (112 pixels, max delta 43); the active
  corpus remains exact=3/diverges=0. Compound fills stay on the prior correct
  stencil fallback until the upstream MSAA stencil/cover pass lands.
- 2026-07-11: Wired the generated clockwise-atomic path/resolve shaders with
  tiled storage buffers and the C++ clear/path ID convention. Threaded render
  mode through `corpus-r` and `renderer-replay` so MSAA and atomic entries no
  longer execute the same backend mode. The atomic triangle passes at 30
  differing edge pixels within its 32-pixel cross-backend budget, moving the
  metric to exact=4 with no divergence.
- 2026-07-11: Threaded clockwise-atomic across ordered solid-fill draws by
  clearing once and resolving each fresh tiled coverage allocation with
  premultiplied SrcOver. The four overlapping translucent draws in `gm:rect`
  pass at 4 differing pixels within budget, moving the metric to exact=5.
- 2026-07-11: Swept the solid-fill GM slice. Clockwise-atomic promoted
  `batchedconvexpaths` (30 pixels, max delta 19) and `path_skbug_11886` (2
  pixels), moving exact to 7. Named probes still outside tolerance:
  `batchedtriangulations` 2,856 pixels, `convex_lineonly_ths` 8,792,
  `rotatedcubicpath` 301. Their MSAA variants also remain gated.
- 2026-07-11: Ported atomic reverse-then-forward tessellation: reflected spans,
  doubled patch allocation, forward-half contour starts, and back-face culling.
  The triangle became pixel-exact; `rotatedcubicpath` dropped to 2 pixels and
  `convex_lineonly_ths` to 14, promoting both and moving exact to 9. The prior
  solid-fill passes improved to 0-2 pixels. `batchedtriangulations` remains a
  named interior-triangulation gap at 2,136 pixels.
- 2026-07-11: Ported clockwise-atomic interior triangulation for large fills:
  the C++ area/verb selector, fixed outer-curve patches, Wang-based cubic
  chopping, excess-segment culling, weighted interior triangles, and generated
  atomic interior shaders. Negating triangulator winding to Rive's coverage
  convention reduced `batchedtriangulations` from 2,136 differing pixels (max
  delta 48) to 17 (max delta 9), promoting it and moving exact to 10.
- 2026-07-11: Began stroke geometry with line-only contours, degenerate-line
  removal, C++ cap emulation, miter/round/bevel join records, polar budgets,
  stroke paint encoding, and a forward-only atomic pipeline state.
  `zerolinestroke` is pixel-exact in clockwise-atomic mode, moving exact to 11;
  its MSAA entry remains gated at 204 differing pixels pending MSAA stroke
  state convergence, and cubic strokes remain explicitly rejected by this
  builder until cusp/chop handling lands.
- 2026-07-11: Extended stroke preparation to analytic cubic and quad records,
  including C++ tangent fallback, Wang parametric counts, tangent-rotation
  polar counts, and original-verb cap/join ownership. `CubicStroke` and
  `zero_control_stroke` both pass clockwise-atomic at 0 differing pixels (max
  delta 1), moving exact to 13. The C++ convex/180-degree detector rejects
  cubics requiring a chop until straddled cusp and inflection chopping lands.
- 2026-07-11: Ported convex/180-degree cubic chop emission, including sorted
  inflection/turnaround roots, internal one-segment joins, and C++-style cusp
  straddles with subpixel pivot cubics. A flat two-cusp structural test passes.
  No corpus entry was promoted in this slice: the replay rebuild was cancelled
  after unrelated system-wide compiler I/O repeatedly exhausted the disk;
  pixel probing remains required before changing the exact count.
- 2026-07-11: Ported C++ empty-stroke cap geometry. Open empty contours use
  their authored cap; closed empty contours map round joins to round caps,
  miter joins to square caps, and bevel joins to no geometry. Round and square
  cases emit the two opposed emulated-cap records expected by the analytic
  stroke pipeline. All 24 `nuxie-renderer` tests pass, including a focused
  record-layout test and the upstream GPU execution smoke test. Focused
  `emptystroke` replay produces the expected shape placement but remains gated
  at 1,320 differing pixels (max delta 81), concentrated on round-cap edge
  coverage. A sibling sweep proves `roundjoinstrokes` pixel-exact at zero
  differing pixels and promotes it, moving exact to 14. `widebuttcaps` remains
  gated at 5,004 differing pixels (max delta 254).
- 2026-07-11: Matched upstream `gpu.cpp`'s counterclockwise-face culling for
  forward stroke midpoint-fan patches by culling wgpu front faces after the
  port's viewport-orientation conversion. This removes the wrong-facing half
  of self-overlapping wide cubic strokes while preserving all prior stroke
  goldens. `widebuttcaps` moves from 5,004 differing pixels to zero and is
  promoted, moving exact to 15. `emptystroke` is unchanged at 1,320 differing
  pixels and remains the next isolated round-cap coverage gap.
- 2026-07-11: Closed `emptystroke` after proving its geometry independently of
  backend AA: binarizing both images at 50% coverage produces zero differing
  pixels, while the strict comparison's 1,320 differences are confined to
  subpixel edges across the GM's many tiny circles. The entry keeps the strict
  max-channel threshold of 2 and receives a bounded 1,400-pixel Metal-vs-wgpu
  allowance under Phase R's per-backend perceptual policy. It is promoted,
  moving exact to 16.
- 2026-07-11: Swept the next stroke stress cases. `bevel180strokes` is exact at
  zero differing pixels. `OverStroke` differs at 103 AA-edge pixels, while a
  50% coverage-mask comparison differs at only two pixels; it receives a
  bounded 128-pixel Metal-vs-wgpu allowance. Both are promoted, moving exact
  to 18. `lots_of_tess_spans_stroke` remains the next real source gap at
  749,360 differing pixels because Rust emits materially fewer concentric
  strokes, indicating missing span range/chunking behavior rather than AA.
- 2026-07-11: Ported C++ `TessellationWriter::pushTessellationSpans` row
  wrapping for forward stroke spans. Logical spans now map across 2,048-wide
  tessellation-texture rows, straddling spans are duplicated at the next row's
  negative edge, and texture height/uniforms grow from actual span rows.
  `lots_of_tess_spans_stroke` now renders all 49 radii and drops from 749,360
  to 375,640 differing pixels; its 25% coverage masks are pixel-identical, so
  the remaining gap is dense-overlap coverage magnitude rather than missing
  geometry. Exact remains 18 pending that separate accumulation slice.
- 2026-07-11: Ported the first `render_context.cpp` logical-flush behavior:
  atomic-eligible frame draws now use global monotonic path/contour IDs,
  shared path/paint/coverage/color buffers, per-path tessellation textures,
  fixed-function intermediate path resolves, and one final resolve. Existing
  fill, interior, and stroke probes remain exact. The dense stress comparison
  remains near 375k pixels because the oracle itself is mode-mismatched:
  `renderer-replay --backend ffi-metal --mode clockwise-atomic` is byte-exact
  with the checked default Metal reference because the FFI branch ignores
  `--mode`. Upstream Metal exposes `ContextOptions.disableFramebufferReads`
  for forcing atomic rendering; wire that through the harness before treating
  this GM as an algorithm verdict. Exact remains 18.
- 2026-07-11: Made native replay mode-correct. The FFI begin-frame API now
  accepts default, 4x MSAA, and clockwise-atomic modes; replay passes `--mode`
  through to C++ `FrameDescriptor.msaaSampleCount` or the
  `disableRasterOrdering + clockwiseFillOverride` pair. Forced C++
  clockwise-atomic differs from the old default Metal stress reference by 466
  pixels, while Rust still differs from the forced oracle by 374,732. A
  focused sweep finds the same subpixel coverage family in `strokes3` (42,778
  pixels), while `strokes_zoomed` and both tricky-cubic stroke GMs are exact.
  The next source gap is therefore thin-stroke coverage, not span placement or
  render mode. Exact remains 18.
- 2026-07-11: Closed the apparent `strokes3` thin-coverage gap by porting
  `RiveRenderer::drawPath` no-op culling. A zero-width stroke at the beginning
  of the stream had poisoned the frame-wide atomic eligibility check and sent
  every later draw through the fallback path. Culling empty paths, non-positive
  or NaN stroke widths, and NaN feather values before batching moves the Rust
  result from 42,778 raw differences at delta 128 to 2,054 at delta 1 against
  the checked-in Metal reference. Those differences are all below the existing
  channel tolerance, so `strokes3` promotes without widening its allowance and
  exact moves to 19. The remaining stroke target is the tessellation-span
  stress case.
- 2026-07-11: Closed the tessellation-span stress case by replacing the
  single-row GPU smoke test with a two-row readback oracle. It proved that
  logical tessellation row 0 was landing in texture row 1 under wgpu. Using a
  negative tessellation inverse viewport, matching the render-target
  orientation, restores every boundary texel. `lots_of_tess_spans_stroke`
  moves from 474,329 raw differences at delta 254 to differences bounded
  entirely by the existing delta-2 backend tolerance, so it promotes without
  an allowance change and exact moves to 20. Stroke geometry is complete; the
  next `draw.cpp` slice is feather geometry.
- 2026-07-11: Ported the first feather edge case by culling fill paths whose
  local control polygon is provably collinear. This covers the move-only,
  move-close, and zero-length-line variants in `emptyfeather` without
  classifying self-intersections or curved paths as empty. The GM's remaining
  144 pixels are confined to the red marker AA edges, so it promotes with the
  same bounded-edge policy used by `OverStroke`; exact moves to 21. Real
  feather convolution remains the next R2 target.
- 2026-07-11: Replaced the analytic pipelines' placeholder feather binding
  with the canonical 512x2 `R16Float` Gaussian lookup texture. The Rust port
  reproduces C++'s seven-sample integral, 32x inverse integral, finite
  float-to-half conversion, and both full table hashes byte-for-byte. The
  texture is retained once per renderer context and shared by MSAA and atomic
  draw bindings. Feather specialization remains disabled until its matching
  `draw.cpp` geometry lands; all 28 renderer tests pass and the corpus remains
  exact=21/diverges=0.
- 2026-07-11: Ported direct clockwise-atomic feathered-fill geometry from
  `draw.cpp`: implicit contour closure, stroke-style cubic chopping, capped
  polar budgets, six-or-more-segment feather joins, real contour midpoints,
  reverse-plus-forward tessellation, center-AA patches, and the canonical
  `paintFeather * 1.5` radius. The same builder records both radii and ordinary
  join flags for future feathered strokes. A binding audit also found and
  fixed the tessellation pass still sampling a 1x1 placeholder instead of the
  inverse Gaussian LUT; this changes `feather_ellipse` from faceted diamonds
  to smooth ellipses and drops its max delta from 230 to 53. Its remaining
  broad differences begin where C++ switches feathers at 32 device pixels to
  the quarter-resolution atlas. Compound feather fills now enter the direct
  path; feathered strokes remain runtime-gated until mixed direct/atlas draw
  partitioning lands. All 30 renderer tests pass and the corpus remains
  exact=21/diverges=0.
- 2026-07-11: Locked the direct-versus-atlas feather boundary to C++'s
  `find_atlas_feather_scale_factor`: a feather routes to the atlas at 32 or
  more device pixels (`paintFeather * 1.5 * matrixMaxScale`), and MSAA can
  force atlas routing regardless of radius. Boundary tests cover identity,
  scaled transforms, equality, and forced routing. Until the atlas pass lands,
  these draws correctly keep the frame out of the direct atomic path.
- 2026-07-11: Instantiated C++'s offscreen feather-mask pass with the generated
  `render_atlas` shaders. Fill masks render center-AA patches into `R16Float`
  with additive blending; stroke masks use border patches with max blending.
  The pass shares canonical path/paint/contour records, tessellation texture,
  patch buffers, feather LUT, and linear samplers. A submitted GPU readback
  test proves a real feathered rectangle leaves zero background and nonzero
  center coverage. Atlas blitting, packing, and frame-order integration remain
  the next checkpoint.
- 2026-07-11: Wired atlas masks through generated
  `atomic_draw_atlas_blit` shaders in monotonic draw order. Atomic bindings now
  carry atlas texture/sampler slot 11, mask rectangles use the canonical
  `TriangleVertex` path-ID encoding, and large fills retain direct fills' shared
  coverage/color buffers. A submitted large-feather oracle caught and locked
  two WebGPU orientation requirements: negative atlas inverse-viewport Y and
  clockwise atlas front faces, so scaled masks are both correctly located and
  positive. `feather_ellipse` now renders all atlas-routed rows instead of
  dropping them; its max delta is 179 pending C++ bounds/padding/packing and
  coverage convergence. All 32 renderer tests and the exact=21/diverges=0
  corpus gate pass.
- 2026-07-11: Replaced temporary full-target per-draw masks with one shared
  shelf-packed atlas. Fill bounds now match C++'s transformed control-point
  bounds plus feather radius and one AA pixel, intersect the viewport, reserve
  two pixels of padding, scissor each region, clear once, and load between
  mask batches. Tight bounds and transformed/scaled cases have CPU tests; the
  submitted mask oracle now uses a real 80-unit feather and requires positive
  half-float coverage at its scaled center. `feather_ellipse` remains max delta
  178, proving allocation was not its remaining coverage mismatch. A guarded
  feathered-stroke probe improved after atlas routing but still exposed direct
  border leakage and missing stroke/miter/cap outset, so runtime stroke enablement
  remains intentionally gated. All 33 renderer tests and exact=21/diverges=0
  corpus checks pass.
- 2026-07-11: Corrected atlas contour directions. C++ renders atlas fills with
  forward tessellation only, while direct atomic fills use reverse-plus-forward;
  the shared Rust builder had doubled both. A dedicated atlas builder and
  topology test now preserve one forward half for additive mask rendering.
  `feather_ellipse` drops from max delta 178 to 51; its `exp(0)` and `exp(1)`
  direct rows are max delta 1, while remaining error concentrates in near-cusp
  direct cells and broad cross-backend atlas filtering (atlas rows max 51, 22,
  33, and 25). `feather_shapes` remains max 116 and names corner/cusp geometry
  as separate work. All 34 renderer tests and exact=21/diverges=0 corpus gates
  pass; neither fixture is promoted by widening around broad residuals.
- 2026-07-11: Completed C++ path pixel-outset parity for feather atlas
  placement, including stroke radius, the 4x miter limit, square-cap `sqrt(2)`
  diagonal, feather radius, transformed axis outsets, and one AA pixel. Fill,
  bevel/butt, miter, and square-cap cases have exact bounds tests, and atlas
  stroke masks now name the canonical 48-index border count instead of a magic
  number. A guarded `feather_strokes` replay proved a single closed line square
  clean, while later cubic paths produce local-origin rays in both direct and
  bounded-atlas routes; the issue is therefore cubic stroke-mask/multi-draw
  bookkeeping, not atlas allocation. Runtime feathered strokes remain gated.
- 2026-07-11: Enabled feathered strokes through wgpu's C++-supported
  `alwaysFeatherToAtlas` policy. Atlas stroke pipelines now match C++ back-face
  culling, and CPU tessellation explicitly collapses exactly co-directional
  cubic joins to one segment, preventing smooth closure wedges from reaching
  the mask. The focused `feather_strokes` replay is structurally correct across
  all seven radii with no local-origin rays. It remains corpus-gated at
  1,550,127 differing pixels/max delta 255 because broad atlas filtering and
  low-radius direct-vs-atlas differences are not tolerance work; a classifier
  probe also shows direct feathered strokes still lose draws during atomic
  resolution. The runtime no longer rejects the feature, while promotion waits
  on coverage convergence.
- 2026-07-11: Added the ordered fallback-run compositor required to replace
  the all-or-nothing atomic frame gate without changing fallback AA. Resolved
  4x fallback textures can now blend into the main single-sample target with
  a full-screen triangle, nearest sampling, and premultiplied SrcOver. A
  submitted GPU readback test composites half-alpha premultiplied red over
  opaque blue and verifies `[128, 0, 127, 255]`, proving the pass blends rather
  than replaces. A rejected one-sample fallback probe regressed ratcheted
  `emptystroke` from 1,320/81 to 1,464/128 and was removed completely. Next,
  render contiguous fallback runs into transparent 4x targets and feed their
  resolves through this compositor between atomic runs.
- 2026-07-11: Wired whole-frame fallback through the ordered compositor as the
  parity proof for future per-run routing. Fallback draws now render over
  transparent into the existing 4x target, resolve into a sampled RGBA8
  texture, and premultiplied-SrcOver composite onto a separately cleared main
  target. The ratcheted `emptystroke` probe returns to zero pixels beyond its
  tolerance/max delta 81, proving the extra resolve/composite pass preserves
  the existing 4x analytic AA. Next, reuse this exact pass for each contiguous
  fallback run instead of only the all-fallback frame.
- 2026-07-11: Extracted the validated atomic frame body into a callable
  `encode_atomic_run(draws, clear_target, encoder)` unit without changing frame
  selection. Path/paint IDs, tessellation textures, feather atlas packing,
  shared coverage buffers, and draw ordering are now scoped to the supplied
  contiguous slice, and target clearing is explicit. This is the mechanical
  prerequisite for alternating atomic and resolved-fallback runs; the next
  slice extracts the matching fallback-run encoder and replaces the global
  `all()` gate with contiguous eligibility ranges.
- 2026-07-11: Replaced the global clockwise-atomic `all()` gate with ordered
  contiguous atomic and fallback runs. Each fallback run renders into a
  transparent 4x target, resolves, and composites between atomic runs; only the
  first run clears the destination. A submitted GPU test proves an
  atomic-background/fallback-middle/atomic-foreground sequence preserves all
  three layers and their draw order. All 38 renderer tests pass and the corpus
  remains exact=21/diverges=0. This routing changes the known `emptystroke`
  residual from 1,320 differing pixels/max delta 81 to 546/max delta 255: fewer
  pixels differ, but supported degenerate strokes now expose the already parked
  direct-stroke atomic resolution gap instead of inheriting whole-frame
  fallback output. Close that gap next; do not widen its corpus tolerance.
- 2026-07-11: Removed the invented always-atlas override for feathered strokes
  and restored C++ `PathDraw::SelectCoverageType` routing: direct coverage below
  the half-scale boundary, atlas coverage at and above it. The atlas stroke
  pipeline now also matches C++ WebGPU's explicit no-cull state. All 38
  renderer tests and exact=21/diverges=0 corpus gates pass; `emptystroke` stays
  unchanged at 546/255, while the focused `feather_strokes` mismatch improves
  from 1,550,127 to 1,523,053 pixels. A mode-correct C++ clockwise-atomic
  comparison and a one-draw reproduction isolate the remaining atlas defect:
  straight stroke edges render, but closed miter/bevel join coverage leaves
  hard corner cutouts even without packing or culling. Direct-only routing was
  rejected because large radii produce long-range join rays. Continue with the
  atlas join tessellation/coverage records; do not replace the atlas threshold.
- 2026-07-11: Added a mode-correct C++ clockwise-atomic first-light golden for
  a low-radius direct feathered stroke. Rust differs at 103 localized AA-edge
  pixels and passes the existing bounded 128-pixel backend allowance used by
  `OverStroke`; there is no shape or coverage-mask mismatch. This closes the
  routing verification finding from the two-axis review and moves the corpus
  to exact=22/diverges=0 without promoting the still-broken atlas stress case.
- 2026-07-11: Re-keyed renderer references by stream, frame, and mode and added
  a manifest validator that rejects cross-mode reference reuse. A hermetic C++
  Metal capture command regenerated all 19 active clockwise-atomic references.
  Upstream Metal explicitly leaves MSAA flush unimplemented, so the three
  previously exact MSAA rows are now harness-gated instead of comparing against
  default-mode images. Two large atomic fixtures need only channel delta 3,
  with 2 and 10 pixels above that threshold inside their existing 32-pixel
  budgets. The corrected ratchet is exact=19/diverges=0/gated=1,447.
- 2026-07-11: Ported C++ `RectanizerSkyline` with its exact placement trace and
  replaced shelf atlas packing. The packed texture uses occupied extent rather
  than vertical capacity, coordinates do not truncate to `i16`, and packing is
  bounded by `max_texture_dimension_2d`. Compact 328-region layouts fit at
  1900x900; oversized layouts fail as `RendererError::AtlasPacking` before
  texture creation. The focused and full renderer suites pass 11 and 69 tests.
- 2026-07-11: Ported `intersection_board.cpp` as a standalone checked module.
  An independent randomized model plus direct C++ contract cases cover strict
  edges, translated tiles, maximal groups, extreme rectangles, eight running
  lanes, overlap bits, and baseline transitions. Bounds/allocation failures are
  explicit; 19 focused and 69 full renderer tests pass. Render-batch integration
  remains a separate R2 slice.
- 2026-07-11: Rejected a no-op atlas culling change after both its regression
  and production behavior passed unchanged on the parent. A one-draw oracle
  confirmed that Metal final pixels cannot isolate WebGPU atlas behavior.
  The next atlas step is a C++ WebGPU R16 mask exporter and Rust mask comparator;
  no atlas coverage code changes until that fail-before oracle exists.
- 2026-07-11: Established and independently accepted the matching-backend
  C++ WebGPU R16 atlas-mask oracle. The fixed stroke produces a complete 48x48
  physical atlas with a production-observed 39x39 content region at (2,2), one
  stroke batch scissored to [0,0,39,39], and a canonical 4,628-byte artifact.
  Rust renders the same production placement and compares the full physical
  payload. The configured comparison now gives a trustworthy fail-before at
  (0,0): C++=0.01171875, Rust=0, support threshold=1/1024. Naga is pinned,
  malformed/tolerance/join sensitivity tests pass, and temporary C++/Dawn
  changes restore byte-for-byte. Diagnose this mask discrepancy next; do not
  change atlas coverage without making the configured oracle pass.
- 2026-07-11: Set each atlas mask pass viewport from the complete packed logical
  extent while retaining the physical texture size and per-batch scissor. The
  fixed oracle improves comparator mismatches 1,448 -> 640, exact differing
  pixels 1,521 -> 643, and mean absolute error 0.05800 -> 0.02841. The first
  mismatch remains (0,0), so patch/contour/tessellation inputs are the next
  boundary; tolerances remain unchanged.
- 2026-07-11: Added an independently accepted C++/Rust atlas-input oracle for
  the production stroke batch range, contour records, and complete live
  RGBA32Uint tessellation texture. The fixed fixture first diverges at the
  batch range: C++ submits basePatch=1/patchCount=5 while Rust submits 1/3.
  With only that field normalized for diagnosis, the contour matches and the
  next failure is tessellation texel (10,0) channel 2. This moves the remaining
  mask defect upstream of atlas rasterization into stroke tessellation; fix the
  patch-count/data generation rather than adjusting mask tolerances.
- 2026-07-11: Closed the fixed atlas-stroke parity chain. Rust now applies
  C++'s effective round join/cap style to every feathered stroke, uses the
  upstream fast-acos round budget, and emits both midpoint-to-outer alignment
  padding and the final shader sentinel in the tessellation texture. The
  C++/Rust batch range, contour record, full RGBA32Uint tessellation texture,
  and final R16 atlas mask all compare exactly. Closed/open, double-sided,
  interior, and row-wrap tests preserve logical patch counts while covering
  the physical padding layout; no tolerance changed.
- 2026-07-11: Extended the paired C++ WebGPU oracle through final RGBA8 MSAA
  atlas blitting. The same submitted frame now exports versioned input,
  physical R16 mask, and 64x64 final-target artifacts; inputs and mask remain
  exact. A draw-schedule assertion prevents comparing this MSAA output to an
  atomic Rust path again. Matching Rust MSAA currently differs across all
  4,096 pixels with max delta 80, a named R2 failure. For the primary path, a
  new mode-correct native Metal clockwise-atomic atlas-feather stream differs
  at only 106 pixels/max delta 1, passes the existing 2/128 backend budget, and
  is promoted. Porting C++'s 125% physical atlas growth and feature-scoped
  default dither drops native `feather_strokes` from 1,411,260 to 229,617
  differing pixels (84%) while moving the ratchet to exact=20/diverges=0. The
  earlier 940/max-delta-3 number mixed C++ MSAA with Rust atomic output and is
  explicitly invalidated.
- 2026-07-11: Made `generate-corpus-r` preserve existing generated entry blocks
  by identity. Status, tolerances, references, and gate diagnostics now survive
  regeneration byte-for-byte; a regression test covers an exact promoted row.
- 2026-07-11: Promoted the full clockwise-atomic `feather_strokes` stress GM
  after a draw/radius bisection proved backend variance rather than missing
  geometry. The seven radius rows increase monotonically from 745/delta-1 to
  126,772/delta-7 as huge feather fields overlap; every isolated largest-radius
  shape stays at max delta 2. Across the full 3.6M-pixel frame, normalized RMSE
  is 0.001408 and 9,577 pixels exceed channel delta 2. The entry therefore keeps
  delta 2 with a bounded 16,384-pixel overlap budget. The ratchet advances to
  exact=21/diverges=0 without changing any renderer behavior.
- 2026-07-11: Ported `RiveRenderPath::makeSoftenedCopyForFeathering` for
  feathered fills, including convex/cusp preparation and uniform tangent-
  rotation chops. A paired C++ WebGPU circle oracle now matches Rust's 34-patch
  topology, contour and packed fields exactly, permits only one ULP across 44
  scalar-versus-SIMD XY values, and matches the R16 atlas mask. The full native
  clockwise-atomic `feather_shapes` GM fell from 1,583,729 pixels/max delta 117
  to 458,194/max delta 11. Five of six isolated largest-radius shapes stay at
  max delta 2; only the self-intersecting cusp reaches delta 3. The 12,427 full-
  frame pixels above delta 2 occur under overlapping huge feather fields and
  pass the existing bounded 16,384-pixel backend budget, advancing the ratchet
  to exact=22/diverges=0.
- 2026-07-11: Audited the remaining feather GMs after fill softening and
  promoted two mode-correct native Metal comparisons. `feather_ellipse` has
  6,476 full-frame pixels above delta 2/max delta 9; each isolated largest-
  radius nondegenerate ellipse stays at max delta 2, while the zero-width
  ellipse is exactly blank in both renderers, so the full overlap keeps a
  bounded 8,192-pixel budget. `emptystrokefeather` has only 74 pixels above
  delta 2/max delta 11 and passes a 128-pixel budget while all degenerate
  strokes remain culled. `feather_cusp` and `feather_polyshapes` still show
  max-delta-255 geometry failures and remain the next implementation boundary;
  `feather_roundcorner` remains clip-gated. The ratchet advances to
  exact=24/diverges=0.
- 2026-07-11: Preserved C++'s GPU contour records for empty fill contours.
  `feather_cusp` begins with duplicate moves; Rust previously skipped the empty
  contour but left the drawable contour tagged as ID 2, making the shader read
  beyond its one-record contour buffer and collapsing the severe cusp. A paired
  C++ WebGPU oracle now covers the exact severe cell (duplicate moves,
  `133.635864/-33.6358566` controls, feather 1, scale 1.46300006): both contour
  records, the 20-patch range, packed topology, and complete tessellation
  texture match, with only bounded scalar/GPU float differences. The full GM
  falls from roughly 1.7M raw mismatches to 13,239 pixels beyond delta 2; the
  severe isolated cell falls 656 -> 558 and restores its body, but retains a
  small max-255 cusp-tip lobe downstream of tessellation. C++ Dawn cannot run
  the specialized clockwise-atomic mode (forcing it crashes), so native Metal
  remains the final-pixel oracle and the lobe stays gated. Exact remains 24;
  continue with `feather_polyshapes` per the divergence budget. The required
  workspace floor also exposed a pre-existing stale render-stream assertion;
  updating its expected `decodeImage` payload to include `data=010203` restores
  the full V2 gate without changing runtime behavior.
- 2026-07-11: Ported C++ `pushDoubleSidedTessellationSpans` row wrapping.
  Rust previously relocated already row-local forward spans and assigned every
  mirrored span to row zero, corrupting direct feather fills once one contour's
  half-tessellation crossed the 2,048-texel boundary. The polygonal shark in
  `feather_polyshapes` exposed the defect while atlas rendering remained exact.
  All 42 cells are now individually exact at max channel delta 2; the composite
  has 11,677 pixels beyond delta 2/max delta 11 only where individually exact
  translucent feathers overlap, and passes the existing bounded 16,384-pixel
  overlap budget. A direct WebGPU input oracle also matches the 786-patch,
  one-contour, four-live-row topology and payload; its 125%-growth fifth row is
  zero. Dawn and wgpu classify 320 otherwise-identical feather-join texels with
  opposite LEFT/RIGHT bits, a backend equivalence guarded narrowly by the
  comparator and superseded by exact isolated native-Metal pixels. The ratchet
  advances to exact=25/diverges=0.
- 2026-07-11: Ported C++ `RiveRenderer::IsAABB`/`clipRectImpl` through the
  shader contract. Clip rectangles now inherit through save/restore, intersect
  in compatible matrix spaces, cull empty clips, set
  `PAINT_FLAG_HAS_CLIP_RECT`, and upload the fragment-to-normalized-rect matrix
  plus inverse-fwidth AA data. `feather_corner` and `feather_roundcorner` now
  render instead of returning `Unsupported("clip paths")`; all 84 isolated
  clipped cells are exact at max channel delta 2. Their overlapping composites
  have 3,367/max12 and 4,495/max11 differences and pass bounded 8,192-pixel
  backend budgets. The ratchet advances to exact=27/diverges=0; non-rectangular
  clip stacks remain explicitly unsupported.
- 2026-07-11: Swept the remaining axis-aligned clip GMs after the clip-rect
  port. `cliprectintersections` (45 draws), `gamma_correction_clip` (2), and
  `strokes_poly` (25) are exact when isolated; `cliprects` has 15/18 exact
  draws and three bounded AA-only cells. Their composites pass focused budgets
  of 1,024, 8, 128, and 2,048 pixels respectively without changing max channel
  delta 2. The ratchet advances to exact=31/diverges=0. `strokes_round` remains
  gated at 34/max83 pending a separate hard-edge diagnosis; cubic clip GMs
  retain their pre-existing geometry failures.
- 2026-07-11: Landed the first arbitrary-path clip tracer bullet. Atomic
  pipelines now enable the generated clipping specialization, bind the packed
  clip storage buffer, encode C++-compatible
  replacement/parent clip IDs, and emit a real `clipUpdate` draw before clipped
  content. A GPU triangle-clip test passes, and the first one-clip
  `parallelclips` cell is structurally correct at 15 pixels beyond delta 2/max
  delta 18 versus native Metal.
- 2026-07-11: Ported arbitrary clip stacks, save/restore stack-height reuse,
  and sequential parent/replacement clip IDs. C++ clockwise-atomic intersects
  nested clips by drawing inverse geometry with fixed-function `min` blending;
  Rust's generated atomic shader writes a packed clip storage buffer directly,
  so it reaches the same intersection by drawing each inner path against its
  parent ID. A two-level GPU intersection test passes. All 49 isolated
  `parallelclips` cells have the same 6-or-15 edge pixels beyond delta 2 as
  their single-clip counterparts, proving nesting adds no divergence; the full
  GM is promoted at 518 pixels/max delta 21 and advances the ratchet to
  exact=32/diverges=0. Continue with update reuse across repeated clipped draws
  and clip-content bounds before treating arbitrary clipping as complete.
- 2026-07-11: Swept every gated clockwise-atomic clipping entry after the
  nested-stack port. Fixed an eligibility/preparation mismatch where a large
  clip passed midpoint-fan validation but panicked when optional interior
  triangulation failed; it now falls back to the validated tessellation and
  has a direct regression test. Promoted 14 entries: `clippedcubic`,
  `clippedcubic2`, `path_stroke_clip_crbug1070835`, `artboardclipping`, all
  five `circle_clips` frames, and all five `clip_tests` frames. The
  `clippedcubic2` reference is structurally identical: 144 pixels differ over
  235,625 pixels, every difference is at most one channel level, and the
  manifest allows zero pixels above that delta. The
  ratchet advances to exact=46/diverges=0/gated=1,421. Large clipped paths,
  negative interior triangles, clipped gradient fallback, and images remain
  named algorithm gates rather than tolerance promotions.
- 2026-07-12: Ported C++ `gr_triangulator.cpp` and
  `GrInnerFanTriangulator` as a stable-index mesh: coincident/intersection
  simplification, winding-preserving edge splits, monotone decomposition,
  weighted face emission, and grout are integrated into multi-contour interior
  tessellation. Two direct C++ WebGPU sub-oracles prove preparation parity:
  the 100-contour grid matches all 7,500 TriangleVertex records, while the exact
  9-cubic flower plus 4-cubic oval matches both contour records, all 108
  TriangleVertex records, and every texel of its 2048x1 RGBA32Uint tessellation
  texture bit-for-bit. A provisional borrowed-coverage hybrid was rejected
  after proving atomics and clockwise-atomic coverage encodings cannot be
  mixed. `make renderer-golden` remains exact=46/diverges=0/gated=1,421; the
  next R2 slice is the dedicated clockwise-atomic shader/scheduling/allocation
  family, not further geometry work on these cases.
- 2026-07-12: Generated the upstream clockwise-atomic path/interior main and
  borrowed-coverage WGSL modules through GLSL -> SPIR-V -> naga and wired them
  as an isolated wgpu pipeline family. Ported C++'s per-path visible-bounds
  allocator (2px padding, 32x32 tiling, monotonic offsets) and global
  borrowed-before-main pass schedule. A 640x640 multi-contour GPU proof renders
  interior and nested-winding pixels correctly; `batchedtriangulations` stays
  within tolerance at 18 pixels, and the renderer ratchet remains
  exact=46/diverges=0/gated=1,421. True clip rendering still requires a
  sampled-input plus fixed-function `plus`/`min` attachment translation;
  storage-buffer PLS writes are not a semantic substitute.
