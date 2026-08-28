use crate::mechanical_port::source::{
    animation::{
        blend_animation_direct::DirectBlendSource,
        blend_state_instance::{
            BlendAnimationDefinition, BlendStateDefinition, BlendStateInstance,
        },
        linear_animation_instance::LinearAnimationInstance,
        state_instance::StateInstanceBehavior,
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};

pub trait BlendAnimationDirectDefinition: BlendAnimationDefinition {
    fn blend_source(&self) -> u32;
    fn mix_value(&self) -> f32;
    fn input_id(&self) -> u32;
    fn bindable_property(&self) -> Option<CoreHandle>;
}

impl<K, T> StateInstanceBehavior for BlendStateDirectInstance<K, T>
where
    K: BlendStateDefinition<T> + std::any::Any,
    T: BlendAnimationDirectDefinition + std::any::Any,
{
    fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        Self::advance(self, seconds, machine);
    }

    fn apply(&mut self, _artboard: &RuntimeArtboardInstanceWeakHandle, mix: f32) {
        self.base.apply(mix);
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
pub struct BlendStateDirectInstance<K, T>
where
    K: BlendStateDefinition<T>,
    T: BlendAnimationDirectDefinition,
{
    pub base: BlendStateInstance<K, T>,
}
impl<K, T> BlendStateDirectInstance<K, T>
where
    K: BlendStateDefinition<T> + std::any::Any,
    T: BlendAnimationDirectDefinition + std::any::Any,
{
    pub fn new(state: CoreHandle, instance: RuntimeArtboardInstanceWeakHandle) -> Self {
        Self {
            base: BlendStateInstance::new(state, instance),
        }
    }
    pub fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        self.base.advance(seconds, machine);
        for animation in &mut self.base.animation_instances {
            let (blend_source, mix_value, bindable_property, input_id) = animation
                .with_blend_animation(|definition| {
                    (
                        definition.blend_source(),
                        definition.mix_value(),
                        definition.bindable_property(),
                        definition.input_id(),
                    )
                });
            if blend_source == DirectBlendSource::MixValue as u32 {
                let value = mix_value;
                animation.mix((value / 100.0).clamp(0.0, 1.0));
            } else if blend_source == DirectBlendSource::DataBindId as u32 {
                let Some(bindable_property) = bindable_property else {
                    continue;
                };
                if let Some(value) = machine.bindable_property_number_value(&bindable_property) {
                    animation.mix((value / 100.0).clamp(0.0, 1.0));
                }
            } else {
                let value = machine
                    .number_input_value(input_id)
                    .expect("inputId direct blends are validated as number inputs during import");
                animation.mix((value / 100.0).clamp(0.0, 1.0));
            }
        }
    }
}
