use crate::mechanical_port::source::viewmodel::viewmodel_instance_color::ViewModelInstanceColor;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_instance_value::ViewModelInstanceValue,
};

pub trait ViewModelInstanceColorBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct ViewModelInstanceColorBase {
    pub base: ViewModelInstanceValue,
    property_value: i32,
}

impl Default for ViewModelInstanceColorBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValue::default(),
            property_value: 0xFF000000u32 as i32,
        }
    }
}

impl ViewModelInstanceColorBase {
    pub const TYPE_KEY: u16 = 426;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 555;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> i32 {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: i32,
        callbacks: &mut impl ViewModelInstanceColorBaseCallbacks,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        callbacks.property_value_changed();
        ViewModelInstanceColorBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PROPERTY_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_property_value_value(&mut self, value: i32) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelInstanceColorBaseCallbacks,
    ) -> ViewModelInstanceColor {
        let mut cloned = ViewModelInstanceColor::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelInstanceColorBaseCallbacks,
    ) {
        self.property_value = object.property_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceColorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelInstanceColorBase {
    type Target = ViewModelInstanceValue;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceColorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
