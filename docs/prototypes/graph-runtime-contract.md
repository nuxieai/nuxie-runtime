# Graph Runtime Contract

Date: 2026-06-28

This document starts the next milestone after binary import parity. It defines
the `rive-graph` seam so post-import graph work can move forward without pulling
new runtime behavior back into `rive-binary`.

Completion is tracked in
[`graph-runtime-completion-matrix.md`](graph-runtime-completion-matrix.md).

## Formal Goal

Define and implement the next runtime seam around `rive-graph`: consume the
verified `rive-binary` imported model and build the post-import artboard graph
lifecycle through a finite, testable graph projection.

The goal is complete when `rive-graph` can project the C++ post-import artboard
graph facts needed by the next runtime slices, while `rive-binary` remains frozen
as the file-loading module except for import-parity bugs or upstream C++ import
drift.

## Scope Lock

`rive-graph` owns graph topology. It does not own frame execution.

The external seam for `rive-graph` is:

- Input: a verified `rive_binary::RuntimeFile`.
- Output: immutable or explicitly snapshot-style graph projections for file-owned
  collections, artboard-local object slots, component hierarchy, dependency edges,
  dependency order/cycles, and immediate post-import graph relationships.
- Errors: structural graph/projection diagnostics that can be derived from the
  imported model without mutating live runtime state.

Everything past that seam belongs to later runtime crates or later `rive-graph`
milestones with their own contracts.

## Owned By `rive-graph`

`rive-graph` owns:

- Artboard-local object slot projection from `rive-binary`.
- Component indexes and stable local/global ID maps.
- Parent/child hierarchy and missing-parent diagnostics.
- Capability classification needed by graph algorithms.
- Dependency edges and dependency order over imported graph relationships.
- Dependency cycle diagnostics.
- Immediate graph relationships established by C++ import, `onAddedDirty`,
  `onAddedClean`, or `buildDependencies`, when those relationships can be
  represented as static graph facts.
- File-level projection convenience over `rive-binary` collections when graph
  consumers need those collections alongside artboards.

Graph relationships should stay ID-based. Do not introduce `Rc<RefCell<dyn Core>>`
or pointer-style ownership to mimic the C++ runtime object graph.

## Not Owned By `rive-graph` Yet

These are future runtime slices, not blockers for the current graph seam:

- Binary file decoding or import-stack ownership rules.
- Mutable artboard instances and cloning.
- Dirt propagation and frame scheduling.
- Local/world transform mutation.
- Constraint solving.
- Layout solving.
- Animation advancement.
- State-machine execution.
- Live data-binding source/target mutation and scheduling.
- Rendering or draw-command emission.
- Text shaping/layout.
- Audio playback.
- Scripting execution.

Some of these may later live in `rive-graph` if the module grows deliberately, but
they need explicit admission through a new contract or a contract amendment.

## Admission Rule For New Graph Work

Before adding a new `rive-graph` relationship, edge family, lifecycle projection,
or C++ probe field, answer:

1. Is this derivable from the already imported `RuntimeFile` without executing a
   frame?
2. Does C++ establish the relationship during import, `onAddedDirty`,
   `onAddedClean`, validation, or `buildDependencies`?
3. Is the result a static graph fact, dependency edge, graph ordering fact, or
   structural diagnostic?
4. Is it needed by the next runtime slice as graph input rather than as live
   behavior?

If the answer is no to all four, do not add it to `rive-graph`.

If the answer requires new file-loading data, first check
[`binary-import-completion-contract.md`](binary-import-completion-contract.md).
Only add to `rive-binary` when the binary contract admits it.

## First Finite Slice

The first slice after this contract is ticket `#7`: complete the parent/child and
dependency graph surface.

The slice should extend the existing edge list toward the audited C++
`buildDependencies()` surface, especially:

- Path composer projections and dependencies.
- Follow-path dependencies.
- Text graph dependencies.
- Layout dependencies.
- Data-binding graph dependencies that are static graph facts.
- Remaining scroll/layout dependencies beyond the covered
  `ScrollConstraint -> ScrollBarConstraint` and
  `ScrollConstraint -> layout-provider content child` edges.
- Paint/effect graph dependencies.

Each edge family should have:

- A short C++ source audit or probe comparison.
- Focused synthetic coverage when practical.
- Corpus comparison coverage when the C++ probe can expose it.
- A classification against the admission rule above.

## Current Starting Point

The existing `rive-graph` prototype already covers:

