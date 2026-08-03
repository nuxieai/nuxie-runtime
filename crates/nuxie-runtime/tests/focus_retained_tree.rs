//! Direct public-surface ports from pinned tests/unit_tests/runtime/focus_test.cpp.

use nuxie_runtime::{FocusEdgeBehavior, FocusManager, FocusNode};

#[test]
fn upstream_focus_node_defaults_and_property_setters() {
    let mut node = FocusNode::new();
    assert!(node.can_focus());
    assert!(node.can_touch());
    assert!(node.can_traverse());
    assert_eq!(node.tab_index(), 0);
    assert_eq!(node.name(), b"");
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ParentScope);
    assert!(!node.has_focus());

    node.set_can_focus(false);
    node.set_can_touch(false);
    node.set_can_traverse(false);
    node.set_tab_index(42);
    node.set_name(b"button".to_vec());
    node.set_edge_behavior(FocusEdgeBehavior::Stop);

    assert!(!node.can_focus());
    assert!(!node.can_touch());
    assert!(!node.can_traverse());
    assert_eq!(node.tab_index(), 42);
    assert_eq!(node.name(), b"button");
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::Stop);
}

#[test]
fn upstream_focus_node_hierarchy_reparents_the_retained_identity() {
    let mut manager = FocusManager::new();
    let first_parent = manager.create_node(FocusNode::structural_scope());
    let second_parent = manager.create_node(FocusNode::structural_scope());
    let child = manager.create_node(FocusNode::new());

    assert!(manager.add_child(None, first_parent));
    assert!(manager.add_child(None, second_parent));
    assert!(manager.add_child(Some(first_parent), child));
    assert_eq!(manager.parent(child), Some(first_parent));
    assert_eq!(manager.children(first_parent), Some([child].as_slice()));

    assert!(manager.add_child(Some(second_parent), child));
    assert_eq!(manager.parent(child), Some(second_parent));
    assert_eq!(manager.children(first_parent), Some([].as_slice()));
    assert_eq!(manager.children(second_parent), Some([child].as_slice()));
    assert!(manager.contains(child));
}
