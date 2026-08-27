use crate::mechanical_port::source::{
    data_bind::bindable_property::BindableProperty,
    generated::animation::transition_property_viewmodel_comparator_base::TransitionPropertyViewModelComparatorBase,
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub trait TransitionPropertyViewModelLayerUse {
    fn bindable_property_instance(&self, property: NonNull<BindableProperty>) -> Option<*mut ()>;
    fn use_target_source_in_layer(&self, instance: *mut (), layer: *mut ());
}
#[derive(Default)]
pub struct TransitionPropertyViewModelComparator {
    pub base: TransitionPropertyViewModelComparatorBase,
    bindable_property: Option<Box<BindableProperty>>,
}
impl TransitionPropertyViewModelComparator {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BindablePropertyImporter>(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) else { return StatusCode::MissingObject };
        self.bindable_property = importer
            .bindable_property()
            .map(|value| unsafe { Box::from_raw(value.as_ptr()) });
        self.base.base.import(stack)
    }
    pub fn value<T: Default>(
        &self,
        resolve: impl FnOnce(NonNull<BindableProperty>) -> Option<T>,
    ) -> T {
        self.bindable_property
            .as_ref()
            .and_then(|property| resolve(NonNull::from(property.as_ref())))
            .unwrap_or_default()
    }
    pub fn use_in_layer(&self, machine: &dyn TransitionPropertyViewModelLayerUse, layer: *mut ()) {
        let Some(property) = self
            .bindable_property
            .as_ref()
            .map(|value| NonNull::from(value.as_ref()))
        else {
            return;
        };
        let Some(instance) = machine.bindable_property_instance(property) else {
            return;
        };
        machine.use_target_source_in_layer(instance, layer);
    }
    pub fn bindable_property(&mut self) -> Option<&mut BindableProperty> {
        self.bindable_property.as_deref_mut()
    }
}
