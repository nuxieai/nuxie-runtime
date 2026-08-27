use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolatorBehavior,
    generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBase,
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub trait InterpolatingKeyFrameContext {
    fn resolve_interpolator(&self, id: u32) -> Option<NonNull<dyn KeyFrameInterpolatorBehavior>>;
}
pub trait KeyFrameValueContext {
    fn bool_value(&self, keyframe: *const ()) -> Option<bool>;
    fn string_value(&self, keyframe: *const ()) -> Option<&str>;
    fn color_value(&self, keyframe: *const ()) -> Option<i32>;
    fn number_value(&self, keyframe: *const ()) -> Option<f32>;
    fn stateful_interpolator(
        &self,
        keyframe: *const (),
        shared: NonNull<dyn KeyFrameInterpolatorBehavior>,
    ) -> Option<NonNull<dyn KeyFrameInterpolatorBehavior>>;
}
#[derive(Default)]
pub struct InterpolatingKeyFrame {
    pub base: InterpolatingKeyFrameBase,
    interpolator: Option<NonNull<dyn KeyFrameInterpolatorBehavior>>,
}
impl InterpolatingKeyFrame {
    pub fn interpolator(&self) -> Option<NonNull<dyn KeyFrameInterpolatorBehavior>> {
        self.interpolator
    }
    pub fn on_added_dirty(&mut self, context: &dyn InterpolatingKeyFrameContext) -> StatusCode {
        if self.base.interpolator_id() != u32::MAX {
            let Some(value) = context.resolve_interpolator(self.base.interpolator_id()) else {
                return StatusCode::MissingObject;
            };
            self.interpolator = Some(value);
        }
        StatusCode::Ok
    }
    pub fn effective_interpolator(
        &self,
        context: Option<&dyn KeyFrameValueContext>,
    ) -> Option<NonNull<dyn KeyFrameInterpolatorBehavior>> {
        let shared = self.interpolator?;
        let Some(context) = context else {
            return Some(shared);
        };
        if unsafe { !shared.as_ref().is_scripted() } {
            return Some(shared);
        }
        context
            .stateful_interpolator(self as *const Self as *const (), shared)
            .or(Some(shared))
    }
}
