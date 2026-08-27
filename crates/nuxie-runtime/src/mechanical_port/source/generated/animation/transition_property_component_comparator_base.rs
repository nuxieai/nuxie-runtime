use crate::mechanical_port::source::{
    animation::transition_property_comparator::TransitionPropertyComparator,
    animation::transition_property_component_comparator::TransitionPropertyComponentComparator,
    core::binary_reader::BinaryReader,
};

pub trait TransitionPropertyComponentComparatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn object_id_changed(&mut self) {}
    fn property_key_changed(&mut self) {}
}

pub struct TransitionPropertyComponentComparatorBase {
    pub base: TransitionPropertyComparator,
    object_id: u32,
    property_key: u32,
}

impl Default for TransitionPropertyComponentComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionPropertyComparator::default(),
            object_id: 0,
            property_key: 0,
        }
    }
}

impl TransitionPropertyComponentComparatorBase {
    pub const TYPE_KEY: u16 = 667;
    pub const OBJECT_ID_PROPERTY_KEY: u16 = 977;
    pub const PROPERTY_KEY_PROPERTY_KEY: u16 = 978;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 478 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn object_id(&self) -> u32 {
        self.object_id
    }
    pub fn set_object_id(
        &mut self,
        value: u32,
        callbacks: &mut impl TransitionPropertyComponentComparatorBaseCallbacks,
    ) {
        if self.object_id == value {
            return;
        }
        self.object_id = value;
        callbacks.object_id_changed();
        callbacks.notify_property_changed(Self::OBJECT_ID_PROPERTY_KEY);
    }
    pub fn property_key(&self) -> u32 {
        self.property_key
    }
    pub fn set_property_key(
        &mut self,
        value: u32,
        callbacks: &mut impl TransitionPropertyComponentComparatorBaseCallbacks,
    ) {
        if self.property_key == value {
            return;
        }
        self.property_key = value;
        callbacks.property_key_changed();
        callbacks.notify_property_changed(Self::PROPERTY_KEY_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TransitionPropertyComponentComparatorBaseCallbacks,
    ) -> TransitionPropertyComponentComparator {
        let mut cloned = TransitionPropertyComponentComparator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransitionPropertyComponentComparatorBaseCallbacks,
    ) {
        self.object_id = object.object_id;
        self.property_key = object.property_key;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransitionPropertyComponentComparatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OBJECT_ID_PROPERTY_KEY => {
                self.object_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::PROPERTY_KEY_PROPERTY_KEY => {
                self.property_key = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
