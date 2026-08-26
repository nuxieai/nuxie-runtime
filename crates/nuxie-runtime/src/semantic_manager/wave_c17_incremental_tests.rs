//! Exact ports of cases 28-36 from pinned `semantic_label_inference_test.cpp`.

use super::*;
use crate::SemanticRole;

fn node(id: u32, role: SemanticRole, label: &str) -> SemanticNodeHandle {
    let node = SemanticNodeHandle::new(id);
    {
        let mut owner = node.borrow_mut();
        owner.set_role(role as u32);
        owner.set_label(label);
    }
    node
}

fn set_bounds(node: &SemanticNodeHandle, bounds: SemanticBounds) {
    node.borrow_mut().set_bounds(bounds);
}

fn drain(manager: &mut SemanticManager) -> SemanticsDiff {
    manager
        .drain_diff()
        .expect("pinned in-memory tree has no unresolved boundary dirt")
}

fn added(diff: &SemanticsDiff, id: u32) -> Option<&SemanticsDiffNode> {
    diff.added.iter().find(|node| node.id == id)
}

fn initial_incremental_tree(label: &str) -> (SemanticManager, SemanticNodeHandle) {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    let button = node(2, SemanticRole::Button, label);
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    set_bounds(&button, SemanticBounds::new(0.0, 0.0, 100.0, 40.0));
    manager.add_child(None, root.clone());
    manager.add_child(Some(&root), button.clone());
    let _ = drain(&mut manager);
    (manager, button)
}

#[test]
fn wave_c17_028_pure_content_dirt_emits_only_updated_semantic() {
    let (mut manager, button) = initial_incremental_tree("Submit");

    button.borrow_mut().set_label("Send");
    manager.mark_node_dirty(2, SemanticDirt::CONTENT);
    let diff = drain(&mut manager);

    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert!(diff.moved.is_empty());
    assert!(diff.children_updated.is_empty());
    assert!(diff.updated_geometry.is_empty());
    assert_eq!(diff.updated_semantic.len(), 1);
    assert_eq!(diff.updated_semantic[0].id, 2);
    assert_eq!(diff.updated_semantic[0].label, "Send");
}

#[test]
fn wave_c17_029_pure_bounds_dirt_emits_only_updated_geometry() {
    let (mut manager, button) = initial_incremental_tree("Submit");

    set_bounds(&button, SemanticBounds::new(5.0, 5.0, 105.0, 45.0));
    manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
    let diff = drain(&mut manager);

    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert!(diff.moved.is_empty());
    assert!(diff.children_updated.is_empty());
    assert!(diff.updated_semantic.is_empty());
    assert_eq!(diff.updated_geometry.len(), 1);
    assert_eq!(diff.updated_geometry[0].id, 2);
    assert_eq!(diff.updated_geometry[0].min_x, 5.0);
    assert_eq!(diff.updated_geometry[0].min_y, 5.0);
    assert_eq!(diff.updated_geometry[0].max_x, 105.0);
    assert_eq!(diff.updated_geometry[0].max_y, 45.0);
}

#[test]
fn wave_c17_030_mixed_content_and_bounds_dirt_emits_into_both_arrays() {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    let button = node(2, SemanticRole::Button, "A");
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    set_bounds(&button, SemanticBounds::new(0.0, 0.0, 50.0, 50.0));
    manager.add_child(None, root.clone());
    manager.add_child(Some(&root), button.clone());
    let _ = drain(&mut manager);

    button.borrow_mut().set_label("B");
    set_bounds(&button, SemanticBounds::new(0.0, 0.0, 60.0, 60.0));
    manager.mark_node_dirty(2, SemanticDirt::CONTENT | SemanticDirt::BOUNDS);
    let diff = drain(&mut manager);

    assert!(diff.added.is_empty());
    assert!(diff.removed.is_empty());
    assert!(diff.moved.is_empty());
    assert!(diff.children_updated.is_empty());
    assert_eq!(diff.updated_semantic.len(), 1);
    assert_eq!(diff.updated_semantic[0].id, 2);
    assert_eq!(diff.updated_semantic[0].label, "B");
    assert_eq!(diff.updated_geometry.len(), 1);
    assert_eq!(diff.updated_geometry[0].id, 2);
    assert_eq!(diff.updated_geometry[0].max_x, 60.0);
}

