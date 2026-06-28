use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::{ArtboardHostKind, DependencyKind, GraphFile};
use rive_schema::definition_by_name;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(path: &str) -> PathBuf {
    repo_root().join("fixtures").join(path)
}

fn reference_runtime_dir() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn reference_unit_fixture_dir() -> PathBuf {
    reference_runtime_dir().join("tests/unit_tests/assets")
}

fn reference_fixture(path: &str) -> PathBuf {
    reference_unit_fixture_dir().join(path)
}

fn default_probe_path() -> PathBuf {
    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };

    repo_root()
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe")
}

fn probe_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RIVE_CPP_PROBE") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Some(path);
        }
        return Some(repo_root().join(path));
    }

    let path = default_probe_path();
    path.exists().then_some(path)
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key_for_name(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return property.key.int;
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)
            .unwrap_or_else(|| panic!("missing ancestor schema definition {ancestor}"));
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return property.key.int;
        }
    }

    panic!("missing property {type_name}.{property_name}");
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, uint_properties: &[(u16, u64)]) {
    push_var_uint(bytes, u64::from(type_key_for_name(type_name)));
    for (key, value) in uint_properties {
        push_var_uint(bytes, u64::from(*key));
        push_var_uint(bytes, *value);
    }
    push_var_uint(bytes, 0);
}

fn push_object_with_properties(
    bytes: &mut Vec<u8>,
    type_name: &str,
    properties: impl FnOnce(&mut Vec<u8>),
) {
    push_var_uint(bytes, u64::from(type_key_for_name(type_name)));
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value);
}

fn push_bytes_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &[u8]) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn synthetic_runtime_file(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);
    object_stream(&mut bytes);
    bytes
}

#[test]
fn graph_file_collections_follow_runtime_file_cpp_parity() {
    let view_model_id_key = property_key_for_name("ViewModelInstance", "viewModelId");
    let bytes = synthetic_runtime_file(7104, |bytes| {
        push_object(bytes, "ViewModel", &[]);
        push_object(bytes, "ViewModelInstance", &[(view_model_id_key, 0)]);
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "ViewModelInstance", &[(view_model_id_key, 0)]);
        push_object(bytes, "ViewModelInstance", &[(view_model_id_key, 1)]);
        push_object(bytes, "ViewModel", &[]);
        push_object(bytes, "ViewModelPropertyBoolean", &[]);
        push_object(bytes, "ViewModelInstance", &[(view_model_id_key, 1)]);
        push_object(bytes, "DataEnumValue", &[]);
        push_object(bytes, "DataEnumCustom", &[]);
        push_object(bytes, "DataEnumValue", &[]);
        push_object(bytes, "DataEnumSystem", &[]);
        push_object(bytes, "DataEnumValue", &[]);
        push_object(bytes, "DataEnum", &[]);
        push_object(bytes, "DataEnumValue", &[]);
    });

    let (_runtime, graph) = read_graph_from_bytes(&bytes, "synthetic/file_collections.riv");

    assert_eq!(
        graph
            .view_models
            .iter()
            .map(|view_model| view_model.global_id)
            .collect::<Vec<_>>(),
        vec![0, 5]
    );
    assert_eq!(
        graph.view_models[0]
            .instances
            .iter()
            .map(|instance| instance.global_id)
            .collect::<Vec<_>>(),
        vec![3],
        "contextless and future-id ViewModelInstance objects must not leak into GraphFile"
    );
    assert_eq!(
        graph.view_models[1]
            .properties
            .iter()
            .map(|property| property.global_id)
            .collect::<Vec<_>>(),
        vec![6]
    );
    assert_eq!(
        graph.view_models[1]
            .instances
            .iter()
            .map(|instance| instance.global_id)
            .collect::<Vec<_>>(),
        vec![7]
    );

    assert_eq!(
        graph
            .enums
            .iter()
            .map(|item| item.global_id)
            .collect::<Vec<_>>(),
        vec![9, 13],
        "GraphFile should mirror RuntimeFile::data_enums(), excluding DataEnumSystem"
    );
    assert_eq!(
        graph.enums[0]
            .values
            .iter()
            .map(|value| value.global_id)
            .collect::<Vec<_>>(),
        vec![10, 12, 14],
        "DataEnumValue objects continue attaching to the latest DataEnumCustom importer"
    );
    assert!(graph.enums[1].values.is_empty());
}

#[test]
fn graph_excludes_backboard_scroll_physics_from_artboard_locals() {
    let parent_id_key = property_key_for_name("Node", "parentId");
    let bytes = synthetic_runtime_file(7105, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "ElasticScrollPhysics", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
    });

    let (_runtime, graph) = read_graph_from_bytes(&bytes, "synthetic/scroll_physics_locals.riv");

    assert_eq!(
        graph.artboards[0]
            .local_objects
            .iter()
            .map(|object| (object.global_id, object.type_name))
            .collect::<Vec<_>>(),
        vec![(1, Some("Artboard")), (3, Some("Node"))],
        "ScrollPhysics imports through BackboardImporter and must not enter Artboard::objects() just because it inherits Component"
    );
}

#[test]
fn cpp_probe_matches_rust_imported_component_graph_shape_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ probe comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    for fixture_path in [
        "minimal/long_name.riv",
        "minimal/two_artboards.riv",
        "graph/dependency_test.riv",
        "graph/clipping_and_draw_order.riv",
        "animation/smi_test.riv",
        "animation/state_machine_transition.riv",
    ] {
        compare_fixture(&probe, fixture_path);
    }
}

#[test]
fn cpp_probe_matches_rust_file_graph_collections_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ probe comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    for fixture_path in [
        "car_widgets_v01.riv",
        "databind_solo_to_enum.riv",
        "hosted_image_file.riv",
        "library_with_text_and_image.riv",
        "viewmodel_runtime_file.riv",
    ] {
        compare_reference_fixture(&probe, fixture_path);
    }
}

#[test]
fn cpp_probe_matches_rust_keyed_object_pruning_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ probe comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    let keyed_object_id = property_key_for_name("KeyedObject", "objectId");
    let keyed_property_key = property_key_for_name("KeyedProperty", "propertyKey");
    let parent_id_key = property_key_for_name("Node", "parentId");
    let node_x_key = property_key_for_name("Node", "x");
    let artboard_width_key = property_key_for_name("Artboard", "width");

    let bytes = synthetic_runtime_file(7101, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "LinearAnimation", &[]);
        push_object(bytes, "KeyedObject", &[(keyed_object_id, 1)]);
        push_object(
            bytes,
            "KeyedProperty",
            &[(keyed_property_key, u64::from(node_x_key))],
        );
        push_object(
            bytes,
            "KeyedProperty",
            &[(keyed_property_key, u64::from(artboard_width_key))],
        );
        push_object(bytes, "KeyedObject", &[(keyed_object_id, 999)]);
        push_object(
            bytes,
            "KeyedProperty",
            &[(keyed_property_key, u64::from(node_x_key))],
        );
    });

    let label = "synthetic/keyed_object_pruning.riv";
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (runtime, rust) = read_graph_from_bytes(&bytes, label);
    compare_artboards(&cpp, &runtime, &rust, label);

    let animation = &rust.artboards[0].animations[0];
    assert_eq!(animation.keyed_objects.len(), 1);
    assert_eq!(animation.keyed_objects[0].object_id, 1);
    assert_eq!(animation.keyed_objects[0].keyed_properties.len(), 1);
    assert_eq!(
        animation.keyed_objects[0].keyed_properties[0].property_key,
        u64::from(node_x_key)
    );
}

#[test]
fn cpp_probe_matches_rust_draw_graph_resolution_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ probe comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    let parent_id_key = property_key_for_name("Component", "parentId");
    let drawable_id_key = property_key_for_name("DrawTarget", "drawableId");
    let placement_value_key = property_key_for_name("DrawTarget", "placementValue");
    let draw_target_id_key = property_key_for_name("DrawRules", "drawTargetId");
    let source_id_key = property_key_for_name("ClippingShape", "sourceId");
    let fill_rule_key = property_key_for_name("ClippingShape", "fillRule");

    let bytes = synthetic_runtime_file(7102, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "DrawTarget",
            &[
                (parent_id_key, 0),
                (drawable_id_key, 1),
                (placement_value_key, 2),
            ],
        );
        push_object(bytes, "DrawTarget", &[(drawable_id_key, 99)]);
        push_object(
            bytes,
            "DrawRules",
            &[(parent_id_key, 0), (draw_target_id_key, 2)],
        );
        push_object(bytes, "DrawRules", &[(draw_target_id_key, 99)]);
        push_object(
            bytes,
            "ClippingShape",
            &[(parent_id_key, 0), (source_id_key, 1), (fill_rule_key, 1)],
        );
        push_object(bytes, "ClippingShape", &[(source_id_key, 99)]);
    });

    let label = "synthetic/draw_graph_resolution.riv";
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (runtime, rust) = read_graph_from_bytes(&bytes, label);
    compare_artboards(&cpp, &runtime, &rust, label);

    let artboard = &rust.artboards[0];
    assert_eq!(artboard.draw_targets.len(), 2);
    assert_eq!(artboard.draw_targets[0].drawable_local, Some(1));
    assert_eq!(artboard.draw_targets[1].drawable_local, None);
    assert_eq!(artboard.draw_rules.len(), 2);
    assert_eq!(artboard.draw_rules[0].active_target_local, Some(2));
    assert_eq!(artboard.draw_rules[1].active_target_local, None);
    assert_eq!(artboard.clipping_shapes.len(), 2);
    assert_eq!(artboard.clipping_shapes[0].source_local, Some(1));
    assert_eq!(artboard.clipping_shapes[1].source_local, None);
    assert_eq!(
        artboard.lifecycle.build_dependencies_edges,
        artboard.dependency_edges.len()
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 1, DependencyKind::ParentChild))
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 2, DependencyKind::ParentChild)),
        "DrawTarget has import-time references but no C++ buildDependencies parent edge"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 3, DependencyKind::ParentChild)),
        "unresolved DrawTarget has no C++ buildDependencies parent edge"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 4, DependencyKind::ParentChild)),
        "DrawRules has import-time references but no C++ buildDependencies parent edge"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 5, DependencyKind::ParentChild)),
        "unresolved DrawRules has no C++ buildDependencies parent edge"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 6, DependencyKind::ParentChild)),
        "ClippingShape::buildDependencies does not call Super, so resolved clipping shapes do not inherit parent-child dependencies"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(0, 7, DependencyKind::ParentChild)),
        "ClippingShape::buildDependencies does not call Super, so unresolved clipping shapes do not inherit parent-child dependencies"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(1, 2, DependencyKind::DrawTargetDrawable))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 4, DependencyKind::DrawRulesTarget))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(1, 6, DependencyKind::ClippingSource))
    );
    let shape_path_composer_node = dependency_node_for_path_composer(artboard, 1);
    let clipping_shape_node = dependency_node_for_component(artboard, 6);
    let unresolved_clipping_shape_node = dependency_node_for_component(artboard, 7);
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_path_composer_node,
            clipping_shape_node,
            DependencyKind::ClippingShapePathComposer
        )),
        "ClippingShape::buildDependencies makes clipping shapes depend on each source shape's PathComposer"
    );
    assert!(
        !artboard
            .dependency_node_edges
            .iter()
            .any(|edge| edge.dependent_node == unresolved_clipping_shape_node
                && edge.kind == DependencyKind::ClippingShapePathComposer),
        "clipping shapes without resolved source shapes do not receive path-composer prerequisites"
    );
    assert_order_before(artboard, 1, 2);
    assert_order_before(artboard, 2, 4);
    assert_order_before(artboard, 1, 6);
    assert_node_order_before(artboard, shape_path_composer_node, clipping_shape_node);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_parent_child_dependencies_follow_cpp_build_dependency_hooks() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let drawable_id_key = property_key_for_name("DrawTarget", "drawableId");
    let draw_target_id_key = property_key_for_name("DrawRules", "drawTargetId");
    let style_id_key = property_key_for_name("TextValueRun", "styleId");

    let bytes = synthetic_runtime_file(7120, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Text", &[(parent_id_key, 0)]);
        push_object(bytes, "TextStyle", &[(parent_id_key, 1)]);
        push_object(bytes, "FocusData", &[(parent_id_key, 0)]);
        push_object(bytes, "SemanticData", &[(parent_id_key, 0)]);
        push_object(bytes, "Image", &[(parent_id_key, 0)]);
        push_object(bytes, "NSlicer", &[(parent_id_key, 5)]);
        push_object(bytes, "AxisX", &[(parent_id_key, 6)]);
        push_object(
            bytes,
            "DrawTarget",
            &[(parent_id_key, 0), (drawable_id_key, 5)],
        );
        push_object(
            bytes,
            "DrawRules",
            &[(parent_id_key, 0), (draw_target_id_key, 8)],
        );
        push_object(
            bytes,
            "TextValueRun",
            &[(parent_id_key, 1), (style_id_key, 2)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/parent_dependency_hooks.riv");
    let artboard = &rust.artboards[0];

    for (source, dependent, label) in [
        (0, 1, "TransformComponent::buildDependencies for Text"),
        (1, 2, "TextStyle::buildDependencies"),
        (0, 3, "FocusData::buildDependencies"),
        (0, 4, "SemanticData::buildDependencies"),
        (0, 5, "TransformComponent::buildDependencies for Image"),
        (5, 6, "NSlicer::buildDependencies"),
    ] {
        assert!(
            artboard.dependency_edges.contains(&edge(
                source,
                dependent,
                DependencyKind::ParentChild
            )),
            "{label} should project a parent dependency"
        );
    }

    for (source, dependent, label) in [
        (6, 7, "AxisX"),
        (0, 8, "DrawTarget"),
        (0, 9, "DrawRules"),
        (1, 10, "TextValueRun"),
    ] {
        assert!(
            !artboard.dependency_edges.contains(&edge(
                source,
                dependent,
                DependencyKind::ParentChild
            )),
            "{label} does not implement a C++ parent build dependency"
        );
    }
}

#[test]
fn cpp_parent_dependency_hooks_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let component_header = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("include/rive/component.hpp"))
            .expect("read C++ component.hpp"),
    );
    assert!(
        component_header.contains("virtualvoidbuildDependencies(){}"),
        "Component::buildDependencies is no longer empty; audit generic parent dependency projection"
    );

    let transform_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/transform_component.cpp"))
            .expect("read C++ transform_component.cpp"),
    );
    let transform_body = cpp_function_body(
        &transform_source,
        "voidTransformComponent::buildDependencies()",
    );
    assert!(
        transform_body.contains("parent()->addDependent(this);"),
        "TransformComponent::buildDependencies no longer registers parent dependencies"
    );

    let text_style_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_style.cpp"))
            .expect("read C++ text_style.cpp"),
    );
    let text_style_body =
        cpp_function_body(&text_style_source, "voidTextStyle::buildDependencies()");
    assert!(
        text_style_body.contains("parent()->addDependent(this);"),
        "TextStyle::buildDependencies no longer registers parent dependencies"
    );

    for (path, signature, label) in [
        (
            "src/focus_data.cpp",
            "voidFocusData::buildDependencies()",
            "FocusData",
        ),
        (
            "src/semantic/semantic_data.cpp",
            "voidSemanticData::buildDependencies()",
            "SemanticData",
        ),
        (
            "src/layout/n_slicer.cpp",
            "voidNSlicer::buildDependencies()",
            "NSlicer",
        ),
    ] {
        let source = compact_cpp_source(
            &std::fs::read_to_string(runtime_dir.join(path)).expect("read C++ parent hook source"),
        );
        let body = cpp_function_body(&source, signature);
        assert!(
            body.contains("parent") && body.contains("addDependent(this);"),
            "{label}::buildDependencies no longer registers a parent dependency"
        );
    }

    for (path, label) in [
        ("src/draw_target.cpp", "DrawTarget"),
        ("src/draw_rules.cpp", "DrawRules"),
        ("src/layout/axis_x.cpp", "AxisX"),
        ("src/text/text_value_run.cpp", "TextValueRun"),
    ] {
        let source = compact_cpp_source(
            &std::fs::read_to_string(runtime_dir.join(path)).expect("read C++ no-parent source"),
        );
        assert!(
            !source.contains("buildDependencies("),
            "{label} added buildDependencies; audit parent dependency projection"
        );
    }
}

