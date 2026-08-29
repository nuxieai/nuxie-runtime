use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle,
    core::CoreHandle,
    generated::animation::transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
pub trait TransitionPropertyViewModelLayerUse {
    fn use_bindable_property_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    );
}

impl TransitionPropertyViewModelLayerUse
    for crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance
{
    fn use_bindable_property_in_layer(
        &self,
        property: &CoreHandle,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        self.use_bindable_property_in_layer(property, layer);
    }
}
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
    pub fn use_in_layer(
        &self,
        machine: &dyn TransitionPropertyViewModelLayerUse,
        layer: Option<RuntimeStateMachineLayerInstanceWeakHandle>,
    ) {
        let Some(property) = self.bindable_property.as_ref() else {
            return;
        };
        machine.use_bindable_property_in_layer(property, layer);
    }
    pub fn bindable_property(&self) -> Option<CoreHandle> {
        self.bindable_property.clone()
    }
}
