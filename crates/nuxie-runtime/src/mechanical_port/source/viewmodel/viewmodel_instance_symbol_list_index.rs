use crate::mechanical_port::source::{
    component::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_symbol_list_index_base::{
        ViewModelInstanceSymbolListIndexBase, ViewModelInstanceSymbolListIndexBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceSymbolListIndex {
    pub base: ViewModelInstanceSymbolListIndexBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceSymbolListIndex {
    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::ListIndex(self.base.property_value() as u64),
            );
        }
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueInteger) {
        if self.base.set_property_value_value(value.value()) {
            self.property_value_changed();
            self.base.base.base.base.base.base.notify_property_changed(
                ViewModelInstanceSymbolListIndexBase::PROPERTY_VALUE_PROPERTY_KEY,
            );
        }
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceSymbolListIndexBaseCallbacks for ViewModelInstanceSymbolListIndex {
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
