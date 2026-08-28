use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::dash_base::DashBase,
    shapes::paint::dash_path::DashPath,
};
pub struct Dash {
    pub base: DashBase,
}
impl Dash {
    pub fn new(base: DashBase) -> Self {
        Self { base }
    }
    pub fn with_value(mut base: DashBase, value: f32, percentage: bool) -> Self {
        base.set_length(value);
        base.set_length_is_percentage(percentage);
        Self { base }
    }
    pub fn normalized_length(&self, contour_length: f32, wraps: bool) -> f32 {
        let mut p = self.base.length();
        if wraps {
            let right = if self.base.length_is_percentage() {
                1.0
            } else {
                contour_length
            };
            p = self.base.length() % right;
            if p < 0.0 {
                p += right;
            }
        }
        if self.base.length_is_percentage() {
            p * contour_length
        } else {
            p
        }
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if !self.base.parent().is::<DashPath>() {
            StatusCode::InvalidObject
        } else {
            StatusCode::Ok
        }
    }
    pub fn length_changed(&mut self) {
        if let Some(parent) = self.base.parent_mut() {
            if let Some(path) = parent.as_mut::<DashPath>() {
                path.invalidate_dash();
            }
        }
    }
    pub fn length_is_percentage_changed(&mut self) {
        self.length_changed();
    }
}