#[test]
fn graph_projects_shape_path_composers_from_imported_shape_paths() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7110, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "PointsPath", &[(parent_id_key, 1)]);
        push_object(bytes, "Ellipse", &[(parent_id_key, 1)]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "Node", &[(parent_id_key, 4)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/path_composers.riv");
    let artboard = &rust.artboards[0];

    assert_eq!(
        artboard
            .path_composers
            .iter()
            .map(|composer| (
                composer.shape_local,
                composer.shape_global,
                composer.path_locals.clone(),
                composer.path_globals.clone(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, 2, vec![2, 3], vec![3, 4]),
            (4, 5, Vec::new(), Vec::new())
        ],
        "every imported Shape owns one synthetic PathComposer projection, with paths coming from the C++-equivalent shape registration list"
    );

    let shape_node = dependency_node_for_component(artboard, 1);
    let points_path_node = dependency_node_for_component(artboard, 2);
    let ellipse_node = dependency_node_for_component(artboard, 3);
    let composer_node = dependency_node_for_path_composer(artboard, 1);
    let empty_composer_node = dependency_node_for_path_composer(artboard, 4);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_node,
            composer_node,
            DependencyKind::PathComposerShape
        )),
        "PathComposer::buildDependencies makes the synthetic composer depend on its owning Shape"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            points_path_node,
            composer_node,
            DependencyKind::PathComposerPath
        )),
        "PathComposer::buildDependencies makes the synthetic composer depend on registered PointsPath children"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            ellipse_node,
            composer_node,
            DependencyKind::PathComposerPath
        )),
        "PathComposer::buildDependencies makes the synthetic composer depend on registered ParametricPath children"
    );
    assert!(
        !artboard
            .dependency_node_edges
            .iter()
            .any(|edge| edge.dependent_node == empty_composer_node
                && edge.kind == DependencyKind::PathComposerPath),
        "non-path children do not become PathComposer path prerequisites"
    );
    assert_node_order_before(artboard, shape_node, composer_node);
    assert_node_order_before(artboard, points_path_node, composer_node);
    assert_node_order_before(artboard, ellipse_node, composer_node);
    assert!(
        !artboard.dependency_order.contains(&composer_node),
        "the compatibility component order remains local-object IDs, not synthetic dependency node IDs"
    );
}

#[test]
fn cpp_path_composer_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let path_composer_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/path_composer.cpp"))
            .expect("read C++ path_composer.cpp"),
    );
    let path_composer_body = cpp_function_body(
        &path_composer_source,
        "voidPathComposer::buildDependencies()",
    );
    assert!(
        path_composer_body.contains("m_shape->addDependent(this);"),
        "PathComposer::buildDependencies no longer depends on its owning Shape"
    );
    assert!(
        path_composer_body.contains("for(autopath:m_shape->paths())"),
        "PathComposer::buildDependencies no longer walks Shape::paths()"
    );
    assert!(
        path_composer_body.contains("path->addDependent(this);"),
        "PathComposer::buildDependencies no longer depends on each registered path"
    );

    let shape_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/shape.cpp"))
            .expect("read C++ shape.cpp"),
    );
    let shape_body = cpp_function_body(&shape_source, "voidShape::buildDependencies()");
    let path_composer_call = shape_body
        .find("m_PathComposer.buildDependencies();")
        .expect("Shape::buildDependencies stopped forwarding to PathComposer");
    let super_call = shape_body
        .find("Super::buildDependencies();")
        .expect("Shape::buildDependencies stopped preserving inherited dependency edges");
    assert!(
        path_composer_call < super_call,
        "Shape::buildDependencies changed the PathComposer dependency build order; audit graph projection ordering"
    );
}

#[test]
fn cpp_clipping_shape_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let clipping_shape_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/clipping_shape.cpp"))
            .expect("read C++ clipping_shape.cpp"),
    );
    let on_added_clean_body = cpp_function_body(
        &clipping_shape_source,
        "StatusCodeClippingShape::onAddedClean",
    );
    assert!(
        on_added_clean_body.contains("m_Source->forAll([this](Component*component)->bool{"),
        "ClippingShape::onAddedClean no longer walks the resolved source subtree; audit shape_locals projection"
    );
    assert!(
        on_added_clean_body.contains("component->is<Shape>()"),
        "ClippingShape::onAddedClean no longer filters source descendants to Shape"
    );
    assert!(
        on_added_clean_body.contains("shape->addFlags(PathFlags::world|PathFlags::clipping);"),
        "ClippingShape::onAddedClean no longer marks source shapes for clipping paths"
    );
    assert!(
        on_added_clean_body.contains("m_Shapes.push_back(shape);"),
        "ClippingShape::onAddedClean no longer stores source shapes for buildDependencies"
    );

    let build_dependencies_body = cpp_function_body(
        &clipping_shape_source,
        "voidClippingShape::buildDependencies()",
    );
    assert!(
        !build_dependencies_body.contains("Super::buildDependencies("),
        "ClippingShape::buildDependencies started calling Super; audit clipping parent/source dependency modeling"
    );
    assert!(
        build_dependencies_body.contains("for(autoshape:m_Shapes)"),
        "ClippingShape::buildDependencies no longer walks collected source shapes"
    );
    assert!(
        build_dependencies_body.contains("shape->pathComposer()->addDependent(this);"),
        "ClippingShape::buildDependencies no longer depends on source shape path composers"
    );
}

#[test]
fn graph_dependency_order_includes_follow_path_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let target_id_key = property_key_for_name("TargetedConstraint", "targetId");

    let bytes = synthetic_runtime_file(7111, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "PointsPath", &[(parent_id_key, 2)]);
        push_object(
            bytes,
            "FollowPathConstraint",
            &[(parent_id_key, 1), (target_id_key, 2)],
        );
        push_object(
            bytes,
            "ListFollowPathConstraint",
            &[(parent_id_key, 1), (target_id_key, 3)],
        );
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "FollowPathConstraint",
            &[(parent_id_key, 1), (target_id_key, 6)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/follow_path_dependency.riv");
    let artboard = &rust.artboards[0];

    let path_composer_node = dependency_node_for_path_composer(artboard, 2);
    let shape_constraint_node = dependency_node_for_component(artboard, 4);
    let path_constraint_node = dependency_node_for_component(artboard, 5);
    let node_target_constraint_node = dependency_node_for_component(artboard, 7);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            path_composer_node,
            shape_constraint_node,
            DependencyKind::FollowPathConstraintTargetPathComposer
        )),
        "FollowPathConstraint::buildDependencies makes shape-target constraints depend on the target Shape's PathComposer"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            path_composer_node,
            path_constraint_node,
            DependencyKind::FollowPathConstraintTargetPathComposer
        )),
        "ListFollowPathConstraint inherits the path-owned-shape PathComposer dependency"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(4, 1, DependencyKind::FollowPathConstraintParent)),
        "FollowPathConstraint::buildDependencies makes the constrained parent depend on the constraint"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(5, 1, DependencyKind::FollowPathConstraintParent)),
        "ListFollowPathConstraint::buildDependencies preserves the inherited parent dependency"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(7, 1, DependencyKind::FollowPathConstraintParent)),
        "FollowPathConstraint adds the parent dependency even when the target is not a Shape or Path"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 1, DependencyKind::TargetedConstraint)),
        "FollowPathConstraint::buildDependencies does not call TargetedConstraint::buildDependencies for shape targets"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(3, 1, DependencyKind::TargetedConstraint)),
        "FollowPathConstraint::buildDependencies does not call TargetedConstraint::buildDependencies for path targets"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(6, 1, DependencyKind::TargetedConstraint)),
        "FollowPathConstraint::buildDependencies does not call TargetedConstraint::buildDependencies for other transform targets"
    );
    assert!(
        !artboard
            .dependency_node_edges
            .iter()
            .any(|edge| edge.dependent_node == node_target_constraint_node
                && matches!(
                    edge.kind,
                    DependencyKind::FollowPathConstraintTargetPathComposer
                        | DependencyKind::FollowPathConstraintTargetPath
                )),
        "non-shape/path follow-path targets do not add target path prerequisites"
    );
    assert_node_order_before(artboard, path_composer_node, shape_constraint_node);
    assert_node_order_before(artboard, path_composer_node, path_constraint_node);
    assert_order_before(artboard, 4, 1);
    assert_order_before(artboard, 5, 1);
    assert_order_before(artboard, 7, 1);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_projects_list_constraint_registrations() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let target_id_key = property_key_for_name("TargetedConstraint", "targetId");

    let bytes = synthetic_runtime_file(7119, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "ArtboardComponentList", &[(parent_id_key, 0)]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "PointsPath", &[(parent_id_key, 2)]);
        push_object(
            bytes,
            "ListFollowPathConstraint",
            &[(parent_id_key, 1), (target_id_key, 2)],
        );
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "ListFollowPathConstraint",
            &[(parent_id_key, 5), (target_id_key, 2)],
        );
        push_object(
            bytes,
            "FollowPathConstraint",
            &[(parent_id_key, 1), (target_id_key, 2)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/list_constraint_registration.riv");
    let artboard = &rust.artboards[0];

    assert_eq!(
        artboard
            .list_constraint_registrations
            .iter()
            .map(|registration| (
                registration.constrainable_list_local,
                registration.constraint_local,
                registration.constraint_type_name
            ))
            .collect::<Vec<_>>(),
        vec![(1, 4, "ListFollowPathConstraint")],
        "ConstrainableList::addListConstraint registers exact ListFollowPathConstraint children under ArtboardComponentList"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(4, 1, DependencyKind::FollowPathConstraintParent)),
        "registered list constraints still use the inherited follow-path dependency edge"
    );
    assert!(
        !artboard
            .list_constraint_registrations
            .iter()
            .any(|registration| registration.constraint_local == 6),
        "ListFollowPathConstraint under non-ConstrainableList parents is not registered"
    );
    assert!(
        !artboard
            .list_constraint_registrations
            .iter()
            .any(|registration| registration.constraint_local == 7),
        "plain FollowPathConstraint does not implement C++ ListConstraint"
    );
}

#[test]
fn graph_projects_artboard_hosts_and_component_lists() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7138, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "NestedArtboard", &[(parent_id_key, 0)]);
        push_object(bytes, "NestedArtboardLeaf", &[(parent_id_key, 0)]);
        push_object(bytes, "NestedArtboardLayout", &[(parent_id_key, 0)]);
        push_object(bytes, "ArtboardComponentList", &[(parent_id_key, 0)]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/artboard_hosts.riv");
    let artboard = &rust.artboards[0];

    assert_eq!(
        artboard
            .nested_artboards
            .iter()
            .map(|nested| (nested.local_id, nested.type_name))
            .collect::<Vec<_>>(),
        vec![
            (1, "NestedArtboard"),
            (2, "NestedArtboardLeaf"),
            (3, "NestedArtboardLayout"),
        ],
        "Artboard::initialize registers exact NestedArtboard host variants"
    );
    assert_eq!(
        artboard
            .component_lists
            .iter()
            .map(|component_list| (component_list.local_id, component_list.type_name))
            .collect::<Vec<_>>(),
        vec![(4, "ArtboardComponentList")],
        "Artboard::initialize stores ArtboardComponentList in m_ComponentLists"
    );
    assert_eq!(
        artboard
            .artboard_hosts
            .iter()
            .map(|host| (host.local_id, host.type_name, host.kind))
            .collect::<Vec<_>>(),
        vec![
            (1, "NestedArtboard", ArtboardHostKind::NestedArtboard),
            (2, "NestedArtboardLeaf", ArtboardHostKind::NestedArtboard),
            (3, "NestedArtboardLayout", ArtboardHostKind::NestedArtboard),
            (4, "ArtboardComponentList", ArtboardHostKind::ComponentList),
        ],
        "m_ArtboardHosts preserves artboard-local object order across nested artboards and component lists"
    );
    assert!(
        !artboard
            .artboard_hosts
            .iter()
            .any(|host| host.local_id == 5),
        "ordinary components are not ArtboardHost objects"
    );
}

