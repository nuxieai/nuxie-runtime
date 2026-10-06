use crate::mechanical_port::source::{
    core::field_types::core_callback_type::CallbackData,
    generated::custom_property_trigger_base::{
        CustomPropertyTriggerBase, CustomPropertyTriggerBaseCallbacks,
    },
};

#[derive(Default)]
pub struct CustomPropertyTrigger {
    pub base: CustomPropertyTriggerBase,
    change_sequence: u64,
}

impl CustomPropertyTrigger {
    pub fn change_sequence(&self) -> u64 {
        self.change_sequence
    }

    pub fn property_value_changed(&mut self) {
        self.change_sequence = crate::source::viewmodel::viewmodel_instance_value::ViewModelInstanceValue::next_change_sequence();
    }
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

impl CustomPropertyTriggerBaseCallbacks for CustomPropertyTrigger {
    fn property_value_changed(&mut self) {
        CustomPropertyTrigger::property_value_changed(self);
    }
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
