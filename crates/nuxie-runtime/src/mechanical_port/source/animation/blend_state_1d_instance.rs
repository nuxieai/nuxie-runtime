use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        animation_reset::{AnimationReset, AnimationResetTarget},
        animation_reset_factory::{AnimationResetFactory, ResetArtboard, ResetLinearAnimation},
        blend_state_instance::{
            BlendAnimationDefinition, BlendStateDefinition, BlendStateInstance,
        },
        layer_state_flags::LayerStateFlags,
        state_machine_instance::StateMachineInstance,
    },
    data_bind::bindable_property::BindableProperty,
};

pub trait BlendAnimation1DDefinition: BlendAnimationDefinition {
    fn value(&self) -> f32;
}

#[derive(Clone, Copy)]
pub enum BlendState1DValueSource {
    Default,
    Input(u32),
    ViewModel(NonNull<BindableProperty>),
}

pub trait BlendState1DDefinition<T>: BlendStateDefinition<T> {
    fn value_source(&self) -> BlendState1DValueSource;
}

pub struct BlendState1DInstance<'a, K, T>
where
    K: BlendState1DDefinition<T>,
    T: BlendAnimation1DDefinition,
{
    pub base: BlendStateInstance<'a, K, T>,
    from: Option<usize>,
    to: Option<usize>,
    animation_reset: Option<AnimationReset>,
}

impl<'a, K, T> BlendState1DInstance<'a, K, T>
where
    K: BlendState1DDefinition<T>,
    T: BlendAnimation1DDefinition,
    T::Animation: ResetLinearAnimation,
{
    pub fn new<R>(state: &'a K, instance: &mut R) -> Self
    where
        R: ResetArtboard,
    {
        let animation_reset = if state.flags() & LayerStateFlags::RESET.0 != 0 {
            let animations: Vec<&dyn ResetLinearAnimation> = state
                .animations()
                .into_iter()
                .map(|blend_animation| blend_animation.animation() as &dyn ResetLinearAnimation)
                .collect();
            Some(AnimationResetFactory::from_animations(
                &animations,
                instance,
                true,
            ))
        } else {
            None
        };

        Self {
            base: BlendStateInstance::new(state, instance as *mut R as *mut ()),
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
                .blend_animation()
                .value();
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

        let value = match self.base.blend_state().value_source() {
            BlendState1DValueSource::Default => 0.0,
            BlendState1DValueSource::Input(input_id) if input_id != u32::MAX => machine
                .number_input_value(input_id)
                .expect("validated one-dimensional blend input remains numeric"),
            BlendState1DValueSource::Input(_) => 0.0,
            BlendState1DValueSource::ViewModel(property) => machine
                .bindable_property_number_value(property)
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
                    .blend_animation()
                    .value()
            })
            .unwrap_or(0.0);
        let from_value = self
            .from
            .map(|slot| {
                self.base.animation_instances[slot]
                    .blend_animation()
                    .value()
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
            let animation_value = animation.blend_animation().value();
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

impl<K, T> Drop for BlendState1DInstance<'_, K, T>
where
    K: BlendState1DDefinition<T>,
    T: BlendAnimation1DDefinition,
{
    fn drop(&mut self) {
        if let Some(animation_reset) = self.animation_reset.take() {
            AnimationResetFactory::release(animation_reset);
        }
    }
}
