use crate::mechanical_port::source::{
    animation::transition_value_comparator::TransitionValueComparator,
    animation::transition_value_id_comparator::TransitionValueIdComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionValueIdComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct TransitionValueIdComparatorBase {
    pub base: TransitionValueComparator,
    value: u32,
}

impl Default for TransitionValueIdComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueComparator::default(),
            value: u32::MAX,
        }
    }
}

impl TransitionValueIdComparatorBase {
    pub const TYPE_KEY: u16 = 601;
    pub const VALUE_PROPERTY_KEY: u16 = 653;

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
        callbacks: &mut impl TransitionValueIdComparatorBaseCallbacks,
    ) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: u32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionValueIdComparatorBaseCallbacks,
    ) -> TransitionValueIdComparator {
        let mut cloned = TransitionValueIdComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionValueIdComparatorBaseCallbacks,
    ) {
        self.value = object.value;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionValueIdComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for TransitionValueIdComparatorBase {
    type Target = TransitionValueComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueIdComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
