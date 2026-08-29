use crate::mechanical_port::source::animation::keyframe_int::KeyFrameInt;

use crate::mechanical_port::source::{
    animation::interpolating_keyframe::InterpolatingKeyFrame, core::binary_reader::BinaryReader,
};

pub trait KeyFrameIntBaseCallbacks: crate::mechanical_port::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct KeyFrameIntBase {
    pub base: InterpolatingKeyFrame,
    value: i32,
}

impl Default for KeyFrameIntBase {
    fn default() -> Self {
        Self {
            base: InterpolatingKeyFrame::default(),
            value: 0,
        }
    }
}

impl KeyFrameIntBase {
    pub const TYPE_KEY: u16 = 1067;
    pub const VALUE_PROPERTY_KEY: u16 = 1068;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 170 | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn set_value(&mut self, value: i32, callbacks: &mut impl KeyFrameIntBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        KeyFrameIntBaseCallbacks::notify_property_changed(callbacks, Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: i32) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyFrameIntBaseCallbacks) -> KeyFrameInt {
        let mut cloned = KeyFrameInt::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameIntBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameIntBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_int_type::CoreIntType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for KeyFrameIntBase {
    type Target = InterpolatingKeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameIntBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
