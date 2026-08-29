use crate::mechanical_port::source::{
    animation::transition_value_comparator::TransitionValueComparator,
    animation::transition_value_number_comparator::TransitionValueNumberComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueNumberComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueNumberComparatorBase {
    pub base: TransitionValueComparator,
    value: f32,
}

impl Default for TransitionValueNumberComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: 0.0,
        }
    }
}

impl TransitionValueNumberComparatorBase {
    pub const TYPE_KEY: u16 = 484;
    pub const VALUE_PROPERTY_KEY: u16 = 652;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 480 | 477)
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
        callbacks: &mut impl TransitionValueNumberComparatorBaseCallbacks,
    ) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionValueNumberComparatorBaseCallbacks,
    ) -> TransitionValueNumberComparator {
        let mut cloned = TransitionValueNumberComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueNumberComparatorBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueNumberComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for TransitionValueNumberComparatorBase {
    type Target = TransitionValueComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueNumberComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