#[test]
fn cpp_artboard_host_registration_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let artboard_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/artboard.cpp"))
            .expect("read C++ artboard.cpp"),
    );
    let initialize_body = cpp_function_body(&artboard_source, "StatusCodeArtboard::initialize()");
    assert!(
        initialize_body.contains("caseNestedArtboardBase::typeKey:"),
        "Artboard::initialize no longer registers base NestedArtboard hosts"
    );
    assert!(
        initialize_body.contains("caseNestedArtboardLeafBase::typeKey:"),
        "Artboard::initialize no longer registers NestedArtboardLeaf hosts"
    );
    assert!(
        initialize_body.contains("caseNestedArtboardLayoutBase::typeKey:"),
        "Artboard::initialize no longer registers NestedArtboardLayout hosts"
    );
    assert!(
        initialize_body.contains("m_NestedArtboards.push_back(object->as<NestedArtboard>());"),
        "Artboard::initialize no longer stores nested artboards in m_NestedArtboards"
    );
    assert!(
        initialize_body.contains("m_ArtboardHosts.push_back(object->as<NestedArtboard>());"),
        "Artboard::initialize no longer stores nested artboards in m_ArtboardHosts"
    );
    assert!(
        initialize_body.contains("caseArtboardComponentListBase::typeKey:"),
        "Artboard::initialize no longer registers ArtboardComponentList hosts"
    );
    assert!(
        initialize_body
            .contains("m_ComponentLists.push_back(object->as<ArtboardComponentList>());"),
        "Artboard::initialize no longer stores component lists in m_ComponentLists"
    );
    assert!(
        initialize_body.contains("m_ArtboardHosts.push_back(object->as<ArtboardComponentList>());"),
        "Artboard::initialize no longer stores component lists in m_ArtboardHosts"
    );
}

#[test]
fn graph_projects_joystick_registration_and_schedule_facts() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let handle_source_id_key = property_key_for_name("Joystick", "handleSourceId");
    let x_id_key = property_key_for_name("Joystick", "xId");
    let y_id_key = property_key_for_name("Joystick", "yId");
    let object_id_key = property_key_for_name("KeyedObject", "objectId");

    let bytes = synthetic_runtime_file(7139, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "NestedArtboard", &[(parent_id_key, 0)]);
        push_object(bytes, "NestedRemapAnimation", &[(parent_id_key, 3)]);
        push_object(
            bytes,
            "Joystick",
            &[
                (parent_id_key, 1),
                (handle_source_id_key, 2),
                (x_id_key, 0),
                (y_id_key, 1),
            ],
        );
        push_object(bytes, "Joystick", &[(parent_id_key, 1)]);
        push_object(bytes, "LinearAnimation", &[]);
        push_object(bytes, "KeyedObject", &[(object_id_key, 4)]);
        push_object(bytes, "LinearAnimation", &[]);
        push_object(bytes, "KeyedObject", &[(object_id_key, 1)]);
        push_object(bytes, "KeyedObject", &[(object_id_key, 4)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/joystick_registration.riv");
    let artboard = &rust.artboards[0];

    assert!(
        !artboard.joysticks_apply_before_update,
        "one custom-handle joystick makes Artboard update joysticks after component update"
    );
    assert_eq!(
        artboard
            .joysticks
            .iter()
            .map(|joystick| (
                joystick.local_id,
                joystick.handle_source_local,
                joystick.can_apply_before_update,
                joystick.x_animation_global,
                joystick.y_animation_global,
            ))
            .collect::<Vec<_>>(),
        vec![
            (5, Some(2), false, Some(8), Some(10)),
            (6, None, true, None, None),
        ],
        "m_Joysticks preserves artboard-local order and records resolved schedule inputs"
    );
    assert_eq!(
        artboard.joysticks[0]
            .nested_remap_dependents
            .iter()
            .map(|dependent| (dependent.local_id, dependent.global_id))
            .collect::<Vec<_>>(),
        vec![(4, 5), (4, 5)],
        "Joystick::addDependents scans y animation before x animation and preserves duplicate nested remap dependents"
    );
    assert!(
        artboard.joysticks[1].nested_remap_dependents.is_empty(),
        "joysticks without resolved animations do not collect nested remap dependents"
    );
}

#[test]
fn cpp_joystick_registration_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let artboard_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/artboard.cpp"))
            .expect("read C++ artboard.cpp"),
    );
    let initialize_body = cpp_function_body(&artboard_source, "StatusCodeArtboard::initialize()");
    assert!(
        initialize_body.contains("caseJoystickBase::typeKey:"),
        "Artboard::initialize no longer registers exact Joystick objects"
    );
    assert!(
        initialize_body.contains("if(!joystick->canApplyBeforeUpdate())"),
        "Artboard::initialize no longer checks joystick schedule ordering"
    );
    assert!(
        initialize_body.contains("m_JoysticksApplyBeforeUpdate=false;"),
        "Artboard::initialize no longer records the custom-handle joystick schedule flag"
    );
    assert!(
        initialize_body.contains("joystick->addDependents(this);"),
        "Artboard::initialize no longer asks joysticks to collect dependent nested remap animations"
    );
    assert!(
        initialize_body.contains("m_Joysticks.push_back(joystick);"),
        "Artboard::initialize no longer stores joysticks in m_Joysticks"
    );

    let joystick_header = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("include/rive/joystick.hpp"))
            .expect("read C++ joystick.hpp"),
    );
    assert!(
        joystick_header.contains("boolcanApplyBeforeUpdate()const{returnm_handleSource==nullptr;}"),
        "Joystick::canApplyBeforeUpdate no longer depends only on resolved handle source state"
    );

    let joystick_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/joystick.cpp"))
            .expect("read C++ joystick.cpp"),
    );
    let on_added_clean_body =
        cpp_function_body(&joystick_source, "StatusCodeJoystick::onAddedClean");
    assert!(
        on_added_clean_body.contains("m_xAnimation=artboard()->animation(xId());"),
        "Joystick::onAddedClean no longer resolves x animation through Artboard::animation"
    );
    assert!(
        on_added_clean_body.contains("m_yAnimation=artboard()->animation(yId());"),
        "Joystick::onAddedClean no longer resolves y animation through Artboard::animation"
    );
    let add_dependents_body = cpp_function_body(
        &joystick_source,
        "voidJoystick::addDependents(Artboard*artboard)",
    );
    assert!(
        add_dependents_body
            .contains("if(m_yAnimation!=nullptr){addAnimationDependents(artboard,m_yAnimation);}"),
        "Joystick::addDependents no longer scans y animation first"
    );
    assert!(
        add_dependents_body
            .contains("if(m_xAnimation!=nullptr){addAnimationDependents(artboard,m_xAnimation);}"),
        "Joystick::addDependents no longer scans x animation after y"
    );
    let add_animation_dependents_body = cpp_function_body(
        &joystick_source,
        "voidJoystick::addAnimationDependents(Artboard*artboard,LinearAnimation*animation)",
    );
    assert!(
        add_animation_dependents_body.contains("autoobject=animation->getObject(i);"),
        "Joystick::addAnimationDependents no longer scans keyed animation objects"
    );
    assert!(
        add_animation_dependents_body
            .contains("coreObject!=nullptr&&coreObject->is<NestedRemapAnimation>()"),
        "Joystick::addAnimationDependents no longer filters keyed targets to NestedRemapAnimation"
    );
    assert!(
        add_animation_dependents_body
            .contains("m_dependents.push_back(coreObject->as<NestedRemapAnimation>());"),
        "Joystick::addAnimationDependents no longer stores nested remap dependents"
    );
}

#[test]
fn cpp_follow_path_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let follow_path_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/follow_path_constraint.cpp"))
            .expect("read C++ follow_path_constraint.cpp"),
    );
    let on_added_clean_body = cpp_function_body(
        &follow_path_source,
        "StatusCodeFollowPathConstraint::onAddedClean",
    );
    assert!(
        on_added_clean_body.contains("shape->addFlags(PathFlags::followPath);"),
        "FollowPathConstraint::onAddedClean no longer marks target shapes for follow-path updates"
    );
    assert!(
        on_added_clean_body.contains("path->addFlags(PathFlags::followPath);"),
        "FollowPathConstraint::onAddedClean no longer marks target paths for follow-path updates"
    );
    assert!(
        on_added_clean_body.contains("returnSuper::onAddedClean(context);"),
        "FollowPathConstraint::onAddedClean stopped preserving inherited clean behavior"
    );

    let build_dependencies_body = cpp_function_body(
        &follow_path_source,
        "voidFollowPathConstraint::buildDependencies()",
    );
    assert!(
        !build_dependencies_body.contains("Super::buildDependencies("),
        "FollowPathConstraint::buildDependencies started calling Super; audit targeted and parent dependency modeling"
    );
    assert!(
        build_dependencies_body.contains("shape->pathComposer()->addDependent(this);"),
        "FollowPathConstraint::buildDependencies no longer depends on target shape path composers"
    );
    assert!(
        build_dependencies_body.contains("Shape*shape=path->shape();"),
        "FollowPathConstraint::buildDependencies no longer checks path ownership"
    );
    assert!(
        build_dependencies_body.contains("path->addDependent(this);"),
        "FollowPathConstraint::buildDependencies no longer falls back to direct path dependencies"
    );
    assert!(
        build_dependencies_body.contains("addDependent(parent());"),
        "FollowPathConstraint::buildDependencies no longer makes the constrained parent depend on the constraint"
    );

    let list_follow_path_source = compact_cpp_source(
        &std::fs::read_to_string(
            runtime_dir.join("src/constraints/list_follow_path_constraint.cpp"),
        )
        .expect("read C++ list_follow_path_constraint.cpp"),
    );
    let list_follow_path_body = cpp_function_body(
        &list_follow_path_source,
        "voidListFollowPathConstraint::buildDependencies()",
    );
    assert!(
        list_follow_path_body.contains("Super::buildDependencies();"),
        "ListFollowPathConstraint::buildDependencies stopped preserving FollowPathConstraint dependencies"
    );
    assert!(
        list_follow_path_body.contains("constrainableList->addListConstraint(this);"),
        "ListFollowPathConstraint::buildDependencies no longer registers on constrainable lists"
    );

    let list_constraint_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/list_constraint.cpp"))
            .expect("read C++ list_constraint.cpp"),
    );
    let list_constraint_body = cpp_function_body(
        &list_constraint_source,
        "ListConstraint*ListConstraint::from(Component*component)",
    );
    assert!(
        list_constraint_body.contains("caseListFollowPathConstraintBase::typeKey:"),
        "ListConstraint::from no longer accepts exact ListFollowPathConstraint objects"
    );

    let constrainable_list_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/constrainable_list.cpp"))
            .expect("read C++ constrainable_list.cpp"),
    );
    let constrainable_list_body = cpp_function_body(
        &constrainable_list_source,
        "ConstrainableList*ConstrainableList::from(Component*component)",
    );
    assert!(
        constrainable_list_body.contains("caseArtboardComponentListBase::typeKey:"),
        "ConstrainableList::from no longer accepts exact ArtboardComponentList objects"
    );
    let add_list_constraint_body = cpp_function_body(
        &constrainable_list_source,
        "voidConstrainableList::addListConstraint(ListConstraint*constraint)",
    );
    assert!(
        add_list_constraint_body.contains("m_listConstraints.push_back(constraint);"),
        "ConstrainableList::addListConstraint no longer stores list constraints for update"
    );

    let component_list_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/artboard_component_list.cpp"))
            .expect("read C++ artboard_component_list.cpp"),
    );
    let update_constraints_body = cpp_function_body(
        &component_list_source,
        "voidArtboardComponentList::updateConstraints()",
    );
    assert!(
        update_constraints_body.contains("m_listConstraints.size()>0&&!virtualizationEnabled()"),
        "ArtboardComponentList::updateConstraints no longer gates list constraints on virtualization"
    );
    assert!(
        update_constraints_body.contains("listConstraint->constrainList(this);"),
        "ArtboardComponentList::updateConstraints no longer consumes registered list constraints"
    );
}

