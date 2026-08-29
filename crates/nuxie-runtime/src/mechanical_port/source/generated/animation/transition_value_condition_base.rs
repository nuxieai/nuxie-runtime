use crate::mechanical_port::source::{
    animation::transition_input_condition::TransitionInputCondition,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueConditionBaseCallbacks: crate::mechanical_port::source::generated::animation::transition_input_condition_base::TransitionInputConditionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn op_value_changed(&mut self) {}
}

pub struct TransitionValueConditionBase {
    pub base: TransitionInputCondition,
    op_value: u32,
}

impl Default for TransitionValueConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionInputCondition::default(),
            op_value: 0,
        }
    }
}

impl TransitionValueConditionBase {
    pub const TYPE_KEY: u16 = 69;
    pub const OP_VALUE_PROPERTY_KEY: u16 = 156;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 67 | 476)
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
        callbacks: &mut impl TransitionValueConditionBaseCallbacks,
    ) {
        if !self.set_op_value_value(value) {
            return;
        }
        callbacks.op_value_changed();
        TransitionValueConditionBaseCallbacks::notify_property_changed(
            callbacks,
            Self::OP_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_op_value_value(&mut self, value: u32) -> bool {
        if self.op_value == value {
            return false;
        }
        self.op_value = value;
        true
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueConditionBaseCallbacks,
    ) {
        self.op_value = object.op_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueConditionBaseCallbacks,
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

impl std::ops::Deref for TransitionValueConditionBase {
    type Target = TransitionInputCondition;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueConditionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
