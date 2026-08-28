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
        self.set_property_value(self.base.property_value().wrapping_add(1));
    }

    fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            CustomPropertyTriggerBaseCallbacks::property_value_changed(self);
            CustomPropertyTriggerBaseCallbacks::notify_property_changed(
                self,
                CustomPropertyTriggerBase::PROPERTY_VALUE_PROPERTY_KEY,
            );
        }
    }
}

impl ResettingComponent for CustomPropertyTrigger {
    fn reset(&mut self) {
        self.set_property_value(0);
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

impl std::ops::Deref for CustomPropertyTrigger {
    type Target = CustomPropertyTriggerBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyTrigger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
