use crate::mechanical_port::source::{
    animation::{
        blend_state_1d_instance::BlendAnimation1DDefinition,
        blend_state_instance::BlendAnimationDefinition, linear_animation::LinearAnimation,
    },
    core_context::CoreContext,
    generated::animation::blend_animation_1d_base::BlendAnimation1DBase,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct BlendAnimation1D {
    pub base: BlendAnimation1DBase,
}

impl BlendAnimationDefinition for BlendAnimation1D {
    type Animation = LinearAnimation;

    fn animation(&self) -> &Self::Animation {
        self.base.base.animation()
    }
}

impl BlendAnimation1DDefinition for BlendAnimation1D {
    fn value(&self) -> f32 {
        self.base.value()
    }
}

impl BlendAnimation1D {
    pub fn on_added_dirty(&mut self, _context: &mut CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        StatusCode::Ok
    }
}
