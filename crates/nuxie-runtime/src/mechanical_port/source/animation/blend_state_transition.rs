use crate::mechanical_port::source::{
    animation::{blend_animation::BlendAnimation, state_instance::StateInstance},
    core::CoreHandle,
    generated::animation::blend_state_transition_base::BlendStateTransitionBase,
};
pub trait BlendTransitionStateInstance {
    fn state_type(&self) -> u16;
    fn animation_timing(&self, animation: &CoreHandle) -> Option<ExitAnimationTiming>;
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExitAnimationTiming {
    pub last_total_time: f32,
    pub total_time: f32,
    pub duration_seconds: f32,
    pub loop_value: i32,
}
#[derive(Default)]
pub struct BlendStateTransition {
    pub base: BlendStateTransitionBase,
    exit_blend_animation: Option<CoreHandle>,
}
impl BlendStateTransition {
    pub fn exit_blend_animation(&self) -> Option<CoreHandle> {
        self.exit_blend_animation.clone()
    }
    pub(crate) fn set_exit_blend_animation(&mut self, value: Option<CoreHandle>) {
        self.exit_blend_animation = value;
    }
    pub fn exit_time_animation_instance(
        &self,
        from: Option<&dyn BlendTransitionStateInstance>,
    ) -> Option<ExitAnimationTiming> {
        let from = from?;
        let animation = self.exit_blend_animation.clone()?;
        match from.state_type() {
            crate::mechanical_port::source::generated::animation::blend_state_1d_input_base::BlendState1DInputBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_direct_base::BlendStateDirectBase::TYPE_KEY => from.animation_timing(&animation),
            _ => None,
        }
    }
    pub fn exit_time_animation(&self, _from: Option<&StateInstance>) -> Option<CoreHandle> {
        self.exit_blend_animation
            .as_ref()
            .and_then(|animation| {
                animation.with_downcast::<BlendAnimation, _>(BlendAnimation::animation)
            })
            .flatten()
    }
}
