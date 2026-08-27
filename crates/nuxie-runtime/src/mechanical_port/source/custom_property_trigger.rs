use crate::mechanical_port::source::{
    core::field_types::core_callback_type::CallbackData,
    generated::custom_property_trigger_base::{
        CustomPropertyTriggerBase, CustomPropertyTriggerBaseCallbacks,
    },
    resetting_component::ResettingComponent,
};

#[derive(Default)]
pub struct CustomPropertyTrigger {
    pub base: CustomPropertyTriggerBase,
}

impl CustomPropertyTrigger {
    pub fn fire(&mut self, _value: &CallbackData<'_>) {
        let value = self.base.property_value() + 1;
        let base = &mut self.base as *mut CustomPropertyTriggerBase;
        unsafe { &mut *base }.set_property_value(value, self);
    }
}

impl ResettingComponent for CustomPropertyTrigger {
    fn reset(&mut self) {
        let base = &mut self.base as *mut CustomPropertyTriggerBase;
        unsafe { &mut *base }.set_property_value(0, self);
    }
}

impl CustomPropertyTriggerBaseCallbacks for CustomPropertyTrigger {
    fn fire(&mut self, value: &mut CallbackData<'_>) {
        CustomPropertyTrigger::fire(self, value);
    }

    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
