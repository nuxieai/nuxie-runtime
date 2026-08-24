//! One-for-one ports of the ten active cases in pinned
//! `tests/unit_tests/runtime/library_asset_test.cpp`.

use nuxie::{File, FileAssetKind};
use nuxie_binary::RuntimeObject;
use nuxie_graph::ArtboardGraph;
use nuxie_runtime::ArtboardInstance as RuntimeArtboardInstance;
use nuxie_schema::definition_by_name;
use std::path::PathBuf;

fn fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load(name: &str) -> File {
    File::import(&fixture(name)).unwrap_or_else(|error| panic!("import {name}: {error:#}"))
}

fn object_named<'a>(
    file: &'a File,
    graph: &ArtboardGraph,
    kind: &str,
    name: &str,
) -> &'a RuntimeObject {
    let global_id = graph
        .local_objects
        .iter()
        .find(|object| object.type_name == Some(kind) && object.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {kind} named {name}"))
        .global_id;
    file.runtime().objects[global_id as usize]
        .as_ref()
        .unwrap_or_else(|| panic!("missing imported object {global_id}"))
}

fn local_named(graph: &ArtboardGraph, kind: &str, name: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.type_name == Some(kind) && object.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {kind} named {name}"))
        .local_id
}

fn local_of_type(graph: &ArtboardGraph, kind: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.type_name == Some(kind))
        .unwrap_or_else(|| panic!("missing {kind}"))
        .local_id
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return property.key.int;
    }
    for ancestor_name in definition.ancestors {
        let ancestor = definition_by_name(ancestor_name)
            .unwrap_or_else(|| panic!("missing ancestor definition {ancestor_name}"));
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return property.key.int;
        }
    }
    panic!("missing property {type_name}.{property_name}")
}

fn graph_for_global(file: &File, global_id: u32) -> &ArtboardGraph {
    file.graph()
        .artboards
        .iter()
        .find(|graph| graph.global_id == global_id)
        .unwrap_or_else(|| panic!("missing artboard graph {global_id}"))
}

fn string_property(
    instance: &RuntimeArtboardInstance,
    local_id: usize,
    kind: &str,
    name: &str,
) -> String {
    let bytes = instance
        .debug_string_property(local_id, property_key(kind, name))
        .unwrap_or_else(|| panic!("missing live {kind}.{name}"));
    String::from_utf8(bytes.to_vec()).expect("live string is UTF-8")
}

#[test]
fn file_with_library_artboard_loads() {
    let file = load("library_export_test.riv");
    let host = file.artboard(0).expect("host artboard");
    let nested = object_named(&file, host.graph(), "NestedArtboard", "The nested artboard");
    assert_eq!(nested.string_property("name"), Some("The nested artboard"));
    assert_eq!(nested.double_property("x"), Some(1.0));
    assert_eq!(nested.double_property("y"), Some(2.0));
    assert_eq!(nested.uint_property("artboardId"), Some(1));

    let artboard = file.artboard(1).expect("library artboard");
    assert_eq!(artboard.name(), Some("Rocket"));
    assert_eq!(artboard.dimensions(), Some((512.0, 513.0)));
    assert_eq!(file.asset_count(), 0);
}

#[test]
fn file_with_library_animation_loads() {
    let file = load("library_export_animation_test.riv");
    let host = file.artboard(0).expect("host artboard");
    let nested = object_named(&file, host.graph(), "NestedArtboard", "The nested artboard");
    assert_eq!(nested.string_property("name"), Some("The nested artboard"));
    assert_eq!(nested.double_property("x"), Some(1.0));
    assert_eq!(nested.double_property("y"), Some(2.0));
    assert_eq!(nested.uint_property("artboardId"), Some(1));

    let library = file.artboard(1).expect("library artboard");
    assert_eq!(library.animation_count(), 1);
    assert_eq!(library.animation_name(0), Some("LA Rocket"));
    let nested_animations = host
        .graph()
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("NestedSimpleAnimation"))
        .collect::<Vec<_>>();
    assert_eq!(nested_animations.len(), 1);
    let nested_animation = file.runtime().objects[nested_animations[0].global_id as usize]
        .as_ref()
        .expect("nested animation object");
    assert_eq!(nested_animation.string_property("name"), Some(""));
    assert_eq!(nested_animation.uint_property("animationId"), Some(0));
    assert_eq!(file.asset_count(), 0);
}

