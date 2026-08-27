use crate::mechanical_port::source::generated::animation::blend_animation_1d_base::BlendAnimation1DBase;
use crate::mechanical_port::source::{core_context::CoreContext, status_code::StatusCode};

#[derive(Default)]
pub struct BlendAnimation1D {
    pub base: BlendAnimation1DBase,
}

impl BlendAnimation1D {
    pub fn on_added_dirty(&mut self, _context: &mut CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        StatusCode::Ok
    }
}
