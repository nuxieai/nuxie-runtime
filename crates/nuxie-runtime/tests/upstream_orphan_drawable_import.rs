//! Literal translation of orphan_drawable_import_test.cpp at 439ffe0c.
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
#[path = "support/import_439.rs"]
mod support;
use nuxie_runtime::source::generated::{
    component_base::ComponentBase, node_base::NodeBase, shapes::shape_base::ShapeBase,
};
use riv_bytes::{RivBytes, write_artboard};
fn load(with_valid_sibling: bool) {
    let mut riv = RivBytes::default();
    write_artboard(&mut riv);
    if with_valid_sibling {
        riv.object(NodeBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
        riv.end();
    }
    riv.object(ShapeBase::TYPE_KEY);
    riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 121);
    riv.end();
    support::artboard(&support::import(&riv.bytes()));
}
#[test]
fn an_orphan_drawable_next_to_a_valid_node_is_dropped() {
    load(true);
}
#[test]
fn an_artboard_holding_only_an_orphan_drawable_loads() {
    load(false);
}
