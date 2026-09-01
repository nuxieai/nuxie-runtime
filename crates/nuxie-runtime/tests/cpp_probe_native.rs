//! Differential tests over the actual pinned Rust owners and the pinned C++ probe.
#![cfg(feature = "tools")]

use nuxie_render_api::{Mat2D, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        keyed_object::KeyedObject, keyed_property::KeyedProperty, linear_animation::LinearAnimation,
    },
    artboard::{Artboard as NativeArtboard, RuntimeArtboardInstanceHandle},
    component_dirt::ComponentDirt,
    constraints::{
        follow_path_constraint::FollowPathConstraint,
        scrolling::scroll_bar_constraint::ScrollBarConstraint,
    },
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::{File as NativeFile, RuntimeFileHandle},
    generated::{component_base::ComponentBase, core_registry::CoreRegistry as NativeCoreRegistry},
    math::mat2d::Mat2D as NativeMat2D,
};
use serde::Deserialize;
use std::path::PathBuf;

mod cpp_probe_support;
use cpp_probe_support::*;

// These are wire observations only. Unknown probe fields are left to the
// remaining probe families; every observation asserted below is retained.
#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}
#[derive(Debug, Deserialize)]
struct CppArtboard {
    #[serde(rename = "objectCount")]
    object_count: usize,
    objects: Vec<Option<CppObject>>,
    #[serde(default)]
    animations: Vec<CppLinearAnimationDefinition>,
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
}
#[derive(Debug, Deserialize)]
struct CppObject {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "isComponent")]
    is_component: bool,
}
#[derive(Debug, Deserialize)]
struct CppLinearAnimationDefinition {
    #[serde(rename = "keyedObjects")]
    keyed_objects: Vec<serde_json::Value>,
}
#[derive(Debug, Deserialize)]
struct CppRuntimeUpdate {
    #[serde(rename = "didUpdate")]
    did_update: bool,
    #[serde(rename = "hasComponentsDirt")]
    has_components_dirt: bool,
    components: Vec<CppRuntimeComponent>,
}
#[derive(Debug, Deserialize)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "graphOrder")]
    graph_order: Option<usize>,
    scheduled: bool,
    dirt: u16,
    collapsed: bool,
    #[serde(rename = "worldTransform")]
    world_transform: Option<[f32; 6]>,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
    #[serde(rename = "renderOpacity")]
    render_opacity: Option<f32>,
}

fn rust_accepts_artboard_instance(bytes: &[u8]) -> bool {
    let mut factory = nuxie_render_api::PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory");
    let Some(file) = NativeFile::import(bytes, factory, None, None, None) else {
        return false;
    };
    file.with_file(|file| file.artboard_default()).is_some()
}

fn read_native_instance_from_bytes(
    bytes: &[u8],
    label: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let mut factory = nuxie_render_api::PersistentFactory::new(RecordingFactory::new());
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained native factory");
    let file = NativeFile::import(bytes, factory, None, None, None)
        .unwrap_or_else(|| panic!("failed to import {label}"));
    let instance = file
        .with_file(|file| file.artboard_default())
        .unwrap_or_else(|| panic!("failed to instantiate native artboard for {label}"));
    (file, instance)
}

fn native_object(instance: &RuntimeArtboardInstanceHandle, local_id: usize) -> CoreHandle {
    instance
        .with_artboard(|artboard| artboard.base.resolve_handle(local_id as u32))
        .unwrap_or_else(|| panic!("missing native object {local_id}"))
}

fn native_double(instance: &RuntimeArtboardInstanceHandle, local_id: usize, key: u16) -> f32 {
    NativeCoreRegistry::get_double_handle(&native_object(instance, local_id), i32::from(key))
        .unwrap_or_else(|| panic!("missing native property {local_id}:{key}"))
}

