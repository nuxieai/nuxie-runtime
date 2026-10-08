use crate::mechanical_port::source::component::Component;
use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    generated::component_base::ComponentBaseCallbacks,
};

pub struct CustomPropertyBase {
    pub base: Component,
    name_id: u32,
}
impl Default for CustomPropertyBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            name_id: u32::MAX,
        }
    }
}
impl CustomPropertyBase {
    pub const TYPE_KEY: u16 = 167;
    pub const NAME_ID_PROPERTY_KEY: u16 = 449;
    pub fn name_id(&self) -> u32 {
        self.name_id
    }
    pub fn set_name_id(&mut self, value: u32, callbacks: &mut impl ComponentBaseCallbacks) {
        if self.set_name_id_value(value) {
            callbacks.notify_property_changed(Self::NAME_ID_PROPERTY_KEY);
        }
    }
    pub(crate) fn set_name_id_value(&mut self, value: u32) -> bool {
        if self.name_id == value {
            return false;
        }
        self.name_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ComponentBaseCallbacks) {
        self.name_id = object.name_id;
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ComponentBaseCallbacks,
    ) -> bool {
        if key == Self::NAME_ID_PROPERTY_KEY {
            self.name_id = CoreUintType::deserialize(reader);
            true
        } else {
            self.base.base.deserialize(key, reader, callbacks)
        }
    }
    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for CustomPropertyBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
