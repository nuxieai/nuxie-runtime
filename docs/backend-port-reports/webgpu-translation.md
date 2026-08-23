# WebGPU translation closeout

Date: 2026-08-23

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN; the ordered WebGL2 translation queue may open after this
barrier is checkpointed**

## Closed denominator

- 30/30 semantic WebGPU, Wagyu, and ORE WGPU source owners have unique,
  current translation receipts. The two remaining WebGPU inventory rows,
  Wagyu `.clang-format` and `README.md`, are frozen nonsemantic evidence and
  are deliberately rejected by the translation-receipt gate.
- 20/20 semantic WebGPU ownership units are complete in the frozen SCC and
  dependency order.
- 18,318/18,318 pinned semantic source lines have compiled source-shaped
  targets and byte-exact in-repository source snapshots.
- Component 094's two-file `load_store_actions_ext` owner is also complete as
  the exact graph-required WebGPU prerequisite. The pinned upstream WebGPU
  build compiles that GL-owned implementation directly. The translation gate
  now admits only the transitive frozen prerequisite closure of the active
  queue, so this does not open unrelated WebGL2 owners or permit phase jumps.
- The complete WebGPU render context includes capability and adapter
  admission, exact generated GLSL assembly, every pipeline family, buffer and
  storage-texture rings, texture and render-target owners, PLS/atomic/MSAA draw
  passes, draw-list execution, flush and command submission, Canvas and ORE
  construction, and explicit reverse ownership teardown.
- The complete ORE WGPU context includes device and queue ownership, feature
  admission, shader compilation, concrete buffer/texture/sampler/pipeline/
  bind-group/render-pass factories, dynamic offsets, mapping and submission,
  and native handle teardown.
- Wagyu's C and C++ bindings retain the pinned ABI surface, including the
  exact advanced-blend extension values, chained structures, handle wrappers,
  and JavaScript compatibility/build inputs.
- 51 generated GLSL artifacts used by the renderer owner are frozen as exact
  bytes: 17 minified programs, 17 generated headers, and 17 export maps.
- The focused WebGPU mechanical suite passes 79/79. The full WebGPU-enabled
  renderer library suite passes 569 tests with 40 intentionally ignored, and
  the no-default-feature WebGPU configuration compiles.

## Replayable gate

```text
cargo test -p nuxie-renderer --lib --no-default-features --features native-webgpu-experimental mechanical_port::webgpu
cargo test -p nuxie-renderer --lib --no-default-features --features native-webgpu-experimental
cargo check -p nuxie-renderer --no-default-features --features native-webgpu-experimental
python3 tools/backend-port/check_translation.py --repo-root . --upstream-root /Users/levi/dev/oss/rive-runtime --manifest docs/backend-port-campaign.toml
```

The translation gate adds no shipping selection, automatic fallback,
fixture-selected behavior, or shared backend abstraction. Global
source-semantics and ownership/lifetime/ABI reviews remain deferred until the
WebGL2 translation closes. The legacy Rust-WGPU renderer remains intact until
all three campaigns and every closeout gate pass.