fn native_set_double(
    instance: &RuntimeArtboardInstanceHandle,
    local_id: usize,
    key: u16,
    value: f32,
) -> bool {
    NativeCoreRegistry::set_double_handle(&native_object(instance, local_id), i32::from(key), value)
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

fn native_animation(instance: &RuntimeArtboardInstanceHandle, index: usize) -> CoreHandle {
    instance
        .with_artboard(|artboard| artboard.base.animation_handle_at(index))
        .unwrap_or_else(|| panic!("missing native animation {index}"))
}

fn native_apply_animation(
    instance: &RuntimeArtboardInstanceHandle,
    index: usize,
    time: f32,
    mix: f32,
) -> bool {
    native_animation(instance, index)
        .with_downcast_mut::<LinearAnimation, _>(|animation| {
            instance.apply_linear_animation(animation, time, mix, None);
        })
        .is_some()
}

fn assert_upstream_follow_path_fixture_matches_target(fixture: &str) {
    let bytes = std::fs::read(cpp_runtime_fixture(fixture))
        .unwrap_or_else(|error| panic!("read {fixture}: {error}"));
    let (_file, artboard) = read_native_instance_from_bytes(&bytes, fixture);
    let named = |name: &str| {
        artboard
            .with_artboard(|artboard| {
                artboard
                    .base
                    .find_handle::<nuxie_runtime::source::component::Component>(name)
            })
            .unwrap_or_else(|| panic!("{fixture} missing {name}"))
    };
    let target = named("target");
    let rectangle = named("rect");

    artboard.update_pass(true);
    let target_world = native_world_transform(&target);
    let rectangle_world = native_world_transform(&rectangle);
    assert_eq!(target_world.0[4], rectangle_world.0[4]);
    assert_eq!(target_world.0[5], rectangle_world.0[5]);
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

#[test]
fn runtime_update_matches_cpp_for_transform_hierarchy() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    let objects = rust.with_artboard(|artboard| artboard.base.objects().to_vec());
    assert_eq!(cpp_artboard.object_count, objects.len());
    assert_eq!(cpp_artboard.objects.len(), objects.len());
    for (local_id, (cpp_object, rust_object)) in
        cpp_artboard.objects.iter().zip(&objects).enumerate()
    {
        if let Some(object) = rust_object {
            assert_eq!(
                rust.with_artboard(|artboard| artboard.base.object_index(object)) as usize,
                local_id
            );
        }
        match (cpp_object, rust_object) {
            (Some(cpp_object), Some(object)) => {
                assert_eq!(cpp_object.local_id, local_id);
                assert_eq!(
                    Some(cpp_object.core_type),
                    object.core_type(),
                    "slot core type mismatch for local {local_id} in {label}"
                );
                assert_eq!(
                    cpp_object.is_component,
                    object.is_type_of(ComponentBase::TYPE_KEY)
                );
            }
            (None, None) => {}
            _ => panic!("slot presence mismatch for local {local_id} in {label}"),
        }
    }

    assert_eq!(cpp_update.did_update, report);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.with_artboard(|artboard| artboard.base.has_component_dirt())
    );

    for cpp_component in &cpp_update.components {
        compare_native_component(cpp_component, &rust, label);
    }
}

#[test]
fn image_mesh_and_in_band_asset_fixtures_match_pinned_cpp_update() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ image/mesh comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for relative in ["tape.riv", "in_band_asset.riv"] {
        let cpp = read_cpp_probe_fixture_with_args(&probe, relative, &[]);
        let fixture = cpp_runtime_fixture(relative);
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture.display()));
        let (_file, rust) = read_native_instance_from_bytes(&bytes, relative);
        let _ = NativeArtboard::update_components_handle(&rust.core_handle());
        let report = NativeArtboard::update_components_handle(&rust.core_handle());
        compare_native_runtime_update(&cpp, &rust, report, relative);
    }
}

