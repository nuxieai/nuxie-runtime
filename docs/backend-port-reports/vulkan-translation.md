# Vulkan translation closeout

Date: 2026-08-23

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN; the ordered WebGPU translation queue may open after this
barrier is checkpointed**

## Closed denominator

- 40/40 Vulkan and ORE Vulkan source owners have unique, current translation
  receipts.
- 22/22 Vulkan ownership units are complete in the frozen SCC and dependency
  order.
- 14,691/14,691 pinned logical source lines have compiled source-shaped targets and
  byte-exact in-repository source snapshots.
- The complete Vulkan render context includes every buffer, texture, external
  target, texture-backed RenderCanvas, and ORE factory; every resource
  prepass; the full PLS, MSAA, offscreen, barrier, descriptor, pipeline, and
  draw topology; platform/vendor gates; async pipeline creation; and explicit
  reverse ownership teardown.
- The optional `RIVE_CANVAS` path is independently compiler-rooted by
  `native-ore-vulkan-experimental`. It does not select a product backend.
- The focused Vulkan mechanical suite passes 31/31 with the Canvas/ORE and
  tools predicates enabled.
- Native Metal ORE and Vulkan ORE compile together without collapsing their
  separately owned concrete contexts.

## Replayable gate

```text
cargo test -p nuxie-renderer --lib --no-run --no-default-features --features native-ore-vulkan-experimental,with-rive-tools
cargo test -p nuxie-renderer --no-default-features --features native-ore-vulkan-experimental,with-rive-tools mechanical_port::vulkan --lib
cargo check -p nuxie-renderer --no-default-features --features native-ore-metal-experimental,native-ore-vulkan-experimental
python3 tools/backend-port/check_translation.py --repo-root . --upstream-root /Users/levi/dev/oss/rive-runtime --manifest docs/backend-port-campaign.toml
```

The translation gate adds no shipping selection, fallback, fixture-selected
behavior, or premature cross-backend abstraction. Global source-semantics and
ownership/lifetime/ABI reviews remain deferred until Vulkan, WebGPU, and
WebGL2 translations all close. The legacy Rust-WGPU renderer remains intact
until all three campaigns and every closeout gate pass.
