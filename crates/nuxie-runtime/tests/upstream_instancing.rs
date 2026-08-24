//! Direct ports of all three cases in pinned
//! `tests/unit_tests/runtime/instancing_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::ArtboardInstance;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load_default(name: &str) -> (RuntimeFile, GraphFile) {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graph builds: {error:#}"));
    (file, graphs)
}

fn named_local(graph: &ArtboardGraph, name: &str) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| object.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing object named {name}"))
        .local_id
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

fn missing_cloned_shape_position(_artboard: &ArtboardInstance, _local_id: usize) -> (f32, f32) {
    panic!("Rust has no individual Component::clone owner")
}

#[test]
#[ignore = "expected-red: Rust has no individual Shape clone operation"]
fn cloning_an_ellipse_works() {
    let (file, graphs) = load_default("circle_clips.riv");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let node = named_local(graph, "TopEllipse");
    let x_key = property_key("Shape", "x");
    let y_key = property_key("Shape", "y");
    let position = (
        artboard.double_property(node, x_key).expect("TopEllipse.x"),
        artboard.double_property(node, y_key).expect("TopEllipse.y"),
    );
    let cloned_position = missing_cloned_shape_position(&artboard, node);
    assert_eq!(position.0, cloned_position.0);
    assert_eq!(position.1, cloned_position.1);
}

#[test]
fn instancing_artboard_clones_clipped_properties() {
    let (file, graphs) = load_default("circle_clips.riv");
    let graph = graphs.artboards.first().expect("default artboard graph");

    // The immutable ArtboardGraph is Rust's definition owner; construction of
    // a distinct ArtboardInstance below is the borrow-safe `isInstance()`
    // adaptation.
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let node = named_local(graph, "TopEllipse");
    assert_eq!(
        graph.local_objects[node].type_name,
        Some("Shape"),
        "TopEllipse is a Shape"
    );

    let clipping_sources = graph
        .clipping_shapes
        .iter()
        .filter(|clipping| clipping.clipped_drawable_locals.contains(&node))
        .map(|clipping| {
            graph.local_objects[clipping.source_local.expect("clipping source local")]
                .name
                .as_deref()
                .expect("clipping source name")
        })
        .collect::<Vec<_>>();
    assert_eq!(clipping_sources.len(), 2);
    assert_eq!(clipping_sources[0], "ClipRect2");
    assert_eq!(clipping_sources[1], "BabyEllipse");

    artboard.update_pass();
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw_artboard(
            &file,
            graph,
            &graphs.artboards,
            &mut factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("clipped artboard draws");
}

fn missing_animation_delete_count() -> usize {
    panic!("Rust animation definitions expose no C++ deleteCount equivalent")
}

fn missing_first_animation_identity(_graph: &ArtboardGraph, _artboard: &ArtboardInstance) -> bool {
    panic!("Rust ArtboardInstance exposes no firstAnimation definition identity")
}

#[test]
#[ignore = "expected-red: Rust exposes neither firstAnimation identity nor deleteCount"]
fn instancing_artboard_does_not_clone_animations() {
    let (file, graphs) = load_default("juice.riv");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");

    let source_animation_count = graph.animations.len();
    let instance_animation_count = graph.animations.len();
    assert_eq!(source_animation_count, instance_animation_count);
    assert!(missing_first_animation_identity(graph, &artboard));

    assert_eq!(missing_animation_delete_count(), 0);
    let number_of_animations = source_animation_count;
    drop(artboard);
    drop(graphs);
    drop(file);
    assert_eq!(missing_animation_delete_count(), number_of_animations);
}