#[test]
fn translation_constraint_bone_virtual_offsets_match_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_translation_constraint_bone_offset.riv";
    let bytes = synthetic_runtime_file(8222, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "RootBone", |bytes| {
            push_uint_property(bytes, "RootBone", "parentId", 0);
            push_f32_property(bytes, "RootBone", "length", 10.0);
        });
        push_object_with_properties(bytes, "Bone", |bytes| {
            push_uint_property(bytes, "Bone", "parentId", 1);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 20.0);
            push_f32_property(bytes, "Node", "y", 7.0);
        });
        push_object_with_properties(bytes, "TranslationConstraint", |bytes| {
            push_uint_property(bytes, "TranslationConstraint", "parentId", 2);
            push_uint_property(bytes, "TranslationConstraint", "targetId", 3);
            push_bool_property(bytes, "TranslationConstraint", "offset", true);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    assert_eq!(cpp_update.did_update, report);
    let cpp_bone = cpp_update
        .components
        .iter()
        .find(|component| component.local_id == 2)
        .expect("C++ constrained Bone");
    let rust_bone = native_world_transform(&native_object(&rust, 2));
    compare_mat2d(
        cpp_bone.world_transform,
        Some(rust_bone),
        "world transform",
        2,
        label,
    );
    assert_eq!(
        cpp_bone
            .world_transform
            .map(|matrix| (matrix[4], matrix[5])),
        Some((30.0, 7.0))
    );
}

#[test]
fn transform_constraint_reads_retained_layout_target_bounds_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_transform_constraint_layout_target.riv";
    let bytes = synthetic_runtime_file(8234, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_f32_property(bytes, "LayoutComponent", "width", 200.0);
            push_f32_property(bytes, "LayoutComponent", "height", 100.0);
        });
        push_object_with_properties(bytes, "LayoutComponent", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "LayoutComponent", "width", 100.0);
            push_f32_property(bytes, "LayoutComponent", "height", 50.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 2);
        });
        push_object_with_properties(bytes, "LayoutComponentStyle", |_| {});
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "TransformConstraint", |bytes| {
            push_uint_property(bytes, "TransformConstraint", "parentId", 3);
            push_uint_property(bytes, "TransformConstraint", "targetId", 1);
            push_f32_property(bytes, "TransformConstraint", "originX", 0.5);
            push_f32_property(bytes, "TransformConstraint", "originY", 0.5);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    assert_eq!(cpp_update.did_update, report);
    let cpp_node = cpp_update
        .components
        .iter()
        .find(|component| component.local_id == 3)
        .expect("C++ constrained Node");
    let rust_node = native_world_transform(&native_object(&rust, 3));
    compare_mat2d(
        cpp_node.world_transform,
        Some(rust_node),
        "layout-target constrained world transform",
        3,
        label,
    );
    assert_eq!(
        cpp_node
            .world_transform
            .map(|matrix| (matrix[4], matrix[5])),
        Some((50.0, 25.0))
    );
}

#[test]
fn styleless_artboard_settles_top_level_children_as_column_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/univ_1276_styleless_artboard_root_flow.riv";
    let bytes = synthetic_runtime_file(9871, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_f32_property(bytes, "LayoutComponent", "width", 200.0);
            push_f32_property(bytes, "LayoutComponent", "height", 100.0);
        });
        // Local 1/2: first fixed 100x20 top-level flow item.
        push_object_with_properties(bytes, "LayoutComponent", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "LayoutComponent", "width", 100.0);
            push_f32_property(bytes, "LayoutComponent", "height", 20.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 2);
        });
        push_object_with_properties(bytes, "LayoutComponentStyle", |_| {});
        // Local 3/4: second fixed item; its settled origin discloses the
        // root flow direction (row => x=100, column => y=20).
        push_object_with_properties(bytes, "LayoutComponent", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "LayoutComponent", "width", 100.0);
            push_f32_property(bytes, "LayoutComponent", "height", 20.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 4);
        });
        push_object_with_properties(bytes, "LayoutComponentStyle", |_| {});
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    rust.update_pass(true);
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    for (local, expected_xy) in [(1usize, (0.0, 0.0)), (3usize, (0.0, 20.0))] {
        let cpp_xy = cpp_update
            .components
            .iter()
            .find(|component| component.local_id == local)
            .and_then(|component| component.world_transform)
            .map(|matrix| (matrix[4], matrix[5]));
        assert_eq!(
            cpp_xy,
            Some(expected_xy),
            "C++ settled xy for local {local}"
        );
        let rust_bounds = native_object(&rust, local)
            .with(|object| {
                object
                    .as_layout_component()
                    .map(|layout| layout.layout_bounds())
            })
            .flatten()
            .unwrap_or_else(|| panic!("Rust solved bounds for local {local}"));
        assert_eq!(
            (rust_bounds.left(), rust_bounds.top()),
            expected_xy,
            "Rust settled xy for local {local}"
        );
        assert_eq!(
            (rust_bounds.width(), rust_bounds.height()),
            (100.0, 20.0),
            "Rust settled size for local {local}"
        );
    }
}

