use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::axis_y_base::AxisYBase,
    layout::n_slicer_details,
};

impl std::ops::Deref for AxisY {
    type Target = AxisYBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AxisY {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl AxisY {
    pub const TYPE_KEY: u16 = AxisYBase::TYPE_KEY;
}

#[derive(Default)]
pub struct AxisY {
    pub base: AxisYBase,
}
impl AxisY {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::MissingObject;
        };
        let added = n_slicer_details::add_axis_y(&parent, this);
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
