//! Native file/import and basic animation differentials against the pinned C++ probe.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        keyed_object::KeyedObject, keyed_property::KeyedProperty,
        linear_animation::LinearAnimation, linear_animation_instance::LinearAnimationInstance,
        nested_remap_animation::NestedRemapAnimation,
    },
    artboard::{Artboard as NativeArtboard, RuntimeArtboardInstanceHandle},
    assets::script_asset::ScriptAsset,
    component::Component,
    component_origin::ComponentOrigin,
    constraints::{
        rotation_constraint::RotationConstraint, scale_constraint::ScaleConstraint,
        translation_constraint::TranslationConstraint,
    },
    core::{CoreHandle, CoreObject, CoreType},
    factory::RuntimeFactoryHandle,
    file::{File as NativeFile, ImportResult, RuntimeFileHandle},
    generated::{
        artboard_base::ArtboardBase,
        assets::{image_asset_base::ImageAssetBase, script_asset_base::ScriptAssetBase},
        core_registry::CoreRegistry as NativeCoreRegistry,
        nested_artboard_base::NestedArtboardBase,
        node_base::NodeBase,
    },
    layout::{layout_component_style::LayoutComponentStyle, layout_enums::LayoutScaleType},
    layout_component::LayoutComponent,
    math::mat2d::Mat2D as NativeMat2D,
    status_code::StatusCode,
};
use serde::Deserialize;
use std::collections::HashSet;

