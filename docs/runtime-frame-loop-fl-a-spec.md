# FL-A Component / Update Owner-Family Specification

Status: binding implementation specification; production translation has not
started.

Pinned authority:
`/Users/levi/dev/oss/rive-runtime` at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

This specification closes the first dependency wave in
`docs/runtime-frame-loop-port-map.md`. It covers the complete 52-file
`component-update-graph` source set and all six pending `component.*` member
rows. It is derived from complete reads of the pinned sources and headers,
the committed Rust owners, the FLR rulebook, and three independent read-only
cluster audits. Benchmark scenes and timing rankings did not select any work
below.

## Boundary

FL-A begins at occurrence construction and parent linking, includes retained
Component identity, dependency construction and sorting, dirt and collapse,
component update, transforms, bones, constraints, scrolling, advancing,
resetting, virtualization, and the Component-facing part of Drawable, then
stops at already-closed live draw owners and the existing
`Renderer` / `RenderFactory` interface.

Renderer backends, Dawn, wgpu, shaders, atlases, tessellation, GPU batching,
and renderer API design are excluded. The existing renderer-facing Drawable
linked list and backend resources remain intact.

## Structural target

The target is the pinned C++ owner graph, expressed safely in Rust:

1. `ArtboardInstance` uniquely owns one dense object-occurrence arena.
2. `ComponentHandle` is an occurrence-local typed index, the Rust equivalent
   of a retained non-owning `Component*` under AF-1, FLR-2, and FLR-11.
3. Authored local/global IDs remain serialized identity and public lookup
   keys. They are not internal retained relationships.
4. Every retained parent, child, dependent, constraint, collapsable,
   advancing, resetting, virtualization, target, bone, tendon, physics,
   proxy, and traversal relationship is a typed occurrence-local handle.
5. Import-time type classification and immutable dependency blueprints may
   remain under AF-5. Each Artboard occurrence consumes them once to create
   its own links; copied dependent lists and copied graph order do not remain
   runtime truth.
6. Construction uses a builder state where graph order is unset. Only the
   completed arena exposes assigned order, as required by FLR-3.
7. Component dirt is stored on the Component occurrence. Concrete family
   dispatch owns `onDirty`, `update`, collapse hooks, and queries. Artboard
   mediates its back-pointer callback without rediscovering the owner.
8. Artboard retains the exact dependency, advancing, and resetting schedules
   in object/insertion order. Steady traversal dereferences handles directly.
9. Clone allocates a fresh occurrence arena and applies the concrete copy,
   reset, rebuild, and relation-remap policy for each object family. Drop
   follows the pinned owner order through RAII and explicit owner-mediated
   teardown where another owner is mutated.

The resulting steady path has one owner graph. A local-ID map, type-name
redispatch, property-name dirt allowlist, graph scan, or Artboard side vector
may remain only at a genuine external/import boundary and only with a ledger
row proving that boundary.

### One logical concrete-object owner

The unit of mutable runtime identity is one `RuntimeObjectOccurrence` selected
by the authored object slot. It contains the sole mutable generated-property
storage for that object, an optional embedded `ComponentOccurrence` base, and
a closed concrete-family payload for Component subclass state.

`ComponentHandle` and concrete handles are typed views of that same object
slot, not indexes into independently authoritative property, Component, and
subclass vectors. Downcast is a closed enum match established by validation;
steady code never re-dispatches by `type_name`.

Deserialize writes generated backing fields directly before lifecycle
callbacks, exactly like the generated C++ `deserialize` methods. Later
animation, DataBind, and public property writes resolve the object slot once,
run the generated equality guard, assign the sole backing field, invoke the
concrete `*Changed` callback, then publish property notification. The callback
therefore reaches the same embedded Component/subclass owner that update and
draw read.

Existing heavyweight renderer-facing resources remain under the accepted
one-to-one RF owner adaptations; their handles live in the concrete payload
and do not duplicate mutable generated fields. In particular,
`Drawable::isHidden()` reads the sole generated `drawableFlags` field plus
the same occurrence's collapse bit. A second hidden bool is forbidden. The
same single-field rule applies to constraints, bones, and scrolling owners.

```text
ArtboardInstance
└── RuntimeObjectArena
    └── RuntimeObjectOccurrence (one authored slot)
        ├── generated fields (sole mutable backing)
        ├── ComponentOccurrence (parent/dirt/dependents/order)
        └── ConcreteComponentState
            ├── Transform / Bone / Constraint / Scroll / Drawable …
            └── typed handles to accepted owned resources
```

## Exact lifecycle

### Construction

The occurrence builder performs these phases in order:

1. Create object slots and concrete occurrence values in source object order.
2. Run `onAddedDirty` in object order. Resolve each Component parent once,
   set the Artboard owner mediation, and append the child to the retained
   parent list. `Constraint::onAddedDirty` validates/registers its parent,
   then `TargetedConstraint::onAddedDirty` resolves and validates its target
   before returning; required-target failure and inherited failure ordering
   match `src/constraints/constraint.cpp:9-20` and
   `src/constraints/targeted_constraint.cpp:23-39`.
