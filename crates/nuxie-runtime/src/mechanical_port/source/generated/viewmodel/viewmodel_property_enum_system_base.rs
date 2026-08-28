use crate::mechanical_port::source::viewmodel::viewmodel_property_enum_system::ViewModelPropertyEnumSystem;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property_enum::ViewModelPropertyEnum,
};

pub trait ViewModelPropertyEnumSystemBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_property_base::ViewModelPropertyBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn enum_type_changed(&mut self) {}
}

pub struct ViewModelPropertyEnumSystemBase {
    pub base: ViewModelPropertyEnum,
    enum_type: u32,
}

impl Default for ViewModelPropertyEnumSystemBase {
    fn default() -> Self {
        Self {
            base: ViewModelPropertyEnum::default(),
            enum_type: 0,
        }
    }
}

impl ViewModelPropertyEnumSystemBase {
    pub const TYPE_KEY: u16 = 511;
    pub const ENUM_TYPE_PROPERTY_KEY: u16 = 708;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 509 | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn enum_type(&self) -> u32 {
        self.enum_type
    }
    pub fn set_enum_type(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyEnumSystemBaseCallbacks,
    ) {
        if !self.set_enum_type_value(value) {
            return;
        }
        callbacks.enum_type_changed();
        callbacks.notify_property_changed(Self::ENUM_TYPE_PROPERTY_KEY);
    }

    pub(crate) fn set_enum_type_value(&mut self, value: u32) -> bool {
        if self.enum_type == value {
            return false;
        }
        self.enum_type = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelPropertyEnumSystemBaseCallbacks,
    ) -> ViewModelPropertyEnumSystem {
        let mut cloned = ViewModelPropertyEnumSystem::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelPropertyEnumSystemBaseCallbacks,
    ) {
        self.enum_type = object.enum_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelPropertyEnumSystemBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ENUM_TYPE_PROPERTY_KEY => {
                self.enum_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelPropertyEnumSystemBase {
    type Target = ViewModelPropertyEnum;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyEnumSystemBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
