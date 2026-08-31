use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::{
        color_channels_base::ColorChannels, gradient_stop_base::GradientStopBase,
        linear_gradient_base::LinearGradientBase, radial_gradient_base::RadialGradientBase,
    },
    shapes::paint::{linear_gradient::LinearGradient, radial_gradient::RadialGradient},
};
impl std::ops::Deref for GradientStop {
    type Target = GradientStopBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GradientStop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl GradientStop {
    pub const TYPE_KEY: u16 = GradientStopBase::TYPE_KEY;
}

#[derive(Default)]
pub struct GradientStop {
    pub base: GradientStopBase,
}
impl GradientStop {
    fn with_parent_gradient<R>(&self, f: impl FnOnce(&mut LinearGradient) -> R) -> Option<R> {
        let parent = self.base.parent_handle()?;
        if !parent.is_type_of(LinearGradientBase::TYPE_KEY) {
            return None;
        }
        // C++ as<LinearGradient>() projects the inherited owner, including
        // RadialGradientBase::Super. An exact Any downcast rejects radial stops.
        let is_radial = parent.is_type_of(RadialGradientBase::TYPE_KEY);
        parent.with_mut(|parent| {
            if is_radial {
                let radial = parent
                    .as_any_mut()
                    .downcast_mut::<RadialGradient>()
                    .expect("native RadialGradient owner");
                f(&mut radial.base.base)
            } else {
                f(parent
                    .as_any_mut()
                    .downcast_mut::<LinearGradient>()
                    .expect("native LinearGradient owner"))
            }
        })
    }

    pub fn color_red(&self) -> u32 {
        ColorChannels::color_red(self)
    }
    pub fn set_color_red(&mut self, value: u32) {
        ColorChannels::set_color_red(self, value);
    }
    pub fn color_green(&self) -> u32 {
        ColorChannels::color_green(self)
    }
    pub fn set_color_green(&mut self, value: u32) {
        ColorChannels::set_color_green(self, value);
    }
    pub fn color_blue(&self) -> u32 {
        ColorChannels::color_blue(self)
    }
    pub fn set_color_blue(&mut self, value: u32) {
        ColorChannels::set_color_blue(self, value);
    }
    pub fn color_alpha(&self) -> u32 {
        ColorChannels::color_alpha(self)
    }
    pub fn set_color_alpha(&mut self, value: u32) {
        ColorChannels::set_color_alpha(self, value);
    }

    pub fn set_color_value(&mut self, value: i32) {
        if !self.base.set_color_value_value(value) {
            return;
        }
        self.color_value_changed();
        self.base
            .base
            .notify_property_changed(GradientStopBase::COLOR_VALUE_PROPERTY_KEY);
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        if self
            .with_parent_gradient(|gradient| gradient.add_stop(this))
            .is_none()
        {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn color_value_changed(&mut self) {
        self.with_parent_gradient(LinearGradient::mark_gradient_dirty);
    }
    pub fn position_changed(&mut self) {
        self.with_parent_gradient(LinearGradient::mark_stops_dirty);
    }
}

impl ColorChannels for GradientStop {
    fn color_value(&self) -> i32 {
        self.base.color_value()
    }
    fn set_color_value(&mut self, value: i32) {
        GradientStop::set_color_value(self, value);
    }
}
