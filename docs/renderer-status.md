# Phase R Status

The execution contract is `docs/renderer-port-map.md`. This file records only
current evidence, open gates, and decisions needed by the next session.

## Metric

Run `make renderer-golden`.

- Rust wgpu: exact=155, diverges=0, gated=1,313, total=1,468.
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
  `gm-feather_cusp-clockwise-atomic`,
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
  `gm-emptystrokefeather-clockwise-atomic`, plus
  `gm-largeclippedpath_clockwise-clockwise-atomic` and
  `gm-largeclippedpath_clockwise_nested-clockwise-atomic`, and the
  `gm-largeclippedpath_{winding,evenodd}{,_nested}-clockwise-atomic` matrix,
  `gm-negative_interior_triangles-clockwise-atomic`, and
  `gm-negative_interior_triangles_as_clip-clockwise-atomic`, and
  `gm-convexpaths-clockwise-atomic`, `gm-pathfill-clockwise-atomic`,
  `gm-oval-clockwise-atomic`, and
  `gm-mutating_fill_rule-clockwise-atomic`, plus
  `gm-concavepaths-clockwise-atomic` and
  the `gm-poly_{clockwise,evenOdd,nonZero}-clockwise-atomic` family, plus
  `gm-cubicpath-clockwise-atomic` and
  `gm-cubicclosepath-clockwise-atomic`, plus `gm-beziers-clockwise-atomic`
  and the `gm-bug{5099,6083,615686,6987,7792}-clockwise-atomic` set, plus
  `gm-bug339297-clockwise-atomic` and
  `gm-bug339297_as_clip-clockwise-atomic`, plus
  `gm-hittest_evenOdd-clockwise-atomic` and
  `gm-hittest_nonZero-clockwise-atomic`, plus
  `gm-image_filter_options-clockwise-atomic`,
  `gm-image_lod-clockwise-atomic`, and
  `gm-image-clockwise-atomic`,
  `gm-image_aa_border-clockwise-atomic`, and
  `gm-mesh-clockwise-atomic`, and
  `gm-degengrad-clockwise-atomic`,
  `gm-rect_grad-clockwise-atomic`,
  `gm-strokedlines-clockwise-atomic`,
  `gm-verycomplexgrad-clockwise-atomic`, and
  `gm-xfermodes2-clockwise-atomic`, and
  `riv-clipping_and_draw_order-frame-0-clockwise-atomic`, plus
  `riv-tape-frame-0-clockwise-atomic`, and
  `riv-superbowl-frame-0-clockwise-atomic`, and
  `riv-jellyfish_test-frame-0-clockwise-atomic`, plus
  `riv-death_knight-frame-0-clockwise-atomic`,
  `riv-deterministic_mode-frame-0-clockwise-atomic`,
  `riv-interactive_scrolling-frame-0-clockwise-atomic`,
  `riv-rocket-frame-{0..4}-clockwise-atomic`,
  `riv-scroll_test-frame-0-clockwise-atomic`,
  `riv-scroll_threshold-frame-0-clockwise-atomic`, and
  `riv-zombie_skins-frame-0-clockwise-atomic`, plus
  `riv-new_text-frame-0-clockwise-atomic` and
  `riv-ai_assitant-frame-0-clockwise-atomic`, plus
  `riv-db_health_tracker-frame-0-clockwise-atomic` and
  `riv-off_road_car-frame-{0..4}-clockwise-atomic`, plus
  `riv-joel_signed-frame-{0..4}-clockwise-atomic`, plus
  `riv-juice-frame-{0..4}-clockwise-atomic`, plus
  `riv-bad_skin-frame-0-clockwise-atomic`, plus 26 newly promoted GM entries:
  `crbug_996140`, both empty-clear cases, Montserrat and Roboto feather text,
  `inner_join_geometry`, `interleavedfillrule`, all three labyrinth variants,
  `mandoline`, both `mesh_ht` cases, transparent overfill, opaque and
  transparent overstroke, `path_skbug_11859`, `quadcap`, `skbug12244`,
  `strokes_zoomed`, `teenyStrokes`, all three tricky-cubic stroke variants,
  and both transparent-clear blend cases, plus the mirrored Montserrat and
  Roboto feather-text entries, plus
  `gm-overstroke_blendmodes-clockwise-atomic` and
  `gm-zeroPath-clockwise-atomic`, plus
  `gm-overfill_blendmodes-clockwise-atomic` and
  `gm-overfill_opaque-clockwise-atomic`, plus
  `gm-strokes_round-clockwise-atomic` and
  `gm-strokefill-clockwise-atomic`, plus
  `gm-rawtext-clockwise-atomic`, plus the zero-delta
  `first-light-nested-clip-probe-clockwise-atomic` sampled-plane oracle.

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
- [x] R2: Algorithm core.
- [ ] R3: Corpus convergence.
- [ ] R4: Performance parity.
- [ ] R5: Native fast paths and extensions; demand-gated after R4.

## Next

1. [x] Build the R3 renderer fuzz-replay harness for both C++ and Rust with
   NaN/huge transforms, zero-area paths, absurd stroke widths, deep clip
   stacks, and hostile gradient stops. Rust must not panic, hang, or lose the
   device; behavioral deltas become named findings and a smoke gate enters CI.
2. [ ] Probe the first ten gated clockwise-atomic `.riv` entries against their
   pinned Metal references: `advance_blend_mode` frames 0-1, `align_target`,
   `animated_clipping`, `animation_reset_cases` frames 0-4, and
   `artboard_list_map_rules`. Promote unchanged-contract passes and replace
   the first failing `algorithm-core` placeholder with an evidence-backed
   diagnostic.

## R2 Completion Record

