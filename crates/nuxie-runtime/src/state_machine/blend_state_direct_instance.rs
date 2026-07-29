use super::*;

#[derive(Debug, Clone)]
pub(crate) struct BlendStateDirectInstance {
    pub(super) animations: Vec<BlendAnimationDirectInstance>,
}

impl BlendStateDirectInstance {
    pub(crate) fn new(blend_state: &RuntimeBlendStateDirect, artboard: &ArtboardInstance) -> Self {
        let animations = blend_state
            .animations
            .iter()
            .enumerate()
            .filter_map(|(definition_index, animation)| {
                let linear_animation = artboard.linear_animation_definition(animation.animation)?;
                Some(BlendAnimationDirectInstance {
                    definition: RuntimeBlendAnimationHandle::new(definition_index),
                    animation: LinearAnimationInstance::new(
                        animation.animation,
                        linear_animation,
                        1.0,
                    ),
                    mix: 0.0,
                })
            })
            .collect();

        Self { animations }
    }

    pub(crate) fn advance(
        &mut self,
        blend_state: &RuntimeBlendStateDirect,
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
        blend_state: &RuntimeBlendStateDirect,
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
        blend_state: &RuntimeBlendStateDirect,
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
        blend_state: &RuntimeBlendStateDirect,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        for animation in &mut self.animations {
            let Some(definition) = blend_state.animations.get(animation.definition.index()) else {
                continue;
            };
            let value = match definition.source {
                RuntimeDirectBlendSource::Input { input_index } => inputs
                    .get(input_index)
                    .and_then(StateMachineInputInstance::number_value)
                    .unwrap_or(0.0),
                RuntimeDirectBlendSource::MixValue { value } => value,
                RuntimeDirectBlendSource::BindableProperty { global_id } => {
                    let Some(value) = global_id
                        .and_then(|global_id| bindable_number_value(bindable_numbers, global_id))
                    else {
                        // C++ leaves the current mix untouched when the authored
                        // bindable property cannot produce a number instance.
                        continue;
                    };
                    value
                }
            };
            animation.mix = (value / 100.0).max(0.0).min(1.0);
        }
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .iter()
            .find(|animation| animation.definition.index() == index)
            .map(|animation| &animation.animation)
    }

    pub(super) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animations {
            callback(&mut animation.animation);
        }
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |= animation.animation.apply(artboard, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
pub(super) struct BlendAnimationDirectInstance {
    pub(super) definition: RuntimeBlendAnimationHandle,
    pub(super) animation: LinearAnimationInstance,
    pub(super) mix: f32,
}
