use std::mem::MaybeUninit;

use crate::mechanical_port::source::{
    animation::keyed_property::KeyedProperty,
    generated::animation::keyframe_base::KeyFrameBase,
    importers::{import_stack::ImportStack, keyed_property_importer::KeyedPropertyImporter},
    status_code::StatusCode,
};

pub struct KeyFrame {
    pub base: KeyFrameBase,
    seconds: MaybeUninit<f32>,
}

impl Default for KeyFrame {
    fn default() -> Self {
        Self {
            base: KeyFrameBase::default(),
            seconds: MaybeUninit::uninit(),
        }
    }
}

impl KeyFrame {
    pub fn seconds(&self) -> f32 {
        unsafe { self.seconds.assume_init() }
    }

    pub fn compute_seconds(&mut self, fps: i32) {
        self.seconds.write(self.base.frame() as f32 / fps as f32);
    }

    pub fn import(self: Box<Self>, import_stack: &mut ImportStack) -> StatusCode {
        let object = Box::into_raw(self);
        let Some(importer) = import_stack.latest::<KeyedPropertyImporter>(KeyedProperty::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(object)) };
            return StatusCode::MissingObject;
        };
        importer.add_key_frame(unsafe { Box::from_raw(object) });
        unsafe { (*object).base.base.import(import_stack) }
    }
}