1. Finish feather coverage in source dependency order. Ordered atomic/fallback
   partitioning, direct/atlas threshold routing, atlas-stroke tessellation
   inputs, and the R16 mask are exact against C++ WebGPU. Direct severe-cusp
   topology, tessellation, normalized atomic coverage, and same-backend final
   pixels are now exact or within the standard `2/32` contract. Porting C++'s
   fixed-color generic-atomic face selection and clockwise `DrawContents`
   encoding removes the max-255 cusp-tip lobe. Fresh native Metal output has
   9,480 pixels beyond delta 2/max delta 11, matching the bounded cross-backend
   feather-overlap family, so Sol approved promotion under a 16,384-pixel
   allowance. Double-sided tessellation now wraps paired
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
   path/interior main, borrowed, outer-clip, and nested-clip shaders are now
   generated from upstream. The isolated family implements the global
   borrowed-to-main barrier, tiled visible-bounds allocations, a sampled
   WebGPU clip plane, and fixed-function `plus`/`min` clip attachments. The
   full large-path clockwise/winding/even-odd matrix is promoted under the
   forced-clockwise oracle. Negative-determinant interior preparation now uses
   C++'s physical forward-then-reverse tessellation layout, reducing the
   unclipped GM from 16,845 pixels to a bounded 1,040 edge pixels and promoting
   it. Counterclockwise culling on clip path/interior passes reduces the
   positive nested-clip draw from 15,408 pixels to 23. The mirrored nested
   inverse clip is now closed. Test-only snapshots prove both
   determinants produce the same borrowed word (`0x13f800`), main word
   (`0x140000`), and white clip-attachment pixel at corresponding interior
   points. The missing output was the clipped full-rectangle content:
   midpoint-fan double-sided preparation always used reverse-then-forward and
   omitted C++'s negative-determinant coverage flag. Porting determinant-aware
   forward-then-reverse layout reduces the GM from 166,809 pixels/max 208 to 46
   pixels beyond delta 2/max 7 and promotes it. A ten-entry basic-fill sweep
   then promoted `convexpaths` after porting missing forward-span row wrapping.
   `pathfill` is also promoted after connected-component analysis proved its
   253 hard-edge pixels are sparse one-pixel placement differences with 99.5%
   support overlap. `oval` then exposed two stale midpoint-fan boundaries:
   small compound fills were rejected into fallback, and midpoint-fan and
   outer-curve patches shared one atomic cull state. Admitting compound fills,
   splitting the atomic path pipelines by patch class, and applying C++'s
   counterclockwise-face cull to the CWA main path restores every oval, hole,
   and overlap. `mutating_fill_rule` is promoted after its remaining 45 pixels
   prove to be four one-pixel edge components with identical foreground
   support. Topologically complex fills are now isolated into the true
   clockwise-atomic pipeline, restoring self-intersections, repeated vertices,
   and compound clockwise contours without regressing large interior
   triangulation. `concavepaths` falls from 4,052 structural pixels to 9 edge
   pixels and `poly_clockwise` becomes pixel-exact. The remaining
   `poly_evenOdd`/`poly_nonZero` pair exposed a floating-point parity bug in
   dominant-winding selection: Rust summed per-contour areas, while C++
   accumulates the raw path in stream order before halving. Porting that exact
   accumulator restores both files to 2 edge pixels. The tied
   `cubicpath`/`cubicclosepath` gap was paint API parity: C++ stores the
   absolute value of stroke thickness, while Rust retained the GM's `-1` and
   rejected all twelve frame strokes. Porting the setter makes both files
   pixel-exact and closes the ten-entry basic-fill sweep. A read-only
   ten-entry fill/clip rescout then found five byte/threshold-exact bug GMs and
   `beziers` at 17 isolated delta-4 edge pixels; all six are promoted under
   their existing 32-pixel contract. The shared
   `bug339297`/`bug339297_as_clip` pair has identical binary support and
   identical black/white counts across Metal and wgpu; only two full-width AA
   scanlines differ under million-scale coordinate cancellation. Both are
   promoted with a documented 1,280-pixel backend allowance. The
   `hittest_evenOdd`/`hittest_nonZero` pair exposed unbounded invented-wgpu
   resource use: 32,580 tiny draws each allocated a 2,048-wide tessellation
   texture, bind group, and render pass in one command buffer, ending in an
   async-map failure. Homogeneous midpoint fills now shelf-pack into one
   tessellation texture, share one flush bind group and render pass, and use
   the translated intersection board to separate AA-overlapping draws into
   ordered atomic groups. Submitting and polling after each group bounds the
   command-buffer lifetime. Clockwise and clip-update batches retain their
   prior texture dimensions and pass topology. Both hit-test GMs now complete
   and are promoted at 382 pixels beyond delta 2/max delta 7 under a 512-pixel
   backend allowance. The per-group wait is a correctness-first R2 choice and
   remains an explicit R4 performance measurement target.
   The first image vertical slice now ports PNG decode to premultiplied RGBA,
   C++'s `ImageRectDraw` vertices/uniforms, the generated fixed-color atomic
   image shaders, authored sampler modes, and C++ WebGPU's generated-shader
   mipmap pass. `image_filter_options` is exact at the standard threshold;
   `image_lod` falls from 60,631 divergent pixels without mips to 276 sparse
   backend-filter pixels and is promoted under a 512-pixel allowance. The
   encoded-image dispatch now also decodes JPEG, restoring both the circularly
   clipped and later unclipped image in `clipping_and_draw_order`; the clip
   boundary and authored draw order match C++ Metal. Embedded PNG ICC profiles
   are now transformed to sRGB before premultiplication, matching ImageIO's
   decode order and sharply reducing the shared LG UltraFine-profile delta.
   `image`, `image_aa_border`, and `mesh` are promoted under measured
   Metal-vs-wgpu decoder/filter allowances. C++'s
   `ImageMeshDraw` is now ported with retained position/UV/index buffers,
   immutable unmap snapshots, generated atomic mesh shaders, clip IDs, and
   authored opacity/samplers. The non-fixed atomic color path now adds C++'s
   tiled color storage plane, destination-copy initialization, monotonic image
   z indices, generated advanced-blend shaders, and coalesced resolve. GPU
   regressions pin screen, darken, exclusion, and luminosity. `tape` and all
   three focused image GMs are promoted. Requesting the selected adapter's
   actual 2D texture limit instead of the 2,048 downlevel bucket unblocks both
   `superbowl` and the 2,080-square `jellyfish_test`. A draw-prefix and no-mip
   sub-oracle disproved the original mipmap attribution: every rendered image
   selects level zero, disabling generated mips is byte-identical, and the
   pre-image radial-gradient background alone carried 866,438 divergent
   pixels. C++'s simple/complex gradient-ramp layout, generated color-ramp
   pass, opacity modulation, and inverse paint transforms are now ported.
   Five focused gradient GMs and `jellyfish_test` are promoted. A mechanical
   sweep of the remaining gradient-bearing `.riv` corpus captured 30 fresh
   C++ references and promoted 11 entries without changing their 32-pixel
   budgets. Eight entries stop on native clockwise-atomic advanced-feather
   parity or incompatible clip diagnostics. `bad_skin` is now promoted after
   ordering generic-atomic outer
   and interior passes and proving all 69 isolated draws are bounded. The
   matching WebGPU MSAA final blit is now byte-exact across all 4,096 oracle
   pixels for the solid, unclipped, source-over slice. Parent-tight clip bounds
   are a later performance refinement, not a correctness gate. MSAA rectangle
   clip-distance is now byte-exact against C++ WebGPU. The unchanged outer
   non-zero path-clip slice is also exact: three stencil-update batches feed a
   fixed-function clipped atlas draw, including the combined path-plus-rect
   state. Changing unrelated outer clips is now exact as well: a generated
   MSAA stencil reset clears the previous clip before the next three-pass
   update, while unchanged clips reuse their stencil state and unclipped draws
   ignore retained stencil. Nested non-zero atlas clips are now exact too:
   `msaaMidpointFanPathsStencil` accumulates the inner winding and an
   intersecting `clipReset` rewrites the parent clip bit. Even-odd and
   clockwise transitions are exact too: outer even-odd uses stencil/cover,
   outer clockwise preserves the clip bit during cleanup, nested even-odd
   writes parity, and nested clockwise selects the `0xc0` reset mask. Filled
   C++ Dawn fixtures behaviorally distinguish every special mode with holes
   and opposite-winding contours. Destination-copy shader blending is now
   byte-exact for solid feather-atlas draws. The translated intersection board
   now schedules disjoint MSAA draws with the C++ layer reservations. A forced
   C++ clockwise-atomic sweep of the 38 remaining gated GMs promoted 24 under
   their unchanged contracts. Two more clear-state GMs became exact after
   frame attachments adopted C++'s integer-premultiplied clear color. C++'s
   determinant-aware contour direction now closes the mirrored
   Montserrat/Roboto feather-text pair. Two gated clockwise-atomic GM
   divergences remain.
   `interleavedfeather` remains parked as a named native-Metal-versus-WebGPU
   atomic intermediate-precision discontinuity. Its exact-source draws 13-14
   now have a dedicated C++ Dawn WebGPU suboracle: dimensions, f32 path bits,
   transforms, paints, and initialize/fill/stroke/resolve schedule are pinned;
   normalized raw coverage is exact; and only two packed color words
   differ at max byte delta one, producing exactly the same two resolved
   pixels at max channel delta seven. The oracle exposed and fixed generic
   feathered-clockwise preparation retaining the nonzero flag, plus the
   advanced feather-fill pipeline culling the wrong face. This closes the Rust
   defect without justifying a native-Metal corpus tolerance. A pinned
   independent full-stream C++ WebGPU-on-Metal lane now replays all 451 draws
   and passes the existing `2/32` contract at 6 pixels over delta 2. C++ Dawn
   and Rust differ at 84 byte-inexact pixels/max 26, while native Metal differs
   from both WebGPU paths almost identically: 18,492 and 18,495 pixels over
   delta 2/max 78. Sol accepted algorithm parity and required the corpus gate
   be renamed, not widened or promoted.
   C++'s empty-segment outcome is now matched in
   stroke/feather preparation for coincident cubics, closing `zeroPath`.
   `dstreadshuffle` is parked under the same intermediate-color precision
   boundary after an independent full-stream C++ Dawn WebGPU-on-Metal lane
   replayed all 97 draws. The untouched stream remains an intentionally failing
   configured gate at roughly 22.84k pixels over delta 2/max 61. A separately
   pinned control changes only the 97 paint blend-mode setters to SrcOver;
   three Rust samples pass the unchanged `2/32` contract at 11, 13, and 13
   pixels over delta 2/max 4. Exact generated-line comparison proves geometry,
   transforms, colors, ordering, dimensions, and opaque clear are unchanged.
   Sol approved removing the algorithm attribution while keeping the entry
   gated with its native reference and tolerance unchanged. A fresh forced
   reference also promotes `overfill_blendmodes` unchanged. `overfill_opaque`
   is now promoted under a bounded 48-pixel cubic-edge allowance: its two translated
   colored draws each contribute the same 20-pixel residual, while binary
   foreground support is exact. The `strokes_round` draw-38 CPU
   `TessVertexSpan` range now matches C++ all 11 records/176 words exactly
   after restoring five-segment miter/bevel joins, preserving raw line
   tangents, and writing flush padding before geometry. Native Metal
   comparison is clean at zero pixels beyond delta 2, so the entry is promoted
   under its unchanged `2/32` contract.
   `strokefill` is also promoted as a bounded native-Metal-versus-wgpu edge
   case. Its 14 isolated draws contribute no structural jump: each has at most
   30 pixels beyond delta 2, the full frame has 109 pixels/max delta 48 across
   19 components with largest area 15, and thresholded support IoU remains
   above 99.985%. It keeps channel delta 2 with a 128-pixel allowance.
   `rawtext` is promoted after a production-ring C++ oracle proved its first
   compound fill exact before rasterization: all 438 CPU span records, the
   `1+318` patch range, 36 contour records, and the complete 2,048x2
   tessellation texture match Rust byte-for-byte. The source fixes restore
   C++ flush-padding order, line/cubic tangent provenance, fused SIMD line
   conversion, and unsigned reflected-row wrapping. Fresh native Metal and
   the legacy reference agree at the standard threshold; the full Rust frame
   has 263 pixels/max delta 80 split across 76 components with largest area
   10, while its two isolated draws contribute 190 and 73 pixels. Foreground
   support IoU remains above 99.822%, so the entry is promoted at unchanged
   channel delta 2 with a bounded 288-pixel backend allowance.
   The remaining logical-flush sort-key fields are
   deferred until pass-level batching is an explicit R4 task: whole-draw Rust
   execution already preserves their only current correctness dependency.
2. Expand corpus entries only as focused pixel replay proves each feature.
   Do not tune broad tolerances around missing algorithm work.

3. [x] Complete the mid-R2 adversarial review of the invented wgpu
   resource/binding plumbing. `docs/renderer-wgpu-adversarial-review.md`
   records the binding, buffer, synchronization, pipeline-cache, and replay
   findings. Texture extents and atomic path-ID exhaustion are hardened; full
   logical-flush rollover and hostile resource/numeric streams are named R3
   work. The R3 semantic-trap and fuzz-replay entry gates remain open.

## Decisions

- 2026-07-13: Close the sampled clockwise-atomic clip-plane finding with a
  production-path readout rather than private Metal instrumentation. A large,
  pixel-aligned compound outer clip forces the global clockwise scheduler; an
  asymmetric nested clip followed by opaque white content records the exact
  `OutermostClip`, `NestedClip`, `ClippedContent` sequence. Rust proves its
  complete captured clip texture equals the probe output, and the 640x640
  output matches the pinned native Metal reference at zero delta. Sol rejected
  the initial small generic-atomic fixture, then approved this routed oracle.
- 2026-07-13: Pin the complete renderer shader lineage before closing R3's
  semantic-trap audit. Clean regeneration now requires runtime `7c778d13`,
  Naga 30.0.0, glslang 16.2.0, SPIRV-Tools 2026.1, ply 3.11, clean tracked and
  untracked shader inputs, and a fixed Python hash seed for upstream's
  otherwise nondeterministic WGSL identifier minifier. CI regenerates and
  asserts 60 raw Rust WGSL modules plus 50 canonical minified C++
  compiler-input headers by exact digest. Sol approved the architecture,
  source/tool fence, CI installation, and evidence wording. The remaining
  audit work is limited to the sampled clip-plane and decoded-image byte
  oracles documented in `docs/renderer-gpu-semantic-trap-audit.md`.
