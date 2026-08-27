use crate::mechanical_port::source::{
    animation::transition_condition::TransitionCondition, core::binary_reader::BinaryReader,
};

pub trait TransitionViewModelConditionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn op_value_changed(&mut self) {}
}

pub struct TransitionViewModelConditionBase {
    pub base: TransitionCondition,
    op_value: u32,
}

impl Default for TransitionViewModelConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionCondition::default(),
            op_value: 0,
        }
    }
}

impl TransitionViewModelConditionBase {
    pub const TYPE_KEY: u16 = 482;
    pub const OP_VALUE_PROPERTY_KEY: u16 = 650;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn op_value(&self) -> u32 {
        self.op_value
    }
    pub fn set_op_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TransitionViewModelConditionBaseCallbacks,
    ) {
        if self.op_value == value {
            return;
        }
        self.op_value = value;
        callbacks.op_value_changed();
        callbacks.notify_property_changed(Self::OP_VALUE_PROPERTY_KEY);
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionViewModelConditionBaseCallbacks,
    ) {
        self.op_value = object.op_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionViewModelConditionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OP_VALUE_PROPERTY_KEY => {
                self.op_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
