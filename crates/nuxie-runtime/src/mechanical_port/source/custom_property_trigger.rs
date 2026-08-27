use crate::mechanical_port::source::{
    core::field_types::core_callback_type::CallbackData,
    generated::custom_property_trigger_base::CustomPropertyTriggerBase,
    resetting_component::ResettingComponent,
};

pub struct CustomPropertyTrigger {
    pub base: CustomPropertyTriggerBase,
}

impl CustomPropertyTrigger {
    pub fn fire(&mut self, _value: &CallbackData<'_>) {
        self.base.set_property_value(self.base.property_value() + 1);
    }
}

impl ResettingComponent for CustomPropertyTrigger {
    fn reset(&mut self) {
        self.base.set_property_value(0);
    }
}
