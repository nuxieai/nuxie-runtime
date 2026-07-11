# Rive Renderer Port Map (Phase R)

Working directory: `/Users/levi/dev/rive-rust`

Reference renderer: `/Users/levi/dev/oss/rive-runtime/renderer`

Companion to: `docs/porting-map-v2.md` (V2). Phase R begins only after V2's
M7 exit criteria are met and the user activates it. Until then this document
is a plan, not a work queue. Once activated, the `/goal` ground rules apply
verbatim — port code not behaviors, a-posteriori verification, corpus-driven
priority, weeds tripwires — with the renderer-specific amendments below.

## Goal

A Rust implementation of the Rive Renderer's vector algorithm running on
`wgpu`, shipped behind the existing `rive-render-api` trait seam, side by side
with the C++ FFI renderer. The FFI renderer remains available as the proven
fallback for as long as it is useful; files flip to the Rust renderer
per-corpus-entry as they pass.

## The Layer Analysis (what gets ported and what does not)

The C++ renderer is a three-layer cake. The port keeps one layer.

**Ported — the algorithm layer (~14k C++ + ~12k shader lines):**

```text
renderer/src/render_context.cpp      # frame orchestration, flush logic, resource rings
renderer/src/draw.cpp                # path → tessellation → triangle-patch pipeline
renderer/src/gr_triangulator.cpp     # interior triangulation (Skia lineage)
renderer/src/gpu.cpp                 # GPU data layouts, uniform structs, enums
renderer/src/intersection_board.cpp  # overlap-aware draw batching/reordering
renderer/src/rive_renderer.cpp       # rive::Renderer implementation (state stack, clip)
renderer/src/rive_render_path.cpp    # retained path objects
renderer/src/rive_render_paint.cpp   # retained paint objects
renderer/src/gradient.cpp            # gradient color-ramp management
renderer/src/rive_render_factory.cpp # factory
renderer/src/sk_rectanizer_skyline.cpp # atlas packing
renderer/src/shaders/                # GLSL sources + generated WGSL/MSL/HLSL/SPIR-V
```

**Not ported — the HAL:** `renderer/src/ore/` (Rive's WebGPU-shaped
abstraction over Metal/GL/D3D11/D3D12/Vulkan/WebGPU) and the legacy
per-backend `RenderContextImpl`s (`src/gl`, `src/metal`, `src/vulkan`,
`src/d3d11`, `src/d3d12`, `src/webgpu`), plus `glad`, `rive_vk_bootstrap`,
and platform bootstrap. `wgpu` replaces this entire layer. ORE's API shape
(Buffer/Texture/TextureView/Sampler/ShaderModule/BindGroupLayout/BindGroup/
Pipeline/RenderPass/Context) maps nearly one-to-one onto wgpu types — when
porting algorithm-layer code that touches GPU resources, translate ORE/impl
concepts to wgpu directly rather than recreating an abstraction between them.

**Render modes.** The C++ renderer selects among draw modes by hardware
capability: raster-ordering modes (pixel local storage / fragment-shader
interlock / rasterizer-ordered views), atomic mode, clockwise-atomic mode,
and MSAA. wgpu does not expose raster-order guarantees, so the port targets:

- `clockwiseAtomic` — primary mode (storage-buffer atomics; what the C++
  WebGPU backend runs, so the algorithm provably works on this API class).
- `msaa` — fallback mode.

Raster-order modes are explicitly out of scope until #R-5 native fast paths,
and only if profiling justifies them.

## Verification Model

Pixel comparison, never stream comparison — the V2 golden-stream harness
verifies the runtime up to the renderer boundary; Phase R verifies pixels on
the other side of it.

- **Reference images** come from the C++ renderer executing a recorded
  render-call stream. **Test images** come from the Rust renderer executing
  the same stream. Comparing stream-replay against stream-replay isolates the
  renderer from the runtime completely.
- **GM corpus without porting GMs.** The C++ repo has a Skia-style GM suite
  (`tests/gm/`, ~84 scene files) with golden images and `image_diff`
  tooling. Do not port GM scene code. Instead: run each C++ GM once through
  the existing `RecordingRenderer` to capture its draw stream, check the
  streams in as fixtures, and replay them through both renderers. GMs
  exercise renderer features (strokes, joins, triangulations, bicubics,
  blend modes) far beyond what the `.riv` corpus reaches.
- **`.riv` corpus end-to-end.** Every corpus file also renders through the
  full Rust runtime + Rust renderer at the same samples/scripts as
  `corpus.toml`, diffed against C++ runtime + C++ renderer pixels.
- **Tolerances are per-backend and perceptual, never bit-exact.** The C++
  renderer's own backends do not produce identical pixels to each other, and
  their golden tooling already encodes per-backend thresholds. Mirror that
  model: a small max-per-channel delta plus a bounded count of differing
  pixels, tuned per (backend, mode). Chasing bit-exactness across GPUs or
  across draw modes is the Phase R equivalent of V1 pinning — a tripwire.
