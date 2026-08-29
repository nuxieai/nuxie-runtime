use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_string_type::CoreStringType},
    custom_property::CustomProperty,
    custom_property_string::CustomPropertyString,
};

pub trait CustomPropertyStringBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn property_value_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}
pub struct CustomPropertyStringBase {
    pub base: CustomProperty,
    property_value: String,
}
impl Default for CustomPropertyStringBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: String::new(),
        }
    }
}
impl CustomPropertyStringBase {
    pub const TYPE_KEY: u16 = 130;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 246;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> &str {
        &self.property_value
    }
    pub fn set_property_value<C: CustomPropertyStringBaseCallbacks>(
        &mut self,
        value: String,
        c: &mut C,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        c.property_value_changed();
        CustomPropertyStringBaseCallbacks::notify_property_changed(
            c,
            Self::PROPERTY_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_property_value_value(&mut self, value: String) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn copy<C: CustomPropertyStringBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.property_value.clone_from(&object.property_value);
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: CustomPropertyStringBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: CustomPropertyStringBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> CustomPropertyString {
        let mut cloned = CustomPropertyString::default();
        cloned.base.copy(self, c);
        cloned
    }
}

impl std::ops::Deref for CustomPropertyStringBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
