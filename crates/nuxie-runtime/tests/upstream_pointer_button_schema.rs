//! Generated owner/registry translation of pointer_button at upstream 2dbbfe18.
use nuxie_runtime::source::{
    core::binary_reader::BinaryReader, generated::core_registry::CoreRegistry,
    pointer_button::PointerButton,
};

#[test]
fn pointer_button_wire_storage_and_unknown_enum_values_match_cpp() {
    let mut object = CoreRegistry::make_core_box(155).expect("pointer input");
    assert_eq!(object.core_type(), 155);
    assert!(object.type_predicate()(658));
    assert_eq!(
        object.listener_input_type_pointer_button(),
        Some(PointerButton::Primary)
    );
    for (wire, expected) in [(1, 1), (2, 2), (255, 255), (256, 0), (511, 255)] {
        CoreRegistry::set_uint(object.as_mut(), 468, wire);
        assert_eq!(CoreRegistry::get_uint(object.as_mut(), 468), expected);
        assert_eq!(
            object.listener_input_type_pointer_button(),
            Some(PointerButton(expected as i32))
        );
        let cloned = object.clone_boxed().expect("clone");
        assert_eq!(
            cloned.listener_input_type_pointer_button(),
            object.listener_input_type_pointer_button()
        );
    }
    let mut reader = BinaryReader::new(&[0xff, 0x03]); // varuint 511, assigned to uint8_t.
    assert!(object.deserialize(468, &mut reader));
    assert!(!reader.has_error());
    assert_eq!(
        object.listener_input_type_pointer_button(),
        Some(PointerButton(255))
    );
    let mut inherited = BinaryReader::new(&[7]);
    assert!(object.deserialize(965, &mut inherited));
    assert_eq!(object.listener_input_type_value(), Some(7));
    for type_key in [658, 659, 665, 666, 669, 660, 973] {
        assert_eq!(
            CoreRegistry::make_core_box(type_key)
                .unwrap()
                .listener_input_type_pointer_button(),
            Some(PointerButton::Primary)
        );
    }
}
