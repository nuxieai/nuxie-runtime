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
    /// Registered values release the source before its synchronous dependents run.
    pub fn set_value_handle(
        owner: &crate::mechanical_port::source::core::CoreHandle,
        value: f32,
    ) -> bool {
        let Some(changed) = owner.with_downcast_mut::<Self, _>(|number| {
            let changed = number.base.set_property_value_value(value);
            if changed {
                crate::host_viewmodel::capture_native_change(
                    owner.clone(),
                    crate::RuntimeViewModelChangeValue::Number(value),
                );
            }
            changed
        }) else {
            return false;
        };
        if !changed {
            return true;
        }
        super::viewmodel_instance_value::ViewModelInstanceValue::add_dirt_handle(
            owner,
            ComponentDirt::BINDINGS,
        );
        // Both the callback and its argument are read AFTER dependency callbacks.
        // Its explicit &mut Self API remains a borrowed callback boundary.
        #[cfg(feature = "tools")]
        owner.with_downcast_mut::<Self, _>(|number| {
            if let Some(callback) = number.changed_callback {
                let value = number.value();
                if !crate::view_model_cell::defer_transaction_tools_callback(
                    number,
                    move |number| callback(number, value),
                ) {
                    callback(number, value);
                }
            }
        });
        super::viewmodel_instance_value::ViewModelInstanceValue::on_value_changed_handle(owner);
        if let Some(observers) = owner.property_observers() {
            observers.notify(ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
        true
    }

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
