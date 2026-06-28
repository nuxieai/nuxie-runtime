use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::{DependencyKind, GraphFile};
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
        artboard
            .dependency_edges
            .contains(&edge(0, 2, DependencyKind::ParentChild))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 3, DependencyKind::ParentChild))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 4, DependencyKind::ParentChild))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 5, DependencyKind::ParentChild))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 6, DependencyKind::ParentChild))
    );
    assert!(
        artboard
            .dependency_edges
            .contains(&edge(0, 7, DependencyKind::ParentChild))
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
    assert_order_before(artboard, 1, 2);
    assert_order_before(artboard, 2, 4);
    assert_order_before(artboard, 1, 6);
    assert!(artboard.dependency_cycles.is_empty());
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