mod cpp_probe_support;
use cpp_probe_support::*;

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppArtboard {
    animations: Vec<CppLinearAnimationDefinition>,
    #[serde(default)]
    nested_remap_animations: Vec<CppNestedRemapAnimation>,
    #[serde(default)]
    runtime_animation_advances: Vec<CppRuntimeAnimationAdvance>,
    runtime_update: Option<CppRuntimeUpdate>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppLinearAnimationDefinition {
    keyed_objects: Vec<CppKeyedObjectDefinition>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppKeyedObjectDefinition {
    keyed_properties: Vec<CppKeyedPropertyDefinition>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppKeyedPropertyDefinition {
    key_frames: Vec<CppKeyFrameDefinition>,
}
#[derive(Debug, Deserialize)]
struct CppKeyFrameDefinition {
    frame: u64,
    seconds: f32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppNestedRemapAnimation {
    local_id: usize,
    animation_time: Option<f32>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppRuntimeAnimationAdvance {
    animation_index: usize,
    advanced: bool,
    keep_going: bool,
    time: f32,
    direction: f32,
    directed_speed: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    did_loop: bool,
    loop_value: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppRuntimeUpdate {
    did_update: bool,
    has_components_dirt: bool,
    components: Vec<CppRuntimeComponent>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppRuntimeComponent {
    local_id: usize,
    graph_order: Option<usize>,
    scheduled: bool,
    dirt: u16,
    collapsed: bool,
    world_transform: Option<[f32; 6]>,
    local_transform: Option<[f32; 6]>,
    render_opacity: Option<f32>,
}

fn read_native_file(bytes: &[u8], label: &str) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory");
    NativeFile::import(bytes, factory, None, None, None)
        .unwrap_or_else(|| panic!("failed to import {label}"))
}
fn read_native_instance_from_bytes(
    bytes: &[u8],
    label: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let file = read_native_file(bytes, label);
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("native artboard instance");
    (file, artboard)
}
fn native_named<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.base.find_handle::<T>(name))
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn native_parent(object: &CoreHandle) -> CoreHandle {
    object
        .with(|object| object.as_component().unwrap().parent_handle())
        .flatten()
        .expect("component parent")
}
fn native_name(object: &CoreHandle) -> String {
    object
        .with(|object| object.as_component().unwrap().name().to_owned())
        .unwrap()
}
fn native_keyed_objects(artboard: &RuntimeArtboardInstanceHandle, index: usize) -> Vec<CoreHandle> {
    native_animation(artboard, index)
        .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().to_vec())
        .unwrap()
}
fn native_frames(artboard: &RuntimeArtboardInstanceHandle, index: usize) -> Vec<CoreHandle> {
    let keyed = native_keyed_objects(artboard, index);
    let property = keyed[0]
        .with_downcast::<KeyedObject, _>(|object| object.keyed_properties()[0].clone())
        .unwrap();
    property
        .with_downcast::<KeyedProperty, _>(|property| property.keyframes().to_vec())
        .unwrap()
}
fn set_double(object: &CoreHandle, owner: &str, property: &str, value: f32) {
    assert!(NativeCoreRegistry::set_double_handle(
        object,
        i32::from(property_key_for_name(owner, property)),
        value
    ));
}

fn native_object(instance: &RuntimeArtboardInstanceHandle, local_id: usize) -> CoreHandle {
    instance
        .with_artboard(|artboard| artboard.base.resolve_handle(local_id as u32))
        .unwrap_or_else(|| panic!("missing native object {local_id}"))
}

fn native_world_transform(object: &CoreHandle) -> Mat2D {
    object
        .with(|object| {
            Mat2D(
                *object
                    .as_world_transform_component()
                    .expect("native WorldTransformComponent")
                    .world_transform()
                    .values(),
            )
        })
        .expect("live native transform")
}

fn fixed_layout_box(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|artboard| {
            artboard
                .objects_typed::<LayoutComponent>()
                .iter()
                .filter(|object| !object.is_type_of(ArtboardBase::TYPE_KEY))
                .filter(|object| {
                    object
                        .with(|object| object.as_layout_component().unwrap().style_handle())
                        .flatten()
                        .is_some_and(|style| {
                            style
                                .with_downcast::<LayoutComponentStyle, _>(|style| {
                                    !style.is_stack()
                                        && style.width_scale_type() != LayoutScaleType::Fill
                                })
                                .unwrap()
                        })
                })
                .last()
        })
        .expect("fixed 40x40 box")
}

fn add_layout_origin(
    artboard: &RuntimeArtboardInstanceHandle,
    owner: &CoreHandle,
    x: f32,
    y: f32,
) -> CoreHandle {
    let origin = artboard
        .core_handle()
        .insert_sibling(ComponentOrigin::default())
        .expect("insert ComponentOrigin");
    artboard.with_artboard_mut(|artboard| artboard.base.add_object(Some(origin.clone())));
    let parent_id = artboard.with_artboard(|artboard| artboard.base.id_of(owner));
    assert!(NativeCoreRegistry::set_uint_handle(
        &origin,
        i32::from(property_key_for_name("Component", "parentId")),
        parent_id,
    ));
    let status = artboard.with_artboard_mut(|artboard| {
        origin
            .with_mut(|origin| origin.on_added_dirty(&mut artboard.base))
            .expect("live ComponentOrigin")
    });
    assert_eq!(status, StatusCode::Ok);
    set_double(&origin, "ComponentOrigin", "originX", x);
    set_double(&origin, "ComponentOrigin", "originY", y);
    origin
}

fn add_layout_constraint<T: CoreObject + Default>(
    artboard: &RuntimeArtboardInstanceHandle,
    owner: &CoreHandle,
) -> CoreHandle {
    let constraint = artboard
        .core_handle()
        .insert_sibling(T::default())
        .expect("insert constraint");
    artboard.with_artboard_mut(|artboard| artboard.base.add_object(Some(constraint.clone())));
    let target = artboard.core_handle();
    let (parent_id, target_id) = artboard
        .with_artboard(|artboard| (artboard.base.id_of(owner), artboard.base.id_of(&target)));
    assert!(NativeCoreRegistry::set_uint_handle(
        &constraint,
        i32::from(property_key_for_name("Component", "parentId")),
        parent_id,
    ));
    assert!(NativeCoreRegistry::set_uint_handle(
        &constraint,
        i32::from(property_key_for_name("TargetedConstraint", "targetId")),
        target_id,
    ));
    let status = artboard.with_artboard_mut(|artboard| {
        constraint
            .with_mut(|constraint| constraint.on_added_dirty(&mut artboard.base))
            .expect("live constraint")
    });
    assert_eq!(status, StatusCode::Ok);
    constraint
}

fn native_layout_anchor(object: &CoreHandle) -> (f32, f32) {
    object
        .with(|object| {
            let layout = object.as_layout_component().expect("LayoutComponent");
            let anchor = layout.local_anchor();
            let world = layout.base.world_transform();
            (
                world[0] * anchor.x + world[2] * anchor.y + world[4],
                world[1] * anchor.x + world[3] * anchor.y + world[5],
            )
        })
        .expect("live LayoutComponent")
}

fn native_animation(instance: &RuntimeArtboardInstanceHandle, index: usize) -> CoreHandle {
    instance
        .with_artboard(|artboard| artboard.base.animation_handle_at(index))
        .unwrap_or_else(|| panic!("missing native animation {index}"))
}

fn compare_native_runtime_update(
    cpp: &CppProbeFile,
    rust: &RuntimeArtboardInstanceHandle,
    did_update: bool,
    label: &str,
) {
    let cpp_update = cpp
        .artboards
        .first()
        .and_then(|artboard| artboard.runtime_update.as_ref())
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));
    assert_eq!(cpp_update.did_update, did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.with_artboard(|artboard| artboard.base.has_component_dirt())
    );
    for cpp_component in &cpp_update.components {
        compare_native_component(cpp_component, rust, label);
    }
}

fn compare_native_component(
    cpp: &CppRuntimeComponent,
    artboard: &RuntimeArtboardInstanceHandle,
    label: &str,
) {
    let object = native_object(artboard, cpp.local_id);
    let scheduled = artboard.with_artboard(|artboard| {
        artboard
            .base
            .dependency_order()
            .iter()
            .any(|component| component.authored() == Some(&object))
    });
    let (graph_order, dirt, collapsed, local, world, opacity) = object
        .with(|object| {
            let component = object.as_component().expect("native Component");
            (
                scheduled.then_some(component.graph_order() as usize),
                component.dirt().0,
                object
                    .as_layout_component()
                    .map(|layout| layout.is_collapsed())
                    .unwrap_or_else(|| component.is_collapsed()),
                object
                    .as_transform_component()
                    .map(|transform| Mat2D(*transform.transform().values())),
                object
                    .as_world_transform_component()
                    .map(|transform| Mat2D(*transform.world_transform().values())),
                object
                    .as_transform_component()
                    .map(|transform| transform.render_opacity()),
            )
        })
        .expect("live native Component");
    assert_eq!(
        cpp.scheduled, scheduled,
        "schedule membership mismatch for local {} in {label}",
        cpp.local_id
    );
    if cpp.scheduled {
        assert_eq!(
            cpp.graph_order, graph_order,
            "graph order mismatch for scheduled local {} in {label}",
            cpp.local_id
        );
        assert!(
            cpp.graph_order.is_some(),
            "scheduled local {} omitted graph order in {label}",
            cpp.local_id
        );
    } else {
        assert_eq!(
            None, cpp.graph_order,
            "unscheduled local {} exposed indeterminate C++ graph order in {label}",
            cpp.local_id
        );
        assert_eq!(
            None, graph_order,
            "unscheduled local {} manufactured graph order in {label}",
            cpp.local_id
        );
    }
    assert_eq!(
        cpp.dirt, dirt,
        "dirt mismatch for local {} in {label}",
        cpp.local_id
    );
    assert_eq!(
        cpp.collapsed, collapsed,
        "collapsed flag mismatch for local {} in {label}",
        cpp.local_id
    );
    compare_mat2d(
        cpp.local_transform,
        local,
        "local transform",
        cpp.local_id,
        label,
    );
    compare_mat2d(
        cpp.world_transform,
        world,
        "world transform",
        cpp.local_id,
        label,
    );
    compare_optional_f32(
        cpp.render_opacity,
        opacity,
        "render opacity",
        cpp.local_id,
        label,
    );
}

fn compare_animation_advance(
    cpp: &CppRuntimeAnimationAdvance,
    animation_index: usize,
    rust: &LinearAnimationInstance,
    keep_going: bool,
    keep_going_after: bool,
    label: &str,
) {
    assert_eq!(cpp.animation_index, animation_index, "{label}");
    assert_eq!(cpp.advanced, keep_going, "{label} advance return mismatch");
    assert_eq!(
        cpp.keep_going, keep_going_after,
        "{label} keepGoing mismatch"
    );
    assert_close(cpp.time, rust.time(), &format!("{label} time"));
    assert_close(
        cpp.direction,
        rust.direction(),
        &format!("{label} direction"),
    );
    assert_close(
        cpp.directed_speed,
        rust.directed_speed(),
        &format!("{label} directedSpeed"),
    );
    assert_close(
        cpp.total_time,
        rust.total_time(),
        &format!("{label} totalTime"),
    );
    assert_close(
        cpp.last_total_time,
        rust.last_total_time(),
        &format!("{label} lastTotalTime"),
    );
    assert_close(
        cpp.spilled_time,
        rust.spilled_time(),
        &format!("{label} spilledTime"),
    );
    assert_eq!(cpp.did_loop, rust.did_loop(), "{label} didLoop mismatch");
    assert_eq!(
        cpp.loop_value,
        i64::from(rust.loop_value()),
        "{label} loopValue override mismatch"
    );
}

fn synthetic_two_animation_importer_cursor(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 10, 12.0, 1);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 20);
            push_uint_property(bytes, "LinearAnimation", "duration", 40);
        });
        push_keyframe_double(bytes, 20, 22.0, 0);
    })
}

