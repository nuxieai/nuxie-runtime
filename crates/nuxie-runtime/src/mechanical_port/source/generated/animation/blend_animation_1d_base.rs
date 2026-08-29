use crate::mechanical_port::source::animation::blend_animation_1d::BlendAnimation1D;

use crate::mechanical_port::source::{
    animation::blend_animation::BlendAnimation, core::binary_reader::BinaryReader,
};

pub trait BlendAnimation1DBaseCallbacks: crate::mechanical_port::source::generated::animation::blend_animation_base::BlendAnimationBaseCallbacks {
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
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        BlendAnimation1DBaseCallbacks::notify_property_changed(callbacks, Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
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

impl std::ops::Deref for BlendAnimation1DBase {
    type Target = BlendAnimation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BlendAnimation1DBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