#[test]
fn artboard_own_column_style_drives_root_flow_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/univ_1276_artboard_column_style_root_flow.riv";
    let bytes = synthetic_runtime_file(9872, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_f32_property(bytes, "LayoutComponent", "width", 200.0);
            push_f32_property(bytes, "LayoutComponent", "height", 100.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 1);
        });
        // Local 1: the artboard's own root style, column direction.
        push_object_with_properties(bytes, "LayoutComponentStyle", |bytes| {
            push_uint_property(bytes, "LayoutComponentStyle", "flexDirectionValue", 0);
        });
        // Local 2/3: first fixed 100x20 top-level flow item.
        push_object_with_properties(bytes, "LayoutComponent", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "LayoutComponent", "width", 100.0);
            push_f32_property(bytes, "LayoutComponent", "height", 20.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 3);
        });
        push_object_with_properties(bytes, "LayoutComponentStyle", |_| {});
        // Local 4/5: second fixed item; column semantics settle it at y=20.
        push_object_with_properties(bytes, "LayoutComponent", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "LayoutComponent", "width", 100.0);
            push_f32_property(bytes, "LayoutComponent", "height", 20.0);
            push_uint_property(bytes, "LayoutComponent", "styleId", 5);
        });
        push_object_with_properties(bytes, "LayoutComponentStyle", |_| {});
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    rust.update_pass(true);
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    for (local, expected_xy) in [(2usize, (0.0, 0.0)), (4usize, (0.0, 20.0))] {
        let cpp_xy = cpp_update
            .components
            .iter()
            .find(|component| component.local_id == local)
            .and_then(|component| component.world_transform)
            .map(|matrix| (matrix[4], matrix[5]));
        assert_eq!(
            cpp_xy,
            Some(expected_xy),
            "C++ settled xy for local {local}"
        );
        let rust_bounds = native_object(&rust, local)
            .with(|object| {
                object
                    .as_layout_component()
                    .map(|layout| layout.layout_bounds())
            })
            .flatten()
            .unwrap_or_else(|| panic!("Rust solved bounds for local {local}"));
        assert_eq!(
            (rust_bounds.left(), rust_bounds.top()),
            expected_xy,
            "Rust settled xy for local {local}"
        );
        assert_eq!(
            (rust_bounds.width(), rust_bounds.height()),
            (100.0, 20.0),
            "Rust settled size for local {local}"
        );
    }
}

#[test]
fn follow_path_constraint_retains_measure_and_applies_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_follow_path_retained_measure.riv";
    let bytes = synthetic_runtime_file(8235, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
            push_f32_property(bytes, "StraightVertex", "x", 10.0);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "FollowPathConstraint", |bytes| {
            push_uint_property(bytes, "FollowPathConstraint", "parentId", 5);
            push_uint_property(bytes, "FollowPathConstraint", "targetId", 1);
            push_f32_property(bytes, "FollowPathConstraint", "distance", 0.5);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    assert_eq!(cpp_update.did_update, report);
    for local_id in [1, 2, 5, 6] {
        let cpp_component = cpp_update
            .components
            .iter()
            .find(|component| component.local_id == local_id)
            .unwrap_or_else(|| panic!("missing C++ component {local_id}"));
        compare_native_component(cpp_component, &rust, label);
    }

    // StraightVertex is not a dependency-graph node in pinned C++; its
    // graphOrder scalar is therefore indeterminate and is intentionally not
    // compared. The constrained Node proves FollowPath::update retained and
    // consumed the path measure (`follow_path_constraint.cpp:119-145`).
    let cpp_node = cpp_update
        .components
        .iter()
        .find(|component| component.local_id == 5)
        .expect("C++ constrained Node");
    let rust_node = native_world_transform(&native_object(&rust, 5));
    compare_mat2d(
        cpp_node.world_transform,
        Some(rust_node),
        "follow-path constrained world transform",
        5,
        label,
    );
    assert_eq!(
        cpp_node
            .world_transform
            .map(|matrix| (matrix[4], matrix[5])),
        Some((5.0, 0.0))
    );
}

#[test]
fn follow_path_offset_reads_the_constrained_occurrence_without_reborrowing_it() {
    let label = "synthetic/runtime_follow_path_offset_borrow.riv";
    let bytes = synthetic_runtime_file(8237, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
            push_f32_property(bytes, "StraightVertex", "x", 10.0);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 7.0);
        });
        push_object_with_properties(bytes, "FollowPathConstraint", |bytes| {
            push_uint_property(bytes, "FollowPathConstraint", "parentId", 5);
            push_uint_property(bytes, "FollowPathConstraint", "targetId", 1);
            push_f32_property(bytes, "FollowPathConstraint", "distance", 0.5);
            push_bool_property(bytes, "FollowPathConstraint", "offset", true);
        });
    });

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    NativeArtboard::update_components_handle(&rust.core_handle());
    let world = native_world_transform(&native_object(&rust, 5));
    assert_eq!((world.0[4], world.0[5]), (12.0, 0.0));
}