#[test]
fn graph_dependency_order_includes_text_follow_path_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let target_id_key = property_key_for_name("TextTargetModifier", "targetId");

    let bytes = synthetic_runtime_file(7112, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Text", &[(parent_id_key, 0)]);
        push_object(bytes, "TextModifierGroup", &[(parent_id_key, 1)]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "PointsPath", &[(parent_id_key, 3)]);
        push_object(
            bytes,
            "TextFollowPathModifier",
            &[(parent_id_key, 2), (target_id_key, 3)],
        );
        push_object(
            bytes,
            "TextFollowPathModifier",
            &[(parent_id_key, 2), (target_id_key, 4)],
        );
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "TextFollowPathModifier",
            &[(parent_id_key, 2), (target_id_key, 7)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/text_follow_path_dependency.riv");
    let artboard = &rust.artboards[0];

    let path_composer_node = dependency_node_for_path_composer(artboard, 3);
    let path_node = dependency_node_for_component(artboard, 4);
    let shape_modifier_node = dependency_node_for_component(artboard, 5);
    let path_modifier_node = dependency_node_for_component(artboard, 6);
    let node_target_modifier_node = dependency_node_for_component(artboard, 8);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            path_composer_node,
            shape_modifier_node,
            DependencyKind::TextFollowPathModifierTargetPathComposer
        )),
        "TextFollowPathModifier::buildDependencies makes shape-target modifiers depend on the target Shape's PathComposer"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            path_node,
            path_modifier_node,
            DependencyKind::TextFollowPathModifierTargetPath
        )),
        "TextFollowPathModifier::buildDependencies makes path-target modifiers depend directly on the target Path"
    );
    assert!(
        !artboard.dependency_node_edges.contains(&node_edge(
            path_composer_node,
            path_modifier_node,
            DependencyKind::TextFollowPathModifierTargetPathComposer
        )),
        "TextFollowPathModifier does not swap path targets to their owning Shape's PathComposer"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(5, 1, DependencyKind::TextFollowPathModifierText)),
        "TextFollowPathModifier::buildDependencies makes the Text depend on the modifier"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(6, 1, DependencyKind::TextFollowPathModifierText)),
        "path-target TextFollowPathModifier keeps the modifier-to-Text dependency"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(8, 1, DependencyKind::TextFollowPathModifierText)),
        "TextFollowPathModifier adds the Text dependency even when the target is not a Shape or Path"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 2, DependencyKind::ParentChild)),
        "TextModifierGroup does not inherit TransformComponent::buildDependencies"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 5, DependencyKind::ParentChild)),
        "TextFollowPathModifier does not inherit TransformComponent::buildDependencies"
    );
    assert!(
        !artboard
            .dependency_node_edges
            .iter()
            .any(|edge| edge.dependent_node == node_target_modifier_node
                && matches!(
                    edge.kind,
                    DependencyKind::TextFollowPathModifierTargetPathComposer
                        | DependencyKind::TextFollowPathModifierTargetPath
                )),
        "non-shape/path text follow-path targets do not add target path prerequisites"
    );
    assert_node_order_before(artboard, path_composer_node, shape_modifier_node);
    assert_node_order_before(artboard, path_node, path_modifier_node);
    assert_order_before(artboard, 5, 1);
    assert_order_before(artboard, 6, 1);
    assert_order_before(artboard, 8, 1);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_text_follow_path_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let text_follow_path_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_follow_path_modifier.cpp"))
            .expect("read C++ text_follow_path_modifier.cpp"),
    );
    let build_dependencies_body = cpp_function_body(
        &text_follow_path_source,
        "voidTextFollowPathModifier::buildDependencies()",
    );
    assert!(
        !build_dependencies_body.contains("Super::buildDependencies("),
        "TextFollowPathModifier::buildDependencies started calling Super; audit parent dependency modeling"
    );
    assert!(
        build_dependencies_body.contains("shape->pathComposer()->addDependent(this);"),
        "TextFollowPathModifier::buildDependencies no longer depends on target shape path composers"
    );
    assert!(
        build_dependencies_body.contains("path->addDependent(this);"),
        "TextFollowPathModifier::buildDependencies no longer depends directly on target paths"
    );
    assert!(
        build_dependencies_body.contains("Text*text=textComponent();"),
        "TextFollowPathModifier::buildDependencies no longer resolves the owning text component"
    );
    assert!(
        build_dependencies_body.contains("addDependent(text);"),
        "TextFollowPathModifier::buildDependencies no longer makes the Text depend on the modifier"
    );

    let on_added_clean_body = cpp_function_body(
        &text_follow_path_source,
        "StatusCodeTextFollowPathModifier::onAddedClean",
    );
    assert!(
        on_added_clean_body.contains("shape->addFlags(PathFlags::followPath);"),
        "TextFollowPathModifier::onAddedClean no longer marks target shapes for follow-path updates"
    );
    assert!(
        on_added_clean_body.contains("path->addFlags(PathFlags::followPath);"),
        "TextFollowPathModifier::onAddedClean no longer marks target paths for follow-path updates"
    );
    assert!(
        on_added_clean_body.contains("returnSuper::onAddedClean(context);"),
        "TextFollowPathModifier::onAddedClean stopped preserving inherited target resolution"
    );

    let text_target_modifier_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_target_modifier.cpp"))
            .expect("read C++ text_target_modifier.cpp"),
    );
    let text_component_body = cpp_function_body(
        &text_target_modifier_source,
        "Text*TextTargetModifier::textComponent()const",
    );
    assert!(
        text_component_body.contains("parent()->as<TextModifierGroup>()->textComponent();"),
        "TextTargetModifier::textComponent no longer resolves through TextModifierGroup"
    );

    let text_modifier_group_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_modifier_group.cpp"))
            .expect("read C++ text_modifier_group.cpp"),
    );
    let text_group_component_body = cpp_function_body(
        &text_modifier_group_source,
        "Text*TextModifierGroup::textComponent()const",
    );
    assert!(
        text_group_component_body.contains("parent()->is<Text>()"),
        "TextModifierGroup::textComponent no longer requires an owning Text parent"
    );
}

#[test]
fn graph_dependency_order_includes_text_variation_helper_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7117, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Text", &[(parent_id_key, 0)]);
        push_object(bytes, "TextStyle", &[(parent_id_key, 1)]);
        push_object(bytes, "TextStyle", &[(parent_id_key, 1)]);
        push_object(bytes, "TextStyleAxis", &[(parent_id_key, 3)]);
        push_object(bytes, "TextStylePaint", &[(parent_id_key, 1)]);
        push_object(bytes, "TextStyleFeature", &[(parent_id_key, 5)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/text_variation_helper.riv");
    let artboard = &rust.artboards[0];

    assert_eq!(
        artboard
            .text_variation_helpers
            .iter()
            .map(|helper| (
                helper.text_style_local,
                helper.text_local,
                helper.artboard_local
            ))
            .collect::<Vec<_>>(),
        vec![(3, 1, 0), (5, 1, 0)],
        "TextVariationHelper exists only for TextStyle descendants with axis/feature children"
    );

    let artboard_node = dependency_node_for_component(artboard, 0);
    let text_node = dependency_node_for_component(artboard, 1);
    let axis_helper_node = dependency_node_for_text_variation_helper(artboard, 3);
    let feature_helper_node = dependency_node_for_text_variation_helper(artboard, 5);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            artboard_node,
            axis_helper_node,
            DependencyKind::TextVariationHelperArtboard
        )),
        "TextVariationHelper::buildDependencies makes variation helpers depend on the owning artboard"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            axis_helper_node,
            text_node,
            DependencyKind::TextVariationHelperText
        )),
        "TextVariationHelper::buildDependencies makes text depend on the variation helper"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            artboard_node,
            feature_helper_node,
            DependencyKind::TextVariationHelperArtboard
        )),
        "TextStylePaint inherits TextStyle variation helper construction"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            feature_helper_node,
            text_node,
            DependencyKind::TextVariationHelperText
        )),
        "TextStylePaint variation helpers feed the owning text like TextStyle helpers"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(3, 4, DependencyKind::ParentChild)),
        "TextStyleAxis registers variation state but does not add a C++ buildDependencies parent edge"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(5, 6, DependencyKind::ParentChild)),
        "TextStyleFeature registers feature state but does not add a C++ buildDependencies parent edge"
    );
    assert_node_order_before(artboard, artboard_node, axis_helper_node);
    assert_node_order_before(artboard, axis_helper_node, text_node);
    assert_node_order_before(artboard, artboard_node, feature_helper_node);
    assert_node_order_before(artboard, feature_helper_node, text_node);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_text_variation_helper_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let text_style_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_style.cpp"))
            .expect("read C++ text_style.cpp"),
    );
    let on_added_clean_body =
        cpp_function_body(&text_style_source, "StatusCodeTextStyle::onAddedClean");
    assert!(
        on_added_clean_body.contains("!m_variations.empty()||!m_styleFeatures.empty()"),
        "TextStyle::onAddedClean no longer gates variation helpers on axis/feature children"
    );
    assert!(
        on_added_clean_body
            .contains("m_variationHelper=std::make_unique<TextVariationHelper>(this);"),
        "TextStyle::onAddedClean no longer creates TextVariationHelper"
    );

    let text_style_body =
        cpp_function_body(&text_style_source, "voidTextStyle::buildDependencies()");
    assert!(
        text_style_body.contains("m_variationHelper->buildDependencies();"),
        "TextStyle::buildDependencies no longer delegates to TextVariationHelper"
    );
    assert!(
        text_style_body.contains("parent()->addDependent(this);"),
        "TextStyle::buildDependencies no longer keeps the parent-to-style dependency"
    );
    assert!(
        text_style_body.contains("Super::buildDependencies();"),
        "TextStyle::buildDependencies stopped preserving inherited dependencies"
    );

    let helper_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_variation_helper.cpp"))
            .expect("read C++ text_variation_helper.cpp"),
    );
    let helper_body = cpp_function_body(
        &helper_source,
        "voidTextVariationHelper::buildDependencies()",
    );
    assert!(
        helper_body.contains("autotext=m_textStyle->parent();"),
        "TextVariationHelper::buildDependencies no longer resolves text through its owning style"
    );
    assert!(
        helper_body.contains("text->artboard()->addDependent(this);"),
        "TextVariationHelper::buildDependencies no longer depends on the owning artboard"
    );
    assert!(
        helper_body.contains("addDependent(text);"),
        "TextVariationHelper::buildDependencies no longer makes text depend on the helper"
    );
    let helper_update_body = cpp_function_body(&helper_source, "voidTextVariationHelper::update");
    assert!(
        helper_update_body.contains("m_textStyle->updateVariableFont();"),
        "TextVariationHelper::update no longer owns variable-font update behavior; audit graph/runtime scope"
    );

    let axis_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_style_axis.cpp"))
            .expect("read C++ text_style_axis.cpp"),
    );
    assert!(
        !axis_source.contains("TextStyleAxis::buildDependencies"),
        "TextStyleAxis added buildDependencies; audit text variation child dependency modeling"
    );
    let axis_on_added_body =
        cpp_function_body(&axis_source, "StatusCodeTextStyleAxis::onAddedDirty");
    assert!(
        axis_on_added_body.contains("style->addVariation(this);"),
        "TextStyleAxis::onAddedDirty no longer registers variations with TextStyle"
    );

    let feature_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_style_feature.cpp"))
            .expect("read C++ text_style_feature.cpp"),
    );
    assert!(
        !feature_source.contains("TextStyleFeature::buildDependencies"),
        "TextStyleFeature added buildDependencies; audit text variation child dependency modeling"
    );
    let feature_on_added_body =
        cpp_function_body(&feature_source, "StatusCodeTextStyleFeature::onAddedDirty");
    assert!(
        feature_on_added_body.contains("style->addFeature(this);"),
        "TextStyleFeature::onAddedDirty no longer registers features with TextStyle"
    );

    let text_style_paint_header = compact_cpp_source(
        &std::fs::read_to_string(
            runtime_dir.join("include/rive/generated/text/text_style_paint_base.hpp"),
        )
        .expect("read C++ text_style_paint_base.hpp"),
    );
    assert!(
        text_style_paint_header.contains("classTextStylePaintBase:publicTextStyle"),
        "TextStylePaint no longer inherits TextStyle variation helper behavior"
    );
}

#[test]
fn graph_dependency_order_includes_stroke_path_builder_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7113, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "Stroke", &[(parent_id_key, 1)]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 0)]);
        push_object(bytes, "Stroke", &[(parent_id_key, 3)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/stroke_dependency.riv");
    let artboard = &rust.artboards[0];

    let shape_path_composer_node = dependency_node_for_path_composer(artboard, 1);
    let shape_stroke_node = dependency_node_for_component(artboard, 2);
    let layout_node = dependency_node_for_component(artboard, 3);
    let layout_stroke_node = dependency_node_for_component(artboard, 4);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_path_composer_node,
            shape_stroke_node,
            DependencyKind::StrokePathBuilder
        )),
        "Stroke::buildDependencies makes strokes under Shape depend on the Shape's PathComposer"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            layout_node,
            layout_stroke_node,
            DependencyKind::StrokePathBuilder
        )),
        "Stroke::buildDependencies makes strokes under non-Shape ShapePaintContainer depend on that container's path builder"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 2, DependencyKind::ParentChild)),
        "Stroke::buildDependencies does not inherit a generic parent-child dependency under Shape"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(3, 4, DependencyKind::ParentChild)),
        "Stroke::buildDependencies does not inherit a generic parent-child dependency under non-Shape paint containers"
    );
    assert_node_order_before(artboard, shape_path_composer_node, shape_stroke_node);
    assert_node_order_before(artboard, layout_node, layout_stroke_node);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_stroke_path_builder_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let stroke_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/stroke.cpp"))
            .expect("read C++ stroke.cpp"),
    );
    let stroke_body = cpp_function_body(&stroke_source, "voidStroke::buildDependencies()");
    assert!(
        !stroke_body.contains("Super::buildDependencies("),
        "Stroke::buildDependencies started calling Super; audit paint parent dependency modeling"
    );
    assert!(
        stroke_body.contains("autocontainer=ShapePaintContainer::from(parent());"),
        "Stroke::buildDependencies no longer resolves its parent through ShapePaintContainer::from"
    );
    assert!(
        stroke_body.contains("container->pathBuilder()->addDependent(this);"),
        "Stroke::buildDependencies no longer depends on the container path builder"
    );

    let shape_paint_container_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/shape_paint_container.cpp"))
            .expect("read C++ shape_paint_container.cpp"),
    );
    let shape_paint_container_body = cpp_function_body(
        &shape_paint_container_source,
        "ShapePaintContainer*ShapePaintContainer::from(Component*component)",
    );
    for case in [
        "caseArtboard::typeKey:",
        "caseLayoutComponent::typeKey:",
        "caseShape::typeKey:",
        "caseTextStylePaint::typeKey:",
        "caseForegroundLayoutDrawable::typeKey:",
        "caseTextInputCursor::typeKey:",
        "caseTextInputSelection::typeKey:",
        "caseTextInputText::typeKey:",
        "caseTextInputSelectedText::typeKey:",
    ] {
        assert!(
            shape_paint_container_body.contains(case),
            "ShapePaintContainer::from no longer includes {case}; audit stroke path-builder resolution"
        );
    }

    let shape_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/shape.cpp"))
            .expect("read C++ shape.cpp"),
    );
    assert!(
        shape_source.contains("Component*Shape::pathBuilder(){return&m_PathComposer;}"),
        "Shape::pathBuilder no longer returns the synthetic PathComposer"
    );

    let layout_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/layout_component.cpp"))
            .expect("read C++ layout_component.cpp"),
    );
    assert!(
        layout_source.contains("Component*LayoutComponent::pathBuilder(){returnthis;}"),
        "LayoutComponent::pathBuilder no longer returns itself"
    );

    let text_style_paint_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/text/text_style_paint.cpp"))
            .expect("read C++ text_style_paint.cpp"),
    );
    assert!(
        text_style_paint_source
            .contains("Component*TextStylePaint::pathBuilder(){returnparent();}"),
        "TextStylePaint::pathBuilder no longer returns its parent Text"
    );

    let foreground_layout_drawable_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/foreground_layout_drawable.cpp"))
            .expect("read C++ foreground_layout_drawable.cpp"),
    );
    assert!(
        foreground_layout_drawable_source
            .contains("Component*ForegroundLayoutDrawable::pathBuilder(){returnparent();}"),
        "ForegroundLayoutDrawable::pathBuilder no longer returns its parent LayoutComponent"
    );

    let text_input_drawable_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("include/rive/text/text_input_drawable.hpp"))
            .expect("read C++ text_input_drawable.hpp"),
    );
    assert!(
        text_input_drawable_source.contains("Component*pathBuilder()override{returnparent();}"),
        "TextInputDrawable::pathBuilder no longer returns its parent TextInput"
    );
}

