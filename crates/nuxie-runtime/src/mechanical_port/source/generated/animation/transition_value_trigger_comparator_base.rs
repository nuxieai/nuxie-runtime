use crate::mechanical_port::source::{
    animation::transition_value_comparator::TransitionValueComparator,
    animation::transition_value_trigger_comparator::TransitionValueTriggerComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueTriggerComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueTriggerComparatorBase {
    pub base: TransitionValueComparator,
    value: u32,
}

impl Default for TransitionValueTriggerComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: 0,
        }
    }
}

impl TransitionValueTriggerComparatorBase {
    pub const TYPE_KEY: u16 = 505;
    pub const VALUE_PROPERTY_KEY: u16 = 689;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn set_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TransitionValueTriggerComparatorBaseCallbacks,
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
        callbacks: &mut impl TransitionValueTriggerComparatorBaseCallbacks,
    ) -> TransitionValueTriggerComparator {
        let mut cloned = TransitionValueTriggerComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueTriggerComparatorBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueTriggerComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
