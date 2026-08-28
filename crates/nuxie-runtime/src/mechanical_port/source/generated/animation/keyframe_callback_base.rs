use crate::mechanical_port::source::animation::keyframe_callback::KeyFrameCallback;

use crate::mechanical_port::source::{core::binary_reader::BinaryReader, animation::keyframe::KeyFrame};

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
        matches!(type_key, Self::TYPE_KEY | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> KeyFrameCallback {
        let mut cloned = KeyFrameCallback::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for KeyFrameCallbackBase {
    type Target = KeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for KeyFrameCallbackBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