3. Run `onAddedClean` in object order. Resolve parent-transform, bones, paths,
   lists, and other typed invariants not owned by the preceding dirty phase;
   classify resettable and advancing owners into insertion-ordered Artboard
   lists. FollowPath's clean phase marks its already-resolved Shape/Path
   target for follow-path use
   (`src/constraints/follow_path_constraint.cpp:149-165`); it does not defer
   target resolution or construct a path measure.
4. Run every concrete Component `buildDependencies` in object order. Append
   unique dependents in the order C++ calls `addDependent`.
5. Run the occurrence-local DependencySorter from the Artboard root, retain
   its order, assign every Component graph order, and mark the Artboard
   Components-dirty.
6. Complete concrete construction-only retained state such as Skin's bone
   buffer, IK's FK chain, physics/proxy ownership, and clone-local Drawable
   membership. FollowPath's initial RawPath/PathMeasure is built only when
   normal initial FILTHY update dirt is consumed
   (`src/constraints/follow_path_constraint.cpp:122-147`), and later target
   path dirt rebuilds it at that same update site.

Authorities:
`src/component.cpp:13-29`;
`src/artboard.cpp:264-288,325-428,846-855`;
`src/dependency_sorter.cpp:6-48`;
FLR-1, FLR-3, FLR-11, and FLR-12.

### Dirt and collapse

`add_dirt(handle, value, recurse)` preserves
`src/component.cpp:32-54` literally:

1. return `false` when every incoming bit is already present;
2. publish the accumulated Component dirt;
3. call the concrete owner's `on_dirty(accumulated)`;
4. call Artboard `on_component_dirty(handle)`;
5. if requested, recurse over retained dependents in insertion order;
6. return `true`.

No text, shape, mesh, epoch, or Skin side effect occurs before step 2.
Concrete C++ owners perform those effects from their own callback.

Collapse preserves `src/component.cpp:76-95,108-127`: change only the
Collapsed bit, call concrete `on_dirty`, notify Artboard, then notify unique
collapsables in insertion order. Container child propagation and
TransformComponent constrained-dependent propagation run at their exact
overrides, not in a broad Artboard invalidator. Closed dispatch must also
preserve virtual exceptions: Solo and LayoutComponent intentionally call
`Component::collapse` directly and then run their selective propagation;
they do not inherit ContainerComponent's blind child tail
(`src/solo.cpp:29-39`; `src/layout_component.cpp:243-250`).

### Component update

Artboard preserves `src/artboard.cpp:997-1008,1204-1242`:

- a clean Artboard returns without work;
- each pass clears only Artboard Components dirt;
- the retained schedule is walked by direct handle;
- clean and collapsed owners are skipped;
- owner dirt is cleared before concrete `update`;
- a newly dirtied earlier graph order breaks the current walk;
- at most 100 passes run and pending Components dirt remains observable.

TransformComponent preserves
`src/transform_component.cpp:54-61,73-113`: the Transform-bit addition gates
recursive WorldTransform dirt; update order is local transform, world
transform followed immediately by retained constraints, then render opacity.
Parent transform and opacity reads use retained typed handles and the exact
base/override `childOpacity` contract.

### Advance, reset, clone, and drop

- Artboard retains one mixed-family advancing list in object order and calls
  it before DataBinds (`src/advancing_component.cpp:17-44`;
  `src/artboard.cpp:330-395,1463-1480`).
- Artboard retains one resetting list in object order and walks it exactly
  (`src/resetting_component.cpp:12-25`;
  `src/artboard.cpp:340-346,1483-1493`).
- Clone applies each concrete copy policy below; definition state remains
  shared only where C++ shares it.
- Drop releases owned constraint physics/proxies, Skin buffers, and
  list-owned mounted child occurrences in concrete owner order. A
  ScrollVirtualizer does not own those children. No handle may resolve into
  the source occurrence after clone or survive its owner after drop.

### Concrete clone/drop policy

There is no generic “copy and remap every member” operation. Each payload
applies its C++ copy/default/rebuild policy:

