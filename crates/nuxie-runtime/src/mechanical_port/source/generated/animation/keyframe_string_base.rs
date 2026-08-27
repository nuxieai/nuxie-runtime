use crate::mechanical_port::source::animation::keyframe_string::KeyFrameString;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, interpolating_key_frame::InterpolatingKeyFrame,
};

pub trait KeyFrameStringBaseCallbacks {
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
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
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
