use crate::mechanical_port::source::{
    core::CoreHandle, generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBase,
    status_code::StatusCode,
};
pub trait InterpolatingKeyFrameContext {
    fn resolve_interpolator(&self, id: u32) -> Option<CoreHandle>;
}
pub trait KeyFrameValueContext {
    fn bool_value(&self, keyframe: &CoreHandle) -> Option<bool>;
    fn string_value(&self, keyframe: &CoreHandle) -> Option<String>;
    fn color_value(&self, keyframe: &CoreHandle) -> Option<i32>;
    fn number_value(&self, keyframe: &CoreHandle) -> Option<f32>;
    fn stateful_interpolator_transform_value(
        &self,
        keyframe: &CoreHandle,
        shared: &CoreHandle,
        from: f32,
        to: f32,
        factor: f32,
    ) -> Option<f32>;
    fn stateful_interpolator_transform(
        &self,
        keyframe: &CoreHandle,
        shared: &CoreHandle,
        factor: f32,
    ) -> Option<f32>;
}
#[derive(Default)]
pub struct InterpolatingKeyFrame {
    pub base: InterpolatingKeyFrameBase,
    interpolator: Option<CoreHandle>,
}
impl InterpolatingKeyFrame {
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.interpolator.clone()
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
    pub fn transform_value(
        &self,
        context: Option<&dyn KeyFrameValueContext>,
        from: f32,
        to: f32,
        factor: f32,
    ) -> Option<f32> {
        let shared = self.interpolator.as_ref()?;
        let scripted = shared
            .with(|interpolator| interpolator.keyframe_interpolator_is_scripted())
            .flatten()
            .unwrap_or(false);
        if scripted {
            if let Some(value) = self.base.handle().and_then(|keyframe| {
                context.and_then(|context| {
                    context
                        .stateful_interpolator_transform_value(&keyframe, shared, from, to, factor)
                })
            }) {
                return Some(value);
            }
        }
        shared
            .with_mut(|interpolator| {
                interpolator.keyframe_interpolator_transform_value(from, to, factor)
            })
            .flatten()
    }

    pub fn transform(
        &self,
        context: Option<&dyn KeyFrameValueContext>,
        factor: f32,
    ) -> Option<f32> {
        let shared = self.interpolator.as_ref()?;
        let scripted = shared
            .with(|interpolator| interpolator.keyframe_interpolator_is_scripted())
            .flatten()
            .unwrap_or(false);
        if scripted {
            if let Some(value) = self.base.handle().and_then(|keyframe| {
                context.and_then(|context| {
                    context.stateful_interpolator_transform(&keyframe, shared, factor)
                })
            }) {
                return Some(value);
            }
        }
        shared
            .with_mut(|interpolator| interpolator.keyframe_interpolator_transform(factor))
            .flatten()
    }
}
impl std::ops::Deref for InterpolatingKeyFrame {
    type Target = InterpolatingKeyFrameBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for InterpolatingKeyFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::keyframe_base::KeyFrameBaseCallbacks
    for InterpolatingKeyFrame
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
impl crate::mechanical_port::source::generated::animation::interpolating_keyframe_base::InterpolatingKeyFrameBaseCallbacks for InterpolatingKeyFrame { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
