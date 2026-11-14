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
    /// What the host that set the current value keeps alive with it (a
    /// script's charge for the string); dropped on the next change of value,
    /// whoever makes it, and with the property.
    host_owner: Option<crate::host_viewmodel::HostOwnerToken>,
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

    /// Sets the value and keeps `owner` for as long as the property holds it.
    pub(crate) fn set_value_owned(
        &mut self,
        value: impl Into<String>,
        owner: crate::host_viewmodel::HostOwnerToken,
    ) {
        self.set_value(value);
        self.host_owner = Some(owner);
    }

    pub(crate) fn host_owner(&self) -> Option<&crate::host_viewmodel::HostOwnerToken> {
        self.host_owner.as_ref()
    }

    /// Keeps `owner` for the value the property holds now: one a rollback
    /// restored, or one copied with its share of a charge.
    pub(crate) fn keep_host_owner(&mut self, owner: Option<crate::host_viewmodel::HostOwnerToken>) {
        self.host_owner = owner;
    }

    pub fn property_value_changed(&mut self) {
        // The value the owner paid for is gone, whichever path replaced it.
        self.host_owner = None;
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::String(std::sync::Arc::from(
                    self.base.property_value().as_bytes(),
                )),
            );
        }
        let snapshot = super::viewmodel_value_dependent::TextStyleSourceValue::String(
            self.base.property_value().to_owned(),
        );
        self.base
            .add_dirt_from_text_style_source(ComponentDirt::BINDINGS, snapshot);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            let value = self.base.property_value().to_owned();
            let captured = value.clone();
            if !crate::view_model_cell::defer_transaction_tools_callback(self, move |owner| {
                callback(owner, &captured);
            }) {
                callback(self, &value);
            }
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
