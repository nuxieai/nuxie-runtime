//! New pure FocusManager cases from upstream 9b9cd7b1 focus_test.cpp.
use super::*;
use nuxie_runtime::source::semantic::semantic_snapshot::Bounds;

fn manager() -> RuntimeFocusManagerHandle {
    RuntimeFocusManagerHandle::new(FocusManager::new())
}
fn add(m: &RuntimeFocusManagerHandle, parent: Option<&FocusNodeRef>, node: &FocusNodeRef) {
    m.with_focus_manager_mut(|m| m.add_child(parent.cloned(), node.clone(), None));
}
fn plain(m: &RuntimeFocusManagerHandle, parent: Option<&FocusNodeRef>) -> FocusNodeRef {
    let node = FocusNode::new(None);
    add(m, parent, &node);
    node
}
fn backed(m: &RuntimeFocusManagerHandle, parent: Option<&FocusNodeRef>) -> FocusNodeRef {
    let node = FocusNode::new(Some(binding_focus_observer()));
    add(m, parent, &node);
    node
}
fn focus(m: &RuntimeFocusManagerHandle, node: &FocusNodeRef) {
    m.with_focus_manager_mut(|m| m.set_focus(node.clone()));
}
fn next(m: &RuntimeFocusManagerHandle, expected: &FocusNodeRef) {
    m.with_focus_manager_mut(FocusManager::focus_next);
    assert!(binding_focus_is_primary(m, expected));
}
fn previous(m: &RuntimeFocusManagerHandle, expected: &FocusNodeRef) {
    m.with_focus_manager_mut(FocusManager::focus_previous);
    assert!(binding_focus_is_primary(m, expected));
}
fn clear(m: &RuntimeFocusManagerHandle) {
    m.with_focus_manager_mut(FocusManager::clear_focus);
}

#[test]
fn closed_loop_skips_scope_that_cannot_be_focused() {
    let m = manager();
    let scope = plain(&m, None);
    let one = plain(&m, Some(&scope));
    let two = plain(&m, Some(&scope));
    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);
    scope.borrow_mut().set_can_focus(false);
    focus(&m, &two);
    next(&m, &one);
}

#[test]
fn traversal_is_preorder_in_both_directions() {
    let m = manager();
    let root = plain(&m, None);
    let a = plain(&m, Some(&root));
    let a1 = plain(&m, Some(&a));
    let a2 = plain(&m, Some(&a));
    let b = plain(&m, Some(&root));
    let order = [root, a, a1, a2, b];
    for node in &order {
        next(&m, node);
    }
    for node in order.iter().rev().skip(1) {
        previous(&m, node);
    }
}

#[test]
fn parents_flags_do_not_decide_for_children() {
    // Each block is one upstream SECTION, with a fresh manager and mocks.
    {
        let m = manager();
        let container = backed(&m, None);
        let child = backed(&m, Some(&container));
        container.borrow_mut().set_can_focus(false);
        assert_eq!(
            m.with_focus_manager(|m| m.get_traversable_nodes(None).len()),
            1
        );
        next(&m, &child);
    }
    {
        let m = manager();
        let container = backed(&m, None);
        let child = backed(&m, Some(&container));
        container.borrow_mut().set_can_traverse(false);
        next(&m, &child);
    }
    {
        let m = manager();
        let container = backed(&m, None);
        let child = backed(&m, Some(&container));
        let sibling = backed(&m, None);
        container.borrow_mut().set_can_focus(false);
        container.borrow_mut().world_bounds = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 50.0,
            max_y: 50.0,
        };
        child.borrow_mut().world_bounds = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        };
        sibling.borrow_mut().world_bounds = Bounds {
            min_x: 100.0,
            min_y: 0.0,
            max_x: 110.0,
            max_y: 10.0,
        };
        focus(&m, &sibling);
        assert!(m.with_focus_manager_mut(FocusManager::focus_left));
        assert!(binding_focus_is_primary(&m, &child));
    }
    {
        let m = manager();
        let before = backed(&m, None);
        let container = backed(&m, None);
        let child = backed(&m, Some(&container));
        container.borrow_mut().set_can_focus(false);
        focus(&m, &child);
        previous(&m, &before);
    }
    {
        let m = manager();
        let outer = backed(&m, None);
        let inner = backed(&m, Some(&outer));
        let child = backed(&m, Some(&inner));
        outer.borrow_mut().set_can_focus(false);
        inner.borrow_mut().set_can_traverse(false);
        next(&m, &child);
    }
}