- 2026-07-13: Complete R2 after the exit-contract audit clarified the
  algorithm milestone boundary. All 108 upstream clockwise-atomic GMs are
  accounted for: 106 pass their committed contracts and the remaining two,
  `dstreadshuffle` and `interleavedfeather`, are independently reviewed
  `metal-webgpu-atomic-intermediate-precision` gates backed by SHA- and
  provenance-bound C++ Dawn WebGPU-on-Metal evidence. Zero `algorithm-core`
  gates remain. Both native references and tolerances stay unchanged. The
  invented-wgpu adversarial review, workspace suite, renderer ratchet
  (154/0/1,313), full V2 floor (263 files/584 segments), and scripted V2 floor
  (27/35) are green with no `.riv` regression. R3 starts with the GPU
  semantic-trap audit and renderer fuzz-replay harness.
- 2026-07-13: Reclassify `dstreadshuffle` from `algorithm-core` to
  `metal-webgpu-atomic-intermediate-precision` after pinned untouched and
  SrcOver-control C++ Dawn WebGPU-on-Metal lanes. The strict compiler validates
  stream SHA-256, opaque clear, 97 draws, 96 transforms, 97 saves/restores, 193
  path declarations, 97 paints, and every path/paint snapshot independently of
  the Rust parser. The untouched configured comparison intentionally retains
  and fails the existing `2/32` contract at 22,841-22,851 over-threshold pixels
  across repeated samples/max 61. The control changes exactly 97 blend setters
  and no other generated replay line; three samples pass at 11, 13, and 13
  pixels over delta 2/max 4. Artifact provenance pins the runtime, Dawn,
  adapter, driver, stream, artifact digest, and control override. Sol approved
  the narrower attribution, not promotion or a fitted tolerance. Status,
  native reference, and `2/32` contract remain unchanged.
- 2026-07-13: Reclassify `interleavedfeather` from `algorithm-core` to
  `metal-webgpu-atomic-intermediate-precision` after a pinned full-stream C++
  Dawn WebGPU-on-Metal oracle. A strict stream compiler validates the SHA-256,
  header, 451 draws, 900 transforms, 301 saves/restores, and exact path/paint
  snapshots without using the Rust parser. Artifact provenance records the
  C++ runtime, Dawn, adapter, and driver. Rust passes the entry's pre-existing
  `2/32` contract against C++ Dawn at 6 pixels over delta 2; there are 84
  byte-inexact pixels/max 26. Fresh native Metal differs from C++ Dawn and Rust
  nearly identically at 18,492 and 18,495 pixels over delta 2/max 78. Sol
  approved removing the algorithm attribution while keeping the entry gated,
  its native reference unchanged, and its tolerance unchanged.
  `dstreadshuffle` is next.
- 2026-07-13: Keep both remaining clockwise-atomic GMs gated while accepting
  the isolated `interleavedfeather` ColorBurn pair as Dawn-versus-wgpu
  quantization. The source-generated C++ fixture pins all input f32 bits and
  the four production batches. Sol rejected an initial semantic normalization:
  3,813 opposite-sign coverage words exposed that Rust encoded clockwise as
  nonzero and culled the wrong advanced feather face. Correcting generic
  feathered-clockwise preparation and the two advanced feather-fill pipelines
  makes normalized raw coverage exact. The packed color plane differs at
  exactly two words/max byte delta one, and the resolved frame differs at
  exactly those two coordinates/max
  channel delta seven. No corpus status or tolerance changes in this slice.
  Full-stream C++ WebGPU references for `interleavedfeather` and
  `dstreadshuffle` are the next independent gates.
- 2026-07-13: Promote `feather_cusp` under a bounded 16,384-pixel
  Metal-versus-WebGPU allowance. C++ and Rust match the severe direct cusp's
  complete tessellation inputs and every non-clear atomic coverage word; after
  matching C++'s fixed-color generic-atomic face and clockwise paint encoding,
  the same-backend final blit passes `2/32`. The full native Metal GM retains
  9,480 pixels beyond delta 2/max delta 11, in line with promoted overlapping
  feather families. Sol approved that cross-backend classification after
  catching and requiring fixes for advanced-blend culling and clipped authored
  fill-rule preservation; both now have focused regressions and C++ oracles.
- 2026-07-13: The mid-R2 wgpu resource-seam review is complete. Generated WGSL
  bindings, retained resource lifetimes, queue/readback ordering, and
  factory-lifetime pipeline ownership have no observed correctness mismatch.
  Render-target and decoded-image extents are now validated before wgpu, and
  generic disjoint atomic groups split before their 16-bit path IDs overflow.
  Clip-dependent logical-flush rollover remains a named R3 parity task because
  C++ budgets paths, contours, and tessellation resources together. Buffer
  rings, per-draw dummy resources, submit/wait cadence, and cross-factory
  pipeline caches remain measurement-led R4 work.
- 2026-07-13: Promoted `rawtext` after closing its complete pre-raster path.
  A deterministic stream-derived C++ production oracle pins provenance and
  matches Rust across all 438 CPU tessellation-span records (7,008 words),
  the `1+318` patch range, 36 contour records, and every texel of the 2,048x2
  RGBA32Uint tessellation texture. It exposed four shared preparation gaps:
  flush padding order, line-versus-cubic tangent provenance, C++'s fused SIMD
  line conversion, and unsigned reflected-row wrapping. After porting them,
  the final 263-pixel/max-80 residual is distributed across 76 components
  with largest area 10; isolated draws account for 190 and 73 pixels, and
  thresholded support IoU stays at or above 99.822%. Fresh forced-clockwise
  Metal differs from the legacy reference by zero pixels beyond delta 2. The
  unchanged channel delta 2 and bounded 288-pixel allowance advance the
  ratchet to exact=153/diverges=0/gated=1,314. Renderer golden, the full
  workspace suite, and both V2 golden floors pass.
- 2026-07-13: Promoted `strokefill` under a bounded 128-pixel
  native-Metal-versus-wgpu allowance at unchanged channel delta 2. Prefix
  replay shows no structural jump across its 14 mixed fill/stroke draws; every
  isolated draw is at or below 30 threshold pixels, while the full frame has
  109 pixels/max delta 48 split across 19 components with largest area 15.
  Foreground-support IoU stays above 99.985% at darkness thresholds from 1 to
  192, and the last four authored shapes are byte/threshold exact. A fresh
  forced-clockwise Metal reference agrees with the legacy native reference at
  zero pixels beyond delta 2. The ratchet advances to
  exact=152/diverges=0/gated=1,315; renderer golden, both V2 golden floors,
  and the full workspace tests pass. `rawtext` is the next unresolved GM.
- 2026-07-13: Promoted `strokes_round` without changing tolerance. A new
  production-ring `RIVEATS` oracle pins `firstSpan=0`, `spanCount=11`, the
  64-byte ABI, stream provenance, and exact C++/Rust equality across all 176
  words. It exposed three Rust departures: non-round joins had an invented
  one-segment shortcut instead of C++'s fixed five segments, line tangents
  came from the cubicized one-third control handle instead of the raw line,
  and tail padding followed geometry instead of preceding it at flush scope.
  Porting all three makes the CPU span oracle exact; the post-tessellation
  oracle is exact outside a bounded `0.00035`-radian backend angle delta. Fresh
  native comparison has zero pixels beyond delta 2 and max delta 2 under the
  unchanged `2/32` contract. The ratchet advances to
  exact=151/diverges=0/gated=1,316.
- 2026-07-13: Kept `strokes_round` gated after a 100-draw prefix sweep and
  isolated-draw oracle. Every isolated draw stays at max delta 1 except draw
  38, whose only five threshold violations are contiguous at the smooth
  start/close seam `(25,68..72)`; C++ leaves those pixels white while Rust
  renders the stroke. Restoring C++'s five direct-miter join segments had no
  pixel effect and was reverted. Sol rejected a shared 48-pixel allowance
  because the foreground-support disagreement still admits a tessellation
  input mismatch; the next proof is a record-exact C++/Rust pre-raster oracle
  for draw 38. Separately, fresh `overfill_opaque` prefixes are exact through
  the first colored draw, then add two identical 20-pixel/max-16 cubic-edge
  residuals under a 60-pixel translation. C++ repeats byte-exactly and the
  final C++/Rust foreground support is identical. A bounded 48-pixel
  Metal-vs-wgpu allowance promotes that entry and raises the ratchet to 150.
  Renderer verification is 150/0/1,317; V2 remains 263 files/584 segments,
  scripted V2 remains 27/35, and `cargo test --workspace` is green.
- 2026-07-13: Classified `dstreadshuffle` as the same native-Metal-versus-WebGPU
  atomic intermediate-color precision boundary as `interleavedfeather`.
  Prefixes through Lighten pass; ColorDodge first reaches 198 pixels/max-7,
  ColorBurn reaches 2,042/max-11, isolated ColorDodge passes at 1/max-3, and
  isolated ColorBurn differs at 768/max-9. A full SrcOver control reduces the
  complete board from 23,086 pixels/max-146 to 490/max-24, leaving only sparse
  geometry edges. No tolerance or shader fork is justified. A fresh forced
  native reference promotes `overfill_blendmodes` unchanged at 7/max-3, raising
  the ratchet to 149 exact entries. `strokes_round` is next at 34/max-83.
- 2026-07-13: Matched C++ `RawPath::pruneEmptySegments` behavior in
  stroke/feather preparation for cubic strokes whose four points coincide.
  Rust had retained these curves with zero
  tangents, dropping the round and square cap geometry in `zeroPath`; treating
  them as empty contours emits C++'s opposed synthetic cap joins. A unit test
  pins both cap types. Fresh native Metal comparison falls from 1,490
  pixels/max-128 to 26 sparse edge pixels/max-55 under the unchanged 2/32
  contract, promoting `zeroPath` and raising the ratchet to 148 exact entries.
  `dstreadshuffle` is next.
