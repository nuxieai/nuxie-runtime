use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::axis_x_base::AxisXBase,
    layout::n_slicer_details,
};

impl std::ops::Deref for AxisX {
    type Target = AxisXBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AxisX {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl AxisX {
    pub const TYPE_KEY: u16 = AxisXBase::TYPE_KEY;
}

#[derive(Default)]
pub struct AxisX {
    pub base: AxisXBase,
}
impl AxisX {
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
        let added = n_slicer_details::add_axis_x(&parent, this);
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