| owner | generated/deep copy | default on clone | rebuild site | non-owning relations | teardown |
|---|---|---|---|---|---|
| Component / Container / Transform / Node | generated/base fields; no deep heap | dirt, graph order, lazy computed-local state | parent/child in dirty; parent-transform, constraints, dependencies, order in later phases | rebuilt at the cited lifecycle site | arena relations vanish after callbacks |
| Bone / RootBone | generated/base fields | peer/child relation vectors | clean/dependencies | Bone and peer-constraint handles rebuilt | no owned heap |
| Skin | generated/base fields; no buffer copy | Skinnable/Tendon handles and bone buffer | dirty/clean/dependencies; initial update | rebuilt from clone objects | release Skin-owned buffer once (`src/bones/skin.cpp:11`) |
| Tendon / Weight / Skinnable | generated/base and owned value arrays | Bone/Skin links | dirty/clean/dependencies | rebuilt | value/handle RAII |
| FollowPath / ListFollowPath | generated/base; no measure copy | registrations, RawPath, PathMeasure | target in dirty; follow marking in clean; dependencies; update dirt | rebuilt | path/measure RAII |
| IK | generated/base; no FK-chain copy | FK chain and peer/off-chain links | clean/dependencies | rebuilt root-to-tip | chain value RAII |
| ScrollPhysics / Clamped / Elastic | generated fields; concrete physics clone only where invoked | clock/run scratch per concrete initializer | import/init/prepare | constraint handle resolved by owner lifecycle | concrete helper/physics RAII |
| ScrollConstraint | generated base plus cloned physics only (`scroll_constraint.cpp:364-373`) | virtualizer, layout children, offsets/scratch/drag/intents omitted by that initializer | virtualizer in dirty; children in dependencies; state in init/update | rebuilt | delete virtualizer, clear non-owning children, delete physics (`scroll_constraint.cpp:14-23`) |
| ScrollVirtualizer | constructed by ScrollConstraint; no child copy | visible/current range | ScrollConstraint dirty/init/virtualize | borrows list/children; owns none | reset indices only (`scroll_virtualizer.cpp:8-10`) |
| Draggable / ScrollBar / proxies | generated fields; owned proxy/listener objects only at their C++ creation sites | pointer/drag phases | owner dirty/clean/init | rebuilt | listener-group/constraint owner releases proxies |
| ArtboardComponentList | generated/base; mounted children only under its concrete copy path | adapter registrations/order state | dirty/clean/dependencies/list settlement | list owns mounted children | list releases mounted occurrences |
| Drawable | generated/base and only approved RF-owned resources | clipping/base-proxy relations | dirty/clean/dependencies | rebuilt; flags are never copied to a mirror | occurrence links release before owned resources |

Before a concrete payload lands, its implementation slice expands the
corresponding row with exact constructor/copy/clone citations and tests copied,
reset, and rebuilt members independently. FLR-7, not a generic remapper, is
the oracle.

### Artboard cross-wave handoff

The whole `src/artboard.cpp` file row belongs to FL-D, but FL-A owns and
freezes these methods/portions:

- Component dirty/clean classification and `buildDependencies` phases
  (`artboard.cpp:264-428`);
- `sortDependencies` and graph-order assignment
  (`artboard.cpp:846-855`);
- `onComponentDirty` and dirt-depth lowering
  (`artboard.cpp:997-1008`);
- `updateComponents` traversal (`artboard.cpp:1204-1242`);
- retained advancing/resetting classification and walks
  (`artboard.cpp:330-395,1463-1493`);
- the advancing→DataBind→`updatePass(true)` frame interleaving and
  residual-dirt return (`artboard.cpp:1463-1508`).

FL-A records method-level lifecycle evidence and negative legacy ratchets on
the still-pending FL-D Artboard row; it does not promote that whole file.
FL-D may replace Event/DataBind/ViewModel/settlement portions only and must
preserve these FL-A contracts. The frame-loop checker validates the frozen
method anchors and rejects reintroduction of copied graph/schedule paths even
while the Artboard file row remains pending.

## Six member-row closure contracts

| member row | required closed evidence |
|---|---|
| `component.identity` | one dense occurrence arena; typed internal handles; authored IDs only at external/import lookup; clone-local remap; no steady owner rediscovery |
| `component.dirt` | exact accumulated-mask callback order; one concrete on-dirty path; exact collapse order; duplicate short-circuit; Artboard dirt-depth callback |
| `component.dependents` | unique insertion-ordered occurrence handles built by concrete owners; recursion and sorting consume the same list |
| `component.update_order` | occurrence-local dependency sort; construction-state unset order; direct retained schedule; clear-before-update, early restart, and 100-pass guard |
| `component.transforms` | retained parent-transform and constraint handles; gated Transform→World dirt; distinct authored and Node computed-local transforms; polymorphic child opacity |
| `component.clone_drop` | fresh occurrence arena; per-concrete copied/default/rebuilt relation policy; exact deep ownership; ordered owner-mediated teardown |

No row is promoted until construct, retain, dirty, update/advance/query, clone,
and drop are all evidenced and every displaced path named below is absent.

## 52-file closure matrix

### Core Component family — 11 files

