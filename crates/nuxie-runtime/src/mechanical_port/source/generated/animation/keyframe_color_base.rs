use crate::mechanical_port::source::animation::keyframe_color::KeyFrameColor;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, animation::interpolating_keyframe::InterpolatingKeyFrame,
};

pub trait KeyFrameColorBaseCallbacks: crate::mechanical_port::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct KeyFrameColorBase {
    pub base: InterpolatingKeyFrame,
    value: i32,
}

impl Default for KeyFrameColorBase {
    fn default() -> Self {
        Self {
            base: InterpolatingKeyFrame::default(),
            value: 0,
        }
    }
}

impl KeyFrameColorBase {
    pub const TYPE_KEY: u16 = 37;
    pub const VALUE_PROPERTY_KEY: u16 = 88;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 170 | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn set_value(&mut self, value: i32, callbacks: &mut impl KeyFrameColorBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: i32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyFrameColorBaseCallbacks) -> KeyFrameColor {
        let mut cloned = KeyFrameColor::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameColorBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameColorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for KeyFrameColorBase {
    type Target = InterpolatingKeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameColorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
