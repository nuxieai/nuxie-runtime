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
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, &str)>,
}

impl ViewModelInstanceString {
    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            let value = self.base.property_value().to_owned();
            callback(self, &value);
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueString) {
        let this = self as *mut Self;
        unsafe {
            (*this)
                .base
                .set_property_value(value.value().to_owned(), &mut *this)
        };
    }
    #[cfg(feature = "rive_tools")]
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