#[test]
fn graph_dependency_order_includes_fill_and_feather_path_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7114, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "Fill", &[(parent_id_key, 1)]);
        push_object(bytes, "Fill", &[(parent_id_key, 1)]);
        push_object(bytes, "DashPath", &[(parent_id_key, 3)]);
        push_object(bytes, "Feather", &[(parent_id_key, 3)]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 0)]);
        push_object(bytes, "Fill", &[(parent_id_key, 6)]);
        push_object(bytes, "DashPath", &[(parent_id_key, 7)]);
        push_object(bytes, "Feather", &[(parent_id_key, 7)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/fill_feather_dependency.riv");
    let artboard = &rust.artboards[0];

    let shape_path_composer_node = dependency_node_for_path_composer(artboard, 1);
    let no_effect_fill_node = dependency_node_for_component(artboard, 2);
    let effect_fill_node = dependency_node_for_component(artboard, 3);
    let shape_feather_node = dependency_node_for_component(artboard, 5);
    let layout_node = dependency_node_for_component(artboard, 6);
    let layout_fill_node = dependency_node_for_component(artboard, 7);
    let layout_feather_node = dependency_node_for_component(artboard, 9);

    assert!(
        !artboard
            .dependency_node_edges
            .iter()
            .any(|edge| edge.dependent_node == no_effect_fill_node
                && edge.kind == DependencyKind::FillPathBuilder),
        "Fill::buildDependencies does not add a path-builder dependency when the fill has no registered stroke effects"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 2, DependencyKind::ParentChild)),
        "Fill::buildDependencies does not inherit a generic parent-child dependency when it has no effects"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_path_composer_node,
            effect_fill_node,
            DependencyKind::FillPathBuilder
        )),
        "effect-bearing Fill::buildDependencies makes fills under Shape depend on the Shape's PathComposer"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            layout_node,
            layout_fill_node,
            DependencyKind::FillPathBuilder
        )),
        "effect-bearing Fill::buildDependencies makes fills under non-Shape containers depend on that container's path builder"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_path_composer_node,
            shape_feather_node,
            DependencyKind::FeatherPathBuilder
        )),
        "Feather::buildDependencies makes feathers under Shape paints depend on the Shape's PathComposer"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            layout_node,
            layout_feather_node,
            DependencyKind::FeatherPathBuilder
        )),
        "Feather::buildDependencies makes feathers under non-Shape paint containers depend on the container component"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 3, DependencyKind::ParentChild)),
        "effect-bearing Fill::buildDependencies does not inherit a generic parent-child dependency"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(3, 5, DependencyKind::ParentChild)),
        "DashPath does not add a buildDependencies parent edge after registering as a stroke effect"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(3, 6, DependencyKind::ParentChild)),
        "Feather::buildDependencies does not inherit a generic parent-child dependency"
    );
    assert_node_order_before(artboard, shape_path_composer_node, effect_fill_node);
    assert_node_order_before(artboard, layout_node, layout_fill_node);
    assert_node_order_before(artboard, shape_path_composer_node, shape_feather_node);
    assert_node_order_before(artboard, layout_node, layout_feather_node);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_fill_and_feather_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let fill_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/fill.cpp"))
            .expect("read C++ fill.cpp"),
    );
    let fill_body = cpp_function_body(&fill_source, "voidFill::buildDependencies()");
    assert!(
        !fill_body.contains("Super::buildDependencies("),
        "Fill::buildDependencies started calling Super; audit paint parent dependency modeling"
    );
    assert!(
        fill_body.contains("effects()->size()>0"),
        "Fill::buildDependencies no longer gates path-builder dependencies on registered effects"
    );
    assert!(
        fill_body.contains("autocontainer=ShapePaintContainer::from(parent());"),
        "Fill::buildDependencies no longer resolves its parent through ShapePaintContainer::from"
    );
    assert!(
        fill_body.contains("container->pathBuilder()->addDependent(this);"),
        "Fill::buildDependencies no longer depends on the container path builder"
    );

    let feather_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/feather.cpp"))
            .expect("read C++ feather.cpp"),
    );
    let feather_body = cpp_function_body(&feather_source, "voidFeather::buildDependencies()");
    assert!(
        !feather_body.contains("Super::buildDependencies("),
        "Feather::buildDependencies started calling Super; audit paint parent dependency modeling"
    );
    assert!(
        feather_body.contains("autoshape=parent()->as<ShapePaint>()->parent();"),
        "Feather::buildDependencies no longer resolves the owning paint container"
    );
    assert!(
        feather_body.contains("shape->as<Shape>()->pathComposer()->addDependent(this);"),
        "Feather::buildDependencies no longer depends on Shape path composers"
    );
    assert!(
        feather_body.contains("shape->addDependent(this);"),
        "Feather::buildDependencies no longer falls back to direct non-Shape container dependencies"
    );

    let dash_path_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/dash_path.cpp"))
            .expect("read C++ dash_path.cpp"),
    );
    let dash_path_body = cpp_function_body(&dash_path_source, "StatusCodeDashPath::onAddedClean");
    assert!(
        dash_path_body.contains("EffectsContainer::from(parent());"),
        "DashPath::onAddedClean no longer resolves parent effects containers"
    );
    assert!(
        dash_path_body.contains("effectsContainer->addStrokeEffect(this);"),
        "DashPath::onAddedClean no longer registers as a stroke effect"
    );
    assert!(
        !dash_path_source.contains("DashPath::buildDependencies"),
        "DashPath added buildDependencies; audit stroke-effect parent dependency modeling"
    );
}

#[test]
fn graph_dependency_order_includes_effect_parent_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7115, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "Stroke", &[(parent_id_key, 1)]);
        push_object(bytes, "GroupEffect", &[(parent_id_key, 2)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/effect_parent_dependency.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 3, DependencyKind::GroupEffectParent)),
        "GroupEffect::buildDependencies makes the effect depend on its parent effects container"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 3, DependencyKind::ParentChild)),
        "GroupEffect::buildDependencies uses its explicit parent dependency, not generic parent-child inheritance"
    );
    assert_order_before(artboard, 2, 3);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_effect_parent_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let group_effect_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/group_effect.cpp"))
            .expect("read C++ group_effect.cpp"),
    );
    let group_effect_body =
        cpp_function_body(&group_effect_source, "voidGroupEffect::buildDependencies()");
    assert!(
        group_effect_body.contains("Super::buildDependencies();"),
        "GroupEffect::buildDependencies stopped preserving inherited dependencies"
    );
    assert!(
        group_effect_body.contains("parent()->addDependent(this);"),
        "GroupEffect::buildDependencies no longer makes effects depend on their parent"
    );

    let scripted_effect_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/scripted/scripted_path_effect.cpp"))
            .expect("read C++ scripted_path_effect.cpp"),
    );
    let scripted_effect_body = cpp_function_body(
        &scripted_effect_source,
        "voidScriptedPathEffect::buildDependencies()",
    );
    assert!(
        scripted_effect_body.contains("Super::buildDependencies();"),
        "ScriptedPathEffect::buildDependencies stopped preserving inherited dependencies"
    );
    assert!(
        scripted_effect_body.contains("parent()->addDependent(this);"),
        "ScriptedPathEffect::buildDependencies no longer makes effects depend on their parent"
    );
    let scripted_on_added_clean_body = cpp_function_body(
        &scripted_effect_source,
        "StatusCodeScriptedPathEffect::onAddedClean",
    );
    assert!(
        scripted_on_added_clean_body.contains("effectsContainer->addStrokeEffect(this);"),
        "ScriptedPathEffect::onAddedClean no longer registers as a stroke effect"
    );
}

#[test]
fn graph_dependency_order_includes_linear_gradient_paint_container_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7116, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Shape", &[(parent_id_key, 0)]);
        push_object(bytes, "Fill", &[(parent_id_key, 1)]);
        push_object(bytes, "LinearGradient", &[(parent_id_key, 2)]);
        push_object(bytes, "Fill", &[(parent_id_key, 1)]);
        push_object(bytes, "RadialGradient", &[(parent_id_key, 4)]);
        push_object(bytes, "Text", &[(parent_id_key, 0)]);
        push_object(bytes, "TextStylePaint", &[(parent_id_key, 6)]);
        push_object(bytes, "Fill", &[(parent_id_key, 7)]);
        push_object(bytes, "LinearGradient", &[(parent_id_key, 8)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/linear_gradient_dependency.riv");
    let artboard = &rust.artboards[0];

    let shape_node = dependency_node_for_component(artboard, 1);
    let linear_gradient_node = dependency_node_for_component(artboard, 3);
    let radial_gradient_node = dependency_node_for_component(artboard, 5);
    let text_node = dependency_node_for_component(artboard, 6);
    let text_linear_gradient_node = dependency_node_for_component(artboard, 9);

    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_node,
            linear_gradient_node,
            DependencyKind::LinearGradientPaintContainer
        )),
        "LinearGradient::buildDependencies makes shape-owned gradients depend on the owning Node"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            shape_node,
            radial_gradient_node,
            DependencyKind::LinearGradientPaintContainer
        )),
        "RadialGradient inherits LinearGradient::buildDependencies"
    );
    assert!(
        artboard.dependency_node_edges.contains(&node_edge(
            text_node,
            text_linear_gradient_node,
            DependencyKind::LinearGradientPaintContainer
        )),
        "LinearGradient::buildDependencies walks from TextStylePaint to the first owning Node"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 3, DependencyKind::ParentChild)),
        "LinearGradient::buildDependencies does not inherit a generic parent-child dependency"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(4, 5, DependencyKind::ParentChild)),
        "RadialGradient inherits the LinearGradient no-super dependency behavior"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(8, 9, DependencyKind::ParentChild)),
        "LinearGradient under TextStylePaint uses the explicit paint-container dependency"
    );
    assert_node_order_before(artboard, shape_node, linear_gradient_node);
    assert_node_order_before(artboard, shape_node, radial_gradient_node);
    assert_node_order_before(artboard, text_node, text_linear_gradient_node);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_linear_gradient_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let linear_gradient_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/paint/linear_gradient.cpp"))
            .expect("read C++ linear_gradient.cpp"),
    );
    let linear_gradient_body = cpp_function_body(
        &linear_gradient_source,
        "voidLinearGradient::buildDependencies()",
    );
    assert!(
        !linear_gradient_body.contains("Super::buildDependencies("),
        "LinearGradient::buildDependencies started calling Super; audit paint parent dependency modeling"
    );
    assert!(
        linear_gradient_body.contains("ContainerComponent*grandParent=p->parent();"),
        "LinearGradient::buildDependencies no longer resolves the owning paint container through the parent paint"
    );
    assert!(
        linear_gradient_body.contains("ShapePaintContainer::from(grandParent)"),
        "LinearGradient::buildDependencies no longer asserts a shape-paint container parent"
    );
    assert!(
        linear_gradient_body.contains(
            "for(ContainerComponent*container=grandParent;container!=nullptr;container=container->parent())"
        ),
        "LinearGradient::buildDependencies no longer walks the container parent chain"
    );
    assert!(
        linear_gradient_body.contains("container->is<Node>()"),
        "LinearGradient::buildDependencies no longer searches for the first Node"
    );
    assert!(
        linear_gradient_body.contains("m_shapePaintContainer->addDependent(this);"),
        "LinearGradient::buildDependencies no longer depends on the first owning Node"
    );
    assert!(
        linear_gradient_body.contains("grandParent->addDependent(this);"),
        "LinearGradient::buildDependencies no longer falls back to the paint container"
    );
    assert!(
        linear_gradient_body.contains("updateDeformer();"),
        "LinearGradient::buildDependencies no longer updates deformer state; audit whether this remains out of graph scope"
    );

    let radial_gradient_header = compact_cpp_source(
        &std::fs::read_to_string(
            runtime_dir.join("include/rive/generated/shapes/paint/radial_gradient_base.hpp"),
        )
        .expect("read C++ radial_gradient_base.hpp"),
    );
    assert!(
        radial_gradient_header.contains("classRadialGradientBase:publicLinearGradient"),
        "RadialGradient no longer inherits LinearGradient::buildDependencies"
    );
}

#[test]
fn graph_dependency_order_reports_targeted_constraint_cycles() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let target_id_key = property_key_for_name("TargetedConstraint", "targetId");

    let bytes = synthetic_runtime_file(7105, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "Node", &[(parent_id_key, 1)]);
        push_object(
            bytes,
            "TranslationConstraint",
            &[(parent_id_key, 1), (target_id_key, 2)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/dependency_cycle.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 1, DependencyKind::TargetedConstraint)),
        "TargetedConstraint::buildDependencies makes the constrained component depend on the target"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 3, DependencyKind::ParentChild)),
        "TargetedConstraint::buildDependencies overrides Constraint::buildDependencies without calling Super"
    );
    assert_eq!(artboard.dependency_cycles.len(), 1);
    assert_eq!(artboard.lifecycle.dependency_cycles, 1);
    assert_eq!(artboard.dependency_cycles[0].local_ids, vec![1, 2, 1]);
}

#[test]
fn cpp_targeted_constraint_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constraint_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/constraint.cpp"))
            .expect("read C++ constraint.cpp"),
    );
    let constraint_body =
        cpp_function_body(&constraint_source, "voidConstraint::buildDependencies()");
    assert!(
        constraint_body.contains("Super::buildDependencies();"),
        "Constraint::buildDependencies stopped calling Super; audit generic constraint dependency modeling"
    );
    assert!(
        constraint_body.contains("parent()->addDependent(this);"),
        "Constraint::buildDependencies no longer adds a parent-to-constraint dependency"
    );

    let targeted_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/targeted_constraint.cpp"))
            .expect("read C++ targeted_constraint.cpp"),
    );
    let targeted_body = cpp_function_body(
        &targeted_source,
        "voidTargetedConstraint::buildDependencies()",
    );
    assert!(
        !targeted_body.contains("Super::buildDependencies("),
        "TargetedConstraint::buildDependencies started calling Super; audit targeted constraint parent-child edge modeling"
    );
    assert!(
        targeted_body.contains("m_Target->addDependent(parent());"),
        "TargetedConstraint::buildDependencies no longer makes the constrained component depend on the target"
    );
}

