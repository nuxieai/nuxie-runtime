use crate::mechanical_port::source::{
    component::ComponentDirt, data_bind::data_values::data_value_number::DataValueNumber,
    generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
};

pub struct ViewModelInstanceNumber {
    pub base: ViewModelInstanceNumberBase,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, f32)>,
}

impl ViewModelInstanceNumber {
    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueNumber) {
        self.base.set_property_value(value.value());
    }
    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, f32)>) {
        self.changed_callback = callback;
    }
}
