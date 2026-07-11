# Phase R Status

The execution contract is `docs/renderer-port-map.md`. This file records only
current evidence, open gates, and decisions needed by the next session.

## Metric

Run `make renderer-golden`.

- Rust wgpu: exact=2, diverges=0, gated=1, total=3.
- Stub baseline: exact=0, diverges=3, gated=0, total=3.
- Exact: `first-light-rectangle-clockwise-atomic` (GM stream) and
  `artboardclipping-frame-0-clockwise-atomic` (real `.riv` stream).
- Gated: `first-light-triangle-clockwise-atomic`; the geometry and interior
  pixels match, but C++ analytic edge coverage emits thirds while bootstrap
  4x MSAA emits quarters on 112 edge pixels.

## Milestones

- [ ] R0: Pixel golden harness. Parser/replay, PNG comparator, artifacts,
  manifest ratchet, checked-in references, stub baseline, and CI are landed.
  Remaining: capture the upstream non-ORE GM registry as streams, generate the
  complete `.riv` stream inventory, and expand `corpus-r.toml` to both modes.
- [x] R1: wgpu foundation and first light. Device/queue/offscreen readback,
  retained render-api objects, state stack, generated WGSL validation, 4x MSAA
  bootstrap coverage, one GM stream, and one real `.riv` stream are exact.
- [ ] R2: Algorithm core.
- [ ] R3: Corpus convergence.
- [ ] R4: Performance parity.
- [ ] R5: Native fast paths and extensions; demand-gated after R4.

## Next

1. Complete R0 GM capture with a recording `TestingWindow`; gate ORE/direct
   RenderContext GMs by named feature rather than porting their scene code.
2. Port `gpu.cpp` data contracts and `draw.cpp` path processing. The first
   convergence target is analytic edge coverage for the triangle fixture.
3. Expand corpus entries as features become replayable. Do not tune broad
   tolerances around missing algorithm work.

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
