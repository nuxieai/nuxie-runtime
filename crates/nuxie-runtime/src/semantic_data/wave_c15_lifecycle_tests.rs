//! Exact executable ports of lifecycle cases 1-6 from pinned
//! `semantic_data_lifecycle_test.cpp`.

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

fn author_properties(data: &mut RuntimeSemanticData, artboard: &mut ArtboardInstance) {
    data.set_role(SemanticRole::Button as u32, None);
    data.set_label("Submit", None);
    data.set_value("", None);
    data.set_hint("Tap to send", None);
    data.set_heading_level(0, None);
    data.set_trait_flags(SemanticTrait::ENABLABLE.0, None);
    data.set_state_flags(0, None, artboard);
}

#[test]
fn wave_c15_011_has_semantic_node_flips_after_semantic_node_is_called() {
    let (mut artboard, mut data) = fixture_data();
    assert!(!data.has_semantic_node());

    let _node = data.semantic_node(&mut artboard);
    assert!(data.has_semantic_node());
    assert!(data.node_handle().is_some());
}

#[test]
fn wave_c15_012_semantic_node_returns_the_same_instance_repeatedly() {
    let (mut artboard, mut data) = fixture_data();
    let first = data.semantic_node(&mut artboard);
    let second = data.semantic_node(&mut artboard);
    assert!(first.ptr_eq(&second));
}

#[test]
fn wave_c15_013_semantic_node_snapshots_authored_properties() {
    let (mut artboard, mut data) = fixture_data();
    author_properties(&mut data, &mut artboard);

    let node = data.semantic_node(&mut artboard);
    let node = node.borrow();

    assert_eq!(node.role(), SemanticRole::Button as u32);
    assert_eq!(node.label(), "Submit");
    assert_eq!(node.hint(), "Tap to send");
    assert!(has_semantic_trait(
        node.trait_flags(),
        SemanticTrait::ENABLABLE
    ));
    assert_eq!(node.state_flags(), 0);
}

#[test]
fn wave_c15_014_semantic_node_back_reference_points_at_its_owner() {
    let (mut artboard, mut data) = fixture_data();
    let node = data.semantic_node(&mut artboard);
    assert_eq!(node.borrow().semantic_data_local_id(), Some(data.local_id));
}

#[test]
fn wave_c15_015_setters_after_semantic_node_creation_propagate_to_the_node() {
    let (mut artboard, mut data) = fixture_data();
    author_properties(&mut data, &mut artboard);
    let node = data.semantic_node(&mut artboard);

    data.set_role(SemanticRole::Link as u32, None);
    assert_eq!(node.borrow().role(), SemanticRole::Link as u32);

    data.set_label("Learn more", None);
    assert_eq!(node.borrow().label(), "Learn more");

    data.set_value("$", None);
    assert_eq!(node.borrow().value(), "$");

    data.set_hint("External link", None);
    assert_eq!(node.borrow().hint(), "External link");

    data.set_heading_level(2, None);
    assert_eq!(node.borrow().heading_level(), 2);

    let new_traits = SemanticTrait::ENABLABLE.0 | SemanticTrait::EXPANDABLE.0;
    data.set_trait_flags(new_traits, None);
    assert_eq!(node.borrow().trait_flags(), new_traits);

    let new_states = SemanticState::SELECTED.0;
    data.set_state_flags(new_states, None, &mut artboard);
    assert_eq!(node.borrow().state_flags(), new_states);
}

#[test]
fn wave_c15_016_mutating_a_property_does_not_recreate_the_semantic_node() {
    let (mut artboard, mut data) = fixture_data();
    author_properties(&mut data, &mut artboard);
    let first = data.semantic_node(&mut artboard);

    data.set_label("Different", None);
    data.set_role(SemanticRole::Link as u32, None);

    let second = data.semantic_node(&mut artboard);
    assert!(first.ptr_eq(&second));
}
