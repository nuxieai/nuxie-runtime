use crate::mechanical_port::source::{
    animation::{
        blend_animation::BlendAnimation, linear_animation::LinearAnimation,
        linear_animation_instance::LinearAnimationInstance, state_instance::StateInstance,
    },
    generated::animation::blend_state_transition_base::BlendStateTransitionBase,
};
use std::ptr::NonNull;
pub trait BlendTransitionStateInstance {
    fn state_type(&self) -> u16;
    fn animation_instance(
        &self,
        animation: NonNull<BlendAnimation>,
    ) -> Option<NonNull<LinearAnimationInstance>>;
}
#[derive(Default)]
pub struct BlendStateTransition {
    pub base: BlendStateTransitionBase,
    exit_blend_animation: Option<NonNull<BlendAnimation>>,
}
impl BlendStateTransition {
    pub fn exit_blend_animation(&self) -> Option<NonNull<BlendAnimation>> {
        self.exit_blend_animation
    }
    pub(crate) fn set_exit_blend_animation(&mut self, value: Option<NonNull<BlendAnimation>>) {
        self.exit_blend_animation = value;
    }
    pub fn exit_time_animation_instance(
        &self,
        from: Option<&dyn BlendTransitionStateInstance>,
    ) -> Option<NonNull<LinearAnimationInstance>> {
        let from = from?;
        let animation = self.exit_blend_animation?;
        match from.state_type() {
            crate::mechanical_port::source::generated::animation::blend_state_1d_input_base::BlendState1DInputBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_direct_base::BlendStateDirectBase::TYPE_KEY => from.animation_instance(animation),
            _ => None,
        }
    }
    pub fn exit_time_animation(
        &self,
        _from: Option<&StateInstance>,
    ) -> Option<NonNull<LinearAnimation>> {
        self.exit_blend_animation
            .map(|animation| unsafe { NonNull::from(animation.as_ref().animation()) })
    }
}
