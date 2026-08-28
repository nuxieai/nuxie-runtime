use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::{
        component_base::ComponentBaseCallbacks,
        layout::axis_base::{AxisBase, AxisBaseCallbacks},
    },
    layout::n_slicer_details,
};

#[repr(i32)]
pub enum AxisType {
    X = 0,
    Y = 1,
}

impl AxisBaseCallbacks for Axis {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }

    fn offset_changed(&mut self) {
        Axis::offset_changed(self);
    }
}

impl ComponentBaseCallbacks for Axis {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

pub struct Axis {
    pub base: AxisBase,
}

impl Axis {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::MissingObject;
        };
        if !n_slicer_details::is_details(&parent) {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn offset_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            n_slicer_details::axis_changed(&parent);
        }
    }
}
