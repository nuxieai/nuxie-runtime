use crate::mechanical_port::source::{
    animation::{interpolating_keyframe::KeyFrameValueContext, keyframe::KeyFrame},
    generated::{
        animation::keyframe_uint_base::KeyFrameUintBase,
        core_registry::{CoreRegistry, CoreRegistryObject},
    },
};

#[derive(Default)]
pub struct KeyFrameUint {
    pub base: KeyFrameUintBase,
}

impl KeyFrameUint {
    pub fn apply(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _mix: f32,
        _context: Option<&dyn KeyFrameValueContext>,
    ) {
        CoreRegistry::set_uint(object, property_key, self.base.value());
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
        CoreRegistry::set_uint(object, property_key, self.base.value());
    }
}
