use crate::mechanical_port::source::{
    animation::transition_number_condition::TransitionNumberCondition,
    animation::transition_value_condition::TransitionValueCondition,
    core::binary_reader::BinaryReader,
};

pub trait TransitionNumberConditionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionNumberConditionBase {
    pub base: TransitionValueCondition,
    value: f32,
}

impl Default for TransitionNumberConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionValueCondition::default(),
            value: 0.0,
        }
    }
}

impl TransitionNumberConditionBase {
    pub const TYPE_KEY: u16 = 70;
    pub const VALUE_PROPERTY_KEY: u16 = 157;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 69 | 67 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(
        &mut self,
        value: f32,
        callbacks: &mut impl TransitionNumberConditionBaseCallbacks,
    ) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionNumberConditionBaseCallbacks,
    ) -> TransitionNumberCondition {
        let mut cloned = TransitionNumberCondition::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionNumberConditionBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionNumberConditionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
