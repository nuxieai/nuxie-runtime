use crate::mechanical_port::source::{
    animation::transition_value_comparator::TransitionValueComparator,
    animation::transition_value_string_comparator::TransitionValueStringComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueStringComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueStringComparatorBase {
    pub base: TransitionValueComparator,
    value: String,
}

impl Default for TransitionValueStringComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: "".to_owned(),
        }
    }
}

impl TransitionValueStringComparatorBase {
    pub const TYPE_KEY: u16 = 486;
    pub const VALUE_PROPERTY_KEY: u16 = 654;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(
        &mut self,
        value: String,
        callbacks: &mut impl TransitionValueStringComparatorBaseCallbacks,
    ) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: String) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionValueStringComparatorBaseCallbacks,
    ) -> TransitionValueStringComparator {
        let mut cloned = TransitionValueStringComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueStringComparatorBaseCallbacks,
    ) {
        self.value.clone_from(&object.value);
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueStringComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for TransitionValueStringComparatorBase {
    type Target = TransitionValueComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueStringComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
