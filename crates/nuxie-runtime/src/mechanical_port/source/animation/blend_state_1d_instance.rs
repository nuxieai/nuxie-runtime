use crate::mechanical_port::source::{
    animation::{
        animation_reset::{AnimationReset, AnimationResetTarget},
        animation_reset_factory::{AnimationResetFactory, ResetArtboard},
        blend_state_instance::{
            BlendAnimationDefinition, BlendStateDefinition, BlendStateInstance,
        },
        layer_state_flags::LayerStateFlags,
        linear_animation_instance::LinearAnimationInstance,
        state_instance::StateInstanceBehavior,
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub trait BlendAnimation1DDefinition: BlendAnimationDefinition {
    fn value(&self) -> f32;
}

impl<K, T> StateInstanceBehavior for BlendState1DInstance<K, T>
where
    K: BlendState1DDefinition<T> + std::any::Any,
    T: BlendAnimation1DDefinition + std::any::Any,
{
    fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        Self::advance(self, seconds, machine);
    }

    fn apply(&mut self, artboard: &RuntimeArtboardInstanceWeakHandle, mix: f32) {
        let _ = artboard.with_artboard_mut(|artboard| {
            BlendState1DInstance::apply(self, artboard, mix);
        });
    }

    fn keep_going(&self) -> bool {
        self.base.keep_going()
    }

    fn clear_spilled_time(&mut self) {
        self.base.clear_spilled_time();
    }

    fn for_each_animation_instance(
        &mut self,
        callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
        self.base.for_each_animation_instance(callback);
    }

    fn with_animation_instance_for_blend(
        &mut self,
        blend_animation: &CoreHandle,
        callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
        self.base
            .with_animation_instance_mut(blend_animation, callback);
    }
}

#[derive(Clone)]
pub enum BlendState1DValueSource {
    Default,
    Input(u32),
    ViewModel(CoreHandle),
}

pub trait BlendState1DDefinition<T>: BlendStateDefinition<T> {
    fn value_source(&self) -> BlendState1DValueSource;
}

pub struct BlendState1DInstance<K, T>
where
    K: BlendState1DDefinition<T>,
    T: BlendAnimation1DDefinition,
{
    pub base: BlendStateInstance<K, T>,
    from: Option<usize>,
    to: Option<usize>,
    animation_reset: Option<AnimationReset>,
}

impl<K, T> BlendState1DInstance<K, T>
where
    K: BlendState1DDefinition<T> + std::any::Any,
    T: BlendAnimation1DDefinition + std::any::Any,
{
    pub fn new<R>(
        state: CoreHandle,
        instance: &mut R,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Self
    where
        R: ResetArtboard,
    {
        let (flags, blend_animations) = state
            .with_downcast::<K, _>(|state| (state.flags(), state.animations()))
            .expect("BlendState1DInstance retains its typed BlendState");
        let animation_reset = if flags & LayerStateFlags::RESET.0 != 0 {
            let animations: Vec<CoreHandle> = blend_animations
                .iter()
                .filter_map(|blend_animation| {
                    blend_animation
                        .with_downcast::<T, _>(BlendAnimationDefinition::animation)
                        .flatten()
                })
                .collect();
            Some(AnimationResetFactory::from_animation_handles(
                &animations,
                instance,
                true,
            ))
        } else {
            None
        };

        Self {
            base: BlendStateInstance::new(state, artboard),
            from: None,
            to: None,
            animation_reset,
        }
    }

    fn animation_index(&self, value: f32) -> usize {
        let mut index = 0;
        let mut start = 0;
        let mut end = self.base.animation_instances.len() as i32 - 1;
        while start <= end {
            let middle = (start + end) >> 1;
            let closest_value = self.base.animation_instances[middle as usize]
                .with_blend_animation(BlendAnimation1DDefinition::value);
            if closest_value < value {
                start = middle + 1;
            } else if closest_value > value {
                end = middle - 1;
            } else {
                index = middle as usize;
                break;
            }
            index = start as usize;
        }
        index
    }

    pub fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        self.base.advance(seconds, machine);

        let value_source = self
            .base
            .with_blend_state(BlendState1DDefinition::value_source);
        let value = match value_source {
            BlendState1DValueSource::Default => 0.0,
            BlendState1DValueSource::Input(input_id) if input_id != u32::MAX => machine
                .number_input_value(input_id)
                .expect("validated one-dimensional blend input remains numeric"),
            BlendState1DValueSource::Input(_) => 0.0,
            BlendState1DValueSource::ViewModel(property) => machine
                .bindable_property_number_value(&property)
                .unwrap_or(0.0),
        };

        let index = self.animation_index(value);
        self.to = (index < self.base.animation_instances.len()).then_some(index);
        self.from = index
            .checked_sub(1)
            .filter(|candidate| *candidate < self.base.animation_instances.len());

        let to_value = self
            .to
            .map(|slot| {
                self.base.animation_instances[slot]
                    .with_blend_animation(BlendAnimation1DDefinition::value)
            })
            .unwrap_or(0.0);
        let from_value = self
            .from
            .map(|slot| {
                self.base.animation_instances[slot]
                    .with_blend_animation(BlendAnimation1DDefinition::value)
            })
            .unwrap_or(0.0);
        let (to_mix, from_mix) =
            if self.to.is_none() || self.from.is_none() || to_value == from_value {
                (1.0, 1.0)
            } else {
                let to_mix = (value - from_value) / (to_value - from_value);
                (to_mix, 1.0 - to_mix)
            };

        for animation in &mut self.base.animation_instances {
            let animation_value = animation.with_blend_animation(BlendAnimation1DDefinition::value);
            if self.to.is_some() && animation_value == to_value {
                animation.mix(to_mix);
            } else if self.from.is_some() && animation_value == from_value {
                animation.mix(from_mix);
            } else {
                animation.mix(0.0);
            }
        }
    }

    pub fn apply<R: AnimationResetTarget>(&mut self, artboard: &mut R, mix: f32) {
        if let Some(animation_reset) = &self.animation_reset {
            animation_reset.apply(artboard);
        }
        self.base.apply(mix);
    }
}

impl<K, T> Drop for BlendState1DInstance<K, T>
where
    K: BlendState1DDefinition<T> + std::any::Any,
    T: BlendAnimation1DDefinition + std::any::Any,
{
    fn drop(&mut self) {
        if let Some(animation_reset) = self.animation_reset.take() {
            AnimationResetFactory::release(animation_reset);
        }
    }
}