fn synthetic_invalid_keyed_object_property_sink(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 0, 2.0, 1);
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 99);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, 10, 12.0, 0);
    })
}

fn synthetic_negative_speed_nested_remap(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 0);
            push_uint_property(bytes, "NestedArtboard", "artboardId", 1);
        });
        push_object_with_properties(bytes, "NestedRemapAnimation", |bytes| {
            push_uint_property(bytes, "Component", "parentId", 1);
            push_uint_property(bytes, "NestedAnimation", "animationId", 0);
            push_f32_property(bytes, "NestedRemapAnimation", "time", 0.0);
        });
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
            push_f32_property(bytes, "LinearAnimation", "speed", -1.0);
            push_uint_property(bytes, "LinearAnimation", "loopValue", 0);
        });
    })
}

fn synthetic_two_animation_loop_values(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "loopValue", 1);
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "loopValue", 2);
        });
    })
}

#[test]
fn upstream_component_origin_overrides_the_mounted_instance_origin() {
    let bytes = std::fs::read(cpp_runtime_fixture("nested_artboard_opacity.riv"))
        .expect("read nested-artboard fixture");
    let file = read_native_file(&bytes, "nested_artboard_opacity.riv");
    let artboard = file
        .with_file(|file| file.artboard_named("Parent Artboard"))
        .expect("Parent Artboard");
    artboard.update_pass(true);
    let nested = native_named::<nuxie_runtime::source::nested_artboard::NestedArtboard>(
        &artboard,
        "Nested artboard container",
    );
    let child = nested
        .with(|nested| {
            nested
                .as_nested_artboard()
                .unwrap()
                .artboard_instance_handle(0)
        })
        .flatten()
        .expect("mounted child");
    set_double(&child.core_handle(), "Artboard", "originX", 0.0);
    set_double(&child.core_handle(), "Artboard", "originY", 0.0);
    nuxie_runtime::source::nested_artboard::NestedArtboard::apply_origin_override_occurrence(
        &nested,
    );
    assert_eq!(child.with_artboard(|child| child.origin_x()), 0.0);
    assert_eq!(child.with_artboard(|child| child.origin_y()), 0.0);
    assert_eq!(
        artboard.with_artboard(|artboard| artboard
            .objects()
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(NestedArtboardBase::TYPE_KEY))
            .count()),
        1
    );

    // Exact pinned addChild/addObject sequence; no implicit lifecycle.
    let origin = artboard
        .core_handle()
        .insert_sibling(ComponentOrigin::default())
        .unwrap();
    set_double(&origin, "ComponentOrigin", "originX", 0.25);
    set_double(&origin, "ComponentOrigin", "originY", 0.75);
    nested.with_mut(|nested| {
        nested
            .as_container_component_mut()
            .unwrap()
            .add_child(origin.clone())
    });
    artboard.with_artboard_mut(|artboard| artboard.base.add_object(Some(origin)));
    nuxie_runtime::source::nested_artboard::NestedArtboard::apply_origin_override_occurrence(
        &nested,
    );
    assert_eq!(child.with_artboard(|child| child.origin_x()), 0.25);
    assert_eq!(child.with_artboard(|child| child.origin_y()), 0.75);
}

