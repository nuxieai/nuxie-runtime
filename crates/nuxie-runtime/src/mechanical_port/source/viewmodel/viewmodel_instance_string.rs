use crate::mechanical_port::source::{
    component::ComponentDirt,
    data_bind::data_values::data_value_string::DataValueString,
    generated::viewmodel::viewmodel_instance_string_base::{
        ViewModelInstanceStringBase, ViewModelInstanceStringBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceString {
    pub base: ViewModelInstanceStringBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, &str)>,
}

impl ViewModelInstanceString {
    pub fn value(&self) -> String {
        self.base.property_value().to_owned()
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        if self.base.set_property_value_value(value.into()) {
            self.property_value_changed();
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ViewModelInstanceStringBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            let value = self.base.property_value().to_owned();
            callback(self, &value);
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueString) {
        self.set_value(value.value());
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, &str)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceStringBaseCallbacks for ViewModelInstanceString {
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