#[test]
fn flag_matrix_decides_focus_and_navigation_separately() {
    for (can_focus, can_traverse) in [(true, true), (true, false), (false, true), (false, false)] {
        let m = manager();
        let before = backed(&m, None);
        let subject = backed(&m, None);
        let after = backed(&m, None);
        subject.borrow_mut().set_can_focus(can_focus);
        subject.borrow_mut().set_can_traverse(can_traverse);
        before.borrow_mut().world_bounds = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 10.0,
            max_y: 10.0,
        };
        subject.borrow_mut().world_bounds = Bounds {
            min_x: 50.0,
            min_y: 0.0,
            max_x: 60.0,
            max_y: 10.0,
        };
        after.borrow_mut().world_bounds = Bounds {
            min_x: 100.0,
            min_y: 0.0,
            max_x: 110.0,
            max_y: 10.0,
        };
        focus(&m, &subject);
        assert_eq!(binding_focus_is_primary(&m, &subject), can_focus);
        clear(&m);
        focus(&m, &before);
        m.with_focus_manager_mut(FocusManager::focus_next);
        assert_eq!(
            binding_focus_is_primary(&m, &subject),
            can_focus && can_traverse
        );
        focus(&m, &after);
        m.with_focus_manager_mut(FocusManager::focus_previous);
        assert_eq!(
            binding_focus_is_primary(&m, &subject),
            can_focus && can_traverse
        );
        focus(&m, &after);
        m.with_focus_manager_mut(FocusManager::focus_left);
        assert_eq!(
            binding_focus_is_primary(&m, &subject),
            can_focus && can_traverse
        );
        if can_focus {
            focus(&m, &subject);
            assert!(binding_focus_is_primary(&m, &subject));
            m.with_focus_manager_mut(FocusManager::drop_focus_if_focus_target_hidden);
            assert!(binding_focus_is_primary(&m, &subject));
        }
    }
}

#[test]
fn follows_flags_when_they_change_at_runtime() {
    {
        let m = manager();
        let observer = binding_focus_observer();
        let subject = FocusNode::new(Some(observer.clone()));
        let parent = plain(&m, None);
        add(&m, Some(&parent), &subject);
        let other = backed(&m, Some(&parent));
        focus(&m, &subject);
        assert!(binding_focus_is_primary(&m, &subject));
        subject.borrow_mut().set_can_focus(false);
        m.with_focus_manager_mut(FocusManager::drop_focus_if_focus_target_hidden);
        assert!(binding_focus_is_primary(&m, &other));
        assert_eq!(observer.borrow().blurred_count, 1);
    }
    {
        let m = manager();
        let observer = binding_focus_observer();
        let subject = FocusNode::new(Some(observer.clone()));
        add(&m, None, &subject);
        let other = backed(&m, None);
        focus(&m, &subject);
        assert!(binding_focus_is_primary(&m, &subject));
        subject.borrow_mut().set_can_traverse(false);
        m.with_focus_manager_mut(FocusManager::drop_focus_if_focus_target_hidden);
        assert!(binding_focus_is_primary(&m, &subject));
        assert_eq!(observer.borrow().blurred_count, 0);
        focus(&m, &other);
        m.with_focus_manager_mut(FocusManager::focus_previous);
        assert!(!binding_focus_is_primary(&m, &subject));
    }
    {
        let m = manager();
        let first = backed(&m, None);
        let second = backed(&m, None);
        next(&m, &first);
        clear(&m);
        first.borrow_mut().set_can_traverse(false);
        next(&m, &second);
        clear(&m);
        first.borrow_mut().set_can_traverse(true);
        first.borrow_mut().set_can_focus(false);
        next(&m, &second);
    }
}

#[test]
fn authored_focus_flags_behave_like_node_flags() {
    let m = manager();
    // The Rust Focusable adapter reaches its FocusData through its arena
    // identity; retain both real owners for the same lifetime as C++ locals.
    let arena = CoreArena::default();
    let container = arena.insert(FocusData::default());
    let child = arena.insert(FocusData::default());
    let flags = container
        .with_downcast::<FocusData, _>(|data| data.base.focus_flags())
        .expect("container FocusData");
    assert!(CoreRegistry::set_uint_handle(
        &container,
        FocusDataBase::FOCUS_FLAGS_PROPERTY_KEY.into(),
        flags & !FocusDataBase::CAN_FOCUS_BITMASK,
    ));
    let container_node = container
        .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        .expect("container focus node");
    let child_node = child
        .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
        .expect("child focus node");
    add(&m, None, &container_node);
    add(&m, Some(&container_node), &child_node);
    assert!(!container_node.borrow().can_focus());
    assert_eq!(
        m.with_focus_manager(|m| m.get_traversable_nodes(None).len()),
        1
    );
    focus(&m, &container_node);
    assert!(binding_primary(&m).is_none());
    next(&m, &child_node);
}

#[test]
fn queued_focus_request_never_lands_on_nonfocusable_target() {
    let m = manager();
    let observer = binding_focus_observer();
    let target = FocusNode::new(Some(observer.clone()));
    target.borrow_mut().set_can_focus(false);
    add(&m, None, &target);
    m.with_focus_manager_mut(|m| m.request_focus(Some(target.clone()), None));
    assert!(binding_primary(&m).is_none());
    m.with_focus_manager_mut(|m| m.process_pending_focus_requests(None));
    assert!(binding_primary(&m).is_none());
    m.with_focus_manager_mut(|m| m.finish_pending_focus_requests(None));
    assert!(binding_primary(&m).is_none());
    assert_eq!(observer.borrow().focused_count, 0);
}