#[test]
fn upstream_component_origin_pivots_a_layout_transform() {
    let bytes = std::fs::read(cpp_runtime_fixture("layout/stack.riv")).expect("read stack fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "layout/stack.riv");
    artboard.advance_default(0.0);
    let layouts = artboard.with_artboard(|artboard| {
        artboard
            .objects_typed::<LayoutComponent>()
            .iter()
            .collect::<Vec<_>>()
    });
    let layout = layouts
        .into_iter()
        .filter(|object| !object.is_type_of(ArtboardBase::TYPE_KEY))
        .filter(|object| {
            object
                .with(|object| object.as_layout_component().unwrap().style_handle())
                .flatten()
                .is_some_and(|style| {
                    style
                        .with_downcast::<LayoutComponentStyle, _>(|style| {
                            !style.is_stack() && style.width_scale_type() != LayoutScaleType::Fill
                        })
                        .unwrap()
                })
        })
        .last()
        .expect("fixed 40x40 box");
    assert_eq!(
        layout.with(|object| object.as_layout_component().unwrap().layout_width()),
        Some(40.0)
    );
    assert_eq!(
        layout.with(|object| object.as_layout_component().unwrap().layout_height()),
        Some(40.0)
    );
    set_double(
        &layout,
        "TransformComponent",
        "rotation",
        std::f32::consts::FRAC_PI_2,
    );
    artboard.advance_default(0.0);
    let world = native_world_transform(&layout);
    assert_close(world.0[0], 0.0, "box world xx");
    assert_close(world.0[1], 1.0, "box world xy");
    assert_close(world.0[4], 160.0, "box world tx");
    assert_close(world.0[5], 160.0, "box world ty");

    let origin = artboard
        .core_handle()
        .insert_sibling(ComponentOrigin::default())
        .unwrap();
    artboard.with_artboard_mut(|artboard| artboard.base.add_object(Some(origin.clone())));
    let parent_id = artboard.with_artboard(|artboard| artboard.base.id_of(&layout));
    assert!(NativeCoreRegistry::set_uint_handle(
        &origin,
        i32::from(property_key_for_name("Component", "parentId")),
        parent_id
    ));
    let status = artboard.with_artboard_mut(|artboard| {
        origin
            .with_mut(|origin| origin.on_added_dirty(&mut artboard.base))
            .unwrap()
    });
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(native_parent(&origin), layout);
    set_double(&origin, "ComponentOrigin", "originX", 0.5);
    set_double(&origin, "ComponentOrigin", "originY", 0.5);
    artboard.advance_default(0.0);
    let world = native_world_transform(&layout);
    assert_close(world.0[4], 200.0, "pivoted box world tx");
    assert_close(world.0[5], 160.0, "pivoted box world ty");
    layout.with(|object| {
        let layout = object.as_layout_component().unwrap();
        assert_close(layout.origin_offset().x, 20.0, "layout origin offset");
        assert_close(
            layout.local_bounds().left(),
            0.0,
            "layout local bounds left",
        );
        assert_close(layout.local_anchor().x, 20.0, "layout local anchor");
    });
}

#[test]
fn upstream_translation_constraint_lands_a_layout_origin_on_the_target() {
    let bytes = std::fs::read(cpp_runtime_fixture("layout/stack.riv")).expect("read stack fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "layout/stack.riv");
    artboard.advance_default(0.0);
    let layout = fixed_layout_box(&artboard);
    add_layout_origin(&artboard, &layout, 0.5, 0.5);
    let constraint = add_layout_constraint::<TranslationConstraint>(&artboard, &layout);
    constraint
        .with_downcast_mut::<TranslationConstraint, _>(|constraint| {
            constraint.base.mark_constraint_dirty()
        })
        .expect("TranslationConstraint");
    artboard.advance_default(0.0);

    let anchor = native_layout_anchor(&layout);
    let target = native_world_transform(&artboard.core_handle());
    let world = native_world_transform(&layout);
    assert_close(anchor.0, target.0[4], "translation anchor x");
    assert_close(anchor.1, target.0[5], "translation anchor y");
    assert_close(world.0[4], target.0[4] - 20.0, "translation corner x");
    assert_close(world.0[5], target.0[5] - 20.0, "translation corner y");
}

#[test]
fn upstream_translation_constraint_without_origin_is_uncorrected() {
    let bytes = std::fs::read(cpp_runtime_fixture("layout/stack.riv")).expect("read stack fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "layout/stack.riv");
    artboard.advance_default(0.0);
    let layout = fixed_layout_box(&artboard);
    let constraint = add_layout_constraint::<TranslationConstraint>(&artboard, &layout);
    constraint
        .with_downcast_mut::<TranslationConstraint, _>(|constraint| {
            constraint.base.mark_constraint_dirty()
        })
        .expect("TranslationConstraint");
    artboard.advance_default(0.0);

    let target = native_world_transform(&artboard.core_handle());
    let world = native_world_transform(&layout);
    layout.with(|object| assert_eq!(object.as_layout_component().unwrap().local_anchor().x, 0.0));
    assert_close(world.0[4], target.0[4], "uncorrected corner x");
    assert_close(world.0[5], target.0[5], "uncorrected corner y");
}

#[test]
fn upstream_rotation_constraint_keeps_a_layout_anchor_fixed() {
    let bytes = std::fs::read(cpp_runtime_fixture("layout/stack.riv")).expect("read stack fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "layout/stack.riv");
    let layout = fixed_layout_box(&artboard);
    add_layout_origin(&artboard, &layout, 0.5, 0.5);
    set_double(
        &layout,
        "TransformComponent",
        "rotation",
        std::f32::consts::FRAC_PI_2,
    );
    artboard.advance_default(0.0);
    let before = native_layout_anchor(&layout);
    let linear_before = native_world_transform(&layout).0[0];

    let constraint = add_layout_constraint::<RotationConstraint>(&artboard, &layout);
    constraint
        .with_downcast_mut::<RotationConstraint, _>(|constraint| {
            constraint.base.mark_constraint_dirty()
        })
        .expect("RotationConstraint");
    artboard.advance_default(0.0);
    let after = native_layout_anchor(&layout);
    let linear_after = native_world_transform(&layout).0[0];
    assert_close(after.0, before.0, "rotation anchor x");
    assert_close(after.1, before.1, "rotation anchor y");
    assert!((linear_after - linear_before).abs() > 0.01);
}

#[test]
fn upstream_scale_constraint_keeps_a_layout_anchor_fixed() {
    let bytes = std::fs::read(cpp_runtime_fixture("layout/stack.riv")).expect("read stack fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "layout/stack.riv");
    let layout = fixed_layout_box(&artboard);
    add_layout_origin(&artboard, &layout, 0.5, 0.5);
    set_double(&layout, "TransformComponent", "scaleX", 2.0);
    set_double(&layout, "TransformComponent", "scaleY", 2.0);
    artboard.advance_default(0.0);
    let before = native_layout_anchor(&layout);
    let linear_before = native_world_transform(&layout).0[0];

    let constraint = add_layout_constraint::<ScaleConstraint>(&artboard, &layout);
    constraint
        .with_downcast_mut::<ScaleConstraint, _>(|constraint| {
            constraint.base.mark_constraint_dirty()
        })
        .expect("ScaleConstraint");
    artboard.advance_default(0.0);
    let after = native_layout_anchor(&layout);
    let linear_after = native_world_transform(&layout).0[0];
    assert_close(after.0, before.0, "scale anchor x");
    assert_close(after.1, before.1, "scale anchor y");
    assert!((linear_after - linear_before).abs() > 0.01);
}

#[test]
fn upstream_transform_order_is_as_expected() {
    let translation = NativeMat2D::new(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
    let rotation = NativeMat2D::from_rotation(3.14 / 2.0);
    let scale = NativeMat2D::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0);
    let composed = translation * rotation * scale;
    let mut explicit = NativeMat2D::from_rotation(3.14 / 2.0);
    explicit[0] *= 2.0;
    explicit[1] *= 2.0;
    explicit[2] *= 3.0;
    explicit[3] *= 3.0;
    explicit[4] = 10.0;
    explicit[5] = 20.0;
    assert_eq!(explicit, composed);
}

#[test]
fn upstream_file_with_animation_can_be_read() {
    let bytes = std::fs::read(cpp_runtime_fixture("juice.riv")).expect("read juice.riv");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "juice.riv");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.name().to_owned()),
        "New Artboard"
    );
    let shin = native_named::<Component>(&artboard, "shin_right");
    assert_eq!(shin.core_type(), Some(NodeBase::TYPE_KEY));
    let leg = native_parent(&shin);
    assert_eq!(native_name(&leg), "leg_right");
    let root = native_parent(&leg);
    assert_eq!(native_name(&root), "root");
    assert_eq!(native_parent(&root), artboard.core_handle());
    let walk = artboard
        .with_artboard(|artboard| artboard.base.animation_named("walk"))
        .expect("walk animation");
    assert_eq!(
        walk.with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().len()),
        Some(22)
    );
}

#[test]
fn upstream_file_dependencies_are_as_expected() {
    let bytes =
        std::fs::read(cpp_runtime_fixture("dependency_test.riv")).expect("read dependency fixture");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "dependency_test.riv");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.name().to_owned()),
        "Blue"
    );
    let named = |name| native_named::<Component>(&artboard, name);
    let node_a = named("A");
    let node_b = named("B");
    let node_c = named("C");
    let shape = named("Rectangle");
    let path = named("Rectangle Path");
    assert_eq!(native_parent(&node_a), artboard.core_handle());
    assert_eq!(native_parent(&node_b), node_a);
    assert_eq!(native_parent(&node_c), node_b);
    assert_eq!(native_parent(&shape), node_b);
    assert_eq!(native_parent(&path), shape);
    assert_eq!(
        node_b.with(|object| object.as_component().unwrap().dependents().len()),
        Some(2)
    );
    let graph_order = |object: &CoreHandle| {
        object
            .with(|object| object.as_component().unwrap().graph_order())
            .unwrap()
    };
    let artboard_graph_order = graph_order(&artboard.core_handle());
    assert_eq!(artboard_graph_order, 0);
    assert!(graph_order(&node_a) > artboard_graph_order);
    assert!(graph_order(&node_b) > graph_order(&node_a));
    assert!(graph_order(&node_c) > graph_order(&node_b));
    assert!(graph_order(&shape) > graph_order(&node_b));
    assert!(graph_order(&path) > graph_order(&shape));
    artboard.update_pass(true);
    let world = native_world_transform(&shape);
    assert_eq!(world.0[4], 39.203125);
    assert_eq!(world.0[5], 29.535156);
}

