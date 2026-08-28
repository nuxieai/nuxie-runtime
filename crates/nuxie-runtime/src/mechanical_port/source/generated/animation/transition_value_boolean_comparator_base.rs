use crate::mechanical_port::source::{
    animation::transition_value_boolean_comparator::TransitionValueBooleanComparator,
    animation::transition_value_comparator::TransitionValueComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueBooleanComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueBooleanComparatorBase {
    pub base: TransitionValueComparator,
    value: bool,
}

impl Default for TransitionValueBooleanComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: false,
        }
    }
}

impl TransitionValueBooleanComparatorBase {
    pub const TYPE_KEY: u16 = 481;
    pub const VALUE_PROPERTY_KEY: u16 = 647;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> bool {
        self.value
    }
    pub fn set_value(
        &mut self,
        value: bool,
        callbacks: &mut impl TransitionValueBooleanComparatorBaseCallbacks,
    ) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: bool) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionValueBooleanComparatorBaseCallbacks,
    ) -> TransitionValueBooleanComparator {
        let mut cloned = TransitionValueBooleanComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueBooleanComparatorBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueBooleanComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for TransitionValueBooleanComparatorBase {
    type Target = TransitionValueComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueBooleanComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
