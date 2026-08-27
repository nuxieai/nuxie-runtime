use crate::mechanical_port::source::{
    animation::transition_property_artboard_comparator::TransitionPropertyArtboardComparator,
    animation::transition_property_comparator::TransitionPropertyComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionPropertyArtboardComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_type_changed(&mut self) {}
}

pub struct TransitionPropertyArtboardComparatorBase {
    pub base: TransitionPropertyComparator,
    property_type: u32,
}

impl Default for TransitionPropertyArtboardComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionPropertyComparator::default(),
            property_type: 0,
        }
    }
}

impl TransitionPropertyArtboardComparatorBase {
    pub const TYPE_KEY: u16 = 496;
    pub const PROPERTY_TYPE_PROPERTY_KEY: u16 = 677;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 478 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_type(&self) -> u32 {
        self.property_type
    }
    pub fn set_property_type(
        &mut self,
        value: u32,
        callbacks: &mut impl TransitionPropertyArtboardComparatorBaseCallbacks,
    ) {
        if self.property_type == value {
            return;
        }
        self.property_type = value;
        callbacks.property_type_changed();
        callbacks.notify_property_changed(Self::PROPERTY_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionPropertyArtboardComparatorBaseCallbacks,
    ) -> TransitionPropertyArtboardComparator {
        let mut cloned = TransitionPropertyArtboardComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionPropertyArtboardComparatorBaseCallbacks,
    ) {
        self.property_type = object.property_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionPropertyArtboardComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_TYPE_PROPERTY_KEY => {
                self.property_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
