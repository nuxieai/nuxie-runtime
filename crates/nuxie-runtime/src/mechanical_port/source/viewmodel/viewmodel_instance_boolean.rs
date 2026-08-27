use crate::mechanical_port::source::{
    component::ComponentDirt,
    data_bind::data_values::data_value_boolean::DataValueBoolean,
    generated::viewmodel::viewmodel_instance_boolean_base::{
        ViewModelInstanceBooleanBase, ViewModelInstanceBooleanBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceBoolean {
    pub base: ViewModelInstanceBooleanBase,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, bool)>,
}

impl ViewModelInstanceBoolean {
    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueBoolean) {
        let this = self as *mut Self;
        unsafe { (*this).base.set_property_value(value.value(), &mut *this) };
    }
    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, bool)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceBooleanBaseCallbacks for ViewModelInstanceBoolean {
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