- 2026-07-13: Parked `interleavedfeather` after isolating its first meaningful
  failure to draws 13-14. Each draw alone is within one channel value of C++,
  but their ColorBurn pair differs at 97 pixels/max-18 and the complete GM at
  18,487 pixels/max-78; replacing ColorBurn with SrcOver makes the pair exact.
  A generated f16 color-plane storage/arithmetic experiment worsened the pair
  and full frame and was reverted. The remaining gap is a named native Metal
  versus WebGPU atomic intermediate-precision discontinuity, not a tolerance
  candidate. A fresh forced C++ reference proves
  `overstroke_blendmodes` passes unchanged at 1 pixel/max-3, promoting it and
  raising the ratchet to 147 exact entries. `zeroPath` is next.
- 2026-07-13: Mirrored clockwise feather fills now use C++'s contour-direction
  contract: direct fills write forward-then-reverse tessellation and atlas
  fills write single-sided descending spans, with matching contour anchors and
  negative-coverage flags. Structural row-wrap tests and fresh native Metal
  references pin the layout. Montserrat drops from 753,955 differing pixels to
  14 and Roboto from 744,884 to zero under the unchanged 2/32 contract, raising
  the ratchet from 144 to 146 exact entries. `interleavedfeather` is next; the
  already-isolated `feather_cusp` residual remains named rather than tolerated.
- 2026-07-13: Reclassified all 38 gated clockwise-atomic GMs against freshly
  forced C++ Metal output. Twenty-four already pass their existing contracts
  and are promoted with mode-correct references; no tolerance changed. The
  two 64x64 transparent-clear fixtures then isolated a real all-pixel defect:
  Rust supplied straight RGB to the frame attachment clear while C++ stores
  integer-premultiplied RGBA. Premultiplying each channel before the wgpu clear
  makes both outputs exact and is pinned by a scalar regression. The renderer
  ratchet rises from 118 to 144 exact entries with zero divergence. The
  remaining 12 GMs are now an evidence-backed queue rather than generic
  `algorithm-core` guesses. A read-only logical-sort audit also found that
  `drawType`, texture, scissor, and contents keys currently affect batching,
  while whole-draw execution conservatively preserves subpass correctness;
  full key parity is therefore R4 work unless a pixel counterexample appears.
  V2 remains 263 files/584 segments, scripted V2 remains 27 files/35 segments,
  and `cargo test --workspace` passes.
- 2026-07-13: Integrated the translated intersection board into MSAA fallback
  scheduling. Rust now reserves C++'s `max(prepassCount, subpassCount)` layers:
  three for fast fills, two for even-odd fills, and one for strokes, atlas
  draws, nested clip updates, and clip resets. Advanced destination-copy
  frames conservatively collapse to one layer per draw, unknown bounds block
  reordering, and the board resets before its signed group index can overflow.
  A full-image regression proves the board order `[0,2],[1]` is byte-identical
  to serialized source order while preserving the authored overlap. The C++
  Dawn oracle independently pins nine tagged MSAA batches: opaque draw 0
  reserves groups 1-3, disjoint translucent draw 2 occupies groups 1-3, and
  overlapping translucent draw 1 begins at group 4. Sol rejected two weaker
  fixtures whose ordering could be explained without all three board layers;
  the final type-10/type-8 boundary distinguishes a short reservation, and
  Sol's closure review reports no findings. `gm-batchedconvexpaths-msaa` now
  executes but remains gated because native Metal has no valid MSAA reference.
  The ratchet remains exact=118/diverges=0/gated=1,349; normal V2 remains 263
  files/584 segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Intra-group draw-type, texture, scissor, and
  subpass sorting remain separate `render_context.cpp` work.
- 2026-07-13: Added generic-atomic advanced blending for feathered fills and
  strokes. The generated non-fixed atomic path and atlas-blit shaders now have
  standard and HSL specializations; shared-flush, per-draw, and atlas paths
  select them, while destination initialization, storage color, and coalesced
  resolve remain unchanged. A C++ Dawn fixture pins the exact
  initialize, `midpointFanCenterAAPatches`, resolve schedule with
  `ENABLE_ADVANCED_BLEND | ENABLE_FEATHER | ENABLE_DITHER`. Dawn and wgpu stay
  within a tested backend envelope of eight pixels at channel delta 1, and the
  ordinary suite exercises all 15 advanced blend modes on direct paths. Sol
  found that atlas-required feathers still selected the fixed-color blit and
  could lose intermediate draws during advanced resolve. The accepted fix adds
  non-fixed standard/HSL atlas pipelines; a two-atlas-draw regression preserves
  both contributions in standard and HSL modes, all seven formerly unsupported
  `.riv` clockwise-atomic entries replay, and Sol's closure review reports no
  findings. A fresh native Metal bankcard reference still differs in 1,485,513
  pixels (max delta 20), so none are promoted; their corpus gate is narrowed to
  `native-clockwise-atomic-advanced-feather-parity`. This slice adds execution
  support without moving the renderer ratchet: exact=118/diverges=0/gated=1,349;
  normal V2 remains 263 files/584 segments, scripted V2 is 27 files/35 segments,
  and `cargo test --workspace` passes. MSAA board-group scheduling is the next
  measured `render_context.cpp` candidate.
- 2026-07-12: Closed MSAA destination-copy shader blending for solid
  feather-atlas draws. A C++ Dawn fixture pins the unextended WebGPU schedule:
  resolve at `dstBlend`, copy the intersected draw bounds into a single-sample
  sampled texture, then restart MSAA with color/depth/stencil load operations.
  Rust now segments the pass at each advanced atlas draw, preserves the old
  fixed-function path, and selects generated standard or HSL atlas shaders for
  all 15 advanced blend modes. The 64x64 ColorDodge fixture matches all 4,096
  RGBA pixels byte-for-byte; GPU regressions compare all modes to the generated
  atomic shader and preserve a non-rectangular path clip across two destination
  copies. A real `bankcard` MSAA replay also completes. Sol found one silent
  omission for gradient-backed advanced feather paints; the accepted fix
  retains a named `Unsupported` boundary until atlas gradient resources are
  implemented. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349; normal V2 remains 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Remaining `render_context.cpp` behavior and integration of the translated
  intersection board are next.
- 2026-07-12: Closed even-odd and clockwise MSAA path clips for feather-atlas
  draws. Rust now selects C++'s exact outer schedules: non-zero and clockwise
  run borrowed/update/cleanup with clockwise cleanup limited to write mask
  `0x7f`, while even-odd runs parity stencil then cover/reset. Nested clips use
  write mask `0x01` for even-odd or `0x7f` otherwise; the parent intersection
  reset reads `0xc0` for clockwise and `0xff` for non-zero/even-odd. Clip
  tessellation now mirrors C++ contour orientation under negative transforms.
  Two C++ Dawn fixtures pin the exact five/six-batch schedules and all 4,096
  RGBA pixels. Sol found that the first stroked fixtures did not behaviorally
  distinguish the special pipelines; the accepted filled fixtures now expose
  nested even-odd and outer even-odd holes plus outer/nested opposite-winding
  rejection. Sol's closure review reports no remaining findings. Terra's
  bounded README/harness update passed all 16 format tests and was accepted
  after local diff review. The renderer ratchet remains
  exact=118/diverges=0/gated=1,349; normal V2 remains 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Destination-copy shader blending is next.
- 2026-07-12: Closed nested non-zero MSAA path clips for feather-atlas draws.
  The C++ Dawn oracle pins the exact `8,9,10,11,14,4` schedule: outer
  borrowed/update/cleanup, double-sided nested winding stencil, parent-bounds
  intersection reset, and clipped atlas blit. Rust ports the generated path
  and stencil shaders with C++'s depth, cull, reference, compare, and write-mask
  state and matches all 4,096 RGBA pixels byte-for-byte. Active clip stacks now
  retain their C++ clip ID and render only a newly appended suffix; a focused
  regression proves `[A] -> [A,B]` consumes one ID and schedules only nested
  update, intersect reset, and content. Sol found and verified that incremental
  correction, then reported the integrated seam clean. The renderer ratchet
  remains exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584
  segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Even-odd and clockwise MSAA clip transitions
  are next.
- 2026-07-12: Closed changing outer non-zero MSAA path clips for atlas draws.
  Rust now ports C++ `ClipReset::clearPreviousClip` through the generated
  `draw_msaa_stencil` shader, exact `Depth24PlusStencil8` not-equal/zero state,
  clockwise front face with counterclockwise culling, transformed `roundOut()`
  reset bounds, and the six `TriangleVertex` rectangle. Unchanged clips skip
  both ID allocation and redundant stencil updates; an unclipped draw may run
  while the prior stencil remains retained, and the next unrelated clip still
  resets it. The Dawn oracle asserts the exact nine-batch schedule
  `8,9,10,4,14,8,9,10,4`, every base/count range, the 97x48 atlas, 2048x1
  tessellation texture, and exact left/reset-gap/right pixels. Rust matches
  all 4,096 pixels, while the previous unclipped, rectangle-clipped, and
  single-path-clipped frames remain exact. Terra identified reset-adjacent GM
  streams, but Sol correctly classified them as negative future boundaries,
  not positive coverage for this feather-atlas slice. Sol's adversarial review
  also tightened the oracle, reset-pixel test, retained-stencil transition,
  and native bounds parity before reporting no remaining findings. The
  renderer ratchet remains exact=118/diverges=0/gated=1,349; normal V2 is 263
  files/584 segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Nested non-zero atlas clip intersection is
  the next source-order R2 boundary.
- 2026-07-12: Closed the unchanged outer non-zero MSAA path-clip slice for
  atlas draws. The fallback attachment is now depth/stencil, the path pipeline
  implements C++'s borrowed/update/cleanup stencil states and exact inner-fan
  range, and the atlas pipeline selects fixed-function path, rectangle, or
  combined path-plus-rectangle clipping. A new C++ Dawn oracle asserts the
  exact four-batch schedule: three clip-update draws followed by an
  active-clip atlas draw whose shader features remain dither-only. Rust matches
  all 4,096 pixels for unclipped, rectangle-clipped, and path-clipped atlas
  frames. Nested clips, changing outer clips, alternate clip fill rules,
  non-atlas MSAA path clips, and MSAA images remain named `Unsupported`
  boundaries with early-ingress regressions. Terra supplied the bounded C++
  batch inventory; the executable oracle corrected its shader-feature
  inference. Sol then found and closed the alternate-fill and image-mesh
  ingress leaks before reporting the final diff clean. `make renderer-golden`
  remains exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584
  segments, scripted V2 is 27 files/35 segments, and
  `cargo test --workspace` passes. Clip reset and the remaining clip-stack
  transitions are the next source-order R2 boundary.