| pinned C++ file | baseline verdict | FL-A action |
|---|---|---|
| `src/advancing_component.cpp` | graph classification exists; runtime owner missing | retain the exact closed-family handle list and mixed object-order dispatch |
| `src/component.cpp` | dirt bits faithful; owner graph and callback order divergent | port parent/Artboard/dependents/collapsables/dirt and concrete dispatch |
| `src/container_component.cpp` | behavior approximated by scans | retain ordered child handles; port collapse/forAll/forEachChild; delete scans |
| `src/dependency_sorter.cpp` | algorithm adapted in import graph; runtime owner/order divergent | sort occurrence dependents in insertion order and assign occurrence graph order |
| `src/drawable.cpp` | renderer-facing list faithful; Component-facing queries divergent | retain clipping membership, validate blend on add, read one hidden owner, port exact layout/hit proxy queries |
| `src/node.cpp` | computed-local owner missing | retain a second lazy computed-local transform and recompute bit |
| `src/parent_traversal.cpp` | missing | port the stateful same-/cross-Artboard parent traversal and crossing metadata |
| `src/resetting_component.cpp` | graph classification exists; runtime owner missing | retain and dispatch the exact resettable family in object order |
| `src/transform_component.cpp` | arithmetic adapted; retained parent/constraints and dirt gate divergent | port the complete transform owner and update/collapse lifecycle |
| `src/virtualizing_component.cpp` | behavior spread across Artboard maps | port the ArtboardComponentList adapter and exact interface conversation |
| `src/world_transform_component.cpp` | fields adapted; child-opacity polymorphism incomplete | port exact WorldTransform/RenderOpacity propagation and family query |

### Bones family — 6 files

| pinned C++ file | baseline verdict | FL-A action |
|---|---|---|
| `src/bones/bone.cpp` | formula/state partially present; pointer owner divergent | retain child-bone and peer-constraint handles; port dirt/update/clone lifecycle |
| `src/bones/root_bone.cpp` | authored-root math present; owner/callback divergent | move root authored-position settlement and dirt to the concrete occurrence |
| `src/bones/skin.cpp` | graph IDs and draw-time reconstruction replace owned buffer | retain exactly one Skinnable parent handle, authored-order Tendon handles, distinct graph dependents, and the Skin-owned transform buffer; concrete `onDirty` reaches the Skinnable before Artboard notification; rebuild only from Skin dirt; RAII drop |
| `src/bones/skinnable.cpp` | stable ID substitutes for retained Skin pointer | retain the typed Skin handle on supported PointsPath/Mesh occurrences |
| `src/bones/tendon.cpp` | values projected; owner links divergent | retain its Bone handle and inverse bind once; Skin retains Tendons in authored order |
| `src/bones/weight.cpp` | deformation arithmetic present; inputs reconstructed | retain exact packed value/index and in/out fields with the vertex occurrence; preserve absent/malformed behavior and consume Skin's settled buffer without normalization or draw-time owner reconstruction |

### Constraint and scrolling family — 21 files

| pinned C++ file | baseline verdict | FL-A action |
|---|---|---|
| `src/constraints/constrainable_list.cpp` | global list sidecar | retain unique ordered list-constraint handles on ArtboardComponentList and exact layout/list/ordinary order |
| `src/constraints/constraint.cpp` | central dispatch and partial dirt allowlist | port concrete occurrence registration, universal on-dirty, parent-world, dependency, generated callbacks |
| `src/constraints/distance_constraint.cpp` | arithmetic near-faithful; owner divergent | move formula and retained fields behind a typed occurrence |
| `src/constraints/draggable_constraint.cpp` | missing | port owned listener/proxy/hit target, pointer phase, scroll flag, and drag events |
| `src/constraints/follow_path_constraint.cpp` | formula present; path/measure rebuilt per apply | retain RawPath/PathMeasure and rebuild on update dirt only |
| `src/constraints/ik_constraint.cpp` | solve present; FK chain allocated per apply | retain the validated FK chain, peer/off-chain dependencies, and chain dirt |
| `src/constraints/list_constraint.cpp` | centralized type grouping | port the non-owning polymorphic list-constraint relationship |
| `src/constraints/list_follow_path_constraint.cpp` | distribution math present; owner/dirt/order divergent | port retained fields/callbacks/list registration and exact list mutation |
| `src/constraints/rotation_constraint.cpp` | arithmetic near-faithful; owner divergent | retain target/scratch and complete inherited callbacks |
| `src/constraints/scale_constraint.cpp` | arithmetic near-faithful; owner divergent | retain target/scratch and complete inherited callbacks |
| `src/constraints/scrolling/clamped_scroll_physics.cpp` | missing | port retained clamp and prepare/run/one-advance-stop lifecycle |
| `src/constraints/scrolling/elastic_scroll_physics.cpp` | missing | port owned axis helpers, literal formulas, snap/range/run/clone/reset lifecycle |
| `src/constraints/scrolling/scroll_bar_constraint.cpp` | dependency shell only | port retained ScrollConstraint target, math, validation, dependencies, proxies |
| `src/constraints/scrolling/scroll_bar_constraint_proxy.cpp` | missing | port thumb/track start-drag-end state and physics/drag flag effects |
| `src/constraints/scrolling/scroll_constraint.cpp` | major owner/advance divergence | port complete retained offsets/intents/children/physics/virtualizer/drag/advance/clone/drop lifecycle |
| `src/constraints/scrolling/scroll_constraint_proxy.cpp` | missing | port viewport proxy threshold/direction/interactive start-drag-end state |
| `src/constraints/scrolling/scroll_physics.cpp` | missing | port clock/speed/acceleration/direction/import/reset with deterministic time |
| `src/constraints/scrolling/scroll_virtualizer.cpp` | DTO/window recomputation substitutes for owner | retain previous/current range, recycling, mounted identity, positions, item sizes, notifications |
| `src/constraints/targeted_constraint.cpp` | dependency projected; target rediscovered | resolve/validate/retain target once and preserve ordinary vs FollowPath edge shapes |
| `src/constraints/transform_constraint.cpp` | arithmetic near-faithful; owner/dirt divergent | retain scratch/origin fields and complete callbacks |
| `src/constraints/translation_constraint.cpp` | arithmetic near-faithful; owner/dirt divergent | retain nullable target/scratch and complete callbacks |

