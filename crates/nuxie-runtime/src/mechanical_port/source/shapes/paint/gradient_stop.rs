use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::gradient_stop_base::GradientStopBase,
    shapes::paint::linear_gradient::LinearGradient,
};
pub struct GradientStop {
    pub base: GradientStopBase,
}
impl GradientStop {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.base.parent().is::<LinearGradient>() {
            return StatusCode::MissingObject;
        }
        let this = self as *mut _;
        self.base
            .parent_mut()
            .as_mut::<LinearGradient>()
            .unwrap()
            .add_stop(unsafe { &mut *this });
        StatusCode::Ok
    }
    pub fn color_value_changed(&mut self) {
        self.base
            .parent_mut()
            .as_mut::<LinearGradient>()
            .unwrap()
            .mark_gradient_dirty();
    }
    pub fn position_changed(&mut self) {
        self.base
            .parent_mut()
            .as_mut::<LinearGradient>()
            .unwrap()
            .mark_stops_dirty();
    }
}
