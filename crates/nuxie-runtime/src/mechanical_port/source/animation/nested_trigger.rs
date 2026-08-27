use crate::mechanical_port::source::{
    animation::state_machine_input_instance::SMITrigger,
    core::field_types::core_callback_type::CallbackData,
    generated::animation::nested_trigger_base::NestedTriggerBase,
};

pub trait NestedTriggerInput {
    fn trigger_input(&mut self) -> Option<&mut SMITrigger>;
}

#[derive(Default)]
pub struct NestedTrigger {
    pub base: NestedTriggerBase,
}

impl NestedTrigger {
    pub fn fire(&mut self, _value: &CallbackData<'_>, input: &mut dyn NestedTriggerInput) {
        self.apply_value(input);
    }

    pub fn apply_value(&mut self, input: &mut dyn NestedTriggerInput) {
        if let Some(trigger) = input.trigger_input() {
            trigger.fire();
        }
    }
}
