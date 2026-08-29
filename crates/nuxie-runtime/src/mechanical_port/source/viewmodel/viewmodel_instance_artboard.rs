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
    pub fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            crate::mechanical_port::source::core::CoreObject::core_mut(self)
                .notify_property_changed(
                    ViewModelInstanceArtboardBase::PROPERTY_VALUE_PROPERTY_KEY,
                );
        }
    }
    pub(crate) fn restore_host_asset(
        &mut self,
        asset: Option<RuntimeBindableArtboardHandle>,
        instance: Option<CoreHandle>,
    ) {
        self.bindable_artboard = asset;
        self.bound_view_model_instance = instance;
    }
    pub fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Artboard(self.base.property_value() as u64),
            );
        }
        self.bindable_artboard = None;
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            let value = self.base.property_value();
            if !crate::view_model_cell::defer_transaction_tools_callback(self, move |owner| {
                callback(owner, value);
            }) {
                callback(self, value);
            }
        }
        self.base.on_value_changed();
    }

    pub fn set_asset(&mut self, value: Option<RuntimeBindableArtboardHandle>) {
        self.set_property_value(u32::MAX);
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
        self.set_property_value(data_value.value());
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
