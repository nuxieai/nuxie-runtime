//! Exact ports of cases 16-23 from pinned `semantic_label_inference_test.cpp`.

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

fn children(diff: &SemanticsDiff, parent_id: i32) -> Option<&SemanticsChildrenUpdate> {
    diff.children_updated
        .iter()
        .find(|update| update.parent_id == parent_id)
}

#[test]
fn wave_c17_016_children_sort_by_bounds_position() {
    let mut manager = SemanticManager::new();
    let parent = node(1, SemanticRole::Group, "container");
    set_bounds(&parent, SemanticBounds::new(0.0, 0.0, 50.0, 250.0));
    let child_a = node(2, SemanticRole::Button, "A");
    set_bounds(&child_a, SemanticBounds::new(0.0, 200.0, 50.0, 250.0));
    let child_b = node(3, SemanticRole::Button, "B");
    set_bounds(&child_b, SemanticBounds::new(0.0, 100.0, 50.0, 150.0));
    let child_c = node(4, SemanticRole::Button, "C");
    set_bounds(&child_c, SemanticBounds::new(0.0, 0.0, 50.0, 50.0));
    manager.add_child(None, parent.clone());
    manager.add_child(Some(&parent), child_a);
    manager.add_child(Some(&parent), child_b);
    manager.add_child(Some(&parent), child_c);
    let diff = drain(&mut manager);
    let update = children(&diff, 1).expect("parent children update");
    assert_eq!(update.child_ids.len(), 3);
    assert_eq!(update.child_ids[0], 4);
    assert_eq!(update.child_ids[1], 3);
    assert_eq!(update.child_ids[2], 2);
}

#[test]
fn wave_c17_017_x_position_breaks_same_y_ties() {
    let mut manager = SemanticManager::new();
    let parent = node(1, SemanticRole::Group, "row");
    set_bounds(&parent, SemanticBounds::new(0.0, 0.0, 300.0, 50.0));
    let child_a = node(2, SemanticRole::Button, "A");
    set_bounds(&child_a, SemanticBounds::new(200.0, 0.0, 300.0, 50.0));
    let child_b = node(3, SemanticRole::Button, "B");
    set_bounds(&child_b, SemanticBounds::new(0.0, 0.0, 100.0, 50.0));
    manager.add_child(None, parent.clone());
    manager.add_child(Some(&parent), child_a);
    manager.add_child(Some(&parent), child_b);
    let diff = drain(&mut manager);
    let update = children(&diff, 1).expect("parent children update");
    assert_eq!(update.child_ids.len(), 2);
    assert_eq!(update.child_ids[0], 3);
    assert_eq!(update.child_ids[1], 2);
}

#[test]
fn wave_c17_018_bounds_swap_triggers_reorder_in_diff() {
    let mut manager = SemanticManager::new();
    let parent = node(1, SemanticRole::Group, "container");
    set_bounds(&parent, SemanticBounds::new(0.0, 0.0, 50.0, 300.0));
    let child_a = node(2, SemanticRole::Button, "A");
    set_bounds(&child_a, SemanticBounds::new(0.0, 0.0, 50.0, 50.0));
    let child_b = node(3, SemanticRole::Button, "B");
    set_bounds(&child_b, SemanticBounds::new(0.0, 100.0, 50.0, 150.0));
    manager.add_child(None, parent.clone());
    manager.add_child(Some(&parent), child_a.clone());
    manager.add_child(Some(&parent), child_b);
    let first = drain(&mut manager);
    let first_update = children(&first, 1).expect("initial children update");
    assert_eq!(first_update.child_ids.len(), 2);
    assert_eq!(first_update.child_ids[0], 2);
    assert_eq!(first_update.child_ids[1], 3);
    set_bounds(&child_a, SemanticBounds::new(0.0, 200.0, 50.0, 250.0));
    manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
    let second = drain(&mut manager);
    let second_update = children(&second, 1).expect("reordered children update");
    assert_eq!(second_update.child_ids.len(), 2);
    assert_eq!(second_update.child_ids[0], 3);
    assert_eq!(second_update.child_ids[1], 2);
}

