use crate::mechanical_port::source::{
    component::ComponentDirt,
    data_bind::data_values::data_value_color::DataValueColor,
    generated::viewmodel::viewmodel_instance_color_base::{
        ViewModelInstanceColorBase, ViewModelInstanceColorBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceColor {
    pub base: ViewModelInstanceColorBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, i32)>,
}

impl ViewModelInstanceColor {
    pub fn value(&self) -> i32 {
        self.base.property_value()
    }

    pub fn set_value(&mut self, value: i32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ViewModelInstanceColorBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }

    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Color(self.base.property_value() as u32),
            );
        }
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }
    pub fn apply_value(&mut self, value: &DataValueColor) {
        self.set_value(value.value());
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, i32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceColorBaseCallbacks for ViewModelInstanceColor {
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
