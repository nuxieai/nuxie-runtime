use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::axis_base::AxisBase,
    layout::n_slicer_details,
};

#[repr(i32)]
pub enum AxisType {
    X = 0,
    Y = 1,
}

pub struct Axis {
    pub base: AxisBase,
}

impl Axis {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if n_slicer_details::from(self.base.parent_mut()).is_none() {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn offset_changed(&mut self) {
        if let Some(details) = n_slicer_details::from(self.base.parent_mut()) {
            details.axis_changed();
        }
    }
}
