use crate::mechanical_port::source::{
    bindable_artboard::RuntimeBindableArtboardHandle, component_dirt::ComponentDirt,
    core::CoreHandle, data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
};

#[derive(Default)]
pub struct ViewModelInstanceArtboard {
    pub base: ViewModelInstanceArtboardBase,
    bindable_artboard: Option<RuntimeBindableArtboardHandle>,
    bound_view_model_instance: Option<CoreHandle>,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceArtboard {
    pub fn property_value_changed(&mut self) {
        self.bindable_artboard = None;
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_asset(&mut self, value: Option<RuntimeBindableArtboardHandle>) {
        self.base.set_property_value(u32::MAX);
        self.bindable_artboard = value;
        self.base.add_dirt(ComponentDirt::BINDINGS);
    }

    pub fn asset(&self) -> Option<RuntimeBindableArtboardHandle> {
        self.bindable_artboard.clone()
    }

    pub fn set_bound_view_model_instance(&mut self, value: Option<CoreHandle>) {
        self.bound_view_model_instance = value;
    }

    pub fn bound_view_model_instance(&self) -> Option<CoreHandle> {
        self.bound_view_model_instance.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.base.set_property_value(data_value.value());
    }

    pub fn advanced(&mut self) {
        if let Some(instance) = &self.bound_view_model_instance {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.advanced();
                }
            });
        }
        self.base.advanced();
    }

    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}
