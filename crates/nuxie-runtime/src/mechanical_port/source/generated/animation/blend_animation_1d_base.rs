use crate::mechanical_port::source::animation::blend_animation_1d::BlendAnimation1D;

use crate::mechanical_port::source::{
    animation::blend_animation::BlendAnimation, core::binary_reader::BinaryReader,
};

pub trait BlendAnimation1DBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct BlendAnimation1DBase {
    pub base: BlendAnimation,
    value: f32,
}

impl Default for BlendAnimation1DBase {
    fn default() -> Self {
        Self {
            base: BlendAnimation::default(),
            value: 0.0,
        }
    }
}

impl BlendAnimation1DBase {
    pub const TYPE_KEY: u16 = 75;
    pub const VALUE_PROPERTY_KEY: u16 = 166;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 74)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32, callbacks: &mut impl BlendAnimation1DBaseCallbacks) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl BlendAnimation1DBaseCallbacks,
    ) -> BlendAnimation1D {
        let mut cloned = BlendAnimation1D::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl BlendAnimation1DBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl BlendAnimation1DBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
