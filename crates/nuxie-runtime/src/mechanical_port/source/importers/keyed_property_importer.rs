use std::any::Any;

use crate::mechanical_port::source::animation::{
    keyed_property::KeyedProperty, keyframe::KeyFrame, linear_animation::LinearAnimation,
};
use crate::mechanical_port::source::core::CoreHandle;

use super::import_stack::ImportStackObject;

pub struct KeyedPropertyImporter {
    animation: CoreHandle,
    keyed_property: CoreHandle,
}

impl KeyedPropertyImporter {
    pub fn new(animation: CoreHandle, keyed_property: CoreHandle) -> Self {
        Self {
            animation,
            keyed_property,
        }
    }

    pub fn add_key_frame(&mut self, owner: CoreHandle, key_frame: &mut KeyFrame) {
        let fps = self
            .animation
            .with_downcast::<LinearAnimation, _>(|animation| animation.base.fps())
            .expect("KeyedPropertyImporter retains a LinearAnimation");
        // KeyFrame::import already borrows this same occurrence. Use its actual
        // embedded base for computeSeconds before transferring its handle to
        // the property, matching upstream without borrowing the arena slot again.
        key_frame.compute_seconds(fps as i32);
        self.keyed_property
            .with_downcast_mut::<KeyedProperty, _>(|property| property.add_key_frame(owner))
            .expect("KeyedPropertyImporter retains a KeyedProperty");
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
