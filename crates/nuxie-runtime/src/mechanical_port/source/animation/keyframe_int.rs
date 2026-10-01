use crate::mechanical_port::source::{
    animation::{interpolating_keyframe::KeyFrameValueContext, keyframe::KeyFrame},
    generated::{
        animation::keyframe_int_base::KeyFrameIntBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};

#[derive(Default)]
pub struct KeyFrameInt {
    pub base: KeyFrameIntBase,
}

impl KeyFrameInt {
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
    ) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_with_completion(object, property_key, _mix, _context, &mut completion);
        completion.finish();
    }
    pub fn apply_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        CoreRegistry::set_int_with_completion(object, property_key, self.base.value(), completion);
    }

    pub fn apply_interpolation(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _current_time: f32,
        _next_frame: &KeyFrame,
        _mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
    ) {
        let mut completion = crate::source::core::PropertySetterCompletion::default();
        self.apply_interpolation_with_completion(
            object,
            property_key,
            _current_time,
            _next_frame,
            _mix,
            _context,
            &mut completion,
        );
        completion.finish();
    }
    pub fn apply_interpolation_with_completion(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _current_time: f32,
        _next_frame: &KeyFrame,
        _mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
        completion: &mut crate::source::core::PropertySetterCompletion,
    ) {
        CoreRegistry::set_int_with_completion(object, property_key, self.base.value(), completion);
    }
}
