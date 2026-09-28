use nuxie_runtime::source::{
    focus_data::FocusData,
    generated::{core_registry::CoreRegistry, focus_data_base::FocusDataBase},
    layout_component::LayoutComponent,
    node::Node,
};

#[test]
fn unrooted_computed_coordinates_are_zero() {
    // Both upstream owners return before touching transforms or layout anchors.
    let mut node = Node::default();
    assert_eq!(node.computed_root_x(), 0.0);
    assert_eq!(node.computed_root_y(), 0.0);
    let layout = LayoutComponent::default();
    assert_eq!(layout.computed_root_x(), 0.0);
    assert_eq!(layout.computed_root_y(), 0.0);
}

#[test]
fn id_registry_dispatch_overlaps_uint_dispatch() {
    let mut node = Node::default();
    CoreRegistry::set_id(&mut node, 5, 17);
    assert_eq!(CoreRegistry::get_id(&mut node, 5), 17);
    assert_eq!(CoreRegistry::get_uint(&mut node, 5), 17);
    CoreRegistry::set_uint(&mut node, 5, 99);
    assert_eq!(CoreRegistry::get_id(&mut node, 5), 99);
    CoreRegistry::set_id(&mut node, 5, u32::MAX);
    assert_eq!(CoreRegistry::get_id(&mut node, 5), u32::MAX);
    assert_eq!(CoreRegistry::get_uint(&mut node, 5), u32::MAX);
    CoreRegistry::set_id(&mut node, 5, 0);
    assert_eq!(CoreRegistry::get_id(&mut node, 5), 0);
    assert_eq!(CoreRegistry::get_uint(&mut node, 5), 0);
}

#[test]
fn focus_boolean_registry_getters_read_owner_flags() {
    let mut focus = FocusData::default();
    for (key, mask) in [
        (953, FocusDataBase::CAN_FOCUS_BITMASK),
        (954, FocusDataBase::CAN_TOUCH_BITMASK),
        (955, FocusDataBase::CAN_TRAVERSE_BITMASK),
    ] {
        assert_eq!(
            CoreRegistry::get_bool(&mut focus, key),
            focus.base.focus_flags() & mask != 0,
        );
        for value in [false, true, false] {
            CoreRegistry::set_bool(&mut focus, key, value);
            assert_eq!(focus.base.focus_flags() & mask != 0, value);
            assert_eq!(CoreRegistry::get_bool(&mut focus, key), value);
        }
    }
}