- **The metric** is the count of `pixel-exact-within-tolerance` entries in a
  new `corpus-r.toml` (GM streams + riv files × modes × backends), tracked in
  the status file exactly like the V2 exact count.

## Execution Strategy: Incremental vs Big-Bang (decide at activation)

**Decision (2026-07-10): incremental.** The user explicitly activated Phase R
after V2 completion. The default R0-R5 sequence remains authoritative: establish
the independent pixel oracle first, keep the corpus ratchet green, and port the
algorithm behind the existing render-api boundary. The big-bang variant remains
documented but inactive.

Added 2026-07-09, informed by Bun's Zig→Rust migration
(https://bun.com/blog/bun-in-rust): 1,448 files mechanically translated in 11
days by ~64 parallel agents — translate everything (tree deliberately
broken), burn the compiler-error list as a work queue with per-task
implementer → 2 adversarial split-context reviewers → fixer pipelines, then
converge on a language-independent test oracle.

Phase R qualifies for that strategy: a bounded file set (~26k lines of
algorithm layer + shaders), mechanical-translation viability, and a strong
independent oracle (#R-0 pixel goldens + the FFI renderer as control group).
The V2 single-writer rule is a property of the always-green ratchet — during
a big-bang translation phase there is no green to protect, so it is replaced
by Bun-style worktree discipline (per-file commits only; no `git stash`/
`reset`; no slow commands in workers) until convergence, when the ratchet
resumes as the gate.

Big-bang variant of the tickets: #R-0 unchanged (the oracle comes first
either way) → translate ALL algorithm-layer files in parallel (PORTING.md
idiom codex as the shared brief) → compiler-error work queue with
implementer/reviewer/fixer pipelines across worktrees → first-triangle smoke
→ GM-stream pixel convergence (#R-3) → perf (#R-4). Expected wall-clock:
days rather than weeks, at materially higher token cost. Choose at
activation based on budget and appetite; the incremental R0–R5 path below
remains the default.

## #R-0: Pixel Golden Harness

Blocked by: V2 M7 + user activation

### Deliverables

1. Stream-replay support in the C++ golden runner: load a recorded stream,
   execute it against the real Rive Renderer into an offscreen target, write
   a PNG.
2. GM stream capture: build the C++ GM suite against `RecordingRenderer`,
   capture one stream per GM, commit as fixtures.
3. `tools/pixel-compare`: perceptual diff with per-backend/mode tolerance
   config, failure artifacts (side-by-side + heatmap images), CI wiring.
4. `corpus-r.toml`: GM streams + `.riv` corpus entries × {clockwiseAtomic,
   msaa} × available backends, each with status and tolerance.
5. Reference image generation for the full manifest via the C++ renderer.

### Exit Criteria

CI renders references, diffs against a stub Rust renderer (all failing), and
reports the metric. Failure artifacts are inspectable.

## #R-1: wgpu Foundation And Shaders

Blocked by: #R-0

### Deliverables

1. `crates/rive-renderer`: wgpu device/queue/offscreen-target setup,
   implementing the `rive-render-api` traits end to end (factory, retained
   paths/paints/images, renderer state stack).
2. Shader ingestion: start from the C++ build's generated WGSL (their shader
   Makefile already emits `.wgsl` via SPIR-V) for the clockwise-atomic and
   MSAA pipelines; validate through naga; commit as generated artifacts with
   a regeneration script. Hand-maintained WGSL source is a later cleanup,
   not a prerequisite.
3. Resource plumbing translated from the algorithm layer's needs: uniform
   buffer ring, per-flush descriptor/bind-group layout mirroring
   `gpu.cpp`'s struct layouts, texture/atlas allocation, buffer mapping
   strategy.
4. First light: a single solid-color path rendered through the
   clockwise-atomic pipeline, passing its pixel golden.

### Exit Criteria

One GM stream and one trivial `.riv` entry pass within tolerance on the
primary development backend.

## #R-2: Algorithm Core Port

Blocked by: #R-1

Port order follows the dependency chain; each item is a coarse class/file
translation with a source-file reference comment, landed behind the trait and
judged by pixel goldens:

1. `gpu.cpp` — data layouts, enums, uniform structs, math helpers.
2. `draw.cpp` — the tessellation/patch pipeline: curve flattening budgets,
   patch vertex generation, stroke geometry (caps/joins), feather geometry.
3. `render_context.cpp` — frame lifecycle, logical flushes, resource ring
   management, draw-batch assembly, atlas scheduling.
4. `gr_triangulator.cpp` — interior triangulation for large/complex paths.
5. `intersection_board.cpp` — overlap detection and batch reordering.
6. `gradient.cpp` + color-ramp texture management.
7. `rive_renderer.cpp` — state stack, clipping (clip stack → clip buffers),
   `computeAlignment`, image/mesh draws.
8. Blend modes, including advanced-blend shader paths.
9. `sk_rectanizer_skyline.cpp` — atlas packing (feather/image atlases).

Rust-idiom notes carried over from the V2 performance guidance: retained
objects own arena state rather than `Arc`-counting; per-frame allocations
reuse buffers; no `RefCell` graphs; hot loops iterate slices.

**Mid-R2 adversarial review (added 2026-07-11, Bun lesson).** The V2 audits
proved bugs concentrate in INVENTED seams, not translated code. Phase R's
invented seam is the wgpu resource/binding layer that replaced ORE. Before
R2 exits, run a split-context assume-it's-wrong review of that plumbing —
bind-group lifecycles, buffer reuse/rewind, readback synchronization,
pipeline caching — plus the stream-replay glue. Findings fixed or explicitly
accepted with rationale, same as the M8 audit protocol.

### Exit Criteria

The GM stream corpus passes within tolerance in clockwise-atomic mode on the
primary backend; no `.riv` regression versus #R-1.

## #R-3: Corpus Convergence

Blocked by: #R-2, plus two entry criteria (added 2026-07-11, Bun lessons —
Bun's 19 regressions were all cross-language semantic traps found in
production; ours get found before convergence is declared):

