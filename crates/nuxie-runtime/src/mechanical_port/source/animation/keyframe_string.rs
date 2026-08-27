use crate::mechanical_port::source::{
    animation::interpolating_keyframe::KeyFrameValueContext,
    generated::{
        animation::keyframe_string_base::KeyFrameStringBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};
#[derive(Default)]
pub struct KeyFrameString {
    pub base: KeyFrameStringBase,
}
impl KeyFrameString {
    pub fn effective_value<'a>(&'a self, context: Option<&'a dyn KeyFrameValueContext>) -> &'a str {
        context
            .and_then(|c| c.string_value(self as *const Self as *const ()))
            .unwrap_or_else(|| self.base.value())
    }
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        CoreRegistry::set_string(object, key, self.effective_value(context).to_owned());
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
        CoreRegistry::set_string(object, key, self.effective_value(context).to_owned());
    }
}
