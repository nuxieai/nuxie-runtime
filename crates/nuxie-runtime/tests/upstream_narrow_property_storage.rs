//! Source-derived coverage for 61f00897's uint8/uint16 generated properties.
//! Wire values remain varuint; C++ assignment and setter calls narrow modulo
//! the field width rather than rejecting or saturating oversized values.

use nuxie_runtime::source::{
    animation::{
        state_machine_fire_action::StateMachineFireOccurance,
        state_machine_fire_trigger::StateMachineFireTrigger,
        transition_condition_op::TransitionConditionOp,
        transition_number_condition::TransitionNumberCondition,
    },
    core::{CoreObject, binary_reader::BinaryReader},
    data_bind::data_values::data_type::DataType,
    generated::core_registry::CoreRegistry,
    shapes::{
        clipping_shape::ClippingShape,
        paint::{fill::Fill, stroke::Stroke},
        shape::Shape,
    },
    viewmodel::viewmodel_property::ViewModelProperty,
};

#[test]
fn narrowed_generated_properties_truncate_binary_and_registry_values() {
    let cases: Vec<(Box<dyn CoreObject>, u16, u32)> = vec![
        (Box::new(StateMachineFireTrigger::default()), 393, 255),
        (Box::new(TransitionNumberCondition::default()), 156, 255),
        (Box::new(Shape::default()), 23, 255),
        (Box::new(Shape::default()), 129, 65535),
        (Box::new(ClippingShape::default()), 93, 255),
        (Box::new(Fill::default()), 40, 255),
        (Box::new(Stroke::default()), 48, 255),
        (Box::new(Stroke::default()), 49, 255),
        (Box::new(ViewModelProperty::default()), 875, 255),
        (Box::new(ViewModelProperty::default()), 957, 255),
    ];
    for (mut object, key, mask) in cases {
        // Maximum uint32 varuint is narrowed by generated deserialization.
        let mut reader = BinaryReader::new(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert!(object.deserialize(key, &mut reader));
        assert_eq!(CoreRegistry::get_uint(&mut *object, i32::from(key)), mask);
        let input = mask + 3;
        CoreRegistry::set_uint(&mut *object, i32::from(key), input);
        assert_eq!(CoreRegistry::get_uint(&mut *object, i32::from(key)), 2);
    }
}

#[test]
fn narrowed_enum_and_public_getter_types_match_upstream() {
    assert_eq!(std::mem::size_of::<DataType>(), 1);
    assert_eq!(std::mem::size_of::<TransitionConditionOp>(), 1);
    assert_eq!(std::mem::size_of::<StateMachineFireOccurance>(), 1);
    let _: u8 = Fill::default().base.fill_rule();
    let _: u8 = Stroke::default().base.cap();
    let _: u16 = Shape::default().base.drawable_flags();
}