- 2026-07-12: Closed MSAA rectangle clip-distance atlas blits. Device creation
  requests `CLIP_DISTANCES` only when the adapter exposes it; the atlas blit
  pipeline then selects upstream's generated clip-distance vertex permutation
  and uploads the existing C++-shaped clip inverse matrix through
  `PaintAuxData`. Adapters without the feature retain the named `Unsupported`
  result. The C++ Dawn oracle now enables WebGPU clip planes only for a new
  clipped case, asserts the `ENABLE_DITHER | ENABLE_CLIP_RECT` batch, and emits
  a complete 64x64 `RIVEABL` artifact. Rust matches all 4,096 pixels exactly;
  an always-on GPU fence also proves output is confined to the clip rectangle.
  Sol's adversarial review found and closed two portability gaps: the ordinary
  test now preserves the unsupported branch, and unrelated C++ oracle modes no
  longer require clip distances. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349; normal V2 is 263 files/584 segments,
  scripted V2 is 27 files/35 segments, and `cargo test --workspace` passes.
  Path-clip/stencil is the next R2 boundary, followed by destination-copy
  shader blending.
- 2026-07-12: Closed the matching C++ WebGPU MSAA atlas final-blit oracle.
  MSAA now forces feathered solid fills and strokes through the translated
  atlas tessellation and R16 mask passes, then draws the upstream six-vertex
  atlas rectangle through a dedicated 4x fixed-function pipeline. The original
  blank Rust frame differed at all 4,096 pixels/max delta 80; the final RGBA8
  frame is byte-exact, including upstream dither and premultiplied output.
  Always-on GPU tests cover the canonical stroke, two ordered feathered fills,
  and resource retention across multiple atlas draws. Sol's adversarial review
  found that the first patch silently accepted clip rectangles, path clips,
  and non-source-over blending without their required shader permutations.
  Those cases now return named `Unsupported` errors with regressions instead
  of drawing incorrect pixels; Sol's final review reports no remaining blocker
  for the intentionally narrow slice. `make renderer-golden` remains
  exact=118/diverges=0/gated=1,349. Rectangle clip-distance, path-clip/stencil,
  and destination-copy blend variants remained as the next source-order work.
- 2026-07-12: Removed a generic-atomic interior race in the invented wgpu
  batching seam. Shared flush groups had submitted outer-curve patches and
  interior triangles in one render pass; five identical isolated `bad_skin`
  hair renders differed at 114-264 pixels/max delta 255 because attachment
  writes were not ordered with the storage atomics. Triangle-backed draws now
  use the existing ordered outer and interior render passes. A GPU regression
  is red without the split and byte-stable across five repeats with it; the
  isolated hair falls to one pixel/max delta 31 versus Metal.
  A new direct C++ WebGPU preparation oracle reproduces the exact authored
  non-zero hair path, transform, and frame under the clockwise override. It
  captures one contour, 48 `TriangleVertex` records, and the complete 2048x1
  RGBA32Uint tessellation texture. Sol rejected the lane's first authored-
  clockwise capture as a false oracle; the amended non-zero capture passed
  closure review. The current single-contour ear clip differs canonically from
  C++ triangles, while substituting the global inner-fan stream and broad CWA
  routing regressed four exact entries, so that separate port boundary was
  reverted rather than traded against this fix.
  Three full post-fix renders are stable at 2,701 pixels beyond delta 2/max
  delta 159. Across 2,635 residual components, 2,626 are single pixels and
  only four exceed area three; all 69 isolated draws stay at or below 40
  pixels beyond delta 2. Sol accepted a bounded 4,096-pixel composite/backend
  allowance at unchanged channel delta 2. The ratchet advances to
  exact=118/diverges=0/gated=1,349; the matching WebGPU MSAA final-blit oracle
  is next.
- 2026-07-12: Added the combined clockwise-atomic and advanced-blend color
  path. Draw-prefix Metal replay isolated `juice`'s first structural jump to
  draw 15, an overlay compound fill routed through fixed-color CWA shaders;
  the generic advanced shader already matched preceding multiply content.
  Advanced CWA draws are now isolated at run boundaries and render their paint
  and coverage through the existing fixed-function CWA pipeline into a
  transparent intermediate. A one-fragment-per-pixel composite then applies
  the upstream advanced equations against an explicit destination copy. This
  preserves C++'s hardware source-over ordering without cross-fragment storage
  races. Rectangle-clip specialization is supported, while path clips and
  feather retain explicit gates after internal CWA selection. Focused GPU
  regressions pin white overlay over a prior compound fill through a partial
  clip rect, compare all fifteen advanced modes against the generated generic
  atomic shader, and prove advanced CWA path clips return `Unsupported` rather
  than panicking.
  All 18 cumulative `juice` prefixes track Metal; all five byte-identical full
  frames retain 140 edge pixels beyond delta 2/max delta 12 and are promoted
  under a 256-pixel allowance. The ratchet advances to
  exact=117/diverges=0/gated=1,350; `bad_skin` is next.
- 2026-07-12: Preserved C++'s frame-wide clockwise fill override after an
  axis-aligned clip is reduced to paint metadata. Rust had excluded every
  clip-rect draw from the true clockwise pipeline, so `joel_signed` rendered a
  detached opposite-winding leaf as non-zero content. Clip-rect-enabled path,
  interior, sampled-clip path, and sampled-clip interior pipeline variants now
  select upstream specialization ID 1 per draw. A visible-bounds eligibility
  check mirrors C++'s pre-allocation cull for offscreen paths. Exact first-draw
  and partial-clip GPU regressions pin winding and clip-rect behavior;
  `gm-mesh` retains its partial clip at 14 pixels beyond delta 2, and
  `db_health_tracker` no longer reaches coverage allocation for its offscreen
  draw. All five byte-identical Joel references
  retain only 191 mostly isolated edge pixels/max delta 5, so they keep delta 2
  with a bounded 256-pixel allowance. The ratchet advances to
  exact=112/diverges=0/gated=1,355; `juice` is next.
- 2026-07-12: C++ `RiveRenderer::applyClip` generates a new clip ID whenever
  an element is rendered into the clip buffer. Rust had encoded stack depth as
  the ID, so unrelated root clips all reused ID 1 and stale coverage from an
  earlier clip admitted `off_road_car`'s windshield gradient around the front
  grille. Unique per-render clip IDs across paths, images, and meshes remove
  that coherent 2,042-pixel leak; a two-root-clip regression pins the lifecycle.
  All five recorded samples are byte-identical on each backend and now share an
  identical 1,862-pixel residual. Its 55 components are confined to thin ground,
  stripe, and small car edges; the largest occupies a 243x13 strip. Replacing
  the implicated gradient with solid cyan leaves exactly the same residual,
  proving the structural clip error is closed. The family keeps max channel
  delta 2 with a bounded 2,048-pixel backend allowance and advances the ratchet
  to exact=107/diverges=0/gated=1,360.
- 2026-07-12: `db_health_tracker` keeps max channel delta 2 with a bounded
  1,152-pixel allowance. Draw-prefix replay proves its first 430 of 473 draws
  exact, including the chart's 353-stroke batch, clips, cards, and markers.
  Divergence starts in the late solid text-outline fills and accumulates in
  small increments to 1,071 pixels across a 2,073,600-pixel frame; 432
  connected components are present, none larger than 13 pixels, and only 32
  samples exceed delta 32. Flat fills and the two gradient draws add no
  residual. This is repeated native Metal/wgpu glyph/path-edge placement, not
  missing algorithm work. The ratchet advances to
  exact=102/diverges=0/gated=1,365.
- 2026-07-12: `ai_assitant` keeps max channel delta 2 with a bounded 384-pixel
  allowance. Its exact background and 16 repeated rotated stroke pairs rise
  smoothly from 1 to 341 residual pixels with no structural jump; 340 of 341
  connected components are single pixels and foreground-support IoU remains
  99.3-99.8%. Replacing every gradient shader with solid cyan preserves the
  residual class (350 pixels, max delta 15), disproving gradient math. Per-draw
  cutoffs concentrate the accumulation in each intact `feather=12` companion,
  isolating native Metal/wgpu feather-edge placement rather than missing feather
  geometry. The ratchet advances to exact=101/diverges=0/gated=1,366.
- 2026-07-12: `new_text` keeps max channel delta 2 with a bounded 48-pixel
  allowance. Draw-prefix replay attributes the first divergence to its compound
  text path: 44 residual pixels split into 22 components, none larger than four
  pixels, with foreground-support IoU near 99.5%. Replacing the path's gradient
  with solid white preserves the same residual class (40 pixels, max delta 47),
  disproving gradient math and isolating native Metal/wgpu path-edge placement.
  The ratchet advances to exact=100/diverges=0/gated=1,367.
- 2026-07-12: Completed the post-gradient `.riv` sweep. The host port now uses
  C++'s exact `math::EPSILON` (`1/4096`) and forward/backward monotonic stop
  clamps; all five gradient GM oracles remain unchanged. Of 38 gated
  gradient-bearing clockwise entries, 30 render and now have fresh C++ Metal
  references, while eight retain explicit native clockwise-atomic
  advanced-feather parity or clip-rect diagnostics. Eleven pass under the
  existing strict delta-2/32-pixel budget:
  `death_knight`, `deterministic_mode`, `interactive_scrolling`, all five
  `rocket` samples, `scroll_test`, `scroll_threshold`, and `zombie_skins`.
  Larger measured residuals remain gated rather than tolerated. The ratchet
  advances to exact=99/diverges=0/gated=1,368.