### Generated constraint property contract

Every ordinary generated setter follows one sequence: equality return,
assignment to the sole generated backing field, concrete `*Changed`, then
property notification. Generated deserialize writes the backing member
directly and calls neither callback nor notification. Computed ScrollConstraint
properties compare through their getter, call their `set*` method, then run the
callback/notification sequence; they have no serialized backing-field
deserialize case.

The field table is binding. “No-op” is intentional and must not be replaced
by broad parent dirt.

| owner fields | defaults/backing | concrete changed callback | deserialize |
|---|---|---|---|
| Constraint `strength` | `1.0` | `markConstraintDirty` | direct backing write |
| Targeted `targetId` | `u32(-1)` | no-op; target lifecycle is construction-time | direct backing write |
| TransformSpace `sourceSpaceValue`, `destSpaceValue` | `0`, `0` | no-op | direct backing writes |
| TransformComponent X `minMaxSpaceValue`, `copyFactor`, `minValue`, `maxValue`, `offset`, `doesCopy`, `min`, `max` | `0`, `1`, `0`, `0`, `false`, `true`, `false`, `false` | all no-op | direct backing writes |
| TransformComponent Y `copyFactorY`, `minValueY`, `maxValueY`, `doesCopyY`, `minY`, `maxY` | `1`, `0`, `0`, `true`, `false`, `false` | all no-op | direct backing writes |
| Distance `distance`, `modeValue` | `100`, `0` | both `markConstraintDirty` | direct backing writes |
| IK `invertDirection`, `parentBoneCount` | `false`, `0` | invert dirties the chain; parent count no-op | direct backing writes |
| FollowPath `distance`, `orient`, `offset` | `0`, `true`, `false` | distance/orient `markConstraintDirty`; offset no-op | direct backing writes |
| ListFollowPath `distanceEnd`, `distanceOffset` | `1`, `0` | both `markConstraintDirty` | direct backing writes |
| Transform `originX`, `originY` | `0`, `0` | both `markConstraintDirty` | direct backing writes |
| Draggable `directionValue` | `1` | no-op | direct backing write |
| ScrollBar `scrollConstraintId`, `autoSize` | `u32(-1)`, `true` | both no-op | direct backing writes |
| ScrollPhysics `constraintId` | `u32(-1)` | no-op | direct backing write |
| Elastic physics `friction`, `speedMultiplier`, `elasticFactor` | `8`, `1`, `.66` | all no-op | direct backing writes |
| ScrollConstraint stored `scrollOffsetX`, `scrollOffsetY` | `0`, `0` | call `offsetX` / `offsetY` respectively | direct backing writes |
| ScrollConstraint stored `snap`, `physicsTypeValue`, `physicsId`, `virtualize`, `infinite`, `interactive`, `threshold`, `dragMultiplier` | `false`, `0`, `u32(-1)`, `false`, `false`, `true`, `0`, `1` | all no-op | direct backing writes |
| ScrollConstraint computed `scrollPercentX`, `scrollPercentY`, `scrollIndex` | owner getter/setter; no generated backing | `set*` returns while dragging; otherwise stops physics, then either writes a resolved offset or retains the unresolved intent; changed callback no-op (`scroll_constraint.cpp:497-532,605-641`) | not deserialized by generated base |
| ScrollConstraint computed `velocityX`, `velocityY`, `scrollActive` | owner getter/setter; no generated backing | concrete `set*` intentionally does nothing; after a non-equal getter comparison the generated wrapper still runs the no-op changed callback then notifies, so repeated requests may notify without mutation (`scroll_constraint.hpp:144-150`) | not deserialized by generated base |