#[test]
fn translation_constraint_can_target_its_own_parent_occurrence() {
    let label = "synthetic/runtime_translation_constraint_self_target.riv";
    let bytes = synthetic_runtime_file(8238, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
            push_f32_property(bytes, "Node", "x", 11.0);
            push_f32_property(bytes, "Node", "y", 13.0);
        });
        push_object_with_properties(bytes, "TranslationConstraint", |bytes| {
            push_uint_property(bytes, "TranslationConstraint", "parentId", 1);
            push_uint_property(bytes, "TranslationConstraint", "targetId", 1);
        });
    });

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let node = native_object(&rust, 1);
    let applied = native_object(&rust, 2)
        .with_mut(|constraint| constraint.constraint_apply(node.clone()))
        .expect("live TranslationConstraint");
    assert!(applied);
    let world = native_world_transform(&native_object(&rust, 1));
    assert!(world.0.iter().all(|value| value.is_finite()));
}

#[test]
fn follow_path_local_spaces_keep_target_and_component_parents_distinct() {
    let label = "synthetic/runtime_follow_path_local_parent_frames.riv";
    let bytes = synthetic_runtime_file(8239, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
            push_f32_property(bytes, "StraightVertex", "x", 10.0);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "FollowPathConstraint", |bytes| {
            push_uint_property(bytes, "FollowPathConstraint", "parentId", 5);
            push_uint_property(bytes, "FollowPathConstraint", "targetId", 1);
            push_uint_property(bytes, "FollowPathConstraint", "sourceSpaceValue", 1);
            push_uint_property(bytes, "FollowPathConstraint", "destSpaceValue", 1);
        });
    });
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let components = native_object(&rust, 6)
        .with_downcast_mut::<FollowPathConstraint, _>(|constraint| {
            let component_world = NativeMat2D::from_translate(0.0, 0.0);
            let mut target_world = NativeMat2D::from_translate(110.0, 0.0);
            let component_parent_world = NativeMat2D::from_translate(20.0, 0.0);
            let target_parent_world = NativeMat2D::from_translate(100.0, 0.0);
            constraint.constrain_helper(
                &component_world,
                &mut target_world,
                &component_parent_world,
                &target_parent_world,
            )
        })
        .expect("FollowPathConstraint");
    assert_eq!((components.x(), components.y()), (30.0, 0.0));
}

#[test]
fn scroll_bar_constraint_applies_to_its_thumb_without_reborrowing_it() {
    let fixture = "layout/scroll_velocity.riv";
    let bytes = std::fs::read(cpp_runtime_fixture(fixture))
        .unwrap_or_else(|error| panic!("read {fixture}: {error}"));
    let (_file, rust) = read_native_instance_from_bytes(&bytes, fixture);
    NativeArtboard::update_components_handle(&rust.core_handle());
    let constraints =
        rust.with_artboard(|artboard| artboard.find_all_handles::<ScrollBarConstraint>());
    assert!(!constraints.is_empty());
    for constraint in constraints {
        let thumb = constraint
            .with(|constraint| constraint.component_parent_handle())
            .flatten()
            .expect("ScrollBarConstraint thumb");
        let applied = constraint
            .with_mut(|constraint| constraint.constraint_apply(thumb))
            .expect("live ScrollBarConstraint");
        assert!(applied);
    }
}

#[test]
fn upstream_follow_path_constraint_updates_world_transform() {
    assert_upstream_follow_path_fixture_matches_target("follow_path.riv");
}

#[test]
fn upstream_follow_path_with_zero_opacity_constraint_updates_world_transform() {
    assert_upstream_follow_path_fixture_matches_target("follow_path_with_0_opacity.riv");
}

#[test]
fn upstream_follow_path_with_zero_opacity_path_updates_world_transform() {
    assert_upstream_follow_path_fixture_matches_target("follow_path_path_0_opacity.riv");
}

