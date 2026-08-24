//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_path_effect_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
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

fn assert_matches_upstream_sriv(_actual: &str, expected: &str) {
    panic!("Rust has no active scripted .sriv comparator for {expected}")
}

#[test]
#[ignore = "expected-red: the scripted reuse_path_in_effect .sriv row is not executable yet"]
fn reusing_a_path_in_multiple_passes_works_correctly() {
    let file = read_runtime_file(&pinned_fixture("reuse_path_in_effect.riv"))
        .expect("reuse_path_in_effect.riv imports");
    let graphs =
        GraphFile::from_runtime_file(&file).expect("reuse_path_in_effect.riv graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");

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

    let mut state_machine = artboard
        .state_machine_instance(0)
        .expect("state machine zero instantiates");
    assert!(state_machine.bind_default_view_model_context_on_artboard(&mut artboard));

    let mut renderer = factory.make_renderer();
    state_machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("state machine advances and applies");
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
        .expect("artboard draws");

    assert_matches_upstream_sriv(&factory.stream(), "reuse_path_in_effect.sriv");
}
