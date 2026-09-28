//! Seven focus_test.cpp additions at 58c2f5e80bd7398496561197037f93e7fbe35d6f.
use nuxie_runtime::source::{
    core::CoreArena,
    focus_data::FocusData,
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::{FocusNode, FocusNodeRef, FocusableRef},
        focusable::{Focusable, Key, KeyModifiers},
    },
};
use std::{cell::RefCell, rc::Rc};

struct MockFocusable;
impl Focusable for MockFocusable {
    fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        false
    }
    fn text_input(&mut self, _: &str) -> bool {
        false
    }
    fn focused(&mut self) {}
    fn blurred(&mut self) {}
}
fn manager() -> RuntimeFocusManagerHandle {
    RuntimeFocusManagerHandle::new(FocusManager::new())
}
fn backing() -> FocusableRef {
    Rc::new(RefCell::new(MockFocusable))
}
fn owned_by(node: &FocusNodeRef, manager: &RuntimeFocusManagerHandle) -> bool {
    node.borrow()
        .manager()
        .is_some_and(|owner| owner.ptr_eq(manager))
}
fn focused(manager: &RuntimeFocusManagerHandle, node: &FocusNodeRef) -> bool {
    manager.with_focus_manager(|manager| {
        manager
            .primary_focus()
            .is_some_and(|focus| Rc::ptr_eq(&focus, node))
    })
}
fn data_node(arena: &CoreArena) -> FocusNodeRef {
    // The arena supplies the weak owner used by Rust's FocusDataFocusable adapter.
    // Its scope owns the real FocusData, mirroring the upstream stack lifetime.
    arena
        .insert(FocusData::default())
        .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        .unwrap()
}

#[test]
fn a_node_that_loses_its_focusable_stops_being_a_focus_stop() {
    let manager = manager();
    let backing = backing();
    let live = FocusNode::new(Some(backing.clone()));
    let defunct = FocusNode::new(Some(backing));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, live.clone(), None);
        manager.add_child(None, defunct.clone(), None);
    });
    assert_eq!(
        manager.with_focus_manager(|m| m.get_traversable_nodes(None).len()),
        2
    );
    defunct.borrow_mut().clear_focusable();
    assert!(defunct.borrow().had_focusable());
    let traversable = manager.with_focus_manager(|m| m.get_traversable_nodes(None));
    assert_eq!(traversable.len(), 1);
    assert!(Rc::ptr_eq(&traversable[0], &live));
    manager.with_focus_manager_mut(|m| m.set_focus(defunct.clone()));
    assert!(!focused(&manager, &defunct));
}

#[test]
fn a_node_that_never_had_a_focusable_stays_focusable() {
    let manager = manager();
    let external = FocusNode::new(None);
    assert!(!external.borrow().had_focusable());
    manager.with_focus_manager_mut(|m| m.add_child(None, external.clone(), None));
    let traversable = manager.with_focus_manager(|m| m.get_traversable_nodes(None));
    assert_eq!(traversable.len(), 1);
    assert!(Rc::ptr_eq(&traversable[0], &external));
    manager.with_focus_manager_mut(|m| m.set_focus(external.clone()));
    assert!(focused(&manager, &external));
}

#[test]
fn traversal_still_descends_through_a_defunct_node_to_live_children() {
    let manager = manager();
    let backing = backing();
    let scope = FocusNode::new(Some(backing.clone()));
    let child = FocusNode::new(Some(backing));
    manager.with_focus_manager_mut(|m| {
        m.add_child(None, scope.clone(), None);
        m.add_child(Some(scope.clone()), child.clone(), None);
    });
    scope.borrow_mut().clear_focusable();
    assert!(manager.with_focus_manager_mut(FocusManager::focus_next));
    assert!(focused(&manager, &child));
    manager.with_focus_manager_mut(|m| {
        m.clear_focus();
        m.set_focus(scope);
    });
    assert!(manager.with_focus_manager(|m| m.primary_focus().is_none()));
}

#[test]
fn adding_a_node_claims_its_whole_subtree_for_the_manager() {
    let manager = manager();
    let backing = backing();
    let parent = FocusNode::new(Some(backing.clone()));
    let child = FocusNode::new(Some(backing.clone()));
    let grandchild = FocusNode::new(Some(backing));
    manager.with_focus_manager_mut(|m| {
        m.add_child(None, parent.clone(), None);
        m.add_child(Some(parent.clone()), child.clone(), None);
        m.add_child(Some(child.clone()), grandchild.clone(), None);
    });
    for node in [&parent, &child, &grandchild] {
        assert!(owned_by(node, &manager));
    }
    manager.with_focus_manager_mut(|m| m.detach_child(&parent));
    for node in [&parent, &child, &grandchild] {
        assert!(node.borrow().manager().is_none());
    }
    assert_eq!(parent.borrow().children().len(), 1);
    manager.with_focus_manager_mut(|m| m.add_child(None, parent.clone(), None));
    for node in [&parent, &child, &grandchild] {
        assert!(owned_by(node, &manager));
    }
}

#[test]
fn a_rebuild_that_misses_a_node_still_lets_its_focus_data_clean_up() {
    let manager = manager();
    let root = FocusNode::new(Some(backing()));
    manager.with_focus_manager_mut(|m| m.add_child(None, root.clone(), None));
    {
        let arena = CoreArena::default();
        let missed = data_node(&arena);
        manager.with_focus_manager_mut(|m| {
            m.add_child(Some(root.clone()), missed.clone(), None);
            m.detach_child(&root);
            m.add_child(None, root.clone(), None);
        });
        assert!(owned_by(&missed, &manager));
        assert_eq!(root.borrow().children().len(), 1);
    }
    assert!(root.borrow().children().is_empty());
    assert!(manager.with_focus_manager(|m| m.get_traversable_nodes(Some(&root)).is_empty()));
}

#[test]
fn a_dying_focus_data_removes_its_node_via_an_ancestors_manager() {
    let manager = manager();
    let scope = FocusNode::new(Some(backing()));
    manager.with_focus_manager_mut(|m| m.add_child(None, scope.clone(), None));
    assert!(owned_by(&scope, &manager));
    {
        let arena = CoreArena::default();
        let node = data_node(&arena);
        FocusNode::add_child(&scope, node.clone());
        assert!(node.borrow().manager().is_none());
        assert_eq!(scope.borrow().children().len(), 1);
        manager.with_focus_manager_mut(|m| m.set_focus(node.clone()));
        assert!(focused(&manager, &node));
    }
    assert!(scope.borrow().children().is_empty());
    assert!(manager.with_focus_manager(|m| m.primary_focus().is_none()));
}

#[test]
fn a_dying_focus_data_detaches_its_node_when_no_manager_is_reachable() {
    let scope = FocusNode::new(None);
    {
        let arena = CoreArena::default();
        FocusNode::add_child(&scope, data_node(&arena));
        assert_eq!(scope.borrow().children().len(), 1);
    }
    assert!(scope.borrow().children().is_empty());
}
