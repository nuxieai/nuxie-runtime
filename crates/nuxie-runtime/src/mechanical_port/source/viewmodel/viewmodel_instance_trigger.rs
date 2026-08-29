use crate::mechanical_port::source::{
    component::ComponentDirt,
    core::field_types::core_callback_type::CallbackData,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_trigger_base::{
        ViewModelInstanceTriggerBase, ViewModelInstanceTriggerBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceTrigger {
    pub base: ViewModelInstanceTriggerBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceTrigger {
    fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            crate::mechanical_port::source::core::CoreObject::core_mut(self)
                .notify_property_changed(ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }
    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Trigger(self.base.property_value() as u64),
            );
        }
        let value = self.base.property_value();
        self.base
            .add_dirt_from_trigger(ComponentDirt::BINDINGS, value);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            let value = self.base.property_value();
            if !crate::view_model_cell::defer_transaction_tools_callback(self, move |owner| {
                callback(owner, value);
            }) {
                callback(self, value);
            }
        }
        self.base.on_value_changed();
    }
    pub fn advanced(&mut self) {
        let suppressed = self.base.suppress_delegation();
        self.set_property_value(0);
        self.base.advanced();
        if suppressed {
            self.base.restore_delegation();
        }
    }
    pub fn fire(&mut self, _value: &CallbackData<'_>) {
        self.set_property_value(self.base.property_value().wrapping_add(1));
    }
    pub fn trigger(&mut self) {
        self.set_property_value(self.base.property_value().wrapping_add(1));
    }
    pub fn apply_value(&mut self, value: &DataValueInteger) {
        self.set_property_value(value.value());
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceTriggerBaseCallbacks for ViewModelInstanceTrigger {
    fn fire(&mut self, value: &mut CallbackData<'_>) {
        Self::fire(self, value);
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

    fn property_value_changed(&mut self) {
        Self::property_value_changed(self);
    }
}
