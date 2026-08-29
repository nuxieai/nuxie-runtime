use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::{
        component_base::ComponentBaseCallbacks,
        layout::axis_base::{AxisBase, AxisBaseCallbacks},
    },
    layout::n_slicer_details,
};

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
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

impl std::ops::Deref for Axis {
    type Target = AxisBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Axis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Axis {
    pub const TYPE_KEY: u16 = AxisBase::TYPE_KEY;
}

#[derive(Default)]
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
