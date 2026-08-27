use crate::mechanical_port::source::{core::binary_reader::BinaryReader, key_frame::KeyFrame};

pub struct KeyFrameCallbackBase {
    pub base: KeyFrame,
}

impl Default for KeyFrameCallbackBase {
    fn default() -> Self {
        Self {
            base: KeyFrame::default(),
        }
    }
}

impl KeyFrameCallbackBase {
    pub const TYPE_KEY: u16 = 171;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
