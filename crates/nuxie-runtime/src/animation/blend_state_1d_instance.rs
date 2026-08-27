use crate::state_machine::{
    AnimationReset, AnimationResetFactory, RuntimeBlendAnimationHandle, RuntimeBlendState1D,
    RuntimeBlendState1DSource, StateMachineBindableNumberInstance, StateMachineInputInstance,
};

// Mirrors src/animation/blend_state_1d_instance.cpp and
// include/rive/animation/blend_state_1d_instance.hpp.
#[derive(Debug, Clone)]
pub(crate) struct BlendState1DInstance {
    pub(crate) animations: Vec<BlendAnimation1DInstance>,
    pub(crate) from: Option<RuntimeBlendAnimationHandle>,
    pub(crate) to: Option<RuntimeBlendAnimationHandle>,
    animation_reset: Option<AnimationReset>,
}

impl BlendState1DInstance {
    pub(crate) fn new(
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
        reset_blend_values: bool,
    ) -> Self {
        let animations: Vec<BlendAnimation1DInstance> = blend_state
            .animations()
            .iter()
            .enumerate()
            .filter_map(|(definition_index, animation)| {
                Some(BlendAnimation1DInstance {
                    definition: RuntimeBlendAnimationHandle::new(definition_index),
                    animation: LinearAnimationInstance::new(
                        animation.animation(),
                        Arc::clone(animation_definitions),
                        Arc::clone(empty_animation_definition),
                        1.0,
                    )?,
                    mix: 0.0,
                })
            })
            .collect();
        let animation_reset = if reset_blend_values {
            Some(AnimationResetFactory::from_animation_instances(
                artboard,
                animations.iter().map(|animation| &animation.animation),
                true,
            ))
        } else {
            None
        };

        Self {
            animations,
            from: None,
            to: None,
            animation_reset,
        }
    }

    fn animation_index(&self, blend_state: &RuntimeBlendState1D, value: f32) -> usize {
        let mut index = 0_usize;
        let mut start = 0_isize;
        let mut end = self.animations.len() as isize - 1;

        while start <= end {
            let mid = (start + end) >> 1;
            let closest_value = self
                .animations
                .get(mid as usize)
                .and_then(|animation| blend_state.animations().get(animation.definition.index()))
                .map(|animation| animation.value)
                .unwrap_or(0.0);
            if closest_value < value {
                start = mid + 1;
            } else if closest_value > value {
                end = mid - 1;
            } else {
                index = mid as usize;
                break;
            }

            index = start as usize;
        }

        index
    }

    pub(crate) fn apply(&mut self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        if let Some(reset) = self.animation_reset.as_ref() {
            changed |= reset.apply(artboard);
        }
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |= animation.animation.apply(artboard, animation_mix);
        }
        changed
    }

    pub(crate) fn advance(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            blend_state,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                if let Some(events) = reported_events.as_mut() {
                    artboard.advance_linear_animation_instance_with_events(
                        &mut animation.animation,
                        elapsed_seconds,
                        *events,
                    );
                } else {
                    artboard.advance_linear_animation_instance(
                        &mut animation.animation,
                        elapsed_seconds,
                    );
                }
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        if self.animations.is_empty() {
            return;
        }

        let value = match blend_state.source {
            RuntimeBlendState1DSource::Input { input_index } => {
                RuntimeBlendState1DInput::input_index(input_index)
                    .and_then(|input_index| inputs.get(input_index))
                    .and_then(StateMachineInputInstance::number_value)
                    .unwrap_or(0.0)
            }
            RuntimeBlendState1DSource::BindableProperty { global_id } => {
                RuntimeBlendState1DViewModel::value(global_id, bindable_numbers)
            }
        };

        let to_index = self.animation_index(blend_state, value);
        self.to = (to_index < self.animations.len()).then(|| self.animations[to_index].definition);
        self.from = to_index
            .checked_sub(1)
            .and_then(|index| self.animations.get(index))
            .map(|animation| animation.definition);
        let to_value = self
            .to
            .and_then(|handle| blend_state.animations().get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let from_value = self
            .from
            .and_then(|handle| blend_state.animations().get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let (mix, mix_from) = if self.to.is_none() || self.from.is_none() || to_value == from_value
        {
            (1.0, 1.0)
        } else {
            let mix = (value - from_value) / (to_value - from_value);
            (mix, 1.0 - mix)
        };

        for animation in &mut self.animations {
            let animation_value = blend_state
                .animations()
                .get(animation.definition.index())
                .map(|definition| definition.value)
                .unwrap_or(0.0);
            if self.to.is_some() && animation_value == to_value {
                animation.mix = mix;
            } else if self.from.is_some() && animation_value == from_value {
                animation.mix = mix_from;
            } else {
                animation.mix = 0.0;
            }
        }
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .iter()
            .find(|animation| animation.definition.index() == index)
            .map(|animation| &animation.animation)
    }

    pub(crate) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animations {
            callback(&mut animation.animation);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BlendAnimation1DInstance {
    pub(crate) definition: RuntimeBlendAnimationHandle,
    pub(crate) animation: LinearAnimationInstance,
    pub(crate) mix: f32,
}
