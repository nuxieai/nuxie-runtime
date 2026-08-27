use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::animation::{
    keyed_property::KeyedProperty, keyframe::KeyFrame, linear_animation::LinearAnimation,
};

use super::import_stack::ImportStackObject;

pub struct KeyedPropertyImporter {
    animation: NonNull<LinearAnimation>,
    keyed_property: NonNull<KeyedProperty>,
}

impl KeyedPropertyImporter {
    pub fn new(
        animation: NonNull<LinearAnimation>,
        keyed_property: NonNull<KeyedProperty>,
    ) -> Self {
        Self {
            animation,
            keyed_property,
        }
    }

    pub fn add_key_frame(&mut self, mut key_frame: Box<KeyFrame>) {
        let fps = unsafe { self.animation.as_ref().fps() };
        key_frame.compute_seconds(fps);
        unsafe { self.keyed_property.as_mut().add_key_frame(key_frame) };
    }
}

impl ImportStackObject for KeyedPropertyImporter {
    fn read_null_object(&mut self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
