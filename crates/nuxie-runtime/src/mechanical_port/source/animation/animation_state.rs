use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        animation_state_instance::AnimationStateInstance, linear_animation::LinearAnimation,
    },
    generated::animation::animation_state_base::AnimationStateBase,
};

pub struct AnimationState {
    pub base: AnimationStateBase,
    animation: Option<NonNull<LinearAnimation>>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            base: AnimationStateBase::default(),
            animation: None,
        }
    }
}

impl AnimationState {
    pub fn animation(&self) -> Option<&LinearAnimation> {
        self.animation
            .map(|animation| unsafe { animation.as_ref() })
    }

    pub(crate) fn set_animation(&mut self, animation: Option<NonNull<LinearAnimation>>) {
        self.animation = animation;
    }

    #[cfg(feature = "testing")]
    pub fn animation_for_testing(&mut self, animation: &mut LinearAnimation) {
        self.animation = Some(NonNull::from(animation));
    }

    pub fn speed(&self) -> f32 {
        self.base.base.base.speed()
    }

    pub fn make_instance(&self, instance: *mut ()) -> AnimationStateInstance {
        AnimationStateInstance::new(self, instance)
    }
}
