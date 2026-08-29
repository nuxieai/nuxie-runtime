use crate::mechanical_port::source::viewmodel::viewmodel_property_enum_custom::ViewModelPropertyEnumCustom;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_property_enum::ViewModelPropertyEnum,
};

pub trait ViewModelPropertyEnumCustomBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_property_base::ViewModelPropertyBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn enum_id_changed(&mut self) {}
}

pub struct ViewModelPropertyEnumCustomBase {
    pub base: ViewModelPropertyEnum,
    enum_id: u32,
}

impl Default for ViewModelPropertyEnumCustomBase {
    fn default() -> Self {
        Self {
            base: ViewModelPropertyEnum::default(),
            enum_id: u32::MAX,
        }
    }
}

impl ViewModelPropertyEnumCustomBase {
    pub const TYPE_KEY: u16 = 439;
    pub const ENUM_ID_PROPERTY_KEY: u16 = 574;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 509 | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn enum_id(&self) -> u32 {
        self.enum_id
    }
    pub fn set_enum_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyEnumCustomBaseCallbacks,
    ) {
        if !self.set_enum_id_value(value) {
            return;
        }
        callbacks.enum_id_changed();
        ViewModelPropertyEnumCustomBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ENUM_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_enum_id_value(&mut self, value: u32) -> bool {
        if self.enum_id == value {
            return false;
        }
        self.enum_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelPropertyEnumCustomBaseCallbacks,
    ) -> ViewModelPropertyEnumCustom {
        let mut cloned = ViewModelPropertyEnumCustom::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelPropertyEnumCustomBaseCallbacks,
    ) {
        self.enum_id = object.enum_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelPropertyEnumCustomBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ENUM_ID_PROPERTY_KEY => {
                self.enum_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelPropertyEnumCustomBase {
    type Target = ViewModelPropertyEnum;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyEnumCustomBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
