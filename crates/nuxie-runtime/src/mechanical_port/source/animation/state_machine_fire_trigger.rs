use crate::mechanical_port::source::{
    animation::state_machine_instance::StateMachineInstance,
    data_bind_path_referencer::DataBindPathReferencer,
    generated::animation::state_machine_fire_trigger_base::StateMachineFireTriggerBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
    viewmodel::viewmodel_instance_trigger::ViewModelInstanceTrigger,
};

#[derive(Default)]
pub struct StateMachineFireTrigger {
    pub base: StateMachineFireTriggerBase,
    pub data_bind_path_referencer: DataBindPathReferencer,
}

impl StateMachineFireTrigger {
    pub fn perform(&self, state_machine_instance: &mut StateMachineInstance) {
        let Some(trigger) = self
            .data_bind_path_referencer
            .with_data_bind_path_mut(|path| {
                state_machine_instance.data_context().and_then(|context| {
                    context.with_context(|context| context.get_property_from_path(path))
                })
            })
            .flatten()
        else {
            return;
        };
        let _ =
            trigger.with_downcast_mut::<ViewModelInstanceTrigger, _>(|trigger| trigger.trigger());
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        self.base.base.import(import_stack)
    }

    pub fn decode_view_model_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_view_model_path_ids(&mut self, object: &Self) {
        self.data_bind_path_referencer
            .copy_data_bind_path(&object.data_bind_path_referencer);
    }
}

impl std::ops::Deref for StateMachineFireTrigger {
    type Target = StateMachineFireTriggerBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineFireTrigger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
