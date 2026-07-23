# RD-1 Renderer-feed restoration mini-map

Status: execution map committed before production changes.  The C++ oracle is
`rive-app/rive-runtime@d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

## Fixed retention boundary

RD-1 replaces this Rust scene-level ownership chain:

`host/artboard cache -> globally keyed prepared scene -> retained draw-command replay`

with the pinned C++ chain:

`Artboard live drawable list -> Drawable subclass -> object-owned render resources -> Renderer`

`RenderPath`, `RenderPaint`, `RenderImage`, mesh data, and Text's shaped draw
commands remain retained on the corresponding live object and are mutated during
ordinary dirt propagation.  Backend-private batching below `Renderer` is outside
RD-1.  Imported graph topology may seed object construction, but it must not remain
the per-frame rendering feed.

## Binding sequence

1. **RD-1a — this mini-map.** Land the correspondence, ownership seam, ordered
   lanes, and deletion inventory without changing production code.
2. **RD-1b — measured spike.** Add a non-default live per-frame traversal for a
   representative plain-shape/image/nested-artboard corpus slice.  Reuse the
   current render resources and delete nothing.  Run `make r4-timing-gate` and
   `make perf-hot-loop`, commit the measurements, and stop for the user to review
   the delta before any demolition.
3. **RD-1b2 — rulebook and stress translation.** Extend `docs/PORTING.md` with
   renderer-feed rules.  Translate `src/drawable.cpp`, `src/shapes/shape.cpp`,
   and `src/artboard.cpp:1606-1698` twice: once rulebook-strict and once as a
   senior Rust engineer.  Diff them, turn every disagreement into an explicit
   rule, and discard both translations before implementation fans out.
4. **RD-C1/RD-C2 performance checkpoint.** Remove the temporary command-
   materialization seam, rerun the measured comparison, and report the delta
   to the user before any scene-cache deletion.  This checkpoint cannot be
   self-cleared and blocks RD-C7 demolition only; additive RD-C3 through RD-C6
   may proceed while a quiet-host measurement is deferred.
5. Execute RD-C1 through RD-C7 in order.  C2 and C3 may investigate in parallel
   after C1's interfaces settle, but production edits in the monolithic Rust draw
   path serialize.  No adjacent lane may hide attribution by landing concurrently.

The #B-6 prerequisite work may proceed beside the measured spike because it is
documentation-only: add the RB-1 idiom mappings to `docs/PORTING.md`, and seed the
file-correspondence manifest.  RB-1 rows remain `pending-verification` until the
orchestrator's independent battery reports success; only that run may promote
them to `faithful`.

## Ordered file-correspondence lanes

| Lane | Pinned C++ draw/traversal sources | Rust landing area | Retention boundary crossed | Dependency / merge condition |
|---|---|---|---|---|
| **RD-C1: live drawable and order foundation** | `include/rive/drawable.hpp`, `src/drawable.cpp`; `src/draw_target.cpp`; `src/draw_rules.cpp`; `src/artboard.cpp:429-840,1159-1169` | retained runtime drawable handles/list links; live hidden/`willDraw` state; target placement; clip proxies and save elision; graph order only as construction seed | prepared/sorted-order frames and draw-order epochs -> object-owned linked draw order updated by ordinary `ComponentDirt::DrawOrder` | First implementation lane. Establishes the live interfaces required by every drawable family. |
| **RD-C2: Shape, paths, and paints** | `include/rive/shapes/shape.hpp`, `src/shapes/shape.cpp`; `src/shapes/path_composer.cpp`; `src/shapes/paint/shape_paint_path.cpp`; `src/shapes/paint/shape_paint.cpp`; `solid_color.cpp`, `linear_gradient.cpp`, `radial_gradient.cpp`, `stroke.cpp`, `fill.cpp`, `clipping_shape.cpp` | Shape-owned composer and local/world paths; paint-owned render paint; retained shape-paint paths; live Shape draw/`willDraw` | globally keyed path/paint/configuration caches -> per-object `RenderPath`/`RenderPaint` mutated in place under component dirt | Depends on C1. Freeze the retained path/paint interface before Text begins. Removing the command-materialization seam triggers the second user performance checkpoint; no scene-cache deletion may precede its review. |
| **RD-C3: Image and mesh** | `include/rive/assets/image_asset.hpp`, `src/assets/image_asset.cpp`; `src/shapes/image.cpp`; `src/shapes/mesh.cpp`; `src/shapes/slice_mesh.cpp` | asset-owned `RenderImage`; live Image draw/`willDraw`; object-owned mesh/slice buffers | central render-image and mesh slots -> live Image/ImageAsset/mesh resource ownership | Depends on C1. May be investigated beside C2, but shared draw/resource edits serialize. |
| **RD-C4: Text and TextInput** | `src/text/text.cpp:620-742,845-875`; `src/text/text_style_paint.cpp`; `src/text/text_input_drawable.cpp`; supporting `raw_text.cpp` and `raw_text_input.cpp` paths reached by those files | Text-owned shaped lines, style paths, pools, clip path, emoji cache, and `m_drawCommands`; live TextInput paths | epoch-keyed scene text reconstruction and generic text command replay -> Text-owned retained commands and style resources | Depends on C2. Preserve the C++ per-object `m_drawCommands` cache; it is not scene replay. |
| **RD-C5: layout and nested traversal** | `src/layout_component.cpp:317-374`; `src/foreground_layout_drawable.cpp`; `src/nested_artboard_leaf.cpp`; `src/nested_artboard_layout.cpp`; `src/nested_artboard.cpp:491-508` | live layout proxy/background/foreground drawing; mounted child identity; direct recursive child `drawInternal` | layout/nested prepared frames and cloned child snapshots -> mounted child object plus ordinary per-object layout paths and direct recursion | Depends on C1/C2 and the live traversal interface. Do not remove non-render geometry-query state merely because it shares an epoch name. |
| **RD-C6: remaining virtual family and Artboard cutover** | `src/artboard_component_list.cpp:977-980`; `src/scripted/scripted_drawable.cpp:312-315,343`; `src/artboard.cpp:1606-1698` | component-list and scripted live dispatch; complete `Artboard::draw`/`drawInternal` walk with lazy clipping and virtual draw | command-kind dispatch over a prepared tree -> one per-frame walk of the complete live drawable family | Depends on C1-C5. Cut over only after every reachable drawable kind has a live implementation. |
| **RD-C7: facade cutover and deletion gate** | C++ retention result established by the files above; no compensating C++ cache layer exists | reduce/remove `ArtboardRenderCache` and scene callers; migrate independently justified geometry state; remove replay-only exports and C API cache plumbing | external scene cache, prepared artboard frames, retained `RuntimeDrawCommand` streams, scene path/paint/text/nested caches, sorted/layout replay frames, and renderer epoch/revision bridges -> object-owned resources only | Last lane. Re-run #B-6 renderer clusters and remove D-12 only when zero scene-level mutation-gated mechanisms remain. |

## Oracle call chain and present Rust seams

The exact C++ order is:

1. `Artboard::buildDrawOrder` collects drawables and layout proxies
   (`src/artboard.cpp:429-510`), sorts draw-target dependencies
   (`530-569`), relinks before/after target groups (`575-666`), and
   interleaves clipping proxies plus save elision (`668-840`).
2. Ordinary update applies draw-order/clipping dirt
   (`src/artboard.cpp:1159-1169`).
3. `Artboard::draw` enters `drawInternal` (`src/artboard.cpp:1606-1651`).
4. `drawInternal` walks `m_FirstDrawable` each frame, invokes live
   `willDraw`, accounts for empty clips, starts clips lazily, and calls the
   drawable's virtual `draw` (`src/artboard.cpp:1652-1698`).

The present Rust counterparts being replaced are concentrated in
`crates/nuxie-runtime/src/draw.rs`: `RuntimePreparedArtboardFrame`,
`RuntimeDrawCommand`, `RuntimeRenderPathCache`, `RuntimeRenderPaintCache`,
prepared/sorted/layout/text/nested frames, command construction, and prepared
replay.  Their invalidation bridge is on `ArtboardInstance` in
`crates/nuxie-runtime/src/artboard.rs`; host-level ownership begins at
`ArtboardRenderCache` in `crates/nuxie/src/lib.rs`.  The existing
`nuxie-render-api` renderer contracts and backend command batching survive.

## Deletion inventory and preservation guardrails

RD-C7 deletes only the scene-level replay design:

- prepared artboard/frame ownership and retained `RuntimeDrawCommand` streams;
- scene-global path, paint, image, mesh, text-command, layout, sorted-order,
  and nested replay caches;
- cache/prepared/command/path/layout/text/draw-order/tree-paint epoch or
  revision bridges whose sole consumer is that replay design;
- outer cache APIs and call-site plumbing that exist only to retain/replay a
  prepared scene.

RD-C7 preserves:

- per-object `RenderPath`, `RenderPaint`, `RenderImage`, mesh buffers, Text
  draw commands/style paths, component-list ordering, and emoji images;
- independent geometry/layout query state with a non-render API consumer;
- imported topology used to construct the live object graph;
- renderer/backend-private batching beneath the renderer interface;
- ordinary dirt propagation and `did_change` mechanisms that correspond to
  C++ rather than compensate for scene replay.

## Lane ratchets

Every implementation lane must keep the 1,468-case renderer pixel corpus exact
and both ordinary and scripted golden gates at zero failures.  Renderer pixels
referee every RD merge; no tolerance, exclusion, or gate may be loosened.  Run
`make scripted-golden-compare` explicitly before every RD cut.  RD-C7 also runs
the full workspace/C API/probe floors, the #B-6 renderer audit, and both size
variants under the fixed 9 MiB (9,437,184 byte) limit.  After RD-C1/RD-C2
remove the materialization seam, a second measured performance report and user
review are mandatory before any scene-cache deletion.
