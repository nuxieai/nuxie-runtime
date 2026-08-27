use std::ptr::NonNull;

use crate::mechanical_port::source::animation::{
    animation_state::AnimationState, linear_animation::LinearAnimation,
    linear_animation_instance::LinearAnimationInstance, state_instance::StateInstance,
    state_machine_instance::StateMachineInstance,
};

pub struct AnimationStateInstance {
    pub base: StateInstance,
    animation_instance: LinearAnimationInstance,
    empty_animation: Option<Box<LinearAnimation>>,
    keep_going: bool,
    state: NonNull<AnimationState>,
}

impl AnimationStateInstance {
    pub fn new(state: &AnimationState, instance: *mut ()) -> Self {
        let empty_animation = state
            .animation()
            .is_none()
            .then(|| Box::new(LinearAnimation::default()));
        let animation = state
            .animation()
            .unwrap_or_else(|| empty_animation.as_deref().unwrap());
        let animation_instance =
            LinearAnimationInstance::with_speed(animation, instance, state.speed());
        Self {
            base: StateInstance::new(state),
            animation_instance,
            empty_animation,
            keep_going: true,
            state: NonNull::from(state),
        }
    }

    pub fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance) {
        let speed = unsafe { self.state.as_ref().speed() };
        self.keep_going = self
            .animation_instance
            .advance(seconds * speed, state_machine_instance);
    }

    pub fn apply(&mut self, _instance: *mut (), mix: f32) {
        self.animation_instance.apply(mix);
    }

    pub fn keep_going(&self) -> bool {
        self.keep_going
    }

    pub fn clear_spilled_time(&mut self) {
        self.animation_instance.clear_spilled_time();
    }

    pub fn animation_instance(&self) -> &LinearAnimationInstance {
        &self.animation_instance
    }

    pub fn animation_instance_mut(&mut self) -> &mut LinearAnimationInstance {
        &mut self.animation_instance
    }

    pub fn for_each_animation_instance(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        callback(&mut self.animation_instance);
    }

    pub fn uses_empty_animation(&self) -> bool {
        self.empty_animation.is_some()
    }
}