Authorities:
`include/rive/generated/constraints/*_base.hpp`;
`include/rive/generated/constraints/scrolling/*_base.hpp`;
`src/constraints/constraint.cpp:22-40`;
`src/constraints/distance_constraint.cpp:59-61`;
`src/constraints/follow_path_constraint.cpp:18-19`;
`src/constraints/ik_constraint.cpp:186`;
`src/constraints/list_follow_path_constraint.cpp:8-12`;
`src/constraints/transform_constraint.cpp:18-20`;
`include/rive/constraints/scrolling/scroll_constraint.hpp:129-150`;
`src/constraints/scrolling/scroll_constraint.cpp:468-532,605-641`.

### Math support family — 14 files

| pinned C++ file | baseline verdict | FL-A action |
|---|---|---|
| `src/math/aabb.cpp` | isomorphic value utility | retain; add/keep direct oracle coverage |
| `src/math/bezier_utils.cpp` | isomorphic stateless utility | retain |
| `src/math/bit_field_loc.cpp` | adapted inline schema masking | retain under AF-5/AF-7 with exact mask tests |
| `src/math/contour_measure.cpp` | adapted owned-Vec representation | retain under AF-7; prove segment/order/query correspondence |
| `src/math/hit_test.cpp` | adapted consolidated command decoding | retain only after exact winding/crossing oracle coverage |
| `src/math/mat2d.cpp` | isomorphic six-float value type | retain with bit-exact operation tests |
| `src/math/mat2d_find_max_scale.cpp` | no Rust implementation; cold/no pinned caller | port the literal helper and direct tests so the row closes without an absence exception |
| `src/math/n_slicer_helpers.cpp` | isomorphic value calculation | retain |
| `src/math/path_measure.cpp` | adapted owned-Vec representation | retain under AF-7 and consume from concrete FollowPath owner |
| `src/math/random.cpp` | adapted retained formula-source state | retain exact sequence/cursor/call-count/clone/reset contract |
| `src/math/raw_path.cpp` | adapted at RenderPath ownership seam | retain under AF-7/RF rules; no parallel Component cache |
| `src/math/raw_path_utils.cpp` | isomorphic direct value helpers | retain |
| `src/math/rectangles_to_contour.cpp` | missing | port the literal rectangle-to-contour algorithm and direct fixtures |
| `src/math/vec2d.cpp` | isomorphic two-float value type | retain |

Already-isomorphic math code is evidence work, not a rewrite target. The two
missing helpers are small literal ports; implementing them avoids classifying
an in-scope source row by feature absence. Value primitives stay in the
existing Rust value modules; path/measure/NSlicer primitives stay with the
already-closed draw owners. FL-A must not create a second math subsystem in
`components.rs`. Each row closes with a concrete Rust symbol plus
operation/edge-case tests; pixels alone are insufficient for Mat2D, Vec2D,
inverse, max-scale, contour, or measure semantics.

## Dependency-ordered internal landings

These are implementation landings inside one FL-A wave. File/member promotion
and performance measurement occur only after FL-A closes as a whole.

### A1 — Non-production scaffold only

- Add typed Component and concrete-owner handles.
- Add construction-state versus completed-state graph order.
- Make object slots the sole authored-ID ingress.
- Add builder, per-owner clone-policy, and invariant tests.

This scaffold is private/unreachable and crosses no production retention
boundary. It is not a separately committable or mergeable production state.
Its first production use is squashed or landed atomically with A2. At no green
commit may both the new owner graph and the copied-ID runtime graph be live.

### A2 — Atomic occurrence-graph replacement

- Make the A1 arena/handles the production object owner.
- Port Component parent/child/dependent/collapsable/Artboard ownership.
- Install exact add-dirt and collapse dispatch/order.
- Run dirty/clean/buildDependencies phases in object order.
- Import/remap every already-closed mixed dependency node—ordinary Component,
  PathComposer, and TextVariationHelper—into one occurrence-local schedule.
  PathComposer keeps its accepted concrete draw-owner state while participating
  in this schedule under RF-29.
- Sort the occurrence graph and assign graph order. Before removing the old
  graph schedule, prove an ordinary Component → PathComposer → downstream
  dependent has identical edges and visitation order.
- Delete runtime consumption of copied `graph_order`,
  `dependent_locals`, `runtime_dependency_node_order`, subtree scans, and
  dirt-time `component_by_local` resolution.
- Add negative ratchets for the old relation fields and schedule reads in the
  same atomic landing.

Retention boundary crossed: one retained graph is both dirt source and update
schedule.

### A3 — Transform, Node, ParentTraversal, and bones

- Port retained parent-transform/constraint handles and exact transform dirt
  gate/update/opacity behavior.
- Add Node's lazy computed-local transform.
- Add stateful ParentTraversal.
- Port Bone/RootBone/Tendon/Weight/Skin/Skinnable owners and Skin buffer.
- Delete repeated parent lookup, authored/computed-local conflation,
  draw-time bone reconstruction, and bone/Skin epoch compensation.

Retention boundary crossed: settled transform and skin state live on their
C++ owner.

### A4 — Constraint base and stateless transform constraints

- Port Constraint/Targeted/space bases and the exact generated-property table
  above, including intentional no-op callbacks and direct deserialize writes.