- **GPU semantic-trap audit**: enumerate and check the GLSL→WGSL/naga
  divergence surface (uniformity analysis, texture/sampler semantics, float
  precision/denormals, matrix packing/majorness, integer wrap in shaders,
  MSAA resolve rules, coordinate conventions) against the ported pipelines —
  the pixel-domain analog of the M8 semantic-trap sweep. Findings ranked,
  fixed or accepted with rationale.
- **Renderer fuzz-replay harness**: replay degenerate/fuzzed streams (NaN
  and huge transforms, zero-area paths, absurd stroke widths, deep clip
  stacks, hostile gradient stops) through BOTH renderers; assert the Rust
  renderer never panics, hangs, or loses the device, and record behavioral
  deltas vs C++ as named findings. Extends the M8 fuzz harness's negative-
  input discipline into the pixel domain; wire a smoke gate into CI.

1. Full `corpus-r.toml` sweep: all GM streams and all `.riv` entries, both
   modes, all backends available in CI.
2. Divergence ladder, GPU edition: pixel heatmap → identify the draw batch →
   replay a truncated stream up to that batch → single-patch reproduction →
   read the two implementations side by side. GPU captures (Metal frame
   capture / RenderDoc, mirroring the C++ repo's `renderdoc/` tooling) are
   the equivalent of V2's stream bisection.
3. Per-file flip: a corpus entry passing on all tested backends flips its
   production renderer flag from FFI to Rust in the manifest.
4. MSAA-mode parity for entries whose hardware tier requires it.
5. Vendor-quirk findings become Decision-log entries with tolerance
   adjustments, never per-behavior pins.

### Exit Criteria

Metric at 100% of non-gated `corpus-r.toml` entries on CI backends; each
remaining gated entry has a named diagnostic (feature, mode, or documented
vendor quirk).

## #R-4: Performance Parity

Blocked by: #R-3

1. Port/adapt the `path_fiddle` benchmark scenes as the benchmark suite;
   measure frame time and flush counts, Rust-on-wgpu versus C++ on the same
   backend and mode.
2. Close gaps in order of measurement: batching parity first (flush counts
   and draw counts should match C++ almost exactly — the intersection board
   and atlas decisions are deterministic), then CPU-side encode cost, then
   GPU occupancy.
3. Output a per-scene comparison report in CI so regressions are visible.

### Exit Criteria

Frame times within an agreed factor of the C++ renderer per scene on the
primary backends, with flush/draw counts matching.

## #R-5: Native Fast Paths And Extensions (gated)

Blocked by: #R-4; each item requires profiling evidence or corpus demand

- Raster-order modes (fragment-shader interlock / PLS / ROV) via `wgpu-hal`
  escape hatches or a dedicated native backend behind the same trait — only
  where #R-4 shows the atomic mode leaves real performance on specific
  hardware.
- WebAssembly/WebGPU target — wgpu makes this nearly free; gate on an actual
  embedding need.
- RSTB shader-asset consumption (editor-exported `wgsl`/`spirv` targets map
  directly onto wgpu) — gate on scripted-shader/3D corpus content, aligned
  with V2's scripting gate.

## Long-Tail Strategy (renderer edition)

Identical philosophy to V2, with the GPU-specific amendments:

1. Pixel goldens are the oracle; GM streams cover renderer features the
   `.riv` corpus never reaches.
2. Never chase bit-exactness across GPUs, backends, or draw modes —
   tolerance config plus a Decision entry is the correct fix for a vendor
   difference; a per-behavior pin is not.
3. The escalation ladder ends in a single-patch reproduction, not a contract
   doc.
4. Unsupported render features fail loudly with a named diagnostic in
   `corpus-r.toml`, exactly like V2's import diagnostics.
5. The FFI renderer is the permanent control group: any suspected Rust
   renderer bug is first replayed through the FFI renderer to confirm which
   side owns it.
