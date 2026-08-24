//! Direct ports of both cases in pinned
//! `tests/unit_tests/runtime/trim_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::{ArtboardInstance, RuntimePathCommand};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn load_fixture(name: &str) -> (RuntimeFile, GraphFile) {
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

fn draw(
    artboard: &mut ArtboardInstance,
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    graphs: &GraphFile,
) {
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    artboard
        .draw_artboard(
            file,
            graph,
            &graphs.artboards,
            &mut factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("artboard draws");
}

#[test]
fn a_zero_scale_path_will_trim_with_no_crash() {
    let (file, graphs) = load_fixture("trim.riv");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let node = named_local(graph, "I");
    let scale_x = property_key("Node", "scaleX");
    let scale_y = property_key("Node", "scaleY");
    assert_ne!(artboard.double_property(node, scale_x), Some(0.0));
    assert_ne!(artboard.double_property(node, scale_y), Some(0.0));

    artboard.advance(0.0).expect("initial advance");
    draw(&mut artboard, &file, graph, &graphs);

    assert!(artboard.set_double_property(node, scale_x, 0.0));
    assert!(artboard.set_double_property(node, scale_y, 0.0));
    artboard.advance(0.0).expect("zero-scale advance");
    draw(&mut artboard, &file, graph, &graphs);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathVerb {
    Move,
    Line,
    Cubic,
    Close,
}

fn path_verbs(commands: &[RuntimePathCommand]) -> Vec<PathVerb> {
    commands
        .iter()
        .map(|command| match command {
            RuntimePathCommand::Move { .. } => PathVerb::Move,
            RuntimePathCommand::Line { .. } => PathVerb::Line,
            RuntimePathCommand::Cubic { .. } => PathVerb::Cubic,
            RuntimePathCommand::Close => PathVerb::Close,
        })
        .collect()
}

fn test_raw_path(
    artboard: &ArtboardInstance,
    graph: &ArtboardGraph,
    shape_name: &str,
    verbs: &[PathVerb],
) {
    let shape = named_local(graph, shape_name);
    let dispatch = artboard
        .draw_commands(graph)
        .into_iter()
        .find(|command| command.local_id == Some(shape))
        .unwrap_or_else(|| panic!("drawable dispatch for {shape_name}"));
    let effect_path = dispatch
        .shape_paints
        .iter()
        .find(|paint| paint.has_effect_path)
        .unwrap_or_else(|| panic!("stroke effect path for {shape_name}"));
    assert_eq!(path_verbs(&effect_path.effect_path_commands), verbs);
}

#[test]
fn different_types_of_trim_paths() {
    let (file, graphs) = load_fixture("trim_path.riv");
    let graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.name.as_deref() == Some("artboard-2"))
        .expect("artboard-2 graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("artboard-2 instantiates");
    artboard.update_pass();

    test_raw_path(
        &artboard,
        graph,
        "clipped-rect",
        &[PathVerb::Move, PathVerb::Line, PathVerb::Line],
    );
    test_raw_path(
        &artboard,
        graph,
        "clipped-rect-open",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "clipped-rect-multi",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "clipped-rect-multi-sync",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "pen-shape",
        &[PathVerb::Move, PathVerb::Cubic, PathVerb::Cubic],
    );
    test_raw_path(
        &artboard,
        graph,
        "pen-shape-close",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "mixed-shapes",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "mixed-shapes-synced",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "mixed-shapes-synced-100",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        graph,
        "mixed-shapes-100",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
        ],
    );
}
