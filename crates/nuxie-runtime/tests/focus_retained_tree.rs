//! Direct public-surface ports from pinned tests/unit_tests/runtime/focus_test.cpp.

use std::rc::Rc;

use nuxie_runtime::source::input::{
    focus_manager::{FocusManager, RuntimeFocusManagerHandle},
    focus_node::{EdgeBehavior as FocusEdgeBehavior, FocusNode},
};

#[test]
fn upstream_focus_node_defaults_and_property_setters() {
    let node = FocusNode::new(None);
    let mut node = node.borrow_mut();
    assert!(node.can_focus());
    assert!(node.can_touch());
    assert!(node.can_traverse());
    assert_eq!(node.tab_index(), 0);
    assert_eq!(node.name.as_bytes(), b"");
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ParentScope);
    assert!(!node.has_focus());

    node.set_can_focus(false);
    node.set_can_touch(false);
    node.set_can_traverse(false);
    node.set_tab_index(42);
    node.name = "button".to_owned();
    node.set_edge_behavior(FocusEdgeBehavior::Stop);

    assert!(!node.can_focus());
    assert!(!node.can_touch());
    assert!(!node.can_traverse());
    assert_eq!(node.tab_index(), 42);
    assert_eq!(node.name.as_bytes(), b"button");
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::Stop);
}

#[test]
fn upstream_focus_node_hierarchy_reparents_the_retained_identity() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let first_parent = FocusNode::make_structural_scope();
    let second_parent = FocusNode::make_structural_scope();
    let child = FocusNode::new(None);

    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, first_parent.clone(), None);
        manager.add_child(None, second_parent.clone(), None);
        manager.add_child(Some(first_parent.clone()), child.clone(), None);
    });
    assert!(Rc::ptr_eq(&child.borrow().parent().unwrap(), &first_parent));
    assert_eq!(first_parent.borrow().children().len(), 1);
    assert!(Rc::ptr_eq(&first_parent.borrow().children()[0], &child));

    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(second_parent.clone()), child.clone(), None);
    });
    assert!(Rc::ptr_eq(
        &child.borrow().parent().unwrap(),
        &second_parent
    ));
    assert!(first_parent.borrow().children().is_empty());
    assert_eq!(second_parent.borrow().children().len(), 1);
    assert!(Rc::ptr_eq(&second_parent.borrow().children()[0], &child));
    assert!(child.borrow().manager().unwrap().ptr_eq(&manager));
}

#[test]
fn manager_move_detach_and_drop_keep_the_actual_owner() {
    let first = RuntimeFocusManagerHandle::new(FocusManager::new());
    let second = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::make_structural_scope();
    let child = FocusNode::new(None);
    first.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    second.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    assert!(first.with_focus_manager(|manager| manager.root_nodes().is_empty()));
    assert!(scope.borrow().manager().unwrap().ptr_eq(&second));
    second.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), child.clone(), None);
        manager.detach_child(&scope);
    });
    assert!(scope.borrow().manager().is_none());
    assert!(child.borrow().manager().is_none());
    assert!(Rc::ptr_eq(&child.borrow().parent().unwrap(), &scope));
    second.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), None));
    drop(second);
    assert!(scope.borrow().manager().is_none());
    drop(scope);
    assert!(child.borrow().parent().is_none());
}

#[test]
fn node_changes_invalidate_the_live_manager_without_reborrowing_it() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::make_structural_scope();
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, scope.clone(), None);
        assert!(!manager.has_focusable_content());
        scope.borrow_mut().set_can_focus(true);
        assert!(manager.has_focusable_content());
        scope.borrow_mut().set_can_focus(false);
        assert!(!manager.has_focusable_content());
        FocusNode::add_child(&scope, FocusNode::new(None));
        assert!(manager.has_focusable_content());
    });
}
