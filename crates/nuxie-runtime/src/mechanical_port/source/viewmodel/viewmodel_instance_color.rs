use crate::mechanical_port::source::{
    component::ComponentDirt, data_bind::data_values::data_value_color::DataValueColor,
    generated::viewmodel::viewmodel_instance_color_base::ViewModelInstanceColorBase,
};

pub struct ViewModelInstanceColor {
    pub base: ViewModelInstanceColorBase,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, i32)>,
}

impl ViewModelInstanceColor {
    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueColor) {
        self.base.set_property_value(value.value());
    }
    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, i32)>) {
        self.changed_callback = callback;
    }
}
