use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

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
        if self.frame == value {
            return;
        }
        self.frame = value;
        callbacks.frame_changed();
        callbacks.notify_property_changed(Self::FRAME_PROPERTY_KEY);
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
