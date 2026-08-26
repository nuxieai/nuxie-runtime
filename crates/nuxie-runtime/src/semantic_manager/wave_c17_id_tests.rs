//! Exact ports of cases 24-27 from pinned `semantic_label_inference_test.cpp`.

use super::*;
use crate::SemanticRole;

fn set_role(node: &SemanticNodeHandle, role: SemanticRole) {
    node.borrow_mut().set_role(role as u32);
}

#[test]
fn wave_c17_024_nodes_constructed_without_an_id_receive_one_on_add_child() {
    let mut manager = SemanticManager::new();

    let a = SemanticNodeHandle::new(0);
    let b = SemanticNodeHandle::new(0);
    set_role(&a, SemanticRole::Group);
    set_role(&b, SemanticRole::Text);
    b.borrow_mut().set_label("hello");

    assert_eq!(a.borrow().id(), 0);
    assert_eq!(b.borrow().id(), 0);

    manager.add_child(None, a.clone());
    manager.add_child(Some(&a), b.clone());

    assert_ne!(a.borrow().id(), 0);
    assert_ne!(b.borrow().id(), 0);
    assert_ne!(a.borrow().id(), b.borrow().id());
}

#[test]
fn wave_c17_025_two_independent_semantic_managers_do_not_share_an_id_space() {
    let mut manager_a = SemanticManager::new();
    let mut manager_b = SemanticManager::new();

    let a0 = SemanticNodeHandle::new(0);
    let a1 = SemanticNodeHandle::new(0);
    manager_a.add_child(None, a0.clone());
    manager_a.add_child(Some(&a0), a1.clone());

    let b0 = SemanticNodeHandle::new(0);
    let b1 = SemanticNodeHandle::new(0);
    manager_b.add_child(None, b0.clone());
    manager_b.add_child(Some(&b0), b1.clone());

    assert_eq!(a0.borrow().id(), b0.borrow().id());
    assert_eq!(a1.borrow().id(), b1.borrow().id());
    assert!(
        manager_a
            .node_by_id(a0.borrow().id())
            .is_some_and(|node| node.ptr_eq(&a0))
    );
    assert!(
        manager_b
            .node_by_id(b0.borrow().id())
            .is_some_and(|node| node.ptr_eq(&b0))
    );
}

#[test]
fn wave_c17_026_explicit_ids_are_preserved_and_bump_the_manager_watermark() {
    let mut manager = SemanticManager::new();

    let explicit10 = SemanticNodeHandle::new(10);
    manager.add_child(None, explicit10.clone());
    assert_eq!(explicit10.borrow().id(), 10);

    let auto1 = SemanticNodeHandle::new(0);
    manager.add_child(Some(&explicit10), auto1.clone());
    assert!(auto1.borrow().id() > 10);

    let explicit5 = SemanticNodeHandle::new(5);
    manager.add_child(Some(&explicit10), explicit5.clone());
    assert_eq!(explicit5.borrow().id(), 5);

    let auto2 = SemanticNodeHandle::new(0);
    manager.add_child(Some(&explicit10), auto2.clone());
    assert!(auto2.borrow().id() > 10);
    assert_ne!(auto2.borrow().id(), auto1.borrow().id());
}

#[test]
fn wave_c17_027_explicit_id_collision_reassigns_the_second_resident_node() {
    let mut manager = SemanticManager::new();

    let first = SemanticNodeHandle::new(42);
    manager.add_child(None, first.clone());
    assert_eq!(first.borrow().id(), 42);

    let second = SemanticNodeHandle::new(42);
    manager.add_child(Some(&first), second.clone());

    assert_ne!(second.borrow().id(), 0);
    assert_ne!(second.borrow().id(), 42);
    assert!(
        manager
            .node_by_id(42)
            .is_some_and(|node| node.ptr_eq(&first))
    );
    assert!(
        manager
            .node_by_id(second.borrow().id())
            .is_some_and(|node| node.ptr_eq(&second))
    );
}
