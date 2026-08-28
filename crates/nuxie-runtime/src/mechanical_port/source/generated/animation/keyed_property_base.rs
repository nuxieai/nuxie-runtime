use crate::mechanical_port::source::{
    animation::keyed_property::KeyedProperty, core::binary_reader::BinaryReader, core::Core,
};

pub trait KeyedPropertyBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_key_changed(&mut self) {}
}

pub struct KeyedPropertyBase {
    pub base: Core,
    property_key: u32,
}

impl Default for KeyedPropertyBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            property_key: Core::invalidPropertyKey,
        }
    }
}

impl KeyedPropertyBase {
    pub const TYPE_KEY: u16 = 26;
    pub const PROPERTY_KEY_PROPERTY_KEY: u16 = 53;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_key(&self) -> u32 {
        self.property_key
    }
    pub fn set_property_key(
        &mut self,
        value: u32,
        callbacks: &mut impl KeyedPropertyBaseCallbacks,
    ) {
        if !self.set_property_key_value(value) {
            return;
        }
        callbacks.property_key_changed();
        callbacks.notify_property_changed(Self::PROPERTY_KEY_PROPERTY_KEY);
    }

    pub(crate) fn set_property_key_value(&mut self, value: u32) -> bool {
        if self.property_key == value {
            return false;
        }
        self.property_key = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyedPropertyBaseCallbacks) -> KeyedProperty {
        let mut cloned = KeyedProperty::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyedPropertyBaseCallbacks) {
        self.property_key = object.property_key;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyedPropertyBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_KEY_PROPERTY_KEY => {
                self.property_key = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for KeyedPropertyBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyedPropertyBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