#[test]
fn wave_c17_019_bounds_change_without_order_change_stays_incremental() {
    let mut manager = SemanticManager::new();
    let parent = node(1, SemanticRole::Group, "container");
    set_bounds(&parent, SemanticBounds::new(0.0, 0.0, 50.0, 300.0));
    let child_a = node(2, SemanticRole::Button, "A");
    set_bounds(&child_a, SemanticBounds::new(0.0, 0.0, 50.0, 50.0));
    let child_b = node(3, SemanticRole::Button, "B");
    set_bounds(&child_b, SemanticBounds::new(0.0, 100.0, 50.0, 150.0));
    manager.add_child(None, parent.clone());
    manager.add_child(Some(&parent), child_a.clone());
    manager.add_child(Some(&parent), child_b);
    let _first = drain(&mut manager);
    set_bounds(&child_a, SemanticBounds::new(0.0, 10.0, 50.0, 60.0));
    manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
    let second = drain(&mut manager);
    assert!(second.children_updated.is_empty());
    let mut has_geometry = false;
    for owner in &second.updated_geometry {
        if owner.id == 2 {
            has_geometry = true;
        }
    }
    assert!(has_geometry);
}

#[test]
fn wave_c17_020_empty_bounds_preserve_insertion_order() {
    let mut manager = SemanticManager::new();
    let a = node(1, SemanticRole::Button, "A");
    let b = node(2, SemanticRole::Button, "B");
    let c = node(3, SemanticRole::Button, "C");
    manager.add_child(None, a);
    manager.add_child(None, b);
    manager.add_child(None, c);
    let diff = drain(&mut manager);
    let update = children(&diff, -1).expect("root children update");
    assert_eq!(update.child_ids.len(), 3);
    assert_eq!(update.child_ids[0], 1);
    assert_eq!(update.child_ids[1], 2);
    assert_eq!(update.child_ids[2], 3);
}

#[test]
fn wave_c17_021_boundary_children_reorder_when_bounds_swap() {
    let mut manager = SemanticManager::new();
    let list = node(1, SemanticRole::List, "menu");
    set_bounds(&list, SemanticBounds::new(0.0, 0.0, 200.0, 300.0));
    let boundary0 = SemanticNodeHandle::new(100);
    boundary0.borrow_mut().set_boundary_node(true);
    set_bounds(&boundary0, SemanticBounds::new(0.0, 0.0, 200.0, 50.0));
    let boundary1 = SemanticNodeHandle::new(101);
    boundary1.borrow_mut().set_boundary_node(true);
    set_bounds(&boundary1, SemanticBounds::new(0.0, 100.0, 200.0, 150.0));
    let item0 = node(2, SemanticRole::ListItem, "Item A");
    set_bounds(&item0, SemanticBounds::new(0.0, 0.0, 200.0, 50.0));
    let item1 = node(3, SemanticRole::ListItem, "Item B");
    set_bounds(&item1, SemanticBounds::new(0.0, 100.0, 200.0, 150.0));
    manager.add_child(None, list.clone());
    manager.add_child(Some(&list), boundary0.clone());
    manager.add_child(Some(&boundary0), item0.clone());
    manager.add_child(Some(&list), boundary1.clone());
    manager.add_child(Some(&boundary1), item1.clone());
    let first = drain(&mut manager);
    let first_update = children(&first, 1).expect("initial list children");
    assert_eq!(first_update.child_ids.len(), 2);
    assert_eq!(first_update.child_ids[0], 2);
    assert_eq!(first_update.child_ids[1], 3);
    set_bounds(&boundary0, SemanticBounds::new(0.0, 100.0, 200.0, 150.0));
    set_bounds(&boundary1, SemanticBounds::new(0.0, 0.0, 200.0, 50.0));
    set_bounds(&item0, SemanticBounds::new(0.0, 100.0, 200.0, 150.0));
    set_bounds(&item1, SemanticBounds::new(0.0, 0.0, 200.0, 50.0));
    manager.mark_node_dirty(100, SemanticDirt::BOUNDS);
    manager.mark_node_dirty(101, SemanticDirt::BOUNDS);
    manager.mark_node_dirty(2, SemanticDirt::BOUNDS);
    manager.mark_node_dirty(3, SemanticDirt::BOUNDS);
    let second = drain(&mut manager);
    let second_update = children(&second, 1).expect("reordered list children");
    assert_eq!(second_update.child_ids.len(), 2);
    assert_eq!(second_update.child_ids[0], 3);
    assert_eq!(second_update.child_ids[1], 2);
}