- 2026-07-12: The `jellyfish_test` mip gate was false attribution. A bounded
  draw/LOD inventory found all 22 image draws at LOD 0; a no-mip render is
  byte-identical to the corrected nearest-mip render. Prefix replay then proved
  the solid background exact, the first radial gradient added 589,692 divergent
  pixels, and both radial gradients added 866,438 before any image draw. Porting
  `render_context.cpp`'s 512-wide simple/complex `GradientSpan` layout through
  the generated color-ramp WGSL, plus C++ gradient normalization, opacity, and
  inverse paint matrices, makes `degengrad`, `rect_grad`, and `verycomplexgrad`
  exact at delta 2; `strokedlines` and `xfermodes2` retain only 4 and 8 edge
  pixels under their existing 32-pixel budgets. `jellyfish_test` falls from
  604,916/max 139 to 22,363/max 7 and is promoted under a 23,000-pixel strict
  delta-2 image/backend allowance. Matching C++'s nearest mip selection also
  shrinks the stale image allowances: `image`/`image_aa_border`/`image_lod`/
  `mesh` to 32 pixels, `tape` to 64, and `superbowl` to 128. The ratchet advances
  to exact=88/diverges=0/gated=1,379.
- 2026-07-12: Image allocation now requests the selected adapter's supported
  `max_texture_dimension_2d` instead of inheriting wgpu's 2,048 downlevel
  default. A 2,080-pixel decode regression pins the request; `jellyfish_test`
  (2,080x2,080) and `superbowl` (2,914x296) both render instead of panicking.
  Fresh `superbowl` output is visually coincident and promoted at strict delta
  2 with 11,268/max 64 samples under a 12,000-pixel backend allowance.
  `jellyfish_test` is not tolerated: all 23 CoreGraphics decodes match Rust
  premultiplication within one byte with identical alpha, while its 604,916/max
  139 frame delta is isolated to translucent glows and edges. It is reclassified
  to `platform-mipmap-filtering` pending a mip-level oracle. The ratchet advances
  to exact=82/diverges=0/gated=1,385.
- 2026-07-12: PNG decode now honors `iCCP` metadata with a pure-Rust moxcms
  transform from the embedded profile to sRGB before alpha premultiplication,
  matching C++ ImageIO's color-convert-then-premultiply order. On `gm-mesh`
  this reduces the fresh C++ delta from 140,327/max 56 to 17,450/max 34 while
  preserving all twelve transforms, clips, and blend modes. The remaining
  Metal-vs-wgpu decoder/filter samples are bounded at strict delta 2:
  `image` 8,814/max 39 under 9,000, `image_aa_border` 5,745/max 71 under 6,000,
  and `mesh` 17,450/max 34 under 18,000. These are whole-entry backend
  allowances over visually coincident output, not missing-algorithm
  tolerances; the ratchet advances to exact=81/diverges=0/gated=1,386.
- 2026-07-12: Advanced atomic image blending follows C++ WebGPU's non-fixed
  color-output lifecycle rather than per-draw framebuffer copies: request the
  seventh fragment storage binding, copy the current target before each
  advanced atomic run, initialize a tiled `u32` color plane through
  `loadColorFromDstTexture`, assign authored z indices, run the generated
  non-fixed image/path shaders, and coalesced-resolve once. A 1x1 GPU oracle
  pins screen, darken, exclusion, and luminosity over a known destination.
  Fresh C++ `gm-mesh` renders all twelve meshes with matching geometry and
  blend character, but its 319x320 PNG carries a large ICC profile; even the
  srcOver control column differs at 19,752 pixels/max delta 56, while the full
  file differs at 140,327 pixels/max delta 56. The entry is reclassified to
  `platform-image-decode-color-profile`; no tolerance was widened.
- 2026-07-12: `ImageMeshDraw` follows C++'s retained-buffer contract: position
  and UV streams are separate `float2` vertex buffers, indices are `u16`, and
  every unmap snapshots a new submitted wgpu buffer so later mutations cannot
  rewrite queued draws. The generated fixed-color atomic mesh shaders provide
  `srcOver`, clipping, clip-rect, transform, opacity, and sampler parity.
  `tape` retains delta 2 with a bounded 6,400-pixel allowance: 6,162 pixels
  differ versus fresh C++ Metal (max delta 31), all inside the three decoded
  image interiors; foreground-support masks differ at only 89-192 sparse edge
  pixels across 1%-20% thresholds. Advanced image blend modes remain named
  algorithm work and are not covered by this allowance.
- 2026-07-12: Encoded-image dispatch supports both corpus formats: PNG and
  JPEG. `clipping_and_draw_order` was a decode gate, not a clip-buffer failure:
  its embedded bytes begin with JPEG SOI, and the PNG-only decoder returned an
  empty image before either draw reached the renderer. With pure-Rust JPEG
  decode, both 278x278 images, the circular clip boundary, and all authored
  ordering are present. Its bounded 10,000-pixel allowance at delta 2 covers
  the measured 9,494 ImageIO-versus-`jpeg-decoder` color samples (max delta 18),
  all confined to the two image interiors; the pre-fix missing-image result was
  104,981 pixels, over ten times the allowance.
- 2026-07-12: ImageRect uses the upstream generated fixed-color atomic shader,
  not the separate atomic color-buffer variant. PNGs upload as premultiplied
  RGBA with the full C++ mip count, and each remaining mip is generated through
  the upstream WebGPU filtered-blit shaders. `image_lod` retains delta 2 with a
  bounded 512-pixel allowance: 276 pixels differ after mip generation, max 43,
  with all authored images and transforms present. Metal platform decode color
  management and clipped-image atomics remain named gates, not tolerances. Sol
  review also made MSAA/fallback images explicitly unsupported until their
  pipelines exist, and hoisted ImageRect geometry, dummy bindings, and all 18
  sampler permutations so non-image draws do not inherit image resource churn.
- 2026-07-12: Legacy homogeneous midpoint-fill batches may share shelf-packed
  tessellation storage and a render pass. Clockwise and clip-update batches
  preserve the established per-draw resource/pass topology. Intersection-board
  groups are submitted independently to bound backend resource lifetime; R4
  must measure and optimize the wait policy without weakening corpus parity.
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
- 2026-07-12: The two large clockwise clip entries retain max channel delta 2
  with a bounded 640-pixel Metal-vs-wgpu allowance. Their 50%-coverage masks
  are pixel-identical; the 592-593 residual pixels are confined to clip
  boundaries, with no missing or extra binary coverage.
- 2026-07-12: Sol review confirmed that forced clockwise-atomic mode
  intentionally replaces authored nonzero/even-odd fill semantics; preserving
  parity would contradict the C++ oracle. Viewport-bounded nested inverses are
  behaviorally equivalent while the parent clip remains active, so parent
  content/tightened bounds stay a performance task unless pixels prove otherwise.
- 2026-07-12: `negative_interior_triangles` keeps max channel delta 2 with a
  bounded 1,152-pixel allowance. The two isolated determinant draws differ at
  553 and 487 pixels, the combined 1%-coverage support masks differ at only 26
  pixels, and the 1,040 residuals are sparse backend edge coverage rather than
  missing geometry. At this point the mirrored as-clip case remained gated
  because its broad blank region was still an algorithm failure.
- 2026-07-12: `negative_interior_triangles_as_clip` keeps max channel delta 2
  with a bounded 64-pixel allowance. After the mirrored fallback fix, only 46
  pixels exceed delta 2 across 2.56M pixels and max delta is 7; both shapes,
  checkerboard clipping, and corresponding interior support are restored.
- 2026-07-12: `convexpaths` keeps max channel delta 2 with a bounded 64-pixel
  allowance. After the row-wrap fix, only 43 pixels exceed delta 2 across
  1.32M pixels; the remaining max-103 samples are sparse hard-edge backend
  differences, not missing support.
- 2026-07-12: `pathfill` keeps max channel delta 2 with a bounded 256-pixel
  allowance. Its 253 residuals have 99.5% support overlap and split into tiny
  hard-edge components; the largest is 56 pixels inside a 19x13 box. Max-255
  samples are one-pixel binary edge placement, not missing shapes.
- 2026-07-12: `oval` keeps max channel delta 2 with a bounded 128-pixel
  allowance. After the midpoint-fan admission/cull fix, all 109 residuals are
  one-pixel edge components, the largest is 16 pixels, and foreground support
  has 99.9965% IoU with equal expected/actual support counts.
- 2026-07-12: `mutating_fill_rule` keeps max channel delta 2 with a bounded
  64-pixel allowance. All 45 residuals form four one-pixel vertical edge
  components, max delta is 11, and expected/actual foreground support is
  identical (IoU 1.0).
- 2026-07-12: Self-intersecting and compound fills form their own
  clockwise-atomic runs. Endpoint normalization keeps ordinary closed cubics
  on the legacy analytic path; this preserves the promoted large-path corpus
  while matching C++ atomic accumulation for complex topology.
- 2026-07-12: Dominant winding uses C++ `RawPath::computeCoarseArea` stream
  order, including coarse cubic subdivision. This order is observable when
  opposite contours nearly cancel and must not be replaced by independently
  rounded per-contour areas.
- 2026-07-12: Render-paint stroke thickness follows C++ and stores `abs(value)`;
  invalid `NaN` remains invalid. Negative GM inputs therefore become positive
  strokes before draw-time culling.
- 2026-07-12: `beziers` retains the standard max-delta-2/32-pixel contract.
  Its 17 residual pixels are disconnected one-pixel edge samples at max delta
  4; no geometry or connected support is missing.
- 2026-07-12: `bug339297` and `bug339297_as_clip` retain max channel delta 2
  with a 1,280-pixel allowance. Both backend pairs have zero binary-support
  differences and identical black/white pixel counts; all residuals occupy
  the same two antialiased full-width scanlines under million-scale coordinate
  cancellation, so this is a Metal-versus-wgpu precision difference rather
  than missing fill or clip geometry.

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
- 2026-07-12: Completed the clockwise-atomic clip plane. Dedicated upstream
  outer/nested clip fragments render to an RGBA8 attachment with `plus`/`min`
  blending, while a checked-in upstream wrapper samples that attachment for
  clipped path and interior draws. Corrected WebGPU borrowed-face culling,
  threaded the real maximum path ID, and ported nested inverse-path creation.
  `largeclippedpath_clockwise_nested` improved from 145,064 differing pixels
  to 593, and both promoted large-clockwise entries have pixel-identical 50%
  coverage masks versus native Metal. The renderer ratchet advances to
  exact=48/diverges=0/gated=1,419; direct C++ preparation oracles, all 122
  active renderer unit tests, both V2 floors (584 and 35 exact segments), and
  the full workspace pass.