#[test]
fn file_with_library_state_machine_loads() {
    let file = load("library_export_state_machine_test.riv");
    let host = file.artboard(0).expect("host artboard");
    let nested = object_named(&file, host.graph(), "NestedArtboard", "The nested artboard");
    assert_eq!(nested.string_property("name"), Some("The nested artboard"));
    assert_eq!(nested.double_property("x"), Some(1.0));
    assert_eq!(nested.double_property("y"), Some(2.0));
    assert_eq!(nested.uint_property("artboardId"), Some(1));

    let library = file.artboard(1).expect("library artboard");
    assert_eq!(library.state_machine_count(), 1);
    assert_eq!(library.state_machine_name(0), Some("SM Rocket"));
    let nested_animations = host
        .graph()
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("NestedStateMachine"))
        .collect::<Vec<_>>();
    assert_eq!(nested_animations.len(), 1);
    let nested_animation = file.runtime().objects[nested_animations[0].global_id as usize]
        .as_ref()
        .expect("nested state machine object");
    assert_eq!(nested_animation.string_property("name"), Some(""));
    assert_eq!(nested_animation.uint_property("animationId"), Some(0));
    assert_eq!(file.asset_count(), 0);
}

#[test]
#[ignore = "expected-red: ScriptAsset.moduleName is not retained by the Rust import"]
fn library_script_exports_flat_under_its_mangle_prefix() {
    let file = load("library_scope_edge_test.riv");
    let script = file
        .assets()
        .find(|asset| asset.kind() == FileAssetKind::Script)
        .expect("script asset");
    assert_eq!(
        script.descriptor().string_property("moduleName"),
        Some("FruitsLib@4/FruitModule")
    );
}

#[test]
#[ignore = "expected-red: ScriptAsset.moduleName is not retained by the Rust import"]
fn nested_library_scripts_export_flat_under_distinct_prefixes() {
    let file = load("nested_library_scope_test.riv");
    let useb = file
        .assets()
        .find(|asset| asset.kind() == FileAssetKind::Script && asset.name() == Some("useb"))
        .expect("useb script");
    let mesh = file
        .assets()
        .find(|asset| asset.kind() == FileAssetKind::Script && asset.name() == Some("mesh"))
        .expect("mesh script");
    assert_eq!(
        useb.descriptor().string_property("moduleName"),
        Some("OuterLib@6/useb")
    );
    assert_eq!(
        mesh.descriptor().string_property("moduleName"),
        Some("InnerLib@4/mesh")
    );
}

#[test]
fn file_with_library_including_image() {
    let file = load("library_with_image.riv");
    assert_eq!(file.asset_count(), 1);
    let host = file.artboard(0).expect("host artboard");
    let nested = object_named(&file, host.graph(), "NestedArtboard", "The instance");
    let source = file
        .artboard(
            nested
                .uint_property("artboardId")
                .expect("source artboard id") as usize,
        )
        .expect("source artboard");
    let images = source
        .graph()
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("Image"))
        .collect::<Vec<_>>();
    assert_eq!(images.len(), 1);
    let image = file.runtime().objects[images[0].global_id as usize]
        .as_ref()
        .expect("image object");
    assert_eq!(image.uint_property("assetId"), Some(0));
    assert!(
        file.asset(image.uint_property("assetId").expect("asset id") as usize)
            .is_some()
    );
}

#[test]
fn file_with_multiple_libraries_including_image() {
    let file = load("double_library_with_image.riv");
    assert_eq!(file.asset_count(), 2);
    let host = file.artboard(0).expect("host artboard");
    for (nested_name, asset_name) in [
        ("The nested artboard", "MyFirstImageAsset"),
        ("Another nested artboard", "MyOtherImageAsset"),
    ] {
        let nested = object_named(&file, host.graph(), "NestedArtboard", nested_name);
        let source = file
            .artboard(
                nested
                    .uint_property("artboardId")
                    .expect("source artboard id") as usize,
            )
            .expect("source artboard");
        let images = source
            .graph()
            .local_objects
            .iter()
            .filter(|object| object.type_name == Some("Image"))
            .collect::<Vec<_>>();
        assert_eq!(images.len(), 1);
        let image = file.runtime().objects[images[0].global_id as usize]
            .as_ref()
            .expect("image object");
        let asset = file
            .asset(image.uint_property("assetId").expect("asset id") as usize)
            .expect("image asset");
        assert_eq!(asset.name(), Some(asset_name));
        assert!(asset.resource().is_some());
    }
}