#[test]
fn wave_c17_022_added_array_is_first_frame_tree_preorder() {
    let mut manager = SemanticManager::new();
    let root = node(1, SemanticRole::Group, "");
    let g0 = node(2, SemanticRole::Group, "");
    let g1 = node(5, SemanticRole::Group, "");
    let t0 = node(3, SemanticRole::Text, "a");
    let t1 = node(4, SemanticRole::Text, "b");
    let t2 = node(6, SemanticRole::Text, "c");
    let t3 = node(7, SemanticRole::Text, "d");
    set_bounds(&root, SemanticBounds::new(0.0, 0.0, 100.0, 400.0));
    set_bounds(&g0, SemanticBounds::new(0.0, 0.0, 100.0, 200.0));
    set_bounds(&g1, SemanticBounds::new(0.0, 200.0, 100.0, 400.0));
    set_bounds(&t0, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    set_bounds(&t1, SemanticBounds::new(0.0, 100.0, 100.0, 200.0));
    set_bounds(&t2, SemanticBounds::new(0.0, 200.0, 100.0, 300.0));
    set_bounds(&t3, SemanticBounds::new(0.0, 300.0, 100.0, 400.0));
    manager.add_child(None, root.clone());
    manager.add_child(Some(&root), g0.clone());
    manager.add_child(Some(&root), g1.clone());
    manager.add_child(Some(&g0), t0);
    manager.add_child(Some(&g0), t1);
    manager.add_child(Some(&g1), t2);
    manager.add_child(Some(&g1), t3);
    let diff = drain(&mut manager);
    assert_eq!(diff.added.len(), 7);
    for (index, id) in [1, 2, 3, 4, 5, 6, 7].into_iter().enumerate() {
        assert_eq!(diff.added[index].id, id);
    }
    assert!(diff.children_updated.len() >= 3);
    assert_eq!(diff.children_updated[0].parent_id, -1);
    assert_eq!(diff.children_updated[1].parent_id, 1);
    assert_eq!(diff.children_updated[2].parent_id, 2);
    assert_eq!(diff.children_updated[3].parent_id, 5);
}

#[test]
fn wave_c17_023_subsequent_diff_arrays_emit_in_tree_order() {
    let mut manager = SemanticManager::new();
    let list = node(1, SemanticRole::List, "");
    let item0 = node(2, SemanticRole::ListItem, "A");
    let item1 = node(3, SemanticRole::ListItem, "B");
    let item2 = node(4, SemanticRole::ListItem, "C");
    set_bounds(&list, SemanticBounds::new(0.0, 0.0, 100.0, 300.0));
    set_bounds(&item0, SemanticBounds::new(0.0, 0.0, 100.0, 100.0));
    set_bounds(&item1, SemanticBounds::new(0.0, 100.0, 100.0, 200.0));
    set_bounds(&item2, SemanticBounds::new(0.0, 200.0, 100.0, 300.0));
    manager.add_child(None, list.clone());
    manager.add_child(Some(&list), item0.clone());
    manager.add_child(Some(&list), item1.clone());
    manager.add_child(Some(&list), item2.clone());
    let _first = drain(&mut manager);
    let item3 = node(5, SemanticRole::ListItem, "D");
    set_bounds(&item3, SemanticBounds::new(0.0, 300.0, 100.0, 400.0));
    manager.add_child(Some(&list), item3);
    item2.borrow_mut().set_label("C2");
    item0.borrow_mut().set_label("A2");
    manager.mark_node_dirty(4, SemanticDirt::CONTENT);
    manager.mark_node_dirty(2, SemanticDirt::CONTENT);
    set_bounds(&item1, SemanticBounds::new(0.0, 105.0, 100.0, 205.0));
    manager.mark_node_dirty(3, SemanticDirt::BOUNDS);
    let diff = drain(&mut manager);
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].id, 5);
    assert!(diff.updated_semantic.len() >= 2);
    let mut index2 = usize::MAX;
    let mut index4 = usize::MAX;
    for (index, owner) in diff.updated_semantic.iter().enumerate() {
        if owner.id == 2 {
            index2 = index;
        }
        if owner.id == 4 {
            index4 = index;
        }
    }
    assert_ne!(index2, usize::MAX);
    assert_ne!(index4, usize::MAX);
    assert!(index2 < index4);
}