#[test]
fn graph_dependency_order_includes_skinning_dependencies() {
    let bytes = synthetic_runtime_file(7106, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(
            bytes,
            "RootBone",
            &[(property_key_for_name("RootBone", "parentId"), 0)],
        );
        push_object(
            bytes,
            "Bone",
            &[(property_key_for_name("Bone", "parentId"), 1)],
        );
        push_object(
            bytes,
            "Node",
            &[(property_key_for_name("Node", "parentId"), 0)],
        );
        push_object(
            bytes,
            "IKConstraint",
            &[
                (property_key_for_name("IKConstraint", "parentId"), 2),
                (property_key_for_name("IKConstraint", "targetId"), 3),
                (property_key_for_name("IKConstraint", "parentBoneCount"), 1),
            ],
        );
        push_object_with_properties(bytes, "Mesh", |bytes| {
            push_uint_property(bytes, "Mesh", "parentId", 0);
            push_bytes_property(bytes, "Mesh", "triangleIndexBytes", &[0]);
        });
        push_object(
            bytes,
            "MeshVertex",
            &[(property_key_for_name("MeshVertex", "parentId"), 5)],
        );
        push_object(
            bytes,
            "Skin",
            &[(property_key_for_name("Skin", "parentId"), 5)],
        );
        push_object(
            bytes,
            "Tendon",
            &[
                (property_key_for_name("Tendon", "parentId"), 7),
                (property_key_for_name("Tendon", "boneId"), 1),
            ],
        );
        push_object(
            bytes,
            "PointsPath",
            &[(property_key_for_name("PointsPath", "parentId"), 0)],
        );
        push_object(
            bytes,
            "Skin",
            &[(property_key_for_name("Skin", "parentId"), 9)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/skinning_dependency.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(7, 5, DependencyKind::SkinMesh)),
        "Mesh::buildDependencies makes a skinned mesh depend on its Skin"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(10, 9, DependencyKind::SkinPointsPath)),
        "PointsPath::buildDependencies makes a skinned points path depend on its Skin"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(5, 7, DependencyKind::ParentChild)),
        "Skin::buildDependencies does not call Super::buildDependencies, so the mesh parent should not also depend on its Skin child"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(9, 10, DependencyKind::ParentChild)),
        "Skin::buildDependencies does not call Super::buildDependencies for PointsPath skins either"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(10, 9, DependencyKind::SkinMesh)),
        "skinned PointsPath dependencies are classified separately from Mesh dependencies"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(7, 5, DependencyKind::SkinPointsPath)),
        "skinned Mesh dependencies are classified separately from PointsPath dependencies"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(1, 7, DependencyKind::SkinBone)),
        "Skin::buildDependencies makes each tendon bone a Skin prerequisite"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 7, DependencyKind::SkinBoneConstraintParent)),
        "Skin::buildDependencies makes IK peer constraint parents Skin prerequisites"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(3, 4, DependencyKind::IkConstraintTarget)),
        "IKConstraint::buildDependencies makes the constraint depend on its target"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(3, 2, DependencyKind::TargetedConstraint)),
        "IKConstraint::buildDependencies still inherits the targeted target-to-parent dependency"
    );
    assert_order_before(artboard, 7, 5);
    assert_order_before(artboard, 1, 7);
    assert_order_before(artboard, 2, 7);
    assert_order_before(artboard, 3, 4);
    assert_order_before(artboard, 3, 2);
    assert_order_before(artboard, 10, 9);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_dependency_order_includes_ik_tip_child_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let target_id_key = property_key_for_name("TargetedConstraint", "targetId");
    let parent_bone_count_key = property_key_for_name("IKConstraint", "parentBoneCount");

    let bytes = synthetic_runtime_file(7118, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "RootBone", &[(parent_id_key, 0)]);
        push_object(bytes, "Bone", &[(parent_id_key, 1)]);
        push_object(bytes, "Bone", &[(parent_id_key, 1)]);
        push_object(bytes, "Node", &[(parent_id_key, 1)]);
        push_object(bytes, "Node", &[(parent_id_key, 2)]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "IKConstraint",
            &[
                (parent_id_key, 2),
                (target_id_key, 6),
                (parent_bone_count_key, 1),
            ],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/ik_tip_child_dependency.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 3, DependencyKind::IkConstraintTipChild)),
        "IKConstraint::onAddedClean makes off-chain child bones depend on the constrained tip bone"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 4, DependencyKind::IkConstraintTipChild)),
        "IKConstraint::onAddedClean makes off-chain transform children depend on the constrained tip bone"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 5, DependencyKind::IkConstraintTipChild)),
        "IKConstraint::onAddedClean only rewires direct children of IK chain ancestors"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(2, 2, DependencyKind::IkConstraintTipChild)),
        "IKConstraint::onAddedClean skips the chain child when walking ancestor children"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(6, 7, DependencyKind::IkConstraintTarget)),
        "IKConstraint::buildDependencies still makes the constraint depend on its target"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(6, 2, DependencyKind::TargetedConstraint)),
        "IKConstraint::buildDependencies still makes the constrained tip depend on its target"
    );
    assert_order_before(artboard, 2, 3);
    assert_order_before(artboard, 2, 4);
    assert_order_before(artboard, 6, 2);
    assert_order_before(artboard, 6, 7);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_dependency_order_includes_joystick_handle_source_dependencies() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let handle_source_id_key = property_key_for_name("Joystick", "handleSourceId");

    let bytes = synthetic_runtime_file(7107, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(bytes, "DrawTarget", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "Joystick",
            &[(parent_id_key, 1), (handle_source_id_key, 2)],
        );
        push_object(bytes, "Joystick", &[(parent_id_key, 1)]);
        push_object(
            bytes,
            "Joystick",
            &[(parent_id_key, 1), (handle_source_id_key, 3)],
        );
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/joystick_dependency.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(1, 4, DependencyKind::JoystickParent)),
        "Joystick::buildDependencies makes custom-handle joysticks depend on their parent"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 4, DependencyKind::JoystickHandleSource)),
        "Joystick::buildDependencies makes custom-handle joysticks depend on their handle source"
    );
    assert!(
        !artboard
            .dependency_edges
            .contains(&edge(1, 4, DependencyKind::ParentChild)),
        "Joystick::buildDependencies overrides Component::buildDependencies without calling Super"
    );
    assert!(
        !artboard
            .dependency_edges
            .iter()
            .any(|edge| edge.dependent_local == 5
                && matches!(
                    edge.kind,
                    DependencyKind::ParentChild
                        | DependencyKind::JoystickParent
                        | DependencyKind::JoystickHandleSource
                )),
        "joysticks without a custom handle source do not add dependency edges"
    );
    assert!(
        !artboard
            .dependency_edges
            .iter()
            .any(|edge| edge.dependent_local == 6
                && matches!(
                    edge.kind,
                    DependencyKind::ParentChild
                        | DependencyKind::JoystickParent
                        | DependencyKind::JoystickHandleSource
                )),
        "joysticks with unresolved non-transform handle sources do not add dependency edges"
    );
    assert_order_before(artboard, 1, 4);
    assert_order_before(artboard, 2, 4);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_dependency_order_includes_scroll_bar_constraint_dependency() {
    let parent_id_key = property_key_for_name("Component", "parentId");
    let scroll_constraint_id_key =
        property_key_for_name("ScrollBarConstraint", "scrollConstraintId");

    let bytes = synthetic_runtime_file(7108, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 0)]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 1)]);
        push_object(bytes, "ScrollConstraint", &[(parent_id_key, 2)]);
        push_object(
            bytes,
            "ScrollBarConstraint",
            &[(parent_id_key, 2), (scroll_constraint_id_key, 3)],
        );
        push_object(bytes, "Node", &[(parent_id_key, 0)]);
        push_object(
            bytes,
            "ScrollBarConstraint",
            &[(parent_id_key, 2), (scroll_constraint_id_key, 5)],
        );
        push_object(bytes, "ScrollBarConstraint", &[(parent_id_key, 2)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/scroll_bar_dependency.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard
            .dependency_edges
            .contains(&edge(3, 4, DependencyKind::ScrollBarConstraint)),
        "ScrollBarConstraint::buildDependencies makes the scroll bar depend on its ScrollConstraint"
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(2, 4, DependencyKind::ParentChild)),
        "ScrollBarConstraint::buildDependencies delegates to Super, preserving the parent dependency"
    );
    assert!(
        !artboard
            .dependency_edges
            .iter()
            .any(|edge| edge.dependent_local == 6
                && matches!(
                    edge.kind,
                    DependencyKind::ParentChild | DependencyKind::ScrollBarConstraint
                )),
        "scroll bars with non-ScrollConstraint references are rejected before dependency projection"
    );
    assert!(
        !artboard
            .dependency_edges
            .iter()
            .any(|edge| edge.dependent_local == 7
                && matches!(
                    edge.kind,
                    DependencyKind::ParentChild | DependencyKind::ScrollBarConstraint
                )),
        "scroll bars without a resolved ScrollConstraint do not add dependency edges"
    );
    assert_order_before(artboard, 3, 4);
    assert_order_before(artboard, 2, 4);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn graph_dependency_order_includes_scroll_constraint_layout_children() {
    let parent_id_key = property_key_for_name("Component", "parentId");

    let bytes = synthetic_runtime_file(7109, |bytes| {
        push_object(bytes, "Backboard", &[]);
        push_object(bytes, "Artboard", &[]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 0)]);
        push_object(bytes, "ScrollConstraint", &[(parent_id_key, 1)]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 1)]);
        push_object(bytes, "NestedArtboardLayout", &[(parent_id_key, 1)]);
        push_object(bytes, "ArtboardComponentList", &[(parent_id_key, 1)]);
        push_object(bytes, "Node", &[(parent_id_key, 1)]);
        push_object(bytes, "LayoutComponent", &[(parent_id_key, 0)]);
    });

    let (_, rust) = read_graph_from_bytes(&bytes, "synthetic/scroll_constraint_children.riv");
    let artboard = &rust.artboards[0];

    assert!(
        artboard.dependency_edges.contains(&edge(
            2,
            3,
            DependencyKind::ScrollConstraintLayoutChild
        )),
        "ScrollConstraint::buildDependencies makes layout-provider content children depend on the scroll constraint"
    );
    assert!(
        artboard.dependency_edges.contains(&edge(
            2,
            4,
            DependencyKind::ScrollConstraintLayoutChild
        )),
        "NestedArtboardLayout is a C++ LayoutNodeProvider child"
    );
    assert!(
        artboard.dependency_edges.contains(&edge(
            2,
            5,
            DependencyKind::ScrollConstraintLayoutChild
        )),
        "ArtboardComponentList is a C++ LayoutNodeProvider child"
    );
    assert!(
        !artboard.dependency_edges.contains(&edge(
            2,
            6,
            DependencyKind::ScrollConstraintLayoutChild
        )),
        "non-layout-provider content children do not receive scroll constraint dependency edges"
    );
    assert!(
        !artboard.dependency_edges.contains(&edge(
            2,
            7,
            DependencyKind::ScrollConstraintLayoutChild
        )),
        "layout providers outside the scroll constraint content children are not registered"
    );
    assert_order_before(artboard, 2, 3);
    assert_order_before(artboard, 2, 4);
    assert_order_before(artboard, 2, 5);
    assert!(artboard.dependency_cycles.is_empty());
}

#[test]
fn cpp_skinning_dependency_methods_are_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let skin_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/bones/skin.cpp"))
            .expect("read C++ skin.cpp"),
    );
    let skin_body = cpp_function_body(&skin_source, "voidSkin::buildDependencies()");
    assert!(
        !skin_body.contains("Super::buildDependencies("),
        "Skin::buildDependencies started calling Super; audit the Skin parent-child edge model"
    );
    assert!(
        skin_body.contains("bone->addDependent(this);"),
        "Skin::buildDependencies no longer depends on tendon bones; audit skin dependency modeling"
    );
    assert!(
        skin_body.contains("constraint->parent()->addDependent(this);"),
        "Skin::buildDependencies no longer depends on IK peer constraint parents; audit skin dependency modeling"
    );

    let ik_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/constraints/ik_constraint.cpp"))
            .expect("read C++ ik_constraint.cpp"),
    );
    let ik_body = cpp_function_body(&ik_source, "voidIKConstraint::buildDependencies()");
    assert!(
        ik_body.contains("Super::buildDependencies();"),
        "IKConstraint::buildDependencies stopped calling Super; audit targeted IK dependency modeling"
    );
    assert!(
        ik_body.contains("m_Target->addDependent(this);"),
        "IKConstraint::buildDependencies no longer makes IK constraints depend on their targets"
    );
    let ik_on_added_clean_body =
        cpp_function_body(&ik_source, "StatusCodeIKConstraint::onAddedClean");
    assert!(
        ik_on_added_clean_body.contains("while(bone->parent()->is<Bone>()&&boneCount>0)"),
        "IKConstraint::onAddedClean no longer walks the parent bone chain"
    );
    assert!(
        ik_on_added_clean_body.contains("bone->addPeerConstraint(this);"),
        "IKConstraint::onAddedClean no longer registers peer constraints on ancestor bones"
    );
    assert!(
        ik_on_added_clean_body.contains("autotip=parent()->as<Bone>();"),
        "IKConstraint::onAddedClean no longer resolves the constrained tip bone"
    );
    assert!(
        ik_on_added_clean_body.contains("for(inti=1;i<numBones;i++)"),
        "IKConstraint::onAddedClean no longer walks FK-chain ancestors"
    );
    assert!(
        ik_on_added_clean_body.contains("Bone*ancestor=bones[i];"),
        "IKConstraint::onAddedClean no longer identifies ancestor bones for child dependency rewiring"
    );
    assert!(
        ik_on_added_clean_body.contains("Bone*chainChild=bones[i-1];"),
        "IKConstraint::onAddedClean no longer identifies the chain child to skip"
    );
    assert!(
        ik_on_added_clean_body.contains("for(Component*child:ancestor->children())"),
        "IKConstraint::onAddedClean no longer walks ancestor children"
    );
    assert!(
        ik_on_added_clean_body.contains("!child->is<TransformComponent>()||child==chainChild"),
        "IKConstraint::onAddedClean no longer filters to off-chain transform children"
    );
    assert!(
        ik_on_added_clean_body.contains("tip->addDependent(child->as<TransformComponent>());"),
        "IKConstraint::onAddedClean no longer makes off-chain transform children depend on the tip"
    );

    let mesh_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/mesh.cpp"))
            .expect("read C++ mesh.cpp"),
    );
    let mesh_body = cpp_function_body(&mesh_source, "voidMesh::buildDependencies()");
    assert!(
        mesh_body.contains("Super::buildDependencies();"),
        "Mesh::buildDependencies stopped calling Super; audit Mesh parent dependency modeling"
    );
    assert!(
        mesh_body.contains("skin()->addDependent(this);"),
        "Mesh::buildDependencies no longer makes skinned meshes depend on Skin"
    );
    assert!(
        mesh_body.contains("parent()->addDependent(this);"),
        "Mesh::buildDependencies no longer adds the explicit parent dependency"
    );

    let points_path_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/shapes/points_path.cpp"))
            .expect("read C++ points_path.cpp"),
    );
    let points_path_body =
        cpp_function_body(&points_path_source, "voidPointsPath::buildDependencies()");
    assert!(
        points_path_body.contains("Super::buildDependencies();"),
        "PointsPath::buildDependencies stopped preserving inherited path dependency edges"
    );
    assert!(
        points_path_body.contains("skin()->addDependent(this);"),
        "PointsPath::buildDependencies no longer makes skinned points paths depend on Skin"
    );

    let skinnable_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/bones/skinnable.cpp"))
            .expect("read C++ skinnable.cpp"),
    );
    let skinnable_body = cpp_function_body(
        &skinnable_source,
        "Skinnable*Skinnable::from(Component*component)",
    );
    assert!(
        skinnable_body.contains("casePointsPath::typeKey:"),
        "Skinnable::from no longer accepts exact PointsPath objects"
    );
    assert!(
        skinnable_body.contains("caseMesh::typeKey:"),
        "Skinnable::from no longer accepts exact Mesh objects"
    );
}

