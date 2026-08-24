//! Direct ports of all three cases in pinned
//! `tests/unit_tests/runtime/nested_artboard_opacity_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::{ArtboardInstance, RuntimeDrawableDispatch};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
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

fn fixture() -> (RuntimeFile, GraphFile, ArtboardInstance) {
    let file = read_runtime_file(&pinned_fixture("nested_artboard_opacity.riv"))
        .expect("nested_artboard_opacity.riv imports");
    let graphs =
        GraphFile::from_runtime_file(&file).expect("nested_artboard_opacity.riv graph builds");
    let graph = graphs.artboards.first().expect("Parent Artboard graph");
    assert_eq!(graph.name.as_deref(), Some("Parent Artboard"));
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("Parent Artboard instantiates");
    (file, graphs, artboard)
}

fn nested_host_local(graph: &ArtboardGraph) -> usize {
    graph
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedArtboard")
                && object.name.as_deref() == Some("Nested artboard container")
        })
        .expect("Nested artboard container")
        .local_id
}

fn nested_host_dispatch(
    artboard: &ArtboardInstance,
    graph: &ArtboardGraph,
) -> RuntimeDrawableDispatch {
    let host = nested_host_local(graph);
    artboard
        .draw_commands(graph)
        .into_iter()
        .find(|command| command.local_id == Some(host))
        .expect("Nested artboard container dispatch")
}

fn approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "expected {expected}, got {actual}"
    );
}

#[test]
fn nested_artboard_background_renders_with_opacity() {
    let (_file, graphs, mut artboard) = fixture();
    let graph = &graphs.artboards[0];
    artboard.update_pass();

    let mut nested_count = 0;
    artboard
        .try_visit_nested_artboard_instances_mut(&mut |_depth,
                                                       graph_global_id,
                                                       child|
         -> Result<(), ()> {
            let child_graph = graphs
                .artboards
                .iter()
                .find(|candidate| candidate.global_id == graph_global_id)
                .expect("nested graph");
            assert_eq!(child_graph.name.as_deref(), Some("Nested artboard"));
            child.update_pass();
            nested_count += 1;
            Ok(())
        })
        .expect("nested tree visits");
    assert_eq!(nested_count, 1);
    assert_eq!(
        nested_host_dispatch(&artboard, graph).render_opacity,
        0.3275
    );
}

#[test]
fn paused_nested_artboard_still_propagates_host_opacity() {
    let (_file, graphs, mut artboard) = fixture();
    let graph = &graphs.artboards[0];
    let host = nested_host_local(graph);
    artboard.advance(0.0).expect("initial tree advance");
    let baseline = nested_host_dispatch(&artboard, graph).render_opacity;
    assert!(baseline > 0.0);

    assert!(artboard.set_bool_property(host, property_key("NestedArtboard", "isPaused"), true));
    assert!(artboard.set_double_property(
        host,
        property_key("NestedArtboard", "opacity"),
        baseline * 0.5,
    ));
    artboard.advance(0.0).expect("paused tree advance");

    approx_eq(
        nested_host_dispatch(&artboard, graph).render_opacity,
        baseline * 0.5,
    );
}

#[test]
fn nested_artboard_own_opacity_combines_with_host_opacity() {
    let (file, graphs, mut artboard) = fixture();
    let graph = &graphs.artboards[0];
    let host = nested_host_local(graph);
    let artboard_opacity = property_key("Artboard", "opacity");
    let nested_host_opacity = property_key("NestedArtboard", "opacity");

    let mut nested_count = 0;
    artboard
        .try_visit_nested_artboard_instances_mut(&mut |_depth,
                                                       _graph_global_id,
                                                       child|
         -> Result<(), ()> {
            assert!(child.set_double_property(0, artboard_opacity, 0.4));
            nested_count += 1;
            Ok(())
        })
        .expect("nested tree visits");
    assert_eq!(nested_count, 1);
    assert!(artboard.set_double_property(host, nested_host_opacity, 0.5));
    artboard.advance(0.0).expect("tree advances");

    artboard
        .try_visit_nested_artboard_instances_mut(&mut |_depth,
                                                       _graph_global_id,
                                                       child|
         -> Result<(), ()> {
            assert_eq!(child.double_property(0, artboard_opacity), Some(0.4));
            Ok(())
        })
        .expect("nested tree visits");

    let mut factory = RecordingFactory::new();
    artboard
        .initialize_artboard_renderer(
            &file,
            graph,
            &graphs.artboards,
            &BTreeMap::new(),
            &mut factory,
            None,
        )
        .expect("renderer initializes");
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
        .expect("nested tree draws");
    assert!(
        factory.stream().contains("color=0x33ff0000"),
        "the nested red background receives own 0.4 × host 0.5 opacity"
    );
}
