# Product crate seams

Status: physical package contract. Scene authoring now lives in nuxie-dev.

UNIV-1623 gives each upper layer an explicit Cargo package before code moves
across packages or repositories. The packages expose the current types without
wrappers, so the new and legacy paths have identical type identity, ordering,
errors, and behavior.

```text
nuxie-dev authoring ----+----> nuxie baseline facade ----> baseline crates
nuxie-product ----------+
                        |
nuxie-browser-adapter --+----> nuxie-renderer
nuxie-apple-adapter ----+----> nuxie-renderer
```

`nuxie` remains mixed only while Flow lives there. Scene/SceneTx, generated
authoring vocabulary, lowering, transactions, export, remounting, stable
identity, and authored observations are owned by nuxie-dev's
`nuxie-authoring` crate. Protected baseline, portable-ABI, replay, oracle,
fuzz, golden, and performance packages may not add an upward dependency on an
authoring or product package.

## Package ownership and interface

| Package | Owns during migration | Direct workspace dependency | Deliberately does not expose |
|---|---|---|---|
| `nuxie-product` | Shared product execution and the Flow protocol | `nuxie` with defaults disabled | Renderer/device internals or an Apple ABI |
| nuxie-dev `nuxie-authoring` | Scene/SceneTx as one deep authoring module | imported `nuxie` with defaults disabled plus binary test-support construction | A second runtime scene facade or product host policy |
| `nuxie-browser-adapter` | Browser canvas presentation | `nuxie-renderer` and `nuxie-render-api` on wasm only | `wgpu`, device, queue, surface, or texture state |
| `nuxie-apple-adapter` | Apple drawable presentation and trusted-image admission | `nuxie-renderer` on Apple plus Objective-C/Metal platform bindings | `wgpu`, renderer device/queue objects, or texture state |

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
This is a temporary re-export, not a duplicate adapter. Later tickets move the
Flow implementation once and remove the lower compatibility path after all
callers have switched.

UNIV-1627 completed the authoring cut: the runtime workspace no longer owns an
authoring package or exports Scene symbols. `nuxie-binary` exposes authored
record construction only through its non-default `test-support` feature; its
default shipping interface remains byte-import only.

UNIV-1625 completed the browser cut: `BrowserFactory`, `BrowserFrame`, and
`BrowserResizeError` are owned only by `nuxie-browser-adapter`. The renderer
retains opaque presentation surface/frame primitives and exposes no raw wgpu
device, queue, surface, or texture state.

UNIV-1626 completed the Apple cut: surface lifecycle, CAMetalDrawable
validation, presentation scheduling/completion, failure disposition, and
trusted-image admission are owned only by `nuxie-apple-adapter`. The renderer
retains an opaque `WgpuMetalPresenter` for final blit and shared device health;
the portable `nux-capi` package has no Apple feature or measurement roots.
