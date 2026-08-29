use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
    custom_property::CustomProperty,
    custom_property_number::CustomPropertyNumber,
};

pub trait CustomPropertyNumberBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn property_value_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}
pub struct CustomPropertyNumberBase {
    pub base: CustomProperty,
    property_value: f32,
}
impl Default for CustomPropertyNumberBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: 0.0,
        }
    }
}
impl CustomPropertyNumberBase {
    pub const TYPE_KEY: u16 = 127;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 243;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> f32 {
        self.property_value
    }
    pub fn set_property_value<C: CustomPropertyNumberBaseCallbacks>(
        &mut self,
        value: f32,
        c: &mut C,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        c.property_value_changed();
        CustomPropertyNumberBaseCallbacks::notify_property_changed(
            c,
            Self::PROPERTY_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_property_value_value(&mut self, value: f32) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn copy<C: CustomPropertyNumberBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.property_value = object.property_value;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: CustomPropertyNumberBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: CustomPropertyNumberBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> CustomPropertyNumber {
        let mut cloned = CustomPropertyNumber::default();
        cloned.base.copy(self, c);
        cloned
    }
}

impl std::ops::Deref for CustomPropertyNumberBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyNumberBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