#[test]
fn wave_c17_031_no_op_content_dirty_mark_produces_no_updated_semantic_entry() {
    let (mut manager, _button) = initial_incremental_tree("Submit");
    manager.mark_node_dirty(2, SemanticDirt::CONTENT);
    let diff = drain(&mut manager);
    assert!(diff.updated_semantic.is_empty());
    assert!(diff.is_empty());
}

#[test]
fn wave_c17_032_no_op_bounds_dirty_mark_produces_no_updated_geometry_entry() {
    let (mut manager, _button) = initial_incremental_tree("Submit");
    manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
    let diff = drain(&mut manager);
    assert!(diff.updated_geometry.is_empty());
    assert!(diff.is_empty());
}

#[test]
fn wave_c17_033_dirty_mark_on_an_unknown_id_is_silently_ignored() {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    manager.add_child(None, root);
    let _ = drain(&mut manager);

    manager.mark_node_dirty(9999, SemanticDirt::CONTENT | SemanticDirt::BOUNDS);
    let diff = drain(&mut manager);

    assert!(diff.updated_semantic.is_empty());
    assert!(diff.updated_geometry.is_empty());
    assert!(diff.is_empty());
}

#[test]
fn wave_c17_034_drain_diff_after_no_changes_returns_an_empty_diff() {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    manager.add_child(None, root);
    let _ = drain(&mut manager);
    let diff = drain(&mut manager);
    assert!(diff.is_empty());
}

#[test]
fn wave_c17_035_absorbed_child_content_change_escalates_to_rederivation() {
    let mut manager = SemanticManager::new();
    let button = node(1, SemanticRole::Button, "");
    let text = node(2, SemanticRole::Text, "Play");
    set_bounds(&button, SemanticBounds::new(0.0, 0.0, 100.0, 40.0));
    set_bounds(&text, SemanticBounds::new(0.0, 0.0, 100.0, 40.0));
    manager.add_child(None, button.clone());
    manager.add_child(Some(&button), text.clone());

    let first = drain(&mut manager);
    let first_button = added(&first, 1).expect("button in first diff");
    assert_eq!(first_button.label, "Play");
    assert!(added(&first, 2).is_none());

    text.borrow_mut().set_label("Pause");
    manager.mark_node_dirty(2, SemanticDirt::CONTENT);
    let diff = drain(&mut manager);

    let mut saw_button = false;
    for node in &diff.updated_semantic {
        if node.id == 1 {
            assert_eq!(node.label, "Pause");
            saw_button = true;
        }
    }
    assert!(saw_button);
    assert!(added(&diff, 2).is_none());
    for node in &diff.updated_semantic {
        assert_ne!(node.id, 2);
    }
}

#[test]
fn wave_c17_036_removed_array_is_emitted_in_previous_tree_preorder() {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    let a = node(2, SemanticRole::Text, "A");
    let b = node(3, SemanticRole::Text, "B");
    let c = node(4, SemanticRole::Text, "C");
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 300.0));
    set_bounds(&a, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    set_bounds(&b, SemanticBounds::new(0.0, 100.0, 100.0, 200.0));
    set_bounds(&c, SemanticBounds::new(0.0, 200.0, 100.0, 300.0));
    manager.add_child(None, root.clone());
    manager.add_child(Some(&root), a.clone());
    manager.add_child(Some(&root), b);
    manager.add_child(Some(&root), c.clone());
    let _ = drain(&mut manager);

    manager.remove_child(&c);
    manager.remove_child(&a);
    let diff = drain(&mut manager);

    assert_eq!(diff.removed.len(), 2);
    assert_eq!(diff.removed[0], 2);
    assert_eq!(diff.removed[1], 4);
}
