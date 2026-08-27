use crate::mechanical_port::source::{
    generated::animation::keyframe_interpolator_base::KeyFrameInterpolatorBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub trait InterpolatorHost {
    fn overrides_keyed_interpolation(&mut self, property_key: i32) -> bool;
}
pub trait KeyFrameInterpolatorBehavior {
    fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32;
    fn transform(&self, factor: f32) -> f32;
    fn initialize(&mut self) {}
    fn is_scripted(&self) -> bool {
        false
    }
}
#[derive(Default)]
pub struct KeyFrameInterpolator {
    pub base: KeyFrameInterpolatorBase,
    scripted: bool,
}
impl KeyFrameInterpolator {
    pub fn is_scripted(&self) -> bool {
        self.scripted
    }
    pub fn set_scripted(&mut self, value: bool) {
        self.scripted = value;
    }
    pub fn host_from(
        component_type: u16,
        layout_host: Option<NonNull<dyn InterpolatorHost>>,
    ) -> Option<NonNull<dyn InterpolatorHost>> {
        if component_type == crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY { layout_host } else { None }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let pointer = NonNull::from(&mut *self);
        if let Some(importer) = stack.latest::<ArtboardImporter>(
            crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
        ) {
            importer.add_component(Some(pointer.cast()));
        } else if let Some(importer) = stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) {
            importer.add_interpolator(pointer);
        } else {
            return StatusCode::MissingObject;
        }
        self.base.base.import(stack)
    }
    pub fn initialize(&mut self) {}
}
