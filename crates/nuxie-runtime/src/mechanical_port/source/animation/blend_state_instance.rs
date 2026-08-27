use crate::mechanical_port::source::animation::{
    linear_animation_instance::LinearAnimationInstance, state_instance::StateInstance,
    state_machine_instance::StateMachineInstance,
};

pub trait BlendAnimationDefinition {
    type Animation;
    fn animation(&self) -> &Self::Animation;
}

pub trait BlendStateDefinition<T> {
    fn animations(&self) -> &[T];
    fn flags(&self) -> u8;
}

pub struct BlendStateAnimationInstance<'a, T: BlendAnimationDefinition> {
    blend_animation: &'a T,
    animation_instance: LinearAnimationInstance,
    mix: f32,
}

impl<'a, T: BlendAnimationDefinition> BlendStateAnimationInstance<'a, T> {
    pub fn new(blend_animation: &'a T, instance: *mut ()) -> Self {
        Self {
            blend_animation,
            animation_instance: LinearAnimationInstance::new(blend_animation.animation(), instance),
            mix: 0.0,
        }
    }

    pub fn blend_animation(&self) -> &T {
        self.blend_animation
    }
    pub fn animation_instance(&self) -> &LinearAnimationInstance {
        &self.animation_instance
    }
    pub fn mix(&mut self, value: f32) {
        self.mix = value;
    }
}

pub struct BlendStateInstance<'a, K, T>
where
    K: BlendStateDefinition<T>,
    T: BlendAnimationDefinition,
{
    pub base: StateInstance,
    pub(crate) animation_instances: Vec<BlendStateAnimationInstance<'a, T>>,
    keep_going: bool,
    blend_state: &'a K,
}

impl<'a, K, T> BlendStateInstance<'a, K, T>
where
    K: BlendStateDefinition<T>,
    T: BlendAnimationDefinition,
{
    pub fn new(blend_state: &'a K, instance: *mut ()) -> Self {
        let mut animation_instances = Vec::with_capacity(blend_state.animations().len());
        for blend_animation in blend_state.animations() {
            animation_instances.push(BlendStateAnimationInstance::new(blend_animation, instance));
        }

        // Upstream gathers the reset animations when the Reset bit is set; the
        // resulting local vector is intentionally discarded there as well.
        if blend_state.flags() & (1 << 1) != 0 {
            let animations: Vec<_> = blend_state
                .animations()
                .iter()
                .map(BlendAnimationDefinition::animation)
                .collect();
            drop(animations);
        }

        Self {
            base: StateInstance::new(blend_state),
            animation_instances,
            keep_going: true,
            blend_state,
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

    pub fn for_each_animation_instance(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animation_instances {
            callback(&mut animation.animation_instance);
        }
    }

    pub fn animation_instance(&self, blend_animation: &T) -> Option<&LinearAnimationInstance> {
        self.animation_instances
            .iter()
            .find(|animation| std::ptr::eq(animation.blend_animation, blend_animation))
            .map(BlendStateAnimationInstance::animation_instance)
    }

    pub fn blend_state(&self) -> &K {
        self.blend_state
    }
}