- Retain ordered constraint handles on each constrained transform.
- Move Distance/Rotation/Scale/Transform/Translation formulas behind concrete
  owners.
- Delete vector cloning, type-name redispatch, and global property-name dirt
  cases for the covered family.

Retention boundary crossed: constraint registration, target, scratch, and
dirt are occurrence-owned.

### A5 — FollowPath, list constraints, and IK

- Retain FollowPath RawPath/PathMeasure.
- Port ConstrainableList/ListConstraint/ListFollowPath order and gating.
- Retain IK's FK chain and exact dependency rewiring.
- Delete Artboard follow/list/IK side vectors, per-apply path measurement, and
  per-apply IK allocation.

Retention boundary crossed: derived path/chain work is update-owned and
dirt-gated.

### A6 — Advancing, resetting, virtualization, and scrolling

- Install exact advancing/resetting handle schedules.
- Port Draggable and ScrollPhysics owners.
- Port complete ScrollConstraint child rendezvous: a missing child transform
  does not increment; transform application precedes one successful increment;
  ordinary settlement waits for every registered child; `force=true` may
  settle an incomplete set; virtualizer notification occurs only afterward
  (`scroll_constraint.cpp:203-237`).
- Port exact ScrollConstraint advancement: reject missing AdvanceNested or
  collapsed, reject null physics, advance/publish running physics, perform
  NewFrame paused-drag velocity clearing and snapshots, then return
  `physics.enabled || scrollBarDragging || dragging`
  (`scroll_constraint.cpp:299-336`).
- Port ScrollVirtualizer, ScrollBar, and both proxy families.
- Route ArtboardComponentList through the VirtualizingComponent adapter.
- Preserve full Artboard frame interleaving: async poll → advancing owners in
  object order → DataBinds → `updatePass(true)` → return including residual
  Components dirt (`artboard.cpp:1463-1508`). Reset preserves empty-list early
  return and one insertion-order visit (`artboard.cpp:1483-1493`).
- Delete family-specific top-level advance/reset sweeps, parent-side scroll
  child batch, reconstructed virtualization DTO/windows, and corresponding
  Artboard maps/side vectors.

Retention boundary crossed: mixed-family lifecycle and scroll state are
retained once in source object order.

### A7 — Drawable Component-facing remainder and math closure

- Move clipping membership to the Drawable occurrence and read mutable hidden
  state from its sole generated flags owner.
- Validate blend during add-dirty. Port only
  `Drawable::hitTestPoint`, `hittableComponent`, and base `DrawableProxy`
  delegation/opacity from `src/drawable.cpp:62-88`.
- Preserve the already-closed renderer-facing list and resources.
- Preserve existing concrete Shape/Text/Image/layout/clipping geometry hit
  algorithms and renderer/backend clipping resources until their own rows.
- Add the two missing math utilities and adjudicate every retained math row
  with direct tests.
- Delete `clipping_by_drawable`, the hidden mirror, boolean-only layout
  ancestry, and only the second base/proxy delegation topology.

Retention boundary crossed: live draw reads the single Component/Drawable
owner without reopening renderer architecture.

### A8 — Structural deletion and wave acceptance

- Remove every displaced mechanism and add fail-closed negative ratchets.
- Close all 52 file rows and six member rows with lifecycle citations.
- Close the FL-A portions of FL-G02, FL-G05, FL-G07, and FL-G08.
- Run two adversarial reviews: owner/identity/clone/drop and
  dirt/order/advance/draw-time reads.
- Run every required floor.
- Commit the complete wave, then run one whole-corpus canonical checkpoint.

No benchmark entry or sorted residual becomes an internal landing.

## Displaced mechanisms that must be absent

At FL-A closure, no production path may retain these as a second
architecture:

1. `RuntimeComponent.parent_local`, `constraint_locals`,
   `dependent_locals`, or imported plain `graph_order` as internal relations.
2. `ArtboardInstance.component_by_local` in steady dirt/update/constraint/
   parent traversal. Authored-ID ingress must resolve through object slots
   once.
3. `(graph_order, local_id)` schedule resort and runtime consumption of
   `graph.runtime_dependency_node_order`, but only after all existing
   Component/PathComposer/TextVariationHelper nodes have been remapped into
   the replacement occurrence schedule.
4. Pre-store text/shape/mesh mutation and broad epoch fan-out in
   `ArtboardInstance::add_dirt`.
5. Full-vector child/parent scans and temporary topology vectors.
6. Pending-shape and post-loop callback-mutator settlement when the concrete
   C++ owner supplies the exact update site.
7. Artboard-level follow-path, list-follow, scroll, and IK side vectors.
8. Per-apply constraint topology cloning and type/property redispatch.
9. Per-apply FollowPath measurement and IK chain construction.
10. Parent-side scroll child batches and recomputed virtualizer windows.
11. Family-specific advancing/resetting sweeps or deferred queues that
    duplicate the retained interface lists.
