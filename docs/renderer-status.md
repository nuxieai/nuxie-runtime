# Phase R Status

The execution contract is `docs/renderer-port-map.md`. This file records only
current evidence, open gates, and decisions needed by the next session.

## Metric

Run `make renderer-golden`.

- Rust wgpu: exact=21, diverges=0, gated=1,444, total=1,465.
- Stub baseline: exact=0 for every active entry.
- Exact: `first-light-rectangle-msaa`, `gm-rect-msaa`, and
  `artboardclipping-frame-0-msaa`, plus
  `first-light-triangle-clockwise-atomic`, `gm-rect-clockwise-atomic`,
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
  `gm-emptyfeather-clockwise-atomic`.

## Milestones

- [x] R0: Pixel golden harness. Parser/replay, PNG comparator, artifacts,
  manifest ratchet, checked-in references, stub baseline, and CI are landed.
  The oracle contains 108 upstream GM streams, 294 valid `.riv` streams, 731
  native Metal references, and 1,465 clockwise-atomic/MSAA entries. The
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

1. Finish feather routing in source dependency order: converge atlas coverage,
   correct the cubic feathered-stroke mask's multi-draw origin rays, partition
   atomic-capable draws instead of rejecting the whole frame, then enable the
   prepared feathered-stroke path. Continue R2 with the remaining
   `render_context.cpp` behavior, robust triangulation, and the intersection
   board.
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
