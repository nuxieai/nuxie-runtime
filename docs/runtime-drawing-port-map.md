# C++ runtime drawing port

This is the plain-language execution map for the work previously called the
“RD-C7 ownership boundary.” The old label remains only as an internal ticket
reference in historical status entries.

## Scope

We are changing the **runtime code that decides what to draw and owns the
objects used to draw it**:

- Artboard, Drawable, Shape, PathComposer, ShapePaintPath, and ShapePaint;
- ImageAsset, Image, Mesh, Text, Layout, nested artboards, and component lists;
- their construction, dirt/update, draw, clone, and drop behavior;
- the host/runtime cache seam that currently retains a second scene graph.

We are **not** changing the renderer that turns those objects into pixels:

- no wgpu or Dawn backend work;
- no shader, atlas, batching, tessellation, or GPU algorithm work;
- no renderer API redesign.

The boundary is:

`runtime objects and live traversal -> existing Renderer/RenderFactory API -> stop`

The 1,468-row pixel corpus is the referee, not the implementation target.

## Why the previous checklist was insufficient

`file-correspondence-manifest.toml` records one status for an entire C++ file.
That allowed a file to appear translated while some of its members still had a
different owner or lifecycle in Rust. For example, the Rust Shape path calls
`ensure_runtime_shape_paint_owner` during draw and obtains RenderPaint and
RenderPath from scene caches, while C++ performs path/effect/paint work in
component update and stores the backend objects on ShapePaint/ShapePaintPath.

The binding completion artifact is now
`docs/runtime-drawing-ownership.toml`: one row per state-bearing member or
ownership edge, including its full lifecycle.

## Ordered replacement batches

Each batch begins from the named C++ owners. It replaces the whole owner
lifecycle and deletes that family’s old cache representation in the same
landing. A batch that leaves both ownership systems alive is incomplete.

| Order | Batch | C++ owners translated together | Old Rust state removed by that batch | Merge proof |
|---:|---|---|---|---|
| 1 | Shape, path, and paint | `Shape`, `PathComposer`, `ShapePaintPath`, `ShapePaint`, paint mutators, effects, Feather | draw-time `ensure_runtime_shape_paint_owner`; Shape entries in global paint/path/configuration tables; final-only effect snapshots; Shape path/paint epochs | member ledger closed for the batch; unchanged draw performs no CPU composition/effect/paint configuration; pixels and runtime floors green |
| 2 | Image and mesh | `ImageAsset`, `Image`, `Mesh`, `SliceMesh`, NSlicer resource ownership | ImageAsset and mesh resources retained by the outer scene paint cache | one resource per owner occurrence; clone/remount tests; pixels and floors green |
| 3 | Text | `Text`, `RawText`, `TextStylePaint`, `TextInputDrawable` | scene-owned Text paint pools and clip paths; generic replay adapters | preserve Text’s own `m_drawCommands`; delete only scene replay; pixels and floors green |
| 4 | Layout, nested artboards, and component lists | `LayoutComponent`, foreground proxy, mounted nested Artboard, `ArtboardComponentList` | layout/nested backend paths and child caches with facade lifetime | mounted child and layout resources have occurrence lifetime; pixels and floors green |
| 5 | Artboard and host facade | `Artboard::draw`, `drawInternal`, live list, frame origin, query ownership | prepared frames, `RuntimeDrawCommand`, scene path/paint caches, prepare/replay APIs, `ArtboardRenderCache`, replay-only C/C++ API plumbing | closed-mode structural checker; B-6 rerun; full battery; size under 9 MiB |

The dependency order is machine-checked from the ownership ledger. Production
edits in the monolithic runtime draw path remain serialized.

## Per-batch mechanical loop

1. Select the batch’s open ownership rows.
2. Read the complete C++ headers and sources for those owners.
3. Translate every row’s construct, update, draw, clone, and drop behavior.
4. Delete the corresponding old cache representation in the same batch.
5. Run the structural checker and lifecycle tests.
6. Run renderer pixels plus the runtime regression floor.
7. Run two adversarial reviews:
   - ownership, clone, and drop;
   - dirt/update order and draw-time reads.
8. A repeated finding changes `docs/PORTING.md` or the ledger, then the batch
   is regenerated. It is not patched independently across call sites.

Every finding cites a pinned C++ line and a rule. C++ ownership wins when an
idiomatic Rust design disagrees.

## Structural judges

Run:

```sh
make runtime-drawing-port-check
```

This verifies:

- the pinned C++ checkout;
- every ownership row’s four lifecycle phases and citations;
- the batch dependency graph;
- every C++→Rust gap decision and rule;
- exact status counts;
- non-increasing counts for every legacy scene-cache symbol.

The current pre-removal state is allowed to contain explicitly counted open
rows and legacy symbols. Final removal runs:

```sh
make runtime-drawing-port-closed
```

That command fails unless:

- all ownership rows are `exact` or `adapted`;
- no row is `pending` or `compensation`;
- every legacy scene-cache ratchet is zero.

The checker’s unit suite includes a negative control that inserts
`RuntimeRenderPathCache` into an otherwise closed draw source and proves the
gate rejects it.

## Required lifecycle tests

The runtime test factory must prove, per migrated owner family:

- construction creates each backend resource once;
- an unchanged second draw creates, composes, shapes, and configures nothing;
- one property dirt update rebuilds exactly once at the C++ update site;
- draw reads settled state;
- clone creates fresh derived state and backend slots;
- drop/remount cannot reuse stale resources;
- a deliberately failing factory cannot unbalance renderer save/restore.

Negative controls must also prove the judges reject:

- missing dirt propagation;
- a swapped resource owner;
- a global-id or scene-cache lookup reintroduced into live draw;
- a copied visibility/transform snapshot;
- an old and new ownership path alive together.

## Stop rules

- No implementation begins for a C++ member without a ledger row.
- No owner is split across construct/update/draw/clone/drop batches.
- No batch lands with both the old cache owner and the new object owner.
- No `faithful` file status while any member row is open.
- No paragraph-long workaround; add a gap rule or stop.
- Two findings of the same class stop the batch and repair the rulebook/checker.
- No renderer-backend edits in this project slice.
- No tolerance, corpus expectation, or gate threshold changes.

## Current checkpoint

The ownership ledger, gap inventory, checker, and disposable Shape/path/paint
translation stress test are prerequisites. They change no production drawing
code and remove no scene cache. The ordered replacement batches begin only
after this map and the stress-test resolutions are reviewed.
