//! Exact ports of dispatch cases 8-11 and 14-16 from pinned
//! `semantic_dispatch_test.cpp`.

use std::cell::Cell;
use std::rc::Rc;

use super::*;
use crate::SemanticRole;
use crate::semantic_data::{RuntimeSemanticData, SemanticActionType, SemanticListener};
use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;

#[derive(Debug, Default)]
struct MockSemanticListener {
    tap_count: Cell<i32>,
}

impl SemanticListener for MockSemanticListener {
    fn on_semantic_tap(&self) {
        self.tap_count.set(self.tap_count.get() + 1);
    }

    fn on_semantic_increase(&self) {}

    fn on_semantic_decrease(&self) {}
}

fn fixture_data() -> (crate::ArtboardInstance, RuntimeSemanticData) {
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
    let artboard =
        crate::ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
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
fn wave_c16_008_manager_lookup_routes_dispatch_to_the_semantic_data_owner() {
    let mut manager = SemanticManager::new();
    let (mut artboard, mut data) = fixture_data();
    data.set_role(SemanticRole::Button as u32, None);
    data.set_label("Go", None);

    let node = data.semantic_node(&mut artboard);
    manager.add_child(None, node.clone());
    let id = node.borrow().id();
    assert_ne!(id, 0);

    let listener = Rc::new(MockSemanticListener::default());
    let erased: Rc<dyn SemanticListener> = listener.clone();
    data.add_semantic_listener(erased.clone());
    let owners = [data];

    let found = manager.node_by_id(id).expect("registered semantic node");
    let owner_local_id = found
        .borrow()
        .semantic_data_local_id()
        .expect("SemanticData-created node owner identity");
    let owner = owners
        .iter()
        .find(|owner| owner.local_id == owner_local_id)
        .expect("retained SemanticData owner");
    assert!(std::ptr::eq(owner, &owners[0]));

    owner.fire(SemanticActionType::Tap);
    assert_eq!(listener.tap_count.get(), 1);

    let mut owner = owners.into_iter().next().expect("single owner");
    owner.remove_semantic_listener(&erased);
}

#[test]
fn wave_c16_009_manager_lookup_returns_none_for_an_unknown_id() {
    let mut manager = SemanticManager::new();
    let (mut artboard, mut data) = fixture_data();
    data.set_role(SemanticRole::Button as u32, None);
    let node = data.semantic_node(&mut artboard);
    manager.add_child(None, node);
    assert!(manager.node_by_id(9999).is_none());
}

#[test]
fn wave_c16_010_boundary_node_has_no_owning_semantic_data() {
    let mut manager = SemanticManager::new();
    let boundary = SemanticNodeHandle::new(0);
    boundary.borrow_mut().set_boundary_node(true);
    manager.add_child(None, boundary.clone());

    let found = manager
        .node_by_id(boundary.borrow().id())
        .expect("registered boundary node");
    assert!(found.borrow().is_boundary_node());
    assert!(found.borrow().semantic_data_local_id().is_none());
}

#[test]
fn wave_c16_011_removing_semantic_data_drops_its_id_from_the_index() {
    let mut manager = SemanticManager::new();
    let (mut artboard, mut data) = fixture_data();
    data.set_role(SemanticRole::Button as u32, None);
    let node = data.semantic_node(&mut artboard);
    manager.add_child(None, node.clone());
    let id = node.borrow().id();

    assert!(
        manager
            .node_by_id(id)
            .is_some_and(|found| found.ptr_eq(&node))
    );

    manager.remove_child(&node);
    assert!(manager.node_by_id(id).is_none());
}

#[test]
fn wave_c16_014_request_focus_with_an_unknown_id_returns_false() {
    let manager = SemanticManager::new();
    assert!(!manager.request_focus(42, |_| true));
}

#[test]
fn wave_c16_015_request_focus_without_a_core_owner_returns_false() {
    let mut manager = SemanticManager::new();
    let node = SemanticNodeHandle::new(0);
    manager.add_child(None, node.clone());
    assert!(node.borrow().core_owner_local_id().is_none());
    assert!(!manager.request_focus(node.borrow().id(), |_| true));
}

#[test]
fn wave_c16_016_request_focus_on_a_boundary_node_returns_false() {
    let mut manager = SemanticManager::new();
    let boundary = SemanticNodeHandle::new(0);
    boundary.borrow_mut().set_boundary_node(true);
    manager.add_child(None, boundary.clone());
    assert!(!manager.request_focus(boundary.borrow().id(), |_| true));
}