#[test]
fn upstream_long_name_object_is_parsed_correctly() {
    let bytes = std::fs::read(cpp_runtime_fixture("long_name.riv")).expect("read long_name.riv");
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, "long_name.riv");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.objects().len()),
        7
    );
}

#[test]
fn upstream_file_with_in_band_images_can_have_them_stripped() {
    let bytes =
        std::fs::read(cpp_runtime_fixture("jellyfish_test.riv")).expect("read jellyfish fixture");
    let (file, artboard) = read_native_instance_from_bytes(&bytes, "jellyfish_test.riv");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.name().to_owned()),
        "Jellyfish"
    );
    assert!(file.with_file(|file| {
        file.assets()
            .iter()
            .any(|asset| asset.is_type_of(ImageAssetBase::TYPE_KEY))
    }));
    let mut result = ImportResult::Malformed;
    let unchanged = NativeFile::strip_assets(&bytes, &HashSet::new(), Some(&mut result));
    assert_eq!(result, ImportResult::Success);
    assert_eq!(unchanged.len(), bytes.len());
    assert_eq!(unchanged, bytes);
    let stripped = NativeFile::strip_assets(
        &bytes,
        &HashSet::from([ImageAssetBase::TYPE_KEY]),
        Some(&mut result),
    );
    assert_eq!(result, ImportResult::Success);
    assert!(stripped.len() < bytes.len());
}

