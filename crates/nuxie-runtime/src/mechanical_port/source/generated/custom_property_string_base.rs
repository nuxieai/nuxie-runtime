use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_string_type::CoreStringType},
    custom_property::CustomProperty,
    custom_property_string::CustomPropertyString,
};

pub trait CustomPropertyStringBaseCallbacks {
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
        if self.property_value == value {
            return;
        }
        self.property_value = value;
        c.property_value_changed();
        c.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
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
