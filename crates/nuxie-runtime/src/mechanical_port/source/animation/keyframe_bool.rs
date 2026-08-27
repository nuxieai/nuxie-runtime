use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    generated::{
        animation::keyframe_bool_base::KeyFrameBoolBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};
#[derive(Default)]
pub struct KeyFrameBool {
    pub base: KeyFrameBoolBase,
}
impl KeyFrameBool {
    pub fn effective_value(&self, context: Option<&dyn KeyFrameValueContext>) -> bool {
        context
            .and_then(|c| c.bool_value(self as *const Self as *const ()))
            .unwrap_or_else(|| self.base.value())
    }
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        CoreRegistry::set_bool(object, key, self.effective_value(context));
    }
    pub fn apply_interpolation(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _seconds: f32,
        _next: &Self,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        CoreRegistry::set_bool(object, key, self.effective_value(context));
    }
}