- `GraphFile::from_runtime_file`.
- C++-style artboard-local object slots.
- File-level asset, view-model, data-enum, animation, and state-machine projection.
- Component parent resolution and C++ `children()` list projection, including
  import-time child adoption that is not a simple inversion of serialized
  `parentId` values.
- Static transform constraint registration lists on `ComponentNode`, matching
  C++ `TransformComponent::constraints()` append order from
  `Constraint::onAddedDirty`, without admitting constraint solving, dirt
  propagation, or transform mutation.
- Static component dependent adjacency on `ComponentNode`, matching the
  artboard-local portion of C++ `Component::dependents()` after
  `buildDependencies` and draw-target initialization. Synthetic
  `PathComposer`/`TextVariationHelper` dependents and temporary draw-target
  roots remain represented through dependency nodes or draw-target projections,
  not as serialized local components.
- Aggregated structural diagnostics on `ArtboardGraph`, covering missing
  component parents, unresolved nonzero draw/clipping references, dependency
  cycles, dependency-node cycles, and draw-target cycles without mutating or
  rejecting the imported graph.
- Capability flags for artboard/container/world-transform/transform/drawable.
- Draw target, draw rules, and clipping source relationships.
- Static drawable-order initialization projection matching C++ `m_Drawables`:
  imported drawable collection, `ForegroundLayoutDrawable` reordering, layout
  `DrawableProxy` injection, and flattened draw-rule ownership, without
  admitting `sortDrawOrder()`, render linked-list mutation, clipping-stack
  operations, renderer commands, or GPU work.
- Static draw-target dependency ordering matching C++ `m_DrawTargets`
  initialization: parent-ordered draw-rule groups, synthetic root target
  dependents for resolved target drawables, flattened-rule target dependencies,
  and target-cycle diagnostics, without admitting active target linked lists,
  placement splicing,
  clipping-stack operations, renderer commands, or GPU work.
- Clipping source/clipped drawable projections.
- Static `on_added_clean` artboard host registries for exact C++
  `NestedArtboard`, `NestedArtboardLeaf`, `NestedArtboardLayout`, and
  `ArtboardComponentList` objects, matching `m_NestedArtboards`,
  `m_ComponentLists`, and `m_ArtboardHosts`, plus static
  `ArtboardComponentList` map-rule tables from `ArtboardListMapRule`
  registration, without admitting nested-artboard, component-list, cloning,
  layout, data-context binding, or advance behavior.
- Static `on_added_clean` joystick registration facts, matching `m_Joysticks`,
  `m_JoysticksApplyBeforeUpdate`, resolved x/y animations, and the nested remap
  animation dependents collected by `Joystick::addDependents` without admitting
  `Joystick::apply`, component updates, data-bind scheduling, or animation
  advancement.
- Static reset and advance lifecycle registries, matching `m_Resettables` from
  `ResettingComponent::from` and `m_advancingComponents` from
  `AdvancingComponent::from` without admitting `reset()`,
  `advanceComponent()`, data-bind advancement, component updates, or frame
  scheduling.
- Static artboard-owned and state-machine-owned data-bind container
  registrations, exposed through `ArtboardGraph::data_binds` and
  `StateMachineGraph::data_binds`; artboard-owned binds use C++ initialized
  `DataBindContainer::sortDataBinds` order, while state-machine-owned binds
  match `StateMachine::addDataBind` ownership for bindable-property targets.
  These facts do not admit data-context binding, dirty queues, property
  observers, converter execution, source/target mutation, state-machine
  execution, or data-bind advancement.
- Static state-machine scripted-object registrations, exposed through
  `StateMachineGraph::scripted_objects`, matching
  `StateMachineImporter::addScriptedObject`, `StateMachine::addScriptedObject`,
  and `ScriptedObjectImporter::addInput` for exact C++ state-machine
  scripted-object owners. These facts do not admit script asset initialization,
  VM registration, cloning, script input hydration, script execution, or
  state-machine execution.
- Synthetic path composer projections for each imported `Shape`, with path inputs
  sourced from `rive-binary`'s C++-equivalent shape registration facts.
- Static mesh/path geometry registration projections, exposed through
  `ArtboardGraph::meshes` and `ArtboardGraph::paths`, matching
  `MeshVertex::onAddedDirty`, `PathVertex::onAddedDirty`, ordered
  `Mesh::addVertex`/`Path::addVertex`, and `Weight::onAddedDirty` attachment to
  vertices. These facts do not admit vertex deformation, skinning math,
  `Path::buildPath`, contour/path tessellation, weight blending, or dirty
  propagation.
