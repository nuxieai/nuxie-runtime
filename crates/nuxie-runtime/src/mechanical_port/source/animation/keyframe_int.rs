use crate::mechanical_port::source::{
    animation::{keyframe::KeyFrame, linear_animation_instance::LinearAnimationInstance},
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
        _context: Option<&LinearAnimationInstance>,
    ) {
        CoreRegistry::set_int(object, property_key, self.base.value());
    }

    pub fn apply_interpolation(
        &self,
        object: &mut dyn CoreRegistryObject,
        property_key: i32,
        _current_time: f32,
        _next_frame: &KeyFrame,
        _mix: f32,
        _context: Option<&LinearAnimationInstance>,
    ) {
        CoreRegistry::set_int(object, property_key, self.base.value());
    }
}
