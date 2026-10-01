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
    pub fn effective_value(&self, context: Option<&dyn KeyFrameValueContext>) -> String {
        context
            .and_then(|c| {
                self.base
                    .handle()
                    .and_then(|keyframe| c.string_value(&keyframe))
            })
            .unwrap_or_else(|| self.base.value().to_owned())
    }
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_with_completion(object, key, _mix, context, &mut completion);
        completion.finish();
    }
    pub fn apply_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        CoreRegistry::set_string_with_completion(
            object,
            key,
            self.effective_value(context),
            completion,
        );
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
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_interpolation_with_completion(
            object,
            key,
            _seconds,
            _next,
            _mix,
            context,
            &mut completion,
        );
        completion.finish();
    }
    pub fn apply_interpolation_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        key: i32,
        _seconds: f32,
        _next: &Self,
        _mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        CoreRegistry::set_string_with_completion(
            object,
            key,
            self.effective_value(context),
            completion,
        );
    }
}
