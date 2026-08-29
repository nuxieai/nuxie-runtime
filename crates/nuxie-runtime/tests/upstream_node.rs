//! Direct safe-Rust ports of pinned `tests/unit_tests/runtime/node_test.cpp`.

use nuxie_runtime::source::node::Node;

#[test]
fn node_instances() {
    assert_eq!(Node::default().base.x(), 0.0);
}

#[test]
fn node_x_function_returns_x_value() {
    let mut node = Node::default();
    assert_eq!(node.base.x(), 0.0);
    node.set_x(2.0);
    assert_eq!(node.base.x(), 2.0);
}
