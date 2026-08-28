use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, shapes::paint::solid_color::SolidColor,
};

pub trait SolidColorBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn color_value_changed(&mut self) {}
}

pub struct SolidColorBase {
    pub base: Component,
    color_value: i32,
}

impl Default for SolidColorBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            color_value: 0xFF747474u32 as i32,
        }
    }
}

impl SolidColorBase {
    pub const TYPE_KEY: u16 = 18;
    pub const COLOR_VALUE_PROPERTY_KEY: u16 = 37;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn color_value(&self) -> i32 {
        self.color_value
    }
    pub fn set_color_value(&mut self, value: i32, callbacks: &mut impl SolidColorBaseCallbacks) {
        if !self.set_color_value_value(value) {
            return;
        }
        callbacks.color_value_changed();
        callbacks.notify_property_changed(Self::COLOR_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_color_value_value(&mut self, value: i32) -> bool {
        if self.color_value == value {
            return false;
        }
        self.color_value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl SolidColorBaseCallbacks) -> SolidColor {
        let mut cloned = SolidColor::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl SolidColorBaseCallbacks) {
        self.color_value = object.color_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl SolidColorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::COLOR_VALUE_PROPERTY_KEY => {
                self.color_value = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for SolidColorBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SolidColorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
