use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::dash_base::DashBase,
    shapes::paint::dash_path::DashPath,
};
impl std::ops::Deref for Dash {
    type Target = DashBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Dash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Dash {
    pub const TYPE_KEY: u16 = DashBase::TYPE_KEY;
}

#[derive(Default)]
pub struct Dash {
    pub base: DashBase,
}
impl Dash {
    pub fn new(base: DashBase) -> Self {
        Self { base }
    }
    pub fn with_value(base: DashBase, value: f32, percentage: bool) -> Self {
        let mut result = Self { base };
        if result.base.set_length_value(value) {
            result.length_changed();
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut result)
                .notify_property_changed(DashBase::LENGTH_PROPERTY_KEY);
        }
        if result.base.set_length_is_percentage_value(percentage) {
            result.length_is_percentage_changed();
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut result)
                .notify_property_changed(DashBase::LENGTH_IS_PERCENTAGE_PROPERTY_KEY);
        }
        result
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
        if !self
            .base
            .parent_handle()
            .is_some_and(|parent| parent.is_type_of(DashPath::TYPE_KEY))
        {
            StatusCode::InvalidObject
        } else {
            StatusCode::Ok
        }
    }
    pub fn length_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_downcast_mut::<DashPath, _>(|path| {
                path.invalidate_dash();
            });
        }
    }
    pub fn length_is_percentage_changed(&mut self) {
        self.length_changed();
    }
}
