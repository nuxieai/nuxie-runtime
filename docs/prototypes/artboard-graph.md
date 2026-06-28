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
- `on_added_clean`: project C++ artboard host registries for exact `NestedArtboard`, `NestedArtboardLeaf`, `NestedArtboardLayout`, and `ArtboardComponentList` objects, exposing `nested_artboards`, `component_lists`, and the combined `artboard_hosts` list without running nested-artboard update, component-list layout, cloning, or advance behavior.
- `on_added_clean`: project C++ joystick registration facts, exposing `joysticks`, `joysticks_apply_before_update`, resolved custom handle sources, resolved x/y animations, and nested remap dependents collected from keyed animation targets without running `Joystick::apply`, component updates, data-bind scheduling, or animation advancement.
- `on_added_clean`: project C++ reset and advance lifecycle registries, exposing `resetting_components` and `advancing_components` from the exact `ResettingComponent::from` and `AdvancingComponent::from` switches without running `reset()`, `advanceComponent()`, data-bind advancement, component updates, or frame scheduling.
- `build_dependencies`: project C++'s per-`Shape` synthetic `PathComposer` node as a graph fact, with its owning shape and registered paths sourced from `rive-binary`'s imported shape list. The synthetic node participates in `dependency_nodes`, `dependency_node_edges`, and `dependency_node_order`; the compatibility `dependency_order` field remains a filtered real-component local-ID order.
- `build_dependencies`: derive C++ parent dependency, targeted-constraint, IK-target, IK chain child, draw-target, draw-rule, clipping-source, skinned `Mesh`/`PointsPath`, tendon-bone, skin peer-constraint-parent, Joystick custom-handle, scroll-constraint-to-scroll-bar, scroll-constraint-to-layout-child, path-composer, clipping-shape path-composer, follow-path, text-follow-path, text variation helper, stroke/fill/feather path-builder, effect-parent, and linear/radial gradient paint-container dependency-node edges, order dependency nodes over the explicit edge list, expose a real-component filtered order, and surface dependency-cycle diagnostics. C++ parent dependency edges are emitted only when an audited `buildDependencies()` implementation calls `parent()->addDependent(this)`, not for every structural parent/child relation. C++ override exceptions are modeled where audited: `Skin::buildDependencies`, `TargetedConstraint::buildDependencies`, `Joystick::buildDependencies`, `ClippingShape::buildDependencies`, `FollowPathConstraint::buildDependencies`, `TextFollowPathModifier::buildDependencies`, `Stroke::buildDependencies`, `Fill::buildDependencies`, `Feather::buildDependencies`, and `LinearGradient::buildDependencies` do not call their parent dependency implementation, while `IKConstraint::buildDependencies` reuses the targeted edge and adds its own target-to-constraint edge. `GroupEffect::buildDependencies` and `ScriptedPathEffect::buildDependencies` preserve inherited dependencies and add explicit effect-parent dependencies.
- `build_dependencies`: project exact C++ list-constraint registration facts by recording `ListFollowPathConstraint` children whose parent is an `ArtboardComponentList`, matching `ConstrainableList::addListConstraint` without running `constrainList()`, list layout, or virtualization.
- `post_build_dependencies`: project C++ initialized `m_Drawables` as a static `drawable_order` list, including imported drawables, `ForegroundLayoutDrawable` reordering before its parent, inherited flattened draw-rule owner IDs, and layout `DrawableProxy` insertion. `sortDrawOrder()`, active draw-target linked-list mutation, clipping-stack operations, draw-rule target ordering, renderer commands, and GPU work remain out of scope.

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
- `on_added_clean`-style artboard host projections: `nested_artboards`, `component_lists`, and the combined `artboard_hosts` list mirror C++ `m_NestedArtboards`, `m_ComponentLists`, and `m_ArtboardHosts` for exact nested-artboard host variants and exact `ArtboardComponentList` objects. Runtime host updates, nested-artboard cloning, component-list layout, and advance scheduling remain out of scope.
- `on_added_clean`-style joystick projections: `joysticks` mirrors C++ `m_Joysticks`, `joysticks_apply_before_update` mirrors the artboard scheduling flag derived from `Joystick::canApplyBeforeUpdate`, and each `JoystickNode` records resolved handle source, x/y animation globals, and `Joystick::addDependents` nested remap targets in C++ y-then-x order. Joystick application, animation advancement, data-bind scheduling, and component update passes remain future runtime work.
- `on_added_clean`-style reset and advance projections: `resetting_components` mirrors C++ `m_Resettables` for exact nested-artboard host variants, `ArtboardComponentList`, and `CustomPropertyTrigger`, while `advancing_components` mirrors C++ `m_advancingComponents` for exact `Artboard`, nested-artboard host variants, layout/list/scroll/text/scripted objects admitted by `AdvancingComponent::from`. `Artboard::reset()`, `Artboard::advanceInternal()`, `advanceComponent()`, data-bind advancement, component updates, and frame scheduling remain future runtime work.
- `build_dependencies`-style dependency nodes for real imported components plus synthetic path composers and text variation helpers. Every imported `Shape` gets one `PathComposerNode` and one synthetic `DependencyNode`, keyed by its shape local/global IDs and the registered path local/global IDs that C++ would make prerequisites of the path composer. Every imported `TextStyle` descendant with `TextStyleAxis` or `TextStyleFeature` children gets one synthetic text variation helper dependency node.
- `build_dependencies`-style dependency edges for C++ parent dependency hooks, targeted constraints, IK constraints, IK chain off-branch children, draw targets, draw rules, clipping sources, skinning, Joystick custom-handle updates, scroll-constraint-to-scroll-bar links, scroll-constraint-to-layout-child links, clipping-shape path-composer links, follow-path target/parent links, text-follow-path target/text links, text variation helper links, stroke/fill/feather path-builder links, explicit `GroupEffect`/`ScriptedPathEffect` parent links, and linear/radial gradient paint-container links. `ParentChild` edges are deliberately narrower than structural `children`: they cover audited C++ parent dependency hooks such as `TransformComponent`, `Constraint`, `TextStyle`, `FocusData`, `SemanticData`, and `NSlicer`, while import-only graph records such as `DrawTarget`, `DrawRules`, `AxisX`, and `TextValueRun` retain hierarchy without dependency-order edges. Targeted-constraint edges now mirror C++ by making the constrained component depend on the target, rather than making the constraint object itself the dependent, and targeted constraints no longer receive the generic parent-child edge because the C++ override does not call `Super::buildDependencies`. `IKConstraint::buildDependencies` adds the extra target-to-constraint edge, and its `onAddedClean` IK-chain walk is modeled by making off-chain transform children of ancestor bones depend on the constrained tip bone. IK solving and transform dirt propagation remain future runtime work. `Skin::buildDependencies` is modeled by omitting the generic parent-child dependency for the skin child, making exact C++ skinnables (`Mesh` and `PointsPath`) depend on their skin, and making skins depend on tendon bones plus IK peer constraint parents discovered from `parentBoneCount`. `Joystick::buildDependencies` is modeled by omitting the generic parent-child dependency for all joysticks and adding parent-to-joystick plus handle-source-to-joystick edges only when `handleSourceId` resolves to an artboard-local `TransformComponent`. `ScrollBarConstraint::buildDependencies` is modeled by adding the resolved `ScrollConstraint` as a prerequisite while preserving the inherited parent dependency. `ScrollConstraint::buildDependencies` is modeled by making exact C++ `LayoutNodeProvider` children of the scroll content depend on the scroll constraint. `ClippingShape::buildDependencies` is modeled by making each clipping source shape's synthetic `PathComposer` dependency node a prerequisite of the clipping shape and by omitting the generic parent-child edge because the override does not call `Super::buildDependencies`. `FollowPathConstraint::buildDependencies` is modeled by making target shape/path composers prerequisites of the constraint, falling back to a direct path prerequisite for shape-less paths, and making the constrained parent depend on the constraint; `ListFollowPathConstraint` preserves those inherited edges. `TextFollowPathModifier::buildDependencies` is modeled by making target shapes' synthetic `PathComposer` nodes or direct target paths prerequisites of the modifier and making the owning `Text` depend on the modifier, without admitting text shaping or glyph layout. `TextVariationHelper::buildDependencies` is modeled by adding `artboard -> helper -> text` edges when imported text style axis/feature children cause C++ to allocate the helper; `TextVariationHelper::update()` and variable-font mutation remain future text runtime work. `Stroke::buildDependencies` is modeled by resolving C++ `ShapePaintContainer::pathBuilder()` as either a shape's synthetic `PathComposer` node or a real path-builder component. Effect-bearing `Fill::buildDependencies` reuses the same path-builder resolution only when registered stroke effects exist, and `Feather::buildDependencies` uses the owning paint container directly, with shapes again routed through their synthetic path composers. `LinearGradient::buildDependencies`, inherited by `RadialGradient`, is modeled by making gradients depend on the first owning `Node` above their shape paint, or the immediate paint container when no node exists; its `updateDeformer()` call remains future runtime/deformer state. Generic parent-child dependencies are deliberately skipped for the audited no-super shape/paint/effect families (`ClippingShape`, `Fill`, `Stroke`, `Feather`, `DashPath`, `TargetEffect`, `TrimPath`, `GroupEffect`, `ScriptedPathEffect`, `LinearGradient`, and `RadialGradient`) and for imported records that do not add C++ dependency edges so the graph does not overstate C++ build-dependency inheritance. None of these admit render-paint mutation, effect execution, IK solving, transform dirt propagation, deformer updates, text shaping, variable-font mutation, or draw-command behavior.
- `build_dependencies`-style list constraint registrations for exact `ListFollowPathConstraint` children whose parent is an exact `ArtboardComponentList`, matching C++ `ConstrainableList::from`, `ListConstraint::from`, and `ConstrainableList::addListConstraint`. This remains a static graph registration; `ArtboardComponentList::updateConstraints()`, `ListFollowPathConstraint::constrainList()`, list layout solving, and virtualization remain future runtime work.
- `build_dependencies`-style topological dependency order over explicit dependency-node edges, plus a filtered real-component local-ID order for existing callers, with cycle diagnostics for both node chains and all-real-component local-id chains.
- Static drawable-order projection through `drawable_order`, matching C++ `Artboard::initialize()` for `m_Drawables.push_back(drawable)`, `ForegroundLayoutDrawable` parent-bound reordering, parent-chain flattened draw-rule assignment, and layout `DrawableProxy` insertion. Final `sortDrawOrder()`, clipping-stack proxy operations, active draw-target linked lists, draw-rule target ordering, draw commands, and rendering remain future work.
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

Known limit: dependency-node order now consumes the explicit edge list and reports cycles, but the edge list still covers only the first resolved runtime relationships. Ticket `#7` should continue expanding this toward the full C++ `buildDependencies()` surface, using synthetic dependency nodes for path composers and text variation helpers as the starting point for layout, data-binding, remaining scroll/layout, remaining text, and the remaining paint/effect runtime edge families beyond the covered `Stroke`, effect-bearing `Fill`, `Feather`, `GroupEffect`, `ScriptedPathEffect`, and linear/radial gradient dependency edges. `drawable_order` now covers initialized C++ `m_Drawables`; Ticket `#10` should add `sortDrawOrder()`, clipping-stack proxy operations, active draw-target linked lists, draw-rule target ordering, and a renderer-independent draw command stream.