#[test]
fn upstream_file_with_bad_keyed_property_loads() {
    let bytes = std::fs::read(cpp_runtime_fixture("magic_alley_db_reduced_export.riv"))
        .expect("read bad keyed-property fixture");
    let (_file, artboard) =
        read_native_instance_from_bytes(&bytes, "magic_alley_db_reduced_export.riv");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.name().to_owned()),
        "Artboard"
    );
    artboard.update_pass(true);
}

#[test]
fn upstream_file_can_be_read_with_verified_signed_scripts() {
    let bytes =
        std::fs::read(cpp_runtime_fixture("joel_signed.riv")).expect("read signed-script fixture");
    let file = read_native_file(&bytes, "joel_signed.riv");
    let scripts = file.with_file(|file| {
        file.assets()
            .iter()
            .filter(|asset| asset.is_type_of(ScriptAssetBase::TYPE_KEY))
            .cloned()
            .collect::<Vec<_>>()
    });
    assert!(!scripts.is_empty());
    for script in scripts {
        assert!(
            script
                .with_downcast::<ScriptAsset, _>(ScriptAsset::verified)
                .unwrap()
        );
    }
}

#[test]
fn keyed_property_importer_cursor_survives_next_animation_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_two_animation_importer_cursor_cpp.riv";
    let bytes = synthetic_two_animation_importer_cursor(8219);
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let cpp_frames =
        &cpp.artboards[0].animations[0].keyed_objects[0].keyed_properties[0].key_frames;
    assert_eq!(
        cpp_frames
            .iter()
            .map(|frame| (frame.frame, frame.seconds))
            .collect::<Vec<_>>(),
        vec![(10, 1.0), (20, 2.0)]
    );
    assert!(cpp.artboards[0].animations[1].keyed_objects.is_empty());

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let rust_frame_times = native_frames(&rust,0).iter().map(|frame| {
        assert!(frame.is_type_of(nuxie_runtime::source::generated::animation::keyframe_double_base::KeyFrameDoubleBase::TYPE_KEY));
        frame.with(|object| {
            let frame=object.as_key_frame().unwrap();
            (u64::from(frame.frame()),frame.seconds())
        }).unwrap()
    }).collect::<Vec<_>>();
    assert_eq!(
        rust_frame_times,
        cpp_frames
            .iter()
            .map(|frame| (frame.frame, frame.seconds))
            .collect::<Vec<_>>()
    );
    assert!(native_keyed_objects(&rust, 1).is_empty());
}

