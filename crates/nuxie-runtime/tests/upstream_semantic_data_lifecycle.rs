//! Exact executable port of lifecycle case 9 from pinned
//! `semantic_data_lifecycle_test.cpp`.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, SemanticActionType, SemanticState, has_semantic_state};

const DROPDOWN_LABEL: &str = "Select a fandom";

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets/semantic")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn wave_c15_019_state_machine_property_change_appears_in_updated_semantic() {
    let file = read_runtime_file(&pinned_fixture("data_binding_lists.riv"))
        .expect("data_binding_lists imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("data_binding_lists graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut state_machine = artboard
        .state_machine_instance(0)
        .expect("state machine zero");
    state_machine.enable_semantics();
    let _ = state_machine.bind_default_view_model_context_on_artboard(&mut artboard);
    for _ in 0..10 {
        state_machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture settles");
    }

    let initial = state_machine
        .drain_semantics_diff(&mut artboard)
        .expect("initial semantic diff");
    let initial_button = initial
        .added
        .iter()
        .find(|node| node.label == DROPDOWN_LABEL)
        .expect("initial dropdown button");
    assert!(has_semantic_state(
        initial_button.state_flags,
        SemanticState::EXPANDED
    ));
    let button_id = initial_button.id;

    state_machine.fire_semantic_action(button_id, SemanticActionType::Tap as u32);
    for _ in 0..10 {
        state_machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture settles after tap");
    }
    let follow = state_machine
        .drain_semantics_diff(&mut artboard)
        .expect("follow-up semantic diff");

    let updated = follow
        .updated_semantic
        .iter()
        .find(|node| node.id == button_id)
        .expect("dropdown semantic update");
    assert!(!has_semantic_state(
        updated.state_flags,
        SemanticState::EXPANDED
    ));
}