#[test]
fn file_with_data_enum() {
    let file = load("library_data_enum_test.riv");
    let artboard = file.artboard(0).expect("artboard");
    let mut instance = artboard.instantiate().expect("artboard instance");
    let view_model = instance
        .instantiate_default_view_model_instance()
        .expect("default view model instance");
    assert!(instance.bind_view_model(&view_model));
    assert!(artboard.graph().local_objects.iter().any(|object| {
        object.type_name == Some("Event") && object.name.as_deref() == Some("my_event")
    }));
    instance.advance(0.0);
    let property = local_named(
        artboard.graph(),
        "CustomPropertyString",
        "my_event_property",
    );
    assert_eq!(
        string_property(
            instance.raw(),
            property,
            "CustomPropertyString",
            "propertyValue"
        ),
        "red3"
    );
}

#[test]
fn file_with_view_model() {
    let file = load("library_view_model_test.riv");
    let artboard = file.artboard(0).expect("artboard");
    let mut instance = artboard.instantiate().expect("artboard instance");
    let view_model = instance
        .instantiate_default_view_model_instance()
        .expect("default view model instance");
    assert!(instance.bind_view_model(&view_model));
    instance.advance(0.0);

    let mut saw_two = false;
    let mut saw_one = false;
    instance
        .raw_mut()
        .try_visit_nested_artboard_instances_mut(&mut |depth, global_id, child| {
            let graph = graph_for_global(&file, global_id);
            if depth == 1 {
                assert_eq!(graph.name.as_deref(), Some("2"));
                saw_two = true;
            } else if depth == 2 {
                assert_eq!(graph.name.as_deref(), Some("1"));
                saw_one = true;
                let for_string = local_named(graph, "CustomPropertyString", "for_string");
                assert_eq!(
                    string_property(child, for_string, "CustomPropertyString", "propertyValue"),
                    "hello"
                );
                let for_enum = local_named(graph, "CustomPropertyString", "for_enum");
                assert_eq!(
                    string_property(child, for_enum, "CustomPropertyString", "propertyValue"),
                    "uk"
                );
                let rectangle = local_of_type(graph, "Rectangle");
                assert_eq!(
                    child.double_property(rectangle, property_key("Rectangle", "width")),
                    Some(123.0)
                );
                assert_eq!(
                    child.double_property(rectangle, property_key("Rectangle", "height")),
                    Some(123.0)
                );
                let solid = local_of_type(graph, "SolidColor");
                assert_eq!(
                    child.color_property(solid, property_key("SolidColor", "colorValue")),
                    Some(0xff0a0f42)
                );
            }
            Ok::<_, ()>(())
        })
        .expect("visit nested artboards");
    assert!(saw_two);
    assert!(saw_one);
}

#[test]
fn library_vmtest_1_host() {
    let file = load("library_vmtest_1_host.riv");
    let artboard = file.artboard(0).expect("artboard");
    let mut instance = artboard.instantiate().expect("artboard instance");
    let view_model = instance
        .instantiate_default_view_model_instance()
        .expect("default view model instance");
    assert!(instance.bind_view_model(&view_model));
    instance.advance(0.0);

    let mut occurrences = 0;
    instance
        .raw_mut()
        .try_visit_nested_artboard_instances_mut(&mut |depth, global_id, child| {
            if depth != 1 {
                return Ok::<_, ()>(());
            }
            occurrences += 1;
            let graph = graph_for_global(&file, global_id);
            assert_eq!(graph.name.as_deref(), Some("lib2artboard"));
            assert_eq!(
                graph
                    .shape_paint_containers
                    .iter()
                    .map(|container| container.paints.len())
                    .sum::<usize>(),
                1
            );
            let solid = local_of_type(graph, "SolidColor");
            assert_eq!(
                child.color_property(solid, property_key("SolidColor", "colorValue")),
                Some(0xff101566)
            );
            Ok(())
        })
        .expect("visit nested artboards");
    assert_eq!(occurrences, 1);
}
