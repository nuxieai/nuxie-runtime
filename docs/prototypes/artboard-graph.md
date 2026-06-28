# Minimal Artboard Graph Prototype

Ticket: `#6`

Next seam contract: post-import graph/runtime work is governed by
[`graph-runtime-contract.md`](graph-runtime-contract.md). Use that document to
decide whether new work belongs in `rive-graph`, `rive-binary`, or a later
runtime crate.

Question: what is the smallest useful Rust implementation of the artboard runtime graph lifecycle?

Prototype command:

```sh
make graph
```

The command imports `fixtures/graph/dependency_test.riv`, projects the flat runtime object stream into C++-style file and artboard-owned collections, resolves `Component.parentId` references, builds child indexes, derives explicit dependency edges plus a topological dependency order, and exposes the first renderer-adjacent graph relationships without producing draw commands.

Lifecycle mapping:

- `import`: find serialized `Artboard` ranges, then compact them into the same shape as C++ `Artboard::objects()`: include component objects, artboard-owned user-input/interpolator objects, and null/abstract slots; exclude concrete non-components routed to other runtime lists.
- `import`: project C++-owned collections for `File::assets()`, file view models, data enums, `Artboard::animation(i)`, and `Artboard::stateMachine(i)` from the decoded object stream. File-level assets, view models, and data enums are projected from `rive-binary`'s public `RuntimeFile` collection helpers so graph does not duplicate import-stack ownership rules.
- `on_added_dirty`: resolve local `parentId` values against the artboard-local component index.
- `on_added_dirty`: resolve nullable draw target, draw rule, and clipping source references while preserving the raw serialized IDs for missing-target diagnostics.
- `on_added_dirty`: mirror the C++ initialization quirk that creates an `"Auto Generated State Machine"` when an artboard has neither animations nor explicit state machines.
- `on_added_clean`: build derived child lists and clipping relationships for source shape descendants and clipped drawable descendants.
- `build_dependencies`: project C++'s per-`Shape` synthetic `PathComposer` node as a graph fact, with its owning shape and registered paths sourced from `rive-binary`'s imported shape list. The synthetic node is not yet inserted into the real-component dependency order because it has no serialized artboard-local slot.
- `build_dependencies`: derive parent-child, targeted-constraint, IK-target, draw-target, draw-rule, clipping-source, skinned `Mesh`/`PointsPath`, tendon-bone, skin peer-constraint-parent, Joystick custom-handle, scroll-constraint-to-scroll-bar, and scroll-constraint-to-layout-child dependency edges, order components over the explicit edge list, and surface dependency-cycle diagnostics. C++ override exceptions are modeled where audited: `Skin::buildDependencies`, `TargetedConstraint::buildDependencies`, and `Joystick::buildDependencies` do not call their parent dependency implementation, while `IKConstraint::buildDependencies` reuses the targeted edge and adds its own target-to-constraint edge.

Verdict: the local-ID model now matches the first C++ graph expectations and the C++ probe output for the starter fixture set.

Implemented:

- `crates/rive-graph` as a pure projection over `rive-binary::RuntimeFile`.
- Artboard-local object slots matching C++ `Artboard::objects()`, preserving null/abstract slots, preserving C++ validation null slots such as invalid targeted constraints, text styles, nested animations, scroll-bar constraints, and feather effects, and excluding serialized records that C++ routes to other lists.
- File-level asset, view-model, and data-enum collection projections matching the C++ probe on reference fixtures, backed by `RuntimeFile::file_assets()`, `view_models()`, and `data_enums()` for C++ importer parity.
- Artboard animation and state-machine projections from `rive-binary` public helpers, including linear-animation keyed-object grouping and state-machine layer/input/listener/action/listener-input-type/data-bind grouping.
- C++ auto-generated empty state-machine projection for artboards without authored animations or state machines.
- Global-to-local relationship through `LocalObject { local_id, global_id }`.
- Component detection through generated schema inheritance.
- Capability flags for artboard, container, world transform, transform, and drawable categories.
- `on_added_dirty`-style parent resolution from local `parentId`.
- `on_added_dirty`-style draw graph relationships: `DrawTarget.drawableId` to nullable drawable local ID, `DrawRules.drawTargetId` to nullable active draw target local ID, and `ClippingShape.sourceId` to nullable source node local ID.
- `on_added_clean`-style child indexing.
- `on_added_clean`-style clipping shape projections: source shape locals from the clipping source subtree and clipped drawable locals from the clipping shape parent subtree.
- `build_dependencies`-style synthetic path composer projections: every imported `Shape` gets one `PathComposerNode` keyed by its shape local/global IDs and the registered path local/global IDs that C++ would make prerequisites of the path composer.
- `build_dependencies`-style dependency edges for parent-child links, targeted constraints, IK constraints, draw targets, draw rules, clipping sources, skinning, Joystick custom-handle updates, scroll-constraint-to-scroll-bar links, and scroll-constraint-to-layout-child links. Targeted-constraint edges now mirror C++ by making the constrained component depend on the target, rather than making the constraint object itself the dependent, and targeted constraints no longer receive the generic parent-child edge because the C++ override does not call `Super::buildDependencies`. `IKConstraint::buildDependencies` adds the extra target-to-constraint edge. `Skin::buildDependencies` is modeled by omitting the generic parent-child dependency for the skin child, making exact C++ skinnables (`Mesh` and `PointsPath`) depend on their skin, and making skins depend on tendon bones plus IK peer constraint parents discovered from `parentBoneCount`. `Joystick::buildDependencies` is modeled by omitting the generic parent-child dependency for all joysticks and adding parent-to-joystick plus handle-source-to-joystick edges only when `handleSourceId` resolves to an artboard-local `TransformComponent`. `ScrollBarConstraint::buildDependencies` is modeled by adding the resolved `ScrollConstraint` as a prerequisite while preserving the inherited parent dependency. `ScrollConstraint::buildDependencies` is modeled by making exact C++ `LayoutNodeProvider` children of the scroll content depend on the scroll constraint.
- `build_dependencies`-style topological dependency order over explicit dependency edges, with cycle diagnostics for cyclic local-id chains.
- `graph-inspect` JSON CLI.
- `tools/cpp-probe` C++ oracle plus `make cpp-compare`, which validates artboard object counts, local type keys, component names, serialized parent IDs, resolved parent local IDs, draw target/rule links, and clipping relationships against C++.

Focused `dependency_test.riv` result:

```text
Blue -> A -> B -> {C, Rectangle}
Rectangle -> Rectangle Path
B children = [C, Rectangle]
dependency order starts [Blue, A, B, C, Rectangle, Rectangle Path]
```

Starter corpus result:

```text
fixtures/minimal/long_name.riv artboards=1 names=New Artboard objects=7 components=7
fixtures/minimal/two_artboards.riv artboards=2 names=Two,One objects=21 components=18
fixtures/graph/clipping_and_draw_order.riv artboards=2 names=Artboard,child objects=53 components=53
fixtures/graph/dependency_test.riv artboards=1 names=Blue objects=17 components=16
fixtures/graph/draw_rule_cycle.riv artboards=1 names=New Artboard objects=29 components=25
fixtures/animation/smi_test.riv artboards=2 names=main artboard,artboard to nest objects=13 components=13
fixtures/animation/state_machine_transition.riv artboards=1 names=Artboard-Test objects=57 components=57
```

Probe result:

```sh
make cpp-compare
```

This builds `tools/cpp-probe`, imports the starter fixtures with the C++ runtime, and compares the compact artboard object arena, component hierarchy, draw graph relationships, animation/state-machine grouping, and file-level asset/view-model/enum collections against Rust. The C++ runtime library can be built without the renderer/Metal toolchain from the reference root with:

```sh
RIVE_PREMAKE_ARGS="--file=premake5_v2.lua --with_rive_text --with_rive_layout" ./build/build_rive.sh
```

Known limit: dependency order now consumes the explicit edge list and reports cycles, but the edge list still covers only the first resolved runtime relationships. Ticket `#7` should continue expanding this toward the full C++ `buildDependencies()` surface, using the path-composer projection as the starting point for follow-path, text, layout, data-binding, remaining scroll/layout, and paint/effect runtime edge families. Ticket `#10` should add final drawable ordering, clipping-stack proxy operations, and a renderer-independent draw command stream.
