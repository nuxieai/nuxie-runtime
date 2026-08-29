use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_bool_type::CoreBoolType},
    custom_property::CustomProperty,
    custom_property_boolean::CustomPropertyBoolean,
};

pub trait CustomPropertyBooleanBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn property_value_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}
pub struct CustomPropertyBooleanBase {
    pub base: CustomProperty,
    property_value: bool,
}
impl Default for CustomPropertyBooleanBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: false,
        }
    }
}
impl CustomPropertyBooleanBase {
    pub const TYPE_KEY: u16 = 129;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 245;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> bool {
        self.property_value
    }
    pub fn set_property_value<C: CustomPropertyBooleanBaseCallbacks>(
        &mut self,
        value: bool,
        c: &mut C,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        c.property_value_changed();
        CustomPropertyBooleanBaseCallbacks::notify_property_changed(
            c,
            Self::PROPERTY_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_property_value_value(&mut self, value: bool) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn copy<C: CustomPropertyBooleanBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.property_value = object.property_value;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: CustomPropertyBooleanBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: CustomPropertyBooleanBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> CustomPropertyBoolean {
        let mut cloned = CustomPropertyBoolean::default();
        cloned.base.copy(self, c);
        cloned
    }
}

impl std::ops::Deref for CustomPropertyBooleanBase {
    type Target = CustomProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyBooleanBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
