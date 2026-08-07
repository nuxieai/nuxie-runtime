# Product crate seams

Status: physical package and workspace-layering contract. Scene authoring now
lives in nuxie-dev.

UNIV-1623 gives each upper layer an explicit Cargo package before code moves
across packages or repositories. The packages expose the current types without
wrappers, so the new and legacy paths have identical type identity, ordering,
errors, and behavior.

```text
nuxie-dev authoring ----+----> nuxie baseline facade ----> baseline crates
nuxie-product crate ----+
nuxie-project-data -----+----> neutral external-data seam in nuxie-runtime
nuxie-product-scripting-+
                        |
nuxie-dev browser adapter +----> nuxie-renderer
nuxie-ios Apple adapter +----> nuxie-renderer
```

Flow now lives in `nuxie-product`; Nux artifact trust, the private Nuxie Luau
module, ordered host effects, and their quotas live in
`nuxie-product-scripting`. The shipping `nuxie` facade contains no Flow module
or product dependency; it owns the neutral host-extension interface and an
opaque exact-byte capability consumed during baseline VM setup.
Transactional runtime mechanics cross that boundary through
`ArtboardTransaction`: dropping an uncommitted operation rolls back its effect
checkpoint, and state batches receive only an opaque validated candidate to
commit. Listener-table adoption, DataContext rehoming, hydration, and source
synchronization remain baseline implementation details rather than a sequence
coordinated by `nuxie-product`.
ProjectDO's value model, program compiler, evaluator, and adapter live in
`nuxie-project-data`; the baseline owns only the product-neutral runtime-value,
program, state, resolver, and registry interfaces used by the bind graph.
Scene/SceneTx, generated authoring vocabulary,
lowering, transactions, export, remounting, stable identity, and authored
observations are owned by nuxie-dev's `nuxie-authoring` crate. Protected
baseline, portable-ABI, replay, oracle, fuzz, golden, and performance packages
may not add an upward dependency on an authoring or product package.

The product crates are permanent upper-layer members of this workspace, not a
second repository or release unit. Crate boundaries preserve the dependency
direction: protected parity/runtime crates cannot depend on product crates;
product crates may depend inward on the public runtime seams they need. A
separately named product ABI may expose experience/session/product operations,
but it does not extend or wrap `nux-capi` with product vocabulary.

## Workspace provider and release contract

`nuxie-runtime` is the single source revision, lockfile, and release unit for
the baseline and its optional product crates. The workspace contract is:

- product crates remain explicitly named, unprotected upper-layer packages;
- baseline, portable-ABI, replay, oracle, fuzz, golden, and performance
  packages never depend upward on them;
- product crate sources cannot import Apple UI/Metal or renderer/wgpu
  implementation APIs;
- consumers pin one qualified `nuxie-runtime` revision rather than coordinating
  a second repository revision; and
- editor authoring/browser code and Apple surfaces, C ABI, XCFramework
  assembly, and Swift packaging remain in `nuxie-dev` and `nuxie-ios`
  respectively.

## Package ownership and interface

| Package | Owns | Direct workspace dependency | Deliberately does not expose |
|---|---|---|---|
| `nuxie-product` | Shared product execution and the Flow protocol | `nuxie` with defaults disabled | Renderer/device internals or an Apple ABI |
| `nuxie-project-data` | ProjectDO value model, program compiler/evaluator, encoded artifact envelope, and adapter registration | `nuxie-runtime` with defaults disabled | Baseline bind-graph internals, editor authoring, Flow, or platform ABI policy |
| `nuxie-product-scripting` | Nux package vocabulary, exact-artifact verification, private Luau module, host effects, and product quotas | `nuxie`, `nuxie-scripting`, and `nux-container` | Rive bytecode validation, VM memory/safepoints, or imported Rive bindings |
| nuxie-dev `nuxie-authoring` | Scene/SceneTx as one deep authoring module | imported `nuxie` with defaults disabled plus binary test-support construction | A second runtime scene facade or product host policy |
| nuxie-dev `nuxie-browser-adapter` | Browser canvas presentation | pinned `nuxie-renderer` and `nuxie-render-api` on wasm only | `wgpu`, device, queue, surface, or texture state |
| nuxie-ios `nuxie-apple-adapter` | Apple drawable presentation and trusted-image admission | the pinned `nuxie-renderer` on Apple plus Objective-C/Metal platform bindings | `wgpu`, renderer device/queue objects, or texture state |

The browser and Apple interfaces re-export only the existing high-level
factory/frame or surface lifecycle. The Apple package is built in the
nuxie-ios native workspace and consumes the runtime repository only through
the public opaque `WgpuMetalPresenter`; exposing renderer internals to make
cross-repository ownership easier is not permitted.

## Build selectors

Each cut has a named selector:

```sh
make crate-seams-baseline-check
make crate-seams-product-check
make crate-seams-browser-check
make crate-seams-apple-check
make crate-seams-full-check
```

The product selector compiles and runs the product-owned scripted-listener
lifecycle suite. It also checks that `nuxie-product --no-default-features
--features scripting` does not recover `js-host-seed` through a transitive
dependency; this is the self-contained publisher-wasm profile. The browser
selector targets `wasm32-unknown-unknown` so it cannot pass by compiling away
wasm-only code. The Apple selector is run in the macOS CI tier. The full
selector is the ordinary whole-workspace build.

## Compatibility closeout

The temporary compatibility paths are removed. Product consumers import the
Flow protocol through `nuxie_product::flow_session`; the crate root does not
flatten that vocabulary. The scripted-listener lifecycle suite is owned and
compiled by `nuxie-product`, so baseline `nuxie` no longer includes product
source even in its test build.

The former `nuxie::flow_session` shipping path remains removed. Product
consumers must depend on `nuxie-product` directly.

Forty-five lifecycle cases moved with the product owner through public host
seams. The one concrete `FileScriptArtboard` trigger-consumption case remains
with the baseline facade's private unit tests, where it can exercise that
implementation detail without product source inclusion.

UNIV-1627 completed the authoring cut, and UNIV-1788 removed the remaining
test vocabulary debt: the runtime workspace no longer owns an authoring
package or exports Scene symbols. `nuxie-binary` exposes neutral synthetic
fixture construction only through its non-default `test-support` feature; its
default shipping interface remains byte-import only. Hidden aliases retain
source compatibility for editor revisions pinned before UNIV-1788 without
allowing authoring vocabulary back into protected runtime consumers.

UNIV-1795 completed the physical browser cut: `BrowserFactory`, `BrowserFrame`,
`BrowserResizeError`, canvas attachment, and bounded recovery are owned only by
nuxie-dev's `nuxie-browser-adapter`, against the exact runtime gitlink pinned by
that repository. Nuxie-runtime retains its WebGPU renderer, opaque presentation
surface/frame primitives, and backend/parity smoke; it exposes no raw wgpu
device, queue, surface, or texture state to the browser owner.

UNIV-1792 completed the Apple repository cut: surface lifecycle, CAMetalDrawable
validation, presentation scheduling/completion, failure disposition, and
trusted-image admission are owned only by nuxie-ios's
`native/nuxie-apple-adapter`. The renderer
retains an opaque `WgpuMetalPresenter` for final blit and shared device health;
the portable `nux-capi` package has no Apple feature, and runtime size tooling
retains only renderer-owned measurement roots.
