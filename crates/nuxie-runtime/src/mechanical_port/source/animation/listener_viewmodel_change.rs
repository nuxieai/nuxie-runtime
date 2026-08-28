use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    core::CoreHandle,
    generated::animation::listener_viewmodel_change_base::ListenerViewModelChangeBase,
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
pub trait ListenerViewModelChangeStateMachine {
    fn bindable_property_instance(&mut self, property: &CoreHandle) -> Option<CoreHandle>;
    fn source_bind_targets_viewmodel(&self, instance: &CoreHandle) -> bool;
    fn main_view_model_instance(&self) -> Option<CoreHandle>;
    fn set_source_target_view_model(&mut self, instance: &CoreHandle, value: CoreHandle);
    fn update_source_binding(&mut self, instance: &CoreHandle, force: bool);
    fn dirty_target_binding(&mut self, instance: &CoreHandle, recurse: bool);
    fn has_source_binding(&self, instance: &CoreHandle) -> bool;
    fn has_target_binding(&self, instance: &CoreHandle) -> bool;
}
#[derive(Default)]
pub struct ListenerViewModelChange {
    pub base: ListenerViewModelChangeBase,
    bindable_property: Option<CoreHandle>,
}
impl ListenerViewModelChange {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BindablePropertyImporter>(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) else { return StatusCode::MissingObject };
        self.bindable_property = importer.bindable_property();
        self.base.base.import(stack)
    }
    pub fn perform(
        &self,
        machine: &mut dyn ListenerViewModelChangeStateMachine,
        _invocation: &ListenerInvocation,
    ) {
        let Some(property) = self.bindable_property.as_ref() else {
            return;
        };
        let Some(instance) = machine.bindable_property_instance(property) else {
            return;
        };
        if machine.has_source_binding(&instance) {
            if machine.source_bind_targets_viewmodel(&instance) {
                if let Some(value) = machine.main_view_model_instance() {
                    machine.set_source_target_view_model(&instance, value);
                }
            }
            machine.update_source_binding(&instance, true);
        }
        if machine.has_target_binding(&instance) {
            machine.dirty_target_binding(&instance, true);
        }
    }
}
