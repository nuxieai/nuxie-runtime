use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
    },
    core::CoreHandle,
    generated::{
        animation::transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
        data_bind::bindable_property_trigger_base::BindablePropertyTriggerBase,
    },
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
#[derive(Default)]
pub struct TransitionPropertyViewModelComparator {
    pub base: TransitionPropertyViewModelComparatorBase,
    bindable_property: Option<CoreHandle>,
}
impl TransitionPropertyViewModelComparator {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BindablePropertyImporter>(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) else { return StatusCode::MissingObject };
        self.bindable_property = importer.bindable_property();
        self.base.base.import(stack)
    }
    pub fn value<T: Default>(&self, resolve: impl FnOnce(&CoreHandle) -> Option<T>) -> T {
        self.bindable_property
            .as_ref()
            .and_then(resolve)
            .unwrap_or_default()
    }
    // The view model value bound to this comparator's bindable, if any.
    fn source(&self, machine: &StateMachineInstance) -> Option<CoreHandle> {
        Self::source_for_property(self.bindable_property.as_ref()?, machine)
    }
    fn source_for_property(
        property: &CoreHandle,
        machine: &StateMachineInstance,
    ) -> Option<CoreHandle> {
        let property = machine.bindable_property_instance(property)?;
        machine
            .bindable_data_bind_to_target(&property)?
            .with(|bind| bind.as_data_bind().and_then(|bind| bind.source()))
            .flatten()
    }
    pub fn use_in_layer(
        &self,
        machine: &StateMachineInstance,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        if let Some(source) = self.source(machine) {
            machine.use_view_model_value(&source, layer);
        }
    }
    // A right-hand trigger is used only when this layer could read its fire.
    // Otherwise marking it would hide a later fire in the same frame.
    pub fn use_trigger_fire_in_layer(
        &self,
        machine: &StateMachineInstance,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        Self::use_trigger_property_fire_in_layer(self.bindable_property.as_ref(), machine, layer);
    }
    // Capability-based caller preserves upstream isTypeOf semantics for
    // projected/derived comparator owners without exact Rust Any downcasts.
    pub(super) fn use_trigger_property_fire_in_layer(
        property: Option<&CoreHandle>,
        machine: &StateMachineInstance,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        let Some(property) =
            property.filter(|property| property.is_type_of(BindablePropertyTriggerBase::TYPE_KEY))
        else {
            return;
        };
        let Some(source) = Self::source_for_property(property, machine) else {
            return;
        };
        if machine.view_model_value_changed(&source, layer.as_ref()) {
            machine.use_view_model_value(&source, layer);
        }
    }
    pub fn bindable_property(&self) -> Option<CoreHandle> {
        self.bindable_property.clone()
    }
}