#[test]
fn list_follow_path_constraint_registers_and_updates_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_list_follow_path_registration.riv";
    let bytes = synthetic_runtime_file(8236, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "ArtboardComponentList", |bytes| {
            push_uint_property(bytes, "ArtboardComponentList", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 2);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 3);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 3);
            push_f32_property(bytes, "StraightVertex", "x", 10.0);
        });
        push_object_with_properties(bytes, "ListFollowPathConstraint", |bytes| {
            push_uint_property(bytes, "ListFollowPathConstraint", "parentId", 1);
            push_uint_property(bytes, "ListFollowPathConstraint", "targetId", 2);
            push_f32_property(bytes, "ListFollowPathConstraint", "distanceEnd", 1.0);
        });
    });

    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());
    let cpp_update = cpp.artboards[0]
        .runtime_update
        .as_ref()
        .expect("C++ runtime update");
    assert_eq!(cpp_update.did_update, report);
    for local_id in [1, 2, 3, 6] {
        let cpp_component = cpp_update
            .components
            .iter()
            .find(|component| component.local_id == local_id)
            .unwrap_or_else(|| panic!("missing C++ component {local_id}"));
        compare_native_component(cpp_component, &rust, label);
    }
}

#[test]
fn mutated_instance_transform_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_mutated_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let x_key = property_key_for_name("Node", "x");
    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-set-double".to_owned(),
            "1".to_owned(),
            x_key.to_string(),
            "12.0".to_owned(),
        ],
    );
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    assert!(native_set_double(
        &rust,
        1,
        property_key_for_name("Node", "x"),
        12.0
    ));
    let report = NativeArtboard::update_components_handle(&rust.core_handle());

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    assert_eq!(cpp_update.did_update, report);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.with_artboard(|artboard| artboard.base.has_component_dirt())
    );

    for cpp_component in &cpp_update.components {
        compare_native_component(cpp_component, &rust, label);
    }
}

#[test]
fn mutable_instance_transform_does_not_mutate_source_object() {
    let label = "synthetic/runtime_transform_source_separation.riv";
    let bytes = synthetic_transform_hierarchy();
    let (runtime, rust) = read_native_instance_from_bytes(&bytes, label);
    let source_artboard = runtime
        .with_file(|file| file.artboard_at_source(0))
        .expect("source Artboard");
    let source_child = source_artboard
        .with_downcast::<NativeArtboard, _>(|artboard| artboard.resolve_handle(1))
        .flatten()
        .expect("source child should exist");

    assert_eq!(
        NativeCoreRegistry::get_double_handle(
            &source_child,
            i32::from(property_key_for_name("Node", "x"))
        ),
        Some(2.0)
    );

    assert!(native_set_double(
        &rust,
        1,
        property_key_for_name("Node", "x"),
        12.0
    ));
    assert_eq!(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        12.0
    );
    assert_eq!(
        NativeCoreRegistry::get_double_handle(
            &source_child,
            i32::from(property_key_for_name("Node", "x"))
        ),
        Some(2.0),
        "mutating instance transform state must not mutate imported source data"
    );

    NativeArtboard::update_components_handle(&rust.core_handle());

    assert_eq!(
        native_object(&rust, 1)
            .with(|object| object.as_transform_component().unwrap().transform()[4])
            .unwrap(),
        12.0
    );
    assert_eq!(
        NativeCoreRegistry::get_double_handle(
            &source_child,
            i32::from(property_key_for_name("Node", "x"))
        ),
        Some(2.0)
    );
}

#[test]
fn linear_animation_apply_interpolates_keyframe_double_transform() {
    let label = "synthetic/runtime_linear_animation_interpolation.riv";
    let bytes = synthetic_linear_animation(8201, 0, 2.0, 10, 12.0, 1, false);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);

    assert_eq!(
        rust.with_artboard(|artboard| artboard.base.animation_count()),
        1
    );
    let keyed_object = native_animation(&rust, 0)
        .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects()[0].clone())
        .unwrap();
    let keyed_property = keyed_object
        .with_downcast::<KeyedObject, _>(|object| object.keyed_properties()[0].clone())
        .unwrap();
    assert_eq!(
        keyed_property
            .with_downcast::<KeyedProperty, _>(|property| property.keyframes().len())
            .unwrap(),
        2
    );

    assert!(native_apply_animation(&rust, 0, 0.5, 1.0));
    assert_close(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        7.0,
        "interpolated x",
    );
    assert!(
        native_object(&rust, 1)
            .with(|object| object
                .as_component()
                .unwrap()
                .dirt()
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM))
            .unwrap()
    );
}

#[test]
fn linear_animation_apply_holds_when_interpolation_type_is_zero() {
    let label = "synthetic/runtime_linear_animation_hold.riv";
    let bytes = synthetic_linear_animation(8202, 0, 4.0, 10, 12.0, 0, false);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);

    assert!(native_apply_animation(&rust, 0, 0.5, 1.0));
    assert_close(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        4.0,
        "held x",
    );
}

