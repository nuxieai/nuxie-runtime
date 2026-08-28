use std::{any::Any, marker::PhantomData};

use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance, state_instance::StateInstance,
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub trait BlendAnimationDefinition {
    fn animation(&self) -> Option<CoreHandle>;
}

pub trait BlendStateDefinition<T> {
    fn animations(&self) -> Vec<CoreHandle>;
    fn flags(&self) -> u8;
}

pub struct BlendStateAnimationInstance<T: BlendAnimationDefinition + Any> {
    blend_animation: CoreHandle,
    animation_instance: LinearAnimationInstance,
    mix: f32,
    definition: PhantomData<T>,
}

impl<T: BlendAnimationDefinition + Any> BlendStateAnimationInstance<T> {
    pub fn new(blend_animation: CoreHandle, instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        let animation = blend_animation
            .with_downcast::<T, _>(BlendAnimationDefinition::animation)
            .flatten()
            .expect("a validated BlendAnimation retains a LinearAnimation");
        Self {
            blend_animation,
            animation_instance: LinearAnimationInstance::new(animation, instance, 1.0),
            mix: 0.0,
            definition: PhantomData,
        }
    }

    pub fn blend_animation(&self) -> CoreHandle {
        self.blend_animation.clone()
    }
    pub fn with_blend_animation<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.blend_animation
            .with_downcast::<T, _>(f)
            .expect("BlendStateAnimationInstance retains its typed definition")
    }
    pub fn animation_instance(&self) -> &LinearAnimationInstance {
        &self.animation_instance
    }
    pub fn mix(&mut self, value: f32) {
        self.mix = value;
    }
}

pub struct BlendStateInstance<K, T>
where
    K: BlendStateDefinition<T> + Any,
    T: BlendAnimationDefinition + Any,
{
    pub base: StateInstance,
    pub(crate) animation_instances: Vec<BlendStateAnimationInstance<T>>,
    keep_going: bool,
    blend_state: CoreHandle,
    definition: PhantomData<K>,
}

impl<K, T> BlendStateInstance<K, T>
where
    K: BlendStateDefinition<T> + Any,
    T: BlendAnimationDefinition + Any,
{
    pub fn new(blend_state: CoreHandle, instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        let (animations, flags) = blend_state
            .with_downcast::<K, _>(|state| (state.animations(), state.flags()))
            .expect("BlendStateInstance retains its typed BlendState");
        let mut animation_instances = Vec::with_capacity(animations.len());
        for blend_animation in animations {
            animation_instances.push(BlendStateAnimationInstance::new(
                blend_animation,
                instance.clone(),
            ));
        }

        // Upstream gathers the reset animations when the Reset bit is set; the
        // resulting local vector is intentionally discarded there as well.
        if flags & (1 << 1) != 0 {
            let animations: Vec<_> = animation_instances
                .iter()
                .map(|animation| {
                    animation.with_blend_animation(BlendAnimationDefinition::animation)
                })
                .collect();
            drop(animations);
        }

        Self {
            base: StateInstance::new(blend_state.clone()),
            animation_instances,
            keep_going: true,
            blend_state,
            definition: PhantomData,
        }
    }

    pub fn keep_going(&self) -> bool {
        self.keep_going
    }

    pub fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance) {
        for animation in &mut self.animation_instances {
            if animation.animation_instance.keep_going() {
                animation
                    .animation_instance
                    .advance(seconds, state_machine_instance);
            }
        }
    }

    pub fn apply(&mut self, mix: f32) {
        for animation in &mut self.animation_instances {
            let animation_mix = mix * animation.mix;
            if animation_mix != 0.0 {
                animation.animation_instance.apply(animation_mix);
            }
        }
    }

    pub fn clear_spilled_time(&mut self) {
        for animation in &mut self.animation_instances {
            animation.animation_instance.clear_spilled_time();
        }
    }

    pub fn for_each_animation_instance(
        &mut self,
        callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animation_instances {
            callback(&mut animation.animation_instance);
        }
    }

    pub fn animation_instance(
        &self,
        blend_animation: &CoreHandle,
    ) -> Option<&LinearAnimationInstance> {
        self.animation_instances
            .iter()
            .find(|animation| &animation.blend_animation == blend_animation)
            .map(BlendStateAnimationInstance::animation_instance)
    }

    pub fn with_animation_instance_mut(
        &mut self,
        blend_animation: &CoreHandle,
        callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
        if let Some(animation) = self
            .animation_instances
            .iter_mut()
            .find(|animation| &animation.blend_animation == blend_animation)
        {
            callback(&mut animation.animation_instance);
        }
    }

    pub fn with_blend_state<R>(&self, f: impl FnOnce(&K) -> R) -> R {
        self.blend_state
            .with_downcast::<K, _>(f)
            .expect("BlendStateInstance retains its typed BlendState")
    }
}