12. Draw-time Skin/bone reconstruction and its epoch invalidation bridge.
13. Drawable central clipping map, hidden mirror, boolean-only layout
    ancestry, and duplicate base/proxy delegation topology. Concrete geometry
    hit algorithms and renderer/backend clipping owners remain.
14. Clone repair passes whose only purpose is reconnecting copied IDs after a
    generic aggregate clone.

Some named functions also serve later FL-D/FL-E owners. Delete only the
FL-A compensation responsibility in this wave; retain a path only when its
remaining owner has a specific pending ledger row and no FL-A steady work.

## Deterministic acceptance tests

The implementation adds or extends tests for:

- construction phase and mixed interface classification order;
- TargetedConstraint valid/missing/wrong/optional target and inherited-failure
  ordering across parent registration and target resolution;
- duplicate/accumulated dirt callback order and dependent insertion order;
- unique collapsable initial synchronization and later collapse ordering;
- occurrence dependency sort, diamonds, multiple roots, cycles, and an
  ordinary Component → PathComposer → downstream mixed-node schedule;
- same-pass versus earlier-order restart and the 100-pass guard;
- clone-local parent/child/dependent/constraint/collapsable/interface handles;
- drop order and no source-occurrence back-reference;
- Transform→World dirt gating, parent opacity polymorphism, and constraint
  application order;
- Node computed-local lazy invalidation, including a singular parent;
- Bone/Skin/Tendon retained links, authored order, packed Weight variants,
  accumulated Skin dirt → Skinnable callback → Artboard callback → dependent
  order, distinct PointsPath(Path dirt) versus Mesh(Vertices dirt), clone/drop
  isolation, and zero unchanged-frame buffer rebuild;
- every generated constraint field's default, equality short-circuit, sole
  backing assignment, positive or intentional-no-op changed callback,
  notification order, and callback-free deserialize; computed
  velocity/active writes must leave speed, drag flags, physics running state,
  and scroll activity unchanged while notifying after every non-equal getter
  comparison, including a repeated request;
- FollowPath measure rebuild only on dirt and IK chain construction only at
  lifecycle sites;
- Drag/physics/ScrollConstraint/virtualizer/ScrollBar exact lifecycle,
  including two-child final-only settlement, missing-transform non-increment,
  forced incomplete settlement, transform-before-count ordering, and
  virtualized/non-virtualized list constraint order;
- advancing/resetting mixed object-order, flags, one call per owner,
  advancing→DataBind→updatePass interleaving, residual-dirt return, exact
  ScrollConstraint advance order/return, and empty/non-empty reset;
- Container, Solo, LayoutComponent, and ParentTraversal exact hierarchy and
  virtual collapse semantics;
- Drawable add-time validation, clipping order, hidden/collapse, exact layout
  identity, and hit proxy routing;
- direct math helper oracle fixtures.

Structural counters must include:

- Component owner resolutions, dirt additions, and dirt consumptions;
- dependency builds/sorts and update passes;
- constraint applications, FollowPath measure rebuilds, and IK chain builds;
- ScrollPhysics advances, child applies, and virtualizer settlements;
- Skin buffer rebuilds;
- advancing/resetting dispatches;
- per-frame allocations and owner rediscovery.

An unchanged frame must perform zero internal owner rediscovery and zero
derived-state rebuild for dirt-gated owners. The canonical corpus compares
whole-wave counter totals; individual scenes are never work slices.

## Floor and landing contract

Each internal landing runs targeted tests, runtime tests, formatting, lint,
the frame-loop checker, and applicable parity floors. Before the FL-A merge or
push, run the complete unchanged floor:

- `cargo test -p nuxie-runtime --lib`
- `cargo test -p nuxie --lib`
- `env -u CPP_CONFIG -u RUST_PROFILE make golden-compare`
- `env -u CPP_CONFIG -u RUST_PROFILE make scripted-golden-compare`
- `env -u CPP_CONFIG -u RUST_PROFILE make cpp-oracle-workspace-tests`
- `make renderer-golden`
- `make capi-smoke`
- `make apple-runtime-check`
- `make lint-gate`
- `cargo fmt --all -- --check`
- `git diff --check`
- `make runtime-frame-loop-port-check`
- `make size-report` on the committed tree

Every runtime and nuxie test must pass (the pre-FL-A floor contains 414 and
140 respectively), and every C++ probe must pass (the pre-FL-A floor contains
721). Added regression tests increase those totals; they never replace or
weaken an existing test. Ordinary and scripted remain 317/317 entries with
647/647 exact segments, renderer remains 1,468/1,468 with zero divergence or
gated failure, and both SDK variants remain below 9,437,184 bytes.

After all rows are closed and floors are green, run one canonical
whole-corpus `perf-hot-loop` checkpoint with unchanged provenance and validity
checks. Record the number as FL-A acceptance evidence and continue to FL-B
from the dependency map. Do not sort scenes into the next queue.