#[test]
fn linear_animation_apply_uses_first_and_last_keyframes_outside_range() {
    let before_label = "synthetic/runtime_linear_animation_before_first.riv";
    let before_bytes = synthetic_linear_animation(8203, 10, 4.0, 20, 12.0, 1, false);
    let (_before_file, before) = read_native_instance_from_bytes(&before_bytes, before_label);

    assert!(native_apply_animation(&before, 0, 0.0, 1.0));
    assert_close(
        native_double(&before, 1, property_key_for_name("Node", "x")),
        4.0,
        "before first x",
    );

    let after_label = "synthetic/runtime_linear_animation_after_last.riv";
    let after_bytes = synthetic_linear_animation(8204, 0, 4.0, 10, 12.0, 1, false);
    let (_after_file, after) = read_native_instance_from_bytes(&after_bytes, after_label);

    assert!(native_apply_animation(&after, 0, 2.0, 1.0));
    assert_close(
        native_double(&after, 1, property_key_for_name("Node", "x")),
        12.0,
        "after last x",
    );
}

#[test]
fn linear_animation_apply_quantizes_seconds_before_sampling() {
    let label = "synthetic/runtime_linear_animation_quantized.riv";
    let bytes = synthetic_linear_animation(8205, 0, 2.0, 10, 12.0, 1, true);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);

    assert!(native_apply_animation(&rust, 0, 0.99, 1.0));
    assert_close(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        11.0,
        "quantized x",
    );
}

#[test]
fn linear_animation_apply_mixes_with_current_transform_value() {
    let label = "synthetic/runtime_linear_animation_mixed.riv";
    let bytes = synthetic_linear_animation(8206, 0, 2.0, 10, 12.0, 1, false);
    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);

    assert!(native_apply_animation(&rust, 0, 1.0, 0.25));
    assert_close(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        4.5,
        "mixed x",
    );
}

#[test]
fn linear_animation_apply_does_not_mutate_source_object() {
    let label = "synthetic/runtime_linear_animation_source_separation.riv";
    let bytes = synthetic_linear_animation(8207, 0, 2.0, 10, 12.0, 1, false);
    let (runtime, rust) = read_native_instance_from_bytes(&bytes, label);
    let source_artboard = runtime
        .with_file(|file| file.artboard_at_source(0))
        .expect("source Artboard");
    let source_child = source_artboard
        .with_downcast::<NativeArtboard, _>(|artboard| artboard.resolve_handle(1))
        .flatten()
        .expect("source child should exist");

    assert_eq!(
        NativeCoreRegistry::get_double_handle(
            &source_child,
            i32::from(property_key_for_name("Node", "x"))
        ),
        Some(2.0)
    );

    assert!(native_apply_animation(&rust, 0, 1.0, 1.0));
    assert_close(
        native_double(&rust, 1, property_key_for_name("Node", "x")),
        12.0,
        "applied x",
    );
    assert_eq!(
        NativeCoreRegistry::get_double_handle(
            &source_child,
            i32::from(property_key_for_name("Node", "x"))
        ),
        Some(2.0)
    );
}

#[test]
fn linear_animation_apply_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_cpp_exact.riv",
            synthetic_linear_animation(8211, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_before_first.riv",
            synthetic_linear_animation(8212, 10, 4.0, 20, 12.0, 1, false),
            0.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_interpolated.riv",
            synthetic_linear_animation(8213, 0, 2.0, 10, 12.0, 1, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_held.riv",
            synthetic_linear_animation(8214, 0, 4.0, 10, 12.0, 0, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_after_last.riv",
            synthetic_linear_animation(8215, 0, 4.0, 10, 12.0, 1, false),
            2.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_quantized.riv",
            synthetic_linear_animation(8216, 0, 2.0, 10, 12.0, 1, true),
            0.99,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_mixed.riv",
            synthetic_linear_animation(8217, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-apply-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
        native_apply_animation(&rust, 0, seconds, mix);
        let report = NativeArtboard::update_components_handle(&rust.core_handle());

        compare_native_runtime_update(&cpp, &rust, report, label);
    }
}

#[test]
fn invalid_keyframe_interpolator_erases_entire_keyed_object_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_linear_animation_wrong_type_interpolator_cpp.riv";
    let bytes = synthetic_linear_animation_wrong_type_interpolator(8218);
    let args = [
        "--runtime-apply-animation".to_owned(),
        "0".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
    ];
    let cpp = read_cpp_probe_bytes_with_args(&probe, label, &bytes, &args);
    assert_eq!(cpp.artboards[0].animations[0].keyed_objects.len(), 0);

    let (_file, rust) = read_native_instance_from_bytes(&bytes, label);
    assert_eq!(
        native_animation(&rust, 0)
            .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().len())
            .expect("Rust animation"),
        cpp.artboards[0].animations[0].keyed_objects.len(),
        "wrong-type interpolator must erase the keyed object, including its valid sibling property"
    );
    native_apply_animation(&rust, 0, 0.0, 1.0);
    let report = NativeArtboard::update_components_handle(&rust.core_handle());
    compare_native_runtime_update(&cpp, &rust, report, label);
}

