# Graph Runtime Completion Matrix

This document is the operating checklist for the active `rive-graph` goal. It
turns the broader graph contract into a finite audit so the port can finish this
slice without letting every C++ runtime helper pull the work into frame
execution.

## Scope Statement

`rive-graph` parity means static post-import artboard graph parity:

- `.riv` bytes have already become a verified `rive_binary::RuntimeFile`.
- `rive-graph` projects the artboard-local object arena, file/artboard/runtime
  collections, component hierarchy, dependency relationships, graph order, and
  structural diagnostics that C++ establishes during import, `onAddedDirty`,
  `onAddedClean`, validation, initialization, or `buildDependencies`.
- The projection is immutable or snapshot-style. It does not advance frames,
  propagate dirt, solve transforms, run constraints, execute data binds, or emit
  renderer commands.

When a C++ helper needs live state, mutable queues, or per-frame evaluation, it
belongs to a later runtime contract even if it is adjacent to graph code.

## Completion Gate

The active graph goal is complete when all of these are true:

- Every C++ graph-owned `buildDependencies()` / `addDependent()` family is either
  represented in `DependencyKind`/graph projections or explicitly classified as a
  later runtime concern.
- `ArtboardGraph::diagnostics` covers unresolved static graph references and
  graph-cycle facts needed before later schedulers run.
- `ComponentNode` exposes the static adjacency facts later dirt scheduling needs:
  children, transform constraints, local component dependents, and C++
  `graphOrder` parity.
- The C++ probe comparison validates the supported corpus for the covered graph
  facts.
- No new post-import runtime behavior is added to `rive-binary` for this graph
  work.
- Remaining runtime behavior is documented under "Deferred Runtime Work" below.

Suggested final verification remains:

```sh
make test
make cpp-compare
```

## Implemented Static Graph Surface

| Surface | C++ Source / Phase | Rust Surface | Status |
| --- | --- | --- | --- |
| Artboard-local object slots | import / validation | `ArtboardGraph::objects` | Covered |
| File collections | import | `GraphFile` assets, view models, enums | Covered |
| Animation and state-machine grouping | import | `ArtboardGraph::animations`, `state_machines` | Covered |
| Auto empty state machine | `Artboard` initialization quirk | `StateMachineGraph` projection | Covered |
| Component identity and capabilities | imported objects + schema inheritance | `ComponentNode`, `ComponentCapabilities` | Covered |
| Parent resolution and child lists | `onAddedDirty` | `parent_local`, `children`, missing-parent diagnostics | Covered |
| Layout style child adoption | `LayoutComponent::onAddedDirty` | C++ child-list projection | Covered |
| Transform constraint registration | `Constraint::onAddedDirty` | `ComponentNode::constraint_locals` | Covered |
| Component dependents | `buildDependencies`, draw-target initialization | `ComponentNode::dependent_locals` | Covered |
| C++ dependency order | `Artboard::sortDependencies` | `ComponentNode::graph_order`, dependency order | Covered |
| Dependency cycles | dependency sorter projection | `GraphDiagnostic::DependencyCycle`, `DependencyNodeCycle` | Covered |
| Draw target/rule/clipping references | `onAddedDirty`, `initialize` | draw/clipping nodes and diagnostics | Covered |
| Drawable order | `Artboard::initialize` drawable setup | `drawable_order` | Covered |
| Draw target order | `Artboard::initialize` target setup | `draw_target_order`, target-cycle diagnostics | Covered |
| Path composers | `PathComposer::buildDependencies` | synthetic `PathComposer` nodes and edges | Covered |
| Clipping-shape path-composer edges | `ClippingShape::buildDependencies` | `ClippingShapePathComposer` edges | Covered |
| Follow path constraints | `FollowPathConstraint::buildDependencies` | target and parent dependency edges | Covered |
| Text follow path modifiers | `TextFollowPathModifier::buildDependencies` | target and text dependency edges | Covered |
| Text variation helpers | `TextStyle::onAddedClean/buildDependencies` | synthetic helper nodes and edges | Covered |
| Paint path builders | `Stroke`, effect-bearing `Fill`, `Feather` dependencies | path-builder dependency edges | Covered |
| Effect parent dependencies | `GroupEffect`, `ScriptedPathEffect` | explicit parent dependency edges | Covered |
| Gradient paint-container dependencies | `LinearGradient` / `RadialGradient` | gradient dependency edges | Covered |
| Skins, tendons, IK peer parents | `Skin`, `Tendon`, `IKConstraint` hooks | skeletal skin dependency edges | Covered |
| Joystick graph dependencies | `Joystick::buildDependencies` | parent/handle-source edges and joystick projection | Covered |
| Scroll constraint dependencies | `ScrollBarConstraint`, `ScrollConstraint` | scroll-bar and layout-child edges | Covered |
| Host registries | `onAddedClean` artboard host lists | nested artboards, component lists, hosts | Covered |
| Component-list map rules | `ArtboardListMapRule::onAddedDirty` | `ComponentListNode::map_rules` | Covered |
| Reset/advance registries | `ResettingComponent::from`, `AdvancingComponent::from` | reset and advance projection lists | Covered |
| Artboard/state-machine data-bind membership | import / initialize / sort | graph data-bind projections | Covered as static membership |
| State-machine scripted objects | importer registration | scripted object projections | Covered |
| Mesh/path geometry registrations | vertex and weight `onAddedDirty` | `meshes`, `paths` | Covered |
| Shape-paint registrations | paint/mutator/effect `onAdded*` hooks | `shape_paint_containers` | Covered |
| NSlicer details | `NSlicerDetails::from`, axis/tile registrations | `n_slicer_details` | Covered |
| Shape deformer cache | `Shape::onAddedClean` | `shape_deformers` | Covered |
| Skeletal caches | `Bone`, `Skin`, `Tendon`, IK clean hooks | `skeletal_bones`, `skeletal_skins` | Covered |

