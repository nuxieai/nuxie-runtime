# WebGL2 translation closeout

Date: 2026-08-23

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN; the ordered global source-review queue may open after this
barrier is checkpointed**

## Closed denominator

- 31/31 semantic WebGL2 renderer and ORE GL source owners have unique,
  current translation receipts. The ten remaining WebGL2 inventory rows are
  exact pinned-build exclusions: the native loader implementation, native
  EXT-PLS and read/write-texture PLS implementations, and seven Objective-C++
  ORE GL implementations that the Emscripten WebGL2 build does not compile.
- 17/17 semantic WebGL2 ownership units are complete in the frozen SCC and
  dependency order.
- 9,929/9,929 pinned semantic source lines have compiled source-shaped targets
  and byte-exact in-repository source snapshots.
- The complete GL render context includes extension and capability admission,
  shader/program construction, cached GL state, buffers, textures, render
  targets, Canvas/ORE construction, PLS, atomics, MSAA, draw and flush paths,
  readback, context loss, deferred release, and reverse ownership teardown.
- The complete ORE GL context includes concrete buffer, texture, sampler,
  shader, pipeline, bind-group, render-pass, and render-target behavior through
  one retained GL execution domain.
- The exact WebGL PLS owner preserves the four-state coherent-extension
  admission matrix, retained PLS and provoking-vertex browser contracts, all
  eight JavaScript bridges and C wrappers, plane bindings, load/store arrays,
  premultiplied clears, external-framebuffer pre-blit/copy-back ordering, and
  factory selection. The concrete browser executor remains deliberately owned
  by the later behavior/platform queue; translation does not claim browser
  product integration.
- The focused WebGL2 mechanical suite passes 78/78. The production renderer
  library compiles both with and without the tools predicate.
- The translation checker closes 188/188 semantic sources and 131/131 semantic
  ownership units across shared authority, Vulkan, WebGPU, and WebGL2.

## Replayable gate

```text
cargo test -p nuxie-renderer --lib mechanical_port::webgl2 --no-default-features --features native-webgpu-experimental,ore-gl,with-rive-tools
cargo check -p nuxie-renderer --lib --no-default-features --features native-webgpu-experimental,ore-gl,with-rive-tools
cargo check -p nuxie-renderer --lib --no-default-features --features native-webgpu-experimental,ore-gl
cargo fmt --all -- --check
git diff --check
python3 tools/backend-port/check_translation.py --repo-root . --upstream-root /Users/levi/dev/oss/rive-runtime --manifest docs/backend-port-campaign.toml
```

The translation gate adds no shipping selection, automatic fallback,
fixture-selected behavior, or shared backend abstraction. Source-semantics and
ownership/lifetime/ABI reviews now run as separate global passes. Explicit
WebGPU/WebGL2 editor selection, the concrete browser executor, and legacy
Rust-WGPU deletion remain later queues after all three exact ports pass frozen
closeout.
