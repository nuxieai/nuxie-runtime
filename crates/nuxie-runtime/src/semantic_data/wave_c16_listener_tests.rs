//! Exact ports of dispatch cases 1-7 from pinned `semantic_dispatch_test.cpp`.

use std::cell::Cell;
use std::rc::Rc;

use super::*;
use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;

#[derive(Debug, Default)]
struct MockSemanticListener {
    tap_count: Cell<i32>,
    increase_count: Cell<i32>,
    decrease_count: Cell<i32>,
}

impl SemanticListener for MockSemanticListener {
    fn on_semantic_tap(&self) {
        self.tap_count.set(self.tap_count.get() + 1);
    }

    fn on_semantic_increase(&self) {
        self.increase_count.set(self.increase_count.get() + 1);
    }

    fn on_semantic_decrease(&self) {
        self.decrease_count.set(self.decrease_count.get() + 1);
    }
}

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
fn wave_c16_001_fire_semantic_tap_invokes_all_registered_listeners() {
    let mut data = RuntimeSemanticData::new(0, None);
    let a = Rc::new(MockSemanticListener::default());
    let b = Rc::new(MockSemanticListener::default());
    let a_listener: Rc<dyn SemanticListener> = a.clone();
    let b_listener: Rc<dyn SemanticListener> = b.clone();
    data.add_semantic_listener(a_listener);
    data.add_semantic_listener(b_listener);

    data.fire(SemanticActionType::Tap);

    assert_eq!(a.tap_count.get(), 1);
    assert_eq!(b.tap_count.get(), 1);
    assert_eq!(a.increase_count.get(), 0);
    assert_eq!(a.decrease_count.get(), 0);
}

#[test]
fn wave_c16_002_fire_semantic_increase_only_invokes_increase() {
    let mut data = RuntimeSemanticData::new(0, None);
    let listener = Rc::new(MockSemanticListener::default());
    let erased: Rc<dyn SemanticListener> = listener.clone();
    data.add_semantic_listener(erased);

    data.fire(SemanticActionType::Increase);

    assert_eq!(listener.increase_count.get(), 1);
    assert_eq!(listener.tap_count.get(), 0);
    assert_eq!(listener.decrease_count.get(), 0);
}

#[test]
fn wave_c16_003_fire_semantic_decrease_only_invokes_decrease() {
    let mut data = RuntimeSemanticData::new(0, None);
    let listener = Rc::new(MockSemanticListener::default());
    let erased: Rc<dyn SemanticListener> = listener.clone();
    data.add_semantic_listener(erased);

    data.fire(SemanticActionType::Decrease);

    assert_eq!(listener.decrease_count.get(), 1);
    assert_eq!(listener.tap_count.get(), 0);
    assert_eq!(listener.increase_count.get(), 0);
}

#[test]
fn wave_c16_004_remove_semantic_listener_stops_future_dispatches() {
    let mut data = RuntimeSemanticData::new(0, None);
    let listener = Rc::new(MockSemanticListener::default());
    let erased: Rc<dyn SemanticListener> = listener.clone();
    data.add_semantic_listener(erased.clone());

    data.fire(SemanticActionType::Tap);
    assert_eq!(listener.tap_count.get(), 1);

    data.remove_semantic_listener(&erased);
    data.fire(SemanticActionType::Tap);
    assert_eq!(listener.tap_count.get(), 1);
}

#[test]
fn wave_c16_005_remove_unregistered_listener_is_a_no_op() {
    let mut data = RuntimeSemanticData::new(0, None);
    let registered = Rc::new(MockSemanticListener::default());
    let registered_erased: Rc<dyn SemanticListener> = registered.clone();
    let ghost: Rc<dyn SemanticListener> = Rc::new(MockSemanticListener::default());
    data.add_semantic_listener(registered_erased);

    data.remove_semantic_listener(&ghost);

    data.fire(SemanticActionType::Tap);
    assert_eq!(registered.tap_count.get(), 1);
}

#[test]
fn wave_c16_006_fire_semantic_actions_without_listeners_are_silent_no_ops() {
    let data = RuntimeSemanticData::new(0, None);
    data.fire(SemanticActionType::Tap);
    data.fire(SemanticActionType::Increase);
    data.fire(SemanticActionType::Decrease);
}

#[test]
fn wave_c16_007_semantic_node_retains_its_semantic_data_owner_identity() {
    let (mut artboard, mut data) = fixture_data();
    let node = data.semantic_node(&mut artboard);
    assert_eq!(node.borrow().semantic_data_local_id(), Some(data.local_id));
}
