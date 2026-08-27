use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_enum_base::{
        ViewModelInstanceEnumBase, ViewModelInstanceEnumBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceEnum {
    pub base: ViewModelInstanceEnumBase,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceEnum {
    fn set_property_value(&mut self, value: u32) {
        let this = self as *mut Self;
        unsafe { (*this).base.set_property_value(value, &mut *this) };
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value_named(&mut self, name: &str) -> bool {
        let enum_property = self.base.view_model_property_enum();
        let index = enum_property.value_index_named(name);
        if index == -1 {
            return false;
        }
        self.set_property_value(index as u32);
        true
    }

    pub fn set_value_at(&mut self, index: u32) -> bool {
        let enum_property = self.base.view_model_property_enum();
        if enum_property.value_index_at(index) == -1 {
            return false;
        }
        self.set_property_value(index);
        true
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.set_property_value(data_value.value());
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceEnumBaseCallbacks for ViewModelInstanceEnum {
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
