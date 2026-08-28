use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    animation::{
        animation_state::AnimationState,
        linear_animation::LinearAnimation,
        linear_animation_instance::LinearAnimationInstance,
        state_instance::{StateInstance, StateInstanceBehavior},
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub struct AnimationStateInstance {
    pub base: StateInstance,
    animation_instance: LinearAnimationInstance,
    empty_animation: Option<Rc<RefCell<LinearAnimation>>>,
    keep_going: bool,
    state: CoreHandle,
}

impl StateInstanceBehavior for AnimationStateInstance {
    fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        Self::advance(self, seconds, machine);
    }

    fn apply(&mut self, artboard: &RuntimeArtboardInstanceWeakHandle, mix: f32) {
        Self::apply(self, artboard, mix);
    }

    fn keep_going(&self) -> bool {
        Self::keep_going(self)
    }

    fn clear_spilled_time(&mut self) {
        Self::clear_spilled_time(self);
    }

    fn for_each_animation_instance(
        &mut self,
        callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
        callback(&mut self.animation_instance);
    }
}

impl AnimationStateInstance {
    pub fn new(state: CoreHandle, instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        let (animation, speed) = state
            .with_downcast::<AnimationState, _>(|state| (state.animation(), state.speed()))
            .expect("AnimationStateInstance retains an AnimationState");
        let empty_animation = animation
            .is_none()
            .then(|| Rc::new(RefCell::new(LinearAnimation::default())));
        let animation_instance = match animation {
            Some(animation) => LinearAnimationInstance::new(animation, instance, speed),
            None => LinearAnimationInstance::new_runtime(
                empty_animation.as_ref().unwrap().clone(),
                instance,
                speed,
            ),
        };
        Self {
            base: StateInstance::new(state.clone()),
            animation_instance,
            empty_animation,
            keep_going: true,
            state,
        }
    }

    pub fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance) {
        let speed = self
            .state
            .with_downcast::<AnimationState, _>(AnimationState::speed)
            .expect("AnimationStateInstance retains an AnimationState");
        self.keep_going = self
            .animation_instance
            .advance(seconds * speed, state_machine_instance);
    }

    pub fn apply(&mut self, _instance: &RuntimeArtboardInstanceWeakHandle, mix: f32) {
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
