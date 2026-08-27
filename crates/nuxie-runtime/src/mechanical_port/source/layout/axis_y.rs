use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::axis_y_base::AxisYBase,
    layout::n_slicer_details,
};

pub struct AxisY {
    pub base: AxisYBase,
}
impl AxisY {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        n_slicer_details::from(self.base.parent_mut())
            .unwrap()
            .add_axis_y(self.base.as_axis_mut_ptr());
        StatusCode::Ok
    }
}
