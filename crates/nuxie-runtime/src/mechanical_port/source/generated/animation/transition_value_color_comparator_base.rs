use crate::mechanical_port::source::{
    animation::transition_value_color_comparator::TransitionValueColorComparator,
    animation::transition_value_comparator::TransitionValueComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueColorComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueColorComparatorBase {
    pub base: TransitionValueComparator,
    value: i32,
}

impl Default for TransitionValueColorComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: 0xFF1D1D1Du32 as i32,
        }
    }
}

impl TransitionValueColorComparatorBase {
    pub const TYPE_KEY: u16 = 483;
    pub const VALUE_PROPERTY_KEY: u16 = 651;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn set_value(
        &mut self,
        value: i32,
        callbacks: &mut impl TransitionValueColorComparatorBaseCallbacks,
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
        callbacks: &mut impl TransitionValueColorComparatorBaseCallbacks,
    ) -> TransitionValueColorComparator {
        let mut cloned = TransitionValueColorComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueColorComparatorBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueColorComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