#[test]
fn invalid_keyed_object_replaces_property_cursor_with_sink_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_invalid_keyed_object_property_sink_cpp.riv";
    let bytes = synthetic_invalid_keyed_object_property_sink(8229);
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let cpp_frames =
        &cpp.artboards[0].animations[0].keyed_objects[0].keyed_properties[0].key_frames;
    assert_eq!(cpp_frames.len(), 1);

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    assert_eq!(native_keyed_objects(&rust, 0).len(), 1);
    let rust_frames = native_frames(&rust, 0);
    assert_eq!(
        rust_frames.len(),
        cpp_frames.len(),
        "the frame owned by the invalid keyed object must be erased with its replacement property"
    );
}

#[test]
fn negative_speed_nested_remap_uses_effective_start_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_negative_speed_nested_remap_cpp.riv";
    let bytes = synthetic_negative_speed_nested_remap(8220);
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let cpp_report = cpp.artboards[0]
        .nested_remap_animations
        .first()
        .unwrap_or_else(|| panic!("missing C++ nested remap report for {label}"));
    assert_eq!(cpp_report.local_id, 2);
    assert_close(
        cpp_report
            .animation_time
            .unwrap_or_else(|| panic!("missing C++ remap animation for {label}")),
        2.0,
        "C++ negative-speed remap time",
    );

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let remap = rust
        .with_artboard(|artboard| artboard.objects_typed::<NestedRemapAnimation>().first())
        .expect("native remap animation");
    assert_eq!(
        rust.with_artboard(|artboard| artboard.id_of(&remap)) as usize,
        cpp_report.local_id
    );
    let animation_time = remap
        .with_downcast::<NestedRemapAnimation, _>(|remap| {
            remap
                .base
                .base
                .animation_instance()
                .expect("native nested animation instance")
                .time()
        })
        .unwrap();
    assert_close(
        animation_time,
        cpp_report.animation_time.unwrap(),
        "Rust negative-speed remap time",
    );
}

