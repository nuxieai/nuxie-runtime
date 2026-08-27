use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        blend_animation_direct::DirectBlendSource,
        blend_state_instance::{
            BlendAnimationDefinition, BlendStateDefinition, BlendStateInstance,
        },
        state_machine_instance::StateMachineInstance,
    },
    data_bind::bindable_property::BindableProperty,
};

pub trait BlendAnimationDirectDefinition: BlendAnimationDefinition {
    fn blend_source(&self) -> u32;
    fn mix_value(&self) -> f32;
    fn input_id(&self) -> u32;
    fn bindable_property(&self) -> Option<NonNull<BindableProperty>>;
}
pub struct BlendStateDirectInstance<'a, K, T>
where
    K: BlendStateDefinition<T>,
    T: BlendAnimationDirectDefinition,
{
    pub base: BlendStateInstance<'a, K, T>,
}
impl<'a, K, T> BlendStateDirectInstance<'a, K, T>
where
    K: BlendStateDefinition<T>,
    T: BlendAnimationDirectDefinition,
{
    pub fn new(state: &'a K, instance: *mut ()) -> Self {
        Self {
            base: BlendStateInstance::new(state, instance),
        }
    }
    pub fn advance(&mut self, seconds: f32, machine: &mut StateMachineInstance) {
        self.base.advance(seconds, machine);
        for animation in &mut self.base.animation_instances {
            let definition = animation.blend_animation();
            if definition.blend_source() == DirectBlendSource::MixValue as u32 {
                let value = definition.mix_value();
                animation.mix((value / 100.0).clamp(0.0, 1.0));
            } else if definition.blend_source() == DirectBlendSource::DataBindId as u32 {
                let Some(bindable_property) = definition.bindable_property() else {
                    continue;
                };
                if let Some(value) = machine.bindable_property_number_value(bindable_property) {
                    animation.mix((value / 100.0).clamp(0.0, 1.0));
                }
            } else {
                let value = machine
                    .number_input_value(definition.input_id())
                    .expect("inputId direct blends are validated as number inputs during import");
                animation.mix((value / 100.0).clamp(0.0, 1.0));
            }
        }
    }
}
