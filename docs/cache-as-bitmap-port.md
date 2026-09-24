# Cache as bitmap (out-of-order port)

Authority: Rive commit `a4dbc3ffa50fa4e9c0346c5fdeddb4a664911cec` ("cache
components as bitmap", #13778). Work:
[UNIV-3544](https://universe.basis.dev/issue/UNIV-3544).

This commit is 91 commits past `LAST_SYNCED_SHA` (`5892bb05`). It was ported on
its own, ahead of the sync, and adapted onto the older base. The sync map
records the exception. The checkpoint does not move.

## What is ported

| Upstream owner | Rust owner |
| --- | --- |
| `bitmap_cache.hpp/.cpp` | `crates/nuxie-runtime/src/mechanical_port/source/bitmap_cache.rs` |
| `generated/bitmap_cache_base.hpp/.cpp` | `.../source/generated/bitmap_cache_base.rs` |
| `generated/core_registry.hpp` (BitmapCache arms) | `.../source/generated/core_registry.rs`; `crates/nuxie-schema/src/generated/schema.rs` (regenerated) |
| `artboard.hpp/.cpp` (`drawInternal`, `drawContent`, `drawCachedAsBitmap`, `renderIntoCanvas`, the `m_BitmapCache` import) | `.../source/artboard.rs` (`draw_internal_handle`, `draw_content_handle`, `draw_cached_as_bitmap_handle`, `render_into_canvas_handle`, `bitmap_cache`) |
| `renderer.hpp` (`currentTransform`, `currentModulatedOpacity`) | `nuxie-render-api` `Renderer` (both return `Option`); the mechanical `RendererContract` in `nuxie-renderer` |
| `rive_renderer.hpp` (both queries) | `rive_renderer_cpp.rs` `RendererContract for RiveRenderer`, forwarded by the exact-source adapters, the Metal, Vulkan, WebGPU and WebGL2 frames, the deferred session's scoped renderer, the Lua deferred canvas frame, and the replay frame wrappers (GM host, Apple and Android C API) |
| `factory.hpp` (`canvasContentHost`) | `nuxie-render-api` `Factory::canvas_content_host`, forwarded by both persistent proxies |
| `deferred_canvas_host.hpp` (`makeContentCanvas`, `contentCanvasImage`, `compositeRenderer`) | `nuxie-render-api/src/routing.rs` `DeferredCanvasHost` |
| `deferred_render_factory.hpp` (CTM shadow, `resetTransform`) | `deferred_render_factory.rs` `DeferredRenderer` |
| `deferred_session.hpp` (host methods, frame-boundary shadow reset) | `deferred_session.rs` `DeferredSession` |
| `tests/common/render_context_null.*` (`ensureCanvasBacking`) | `deferred/cmd/tests/render_context_null.rs` |
| `tests/common/testing_window_deferred_sink.hpp` (`canvasFrames`) | `deferred/gm/ore_gm_helper.rs` `GmHost::canvas_frames` |

## Adaptations

- **Schema.** Upstream removed `dev/defs` in `d4fe1022`, before this commit, so
  there is no upstream `bitmap_cache.json`. `defs/upstream-overlay/bitmap_cache.json`
  is reconstructed from the generated C++ header: type 136, `resolution` 417
  (double, 1.0), `cacheFlags` 418 (uint, 1), and passthrough bits
  `cacheEnabled` 419 (bit 0) and `dither` 420 (bit 1). The header does not
  record `animates`/`bindable`. They are set to true because the commit's own
  comments describe the cache being "animated/bound off". `make schema` now
  overlays these files on the pinned defs. It reproduces the committed schema
  byte for byte from `5892bb05` defs, which the hand-edited schema on main
  could not do.
- **Friend access.** Upstream's `Artboard` is a friend of `BitmapCache`. The
  Rust fields are `pub(crate)`, and the artboard reaches the cache through its
  `CoreHandle`. Content is drawn through `draw_content_handle` with no artboard
  or cache borrow held, so a nested cached artboard can re-enter safely.
- **Invalidation.** `BitmapCache::invalidate` marks the owning artboard
  changed through its dirty handle, the shared component path. It does not
  borrow the artboard.
- **Host methods.** They return `Option`, not a nullable pointer, and
  `composite_renderer` returns an owned `Box<dyn Renderer>`. Upstream's host
  keeps ownership of that renderer. Here the artboard drops it after the
  composite, and also immediately when it goes unused (no transform or opacity
  to carry). A host that implements it must tolerate both. No Rust host
  implements it yet. `DeferredSession`
  mints content canvases through its bound render context's
  `make_deferred_render_canvas`, the same route the Lua canvas uses.
- **Screen recorder reset.** Upstream downcasts each retained screen recorder
  to reset its shadow at `resetFrame`. Rust keeps the recorder type-erased, so
  the session holds each screen recorder's shadow beside it and resets that.
- **NaN.** C++ `std::min`/`std::max` propagate a NaN first argument; Rust's
  `f32::min`/`max` drop it. The resolution clamp uses C++-ordered helpers so a
  NaN resolution still falls back to the vector draw, as upstream relies on.
- **Renderers that cannot answer.** The host callback renderer in the C API,
  the FFI frame over a foreign renderer, and the recording/serializing
  renderers in `nuxie-render-api` report no transform, the equivalent of
  upstream's `false`. A cached artboard drawn through one of them is sized at
  device scale 1 and composited without a pixel snap.
- **Scoped content renderer.** The content renderer is dropped before
  `end_canvas_content`, as the Lua canvas does.

## Known limitations (shared with upstream)

- A raster is marked clean when it is recorded. If that frame's replay fails
  before the canvas is backed (a device loss, a poisoned replay), later static
  frames composite an unbacked canvas until the content changes. A canvas
  backed by one replay device also keeps that device's context alive after the
  host switches devices. Upstream has the same shape; the Rust reference
  counting makes the second point visible.
- Clearing `cacheEnabled` from inside the cached content's own draw (a data
  bind or a script) drops the canvas mid-bracket. That frame then draws the
  content again as vectors and wastes the recorded bracket.

## Not ported

- `utils/serializing_factory.*`, `utils/serialized_replay.*` and
  `serialize_ops.hpp` (cache capture in serialized silvers). These build on the
  unported `paintModulatedImage` op from `d9747935`. Without them a silver
  factory draws cached artboards as vectors, which matches upstream's default
  (its hooks are opt-in). The upstream "silver factory drives the cache only
  once enabled" case goes with them.
- `serialized_replay_test.cpp` additions, which belong to the same utilities.
- `tests/player/*`, `tests/deploy_tests.py`, `.rive_head`: C++ harness and editor pointers.
- `render_canvas_dag.cpp`: upstream only removes a GL-specific flip and moves
  its sink into shared test code. The Rust GM never had the flip, and
  `GmHost` already is that shared sink.
- The earlier cases in `modulate_opacity_test.cpp` have no Rust port yet. Only
  the new query case is added.

Intervening upstream commits that also touch these files are not ported:
`845a82a9`, `d4fe1022`, `c55840a8`, `d8727299` (the artboard watermark branch
in `draw`), `6951a4b3`, `85d7f952`, `d9747935`. The sync applies them in
order later.

## Qualification

```sh
make fixtures
cargo test -p nuxie-renderer --features renderer-metal --lib artboard_bitmap_cache_test
cargo test -p nuxie-renderer --features renderer-metal --lib bitmap_cache_pixel -- --test-threads=1
cargo test -p nuxie-renderer --features renderer-metal --lib modulate_opacity_test
cargo test -p nuxie-renderer --lib deferred_transform_shadow_test
make schema DEFS_DIR=<rive-runtime at 5892bb05>/dev/defs   # reproduces schema.rs
```

- **`artboard_bitmap_cache_test`**: 23 of the 24 upstream cases. The harness
  records into a `DeferredSession` bound to the null render context and counts
  rasters from each frame's content canvases and canvas segments, in place of
  upstream's `CountingSession` subclass. The GPU-work cases replay onto the
  null device, which now backs canvases (`ensureCanvasBacking`) and reports
  render-target area.
- **`bitmap_cache_pixel`**: all three upstream pixel cases on the live Metal
  backend, plus one added case. The added case records and replays two frames
  on one session and one device and checks that the second, which only
  composites the first frame's raster, matches the vector draw. Upstream's
  cases each use a fresh session, so they never cover that path. The cached composite matches the vector draw within one channel at
  1x and 2x. Resolution 0.5 differs on about 5% of pixels, matching upstream's
  measurement. The fractionally placed composite keeps the aligned render's
  gradient energy (ratio 1.0000).
