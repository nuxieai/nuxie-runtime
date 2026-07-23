# C++ runtime drawing port — disposable translation stress test

Date: 2026-07-23
C++ source: `rive-runtime@d788e8ec6e8b598526607d6a1e8818e8b637b60c`

## Scope and method

Two independent, read-only translations covered the headers and sources for:

- `ShapePaintPath`;
- `ShapePaint`;
- `Shape`;
- the coupled `PathComposer`, paint-mutator, effect, and Feather ownership
  needed to make those three files complete.

Translation A followed `docs/PORTING.md` strictly. Translation B approached
the same source as a senior Rust engineer. Neither translation wrote or
retained production code. Their output was a disposable type/API and lifecycle
sketch only.

The comparison included every state-bearing member plus construction,
membership/dependency building, dirt/update, draw, clone, and drop. It did not
compare only final pixels.

## Shared conclusion

Both translations independently rejected the current Shape representation.
Rust presently has clone-owned records named as owners, but still:

- stores RenderPaint and RenderPath in facade/scene caches;
- constructs a command-like `RuntimeOwnedShapePaint` snapshot from several C++
  objects;
- runs path, effect, paint, visibility, and transform settlement through
  `ensure_runtime_shape_paint_owner` reached from draw;
- omits `PathComposer` as an executable synthetic update node;
- derives Clone over dirty/retained runtime state that C++ reconstructs.

Both proposed real runtime Shape, PathComposer, ShapePaintPath, ShapePaint,
paint-mutator, and effect objects, with stable typed ids for C++ non-owning
pointers and one-to-one backend resource slots.

## Disagreements and binding resolutions

| Question | Rulebook-strict translation | Senior-Rust translation | Resolution |
|---|---|---|---|
| Where does Shape world transform live? | On the inherited live Component/Drawable object, exactly as C++ | Proposed a `world_transform` field on `RuntimeShape` | Strict wins. It is inherited state, not a Shape member. Copying it creates a second source of truth. **RF-34**. |
| How physically store `Box<dyn RenderPath/RenderPaint>`? | One-to-one slot attached to the exact owner occurrence; typed table permitted only as physical storage | Inline `OwnedRenderPath`/`OwnedRenderPaint` fields | Logical C++ ownership is mandatory; physical inline versus typed table is an allowed Rust adaptation only when identity, clone, and drop follow the owner. No global id, scene key, epoch, or facade lifetime. **RF-30**. |
| What may wait for the first factory-bearing call? | Only backend allocation; CPU update and dependency work never waits | Suggested a factory-bearing render-resource update phase and allowed a legacy advance to leave work pending | Only the backend object may wait. CPU memberships, path composition, effects, visibility, transform, and settled paint state run at C++ update sites. First materialization applies settled state once. **RF-28/RF-29**. |
| Does `Shape::draw` receive a factory? | C++ receives only Renderer; Rust factory access is tolerated solely for C++'s lazy `ShapePaintPath::renderPath` materialization | Proposed a `RuntimeDrawContext` with factory and renderer | The context is allowed, but the factory cannot authorize CPU `ensure_*` work or RenderPaint reconstruction in draw. **RF-28/RF-29**. |
| What owns effect paths? | Each StrokeEffect owns one EffectPath per PathProvider identity | Listed effect ids on ShapePaint but did not initially require the provider-keyed owned map | C++ pair identity wins. A final-only paint snapshot is rejected. **RF-35**. |
| How are clones produced? | New generated objects, then construction/add/dependency phases; derived state and backend slots start fresh | Recommended custom reconstruction rather than derived Clone | Agreement after comparison. The complete lifecycle is atomic, and backend slots clone empty. **RF-28/RF-31**. |
| Are current “owner” snapshots acceptable after their backend fields move? | No; selected path, feather state, visibility, transform, and paint state belong to different live C++ objects | Initially used owner structs in the deletion list, then proposed separate typed runtime objects | The member ledger, not the struct name, decides. Command-shaped multi-owner snapshots are deleted. **RF-27/RF-34**. |
| Can geometry query state remain in `RuntimeGeometryCache`? | Yes only after it no longer embeds the render scene cache; C++ bounds/length return to Shape | Preserve query state if it does not feed production draw | Agreement after defining the split. **RF-32**. |

Every disagreement above produced or sharpened RF-27 through RF-35 in
`docs/PORTING.md`.

## Newly proven missing details

The strict pass found that `nuxie-graph` already represents and orders
PathComposer dependency nodes, while `ArtboardInstance` reduces its runtime
update list to authored Components. That omission is more fundamental than a
missing cache invalidation: the C++ update object does not execute at all.

The senior pass independently demonstrated the consequence through the
desired draw signature. A closed Shape draw needs live runtime object tables,
Renderer, and at most owner-local lazy backend allocation. If it still accepts
`RuntimeFile`, immutable graph records, layout bounds, a global paint table,
scene path cache, configuration epochs, or a prepared command, the C++ owner
boundary has not been reached.

Both passes also found that the current representation can copy mutable state.
In particular, authored/imported paint visibility and live visibility must not
be combined; C++ has one generated live property.

## Shape batch acceptance assertions

The first production replacement batch is rejected unless all are true:

1. PathComposer participates in the runtime dependency/update order.
2. CPU path composition happens in PathComposer update, not draw.
3. Effects update under ShapePaint Path dirt with per-effect/provider identity.
4. ShapePaint owns or uniquely reaches its one-to-one RenderPaint slot.
5. ShapePaintPath owns or uniquely reaches its one-to-one RenderPath slot.
6. Clone rebuilds memberships, dependencies, derived paths, and empty backend
   slots rather than copying retained snapshots.
7. Shape draw reads live visibility, inherited transform, and ordered paint
   ids.
8. Shape and ShapePaint draw call graphs contain no scene cache, global-id
   resource lookup, epoch, prepared frame, `RuntimeDrawCommand`, or CPU
   `ensure_*`.
9. An unchanged second draw performs no CPU composition, effect evaluation,
   or paint configuration.
10. The Shape family’s legacy ratchets reach zero in the same landing.

## Disposition

Both translations were discarded. Only the rule changes, ownership/gap
ledgers, dependency order, and acceptance assertions above survive.
