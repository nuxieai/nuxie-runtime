use crate::mechanical_port::source::{
    component::ComponentDirt,
    core::{CoreHandle, field_types::core_callback_type::CallbackData},
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_trigger_base::{
        ViewModelInstanceTriggerBase, ViewModelInstanceTriggerBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ViewModelInstanceTrigger {
    pub base: ViewModelInstanceTriggerBase,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&CoreHandle, u32)>,
}

impl ViewModelInstanceTrigger {
    /// Source property mutation split at its synchronous tools callback so
    /// observing/removing this same occurrence never crosses a Rust borrow.
    pub fn set_property_value_handle(owner: &CoreHandle, next: u32) -> bool {
        let _retained = owner.retain_arena();
        let Some(changed) = owner.with_downcast_mut::<Self, _>(|value| {
            let changed = value.base.set_property_value_value(next);
            if changed {
                crate::host_viewmodel::capture_native_change(
                    owner.clone(),
                    crate::RuntimeViewModelChangeValue::Trigger(next as u64),
                );
                value
                    .base
                    .add_dirt_from_trigger(ComponentDirt::BINDINGS, next);
            }
            changed
        }) else {
            return false;
        };
        if !changed {
            return true;
        }
        #[cfg(feature = "tools")]
        if let Some(callback) = owner
            .with_downcast::<Self, _>(|value| value.changed_callback)
            .flatten()
        {
            let deferred = owner.clone();
            if !crate::view_model_cell::defer_transaction_notification(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback(&deferred, next)
                }));
            }) {
                callback(owner, next);
            }
        }
        owner.with_downcast_mut::<Self, _>(|value| {
            value.base.on_value_changed();
        });
        owner.notify_property_changed(ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY);
        true
    }

    pub fn trigger_handle(owner: &CoreHandle) -> bool {
        let Some(next) =
            owner.with_downcast::<Self, _>(|value| value.base.property_value().wrapping_add(1))
        else {
            return false;
        };
        Self::set_property_value_handle(owner, next)
    }

    pub fn apply_value_handle(owner: &CoreHandle, value: &DataValueInteger) -> bool {
        Self::set_property_value_handle(owner, value.value())
    }
    fn set_property_value(&mut self, value: u32) {
        if self.base.set_property_value_value(value) {
            self.property_value_changed();
            crate::mechanical_port::source::core::CoreObject::core_mut(self)
                .notify_property_changed(ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY);
        }
    }
    // Borrowed generated callbacks also serve detached construction. Installed
    // tools observers require the handle mutation API, which can release the
    // owner borrow at the source callback point. Never skip such an observer.
    pub(crate) fn property_value_changed(&mut self) {
        #[cfg(feature = "tools")]
        assert!(
            self.changed_callback.is_none(),
            "trigger with tools callback requires handle mutation"
        );
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_change(
                owner,
                crate::RuntimeViewModelChangeValue::Trigger(self.base.property_value() as u64),
            );
        }
        let value = self.base.property_value();
        self.base
            .add_dirt_from_trigger(ComponentDirt::BINDINGS, value);
        self.base.on_value_changed();
    }

    pub(crate) fn fire(&mut self, _value: &CallbackData<'_>) {
        self.set_property_value(self.base.property_value().wrapping_add(1));
    }
    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&CoreHandle, u32)>) {
        self.changed_callback = callback;
    }
}

impl ViewModelInstanceTriggerBaseCallbacks for ViewModelInstanceTrigger {
    fn fire(&mut self, value: &mut CallbackData<'_>) {
        Self::fire(self, value);
    }
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