#[test]
fn cpp_joystick_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let joystick_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/joystick.cpp"))
            .expect("read C++ joystick.cpp"),
    );
    let joystick_body = cpp_function_body(&joystick_source, "voidJoystick::buildDependencies()");
    assert!(
        !joystick_body.contains("Super::buildDependencies("),
        "Joystick::buildDependencies started calling Super; audit joystick parent dependency modeling"
    );
    assert!(
        joystick_body.contains("m_handleSource!=nullptr&&parent()!=nullptr"),
        "Joystick::buildDependencies no longer gates dependency edges on a resolved custom handle source and parent"
    );
    assert!(
        joystick_body.contains("parent()->addDependent(this);"),
        "Joystick::buildDependencies no longer makes custom-handle joysticks depend on their parent"
    );
    assert!(
        joystick_body.contains("m_handleSource->addDependent(this);"),
        "Joystick::buildDependencies no longer makes custom-handle joysticks depend on their handle source"
    );
}

#[test]
fn cpp_scroll_bar_constraint_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let scroll_bar_source = compact_cpp_source(
        &std::fs::read_to_string(
            runtime_dir.join("src/constraints/scrolling/scroll_bar_constraint.cpp"),
        )
        .expect("read C++ scroll_bar_constraint.cpp"),
    );
    let scroll_bar_body = cpp_function_body(
        &scroll_bar_source,
        "voidScrollBarConstraint::buildDependencies()",
    );
    assert!(
        scroll_bar_body.contains("m_scrollConstraint->addDependent(this);"),
        "ScrollBarConstraint::buildDependencies no longer depends on the resolved ScrollConstraint"
    );
    assert!(
        scroll_bar_body.contains("Super::buildDependencies();"),
        "ScrollBarConstraint::buildDependencies stopped preserving its Super dependency edges"
    );
}

#[test]
fn cpp_scroll_constraint_dependency_method_is_tracked_by_graph_model() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let scroll_constraint_source = compact_cpp_source(
        &std::fs::read_to_string(
            runtime_dir.join("src/constraints/scrolling/scroll_constraint.cpp"),
        )
        .expect("read C++ scroll_constraint.cpp"),
    );
    let scroll_constraint_body = cpp_function_body(
        &scroll_constraint_source,
        "voidScrollConstraint::buildDependencies()",
    );
    assert!(
        scroll_constraint_body.contains("Super::buildDependencies();"),
        "ScrollConstraint::buildDependencies stopped preserving inherited constraint edges"
    );
    assert!(
        scroll_constraint_body.contains("for(autochild:content()->children())"),
        "ScrollConstraint::buildDependencies no longer walks content children"
    );
    assert!(
        scroll_constraint_body.contains("autolayout=LayoutNodeProvider::from(child);"),
        "ScrollConstraint::buildDependencies no longer gates child dependencies through LayoutNodeProvider::from"
    );
    assert!(
        scroll_constraint_body.contains("addDependent(child);"),
        "ScrollConstraint::buildDependencies no longer makes layout children depend on the scroll constraint"
    );
    assert!(
        scroll_constraint_body
            .contains("layout->addLayoutConstraint(static_cast<LayoutConstraint*>(this));"),
        "ScrollConstraint::buildDependencies no longer registers itself on layout children"
    );

    let layout_provider_source = compact_cpp_source(
        &std::fs::read_to_string(runtime_dir.join("src/layout/layout_node_provider.cpp"))
            .expect("read C++ layout_node_provider.cpp"),
    );
    let layout_provider_body = cpp_function_body(
        &layout_provider_source,
        "LayoutNodeProvider*LayoutNodeProvider::from(Component*component)",
    );
    assert!(
        layout_provider_body.contains("caseLayoutComponent::typeKey:"),
        "LayoutNodeProvider::from no longer accepts exact LayoutComponent objects"
    );
    assert!(
        layout_provider_body.contains("caseNestedArtboardLayout::typeKey:"),
        "LayoutNodeProvider::from no longer accepts exact NestedArtboardLayout objects"
    );
    assert!(
        layout_provider_body.contains("caseArtboardComponentListBase::typeKey:"),
        "LayoutNodeProvider::from no longer accepts ArtboardComponentList objects"
    );
}

fn edge(
    source_local: usize,
    dependent_local: usize,
    kind: DependencyKind,
) -> rive_graph::DependencyEdge {
    rive_graph::DependencyEdge {
        source_local,
        dependent_local,
        kind,
    }
}

fn node_edge(
    source_node: usize,
    dependent_node: usize,
    kind: DependencyKind,
) -> rive_graph::DependencyNodeEdge {
    rive_graph::DependencyNodeEdge {
        source_node,
        dependent_node,
        kind,
    }
}

fn dependency_node_for_component(artboard: &rive_graph::ArtboardGraph, local_id: usize) -> usize {
    artboard
        .dependency_nodes
        .iter()
        .find_map(|node| match &node.kind {
            rive_graph::DependencyNodeKind::Component {
                local_id: component_local_id,
                ..
            } if *component_local_id == local_id => Some(node.node_id),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing dependency node for component local {local_id}"))
}

fn dependency_node_for_path_composer(
    artboard: &rive_graph::ArtboardGraph,
    shape_local: usize,
) -> usize {
    artboard
        .dependency_nodes
        .iter()
        .find_map(|node| match &node.kind {
            rive_graph::DependencyNodeKind::PathComposer {
                shape_local: composer_shape_local,
                ..
            } if *composer_shape_local == shape_local => Some(node.node_id),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("missing dependency node for path composer shape local {shape_local}")
        })
}

fn dependency_node_for_text_variation_helper(
    artboard: &rive_graph::ArtboardGraph,
    text_style_local: usize,
) -> usize {
    artboard
        .dependency_nodes
        .iter()
        .find_map(|node| match &node.kind {
            rive_graph::DependencyNodeKind::TextVariationHelper {
                text_style_local: helper_text_style_local,
                ..
            } if *helper_text_style_local == text_style_local => Some(node.node_id),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("missing dependency node for text variation helper local {text_style_local}")
        })
}

fn assert_order_before(artboard: &rive_graph::ArtboardGraph, source: usize, dependent: usize) {
    let source_position = artboard
        .dependency_order
        .iter()
        .position(|local| *local == source)
        .unwrap_or_else(|| panic!("missing source local {source} in dependency order"));
    let dependent_position = artboard
        .dependency_order
        .iter()
        .position(|local| *local == dependent)
        .unwrap_or_else(|| panic!("missing dependent local {dependent} in dependency order"));
    assert!(
        source_position < dependent_position,
        "expected local {source} to be ordered before local {dependent}; order = {:?}",
        artboard.dependency_order
    );
}

fn assert_node_order_before(artboard: &rive_graph::ArtboardGraph, source: usize, dependent: usize) {
    let source_position = artboard
        .dependency_node_order
        .iter()
        .position(|node| *node == source)
        .unwrap_or_else(|| panic!("missing source node {source} in dependency node order"));
    let dependent_position = artboard
        .dependency_node_order
        .iter()
        .position(|node| *node == dependent)
        .unwrap_or_else(|| panic!("missing dependent node {dependent} in dependency node order"));
    assert!(
        source_position < dependent_position,
        "expected dependency node {source} to be ordered before node {dependent}; order = {:?}",
        artboard.dependency_node_order
    );
}

fn compact_cpp_source(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn cpp_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing C++ function signature {signature}"));
    let open = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing C++ function body for {signature}"));
    let mut depth = 0usize;
    for (offset, byte) in source[open..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[open + 1..open + offset];
                }
            }
            _ => {}
        }
    }

    panic!("unterminated C++ function body for {signature}");
}

fn compare_fixture(probe: &Path, fixture_path: &str) {
    let path = fixture(fixture_path);
    let cpp = read_cpp_probe(probe, &path, fixture_path);
    let (runtime, rust) = read_graph(&path, fixture_path);

    compare_artboards(&cpp, &runtime, &rust, fixture_path);
}

fn compare_reference_fixture(probe: &Path, fixture_path: &str) {
    let path = reference_fixture(fixture_path);
    let cpp = read_cpp_probe(probe, &path, fixture_path);
    let (_, rust) = read_graph(&path, fixture_path);

    compare_file_collections(&cpp, &rust, fixture_path);
}

fn read_cpp_probe(probe: &Path, path: &Path, label: &str) -> CppProbeFile {
    let output = Command::new(probe)
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let cpp: CppProbeFile = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {label}: {err}"));

    cpp
}

fn read_cpp_probe_bytes(probe: &Path, label: &str, bytes: &[u8]) -> CppProbeFile {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-graph-{}-{}.riv",
        std::process::id(),
        label.replace('/', "-")
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe(probe, &path, label);
    let _ = std::fs::remove_file(&path);
    cpp
}

fn read_graph(path: &Path, label: &str) -> (RuntimeFile, GraphFile) {
    let bytes = std::fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read fixture {}: {err}", path.display());
    });
    read_graph_from_bytes(&bytes, label)
}

fn read_graph_from_bytes(bytes: &[u8], label: &str) -> (RuntimeFile, GraphFile) {
    let runtime = read_runtime_file(bytes).unwrap_or_else(|err| {
        panic!("failed to import {label}: {err:#}");
    });
    let rust = GraphFile::from_runtime_file(&runtime).unwrap_or_else(|err| {
        panic!("failed to build Rust graph for {label}: {err:#}");
    });

    (runtime, rust)
}

fn compare_artboards(cpp: &CppProbeFile, runtime: &RuntimeFile, rust: &GraphFile, label: &str) {
    assert_eq!(
        cpp.artboards.len(),
        rust.artboards.len(),
        "artboard count mismatch for {label}"
    );

    for (index, (cpp_artboard, rust_artboard)) in
        cpp.artboards.iter().zip(&rust.artboards).enumerate()
    {
        assert_eq!(
            cpp_artboard.name,
            rust_artboard.name.clone().unwrap_or_default(),
            "artboard {index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_artboard.object_count,
            rust_artboard.local_objects.len(),
            "artboard {index} object count mismatch for {label}"
        );

        for (cpp_object, rust_object) in cpp_artboard
            .objects
            .iter()
            .zip(&rust_artboard.local_objects)
        {
            assert_eq!(
                cpp_object.as_ref().map(|object| object.core_type),
                type_key_for_local_object(&runtime, rust_object.global_id as usize),
                "object {} core type mismatch in artboard {index} for {label}",
                rust_object.local_id
            );
        }

        let rust_components = rust_artboard
            .components
            .iter()
            .map(|component| (component.local_id, component))
            .collect::<BTreeMap<_, _>>();

        for cpp_component in &cpp_artboard.components {
            let rust_component =
                rust_components
                    .get(&cpp_component.local_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "missing Rust component local id {} in artboard {index} for {label}",
                            cpp_component.local_id
                        )
                    });
            let rust_type_key = definition_by_name(rust_component.type_name)
                .unwrap_or_else(|| panic!("missing schema definition {}", rust_component.type_name))
                .type_key
                .int;

            assert_eq!(
                cpp_component.core_type, rust_type_key,
                "component {} core type mismatch in artboard {index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.name,
                rust_component.name.clone().unwrap_or_default(),
                "component {} name mismatch in artboard {index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.parent_id,
                rust_component.parent_local.unwrap_or(0) as u64,
                "component {} parent id mismatch in artboard {index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.parent_local, rust_component.parent_local,
                "component {} resolved parent mismatch in artboard {index} for {label}",
                cpp_component.local_id
            );
        }

        compare_artboard_import_collections(cpp_artboard, rust_artboard, index, label);
        compare_artboard_draw_graph(cpp_artboard, rust_artboard, index, label);
    }
}

