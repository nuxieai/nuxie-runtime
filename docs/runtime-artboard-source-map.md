# Artboard source-file decomposition

This map is the structural companion to the frame-loop port. It was derived
from Rust commit `95a601b757a56084e44384a3bd35efb4254a3e7c` and pinned C++
commit `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

The target convention is:

- one meaningful C++ `.hpp`/`.cpp` owner family maps to one similarly named
  Rust `.rs` file;
- `artboard.rs` retains `ArtboardInstance`, construction, authored-order
  orchestration, and the small stable public facade;
- concrete owner behavior moves to child modules, which may implement
  `ArtboardInstance` methods without exposing its private storage;
- mixed dispatches stay in `artboard.rs` only as thin ordered dispatch;
- a file move is behavior-preserving. Semantic corrections remain in their
  mapped frame-loop owner-family slice.

Status terms are deliberately strict:

- **extracted** means the stated initial boundary now lives in the direct Rust
  file on this structural branch;
- **partial** means a direct Rust file exists, but named C++ members still live
  elsewhere and are listed in the row;
- **queued** means the destination and retention boundary are mapped but no
  move has landed;
- **active collision** means the current listener/action writer owns the same
  code and the structural move waits for that family boundary.

This branch is a staging branch, not a frame-loop promotion candidate. File
paths in both mechanical ledgers are updated here, but every existing
faithfulness/verification status remains unchanged. The active listener/action
family must integrate these moves at its frozen boundary, rebuild the
provenance-bound trace runners, refresh the source fingerprint/trace evidence,
repair every moved member's lifecycle citations, and make the canonical
structural check green before the combined tree can land. The staging branch
does not rewrite semantic trace evidence or claim that pre-move line citations
remain valid for behavior-preserving file moves.

## Direct file map

| Pinned C++ owner | Rust destination | Initial extraction boundary | Current status or collision |
|---|---|---|---|
| `artboard.hpp`, `artboard.cpp` | `artboard.rs` | instance state, construction, clone, dimensions, ordered update/advance orchestration | retained orchestration; active listener/frame-loop work |
| `advancing_component.hpp`, `advancing_component.cpp` | `advancing_component.rs` | retained advancing entry and schedule builder | extracted |
| `resetting_component.hpp`, `resetting_component.cpp` | `resetting_component.rs` | retained reset entry, schedule, and reset dispatch | extracted |
| `event.hpp`, `event.cpp` | `event.rs` | event/custom-property projection | active live-event work |
| `artboard_component_list.hpp`, `artboard_component_list.cpp` | `artboard_component_list.rs` | retained rows, context, sync/create, advance/reset | partial: virtualized mounted-item methods extracted; retained row storage, context, create/sync, advance, reset, and active focus installation remain in `artboard.rs` |
| `virtualizing_component.hpp`, `virtualizing_component.cpp` | `virtualizing_component.rs` | exact component-to-virtualizer adapter | extracted |
| `nested_artboard.hpp`, `nested_artboard.cpp` | `nested_artboard.rs` | retained child occurrence, collection, replacement, context, advance | active nested/focus work |
| `nested_artboard_layout.hpp`, `nested_artboard_layout.cpp` | `nested_artboard_layout.rs` | retained layout bounds/cache transfer and neutral Fixed/Fill/Hug override policy | partial: extracted; Taffy materialization remains renderer-owned, while exact `-1` and pinned height-via-width scale semantics remain for its semantic closure |
| `nested_artboard_origin.hpp`, `nested_artboard_origin.cpp` | `nested_artboard_origin.rs` | origin callback and builder override | extracted |
| `nested_animation.hpp`, `nested_animation.cpp` | `animation/nested_animation.rs` | common nested-animation occurrence and dispatch | active visibility change |
| `nested_linear_animation.hpp`, `nested_linear_animation.cpp` | `animation/nested_linear_animation.rs` | retained mix owner | queued behind active listener/action integration |
| `nested_simple_animation.hpp`, `nested_simple_animation.cpp` | `animation/nested_simple_animation.rs` | speed/play owner and builder | queued behind active listener/action integration |
| `nested_remap_animation.hpp`, `nested_remap_animation.cpp` | `animation/nested_remap_animation.rs` | remap-time owner and builder | queued behind active listener/action integration |
| `nested_input.hpp`, `nested_input.cpp` | `animation/nested_input.rs` | common nested-input target lookup | active nested listener actions |
| `nested_bool.hpp`, `nested_bool.cpp` | `animation/nested_bool.rs` | nested bool forwarding | active nested listener actions |
| `nested_number.hpp`, `nested_number.cpp` | `animation/nested_number.rs` | nested number forwarding | active nested listener actions |
| `nested_trigger.hpp`, `nested_trigger.cpp` | `animation/nested_trigger.rs` | repeated trigger callback | active nested listener actions |
| `scripted_object.hpp`, `scripted_object.cpp` | `scripted/scripted_object.rs` | occurrence attachment, context, input, init/retry lifecycle | active scripted-listener closure |
| `scripted_drawable.hpp`, `scripted_drawable.cpp` | `scripted/scripted_drawable.rs` | update/advance lifecycle | active scripted-listener closure |
| `scripted_path_effect.hpp`, `scripted_path_effect.cpp` | `scripted/scripted_path_effect.rs` | attachment, hydration, update/advance | active scripted-listener closure |
| `scripted_data_converter.hpp`, `scripted_data_converter.cpp` | `scripted/scripted_data_converter.rs` | retained converter occurrence and advance entry | active scripted-listener closure |
| `joystick.hpp`, `joystick.cpp` | `joystick.rs` | runtime joystick definition, builder, apply, axis time | extracted |
| `solo.hpp`, `solo.cpp` | `solo.rs` | solo mapping, active child, and collapse propagation | extracted |
| `weight.hpp`, `weight.cpp` | `bones/weight.rs` | retained weight state, `onAddedDirty`, and `Weight::deform` | extracted |
| `cubic_weight.hpp` | `bones/cubic_weight.rs` | independent in/out retained translations | retained state extracted |
| `path_vertex.hpp`, `path_vertex.cpp` | `shapes/path_vertex.rs` | authored-order parent registration and parent-owned Skin/Path geometry dirt bridge | partial: relation and geometry dirt extracted; render-path construction remains in `draw.rs` |
| `vertex.hpp`, `vertex.cpp` | `shapes/vertex.rs` | weight-backed deformation, x/y callback, and render translation | partial: weight lookup/deformation and x/y callback extracted through `shapes/path_vertex.rs`; retained weight storage/clone remains in `components.rs`, attachment remains in `bones/weight.rs`, and render translation remains in `draw.rs` |
| `cubic_vertex.hpp`, `cubic_vertex.cpp` | `shapes/cubic_vertex.rs` | x/y super dispatch, in/out point cache, and cubic deformation dispatch | partial: x/y super dispatch and weighted in/out deformation extracted; Rust still computes points on demand, while point/render selection remains in `draw.rs` |
| `mesh_vertex.hpp`, `mesh_vertex.cpp` | `shapes/mesh_vertex.rs` | authored-order Mesh registration and exact-type x/y geometry dirt | partial: relation and existing property callback extracted; inherited callback parity and render geometry remain for the FL-E semantic wave |
| `contour_mesh_vertex.hpp` (generated base + behaviorless concrete subclass) | `shapes/mesh_vertex.rs` | inherits `MeshVertex::onAddedDirty`; pinned C++ also inherits `markGeometryDirty` | generated/helper boundary: registration uses schema inheritance instead of duplicating an empty module; inherited x/y dirt is a recorded FL-E semantic gap because frozen Rust base dispatches only exact `MeshVertex` |
| `straight_vertex.hpp`, `straight_vertex.cpp` | `shapes/straight_vertex.rs` | radius callback | partial: callback extracted; point construction remains in `draw.rs` |
| `cubic_mirrored_vertex.hpp`, `cubic_mirrored_vertex.cpp` | `shapes/cubic_mirrored_vertex.rs` | rotation/distance callbacks | partial: callbacks extracted; `computeIn`/`computeOut` remain in `draw.rs` |
| `cubic_asymmetric_vertex.hpp`, `cubic_asymmetric_vertex.cpp` | `shapes/cubic_asymmetric_vertex.rs` | rotation/inDistance/outDistance callbacks | partial: callbacks extracted; `computeIn`/`computeOut` remain in `draw.rs` |
| `cubic_detached_vertex.hpp`, `cubic_detached_vertex.cpp` | `shapes/cubic_detached_vertex.rs` | in/out rotation and distance callbacks | partial: callbacks extracted; `computeIn`/`computeOut` remain in `draw.rs` |
| `transform_component.hpp`, `transform_component.cpp` | `transform_component.rs` | transform facade, dirtying, and concrete update | partial: initial owner shard extracted; generic property dispatch, dependency construction, and constraint orchestration remain |
| `node.hpp`, `node.cpp` | `node.rs` | x/y keys and callbacks plus computed local transform | partial: position callbacks and computed-local owner extracted; root-space accessors, layout dirt, and generic property/dependency dispatch remain |
| `world_transform_component.hpp`, `world_transform_component.cpp` | `world_transform_component.rs` | world transform and render-opacity propagation | partial: update shard extracted; ordered Artboard update dispatch remains |
| `bone.hpp`, `bone.cpp` | `bones/bone.rs` | bone relation, length, and callback | partial: state/onDirty/update shard extracted; `tipWorldTranslation` remains in `constraints.rs` |
| `root_bone.hpp`, `root_bone.cpp` | `bones/root_bone.rs` | TransformComponent clean-phase bypass plus x/y keys and dirt callbacks | extracted |
| `skin.hpp`, `skin.cpp` | `bones/skin.rs` | tendon/bone buffer ownership and dirty/update behavior | partial: state/onDirty/update shard extracted; `deform` remains in `draw.rs` and buffer allocation remains in relation construction |
| `component.hpp`, `component.cpp` | `component.rs` | component facade, dirt base, collapse base, and parent hit-test walk | partial: public occurrence facade, retained accessors, exact `addDirt`, and base `hitTestPoint` extracted; relation construction, collapse, concrete dirty dispatch, and one active type-name accessor remain elsewhere |
| `container_component.hpp`, `container_component.cpp` | `container_component.rs` | parent/child relation and guarded subtree recursion | partial: retained-child loop extracted; parent/child construction remains in `objects.rs`/`artboard.rs`, while the pre-existing unchanged-collapse continuation mismatch remains for semantic closure |
| `drawable.hpp`, `drawable.cpp` | `drawable.rs` | drawable hit testing; renderer traversal remains renderer-owned | partial: base hit-test adapter extracted; LayoutComponent/ClippingShape virtual `isHidden` overrides remain for semantic closure |
| `layout_component.hpp`, `layout_component.cpp` | `layout_component.rs` | hit test, advance, dirt, update, collapse | partial: hit-test, interpolation advance, and display-collapse child loop extracted; virtual hidden state, post-loop Collapsable notification, remaining dirt, and update stay queued before the semantic port |
| `layout/layout_component_style.hpp`, `layout/layout_component_style.cpp` | `layout_component_style.rs` | style owner callbacks and interpolation | partial: retained-parent display dirt and animation-style inheritance extracted; remaining layout/style callbacks stay queued for the semantic port |
| `text_input.hpp`, `text_input.cpp` | `text_input.rs` | move remaining Artboard advance/property/update fragments | active collision |
| `text/text_style.hpp`, `text/text_style.cpp` | `text/text_style.rs` | dependencies, font overrides, and shape dirt | partial: existing metric-to-shape dirt bridge extracted; retained dependencies, font overrides, and exact owner-local dirt remain for the text wave |
| `text_value_run.hpp`, `text_value_run.cpp` | `text/text_value_run.rs` | root run lookup/write and shape dirt | Artboard-facing shard extracted; full style/offset/hit owner remains for text wave |
| `text_variation_helper.hpp`, `text_variation_helper.cpp` | `text/text_variation_helper.rs` | variation dependency and update helper | update extracted; authored-order construction/dependency insertion remains |
| `linear_animation_instance.hpp`, `linear_animation_instance.cpp` | `animation/linear_animation_instance.rs` | instance construction, advance, apply, events | extracted |
| `state_machine_instance.hpp`, `state_machine_instance.cpp` | existing `state_machine/instance.rs` | move Artboard facade/advance/apply fragments to the existing owner | active frame-loop work |
| focus owner files | existing focused files under `state_machine/` and `focus.rs` | move remaining Artboard focus installation fragments by exact owner | active listener/focus work |

“Current collision” means extraction waits for the active owner-family commit.
It does not authorize leaving the code in `artboard.rs`.

## Mixed orchestration to dissolve incrementally

The following functions contain several C++ owners. They must not simply be
renamed into another warehouse file.

| Current Artboard cluster | Required decomposition |
|---|---|
| occurrence-relation construction | retain authored-order orchestration; delegate Component/Container, constraints, Skin/Weight, Transform/Path/Bone, TextStyle, Shape/Text, Mesh/PointsPath, and dependency sorting to their owner files |
| path-target initialization | delegate separately to FollowPathConstraint, ClippingShape, and TextFollowPathModifier |
| generic property facade | retain stable reads/writes; move SolidColor, Image, Text, and generated owner callbacks |
| retained advance loop | retain ordered Artboard walk; delegate LayoutComponent, TextInput, ScriptedDataConverter, NestedArtboard, and ComponentList entries |
| dirt dispatch | retain thin virtual dispatch; move Component, Constraint, Skin, Path/Shape, TextStyle, and layout dirt to owners |
| component update | retain `Artboard::updatePass` orchestration; move concrete Transform, WorldTransform, Layout, ComponentList, Skin/Mesh/PointsPath, and scripted updates |
| generated property callbacks | make a thin registry dispatcher to DrawRules, DrawTarget, Solo, LayoutStyle, constraints, nested owners, Text, Gradient, Mesh, Axis, and Artboard |
| collapse dispatch | retain one guarded traversal adapter; delegate Component, ContainerComponent, Solo, LayoutComponent, and TransformComponent behavior |
| epoch/property classifiers | dissolve during the owning semantic wave into C++-shaped dirt; retain only source-cited renderer-adapter state |

## Semantic gaps exposed by the structural audit

The staging branch deliberately does not fix these behaviors inside file-move
commits. They are integration blockers for the owning semantic wave:

| Pinned C++ owner | Existing Rust mismatch preserved by the move | Required integration action |
|---|---|---|
| `src/container_component.cpp:9-20` | the Rust virtual adapter continues into concrete propagation when the base Collapsed bit is already equal; C++ returns immediately | add an owned component/layout gap and reopen the whole-file `faithful` row if the ledger cannot scope it to verified members |
| `src/drawable.cpp`, `src/layout_component.cpp`, `include/rive/shapes/clipping_shape.hpp` | Rust hit testing applies the base hidden/collapsed check where C++ uses concrete virtual `isHidden` overrides | add an owned drawable/layout gap and reopen the whole-file `faithful` row if member scoping is unavailable |
| `src/layout_component.cpp:233-250` | Rust propagates effective display collapse to children but does not perform C++'s post-loop `updateCollapsables()` with the virtual collapsed state | retain the layout row as pending until the semantic layout family closes it |

The mechanical ledgers on this branch retain their previously published status
labels so the staging branch does not masquerade as an acceptance/promotion
commit. The FL-C4 integration owner must reconcile the explicit gaps above into
the canonical gap ledger and member/file status before the combined candidate
can pass structural acceptance.

## Extraction order

1. File-disjoint mechanical moves: Solo, Joystick, the Artboard-owned
   Weight/CubicWeight shards,
   AdvancingComponent, ResettingComponent, VirtualizingComponent,
   LinearAnimationInstance, TextValueRun, and TextVariationHelper.
2. Integrate the active listener/action family, then extract Event,
   ComponentList, NestedInput/NestedArtboard, scripted owners, focus fragments,
   and the StateMachineInstance facade.
3. Extract layout/text owners before their semantic owner-family port.
4. Reduce `artboard.rs` to Artboard-owned state and orchestration, then enforce
   source-file correspondence in the structural checker.

Each extraction branch starts at a frozen base, changes no behavior, runs its
focused tests, and lands independently. Direct file paths are reconciled on
this aggregate branch; trace provenance and any status transition are
reconciled once by the integration owner at the frozen family boundary.
