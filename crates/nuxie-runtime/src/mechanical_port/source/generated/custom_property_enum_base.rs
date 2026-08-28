use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, custom_property::CustomProperty,
    custom_property_enum::CustomPropertyEnum,
};

pub trait CustomPropertyEnumBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
    fn enum_id_changed(&mut self) {}
}

pub struct CustomPropertyEnumBase {
    pub base: CustomProperty,
    property_value: u32,
    enum_id: u32,
}

impl Default for CustomPropertyEnumBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: u32::MAX,
            enum_id: u32::MAX,
        }
    }
}

impl CustomPropertyEnumBase {
    pub const TYPE_KEY: u16 = 616;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 872;
    pub const ENUM_ID_PROPERTY_KEY: u16 = 873;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> u32 {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: u32,
        callbacks: &mut impl CustomPropertyEnumBaseCallbacks,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        callbacks.property_value_changed();
        callbacks.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_property_value_value(&mut self, value: u32) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn enum_id(&self) -> u32 {
        self.enum_id
    }
    pub fn set_enum_id(
        &mut self,
        value: u32,
        callbacks: &mut impl CustomPropertyEnumBaseCallbacks,
    ) {
        if !self.set_enum_id_value(value) {
            return;
        }
        callbacks.enum_id_changed();
        callbacks.notify_property_changed(Self::ENUM_ID_PROPERTY_KEY);
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
        callbacks: &mut impl CustomPropertyEnumBaseCallbacks,
    ) -> CustomPropertyEnum {
        let mut cloned = CustomPropertyEnum::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl CustomPropertyEnumBaseCallbacks) {
        self.property_value = object.property_value;
        self.enum_id = object.enum_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl CustomPropertyEnumBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ENUM_ID_PROPERTY_KEY => {
                self.enum_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for CustomPropertyEnumBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyEnumBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