- 2026-07-12: Captured fresh forced-CWA C++ references for the winding and
  even-odd large-path variants after Terra reconnaissance and Sol review.
  Winding and clockwise references are byte-identical; even-odd uses different
  authored geometry but the same effective clockwise rule. All four Rust
  comparisons have the already-proven 593 boundary pixels/max 128 and
  pixel-identical 50% coverage masks, so they inherit the bounded 640-pixel,
  delta-2 allowance and advance the ratchet to exact=52/diverges=0/gated=1,415.
  The adjacent negative-interior probe remains a real geometry/coverage gap:
  16,845 pixels unclipped and 181,923 as a clip, both max delta 255.
- 2026-07-12: Ported C++'s `forwardThenReverse` physical tessellation layout
  for negative clockwise coverage and counterclockwise face culling for clip
  path/interior passes. Regression tests pin the C++ contour indices (493
  normal, 17 mirrored) and three formerly missing nested-clip pixels. The
  unclipped negative-interior GM improves 16,845 -> 1,040 pixels and advances
  the ratchet to exact=53/diverges=0/gated=1,414. A Terra oracle lane remains
  isolated and unmerged after Sol review found that its first capture modeled
  an opaque standalone draw instead of the real borrowed/main split; its
  forced-CWA Dawn amendment then failed binding validation. Continue linearly
  with a narrow mirrored inverse-clip oracle rather than merging that lane.
- 2026-07-12: Sol reviewed two read-only Terra scouts against direct probes for
  the mirrored inverse clip. The first found real source differences in parent
  clip bounds and fallback fan direction, but applying tight inverse bounds did
  not move the 166,809-pixel result; the fallback is not active in this GM. The
  second correctly ruled out coverage initialization and front-face mapping,
  but its reported shader mismatch was a temporary `abs` diagnostic and was
  rejected. A determinant-paired preparation probe then matched contour/face
  counts, face orientation, and coverage ranges. The next useful evidence is a
  borrowed/main coverage-buffer capture; all diagnostic code was reverted.
- 2026-07-12: Added opt-in CWA storage-buffer and clip-attachment snapshots at
  the borrowed/main boundary. Positive and mirrored nested clips are identical
  at all captured stages, proving the clip was correct; the following clipped
  rectangle was blank because midpoint-fan double-sided preparation ignored
  C++ `forwardThenReverse` plus `NEGATE_PATH_FILL_COVERAGE_FLAG` semantics.
  Porting that shared direction rule restores the mirrored draw and advances
  the ratchet to exact=54/diverges=0/gated=1,413. A Terra scout confirmed native
  Metal has no executable CWA storage-buffer mode, so implementing an entire
  backend solely for a redundant C++ buffer capture was rejected; native final
   pixels remain the cross-implementation oracle.
- 2026-07-12: A read-only Terra sweep measured ten basic gated CWA fills after
  the mirrored fallback fix. `convexpaths` exposed the highest-priority result:
  a pre-frame panic from packing global tessellation locations into signed
  16-bit row-local fields. Porting C++'s existing forward-span row wrapping
  removes the panic and leaves only 43 pixels beyond delta 2/max 103 across
  1.32M pixels, promoting the entry and advancing the ratchet to
  exact=55/diverges=0/gated=1,412. `pathfill` is the nearest next candidate at
  253 pixels beyond delta 2; the remaining eight have named winding/interior
  geometry gaps from 4,578 to 32,596 pixels.
- 2026-07-12: Promoted `pathfill` after a 50%-support/connected-component audit
  localized all 253 pixels beyond delta 2 to sparse hard edges across its
  compound icon stress set. The ratchet advances to
  exact=56/diverges=0/gated=1,411 without renderer changes.
- 2026-07-12: Promoted `oval` by admitting small compound midpoint fans to the
  atomic path, separating midpoint-fan and outer-curve cull state, and porting
  C++'s counterclockwise-face cull to the clockwise-atomic main path. Two GPU
  regressions cover same-direction cubic union and opposite-direction holes;
  the residual is 109 sparse edge pixels and the ratchet advances to
  exact=57/diverges=0/gated=1,410.
- 2026-07-12: A fresh post-`oval` basic-fill scout measured
  `mutating_fill_rule` at 45 pixels beyond delta 2/max 11. `concavepaths`, the
  three `poly_*` variants, `cubicpath`, and `cubicclosepath` retain structural
  topology/primitive gaps from 4,052 to 16,169 pixels, so
  `mutating_fill_rule` is the next R2 target.
- 2026-07-12: Promoted `mutating_fill_rule` after an independent component and
  support audit localized all 45 residuals to four one-pixel circle edges.
  The ratchet advances to exact=58/diverges=0/gated=1,409; `concavepaths` is
  the next measured structural fill target at 4,052 pixels beyond delta 2.
- 2026-07-12: Routed only topologically complex fills through the true
  clockwise-atomic coverage pipeline. Prefix replay localized the first
  `concavepaths` failure to the self-intersecting bowtie; full CWA replay proved
  the upstream behavior, and run splitting retained the established ordinary
  fill path. `concavepaths` now has 9 pixels beyond delta 2/max 13 and
  `poly_clockwise` is pixel-exact, advancing the ratchet to
  exact=60/diverges=0/gated=1,407.
- 2026-07-12: Ported C++ coarse-area accumulation order after the isolated
  counterclockwise six-point polygon proved that equal opposite contours used
  the wrong floating-point tie-break in Rust. `poly_evenOdd` and `poly_nonZero`
  each fall from 9,121 structural pixels to 2 edge pixels/max 17, advancing
  the ratchet to exact=62/diverges=0/gated=1,405.
- 2026-07-12: Ported `RiveRenderPaint::thickness` absolute-value semantics.
  This restores the twelve one-pixel rectangle frames shared by `cubicpath`
  and `cubicclosepath`; both become pixel-exact, the basic-fill sweep closes,
  and the ratchet advances to exact=64/diverges=0/gated=1,403.
- 2026-07-12: A bounded Terra rescout measured ten gated fill/clip GMs.
  `bug5099`, `bug6083`, `bug615686`, `bug6987`, and `bug7792` have zero pixels
  beyond delta 2, while `beziers` has 17 isolated delta-4 edge pixels within
  its existing budget. Promoting all six advances the ratchet to
  exact=70/diverges=0/gated=1,397; the shared two-row `bug339297` family is
  next.
- 2026-07-12: Audited the `bug339297` pair with independent threshold masks
  and color histograms. Support is pixel-identical in clipped and unclipped
  forms, while 1,280 AA samples differ across two scanlines. The documented
  backend allowance promotes both entries and advances the ratchet to
  exact=72/diverges=0/gated=1,395; the hit-test readback failure is next.
- 2026-07-12: Added JPEG decode alongside PNG and captured a dedicated C++
  clockwise-atomic reference for `clipping_and_draw_order`. Both image draws,
  including the circular clip, are restored; the renderer ratchet advances to
  exact=77/diverges=0/gated=1,390. `ImageMeshDraw` is the next R2 image slice.
- 2026-07-12: Ported C++ `ImageMeshDraw` with snapshotted retained buffers and
  the generated fixed-color atomic mesh shaders. A GPU regression pins indexed
  position/UV sampling, and `tape` matches fresh C++ geometry and support under
  its bounded decoder/filter allowance. The ratchet advances to
  exact=78/diverges=0/gated=1,389; advanced image blending is next.
- 2026-07-12: Ported the C++ WebGPU non-fixed atomic color lifecycle and
  generated advanced image shaders. `gm-mesh` now renders every authored blend
  and is reclassified from algorithm work to its measured ICC decoder gate;
  the ratchet remains exact=78/diverges=0/gated=1,389 without widening a
  tolerance. Color-managed PNG decode is next.
- 2026-07-12: Added embedded ICC-to-sRGB conversion before PNG
  premultiplication and promoted `image`, `image_aa_border`, and `mesh` under
  their measured decoder/filter allowances. The ratchet advances to
  exact=81/diverges=0/gated=1,386; rescouting the larger image/mesh corpus is
  next.
- 2026-07-12: Replaced the self-imposed 2,048 texture cap with the adapter's
  supported limit. `superbowl` is promoted under its measured image-backend
  allowance; `jellyfish_test` now renders but remains gated on a mip-level
  oracle. The ratchet advances to exact=82/diverges=0/gated=1,385.
- 2026-07-12: Draw-prefix replay disproved the `jellyfish_test` mipmap gate and
  isolated the missing radial-gradient background. Ported generated color-ramp
  rendering, gradient paint data/transforms, and nearest mip selection; five
  gradient GMs plus `jellyfish_test` advance the ratchet to
  exact=88/diverges=0/gated=1,379, with stale image allowances tightened.
- 2026-07-12: Swept all 38 remaining gradient-bearing `.riv` entries, captured
  30 runnable C++ references, and promoted 11 under unchanged tolerances. The
  precise gradient epsilon/clamp semantics are pinned; the ratchet advances to
  exact=99/diverges=0/gated=1,368 and `new_text` is the next residual.
- 2026-07-12: Classified `new_text` through draw-prefix, connected-component,
  support-mask, and solid-paint controls. Its 44 sparse compound-text edge
  pixels fit a bounded 48-pixel backend allowance; the ratchet advances to
  exact=100/diverges=0/gated=1,367 and `ai_assitant` is next.
- 2026-07-12: Classified `ai_assitant` through paired-draw prefixes,
  connected-component/support masks, and a full solid-paint control. Its 341
  almost entirely singleton stroke-edge pixels fit a bounded 384-pixel backend
  allowance; the ratchet advances to exact=101/diverges=0/gated=1,366 and
  `db_health_tracker` is next.
