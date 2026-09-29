//! All three cases from upstream 293eaf00 semantic_diff_fields_test.cpp.
use nuxie_runtime::source::semantic::{
    semantic_dirt::SemanticDirt,
    semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager},
    semantic_node::{SemanticNode, SemanticNodeRef},
    semantic_role::SemanticRole,
    semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
};

fn make_node(id: u32, role: SemanticRole, label: &str) -> SemanticNodeRef {
    let node = SemanticNode::new(id);
    node.borrow_mut().role = role as u32;
    node.borrow_mut().label = label.into();
    node
}
fn find_updated(diff: &SemanticsDiff, id: u32) -> Option<&SemanticsDiffNode> {
    diff.updated_semantic.iter().find(|node| node.id == id)
}
fn find_added(diff: &SemanticsDiff, id: u32) -> Option<&SemanticsDiffNode> {
    diff.added.iter().find(|node| node.id == id)
}
fn drain(manager: &RuntimeSemanticManagerHandle) -> SemanticsDiff {
    manager.with_semantic_manager_mut(|manager| manager.drain_diff())
}

#[test]
fn full_reflatten_emits_value_hint_heading_level_changes() {
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::default());
    let slider = make_node(1, SemanticRole::Slider, "Volume");
    slider.borrow_mut().value = "50".into();
    manager.add_child(None, slider.clone());
    let baseline = drain(&manager);
    let added = find_added(&baseline, slider.borrow().id()).expect("slider added");
    assert_eq!(added.value, "50");
    slider.borrow_mut().value = "75".into();
    slider.borrow_mut().hint = "Drag to adjust".into();
    slider.borrow_mut().heading_level = 2;
    manager.with_semantic_manager_mut(|manager| {
        manager.mark_node_dirty(slider.borrow().id(), SemanticDirt::CONTENT)
    });
    let text = make_node(2, SemanticRole::Text, "Other");
    manager.add_child(None, text.clone());
    let diff = drain(&manager);
    assert!(find_added(&diff, text.borrow().id()).is_some());
    let updated = find_updated(&diff, slider.borrow().id()).expect("slider updated");
    assert_eq!(updated.value, "75");
    assert_eq!(updated.hint, "Drag to adjust");
    assert_eq!(updated.heading_level, 2);
    let after = drain(&manager);
    assert!(find_updated(&after, slider.borrow().id()).is_none());
}

#[test]
fn explicit_label_set_at_runtime_beats_stale_derived_label() {
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::default());
    let button = make_node(1, SemanticRole::Button, "");
    let text = make_node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(button.clone()), text.clone());
    let baseline = drain(&manager);
    let btn = find_added(&baseline, button.borrow().id()).expect("button added");
    assert_eq!(btn.label, "Play");
    assert!(find_added(&baseline, text.borrow().id()).is_none());
    button.borrow_mut().label = "Pause".into();
    manager.with_semantic_manager_mut(|manager| {
        manager.mark_node_dirty(button.borrow().id(), SemanticDirt::CONTENT)
    });
    let diff = drain(&manager);
    let updated = find_updated(&diff, button.borrow().id()).expect("button updated");
    assert_eq!(updated.label, "Pause");
    assert!(find_added(&diff, text.borrow().id()).is_some());
}

#[test]
fn clearing_an_explicit_label_rederives_from_children() {
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::default());
    let button = make_node(1, SemanticRole::Button, "Authored");
    let text = make_node(2, SemanticRole::Text, "Play");
    manager.add_child(None, button.clone());
    manager.add_child(Some(button.clone()), text.clone());
    let baseline = drain(&manager);
    let btn = find_added(&baseline, button.borrow().id()).expect("button added");
    assert_eq!(btn.label, "Authored");
    assert!(find_added(&baseline, text.borrow().id()).is_some());
    button.borrow_mut().label.clear();
    manager.with_semantic_manager_mut(|manager| {
        manager.mark_node_dirty(button.borrow().id(), SemanticDirt::CONTENT)
    });
    let diff = drain(&manager);
    let updated = find_updated(&diff, button.borrow().id()).expect("button updated");
    assert_eq!(updated.label, "Play");
    assert!(diff.removed.contains(&text.borrow().id()));
}
