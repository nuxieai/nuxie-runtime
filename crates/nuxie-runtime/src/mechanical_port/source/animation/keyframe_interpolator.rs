use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::animation::keyframe_interpolator_base::KeyFrameInterpolatorBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    status_code::StatusCode,
};
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
    pub fn host_from(component: CoreHandle) -> Option<CoreHandle> {
        component
            .is_type_of(
                crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
            )
            .then_some(component)
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(handle) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        if let Some(importer) = stack.latest::<ArtboardImporter>(
            crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
        ) {
            importer.add_component(Some(handle));
        } else if let Some(importer) = stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) {
            importer.add_interpolator(handle);
        } else {
            return StatusCode::MissingObject;
        }
        self.base.base.import(stack)
    }
    pub fn initialize(&mut self) {}
}
impl std::ops::Deref for KeyFrameInterpolator {
    type Target = KeyFrameInterpolatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for KeyFrameInterpolator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