#[test]
fn edge_behavior_on_childless_node_is_inert() {
    let m = manager();
    let one = plain(&m, None);
    let two = plain(&m, None);
    one.borrow_mut().set_edge_behavior(EdgeBehavior::ClosedLoop);
    two.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);
    focus(&m, &one);
    next(&m, &two);
}

#[test]
fn closed_loop_with_single_stop_does_not_move_focus() {
    let m = manager();
    let scope = plain(&m, None);
    let only = plain(&m, Some(&scope));
    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);
    scope.borrow_mut().set_can_focus(false);
    focus(&m, &only);
    assert!(!m.with_focus_manager_mut(FocusManager::focus_next));
    assert!(binding_focus_is_primary(&m, &only));
    assert!(!m.with_focus_manager_mut(FocusManager::focus_previous));
    assert!(binding_focus_is_primary(&m, &only));
}

#[test]
fn traversal_reenters_after_focused_node_detached() {
    let m = manager();
    let one = plain(&m, None);
    let two = plain(&m, None);
    focus(&m, &two);
    m.with_focus_manager_mut(|m| m.detach_child(&two));
    assert!(binding_focus_is_primary(&m, &two));
    next(&m, &one);
}

#[test]
fn traversal_reports_nothing_when_tree_holds_no_stop() {
    let m = manager();
    let parent = plain(&m, None);
    let child = plain(&m, Some(&parent));
    parent.borrow_mut().set_can_focus(false);
    child.borrow_mut().set_can_focus(false);
    assert!(!m.with_focus_manager_mut(FocusManager::focus_next));
    assert!(binding_primary(&m).is_none());
    assert!(!m.with_focus_manager_mut(FocusManager::focus_previous));
    assert!(binding_primary(&m).is_none());
    assert!(m.with_focus_manager(|m| m.get_traversable_nodes(None).is_empty()));
}

#[test]
fn traversal_does_not_descend_into_detached_subtree() {
    let m = manager();
    let stay = plain(&m, None);
    let detached = plain(&m, None);
    let child = plain(&m, Some(&detached));
    focus(&m, &detached);
    m.with_focus_manager_mut(|m| m.detach_child(&detached));
    assert!(binding_focus_is_primary(&m, &detached));
    assert!(detached.borrow().manager().is_none());
    assert!(child.borrow().manager().is_none());
    assert_eq!(detached.borrow().children().len(), 1);
    next(&m, &stay);
    clear(&m);
    focus(&m, &detached);
    previous(&m, &stay);
}

#[test]
fn traversable_nodes_lists_container_of_stops() {
    let m = manager();
    let container = backed(&m, None);
    container.borrow_mut().set_can_focus(false);
    assert!(m.with_focus_manager(|m| m.get_traversable_nodes(None).is_empty()));
    let _child = backed(&m, Some(&container));
    let roots = m.with_focus_manager(|m| m.get_traversable_nodes(None));
    assert_eq!(roots.len(), 1);
    assert!(Rc::ptr_eq(&roots[0], &container));
}

#[test]
fn traversal_skips_scope_that_cannot_be_focused() {
    let m = manager();
    let scope = backed(&m, None);
    let one = backed(&m, Some(&scope));
    let two = backed(&m, Some(&scope));
    scope.borrow_mut().set_can_focus(false);
    next(&m, &one);
    next(&m, &two);
}

#[test]
fn backward_traversal_exits_scope_that_cannot_be_focused() {
    let m = manager();
    let root = plain(&m, None);
    let before = plain(&m, Some(&root));
    let scope = plain(&m, Some(&root));
    let inner = plain(&m, Some(&scope));
    scope.borrow_mut().set_can_focus(false);
    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ParentScope);
    focus(&m, &inner);
    previous(&m, &before);
}

#[test]
fn closed_loop_wraps_backward_past_scope_that_cannot_be_focused() {
    let m = manager();
    let scope = plain(&m, None);
    let one = plain(&m, Some(&scope));
    let two = plain(&m, Some(&scope));
    scope.borrow_mut().set_can_focus(false);
    scope
        .borrow_mut()
        .set_edge_behavior(EdgeBehavior::ClosedLoop);
    focus(&m, &one);
    previous(&m, &two);
}

#[test]
fn stop_prevents_backward_traversal_from_focusable_scope() {
    let m = manager();
    let scope = plain(&m, None);
    let one = plain(&m, Some(&scope));
    let _two = plain(&m, Some(&scope));
    scope.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);
    focus(&m, &one);
    previous(&m, &scope);
    previous(&m, &scope);
}
