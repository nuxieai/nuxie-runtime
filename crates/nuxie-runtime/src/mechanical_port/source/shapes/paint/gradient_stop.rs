use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::gradient_stop_base::GradientStopBase,
    shapes::paint::linear_gradient::LinearGradient,
};
pub struct GradientStop {
    pub base: GradientStopBase,
}
impl GradientStop {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(this), Some(parent)) = (self.base.handle(), self.base.parent_handle()) else {
            return StatusCode::MissingObject;
        };
        if parent
            .with_downcast_mut::<LinearGradient, _>(|gradient| gradient.add_stop(this))
            .is_none()
        {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn color_value_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_downcast_mut::<LinearGradient, _>(LinearGradient::mark_gradient_dirty);
        }
    }
    pub fn position_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_downcast_mut::<LinearGradient, _>(LinearGradient::mark_stops_dirty);
        }
    }
}
