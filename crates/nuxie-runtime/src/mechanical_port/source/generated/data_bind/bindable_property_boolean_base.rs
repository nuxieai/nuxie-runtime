use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::bindable_property::BindableProperty,
    data_bind::bindable_property_boolean::BindablePropertyBoolean,
};

pub trait BindablePropertyBooleanBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct BindablePropertyBooleanBase {
    pub base: BindableProperty,
    property_value: bool,
}

impl Default for BindablePropertyBooleanBase {
    fn default() -> Self {
        Self {
            base: BindableProperty::default(),
            property_value: false,
        }
    }
}

impl BindablePropertyBooleanBase {
    pub const TYPE_KEY: u16 = 472;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 634;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 9)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> bool {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: bool,
        callbacks: &mut impl BindablePropertyBooleanBaseCallbacks,
    ) {
        if self.property_value == value {
            return;
        }
        self.property_value = value;
        callbacks.property_value_changed();
        callbacks.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BindablePropertyBooleanBaseCallbacks,
    ) -> BindablePropertyBoolean {
        let mut cloned = BindablePropertyBoolean::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl BindablePropertyBooleanBaseCallbacks,
    ) {
        self.property_value = object.property_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BindablePropertyBooleanBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
