use crate::mechanical_port::source::animation::keyframe_double::KeyFrameDouble;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, interpolating_key_frame::InterpolatingKeyFrame,
};

pub trait KeyFrameDoubleBaseCallbacks: crate::mechanical_port::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct KeyFrameDoubleBase {
    pub base: InterpolatingKeyFrame,
    value: f32,
}

impl Default for KeyFrameDoubleBase {
    fn default() -> Self {
        Self {
            base: InterpolatingKeyFrame::default(),
            value: 0.0,
        }
    }
}

impl KeyFrameDoubleBase {
    pub const TYPE_KEY: u16 = 30;
    pub const VALUE_PROPERTY_KEY: u16 = 70;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 170 | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32, callbacks: &mut impl KeyFrameDoubleBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: f32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyFrameDoubleBaseCallbacks) -> KeyFrameDouble {
        let mut cloned = KeyFrameDouble::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameDoubleBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameDoubleBaseCallbacks,
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

impl std::ops::Deref for KeyFrameDoubleBase {
    type Target = InterpolatingKeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameDoubleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
