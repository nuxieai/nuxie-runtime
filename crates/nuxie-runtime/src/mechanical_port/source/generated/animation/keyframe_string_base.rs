use crate::mechanical_port::source::animation::keyframe_string::KeyFrameString;

use crate::mechanical_port::source::{
    animation::interpolating_keyframe::InterpolatingKeyFrame, core::binary_reader::BinaryReader,
};

pub trait KeyFrameStringBaseCallbacks: crate::mechanical_port::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct KeyFrameStringBase {
    pub base: InterpolatingKeyFrame,
    value: String,
}

impl Default for KeyFrameStringBase {
    fn default() -> Self {
        Self {
            base: InterpolatingKeyFrame::default(),
            value: "".to_owned(),
        }
    }
}

impl KeyFrameStringBase {
    pub const TYPE_KEY: u16 = 142;
    pub const VALUE_PROPERTY_KEY: u16 = 280;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 170 | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, value: String, callbacks: &mut impl KeyFrameStringBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        KeyFrameStringBaseCallbacks::notify_property_changed(callbacks, Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: String) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl KeyFrameStringBaseCallbacks) -> KeyFrameString {
        let mut cloned = KeyFrameString::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameStringBaseCallbacks) {
        self.value.clone_from(&object.value);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameStringBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for KeyFrameStringBase {
    type Target = InterpolatingKeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
