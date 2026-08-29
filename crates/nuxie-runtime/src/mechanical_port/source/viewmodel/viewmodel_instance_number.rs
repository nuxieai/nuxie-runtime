use crate::mechanical_port::source::{
    component::ComponentDirt,
    data_bind::data_values::data_value_number::DataValueNumber,
    generated::viewmodel::viewmodel_instance_number_base::{
        ViewModelInstanceNumberBase, ViewModelInstanceNumberBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceNumber {
    pub base: ViewModelInstanceNumberBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, f32)>,
}

impl ViewModelInstanceNumber {
    pub fn value(&self) -> f32 {
        self.base.property_value()
    }

    pub fn set_value(&mut self, value: f32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }

    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Number(self.base.property_value()),
            );
        }
        let value = self.base.property_value();
        self.base
            .add_dirt_from_number(ComponentDirt::BINDINGS, value);
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
    pub fn apply_value(&mut self, value: &DataValueNumber) {
        self.set_value(value.value());
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, f32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceNumberBaseCallbacks for ViewModelInstanceNumber {
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