#[test]
fn linear_animation_instance_advance_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_instance_one_shot.riv",
            synthetic_linear_animation_with_options(
                8221,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            1.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_loop.riv",
            synthetic_linear_animation_with_options(
                8222,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 1,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_ping_pong.riv",
            synthetic_linear_animation_with_options(
                8223,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 2,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_work_area.riv",
            synthetic_linear_animation_with_options(
                8224,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 20,
                    enable_work_area: true,
                    work_start: 2,
                    work_end: 8,
                    ..Default::default()
                },
            ),
            0.3,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_negative_speed.riv",
            synthetic_linear_animation_with_options(
                8225,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    speed: -1.0,
                    ..Default::default()
                },
            ),
            0.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_mixed.riv",
            synthetic_linear_animation_with_options(
                8226,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            0.5,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-advance-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        let mut animation = rust
            .animation_at(0)
            .unwrap_or_else(|| panic!("missing Rust animation instance for {label}"));
        let keep_going = animation.advance(seconds, None);
        animation.apply(mix);
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        let cpp_animation = cpp_artboard
            .runtime_animation_advances
            .first()
            .unwrap_or_else(|| panic!("missing C++ animation advance report for {label}"));
        let keep_going_after = animation.keep_going();
        compare_animation_advance(
            cpp_animation,
            0,
            &animation,
            keep_going,
            keep_going_after,
            label,
        );
        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn linear_animation_loop_value_definition_fallback_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_linear_animation_loop_value_signed_cpp.riv";
    let bytes = synthetic_linear_animation_with_options(
        8227,
        0,
        2.0,
        10,
        12.0,
        1,
        LinearAnimationFixtureOptions {
            loop_value: 1,
            ..Default::default()
        },
    );
    let cpp_fallback = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-advance-animation".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    );
    let cpp_fallback_value = cpp_fallback.artboards[0].runtime_animation_advances[0].loop_value;
    assert_eq!(cpp_fallback_value, 1);

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let animation = rust
        .animation_at(0)
        .unwrap_or_else(|| panic!("missing Rust animation instance for {label}"));
    assert_eq!(i64::from(animation.loop_value()), cpp_fallback_value);
}

#[test]
fn linear_animation_loop_value_arbitrary_signed_override_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_linear_animation_loop_value_signed_cpp.riv";
    let bytes = synthetic_linear_animation_with_options(
        8227,
        0,
        2.0,
        10,
        12.0,
        1,
        LinearAnimationFixtureOptions {
            loop_value: 1,
            ..Default::default()
        },
    );
    let cpp_override = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-set-animation-loop-value".to_owned(),
            "0".to_owned(),
            "-2".to_owned(),
            "--runtime-advance-animation".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    );
    let cpp_override_value = cpp_override.artboards[0].runtime_animation_advances[0].loop_value;
    assert_eq!(cpp_override_value, -2);

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let mut animation = rust
        .animation_at(0)
        .unwrap_or_else(|| panic!("missing Rust animation instance for {label}"));
    animation.set_loop_value(-2);
    assert_eq!(i64::from(animation.loop_value()), cpp_override_value);
}

#[test]
fn linear_animation_loop_value_uses_retained_definition_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_linear_animation_retained_loop_value_cpp.riv";
    let bytes = synthetic_two_animation_loop_values(8228);
    let cpp_fallback = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-advance-animation".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    );
    let cpp_fallback_value = cpp_fallback.artboards[0].runtime_animation_advances[0].loop_value;
    assert_eq!(cpp_fallback_value, 1);

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let mut animation = rust
        .animation_at(0)
        .unwrap_or_else(|| panic!("missing Rust animation instance A for {label}"));
    let mismatched_definition = native_animation(&rust, 1);
    assert_eq!(
        mismatched_definition
            .with_downcast::<LinearAnimation, _>(|animation| animation.base.loop_value()),
        Some(2)
    );
    drop(rust);
    assert_eq!(
        i64::from(animation.loop_value()),
        cpp_fallback_value,
        "instance A must retain definition A instead of consulting definition B"
    );

    let cpp_override = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-set-animation-loop-value".to_owned(),
            "0".to_owned(),
            "2".to_owned(),
            "--runtime-advance-animation".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    );
    let cpp_override_value = cpp_override.artboards[0].runtime_animation_advances[0].loop_value;
    assert_eq!(cpp_override_value, 2);
    animation.set_loop_value(2);
    assert_eq!(i64::from(animation.loop_value()), cpp_override_value);
}