#[test]
fn state_machine_added_phases_match_cpp() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let runtime_root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let source_path = PathBuf::from(runtime_root).join("src/animation/state_machine.cpp");
    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", source_path.display()));
    let member_body = |start: &str, end: &str| {
        let start = source.find(start).expect("pinned member start");
        let end = source[start..]
            .find(end)
            .map(|offset| start + offset)
            .expect("pinned member end");
        &source[start..end]
    };

    for (member, next) in [
        (
            "StatusCode StateMachine::onAddedDirty",
            "StatusCode StateMachine::onAddedClean",
        ),
        (
            "StatusCode StateMachine::onAddedClean",
            "StatusCode StateMachine::import",
        ),
    ] {
        let body = member_body(member, next);
        let inputs = body.find("m_Inputs").expect("inputs phase");
        let layers = body.find("m_Layers").expect("layers phase");
        let listeners = body.find("m_Listeners").expect("listeners phase");
        assert!(
            inputs < layers && layers < listeners,
            "{member} phase order"
        );
        assert_eq!(
            body.matches("return code;").count(),
            3,
            "{member} must stop on the first child failure"
        );
        assert!(
            !body.contains("m_dataBinds") && !body.contains("m_scriptedObjects"),
            "{member} must not visit definition collections outside the pinned three phases"
        );
    }

    let import = member_body(
        "StatusCode StateMachine::import",
        "void StateMachine::addLayer",
    );
    assert!(
        import.find("artboardImporter == nullptr") < import.find("addStateMachine(this)"),
        "missing importer must fail before attachment"
    );
    assert!(
        import.find("addStateMachine(this)") < import.find("Super::import(importStack)"),
        "attachment must precede the superclass import status"
    );

    // A layer failure is observable through both real runtimes. The source
    // comparison above locks the earlier input callbacks and later-listener
    // suppression; the focused Rust unit test records the retained earlier
    // callbacks to prove the documented no-rollback adaptation.
    let label = "synthetic/fl_c5_added_dirty_invalid_layer.riv";
    let bytes = synthetic_state_machine_missing_system_state(90_505, "ExitState");
    assert!(!cpp_probe_accepts_bytes(&probe, label, &bytes));
    assert!(!rust_accepts_artboard_instance(&bytes));
}

#[test]
fn state_machine_required_system_states_reject_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (index, missing) in ["AnyState", "EntryState", "ExitState"]
        .into_iter()
        .enumerate()
    {
        let label = format!("synthetic/runtime_state_machine_missing_{missing}_cpp.riv");
        let bytes = synthetic_state_machine_missing_system_state(8270 + index as u64, missing);
        assert!(
            !cpp_probe_accepts_bytes(&probe, &label, &bytes),
            "pinned C++ must reject a layer missing {missing}"
        );
        assert!(
            !rust_accepts_artboard_instance(&bytes),
            "Rust must reject a layer missing {missing}"
        );
    }
}

#[test]
fn state_machine_bad_transition_targets_reject_like_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    for (index, state_to_id) in [u64::MAX, 99].into_iter().enumerate() {
        let label = format!("synthetic/runtime_state_machine_bad_target_{state_to_id}_cpp.riv");
        let bytes = synthetic_state_machine_bad_transition_target(8274 + index as u64, state_to_id);
        assert!(
            !cpp_probe_accepts_bytes(&probe, &label, &bytes),
            "pinned C++ must reject stateToId={state_to_id}"
        );
        assert!(
            !rust_accepts_artboard_instance(&bytes),
            "Rust must reject stateToId={state_to_id}"
        );
    }
}
