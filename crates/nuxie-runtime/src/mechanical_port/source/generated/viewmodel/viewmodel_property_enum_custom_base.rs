use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property_enum::ViewModelPropertyEnum,
};

pub trait ViewModelPropertyEnumCustomBaseCallbacks {
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
        matches!(type_key, Self::TYPE_KEY | 0 | 0 | 0)
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
        if self.enum_id == value {
            return;
        }
        self.enum_id = value;
        callbacks.enum_id_changed();
        callbacks.notify_property_changed(Self::ENUM_ID_PROPERTY_KEY);
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
