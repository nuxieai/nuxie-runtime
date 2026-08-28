use crate::mechanical_port::source::{
    animation::keyed_object::KeyedObject, core::binary_reader::BinaryReader, core::Core,
};

pub trait KeyedObjectBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn object_id_changed(&mut self) {}
}

pub struct KeyedObjectBase {
    pub base: Core,
    object_id: u32,
}

impl Default for KeyedObjectBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            object_id: 0,
        }
    }
}

impl KeyedObjectBase {
    pub const TYPE_KEY: u16 = 25;
    pub const OBJECT_ID_PROPERTY_KEY: u16 = 51;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn object_id(&self) -> u32 {
        self.object_id
    }
    pub fn set_object_id(&mut self, value: u32, callbacks: &mut impl KeyedObjectBaseCallbacks) {
        if !self.set_object_id_value(value) {
            return;
        }
        callbacks.object_id_changed();
        callbacks.notify_property_changed(Self::OBJECT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_object_id_value(&mut self, value: u32) -> bool {
        if self.object_id == value {
            return false;
        }
        self.object_id = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyedObjectBaseCallbacks) -> KeyedObject {
        let mut cloned = KeyedObject::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyedObjectBaseCallbacks) {
        self.object_id = object.object_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyedObjectBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OBJECT_ID_PROPERTY_KEY => {
                self.object_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for KeyedObjectBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyedObjectBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
