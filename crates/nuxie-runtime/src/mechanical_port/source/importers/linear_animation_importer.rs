use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::animation::{
    keyed_object::KeyedObject, linear_animation::LinearAnimation,
};

use super::import_stack::ImportStackObject;

pub struct LinearAnimationImporter {
    animation: NonNull<LinearAnimation>,
}

impl LinearAnimationImporter {
    pub fn new(animation: NonNull<LinearAnimation>) -> Self {
        Self { animation }
    }

    pub fn animation(&self) -> NonNull<LinearAnimation> {
        self.animation
    }

    pub fn add_keyed_object(&mut self, object: Box<KeyedObject>) {
        unsafe { self.animation.as_mut().add_keyed_object(object) };
    }
}

impl ImportStackObject for LinearAnimationImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
