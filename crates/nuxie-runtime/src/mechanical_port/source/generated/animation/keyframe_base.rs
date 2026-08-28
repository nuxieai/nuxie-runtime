use crate::mechanical_port::source::{core::binary_reader::BinaryReader, core::Core};

pub trait KeyFrameBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn frame_changed(&mut self) {}
}

pub struct KeyFrameBase {
    pub base: Core,
    frame: u32,
}

impl Default for KeyFrameBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            frame: 0,
        }
    }
}

impl KeyFrameBase {
    pub const TYPE_KEY: u16 = 29;
    pub const FRAME_PROPERTY_KEY: u16 = 67;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn frame(&self) -> u32 {
        self.frame
    }
    pub fn set_frame(&mut self, value: u32, callbacks: &mut impl KeyFrameBaseCallbacks) {
        if !self.set_frame_value(value) {
            return;
        }
        callbacks.frame_changed();
        callbacks.notify_property_changed(Self::FRAME_PROPERTY_KEY);
    }

    pub(crate) fn set_frame_value(&mut self, value: u32) -> bool {
        if self.frame == value {
            return false;
        }
        self.frame = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl KeyFrameBaseCallbacks) {
        self.frame = object.frame;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl KeyFrameBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FRAME_PROPERTY_KEY => {
                self.frame = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for KeyFrameBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