- Static shape-paint container registration projections, exposed through
  `ArtboardGraph::shape_paint_containers`, matching
  `ShapePaint::onAddedClean`, `ShapePaintContainer::addPaint`,
  `ShapePaintMutator::initPaintMutator`, `GradientStop::onAddedDirty`, and
  registered stroke-effect target links. These facts do not admit paint
  mutation, effect execution, gradient stop sorting, path-effect application,
  renderer paint allocation, draw commands, or GPU work.
- Static `NSlicerDetails` registration projections, exposed through
  `ArtboardGraph::n_slicer_details`, matching `NSlicerDetails::from`, ordered
  `addAxisX`/`addAxisY`, and patch-indexed `addTileMode` registrations for
  exact `NSlicer` and `NSlicedNode` details owners. These facts do not admit
  NSlicer deformation math, patch solving, layout updates, path deformation, or
  render-path mutation.
- Static `Shape::onAddedClean` render-path deformer projections for each
  imported `Shape`, recording the first ancestor accepted by
  `RenderPathDeformer::from`, currently exact `NSlicedNode`, without admitting
  `NSlicer` deformation math, `Path::buildPath` deformer application, gradient
  deformer updates, or point-deformation runtime behavior.
- Static skeletal registration projections: `skeletal_bones` records exact
  `Bone::onAddedClean` child-bone caches plus `IKConstraint::onAddedClean` peer
  constraints on ancestor bones, and `skeletal_skins` records exact
  `Skin::onAddedDirty` skinnable parents plus valid
  `Tendon::onAddedClean -> Skin::addTendon` registration order, without
  admitting bone transform solving, IK solving, skin matrices, vertex
  deformation, or skin/transform dirt propagation.
- Dependency nodes for real imported components plus synthetic path composers
  and text variation helpers, with a topological node order and a filtered
  real-component local-ID order.
- C++ `Artboard::sortDependencies` graph-order projection on
  `ComponentNode::graph_order`, using only root-reachable `buildDependencies`
  edges for dirt-depth parity while keeping the complete dependency orders as
  graph diagnostics over all projected relationship edges.
- Dependency edges for C++ `parent()->addDependent(this)` parent relationships,
  targeted constraints, IK constraints, IK chain off-branch children,
  draw-target drawable references, draw-rule target references, clipping sources,
  skinning for exact C++ skinnables
  (`Mesh` and `PointsPath`), Joystick custom-handle dependencies,
  path-composer shape/path prerequisites,
  clipping-shape-to-source-path-composer prerequisites, follow-path target and
  constrained-parent prerequisites, text-follow-path target and text
  prerequisites, text variation helper prerequisites, stroke/fill/feather
  path-builder prerequisites, audited `ClippingShape`/paint/effect
  parent-dependency skips and
  explicit `GroupEffect`/`ScriptedPathEffect` parent prerequisites,
  linear/radial gradient paint container prerequisites, and the static
  `ScrollConstraint -> ScrollBarConstraint` and
  `ScrollConstraint -> layout-provider content child` dependencies.
- Static list-constraint registrations for exact C++
  `ListFollowPathConstraint` children under exact `ArtboardComponentList`
  parents, matching `ConstrainableList::addListConstraint` without admitting
  `constrainList()`, list layout, or virtualization behavior.
- Static layout-constraint registrations for exact C++
  `LayoutNodeProvider` content children under `ScrollConstraint` content,
  exposed through `layout_constraint_registrations` and matching
  `LayoutNodeProvider::addLayoutConstraint` plus
  `ScrollConstraint::addLayoutChild` without admitting layout solving,
  virtualization, Yoga updates, or scroll-constraint execution.
- Topological dependency order and dependency-cycle diagnostics.
- C++ probe comparison through `make cpp-compare`.

## Completion Checklist

This graph-seam milestone can be marked complete when:

- The public `rive-graph` surface is documented as graph projection, not live
  runtime execution.
- The dependency edge families included in the milestone are listed and each one
  has evidence against C++ source/probe behavior.
- `rive-binary` has no new post-import runtime helper expansion for this work.
- Focused Rust tests pass for graph projection and dependency ordering.
- The C++ graph comparison passes for the supported fixture set.
- Any runtime behavior discovered during graph work is recorded as out of scope or
  moved to a later runtime plan.

Suggested verification:

```sh
make test
make cpp-compare
```

If `make cpp-compare` fails in `rive-binary`, triage it through the binary import
contract before changing `rive-binary`. If it fails in `rive-graph`, classify the
failure through this contract before implementing more graph surface.