fn compare_artboard_import_collections(
    cpp_artboard: &CppArtboard,
    rust_artboard: &rive_graph::ArtboardGraph,
    artboard_index: usize,
    label: &str,
) {
    assert_eq!(
        cpp_artboard.animations.len(),
        rust_artboard.animations.len(),
        "artboard {artboard_index} animation count mismatch for {label}"
    );
    for (animation_index, (cpp_animation, rust_animation)) in cpp_artboard
        .animations
        .iter()
        .zip(&rust_artboard.animations)
        .enumerate()
    {
        assert_eq!(
            cpp_animation.core_type,
            type_key_for_name("LinearAnimation"),
            "artboard {artboard_index} animation {animation_index} type mismatch for {label}"
        );
        assert_eq!(
            cpp_animation.name,
            rust_animation.name.clone().unwrap_or_default(),
            "artboard {artboard_index} animation {animation_index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_animation.fps, rust_animation.fps,
            "artboard {artboard_index} animation {animation_index} fps mismatch for {label}"
        );
        assert_eq!(
            cpp_animation.duration, rust_animation.duration,
            "artboard {artboard_index} animation {animation_index} duration mismatch for {label}"
        );
        assert_eq!(
            cpp_animation.keyed_objects.len(),
            rust_animation.keyed_objects.len(),
            "artboard {artboard_index} animation {animation_index} keyed object count mismatch for {label}"
        );
        for (keyed_object_index, (cpp_keyed_object, rust_keyed_object)) in cpp_animation
            .keyed_objects
            .iter()
            .zip(&rust_animation.keyed_objects)
            .enumerate()
        {
            assert_eq!(
                cpp_keyed_object.object_id, rust_keyed_object.object_id,
                "artboard {artboard_index} animation {animation_index} keyed object {keyed_object_index} object id mismatch for {label}"
            );
            assert_eq!(
                cpp_keyed_object.keyed_properties.len(),
                rust_keyed_object.keyed_properties.len(),
                "artboard {artboard_index} animation {animation_index} keyed object {keyed_object_index} property count mismatch for {label}"
            );
            for (property_index, (cpp_property, rust_property)) in cpp_keyed_object
                .keyed_properties
                .iter()
                .zip(&rust_keyed_object.keyed_properties)
                .enumerate()
            {
                assert_eq!(
                    cpp_property.property_key, rust_property.property_key,
                    "artboard {artboard_index} animation {animation_index} keyed object {keyed_object_index} property {property_index} key mismatch for {label}"
                );
            }
        }
    }

    assert_eq!(
        cpp_artboard.state_machines.len(),
        rust_artboard.state_machines.len(),
        "artboard {artboard_index} state machine count mismatch for {label}"
    );
    for (machine_index, (cpp_machine, rust_machine)) in cpp_artboard
        .state_machines
        .iter()
        .zip(&rust_artboard.state_machines)
        .enumerate()
    {
        assert_eq!(
            cpp_machine.core_type,
            type_key_for_name("StateMachine"),
            "artboard {artboard_index} state machine {machine_index} type mismatch for {label}"
        );
        assert_eq!(
            cpp_machine.name,
            rust_machine.name.clone().unwrap_or_default(),
            "artboard {artboard_index} state machine {machine_index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_machine.layer_count,
            rust_machine.layers.len(),
            "artboard {artboard_index} state machine {machine_index} layer count mismatch for {label}"
        );
        assert_eq!(
            cpp_machine.input_count,
            rust_machine.inputs.len(),
            "artboard {artboard_index} state machine {machine_index} input count mismatch for {label}"
        );
        assert_eq!(
            cpp_machine.listener_count,
            rust_machine.listeners.len(),
            "artboard {artboard_index} state machine {machine_index} listener count mismatch for {label}"
        );
        assert_eq!(
            cpp_machine.data_bind_count,
            rust_machine.data_binds.len(),
            "artboard {artboard_index} state machine {machine_index} data bind count mismatch for {label}"
        );
    }
}

fn compare_artboard_draw_graph(
    cpp_artboard: &CppArtboard,
    rust_artboard: &rive_graph::ArtboardGraph,
    artboard_index: usize,
    label: &str,
) {
    assert_eq!(
        cpp_artboard.draw_targets.len(),
        rust_artboard.draw_targets.len(),
        "artboard {artboard_index} draw target count mismatch for {label}"
    );
    for (target_index, (cpp_target, rust_target)) in cpp_artboard
        .draw_targets
        .iter()
        .zip(&rust_artboard.draw_targets)
        .enumerate()
    {
        assert_eq!(
            cpp_target.local_id, rust_target.local_id,
            "artboard {artboard_index} draw target {target_index} local id mismatch for {label}"
        );
        assert_eq!(
            cpp_target.drawable_id, rust_target.drawable_id,
            "artboard {artboard_index} draw target {target_index} drawable id mismatch for {label}"
        );
        assert_eq!(
            cpp_target.drawable_local, rust_target.drawable_local,
            "artboard {artboard_index} draw target {target_index} resolved drawable mismatch for {label}"
        );
        assert_eq!(
            cpp_target.placement_value, rust_target.placement_value,
            "artboard {artboard_index} draw target {target_index} placement mismatch for {label}"
        );
    }

    assert_eq!(
        cpp_artboard.draw_rules.len(),
        rust_artboard.draw_rules.len(),
        "artboard {artboard_index} draw rules count mismatch for {label}"
    );
    for (rules_index, (cpp_rules, rust_rules)) in cpp_artboard
        .draw_rules
        .iter()
        .zip(&rust_artboard.draw_rules)
        .enumerate()
    {
        assert_eq!(
            cpp_rules.local_id, rust_rules.local_id,
            "artboard {artboard_index} draw rules {rules_index} local id mismatch for {label}"
        );
        assert_eq!(
            cpp_rules.draw_target_id, rust_rules.draw_target_id,
            "artboard {artboard_index} draw rules {rules_index} draw target id mismatch for {label}"
        );
        assert_eq!(
            cpp_rules.active_target_local, rust_rules.active_target_local,
            "artboard {artboard_index} draw rules {rules_index} active target mismatch for {label}"
        );
    }

    assert_eq!(
        cpp_artboard.clipping_shapes.len(),
        rust_artboard.clipping_shapes.len(),
        "artboard {artboard_index} clipping shape count mismatch for {label}"
    );
    for (clipping_index, (cpp_clipping, rust_clipping)) in cpp_artboard
        .clipping_shapes
        .iter()
        .zip(&rust_artboard.clipping_shapes)
        .enumerate()
    {
        assert_eq!(
            cpp_clipping.local_id, rust_clipping.local_id,
            "artboard {artboard_index} clipping shape {clipping_index} local id mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.source_id, rust_clipping.source_id,
            "artboard {artboard_index} clipping shape {clipping_index} source id mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.source_local, rust_clipping.source_local,
            "artboard {artboard_index} clipping shape {clipping_index} source local mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.fill_rule, rust_clipping.fill_rule,
            "artboard {artboard_index} clipping shape {clipping_index} fill rule mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.is_visible, rust_clipping.is_visible,
            "artboard {artboard_index} clipping shape {clipping_index} visibility mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.shape_locals, rust_clipping.shape_locals,
            "artboard {artboard_index} clipping shape {clipping_index} source shape locals mismatch for {label}"
        );
        assert_eq!(
            cpp_clipping.clipped_drawable_locals, rust_clipping.clipped_drawable_locals,
            "artboard {artboard_index} clipping shape {clipping_index} clipped drawable locals mismatch for {label}"
        );
    }
}

fn compare_file_collections(cpp: &CppProbeFile, rust: &GraphFile, label: &str) {
    assert_eq!(
        cpp.assets.len(),
        rust.file_assets.len(),
        "file asset count mismatch for {label}"
    );
    for (asset_index, (cpp_asset, rust_asset)) in
        cpp.assets.iter().zip(&rust.file_assets).enumerate()
    {
        assert_eq!(
            cpp_asset.core_type,
            type_key_for_name(rust_asset.type_name),
            "file asset {asset_index} type mismatch for {label}"
        );
        assert_eq!(
            cpp_asset.name,
            rust_asset.name.clone().unwrap_or_default(),
            "file asset {asset_index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_asset.asset_id, rust_asset.asset_id,
            "file asset {asset_index} asset id mismatch for {label}"
        );
        assert_eq!(
            cpp_asset.cdn_base_url,
            rust_asset.cdn_base_url.clone().unwrap_or_default(),
            "file asset {asset_index} CDN base URL mismatch for {label}"
        );
    }

    assert_eq!(
        cpp.view_models.len(),
        rust.view_models.len(),
        "view model count mismatch for {label}"
    );
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust.view_models).enumerate()
    {
        assert_eq!(
            cpp_view_model.core_type,
            type_key_for_name(rust_view_model.type_name),
            "view model {view_model_index} type mismatch for {label}"
        );
        assert_eq!(
            cpp_view_model.name,
            rust_view_model.name.clone().unwrap_or_default(),
            "view model {view_model_index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_view_model.properties.len(),
            rust_view_model.properties.len(),
            "view model {view_model_index} property count mismatch for {label}"
        );
        assert_eq!(
            cpp_view_model.instances.len(),
            rust_view_model.instances.len(),
            "view model {view_model_index} instance count mismatch for {label}"
        );
    }

    assert_eq!(
        cpp.enums.len(),
        rust.enums.len(),
        "data enum count mismatch for {label}"
    );
    for (enum_index, (cpp_enum, rust_enum)) in cpp.enums.iter().zip(&rust.enums).enumerate() {
        assert_eq!(
            cpp_enum.core_type,
            type_key_for_name(rust_enum.type_name),
            "data enum {enum_index} type mismatch for {label}"
        );
        assert_eq!(
            cpp_enum.name,
            rust_enum.name.clone().unwrap_or_default(),
            "data enum {enum_index} name mismatch for {label}"
        );
        assert_eq!(
            cpp_enum.values.len(),
            rust_enum.values.len(),
            "data enum {enum_index} value count mismatch for {label}"
        );
        for (value_index, (cpp_value, rust_value)) in
            cpp_enum.values.iter().zip(&rust_enum.values).enumerate()
        {
            assert_eq!(
                cpp_value.key,
                rust_value.key.clone().unwrap_or_default(),
                "data enum {enum_index} value {value_index} key mismatch for {label}"
            );
            assert_eq!(
                cpp_value.value,
                rust_value.value.clone().unwrap_or_default(),
                "data enum {enum_index} value {value_index} value mismatch for {label}"
            );
        }
    }
}

fn type_key_for_name(type_name: &str) -> u16 {
    definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int
}

fn type_key_for_local_object(runtime: &RuntimeFile, global_id: usize) -> Option<u16> {
    runtime
        .objects
        .get(global_id)
        .and_then(|object| object.as_ref())
        .map(|object| object.type_key)
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    #[serde(default)]
    assets: Vec<CppAsset>,
    #[serde(default, rename = "viewModels")]
    view_models: Vec<CppViewModel>,
    #[serde(default)]
    enums: Vec<CppDataEnum>,
    artboards: Vec<CppArtboard>,
}

#[derive(Debug, Deserialize)]
struct CppAsset {
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "assetId")]
    asset_id: u64,
    #[serde(rename = "cdnBaseUrl")]
    cdn_base_url: String,
}

#[derive(Debug, Deserialize)]
struct CppViewModel {
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default)]
    properties: Vec<CppViewModelProperty>,
    #[serde(default)]
    instances: Vec<CppViewModelInstance>,
}

#[derive(Debug, Deserialize)]
struct CppViewModelProperty {}

#[derive(Debug, Deserialize)]
struct CppViewModelInstance {}

#[derive(Debug, Deserialize)]
struct CppDataEnum {
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default)]
    values: Vec<CppDataEnumValue>,
}

#[derive(Debug, Deserialize)]
struct CppDataEnumValue {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct CppArtboard {
    name: String,
    #[serde(rename = "objectCount")]
    object_count: usize,
    objects: Vec<Option<CppObject>>,
    components: Vec<CppComponent>,
    #[serde(default, rename = "drawTargets")]
    draw_targets: Vec<CppDrawTarget>,
    #[serde(default, rename = "drawRules")]
    draw_rules: Vec<CppDrawRules>,
    #[serde(default, rename = "clippingShapes")]
    clipping_shapes: Vec<CppClippingShape>,
    #[serde(default)]
    animations: Vec<CppAnimation>,
    #[serde(default, rename = "stateMachines")]
    state_machines: Vec<CppStateMachine>,
}

#[derive(Debug, Deserialize)]
struct CppObject {
    #[serde(rename = "coreType")]
    core_type: u16,
}

#[derive(Debug, Deserialize)]
struct CppComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "parentId")]
    parent_id: u64,
    #[serde(rename = "parentLocal")]
    parent_local: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CppDrawTarget {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "drawableId")]
    drawable_id: u64,
    #[serde(rename = "drawableLocal")]
    drawable_local: Option<usize>,
    #[serde(rename = "placementValue")]
    placement_value: u64,
}

#[derive(Debug, Deserialize)]
struct CppDrawRules {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "drawTargetId")]
    draw_target_id: u64,
    #[serde(rename = "activeTargetLocal")]
    active_target_local: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CppClippingShape {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "sourceId")]
    source_id: u64,
    #[serde(rename = "sourceLocal")]
    source_local: Option<usize>,
    #[serde(rename = "fillRule")]
    fill_rule: u64,
    #[serde(rename = "isVisible")]
    is_visible: bool,
    #[serde(default, rename = "shapeLocals")]
    shape_locals: Vec<usize>,
    #[serde(default, rename = "clippedDrawableLocals")]
    clipped_drawable_locals: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct CppAnimation {
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    fps: u64,
    duration: u64,
    #[serde(default, rename = "keyedObjects")]
    keyed_objects: Vec<CppKeyedObject>,
}

#[derive(Debug, Deserialize)]
struct CppKeyedObject {
    #[serde(rename = "objectId")]
    object_id: u64,
    #[serde(default, rename = "keyedProperties")]
    keyed_properties: Vec<CppKeyedProperty>,
}

#[derive(Debug, Deserialize)]
struct CppKeyedProperty {
    #[serde(rename = "propertyKey")]
    property_key: u64,
}

#[derive(Debug, Deserialize)]
struct CppStateMachine {
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "layerCount")]
    layer_count: usize,
    #[serde(rename = "inputCount")]
    input_count: usize,
    #[serde(rename = "listenerCount")]
    listener_count: usize,
    #[serde(rename = "dataBindCount")]
    data_bind_count: usize,
}
