use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::bindable_property_viewmodel::BindablePropertyViewModel,
    generated::animation::listener_viewmodel_change_base::ListenerViewModelChangeBase,
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
#[derive(Default)]
pub struct ListenerViewModelChange {
    pub base: ListenerViewModelChangeBase,
    bindable_property: Option<CoreHandle>,
}
impl Drop for ListenerViewModelChange {
    fn drop(&mut self) {
        if let Some(property) = self.bindable_property.take() {
            property.remove_occurrence();
        }
    }
}

impl std::ops::Deref for ListenerViewModelChange {
    type Target = ListenerViewModelChangeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerViewModelChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl ListenerViewModelChange {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BindablePropertyImporter>(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) else { return StatusCode::MissingObject };
        self.bindable_property = importer.bindable_property();
        self.base.base.import(stack)
    }
    pub fn perform(&self, machine: &mut StateMachineInstance, _invocation: &ListenerInvocation) {
        let Some(property) = self.bindable_property.as_ref() else {
            return;
        };
        let Some(instance) = machine.bindable_property_instance(property) else {
            return;
        };
        let data_bind = machine.bindable_data_bind_to_source(&instance);
        let to_target = machine.bindable_data_bind_to_target(&instance);
        if let Some(data_bind) = data_bind {
            if let Some(target) = data_bind
                .with(|bind| bind.as_data_bind().and_then(|bind| bind.target()))
                .flatten()
            {
                if target
                    .is_type_of(crate::mechanical_port::source::generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase::TYPE_KEY)
                {
                    if let Some(context) = machine.data_context() {
                        let value =
                            context.with_context(|context| context.main_view_model_instance());
                        target.with_downcast_mut::<BindablePropertyViewModel, _>(|target| {
                            target.set_view_model_instance_value(value.clone())
                        });
                        let key = crate::mechanical_port::source::viewmodel::viewmodel_instance::ViewModelInstance::pointer_key(value.as_ref());
                        crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(&target,
                            crate::mechanical_port::source::generated::data_bind::bindable_property_id_base::BindablePropertyIdBase::PROPERTY_VALUE_PROPERTY_KEY as i32, key);
                    }
                }
            }
            crate::mechanical_port::source::data_bind::data_bind::DataBind::update_source_binding_handle(
                &data_bind, true,
            );
        }
        if let Some(to_target) = to_target {
            to_target.with_mut(|bind| {
                bind.as_data_bind_mut()
                    .expect("an owned target binding remains DataBind")
                    .add_dirt(ComponentDirt::BINDINGS.0 as u32, true)
            });
        }
    }
}
