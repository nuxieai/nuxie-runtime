use crate::mechanical_port::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_instance_value::ViewModelInstanceValue,
};

pub trait ViewModelInstanceNumberBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBaseCallbacks {
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
        if !self.set_property_value_value(value) {
            return;
        }
        callbacks.property_value_changed();
        ViewModelInstanceNumberBaseCallbacks::notify_property_changed(
            callbacks,
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

impl std::ops::Deref for ViewModelInstanceNumberBase {
    type Target = ViewModelInstanceValue;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceNumberBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
