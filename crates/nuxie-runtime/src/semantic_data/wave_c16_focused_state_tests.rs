//! Exact ports of dispatch cases 12-13 from pinned `semantic_dispatch_test.cpp`.

use super::*;
use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;

fn fixture_data() -> (ArtboardInstance, RuntimeSemanticData) {
    let file = read_runtime_file(include_bytes!(
        "../../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv"
    ))
    .expect("semantic fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("semantic fixture graph builds");
    let graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.name.as_deref() == Some("Element"))
        .expect("Element artboard graph");
    let artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("Element artboard instantiates");
    let local_id = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "SemanticData")
        .expect("Element SemanticData")
        .local_id;
    let data = RuntimeSemanticData::from_artboard(&artboard, local_id);
    (artboard, data)
}

#[test]
fn wave_c16_012_set_focused_state_toggles_focused_and_preserves_siblings() {
    let (mut artboard, mut data) = fixture_data();
    let node = data.semantic_node(&mut artboard);

    let initial = SemanticState::SELECTED.0 | SemanticState::EXPANDED.0;
    node.borrow_mut().set_state_flags(initial);

    data.set_focused_state(true, None);
    assert!(has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::FOCUSED
    ));
    assert!(has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::SELECTED
    ));
    assert!(has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::EXPANDED
    ));

    data.set_focused_state(false, None);
    assert!(!has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::FOCUSED
    ));
    assert!(has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::SELECTED
    ));
    assert!(has_semantic_state(
        node.borrow().state_flags(),
        SemanticState::EXPANDED
    ));
}

#[test]
fn wave_c16_013_set_focused_state_before_node_creation_is_a_no_op() {
    let mut data = RuntimeSemanticData::new(0, None);
    assert!(!data.has_semantic_node());

    data.set_focused_state(true, None);
    data.set_focused_state(false, None);

    assert!(!data.has_semantic_node());
}