## Add-Dependent Audit

The remaining C++ `addDependent()` calls should be handled as follows:

| C++ Area | Classification | Reason |
| --- | --- | --- |
| `TransformComponent`, `Constraint`, `TextStyle`, `Mesh`, `FocusData`, `SemanticData`, `NSlicer` parent hooks | Graph-owned | Static dependency edges; already represented by `ParentChild` where C++ really calls parent dependency code. |
| `TargetedConstraint`, `IKConstraint`, `FollowPathConstraint`, `TextFollowPathModifier` | Graph-owned | Static build-dependency relationships; already represented by specialized dependency kinds. |
| `PathComposer`, `ClippingShape`, `Stroke`, `Fill`, `Feather`, gradients, effects | Graph-owned | Static source/path-builder/parent relationships; already represented with synthetic nodes or specialized edges. |
| `Skin`, `Tendon`, skinnables | Graph-owned | Static skeletal dependency and registration facts; already represented. |
| `Joystick`, `ScrollBarConstraint`, `ScrollConstraint` | Graph-owned | Static imported graph prerequisites; already represented. |
| `Artboard::buildDrawTargets` root/flattened draw-target dependencies | Graph-owned | Static draw-target initialization facts; already represented outside component dependents where synthetic roots are involved. |
| `DataBind`, `DataConverter*`, `ArtboardComponentList` value dependencies | Deferred runtime work | These need live source/target values, dirty queues, collapse state, or data-context mutation. |
| `ViewModelInstance*`, `StateMachineInstance`, Lua/script runtime dependents | Deferred runtime work | These are instance/runtime execution relationships, not static artboard graph facts. |
| `ListPath` y-value dependency | Deferred runtime work | This depends on live view-model/list data evaluation. |
| Active draw target linked lists and draw-command emission | Partially graph-owned | Active target grouping and before/after placement are now represented by `sorted_drawable_order`; clipping proxies, save-operation elision, and renderer command emission remain deferred draw runtime work. |

## Deferred Runtime Work

These are not blockers for completing the current `rive-graph` milestone:

- Dirt propagation, dirt bit mutation, collapse propagation, and frame scheduling.
- Mutable artboard instances and cloning.
- Local/world transform updates.
- Constraint, IK, skeletal, scroll, and layout solving.
- Data-context binding, data-bind dirty queues, source/target mutation, property
  observers, converter execution, and view-model dependent updates.
- State-machine execution and listener/input processing.
- Lua/script VM initialization and execution.
- Clipping-stack mutation, `clearRedundantOperations()` save-operation elision,
  renderer paint allocation, draw commands, and GPU work.
- Text shaping/layout and variable-font mutation.
- Audio playback.

## Next Narrow Slice

The next implementation slice should only touch `rive-graph` if it fails this
matrix. Good candidates are:

- Add a missing static graph projection that satisfies the admission rule.
- Strengthen C++ source-audit tests for an already modeled dependency family.
- Add diagnostics for unresolved static references that later schedulers need.
- Improve documentation or probe comparison for a graph-owned relationship.

Bad candidates for this goal are:

- Adding more `RuntimeFile` helpers for post-import behavior.
- Modeling `ComponentDirt` mutation or `collapse()` propagation.
- Executing data binds, state machines, constraints, transforms, text layout, or
  draw commands.

Those are valid future milestones, but they need their own runtime contracts.
