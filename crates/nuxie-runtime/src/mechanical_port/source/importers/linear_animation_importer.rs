use std::any::Any;

use crate::mechanical_port::source::{
    animation::linear_animation::LinearAnimation, core::CoreHandle,
};

use super::import_stack::ImportStackObject;

pub struct LinearAnimationImporter {
    animation: CoreHandle,
}

impl LinearAnimationImporter {
    pub fn new(animation: CoreHandle) -> Self {
        animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| {
                animation.set_has_keyed_callbacks(false);
            })
            .expect("LinearAnimationImporter retains a LinearAnimation");
        Self { animation }
    }

    pub fn animation(&self) -> CoreHandle {
        self.animation.clone()
    }

    pub fn add_keyed_object(&mut self, object: CoreHandle) {
        self.animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| animation.add_keyed_object(object))
            .expect("LinearAnimationImporter retains a LinearAnimation");
    }

    /// Add an importing KeyedObject along with its shared cells.
    pub(crate) fn add_keyed_object_with_cells(
        &mut self,
        object: CoreHandle,
        cells: std::rc::Rc<crate::mechanical_port::source::animation::keyframe_cells::KeyedObjectCells>,
    ) {
        self.animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| {
                animation.add_keyed_object_with_cells(object, Some(cells))
            })
            .expect("LinearAnimationImporter retains a LinearAnimation");
    }
}

impl ImportStackObject for LinearAnimationImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
