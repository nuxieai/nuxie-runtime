# Product crate seams

Status: transitional package contract. No runtime behavior moves in this step.

UNIV-1623 gives each upper layer an explicit Cargo package before code moves
across packages or repositories. The packages expose the current types without
wrappers, so the new and legacy paths have identical type identity, ordering,
errors, and behavior.

```text
nuxie-authoring --------+
nuxie-product ----------+----> nuxie mixed facade ----> baseline crates
                        |
nuxie-browser-adapter --+----> nuxie-renderer
nuxie-apple-adapter ----+----> nuxie-renderer
```

This is a migration shape, not the final dependency graph. `nuxie` is still a
mixed compatibility facade while Scene and Flow live there. The pure-runtime
ratchet already classifies it as migration debt and prevents protected baseline,
portable-ABI, replay, oracle, fuzz, golden, and performance packages from adding
an upward dependency on any package in this document.

## Package ownership and interface

| Package | Owns during migration | Direct workspace dependency | Deliberately does not expose |
|---|---|---|---|
| `nuxie-product` | Shared product execution and the Flow protocol | `nuxie` with defaults disabled | Renderer/device internals or an Apple ABI |
| `nuxie-authoring` | Scene/SceneTx as one deep authoring module | `nuxie` with defaults disabled | A second runtime scene facade or product host policy |
| `nuxie-browser-adapter` | Browser canvas presentation | `nuxie-renderer` on wasm only | `wgpu`, device, queue, surface, or texture state |
| `nuxie-apple-adapter` | Apple drawable presentation | `nuxie-renderer` on Apple only | Objective-C, Metal, `wgpu`, device, queue, or texture state |

The browser and Apple interfaces re-export only the existing high-level
factory/frame or surface lifecycle. The moves in UNIV-1625 and UNIV-1626 must
deepen those packages behind the same interfaces; exposing renderer internals
to make the move easier is not permitted.

## Build selectors

Each cut has a named selector:

```sh
make crate-seams-baseline-check
make crate-seams-product-check
make crate-seams-browser-check
make crate-seams-apple-check
make crate-seams-full-check
```

The browser selector targets `wasm32-unknown-unknown` so it cannot pass by
compiling away wasm-only code. The Apple selector is run in the macOS CI tier.
The full selector is the ordinary whole-workspace build.

## Temporary compatibility paths

Existing callers continue to compile during the migration:

- `nuxie::flow_session::*` is identical to `nuxie_product::*` until UNIV-1630;
- `nuxie::*` Scene exports and `nuxie::authoring::*` are identical to
  `nuxie_authoring::*` until UNIV-1627;
- `nuxie_renderer::{BrowserFactory, BrowserFrame, BrowserResizeError}` is
  identical to `nuxie_browser_adapter::*` until UNIV-1625;
- `nuxie_renderer::{AppleSurface, ApplePresentationCompletion,
  SurfaceDisposition, SurfaceError}` is identical to
  `nuxie_apple_adapter::*` until UNIV-1626.

These are temporary re-exports, not duplicate adapters. Later tickets move the
implementation once and remove the lower compatibility path after all callers
have switched.
