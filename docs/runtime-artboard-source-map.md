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

## Direct file map

| Pinned C++ owner | Rust destination | Initial extraction boundary | Current status or collision |
|---|---|---|---|
| `artboard.hpp`, `artboard.cpp` | `artboard.rs` | instance state, construction, clone, dimensions, ordered update/advance orchestration | retained orchestration; active listener/frame-loop work |
| `advancing_component.hpp`, `advancing_component.cpp` | `advancing_component.rs` | retained advancing entry and schedule builder | extracted |
| `resetting_component.hpp`, `resetting_component.cpp` | `resetting_component.rs` | retained reset entry, schedule, and reset dispatch | extracted |
| `event.hpp`, `event.cpp` | `event.rs` | event/custom-property projection | active live-event work |
| `artboard_component_list.hpp`, `artboard_component_list.cpp` | `artboard_component_list.rs` | retained rows, context, sync/create, advance/reset | virtualized mounted-item methods extracted; active focus installation remains |
| `virtualizing_component.hpp`, `virtualizing_component.cpp` | `virtualizing_component.rs` | exact component-to-virtualizer adapter | extracted |
| `nested_artboard.hpp`, `nested_artboard.cpp` | `nested_artboard.rs` | retained child occurrence, collection, replacement, context, advance | active nested/focus work |
| `nested_artboard_layout.hpp`, `nested_artboard_layout.cpp` | `nested_artboard_layout.rs` | retained layout bounds/cache transfer | ready |
| `nested_artboard_origin.hpp`, `nested_artboard_origin.cpp` | `nested_artboard_origin.rs` | origin callback and builder override | ready |
| `nested_animation.hpp`, `nested_animation.cpp` | `animation/nested_animation.rs` | common nested-animation occurrence and dispatch | active visibility change |
| `nested_linear_animation.hpp`, `nested_linear_animation.cpp` | `animation/nested_linear_animation.rs` | retained mix owner | ready |
| `nested_simple_animation.hpp`, `nested_simple_animation.cpp` | `animation/nested_simple_animation.rs` | speed/play owner and builder | ready |
| `nested_remap_animation.hpp`, `nested_remap_animation.cpp` | `animation/nested_remap_animation.rs` | remap-time owner and builder | ready |
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
| `weight.hpp`, `weight.cpp` | `bones/weight.rs` | retained weight state and `Weight::deform` | state/deform extracted; `onAddedDirty` remains in Artboard construction |
| `cubic_weight.hpp` | `bones/cubic_weight.rs` | independent in/out retained translations | retained state extracted |
| `vertex.hpp`, `vertex.cpp` | `shapes/vertex.rs` | weight attachment, render translation, base deformation dispatch | ready |
| `cubic_vertex.hpp`, `cubic_vertex.cpp` | `shapes/cubic_vertex.rs` | in/out point cache and cubic deformation dispatch | ready |
| `transform_component.hpp`, `transform_component.cpp` | `transform_component.rs` | transform facade, dirtying, and concrete update | ready |
| `node.hpp`, `node.cpp` | `node.rs` | computed local transform | ready |
| `world_transform_component.hpp`, `world_transform_component.cpp` | `world_transform_component.rs` | world transform and render-opacity propagation | ready |
| `bone.hpp`, `bone.cpp` | `bones/bone.rs` | bone relation, length, and callback | ready |
| `skin.hpp`, `skin.cpp` | `bones/skin.rs` | tendon/bone buffer ownership and dirty/update behavior | ready |
| `component.hpp`, `component.cpp` | `component.rs` | component facade, dirt base, and collapse base | one active type-name accessor |
| `container_component.hpp`, `container_component.cpp` | `container_component.rs` | parent/child relation and guarded subtree recursion | ready |
| `drawable.hpp`, `drawable.cpp` | `drawable.rs` | drawable hit testing; renderer traversal remains renderer-owned | ready |
| `layout_component.hpp`, `layout_component.cpp` | `layout_component.rs` | hit test, advance, dirt, update, collapse | ready before its semantic port |
| `layout_component_style.hpp`, `layout_component_style.cpp` | `layout_component_style.rs` | style owner callbacks and interpolation | ready before its semantic port |
| `text_input.hpp`, `text_input.cpp` | existing `text_input.rs` | move remaining Artboard advance/property/update fragments | ready |
| `text_style.hpp`, `text_style.cpp` | `text/text_style.rs` | dependencies, font overrides, and shape dirt | ready before its semantic port |
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
focused tests, and lands independently. Mapping/status ledgers are reconciled
once by the integration owner at the frozen family boundary.
