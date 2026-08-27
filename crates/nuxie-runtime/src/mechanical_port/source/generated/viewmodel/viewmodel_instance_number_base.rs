use crate::mechanical_port::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_value::ViewModelInstanceValue,
};

pub trait ViewModelInstanceNumberBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct ViewModelInstanceNumberBase {
    pub base: ViewModelInstanceValue,
    property_value: f32,
}

impl Default for ViewModelInstanceNumberBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValue::default(),
            property_value: 0.0,
        }
    }
}

impl ViewModelInstanceNumberBase {
    pub const TYPE_KEY: u16 = 442;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 575;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> f32 {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: f32,
        callbacks: &mut impl ViewModelInstanceNumberBaseCallbacks,
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
        callbacks: &mut impl ViewModelInstanceNumberBaseCallbacks,
    ) -> ViewModelInstanceNumber {
        let mut cloned = ViewModelInstanceNumber::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelInstanceNumberBaseCallbacks,
    ) {
        self.property_value = object.property_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceNumberBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
