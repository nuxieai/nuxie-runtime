use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::bindable_property::BindableProperty,
    data_bind::bindable_property_string::BindablePropertyString,
};

pub trait BindablePropertyStringBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct BindablePropertyStringBase {
    pub base: BindableProperty,
    property_value: String,
}

impl Default for BindablePropertyStringBase {
    fn default() -> Self {
        Self {
            base: BindableProperty::default(),
            property_value: "".to_owned(),
        }
    }
}

impl BindablePropertyStringBase {
    pub const TYPE_KEY: u16 = 471;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 635;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 9)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> &str {
        &self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: String,
        callbacks: &mut impl BindablePropertyStringBaseCallbacks,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        callbacks.property_value_changed();
        callbacks.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_property_value_value(&mut self, value: String) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BindablePropertyStringBaseCallbacks,
    ) -> BindablePropertyString {
        let mut cloned = BindablePropertyString::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl BindablePropertyStringBaseCallbacks,
    ) {
        self.property_value.clone_from(&object.property_value);
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BindablePropertyStringBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for BindablePropertyStringBase {
    type Target = BindableProperty;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BindablePropertyStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