- 2026-07-12: Classified `db_health_tracker` through all 473 draw prefixes and
  connected components. Draws 1-430 are exact; its 1,071 residuals accumulate
  only across late text-outline edges and fit a bounded 1,152-pixel backend
  allowance. The ratchet advances to exact=102/diverges=0/gated=1,365 and
  `off_road_car` is next.
- 2026-07-12: Ported C++'s unique clip-generation IDs, removing stale root-clip
  coverage from all five identical `off_road_car` samples. The post-fix 1,862
  thin edge pixels fit a bounded 2,048-pixel backend allowance; the ratchet
  advances to exact=107/diverges=0/gated=1,360 and `joel_signed` is next.
- 2026-07-12: Routed clip-rect compound fills through clip-rect-specialized
  clockwise pipelines, promoted all five `joel_signed` frames, and advanced the
  renderer ratchet to exact=112/diverges=0/gated=1,355; `juice` is next.
- 2026-07-12: Ported shader-based advanced blending into the clockwise-atomic
  fill path, promoted all five `juice` frames, and advanced the renderer
  ratchet to exact=117/diverges=0/gated=1,350; `bad_skin` is next.
- 2026-07-12: Ordered generic-atomic outer/interior passes, added the exact
  `bad_skin` C++ preparation oracle, promoted its stable bounded residual, and
  advanced the renderer ratchet to exact=118/diverges=0/gated=1,349; the
  matching WebGPU MSAA final-blit oracle is next.
- 2026-07-12: Ported top-level MSAA `clipReset` for changing outer non-zero
  atlas clips; the nine-batch C++ frame is pixel-exact, all gates stay green at
  exact=118/diverges=0/gated=1,349, and nested clip intersection is next.
- 2026-07-12: Ported nested non-zero MSAA atlas clipping with exact C++ winding,
  intersection-reset, incremental-stack, and full-frame oracle parity; all
  gates remain green at exact=118/diverges=0/gated=1,349, and alternate clip
  fill rules are next.
- 2026-07-12: Ported alternate even-odd and clockwise MSAA atlas clip fills.
  Filled Dawn fixtures distinguish parity holes and opposite-winding rejection;
  commit `44bf47ea` keeps all gates green at
  exact=118/diverges=0/gated=1,349.
- 2026-07-12: Ported MSAA atlas destination-copy shader blending for solid
  feathered draws, including all 15 advanced modes, repeated bounded copies,
  attachment preservation, path clipping, and an exact C++ Dawn frame. The
  renderer ratchet remains exact=118/diverges=0/gated=1,349.
- 2026-07-13: Ported determinant-aware direct and atlas feather contour
  directions, promoted both mirrored feather-text GMs, and advanced the
  renderer ratchet to exact=146/diverges=0/gated=1,321.
- 2026-07-13: Isolated `interleavedfeather` to a ColorBurn-sensitive atomic
  intermediate-precision discontinuity, rejected and reverted destination
  texture and f16 color-plane experiments, and parked the case pending a C++
  color-plane suboracle or backend-matched reference. Promoted the independently
  verified `overstroke_blendmodes` reference under its unchanged 2/32 contract;
  the ratchet is exact=147/diverges=0/gated=1,320 and `zeroPath` is next.
- 2026-07-13: Pruned fully coincident cubics in stroke/feather preparation,
  matching C++ behavior and restoring `zeroPath` round/square caps.
  Fresh native Metal comparison passes the unchanged 2/32 contract at
  26 pixels/max-55; the ratchet is exact=148/diverges=0/gated=1,319 and
  `dstreadshuffle` is next.
- 2026-07-13: Isolated `dstreadshuffle` to the named intermediate-color
  precision boundary and parked it without tolerance changes. Promoted fresh
  `overfill_blendmodes` output at 7 pixels/max-3 under the unchanged 2/32
  contract; the ratchet is exact=149/diverges=0/gated=1,318 and
  `strokes_round` is next.
- 2026-07-13: Localized `strokes_round` to five unresolved foreground-support
  pixels at draw 38's smooth close seam and kept it gated for a pre-raster
  record oracle after Sol rejected a tolerance promotion. Promoted
  `overfill_opaque` under its independently proven 48-pixel cubic-edge
  allowance; all renderer and V2 gates are green at
  exact=150/diverges=0/gated=1,317.
- 2026-07-13: Built a record-exact C++ CPU tessellation-span oracle for
  `strokes_round` draw 38, ported C++'s five-segment non-round joins, full raw
  line tangents, and padding-before-geometry write order, and matched all 11
  spans/176 words. Fresh native output has zero pixels beyond delta 2; the
  unchanged `2/32` contract promotes the entry and advances the ratchet to
  exact=151/diverges=0/gated=1,316.
- 2026-07-13: Audited all 14 `strokefill` draws independently and promoted the
  edge-only 109-pixel residual under a bounded 128-pixel allowance, advancing
  the renderer ratchet to exact=152/diverges=0/gated=1,315. Renderer golden,
  both V2 golden floors, and the workspace tests pass; `rawtext` is next.
- 2026-07-13: Added a stream-derived C++ production oracle for `rawtext`,
  matched all 438 CPU spans and the complete tessellation texture exactly,
  ported four shared fill-preparation details, and promoted the sparse final
  raster residual under a bounded 288-pixel allowance. The renderer ratchet is
  exact=153/diverges=0/gated=1,314; renderer golden, the full workspace suite,
  and both V2 golden floors pass.
- 2026-07-13: Closed the required mid-R2 wgpu resource-seam audit. Added
  adapter-limit preflight for frame and image textures, bounded disjoint atomic
  batches at 65,535 paths, replaced oversized inseparable-run panics with a
  named error, and recorded the R3/R4 boundaries without changing the renderer
  ratchet.
- 2026-07-13: Closed the direct `feather_cusp` structural mismatch with C++'s
  fixed-color generic-atomic face and clockwise paint encoding. Added exact
  C++ atomic-coverage capture plus same-backend final-blit oracles, preserved
  authored clipped fill rules, and kept advanced feather blending green after
  Sol review. Native Metal comparison is bounded at 9,480 pixels/max delta 11;
  promotion advances the renderer ratchet to
  exact=154/diverges=0/gated=1,313. Renderer golden, both V2 golden floors,
  and the full workspace suite pass.
- 2026-07-13: Added the exact-source C++ Dawn atomic ColorBurn pair oracle for
  `interleavedfeather` draws 13-14, including test-only Rust/C++ color and
  coverage plane readbacks. It exposed and fixed generic feathered-clockwise
  paint preparation and advanced feather-fill face culling. Normalized raw
  coverage is exact; the only final difference is two coupled color
  words/pixels at max
  byte/channel deltas one and seven. Both remaining GMs stay gated pending
  independent full-stream C++ WebGPU references. The renderer ratchet remains
  exact=154/diverges=0/gated=1,313 with no tolerance or corpus edit.
- 2026-07-13: Added the pinned full-stream C++ Dawn WebGPU-on-Metal oracle for
  all 451 `interleavedfeather` draws. Rust passes its existing `2/32` contract
  at 6 over-threshold pixels; three-way native Metal comparison proves the
  remaining corpus gap is backend precision rather than algorithm core. The
  entry remains gated under the named backend boundary, with the renderer
  ratchet unchanged at exact=154/diverges=0/gated=1,313.
- 2026-07-13: Added pinned full-stream untouched and SrcOver-control C++ Dawn
  WebGPU-on-Metal oracles for all 97 `dstreadshuffle` draws. The untouched gate
  remains open at roughly 22.84k pixels over delta 2/max 61; changing only the
  97 paint blend modes to SrcOver passes three samples at 11-13 pixels over
  delta 2/max 4. Sol approved reclassifying the corpus diagnostic to the named
  shader-stack precision boundary while preserving gated status, native
  reference, tolerance, and renderer ratchet.
- 2026-07-13: Closed R2 at 106/108 passing clockwise-atomic upstream GMs plus
  two reviewed backend/compiler precision gates and zero remaining
  `algorithm-core` gates. No corpus tolerance or native reference changed.
  R3's semantic-trap audit and dual-renderer fuzz replay are now the active
  entry work.
- 2026-07-13: Pinned reproducible Rust and C++ shader compiler-input lineages,
  including upstream minifier determinism, exact artifact counts/digests, and
  a macOS CI gate. The ABI test now explicitly covers `ImageRectVertex`.
  Renderer tests pass 193/193 active cases and the corpus ratchet remains
  exact=154/diverges=0/gated=1,313 after Sol approval. The sampled nested clip
  plane and decoded-image bytes are the only open semantic-trap oracles.
- 2026-07-13: Closed the sampled nested clip-plane semantic fork with a
  zero-delta 640x640 native Metal versus Rust production readout and a Rust
  routing test that pins `OutermostClip`, `NestedClip`, and `ClippedContent`.
  The renderer ratchet advances to exact=155/diverges=0/gated=1,313; decoded
  image bytes are the only remaining semantic-trap oracle before fuzz replay.
- 2026-07-13: Closed decoded-image color ingress with a native raw-buffer
  oracle over the production C++ and Rust decode paths. The reachable JPEG
  differs at 35,652 pixels/78,669 channels, with 12,509 source pixels over
  delta 2, max delta 37, and exact alpha, confirming a decoder-level difference
  on the same rendered image. The ICC PNG differs at 4,950 pixels/5,013
  channels, max delta 2, exact alpha, proving color conversion is already
  within the corpus threshold.
  `make renderer-decoder-oracle` pins fixture and runtime provenance plus the
  bounded contracts; no corpus tolerance or reference changed. Dual-renderer
  fuzz replay is now the only remaining R3 entry gate.
- 2026-07-13: Closed the R3 dual-renderer fuzz-replay entry gate with five
  deterministic hostile-stream families, per-child wall deadlines, PNG and
  finite-control-region oracles, named C++/Rust pixel findings, and a macOS CI
  smoke target. The first absurd-stroke replay exposed a Rust debug-overflow
  panic; clamping segment arithmetic before integer conversion fixes it and a
  focused unit test pins the regression. Non-finite transforms and degenerate
  geometry are exact, deep clips stay within 21 pixels/max delta 1, and the
  absurd-stroke and invalid-gradient raster differences remain named
  out-of-contract findings. See `docs/renderer-fuzz-replay.md`.
