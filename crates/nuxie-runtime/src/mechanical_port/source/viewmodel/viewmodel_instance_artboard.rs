use crate::mechanical_port::source::{
    bindable_artboard::BindableArtboard, component_dirt::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
    refcnt::RiveRc,
};

use super::viewmodel_instance::ViewModelInstance;

pub struct ViewModelInstanceArtboard {
    pub base: ViewModelInstanceArtboardBase,
    bindable_artboard: Option<RiveRc<BindableArtboard>>,
    bound_view_model_instance: Option<RiveRc<ViewModelInstance>>,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceArtboard {
    pub fn property_value_changed(&mut self) {
        self.bindable_artboard = None;
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_asset(&mut self, value: Option<RiveRc<BindableArtboard>>) {
        self.base.set_property_value(u32::MAX);
        self.bindable_artboard = value;
        self.base.add_dirt(ComponentDirt::BINDINGS);
    }

    pub fn asset(&self) -> Option<RiveRc<BindableArtboard>> {
        self.bindable_artboard.clone()
    }

    pub fn set_bound_view_model_instance(&mut self, value: Option<RiveRc<ViewModelInstance>>) {
        self.bound_view_model_instance = value;
    }

    pub fn bound_view_model_instance(&self) -> Option<RiveRc<ViewModelInstance>> {
        self.bound_view_model_instance.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        self.base.set_property_value(data_value.value());
    }

    pub fn advanced(&mut self) {
        if let Some(instance) = &mut self.bound_view_model_instance {
            instance.advanced();
        }
        self.base.advanced();
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}
