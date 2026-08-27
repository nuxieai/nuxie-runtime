use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, interpolating_key_frame::InterpolatingKeyFrame,
};

pub trait KeyFrameBoolBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn value_changed(&mut self) {}
}

pub struct KeyFrameBoolBase {
    pub base: InterpolatingKeyFrame,
    value: bool,
}

impl Default for KeyFrameBoolBase {
    fn default() -> Self {
        Self {
            base: InterpolatingKeyFrame::default(),
            value: false,
        }
    }
}

impl KeyFrameBoolBase {
    pub const TYPE_KEY: u16 = 84;
    pub const VALUE_PROPERTY_KEY: u16 = 181;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn value(&self) -> bool {
        self.value
    }
    pub fn set_value(&mut self, value: bool, callbacks: &mut impl KeyFrameBoolBaseCallbacks) {
        if self.value == value {
            return;
        }
        self.value = value;
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameBoolBaseCallbacks) {
        self.value = object.value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameBoolBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
