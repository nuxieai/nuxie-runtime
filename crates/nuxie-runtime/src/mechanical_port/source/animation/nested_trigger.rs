use crate::mechanical_port::source::{
    core::field_types::core_callback_type::CallbackData,
    generated::animation::nested_trigger_base::NestedTriggerBase,
};

#[derive(Default)]
pub struct NestedTrigger {
    pub base: NestedTriggerBase,
}

impl NestedTrigger {
    pub fn fire(&mut self, _value: &CallbackData<'_>) {
        self.apply_value();
    }

    pub fn apply_value(&mut self) {
        self.base.base.fire_trigger();
    }
}
